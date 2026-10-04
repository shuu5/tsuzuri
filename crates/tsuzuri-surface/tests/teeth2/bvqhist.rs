//! 行 g-ask-hist の歯（接頭辞 bvqhist_・判断の記録 ADR-27 決定 (4)・見本 board-v2 の qModal の folds の末）: 質問の窓の下の
//! 畳みの末に「これまでの決定 N ›」を置き、答え済みの問いを台帳の更新の時刻の新しい順に、時刻・短い題・答えた決定・
//! 問いの id の行で出す。中身は今のこれまでの決定の段の関数（rulings・hist_rows・ruling_text）を使い、2 つ目を書かない。
//! DOM は host で撃てないので、src の字と xtask の surface-build で組めることで見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{EdgeType, GraphDoc, GraphEdge};
use tsuzuri_contract::ledger::{
    BeadFact, BeadFacts, BeadId, LedgerList, LedgerRow, POLICY_SCOPE_LABEL, QUESTION_LABEL,
};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::askpage::{self, Decision, decisions, fold_title};
use tsuzuri_surface::project::{Body, LEDGER_UNREAD, NOT_READ};
use tsuzuri_surface::view::Fetched;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn id(s: &str) -> BeadId {
    BeadId::new(s).expect("bead の id")
}

fn row(name: &str, kind: &str, status: &str, at: u64, labels: &[&str]) -> LedgerRow {
    LedgerRow {
        id: id(name),
        kind: kind.to_string(),
        title: format!("題 {name}"),
        status: status.to_string(),
        updated_at: at,
        parent: None,
        labels: labels.iter().map(|l| (*l).to_string()).collect(),
    }
}

fn ledger(rows: Vec<LedgerRow>) -> Fetched {
    Fetched::Body(
        wire::encode(&LedgerList {
            rows: Reading::Known(rows),
        })
        .expect("電文"),
    )
}

/// 答え済みの問い 3（q.2 と q.3 は同じ時刻）・open の問い・方針の問い・閉じた task。
fn mixed() -> Fetched {
    let policy = format!("{POLICY_SCOPE_LABEL}all");
    ledger(vec![
        row("q.2", "task", "closed", 300, &[QUESTION_LABEL]),
        row("q.10", "task", "closed", 500, &[QUESTION_LABEL]),
        row("q.3", "task", "closed", 300, &[QUESTION_LABEL]),
        row("q.4", "task", "open", 900, &[QUESTION_LABEL]),
        row("q.5", "task", "closed", 950, &[QUESTION_LABEL, &policy]),
        row("t.1", "task", "closed", 990, &[]),
    ])
}

fn fact(name: &str, short: &str) -> BeadFact {
    BeadFact {
        id: id(name),
        created_at: None,
        short: short.to_string(),
        short_set: true,
        blocks: Vec::new(),
    }
}

/// q.10 と q.2 は metadata の短い題を持ち、q.3 は題から機械で作った字の bead の事実。
fn beads() -> Fetched {
    Fetched::Body(
        wire::encode(&BeadFacts {
            rows: Reading::Known(vec![
                fact("q.10", "十の短い題"),
                fact("q.2", "二の短い題"),
                BeadFact {
                    short_set: false,
                    ..fact("q.3", "問い")
                },
            ]),
        })
        .expect("電文"),
    )
}

/// graph-doc.json に q.10 へ答えた決定の辺を 1 本足した電文。
fn graph() -> Fetched {
    let mut doc: GraphDoc =
        wire::decode(&read("../../tests/fixtures/surface/graph-doc.json")).expect("fixture の電文");
    doc.edges.push(GraphEdge {
        from: "q.10:20261001T1300Z-1".to_string(),
        to: "q.10".to_string(),
        edge_type: EdgeType::Answers,
    });
    Fetched::Body(wire::encode(&doc).expect("電文"))
}

fn filled(b: Body<Vec<Decision>>) -> Vec<Decision> {
    match b {
        Body::Filled(ds) => ds,
        other => panic!("これまでの決定が中身にならない: {other:?}"),
    }
}

/// 新しい順（台帳の更新の時刻の大きい順・同じなら id の自然な順の逆）で、open の問い・方針の問い・task は出さず、
/// 時刻は台帳の updated_at、短い題は bead の事実の short（metadata の short か題から機械で作った字・事実が無い時は
/// 台帳の題）、答えた決定は answers の辺の from。
#[test]
fn bvqhist_newest_first() {
    let ds = filled(decisions(&mixed(), &graph(), &beads()));
    let got: Vec<(&str, u64, &str, Option<&str>)> = ds
        .iter()
        .map(|d| {
            (
                d.entry.item.id.as_str(),
                d.at,
                d.short.as_str(),
                d.entry.ruling.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            ("q.10", 500, "十の短い題", Some("q.10:20261001T1300Z-1")),
            ("q.3", 300, "問い", None),
            ("q.2", 300, "二の短い題", None),
        ]
    );
    assert_eq!(ds[0].entry.item.title, "題 q.10");
    assert_eq!(askpage::count(&mixed()), Reading::Known(3));
    // 事実の無い問いは台帳の題。
    let only = Fetched::Body(
        wire::encode(&BeadFacts {
            rows: Reading::Known(vec![fact("q.10", "十の短い題")]),
        })
        .expect("電文"),
    );
    let shorts: Vec<String> = filled(decisions(&mixed(), &graph(), &only))
        .into_iter()
        .map(|d| d.short)
        .collect();
    assert_eq!(shorts, ["十の短い題", "題 q.3", "題 q.2"]);
    tie_by_natural_id();
}

/// (4) の縁: 同じ時刻の問いは id の自然な順の逆（q.10 が q.9 の前・字の順の逆なら q.9 が前）。
fn tie_by_natural_id() {
    let tie = ledger(vec![
        row("q.9", "task", "closed", 700, &[QUESTION_LABEL]),
        row("q.10", "task", "closed", 700, &[QUESTION_LABEL]),
    ]);
    let ids: Vec<String> = filled(decisions(&tie, &graph(), &beads()))
        .into_iter()
        .map(|d| d.entry.item.id)
        .collect();
    assert_eq!(ids, ["q.10", "q.9"]);
}

/// 台帳が読めない時と 0 件の時は block の中身と同じ理由と 1 行、事実とグラフの口が読めなければ台帳の題で決定は無い。
/// 畳みの見出しは 語 これまでの決定 と件数と ›（測れていなければ件数を出さない）。
#[test]
fn bvqhist_reads_and_title() {
    let none = Fetched::NotRead;
    assert_eq!(
        decisions(&none, &graph(), &beads()),
        Body::Unmeasured(NOT_READ)
    );
    assert_eq!(
        decisions(&Fetched::Failed, &graph(), &beads()),
        Body::Unmeasured(LEDGER_UNREAD)
    );
    let open_only = ledger(vec![row("q.4", "task", "open", 900, &[QUESTION_LABEL])]);
    assert_eq!(
        decisions(&open_only, &graph(), &beads()),
        Body::Empty(askpage::EMPTY)
    );
    let ds = filled(decisions(&mixed(), &none, &none));
    assert!(ds.iter().all(|d| d.entry.ruling.is_none()));
    assert_eq!(ds[0].short, "題 q.10");
    assert_eq!(fold_title(Reading::Known(3)), "これまでの決定 3 ›");
    assert_eq!(fold_title(Reading::Unknown), "これまでの決定 ›");
}

/// `src` の中の `head` で始まる関数の字（`close` の閉じの行まで）。
fn fn_of<'a>(src: &'a str, head: &str, close: &str) -> &'a str {
    let at = src.find(head).unwrap_or_else(|| panic!("{head} が無い"));
    let rest = &src[at..];
    &rest[..rest.find(close).unwrap_or_else(|| panic!("{head} の閉じ"))]
}

/// 質問の窓の frame は窓を開いた link の問いの id を folds に渡し、folds は下の段の div の直下の末（全体への指示の
/// 畳みの後・ほかの畳みの details の外・後ろは段の終わりまで空白だけ）に askpage の inner をその id のまま描く。
fn window_folds() {
    let win = read("src/askwin.rs");
    let frame = fn_of(&win, "pub fn frame(", "\n    }\n");
    assert!(
        frame.contains("let focus = use_context::<AskFocus>().and_then(|f| f.0.get_untracked());")
    );
    assert!(frame.contains("{folds(w, focus)}"));
    let folds = fn_of(
        &win,
        "fn folds(w: Walk, focus: Option<String>) -> AnyView {",
        "\n    }\n",
    );
    let div = &folds[folds.find("<div class=\"folds\">").expect("下の段")..];
    let div = &div[..div.find("</div>").expect("下の段の終わり")];
    let p = div.find("{policy::inner()}").expect("全体への指示の段");
    let hist = "{askpage::inner(focus)}";
    let h = div.find(hist).expect("これまでの決定の段");
    assert!(p < h, "これまでの決定の段が全体への指示の段の前");
    let (before, after) = div.split_at(h);
    assert_eq!(
        before.matches("<details").count(),
        before.matches("</details>").count(),
        "これまでの決定の段がほかの畳みの details の中"
    );
    assert!(
        after[hist.len()..].trim().is_empty(),
        "これまでの決定の段が下の段の末でない"
    );
    assert_eq!(
        folds.matches("focus").count(),
        2,
        "問いの id を inner へそのまま渡さない"
    );
}

/// 質問の窓は下の畳みの末（全体への指示の段の後）に askpage の inner を窓を開いた link の問いの id で描き、
/// inner は 3 つの口を読んで台帳・グラフ・bead の事実の順で decisions に渡し、見出しを読み直すたびに数え直す閉包で
/// details の fold を描き、行を時刻・短い題・答えた決定・問いの id の順に出し、block の view も inner を描く。
/// 字はそれぞれの関数（frame・folds・view・inner・hist_li）の中で照らす。
#[test]
fn bvqhist_window_fold_wiring_text() {
    window_folds();
    let page = read("src/project/askpage.rs");
    let view = fn_of(
        &page,
        "pub fn view() -> leptos::prelude::AnyView {",
        "\n}\n",
    );
    assert!(view.contains("inner(super::ask::focus(&search))"));
    let inner = fn_of(
        &page,
        "pub fn inner(target: Option<String>) -> leptos::prelude::AnyView {",
        "\n}\n",
    );
    for needle in [
        "let graph = crate::net::read(super::map::PATH);",
        "let beads = crate::net::read(crate::ledgerlist::FACTS_PATH);",
        "let fetched = crate::net::read(super::ledger::PATH);",
        "fetched.with(|l| graph.with(|g| beads.with(|b| decisions(l, g, b))))",
        "let title = move || fold_title(fetched.with(count));",
        "<summary>{title}</summary>",
        "let now = crate::net::now();",
        ".map(|d| hist_li(d, &cards, now, mode))",
        "<details class=\"fold\" id=BLOCK.id prop:open=open on:toggle=toggle>",
    ] {
        assert!(
            inner.contains(needle),
            "askpage.rs の inner に {needle} が無い"
        );
    }
    let li = fn_of(&page, "fn hist_li(", "\n}\n");
    let order = [
        "<time>{clock_short(at, now)}</time>",
        "<span class=\"ttl\" title=item.title.clone() data-t=\"\" data-ledger-text=\"\">{crate::widgets::hover::clip(&short)}</span>",
        "<b class=\"aside\">",
        "{ruling_text(&rid)}",
        "<a class=\"lk\" href=href use:attach_some=card><code>{item.id.clone()}</code></a>",
    ]
    .map(|n| {
        li.find(n)
            .unwrap_or_else(|| panic!("行の DOM に {n} が無い"))
    });
    assert!(order.is_sorted(), "行の並びが違う: {order:?}");
}

/// 行の時刻の規則は stylesheet の質問の窓の塊の中で media の塊の外（波括弧の深さ 0）に在り、縮まず小さい字。
/// 同じ選び手の規則はその 1 つだけ（後の規則や media の中の規則が上書きしない）。
#[test]
fn bvqhist_time_rule() {
    let css = read("style.css");
    let block = css.find("/* 質問の窓（行 g-ask-win").expect("質問の窓の塊");
    let next = block + 1 + css[block + 1..].find("/* ").expect("次の塊");
    let rule = ".folds .items > li > time { flex: none; color: var(--ink-3); font-size: var(--fs-1); font-variant-numeric: tabular-nums; }";
    let at = css.find(rule).expect("時刻の規則");
    assert!(block < at && at < next, "時刻の規則が質問の窓の塊の外");
    let head = &css[..at];
    assert_eq!(
        head.matches('{').count(),
        head.matches('}').count(),
        "時刻の規則が media の塊の中"
    );
    assert_eq!(
        css.matches(".folds .items > li > time").count(),
        1,
        "時刻の規則の選び手は 1 つ"
    );
}
