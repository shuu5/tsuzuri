//! 行 c-unref-now の歯（接頭辞 unow_・要件 FR13）: 未反映の代用を外した後、3 種とも「まだ分からない」の
//! project は account board の未反映の欄で数えず（字「―」）、未反映の一覧の理由の字は台帳の読みでなく
//! 器の局面の出力の無さを言う。

use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::{UnreflectedKind, UnreflectedList};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::projects::{PSort, table, unref_count, unref_of};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::ledger::{UNREF_UNKNOWN, unref_list};
use tsuzuri_surface::view::Fetched;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

/// (5) 1 種でも読めた project は数え、3 種とも「まだ分からない」なら数えない（None）。
#[test]
fn unow_card_not_counted() {
    let doc = fixture();
    let Reading::Known(s) = doc.projects[0].ledger.clone() else {
        panic!("fixture の proj-a の台帳が Known でない");
    };
    assert_eq!(unref_of(&doc.projects[0]), Some(unref_count(&s)));

    let mut none = s.clone();
    none.unreflected = 0;
    none.unreflected_kinds = Vec::new();
    none.unreflected_unknown = UnreflectedKind::ALL.to_vec();
    let mut doc2 = doc.clone();
    doc2.projects[0].ledger = Reading::Known(none);
    assert_eq!(unref_of(&doc2.projects[0]), None);

    let t = table(&doc2, PSort::Need, Mode::Beginner);
    for r in t.groups.iter().flat_map(|g| g.rows.iter()) {
        assert_eq!(r.unref, unref_of(&doc2.projects[r.index]), "{}", r.name);
    }
}

/// (6) 3 種とも「まだ分からない」の電文の理由は局面の出力の無さを言い、台帳の読みを言わない。
#[test]
fn unow_list_reason() {
    let all_unknown = UnreflectedList {
        memos: Reading::Unknown,
        rulings: Reading::Unknown,
        requests: Reading::Unknown,
    };
    let fetched = Fetched::Body(wire::encode(&all_unknown).expect("電文"));
    assert_eq!(unref_list(&fetched, fixture().at), Body::Unmeasured(UNREF_UNKNOWN));
    assert!(UNREF_UNKNOWN.contains("局面の出力"), "{UNREF_UNKNOWN}");
    assert!(!UNREF_UNKNOWN.contains("台帳を読めない"), "{UNREF_UNKNOWN}");
}

/// filter の語（main の verify の filter の語を畳んだ語と、同じノートのもう 1 つの行の接頭辞）。
const FILTERS: &[&str] = &[
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
    "kindlab_",
    "klink_",
    "ksum_",
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
    "rbusy_",
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
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "lkind_",
];

/// (7) この file の歯の名は 3 つで unow_ で始まり、名の全体は filter の語を含まない。
#[test]
fn unow_names_clean() {
    assert_eq!(FILTERS.len(), 191);
    let text = read("tests/unow.rs");
    let lines: Vec<&str> = text.lines().collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines[i + 1..]
            .iter()
            .find(|l| l.trim_start().starts_with("fn "))
            .expect("test の属性の後に fn が在る");
        let name = decl.trim_start()["fn ".len()..]
            .split('(')
            .next()
            .unwrap_or_default()
            .to_string();
        names.push(name);
    }
    assert_eq!(names.len(), 3, "{names:?}");
    for name in names {
        assert!(name.starts_with("unow_"), "歯の名 {name} が unow_ で始まらない");
        for w in FILTERS {
            assert!(!name.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
