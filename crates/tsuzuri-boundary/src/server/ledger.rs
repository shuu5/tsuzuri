//! 台帳の読み（bd の読み取りの口を子 process で 1 本撃つだけ・書かない・便 e-src）。
//! 撃つ形は `bd --readonly list --all --limit 0 --json`（cwd は repo の置き場・標準入力は空・標準エラーは捨てる）。
//! 起動できない・rc が 0 でない・JSON として読めない・5 秒を超えて返さない、のどれでも
//! 一覧は 0 件でなく「まだ分からない」（Reading::Unknown）にする。
//! .beads の issues.jsonl は変化の印（更新時刻と長さ）として見るだけで、中身は読まない。
//! 同じ `Source` とその clone の読みは、走っている 1 本の子 process を分け合う（`coalesce`・便 e-coalesce）。

use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BdLine, BeadId, LedgerItem, LedgerList};
use tsuzuri_contract::wire;

use super::coalesce::{Coalesce, GRACE};

/// 既定の program の名（引数 --bd で替える）。
pub const BD: &str = "bd";

/// bd に渡す引数の列（読み取りだけ・closed を含む全部・件数の上限なし・JSON）。
pub const BD_ARGS: [&str; 6] = ["--readonly", "list", "--all", "--limit", "0", "--json"];

/// bd が返すまでの上限（要件 NFR2 の上限・規則の行 R-21 の値）。越えれば止めて Unknown。
pub const BD_TIMEOUT: Duration = Duration::from_secs(5);

/// 走っている読みに合流した呼び出しが待つ上限（`BD_TIMEOUT` に 1 秒を足す・便 e-coalesce）。
pub const BD_WAIT: Duration = BD_TIMEOUT.saturating_add(GRACE);

/// 子 process の終わりを確かめる間隔。
const WAIT_STEP: Duration = Duration::from_millis(5);

/// 台帳の読みの出所（repo の置き場と bd の program）。
/// clone は読みの合流の場を分け合う（比べるのは repo と bd だけ）。
#[derive(Debug, Clone)]
pub struct Source {
    pub repo: PathBuf,
    pub bd: OsString,
    shared: Coalesce<String>,
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
        }
    }

    /// 変化の印の 2 つの file（席の書きで動く issues.jsonl と、器の close を含む状態の変更で動く interactions.jsonl）。
    pub fn marks(&self) -> Vec<PathBuf> {
        let beads = self.repo.join(".beads");
        vec![beads.join("issues.jsonl"), beads.join("interactions.jsonl")]
    }

    /// bd を撃って台帳を読む。
    pub fn read(&self) -> Reading<Vec<LedgerItem>> {
        self.text().map_or(Reading::Unknown, |text| parse_bd(&text))
    }

    /// bd を撃ち、返した字をそのまま返す（導出グラフと指標の入力・便 e-read）。
    /// 起動できない・rc が 0 でない・UTF-8 でない・`BD_TIMEOUT` を越える、のどれでも None。
    /// 走っている読みが在れば新しく撃たず、その終わりを `BD_WAIT` まで待って同じ結果を返す（便 e-coalesce）。
    pub fn text(&self) -> Option<String> {
        self.shared.share(BD_WAIT, || self.text_alone())
    }

    /// 走っている読みを分け合わず、新しい子 process で bd を撃つ（裁定の受付の読み直し・便 e-ask）。
    pub fn text_alone(&self) -> Option<String> {
        let out = capture(&self.bd, BD_ARGS, &self.repo, BD_TIMEOUT)?;
        String::from_utf8(out).ok()
    }
}

/// 子の process group の全体へ KILL の signal を送る道具の名（便 e-reap）。
pub const KILL: &str = "kill";

/// 子 process を 1 本撃ち、rc 0 で `timeout` の内に返した標準出力を返す（それ以外は None）。
/// cwd は `cwd`・標準入力は空・標準エラーは捨てる。
/// 子は新しい process group に入れ（group の id は子の pid）、止めるときは孫まで group ごと止める（便 e-reap）。
pub fn capture<I, S>(program: &OsStr, args: I, cwd: &Path, timeout: Duration) -> Option<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let deadline = Instant::now() + timeout;
    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .ok()?;
    let Some(mut stdout) = child.stdout.take() else {
        stop(child);
        return None;
    };
    // 標準出力は別の thread で読み切る（pipe が詰まって子が止まらないように）。
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut out = Vec::new();
        let _ = tx.send(stdout.read_to_end(&mut out).map(|_| out));
    });
    let Ok(Ok(out)) = rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) else {
        stop(child);
        return None;
    };
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(WAIT_STEP),
            _ => {
                stop(child);
                return None;
            }
        }
    };
    status.success().then_some(out)
}

/// 子 process を group ごと止めて片付ける。
fn stop(child: Child) {
    stop_with(child, OsStr::new(KILL));
}

/// 子の process group（id は子の pid）の全体へ道具 `kill` で KILL の signal を送り、子を待って片付ける。
/// group へ送れたら true。道具が撃てないか失敗したら子だけを止めて false。
/// 孫は待たない（親が居ないので OS の側が片付ける）。
pub fn stop_with(mut child: Child, kill: &OsStr) -> bool {
    let group = format!("-{}", child.id());
    let sent = Command::new(kill)
        .args(["-KILL", "--", &group])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !sent {
        let _ = child.kill();
    }
    let _ = child.wait();
    sent
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

/// 台帳の一覧（口 GET /api/ledger）。
pub fn list(source: &Source) -> LedgerList {
    LedgerList {
        rows: match source.read() {
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

pub fn item(source: &Source, id: &BeadId) -> Lookup {
    match source.read() {
        Reading::Known(items) => items
            .into_iter()
            .find(|i| &i.row.id == id)
            .map_or(Lookup::Missing, Lookup::Found),
        Reading::Unknown => Lookup::Unknown,
    }
}

/// RFC 3339 の時刻（`YYYY-MM-DDTHH:MM:SS[.frac](Z|±HH:MM)`）を UTC の epoch 秒にする。
/// 形が違う・日付が無い・1970 年より前なら None。
pub fn epoch_secs(s: &str) -> Option<EpochSecs> {
    let num = |from: usize, to: usize| -> Option<i64> {
        let t = s.get(from..to)?;
        t.bytes()
            .all(|b| b.is_ascii_digit())
            .then(|| t.parse().ok())?
    };
    let b = s.as_bytes();
    let seps = [(4, b'-'), (7, b'-'), (13, b':'), (16, b':')];
    if b.len() < 20 || seps.iter().any(|&(i, c)| b[i] != c) || !matches!(b[10], b'T' | b't' | b' ')
    {
        return None;
    }
    let (year, month, day) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (hour, min, sec) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    if !(1..=12).contains(&month)
        || day < 1
        || day > days_in_month(year, month)
        || hour > 23
        || min > 59
        || sec > 60
    {
        return None;
    }
    let mut rest = s.get(19..)?;
    if let Some(frac) = rest.strip_prefix('.') {
        let digits = frac.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return None;
        }
        rest = &frac[digits..];
    }
    let offset = match rest.as_bytes() {
        [b'Z' | b'z'] => 0,
        [sign @ (b'+' | b'-'), _, _, b':', _, _] => {
            let (oh, om) = (num(s.len() - 5, s.len() - 3)?, num(s.len() - 2, s.len())?);
            if oh > 23 || om > 59 {
                return None;
            }
            let o = oh * 3600 + om * 60;
            if *sign == b'-' { -o } else { o }
        }
        _ => return None,
    };
    let total = days_from_civil(year, month, day) * 86_400 + hour * 3600 + min * 60 + sec - offset;
    EpochSecs::try_from(total).ok()
}

fn is_leap(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        2 if is_leap(y) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// 1970-01-01 からの日数（先発グレゴリオ暦）。
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
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
