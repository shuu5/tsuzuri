//! 台帳の指標（`stats`）と未反映の一覧（`unreflected`）（設計ノート surface-base 便 d）。
//! 入力は台帳の一覧の字（bd の読み取りの口が返す JSON の配列）と今の時刻（引数）で、
//! どの関数も file も子 process も時計も触らない。字を読んで口に出す側は境界の crate が持つ。
//! bead の種類は導出グラフと同じ順（epic・memo・問い・契約）で決め、指標が数える task は契約の bead。

pub mod stats;
pub mod unreflected;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::graph::NodeKind;

use crate::graph::build::{bead_kind, read_ledger};

pub use stats::{JudgeInput, THRESHOLDS, Thresholds, judge, judge_of, stats};
pub use unreflected::{Unreflected, UnreflectedItem, unreflected};

/// 1 日の秒。
pub const DAY: u64 = 86_400;

/// 数える bead の 1 本（時刻は epoch 秒・読めない時刻は None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Bead {
    pub(crate) id: String,
    pub(crate) kind: NodeKind,
    pub(crate) status: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) created: Option<EpochSecs>,
    pub(crate) updated: Option<EpochSecs>,
    pub(crate) closed: Option<EpochSecs>,
    /// 閉じた理由（bd の欄 close_reason の字のまま・無ければ None）。
    pub(crate) close_reason: Option<String>,
    /// 親（欄 parent か、parent-child の依存の先）。
    pub(crate) parent: Option<String>,
    /// blocks の先の bead。
    pub(crate) blockers: Vec<String>,
    /// acceptance の字（bd の欄 acceptance_criteria か別名 acceptance の字のまま・無ければ空・行 c-pipe-queue）。
    pub(crate) acceptance: String,
}

impl Bead {
    /// 時点 now で open か（status が closed でも tombstone でもなく、now 以前に閉じていない）。
    pub(crate) fn is_open(&self, now: EpochSecs) -> bool {
        self.status != "closed"
            && self.status != "tombstone"
            && !self.closed.is_some_and(|x| x <= now)
    }

    /// 時点 t で open の task として数えるか（created_at が t 以前で、closed_at が無いか t より後）。
    pub(crate) fn open_at(&self, t: EpochSecs) -> bool {
        self.created.is_some_and(|c| c <= t) && !self.closed.is_some_and(|x| x <= t)
    }

    /// blocks の先に open の bead を持つか（台帳に無い先は数えない）。
    pub(crate) fn has_open_blocker(&self, beads: &[Bead], now: EpochSecs) -> bool {
        self.blockers
            .iter()
            .any(|b| beads.iter().any(|x| &x.id == b && x.is_open(now)))
    }
}

/// 台帳の一覧を読む（読めなければ None）。bd が消した bead（tombstone）は数えない。
pub(crate) fn read(ledger: &str) -> Option<Vec<Bead>> {
    let beads = read_ledger(ledger)?;
    Some(
        beads
            .into_iter()
            .filter(|b| b.status.as_deref() != Some("tombstone"))
            .map(|b| {
                let labels = b.labels.unwrap_or_default();
                let deps = b.dependencies.unwrap_or_default();
                let dep_ids = |t: &str| {
                    deps.iter()
                        .filter(|d| d.dep_type == t)
                        .map(|d| d.depends_on_id.clone())
                        .collect::<Vec<_>>()
                };
                let created = b.created_at.as_deref().and_then(epoch_secs);
                Bead {
                    kind: bead_kind(b.issue_type.as_deref().unwrap_or_default(), &labels),
                    status: b.status.unwrap_or_default(),
                    title: b.title.unwrap_or_default(),
                    description: b.description.unwrap_or_default(),
                    created,
                    updated: b.updated_at.as_deref().and_then(epoch_secs).or(created),
                    closed: b.closed_at.as_deref().and_then(epoch_secs),
                    close_reason: b.close_reason,
                    parent: b
                        .parent
                        .or_else(|| dep_ids("parent-child").into_iter().next()),
                    blockers: dep_ids("blocks"),
                    acceptance: b.acceptance_criteria.unwrap_or_default(),
                    id: b.id,
                }
            })
            .collect(),
    )
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

#[cfg(test)]
mod tests {
    use super::{epoch_secs, read};

    #[test]
    fn stats_epoch_secs_reads_rfc3339() {
        assert_eq!(epoch_secs("2026-09-27T00:00:00Z"), Some(1_790_467_200));
        assert_eq!(epoch_secs("2026-09-27T09:00:00+09:00"), Some(1_790_467_200));
        assert_eq!(epoch_secs("2026-09-27T00:00:00.5Z"), Some(1_790_467_200));
        assert_eq!(epoch_secs("2026-02-29T00:00:00Z"), None);
        assert_eq!(epoch_secs("2026-09-27"), None);
    }

    #[test]
    fn stats_read_drops_tombstones_and_reads_parent_and_blocks() {
        let ledger = r#"[
            {"id":"a.1","status":"open","issue_type":"task","created_at":"2026-09-27T00:00:00Z",
             "dependencies":[{"depends_on_id":"a","type":"parent-child"},{"depends_on_id":"a.2","type":"blocks"}]},
            {"id":"a.2","status":"tombstone","issue_type":"task"}
        ]"#;
        let beads = read(ledger).expect("読める");
        assert_eq!(beads.len(), 1);
        assert_eq!(beads[0].parent.as_deref(), Some("a"));
        assert_eq!(beads[0].blockers, vec!["a.2".to_string()]);
        assert_eq!(
            beads[0].updated, beads[0].created,
            "更新時刻が無ければ作った時刻"
        );
        assert_eq!(read(""), None);
        assert_eq!(read("{"), None);
    }
}
