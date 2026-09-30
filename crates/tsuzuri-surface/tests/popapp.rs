//! 行 g-popup-app の歯（接頭辞 popapp_）: app の窓の中から開く名前つきの窓を app の窓にする手
//! （frame の window_features・Press の plain・board の standalone と open_named と plain_click・
//! 3 か所の開きと知らせの題の link の押しの受け手・manifest の feature・この file の歯の名）。
//! wasm の DOM の手は host で撃てないので、字の並びで見る。

use std::path::{Path, PathBuf};

use tsuzuri_surface::frame::{
    self, Mode, POPUP, PageId, Press, STANDALONE_QUERY, switch_url, window_features,
};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// `text` の中の `decl` から始まる fn の本文（次の行頭の閉じ括弧まで）。
fn body<'a>(text: &'a str, decl: &str) -> &'a str {
    let at = text
        .find(decl)
        .unwrap_or_else(|| panic!("{decl} が無い"));
    let rest = &text[at..];
    rest.split("\n}\n")
        .next()
        .or_else(|| rest.split("\n    }\n").next())
        .expect("fn の本文")
}

/// src の下の .rs の file の相対 path の全部。
fn sources(dir: &Path, rel: &str, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("src の dir") {
        let path = entry.expect("dir の entry").path();
        let name = path.file_name().expect("file の名").to_string_lossy().into_owned();
        let rel = format!("{rel}/{name}");
        if path.is_dir() {
            sources(&path, &rel, out);
        } else if name.ends_with(".rs") {
            out.push(rel);
        }
    }
}

fn press(button: i16, ctrl: bool, meta: bool, shift: bool, alt: bool) -> Press {
    Press {
        button,
        ctrl,
        meta,
        shift,
        alt,
    }
}

/// (1) window_features は app の窓の中でだけ popup を返し、plain は左の button で修飾の鍵なしの押しだけ真。
#[test]
fn popapp_features_by_standalone() {
    assert_eq!(POPUP, "popup");
    assert_eq!(STANDALONE_QUERY, "(display-mode: standalone)");
    assert_eq!(window_features(true), "popup");
    assert_eq!(window_features(false), "");

    assert!(press(0, false, false, false, false).plain());
    assert!(Press::default().plain());
    for held in [
        press(0, true, false, false, false),
        press(0, false, true, false, false),
        press(0, false, false, true, false),
        press(0, false, false, false, true),
        press(1, false, false, false, false),
        press(2, false, false, false, false),
    ] {
        assert!(!held.plain(), "{held:?}");
    }
    // 頁の切り替えは同じ plain で読む（振る舞いは変わらない）。
    let to = switch_url(PageId::Home, PageId::Ask, Mode::Beginner, Press::default());
    assert_eq!(to, Some(frame::href(PageId::Ask, Mode::Beginner)));
    assert_eq!(
        switch_url(PageId::Home, PageId::Home, Mode::Beginner, Press::default()),
        None
    );
    assert_eq!(
        switch_url(
            PageId::Home,
            PageId::Ask,
            Mode::Beginner,
            press(0, false, false, false, true)
        ),
        None
    );
}

/// (2) board の standalone は media query を読み、open_named は features を渡して窓を前に出さない。
/// 戻るは open_named で account board の窓を探す。
#[test]
fn popapp_board_helper() {
    let text = read("src/board.rs");
    let standalone = body(&text, "pub fn standalone() -> bool {");
    assert!(standalone.contains(".match_media(frame::STANDALONE_QUERY)"), "{standalone}");
    assert!(standalone.contains("q.matches()"), "{standalone}");
    let open = body(&text, "pub fn open_named(");
    assert!(
        open.contains("open_with_url_and_target_and_features(url, name, frame::window_features(standalone()))"),
        "{open}"
    );
    for (name, part) in [("standalone", standalone), ("open_named", open)] {
        for word in ["focus(", "bringToFront"] {
            assert!(!part.contains(word), "{name} が {word} を含む");
        }
    }
    let back = body(&text, "fn back_to_board(");
    assert!(back.contains("open_named(\"\", ACCOUNT_WIN)"), "{back}");
    assert!(text.contains("pub fn plain_click(e: &ev::MouseEvent) -> bool {"));
}

/// (3) 名前つきの窓は board の open_named だけが開き、3 か所の開きと押しの受け手がそこを通る。
#[test]
fn popapp_sites_through_helper() {
    let mut files = Vec::new();
    sources(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), "src", &mut files);
    let raw: Vec<&String> = files
        .iter()
        .filter(|f| read(f).contains("open_with_url_and_target"))
        .collect();
    assert_eq!(raw, ["src/board.rs"], "open_with_url_and_target を持つ file");
    assert_eq!(
        read("src/board.rs").matches("open_with_url_and_target").count(),
        1
    );
    for rel in ["src/account/windows.rs", "src/project/next.rs"] {
        assert_eq!(read(rel).matches("open_named(").count(), 1, "{rel}");
    }

    // frame の switch_url は plain を呼び、board の press は 4 つの修飾の鍵と button を読む。
    let frame_src = read("src/frame.rs");
    let switch = body(&frame_src, "pub fn switch_url(");
    assert!(switch.contains("press.plain()"), "{switch}");
    assert!(frame_src.contains("pub fn plain(self) -> bool {"));
    let board = read("src/board.rs");
    let press_fn = body(&board, "fn press(e: &ev::MouseEvent) -> Press {");
    for word in ["e.button()", "e.ctrl_key()", "e.meta_key()", "e.shift_key()", "e.alt_key()"] {
        assert!(press_fn.contains(word), "press に {word} が無い");
    }
    assert!(body(&board, "pub fn plain_click(").contains("press(e).plain()"));

    // 知らせの題の link の押しの受け手。
    let notices = read("src/account/notices.rs");
    assert_eq!(notices.matches("on:click=move |e| named_window(e, &project)").count(), 1);
    let named = body(&notices, "fn named_window(");
    assert!(named.contains("crate::board::plain_click(&e)"), "{named}");
    assert!(named.contains("crate::board::standalone()"), "{named}");
    assert_eq!(named.matches("open_named(").count(), 1, "{named}");
    for word in ["focus(", "_key()", "button()"] {
        assert!(!named.contains(word), "named_window が {word} を含む");
    }
}

/// (4) manifest の web-sys の features に MediaQueryList が 1 度在る。
#[test]
fn popapp_manifest_feature() {
    let manifest = read("Cargo.toml");
    assert_eq!(manifest.matches("\"MediaQueryList\"").count(), 1);
}

/// 契約表の verify の filter の語（ほかの語を部分の字として含まない最小の列・main 0aca9908）。
const FILTERS: [&str; 248] = [
    "aaround_",
    "abss_",
    "abst_",
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
    "aface_",
    "afocus_",
    "alean_",
    "aord_",
    "aown_",
    "apark_",
    "apop_",
    "areread_",
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
    "cexcl_",
    "cfsplit_",
    "cg9_",
    "cgdom_",
    "cishard_",
    "cmark_",
    "cnote_",
    "cnret_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "cupd_",
    "cupdlist_",
    "denv_",
    "dnedge_",
    "dngrp_",
    "dnrow_",
    "dnskip_",
    "dretry_",
    "dstg_",
    "ecache_",
    "eheld_acct_",
    "eheld_design_",
    "eheld_held_",
    "eheld_marks_",
    "eheld_vessel_",
    "elazy_",
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
    "f123_",
    "f159_",
    "f212_",
    "f2ret_",
    "fdlt_",
    "fdlv_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fundl_",
    "fxpre_",
    "g3g7_",
    "gacct_",
    "gapspage_",
    "gatt_",
    "gbnote_",
    "gchip_",
    "gcoach_",
    "gext_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gins_",
    "gjst_",
    "glabel_",
    "gmret_",
    "gmretw_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "gwv_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbon_",
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
    "hthr_",
    "http_",
    "hwstore_",
    "iclose_",
    "ilink_",
    "inject_",
    "jcount_",
    "jrun_",
    "kcli_",
    "kdeny_",
    "kg9_",
    "kindlab_",
    "klink_",
    "ksum_",
    "lateface_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lgrp_",
    "lhome_",
    "lidle_",
    "lkind_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstg_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mqask_",
    "mqface_",
    "mstore_",
    "mtips_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntc_",
    "ntime_",
    "nxact_",
    "nxorg_",
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
    "pgsw_",
    "pgz_",
    "pipe_",
    "pkac_",
    "pkview_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "pubfp_",
    "pubscan_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "qsig_",
    "question_",
    "rbusy_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server::design::tests::",
    "server::events::",
    "server_",
    "sesplit_",
    "sgrace_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stbp_",
    "stcli_",
    "steady_",
    "sthr_",
    "stnfy_",
    "stskill_",
    "sttgt_",
    "sxaxis_",
    "tgall_",
    "tgown_",
    "ticker_",
    "tipx_",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "udacct_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "wstrip_",
];

/// (5) この file の歯は 5 本で、名はどれも popapp_ で始まり、残りの字は filter の語を含まない。
#[test]
fn popapp_own_names_clean() {
    let text = read("tests/popapp.rs");
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
    assert_eq!(names.len(), 5, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("popapp_")
            .unwrap_or_else(|| panic!("{name} が popapp_ で始まらない"));
        for w in FILTERS {
            assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
        }
    }
}
