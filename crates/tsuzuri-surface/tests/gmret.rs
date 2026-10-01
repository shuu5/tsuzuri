//! 行 g-map-retired の歯（面）: 廃止した設計ノートを設計の木で畳んだ段に束ねること（圧縮の面は行 m-map-compact で消した）・
//! 欄 retired の無い電文は何も畳まず理由を返すこと・DOM の字（wasm の target のときだけなので src の字で見る）・
//! この file の歯の名。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_contract::graph::{GraphDoc, GraphNode, GraphSource, NodeKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::band::Band;
use tsuzuri_surface::mapview::tree::{Branch, Head, Tree, fold_key, forest};
use tsuzuri_surface::mapview::{RETIRED, RETIRED_UNREAD, retired_notes, retired_unread};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> GraphDoc {
    wire::decode(&read("../../tests/fixtures/surface/graph-doc.json"))
        .expect("fixture が電文として読める")
}

fn owned(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// fixture の設計ノートの行を、ノート na の 2 行（行の番号 3 と 9）・nb の 1 行・nc の 2 行（nc#2 が 5・nc#1 が 2）と
/// 井桁の無い行 lone に替えた電文（欄 retired は無し）。
fn shelf_doc() -> GraphDoc {
    let mut doc = fixture();
    let proto: GraphNode = doc
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::NoteRow)
        .expect("fixture の設計ノートの行")
        .clone();
    doc.nodes.retain(|n| n.kind != NodeKind::NoteRow);
    for (id, line) in [
        ("nc#2", Some(5)),
        ("na#2", Some(9)),
        ("lone", None),
        ("nb#1", Some(1)),
        ("na#1", Some(3)),
        ("nc#1", Some(2)),
    ] {
        let mut n = proto.clone();
        n.id = id.to_string();
        n.title = format!("{id} の題");
        n.line = line;
        doc.nodes.push(n);
    }
    doc.retired = None;
    doc
}

/// 欄 retired を置いた電文。
fn with_retired(retired: Option<&[&str]>) -> GraphDoc {
    let mut doc = shelf_doc();
    doc.retired = retired.map(owned);
    doc
}

/// 項の名（帯は band:名:数・設計ノートは note:名:数・廃止の段は retired:数・節点は id）。
fn name(head: &Head) -> String {
    match head {
        Head::Band { band, count } => format!("band:{}:{count}", band.name()),
        Head::Note { note, count } => format!("note:{note}:{count}"),
        Head::Retired(count) => format!("retired:{count}"),
        Head::Node(item) => item.id.clone(),
    }
}

/// 項を前から順にたどった (深さ・名・open・kids の数) の列。
fn outline(item: &Branch) -> Vec<(usize, String, bool, usize)> {
    fn go(b: &Branch, depth: usize, out: &mut Vec<(usize, String, bool, usize)>) {
        out.push((depth, name(&b.head), b.open, b.kids.len()));
        for k in &b.kids {
            go(k, depth + 1, out);
        }
    }
    let mut out = Vec::new();
    go(item, 0, &mut out);
    out
}

fn rows(expected: &[(usize, &str, bool, usize)]) -> Vec<(usize, String, bool, usize)> {
    expected
        .iter()
        .map(|(d, n, o, k)| (*d, n.to_string(), *o, *k))
        .collect()
}

/// 設計の木の design-note の帯の項。
fn dn_branch(doc: &GraphDoc) -> Branch {
    forest(doc, Tree::Design)
        .items
        .into_iter()
        .find(|b| matches!(b.head, Head::Band { band: Band::DesignNote, .. }))
        .expect("design-note の帯の項")
}

/// file の字 `mod dom {` から後の字（wasm の target のときだけの DOM）。
fn dom_part(rel: &str) -> String {
    let src = read(rel);
    let at = src
        .find("mod dom {")
        .unwrap_or_else(|| panic!("{rel} に mod dom が無い"));
    src[at..].to_string()
}

/// (5) 設計の木: 廃止していないノートの開いた項・井桁の無い項・畳んだ廃止の段（下に畳んだノートの項）の順。
#[test]
fn gmret_tree_folds_retired() {
    let doc = with_retired(Some(&["nb", "nc"]));
    let band = format!("band:{}:6", Band::DesignNote.name());
    assert_eq!(
        outline(&dn_branch(&doc)),
        rows(&[
            (0, &band, true, 3),
            (1, "note:na:2", true, 2),
            (2, "na#1", false, 0),
            (2, "na#2", false, 0),
            (1, "lone", false, 0),
            (1, "retired:2", false, 2),
            (2, "note:nb:1", false, 1),
            (3, "nb#1", false, 0),
            (2, "note:nc:2", false, 2),
            (3, "nc#1", false, 0),
            (3, "nc#2", false, 0),
        ])
    );
    assert_eq!(fold_key(&Head::Retired(2)), "tree:retired");
    assert_eq!(retired_unread(&doc), None);
    let want: BTreeSet<&str> = ["nb", "nc"].into();
    assert_eq!(retired_notes(&doc), want);

    let mut plain = doc.clone();
    plain.retired = None;
    assert_eq!(
        forest(&doc, Tree::Ledger),
        forest(&plain, Tree::Ledger),
        "台帳の木は欄 retired が在っても無くても同じ"
    );
}

/// (6) 欄 retired の無い電文は何も畳まず理由を返す。読めない索引・設計ノートの行の無い電文・空の列は理由も無い。
#[test]
fn gmret_unread_folds_nothing() {
    let doc = with_retired(None);
    assert_eq!(retired_unread(&doc), Some(RETIRED_UNREAD));
    assert_eq!(
        RETIRED_UNREAD,
        "設計ノートの状態の欄が読めない（廃止したノートも畳まずに出す）"
    );
    let band = format!("band:{}:6", Band::DesignNote.name());
    assert_eq!(
        outline(&dn_branch(&doc)),
        rows(&[
            (0, &band, true, 4),
            (1, "note:na:2", true, 2),
            (2, "na#1", false, 0),
            (2, "na#2", false, 0),
            (1, "note:nb:1", true, 1),
            (2, "nb#1", false, 0),
            (1, "note:nc:2", true, 2),
            (2, "nc#1", false, 0),
            (2, "nc#2", false, 0),
            (1, "lone", false, 0),
        ])
    );

    let mut no_index = with_retired(None);
    no_index.unread = vec![GraphSource::Design];
    assert_eq!(retired_unread(&no_index), None);
    let mut no_rows = with_retired(None);
    no_rows.nodes.retain(|n| n.kind != NodeKind::NoteRow);
    assert_eq!(retired_unread(&no_rows), None);
    assert_eq!(retired_unread(&with_retired(Some(&[]))), None);
    assert_eq!(retired_unread(&with_retired(Some(&["zz"]))), None);
}

/// (7) DOM の字: 設計の木は設計の木の時だけ理由を出し、廃止の段の頭に状態の字と数を出す（圧縮の面の段は行 m-map-compact で消した）。
#[test]
fn gmret_dom_wiring() {
    let dom = dom_part("src/mapview/tree.rs");
    for w in [
        "(tree == Tree::Design)",
        ".then(|| retired_unread(doc))",
        ".map(unmeasured);",
        "{retired}",
        "Head::Retired(count) => view! {",
        "<span class=\"mono\">{RETIRED}</span><span class=\"chip num\">{*count}</span>",
    ] {
        assert!(dom.contains(w), "tree.rs の mod dom に {w} が無い");
    }
    assert_eq!(RETIRED, "retired");
}

/// 計画の verify の filter の語を、ほかの語を部分の字として含まない語に畳んだ字（歯の名が含んではならない部分の字）。
const FILTERS: [&str; 222] = [
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
    "udacct_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "wstrip_",
];

/// (8) この file の歯の名。
#[test]
fn gmret_names_clean() {
    let text = read("tests/gmret.rs");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("fn の行");
            rest.split('(').next().expect("fn の名")
        })
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("gmret_")
            .unwrap_or_else(|| panic!("{name} は gmret_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
    for (i, a) in FILTERS.iter().enumerate() {
        for b in &FILTERS[i + 1..] {
            assert_ne!(a, b, "filter の語が重なる");
        }
    }
}
