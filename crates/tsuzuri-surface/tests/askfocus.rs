//! 便 g-ask-focus の歯: URL の `?id=` で名指された問いを読む・card の id と class・名指しの card の番号・
//! これまでの決定の段が名指しの項で開くか・問いの頁の 2 つの module の DOM の字（題の link と寄せる振る舞い）・
//! この file の歯の名が着地済みの行の filter の語を含まない。

use std::path::PathBuf;

use tsuzuri_surface::project::{Body, ask, askpage};
use tsuzuri_surface::view::Fetched;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn question_list() -> Fetched {
    Fetched::Body(read("../../tests/fixtures/surface/question-list.json"))
}

fn ledger_questions() -> Fetched {
    Fetched::Body(read("../../tests/fixtures/surface/ledger-questions.json"))
}

/// 字 `from` が最初に在る所から file の終わりまで（無ければ落ちる）。
fn tail<'a>(text: &'a str, from: &str, module: &str) -> &'a str {
    let at = text
        .find(from)
        .unwrap_or_else(|| panic!("{module} に字 {from} が無い"));
    &text[at..]
}

/// `?id=` の値は `%XX` を戻した字・無いか空なら None。
#[test]
fn afocus_id_param_read() {
    assert_eq!(ask::FOCUS_KEY, "id");
    assert_eq!(
        ask::focus("?page=ask&id=nq.4&mode=beginner"),
        Some("nq.4".to_string())
    );
    assert_eq!(
        ask::focus("?page=ask&mode=expert&id=e.2%3A20260927T0000Z-1"),
        Some("e.2:20260927T0000Z-1".to_string())
    );
    assert_eq!(ask::focus("?page=ask&mode=beginner"), None);
    assert_eq!(ask::focus("?page=ask&id=&mode=beginner"), None);
}

/// card の id は q と 0 から数えた位置・class は名指しの card だけ target を足す。
#[test]
fn afocus_card_anchor_class() {
    assert_eq!(ask::anchor(1), "q0");
    assert_eq!(ask::anchor(2), "q1");
    assert_eq!(ask::card_class(true), "qcard target");
    assert_eq!(ask::card_class(false), "qcard");
}

/// fixture の card で qa.10 は番号 2・一覧に無い id は None（答え済み）。
#[test]
fn afocus_target_card_number() {
    let Body::Filled(cards) = ask::body(&question_list()) else {
        panic!("fixture の card が中身にならない");
    };
    assert_eq!(ask::target_number(&cards, "qa.10"), Some(2));
    assert_eq!(ask::target_number(&cards, "qa.2"), Some(1));
    assert_eq!(ask::target_number(&cards, "lq.9"), None);
    assert_eq!(ask::anchor(2), "q1");
}

/// これまでの決定の段は名指しの問いが項に在れば開く（まだ読んでいない読みと項に無い id と名指しなしは閉じる）。
#[test]
fn afocus_rulings_fold_opens() {
    let fetched = ledger_questions();
    assert!(askpage::opens(&fetched, Some("lq.9")));
    assert!(!askpage::opens(&fetched, Some("lq.3")));
    assert!(!askpage::opens(&fetched, None));
    assert!(!askpage::opens(&Fetched::NotRead, Some("lq.9")));
    const { assert!(!askpage::OPEN, "これまでの決定の段は最初は閉じている") };
    let Body::Filled(items) = askpage::body(&fetched) else {
        panic!("これまでの決定が中身を出さない");
    };
    assert_eq!(askpage::focus_index(&items, "lq.9"), Some(1));
    assert_eq!(askpage::focus_index(&items, "lq.3"), None);
    assert_eq!(askpage::row_selector(1), "#hist ul.items > li:nth-child(2)");
    assert_eq!(askpage::HIGHLIGHT, "background:var(--panel-2)");
}

/// 問いの card の DOM は題を節点の頁への link にして名指しの card を寄せ、段の DOM は名指しの行を光らせて寄せる。
#[test]
fn afocus_src_links_and_scroll() {
    let module = "src/project/ask.rs";
    let text = read(module);
    let dom = tail(&text, "mod dom {", module);
    for want in [
        "anchor(",
        "card_class(",
        "node_href(",
        "focus(",
        "target_number(",
        "get_element_by_id(",
        "scroll_into_view_with_bool(true)",
        "<a class=\"t\"",
    ] {
        assert!(dom.contains(want), "{module} の mod dom に {want} が無い");
    }
    assert!(
        !text.contains("<span class=\"t\""),
        "{module} に題の span が残る"
    );

    let module = "src/project/askpage.rs";
    let text = read(module);
    let view = tail(&text, "pub fn view(", module);
    for want in [
        "opens(",
        "focus(",
        "focus_index(",
        "row_selector(",
        "query_selector(",
        "set_attribute(",
        "HIGHLIGHT",
        "scroll_into_view_with_bool(true)",
    ] {
        assert!(view.contains(want), "{module} の view に {want} が無い");
    }
}

/// 着地済みの行と同じ波の行の verify の filter の語（65）。
const FILTERS: [&str; 65] = [
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
    "hook_",
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
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "hbproc_",
    "hbconf_",
    "hbroute_",
    "hbpost_",
    "hsblock_",
    "hspage_",
    "hsderive_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
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
];

/// この file の test の fn の名は afocus_ で始まり、残りの字は filter の語を含まない。
#[test]
fn afocus_own_names_clean() {
    let text = read("tests/askfocus.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if *line != "#[test]" {
            continue;
        }
        let name = lines[i + 1..]
            .iter()
            .find_map(|l| l.strip_prefix("fn "))
            .and_then(|l| l.split('(').next())
            .unwrap_or_else(|| panic!("{} 行目の test の属性の次に fn が無い", i + 1));
        names.push(name);
    }
    assert_eq!(names.len(), 6, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("afocus_")
            .unwrap_or_else(|| panic!("{name} が afocus_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
