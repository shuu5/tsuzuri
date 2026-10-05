//! 台帳の読み（bd の読み取りの口を子 process で 1 本撃つだけ・書かない）。
//! 撃つ形は `bd --readonly list --all --limit 0 --json`（cwd は repo の置き場・標準入力は空・標準エラーは捨てる）。
//! 起動できない・rc が 0 でない・JSON として読めない・5 秒を超えて返さない、のどれでも
//! 一覧は 0 件でなく「まだ分からない」（Reading::Unknown）にする。
//! 変化の印は store（.beads の metadata.json が名指す db の .dolt/noms）の manifest の字と manifest が名指す file の長さで、
//! store が無ければ issues.jsonl と interactions.jsonl の更新時刻と長さ（jsonl の中身は読まない）。
//! 同じ `Source` とその clone の読みは、走っている 1 本の子 process を分け合う（`coalesce`）。
//! 読めた字（`parse_bd` か中核の台帳の読みが Known の字）は最後に読めた字として持ち、次の読みが落ちたときだけ
//! 上限（既定 `READ_HOLD`・60 秒）まで `got` と `text` が返す。変化の見張りの `read` は持ち回さない。
//! `watched` の Source の `got` と `text` は、最後に始めた合流の読みの前に取った印（`read_mark`）が今の印と同じなら
//! bd を撃たず、最後に終えた読み（変化の見張りの読み）の結果を返し、読みが走っていればその終わりを待って同じ結果を返す。
//! 印が違えば見張りを待たず自分で読み、`form` が在れば台帳の形の行の撃ちへ渡す。
//! `watched` の Source は、今の印が最後に読めた読みの前に取った印（`good_mark`）と同じ間は bd を撃たず、最後に読めた字を
//! 新しい字（stale の無し）として返し、落ちた読みの結果は持たない（次の `got` が読み直す）。変化の見張りの `read` の上限は
//! `WATCH_TIMEOUT`（30 秒）で、口の要求の読みは `BD_TIMEOUT` のまま（判断の記録 ADR-30 決定 (10)）。
//! `with_form` の Source の `read` は、読んだ台帳の字（読めなければ None）で器の doctor の台帳の形の行の撃ち
//! （`Form::kick`）を起こす（待たない・撃ちはその字と台帳の形の行を組で持つ）。

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BdLine, BeadId, LedgerItem, LedgerList, READ_HOLD_S};
use tsuzuri_contract::wire;

use super::coalesce::{Coalesce, GRACE};
use super::events::stamp;
use super::form::Form;
use super::proc::run;

/// 子 process を撃つ部品と時刻の読みは `proc` と `clock` に在り、今までの名のまま再公開する（行 hb-proc）。
pub use super::clock::epoch_secs;
pub use super::proc::{KILL, capture, stop_with};

/// 既定の program の名（引数 --bd で替える）。
pub const BD: &str = "bd";

/// bd に渡す引数の列（読み取りだけ・closed を含む全部・件数の上限なし・JSON）。
pub const BD_ARGS: [&str; 6] = ["--readonly", "list", "--all", "--limit", "0", "--json"];

/// bd が返すまでの上限（要件 NFR2 の上限・規則の行 R-21 の値）。越えれば止めて Unknown。
pub const BD_TIMEOUT: Duration = Duration::from_secs(5);

/// 走っている読みに合流した呼び出しが待つ上限（`BD_TIMEOUT` に 1 秒を足す・便 e-coalesce）。
pub const BD_WAIT: Duration = BD_TIMEOUT.saturating_add(GRACE);

/// 変化の見張りの読み（`Source::read`）の bd が返すまでの上限（判断の記録 ADR-30 決定 (10)・行 e-hold-mark）。
pub const WATCH_TIMEOUT: Duration = Duration::from_secs(30);

/// 変化の見張りの読みが走っている読みに合流して待つ上限（`WATCH_TIMEOUT` に 1 秒を足す）。
pub const WATCH_WAIT: Duration = WATCH_TIMEOUT.saturating_add(GRACE);

/// 読みが落ちても最後に読めた字を返す上限（契約の `READ_HOLD_S` 秒・行 e-hold）。
pub const READ_HOLD: Duration = Duration::from_secs(READ_HOLD_S);

/// repo の .beads の下の台帳の store の dir（その下が db ごとの dir・行 e-marks）。
pub const STORE_DIR: &str = "embeddeddolt";

/// db の dir の下の store の file の置き場。
pub const NOMS: &str = ".dolt/noms";

/// store の manifest（字を印にする）。
pub const MANIFEST: &str = "manifest";

/// dolt の chunk journal の定まった名（manifest が名指す file の 1 つ）。
pub const JOURNAL: &str = "vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv";

/// repo の .beads の下の store の置き方の JSON（欄 dolt_mode と dolt_database を読む・行 e-mark-meta）。
pub const METADATA: &str = "metadata.json";

/// table の file の名に足す字（manifest が名指す名の file が無ければ、この字を足した名の file を見る）。
pub const TABLE_SUFFIX: &str = ".darc";

/// 台帳の変化の印（行 e-marks・行 e-mark-meta）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    /// manifest の字と manifest が名指す file の名と長さ。
    /// bd の読みは manifest を同じ字で置き替え journal と journal.idx の更新時刻を動かすので、
    /// 更新時刻と journal.idx は見ない。
    Store {
        manifest: Vec<u8>,
        sizes: Vec<(String, Option<u64>)>,
    },
    /// store の無いときの `Source::marks` の 2 file の更新時刻と長さ。
    Files(Vec<Option<(SystemTime, u64)>>),
}

/// 台帳の読みの出所（repo の置き場と bd の program）。
/// clone は読みの合流の場と最後に読めた字と最後に終えた読みの結果を分け合う（比べるのは repo と bd だけ）。
/// `got` と `text` は読みが落ちたとき、最後に読めた時から上限（`hold`）より短い間だけ最後に読めた字を返す。
/// `read`（変化の見張りの読み）は持ち回さず、落ちれば Unknown。`text_alone` と `text_within` は合流も持ち回しもしない。
#[derive(Debug, Clone)]
pub struct Source {
    pub repo: PathBuf,
    pub bd: OsString,
    shared: Coalesce<Arc<str>>,
    /// 最後に読めた字と読んだ時刻（一度も読めていなければ字は None で、時刻は `new` を呼んだ時刻）。
    last: Arc<Mutex<(Option<Arc<str>>, Instant)>>,
    /// 持ち回しの上限。
    hold: Duration,
    /// 最後に終えた合流の読みの結果（読めた字・一度も終えていないか最後の読みが落ちていれば None・行 e-snap・行 e-hold-mark）。
    latest: Arc<Mutex<Option<Option<Arc<str>>>>>,
    /// 最後に始めた合流の読みの前に取った印（一度も始めていなければ None・行 e-ledger-lazy）。
    read_mark: Arc<Mutex<Option<Mark>>>,
    /// 最後に読めた合流の読みの前に取った印（一度も読めていなければ None・行 e-hold-mark）。
    good_mark: Arc<Mutex<Option<Mark>>>,
    /// 真なら `got` と `text` は、印が `read_mark` と同じ間は bd を撃たず、最後に終えた読み（変化の見張りの読み）の結果を返す。
    watched: bool,
    /// 見張りの読みの後に撃つ器の doctor の台帳の形の行（行 c-pipe-misfit）。
    form: Option<Form>,
}

/// 持ち回しを含む読みの結果（行 e-hold）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Got {
    /// 読めた字か、落ちたときは上限の内の最後に読めた字（越えたか一度も読めていなければ None）。
    /// 合流の読みの結果と最後に読めた字と最後に終えた読みの結果と同じ確保を分け合う。
    pub text: Option<Arc<str>>,
    /// 読めれば None、落ちれば最後に読めた時刻（一度も読めていなければ `Source::new` の時刻）。
    pub stale: Option<Instant>,
}

impl PartialEq for Source {
    fn eq(&self, other: &Source) -> bool {
        (&self.repo, &self.bd) == (&other.repo, &other.bd)
    }
}

impl Eq for Source {}

impl Source {
    pub fn new(repo: impl Into<PathBuf>, bd: impl Into<OsString>) -> Source {
        Source {
            repo: repo.into(),
            bd: bd.into(),
            shared: Coalesce::new(),
            last: Arc::new(Mutex::new((None, Instant::now()))),
            hold: READ_HOLD,
            latest: Arc::new(Mutex::new(None)),
            read_mark: Arc::new(Mutex::new(None)),
            good_mark: Arc::new(Mutex::new(None)),
            watched: false,
            form: None,
        }
    }

    /// 見張りの読み（`read`）の後に器の doctor の台帳の形の行を撃つ Source（行 c-pipe-misfit）。
    pub fn with_form(self, form: Option<Form>) -> Source {
        Source { form, ..self }
    }

    /// 見張りの読みの後に撃つ器の doctor の台帳の形の行。
    pub fn form(&self) -> Option<&Form> {
        self.form.as_ref()
    }

    /// 変化の見張りが読む Source（`got` と `text` は bd を撃たず、見張りの最後の読みの結果を返す・行 e-snap）。
    pub fn watched(self) -> Source {
        Source {
            watched: true,
            ..self
        }
    }

    /// `watched` の Source か。
    pub fn is_watched(&self) -> bool {
        self.watched
    }

    /// 持ち回しの上限を替えた Source（歯が短い上限で試す）。
    pub fn with_hold(self, hold: Duration) -> Source {
        Source { hold, ..self }
    }

    /// 持ち回しの上限。
    pub fn hold(&self) -> Duration {
        self.hold
    }

    /// 変化の印の 2 つの file（席の書きで動く issues.jsonl と、器の close を含む状態の変更で動く interactions.jsonl）。
    pub fn marks(&self) -> Vec<PathBuf> {
        let beads = self.repo.join(".beads");
        vec![beads.join("issues.jsonl"), beads.join("interactions.jsonl")]
    }

    /// 台帳の store の NOMS の path（.beads の METADATA の欄 dolt_mode が embedded で、欄 dolt_database が
    /// 英数字と _ と - だけの名で、その db の NOMS に MANIFEST の file が在るときだけ）。
    /// db の名は呼ぶたびに METADATA を読んで決める（METADATA が名指さない db の dir は見ない）。
    pub fn store(&self) -> Option<PathBuf> {
        let beads = self.repo.join(".beads");
        let meta = std::fs::read_to_string(beads.join(METADATA)).ok()?;
        if json_str(&meta, "dolt_mode")? != "embedded" {
            return None;
        }
        let db = json_str(&meta, "dolt_database").filter(|db| plain_name(db))?;
        let noms = beads.join(STORE_DIR).join(db).join(NOMS);
        noms.join(MANIFEST).is_file().then_some(noms)
    }

    /// 変化の印（store の MANIFEST が読めれば manifest の字と manifest が名指す file の長さ、
    /// 無ければ `marks` の 2 file の更新時刻と長さ）。METADATA と manifest を読み、名指す file を stat するだけで、
    /// 錠の file（LOCK）は開かない。
    pub fn mark(&self) -> Mark {
        let read = self
            .store()
            .and_then(|noms| Some((std::fs::read(noms.join(MANIFEST)).ok()?, noms)));
        let Some((manifest, noms)) = read else {
            return Mark::Files(self.marks().iter().map(|m| stamp(m)).collect());
        };
        let sizes = named(&String::from_utf8_lossy(&manifest))
            .into_iter()
            .map(|name| {
                let len = std::fs::metadata(noms.join(name))
                    .or_else(|_| std::fs::metadata(noms.join(format!("{name}{TABLE_SUFFIX}"))))
                    .ok()
                    .map(|m| m.len());
                (name.to_string(), len)
            })
            .collect();
        Mark::Store { manifest, sizes }
    }

    /// bd を撃って台帳を読む（変化の見張りの読み・持ち回さない・落ちれば Unknown）。上限は、最後に読めた字が在れば
    /// `WATCH_TIMEOUT`、一度も読めていなければ `BD_TIMEOUT`（server の起動の読みを長く止めない）。
    /// 走っている読みに合流して落ちた結果を受けたときは、自分で読み直す（行 e-hold-mark）。
    /// 読めた字は最後に読めた字に置く。`form` が在れば読んだ字の複製で撃ちを起こす（待たない）。
    pub fn read(&self) -> Reading<Vec<LedgerItem>> {
        let (timeout, wait) = if lock(&self.last).0.is_some() {
            (WATCH_TIMEOUT, WATCH_WAIT)
        } else {
            (BD_TIMEOUT, BD_WAIT)
        };
        let mut own = false;
        let mut text = self.shared.share(wait, || {
            own = true;
            self.shoot(timeout)
        });
        if text.is_none() && !own {
            text = self.shared.share(wait, || self.shoot(timeout));
        }
        if let Some(form) = &self.form {
            form.kick(text.as_deref().map(str::to_string));
        }
        text.map_or(Reading::Unknown, |text| parse_bd(&text))
    }

    /// bd を撃ち、読めた字を返す（導出グラフと指標の入力・便 e-read）。落ちれば上限の内の最後に読めた字（`got`）。
    pub fn text(&self) -> Option<String> {
        self.got().text.as_deref().map(str::to_string)
    }

    /// 合流の読みを撃ち、読めれば読めた字と stale の None、落ちれば最後に読めた時刻と、
    /// その時刻から上限より短い間だけ最後に読めた字を返す（行 e-hold）。
    /// `watched` の Source は、今の印が最後に始めた読みの前の印と違えば見張りを待たず自分で読み（`form` が在れば
    /// 読んだ字で撃ちを起こす）、同じなら bd を撃たず、走っている読みが在ればその終わりを `BD_WAIT` まで待った結果、
    /// 無ければ最後に終えた読みの結果を使う（一度も終えていなければ読む・行 e-snap・行 e-ledger-lazy）。
    /// `watched` の Source は、今の印が `good_mark` と同じなら bd を撃たず最後に読めた字を stale の無しで返す（行 e-hold-mark）。
    pub fn got(&self) -> Got {
        if let Some(text) = self.unmoved() {
            return Got {
                text: Some(text),
                stale: None,
            };
        }
        let text = if self.watched {
            if self.behind() {
                let text = self.fresh();
                if let Some(form) = &self.form {
                    form.kick(text.as_deref().map(str::to_string));
                }
                text
            } else {
                self.shared
                    .join(BD_WAIT)
                    .or_else(|| lock(&self.latest).clone())
                    .unwrap_or_else(|| self.fresh())
            }
        } else {
            self.fresh()
        };
        if let Some(text) = text {
            return Got {
                text: Some(text),
                stale: None,
            };
        }
        let last = lock(&self.last);
        let (text, at) = &*last;
        Got {
            text: text.clone().filter(|_| at.elapsed() < self.hold),
            stale: Some(*at),
        }
    }

    /// `watched` の Source で、今の印が最後に読めた読みの前に取った印と同じなら最後に読めた字（行 e-hold-mark）。
    fn unmoved(&self) -> Option<Arc<str>> {
        if !self.watched {
            return None;
        }
        let good = lock(&self.good_mark).clone()?;
        (good == self.mark())
            .then(|| lock(&self.last).0.clone())
            .flatten()
    }

    /// 今の印が最後に始めた合流の読みの前に取った印と違うか（一度も始めていなければ偽）。
    fn behind(&self) -> bool {
        let was = lock(&self.read_mark).clone();
        was.is_some_and(|was| was != self.mark())
    }

    /// 合流の読み（走っている読みが在れば新しく撃たず、その終わりを `BD_WAIT` まで待って同じ結果・便 e-coalesce）。
    /// 読みは `shoot`（上限 `BD_TIMEOUT`）。
    fn fresh(&self) -> Option<Arc<str>> {
        self.shared.share(BD_WAIT, || self.shoot(BD_TIMEOUT))
    }

    /// 合流の読みの中身（上限 `timeout` の bd の 1 本）。起動できない・rc が 0 でない・UTF-8 でない・上限を越える・
    /// `parse_bd` も中核の台帳の読みも Unknown の字、のどれでも None。始める前に取った印を `read_mark` に置き、
    /// 読めれば字と時刻を最後に読めた字に、その印を `good_mark` に、字を最後に終えた読みの結果（`latest`）に置き、
    /// 落ちれば `latest` を空にする（落ちた結果を持たない・行 e-hold-mark）。
    fn shoot(&self, timeout: Duration) -> Option<Arc<str>> {
        let mark = self.mark();
        *lock(&self.read_mark) = Some(mark.clone());
        let text = self
            .text_within(timeout)
            .ok()
            .filter(|t| readable(t))
            .map(Arc::<str>::from);
        if let Some(text) = &text {
            *lock(&self.last) = (Some(text.clone()), Instant::now());
            *lock(&self.good_mark) = Some(mark);
        }
        *lock(&self.latest) = text.clone().map(Some);
        text
    }

    /// 走っている読みを分け合わず、新しい子 process で bd を撃つ（停止の hook・問いの門・見張りの読み・便 e-ask）。
    /// 持ち回さず、最後に読めた字も置かない（`text_within` を `BD_TIMEOUT` で撃ち、字だけを返す）。
    pub fn text_alone(&self) -> Option<String> {
        self.text_within(BD_TIMEOUT).ok()
    }

    /// `text_alone` と同じ bd の撃ちを上限 `timeout` の `run` で撃ち、読めれば字、落ちれば `Failed::word` の字
    /// （UTF-8 でなければ字 `UTF-8 でない`）を返す（裁定の受付の書きの前の読み・行 e-answer-reread）。
    pub fn text_within(&self, timeout: Duration) -> Result<String, String> {
        let out = run(&self.bd, BD_ARGS, &self.repo, timeout).map_err(|f| f.word())?;
        String::from_utf8(out).map_err(|_| "UTF-8 でない".to_string())
    }
}

/// 読めた字か（`parse_bd` か中核の台帳の読みが Known・created_at と updated_at の無い bead の台帳は中核だけが読む）。
fn readable(text: &str) -> bool {
    matches!(parse_bd(text), Reading::Known(_))
        || matches!(tsuzuri_core::ledger::stats(text, 0), Reading::Known(_))
}

/// 英数字と _ と - だけの空でない字か（db の名と manifest が名指す file の名）。
fn plain_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// manifest の字が名指す file の名（先頭の 5 つの欄の後の、名と chunk の数の組の名を順に・`plain_name` のものだけ）。
fn named(manifest: &str) -> Vec<&str> {
    manifest
        .trim()
        .split(':')
        .skip(5)
        .step_by(2)
        .filter(|name| plain_name(name))
        .collect()
}

/// JSON の object の字から鍵の字の値を読む（引用符で包んだ鍵の後の空白の次がコロンの所・
/// 値が逆斜線を含むか字でなければ None）。境界の crate は serde に直接依存しないので、
/// METADATA の 2 つの欄だけをこの形で読む。
fn json_str<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let quoted = format!("\"{key}\"");
    let mut rest = text;
    while let Some(at) = rest.find(&quoted) {
        rest = &rest[at + quoted.len()..];
        let Some(after) = rest.trim_start().strip_prefix(':') else {
            continue;
        };
        let value = after.trim_start().strip_prefix('"')?;
        let value = &value[..value.find('"')?];
        return (!value.contains('\\')).then_some(value);
    }
    None
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// bd の出力（bead の JSON の配列）を読む（server の読みの経路はこれだけ）。
/// 1 本でも読めなければ全体を Unknown にする（壊れた台帳の一部だけを見せない）。
pub fn parse_bd(text: &str) -> Reading<Vec<LedgerItem>> {
    match wire::decode::<Vec<BdLine>>(text) {
        Ok(lines) => items(lines.into_iter().map(Ok::<_, wire::Error>)),
        Err(_) => Reading::Unknown,
    }
}

/// bead を 1 本 1 行に並べた字（空の行は飛ばす）を読む。file は開かない純粋な関数で、
/// 面の歯が fixture を server と同じ規則で読むために残す（server の読みの経路では使わない）。
pub fn parse(text: &str) -> Reading<Vec<LedgerItem>> {
    items(
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(wire::decode::<BdLine>),
    )
}

/// bd の bead の列を中身にする（bd が消した bead は出さない・1 本でも読めなければ Unknown）。
fn items<E>(lines: impl Iterator<Item = Result<BdLine, E>>) -> Reading<Vec<LedgerItem>> {
    let mut items = Vec::new();
    for bd in lines {
        let Ok(bd) = bd else {
            return Reading::Unknown;
        };
        if bd.is_tombstone() {
            continue;
        }
        let Some(at) = epoch_secs(&bd.updated_at) else {
            return Reading::Unknown;
        };
        items.push(bd.into_item(at));
    }
    Reading::Known(items)
}

/// 持ち回しを含む読みの字を `parse_bd` で読む（字が無ければ Unknown）。
fn parsed(got: &Got) -> Reading<Vec<LedgerItem>> {
    got.text.as_deref().map_or(Reading::Unknown, parse_bd)
}

/// 台帳の一覧（口 GET /api/ledger・`Source::got` の値を受ける）。
pub fn list(got: &Got) -> LedgerList {
    LedgerList {
        rows: match parsed(got) {
            Reading::Known(items) => Reading::Known(items.into_iter().map(|i| i.row).collect()),
            Reading::Unknown => Reading::Unknown,
        },
    }
}

/// 1 本の引き（口 GET /api/ledger/<id>）の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup {
    Found(LedgerItem),
    /// 台帳は読めたが id が無い。
    Missing,
    /// 台帳が読めない。
    Unknown,
}

/// 1 本の引き（`Source::got` の値を受ける）。
pub fn item(got: &Got, id: &BeadId) -> Lookup {
    match parsed(got) {
        Reading::Known(items) => items
            .into_iter()
            .find(|i| &i.row.id == id)
            .map_or(Lookup::Missing, Lookup::Found),
        Reading::Unknown => Lookup::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{Source, epoch_secs, parse, parse_bd};
    use tsuzuri_contract::board::Reading;

    #[test]
    fn server_min_epoch_secs_reads_rfc3339() {
        assert_eq!(epoch_secs("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(epoch_secs("2026-09-27T07:39:00Z"), Some(1_790_494_740));
        assert_eq!(
            epoch_secs("2026-09-27T16:39:00.123456789+09:00"),
            Some(1_790_494_740)
        );
        assert_eq!(epoch_secs("2024-02-29T12:00:00-00:30"), Some(1_709_209_800));
        for bad in [
            "",
            "2026-09-27T07:39:00",
            "2026-02-29T00:00:00Z",
            "2026-13-01T00:00:00Z",
            "2026-09-27T24:00:00Z",
            "2026-09-27T07:39:00.Z",
            "2026-09-27T07:39:00+0900",
            "1969-12-31T23:59:59Z",
            "２０２６-09-27T07:39:00Z",
        ] {
            assert_eq!(epoch_secs(bad), None, "{bad}");
        }
    }

    #[test]
    fn server_min_parse_refuses_broken_lines() {
        let good =
            r#"{"id":"fx.1","title":"t","status":"open","updated_at":"2026-09-27T07:39:00Z"}"#;
        let gone =
            r#"{"id":"fx.2","title":"t","status":"tombstone","updated_at":"2026-09-27T07:39:00Z"}"#;
        let Reading::Known(items) = parse_bd(&format!("[{good},\n{gone}]\n")) else {
            panic!("読める台帳が Unknown");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(parse_bd("[]"), Reading::Known(vec![]));
        let bad_time = good.replace("07:39:00Z", "07:39:00");
        for broken in [
            format!("[{good},{{"),
            format!("[{bad_time}]"),
            good.to_string(),
            String::new(),
            "not json".to_string(),
        ] {
            assert_eq!(parse_bd(&broken), Reading::Unknown, "{broken}");
        }
        // 1 本 1 行の形（面の歯の fixture の読み）も同じ規則で読む。
        let Reading::Known(items) = parse(&format!("{good}\n\n{gone}\n")) else {
            panic!("読める行が Unknown");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(parse(""), Reading::Known(vec![]));
        for broken in [format!("{good}\n{{"), bad_time, "not json".to_string()] {
            assert_eq!(parse(&broken), Reading::Unknown, "{broken}");
        }
    }

    #[test]
    fn server_src_marks_are_two_files() {
        assert_eq!(
            Source::new("/r", "bd").marks(),
            [
                std::path::Path::new("/r/.beads/issues.jsonl"),
                std::path::Path::new("/r/.beads/interactions.jsonl"),
            ]
        );
    }

    #[test]
    fn server_src_unstartable_bd_is_unknown() {
        let source = Source::new(std::env::temp_dir(), "/nonexistent/tz-no-such-bd");
        assert_eq!(source.read(), Reading::Unknown);
    }
}
