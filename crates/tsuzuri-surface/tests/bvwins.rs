//! 行 g-win-parts の歯: 帯の印が開く窓の中身（判断の記録 ADR-27 決定 (4)(8)・見本 board-v2 の各窓）の幅と題の語と、
//! 止まった run の窓の並び、席と口座の窓の 24 時間の幅、今の block の中身の関数を窓が使う形（各 module の inner）、
//! 窓の DOM（wasm の枝）の配線の字、止まった run の窓の規則が stylesheet の窓の塊に在ること。
//! fixture は tests/fixtures/surface/pipeline-board.json（読むだけ）。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::{PipelineBoard, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::{BeadId, LedgerList, LedgerRow};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::{Mode, node_href};
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::pipeline::age_at;
use tsuzuri_surface::project::seat::{Span, WIN_SPAN};
use tsuzuri_surface::topbar::Win;
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::wins::{STALLED_NONE, frame_of, stalled};

/// 描く時の今（UTC の日の正午）。
const NOW: u64 = 1_790_510_400;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn pipe() -> Fetched {
    Fetched::Body(read("../../tests/fixtures/surface/pipeline-board.json"))
}

fn ledger(rows: &[(&str, &str)]) -> Fetched {
    let rows = rows
        .iter()
        .map(|(id, title)| LedgerRow {
            id: BeadId::new(*id).expect("id"),
            kind: "task".to_string(),
            title: (*title).to_string(),
            status: "open".to_string(),
            updated_at: 1,
            parent: None,
            labels: vec![],
        })
        .collect();
    Fetched::Body(
        wire::encode(&LedgerList {
            rows: Reading::Known(rows),
        })
        .expect("電文"),
    )
}

/// 窓の幅と題の語の鍵は見本の各窓の幅（質問 760・止まった run 720・知らせ 600・席と口座 760・抜けの検査 600・
/// 記号の見方 560・表示先 520）で、題の語は語の辞書に在る。
#[test]
fn bvwins_frames_widths_titles() {
    let want = [
        (Win::Ask, 760, "ask_open", "答えを待つ質問"),
        (Win::Stalled, 720, "bar_stall", "止まった run"),
        (Win::Notices, 600, "notice", "席からの知らせ"),
        (Win::Seat, 760, "seat_acct", "席と口座"),
        (Win::Gaps, 600, "gaps", "抜けの検査"),
        (Win::Legend, 560, "status", "記号の見方"),
        (Win::Dest, 520, "stage_target", "表示先"),
    ];
    for (win, px, key, word) in want {
        assert_eq!(frame_of(win), (px, key), "{win:?}");
        assert_eq!(
            vocab().term(key).map(|t| t.label.as_str()),
            Some(word),
            "{key}"
        );
    }
}

/// 止まった run の窓は止まりの列の札だけを新しい順に、題の全体と段と経過と理由と個別の頁の URL で並べる
/// （台帳が読めなければ題は無い）。
#[test]
fn bvwins_stalled_newest_first() {
    let rows = ledger(&[
        ("px.5", "問いで止まった題の全体"),
        ("px.6", "題 6"),
        ("px.3", "走る"),
    ]);
    let Body::Filled(list) = stalled(&pipe(), &rows, NOW, Mode::Expert) else {
        panic!("止まった run の並び");
    };
    let got: Vec<(&str, Option<&str>, &str, Option<&str>)> = list
        .iter()
        .map(|s| {
            (
                s.id.as_str(),
                s.title.as_deref(),
                s.stage.as_str(),
                s.reason.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            ("px.7", None, "Stopped", None),
            (
                "px.5",
                Some("問いで止まった題の全体"),
                "Questioned",
                Some("about:write-set")
            ),
            ("px.6", Some("題 6"), "Failed", Some("verify が赤")),
        ]
    );
    assert_eq!(list[0].age, age_at(Some(1_790_510_340), NOW));
    assert_eq!(list[2].age, age_at(Some(1_790_424_000), NOW));
    assert_eq!(list[1].href, node_href("px.5", Mode::Expert));
    let Body::Filled(no_titles) = stalled(&pipe(), &Fetched::Failed, NOW, Mode::Beginner) else {
        panic!("台帳が読めなくても並ぶ");
    };
    assert!(no_titles.iter().all(|s| s.title.is_none()));
}

/// 止まった run の窓は読めない口で測れていない・止まりの札が無ければ空（STALLED_NONE の 1 行）。
#[test]
fn bvwins_stalled_unread_or_none() {
    let rows = ledger(&[]);
    for f in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
    ] {
        assert!(matches!(
            stalled(&f, &rows, NOW, Mode::Beginner),
            Body::Unmeasured(_)
        ));
    }
    let only_run = PipelineBoard {
        cards: Reading::Known(vec![PipelineCard {
            contract: BeadId::new("px.3").expect("id"),
            runs: 1,
            stage: Stage::Running,
            reason: None,
            account: None,
            since: Some(NOW),
            ci: None,
        }]),
        misfits: Reading::Known(vec![]),
    };
    let f = Fetched::Body(wire::encode(&only_run).expect("電文"));
    assert!(matches!(stalled(&f, &rows, NOW, Mode::Beginner), Body::Empty(l) if l == STALLED_NONE));
}

/// 席と口座の窓の稼働の記録は 24 時間の幅だけで、窓の中の幅の button は stylesheet が出さない
/// （頁の block は今の幅の選びのまま・seat_view は替えない）。
#[test]
fn bvwins_seat_window_24h() {
    assert_eq!(WIN_SPAN, Span::H24);
    let dom = read("src/project_dom/seat.rs");
    for needle in [
        "section(BLOCK, ().into_any(), body(span, states))",
        "body(RwSignal::new(WIN_SPAN), states)",
        "Body::Filled(seat) => seat_view(seat, span, states, group_doc),",
    ] {
        assert_eq!(dom.matches(needle).count(), 1, "席の DOM の {needle}");
    }
    assert!(read("src/project/seat.rs").contains("    dom::inner()\n"));
    assert!(read("style.css").contains("\n.mb .spanbar { display: none; }\n"));
}

/// 知らせ・記号の見方・抜けの検査・表示先の block は中身を inner に分け、block の view も inner を描く（頁の形は替えない）。
#[test]
fn bvwins_blocks_inner_text() {
    for (file, view_uses) in [
        (
            "src/project/notice.rs",
            "section(BLOCK, ().into_any(), inner())",
        ),
        (
            "src/project/legend.rs",
            "super::section(BLOCK, ().into_any(), inner())",
        ),
        ("src/project/gaps.rs", "{inner()}"),
        ("src/project/stage.rs", "{body(f)}"),
    ] {
        let src = read(file);
        assert!(
            src.contains("pub fn inner() -> leptos::prelude::AnyView {"),
            "{file} に inner が無い"
        );
        assert!(
            src.contains(view_uses),
            "{file} の view が {view_uses} を持たない"
        );
    }
    let stage = read("src/project/stage.rs");
    assert!(stage.contains("        let f = Face::new();\n        load(f);\n        body(f)\n"));
}

/// 窓の DOM（wasm の枝）の配線の字: 窓の名ごとに中身を組む関数（質問は 1 問ずつの窓・行 g-ask-win）。
#[test]
fn bvwins_draw_wiring_text() {
    let src = read("src/wins.rs");
    let dom = &src[src.find("mod dom {").expect("wasm の枝")..];
    for needle in [
        "Win::Ask => return crate::askwin::frame(ctx),",
        "Win::Stalled => stalled_view(),",
        "Win::Notices => notice::inner(),",
        "Win::Seat => seat::inner(),",
        "Win::Gaps => view! { {gaps::inner()}{ledger::unref_panel()} }.into_any(),",
        "Win::Legend => legend::inner(),",
        "Win::Dest => stage::inner(),",
        "crate::net::read(pipeline::PATH)",
        "crate::net::read(ledger::PATH)",
        "<a class=\"sm\" href=c.href use:attach=card>",
    ] {
        assert!(dom.contains(needle), "wasm の枝に {needle} が無い");
    }
    assert!(read("src/lib.rs").contains("\npub mod wins;\n"));
}

/// 止まった run の窓の規則は stylesheet の窓の塊（#scrim の規則の後・一覧の型の前）に在る。
#[test]
fn bvwins_classes_in_stylesheet() {
    let css = read("style.css");
    let win = css.find("#scrim { position: fixed;").expect("窓の規則");
    let next = css.find("/* ---------- 一覧の型").expect("一覧の型の塊");
    for rule in [".stc {", ".stc-h {", ".stc-r {", ".stc-b {", ".mb .sm {"] {
        let i = css
            .find(rule)
            .unwrap_or_else(|| panic!("stylesheet に {rule} が無い"));
        assert!(win < i && i < next, "{rule} は窓の塊の中");
    }
}
