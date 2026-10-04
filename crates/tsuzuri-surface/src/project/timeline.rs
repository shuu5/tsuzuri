//! block「run の時間軸」（見本の bead.html の時間軸の panel・便 g-node-timeline・要件 FR10）: 節点の頁の 3 つ目の block。
//! 契約・epic の節点はその bead の走行を 1 行ずつ、走行の節点はその 1 行を、走行の読みの口（契約の型の runs の PATH）から出す。
//! 近傍の読み（nodearound の module の source）の中心の行から読む bead を決め、段の chip は続く同じ段を 1 つにまとめる。
//! 口の path・bead の決め方・段の色・chip・経験者の行・中身の 3 値・走行の link の card は純粋な関数にして host で試す。

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{AroundRow, GraphNode, NodeKind};
use tsuzuri_contract::runs::{self, RunLine, RunStep, RunsDoc};
use tsuzuri_contract::wire;

use super::nodearound::PageState;
use super::node::center;
use super::{Body, NO_CONTENT, NOT_READ};
use crate::account::home::{EXPERT_CHARS, wrap_words};
use crate::frame::{Block, Mode};
use crate::mapview::encode;
use crate::view::Fetched;
use crate::widgets::hover::Card;
use crate::widgets::nodecard::card_in;

pub const BLOCK: Block = Block {
    id: "timeline",
    heading: "timeline",
    class: "panel",
};

/// この file が字を持つ口の path（無い・走行の口の path は契約の型の runs の PATH・行 hs-derived）。
pub const PATHS: &[&str] = &[];

/// この file の畳める段の開き閉じの鍵の形（無い・行 hs-derived）。
pub const FOLDS: &[&str] = &[];

/// 走行の記録の口が読めないときの理由。
pub const REASON: &str = "走行の記録の口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 器の event log が読めないときの理由（口は答えたが走行の列が分からない）。
pub const UNREAD: &str = "器の event log が読めないので走行の列が分からない";

/// 測れて走行が 0 件のときの 1 行。
pub const NO_RUNS: &str = "この項目の run はまだ無い";

/// 値の無い欄の字。
pub const NONE: &str = "―";

/// 審査が通った結びの字（ほかの結びは止まりの色）。
pub const PASS: &str = "PASS";

/// 走行の読みの口の path（bead は `%XX` にする）。
pub fn path(bead: &str) -> String {
    format!("{}?bead={}", runs::PATH, encode(bead))
}

/// 走行の id の bead（最後のハイフンの後ろが器の時刻の形で前が空でなければ前・中核の graph の run_bead と同じ決まり）。
pub fn run_bead(run: &str) -> Option<&str> {
    let (bead, stamp) = run.rsplit_once('-')?;
    let b = stamp.as_bytes();
    let shaped = b.len() == 16
        && b.iter().enumerate().all(|(i, c)| match i {
            8 => *c == b'T',
            15 => *c == b'Z',
            _ => c.is_ascii_digit(),
        });
    (shaped && !bead.is_empty()).then_some(bead)
}

/// 読む走行の口（path）と、走行の頁で絞る走行の id（only）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Want {
    pub path: String,
    pub only: Option<String>,
}

/// 中心の行から読む走行（契約と epic はその bead・走行はその bead をその走行だけに絞る・ほかは読まない）。
pub fn request(row: &AroundRow) -> Option<Want> {
    let id = &row.node.id;
    match row.node.kind {
        NodeKind::Task | NodeKind::Epic => Some(Want {
            path: path(id),
            only: None,
        }),
        NodeKind::Run => run_bead(id).map(|bead| Want {
            path: path(bead),
            only: Some(id.clone()),
        }),
        _ => None,
    }
}

/// 近傍の読みの状態から読む走行（まだ読んでいない・読めない間は前の値を保つ）。
pub fn kept_want(before: Option<Want>, state: &PageState) -> Option<Want> {
    match state {
        PageState::Doc(doc) => center(doc).and_then(request),
        PageState::NotFound => None,
        PageState::NotRead | PageState::Unread(_) => before,
    }
}

/// 段の字・語の鍵・色の名（見本の ui.js の STAGE_COL と同じ）。
pub const STAGES: [(&str, &str, &str); 11] = [
    ("Intake", "stage:Intake", "wait"),
    ("Queued", "stage:Queued", "wait"),
    ("Blocked", "stage:Blocked", "wait"),
    ("Spawned", "stage:Spawned", "run"),
    ("Implemented", "stage:Implemented", "run"),
    ("Gated", "stage:Gated", "run"),
    ("Reviewed", "stage:Reviewed", "wait"),
    ("Questioned", "stage:Questioned", "stop"),
    ("Failed", "stage:Failed", "stop"),
    ("Stopped", "stage:Stopped", "stop"),
    ("Landed", "stage:Landed", "land"),
];

/// 段の色の名（審査の結びが PASS でなければ止まり・表に無い段は待ち）。
pub fn stage_col(stage: &str, verdict: Option<&str>) -> &'static str {
    if verdict.is_some_and(|v| v != PASS) {
        return "stop";
    }
    STAGES
        .iter()
        .find(|(s, _, _)| *s == stage)
        .map_or("wait", |(_, _, c)| c)
}

/// 段の chip（字・class・語の鍵）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chip {
    pub text: String,
    pub class: String,
    pub term: Option<&'static str>,
}

/// 段の列の chip（続く同じ段を 1 つにまとめ、群の中で最後の審査の結びを取る）。
pub fn chips(steps: &[RunStep]) -> Vec<Chip> {
    let mut groups: Vec<(&str, Option<&str>)> = Vec::new();
    for s in steps {
        match groups.last_mut() {
            Some((stage, verdict)) if *stage == s.stage => {
                if let Some(v) = s.verdict.as_deref() {
                    *verdict = Some(v);
                }
            }
            _ => groups.push((&s.stage, s.verdict.as_deref())),
        }
    }
    groups
        .into_iter()
        .map(|(stage, verdict)| Chip {
            text: match verdict {
                Some(v) => format!("{stage} {v}"),
                None => stage.to_string(),
            },
            class: format!("stg c-{}", stage_col(stage, verdict)),
            term: STAGES
                .iter()
                .find(|(s, _, _)| *s == stage)
                .map(|(_, k, _)| *k),
        })
        .collect()
}

/// 経験者向けの行（口座・費用・審査の結びと種類・どの行も 60 字以下・要件 FR14）。
pub fn expert(run: &RunLine) -> Vec<String> {
    let mut words = vec![format!("account={}", run.account.as_deref().unwrap_or(NONE))];
    let c = &run.cost;
    if c.events == 0 {
        words.push(format!("cost={NONE}"));
    } else {
        words.push(format!("turns={}", c.turns));
        words.push(format!("wall={}s", c.wall_ms / 1000));
        words.push(format!("in={}", c.tokens_in));
        words.push(format!("out={}", c.tokens_out));
        words.push(format!("cache={}/{}", c.cache_read, c.cache_create));
    }
    for s in &run.steps {
        if let Some(v) = &s.verdict {
            let kind = s
                .verdict_kind
                .as_deref()
                .map(|k| format!("/{k}"))
                .unwrap_or_default();
            words.push(format!("{}={v}{kind}", s.stage));
        }
    }
    wrap_words(&words.join(" "), EXPERT_CHARS)
}

/// 時間軸の 1 行（走行の id・回の数の字・段の chip・経験者の行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRow {
    pub run: String,
    /// 1 から数えた順の数（走行の頁では空の字）。
    pub nth: String,
    pub chips: Vec<Chip>,
    pub expert: Vec<String>,
}

/// block の中身（0 件と測れていないを分ける・要件 NFR2・`only` は走行の頁の走行の id）。
pub fn body(fetched: &Fetched, only: Option<&str>) -> Body<Vec<RunRow>> {
    let text = match fetched {
        Fetched::NotRead => return Body::Unmeasured(NOT_READ),
        Fetched::Failed => return Body::Unmeasured(REASON),
        Fetched::Body(text) => text,
    };
    let Ok(doc) = wire::decode::<RunsDoc>(text) else {
        return Body::Unmeasured(NO_CONTENT);
    };
    let Reading::Known(lines) = doc.runs else {
        return Body::Unmeasured(UNREAD);
    };
    let rows: Vec<RunRow> = lines
        .iter()
        .filter(|r| only.is_none_or(|o| r.run == o))
        .enumerate()
        .map(|(i, r)| RunRow {
            run: r.run.clone(),
            nth: if only.is_some() {
                String::new()
            } else {
                (i + 1).to_string()
            },
            chips: chips(&r.steps),
            expert: expert(r),
        })
        .collect();
    if rows.is_empty() {
        Body::Empty(NO_RUNS)
    } else {
        Body::Filled(rows)
    }
}

/// 走行の行の link の hover の card（初心者の表示の型の `run_card_in`）。
pub fn run_card(rows: &[AroundRow], run: &str) -> Card {
    run_card_in(rows, run, Mode::Beginner)
}

/// 走行の行の link の表示の型の hover の card（ほかの節点の頁への link と同じ `card_in` の card・行 g-accept-face・行 g-card-mode）。
/// 近傍の行に同じ id の節点が在ればその節点と状態から、無ければ走行の id だけの節点（種類は走行・状態なし）から組む。
pub fn run_card_in(rows: &[AroundRow], run: &str, mode: Mode) -> Card {
    match rows.iter().find(|r| r.node.id == run) {
        Some(r) => card_in(&r.node, r.status.as_deref(), mode),
        None => card_in(
            &GraphNode {
                id: run.to_string(),
                kind: NodeKind::Run,
                file: None,
                digest: None,
                title: String::new(),
                line: None,
                plain: None,
                eng: None,
                updated: None,
            },
            None,
            mode,
        ),
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::view;

/// 時間軸の block の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::{BLOCK, RunRow, Want, body, kept_want, run_card_in};
    use crate::frame::{Mode, node_href};
    use crate::project::nodearound::{PageState, mode_of, source, state};
    use crate::project::{Body, body_view, section, unmeasured};
    use crate::widgets::help::{shows_internal, term};
    use crate::widgets::hover::{Card, attach};

    /// やり直しの印（見本の IC.redo・14 px）。
    const REDO_ICON: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M20 12a8 8 0 1 1-2.3-5.7"/><path d="M20 4v5h-5"/></svg>"#;

    /// 段の間の矢印（見本の IC.arrow）。
    const ARROW_ICON: &str = r#"<svg class="arrow" viewBox="0 0 12 12" aria-hidden="true"><path d="M2 6h7M6 3l3 3-3 3" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>"#;

    /// 時間軸の block（契約・epic・走行でない節点と、近傍をまだ読んでいない間は何も出さない）。
    pub fn view() -> AnyView {
        let src = source();
        let Some(near) = src.read else {
            return ().into_any();
        };
        let mode = mode_of(src.search);
        let want = Memo::new(move |before: Option<&Option<Want>>| {
            let st = near.with(|(f, s)| state(f, *s));
            kept_want(before.cloned().flatten(), &st)
        });
        let path = Signal::derive(move || {
            want.with(|w| w.as_ref().map(|w| w.path.clone()).unwrap_or_default())
        });
        let runs = crate::net::read_path(path);
        let content = move || {
            let Some(w) = want.get() else {
                return ().into_any();
            };
            match runs.with(|(f, _)| body(f, w.only.as_deref())) {
                Body::Unmeasured(reason) => section(BLOCK, ().into_any(), unmeasured(reason)),
                Body::Empty(line) => section(BLOCK, ().into_any(), body_view(Body::Empty(line))),
                Body::Filled(rows) => {
                    let count = view! { <span class="chip num">{rows.len()}</span> }.into_any();
                    // 走行の link の card は近傍の行から引く（読めていなければ走行の id だけの節点）。
                    let around = near.with(|(f, s)| match state(f, *s) {
                        PageState::Doc(doc) => doc.rows,
                        _ => Vec::new(),
                    });
                    let list = rows
                        .into_iter()
                        .map(|r| {
                            let card = run_card_in(&around, &r.run, mode());
                            row_view(r, card, mode)
                        })
                        .collect_view();
                    section(BLOCK, count, view! { <div class="runs">{list}</div> }.into_any())
                }
            }
        };
        view! { {content} }.into_any()
    }

    /// 走行の 1 行（走行の節点の頁への link と hover の card・段の chip を矢印でつなぐ・経験者には口座と費用と審査の行）。
    fn row_view(
        r: RunRow,
        card: Card,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let id = r.run.clone();
        let href = move || node_href(&id, mode());
        let stages = r
            .chips
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                let arrow = (i > 0).then(|| view! { <span inner_html=ARROW_ICON></span> });
                let inner = match c.term {
                    Some(key) => term(key, c.text),
                    None => view! { <span>{c.text}</span> }.into_any(),
                };
                view! { {arrow}<span class=c.class>{inner}</span> }
            })
            .collect_view();
        let lines = r.expert;
        let expert_rows = move || {
            shows_internal(mode()).then(|| {
                let each = lines
                    .iter()
                    .map(|l| view! { <div>{l.clone()}</div> })
                    .collect_view();
                view! { <div class="small muted mono">{each}</div> }
            })
        };
        view! {
            <div class="runrow">
                <a class="nth num" href=href use:attach=card><span inner_html=REDO_ICON></span>" "{r.nth}</a>
                <div>
                    <div class="stages">{stages}</div>
                    {expert_rows}
                </div>
            </div>
        }
        .into_any()
    }
}
