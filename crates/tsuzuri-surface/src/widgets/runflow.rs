//! 吹き出しの run の段の流れと run の歴（判断の記録 ADR-27 決定 (7) と (10)・見本 board-v2 の stepsHTML と histTable）:
//! 吹き出しを開いた時に走行の読みの口（`timeline::path` の `/api/runs?bead=`）を読み、最後の走行の段の流れ（審査・実装・gate の
//! 始まりと長さ・走っている段は「〜」）と、走行ごとの歴（回・終わりの段・結び）と、着地の commit を吹き出しの欄に足す。
//! 段の語は器の case-lifecycle.md §2.1 の便の段の写しの 11 値を写した閉じた表（`STAGES`）で読み、表に無い語はまだ分からない。
//! 審査は Intake から次の段まで・実装は Spawned から次の段まで・gate は Implemented から次の段まで（次の段が無ければ走っている）。
//! 欄の組みは純粋な関数にして host で試し、読みの口の signal は吹き出しの層（`pop::PopLayer`）が持つ。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{Reading, Stage};
use tsuzuri_contract::runs::{RunLine, RunsDoc};
use tsuzuri_contract::wire;

use super::pop::{Fact, Pop, STAGE_KEYS, UNKNOWN_KEY, Val};
use crate::project::pipeline;
use crate::view::Fetched;
use crate::vocab::label;

/// 流れの段（審査・実装・gate）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Review,
    Implement,
    Gate,
}

impl Step {
    /// 語の鍵。
    pub fn key(self) -> &'static str {
        match self {
            Step::Review => "fl_review",
            Step::Implement => "fl_impl",
            Step::Gate => "fl_gate",
        }
    }
}

/// 器の便の段の 11 値（case-lifecycle.md §2.1 の表の順）と、その段で始まる流れの段（None は始めない）。
pub const STAGES: [(&str, Option<Step>); 11] = [
    ("Intake", Some(Step::Review)),
    ("Reviewed", None),
    ("Blocked", None),
    ("Spawned", Some(Step::Implement)),
    ("Questioned", None),
    ("RateLimited", None),
    ("Implemented", Some(Step::Gate)),
    ("Gated", None),
    ("Landed", None),
    ("Stopped", None),
    ("Failed", None),
];

/// 段の流れと run の歴と commit の欄の語の鍵。
pub const FLOW_KEYS: [&str; 3] = ["pf_flow", "pf_hist", "pf_commit"];

/// 着地の event の detail の commit の札の頭（器の RunDone の `sha:<commit> main:<commit>`）。
pub const SHA_HEAD: &str = "sha:";

/// 吹き出しに出す commit の字数。
pub const COMMIT_CHARS: usize = 12;

/// 走っている段の長さの後に付ける字（見本の「〜」）。
pub const OPEN_END: &str = "〜";

/// 段の語が閉じた表に在るか。
pub fn known_stage(word: &str) -> bool {
    STAGES.iter().any(|(s, _)| *s == word)
}

/// 流れの 1 段（始まりと終わりの時刻・次の段の event が無ければ走っている）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seg {
    pub step: Step,
    pub from: Option<EpochSecs>,
    pub to: Option<EpochSecs>,
    pub open: bool,
}

/// 走行の段の流れ（段の event の順に、流れを始める段から次の段の event までを 1 段にする）と、表に無い段の語。
pub fn flow(line: &RunLine) -> (Vec<Seg>, Vec<String>) {
    let mut segs: Vec<Seg> = Vec::new();
    let mut unknown = Vec::new();
    for s in &line.steps {
        if let Some(last) = segs.last_mut().filter(|l| l.open) {
            last.to = s.at;
            last.open = false;
        }
        match STAGES.iter().find(|(w, _)| *w == s.stage) {
            Some((_, Some(step))) => segs.push(Seg {
                step: *step,
                from: s.at,
                to: None,
                open: true,
            }),
            Some((_, None)) => {}
            None => unknown.push(s.stage.clone()),
        }
    }
    (segs, unknown)
}

/// 段の長さの秒（終わりか、走っている段は今から始まりを引く・時刻が欠ければ None）。
pub fn seg_secs(s: &Seg, now: EpochSecs) -> Option<u64> {
    let to = if s.open { Some(now) } else { s.to };
    Some(to?.saturating_sub(s.from?))
}

/// 流れの字（「審査 10m → 実装 22m〜」の形・長さが分からなければ「―」）。
pub fn flow_text(segs: &[Seg], now: EpochSecs) -> String {
    segs.iter()
        .map(|s| {
            let len = seg_secs(s, now).map_or_else(|| pipeline::NO_AGE.to_string(), pipeline::age);
            let open = if s.open { OPEN_END } else { "" };
            format!("{} {len}{open}", label(s.step.key()))
        })
        .collect::<Vec<_>>()
        .join(" → ")
}

/// 走行の歴の 1 行（回・終わりの段の語・結び）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hist {
    pub n: usize,
    pub run: String,
    /// 最後の段の語（表に無い語と段の無い走行は None＝まだ分からない）。
    pub end: Option<String>,
    /// 結び（gate の判定・無ければ審査の判定・無ければ段の審査の結び・どれも無ければ None）。
    pub verdict: Option<String>,
}

/// 走行ごとの歴（走行の順・回は 1 から）。
pub fn history(lines: &[RunLine]) -> Vec<Hist> {
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let end = l
                .steps
                .last()
                .map(|s| s.stage.clone())
                .filter(|w| known_stage(w));
            let verdict = match (&l.gate, &l.review) {
                (Reading::Known(g), _) => Some(g.verdict.clone()),
                (_, Reading::Known(r)) => Some(r.verdict.clone()),
                _ => l.steps.iter().rev().find_map(|s| s.verdict.clone()),
            };
            Hist {
                n: i + 1,
                run: l.run.clone(),
                end,
                verdict,
            }
        })
        .collect()
}

/// 着地の commit（最後の走行の段 Landed の detail の `SHA_HEAD` の札の値の頭 `COMMIT_CHARS` 字）。
pub fn commit(line: &RunLine) -> Option<String> {
    line.steps
        .iter()
        .filter(|s| s.stage == "Landed")
        .filter_map(|s| s.detail.as_deref())
        .flat_map(str::split_whitespace)
        .find_map(|t| t.strip_prefix(SHA_HEAD))
        .map(|sha| sha.chars().take(COMMIT_CHARS).collect())
}

/// 走行の読みの口の本文を走行の列に読む（読めなければ Unknown）。
pub fn read_runs(fetched: &Fetched) -> Reading<Vec<RunLine>> {
    match fetched {
        Fetched::Body(text) => wire::decode::<RunsDoc>(text).map_or(Reading::Unknown, |d| d.runs),
        Fetched::NotRead | Fetched::Failed => Reading::Unknown,
    }
}

/// 欄の値（走行が読めないか走行が無ければまだ分からない）。
fn val_of(runs: &Reading<&[RunLine]>, f: impl FnOnce(&[RunLine]) -> Option<Val>) -> Val {
    match runs {
        Reading::Known(lines) if !lines.is_empty() => f(lines).unwrap_or(Val::Unknown),
        _ => Val::Unknown,
    }
}

/// 欄を鍵の後ろに差す（鍵の欄が無ければ末に足す）。
fn insert_after(facts: &mut Vec<Fact>, after: &str, fact: Fact) {
    let at = facts
        .iter()
        .position(|f| f.key == after)
        .map_or(facts.len(), |i| i + 1);
    facts.insert(at, fact);
}

/// 吹き出しに走行の欄を足す: Running / Gated は run の回の前に段の流れ・止まりは理由の後に run の歴・着地は着地の時刻の後に commit。
/// ほかの段と札の無い bead は替えない。
pub fn with_runs(mut p: Pop, runs: Reading<&[RunLine]>) -> Pop {
    let Some(stage) = p.stage else {
        return p;
    };
    let [flow_key, hist_key, commit_key] = FLOW_KEYS;
    match stage {
        Stage::Running | Stage::Gated => {
            let val = val_of(&runs, |l| {
                let (segs, unknown) = flow(l.last()?);
                Some(Val::Flow(segs, unknown))
            });
            let at = p
                .facts
                .iter()
                .position(|f| f.key == STAGE_KEYS[3])
                .unwrap_or(p.facts.len());
            p.facts.insert(at, Fact { key: flow_key, val });
        }
        Stage::Questioned | Stage::Failed | Stage::Stopped => {
            let val = val_of(&runs, |l| Some(Val::Hist(history(l))));
            insert_after(&mut p.facts, STAGE_KEYS[5], Fact { key: hist_key, val });
        }
        Stage::Landed => {
            let val = val_of(&runs, |l| commit(l.last()?).map(Val::Code));
            insert_after(
                &mut p.facts,
                STAGE_KEYS[6],
                Fact {
                    key: commit_key,
                    val,
                },
            );
        }
        Stage::Blocked | Stage::Held | Stage::Queued => {}
    }
    p
}

/// 表に無い段の語の字（まだ分からないの語と語）。
pub fn unknown_text(word: &str) -> String {
    format!("{}（{word}）", label(UNKNOWN_KEY))
}
