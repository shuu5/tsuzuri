// flip-check: moved t3-hub.92.10.5
//! 観測と印の面（`dispatch ls` の行・止めの行・印の記帳）。親の `dispatch.rs` から純移動した。

use super::{bundle, measure, memo, permits, precheck, Candidate, Input, Turn, WaitReason, DASH, WHY_PREFIX};
use crate::cli_outcome::Outcome;
use crate::fleet::store;
use crate::fleet::Mark;
use std::path::Path;

/// 列の 1 行の書き出し（設計 §6）。
const LINE: &str = "[DISPATCH]";

/// 件数の行の書き出し。
const COUNT: &str = "[DISPATCH-COUNT]";

/// 列が空の周の行（**台帳を読めない周と融合しない**・C10）。
const NONE_LINE: &str = "[DISPATCH-NONE]";

/// 効いている止めの行の書き出し（件数の行の前・候補の理由が `hold` の件ごと）。
const HOLD_LINE: &str = "[DISPATCH-HOLD]";

/// `pipe dispatch` の使い方。
pub fn usage() -> String {
    format!(
        "usage: {} pipe dispatch <ls|first|hold|release> [BEAD] [--reason WORDS（hold は要る）] [--state-dir D] [--repo R] [--bd PATH] [--rules PATH]",
        crate::name::NAME
    )
}

/// 列の 1 周の結果の 1 行（終端と手動の 1 周が stdout に足す・設計 §5）。
///
/// **0 件と「測れない」を融合しない**（C10）: 台帳を読めない周は件数でなく理由を名乗る。
pub fn line(turn: &Turn) -> String {
    match turn.unmeasured {
        Some(reason) => format!("dispatch=unmeasured reason={}", reason.as_str()),
        None => {
            let counts = format!(
                "dispatch=started:{},resumed:{},waiting:{}",
                turn.launches.len(),
                turn.revives.len(),
                turn.candidates.len().saturating_sub(turn.launches.len())
            );
            // **`--drive` の周だけ token を足す**（観測の面を増やさない・§6）: flag の無い周の行は
            // 1 byte も変わらない＝段を手で 1 つずつ進める既存の歯は 1 本も動かない。
            match turn.drive {
                None => counts,
                Some(drive) => format!("{counts} drive={}", drive.as_str()),
            }
        }
    }
}

/// 終端の周の軸を評価した周の 1 行（`vessel=<値>`・評価していない周は `None`・設計 consumer-sync.md §15 形 3）。
///
/// [`line`] とは**別の行**である（`dispatch=` の行の書式と `drive=` の token は 1 字も変えない）。
pub fn vessel_line(turn: &Turn) -> Option<String> {
    turn.vessel.as_ref().map(|found| format!("vessel={}", found.render()))
}

/// 列の 1 周を描く（`dispatch ls`・**観測の面はこの 1 口だけである**・設計 §6）。
pub fn render(turn: &Turn) -> Outcome {
    if let Some(reason) = turn.unmeasured {
        return Outcome::ok(vec![format!("[DISPATCH-UNMEASURED reason={}]", reason.as_str())]);
    }
    if turn.candidates.is_empty() {
        return Outcome::ok(vec![NONE_LINE.to_owned()]);
    }
    let mut out: Vec<String> = turn.candidates.iter().map(line_of).collect();
    out.push(format!("{COUNT} total={} ready={}", turn.candidates.len(), turn.launches.len()));
    Outcome::ok(out)
}

/// `dispatch ls` の全行（[`render`] の件数の行の前に、依存待ちの候補ごとの事前審査の 1 行を足す・結果の file を読むだけで
/// 撃たない・設計 §27 形 7）と、その後ろに直しの束ごとの 1 行（行 y・形 2）を足す。`[DISPATCH]` と `[DISPATCH-COUNT]` の行の字は
/// [`render`] のまま。
pub fn listing(input: &Input<'_>, turn: &Turn) -> Outcome {
    let mut outcome = render(turn);
    if let Some(at) = outcome.out.iter().position(|line| line.starts_with(COUNT)) {
        let held = turn.candidates.iter().filter_map(hold_line);
        outcome.out.splice(at..at, held.chain(precheck::lines(input, turn)).chain(bundle::lines(input.state_dir)));
    }
    outcome
}

/// `dispatch ls` の全行（[`listing`] の後ろに memo ごとの 1 行を足す・同じ 1 回の読みから判じ、台帳を読めない周は足さない・設計 §40）。
pub fn observe(input: &Input<'_>) -> Outcome {
    let (turn, read) = measure(input);
    let mut outcome = listing(input, &turn);
    let now = crate::seat::state::now_secs();
    outcome.out.extend(read.iter().flat_map(|found| memo::lines(input, found, now).into_iter().chain(permits::lines(input, found, now))));
    outcome
}

/// 列の 1 件の行。
fn line_of(candidate: &Candidate) -> String {
    let prio = candidate.priority.map_or(DASH.to_owned(), |found| found.to_string());
    let mark = candidate.mark.map_or(DASH, Mark::as_str);
    let reason = candidate.reason.as_ref().map_or(DASH.to_owned(), WaitReason::render);
    format!("{LINE} bead={} prio={prio} mark={mark} reason={reason}", candidate.bead)
}

/// 理由が `hold` の候補の止めの 1 行（`[DISPATCH-HOLD] bead=<id> since=<印の時刻> why=<理由>`・理由の無い古い印は `why=-`）。
pub(super) fn hold_line(candidate: &Candidate) -> Option<String> {
    let Some(WaitReason::Hold { ref since, ref why }) = candidate.reason else {
        return None;
    };
    Some(format!("{HOLD_LINE} bead={} since={since} why={}", candidate.bead, why.as_deref().unwrap_or(DASH)))
}

/// 印の `--reason` を受けて印の行の detail を返す（`hold` は理由を要り、空白だけと改行を含む理由を断る・
/// ほかの印は `--reason` を断る・断りは印を書かない使い方の誤り）。器は理由の字を判定に使わない（設計 §4）。
pub fn why_of(mark: Mark, words: Option<&str>) -> Result<Option<String>, String> {
    match (mark, words) {
        (Mark::Hold, None) => Err("hold は --reason の理由を要る".to_owned()),
        (Mark::Hold, Some(found)) if found.trim().is_empty() => Err("--reason の理由が空である".to_owned()),
        (Mark::Hold, Some(found)) if found.contains(['\n', '\r']) => Err("--reason の理由が改行を含む".to_owned()),
        (Mark::Hold, Some(found)) => Ok(Some(format!("{WHY_PREFIX}{found}"))),
        (_, Some(_)) => Err(format!("--reason は hold だけが受ける（{}）", mark.as_str())),
        (_, None) => Ok(None),
    }
}

/// 介入の印を記帳する（`dispatch first|hold|release <bead>`・設計 §4・`hold` の detail は [`why_of`] の字）。
///
/// 印は台帳の priority を書き換えない（憲法 C15）。`release` も 1 行として残す——印を外した事実が
/// 記録から消えると「なぜこの順か」が読めなくなる（設計 §10）。
pub fn mark(state_dir: &Path, bead: &str, mark: Mark, detail: Option<String>, policy: store::LockPolicy) -> Outcome {
    match super::emit_mark(state_dir, bead, mark, detail, policy) {
        Ok(()) => Outcome::ok(vec![format!("{LINE} bead={bead} mark={}", mark.as_str())]),
        Err(err) => Outcome::failed_line(crate::cli_outcome::RC_BROKEN, format!("pipe: {err}")),
    }
}
