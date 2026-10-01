//! 行 g-seat-split の歯: project board の block「orchestrator と口座」の seat.rs から、wasm の target のときだけの
//! DOM を src/project_dom/seat.rs へ移し、seat.rs を host の部分と path の属性で読む mod dom の宣言だけにした形と、
//! この file の歯の名。
#![cfg(test)]

use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

const SEAT: &str = "src/project/seat.rs";
const DOM: &str = "src/project_dom/seat.rs";

/// mod dom の外の名の列（base の seat.rs の const BLOCK から mod dom まで・fn strip は 2 度）に impl の 2 つを足した列。
const HOST: &[(&str, &str)] = &[
    ("const", "BLOCK"),
    ("const", "PATH"),
    ("const", "PATHS"),
    ("const", "FOLDS"),
    ("const", "REASON"),
    ("const", "LIMIT_LINE"),
    ("const", "MOVE_WAIT"),
    ("const", "NEXT_TARGET"),
    ("const", "OROW"),
    ("const", "NO_RESET"),
    ("const", "SPAN_KEY"),
    ("const", "WIDTH"),
    ("const", "HEIGHT"),
    ("const", "LOW"),
    ("const", "WINDOWS"),
    ("const", "SHORT"),
    ("fn", "short"),
    ("enum", "Span"),
    ("const", "ALL"),
    ("fn", "key"),
    ("fn", "secs"),
    ("fn", "tick"),
    ("struct", "Tick"),
    ("const", "TICK_MAX"),
    ("const", "TICK_MANY"),
    ("fn", "span_ticks"),
    ("fn", "span_of"),
    ("fn", "with_span"),
    ("fn", "state_value"),
    ("fn", "hm"),
    ("fn", "hmd"),
    ("fn", "until"),
    ("fn", "bar_class"),
    ("struct", "Sign"),
    ("const", "OK"),
    ("const", "NG"),
    ("fn", "sign"),
    ("struct", "Top"),
    ("struct", "Rect"),
    ("struct", "Strip"),
    ("struct", "WindowRow"),
    ("struct", "Low"),
    ("struct", "Band"),
    ("struct", "Seat"),
    ("fn", "strip"),
    ("fn", "card"),
    ("fn", "body"),
    ("fn", "content"),
    ("fn", "seat"),
    ("fn", "top"),
    ("fn", "map"),
    ("fn", "strip"),
    ("fn", "px"),
    ("fn", "rects"),
    ("fn", "strip_class"),
    ("fn", "marks"),
    ("fn", "strip_svg"),
    ("fn", "line"),
    ("const", "LEGEND"),
    ("const", "SAMPLE_W"),
    ("const", "SAMPLE_H"),
    ("fn", "sample_svg"),
    ("fn", "low"),
    ("fn", "window_row"),
    ("fn", "band"),
    ("fn", "view"),
    ("mod", "dom"),
    ("impl", "Span"),
    ("impl", "Seat"),
];

/// mod dom の中の名の列（base の mod dom の中の宣言を順に）。
const INNER: [(&str, &str); 13] = [
    ("const", "HOURGLASS"),
    ("fn", "search"),
    ("fn", "pick"),
    ("fn", "view"),
    ("fn", "unknown"),
    ("fn", "sign_view"),
    ("fn", "seat_view"),
    ("fn", "band_view"),
    ("fn", "big_icon"),
    ("fn", "top_view"),
    ("fn", "strip_view"),
    ("fn", "window_view"),
    ("fn", "low_view"),
];

const KINDS: [&str; 7] = ["const", "static", "fn", "struct", "enum", "impl", "mod"];

/// 1 行が宣言の行なら種と名（行の頭の空白と pub・pub(…) を除いて見る）。
fn decl(line: &str) -> Option<(&str, &str)> {
    let mut s = line.trim_start();
    if let Some(r) = s.strip_prefix("pub ") {
        s = r;
    } else if let Some(r) = s.strip_prefix("pub(") {
        let close = r.find(')')?;
        s = r[close + 1..].strip_prefix(' ')?;
    }
    let (kind, rest) = s.split_once(' ')?;
    if !KINDS.contains(&kind) {
        return None;
    }
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    Some((kind, &rest[..end]))
}

/// file の宣言の行の種と名（`flush` なら字下げの無い行だけ）。
fn decls(text: &str, flush: bool) -> Vec<(&str, &str)> {
    text.lines()
        .filter(|l| !flush || !l.starts_with(char::is_whitespace))
        .filter_map(decl)
        .collect()
}

/// 宣言の行を読む決まりそのもの。
#[test]
fn sesplit_decl_reader() {
    assert_eq!(decl("pub const BLOCK: &str = \"orch\";"), Some(("const", "BLOCK")));
    assert_eq!(decl("    fn map<T>(x: T) {"), Some(("fn", "map")));
    assert_eq!(decl("mod dom;"), Some(("mod", "dom")));
    assert_eq!(decl("pub(crate) fn px(v: u32) -> u32 {"), Some(("fn", "px")));
    assert_eq!(decl("impl Span {"), Some(("impl", "Span")));
    assert_eq!(decl("/// fn doc"), None);
    assert_eq!(decl("let fn_x = 1;"), None);
    assert_eq!(decl("fn stripped()"), Some(("fn", "stripped")));
    assert_ne!(decl("fn stripped()"), Some(("fn", "strip")));
}

/// (1) seat.rs は inline の mod dom を持たず、path の属性で src/project_dom/seat.rs を読む宣言と、
/// dom::view() を呼ぶ pub fn view を持つ。src/project の下に dom.rs も seat の dir も無い。
#[test]
fn sesplit_decl_in_seat() {
    let seat = read(SEAT);
    assert!(!seat.contains("mod dom {"), "seat.rs に inline の mod dom が在る");
    let decl_lines = "#[cfg(target_arch = \"wasm32\")]\n#[path = \"../project_dom/seat.rs\"]\nmod dom;";
    assert_eq!(seat.matches(decl_lines).count(), 1, "path の属性の mod dom の宣言");
    let view = "#[cfg(target_arch = \"wasm32\")]\npub fn view() -> leptos::prelude::AnyView {\n    dom::view()\n}";
    assert!(seat.contains(view), "seat.rs の pub fn view");
    for w in ["fn more_view(", "fn seat_view("] {
        assert!(!seat.contains(w), "seat.rs に {w} が在る");
    }
    assert!(!crate_dir().join("src/project/dom.rs").exists());
    assert!(!crate_dir().join("src/project/seat").exists());
}

/// (2) src/project_dom/seat.rs は mod dom の中の名の列を字下げの無い宣言の行で持ち、seat.rs は fn view のほかを持たない。
#[test]
fn sesplit_dom_file_holds_view() {
    let dom = read(DOM);
    assert!(
        dom.lines().any(|l| l.starts_with("pub fn view() -> AnyView {")),
        "字下げの無い pub fn view"
    );
    assert!(!dom.contains("mod dom {"));
    let flush = decls(&dom, true);
    for want in INNER {
        assert!(flush.contains(&want), "{DOM} に {} {} の宣言が無い", want.0, want.1);
    }
    let seat = read(SEAT);
    let host = decls(&seat, false);
    for want in INNER.iter().filter(|d| **d != ("fn", "view")) {
        assert!(!host.contains(want), "{SEAT} に {} {} が残る", want.0, want.1);
    }
}

/// (3) seat.rs は mod dom の外の名の列の宣言の行を、種と名ごとに列に在る数以上持つ。
#[test]
fn sesplit_host_names_kept() {
    assert_eq!(HOST.len(), 69);
    let seat = read(SEAT);
    let host = decls(&seat, false);
    for want in HOST {
        let need = HOST.iter().filter(|d| *d == want).count();
        let have = host.iter().filter(|d| *d == want).count();
        assert!(have >= need, "{SEAT} の {} {} は {have} 行（{need} 行以上）", want.0, want.1);
    }
}

/// (4) details の要素と view! は seat.rs に無く、src/project_dom/seat.rs に view! が在る。
#[test]
fn sesplit_details_moved() {
    let seat = read(SEAT);
    assert!(!seat.contains("<details"), "seat.rs に details");
    assert!(!seat.contains("view!"), "seat.rs に view!");
    let dom = read(DOM);
    assert!(dom.contains("view!"), "{DOM} に view! が無い");
}

/// filter の語（着地済みの行の verify の filter の語と、第 3 波から第 8 波の行の接頭辞）。
const FILTER: [&str; 106] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_", "server_",
    "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_", "mkeys_",
    "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_", "pmore_",
    "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "qgate_", "hcard_", "ntime_",
    "ptitle_", "ncard_", "hsym_", "smore_", "lcard_", "aaround_", "hruling_", "sxaxis_",
    "plimit_", "bport_", "fstop_", "nsum_", "nsumw_", "fmark_", "fserve_", "cadopt_", "tipx_",
    "hcsess_", "hcproj_", "mstore_", "athr_", "qblock_", "wsteady_", "gsum_", "nstall_", "pfold_",
    "uword_", "cround_", "cgdom_", "csled_", "lhome_", "shb_", "ghb_", "nact_", "aord_", "mtree_",
];

/// (9) この file の歯の名は sesplit_ で始まり、残りの字は filter の語を含まない。
#[test]
fn sesplit_own_names_clean() {
    let me = read("tests/sesplit.rs");
    let lines: Vec<&str> = me.lines().collect();
    let mut n = 0;
    for (i, l) in lines.iter().enumerate() {
        if l.trim() != "#[test]" {
            continue;
        }
        let f = lines[i + 1].trim();
        let name = f
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("test の属性の次が fn でない: {f}"));
        let rest = name
            .strip_prefix("sesplit_")
            .unwrap_or_else(|| panic!("{name} が sesplit_ で始まらない"));
        for w in FILTER {
            assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
        }
        n += 1;
    }
    assert_eq!(n, 6);
}
