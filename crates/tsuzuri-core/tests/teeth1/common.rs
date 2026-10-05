//! tsuzuri-core の歯の群 teeth1 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;
use tsuzuri_contract::account::{AccountRow, GroupCard, MoveRow};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_core::account::host::HostTexts;
use tsuzuri_core::delivery::Said;
use tsuzuri_core::seat::SeatTexts;

#[derive(Deserialize)]
pub(crate) struct Case {
    #[serde(flatten)]
    pub(crate) texts: SeatTexts,
    pub(crate) card: SeatCard,
}

/// 偽の tz が標準出力に出す字。
pub(crate) const FAKE_OUT: &str = "answer-line\n";

/// 計画と contracts の verify の filter の語（ほかの語を部分の字として含まない語に畳んだ語と後の行の接頭辞）。
pub(crate) const FILTER_WORDS: &[&str] = &[
    "aaround_", "accept_", "acchold_", "account_", "acctcore_", "acctdoc_", "accthb_",
    "accthome_", "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_",
    "acctwire_", "afocus_", "aord_", "apop_", "askcard_", "athr_", "batchpanel_", "bhalf_",
    "board_min_", "bport_", "brand_", "btuck_", "cadopt_", "cdorm_", "cgdom_", "cmark_",
    "contract_form_", "cround_", "csled_", "denv_", "dnrow_", "ecache_", "epolq_", "flight_",
    "fmark_", "fprem_", "frame_", "fserve_", "fstop_", "gapspage_", "gbnote_", "gfix_",
    "gfresh_", "ghb_", "gjst_", "glabel_", "gnav_", "gpface_", "gpill_", "gpulse_", "graph_",
    "gsum_", "gtuck_", "gview_", "hacols_", "harest_", "hasplit_", "hbconf_", "hbmark_",
    "hbpost_", "hbproc_", "hbroute_", "hcard_", "hcled_", "hcnx_", "hcproj_", "hcsess_",
    "hdchip_", "hfig_", "hnunk_", "hook_", "hruling_", "hsblock_", "hsderive_", "hshist_",
    "hspage_", "hsym_", "iclose_", "ilink_", "kcli_", "klink_", "launch_", "lcard_",
    "ledgerblock_", "lhome_", "lidle_", "lresume_", "lsnap_", "lspark_", "lstore_", "mapview_",
    "mkeys_", "mlink_", "mstore_", "mtree_", "nact_", "nbatch_", "ncard_", "nextstep_",
    "nodepage_", "nstall_", "nsum_", "nsumw_", "ntime_", "nxact_", "parts_", "pclosed_",
    "pfold_", "pipe_", "plimit_", "pmisfit_", "pmore_", "pquest_", "pqueue_", "project_",
    "ptitle_", "punmap_", "pwhole_", "qblock_", "qgate_", "qkey_", "question_", "relay_",
    "rhold_", "runsdoc_", "rvk_", "saxis_", "sclosed_", "seatblock_", "seatcard_", "server_",
    "sesplit_", "shb_", "skeleton_", "smore_", "stage_", "stats_", "stcli_", "steady_",
    "sxaxis_", "ticker_", "tipx_", "topbar_", "tz_", "urpanel_", "uword_", "wstrip_", "pgz_",
];

pub(crate) const FIXTURE: &str = "tests/fixtures/account/acct-inputs.json";

pub(crate) const INPUTS: &str = "tests/fixtures/account/acct-inputs.json";

#[derive(Deserialize)]
pub(crate) struct Inputs {
    pub(crate) texts: HostTexts,
    pub(crate) accounts: Reading<Vec<AccountRow>>,
    pub(crate) groups: Reading<Vec<GroupCard>>,
    pub(crate) moves: Reading<Vec<MoveRow>>,
}

/// runner の印の file の名。
pub(crate) const MARK_NAME: &str = "scribe2-runner";

/// 今（2026-09-27T12:00:00Z）。
pub(crate) const NOW: u64 = 1_790_510_400;

/// 歯の台帳の bead 1 本（label は問いの label か無し）。
pub(crate) fn bead(id: &str, question_label: bool, status: &str, notes: &[&str]) -> Value {
    let labels: Vec<&str> = if question_label {
        vec!["intake:question"]
    } else {
        vec!["other"]
    };
    json!({"id": id, "labels": labels, "status": status, "notes": notes.join("\n")})
}

/// project の宣言（git config の scribe2.statedir）の在る repo の dir を作る。
pub(crate) fn declared(project: &Path) {
    std::fs::create_dir_all(project).expect("project の dir を作る");
    for args in [
        &["init", "-q"][..],
        &["config", "scribe2.statedir", "/nonexistent/state"],
    ] {
        let ok = Command::new("git")
            .arg("-C")
            .arg(project)
            .args(args)
            .status()
            .expect("git");
        assert!(ok.success(), "git {args:?}");
    }
}

pub(crate) fn fixture(name: &str) -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/case")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// object の鍵を並べ替えた列。
pub(crate) fn keys(v: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = v
        .as_object()
        .unwrap_or_else(|| panic!("object でない: {v}"))
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

pub(crate) fn known<T: std::fmt::Debug>(r: Reading<T>) -> T {
    match r {
        Reading::Known(v) => v,
        Reading::Unknown => panic!("Unknown"),
    }
}

pub(crate) fn log(lines: &[String]) -> String {
    lines.iter().map(|l| format!("{l}\n")).collect()
}

/// 要素 1 つの配列のその要素。
pub(crate) fn only(v: &Value) -> &Value {
    match v.as_array().map(Vec::as_slice) {
        Some([one]) => one,
        _ => panic!("要素 1 つの配列でない: {v}"),
    }
}

pub(crate) fn pair(from: &str, to: &str) -> (String, String) {
    (from.to_string(), to.to_string())
}

/// plugin の形の dir（workspace の根の plugin の bin/tzw と plugin.json を写す・tzw は dir の親の target/debug/tz を引く）。
pub(crate) fn plugin_dir(dir: &Path) {
    for rel in ["bin/tzw", ".claude-plugin/plugin.json"] {
        let to = dir.join(rel);
        std::fs::create_dir_all(to.parent().expect("親の dir")).expect("plugin の dir を作る");
        std::fs::copy(root().join("plugin").join(rel), &to).expect("plugin の file を写す");
    }
}

pub(crate) fn read_hooks() -> Value {
    let path = root().join("plugin/hooks/hooks.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("hooks.json を読む: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("hooks.json は JSON でない: {e}"))
}

pub(crate) fn rid(id: &str) -> RulingId {
    RulingId::new(id).expect("裁定の id")
}

pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(crate) fn said_of(question: &str, id: &str, verbatim: &str) -> Said {
    Said {
        question: BeadId::new(question).expect("bead の id"),
        ruling: rid(id),
        verbatim: verbatim.to_string(),
    }
}
