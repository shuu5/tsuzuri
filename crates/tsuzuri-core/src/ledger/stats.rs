//! 台帳の指標と判定（見本の集計 docs/design/mock3/ledger.js の stats と judge と同じ数え方）。
//! 判定の閾値は `THRESHOLDS` の 1 か所に置き、歯が規則の行 R-18 の value の字の全体と照らす。

use serde::Deserialize;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{LedgerJudge, Reading};
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::stats::{
    DayCount, EpicProgress, LeadDays, LedgerStats, MemoStats, OpenCounts, UnreflectedCount,
    UnreflectedKind,
};

use super::{Bead, DAY, read, unreflected};

/// 判定の閾値（規則の行 R-18）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    /// stale が open の task のこの倍を超えれば滞り。
    pub stale_ratio: f64,
    /// 純減 7d がこれを超えれば積み増し。
    pub gain_7d: i64,
    /// closed/日 がこれ未満なら停滞。
    pub idle_per_day: f64,
    /// 更新がこの日数以上無い open の task が stale。
    pub stale_days: u64,
}

/// 判定の閾値の値（1 か所）。
pub const THRESHOLDS: Thresholds = Thresholds {
    stale_ratio: 0.5,
    gain_7d: 0,
    idle_per_day: 0.1,
    stale_days: 7,
};

impl Thresholds {
    /// 規則の行 R-18 の value の形に閾値を埋めた字。
    pub fn rule_text(&self) -> String {
        format!(
            "滞り = stale が open の task の {} 超・積み増し = 純減 7d が {} 超・停滞 = closed/日 が {} 未満・順に当てて最初の 1 つ・stale = {} 日 更新なし",
            self.stale_ratio, self.gain_7d, self.idle_per_day, self.stale_days
        )
    }
}

/// closed/日 と純減 7d の窓（日）。
pub const WEEK_DAYS: u64 = 7;

/// lead の窓（日）。
pub const LEAD_DAYS: u64 = 30;

/// 日ごとの本数（burndown と sparkline）。
pub const SPARK_DAYS: u64 = 14;

/// 昇格待ちの memo が description に持つ見出し。
pub const PROMOTION_HEADING: &str = "昇格条件";

/// 日の境の時差（日本時間・面の view.rs の JST_OFFSET と同じ）。
pub const DAY_OFFSET_S: u64 = 9 * 60 * 60;

/// t を含む日本の日の 23 時 59 分 59 秒（日ごとの本の end・今の時刻に依らない）。
pub fn day_end(t: EpochSecs) -> EpochSecs {
    ((t + DAY_OFFSET_S) / DAY + 1) * DAY - DAY_OFFSET_S - 1
}

/// 判定の材料（open の task の数・stale の数・純減 7d・closed/日）。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct JudgeInput {
    pub open_tasks: u32,
    pub stale: u32,
    pub net_drop_7d: i64,
    pub closed_per_day: f64,
}

/// 台帳の判定（規則の行 R-18 の順に当てて最初の 1 つ・材料が無ければ台帳なし）。
pub fn judge(x: Option<&JudgeInput>) -> LedgerJudge {
    let t = &THRESHOLDS;
    match x {
        None => LedgerJudge::NoLedger,
        Some(x)
            if x.open_tasks > 0 && f64::from(x.stale) > f64::from(x.open_tasks) * t.stale_ratio =>
        {
            LedgerJudge::Clogged
        }
        Some(x) if x.net_drop_7d > t.gain_7d => LedgerJudge::PilingUp,
        Some(x) if x.closed_per_day < t.idle_per_day => LedgerJudge::Stalled,
        Some(_) => LedgerJudge::OnTrack,
    }
}

/// 指標の読みの判定（台帳が読めなければ台帳なし）。
pub fn judge_of(stats: &Reading<LedgerStats>) -> LedgerJudge {
    match stats {
        Reading::Known(s) => s.judge,
        Reading::Unknown => judge(None),
    }
}

/// 台帳の一覧の字と今の時刻から指標を数える（字が読めなければ「まだ分からない」）。
pub fn stats(ledger: &str, now: EpochSecs) -> Reading<LedgerStats> {
    stats_of(read(ledger).as_deref(), now)
}

/// 読んだ bead（None は台帳が読めない）と今の時刻から指標を数える（None なら「まだ分からない」）。
pub(crate) fn stats_of(beads: Option<&[Bead]>, now: EpochSecs) -> Reading<LedgerStats> {
    match beads {
        Some(beads) => Reading::Known(of_beads(beads, now)),
        None => Reading::Unknown,
    }
}

fn count<T>(items: impl Iterator<Item = T>) -> u32 {
    u32::try_from(items.count()).unwrap_or(u32::MAX)
}

/// 値の列の分位（線形補間・見本の pct と同じ）。空なら None。
fn percentile(values: &[f64], p: f64) -> Option<f64> {
    let mut s = values.to_vec();
    s.sort_by(f64::total_cmp);
    let last = s.len().checked_sub(1)?;
    let i = last as f64 * p;
    let (lo, hi) = (i.floor() as usize, i.ceil() as usize);
    let (below, above) = (*s.get(lo)?, *s.get(hi)?);
    Some(below + (above - below) * (i - lo as f64))
}

/// 秒の差を日にする（負の差も見本と同じに負の日にする）。
fn days_between(from: EpochSecs, to: EpochSecs) -> f64 {
    (to as f64 - from as f64) / DAY as f64
}

/// description に「昇格条件」の見出しの行を持つか（`#` で始まり、`#` と空白の後が見出しの字で始まる行）。
fn has_promotion_heading(description: &str) -> bool {
    description.lines().any(|line| {
        let line = line.trim();
        line.starts_with('#')
            && line
                .trim_start_matches('#')
                .trim_start()
                .starts_with(PROMOTION_HEADING)
    })
}

/// 読めた bead から指標を数える。今の時刻より後に作られた bead は数えない（見本と同じ）。
/// 時点・日ごとの end・memo の作った時刻の中央値は今の時刻をそのまま持たない（年齢と経過は面が今から引く）。
pub(crate) fn of_beads(all: &[Bead], now: EpochSecs) -> LedgerStats {
    let beads: Vec<Bead> = all
        .iter()
        .filter(|b| b.created.is_none_or(|c| c <= now))
        .cloned()
        .collect();
    let t = &THRESHOLDS;
    let tasks: Vec<&Bead> = beads.iter().filter(|b| b.kind == NodeKind::Task).collect();
    let open: Vec<&Bead> = tasks.iter().copied().filter(|b| b.is_open(now)).collect();
    let open_at = |at: EpochSecs| count(tasks.iter().filter(|b| b.open_at(at)));
    let closed_in =
        |from: EpochSecs, to: EpochSecs, b: &Bead| b.closed.is_some_and(|x| x > from && x <= to);
    let ago = |days: u64| now.saturating_sub(days * DAY);

    let blocked = count(open.iter().filter(|b| {
        b.status == "blocked" || (b.status != "in_progress" && b.has_open_blocker(&beads, now))
    }));
    let ready = count(
        open.iter()
            .filter(|b| b.status == "open" && !b.has_open_blocker(&beads, now)),
    );
    let stale = count(open.iter().filter(|b| {
        b.updated
            .is_some_and(|u| now.saturating_sub(u) >= t.stale_days * DAY)
    }));
    let open_tasks = count(open.iter());
    let net_drop_24h = i64::from(open_tasks) - i64::from(open_at(ago(1)));
    let net_drop_7d = i64::from(open_tasks) - i64::from(open_at(ago(WEEK_DAYS)));
    let closed_7d = count(tasks.iter().filter(|b| closed_in(ago(WEEK_DAYS), now, b)));
    let closed_per_day = f64::from(closed_7d) / WEEK_DAYS as f64;
    let leads: Vec<f64> = tasks
        .iter()
        .filter(|b| closed_in(ago(LEAD_DAYS), now, b))
        .filter_map(|b| Some(days_between(b.created?, b.closed?)))
        .collect();
    let lead = percentile(&leads, 0.5)
        .zip(percentile(&leads, 0.9))
        .map(|(p50, p90)| LeadDays { p50, p90 });

    // 14 本は日本の日ごとで、窓は end の 1 日前の秒より後から end まで（今日の本は今の時刻まで）。
    let last = day_end(now);
    let days = (0..SPARK_DAYS)
        .rev()
        .map(|i| {
            let end = last.saturating_sub(i * DAY);
            let (from, to) = (end.saturating_sub(DAY), end.min(now));
            DayCount {
                end,
                created: count(
                    tasks
                        .iter()
                        .filter(|b| b.created.is_some_and(|c| c > from && c <= to)),
                ),
                closed: count(tasks.iter().filter(|b| closed_in(from, to, b))),
                open: open_at(to),
            }
        })
        .collect();

    let open_of = |kind: NodeKind| count(beads.iter().filter(|b| b.kind == kind && b.is_open(now)));
    let epics = beads
        .iter()
        .filter(|b| b.kind == NodeKind::Epic)
        .filter_map(|e| {
            let kids: Vec<&&Bead> = tasks
                .iter()
                .filter(|b| b.parent.as_deref() == Some(e.id.as_str()))
                .collect();
            let epic = BeadId::new(e.id.as_str()).ok()?;
            (!kids.is_empty()).then(|| EpicProgress {
                epic,
                closed: count(kids.iter().filter(|b| !b.is_open(now))),
                total: count(kids.iter()),
            })
        })
        .collect();

    let memos: Vec<&Bead> = beads.iter().filter(|b| b.kind == NodeKind::Memo).collect();
    let open_memos: Vec<&&Bead> = memos.iter().filter(|b| b.is_open(now)).collect();
    let memo_created: Vec<f64> = open_memos
        .iter()
        .filter_map(|b| Some(b.created? as f64))
        .collect();
    let memo = MemoStats {
        open: count(open_memos.iter()),
        awaiting_promotion: count(
            open_memos
                .iter()
                .filter(|b| has_promotion_heading(&b.description)),
        ),
        closed_7d: count(memos.iter().filter(|b| closed_in(ago(WEEK_DAYS), now, b))),
        created_p50: percentile(&memo_created, 0.5).map(|t| t.floor() as EpochSecs),
    };

    // 時点は台帳の最後の記録の時刻（今より後の記録は見ない・今の時刻に依らない）。
    let at = all
        .iter()
        .flat_map(|b| [b.created, b.updated, b.closed])
        .flatten()
        .filter(|&t| t <= now)
        .max()
        .unwrap_or(0);

    // 未反映は種類ごとの件数を 1 度だけ数え、数はその和。
    let list = unreflected::not_yet();
    let unreflected_kinds: Vec<UnreflectedCount> = UnreflectedKind::ALL
        .iter()
        .filter_map(|&kind| match list.get(kind) {
            Reading::Known(items) => Some(UnreflectedCount {
                kind,
                count: count(items.iter()),
            }),
            Reading::Unknown => None,
        })
        .collect();
    let unreflected = unreflected_kinds.iter().map(|k| k.count).sum();

    LedgerStats {
        at,
        judge: judge(Some(&JudgeInput {
            open_tasks,
            stale,
            net_drop_7d,
            closed_per_day,
        })),
        open: OpenCounts {
            task: open_tasks,
            memo: open_of(NodeKind::Memo),
            question: open_of(NodeKind::Question),
            epic: open_of(NodeKind::Epic),
        },
        blocked,
        ready,
        stale,
        net_drop_24h,
        net_drop_7d,
        closed_7d,
        closed_per_day,
        lead,
        days,
        epics,
        memo,
        unreflected,
        unreflected_kinds,
        unreflected_unknown: list.unknown(),
    }
}

#[cfg(test)]
mod tests {
    use super::{has_promotion_heading, percentile};

    #[test]
    fn stats_percentile_interpolates() {
        assert_eq!(percentile(&[], 0.5), None);
        assert_eq!(percentile(&[3.0], 0.9), Some(3.0));
        assert_eq!(percentile(&[4.0, 1.0, 2.0, 3.0], 0.5), Some(2.5));
    }

    #[test]
    fn stats_promotion_heading_is_a_heading_line() {
        assert!(has_promotion_heading(
            "## 出所\nx\n## 昇格条件\n持ち主の裁定"
        ));
        assert!(has_promotion_heading("#昇格条件"));
        assert!(!has_promotion_heading("昇格条件は未定"));
        assert!(!has_promotion_heading("## 候補 昇格条件"));
    }
}
