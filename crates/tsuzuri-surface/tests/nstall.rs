//! 次の一手の判じなかったなしの箱の歯（行 c-next-stall・接頭辞 nstall_）。
//! lead がなしで電文のなしの結果が判じなかったなら、大きい箱は測れていないの箱（`UNJUDGED_KEY`・`UNJUDGED_LINE`）。
//! 面は判じない: 電文の結果を写すだけ。fixture: tests/fixtures/surface/next-step.json。

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::NextMove;
use tsuzuri_contract::stats::{CheckResult, NextStep};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::next::{
    Big, KEYS, Mark, NONE_LINE, UNJUDGED_KEY, UNJUDGED_LINE, big, next, unjudged,
};
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn sets() -> BTreeMap<String, NextStep> {
    wire::decode(&read("../../tests/fixtures/surface/next-step.json"))
        .expect("fixture の組が電文として読める")
}

fn set(name: &str) -> NextStep {
    sets()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"))
}

/// (5) なしを判じなかった電文だけが測れていないの箱になり、ほかの箱は big に lead とその結果を渡した値のまま。
#[test]
fn nstall_unjudged_box() {
    // nothing の組（なしが当たる）は今のなしの箱。
    let nothing = set("nothing");
    assert!(!unjudged(&nothing));
    let n = next(&nothing);
    assert_eq!((n.big.key, n.big.what.as_str()), ("nx_g", NONE_LINE));

    // 限度と移動となしを判じなかった電文は測れていないの箱。
    let mut open = nothing.clone();
    for c in &mut open.checks {
        if matches!(c.kind, NextMove::LimitOrMove | NextMove::Nothing) {
            c.result = CheckResult::NotJudged;
        }
    }
    assert!(unjudged(&open));
    let n = next(&open);
    assert_eq!(
        n.big,
        Big {
            kind: NextMove::Nothing,
            key: UNJUDGED_KEY,
            class: "nxbig none",
            what_class: "what small muted",
            what: UNJUDGED_LINE.to_string(),
            link: None,
            target: None,
        }
    );
    let rows: Vec<(&str, &str, Mark)> = n.rest.iter().map(|r| (r.key, r.class, r.mark)).collect();
    assert_eq!(
        rows,
        vec![
            ("nx_a", "off", Mark::Unmeasured),
            ("nx_b", "off", Mark::Miss),
            ("nx_c", "off", Mark::Miss),
            ("nx_d", "off", Mark::Miss),
            ("nx_e", "off", Mark::Miss),
            ("nx_f", "off", Mark::Miss),
        ]
    );

    // なしの結果が電文に無ければ判じなかったと言わない。
    let bare = NextStep {
        checks: Vec::new(),
        lead: NextMove::Nothing,
    };
    assert!(!unjudged(&bare));
    assert_eq!(next(&bare).big.what, NONE_LINE);

    // lead がなしでない組は今のまま。
    for name in ["stalled", "question", "not-judged"] {
        let s = set(name);
        assert!(!unjudged(&s), "{name}");
        let lead = s.checks.iter().find(|c| c.kind == s.lead);
        assert_eq!(next(&s).big, big(s.lead, lead), "{name}");
    }

    // big 自身は変えない（なしを判じなかった結果を渡しても今のなしの箱）。
    let unjudged_check = open
        .checks
        .iter()
        .find(|c| c.kind == NextMove::Nothing)
        .expect("なしの結果");
    let b = big(NextMove::Nothing, Some(unjudged_check));
    assert_eq!((b.key, b.what.as_str()), ("nx_g", NONE_LINE));

    // 見出しの語は語の辞書の「測れていない」で、中身の字はどの鍵の見出しの語とも違う。
    let v = vocab();
    assert_eq!(
        v.term(UNJUDGED_KEY).map(|t| t.label.as_str()),
        Some("測れていない")
    );
    for key in v.keys() {
        let label = v.term(key).map(|t| t.label.as_str());
        assert_ne!(label, Some(UNJUDGED_LINE), "鍵 {key}");
    }
    assert!(KEYS.iter().all(|(_, k)| *k != UNJUDGED_KEY));
}

/// 着地済みの行の verify の filter の語（106 語・接頭辞 nstall_ は並べない）。
const FILTERS: [&str; 106] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
    "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_",
    "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_",
    "pmore_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "hcard_", "qgate_",
    "nsum_", "ntime_", "ptitle_", "ncard_", "hsym_", "smore_", "lcard_", "aaround_", "hruling_",
    "sxaxis_", "plimit_", "bport_", "fmark_", "fstop_", "fserve_", "nsumw_", "cadopt_", "tipx_",
    "hcsess_", "hcproj_", "sesplit_", "mstore_", "athr_", "qblock_", "wsteady_", "gsum_",
    "pfold_", "uword_", "cround_", "cgdom_", "csled_", "lhome_", "shb_", "ghb_", "nact_",
    "aord_", "mtree_",
];

/// (7) この file の歯の名はどれも nstall_ で始まり、残りの字は filter の語を含まない。
#[test]
fn nstall_face_names_apart() {
    let text = read("tests/nstall.rs");
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
    assert!(names.len() >= 2, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("nstall_")
            .unwrap_or_else(|| panic!("{name} が nstall_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
