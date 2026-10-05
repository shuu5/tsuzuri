//! 行 g-wins-vocab の歯: 語彙の鍵 open_windows の内部の名の字を、開いた窓の一覧を browser の保存
//! （account の windows の WINS_KEY）に残す今の形と計画の決め d4 に揃えること、見本の語彙と同じ字であること。
#![cfg(test)]

use crate::common::read;
use tsuzuri_surface::account::windows::WINS_KEY;
use tsuzuri_surface::vocab::vocab;

/// 字 open_windows を鍵に持つ行から次の閉じ波括弧まで。
fn block(text: &str) -> &str {
    let start = text.find("\"open_windows\": {").expect("鍵 open_windows の行が在る");
    let rest = &text[start..];
    let end = rest.find('}').expect("鍵 open_windows の後に閉じ波括弧が在る");
    &rest[..=end]
}

/// (1) 面の語彙の鍵 open_windows の内部の名は browser の保存に残す 4 行で、器の registry を持たない。
#[test]
fn gwv_internal_follows_store() {
    let term = vocab().term("open_windows").expect("鍵 open_windows が語彙に在る");
    let want = [
        "window.open の handle".to_string(),
        format!("mock は画面の状態と localStorage {WINS_KEY}"),
        format!("本番もその browser の保存 {WINS_KEY} に残す"),
        "便利の写しで正本ではない".to_string(),
    ]
    .join("\n");
    assert_eq!(term.internal, want);
    assert!(!term.internal.contains("registry"));
    assert_eq!(term.label, "開いている窓");
}

/// (2) 面の語彙と見本の語彙の鍵 open_windows の block は同じ字で、どちらも器の window registry を持たない。
#[test]
fn gwv_mock_block_same() {
    let surface = read("vocab.json");
    let mock = read("../../docs/design/mock3/vocab.json");
    assert_eq!(block(&surface), block(&mock));
    assert!(!surface.contains("window registry"), "面の vocab.json");
    assert!(!mock.contains("window registry"), "見本の vocab.json");
}

/// verify の filter の語（main fb2cc24 の 177 語と同じノートのほかの 2 行の接頭辞）。
const FILTERS: [&str; 179] = [
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
    "dngrp_",
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
    "gatt_",
    "gbnote_",
    "gcoach_",
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
    "hthr_",
    "http_",
    "hwstore_",
    "iclose_",
    "ilink_",
    "jcount_",
    "jrun_",
    "kcli_",
    "kindlab_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstg_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
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
    "ntime_",
    "nxact_",
    "nxorg_",
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
    "pgz_",
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
    "stbp_",
    "stcli_",
    "steady_",
    "sttgt_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "ksum_",
    "tkad_",
];

/// (3) この file の歯は 3 つで、名は gwv_ で始まり、残りの字は filter の語を含まない。
#[test]
fn gwv_names_clean() {
    let text = read("tests/teeth3/gwv.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<String> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .map(|w| {
            let name = w[1].strip_prefix("fn ").expect("#[test] の次の行は fn");
            name[..name.find('(').expect("fn の名の後に ( が在る")].to_string()
        })
        .collect();
    assert_eq!(names.len(), 3);
    for name in &names {
        let tail = name
            .strip_prefix("gwv_")
            .unwrap_or_else(|| panic!("{name} は gwv_ で始まる"));
        for word in FILTERS {
            assert!(!tail.contains(word), "{name} は {word} を含まない");
        }
    }
}
