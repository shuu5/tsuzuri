//! 行 g-pop-flow の歯: 吹き出しの run の段の流れ（審査・実装・gate の長さと走っている段の「〜」）と run の歴と着地の commit、
//! 段の語の閉じた表が器の case-lifecycle.md §2.1 の 11 値であることと表に無い語のまだ分からない、
//! 欄を差す所と、走行の読みの口を吹き出しを開いた時に読む配線の字。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{Reading, Stage};
use tsuzuri_contract::runs::{GateVerdict, ReviewVerdict, RunCost, RunLine, RunStep};
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::pop::{Fact, Pop, STAGE_KEYS, Sum, Val};
use tsuzuri_surface::widgets::runflow::{
    COMMIT_CHARS, FLOW_KEYS, Hist, OPEN_END, STAGES, Seg, Step, commit, flow, flow_text, history,
    known_stage, with_runs,
};

const T0: EpochSecs = 1_790_000_000;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn step(at: EpochSecs, stage: &str, detail: Option<&str>) -> RunStep {
    RunStep {
        at: Some(at),
        stage: stage.to_string(),
        detail: detail.map(str::to_string),
        verdict: None,
        verdict_kind: None,
    }
}

fn line(run: &str, steps: Vec<RunStep>) -> RunLine {
    RunLine {
        run: run.to_string(),
        started_at: steps.first().and_then(|s| s.at),
        account: None,
        steps,
        cost: RunCost::default(),
        gate: Reading::Unknown,
        review: Reading::Unknown,
    }
}

fn seg(step: Step, from: EpochSecs, to: Option<EpochSecs>) -> Seg {
    Seg {
        step,
        from: Some(from),
        to,
        open: to.is_none(),
    }
}

/// 段の語の閉じた表は器の便の段の 11 値（case-lifecycle.md §2.1 の表の順）で、流れを始めるのは Intake・Spawned・Implemented だけ。
#[test]
fn bvstep_stages_are_vessel_eleven() {
    let words: Vec<&str> = STAGES.iter().map(|(w, _)| *w).collect();
    assert_eq!(
        words,
        [
            "Intake",
            "Reviewed",
            "Blocked",
            "Spawned",
            "Questioned",
            "RateLimited",
            "Implemented",
            "Gated",
            "Landed",
            "Stopped",
            "Failed"
        ]
    );
    let starts: Vec<(&str, Step)> = STAGES
        .iter()
        .filter_map(|(w, s)| s.map(|s| (*w, s)))
        .collect();
    assert_eq!(
        starts,
        [
            ("Intake", Step::Review),
            ("Spawned", Step::Implement),
            ("Implemented", Step::Gate)
        ]
    );
    for k in FLOW_KEYS
        .into_iter()
        .chain([Step::Review, Step::Implement, Step::Gate].map(Step::key))
    {
        assert!(vocab().term(k).is_some(), "{k} が語の辞書に無い");
    }
}

/// 審査は Intake から次の段・実装は Spawned から次の段・gate は Implemented から次の段（長さは分の字で矢印でつなぐ）。
#[test]
fn bvstep_flow_three_steps_with_minutes() {
    let l = line(
        "b-1-r1",
        vec![
            step(T0, "Intake", None),
            step(T0 + 600, "Reviewed", Some("verdict:PASS")),
            step(T0 + 660, "Spawned", Some("base:abc,account:a")),
            step(T0 + 660 + 1_320, "Implemented", None),
            step(T0 + 660 + 1_320 + 300, "Gated", Some("verdict:PASS")),
        ],
    );
    let (segs, unknown) = flow(&l);
    assert_eq!(
        segs,
        [
            seg(Step::Review, T0, Some(T0 + 600)),
            seg(Step::Implement, T0 + 660, Some(T0 + 1_980)),
            seg(Step::Gate, T0 + 1_980, Some(T0 + 2_280)),
        ]
    );
    assert!(unknown.is_empty());
    let want = format!(
        "{} 10m → {} 22m → {} 5m",
        label("fl_review"),
        label("fl_impl"),
        label("fl_gate")
    );
    assert_eq!(flow_text(&segs, T0 + 9_999), want);
}

/// 次の段の event が無い段は走っていて、長さは今から引き、末に「〜」を付ける。
#[test]
fn bvstep_running_step_open_end() {
    let l = line(
        "b-1-r1",
        vec![
            step(T0, "Intake", None),
            step(T0 + 600, "Reviewed", Some("verdict:PASS")),
            step(T0 + 660, "Spawned", None),
        ],
    );
    let (segs, _) = flow(&l);
    assert_eq!(segs.last(), Some(&seg(Step::Implement, T0 + 660, None)));
    assert_eq!(OPEN_END, "〜");
    let want = format!("{} 10m → {} 7m〜", label("fl_review"), label("fl_impl"));
    assert_eq!(flow_text(&segs, T0 + 660 + 420), want);
}

/// 走行ごとの歴: 回は 1 から・終わりの段は最後の段の語・結びは gate の判定・無ければ審査の判定・無ければ段の審査の結び。
#[test]
fn bvstep_history_per_run() {
    let mut a = line(
        "b-1-r1",
        vec![step(T0, "Intake", None), step(T0 + 9, "Reviewed", None)],
    );
    a.steps[1].verdict = Some("FAIL".to_string());
    let mut b = line(
        "b-1-r2",
        vec![step(T0, "Intake", None), step(T0 + 9, "Gated", None)],
    );
    b.review = Reading::Known(ReviewVerdict {
        verdict: "PASS".to_string(),
        evidence: String::new(),
        kind: None,
        place: None,
        at: None,
    });
    b.gate = Reading::Known(GateVerdict {
        verdict: "INCONCLUSIVE".to_string(),
        evidence: String::new(),
        findings: None,
        at: None,
    });
    let mut c = line("b-1-r3", vec![step(T0, "Stopped", None)]);
    c.review = b.review.clone();
    let d = line("b-1-r4", Vec::new());
    let h = history(&[a, b, c, d]);
    let row = |n, run: &str, end: Option<&str>, v: Option<&str>| Hist {
        n,
        run: run.to_string(),
        end: end.map(str::to_string),
        verdict: v.map(str::to_string),
    };
    assert_eq!(
        h,
        [
            row(1, "b-1-r1", Some("Reviewed"), Some("FAIL")),
            row(2, "b-1-r2", Some("Gated"), Some("INCONCLUSIVE")),
            row(3, "b-1-r3", Some("Stopped"), Some("PASS")),
            row(4, "b-1-r4", None, None),
        ]
    );
}

/// 表に無い段の語は流れに入れずまだ分からないとして返し、歴の終わりの段もまだ分からない（None）。
#[test]
fn bvstep_unknown_step_word() {
    assert!(known_stage("RateLimited") && !known_stage("Queued") && !known_stage("Warped"));
    let l = line(
        "b-1-r1",
        vec![step(T0, "Intake", None), step(T0 + 120, "Warped", None)],
    );
    let (segs, unknown) = flow(&l);
    assert_eq!(segs, [seg(Step::Review, T0, Some(T0 + 120))]);
    assert_eq!(unknown, ["Warped"]);
    assert_eq!(history(&[l])[0].end, None);
}

/// 着地の commit は段 Landed の detail の sha: の札の頭 12 字（札が無ければ None）。
#[test]
fn bvstep_landed_commit() {
    let sha = "6d2a7cd5c6efb1b6130534d1c9b6e547700f5cbf";
    let l = line(
        "b-1-r1",
        vec![
            step(T0, "Gated", Some("verdict:PASS")),
            step(T0 + 5, "Landed", Some(&format!("sha:{sha} main:{sha}"))),
            step(T0 + 9, "Landed", Some("terminal:push:origin")),
        ],
    );
    assert_eq!(COMMIT_CHARS, 12);
    assert_eq!(commit(&l), Some(sha[..12].to_string()));
    let none = line(
        "b-1-r2",
        vec![step(T0, "Landed", Some("terminal:push:origin"))],
    );
    assert_eq!(commit(&none), None);
}

fn base(stage: Stage, keys: &[&'static str]) -> Pop {
    Pop {
        id: "b-1".to_string(),
        short: "b-1".to_string(),
        title: None,
        summary: Sum::Unread,
        stage: Some(stage),
        facts: keys
            .iter()
            .map(|k| Fact {
                key: k,
                val: Val::Unknown,
            })
            .collect(),
        next: None,
    }
}

fn keys(p: &Pop) -> Vec<&'static str> {
    p.facts.iter().map(|f| f.key).collect()
}

/// 欄を差す所: Running / Gated は run の回の前に段の流れ・止まりは理由の後に run の歴・着地は着地の時刻の後に commit。
/// 走行が読めないか無ければまだ分からない。Blocked と Queued と札の無い bead は替えない。
#[test]
fn bvstep_with_runs_places_facts() {
    let [flow_key, hist_key, commit_key] = FLOW_KEYS;
    let run = [line(
        "b-1-r1",
        vec![
            step(T0, "Intake", None),
            step(T0 + 60, "Landed", Some("sha:0123456789abcdef")),
        ],
    )];
    let p = with_runs(
        base(Stage::Running, &["pf_id", STAGE_KEYS[3], STAGE_KEYS[4]]),
        Reading::Known(&run),
    );
    assert_eq!(keys(&p), ["pf_id", flow_key, STAGE_KEYS[3], STAGE_KEYS[4]]);
    assert_eq!(
        p.facts[1].val,
        Val::Flow(vec![seg(Step::Review, T0, Some(T0 + 60))], Vec::new())
    );
    let p = with_runs(
        base(Stage::Failed, &["pf_id", STAGE_KEYS[5], STAGE_KEYS[3]]),
        Reading::Known(&run),
    );
    assert_eq!(keys(&p), ["pf_id", STAGE_KEYS[5], hist_key, STAGE_KEYS[3]]);
    let p = with_runs(
        base(Stage::Landed, &["pf_id", STAGE_KEYS[6], STAGE_KEYS[7]]),
        Reading::Known(&run),
    );
    assert_eq!(
        keys(&p),
        ["pf_id", STAGE_KEYS[6], commit_key, STAGE_KEYS[7]]
    );
    assert_eq!(p.facts[2].val, Val::Code("0123456789ab".to_string()));
    for runs in [Reading::Unknown, Reading::Known(&[][..])] {
        let p = with_runs(base(Stage::Gated, &[STAGE_KEYS[3]]), runs);
        assert_eq!(p.facts[0].val, Val::Unknown);
    }
    for stage in [Stage::Blocked, Stage::Queued] {
        let before = base(stage, &["pf_id"]);
        assert_eq!(with_runs(before.clone(), Reading::Known(&run)), before);
    }
    let mut lone = base(Stage::Running, &["pf_id"]);
    lone.stage = None;
    assert_eq!(with_runs(lone.clone(), Reading::Known(&run)), lone);
}

/// 吹き出しの層は開いた bead の走行の読みの口を path の signal で読み（閉じていれば空の path で読まない）、欄を with_runs で足す。
#[test]
fn bvstep_dom_wiring_text() {
    let src = read("src/widgets/pop.rs");
    for s in [
        ".map(|id| timeline::path(&id))",
        "crate::net::read_path(runs_path)",
        "with_runs(pop(&id, &src), known(&lines))",
        "Val::Flow(segs, unknown) =>",
        "Val::Hist(rows) =>",
    ] {
        assert!(src.contains(s), "{s}");
    }
    let css = read("style.css");
    for c in ["ptl", "ptlt", "plive", "phist", "pfail"] {
        assert!(css.contains(&format!(".{c} ")), "{c}");
    }
}
