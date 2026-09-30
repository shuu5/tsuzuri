//! 時刻の読み（RFC 3339 の字を UTC の epoch 秒にする・行 hb-proc が台帳の読みから割った）。

use tsuzuri_contract::EpochSecs;

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
    if b.len() < 20
        || seps.iter().any(|&(i, c)| b.get(i) != Some(&c))
        || !matches!(b.get(10), Some(b'T' | b't' | b' '))
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
