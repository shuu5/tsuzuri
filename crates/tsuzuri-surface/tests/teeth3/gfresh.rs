//! 行 g-fresh の歯: 中身の古さ（fresh の Fresh）の決め方と、net と 2 つの board がそれを呼ぶ字の並び。
//! 台帳の読みが落ちたときと知らせのつながりが切れたときは最後に読めた中身を出し続け、
//! 古さが READ_WARN_S を越えれば上端の帯に読み込み不良の印を 1 つ出す（規則の行 R-21・要件 NFR2）。
#![cfg(test)]

use std::path::PathBuf;

use crate::common::function;
use tsuzuri_contract::ledger::{READ_AGE_HEADER, READ_HOLD_S, READ_WARN_S};
use tsuzuri_surface::fresh::{
    Fresh, HELD_POLL_MS, HELD_WORD, LOST_SRC, LOST_WORD, WARN_WORD, read_age,
};
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::hover::ROW_CHARS;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// card の値の字（中身の古さの秒と測れていないにする秒）。
fn value(age: u64) -> String {
    format!("中身の古さ {age} 秒・60 秒で測れていない")
}

/// (1) 頭の値の読みと定数。
#[test]
fn gfresh_age_value_and_constants() {
    assert_eq!(HELD_POLL_MS, 5000u64);
    assert_eq!(WARN_WORD, "読み込み不良");
    assert_eq!(HELD_WORD, "台帳の読み");
    assert_eq!(LOST_WORD, "知らせのつながり");
    assert_eq!(LOST_SRC, "SSE");
    assert_eq!(READ_AGE_HEADER, "X-Tz-Read-Age");
    assert_eq!(READ_WARN_S, 15u64);
    assert_eq!(READ_HOLD_S, 60u64);

    assert_eq!(read_age(Some("0")), Some(0));
    assert_eq!(read_age(Some("23")), Some(23));
    assert_eq!(read_age(Some(" 59 ")), Some(59));
    for bad in ["", " ", "-1", "1.5", "abc", "+3", "１"] {
        assert_eq!(read_age(Some(bad)), None, "{bad:?}");
    }
    assert_eq!(read_age(None), None);
}

/// (2) 頭の在る応答は最後に読めた時刻を持ち、頭の無い応答で外す。最終の記録はその最も古い値。
#[test]
fn gfresh_held_then_fresh() {
    let mut f = Fresh::default();
    assert_eq!(f.record(), None);
    assert_eq!(f.age(1000), None);
    assert!(!f.warn(1000));
    assert!(!f.held());

    f.got("/p/a", 1000, None);
    assert_eq!(f.record(), Some(1000));
    assert_eq!(f.age(1000), None);
    f.got("/p/a", 1010, None);
    assert_eq!(f.record(), Some(1010), "同じ本文でも進む");

    f.got("/p/a", 1020, Some(4));
    assert!(f.held());
    assert_eq!(f.record(), Some(1016));
    assert_eq!(f.age(1020), Some(4));
    assert!(!f.warn(1031), "15 秒ちょうどは出さない");
    assert!(f.warn(1032));
    other_paths(f);
}

/// ほかの口の頭が最終の記録を最も古い値にし、頭の無い応答で外れることを見る。
fn other_paths(mut f: Fresh) {
    f.got("/p/b", 1022, Some(8));
    assert_eq!(f.record(), Some(1014));
    f.got("/p/c", 1025, None);
    assert_eq!(f.record(), Some(1014));
    f.got("/p/b", 1026, None);
    assert_eq!(f.record(), Some(1016));
    assert_eq!(f.age(1026), Some(10));

    f.got("/p/a", 1030, None);
    assert!(!f.held());
    assert_eq!(f.record(), Some(1030));
    assert_eq!(f.age(1030), None);
    assert!(!f.warn(1100));
}

/// (3) つながりの切れは最初の誤りの時刻を持ち、開いたで消す（最終の記録は動かさない）。
#[test]
fn gfresh_link_lost_and_back() {
    let mut f = Fresh::default();
    f.got("/p/a", 1990, None);
    assert!(f.lose(2000));
    assert!(!f.lose(2005));
    assert_eq!(f.lost_since(), Some(2000));
    assert_eq!(f.record(), Some(1990));
    assert_eq!(f.age(2015), Some(15));
    assert!(!f.warn(2015));
    assert!(f.warn(2016));

    f.got("/p/b", 2010, Some(20));
    assert_eq!(f.age(2016), Some(26));
    f.back();
    assert_eq!(f.lost_since(), None);
    assert_eq!(f.age(2016), Some(26));

    f.got("/p/b", 2017, None);
    assert_eq!(f.age(2017), None);
    assert!(!f.warn(2100));
    assert!(f.lose(2200));
    assert_eq!(f.lost_since(), Some(2200));
}

/// (4) 見ていない口の古さは数えない。
#[test]
fn gfresh_forget_path() {
    let mut f = Fresh::default();
    f.got("/p/a", 100, Some(30));
    f.got("/p/b", 100, Some(10));
    f.forget("/p/a");
    assert_eq!(f.record(), Some(90));
    assert_eq!(f.age(100), Some(10));
    f.forget("/p/b");
    f.forget("/p/x");
    assert!(!f.held());
    assert_eq!(f.age(100), None);
    assert_eq!(f.record(), None);
}

/// (5) 印の card は印を出す間だけ在り、種類と出所は読みの落ちとつながりの切れの順につなぐ。
#[test]
fn gfresh_card_rows() {
    let mut f = Fresh::default();
    f.got("/p/a", 1000, Some(8));
    assert_eq!(f.card(1007), None);
    let card = f.card(1015).expect("23 秒の card");
    assert_eq!(card.title, WARN_WORD);
    assert_eq!(card.kind, HELD_WORD);
    assert_eq!(card.value, value(23));
    assert_eq!(card.src, READ_AGE_HEADER);
    assert!(card.more.is_empty());
    lost_cards(f, card);
}

/// 切れを足した card と切れだけの card の種類と出所、全部の card の行の字数を見る。
fn lost_cards(mut f: Fresh, card: tsuzuri_surface::widgets::hover::Card) {
    f.lose(1010);
    let both = f.card(1100).expect("108 秒の card");
    assert_eq!(both.kind, format!("{HELD_WORD}・{LOST_WORD}"));
    assert_eq!(both.value, value(108));
    assert_eq!(both.src, format!("{READ_AGE_HEADER}・{LOST_SRC}"));

    let mut g = Fresh::default();
    g.lose(500);
    let lost = g.card(517).expect("17 秒の card");
    assert_eq!(lost.kind, LOST_WORD);
    assert_eq!(lost.src, LOST_SRC);
    assert_eq!(lost.value, value(17));

    for c in [&card, &both, &lost] {
        for (_, row) in c.rows() {
            assert!(row.chars().count() <= ROW_CHARS, "{row}");
        }
    }
}

/// (6) 印の語は語の辞書の見出しの語でない（file の定数の字）。
#[test]
fn gfresh_words_not_headings() {
    let v = vocab();
    for key in v.keys() {
        let heading = &v.term(key).expect("鍵の語").label;
        for word in [WARN_WORD, HELD_WORD, LOST_WORD] {
            assert_ne!(heading, word, "鍵 {key} の見出しの語が {word}");
        }
    }
}

/// (7) net は読めた応答の頭を古さに置き、つながりの切れを READ_HOLD_S 秒待ってから全部「読めない」にする（字の並び）。
#[test]
fn gfresh_net_text() {
    let net = read("src/net.rs");
    let has = |name: &str, words: &[&str]| {
        let body = function(&net, name);
        for w in words {
            assert!(body.contains(w), "{name} に {w} が無い: {body}");
        }
    };
    has(
        "fetch_status",
        &["READ_AGE_HEADER", "read_age(", "note(path, age)"],
    );
    has(
        "note",
        &[
            ".got(path, now(), age)",
            "HELD_POLL_MS",
            "set_timeout(",
            "reload_all()",
        ],
    );
    has(
        "link_lost",
        &[".lose(at)", "READ_HOLD_S", "lose_all()", "lost_since"],
    );
    has("link_back", &["Fresh::back"]);
    has("read", &[".forget("]);
    has("read_path", &[".forget("]);

    let connect = function(&net, "connect");
    for w in [
        "new(link_lost)",
        "new(link_back)",
        "add_event_listener_with_callback(\"open\"",
    ] {
        assert_eq!(connect.matches(w).count(), 1, "{w}: {connect}");
    }
    assert!(!connect.contains("new(lose_all)"), "{connect}");
    assert!(net.contains("pub fn fresh() -> ReadSignal<Fresh>"));

    let lib = read("src/lib.rs");
    let lines: Vec<&str> = lib.lines().map(str::trim).collect();
    let at = lines
        .iter()
        .position(|l| *l == "pub mod fresh;")
        .expect("lib に pub mod fresh; が在る");
    assert!(at > 0, "pub mod fresh; の前の行が無い");
    assert!(
        !lines[at - 1].contains("target_arch"),
        "fresh が wasm の target だけで組まれる"
    );
}

/// (8) 印は fresh の dom が描き、2 つの board は最終の記録の chip の後に 1 つ置く（字の並び）。
#[test]
fn gfresh_boards_text() {
    let fresh = read("src/fresh.rs");
    let dom = &fresh[fresh.find("mod dom {").expect("fresh に mod dom が在る")..];
    for w in [
        "pub fn mark() -> AnyView",
        "class=\"warn\"",
        "crate::net::fresh()",
        "crate::net::ticker()",
        ".warn(",
        ".card(",
        "delegate()",
    ] {
        assert!(dom.contains(w), "fresh の dom に {w} が無い");
    }

    let board = read("src/board.rs");
    let top = function(&board, "top");
    for w in ["net::fresh()", "Fresh::record", "fresh::mark()"] {
        assert!(top.contains(w), "board の top に {w} が無い");
    }
    assert!(!top.contains("updated_at"), "{top}");

    let account = read("src/account/board.rs");
    assert!(function(&account, "updated_chip").contains("Fresh::record"));
    assert!(function(&account, "top").contains("fresh::mark()"));
    assert!(!account.contains("updated.set("));
}

/// (10) この file の歯の名はどれも gfresh_ で始まり、残りの字は着地済みの行の verify の語を含まない。
#[test]
fn gfresh_own_names_clean() {
    let text = read("tests/teeth3/gfresh.rs");
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
    assert!(names.len() >= 8, "{names:?}");
    let words = [words_front(), words_middle(), words_back()].concat();
    names_avoid(names, &words);
}

/// 着地済みの行の verify の語の表の前の部分（40 語）。
fn words_front() -> &'static [&'static str] {
    &[
        "aaround_",
        "accept_",
        "account_",
        "acctcore_",
        "acctdoc_",
        "accthb_",
        "accthome_",
        "acctled_",
        "acctlook_",
        "acctpcore_",
        "acctproj_",
        "acctsess_",
        "acctwin_",
        "acctwire_",
        "afocus_",
        "aord_",
        "apop_",
        "askcard_",
        "athr_",
        "batchpanel_",
        "board_min_",
        "bport_",
        "brand_",
        "btuck_",
        "cadopt_",
        "cgdom_",
        "cmark_",
        "contract_form_",
        "cround_",
        "csled_",
        "denv_",
        "flight_",
        "fmark_",
        "frame_",
        "fserve_",
        "fstop_",
        "gapspage_",
        "ghb_",
        "glabel_",
        "gnav_",
    ]
}

/// 着地済みの行の verify の語の表の中の部分（40 語）。
fn words_middle() -> &'static [&'static str] {
    &[
        "graph_",
        "gsum_",
        "gtuck_",
        "gview_",
        "hbconf_",
        "hbpost_",
        "hbproc_",
        "hbroute_",
        "hcard_",
        "hcled_",
        "hcnx_",
        "hcproj_",
        "hcsess_",
        "hfig_",
        "hook_",
        "hruling_",
        "hsblock_",
        "hsderive_",
        "hspage_",
        "hsym_",
        "iclose_",
        "ilink_",
        "kcli_",
        "klink_",
        "lcard_",
        "ledgerblock_",
        "lhome_",
        "lspark_",
        "mapview_",
        "mkeys_",
        "mlink_",
        "mstore_",
        "mtree_",
        "nact_",
        "nbatch_",
        "ncard_",
        "nextstep_",
        "nodepage_",
        "nstall_",
        "nsum_",
    ]
}

/// 着地済みの行の verify の語の表の後の部分（40 語）。
fn words_back() -> &'static [&'static str] {
    &[
        "nsumw_",
        "ntime_",
        "nxact_",
        "parts_",
        "pclosed_",
        "pfold_",
        "pgz_",
        "pipe_",
        "plimit_",
        "pmore_",
        "pquest_",
        "project_",
        "ptitle_",
        "pwhole_",
        "qblock_",
        "qgate_",
        "qkey_",
        "question_",
        "rhold_",
        "runsdoc_",
        "saxis_",
        "seatblock_",
        "seatcard_",
        "server_",
        "sesplit_",
        "shb_",
        "skeleton_",
        "smore_",
        "stage_",
        "stats_",
        "steady_",
        "stlaunch_",
        "sxaxis_",
        "ticker_",
        "tipx_",
        "topbar_",
        "tz_",
        "urpanel_",
        "uword_",
        "wstrip_",
    ]
}

/// 歯の名の残りの字が着地済みの行の verify の語を含まないことを見る。
fn names_avoid(names: Vec<String>, words: &[&str]) {
    for name in &names {
        let rest = name
            .strip_prefix("gfresh_")
            .unwrap_or_else(|| panic!("{name} が gfresh_ で始まらない"));
        for w in words {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
