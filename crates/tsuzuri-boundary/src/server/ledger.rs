//! 台帳の読み（bd の読み取りの口を子 process で 1 本撃つだけ・書かない・便 e-src）。
//! 撃つ形は `bd --readonly list --all --limit 0 --json`（cwd は repo の置き場・標準入力は空・標準エラーは捨てる）。
//! 起動できない・rc が 0 でない・JSON として読めない・5 秒を超えて返さない、のどれでも
//! 一覧は 0 件でなく「まだ分からない」（Reading::Unknown）にする。
//! 変化の印は台帳の store の manifest の中身と journal の長さで、store が無ければ issues.jsonl と interactions.jsonl の更新時刻と長さ（jsonl の中身は読まない・行 e-marks）。
//! 同じ `Source` とその clone の読みは、走っている 1 本の子 process を分け合う（`coalesce`・便 e-coalesce）。
//! 読めた字（`parse_bd` か中核の台帳の読みが Known の字）は最後に読めた字として持ち、次の読みが落ちたときだけ
//! 上限（既定 `READ_HOLD`・60 秒）まで `got` と `text` が返す（行 e-hold）。変化の見張りの `read` は持ち回さない。

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BdLine, BeadId, LedgerItem, LedgerList, READ_HOLD_S};
use tsuzuri_contract::wire;

use super::coalesce::{Coalesce, GRACE};
use super::events::stamp;

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

/// 読みが落ちても最後に読めた字を返す上限（契約の `READ_HOLD_S` 秒・行 e-hold）。
pub const READ_HOLD: Duration = Duration::from_secs(READ_HOLD_S);

/// repo の .beads の下の台帳の store の dir（その下が db ごとの dir・行 e-marks）。
pub const STORE_DIR: &str = "embeddeddolt";

/// db の dir の下の store の file の置き場。
pub const NOMS: &str = ".dolt/noms";

/// store の manifest（中身を印にする）。
pub const MANIFEST: &str = "manifest";

/// dolt の chunk journal の定まった名（長さを印にする）。
pub const JOURNAL: &str = "vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv";

/// 台帳の変化の印（行 e-marks）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    /// db ごとの NOMS の path と manifest の中身と journal の長さ。
    /// bd の読みは store の file の更新時刻を動かすので、更新時刻は見ない。
    Store(Vec<(PathBuf, Option<Vec<u8>>, Option<u64>)>),
    /// store の無いときの `Source::marks` の 2 file の更新時刻と長さ。
    Files(Vec<Option<(SystemTime, u64)>>),
}

/// 台帳の読みの出所（repo の置き場と bd の program）。
/// clone は読みの合流の場と最後に読めた字を分け合う（比べるのは repo と bd だけ）。
/// `got` と `text` は読みが落ちたとき、最後に読めた時から上限（`hold`）より短い間だけ最後に読めた字を返す。
/// `read`（変化の見張りの読み）は持ち回さず、落ちれば Unknown。`text_alone` は合流も持ち回しもしない。
#[derive(Debug, Clone)]
pub struct Source {
    pub repo: PathBuf,
    pub bd: OsString,
    shared: Coalesce<String>,
    /// 最後に読めた字と読んだ時刻（一度も読めていなければ字は None で、時刻は `new` を呼んだ時刻）。
    last: Arc<Mutex<(Option<String>, Instant)>>,
    /// 持ち回しの上限。
    hold: Duration,
}

/// 持ち回しを含む読みの結果（行 e-hold）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Got {
    /// 読めた字か、落ちたときは上限の内の最後に読めた字（越えたか一度も読めていなければ None）。
    pub text: Option<String>,
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
        }
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

    /// 台帳の store の db ごとの NOMS の path（MANIFEST が file のものを path の順に・dir が読めなければ空）。
    /// db の dir の名は呼ぶたびに dir を読んで決める。
    pub fn stores(&self) -> Vec<PathBuf> {
        let Ok(dir) = std::fs::read_dir(self.repo.join(".beads").join(STORE_DIR)) else {
            return Vec::new();
        };
        let mut stores: Vec<PathBuf> = dir
            .filter_map(|e| Some(e.ok()?.path().join(NOMS)))
            .filter(|noms| noms.join(MANIFEST).is_file())
            .collect();
        stores.sort();
        stores
    }

    /// 変化の印（store が在れば manifest の中身と journal の長さ、無ければ `marks` の 2 file の更新時刻と長さ）。
    pub fn mark(&self) -> Mark {
        let stores = self.stores();
        if stores.is_empty() {
            return Mark::Files(self.marks().iter().map(|m| stamp(m)).collect());
        }
        Mark::Store(
            stores
                .into_iter()
                .map(|noms| {
                    let manifest = std::fs::read(noms.join(MANIFEST)).ok();
                    let journal = std::fs::metadata(noms.join(JOURNAL)).ok().map(|m| m.len());
                    (noms, manifest, journal)
                })
                .collect(),
        )
    }

    /// bd を撃って台帳を読む（変化の見張りの読み・持ち回さない・落ちれば Unknown）。
    /// 読めた字は最後に読めた字に置く。
    pub fn read(&self) -> Reading<Vec<LedgerItem>> {
        self.fresh()
            .map_or(Reading::Unknown, |text| parse_bd(&text))
    }

    /// bd を撃ち、読めた字を返す（導出グラフと指標の入力・便 e-read）。落ちれば上限の内の最後に読めた字（`got`）。
    pub fn text(&self) -> Option<String> {
        self.got().text
    }

    /// 合流の読みを撃ち、読めれば読めた字と stale の None、落ちれば最後に読めた時刻と、
    /// その時刻から上限より短い間だけ最後に読めた字を返す（行 e-hold）。
    pub fn got(&self) -> Got {
        if let Some(text) = self.fresh() {
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

    /// 合流の読み（走っている読みが在れば新しく撃たず、その終わりを `BD_WAIT` まで待って同じ結果・便 e-coalesce）。
    /// 起動できない・rc が 0 でない・UTF-8 でない・`BD_TIMEOUT` を越える・`parse_bd` も中核の台帳の読みも
    /// Unknown の字、のどれでも None。読みを始めた呼びが、読めた字と時刻を最後に読めた字に置く。
    fn fresh(&self) -> Option<String> {
        self.shared.share(BD_WAIT, || {
            let text = self.text_alone().filter(|t| readable(t))?;
            *lock(&self.last) = (Some(text.clone()), Instant::now());
            Some(text)
        })
    }

    /// 走っている読みを分け合わず、新しい子 process で bd を撃つ（裁定の受付の読み直し・便 e-ask）。
    /// 持ち回さず、最後に読めた字も置かない。
    pub fn text_alone(&self) -> Option<String> {
        let out = capture(&self.bd, BD_ARGS, &self.repo, BD_TIMEOUT)?;
        String::from_utf8(out).ok()
    }
}

/// 読めた字か（`parse_bd` か中核の台帳の読みが Known・created_at と updated_at の無い bead の台帳は中核だけが読む）。
fn readable(text: &str) -> bool {
    matches!(parse_bd(text), Reading::Known(_))
        || matches!(tsuzuri_core::ledger::stats(text, 0), Reading::Known(_))
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
