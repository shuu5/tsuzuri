//! 不変条件 g-3 の外の台帳の歯（接頭辞 gext_・行 c-g3-extern）: ほかの project の台帳の読み（`build::outside`）と、
//! 族が自分の台帳に無い行（外の台帳の行）を外の台帳で確かめる g-3（`check::outside_heads` が確かめられない行の
//! 節点の数と族を名指す）。fixture は tests/fixtures/graph/unruled/ の 4 つの file（読むだけ）で、族 s9-far と s7-gone と
//! s8-near（作った名）を引く書き出しの行と外の台帳の字は歯の中で足す。
//! 境界の歯は tsuzuri-boundary の tests/teeth2/gextw.rs（この file の歯の名と合わせて 6 つ）。
#![cfg(test)]

use std::fs;
use std::path::Path;

use tsuzuri_core::graph::build::{add_rulings, outside};
use tsuzuri_core::graph::check::{outside_heads, outside_rulings};
use tsuzuri_core::graph::{Graph, Inputs, Outside, Verdict, build, check};

/// 歯の名が含んではならない部分の字（起草の時の main 11affd91 の契約表の verify の filter の語 303 語を、
/// ほかの語を部分の字として含まない 227 語に畳んだ語）。
const FILTERS: [&str; 227] = [
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
    "fundl_",
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
    "jcount_",
    "jrun_",
    "kcli_",
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

fn read(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/graph/unruled")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

/// 族 s9-far の外の台帳（根と子 3 つと、子の無い族 s9-lone・s9-far.3 の notes は裁定の定型行と方針の定型行を持つ）。
const FAR: &str = r#"[
{"id": "s9-far", "title": "外の根", "status": "open", "issue_type": "epic"},
{"id": "s9-far.2", "title": "外の契約", "status": "closed", "issue_type": "task"},
{"id": "s9-far.3", "title": "外の問い", "status": "closed", "issue_type": "task", "labels": ["intake:question"],
 "notes": "見本の字\n裁定 id = s9-far.3:20260928T0500Z-1・R-1 を発効する\n方針 id = s9-far-p1・範囲 = all"},
{"id": "s9-far.4", "title": "外の memo", "status": "open", "issue_type": "task"},
{"id": "s9-lone", "title": "子の無い根", "status": "open", "issue_type": "epic"}
]"#;

const RULING: &str = "s9-far.3:20260928T0500Z-1";

/// 外の台帳の読み（読めることも確かめる）。
fn read_outside(text: &str) -> Outside {
    outside(text).unwrap_or_else(|| panic!("外の台帳を読む: {text}"))
}

fn far() -> Outside {
    read_outside(FAR)
}

fn row(ruling: &str, form: &str, bead: &str, node: &str, file: &str) -> String {
    format!(
        "{{\"ruling\":\"{ruling}\",\"form\":\"{form}\",\"bead\":\"{bead}\",\"node\":\"{node}\",\"file\":\"{file}\",\"line\":9,\"field\":\"approval.ruling\"}}\n"
    )
}

/// 書き出しに、族 s9-far を引く 3 つの形の行（ADR-8 は bead の形・R-1 は問いの形・FR1 は notes の日時の形）を足した字。
fn plus_far(rulings: &str) -> String {
    format!(
        "{rulings}{}{}{}",
        row("s9-far.2", "bead", "s9-far.2", "ADR-8", "adr/ADR-8.yaml"),
        row(RULING, "question", "s9-far.3", "R-1", "rules.yaml"),
        row(
            "s9-far.4 notes 2026-09-28 10:0x JST",
            "notes-time",
            "s9-far.4",
            "FR1",
            "srs.yaml"
        ),
    )
}

/// 書き出しに、族 s7-gone を引く P-1 の行を足した字。
fn plus_gone(rulings: &str) -> String {
    format!(
        "{rulings}{}",
        row("s7-gone.1", "bead", "s7-gone.1", "P-1", "constitution.yaml")
    )
}

/// 結んだグラフに外の台帳の読みを置く（書き出しを結べることも確かめる）。
fn placed(ledger: &str, rulings: &str, outside: Vec<Option<Outside>>) -> Graph {
    let mut g = build(&Inputs {
        design_index: &read("index.tsv"),
        ledger,
        events: &read("events.jsonl"),
    });
    assert!(add_rulings(&mut g, rulings), "書き出しを結ぶ");
    g.outside = outside;
    g
}

fn verdict_of(g: &Graph, id: &str) -> Verdict {
    check(g)
        .into_iter()
        .find(|i| i.id == id)
        .unwrap_or_else(|| panic!("不変条件 {id}"))
        .verdict
}

fn violation(ids: &[&str]) -> Verdict {
    Verdict::Violation(ids.iter().map(|s| s.to_string()).collect())
}

fn heads(n: usize, families: &[&str]) -> Option<(usize, Vec<String>)> {
    Some((n, families.iter().map(|s| s.to_string()).collect()))
}

fn set(items: &[&str]) -> std::collections::BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn gext_outside_reads_ids() {
    let got = far();
    assert_eq!(got.families, set(&["s9-far", "s9-lone"]));
    assert_eq!(
        got.beads,
        set(&["s9-far", "s9-far.2", "s9-far.3", "s9-far.4", "s9-lone"])
    );
    assert_eq!(got.rulings, set(&[RULING]));

    assert_eq!(outside("[]"), Some(Outside::default()));
    assert_eq!(outside(" [ ] \n"), Some(Outside::default()));
    for bad in [
        "",
        " \n\t ",
        "見本の字",
        "{\"id\": \"s9-far\"}",
        "[{\"title\": \"id の無い行\"}]",
        "[{\"id\": \"s9-far\"}, {\"title\": \"id の無い行\"}]",
        "[{\"id\": \"s9-far\"},",
    ] {
        assert_eq!(outside(bad), None, "{bad}");
    }

    // 知らない欄は読み捨てる。notes の無い bead は裁定を持たない。
    let plain = read_outside(
        "[{\"id\": \"s9-far.1\", \"status\": \"open\", \"dependencies\": [{\"depends_on_id\": \"x\"}], \"metadata\": {}}]",
    );
    assert_eq!(plain.beads, set(&["s9-far.1"]));
    assert_eq!(plain.families, set(&["s9-far"]));
    assert!(plain.rulings.is_empty());

    ruling_forms_held(got);
}

/// 外の台帳の読みが持つ裁定の行と持たない行（定型行・問いと notes の日時と bead の形）。
fn ruling_forms_held(got: Outside) {
    // 受けと方針の定型行は裁定の id に数えない・問いの形と notes の日時の形と bead の形の先の分け。
    let g = placed(&read("ledger.json"), &plus_far(&read("rulings.jsonl")), vec![]);
    let rows = &g.rulings.as_ref().expect("結んだ表");
    let one = |node: &str, n: usize| rows[node][n].clone();
    assert!(got.holds(&one("ADR-8", 1)));
    assert!(got.holds(&one("R-1", 2)));
    assert!(got.holds(&one("FR1", 0)));
    let mut miss = one("R-1", 2);
    miss.ruling = "s9-far-p1".to_string();
    assert!(!got.holds(&miss));
    miss.form = tsuzuri_core::graph::RulingForm::Bead;
    miss.bead = "s9-far.3".to_string();
    assert!(got.holds(&miss));
}

#[test]
fn gext_rows_follow_outside() {
    let rulings = plus_far(&read("rulings.jsonl"));
    let ledger = read("ledger.json");

    // 外の台帳の読みが無ければ確かめられない。
    let g = placed(&ledger, &rulings, vec![]);
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Unknown);
    assert_eq!(outside_heads(&g), heads(3, &["s9-far"]));

    // 読めない組と字 [] の組をその前に置いても、s9-far を持つ組で確かめられる。
    for lead in [vec![], vec![None, Some(read_outside("[]"))]] {
        let mut outs = lead;
        outs.push(Some(far()));
        let g = placed(&ledger, &rulings, outs);
        assert_eq!(verdict_of(&g, "g-3"), Verdict::Pass);
        assert_eq!(verdict_of(&g, "g-7"), Verdict::Pass);
        assert_eq!(outside_heads(&g), None);
        assert_eq!(outside_rulings(&g), None);
    }

    // 外の台帳の先が無ければ、その行の節点を名指す。
    let cases = [
        (FAR.replace("\"s9-far.2\"", "\"s9-far.9\""), "ADR-8"),
        (FAR.replace("裁定 id = ", "見本 id = "), "R-1"),
        (FAR.replace("\"s9-far.4\"", "\"s9-far.8\""), "FR1"),
    ];
    for (text, node) in cases {
        assert_ne!(text, FAR, "{node}: 字を替えた");
        let g = placed(&ledger, &rulings, vec![Some(read_outside(&text))]);
        assert_eq!(verdict_of(&g, "g-3"), violation(&[node]), "{node}");
        assert_eq!(outside_heads(&g), None, "{node}");
    }

    unconfirmed_and_gone(ledger, rulings);
}

/// 外の台帳で確かめられない組と、族 s7-gone を引く行を足した書き出しの g-3 と先。
fn unconfirmed_and_gone(ledger: String, rulings: String) {
    // 読めない組・字 [] の組・族の違う組だけでは確かめられない。
    let near = FAR.replace("s9-far", "s8-near");
    let others = [
        vec![None],
        vec![Some(read_outside("[]"))],
        vec![Some(read_outside(&near))],
    ];
    for outs in others {
        let g = placed(&ledger, &rulings, outs);
        assert_eq!(verdict_of(&g, "g-3"), Verdict::Unknown);
        assert_eq!(outside_heads(&g), heads(3, &["s9-far"]));
    }

    // 族 s7-gone を引く P-1 の行を足す。
    let more = plus_gone(&rulings);
    let g = placed(&ledger, &more, vec![Some(far())]);
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Unknown);
    assert_eq!(outside_heads(&g), heads(1, &["s7-gone"]));
    let g = placed(&ledger, &more, vec![]);
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Unknown);
    assert_eq!(outside_heads(&g), heads(4, &["s7-gone", "s9-far"]));
}

#[test]
fn gext_own_ledger_still_decides() {
    let rulings = plus_gone(&plus_far(&read("rulings.jsonl")));
    let q4 = "裁定 id = q7-fx.4:20260928T0100Z-1・ADR-7 を発効する";
    let cut = read("ledger.json").replace(q4, "見本の字");
    assert_ne!(cut, read("ledger.json"), "台帳の字を替えた");
    let g = placed(&cut, &rulings, vec![Some(far())]);
    assert_eq!(verdict_of(&g, "g-3"), violation(&["ADR-7"]));
    assert_eq!(outside_heads(&g), None);

    // 自分の台帳が空の字なら読めない出所が在り、外の台帳を見ない。
    let g = placed("", &rulings, vec![Some(far())]);
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Unknown);
    assert_eq!(outside_heads(&g), None);

    // 書き出しを結ばないグラフも同じ。
    let mut g = build(&Inputs {
        design_index: &read("index.tsv"),
        ledger: &read("ledger.json"),
        events: &read("events.jsonl"),
    });
    g.outside = vec![Some(far())];
    assert_eq!(g.rulings, None);
    assert_eq!(verdict_of(&g, "g-3"), Verdict::Unknown);
    assert_eq!(outside_heads(&g), None);
    assert_eq!(check::outside_heads(&Graph::default()), None);
}

#[test]
fn gext_names_clean() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut names: Vec<String> = Vec::new();
    for file in [
        dir.join("tests/teeth1/gext.rs"),
        dir.join("../tsuzuri-boundary/tests/teeth2/gextw.rs"),
    ] {
        let text = fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("{} を読む: {e}", file.display()));
        let lines: Vec<&str> = text.lines().collect();
        names.extend(
            lines
                .windows(2)
                .filter(|w| w[0].trim() == "#[test]")
                .map(|w| {
                    let rest = w[1].trim().strip_prefix("fn ").expect("fn の行");
                    rest.split('(').next().expect("fn の名").to_string()
                }),
        );
    }
    assert_eq!(names.len(), 6, "{names:?}");
    const PREFIX: &str = "gext_";
    assert_eq!(FILTERS.len(), 227);
    for word in FILTERS {
        assert!(!PREFIX.contains(word), "接頭辞が {word} に含まれる");
        assert!(!word.contains(PREFIX), "{word} が接頭辞を含む");
    }
    for name in &names {
        let rest = name
            .strip_prefix(PREFIX)
            .unwrap_or_else(|| panic!("{name} は {PREFIX} で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
