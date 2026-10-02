//! 行 g-next-act の歯: 次の一手の大きい箱の、種類ごとの次の手の頁への link（href と字・mode・開く窓）・
//! href の読み直し・Big の対象の id・DOM が link を組む字・link の字と見出しの語・歯の名。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::NextMove;
use tsuzuri_contract::stats::NextStep;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::windows::ACCOUNT_WIN;
use tsuzuri_surface::account::{Tab, selects};
use tsuzuri_surface::frame::{Mode, PageId, param};
use tsuzuri_surface::mapview::decode;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::topbar::Win;
use tsuzuri_surface::project::next::{
    ACCOUNT_LINK, ANSWER_LINK, BATCH_LINK, GAPS_LINK, Link, NONE_LINE, Next, PIPE_LINK,
    SESSION_LINK, action, content, window_of,
};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

const MODES: [Mode; 2] = [Mode::Beginner, Mode::Expert];

fn link(href: String, text: &'static str) -> Option<Link> {
    Some(Link { href, text })
}

/// (1) 2 つの mode と 7 種で、次の手の頁への link は節の表の href と字（止まっている走行となしは None）。
#[test]
fn nxact_links_per_kind() {
    for mode in MODES {
        let m = mode.key();
        let cases: [(NextMove, Option<&str>, Option<Link>); 7] = [
            (
                NextMove::LimitOrMove,
                None,
                link(format!("?board=account&tab=home&mode={m}"), ACCOUNT_LINK),
            ),
            (
                NextMove::Unresponsive,
                None,
                link(format!("?board=account&tab=session&mode={m}"), SESSION_LINK),
            ),
            (NextMove::StalledRun, Some("nx.7"), None),
            (
                NextMove::BatchApproval,
                None,
                link(format!("?mode={m}&win=ask"), BATCH_LINK),
            ),
            (
                NextMove::Question,
                Some("nq.4"),
                link(format!("?mode={m}&win=ask&id=nq.4"), ANSWER_LINK),
            ),
            (
                NextMove::AwaitingEffect,
                None,
                link(format!("?mode={m}&win=gaps"), GAPS_LINK),
            ),
            (NextMove::Nothing, None, None),
        ];
        for (kind, target, want) in cases {
            assert_eq!(action(kind, target, mode), want, "{kind:?} {m}");
        }
        // 質問は対象の id が無ければ質問の窓・id のコロンは %3A にする（行 g-one-screen-a）。
        assert_eq!(
            action(NextMove::Question, None, mode),
            link(format!("?mode={m}&win=ask"), ANSWER_LINK)
        );
        assert_eq!(
            action(NextMove::Question, Some("e.2:x"), mode),
            link(format!("?mode={m}&win=ask&id=e.2%3Ax"), ANSWER_LINK)
        );
    }
    // 開く窓は account board へ飛ぶ 2 種だけ account board の名前つきの窓。
    assert_eq!(ACCOUNT_WIN, "tz-account");
    for kind in NextMove::ALL {
        let want = match kind {
            NextMove::LimitOrMove | NextMove::Unresponsive => Some(ACCOUNT_WIN),
            _ => None,
        };
        assert_eq!(window_of(kind), want, "{kind:?}");
    }
    // 字は節の字。
    assert_eq!(ACCOUNT_LINK, "account board を見る ›");
    assert_eq!(SESSION_LINK, "session を見る ›");
    assert_eq!(BATCH_LINK, "まとめて承認 ›");
    assert_eq!(ANSWER_LINK, "答える ›");
    assert_eq!(GAPS_LINK, "orchestrator に任せる（見るだけ） ›");
}

/// (2) href の井桁より前の字を読み直すと、同じ行き先・同じ対象・渡した mode に戻る。
#[test]
fn nxact_hrefs_read_back() {
    enum Dest {
        Account(Tab),
        Win(Win),
    }
    for mode in MODES {
        let cases: [(NextMove, Option<&str>, Dest, Option<&str>); 7] = [
            (NextMove::LimitOrMove, None, Dest::Account(Tab::Home), None),
            (
                NextMove::Unresponsive,
                None,
                Dest::Account(Tab::Session),
                None,
            ),
            (NextMove::BatchApproval, None, Dest::Win(Win::Ask), None),
            (
                NextMove::Question,
                Some("nq.4"),
                Dest::Win(Win::Ask),
                Some("nq.4"),
            ),
            (
                NextMove::Question,
                Some("e.2:x"),
                Dest::Win(Win::Ask),
                Some("e.2:x"),
            ),
            (NextMove::Question, None, Dest::Win(Win::Ask), None),
            (
                NextMove::AwaitingEffect,
                None,
                Dest::Win(Win::Gaps),
                None,
            ),
        ];
        for (kind, target, dest, want_id) in cases {
            let l = action(kind, target, mode).unwrap_or_else(|| panic!("{kind:?} の link"));
            let search = l.href.split('#').next().expect("井桁より前");
            match dest {
                Dest::Account(tab) => {
                    assert!(selects(search), "{search} が account board を選ばない");
                    assert_eq!(Tab::from_query(search), tab, "{search}");
                }
                Dest::Win(win) => {
                    assert!(!selects(search), "{search}");
                    assert_eq!(PageId::from_query(search), PageId::Home, "{search}");
                    assert_eq!(Win::from_query(search), Some(win), "{search}");
                }
            }
            assert_eq!(
                param(search, "id").map(decode).as_deref(),
                want_id,
                "{search} の対象"
            );
            assert_eq!(Mode::from_query(search), mode, "{search} の mode");
        }
    }
}

/// fixture の 4 組（組の名 → 電文の字）。
fn fixture() -> BTreeMap<String, String> {
    let sets: BTreeMap<String, NextStep> =
        wire::decode(&read("../../tests/fixtures/surface/next-step.json"))
            .expect("fixture の組が電文として読める");
    sets.into_iter()
        .map(|(name, s)| (name, wire::encode(&s).expect("電文")))
        .collect()
}

fn filled(name: &str) -> Next {
    let text = fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"));
    match content(&Fetched::Body(text)) {
        Body::Filled(n) => n,
        other => panic!("組 {name} が中身を出さない: {other:?}"),
    }
}

/// (3) Big は lead の種類の項の対象の id を字で持ち、ほかの欄は今のまま。
#[test]
fn nxact_big_carries_target() {
    let pipe = link("#pipe".to_string(), PIPE_LINK);
    let cases = [
        (
            "stalled",
            Some("nx.7"),
            NextMove::StalledRun,
            "nx_c",
            "nxbig",
            "what",
            "2 件",
            pipe,
        ),
        (
            "question",
            Some("nq.4"),
            NextMove::Question,
            "nx_e",
            "nxbig",
            "what",
            "nq.4 · 1 件",
            None,
        ),
        (
            "nothing",
            None,
            NextMove::Nothing,
            "nx_g",
            "nxbig none",
            "what small muted",
            NONE_LINE,
            None,
        ),
        (
            "not-judged",
            None,
            NextMove::BatchApproval,
            "nx_d",
            "nxbig",
            "what",
            "4 件",
            None,
        ),
    ];
    for (name, target, kind, key, class, what_class, what, want_link) in cases {
        let big = filled(name).big;
        assert_eq!(big.target.as_deref(), target, "{name} の対象");
        assert_eq!(big.kind, kind, "{name} の種類");
        assert_eq!(big.key, key, "{name} の語の鍵");
        assert_eq!(big.class, class, "{name} の箱の class");
        assert_eq!(big.what_class, what_class, "{name} の中身の class");
        assert_eq!(big.what, what, "{name} の中身の字");
        assert_eq!(big.link, want_link, "{name} の link");
    }
}

/// (4) DOM（mod dom から file の終わり）は mode を context か URL から読み、action と window_of で link を組む。
#[test]
fn nxact_src_uses_action() {
    let src = read("src/project/next.rs");
    let at = src.find("mod dom {").expect("mod dom が在る");
    let dom = &src[at..];
    for word in [
        "action(",
        "window_of(",
        "use_context::<HelpCtx>()",
        "Mode::from_query(",
    ] {
        assert!(dom.contains(word), "mod dom に {word} が無い");
    }
    // action と window_of の定義は mod dom より前（wasm の target に限らない）。
    let head = &src[..at];
    assert!(head.contains("pub fn action("));
    assert!(head.contains("pub fn window_of("));
}

/// (5) link の字の 5 つは、語の辞書の日本語の見出しの語のどれとも同じ字でない。
#[test]
fn nxact_texts_not_headings() {
    let words: Vec<String> = vocab()
        .keys()
        .filter_map(|k| vocab().term(k))
        .map(|t| t.label.clone())
        .filter(|l| !l.is_ascii())
        .collect();
    assert!(!words.is_empty());
    for text in [ACCOUNT_LINK, SESSION_LINK, BATCH_LINK, ANSWER_LINK, GAPS_LINK] {
        assert!(
            words.iter().all(|w| w != text),
            "{text} が見出しの語と同じ"
        );
    }
}

/// 着地済みの行と同じ波の行の verify の filter の語（65 語）。
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
    "afocus_",
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

/// (6) この file の歯の名は nxact_ で始まり、残りの字は着地済みと同じ波の filter の語を含まない。
#[test]
fn nxact_own_names_clean() {
    let src = read("tests/nextact.rs");
    let lines: Vec<&str> = src.lines().map(str::trim).collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if *line != "#[test]" {
            continue;
        }
        let fn_line = lines[i + 1..]
            .iter()
            .find(|l| l.starts_with("fn "))
            .expect("test の属性の次に fn が在る");
        let name = fn_line["fn ".len()..]
            .split('(')
            .next()
            .expect("fn の名")
            .to_string();
        names.push(name);
    }
    assert!(names.len() >= 6, "歯の名 {names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("nxact_")
            .unwrap_or_else(|| panic!("{name} が nxact_ で始まらない"));
        for f in FILTERS {
            assert!(!rest.contains(f), "{name} が filter の語 {f} を含む");
        }
    }
}
