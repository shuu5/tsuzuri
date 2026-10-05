//! 行 c-ledger-stage の歯: pipeline の板の札の段の字（`stage_word`）と、札の段の記号を描く所の字。
//! 台帳の一覧の項に出す段（`stages` と段つきの項）は行 g-list-sweep で歯ごと外した。
//! 札は歯の中で組む（fixture の file は使わない・bead の id の接頭辞は fx-l）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::Path;

use crate::common::{crate_dir, read};
use tsuzuri_contract::board::{Ci, PipelineCard, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_surface::project::pipeline::{CLOSED_STAGE, stage_word};

/// 札を描く今（札の since はこの今から経過を引いた時刻）。
const NOW: u64 = 1_790_510_400;

/// 口座 None の札。
fn card(
    id: &str,
    runs: u32,
    (stage, reason): (Stage, Option<&str>),
    elapsed: Option<u64>,
    ci: Option<Ci>,
) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("bead の id"),
        runs,
        stage,
        reason: reason.map(str::to_string),
        account: None,
        since: elapsed.map(|e| NOW - e),
        ci,
    }
}

/// 節の板 LS（8 枚）。
fn ls() -> Vec<PipelineCard> {
    vec![
        card("fx-l.1", 1, (Stage::Running, None), Some(40), None),
        card("fx-l.2", 0, (Stage::Queued, None), None, None),
        card("fx-l.3", 1, (Stage::Failed, Some("verdict:FAIL x")), Some(90), None),
        card("fx-l.4", 1, (Stage::Landed, None), Some(120), Some(Ci::Waiting)),
        card("fx-l.5", 1, (Stage::Landed, None), Some(30), Some(Ci::Success)),
        card("fx-l.6", 1, (Stage::Landed, None), Some(30), None),
        card("fx-l.7", 1, (Stage::Landed, Some("closed:gave up")), Some(30), None),
        card(
            "fx-l.8",
            1,
            (Stage::Failed, Some("terminal:ci:failure")),
            Some(30),
            Some(Ci::Failure),
        ),
    ]
}

/// (1) 札の段の字は閉じた札なら CLOSED_STAGE・ほかは段の名で、札の hover の kind と同じ字。
#[test]
fn lstg_word_rules() {
    let want = [
        "Running",
        "Queued",
        "Failed",
        "Landed",
        "Landed",
        "Landed",
        "閉じた（着地せず）",
        "Failed",
    ];
    assert_eq!(CLOSED_STAGE, want[6]);
    let cards = ls();
    assert_eq!(cards.len(), want.len());
    for (c, w) in cards.iter().zip(want) {
        let word = stage_word(c);
        assert_eq!(word, w, "{}", c.contract);
    }
}

/// 字 `mod dom {` より後の `start` から、次の 4 つの空白と閉じ波括弧だけの行までの字。
fn dom_fn<'a>(src: &'a str, start: &str) -> &'a str {
    let at = src.find("mod dom {").expect("mod dom の字");
    let dom = &src[at..];
    let from = dom
        .find(start)
        .unwrap_or_else(|| panic!("{start} の字が無い"));
    let rest = &dom[from..];
    let end = rest
        .find("\n    }\n")
        .unwrap_or_else(|| panic!("{start} の終わり"));
    &rest[..end]
}

/// src の下の .rs の file の相対の path と字（path の順）。
fn sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
        let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{dir:?} を読む: {e}"));
        for entry in entries {
            let path = entry.expect("dir の項").path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let rel = path
                    .strip_prefix(root)
                    .expect("src の下")
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = std::fs::read_to_string(&path).expect(".rs を読む");
                out.push((rel, text));
            }
        }
    }
    let root = crate_dir().join("src");
    let mut out = Vec::new();
    walk(&root, &root, &mut out);
    out.sort();
    out
}

/// 字の在りか（字が在る file ごとの数）。
fn places(files: &[(String, String)], needle: &str) -> BTreeMap<String, usize> {
    files
        .iter()
        .map(|(rel, text)| (rel.clone(), text.matches(needle).count()))
        .filter(|(_, n)| *n > 0)
        .collect()
}

/// (4) 札の段の記号は pipeline の stage_sym の 1 か所で描く（一覧の項の段の記号は行 g-list-sweep で外した）。
#[test]
fn lstg_dom_wiring() {
    let pipeline = read("src/project/pipeline.rs");
    let kcard_view = dom_fn(&pipeline, "fn kcard_view(");
    assert!(
        kcard_view.contains("let sym = stage_sym(card.closed, card.state);"),
        "{kcard_view}"
    );
    assert!(pipeline.contains("pub use dom::stage_sym;"));
    assert_eq!(pipeline.matches("inner_html=CROSS").count(), 1);
    assert_eq!(pipeline.matches("inner_html=CHECK").count(), 1);

    let needle = "pub fn stage_sym(closed: bool, state: Option<&'static str>) -> AnyView {";
    assert_eq!(
        places(&sources(), needle),
        BTreeMap::from([("project/pipeline.rs".to_string(), 1)]),
        "{needle}"
    );
}

/// verify の filter の語（main の 164 語に畳んだ語と後の行 g-gz の接頭辞・165 語）。
const FILTERS: [&str; 165] = [
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
    "nxorg_",
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
];

/// (6) この file の歯の名は 3 つで、どれも lstg_ で始まり、先頭の lstg_ を除いた字は filter の語を含まない。
#[test]
fn lstg_names_stay_apart() {
    let text = read("tests/teeth4/lstg.rs");
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
            .strip_prefix("lstg_")
            .unwrap_or_else(|| panic!("{name} が lstg_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
