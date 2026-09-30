//! 行 g-next-acct-origin の歯（接頭辞 nxorg_）: 次の一手の link の押しの手（next の Jump と jump・窓の名の window_of）・
//! next.rs の mod dom の押しの字の並び・この file の歯の名。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::NextMove;
use tsuzuri_surface::account::windows::ACCOUNT_WIN;
use tsuzuri_surface::frame::{BLANK, BackHow, back_how};
use tsuzuri_surface::project::next::{Jump, jump, window_of};

/// filter の語（contracts の verify の filter の語を、ほかの語を部分の字として含まない語に畳んだ列と pgz_・kindlab_）。
const FILTERS: [&str; 164] = [
    "aaround_",
    "accept_",
    "acchold_",
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
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadl_",
    "cadopt_",
    "cadq_",
    "cdorm_",
    "cfsplit_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "denv_",
    "dnedge_",
    "dnrow_",
    "dnskip_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "g3g7_",
    "gapspage_",
    "gbnote_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hdchip_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hshist_",
    "hspage_",
    "hspk_",
    "hsym_",
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstore_",
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
    "nsumw_",
    "ntime_",
    "nxact_",
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
    "pipe_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stcli_",
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "pgz_",
    "kindlab_",
];

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 型が Copy と PartialEq と Eq と Debug を持つ（組めることで見る）。
fn traits<T: Copy + PartialEq + Eq + std::fmt::Debug>(_: T) {}

/// 宣言の字 `decl` から、次の 4 つの空白と閉じ波括弧だけの行までの本文（mod dom の中の fn）。
fn body<'a>(dom: &'a str, decl: &str) -> &'a str {
    let start = dom
        .find(decl)
        .unwrap_or_else(|| panic!("mod dom に {decl} が無い"));
    let rest = &dom[start..];
    let end = rest
        .find("\n    }\n")
        .unwrap_or_else(|| panic!("{decl} の閉じの行が無い"));
    &rest[..end]
}

/// 字の列が `text` にこの順に在る（無ければどれが無いかで落ちる）。
fn in_order(text: &str, words: &[&str], what: &str) {
    let mut at = 0;
    for w in words {
        let i = text[at..]
            .find(w)
            .unwrap_or_else(|| panic!("{what} に {w} が並びの順に無い: {text}"));
        at += i + w.len();
    }
}

/// (1) 押しの手は frame の back_how に URL が読めたかを足すだけ・窓の名は限度と移動と応答なしだけ ACCOUNT_WIN。
#[test]
fn nxorg_jump_rules() {
    traits(Jump::Follow);
    let jump_fn: fn(bool, Option<&str>) -> Jump = jump;
    let readable = "http://127.0.0.1:8120/?board=account&tab=home";
    for (returned, href, how, want) in [
        (false, None, BackHow::Blocked, Jump::Follow),
        (true, Some(BLANK), BackHow::Opened, Jump::Follow),
        (true, Some(readable), BackHow::Front, Jump::Follow),
        (true, None, BackHow::Front, Jump::Front),
    ] {
        assert_eq!(back_how(returned, href), how, "{returned} {href:?}");
        assert_eq!(jump_fn(returned, href), want, "{returned} {href:?}");
    }
    assert_eq!(BLANK, "about:blank");

    assert_eq!(ACCOUNT_WIN, "tz-account");
    let named: Vec<(NextMove, &str)> = NextMove::ALL
        .into_iter()
        .filter_map(|k| window_of(k).map(|w| (k, w)))
        .collect();
    assert_eq!(
        named,
        [
            (NextMove::LimitOrMove, ACCOUNT_WIN),
            (NextMove::Unresponsive, ACCOUNT_WIN),
        ]
    );
}

/// (2) mod dom の 2 つの link は押しを press に渡し、press は窓を探して href の読めない窓だけ前に出す・
/// 判定は mod dom より前の jump（back_how に拠る）で、about:blank の字の写しは無い。
#[test]
fn nxorg_dom_wiring() {
    let text = read("src/project/next.rs");
    let (head, dom) = text.split_once("mod dom {").expect("字 mod dom { が在る");
    let click = "on:click=move |e| press(e, win)";
    assert_eq!(dom.matches(click).count(), 2, "mod dom の {click} の数");

    let big = body(dom, "fn big_view(");
    let big_a =
        format!("<a class=\"btn primary\" href=l.href target=win {click}>{{l.text}}</a>");
    assert!(big.contains(&big_a), "big_view に {big_a} が無い: {big}");
    let row = body(dom, "fn row_view(");
    let row_a = format!("<a href=l.href target=win {click}>{{l.text}}</a>");
    assert!(row.contains(&row_a), "row_view に {row_a} が無い: {row}");

    let decl = "fn press(e: ev::MouseEvent, win: Option<&'static str>) {";
    assert!(!dom.contains("pub fn press("), "press が私的でない");
    let press = body(dom, decl);
    in_order(
        press,
        &[
            "let Some(name) = win else {",
            "if e.ctrl_key() || e.meta_key() || e.shift_key() {",
            "return;",
            "let found = crate::board::open_named(\"\", name).ok().flatten();",
            "let href = found.as_ref().and_then(|w| w.location().href().ok());",
            "if jump(found.is_some(), href.as_deref()) == Jump::Front {",
            "e.prevent_default();",
            "let _ = w.focus();",
        ],
        "press",
    );
    for w in ["set_href", "about:blank", "back_how("] {
        assert!(!dom.contains(w), "mod dom に {w} が在る");
    }
    assert!(dom.contains("use leptos::ev;"), "mod dom に use leptos::ev; が無い");

    let at = head
        .find("pub fn jump(returned: bool, href: Option<&str>) -> Jump {")
        .expect("mod dom より前に jump の宣言が在る");
    assert!(
        head[at..].contains("match (back_how(returned, href), href) {"),
        "jump に back_how の match が無い"
    );
    assert!(!text.contains("about:blank"), "next.rs に about:blank が在る");
}

/// (3) この file の歯の名はどれも nxorg_ で始まり、残りの字は filter の語を含まない。
#[test]
fn nxorg_names_stay_apart() {
    let text = read("tests/nxorg.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert_eq!(names.len(), 3, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("nxorg_")
            .unwrap_or_else(|| panic!("{name} が nxorg_ で始まらない"));
        for w in FILTERS {
            assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
        }
    }
}
