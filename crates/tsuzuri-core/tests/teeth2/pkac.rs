//! 区画の席を区画の行の写しで持つ歯（中核・行 c-park-acct・接頭辞 pkac_）。
//! 区画の名の列（account の host の parks と電文の欄 parks）と、区画の anchor の席の card の group の欄 park と、
//! 次の一手の限度と移動が区画の口座を比べないこと、区画の席の移動の写しを測る。
//! 器の 2026-09-29 の出力の形（doctor の Tier1 の行と group=Tier9 kind=park の区画の行と席の行・seat tick status の
//! 欄 heartbeat_by= を持つ行・kind の key を持たない host.toml の区画の宣言）を歯の中で字に組む。口座と anchor と
//! target は架空の名。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::{GroupRow, NextMove, Reading};
use tsuzuri_contract::seat::{SeatCard, SeatState};
use tsuzuri_contract::stats::CheckResult;
use tsuzuri_core::account::host::{HostTexts, groups, parks};
use tsuzuri_core::account::project::doc;
use tsuzuri_core::next_step::next_step_seat;
use tsuzuri_core::seat::{SeatTexts, card};

/// 撃った今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 群の宣言（群 Tier1 と、kind の key を持たない区画 Tier9・区画の anchors は 3 つ）。
const HOST_TOML: &str = "[[account]]\nlabel = \"acct-1\"\n\n[[account]]\nlabel = \"acct-2\"\n\n[[account]]\nlabel = \"acct-3\"\n\n[[account-group]]\nname = \"Tier1\"\nanchors = [\"/work/proj-a\"]\naccounts = [\"acct-1\", \"acct-2\", \"acct-3\"]\n\n[[account-group]]\nname = \"Tier9\"\nanchors = [\"/work/proj-p1\", \"/work/proj-p2\", \"/work/proj-p3\"]\naccounts = [\"acct-1\", \"acct-2\", \"acct-3\"]\n";

/// 群 Tier1 の doctor の行。
const TIER1_ROW: &str = "group=Tier1 accounts=acct-1,acct-2,acct-3 anchors=1 seat-accounts=acct-1 current=acct-1 next=acct-2 refused=- pressure=-\n";

/// doctor の席の行（群の席 pa と区画の席 pp1）。
const SEATS: &str = "seat: role=orchestrator anchor=/work/proj-a target=pa:0.1 account=acct-1 model=opus pane=%1 heartbeat=on tick=healthy\nseat: role=orchestrator anchor=/work/proj-p1 target=pp1:0.1 account=acct-1 model=opus pane=%2 heartbeat=off tick=healthy\n";

/// 区画の席の名と anchor。
const PARK_SEAT: &str = "pp1:0.1";
const PARK_ANCHOR: &str = "/work/proj-p1";

/// 群の席の名と anchor。
const LOT_SEAT: &str = "pa:0.1";
const LOT_ANCHOR: &str = "/work/proj-a";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(root().join(path)).unwrap_or_else(|e| panic!("{path} を読む: {e}"))
}

/// 名と欄 kind（None なら欄の無い行）の区画の行。
fn park_row(name: &str, kind: Option<&str>) -> String {
    let kind = kind.map(|k| format!(" kind={k}")).unwrap_or_default();
    format!(
        "group={name}{kind} accounts=acct-1,acct-2,acct-3 anchors=3 seat-accounts=none current=- next=- refused=- pressure=-\n"
    )
}

/// 区画の行を差し込んだ doctor の字。
fn doctor(rows: &[String]) -> String {
    format!(
        "doctor: state dir ok\n{TIER1_ROW}{}account=acct-1 dir=/work/accounts/acct-1 retired=no\naccount=acct-2 dir=/work/accounts/acct-2 retired=no\naccount=acct-3 dir=/work/accounts/acct-3 retired=no\n{SEATS}",
        rows.concat()
    )
}

fn host(doctor: Option<String>, toml: Option<&str>) -> HostTexts {
    HostTexts {
        host_toml: toml.map(str::to_string),
        doctor,
        ..HostTexts::default()
    }
}

/// 席の card の材料（tail は tick の行の末の欄・last は合図の最後の判定の行）。
fn seat_texts(doctor: String, target: &str, tail: &str, last: &str) -> SeatTexts {
    SeatTexts {
        tick_status: Some(format!(
            "seat tick status: target={target} last=1790510390 age=10 healthy=yes heartbeat=off heartbeat_by=explicit step=5 next=1790510395 reopens=- {tail}\n"
        )),
        doctor: Some(doctor),
        state_log: Some("{\"state\":\"idle\",\"ts\":1790510300}\n".to_string()),
        tick_last: Some(format!("{last}\n")),
        host_toml: Some(HOST_TOML.to_string()),
        ..SeatTexts::default()
    }
}

const QUIET: &str = "move=- grace_left=-";
const CALM: &str = "ts=1790510390 reason=ok";
const PRESSED_LAST: &str = "ts=1790510390 reason=account-pressed";

fn park_card(kind: Option<&str>, tail: &str, last: &str) -> SeatCard {
    let d = doctor(&[park_row("Tier9", kind)]);
    card(
        PARK_SEAT,
        Some(PARK_ANCHOR),
        &seat_texts(d, PARK_SEAT, tail, last),
        NOW,
    )
}

fn known_row(c: &SeatCard) -> GroupRow {
    match &c.group {
        Reading::Known(g) => g.clone(),
        Reading::Unknown => panic!("group が Unknown"),
    }
}

fn names(r: Reading<Vec<tsuzuri_contract::account::GroupCard>>) -> Vec<String> {
    match r {
        Reading::Known(v) => v.into_iter().map(|g| g.row.group).collect(),
        Reading::Unknown => panic!("群の枠の列が Unknown"),
    }
}

fn json(v: &impl serde::Serialize) -> String {
    serde_json::to_string(v).expect("電文")
}

#[test]
fn pkac_doc_names_park() {
    let all = |rows: &[String]| host(Some(doctor(rows)), Some(HOST_TOML));
    let strs = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let map = BTreeMap::new();
    // 区画 Tier9 は parks に入り、群の枠の列は Tier1 だけ。
    let t = all(&[park_row("Tier9", Some("park"))]);
    assert_eq!(parks(&t), strs(&["Tier9"]));
    assert_eq!(names(groups(&t)), strs(&["Tier1"]));
    let d = doc(&t, &map, NOW);
    assert_eq!(d.parks, strs(&["Tier9"]));
    assert!(json(&d).contains("\"parks\":[\"Tier9\"]"), "{}", json(&d));
    // 宣言に無い名 Tier8 の区画の行を足しても Tier9 だけ。
    let t = all(&[
        park_row("Tier8", Some("park")),
        park_row("Tier9", Some("park")),
    ]);
    assert_eq!(parks(&t), strs(&["Tier9"]));
    assert_eq!(doc(&t, &map, NOW).parks, strs(&["Tier9"]));
    // 欄 kind が Park と lot の行と欄 kind の無い行は区画ではなく、電文に鍵 parks を置かない。
    for kind in [Some("Park"), Some("lot"), None] {
        let t = all(&[park_row("Tier9", kind)]);
        assert_eq!(parks(&t), Vec::<String>::new(), "{kind:?}");
        let d = doc(&t, &map, NOW);
        assert!(d.parks.is_empty(), "{kind:?}");
        assert!(!json(&d).contains("\"parks\""), "{kind:?}");
    }
    // 宣言の Tier9 に同じ名の区画の行が無ければ、名 Tier8 の区画の行だけでは区画にならない。
    let t = all(&[park_row("Tier8", Some("park"))]);
    assert_eq!(parks(&t), Vec::<String>::new());
    parks_empty(map);
}

/// doctor か host.toml の字が無い 3 つの組は区画が空で、電文に鍵 parks を置かない。
fn parks_empty(map: BTreeMap<String, tsuzuri_core::account::project::ProjectTexts>) {
    // doctor か host.toml の字が無ければ空。
    for t in [
        host(None, Some(HOST_TOML)),
        host(Some(doctor(&[park_row("Tier9", Some("park"))])), None),
        host(None, None),
    ] {
        assert_eq!(parks(&t), Vec::<String>::new());
        assert!(doc(&t, &map, NOW).parks.is_empty());
        assert!(!json(&doc(&t, &map, NOW)).contains("\"parks\""));
    }
}

#[test]
fn pkac_seat_card_marks_park() {
    let c = park_card(Some("park"), QUIET, CALM);
    let all = ["acct-1", "acct-2", "acct-3"].map(String::from).to_vec();
    assert_eq!(
        c.group,
        Reading::Known(GroupRow {
            group: "Tier9".into(),
            account: "-".into(),
            candidates: all,
            next_account: Some("-".into()),
            remaining: Vec::new(),
            park: true,
        })
    );
    assert_eq!(c.account.as_deref(), Some("acct-1"));
    assert_eq!(c.heartbeat, Reading::Known(false));
    assert_eq!(c.state, SeatState::Wait);
    assert!(json(&c).contains("\"park\":true"), "{}", json(&c));
    // 欄 kind が Park と lot の行と欄 kind の無い行は park false で、電文に鍵 park を置かない。
    for kind in [Some("Park"), Some("lot"), None] {
        let c = park_card(kind, QUIET, CALM);
        assert!(!known_row(&c).park, "{kind:?}");
        assert!(!json(&c).contains("\"park\""), "{kind:?}");
    }
    // 群の席は Tier1・acct-1・park false。
    let d = doctor(&[park_row("Tier9", Some("park"))]);
    let g = card(
        LOT_SEAT,
        Some(LOT_ANCHOR),
        &seat_texts(d, LOT_SEAT, QUIET, CALM),
        NOW,
    );
    let row = known_row(&g);
    assert_eq!((row.group.as_str(), row.account.as_str()), ("Tier1", "acct-1"));
    assert!(!row.park);
    assert!(!json(&g).contains("\"park\""));
}

/// 限度と移動の結果と件数。
fn limit_or_move(c: &SeatCard) -> (CheckResult, u32) {
    let step = next_step_seat("", "", NOW, Some(c));
    let check = step
        .checks
        .iter()
        .find(|k| k.kind == NextMove::LimitOrMove)
        .expect("限度と移動");
    (check.result, check.count)
}

#[test]
fn pkac_next_skips_park() {
    // 区画の席（登録の口座 acct-1・群の行の今の口座 -）は当たらない。
    let c = park_card(Some("park"), QUIET, CALM);
    assert_eq!(c.account.as_deref(), Some("acct-1"));
    assert_eq!(known_row(&c).account, "-");
    assert_eq!(limit_or_move(&c), (CheckResult::Miss, 0));
    // park を false にした card と欄 kind の無い行の card は今の口座 - と違うので当たる。
    let mut off = c.clone();
    if let Reading::Known(g) = &mut off.group {
        g.park = false;
    }
    assert_eq!(limit_or_move(&off), (CheckResult::Hit, 1));
    assert_eq!(
        limit_or_move(&park_card(None, QUIET, CALM)),
        (CheckResult::Hit, 1)
    );
    // 状態が limit なら区画の席でも当たる。
    let limited = park_card(Some("park"), QUIET, PRESSED_LAST);
    assert_eq!(limited.state, SeatState::Limit);
    assert!(known_row(&limited).park);
    assert_eq!(limit_or_move(&limited), (CheckResult::Hit, 1));
}

#[test]
fn pkac_park_move_copied() {
    let c = park_card(Some("park"), "move=acct-2 grace_left=30", CALM);
    assert_eq!(c.move_to, Reading::Known(Some("acct-2".to_string())));
    assert_eq!(c.grace_until, Reading::Known(Some(NOW + 30)));
    assert!(known_row(&c).park);
    let c = park_card(Some("park"), "move=- grace_left=-", CALM);
    assert_eq!(c.move_to, Reading::Known(None));
    assert_eq!(c.grace_until, Reading::Known(None));
    assert!(known_row(&c).park);
}

#[test]
fn pkac_own_names_clean() {
    assert_eq!(FILTERS.len(), 226);
    let mut names = Vec::new();
    for path in [
        "crates/tsuzuri-core/tests/teeth2/pkac.rs",
        "crates/tsuzuri-surface/tests/teeth4/pkacface.rs",
    ] {
        let text = read(path);
        let lines: Vec<&str> = text.lines().collect();
        names.extend(
            lines
                .windows(2)
                .filter(|w| w[0].trim() == "#[test]")
                .map(|w| {
                    let rest = w[1].trim().strip_prefix("fn ").expect("test の次の行は fn");
                    rest[..rest.find('(').expect("fn の名の後に (")].to_string()
                }),
        );
    }
    assert_eq!(names.len(), 6, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("pkac_")
            .unwrap_or_else(|| panic!("{name} は pkac_ で始まる"));
        for w in FILTERS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
    for w in FILTERS {
        assert!(!"pkac_".contains(w), "接頭辞が {w} を含む");
        assert!(!w.contains("pkac_"), "{w} が接頭辞を含む");
    }
}

/// 起草の時の main（6d7e98e5）の契約表の verify の nextest の最後の引数の filter の語（302 語）を、
/// ほかの語を部分の字として含まない語に畳んだ語（226 語）。
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
