//! 行 c-pipe-ci の歯（面）: 札の欄 ci（中核が判じた着地の後の CI の読み）を札の語と印と列に出す。
//! 札は歯の中で組む（fixture の file は使わない・bead の id の接頭辞は fx-cm）。

use std::path::PathBuf;

use tsuzuri_contract::board::{Ci, PipelineBoard, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::pipeline::{
    CI_KEYS, CI_MARK_S, CI_WAIT_STATE, CLOSED_STAGE, Lead, ci_key, ci_shown, ci_style, content,
    kcard,
};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

/// 節の今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 回数 1・口座 None の札。
fn card(
    id: &str,
    stage: Stage,
    reason: Option<&str>,
    elapsed: Option<u64>,
    ci: Option<Ci>,
) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("bead の id"),
        runs: 1,
        stage,
        reason: reason.map(str::to_string),
        account: None,
        since: elapsed.map(|e| NOW - e),
        ci,
    }
}

fn src() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/project/pipeline.rs");
    std::fs::read_to_string(&path).expect("src/project/pipeline.rs を読む")
}

/// (7) 語の辞書の CI の 7 つの鍵と、読みと鍵の対。
#[test]
fn pci_vocab_words() {
    let want = [
        (Ci::Waiting, "ci_wait", "CI 中"),
        (Ci::Success, "ci_success", "CI 成功"),
        (Ci::Failure, "ci_failure", "CI 失敗"),
        (Ci::Unmeasurable, "ci_unmeasurable", "CI 測れず"),
        (Ci::PushFailed, "ci_push_failed", "push 失敗"),
        (Ci::CloseFailed, "ci_close_failed", "close 失敗"),
        (Ci::Unreadable, "ci_unreadable", "終端が読めない"),
    ];
    let pairs: Vec<(Ci, &str)> = want.iter().map(|(c, k, _)| (*c, *k)).collect();
    assert_eq!(CI_KEYS.to_vec(), pairs);
    let source = tsuzuri_surface::vocab::SOURCE;
    let english = source.find("\"english\"").expect("欄 english");
    let rephrase = source.find("\"rephrase\": {").expect("欄 rephrase");
    let mut at = source.find("\"col_land\"").expect("鍵 col_land");
    assert!(english < at && at < rephrase);
    for (ci, key, text) in want {
        assert_eq!(ci_key(ci), key);
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が語の辞書に無い"));
        assert_eq!(term.label, text);
        assert_ne!(term.label, CLOSED_STAGE);
        assert!(!term.note.is_empty(), "{key} の注");
        assert!(!term.internal.is_empty(), "{key} の内部の名");
        let next = source
            .find(&format!("\"{key}\": {{"))
            .unwrap_or_else(|| panic!("鍵 {key} の字"));
        assert!(at < next && next < rephrase, "{key} の置き場");
        at = next;
    }
    assert_eq!(CI_MARK_S, 600);
    let success = vocab().term("ci_success").expect("鍵 ci_success");
    assert!(success.note.contains("10 分"), "{}", success.note);
    assert_eq!(CI_WAIT_STATE, "run");
}

/// (9) 札に出す読みは CI を待つ札と止まった列の札ではいつも、ほかは経過が CI_MARK_S 以下のときだけ。
#[test]
fn pci_shown_rule() {
    use Ci::*;
    use Stage::*;
    let cases = [
        (Landed, Some(172_800), Some(Waiting), Some(Waiting)),
        (Landed, None, Some(Waiting), Some(Waiting)),
        (Landed, Some(600), Some(Success), Some(Success)),
        (Landed, Some(601), Some(Success), None),
        (Landed, None, Some(Success), None),
        (Landed, Some(30), Some(Failure), Some(Failure)),
        (Landed, Some(3600), Some(Failure), None),
        (Failed, Some(172_800), Some(Failure), Some(Failure)),
        (Stopped, None, Some(Unmeasurable), Some(Unmeasurable)),
        (Landed, Some(5), None, None),
        (Failed, Some(5), None, None),
    ];
    for (stage, elapsed, ci, want) in cases {
        let c = card("fx-cm.1", stage, None, elapsed, ci);
        assert_eq!(ci_shown(&c, NOW), want, "{stage:?} {elapsed:?} {ci:?}");
    }
}

/// (10) 札の状態の記号・欄 ci・lead・値の行。
#[test]
fn pci_kcard_marks() {
    let k = |stage, reason, elapsed, ci| kcard(&card("fx-cm.1", stage, reason, elapsed, ci), &[], NOW);

    let wait = k(Stage::Landed, None, Some(120), Some(Ci::Waiting));
    assert_eq!(wait.state, Some("run"));
    assert_eq!(wait.ci, Some(Ci::Waiting));
    assert_eq!(wait.class, "kcard");
    assert_eq!(wait.lead, Lead::Runs(1));
    assert_eq!(wait.run_line, "↻1 · Landed · CI 中");
    assert_eq!(wait.hover.kind, "run · Landed");
    assert_eq!(wait.age, "2m");
    assert!(!wait.closed);
    assert!(wait.run_more.is_empty());

    let ok = k(Stage::Landed, None, Some(600), Some(Ci::Success));
    assert_eq!(ok.state, None);
    assert_eq!(ok.ci, Some(Ci::Success));
    assert_eq!(ok.run_line, "↻1 · Landed · CI 成功");

    let late = k(Stage::Landed, None, Some(601), Some(Ci::Success));
    assert_eq!(late, k(Stage::Landed, None, Some(601), None));
    assert_eq!(late.ci, None);
    assert_eq!(late.run_line, "↻1 · Landed · Landed");

    let fail = k(
        Stage::Failed,
        Some("terminal:ci:failure"),
        Some(7200),
        Some(Ci::Failure),
    );
    assert_eq!(fail.state, Some("wait"));
    assert_eq!(fail.ci, None);
    assert_eq!(fail.class, "kcard why-stop");
    assert_eq!(fail.lead, Lead::Why("CI 失敗".to_string()));
    assert_eq!(fail.run_line, "↻1 · Failed · CI 失敗");

    let unmeasured = k(
        Stage::Stopped,
        Some("terminal:ci:unmeasurable"),
        None,
        Some(Ci::Unmeasurable),
    );
    assert_eq!(unmeasured.lead, Lead::Why("CI 測れず".to_string()));
    assert_eq!(unmeasured.run_line, "↻1 · Stopped · CI 測れず");
    assert_eq!(unmeasured.age, "―");

    for (stage, reason, ci, text) in [
        (
            Stage::Failed,
            "terminal:push:failed:git",
            Ci::PushFailed,
            "push 失敗",
        ),
        (
            Stage::Failed,
            "terminal:close:failed:rc=1 x",
            Ci::CloseFailed,
            "close 失敗",
        ),
        (
            Stage::Stopped,
            "terminal:unreadable",
            Ci::Unreadable,
            "終端が読めない",
        ),
    ] {
        let c = k(stage, Some(reason), Some(60), Some(ci));
        assert_eq!(c.ci, None, "{ci:?}");
        assert_eq!(c.class, "kcard why-stop", "{ci:?}");
        assert_eq!(c.lead, Lead::Why(text.to_string()), "{ci:?}");
        assert_eq!(c.run_line, format!("↻1 · {stage:?} · {text}"));
    }

    let landed_fail = k(Stage::Landed, None, Some(30), Some(Ci::Failure));
    assert_eq!(landed_fail.state, None);
    assert_eq!(landed_fail.ci, Some(Ci::Failure));
    assert_eq!(landed_fail.lead, Lead::Runs(1));
    assert_eq!(landed_fail.run_line, "↻1 · Landed · CI 失敗");
}

/// (11) CI を待つ札は日を問わず Landed の列に出る。
#[test]
fn pci_columns_keep_waiting() {
    let cards = vec![
        card("fx-cm.1", Stage::Landed, None, Some(172_800), Some(Ci::Waiting)),
        card("fx-cm.2", Stage::Landed, None, Some(172_800), None),
        card("fx-cm.3", Stage::Landed, None, Some(172_800), Some(Ci::Success)),
        card("fx-cm.4", Stage::Landed, None, Some(60), Some(Ci::Success)),
        card(
            "fx-cm.5",
            Stage::Failed,
            Some("terminal:ci:failure"),
            Some(172_800),
            Some(Ci::Failure),
        ),
        card("fx-cm.6", Stage::Landed, None, None, Some(Ci::Waiting)),
    ];
    let text = wire::encode(&PipelineBoard {
        cards: Reading::Known(cards),
        misfits: Reading::Known(vec![]),
    })
    .expect("板の電文");
    let Body::Filled(cols) = content(&Fetched::Body(text), &Fetched::NotRead, NOW) else {
        panic!("板が Filled でない");
    };
    let ids: Vec<Vec<&str>> = cols
        .iter()
        .map(|c| c.cards.iter().map(|k| k.id.as_str()).collect())
        .collect();
    let empty: Vec<&str> = Vec::new();
    assert_eq!(
        ids,
        [
            empty.clone(),
            empty,
            vec!["fx-cm.5"],
            vec!["fx-cm.4", "fx-cm.1", "fx-cm.6"],
        ]
    );
}

/// (12) 読みの語の style と、札の DOM の meta の字の順。
#[test]
fn pci_dom_text() {
    assert_eq!(ci_style(Ci::Waiting), "color:var(--s-run)");
    assert_eq!(ci_style(Ci::Success), "color:var(--s-land)");
    for ci in [
        Ci::Failure,
        Ci::Unmeasurable,
        Ci::PushFailed,
        Ci::CloseFailed,
        Ci::Unreadable,
    ] {
        assert_eq!(ci_style(ci), "color:var(--s-stop);font-weight:600", "{ci:?}");
    }
    let text = src();
    let start = text.find("fn kcard_view").expect("fn kcard_view");
    let view = &text[start..];
    assert!(view.contains(".ci\n"));
    assert!(view.contains("<span style=ci_style(c)>{label(ci_key(c))}</span>"));
    let at = |s: &str| view.find(s).unwrap_or_else(|| panic!("字 {s} が無い"));
    assert!(at("{closed}") < at("{ci}"));
    assert!(at("{ci}") < at("inner_html=CLOCK"));
}

/// verify の filter の語（main の 157 語と並行の起草の行の接頭辞 2 語）。
const FILTER_WORDS: [&str; 159] = [
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
];

/// (14) この file の歯の名は 6 つで、どれも pci_ で始まり、名の全体が filter の語を含まない。
#[test]
fn pci_surface_names_clean() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/pcimark.rs");
    let text = std::fs::read_to_string(&path).expect("tests/pcimark.rs を読む");
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
    assert_eq!(names.len(), 6, "{names:?}");
    for name in names {
        assert!(name.starts_with("pci_"), "{name} が pci_ で始まらない");
        for word in FILTER_WORDS {
            assert!(!name.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
