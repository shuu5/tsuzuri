//! 読む側の口（便 e-read）: 3 つの字を集めて中核の crate の関数に渡し、電文を返す。書かない。
//! - GET /api/pipeline — pipeline の板（PipelineBoard・札は集めた今の台帳の字から組み、形の崩れは見張りの読みの周の
//!   台帳の字と器の doctor の台帳の形の行の組から写す・行 c-pipe-misfit・行 c-misfit-pair）
//! - GET /api/metrics — 台帳の指標（読めなければ「まだ分からない」の LedgerStats）
//! - GET /api/next — 次の一手（NextStep・席の card が読めるときは席の card も受ける・便 e-seat）
//! - GET /api/graph — 導出グラフ（GraphDoc・repo に書かず毎回組み直す）
//! - GET /api/graph/view — 地図のグラフの眺め（GraphView・便 e-view）
//! - GET /api/around — 節点の近傍（AroundDoc・便 e-view）
//! - GET /api/unreflected — 未反映の一覧（UnreflectedList・台帳だけを読む・便 e-view）
//!
//! 字は要求のたびに集める。台帳は bd の読み（`ledger::Source`）が返した字、設計の索引は設計の道具の
//! 標準出力（`design::Design`）、走行は器の event log の file（`runs::Runs`）。読めない出所は空の字で渡し、
//! 中核の関数がその部分だけを「まだ分からない」にする。今の時刻は呼ぶ側が時計から取って引数で渡す。

use std::thread;
use std::time::Instant;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineBoard, Reading};
use tsuzuri_contract::graph::{
    AroundDoc, BeadAttr, Fold, GraphDoc, GraphSource, GraphView, InvariantCheck, RunAttr,
    SkippedEdges, Verdict,
};
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::stats::{LedgerStats, NextStep, UnreflectedList, UnreflectedRow};
use tsuzuri_core::graph::{self, Graph, Inputs, Invariant};
use tsuzuri_core::ledger::{Unreflected, UnreflectedItem};

use super::design::Design;
use super::form::{self, Kept};
use super::ledger::Source;
use super::runs::Runs;

/// 読みの出所の 3 つ（台帳・設計の索引・器の event log）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sources {
    pub ledger: Source,
    pub design: Design,
    pub runs: Runs,
}

/// 集めた 3 つの字と設計の索引の要約（読めない出所と集めなかった出所は空の字）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Texts {
    pub design: String,
    pub ledger: String,
    pub events: String,
    /// 設計の索引の要約（節点ごとの行と概要・行 c-summary-wire）。
    pub summary: String,
    /// 設計の道具の裁定の書き出し（folio check --emit-rulings の行・行 c-g3g7）。
    pub rulings: String,
}

impl Texts {
    /// 中核の crate の入力の形。
    pub fn inputs(&self) -> Inputs<'_> {
        Inputs {
            design_index: &self.design,
            ledger: &self.ledger,
            events: &self.events,
        }
    }
}

impl Sources {
    /// 台帳の字と、要るときだけ設計の索引とその要約と裁定の書き出しと event log の字を集める
    /// （子 process は並べて撃つので、待ちは 1 本分の上限まで）。
    pub fn gather(&self, design: bool, events: bool) -> Texts {
        self.gather_held(design, events).0
    }

    /// `gather` と同じ字と、台帳の読みが落ちたときの最後に読めた時刻（`Source::got` の stale・行 e-hold）。
    pub fn gather_held(&self, design: bool, events: bool) -> (Texts, Option<Instant>) {
        thread::scope(|s| {
            let index = design.then(|| s.spawn(|| self.design.text()));
            let summary = design.then(|| s.spawn(|| self.design.summary()));
            let rulings = design.then(|| s.spawn(|| self.design.rulings()));
            let ledger = self.ledger.got();
            let events = if events { self.runs.text() } else { None };
            let index = index.and_then(|h| h.join().ok().flatten());
            let summary = summary.and_then(|h| h.join().ok().flatten());
            let rulings = rulings.and_then(|h| h.join().ok().flatten());
            let texts = Texts {
                design: index.unwrap_or_default(),
                ledger: ledger.text.unwrap_or_default(),
                events: events.unwrap_or_default(),
                summary: summary.unwrap_or_default(),
                rulings: rulings.unwrap_or_default(),
            };
            (texts, ledger.stale)
        })
    }
}

/// pipeline の板（札は集めた台帳の字と event log の字から組み、event log が読めなければ「まだ分からない」・
/// 形の崩れの一覧は持った組から写し、組が無ければ「まだ分からない」）。
pub fn pipeline(texts: &Texts, kept: Option<&Kept>, now: EpochSecs) -> PipelineBoard {
    let mut board = tsuzuri_core::pipeline::board(&texts.ledger, &texts.events, now).board;
    board.misfits = form::misfits(kept, now);
    board
}

/// 台帳の指標（台帳が読めなければ「まだ分からない」）。
pub fn metrics(texts: &Texts, now: EpochSecs) -> Reading<LedgerStats> {
    tsuzuri_core::ledger::stats(&texts.ledger, now)
}

/// 次の一手。
pub fn next(texts: &Texts, now: EpochSecs) -> NextStep {
    tsuzuri_core::next_step::next_step(&texts.ledger, &texts.events, now)
}

/// 席の card も受けた次の一手（席の card が読めるとき・便 e-seat）。
pub fn next_seat(texts: &Texts, card: &SeatCard, now: EpochSecs) -> NextStep {
    tsuzuri_core::next_step::next_step_seat(&texts.ledger, &texts.events, now, Some(card))
}

/// 導出グラフを組み、要約を節点に写す（要約が読めなければ節点の行と概要は無しのまま・行 c-summary-wire）。
/// 裁定の書き出しを節点へ結んで ruled_by の辺を組む（読めなければ結んだ表は無しのまま・行 c-g3g7）。
/// tz graph --check も同じ組みを使う。
pub fn built(texts: &Texts) -> Graph {
    summed(texts).0
}

/// `built` と同じ導出グラフと、要約の無い節点の列（要約か設計の索引か台帳が読めなければまだ分からない・
/// 中核の `check::unsummarized`・行 k-sum-count）。
pub fn built_bare(texts: &Texts) -> (Graph, Reading<Vec<String>>) {
    let (g, summary) = summed(texts);
    let bare = graph::check::unsummarized(&g, summary);
    (g, bare)
}

/// tz graph --check が出す床の値（数えるだけで終了 code は変えない・行 k-g9-count）。
#[derive(Debug, Clone, PartialEq)]
pub struct Floors {
    /// 要約の無い節点の列（中核の `check::unsummarized`・行 k-sum-count）。
    pub bare: Reading<Vec<String>>,
    /// 本文だけで名指した id の対（中核の `check::unfielded_mentions`・g-9 の detect の数・行 c-g9）。
    pub unfielded: Reading<Vec<(String, String)>>,
}

/// `built` と同じ導出グラフと床の値（要約を写せたかを 2 つの床の関数に渡す・行 k-g9-count）。
/// tz graph --check が使う。
pub fn built_floors(texts: &Texts) -> (Graph, Floors) {
    let (g, summary) = summed(texts);
    let floors = Floors {
        bare: graph::check::unsummarized(&g, summary),
        unfielded: graph::check::unfielded_mentions(&g, summary),
    };
    (g, floors)
}

/// 導出グラフを組み、要約を写せたかを添えて返す。
fn summed(texts: &Texts) -> (Graph, bool) {
    let mut g = graph::build(&texts.inputs());
    let summary = graph::build::add_summary(&mut g, &texts.summary);
    graph::build::add_rulings(&mut g, &texts.rulings);
    (g, summary)
}

/// 導出グラフを組み、不変条件を数えて電文にする。
pub fn graph(texts: &Texts) -> GraphDoc {
    let g = built(texts);
    let invariants = graph::check(&g);
    doc(&g, &invariants)
}

/// 地図のグラフの眺め（導出グラフを組んで眺めの関数に渡す・便 e-view）。
pub fn view(texts: &Texts) -> GraphView {
    graph::view(&built(texts))
}

/// 開く列を受けた地図のグラフの眺め（行 c-graph-fold）。
pub fn view_open(texts: &Texts, open: &[String]) -> GraphView {
    graph::view_open(&built(texts), open)
}

/// 節点の近傍（中心の節点が無いか、段数が幅の外なら None・便 e-view）。
pub fn around(texts: &Texts, center: &str, steps: u8, fold: Fold) -> Option<AroundDoc> {
    graph::around(&built(texts), center, steps, fold)
}

/// 未反映の一覧（台帳が読めなければ 3 つとも「まだ分からない」・便 e-view）。
pub fn unreflected(texts: &Texts, now: EpochSecs) -> UnreflectedList {
    list(&tsuzuri_core::ledger::unreflected(&texts.ledger, now))
}

/// 中核の crate の未反映の一覧を電文に写す。
pub fn list(u: &Unreflected) -> UnreflectedList {
    let rows = |r: &Reading<Vec<UnreflectedItem>>| match r {
        Reading::Known(items) => Reading::Known(
            items
                .iter()
                .map(|i| UnreflectedRow {
                    id: i.id.clone(),
                    title: i.title.clone(),
                    age_s: i.age_s,
                })
                .collect(),
        ),
        Reading::Unknown => Reading::Unknown,
    };
    UnreflectedList {
        memos: rows(&u.memos),
        rulings: rows(&u.rulings),
        requests: rows(&u.requests),
    }
}

/// 中核の crate の Graph と check の値を電文に写す。
pub fn doc(g: &Graph, invariants: &[Invariant]) -> GraphDoc {
    GraphDoc {
        nodes: g.nodes.clone(),
        edges: g.edges.clone(),
        unread: g.unread.iter().map(|s| source(*s)).collect(),
        beads: g
            .beads
            .iter()
            .map(|(id, b)| {
                let attr = BeadAttr {
                    kind: b.kind,
                    status: b.status.clone(),
                    labels: b.labels.clone(),
                    pointers: b.pointers.clone(),
                    touches: b.touches.clone(),
                };
                (id.clone(), attr)
            })
            .collect(),
        runs: g
            .runs
            .iter()
            .map(|(id, r)| {
                let attr = RunAttr {
                    stage: r.stage.clone(),
                    account: r.account.clone(),
                };
                (id.clone(), attr)
            })
            .collect(),
        invariants: invariants.iter().map(invariant).collect(),
        skipped: SkippedEdges {
            design: count(g.skipped.design_edges),
            ledger: count(g.skipped.ledger_edges),
            design_nodes: count(g.skipped.design_nodes),
        },
    }
}

fn source(s: graph::Source) -> GraphSource {
    match s {
        graph::Source::Design => GraphSource::Design,
        graph::Source::Ledger => GraphSource::Ledger,
        graph::Source::Runs => GraphSource::Runs,
    }
}

fn invariant(inv: &Invariant) -> InvariantCheck {
    let (verdict, ids) = match &inv.verdict {
        graph::Verdict::Pass => (Verdict::Pass, Vec::new()),
        graph::Verdict::Violation(ids) => (Verdict::Violation, ids.clone()),
        graph::Verdict::Unknown => (Verdict::Unknown, Vec::new()),
    };
    InvariantCheck {
        id: inv.id.to_string(),
        verdict,
        violations: count(ids.len()),
        ids,
    }
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{Texts, doc, graph};
    use tsuzuri_contract::graph::{GraphSource, Verdict};
    use tsuzuri_core::graph::{Invariant, build};

    #[test]
    fn server_read_doc_copies_verdicts() {
        let g = build(&Texts::default().inputs());
        let got = doc(
            &g,
            &[
                Invariant {
                    id: "g-1",
                    verdict: tsuzuri_core::graph::Verdict::Pass,
                },
                Invariant {
                    id: "g-2",
                    verdict: tsuzuri_core::graph::Verdict::Violation(vec!["a".into(), "b".into()]),
                },
                Invariant {
                    id: "g-3",
                    verdict: tsuzuri_core::graph::Verdict::Unknown,
                },
            ],
        );
        assert_eq!(got.unread, GraphSource::ALL.to_vec());
        let brief: Vec<(&str, Verdict, u32, Vec<String>)> = got
            .invariants
            .iter()
            .map(|i| (i.id.as_str(), i.verdict, i.violations, i.ids.clone()))
            .collect();
        assert_eq!(
            brief,
            vec![
                ("g-1", Verdict::Pass, 0, vec![]),
                ("g-2", Verdict::Violation, 2, vec!["a".into(), "b".into()]),
                ("g-3", Verdict::Unknown, 0, vec![]),
            ]
        );
    }

    #[test]
    fn server_read_empty_texts_are_unread() {
        let doc = graph(&Texts::default());
        assert!(doc.nodes.is_empty());
        assert_eq!(doc.invariants.len(), 12);
        assert_eq!(doc.unread, GraphSource::ALL.to_vec());
    }
}
