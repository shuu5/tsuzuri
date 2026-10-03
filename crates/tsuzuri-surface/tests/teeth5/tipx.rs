//! 行 g-tip-expert の歯: 語の鍵を持たない要素の経験者だけの注釈（見本の `data-tip-expert`・help の expert_note）・
//! まとめて承認の重なりの chip の字（batch の OVERLAP_TIP）・抜けの検査の各行の見出しの字（gaps の summary_tip）・
//! 注釈の層と 2 つの block の DOM の字の並び。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::graph::Verdict;
use tsuzuri_surface::account::home::EXPERT_CHARS;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::{batch, gaps};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::widgets::help::{Inline, expert_note, inline};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn text(s: &str) -> Inline {
    Inline::Text(s.to_string())
}

/// 字の数（Unicode の scalar）。
fn chars(s: &str) -> usize {
    s.chars().count()
}

/// 字 `mod dom {` の最初の所より前と後。
fn split_dom(src: &str) -> (&str, &str) {
    let i = src.find("mod dom {").expect("mod dom の字");
    src.split_at(i)
}

/// 経験者の mode のときだけ、空でない行を行ごとに片に分ける。
#[test]
fn tipx_expert_note_rules() {
    assert_eq!(expert_note("a", Mode::Beginner), None);
    assert_eq!(expert_note("", Mode::Expert), None);
    assert_eq!(expert_note("\n\n", Mode::Expert), None);
    assert_eq!(
        expert_note("a · b", Mode::Expert),
        Some(vec![vec![text("a · b")]])
    );
    let two = "x\n\ny `c` {gi:ok}";
    assert_eq!(
        expert_note(two, Mode::Expert),
        Some(vec![
            vec![text("x")],
            vec![
                text("y "),
                Inline::Code("c".to_string()),
                text(" "),
                Inline::Sym("gi:ok".to_string()),
            ],
        ])
    );
    // 行の片は inline と同じ分け方。
    assert_eq!(
        expert_note(two, Mode::Expert).map(|l| l[1].clone()),
        Some(inline("y `c` {gi:ok}"))
    );
    for s in ["a", "x\n\ny", "a · b", two] {
        assert_eq!(expert_note(s, Mode::Beginner), None, "{s:?}");
    }
}

/// 重なりの chip の字は見本の字のまま・経験者の 1 行の字数に収まる。
#[test]
fn tipx_overlap_tip_text() {
    assert_eq!(
        batch::OVERLAP_TIP,
        "選んだ質問の touches の重なり（衝突の兆し）"
    );
    assert_eq!(
        expert_note(batch::OVERLAP_TIP, Mode::Expert),
        Some(vec![vec![text(batch::OVERLAP_TIP)]])
    );
    assert_eq!(expert_note(batch::OVERLAP_TIP, Mode::Beginner), None);
    assert_eq!(EXPERT_CHARS, 60);
    assert!(chars(batch::OVERLAP_TIP) <= EXPERT_CHARS);
}

/// 各行の見出しの字は id と判定の名（見本の gaps.html の pass・fail・unknown）。
#[test]
fn tipx_gaps_summary_lines() {
    let names: Vec<(Verdict, &str)> = Verdict::ALL
        .iter()
        .map(|v| (*v, gaps::verdict_name(*v)))
        .collect();
    assert_eq!(names.len(), 3);
    for (v, name) in names {
        let want = match v {
            Verdict::Pass => "pass",
            Verdict::Violation => "fail",
            Verdict::Unknown => "unknown",
        };
        assert_eq!(name, want, "{v:?}");
    }
    let fetched = Fetched::Body(read("../../tests/fixtures/surface/invariants.json"));
    let checks = gaps::checks(&fetched).expect("fixture を判定の列に読む");
    let rows = gaps::rows(&checks);
    let tips: Vec<String> = rows.iter().map(gaps::summary_tip).collect();
    assert_eq!(
        tips,
        [
            "g-2 · fail",
            "g-10 · fail",
            "g-3 · unknown",
            "g-9 · unknown",
            "g-12 · unknown",
            "g-1 · pass",
            "g-4 · pass",
            "g-5 · pass",
            "g-6 · pass",
            "g-7 · pass",
            "g-8 · pass",
            "g-11 · pass",
        ]
    );
    for (row, tip) in rows.iter().zip(&tips) {
        assert_eq!(
            *tip,
            format!("{} · {}", row.id, gaps::verdict_name(row.verdict))
        );
        assert!(chars(tip) <= EXPERT_CHARS, "{tip}");
        assert_eq!(
            expert_note(tip, Mode::Expert),
            Some(vec![vec![text(tip)]]),
            "{tip}"
        );
    }
}

/// 注釈の層は経験者だけの注釈を語の鍵の注釈より先に内部の段で描き、2 つの block は要素に付ける。
#[test]
fn tipx_dom_text() {
    let help = read("src/widgets/help.rs");
    let (host, dom) = split_dom(&help);
    let start = host.find("pub use dom::{").expect("pub use dom の列");
    let list = &host[start..];
    let list = &list[..list.find("};").expect("列の終わり")];
    assert!(list.contains("expert_tip"), "{list}");

    let start = dom.find("pub fn expert_tip(").expect("expert_tip の宣言");
    let body = &dom[start..];
    let body = &body[..body.find("\n    }\n").expect("expert_tip の終わり")];
    for word in [
        "expert_note(",
        "\"mouseenter\"",
        "\"mouseleave\"",
        "expert: Some(",
    ] {
        assert!(body.contains(word), "expert_tip に {word} が無い");
    }

    let layer = &dom[dom.find("fn TipLayer(").expect("TipLayer")..];
    let ex = layer.find("open.expert").expect("open.expert");
    let key = layer.find("note_in(open.key").expect("note_in(open.key");
    assert!(ex < key, "open.expert が note_in(open.key より後");
    let between = &layer[ex..key];
    assert!(between.contains("expert_note("), "{between}");
    assert!(between.contains("class=\"int\""), "{between}");

    let src = read("src/project/gaps.rs");
    let (_, dom) = split_dom(&src);
    assert!(dom.contains("<summary use:expert_tip=tip>"));
    assert!(dom.contains("summary_tip(&r)"));

    let src = read("src/project/batch.rs");
    let (_, dom) = split_dom(&src);
    assert!(dom.contains("use:expert_tip=OVERLAP_TIP.to_string()><span inner_html=WARN></span>"));
}

/// 歯の名はどれも tipx_ で始まり、先の歯の filter の語を含まない。
#[test]
fn tipx_names_stay_apart() {
    let text = read("tests/teeth5/tipx.rs");
    let mut names = Vec::new();
    let mut rest = text.as_str();
    // 属性の行と次の行の `fn `（この字の literal は逆斜線の escape で改行を持たず当たらない）。
    let mark = "\n#[test]\nfn ";
    while let Some(i) = rest.find(mark) {
        let after = &rest[i + mark.len()..];
        let end = after.find('(').expect("fn の名の終わり");
        names.push(after[..end].trim().to_string());
        rest = &after[end..];
    }
    assert!(names.len() >= 5, "{names:?}");
    names_avoid(names, &[words_front(), words_back()].concat());
}

/// 先の歯の filter の語の前半（43 語）。
fn words_front() -> &'static [&'static str] {
    &[
        "accept_",
        "account_",
        "acctcore_",
        "acctdoc_",
        "acctframe_",
        "accthb_",
        "accthome_",
        "acctled_",
        "acctlook_",
        "acctpcore_",
        "acctproj_",
        "acctsess_",
        "acctwin_",
        "acctwire_",
        "askcard_",
        "batchpanel_",
        "board_min_",
        "contract_form_",
        "frame_",
        "gapspage_",
        "gquestion_",
        "graph_",
        "gview_",
        "hbconf_",
        "hbpost_",
        "hbproc_",
        "hbroute_",
        "hook_",
        "hsblock_",
        "hsderive_",
        "hspage_",
        "ledgerblock_",
        "mapgraph_",
        "mapview_",
        "nextstep_",
        "nodepage_",
        "parts_",
        "pipe_",
        "project_",
        "question_",
        "seatblock_",
        "seatcard_",
        "server_",
    ]
}

/// 先の歯の filter の語の後半（49 語）。
fn words_back() -> &'static [&'static str] {
    &[
        "skeleton_",
        "stage_",
        "stats_",
        "steady_",
        "topbar_",
        "tz_",
        "mlink_",
        "gnav_",
        "mkeys_",
        "klink_",
        "ilink_",
        "afocus_",
        "nxact_",
        "urpanel_",
        "saxis_",
        "ticker_",
        "hfig_",
        "pmore_",
        "lspark_",
        "apop_",
        "brand_",
        "runsdoc_",
        "nbatch_",
        "kcli_",
        "hcard_",
        "qgate_",
        "nsum_",
        "ntime_",
        "ptitle_",
        "ncard_",
        "hsym_",
        "smore_",
        "lcard_",
        "aaround_",
        "hruling_",
        "sxaxis_",
        "plimit_",
        "bport_",
        "fstop_",
        "nsumw_",
        "fmark_",
        "fserve_",
        "cadopt_",
        "hcsess_",
        "hcproj_",
        "sesplit_",
        "mstore_",
        "athr_",
        "qblock_",
    ]
}

/// 歯の名の残りの字が先の歯の filter の語のどれも含まない。
fn names_avoid(names: Vec<String>, words: &[&str]) {
    assert_eq!(words.len(), 92);
    for name in &names {
        let rest = name
            .strip_prefix("tipx_")
            .unwrap_or_else(|| panic!("{name} が tipx_ で始まらない"));
        for w in words {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
