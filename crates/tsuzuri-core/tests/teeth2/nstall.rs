//! 次の一手の止まっている走行の段となしの決まりの歯（行 c-next-stall・接頭辞 nstall_）。
//! 止まっている走行は Questioned の札を数えない（質問の側）。なしは、ほかの種類のどれも当たらず、
//! `NO_INPUT` に無い種類のどれかを判じなかったなら判じなかった（要件 NFR2）。
//! fixture: tests/fixtures/pipeline/next.json と tests/fixtures/seat/seat-inputs.json。
#![cfg(test)]

use std::path::{Path, PathBuf};

use serde_json::Value;
use tsuzuri_contract::board::{NextMove, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};
use tsuzuri_core::next_step::{NO_INPUT, STALLED_STAGES, next_step, next_step_seat};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(rel: &str) -> Value {
    let raw = std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{rel} は JSON: {e}"))
}

fn next_fixture() -> Value {
    fixture("tests/fixtures/pipeline/next.json")
}

fn now_of(v: &Value) -> u64 {
    v["now"].as_u64().expect("now")
}

fn case(v: &Value, i: usize) -> &Value {
    &v["cases"][i]
}

fn ledger_of(c: &Value) -> String {
    c["ledger"].to_string()
}

/// event log の字（配列は 1 行 1 件を改行でつなぐ・字ならそのまま）。bead が `skip` の要素は除く。
fn events_of(c: &Value, skip: Option<&str>) -> String {
    match &c["events"] {
        Value::String(s) => s.clone(),
        Value::Array(items) => items
            .iter()
            .filter(|i| skip.is_none_or(|b| i["bead"] != b))
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n"),
        other => panic!("events の形: {other}"),
    }
}

fn card(name: &str) -> SeatCard {
    let v = fixture("tests/fixtures/seat/seat-inputs.json");
    serde_json::from_value(v[name]["card"].clone()).unwrap_or_else(|e| panic!("組 {name} の card: {e}"))
}

fn of(step: &NextStep, kind: NextMove) -> &NextCheck {
    step.checks
        .iter()
        .find(|c| c.kind == kind)
        .unwrap_or_else(|| panic!("{kind:?} の結果が無い"))
}

fn id(s: &str) -> Option<BeadId> {
    Some(BeadId::new(s).expect("bead の id"))
}

fn is(step: &NextStep, kind: NextMove, result: CheckResult, count: u32, target: Option<BeadId>) {
    let c = of(step, kind);
    assert_eq!((c.result, c.count, &c.target), (result, count, &target), "{kind:?}: {step:?}");
}

fn results(step: &NextStep, kinds: &[NextMove], result: CheckResult) {
    for &k in kinds {
        assert_eq!(of(step, k).result, result, "{k:?}: {step:?}");
    }
}

/// (1) 止まっている走行の段は Failed と Stopped だけで、問いで止まった契約（段 Questioned）は数えない。
#[test]
fn nstall_drops_questioned() {
    assert_eq!(STALLED_STAGES, [Stage::Failed, Stage::Stopped]);
    assert!(!STALLED_STAGES.contains(&Stage::Questioned));
    let v = next_fixture();
    let now = now_of(&v);
    let c0 = case(&v, 0);
    let ledger = ledger_of(c0);

    let step = next_step(&ledger, &events_of(c0, None), now);
    is(&step, NextMove::StalledRun, CheckResult::Hit, 1, id("nx1.1"));
    is(&step, NextMove::Question, CheckResult::Hit, 1, id("nx1.q"));
    assert_eq!(step.lead, NextMove::StalledRun);

    // nx1.1 の 2 行を除けば、残る nx1.2（Questioned）と nx1.3（閉じた）は数えず、質問が大きく出る。
    let events = events_of(c0, Some("nx1.1"));
    assert_eq!(events.lines().count(), 4);
    let step = next_step(&ledger, &events, now);
    is(&step, NextMove::StalledRun, CheckResult::Miss, 0, None);
    is(&step, NextMove::Question, CheckResult::Hit, 1, id("nx1.q"));
    assert_eq!(step.lead, NextMove::Question);
}

/// (2) なしの決まり: ほかのどれかが当たれば Miss・どれも当たらず `NO_INPUT` に無い種類を判じなかったなら
/// NotJudged・ほかは Hit。lead は並びで最初に Hit の種類（無ければなし）。発効待ちはいつも判じない。
#[test]
fn nstall_nothing_rule() {
    assert_eq!(NO_INPUT, [NextMove::AwaitingEffect]);
    let v = next_fixture();
    let now = now_of(&v);
    let run = card("run");
    let limit = card("limit");
    let (c0, c2) = (case(&v, 0), case(&v, 2));
    let others = [
        NextMove::LimitOrMove,
        NextMove::Unresponsive,
        NextMove::StalledRun,
        NextMove::BatchApproval,
        NextMove::Question,
    ];

    // 組 2 と組 run の card: 判じられる種類がどれも当たらないので、なしが当たる。
    let seat = next_step_seat(&ledger_of(c2), &events_of(c2, None), now, Some(&run));
    results(&seat, &others, CheckResult::Miss);
    is(&seat, NextMove::Nothing, CheckResult::Hit, 0, None);
    assert_eq!(seat.lead, NextMove::Nothing);

    // 組 2 と card 無し: 限度と移動と応答なしを判じないので、なしも判じない。
    let bare = next_step(&ledger_of(c2), &events_of(c2, None), now);
    results(
        &bare,
        &[NextMove::LimitOrMove, NextMove::Unresponsive],
        CheckResult::NotJudged,
    );
    is(&bare, NextMove::Nothing, CheckResult::NotJudged, 0, None);
    assert_eq!(bare.lead, NextMove::Nothing);

    // 読めない台帳と組 0 の event log と組 run の card: 台帳の種類は判じない。
    let unread = next_step_seat("", &events_of(c0, None), now, Some(&run));
    results(
        &unread,
        &[NextMove::StalledRun, NextMove::BatchApproval, NextMove::Question],
        CheckResult::NotJudged,
    );
    results(
        &unread,
        &[NextMove::LimitOrMove, NextMove::Unresponsive],
        CheckResult::Miss,
    );
    is(&unread, NextMove::Nothing, CheckResult::NotJudged, 0, None);
    assert_eq!(unread.lead, NextMove::Nothing);

    // 組 0 と組 run の card: 止まっている走行が当たるので、なしは当たらない。
    let hit = next_step_seat(&ledger_of(c0), &events_of(c0, None), now, Some(&run));
    is(&hit, NextMove::Nothing, CheckResult::Miss, 0, None);
    assert_eq!(hit.lead, NextMove::StalledRun);

    // 空の台帳と空の event log と組 limit の card: 限度と移動が当たり、ほかを判じなくてもなしは当たらない。
    let lim = next_step_seat("", "", now, Some(&limit));
    assert_eq!(of(&lim, NextMove::LimitOrMove).result, CheckResult::Hit);
    is(&lim, NextMove::Nothing, CheckResult::Miss, 0, None);
    assert_eq!(lim.lead, NextMove::LimitOrMove);

    for step in [&seat, &bare, &unread, &hit, &lim] {
        let kinds: Vec<NextMove> = step.checks.iter().map(|c| c.kind).collect();
        assert_eq!(kinds, NextMove::ALL.to_vec());
        assert_eq!(
            of(step, NextMove::AwaitingEffect).result,
            CheckResult::NotJudged
        );
        let first = step
            .checks
            .iter()
            .find(|c| c.result == CheckResult::Hit)
            .map_or(NextMove::Nothing, |c| c.kind);
        assert_eq!(step.lead, first, "{step:?}");
    }
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
fn nstall_core_names_apart() {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth2/nstall.rs"))
        .expect("tests/teeth2/nstall.rs を読む");
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
    assert!(names.len() >= 3, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("nstall_")
            .unwrap_or_else(|| panic!("{name} が nstall_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
