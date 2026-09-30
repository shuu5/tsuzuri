//! 行 g-unref-dash-acct の歯（接頭辞 udacct_・要件 FR13）: account board の台帳の tab の未反映の数は、3 種とも分からない行なら
//! 数えない字 ― にして測れていないの印を添えず未反映の列の最大と最小に数えず、1 種か 2 種が分からなければ今の数と印のまま出す。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::{LedgerStats, UnreflectedCount, UnreflectedKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::ledger::{self, HI, LO, LedRow, Sort};
use tsuzuri_surface::account::projects::NONE_MARK;
use tsuzuri_surface::project::ledger::NONE;

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

/// fixture の proj-a の台帳（未反映 3・ruling と request が分からない）。
fn fixture_a(doc: &AccountDoc) -> LedgerStats {
    match &doc.projects[0].ledger {
        Reading::Known(s) => s.clone(),
        Reading::Unknown => panic!("fixture の proj-a の台帳が Known でない"),
    }
}

/// proj-a の台帳の未反映を memo だけの n にした値（読めない種類は無い）。
fn memo_only(doc: &AccountDoc, n: u32) -> Reading<LedgerStats> {
    let mut s = fixture_a(doc);
    s.unreflected = n;
    s.unreflected_kinds = vec![UnreflectedCount {
        kind: UnreflectedKind::Memo,
        count: n,
    }];
    s.unreflected_unknown = Vec::new();
    Reading::Known(s)
}

/// 3 種とも分からない台帳（proj-a の台帳の unreflected を 0・種類別を空・読めない種類を 3 種の全部にした値）。
fn all_unknown(doc: &AccountDoc) -> Reading<LedgerStats> {
    let mut s = fixture_a(doc);
    s.unreflected = 0;
    s.unreflected_kinds = Vec::new();
    s.unreflected_unknown = UnreflectedKind::ALL.to_vec();
    Reading::Known(s)
}

/// 3 行の台帳を差し替えた電文を Sort の Project で組んだ行（電文の順）。
fn rows(ledgers: [Reading<LedgerStats>; 3]) -> Vec<LedRow> {
    let mut doc = fixture();
    for (p, l) in doc.projects.iter_mut().zip(ledgers) {
        p.ledger = l;
    }
    ledger::table(&doc, Sort::Project).rows
}

/// 行の未反映の欄（数の字・cell の class・印・partial）。
fn un_of(row: &LedRow) -> (String, String, Option<&'static str>, bool) {
    let Reading::Known(c) = &row.cells else {
        panic!("{} の台帳が Known でない", row.name);
    };
    (
        row.numbers()[3].clone(),
        c.un_class.clone(),
        row.marks()[2],
        c.unref.partial(),
    )
}

fn want(
    text: &str,
    class: &str,
    mark: Option<&'static str>,
    partial: bool,
) -> (String, String, Option<&'static str>, bool) {
    (text.to_string(), class.to_string(), mark, partial)
}

/// (1) 未反映の数の字・cell の class・印・partial（3 種とも分からない行は ―・印なし・列の最大と最小に数えない）。
#[test]
fn udacct_numbers_and_marks() {
    assert_eq!(NONE, "―");
    assert_eq!(NONE, NONE_MARK, "account の projects の表の字と同じ");
    let doc = fixture();
    assert_eq!(
        doc.projects.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        vec!["proj-a", "proj-b", "proj-c"]
    );

    let got: Vec<_> = rows([memo_only(&doc, 9), all_unknown(&doc), memo_only(&doc, 4)])
        .iter()
        .map(un_of)
        .collect();
    assert_eq!(
        got,
        vec![
            want("9", "c-n c-un hi on", Some(HI), false),
            want("―", "c-n c-un", None, false),
            want("4", "c-n c-un lo on", Some(LO), false),
        ]
    );

    let a = doc.projects[0].ledger.clone();
    let got: Vec<_> = rows([a, all_unknown(&doc), memo_only(&doc, 0)])
        .iter()
        .map(un_of)
        .collect();
    assert_eq!(
        got,
        vec![
            want("3", "c-n c-un hi on", Some(HI), true),
            want("―", "c-n c-un", None, false),
            want("0", "c-n c-un lo", Some(LO), false),
        ]
    );

    let got: Vec<_> = rows([all_unknown(&doc), memo_only(&doc, 5), all_unknown(&doc)])
        .iter()
        .map(un_of)
        .collect();
    assert_eq!(
        got,
        vec![
            want("―", "c-n c-un", None, false),
            want("5", "c-n c-un on", None, false),
            want("―", "c-n c-un", None, false),
        ]
    );
}

/// (2) mod dom の字: 未反映の数の b の中身と印の式を 1 度ずつ持ち、前の字を持たない。
#[test]
fn udacct_dom_wiring() {
    let src = read("src/account/ledger.rs");
    let dom = &src[src.find("mod dom").expect("mod dom が在る")..];
    for w in [
        "<b class=\"num\">{cells.unref.text()}</b>",
        "cells.unref.partial().then(|| state_icon(UNKNOWN))",
    ] {
        assert_eq!(dom.matches(w).count(), 1, "{w}");
    }
    for gone in ["<b class=\"num\">{cells.unref.count}</b>", "cells.unref.unknown.is_empty()"] {
        assert!(!dom.contains(gone), "{gone}");
    }
}

/// filter の語（起草の時の main の契約表の verify の最後の字を、ほかの語を部分の字として含まない語に畳んだ語）。
const FILTERS: &[&str] = &[
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
    "dretry_",
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

/// (3) この file の test の属性の付いた fn は 3 つで、名は udacct_ で始まり、残りの字は filter の語を含まない。
#[test]
fn udacct_names_clean() {
    assert_eq!(FILTERS.len(), 220);
    let text = read("tests/udacct.rs");
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
        let rest = name
            .strip_prefix("udacct_")
            .unwrap_or_else(|| panic!("歯の名 {name} が udacct_ で始まらない"));
        for w in FILTERS {
            assert!(!rest.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
