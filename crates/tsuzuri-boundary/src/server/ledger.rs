//! 台帳の読み（.beads の issues.jsonl を読むだけ・書かない）。
//! file が無いか 1 行でも読めなければ、一覧は 0 件でなく「まだ分からない」（Reading::Unknown）にする。

use std::path::{Path, PathBuf};

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BdLine, BeadId, LedgerItem, LedgerList};
use tsuzuri_contract::wire;

/// repo の置き場の中の台帳の file。
pub fn path(repo: &Path) -> PathBuf {
    repo.join(".beads").join("issues.jsonl")
}

/// 台帳の file を読む（無いか読めなければ Unknown）。
pub fn read(path: &Path) -> Reading<Vec<LedgerItem>> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(_) => Reading::Unknown,
    }
}

/// issues.jsonl の字を読む（空の行は飛ばし、bd が消した bead は出さない）。
/// 1 行でも読めなければ全体を Unknown にする（壊れた台帳の一部だけを見せない）。
pub fn parse(text: &str) -> Reading<Vec<LedgerItem>> {
    let mut items = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let Ok(bd) = wire::decode::<BdLine>(line) else {
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
pub fn list(path: &Path) -> LedgerList {
    LedgerList {
        rows: match read(path) {
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

pub fn item(path: &Path, id: &BeadId) -> Lookup {
    match read(path) {
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
    use super::{epoch_secs, parse};
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
        let Reading::Known(items) = parse(&format!("{good}\n\n{gone}\n")) else {
            panic!("読める台帳が Unknown");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(parse(""), Reading::Known(vec![]));
        let bad_time = good.replace("07:39:00Z", "07:39:00");
        for broken in [format!("{good}\n{{"), bad_time, "not json".to_string()] {
            assert_eq!(parse(&broken), Reading::Unknown, "{broken}");
        }
    }
}
