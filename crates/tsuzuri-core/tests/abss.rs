//! 行 c-abs-seat の歯（中核・接頭辞 abss_・要件 NFR2）: 席の card と account board の電文は、server が組む時の今から作る値
//! （時点・最後の区間の終わり・24 時間の窓の始まり・退避までの残り秒）を持たず、材料が変わらない読み直しでは同じ字になる。
//! 時点は材料の時刻（状態の記録の最後の読めた行と合図の最後の判定の ts の大きい方）・退避は終わる時刻。
//! fixture: tests/fixtures/seat/seat-inputs.json と tests/fixtures/account/acct-inputs.json・acct-doc.json（読むだけ）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tsuzuri_contract::account::{AccountDoc, AccountRow, GroupCard, MoveRow};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{SeatCard, SeatSpan};
use tsuzuri_contract::wire;
use tsuzuri_core::account::host::HostTexts;
use tsuzuri_core::account::project::{ProjectTexts, assemble, doc};
use tsuzuri_core::seat::{SeatTexts, anchor, card};

const SEAT_FIXTURE: &str = "tests/fixtures/seat/seat-inputs.json";
const ACCT_INPUTS: &str = "tests/fixtures/account/acct-inputs.json";
const ACCT_DOC: &str = "tests/fixtures/account/acct-doc.json";

/// 今の見本（2026-09-27T12:00:00Z）と、ずらす秒。
const NOW: u64 = 1_790_510_400;
const SHIFTS: [u64; 4] = [1, 60, 3_600, 86_400];

/// 席の名。
const OWN: &str = "proj-1:0.1";

#[derive(Deserialize)]
struct Case {
    #[serde(flatten)]
    texts: SeatTexts,
    card: SeatCard,
}

#[derive(Deserialize)]
struct Inputs {
    texts: HostTexts,
    accounts: Reading<Vec<AccountRow>>,
    groups: Reading<Vec<GroupCard>>,
    moves: Reading<Vec<MoveRow>>,
}

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

fn cases() -> BTreeMap<String, Case> {
    serde_json::from_str(&read(SEAT_FIXTURE)).expect("fixture の形")
}

fn case(name: &str) -> Case {
    cases()
        .remove(name)
        .unwrap_or_else(|| panic!("組 {name} が無い"))
}

/// server と同じ組み方（anchor は doctor の席の行から）。
fn build(texts: &SeatTexts, now: u64) -> SeatCard {
    let anchor = texts.doctor.as_deref().and_then(|d| anchor(d, OWN));
    card(OWN, anchor.as_deref(), texts, now)
}

/// 電文の字。
fn text(c: &SeatCard) -> String {
    wire::encode(c).expect("電文")
}

/// 合図の健康の出力のうち、席 `OWN` の行の末に空白と `tail` を足した字。
fn with_tail(tick: &str, tail: &str) -> String {
    let key = format!("target={OWN} ");
    tick.lines()
        .map(|l| {
            if l.contains(&key) {
                format!("{l} {tail}\n")
            } else {
                format!("{l}\n")
            }
        })
        .collect()
}

/// 今をずらしても同じ電文の字になること。
fn same_across_now(texts: &SeatTexts, what: &str) {
    let base = text(&build(texts, NOW));
    for shift in SHIFTS {
        assert_eq!(text(&build(texts, NOW + shift)), base, "{what} の今 + {shift} 秒");
    }
}

/// (1) 6 組の card の時点は材料の時刻で、期待の card と同じ・区間は時点の 1 日前以後で最後の区間は時点まで。
#[test]
fn abss_card_same_across_now() {
    let want: [(&str, u64, u64); 6] = [
        ("limit", 1_790_503_200, 1_790_503_200),
        ("no-state", 1_790_510_390, 1_790_510_390),
        ("run", 1_790_510_390, 1_790_510_390),
        ("silent", 1_790_509_000, 1_790_509_000),
        ("unread", 1_790_510_390, 1_790_510_390),
        ("wait", 1_790_510_000, 1_790_510_000),
    ];
    assert_eq!(cases().len(), want.len());
    for (name, at, tick_at) in want {
        let c = case(name);
        assert_eq!(c.card.at, at, "組 {name} の期待の at");
        for shift in [0].into_iter().chain(SHIFTS) {
            let got = build(&c.texts, NOW + shift);
            assert_eq!((got.at, got.tick_at), (at, Some(tick_at)), "組 {name} の今 + {shift}");
            assert_eq!(got, c.card, "組 {name} の今 + {shift}");
            if let Reading::Known(spans) = &got.spans {
                if let Some(last) = spans.last() {
                    assert_eq!(last.to, got.at, "組 {name} の最後の区間の終わり");
                }
                assert!(
                    spans.iter().all(|s| s.from >= got.at - 86_400),
                    "組 {name} の区間の始まりは時点の 1 日前以後"
                );
            }
        }
        same_across_now(&c.texts, name);
    }
}

/// (2) 時点は状態の記録の最後の読めた行と合図の最後の判定の ts の大きい方（どちらも無ければ 0）。
#[test]
fn abss_card_at_from_materials() {
    let run = case("run");
    assert_eq!(build(&run.texts, NOW).at, 1_790_510_390);

    // 合図の最後の判定を除くと、時点は状態の記録の最後の行（1790505000）。
    let mut t = run.texts.clone();
    t.tick_last = None;
    let got = build(&t, NOW);
    assert_eq!(got.at, 1_790_505_000);
    let Reading::Known(spans) = &got.spans else {
        panic!("区間が Unknown");
    };
    let last: &SeatSpan = spans.last().expect("区間");
    assert_eq!(last.to, 1_790_505_000);
    assert_eq!(spans[0].from, 1_790_420_000);
    same_across_now(&t, "合図の最後の判定なし");

    // 状態の記録に ts 1790510395 の行を足すと、時点はその行。
    let mut t = run.texts.clone();
    let log = t.state_log.take().expect("状態の記録");
    t.state_log = Some(format!(
        "{log}{{\"schema\":1,\"state\":\"busy\",\"event\":\"tool\",\"ts\":1790510395,\"sid\":\"s-1\"}}\n"
    ));
    assert_eq!(build(&t, NOW).at, 1_790_510_395);

    // どちらも除くと 0 で区間は Unknown。
    let mut t = run.texts.clone();
    t.tick_last = None;
    t.state_log = None;
    let got = build(&t, NOW);
    assert_eq!((got.at, got.tick_at), (0, None));
    assert_eq!(got.spans, Reading::Unknown);
    same_across_now(&t, "材料なし");
}

/// (3) 猶予の終わる時刻は器の残り秒に組んだ今を足した時刻（`-` は Known(None)・読めない字は Unknown）。
#[test]
fn abss_grace_until_from_left() {
    let run = case("run");
    let tick = run.texts.tick_status.clone().expect("合図の健康の出力");
    let with = |tail: &str| {
        let mut t = run.texts.clone();
        t.tick_status = Some(with_tail(&tick, tail));
        t
    };
    let got = build(&with("reopens=- move=acct-2 grace_left=600"), NOW);
    assert_eq!(got.grace_until, Reading::Known(Some(1_790_511_000)));
    let base = text(&got);
    for d in [1, 60, 600] {
        let t = with(&format!("reopens=- move=acct-2 grace_left={}", 600 - d));
        assert_eq!(text(&build(&t, NOW + d)), base, "今 + {d} 秒");
    }
    let none = with("reopens=- move=- grace_left=-");
    assert_eq!(build(&none, NOW).grace_until, Reading::Known(None));
    same_across_now(&none, "move=- grace_left=-");
    let bad = with("reopens=- move=unreadable grace_left=unreadable");
    assert_eq!(build(&bad, NOW).grace_until, Reading::Unknown);
    same_across_now(&bad, "unreadable");
}

fn inputs() -> Inputs {
    serde_json::from_str(&read(ACCT_INPUTS)).expect("fixture の形")
}

/// proj-a の字（合図の最後の判定は ts 1790508000・状態の記録の最後の行は 1790505060）。
fn projects() -> BTreeMap<String, ProjectTexts> {
    let a = ProjectTexts {
        state_dir_known: true,
        tick_status: Some("seat tick status: target=proj-a:0.1 healthy=yes heartbeat=on\n".into()),
        state_log: Some(
            "{\"state\":\"idle\",\"ts\":1790500000}\n{\"state\":\"busy\",\"ts\":1790505060}\n".into(),
        ),
        tick_last: Some("ts=1790508000 decision=noop reason=seat-busy\n".into()),
        ..ProjectTexts::default()
    };
    BTreeMap::from([("/work/proj-a".to_string(), a)])
}

/// (4) 電文は今をずらしても同じ字・時点は材料の時刻の最大で、材料が無ければ 0。
#[test]
fn abss_doc_same_across_now() {
    let i = inputs();
    let ps = projects();
    let text = |now: u64| wire::encode(&doc(&i.texts, &ps, now)).expect("電文");
    let base = text(1_790_511_000);
    for shift in SHIFTS {
        assert_eq!(text(1_790_511_000 + shift), base, "今 + {shift} 秒");
    }
    let d = doc(&i.texts, &ps, 1_790_511_000);
    let Some(Reading::Known(a)) = d.projects.iter().find(|p| p.name == "proj-a").map(|p| p.seat.clone()) else {
        panic!("proj-a の席の card が読めない");
    };
    assert_eq!(a.at, 1_790_508_000);
    assert_eq!(d.at, 1_790_510_400, "群の移動の最大");

    let empty = assemble(
        Reading::Unknown,
        Reading::Unknown,
        Reading::Unknown,
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(empty.at, 0);
    let moved = assemble(
        i.accounts,
        i.groups,
        i.moves,
        d.projects.clone(),
        d.sessions.clone(),
    );
    assert_eq!(moved.at, 1_790_510_400);
}

/// (5) fixture acct-doc.json は鍵 move_left_s と age_p50_days を持たず、終わる時刻と作った時刻の中央値を持つ。
#[test]
fn abss_doc_fixture_keys() {
    let raw = read(ACCT_DOC);
    for gone in ["\"move_left_s\"", "\"age_p50_days\""] {
        assert!(!raw.contains(gone), "fixture に鍵 {gone} が残る");
    }
    let d: AccountDoc = serde_json::from_str(&raw).expect("fixture の形");
    assert_eq!(d.at, 1_790_510_400);
    assert!(raw.contains("\"at\": 1790510400"));
    let b = d.projects.iter().find(|p| p.name == "proj-b").expect("proj-b");
    assert_eq!(b.move_until, Some(d.at + 1101));
    let a = d.projects.iter().find(|p| p.name == "proj-a").expect("proj-a");
    let Reading::Known(stats) = &a.ledger else {
        panic!("proj-a の台帳が Unknown");
    };
    assert_eq!(stats.memo.created_p50, Some(d.at - 475_200));
}

/// 着地済みの filter の語（起草の時の main f467057 の契約表の verify の最後の字の 217 語）。
const WORDS: [&str; 217] = [
    "aaround_", "abst_", "accept_", "acchold_", "account_", "acctcore_", "acctdoc_", "accthb_",
    "accthome_", "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_",
    "acctwire_", "aface_", "afocus_", "alean_", "aord_", "aown_", "apark_", "apop_", "areread_",
    "askcard_", "athr_", "batchpanel_", "bhalf_", "board_min_", "bport_", "brand_", "btuck_",
    "cadl_", "cadopt_", "cadq_", "cdorm_", "cfsplit_", "cg9_", "cgdom_", "cmark_", "cnote_",
    "contract_form_", "cround_", "csled_", "cspk_", "ctick_", "cupd_", "cupdlist_", "denv_",
    "dnedge_", "dngrp_", "dnrow_", "dnskip_", "dretry_", "ecache_", "epolq_", "eretry_", "esig_",
    "evkind_", "flight_", "fmark_", "fprem_", "frame_", "fserve_", "fstop_", "fxpre_", "g3g7_",
    "gacct_", "gapspage_", "gatt_", "gbnote_", "gchip_", "gcoach_", "gfix_", "gfresh_", "ghb_",
    "gins_", "gjst_", "glabel_", "gnav_", "gpface_", "gpill_", "gpulse_", "graph_", "gsum_",
    "gtuck_", "gview_", "gwv_", "hacols_", "harest_", "hasplit_", "hbconf_", "hbmark_", "hbon_",
    "hbpost_", "hbproc_", "hbroute_", "hcard_", "hcled_", "hcnx_", "hcproj_", "hcsess_",
    "hdchip_", "hfig_", "hnunk_", "hook_", "hruling_", "hsblock_", "hsderive_", "hshist_",
    "hspage_", "hspk_", "hsym_", "hthr_", "http_", "hwstore_", "iclose_", "ilink_", "jcount_",
    "jrun_", "kcli_", "kg9_", "kindlab_", "klink_", "ksum_", "launch_", "lcard_", "ledgerblock_",
    "lgrp_", "lhome_", "lidle_", "lkind_", "lresume_", "lsnap_", "lspark_", "lstg_", "lstore_",
    "mapview_", "mkeys_", "mlink_", "mqask_", "mqface_", "mstore_", "mtips_", "mtree_", "nact_",
    "nbatch_", "ncard_", "nextstep_", "nodepage_", "nstall_", "nsum_", "nsumw_", "ntc_", "ntime_",
    "nxact_", "nxorg_", "parts_", "pci_", "pclosed_", "pfold_", "pgsw_", "pgz_", "pipe_",
    "plimit_", "pmisfit_", "pmore_", "pquest_", "pqueue_", "project_", "ptitle_", "pubscan_",
    "punmap_", "pwhole_", "qblock_", "qgate_", "qkey_", "qsig_", "question_", "rbusy_", "relay_",
    "rhold_", "runsdoc_", "rvk_", "saxis_", "sclosed_", "seatblock_", "seatcard_",
    "server::events::", "server_", "sesplit_", "sgrace_", "shb_", "skeleton_", "smore_", "stage_",
    "stats_", "stbp_", "stcli_", "steady_", "sthr_", "stnfy_", "stskill_", "sttgt_", "sxaxis_",
    "ticker_", "tipx_", "tkad_", "tlic_", "topbar_", "topfit_", "tz_", "udash_", "unow_",
    "urpanel_", "uword_", "wstrip_",
];

/// file の test の属性の次の行の fn の名。
fn test_names(path: &PathBuf) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
    let mut lines = text.lines().map(str::trim);
    let mut names = Vec::new();
    while let Some(line) = lines.next() {
        if line != "#[test]" {
            continue;
        }
        let f = lines.next().expect("属性の次の行");
        let name = f
            .strip_prefix("fn ")
            .and_then(|r| r.split_once('('))
            .map(|(n, _)| n)
            .unwrap_or_else(|| panic!("属性の次が fn でない: {f}"));
        names.push(name.to_string());
    }
    names
}

/// (9) 2 つの file の歯の名は abss_ で始まり 9 つで、残りの字は着地済みの filter の語を含まない。
#[test]
fn abss_own_names_clean() {
    assert_eq!(WORDS.len(), 217);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut names = test_names(&root.join("tests/abss.rs"));
    names.extend(test_names(&root.join("../tsuzuri-surface/tests/abssface.rs")));
    assert_eq!(names.len(), 9, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("abss_")
            .unwrap_or_else(|| panic!("{name} が abss_ で始まらない"));
        for word in WORDS {
            assert!(!rest.contains(word), "{name} が語 {word} を含む");
        }
    }
}
