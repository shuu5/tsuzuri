//! 行 g-seat-thr の歯: project board の席の block の下段の窓の行の棒に閾値の線を、行の右に閾値の印（┆<閾値>）を出す。
//! 値は口 /api/account の電文の caps（器の rules 行の写し）から窓の名の等しい行を写す（account board の cap_of と同じ読み）。
//! 電文・窓の行・値が読めなければ線を引かず ┆? を出し、値が 100 を越えれば線は 100 で止めて字は値のまま出す。

use std::path::PathBuf;

use tsuzuri_contract::account::{AccountDoc, WindowCap};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::home::cap_of;
use tsuzuri_surface::project::seat::{THR_MARK, Thr, WINDOWS, seat_caps, thr};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::label;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_text() -> String {
    read("../../tests/fixtures/account/acct-doc.json")
}

fn fixture() -> AccountDoc {
    wire::decode(&fixture_text()).expect("fixture が AccountDoc として読める")
}

/// 電文を本文に持つ Fetched。
fn body(doc: &AccountDoc) -> Fetched {
    Fetched::Body(wire::encode(doc).expect("電文を字にできる"))
}

/// 線と字と読み上げの字の印。
fn mark(line: Option<u64>, value: &str) -> Thr {
    Thr {
        line,
        text: format!("┆{value}"),
        aria: format!("{} {value}", label("threshold")),
    }
}

/// 値の読める窓の印（`line` は線の位置・`value` は字）。
fn known(line: u64, value: u64) -> Thr {
    mark(Some(line), &value.to_string())
}

/// 線なしの ┆? の印。
fn unknown() -> Thr {
    mark(None, "?")
}

/// (1) 電文の caps を窓の名で写し、account board の cap_of と同じ値にする。100 を越えれば線は 100 で止める。
#[test]
fn sthr_copies_caps() {
    assert_eq!(THR_MARK, "┆");
    assert!(!label("threshold").is_empty());
    let doc = fixture();
    let caps = seat_caps(&Fetched::Body(fixture_text())).expect("fixture の電文から caps");
    assert_eq!(caps, doc.caps);
    for (w, v) in WINDOWS.into_iter().zip([85, 95, 95]) {
        let t = thr(Some(&caps), w);
        assert_eq!(t, known(v, v), "{w}");
        assert_eq!(t.line, cap_of(&doc.caps, w), "{w} は cap_of と同じ値");
    }

    let mut doc = fixture();
    doc.caps[0].cap = Reading::Known(70);
    doc.caps[1].cap = Reading::Known(120);
    let caps = seat_caps(&body(&doc)).expect("書き換えた電文から caps");
    assert_eq!(thr(Some(&caps), "five_hour"), known(70, 70));
    assert_eq!(thr(Some(&caps), "seven_day"), known(100, 120));
    assert_eq!(thr(Some(&caps), "seven_day").text, "┆120");
    assert_eq!(thr(Some(&caps), "seven_day_model"), known(95, 95));
}

/// (2) 電文・窓の行・値が読めなければ線なしの ┆?。
#[test]
fn sthr_unknown_marks() {
    for fetched in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{".to_string()),
    ] {
        let caps = seat_caps(&fetched);
        assert_eq!(caps, None, "{fetched:?}");
        for w in WINDOWS {
            assert_eq!(thr(caps.as_deref(), w), unknown(), "{fetched:?} の {w}");
        }
    }

    let mut doc = fixture();
    doc.caps[0].cap = Reading::Unknown;
    doc.caps.remove(1);
    let caps = seat_caps(&body(&doc)).expect("書き換えた電文から caps");
    assert_eq!(thr(Some(&caps), "five_hour"), unknown(), "値が Unknown の行");
    assert_eq!(thr(Some(&caps), "seven_day"), unknown(), "行の無い窓");
    assert_eq!(thr(Some(&caps), "seven_day_model"), known(95, 95));
    assert_eq!(thr(Some(&caps), "opus_week"), unknown(), "知らない窓");

    let empty: Vec<WindowCap> = Vec::new();
    for w in WINDOWS {
        assert_eq!(thr(Some(&empty), w), unknown(), "空の列の {w}");
    }
}

/// fn の本文（`head` から、字下げの無い閉じ括弧まで）。
fn fn_body<'a>(text: &'a str, head: &str) -> &'a str {
    let start = text.find(head).unwrap_or_else(|| panic!("{head} が在る"));
    let len = text[start..]
        .find("\n}\n")
        .unwrap_or_else(|| panic!("{head} の終わりが在る"));
    &text[start..start + len]
}

/// (3) DOM は窓の行に caps の Memo を渡して thr で線と印を組み、low_view が Memo を 1 回だけ組む。
#[test]
fn sthr_dom_text() {
    let dom = read("src/project_dom/seat.rs");
    let window = fn_body(&dom, "fn window_view(");
    for text in [
        "caps: Memo<Option<Vec<WindowCap>>>",
        "thr(c.as_deref(), &window)",
        r#"<span class="cap" style=format!("left:{c}%")></span>"#,
        r#"<span class="th num" aria-label=aria>{mark}</span>"#,
        r#"term("threshold", th().text)"#,
    ] {
        assert_eq!(window.matches(text).count(), 1, "window_view の {text}");
    }
    let low = fn_body(&dom, "fn low_view(");
    assert_eq!(
        low.matches("Memo::new(move |_| group_doc.with(seat_caps))")
            .count(),
        1
    );
    assert_eq!(low.matches("window_view(r, caps)").count(), 1);
    assert_eq!(dom.matches("Memo::new(").count(), 1);
    assert_eq!(dom.matches("crate::net::read(").count(), 2);
    for body in [window, low] {
        for text in ["set_interval", "fleet.group_pressure", "85", "95"] {
            assert!(!body.contains(text), "{text}");
        }
    }

    let css = read("style.css");
    for rule in [
        "#orch .wrow { display: grid; grid-template-columns: 44px minmax(0, 1fr) 64px;",
        "#orch .wrow .th {",
        ".meter .bar .cap {",
    ] {
        assert!(css.contains(rule), "stylesheet の {rule}");
    }
}

/// main の契約表の verify の filter の語。
const WORDS: &[&str] = &[
    "aaround_",
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
    "cg9_",
    "cgdom_",
    "cmark_",
    "cnote_",
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
    "ecache_",
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "g3g7_",
    "gacct_",
    "gapspage_",
    "gatt_",
    "gbnote_",
    "gchip_",
    "gcoach_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gins_",
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
    "gwv_",
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
    "hthr_",
    "http_",
    "hwstore_",
    "iclose_",
    "ilink_",
    "jcount_",
    "jrun_",
    "kcli_",
    "kg9_",
    "kindlab_",
    "klink_",
    "ksum_",
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
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
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
    "stnfy_",
    "stskill_",
    "sttgt_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "wstrip_",
];

/// (4) test の fn の名は 4 つで、どれも sthr_ で始まり、残りの字は filter の語を含まない。
#[test]
fn sthr_own_names_clean() {
    assert_eq!(WORDS.len(), 212);
    let text = read("tests/sthr.rs");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の次の行は fn");
            &rest[..rest.find('(').expect("fn の名の後に (")]
        })
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("sthr_")
            .unwrap_or_else(|| panic!("{name} は sthr_ で始まる"));
        for w in WORDS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
