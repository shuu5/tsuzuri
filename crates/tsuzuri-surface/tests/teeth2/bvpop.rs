//! 行 g-pop の歯: 札と一覧の行の吹き出し（判断の記録 ADR-27 決定 (7)・見本 board-v2 の popHTML）の共通の欄の順と
//! 段ごとの欄・読めない欄のまだ分からない・待つ相手と相手の段・開閉と外の click の判定・置き場と、
//! 吹き出しの id と class と下からの板の規則が stylesheet に在ることと、語の鍵と、層の DOM（wasm の枝）の配線の字。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{Ci, PipelineCard, Reading, Stage};
use tsuzuri_contract::case::{CaseLinks, CasePart};
use tsuzuri_contract::graph::{BeadAttr, GraphDoc, NodeKind, SkippedEdges};
use tsuzuri_contract::ledger::{BeadFact, BeadId, LedgerRow};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::{NO_CONTENT, NOT_READ, pipeline};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::hover::{Point, Rect, Size};
use tsuzuri_surface::widgets::modal::{FULL_MAX_PX, Hit, closes};
use tsuzuri_surface::widgets::pop::{
    BEADS_PATH, CLASSES, CLOSE_KEY, COMMON_KEYS, Fact, ID, NEXT_KEYS, Open, PAGE_KEY, Partner,
    SCRIM, STAGE_KEYS, Src, Sum, UNKNOWN_KEY, Val, Via, anchor_selectors, board_unread, hit_of,
    next_href, place, pop, toggle,
};

const NOW: EpochSecs = 1_790_000_000;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn id(s: &str) -> BeadId {
    BeadId::new(s).unwrap_or_else(|e| panic!("{s}: {e:?}"))
}

fn fact(x: &str, short: &str, blocks: &[&str]) -> BeadFact {
    BeadFact {
        id: id(x),
        created_at: Some(NOW - 7_200),
        short: short.to_string(),
        short_set: true,
        blocks: blocks.iter().map(|b| id(b)).collect(),
    }
}

fn row(x: &str, kind: &str, status: &str, parent: Option<&str>) -> LedgerRow {
    LedgerRow {
        id: id(x),
        kind: kind.to_string(),
        title: format!("{x} の題の全体"),
        status: status.to_string(),
        updated_at: NOW,
        parent: parent.map(id),
        labels: Vec::new(),
    }
}

fn card(x: &str, stage: Stage) -> PipelineCard {
    PipelineCard {
        contract: id(x),
        runs: 2,
        stage,
        reason: None,
        account: Some("acct-a".to_string()),
        since: Some(NOW - 600),
        ci: None,
    }
}

fn part(x: &str, on: &[&str]) -> CasePart {
    CasePart {
        part: "contract".to_string(),
        id: x.to_string(),
        phase: "contract-queued".to_string(),
        turn: "vessel".to_string(),
        since: Some(NOW - 600),
        reason: Some("dependency".to_string()),
        why: None,
        closed: false,
        links: CaseLinks {
            on: on.iter().map(|s| s.to_string()).collect(),
            runs: Vec::new(),
        },
    }
}

fn graph(pointers: &[(&str, &str)]) -> GraphDoc {
    let beads = pointers
        .iter()
        .map(|(x, p)| {
            let attr = BeadAttr {
                kind: NodeKind::Task,
                status: "open".to_string(),
                labels: Vec::new(),
                pointers: vec![p.to_string()],
                touches: Vec::new(),
            };
            (x.to_string(), attr)
        })
        .collect::<BTreeMap<_, _>>();
    GraphDoc {
        nodes: Vec::new(),
        edges: Vec::new(),
        unread: Vec::new(),
        beads,
        runs: BTreeMap::new(),
        invariants: Vec::new(),
        skipped: SkippedEdges {
            design: 0,
            ledger: 0,
            design_nodes: 0,
        },
        retired: None,
    }
}

fn keys(facts: &[Fact]) -> Vec<&'static str> {
    facts.iter().map(|f| f.key).collect()
}

fn val<'a>(facts: &'a [Fact], key: &str) -> &'a Val {
    &facts
        .iter()
        .find(|f| f.key == key)
        .unwrap_or_else(|| panic!("欄 {key} が無い"))
        .val
}

/// 共通の欄は id・種類・epic・設計の行・起票の順で、段ごとの欄はその後。頭は短い題・題の全体と概要は事実と台帳から。
#[test]
fn bvpop_common_rows_in_order() {
    let facts = [fact("t-1.2", "短い題", &[]), fact("t-1", "まとめ", &[])];
    let rows = [
        row("t-1", "epic", "open", None),
        row("t-1.1", "task", "open", Some("t-1")),
        row("t-1.2", "task", "open", Some("t-1.1")),
    ];
    let cards = [card("t-1.2", Stage::Running)];
    let doc = graph(&[("t-1.2", "design = contracts/n.toml#r-a")]);
    let src = Src {
        facts: Reading::Known(&facts),
        rows: Reading::Known(&rows),
        cards: &cards,
        parts: Reading::Known(&[]),
        graph: Some(&doc),
    };
    let p = pop("t-1.2", &src);
    let mut want = COMMON_KEYS.to_vec();
    want.extend([STAGE_KEYS[3], STAGE_KEYS[4]]);
    assert_eq!(keys(&p.facts), want);
    assert_eq!(*val(&p.facts, "pf_id"), Val::Code("t-1.2".to_string()));
    assert_eq!(
        *val(&p.facts, "pf_kind"),
        Val::Kind("k:契約", Some(Stage::Running))
    );
    assert_eq!(
        *val(&p.facts, "pf_epic"),
        Val::Epic(Partner {
            id: "t-1".to_string(),
            short: "まとめ".to_string(),
            stage: None
        })
    );
    assert_eq!(
        *val(&p.facts, "pf_row"),
        Val::Code("contracts/n.toml#r-a".to_string())
    );
    assert_eq!(*val(&p.facts, "pf_created"), Val::At(NOW - 7_200));
    assert_eq!(*val(&p.facts, "pf_runs"), Val::Text("2".to_string()));
    assert_eq!(
        *val(&p.facts, "pf_account"),
        Val::Text("acct-a".to_string())
    );
    assert_eq!(p.short, "短い題");
    assert_eq!(p.title.as_deref(), Some("t-1.2 の題の全体"));
    // 概要は事実の口から読まず、1 本の引きを読むまでまだ分からない（行 g-pop-sum の with_sum が置く）。
    assert_eq!(p.summary, Sum::Unread);
    assert_eq!((p.stage, p.next), (Some(Stage::Running), None));
    // epic の無い bead と pointer の無い bead は epic と設計の行の欄を出さない。札の無い bead は段ごとの欄を出さない。
    let lone = pop("t-1", &src);
    assert_eq!(
        keys(&lone.facts),
        [COMMON_KEYS[0], COMMON_KEYS[1], COMMON_KEYS[4]]
    );
    assert_eq!(*val(&lone.facts, "pf_kind"), Val::Kind("k:epic", None));
}

/// 口が読めない欄はまだ分からない（種類・epic・設計の行・起票・題の全体・概要・待つ相手・待ちの長さ）。頭は id。
#[test]
fn bvpop_unknown_fields_marked() {
    let mut c = card("t-9", Stage::Blocked);
    c.since = None;
    let cards = [c];
    let src = Src {
        facts: Reading::Unknown,
        rows: Reading::Unknown,
        cards: &cards,
        parts: Reading::Unknown,
        graph: None,
    };
    let p = pop("t-9", &src);
    let mut want = COMMON_KEYS.to_vec();
    want.extend([STAGE_KEYS[0], STAGE_KEYS[1]]);
    assert_eq!(keys(&p.facts), want);
    for f in &p.facts[1..] {
        assert_eq!(f.val, Val::Unknown, "{}", f.key);
    }
    assert_eq!(
        (p.short.as_str(), p.title, p.summary),
        ("t-9", None, Sum::Unread)
    );
}

/// Blocked の待つ相手: 局面の出力の契約の部品が在れば links.on（相手の短い題と札の段）、無ければ事実の blocks のうち
/// 台帳で閉じていない相手、links.on が空ならまだ分からない。
#[test]
fn bvpop_blocked_targets_with_stage() {
    let facts = [
        fact("t-2", "待つ便", &["t-3", "t-4"]),
        fact("t-3", "閉じた相手", &[]),
        fact("t-5", "走る相手", &[]),
    ];
    let rows = [
        row("t-2", "task", "open", None),
        row("t-3", "task", "closed", None),
        row("t-4", "task", "open", None),
    ];
    let cards = [card("t-2", Stage::Blocked), card("t-5", Stage::Running)];
    let parts = [part("t-2", &["t-5", "t-6"])];
    let mut src = Src {
        facts: Reading::Known(&facts),
        rows: Reading::Known(&rows),
        cards: &cards,
        parts: Reading::Known(&parts),
        graph: None,
    };
    let on = |src: &Src<'_>| val(&pop("t-2", src).facts, "pf_wait_on").clone();
    assert_eq!(
        on(&src),
        Val::Partners(vec![
            Partner {
                id: "t-5".to_string(),
                short: "走る相手".to_string(),
                stage: Some(Stage::Running)
            },
            Partner {
                id: "t-6".to_string(),
                short: "t-6".to_string(),
                stage: None
            },
        ])
    );
    src.parts = Reading::Unknown;
    assert_eq!(
        on(&src),
        Val::Partners(vec![Partner {
            id: "t-4".to_string(),
            short: "t-4".to_string(),
            stage: None
        }])
    );
    let empty = [part("t-2", &[])];
    src.parts = Reading::Known(&empty);
    assert_eq!(on(&src), Val::Unknown);
    assert_eq!(
        *val(&pop("t-2", &src).facts, "pf_waited"),
        Val::At(NOW - 600)
    );
}

/// Queued は列に入った時刻（札の since・無ければまだ分からない）。
#[test]
fn bvpop_queued_since() {
    let mut cards = [card("t-7", Stage::Queued)];
    let src = |cards: &[PipelineCard]| {
        let p = pop(
            "t-7",
            &Src {
                facts: Reading::Unknown,
                rows: Reading::Unknown,
                cards,
                parts: Reading::Unknown,
                graph: None,
            },
        );
        (
            keys(&p.facts)[5..].to_vec(),
            val(&p.facts, "pf_queued").clone(),
        )
    };
    assert_eq!(src(&cards), (vec![STAGE_KEYS[2]], Val::At(NOW - 600)));
    cards[0].since = None;
    assert_eq!(src(&cards), (vec![STAGE_KEYS[2]], Val::Unknown));
}

/// 止まりは理由の全文（切らない）と run の回と口座と次の手（Questioned は質問の窓・Failed と Stopped は run の時間軸）。
#[test]
fn bvpop_stop_reason_full() {
    let long = "verdict:FAIL kind:contract-fit 実装が節の字と合わない所が 3 つ在り、gate が落とした（長い理由の全文）";
    for (stage, next) in [
        (Stage::Failed, NEXT_KEYS[1]),
        (Stage::Stopped, NEXT_KEYS[1]),
        (Stage::Questioned, NEXT_KEYS[0]),
    ] {
        let mut c = card("t-8", stage);
        c.reason = Some(long.to_string());
        let cards = [c];
        let p = pop(
            "t-8",
            &Src {
                facts: Reading::Unknown,
                rows: Reading::Unknown,
                cards: &cards,
                parts: Reading::Unknown,
                graph: None,
            },
        );
        assert_eq!(
            keys(&p.facts)[5..],
            [STAGE_KEYS[5], STAGE_KEYS[3], STAGE_KEYS[4]]
        );
        assert_eq!(*val(&p.facts, "pf_reason"), Val::Text(long.to_string()));
        assert_eq!(p.next, Some(next));
    }
    assert_eq!(
        next_href(NEXT_KEYS[0], "t-8", Mode::Beginner),
        "?mode=beginner&win=ask"
    );
    assert_eq!(
        next_href(NEXT_KEYS[1], "t-8", Mode::Expert),
        "?page=node&id=t-8&mode=expert#timeline"
    );
}

/// 着地は着地の時刻と CI の語（読みが無ければまだ分からない）。
#[test]
fn bvpop_landed_time_ci() {
    let mut c = card("t-9", Stage::Landed);
    c.ci = Some(Ci::Success);
    let mut cards = [c];
    let facts = |cards: &[PipelineCard]| {
        pop(
            "t-9",
            &Src {
                facts: Reading::Unknown,
                rows: Reading::Unknown,
                cards,
                parts: Reading::Unknown,
                graph: None,
            },
        )
        .facts
    };
    let f = facts(&cards);
    assert_eq!(keys(&f)[5..], [STAGE_KEYS[6], STAGE_KEYS[7]]);
    assert_eq!(*val(&f, "pf_landed"), Val::At(NOW - 600));
    assert_eq!(*val(&f, "pf_ci"), Val::Text(label("ci_success")));
    cards[0].ci = None;
    assert_eq!(*val(&facts(&cards), "pf_ci"), Val::Unknown);
}

/// 同じ口の 2 度目は閉じ、違う口か違う bead は開く。中の click は閉じず、口の上の click は口に任せ、外の click は閉じる。
#[test]
fn bvpop_toggle_and_outside() {
    let open = |x: &str, via| Open {
        id: x.to_string(),
        via,
    };
    assert_eq!(
        toggle(None, open("a", Via::Card)),
        Some(open("a", Via::Card))
    );
    assert_eq!(
        toggle(Some(&open("a", Via::Card)), open("a", Via::Card)),
        None
    );
    assert_eq!(
        toggle(Some(&open("a", Via::Card)), open("a", Via::Row)),
        Some(open("a", Via::Row))
    );
    assert_eq!(
        toggle(Some(&open("a", Via::Card)), open("b", Via::Card)),
        Some(open("b", Via::Card))
    );
    assert_eq!(hit_of(true, true), Some(Hit::Inside));
    assert_eq!(hit_of(true, false), Some(Hit::Inside));
    assert_eq!(hit_of(false, true), None);
    assert_eq!(hit_of(false, false), Some(Hit::Outside));
    assert!(!closes(Hit::Inside) && closes(Hit::Outside));
    assert_eq!(
        anchor_selectors(&open("t-1.2", Via::Row)),
        [
            "[data-pop-row=\"t-1.2\"]".to_string(),
            "[data-pop-card=\"t-1.2\"]".to_string()
        ]
    );
}

/// 置き場: 口の右・右端を越えれば左・左端も越えれば口の下・下端を越えれば上へ寄せる（上端の下限 60）・口が無ければ真ん中の上。
#[test]
fn bvpop_place_right_left_below() {
    let win = Size {
        width: 1280.0,
        height: 800.0,
    };
    let pop_size = Size {
        width: 380.0,
        height: 300.0,
    };
    let r = |left, top| Rect {
        left,
        top,
        width: 200.0,
        height: 60.0,
    };
    assert_eq!(
        place(Some(r(100.0, 120.0)), pop_size, win),
        Point { x: 308.0, y: 120.0 }
    );
    assert_eq!(
        place(Some(r(900.0, 120.0)), pop_size, win),
        Point { x: 512.0, y: 120.0 }
    );
    let narrow = Size {
        width: 700.0,
        height: 800.0,
    };
    assert_eq!(
        place(Some(r(200.0, 120.0)), pop_size, narrow),
        Point { x: 200.0, y: 186.0 }
    );
    assert_eq!(
        place(Some(r(100.0, 700.0)), pop_size, win),
        Point { x: 308.0, y: 492.0 }
    );
    let short = Size {
        width: 1280.0,
        height: 300.0,
    };
    assert_eq!(
        place(Some(r(100.0, 100.0)), pop_size, short),
        Point { x: 308.0, y: 60.0 }
    );
    assert_eq!(place(None, pop_size, win), Point { x: 450.0, y: 80.0 });
}

/// 吹き出しの id と幕の id と class が stylesheet に在り、幅 600 px 以下（規則の行 R-35）で幕を出し吹き出しを下からの板にする。
#[test]
fn bvpop_classes_in_stylesheet() {
    let css = read("style.css");
    assert_eq!((ID, SCRIM), ("pop", "popscrim"));
    assert!(css.contains("#pop { position: fixed;"));
    assert!(css.contains("#popscrim { display: none; }"));
    for c in CLASSES {
        assert!(css.contains(&format!(".{c} ")), "{c}");
    }
    let start = css.find("#popscrim { display: none; }").unwrap_or(0);
    let media = format!("@media (max-width: {FULL_MAX_PX}px) {{");
    let at = css[start..].find(&media).map(|i| start + i);
    let block = &css[at.unwrap_or(css.len())..];
    assert!(block.contains("#popscrim { display: block;"));
    assert!(
        block.contains("#pop { left: 0 !important; right: 0; top: auto !important; bottom: 0;")
    );
}

/// 欄と口と × とまだ分からないの語の鍵が語の辞書に在る。
#[test]
fn bvpop_word_keys() {
    let all = COMMON_KEYS
        .iter()
        .chain(&STAGE_KEYS)
        .chain(&NEXT_KEYS)
        .chain(&[CLOSE_KEY, PAGE_KEY, UNKNOWN_KEY]);
    for k in all {
        let t = vocab()
            .term(k)
            .unwrap_or_else(|| panic!("{k} が語の辞書に無い"));
        assert!(!t.label.is_empty() && !t.note.is_empty(), "{k}");
    }
}

/// 層の DOM（wasm の枝）の配線: 5 つの口を読み、取り消しの鍵と頁の click を窓の枠の閉じる判定に渡し、App が層を頁に 1 つ置く。
#[test]
fn bvpop_dom_wiring_text() {
    let src = read("src/widgets/pop.rs");
    assert_eq!(BEADS_PATH, "/api/beads");
    for s in [
        "crate::net::read(BEADS_PATH)",
        "crate::net::read(CASES_PATH)",
        "crate::net::read(pipeline::PATH)",
        "crate::net::read(ledger::PATH)",
        "crate::net::read(map::PATH)",
        "window_event_listener(ev::keydown",
        "window_event_listener(ev::click",
        "if closes(h) {",
        "on:click=move |_| ctx.hit(Hit::X)",
        "hit_of(near(&format!(\"#{ID}\")), near(&opener_selector()))",
    ] {
        assert!(src.contains(s), "{s}");
    }
    let board = read("src/board.rs");
    assert!(board.contains("provide_context(PopCtx::default());"));
    assert!(board.contains("<PopLayer/>"));
}

/// 板の札が読めない（まだ読んでいない・口が読めない・電文が読めない）間は board_unread がその理由を返し、層は吹き出しの末に
/// 測れていないの 1 行（unmeasured）で出す（札が無い bead と同じ形の、段ごとの欄の無い吹き出しだけを見せない）。読めた板は None。
#[test]
fn bvpop_board_unread_marked() {
    assert_eq!(board_unread(&Fetched::NotRead), Some(NOT_READ));
    assert_eq!(board_unread(&Fetched::Failed), Some(pipeline::REASON));
    assert_eq!(
        board_unread(&Fetched::Body("{".to_string())),
        Some(NO_CONTENT)
    );
    let board = read("../../tests/fixtures/surface/pipeline-board.json");
    assert_eq!(board_unread(&Fetched::Body(board)), None);
    let src = read("src/widgets/pop.rs");
    let dom = &src[src.find("pub fn PopLayer(").expect("層")..];
    for s in [
        "let unread = pipe.with(board_unread);",
        "{unread.map(unmeasured)}",
    ] {
        assert!(dom.contains(s), "{s}");
    }
}
