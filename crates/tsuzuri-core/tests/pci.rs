//! 行 c-pipe-ci の歯（中核）: 着地の後の器の終端の RunDone の detail の語を CI の読みに畳み、札の段と欄 ci に重ねる。
//! 台帳と event log は歯の中で組む（fixture の file は使わない・bead の id の接頭辞は fx-ci）。
#![cfg(test)]

use std::path::PathBuf;

use serde_json::{Value, json};
use tsuzuri_contract::board::{Ci, NextMove, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::stats::CheckResult;
use tsuzuri_core::next_step::next_step;
use tsuzuri_core::pipeline::{
    Board, CI_STALLS, CI_WORDS, FAULT_WORDS, PUSH_WORD, TERMINAL_TAG, board, ci_after, ci_reading,
};

/// 節の今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 閉じた bead（fx-ci.3・fx-ci.6・fx-ci.9）。
const CLOSED: [u32; 3] = [3, 6, 9];

/// 着地の RunDone の後に 30 秒から 10 秒ごとに足す RunDone の detail（fx-ci.1 から順に）。
const TAILS: [&[&str]; 13] = [
    &["terminal:push:origin"],
    &["terminal:push:origin", "terminal:ci:success"],
    &["terminal:push:origin", "terminal:ci:success", "terminal:close:ok"],
    &["terminal:push:origin", "terminal:ci:failure", "detection:measured"],
    &["terminal:push:origin", "terminal:ci:unmeasurable"],
    &["terminal:push:origin"],
    &[],
    &[
        "terminal:push:origin",
        "terminal:ci:unmeasurable",
        "terminal:push:origin",
    ],
    &["terminal:push:origin", "terminal:ci:failure"],
    &["terminal:push:failed:git"],
    &["terminal:push:origin", "terminal:ci:failure"],
    &[
        "terminal:push:origin",
        "terminal:ci:success",
        "terminal:close:failed:rc=1 x",
    ],
    &["terminal:unreadable"],
];

/// 節の台帳（fx-ci.1 から fx-ci.13 の task）。
fn ledger() -> String {
    Value::Array(
        (1..=13)
            .map(|n| {
                let mut b = json!({
                    "id": format!("fx-ci.{n}"),
                    "title": format!("t{n}"),
                    "status": "open",
                    "issue_type": "task",
                });
                if CLOSED.contains(&n) {
                    b["status"] = json!("closed");
                    b["close_reason"] = json!("landed 0a ci=success");
                }
                b
            })
            .collect(),
    )
    .to_string()
}

/// event の 1 行（run は bead とハイフンと RunCreated の時刻の札）。
fn ev(ts: &str, kind: &str, run: &str, stage: Option<&str>, detail: Option<&str>) -> String {
    let (bead, _) = run.rsplit_once('-').expect("run の id");
    let mut e = json!({"schema": 1, "ts": ts, "kind": kind, "run": run, "bead": bead});
    if let Some(s) = stage {
        e["stage"] = json!(s);
    }
    if let Some(d) = detail {
        e["detail"] = json!(d);
    }
    e.to_string()
}

/// 節の event log（bead ごとの走行 1 つと fx-ci.11 の 2 つ目の走行）。
fn events() -> String {
    let mut out = Vec::new();
    for (i, tail) in TAILS.iter().enumerate() {
        let m = i + 1;
        let run = format!("fx-ci.{m}-20260927T11{m:02}00Z");
        let at = |s: usize| format!("2026-09-27T11:{m:02}:{s:02}Z");
        out.push(ev(&at(0), "RunCreated", &run, None, None));
        out.push(ev(
            &at(10),
            "RunStage",
            &run,
            Some("Spawned"),
            Some("base:0a,account:acct-1"),
        ));
        out.push(ev(
            &at(20),
            "RunDone",
            &run,
            Some("Landed"),
            Some("sha:0a main:0a"),
        ));
        for (k, detail) in tail.iter().enumerate() {
            out.push(ev(
                &at(30 + 10 * k),
                "RunDone",
                &run,
                Some("Landed"),
                Some(detail),
            ));
        }
    }
    let run = "fx-ci.11-20260927T113000Z";
    out.push(ev("2026-09-27T11:30:00Z", "RunCreated", run, None, None));
    out.push(ev(
        "2026-09-27T11:30:10Z",
        "RunStage",
        run,
        Some("Spawned"),
        Some("base:0b,account:acct-2"),
    ));
    out.join("\n")
}

fn card(
    n: u32,
    runs: u32,
    stage: Stage,
    reason: Option<&str>,
    account: &str,
    elapsed: u64,
    ci: Option<Ci>,
) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(format!("fx-ci.{n}").as_str()).expect("bead の id"),
        runs,
        stage,
        reason: reason.map(str::to_string),
        account: Some(account.to_string()),
        since: Some(NOW - elapsed),
        ci,
    }
}

fn cards(b: &Board) -> &[PipelineCard] {
    match &b.board.cards {
        Reading::Known(c) => c,
        Reading::Unknown => panic!("札が Unknown"),
    }
}

/// 項 4 の 13 枚の札。
fn want_with_ledger() -> Vec<PipelineCard> {
    use Stage::*;
    vec![
        card(1, 1, Landed, None, "acct-1", 3510, Some(Ci::Waiting)),
        card(2, 1, Landed, None, "acct-1", 3440, Some(Ci::Success)),
        card(3, 1, Landed, None, "acct-1", 3370, Some(Ci::Success)),
        card(
            4,
            1,
            Failed,
            Some("terminal:ci:failure"),
            "acct-1",
            3310,
            Some(Ci::Failure),
        ),
        card(
            5,
            1,
            Stopped,
            Some("terminal:ci:unmeasurable"),
            "acct-1",
            3260,
            Some(Ci::Unmeasurable),
        ),
        card(6, 1, Landed, None, "acct-1", 3210, None),
        card(7, 1, Landed, None, "acct-1", 3160, None),
        card(8, 1, Landed, None, "acct-1", 3070, Some(Ci::Waiting)),
        card(9, 1, Landed, None, "acct-1", 3020, Some(Ci::Failure)),
        card(
            10,
            1,
            Failed,
            Some("terminal:push:failed:git"),
            "acct-1",
            2970,
            Some(Ci::PushFailed),
        ),
        card(11, 2, Running, None, "acct-2", 1790, None),
        card(
            12,
            1,
            Failed,
            Some("terminal:close:failed:rc=1 x"),
            "acct-1",
            2830,
            Some(Ci::CloseFailed),
        ),
        card(
            13,
            1,
            Stopped,
            Some("terminal:unreadable"),
            "acct-1",
            2790,
            Some(Ci::Unreadable),
        ),
    ]
}

/// (3) 終端の detail の語を前の読みに重ねる。
#[test]
fn pci_words_fold() {
    assert_eq!(TERMINAL_TAG, "terminal:");
    assert_eq!(PUSH_WORD, "push:");
    assert_eq!(
        CI_WORDS,
        [
            ("ci:success", Ci::Success),
            ("ci:failure", Ci::Failure),
            ("ci:unmeasurable", Ci::Unmeasurable),
        ]
    );
    assert_eq!(
        FAULT_WORDS,
        [
            ("push:failed:", Ci::PushFailed),
            ("close:failed:", Ci::CloseFailed),
            ("unreadable", Ci::Unreadable),
        ]
    );
    assert_eq!(
        CI_STALLS,
        [
            (Ci::Failure, Stage::Failed),
            (Ci::PushFailed, Stage::Failed),
            (Ci::CloseFailed, Stage::Failed),
            (Ci::Unmeasurable, Stage::Stopped),
            (Ci::Unreadable, Stage::Stopped),
        ]
    );
    use Ci::*;
    let cases: [(Option<Ci>, &str, Option<Ci>); 22] = [
        (None, "sha:0a main:0a", None),
        (None, "terminal:push:origin", Some(Waiting)),
        (Some(Unmeasurable), "terminal:push:origin", Some(Waiting)),
        (Some(Waiting), "terminal:ci:success", Some(Success)),
        (Some(Waiting), "terminal:ci:failure", Some(Failure)),
        (Some(Waiting), "terminal:ci:unmeasurable", Some(Unmeasurable)),
        (Some(Success), "terminal:close:ok", Some(Success)),
        (Some(Success), "terminal:close:failed:rc=1 x", Some(CloseFailed)),
        (Some(Success), "terminal:close:failed:unlaunchable", Some(CloseFailed)),
        (Some(Success), "terminal:close:failed", Some(Success)),
        (None, "terminal:push:failed:git", Some(PushFailed)),
        (Some(Waiting), "terminal:push:failed:git", Some(PushFailed)),
        (Some(PushFailed), "terminal:push:origin", Some(Waiting)),
        (None, "terminal:unreadable", Some(Unreadable)),
        (Some(Failure), "detection:measured", Some(Failure)),
        (Some(Waiting), "terminal:ci:successful", Some(Waiting)),
        (Some(Waiting), "ci:success", Some(Waiting)),
        (Some(Waiting), " terminal:ci:success", Some(Waiting)),
        (Some(Waiting), "terminal:ci:", Some(Waiting)),
        (Some(Failure), "", Some(Failure)),
        (None, "", None),
        (Some(Waiting), "sha:0a main:0a", Some(Waiting)),
    ];
    for (prev, detail, want) in cases {
        assert_eq!(ci_after(prev, detail), want, "{prev:?} と {detail:?}");
    }
}

/// (4) 台帳の読める板の札は項 4 の 13 枚。
#[test]
fn pci_cards_with_ledger() {
    let b = board(&ledger(), &events(), NOW);
    assert_eq!(cards(&b), want_with_ledger().as_slice());
    assert_eq!(b.unmapped, 0);
    let wire: Value = serde_json::to_value(cards(&b)).expect("札の電文");
    let back: Vec<PipelineCard> = serde_json::from_value(wire).expect("札の電文を読む");
    assert_eq!(back, want_with_ledger());
}

/// (5) 台帳が読めない板は閉じた bead の読みも閉じていない bead の読みにする。
#[test]
fn pci_cards_without_ledger() {
    let b = board("", &events(), NOW);
    let want: Vec<PipelineCard> = want_with_ledger()
        .into_iter()
        .map(|c| match c.contract.as_str() {
            "fx-ci.6" => PipelineCard {
                ci: Some(Ci::Waiting),
                ..c
            },
            "fx-ci.9" => PipelineCard {
                stage: Stage::Failed,
                reason: Some("terminal:ci:failure".to_string()),
                ..c
            },
            _ => c,
        })
        .collect();
    assert_eq!(cards(&b), want.as_slice());
    for (got, with) in cards(&b).iter().zip(want_with_ledger()) {
        assert_eq!(
            (got.runs, &got.account, got.since),
            (with.runs, &with.account, with.since),
            "{}",
            got.contract
        );
    }
}

/// (6) 止まっている走行は CI の読みで止まった open の bead の札を数える。
#[test]
fn pci_stalled_move() {
    let step = next_step(&ledger(), &events(), NOW);
    let stalled = step
        .checks
        .iter()
        .find(|c| c.kind == NextMove::StalledRun)
        .expect("止まっている走行の結果");
    assert_eq!(
        (
            stalled.result,
            stalled.count,
            stalled.target.as_ref().map(BeadId::as_str)
        ),
        (CheckResult::Hit, 5, Some("fx-ci.4"))
    );
}

/// (1) 札の電文の欄 ci の字と読み。
#[test]
fn pci_wire_default() {
    let base = json!({
        "contract": "fx-ci.1",
        "runs": 1,
        "stage": "Landed",
        "reason": null,
        "account": null,
        "since": NOW - 60,
    });
    let read = |ci: Option<Value>| {
        let mut v = base.clone();
        if let Some(ci) = ci {
            v["ci"] = ci;
        }
        serde_json::from_value::<PipelineCard>(v).map(|c| c.ci)
    };
    for (text, want) in [
        ("waiting", Ci::Waiting),
        ("success", Ci::Success),
        ("failure", Ci::Failure),
        ("unmeasurable", Ci::Unmeasurable),
        ("push-failed", Ci::PushFailed),
        ("close-failed", Ci::CloseFailed),
        ("unreadable", Ci::Unreadable),
    ] {
        assert_eq!(read(Some(json!(text))).expect(text), Some(want));
    }
    assert_eq!(read(Some(Value::Null)).expect("null"), None);
    assert_eq!(read(None).expect("欄が無い"), None);
    assert!(read(Some(json!("running"))).is_err());

    let card: PipelineCard = serde_json::from_value(base).expect("札");
    let text = serde_json::to_string(&card).expect("札の電文");
    assert!(text.ends_with(",\"ci\":null}"), "{text}");
}

/// 行の走行の event の行（種類と detail）。
fn rows(of: &[(&str, &str)]) -> Vec<Value> {
    of.iter()
        .map(|(kind, detail)| json!({"kind": kind, "detail": detail}))
        .collect()
}

/// (15) 判定の関数 ci_reading は 1 つで、面は終端の字を持たない。
#[test]
fn pci_reading_one_fn() {
    let read = |of: &[(&str, &str)]| {
        let rows = rows(of);
        let refs: Vec<&Value> = rows.iter().collect();
        ci_reading(&refs)
    };
    assert_eq!(read(&[("RunDone", "sha:0a main:0a")]), None);
    assert_eq!(read(&[]), None);
    assert_eq!(
        read(&[
            ("RunDone", "sha:0a main:0a"),
            ("RunDone", "terminal:push:origin"),
            ("RunDone", "terminal:ci:success"),
            ("RunDone", "terminal:close:failed:rc=1 x"),
        ]),
        Some((Ci::CloseFailed, "terminal:close:failed:rc=1 x".to_string()))
    );
    assert_eq!(
        read(&[
            ("RunDone", "terminal:push:origin"),
            ("RunDone", "terminal:ci:failure"),
            ("RunDone", "detection:measured"),
        ]),
        Some((Ci::Failure, "terminal:ci:failure".to_string()))
    );
    assert_eq!(
        read(&[
            ("RunDone", "terminal:push:origin"),
            ("RunStage", "terminal:ci:failure"),
        ]),
        Some((Ci::Waiting, "terminal:push:origin".to_string()))
    );

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let core = std::fs::read_to_string(root.join("src/pipeline.rs")).expect("src/pipeline.rs");
    let call = "ci_after(";
    assert_eq!(core.matches(call).count(), 2, "字 {call} の数");
    let start = core.find("fn ci_reading(").expect("fn ci_reading");
    let end = start + core[start..].find("\n}\n").expect("fn ci_reading の終わり");
    assert_eq!(core[start..end].matches(call).count(), 1);
    assert_eq!(core.matches("fn ci_after(").count(), 1);

    let surface = std::fs::read_to_string(root.join("../tsuzuri-surface/src/project/pipeline.rs"))
        .expect("面の src/project/pipeline.rs");
    assert!(!surface.contains("terminal:"));
    assert!(!surface.contains("TERMINAL_TAG"));
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

/// (13) この file の歯の名は 7 つで、どれも pci_ で始まり、名の全体が filter の語を含まない。
#[test]
fn pci_core_names_clean() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/pci.rs");
    let text = std::fs::read_to_string(&path).expect("tests/pci.rs を読む");
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
    assert_eq!(names.len(), 7, "{names:?}");
    for name in names {
        assert!(name.starts_with("pci_"), "{name} が pci_ で始まらない");
        for word in FILTER_WORDS {
            assert!(!name.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
