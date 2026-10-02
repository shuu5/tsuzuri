//! 行 g-layout の歯（接頭辞 bvlay_）: 幅の 3 段（1200 px 以上は横並び・601〜1199 px は縦積み・600 px 以下はスマホの形・
//! 規則の行 R-35）とスマホの段の tile（既定で開くのは札の在る段のうち一番急ぐ段・判断の記録 ADR-27 決定 (5)）。
//! 境の値は tiles の定数と stylesheet の media の字と規則の行の字を照らし、tile の組みは host で撃つ（bead の id の接頭辞は fx-l）。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::{PipelineCard, PipelineColumn, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_surface::project::pipeline::columns;
use tsuzuri_surface::tiles::{
    PHONE_MAX_PX, STACK_COL_MIN_PX, STACK_MAX_PX, STACK_MIN_PX, URGENT, WIDE_MIN_PX, default_open,
    open_of, press, tiles,
};
use tsuzuri_surface::widgets::keyline::QUEUED_WARN_S;

/// 板を組む今。
const NOW: u64 = 1_790_510_400;

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn card(id: &str, stage: Stage, since: u64) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("id"),
        runs: 1,
        stage,
        reason: None,
        account: None,
        since: Some(since),
        ci: None,
    }
}

/// stylesheet の中の `head` で始まる media の塊（次の行頭の閉じ括弧まで）。
fn block<'a>(css: &'a str, head: &str) -> &'a str {
    let at = css
        .find(head)
        .unwrap_or_else(|| panic!("stylesheet に {head} が無い"));
    let rest = &css[at..];
    let end = rest.find("\n}\n").expect("塊の閉じ");
    &rest[..end]
}

/// 境の値は 1200・601・1199・600 で、規則の行 R-35 の字と同じ・stylesheet の 3 つの media の境も同じ字。
#[test]
fn bvlay_breakpoints_are_rule_r35() {
    assert_eq!(
        (WIDE_MIN_PX, STACK_MIN_PX, STACK_MAX_PX, PHONE_MAX_PX),
        (1200, 601, 1199, 600)
    );
    assert_eq!(STACK_MIN_PX, PHONE_MAX_PX + 1);
    assert_eq!(STACK_MAX_PX, WIDE_MIN_PX - 1);
    let rules = read("../../design-intent/rules.yaml");
    let row = rules
        .lines()
        .find(|l| l.contains("id: R-35,"))
        .expect("規則の行 R-35 が在る");
    for want in [
        format!("窓の幅 {WIDE_MIN_PX} px 以上は横並び"),
        format!("{STACK_MIN_PX} px 以上 {STACK_MAX_PX} px 以下は縦積み"),
        format!("・{PHONE_MAX_PX} px 以下はスマホの形"),
        "段の tile".to_string(),
    ] {
        assert!(row.contains(&want), "R-35 に {want} が無い: {row}");
    }
    let css = read("style.css");
    let wide = block(&css, &format!("@media (min-width: {WIDE_MIN_PX}px) {{"));
    assert!(wide.contains(".one { grid-template-columns: "), "{wide}");
    let stack = block(
        &css,
        &format!("@media (min-width: {STACK_MIN_PX}px) and (max-width: {STACK_MAX_PX}px) {{"),
    );
    assert!(
        stack.contains("grid-template-rows: minmax(0, 45%) minmax(0, 1fr);"),
        "{stack}"
    );
    assert!(
        stack.contains(".one > .pane:has(#pipe) { order: -1; }"),
        "{stack}"
    );
    assert!(
        css.contains(&format!(
            "@media (max-width: {PHONE_MAX_PX}px) {{\n  .one {{"
        )),
        "スマホの形の塊"
    );
}

/// 縦積みの板は 5 列を保ち、狭い時は pipeline の面の中の板だけを横に scroll させる（列の幅の下限 112 px）。
/// 縦積みの media の中で overflow の字を持つ規則は板の 1 つだけ（面の列や面が横に scroll しない）。
#[test]
fn bvlay_stack_board_scrolls_x() {
    assert_eq!(STACK_COL_MIN_PX, 112);
    let css = read("style.css");
    let stack = block(
        &css,
        &format!("@media (min-width: {STACK_MIN_PX}px) and (max-width: {STACK_MAX_PX}px) {{"),
    );
    let want = format!(
        "#pipe .board {{ grid-template-columns: repeat(5, minmax({STACK_COL_MIN_PX}px, 1fr)); overflow-x: auto; }}"
    );
    assert!(stack.contains(&want), "{stack}");
    assert!(!stack.contains("overflow-x: hidden"), "{stack}");
    assert_eq!(stack.matches("overflow").count(), 1, "{stack}");
}

/// スマホの形は頁を縦に scroll させ（面の高さを決めない）、板を隠して段の tile を出し、pipeline を一覧の上に置く。
/// tile の既定の隠しは media の塊の外の行頭の 1 つで、tile の display の規則はそれとスマホの形の塊の 2 つだけ。
#[test]
fn bvlay_phone_tiles_shown() {
    let css = read("style.css");
    let phone = &css[css
        .find(&format!(
            "@media (max-width: {PHONE_MAX_PX}px) {{\n  .one {{"
        ))
        .expect("スマホの形の塊")..];
    let phone = &phone[..phone.find("\n}\n").expect("塊の閉じ")];
    for want in [
        ".one { display: flex; flex-direction: column; height: auto;",
        ".one > .pane:has(#pipe) { order: -1; }",
        ".one > .pane > .panel { flex: none; overflow: visible; }",
        "#pipe .board { display: none; }",
        "#pipe .ptiles { display: block; }",
    ] {
        assert!(phone.contains(want), "スマホの形の塊に {want} が無い");
    }
    // tile の既定の隠しは media の塊の外（行頭）に置き、どの幅でも板の代わりに出るのはスマホの形だけ。
    assert!(
        css.contains("\n.ptiles { display: none; }\n"),
        "tile の既定の隠し"
    );
    assert_eq!(
        css.matches(".ptiles { display:").count(),
        2,
        "tile の display"
    );
    for rule in [".ptrow {", ".ptile {", ".ptile.on {", ".ptlist {"] {
        assert!(css.contains(rule), "stylesheet に {rule} が無い");
    }
}

/// 札: 段ごとに 1 枚（走り・Queued・Blocked・Failed・着地の順）。
fn fixture() -> [PipelineCard; 5] {
    [
        card("fx-l.1", Stage::Running, NOW - 60),
        card("fx-l.2", Stage::Queued, NOW - 120),
        card("fx-l.3", Stage::Blocked, NOW - 300),
        card("fx-l.4", Stage::Failed, NOW - 400),
        card("fx-l.5", Stage::Landed, NOW - 500),
    ]
}

/// 既定で開く段は札の在る段のうち止まり → Blocked → Queued → Running / Gated → 着地の順で最初の段（札が無ければ None）・
/// tile を押した記録が在ればその値・開いている段を押すと畳む。
#[test]
fn bvlay_tile_opens_most_urgent() {
    assert_eq!(
        URGENT,
        [
            PipelineColumn::QuestionedFailedStopped,
            PipelineColumn::Blocked,
            PipelineColumn::Queued,
            PipelineColumn::RunningGated,
            PipelineColumn::Landed,
        ]
    );
    let all = fixture();
    let open = |n: usize| default_open(&columns(&all[..n], &[], NOW));
    assert_eq!(open(5), Some(PipelineColumn::QuestionedFailedStopped));
    assert_eq!(open(3), Some(PipelineColumn::Blocked));
    assert_eq!(open(2), Some(PipelineColumn::Queued));
    assert_eq!(open(1), Some(PipelineColumn::RunningGated));
    assert_eq!(
        default_open(&columns(&all[4..], &[], NOW)),
        Some(PipelineColumn::Landed)
    );
    assert_eq!(default_open(&columns(&[], &[], NOW)), None);
    let cols = columns(&all, &[], NOW);
    assert_eq!(
        open_of(None, &cols),
        Some(PipelineColumn::QuestionedFailedStopped)
    );
    assert_eq!(open_of(Some(None), &cols), None);
    assert_eq!(
        open_of(Some(Some(PipelineColumn::Landed)), &cols),
        Some(PipelineColumn::Landed)
    );
    assert_eq!(
        press(Some(PipelineColumn::Queued), PipelineColumn::Queued),
        None
    );
    assert_eq!(
        press(Some(PipelineColumn::Queued), PipelineColumn::Landed),
        Some(PipelineColumn::Landed)
    );
    assert_eq!(
        press(None, PipelineColumn::Blocked),
        Some(PipelineColumn::Blocked)
    );
}

/// tile は板の列の順の 5 つで、数は列の札の全部・開いている段だけ開き、Queued の tile は列に在る長さが 30 分を越えた札で注意。
#[test]
fn bvlay_tile_counts_and_warn() {
    let all = fixture();
    let cols = columns(&all, &[], NOW);
    let ts = tiles(&cols, Some(PipelineColumn::Blocked), NOW);
    let got: Vec<(PipelineColumn, usize, bool, bool)> = ts
        .iter()
        .map(|t| (t.lane.column, t.n, t.open, t.warn))
        .collect();
    assert_eq!(
        got,
        vec![
            (PipelineColumn::Blocked, 1, true, false),
            (PipelineColumn::Queued, 1, false, false),
            (PipelineColumn::RunningGated, 1, false, false),
            (PipelineColumn::QuestionedFailedStopped, 1, false, false),
            (PipelineColumn::Landed, 1, false, false),
        ]
    );
    let late = [card("fx-l.6", Stage::Queued, NOW - QUEUED_WARN_S - 1)];
    let ts = tiles(&columns(&late, &[], NOW), None, NOW);
    assert!(ts[1].warn);
    let edge = [card("fx-l.7", Stage::Queued, NOW - QUEUED_WARN_S)];
    assert!(!tiles(&columns(&edge, &[], NOW), None, NOW)[1].warn);
    // 注意は Queued の tile だけ（30 分を越えた Blocked の札では付けない）・数は列の札の全部（閉じた列が出す 3 枚を越えても）。
    let many = [
        card("fx-l.8", Stage::Blocked, NOW - QUEUED_WARN_S - 1),
        card("fx-l.9", Stage::Queued, NOW - 10),
        card("fx-l.10", Stage::Queued, NOW - 20),
        card("fx-l.11", Stage::Queued, NOW - 30),
        card("fx-l.12", Stage::Queued, NOW - 40),
    ];
    let ts = tiles(&columns(&many, &[], NOW), None, NOW);
    assert_eq!(
        (ts[0].n, ts[0].warn, ts[1].n, ts[1].warn),
        (1, false, 4, false)
    );
}

/// mod dom の中の `head` で始まる関数の字（次の 4 字下げの閉じ括弧の行まで）。
fn fn_of<'a>(dom: &'a str, head: &str) -> &'a str {
    let at = dom
        .find(head)
        .unwrap_or_else(|| panic!("pipeline の DOM に {head} が無い"));
    let rest = &dom[at..];
    &rest[..rest.find("\n    }\n").expect("関数の閉じ")]
}

/// 板の block は tile を押した記録を頁の一生の間持ち、札の在る板の後に段の tile と開いた段の札（板と同じ部品）を描く。
/// 記録の signal は view の fn の頭（板を読み直すたびに走る body の閉包の前）で 1 度だけ作り、body の閉包と
/// with_tiles と tiles_view の中では作らない・記録を読むのは開く段の閉包・書くのは tile の押しの 2 所だけ。
/// 字はそれぞれ置き場の関数の中で照らす（列の札の並びにも同じ字が在るので mod dom の全体では照らさない）。
#[test]
fn bvlay_dom_wiring_text() {
    let src = read("src/project/pipeline.rs");
    let dom = &src[src.find("mod dom {").expect("mod dom")..];
    let view = fn_of(dom, "pub fn view() -> AnyView {");
    let at = view.find("let body = move || {").expect("body の閉包");
    let (head, rest) = view.split_at(at);
    let body = &rest[..rest.find("\n        };\n").expect("body の閉包の閉じ")];
    let with = fn_of(dom, "fn with_tiles(");
    let tv = fn_of(dom, "fn tiles_view(");
    let rec = "let picked = RwSignal::new(None);";
    let filled = "Body::Filled(cols) => with_tiles(cols, open, mode, clock, (keys, picked)),";
    let after = "view! { {board_view(cols, open, mode, clock, keys)}{tiles} }.into_any()";
    let tiled: &[&str] = &[
        "let open = move || cols.with_value(|c| open_of(picked.get(), c));",
        "on:click=move |_| picked.set(Some(press(now, column)))",
        "let col = cols.with_value(|c| c.iter().find(|c| c.lane.column == now).cloned())?;",
        ".map(|c| pick_view(c, keys, kcard_view(c, mode(), clock)))",
        "<div class=\"ptiles\"><div class=\"ptrow\">{row}</div>{list}</div>",
    ];
    for (name, part, wants) in [
        ("view の頭（body の閉包の前）", head, &[rec][..]),
        ("body の閉包", body, &[filled][..]),
        ("with_tiles", with, &[after][..]),
        ("tiles_view", tv, tiled),
    ] {
        for want in wants {
            assert!(part.contains(want), "{name} に {want} が無い");
        }
    }
    assert_eq!(dom.matches(rec).count(), 1, "記録を作る字は 1 つ");
    for (name, part) in [
        ("body の閉包", body),
        ("with_tiles", with),
        ("tiles_view", tv),
    ] {
        assert!(
            !part.contains("RwSignal::new("),
            "{name} の中で signal を作る"
        );
    }
    assert_eq!(dom.matches("picked.").count(), 2, "記録の読み書きは 2 所");
    let lib = read("src/lib.rs");
    assert!(lib.contains("pub mod tiles;"));
}
