//! 行 g-ledger-trim の歯（接頭辞 bvtrim_・判断の記録 ADR-27 決定 (8)・要件 FR13）: 台帳の block は見出しの横の判定の
//! 1 語と台帳 open の一覧だけを描き、前の指標の段（上段の 4 数・主な指標・burndown・memo の段）を持たない。一覧の見出しの
//! 14 日の図は burndown の svg で burndown の card を付け、未反映の段は帯の抜けの検査の窓の下へ移す。読み直しの表は
//! bead の事実の口を台帳の合図で、器の局面の出力の口を台帳と event log の合図で読み直す。
//! DOM は host で撃てないので、src の字と xtask の surface-build で組めることで見る。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::{LedgerStats, UnreflectedKind, UnreflectedList};
use tsuzuri_contract::surface::ChangeKind;
use tsuzuri_contract::{case, wire};
use tsuzuri_surface::ledgerlist::{FACTS_PATH, Phases, kpi, kpi_unknown};
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::ledger::{
    self, BURN_H, BURN_W, Unref, burn_svg, burndown, unref_list, unref_shown, unref_stale,
};
use tsuzuri_surface::view::{Fetched, RELOAD_KINDS, path_kinds, reloads};

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の指標の組 filled。
fn filled() -> LedgerStats {
    let mut sets: BTreeMap<String, LedgerStats> =
        wire::decode(&read("../../tests/fixtures/surface/ledger-stats.json"))
            .expect("fixture の組が電文として読める");
    sets.remove("filled").expect("fixture の組 filled")
}

/// src の字の mod dom から後ろ。
fn dom_of(rel: &str) -> String {
    let text = read(rel);
    let at = text.find("mod dom {").unwrap_or_else(|| panic!("{rel} に mod dom が無い"));
    text[at..].to_string()
}

/// `text` の中の `head` から、その後の最初の `close` の手前までの字（関数や束縛の本文を切る）。
fn cut<'a>(text: &'a str, head: &str, close: &str) -> &'a str {
    let at = text.find(head).unwrap_or_else(|| panic!("{head} が無い"));
    let rest = &text[at..];
    &rest[..rest.find(close).unwrap_or_else(|| panic!("{head} の閉じが無い"))]
}

/// stylesheet の注を外した字。
fn uncomment(css: &str) -> String {
    let mut plain = String::new();
    let mut rest = css;
    while let Some(at) = rest.find("/*") {
        plain.push_str(&rest[..at]);
        rest = rest[at..].find("*/").map_or("", |e| &rest[at + e + 2..]);
    }
    plain + rest
}

/// stylesheet の葉の規則（中に塊を持たない規則）の、波括弧の深さ（最上位は 0）と選びの字と宣言の字。
fn leaf_rules(css: &str) -> Vec<(usize, String, String)> {
    let plain = uncomment(css);
    let (mut out, mut stack, mut start) = (Vec::new(), Vec::<(String, usize, bool)>::new(), 0);
    for (i, c) in plain.char_indices() {
        if c == '{' {
            if let Some(top) = stack.last_mut() {
                top.2 = true;
            }
            let sel = plain[start..i].rsplit(';').next().unwrap_or("").trim();
            stack.push((sel.to_string(), i + 1, false));
            start = i + 1;
        } else if c == '}' {
            let (sel, open, nested) = stack.pop().expect("閉じの多い stylesheet");
            if !nested {
                out.push((stack.len(), sel, plain[open..i].trim().to_string()));
            }
            start = i + 1;
        }
    }
    assert!(stack.is_empty(), "閉じの足りない stylesheet");
    out
}

/// 宣言の字が幅か高さ（width・height と min と max）を決めるか。
fn sizes(decls: &str) -> bool {
    decls.split(';').any(|d| {
        let prop = d.split(':').next().unwrap_or("").trim();
        ["width", "height", "min-width", "max-width", "min-height", "max-height"].contains(&prop)
    })
}

/// (2) 台帳の block の DOM は見出しの横に指標の口が読めた時だけ判定の 1 語を置き、中身は一覧の view を 1 度だけ描く。
/// module は前の指標の段の型と表と関数と class の字を持たない。字は block の view の関数の中で照らし、一覧の view を
/// 呼ぶ section は関数の末の式（閉包の中で読み直すたびに作り直さない）・判定の語は 1 つ・section は判定を header の
/// 見出しの横に置き中身をその下に置く。
#[test]
fn bvtrim_block_draws_list_only() {
    let src = read("src/project/ledger.rs");
    let dom = dom_of("src/project/ledger.rs");
    let view = cut(&dom, "pub fn view() -> AnyView {", "\n    }\n");
    for want in [
        "let fetched = crate::net::read(METRICS_PATH);\n        let extra",
        "let extra = move || fetched.with(|f| stats(f).ok().map(|s| judge_view(judge(s.judge))));",
        "section(BLOCK, extra.into_any(), crate::ledgerlist::view())",
    ] {
        assert_eq!(view.matches(want).count(), 1, "{want}");
    }
    assert_eq!(dom.matches("crate::ledgerlist::view()").count(), 1);
    assert!(
        view.ends_with("\n        section(BLOCK, extra.into_any(), crate::ledgerlist::view())"),
        "{view}"
    );
    assert_eq!(view.matches("move ||").count(), 1, "{view}");
    assert!(!view.contains("unref"), "{view}");
    let judge = cut(&dom, "fn judge_view(j: Judge) -> AnyView {", "\n    }\n");
    assert_eq!(judge.matches("{label(").count(), 1, "{judge}");
    let kit = read("src/kit.rs");
    let section = cut(&kit, "pub fn section(block: Block, extra: AnyView, body: AnyView)", "\n    }\n");
    assert!(
        section.contains("<header>{h2(block.heading)}{extra}</header>\n                {body}\n"),
        "{section}"
    );
    assert_eq!(section.matches("{extra}").count(), 1, "{section}");
    for gone in [
        "pub enum Part",
        "pub enum Tier",
        "LAYOUT",
        "pub struct Metrics",
        "pub struct MemoRow",
        "pub struct EpicBar",
        "fn panel(",
        "fn content(",
        "fn metrics(",
        "fn epic_cards(",
        "BURN_CAPTION",
        "class=\"l4\"",
        "class=\"lmid\"",
        "class=\"mpro\"",
        "class=\"lcap",
        "screen.with(count)",
    ] {
        assert!(!src.contains(gone), "src/project/ledger.rs に {gone} が在る");
    }
}

/// (3) 一覧の見出しの 14 日の図は指標の電文の 14 日の burndown の svg（棒 14 本・線は open）で、sparkline でない。
/// 図の tag は tabindex と指標の電文の burndown の card を持ち、図の class の大きさは 120 と 24 の画素。
#[test]
fn bvtrim_head_figure_burndown() {
    let s = filled();
    let body = Fetched::Body(wire::encode(&Reading::Known(s.clone())).expect("enc"));
    let (_, svg) = kpi(&body);
    let svg = svg.expect("図が在る");
    assert_eq!(svg, burn_svg(&burndown(&s.days, BURN_W, BURN_H)));
    assert!(svg.starts_with("<svg class=\"lburn\""), "{svg}");
    assert_eq!(svg.matches("<rect class=\"lb-closed\"").count(), 14);
    assert!(svg.contains("<polyline class=\"lb-open\""), "{svg}");
    assert!(!svg.contains("lspark"), "{svg}");
    assert_eq!(kpi(&Fetched::Failed), (kpi_unknown(), None));

    let dom = dom_of("src/ledgerlist.rs");
    let head = cut(&dom, "fn head_line(kind: RwSignal<Kind>, query: RwSignal<String>)", "\n    }\n");
    for want in [
        "<span class=\"ll-spark\" tabindex=\"0\" use:attach_some=card inner_html=s></span>",
        "let card = metrics.with(|m| stats(m).ok().map(|s| burn_card(&s)));",
        "view! { <span class=\"ll-kchip num\">{text}</span>{unref}{fig} }",
        "<div class=\"ll-l1\">{kpis}</div>",
    ] {
        assert_eq!(head.matches(want).count(), 1, "{want}");
    }
    let view = cut(&dom, "pub fn view() -> AnyView {", "\n    }\n");
    let want = "view! { {head_line(kind, query)}<div class=\"ll-body\">{unread}{list}</div> }.into_any()";
    assert!(view.ends_with(&format!("\n        {want}")), "見出しが一覧の上に無い");
    let css = read("style.css");
    assert_eq!(
        css.matches("#ledger .lburn { display: block; width: 120px; height: 24px; }")
            .count(),
        1
    );
    assert!(!css.contains("#ledger .lburn { display: block; width: 100%; height: 72px; }"));
    assert_eq!(css.matches(".lburn {").count(), 1, "図の大きさの規則が 1 行でない");
    burn_size_rule(&css);
}

/// (3) 図の大きさを決める規則（選びに lburn か ll-spark か、台帳の svg を持つ規則で幅か高さを決めるもの）は、
/// stylesheet の最上位（media などの塊の外）の #ledger .lburn の 1 つだけ。
fn burn_size_rule(css: &str) {
    let burn: Vec<(usize, String, String)> = leaf_rules(css)
        .into_iter()
        .filter(|(_, sel, decls)| {
            let svg = sel.contains("svg") && (sel.contains("#ledger") || sel.contains(".ll-"));
            (sel.contains("lburn") || sel.contains("ll-spark") || svg) && sizes(decls)
        })
        .collect();
    assert_eq!(burn.len(), 1, "{burn:?}");
    assert_eq!((burn[0].0, burn[0].1.as_str()), (0, "#ledger .lburn"), "{burn:?}");
}

/// (4) 未反映の段の数は指標の電文の数そのままで、分からない種類は名の列。抜けの検査の窓は抜けの検査の中身の後に
/// 未反映の段を描き、段は畳みの鍵 ledger:unref と、数の chip と測れていないの印と分からない種類の chip を持つ。
#[test]
fn bvtrim_unref_moves_to_gaps_win() {
    let mut s = filled();
    let u = Unref::of(&s);
    assert_eq!((u.count, u.unknown.clone()), (3, vec!["ruling", "utterance"]));
    assert!(u.partial());
    s.unreflected = 41;
    s.unreflected_unknown = UnreflectedKind::ALL.to_vec();
    let all = Unref::of(&s);
    assert_eq!(all.count, 41);
    assert_eq!(all.unknown, vec!["memo", "ruling", "utterance"]);
    assert_eq!(all.text(), ledger::NONE);

    let wins = read("src/wins.rs");
    let want = "Win::Gaps => view! { {gaps::inner()}{ledger::unref_panel()} }.into_any(),";
    assert_eq!(wins.matches(want).count(), 1, "{want}");
    let dom = dom_of("src/project/ledger.rs");
    let panel = cut(&dom, "pub fn unref_panel() -> AnyView {", "\n    }\n");
    assert_eq!(dom.matches("pub fn unref_panel() -> AnyView {").count(), 1);
    for want in [
        "metrics.with(|f| stats(f).ok().map(|s| Unref::of(&s)))",
        "let partial = u.partial().then(|| state_icon(UNKNOWN));",
        "<span class=unref_chip(u.count)>{u.text()}</span>\n                    {partial}\n                    <span class=\"lchips\">{kinds}</span>",
        "<summary>{hs(\"unref\")}{chips}</summary>",
        "let metrics = crate::net::read(METRICS_PATH);\n        let unref = crate::net::read(UNREF_PATH);",
        "let (open, toggle) = fold(\"ledger:unref\".to_string(), || UNREF_OPEN);",
    ] {
        assert_eq!(panel.matches(want).count(), 1, "{want}");
    }
    assert_eq!(ledger::FOLDS, ["ledger:unref"]);
    assert_eq!(ledger::PATHS, [ledger::PATH, ledger::METRICS_PATH, ledger::UNREF_PATH]);
    unref_case_states();
    unref_bindings(panel);
}

/// (4) 未反映の段の束縛: 一覧の閉包 list は局面の出力の周で分けた一覧だけを、古いの閉包 old は古いの 1 行だけを描き、
/// 段は old を list の前に置く。
fn unref_bindings(panel: &str) {
    let list = cut(panel, "let list = move || {", "\n        };\n");
    let old = cut(panel, "let old = move || {", "\n        };\n");
    assert!(list.contains("unref_shown(") && !list.contains("unref_stale"), "{list}");
    assert!(old.contains(".with(unref_stale)") && !old.contains("unref_shown"), "{old}");
    assert_eq!(panel.matches("{old}\n                {list}").count(), 1, "{panel}");
}

/// (4) の局面の出力の 4 つの周（要件 FR13）: 3 種とも分からない一覧は、出力がまだ無い周は UNREF_UNKNOWN、読めない版の
/// 周は UNREF_UNREADABLE で字が違い、読めた一覧はどちらの周でもそのまま、古さの印の在る一覧は「古い」と印の種類の 1 行を持つ。
fn unref_case_states() {
    let unknown = UnreflectedList {
        memos: Reading::Unknown,
        rulings: Reading::Unknown,
        utterances: Reading::Unknown,
        stale: Vec::new(),
    };
    let body = Fetched::Body(wire::encode(&unknown).expect("電文"));
    let none = unref_shown(unref_list(&body, 0), &Phases::Unknown);
    assert_eq!(none, Body::Unmeasured(ledger::UNREF_UNKNOWN));
    let bad = unref_shown(unref_list(&body, 0), &Phases::Unreadable);
    assert_eq!(bad, Body::Unmeasured(ledger::UNREF_UNREADABLE));
    assert_ne!(none, bad);
    assert_eq!(unref_stale(&body), None);
    let old = UnreflectedList {
        memos: Reading::Known(Vec::new()),
        rulings: Reading::Known(Vec::new()),
        utterances: Reading::Known(Vec::new()),
        stale: vec!["lifecycle-old".to_string(), "input-moved".to_string()],
    };
    let body = Fetched::Body(wire::encode(&old).expect("電文"));
    let empty = Body::Empty(ledger::UNREF_EMPTY);
    assert_eq!(unref_shown(unref_list(&body, 0), &Phases::Unreadable), empty);
    assert_eq!(unref_shown(unref_list(&body, 0), &Phases::Unknown), empty);
    assert_eq!(
        unref_stale(&body).as_deref(),
        Some("古い（lifecycle-old・input-moved）")
    );
    assert_eq!(unref_stale(&Fetched::Failed), None);
    let dom = dom_of("src/project/ledger.rs");
    let panel = cut(&dom, "pub fn unref_panel() -> AnyView {", "\n    }\n");
    for want in [
        "let cases = crate::net::read(case::PATH);",
        "let shown = unref_shown(\n                unref_list(&unref.get(), crate::net::now()),\n                &cases.with(phases),\n            );",
        ".with(unref_stale)\n                .map(|t| view! { <div class=\"small muted\">{t}</div> })",
        "{old}\n                {list}",
    ] {
        assert_eq!(panel.matches(want).count(), 1, "{want}");
    }
}

/// (5) 読み直しの表は 15 行で、bead の事実の口は台帳だけ、器の局面の出力の口は台帳と event log の合図で読み直す
/// （表に無い口の全部の種類でない）。席と設計と account と知らせの合図ではどちらも読み直さない。
#[test]
fn bvtrim_reload_beads_and_cases() {
    assert_eq!(RELOAD_KINDS.len(), 15);
    assert_eq!(FACTS_PATH, "/api/beads");
    assert_eq!(case::PATH, "/api/cases");
    assert_eq!(path_kinds(FACTS_PATH), [ChangeKind::Ledger]);
    assert_eq!(path_kinds(case::PATH), [ChangeKind::Ledger, ChangeKind::Runs]);
    assert!(reloads(case::PATH, &[ChangeKind::Runs]));
    assert!(!reloads(FACTS_PATH, &[ChangeKind::Runs]));
    for kind in [
        ChangeKind::Seat,
        ChangeKind::Design,
        ChangeKind::Account,
        ChangeKind::Notice,
    ] {
        assert!(!reloads(FACTS_PATH, &[kind]), "{kind:?}");
        assert!(!reloads(case::PATH, &[kind]), "{kind:?}");
    }
    for path in [FACTS_PATH, case::PATH] {
        assert_eq!(
            RELOAD_KINDS.iter().filter(|(p, _)| *p == path).count(),
            1,
            "{path}"
        );
    }
}
