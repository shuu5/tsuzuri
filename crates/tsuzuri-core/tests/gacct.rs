//! 行 c-grace-acct の歯（中核）: account board の project の行の退避の終わる時刻 move_until は、席の card の欄 move_to が
//! 口座で grace_until が時刻のときだけその時刻の写し（器の残り秒に組んだ今を足した値・0 は今そのもの・行 c-abs-seat）。
//! 中核と境界の account の読みは猶予の rules 行と合図の file を読まず、語の辞書の鍵 move_grace の説明は器の字に直す。
//! fixture: tests/fixtures/account/acct-inputs.json（読むだけ）。器の §20 の形の欄は歯の中で組む。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use tsuzuri_contract::board::Reading;
use tsuzuri_core::account::host::HostTexts;
use tsuzuri_core::account::project::{ProjectTexts, doc, project_rows};

const INPUTS: &str = "tests/fixtures/account/acct-inputs.json";

/// 今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// proj-a の席の合図の健康の行（器の §20 の欄の前まで）。
const LINE: &str = "seat tick status: target=proj-a:0.1 last=1790510390 age=10 healthy=yes heartbeat=on step=5 next=1790510395";

/// 行の末尾の move= と grace_left= の組と、期待の残り秒（8 つ）。
const PAIRS: [(&str, Option<u64>); 8] = [
    ("reopens=- move=acct-2 grace_left=300", Some(300)),
    ("reopens=- move=acct-2 grace_left=1", Some(1)),
    ("reopens=- move=acct-2 grace_left=0", Some(0)),
    ("reopens=- move=acct-2 grace_left=-", None),
    ("reopens=- move=- grace_left=-", None),
    ("reopens=- move=- grace_left=120", None),
    ("reopens=- move=unreadable grace_left=unreadable", None),
    ("reopens=-", None),
];

#[derive(Deserialize)]
struct Inputs {
    texts: HostTexts,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn host() -> HostTexts {
    serde_json::from_str::<Inputs>(&read(INPUTS))
        .expect("fixture の形")
        .texts
}

/// proj-a の字（合図の健康の行の末尾に `tail`）。
fn projects(tail: &str) -> BTreeMap<String, ProjectTexts> {
    let p = ProjectTexts {
        state_dir_known: true,
        tick_status: Some(format!("{LINE} {tail}\n")),
        state_log: Some("{\"state\":\"busy\",\"ts\":1790505060}\n".into()),
        ..ProjectTexts::default()
    };
    BTreeMap::from([("/work/proj-a".to_string(), p)])
}

#[test]
fn gacct_left_copies_card() {
    let base = host();
    // 群の今の記録（fixture の ts は今）・古い ts の記録・記録の無い host。
    let mut old = base.clone();
    old.records.insert(
        "main.account".into(),
        "account=acct-2\nts=2026-09-20T00:00:00Z\nreason=account-pressed\nprevious=acct-1\n".into(),
    );
    let mut none = base.clone();
    none.records.remove("main.account");
    for (tail, want) in PAIRS {
        let ps = projects(tail);
        for h in [&base, &old, &none] {
            for now in [NOW, NOW + 60, NOW + 3_600] {
                let rows = project_rows(h, &ps, now);
                let a = rows
                    .iter()
                    .find(|r| r.name == "proj-a")
                    .expect("proj-a の行");
                let until = want.map(|s| now + s);
                assert_eq!(a.move_until, until, "{tail} と今 {now}");
                let Reading::Known(card) = &a.seat else {
                    panic!("{tail}: proj-a の席の card が読めない");
                };
                if want.is_some() {
                    assert_eq!(card.grace_until, Reading::Known(until), "{tail}");
                }
                // doc の projects は同じ入力の project_rows と同じ。
                assert_eq!(doc(h, &ps, now).projects, rows, "{tail} と今 {now}");
            }
        }
    }
    // 席の行の無い project（card が Unknown）は無し。
    let mut no_seat = base.clone();
    no_seat.seat_doctors.remove("/work/proj-a");
    let rows = project_rows(&no_seat, &projects(PAIRS[0].0), NOW);
    assert!(rows.iter().all(|r| r.move_until.is_none()), "{rows:?}");
}

#[test]
fn gacct_no_own_reads() {
    let core = read("crates/tsuzuri-core/src/account/project.rs");
    for word in [
        "grace_secs",
        "move_signal",
        "move-signal",
        "checked_add(grace",
        "record_value",
    ] {
        assert!(!core.contains(word), "中核の project.rs に {word} が在る");
    }
    for word in ["fn move_until(card", "&Reading<SeatCard>) -> Option<EpochSecs>"] {
        assert!(core.contains(word), "中核の project.rs に {word} が無い");
    }
    let acct = read("crates/tsuzuri-boundary/src/acct.rs");
    for word in [
        "move_grace_s",
        "move-signal",
        "MOVE_SIGNAL",
        "GRACE_ARGS",
        "grace",
    ] {
        assert!(!acct.contains(word), "境界の acct.rs に {word} が在る");
    }
}

#[test]
fn gacct_vocab_origin() {
    let vocab: Value =
        serde_json::from_str(&read("crates/tsuzuri-surface/vocab.json")).expect("語の辞書の形");
    let entry = vocab
        .as_object()
        .expect("語の辞書の表")
        .values()
        .find_map(|v| v.get("move_grace"))
        .expect("鍵 move_grace");
    let text = |key: &str| {
        entry
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("move_grace の {key}"))
            .to_string()
    };
    assert_eq!(text("label"), "退避までの残り");
    let (plain, internal) = (text("plain"), text("internal"));
    for word in ["・ 起点 = 合図の at（器が送った時刻）", "・ 値 = 器の grace_left の写し"] {
        assert!(plain.contains(word), "plain に {word} が無い: {plain}");
    }
    for word in ["群の記録の ts", "move-signal", "1800"] {
        assert!(!plain.contains(word), "plain に {word} が在る: {plain}");
        assert!(!internal.contains(word), "internal に {word} が在る: {internal}");
    }
}

/// 起草の時の main（e28315b）の契約表の verify の filter の語 210 語と sgrace_（歯の名の見張りの WORDS と同じ 211 語）。
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
    "sgrace_",
];

#[test]
fn gacct_own_names_clean() {
    assert_eq!(WORDS.len(), 211);
    let text = read("crates/tsuzuri-core/tests/gacct.rs");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<String> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の次の行は fn");
            rest[..rest.find('(').expect("fn の名の後に (")].to_string()
        })
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("gacct_")
            .unwrap_or_else(|| panic!("{name} は gacct_ で始まる"));
        for w in WORDS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
