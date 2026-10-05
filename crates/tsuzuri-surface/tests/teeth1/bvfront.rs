//! 行 g-accept-face の歯（接頭辞 bvfront_）: 受入の 12 条の面の側の残り（要件 NFR1・FR14・規則の行 R-23）。
//! account board の tab の数の印を link の外へ出し、節点の頁の走行の link と止まった run の窓の個別の頁への口に hover の card を付け、帯の下の札を面の中身に
//! 重ねず、HOME の次の一手の一覧と各 project の表の要対応の列で面の決まった字を行の数だけ並べない。質問の窓は理由と推奨を
//! 「続き」に畳み、台帳の字の置き場に印 data-ledger-text を付け、台帳の一覧の群の見出しは 600 以下で折り返す。知らせの接続が
//! 頁を開いてから READ_HOLD_S 秒の間に開かなければ、口を全部「読めない」にして理由を印の card に出す（憲法 P-7.2）。稼働の記録の目盛の
//! 段の端の字は段の中へ寄せる。
#![cfg(test)]

use crate::common::read;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::graph::{AroundRow, GraphNode, NodeKind};
use tsuzuri_contract::ledger::READ_HOLD_S;
use tsuzuri_surface::account::projects::{Need, need_l2};
use tsuzuri_surface::fresh::{Fresh, LOST_SRC, UNOPENED_KEY, WARN_WORD};
use tsuzuri_surface::project::ask::{LAYOUT, MORE, MORE_KEY, Part};
use tsuzuri_surface::project::next::{NONE_LINE, UNJUDGED_LINE, row_line};
use tsuzuri_surface::project::timeline::run_card;
use tsuzuri_surface::vocab::label;
use tsuzuri_surface::widgets::nodecard::{NO_GIST, NO_STATE, card_for};
use tsuzuri_surface::wins::{Stalled, stalled_card};

/// `from` の字から次の `to` の字までの切り（`to` を含まない）。
fn between<'a>(text: &'a str, from: &str, to: &str) -> &'a str {
    let at = text.find(from).unwrap_or_else(|| panic!("{from} が無い"));
    let rest = &text[at..];
    let end = rest
        .find(to)
        .unwrap_or_else(|| panic!("{from} の後に {to} が無い"));
    &rest[..end]
}

/// `from` の字から次の `to` の字までの切り（`from` は字の中に 1 つだけ）を、改行と続く字下げを除いてつないだ字
/// （関数の中の鎖や view の子を頭から尾まで照らす）。
fn joined(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "{from}");
    between(text, from, to)
        .split('\n')
        .map(str::trim_start)
        .collect()
}

/// stylesheet の規則の列（注を除く）: 置き場（囲む at-rule の頭を空白でつないだ字・最上位は空）と選びの字と宣言の字。
fn css_rules(css: &str) -> Vec<(String, String, String)> {
    let mut text = String::new();
    let mut rest = css;
    while let Some(at) = rest.find("/*") {
        text.push_str(&rest[..at]);
        let end = rest[at..].find("*/").expect("注の閉じ");
        rest = &rest[at + end + 2..];
    }
    text.push_str(rest);
    let (mut rules, mut heads, mut from) = (Vec::new(), Vec::new(), 0);
    while let Some(off) = text[from..].find(['{', '}', ';']) {
        let at = from + off;
        let head = text[from..at]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        from = at + 1;
        match text.as_bytes()[at] {
            b'{' if head.starts_with('@') => heads.push(head),
            b'{' => {
                let end = at + text[at..].find('}').expect("規則の閉じ");
                rules.push((heads.join(" "), head, text[at + 1..end].to_string()));
                from = end + 1;
            }
            b'}' => {
                heads.pop();
            }
            _ => {}
        }
    }
    rules
}

/// 選びに class `class` を持つ規則が宣言する名 `props` の値を、stylesheet の順（cascade の順）に「置き場 { 選び { 名: 値 } }」
/// の字（最上位の規則は置き場を書かない）で返す。後に名の字が続く `.class` は別の class。
fn decls(css: &str, class: &str, props: &[&str]) -> Vec<String> {
    let dot = format!(".{class}");
    let named = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    let mut out = Vec::new();
    for (at, sel, body) in css_rules(css) {
        if !sel
            .match_indices(&dot)
            .any(|(i, _)| !sel[i + dot.len()..].starts_with(named))
        {
            continue;
        }
        for (name, value) in body.split(';').filter_map(|d| d.split_once(':')) {
            if props.contains(&name.trim()) {
                let rule = format!("{sel} {{ {}: {} }}", name.trim(), value.trim());
                out.push(if at.is_empty() {
                    rule
                } else {
                    format!("{at} {{ {rule} }}")
                });
            }
        }
    }
    out
}

/// 字 `text` の中の `pos` の位置の波括弧の深さ（関数の本文の切りなら 0 が本文の最上位）。
fn depth_at(text: &str, pos: usize) -> usize {
    let head = &text[..pos];
    head.matches('{').count() - head.matches('}').count()
}

fn row(id: &str, kind: NodeKind, title: &str, status: Option<&str>) -> AroundRow {
    AroundRow {
        node: GraphNode {
            id: id.to_string(),
            kind,
            file: None,
            digest: None,
            title: title.to_string(),
            line: None,
            plain: Some("走行の概要".to_string()),
            eng: None,
            updated: None,
        },
        status: status.map(str::to_string),
        col: 1,
        via: None,
        edge_type: None,
        degree: 1,
    }
}

/// tab の link の中は印と語の字だけで、数の印（mark）は link の閉じの直後の隣に置き、印の規則は隣の span を縦の中央に置く。
#[test]
fn bvfront_tab_badge_outside_link() {
    let board = read("src/account/board.rs");
    let link = between(&board, "<a href=move || tab_href(l.tab", "</a>");
    assert!(link.contains("<span inner_html=tab_icon(l.tab)></span>"));
    assert!(link.contains("<span class=\"lbl hd-t\">{label(l.key)}</span>"));
    assert!(!link.contains("{mark}"), "数の印が tab の link の中に在る");
    assert_eq!(
        link.matches('{').count(),
        1,
        "tab の link の中に印と語のほかの字が在る"
    );
    assert!(
        board.contains(
            "{label(l.key)}</span>\n                            </a>\n                            {mark}\n"
        ),
        "数の印が link の閉じの直後の隣に無い"
    );
    // link の開きの tag の行の後は、印の span と語の字の span と link の閉じと数の印だけ（view の閉じまで）。
    let view = between(
        &board,
        "<a href=move || tab_href(l.tab",
        "\n                        }\n",
    );
    let kids: String = view.lines().skip(1).map(str::trim).collect();
    assert_eq!(
        kids,
        "<span inner_html=tab_icon(l.tab)></span><span class=\"lbl hd-t\">{label(l.key)}</span></a>{mark}"
    );
    let css = read("style.css");
    assert!(css.contains(".nav .badge { flex: none; align-self: center; min-width: 20px;"));
    // 印の規則は最上位（どの幅でも効く）に 1 つで、flex と縦の置き場を上書きする規則は無い。
    assert_eq!(
        decls(&css, "badge", &["flex", "align-self"]),
        [
            ".nav .badge { flex: none }",
            ".nav .badge { align-self: center }"
        ]
    );
}

/// 走行の link の card は近傍の行の同じ id の節点と状態の card で、無ければ走行の id だけの節点（種類は走行・状態なし）の card。
/// id の頭が同じほかの行は引かない。時間軸の行の link は card を付ける。
#[test]
fn bvfront_run_link_card() {
    let run = "t3-hub.1/r1";
    let near = vec![
        row("t3-hub.1/r10", NodeKind::Run, "ほかの走行", Some("Failed")),
        row(run, NodeKind::Run, "走行の題", Some("Landed")),
    ];
    assert_eq!(
        run_card(&near, run),
        card_for(&near[1].node, Some("Landed"))
    );
    let card = run_card(&near, run);
    assert_eq!(card.title, "走行の題");
    assert!(card.kind.ends_with(" · Landed"), "{}", card.kind);
    let lone = run_card(&near[..1], run);
    assert_eq!(lone.title, run);
    assert!(
        lone.kind.ends_with(&format!(" · {NO_STATE}")),
        "{}",
        lone.kind
    );
    assert_eq!(lone.value, format!("{run} {NO_GIST}"));
    let bare = GraphNode {
        id: run.to_string(),
        kind: NodeKind::Run,
        file: None,
        digest: None,
        title: String::new(),
        line: None,
        plain: None,
        eng: None,
        updated: None,
    };
    assert_eq!(lone, card_for(&bare, None));
    assert_eq!(run_card(&[], run), lone);
    let timeline = read("src/project/timeline.rs");
    assert!(timeline.contains("<a class=\"nth num\" href=href use:attach=card>"));
    assert!(timeline.contains("let card = run_card_in(&around, &r.run, mode());"));
    // 近傍の行は読めた近傍の頁の行の全部（読めていなければ空）。
    assert_eq!(
        joined(&timeline, "let around = near.with(", ";\n"),
        "let around = near.with(|(f, s)| match state(f, *s) {PageState::Doc(doc) => doc.rows,_ => Vec::new(),})"
    );
}

/// 帯の下の札は高さ 19 px で下から 2 px に置き、1 枚の画面の下の余白（広い段と 601〜1199 の段の小さい方）に収まる。
/// 1 枚の画面でない頁と 600 以下の段は流れの中に置く。
#[test]
fn bvfront_fresh_tag_place() {
    let css = read("style.css");
    let fresh = between(&css, ".freshw { position: fixed;", "}");
    let px = |rule: &str, key: &str| -> u32 {
        let at = rule.find(key).unwrap_or_else(|| panic!("{key} が無い"));
        let rest = &rule[at + key.len()..];
        rest[..rest
            .find("px")
            .unwrap_or_else(|| panic!("{key} の px が無い"))]
            .trim()
            .parse()
            .unwrap_or_else(|e| panic!("{key} の数: {e}"))
    };
    let (bottom, height) = (px(fresh, "bottom:"), px(fresh, "height:"));
    assert_eq!((bottom, height), (2, 19));
    let wide = between(&css, ".one { display: grid;", "}");
    let mid = between(&css, ".one { grid-template-columns: minmax(0, 1fr);", "}");
    for one in [wide, mid] {
        let last: u32 = one
            .split("padding:")
            .nth(1)
            .and_then(|p| p.split(';').next())
            .and_then(|p| p.split_whitespace().last())
            .and_then(|p| p.trim_end_matches("px").parse().ok())
            .unwrap_or_else(|| panic!("下の余白が読めない: {one}"));
        assert!(
            bottom + height <= last,
            "札 {bottom}+{height} px が下の余白 {last} px を越える"
        );
    }
    assert!(css.contains(".freshw > .chip, .freshw > .warn { height: 17px; }"));
    assert!(css.contains(
        "body:not(:has(.one)) .freshw { position: static; width: fit-content; height: auto; margin: 6px 12px 0 auto; }"
    ));
    let narrow = between(&css, "@media (max-width: 600px) {\n  .one {", "\n}");
    assert!(narrow.contains(
        "  .freshw { position: static; width: fit-content; height: auto; margin: 6px 8px 0 auto; }"
    ));
    fresh_rules_placed(&css);
}

/// 札の規則の置き場と cascade の順: 固定の基の規則と chip と warn の 17 px と class one の要素を持たない頁の規則は最上位、
/// 600 以下の流れの中の規則は基の規則より後の 600 の塊の中。1 枚の画面の下の余白は広い段が最上位・601〜1199 と 600 以下が
/// それぞれの塊の中。
fn fresh_rules_placed(css: &str) {
    assert_eq!(
        decls(css, "freshw", &["position"]),
        [
            ".freshw { position: fixed }",
            "body:not(:has(.one)) .freshw { position: static }",
            "@media (max-width: 600px) { .freshw { position: static } }",
        ]
    );
    assert_eq!(
        decls(css, "freshw", &["height"]),
        [
            ".freshw { height: 19px }",
            ".freshw > .chip, .freshw > .warn { height: 17px }",
            "body:not(:has(.one)) .freshw { height: auto }",
            "@media (max-width: 600px) { .freshw { height: auto } }",
        ]
    );
    assert_eq!(
        decls(css, "one", &["padding"]),
        [
            ".page:has(> .one) { padding: 0 }",
            ".one { padding: 10px 12px 24px }",
            "@media (min-width: 601px) and (max-width: 1199px) { .one { padding: 6px 6px 22px } }",
            "@media (max-width: 600px) { .one { padding: 8px } }",
        ]
    );
}

/// 行の 1 行の字 row_line は、なしの箱の字と測れていないの箱の字を空にし、ほかの 1 行はそのまま。各 project の表の要対応の列の
/// 2 段目（need_l2）は 1 行を row_line に通し、1 行の無い欄は空。HOME の次の一手の一覧の行と表の 2 段目の span はこの字を出す。
#[test]
fn bvfront_need_fixed_line_hidden() {
    let need = |line: Option<&str>| Need {
        lead: None,
        key: "st_unknown",
        sev: "lo",
        line: line.map(str::to_string),
    };
    let own = "proj-a の session が限度で止まった";
    assert_eq!(row_line(NONE_LINE), "");
    assert_eq!(row_line(UNJUDGED_LINE), "");
    assert_eq!(row_line(own), own);
    assert_eq!(need_l2(&need(Some(NONE_LINE))), "");
    assert_eq!(need_l2(&need(Some(UNJUDGED_LINE))), "");
    assert_eq!(need_l2(&need(None)), "");
    assert_eq!(need_l2(&need(Some(own))), own);
    let projects = read("src/account/projects.rs");
    // 2 段目の字は need_l2 の戻りそのもので、使い道は幅への収めと span の 2 所だけ。
    assert_eq!(
        projects
            .matches("let l2_text = need_l2(&row.need);")
            .count(),
        1
    );
    assert_eq!(projects.matches("l2_text").count(), 3);
    assert!(projects.contains("let l2 = fit_node(l2_text.clone());"));
    assert!(projects.contains("<span class=\"l2\" node_ref=l2>{l2_text}</span>"));
    assert!(!projects.contains("{row.need.line.clone().unwrap_or_default()}"));
    let home = read("src/account/home.rs");
    assert!(home.contains("<span class=\"nxl\">{row_line(&r.line).to_string()}</span>"));
    assert!(!home.contains("<span class=\"nxl\">{r.line}</span>"));
}

/// 止まった run の窓の個別の頁への口の card は、札の契約の bead の節点（種類は契約・題は台帳の題・無ければ空）と段の字の状態の
/// card_for の card。窓の札の link は card を付ける。
#[test]
fn bvfront_stalled_link_card() {
    let mut s = Stalled {
        id: "t3-hub.9".to_string(),
        title: Some("止まった契約の題".to_string()),
        stage: "Failed".to_string(),
        age: "3 分".to_string(),
        reason: None,
        href: "?page=node&id=t3-hub.9".to_string(),
    };
    let node = |title: &str| GraphNode {
        id: "t3-hub.9".to_string(),
        kind: NodeKind::Task,
        file: None,
        digest: None,
        title: title.to_string(),
        line: None,
        plain: None,
        eng: None,
        updated: None,
    };
    let card = stalled_card(&s);
    assert_eq!(card, card_for(&node("止まった契約の題"), Some("Failed")));
    assert_eq!(card.title, "止まった契約の題");
    assert!(card.kind.ends_with(" · Failed"), "{}", card.kind);
    s.title = None;
    let bare = stalled_card(&s);
    assert_eq!(bare, card_for(&node(""), Some("Failed")));
    assert_eq!(bare.title, "t3-hub.9");
    let wins = read("src/wins.rs");
    assert!(wins.contains("let card = stalled_card(&c);"));
    assert!(wins.contains("<a class=\"sm\" href=c.href use:attach=card>"));
}

/// 質問の窓の card は頭と概要の後に「続き」（語の鍵 q_more・字は 続き）の開き閉じを置き、理由と推奨をその中に畳む。
/// 台帳の字の置き場（題・概要の行・理由・推奨・これまでの決定の短い題・ほかの project の問いの題）は印 data-ledger-text を持ち、
/// 題とこれまでの決定の短い題は 36 字に切って全体を title に置く。
#[test]
fn bvfront_ask_fold_and_marks() {
    assert_eq!(MORE, [Part::Reason, Part::Recommend]);
    assert_eq!((MORE_KEY, label(MORE_KEY).as_str()), ("q_more", "続き"));
    let order: Vec<Part> = LAYOUT.iter().map(|s| s.part).collect();
    let at = |p: Part| order.iter().position(|q| *q == p).unwrap_or(usize::MAX);
    assert!(
        at(Part::Summary) + 1 == at(Part::Reason) && at(Part::Reason) + 1 == at(Part::Recommend)
    );
    assert!(at(Part::Recommend) < at(Part::Answer));
    let ask = read("src/project/ask.rs");
    assert!(
        ask.contains(
            "<details class=\"fold\" prop:open=open on:toggle=toggle><summary>{label(MORE_KEY)}</summary>{more}</details>"
        )
    );
    assert!(ask.contains("let more = LAYOUT.iter().filter(folded).map(part).collect_view();"));
    ask_fold_order(&ask);
    assert!(
        ask.contains("let (open, toggle) = fold(format!(\"ask:more:{}\", card.id), || false);")
    );
    assert_eq!(ask.matches("data-ledger-text=\"\"").count(), 5);
    assert_eq!(
        ask.matches("<span data-t=\"\" data-ledger-text=\"\" title=title.clone()>{hover::clip(&title)}</span>").count(),
        2
    );
    for want in [
        "<span data-t=\"\" data-ledger-text=\"\">{card.reason.clone()}</span>",
        "<span data-t=\"\" data-ledger-text=\"\">{card.recommend.clone()}</span>",
        "<span data-t=\"\" data-ledger-text=\"\">{l.text.clone()}</span>",
    ] {
        assert!(ask.contains(want), "ask.rs に {want} が無い");
    }
    let hist = read("src/project/askpage.rs");
    assert!(hist.contains(
        "<span class=\"ttl\" title=item.title.clone() data-t=\"\" data-ledger-text=\"\">{crate::widgets::hover::clip(&short)}</span>"
    ));
    let win = read("src/askwin.rs");
    assert!(win.contains("<span class=\"ttl\" data-ledger-text=\"\">{t}</span>"));
}

/// 質問の窓の card の描く順: 頭と概要（畳む部分の前）・「続き」の開き閉じ（畳む部分）・答え（畳む部分の後で、畳む部分を
/// 除く）の順で article の子に置く。
fn ask_fold_order(ask: &str) {
    assert_eq!(
        joined(ask, "let folded = |slot: &&Slot|", ";\n"),
        "let folded = |slot: &&Slot| MORE.contains(&slot.part)"
    );
    assert_eq!(
        joined(ask, "let lead = LAYOUT", ";\n"),
        "let lead = LAYOUT.iter().take_while(|s| !folded(s)).map(part).collect_view()"
    );
    assert_eq!(
        joined(ask, "let tail = LAYOUT", ";\n"),
        "let tail = LAYOUT.iter().skip_while(|s| !folded(s)).filter(|s| !folded(s)).map(part).collect_view()"
    );
    let article = between(ask, "<article class=card_class(target)", "</article>");
    let kids: String = article.lines().skip(1).map(str::trim).collect();
    assert_eq!(
        kids,
        "{lead}<details class=\"fold\" prop:open=open on:toggle=toggle><summary>{label(MORE_KEY)}</summary>{more}</details>{tail}"
    );
}

/// 台帳の一覧の群の見出しは 600 以下の段で折り返し、群の名は全体を title に置く。
#[test]
fn bvfront_group_head_wraps() {
    let css = read("style.css");
    let narrow = between(&css, "@media (max-width: 600px) {\n  .one {", "\n}");
    assert!(narrow.contains("  .ll-gh { flex-wrap: wrap; row-gap: 2px; }"));
    // 折り返しを決める規則はその 1 つだけ（基の規則や後の規則が折り返しを上書きしない）。
    assert_eq!(
        decls(&css, "ll-gh", &["flex-wrap"]),
        ["@media (max-width: 600px) { .ll-gh { flex-wrap: wrap } }"]
    );
    let list = read("src/ledgerlist.rs");
    assert!(list.contains(
        "<button type=\"button\" class=\"ll-gname\" title=g.name.clone() on:click=choose>"
    ));
    assert!(list.contains("<span class=\"ll-gname\" title=g.name.clone()>{g.name.clone()}</span>"));
}

/// 頁を開いた時刻（偽の接続の時計の起点）。
const PAGE_AT: EpochSecs = 1000;

/// 偽の知らせの接続: 頁を開き、`open` の時刻に開いたの event を受け（None なら来ない）、頁を開いてから READ_HOLD_S 秒の
/// timer を撃つ（net の `connect` と `link_back` と `link_unopened` の順）。戻りは timer が全部「読めない」にしたかと古さ。
fn fake_link(open: Option<EpochSecs>) -> (bool, Fresh) {
    let mut fresh = Fresh::default();
    let fire = PAGE_AT + READ_HOLD_S;
    if open.is_some_and(|at| at < fire) {
        fresh.back();
    }
    let lost = fresh.unopened(PAGE_AT, fire);
    if open.is_some_and(|at| at >= fire) {
        fresh.back();
    }
    (lost, fresh)
}

/// 知らせの接続が開かないまま READ_HOLD_S 秒が過ぎる偽の接続では、timer が口を全部「読めない」にし、読み込み不良の印の
/// card の種類に理由（語の鍵 link_unopened・字は 知らせの接続が開かない）を出す。READ_HOLD_S 秒の前は待つ。
/// net は頁を開いた時刻から READ_HOLD_S 秒の timer を張り、開いていなければ全部「読めない」にする。
#[test]
fn bvfront_unopened_link_fails() {
    assert_eq!(
        (UNOPENED_KEY, label(UNOPENED_KEY).as_str()),
        ("link_unopened", "知らせの接続が開かない")
    );
    let fire = PAGE_AT + READ_HOLD_S;
    let mut wait = Fresh::default();
    assert!(!wait.unopened(PAGE_AT, fire - 1));
    assert_eq!(wait, Fresh::default());

    let (lost, mut stuck) = fake_link(None);
    assert!(lost);
    assert!(stuck.warn(fire));
    assert_eq!(stuck.age(fire), Some(READ_HOLD_S));
    let card = stuck.card(fire).expect("開かない接続の card");
    assert_eq!(card.title, WARN_WORD);
    assert_eq!(card.kind, "知らせの接続が開かない");
    assert_eq!(card.src, LOST_SRC);
    stuck.lose(fire + 1);
    let both = stuck.card(fire + 2).expect("切れも在る card");
    assert_eq!(both.kind, "知らせのつながり・知らせの接続が開かない");
    assert_eq!(both.src, LOST_SRC);

    let net = read("src/net.rs");
    let unopened = between(&net, "fn link_unopened(at: EpochSecs) {", "\n}\n");
    assert!(unopened.contains("if change(|f| f.unopened(at, now())) {\n        lose_all();"));
    let connect = between(&net, "fn connect() {", "\n}\n");
    assert!(connect.contains("        return;\n    }\n    let at = now();\n"));
    assert!(
        connect.contains(
            "    set_timeout(move || link_unopened(at), Duration::from_secs(READ_HOLD_S));"
        )
    );
    connect_timer_placed(connect);
}

/// connect の時刻と timer は本文の最上位（枝や閉包の中でない）で、timer は接続を作れない枝の後に張る（作れた時も撃つ）。
fn connect_timer_placed(connect: &str) {
    let timer = "set_timeout(move || link_unopened(at),";
    assert_eq!(connect.matches("link_unopened").count(), 1);
    let at = |s: &str| {
        connect
            .find(s)
            .unwrap_or_else(|| panic!("connect に {s} が無い"))
    };
    for s in [
        "let at = now();",
        "let Ok(source) = EventSource::new(EVENTS_PATH) else {",
        timer,
    ] {
        assert_eq!(depth_at(connect, at(s)), 1, "{s}");
    }
    assert!(at("let at = now();") < at("let Ok(source)") && at("};\n    let on_open") < at(timer));
}

/// 偽の接続が READ_HOLD_S 秒の前に開けば、今と同じに読む（timer は何もしない・古さに何も置かない）。後で開けば理由を消す。
/// net は開いたの event で最初の読みを許し（go_live）、古さに開いたを置く（link_back）。
#[test]
fn bvfront_opened_link_reads() {
    let fire = PAGE_AT + READ_HOLD_S;
    let mut opened = Fresh::default();
    opened.back();
    for at in [PAGE_AT, PAGE_AT + 1, fire - 1] {
        let (lost, f) = fake_link(Some(at));
        assert!(!lost, "{at} に開いた");
        assert_eq!(f, opened);
        assert_eq!(f.card(fire + 100), None);
    }
    let (late, back) = fake_link(Some(fire + 3));
    assert!(late);
    assert_eq!(back.card(fire + 4), None);
    assert_eq!(back.age(fire + 4), None);

    let net = read("src/net.rs");
    let connect = between(&net, "fn connect() {", "\n}\n");
    assert!(connect.contains("let on_open = Closure::<dyn FnMut()>::new(go_live);"));
    assert!(between(&net, "fn link_back() {", "\n}\n").contains("change(Fresh::back);"));
    // 2 つの閉包は接続の開いたの event に登録し（onopen と open の受け手）、頁の一生の間持つ（forget）。
    for want in [
        "let on_back = Closure::<dyn FnMut()>::new(link_back);",
        "source.set_onopen(Some(on_open.as_ref().unchecked_ref()));",
        ".add_event_listener_with_callback(\"open\", on_back.as_ref().unchecked_ref())",
        "on_open.forget();",
        "on_back.forget();",
    ] {
        assert_eq!(connect.matches(want).count(), 1, "connect の {want}");
    }
    assert_eq!(
        (
            connect.matches("on_open").count(),
            connect.matches("on_back").count()
        ),
        (3, 3)
    );
}

/// 稼働の記録の目盛の段（class saxis）の字は目盛の位置の中央に置き、最初の字は目盛から右へ、最後の字は目盛から左へ
/// 寄せて段の外へはみ出さない（字を隠さない）。段の子は目盛の字の span だけ（account の session と project の席の block）。
#[test]
fn bvfront_axis_ends_inside() {
    let css = read("style.css");
    let rules = [
        ".saxis span { position: absolute; top: 0; transform: translateX(-50%); white-space: nowrap; }\n",
        ".saxis span:first-child { transform: translateX(0); }\n",
        ".saxis span:last-child { transform: translateX(-100%); }\n",
    ];
    let at: Vec<usize> = rules
        .iter()
        .map(|r| {
            css.find(r)
                .unwrap_or_else(|| panic!("stylesheet に {r} が無い"))
        })
        .collect();
    assert!(at[0] < at[1] && at[1] < at[2], "{at:?}");
    for r in rules {
        assert_eq!(css.matches(r).count(), 1, "{r}");
    }
    let saxis = css
        .lines()
        .filter(|l| l.starts_with(".saxis "))
        .collect::<Vec<_>>();
    assert!(saxis.iter().all(|l| !l.contains("overflow")), "{saxis:?}");
    // 3 つの規則は最上位（どの幅でも効く）にこの順で、class saxis を選びに持つどの規則（塊の中も・祖先の class を
    // 頭に持つ選びも）も overflow を持たない。
    assert_eq!(
        decls(&css, "saxis", &["transform"]),
        [
            ".saxis span { transform: translateX(-50%) }",
            ".saxis span:first-child { transform: translateX(0) }",
            ".saxis span:last-child { transform: translateX(-100%) }",
        ]
    );
    assert_eq!(
        decls(&css, "saxis", &["overflow", "overflow-x", "overflow-y"]),
        Vec::<String>::new()
    );
    let seat = read("src/project_dom/seat.rs");
    assert!(seat.contains("<div class=\"saxis\" aria-hidden=\"true\">{axis}</div>"));
    assert!(seat.contains("view! { <span style=style>{t.label}</span> }"));
    let sess = read("src/account/session.rs");
    assert!(sess.contains(
        "<div class=SAXIS aria-hidden=\"true\">\n                {move || {\n                    axis_labels(at, span.get())"
    ));
    assert!(sess.contains(".map(|(style, label)| view! { <span style=style>{label}</span> })"));
    axis_children(&seat, &sess);
}

/// 目盛の段の子を組む閉包は、目盛の字の span の view を 1 つだけ持ち、その列をそのまま返す（ほかの要素を足さない）。
fn axis_children(seat: &str, sess: &str) {
    let ticks = joined(seat, "    let axis = move || {\n", "\n    };\n");
    assert!(
        ticks.matches("view!").count() == 1
            && ticks.ends_with("view! { <span style=style>{t.label}</span> }}).collect_view()"),
        "{ticks}"
    );
    let labels = joined(sess, "<div class=SAXIS aria-hidden=\"true\">", "</div>");
    assert!(
        labels.matches("view!").count() == 1
            && labels.ends_with(
                ".map(|(style, label)| view! { <span style=style>{label}</span> }).collect_view()}}"
            ),
        "{labels}"
    );
}
