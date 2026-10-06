//! 起動と連鎖（`pipe spawn` / `pipe run`・設計 §5「subcommand」）。
//!
//! runner を起こす経路は [`launch`] 1 本で、`pipe run` は intake → 審査 → spawn → ride_out → gate → land を
//! 1 process で連続させる。`s2-07l.295` で `cli.rs` から純移動した（本文は不変・連鎖の順序は宣言順のまま）。
//! `resume` と `follow_pending` は `cli.rs` に残る（run 1 の裁定 (b)・契約表 row c の閉包を保つ）。
//! **起こせるのは `Reviewed` かつ verdict PASS の便だけ**（FR49・[`Extra::Spawn`]・設計 contract-source.md §4）。

use super::intake::{intake_id, intake_line};
use super::step::{gate_run, land_run};
use super::{broken, int_row, manifest_of, need, refused, resolve, review_then_launch, state_dir_of, Extra, Resolved};
use crate::cli_outcome::{Outcome, RC_BROKEN, RC_OK, RC_REFUSED};
use crate::fleet::json_lite;
use crate::fleet::store::{self, LockPolicy};
use crate::fleet::{Event, EventKind, Stage};
use crate::pipe::follow::{self, Runner, Turn, FAIL_DETAIL, FIX_DETAIL};
use crate::pipe::gate::Verdict;
use crate::pipe::land::verdict_of;
use crate::pipe::ratelimit::{ride_out_rate_limit, Pool};
use crate::pipe::spawn::EndGate;
use crate::pipe::{current, emit, verdict_path, Emit};
use crate::rules::manifest::Manifest;
use std::path::Path;

/// gate の FAIL の直しの周の回数の上限を持つ rules 行（設計 pipeline.md §73）。
const ROW_GATE_FIX_ROUNDS: &str = "runner.gate_fix_rounds";

/// `pipe spawn`。前提 stage = `Reviewed`（verdict PASS）。
pub(super) fn start(args: &[String], policy: LockPolicy) -> Outcome {
    let parsed = (|| Ok::<_, String>((need(args, "--run")?.to_owned(), need(args, "--runner")?.to_owned())))();
    let (id, runner) = match parsed {
        Ok(found) => found,
        Err(reason) => return refused(reason),
    };
    launch(args, &id, &runner, policy, &[Stage::Reviewed])
}

/// 段を確かめてから turn の口を通す。
///
/// **runner を起こす経路はここ 1 本**で、材料を解いた後は `pipe::follow` の turn へ渡す
/// （Precheck → spawn → 追随の後始末が 1 本に収まる＝起こし直しと通常の起動で後始末が
/// 分かれない）。`Reviewed` の便は verdict が PASS の周だけ通る（[`Extra::Spawn`]・他の段は段の一致だけ）。
///
/// 口座は器が選ぶ（設計 account-autonomy.md §4「初回の起動も同じ選定を通す」・FR36）: 口座の宣言（`--rules` の
/// tracked の面 + 置き場の host の面・[`Pool::declared`]）が 1 つ以上在る周は `RateLimited` の再開と同じ 1 関数で
/// label を選び、0 の周は親の環境を継承する。操作役に口座を選ばせる flag は無い。待ちの間に便が居るはずの段は
/// 解いた現在の段（[`Resolved::stage`]）。manifest は `resume` の各段の口（`cli.rs` の `relaunch`）が渡さないので、
/// `--rules`（無ければ埋め込み）から同じ 1 本（[`manifest_of`]）で読み直す。
pub(super) fn launch(
    args: &[String],
    id: &str,
    runner: &str,
    policy: LockPolicy,
    allowed: &[Stage],
) -> Outcome {
    let resolved = match resolve(args, id, allowed, &Extra::Spawn) {
        Ok(found) => found,
        Err(outcome) => return outcome,
    };
    let manifest = match manifest_of(args) {
        Ok(found) => found,
        Err(reason) => return refused(reason),
    };
    let pool = match Pool::declared(args, &manifest, &resolved.state_dir) {
        Ok(found) => found,
        Err(reason) => return refused(reason),
    };
    let gate = EndGate::of(&manifest);
    let runner = Runner { cmd: runner, pool: pool.as_ref(), gate: &gate };
    follow::spawn_selected(&turn_of(id, &resolved, runner, policy), resolved.stage)
}

/// turn の材料を解いた面から組む（起動と別口座での起こし直しが同じ 1 本で組む）。
pub(in crate::pipe) fn turn_of<'a>(id: &'a str, resolved: &'a Resolved, runner: Runner<'a>, policy: LockPolicy) -> Turn<'a> {
    Turn {
        run: id,
        bead: &resolved.bead,
        repo: &resolved.repo,
        state_dir: &resolved.state_dir,
        contract: &resolved.contract,
        runner: Some(runner),
        approved: resolved.approved,
        policy,
    }
}

/// `pipe run`。intake → 審査 → spawn → gate → land を 1 process で連続させる。
///
/// 各段は永続面を読み書きするので、途中で落ちても `resume` が続きを引ける。
pub(super) fn run_all(
    args: &[String],
    manifest: &Manifest,
    policy: LockPolicy,
    driven: &mut Option<super::Driven>,
) -> Outcome {
    let runner = match need(args, "--runner") {
        Ok(found) => found.to_owned(),
        Err(reason) => return refused(reason),
    };
    let (id, index) = match intake_id(args, manifest, policy) {
        Ok(found) => found,
        Err(outcome) => return outcome,
    };
    // **自分が駆動する便を名乗る**（設計 dispatcher.md §5）: 便はいま作ったので入口に段は無い
    // ＝どの段に着いても前進である。`--drive` を読むのは呼び手（[`super::dispatch`]）の 1 か所。
    *driven = Some(super::Driven { run: id.clone(), entry: None });
    // **driver の札をここで置く**（設計 dispatcher.md §5）: この process が死んだら、札が残って列の
    // 1 周が起こし直す。`Drop` で消えるので、どの段で終わっても残らない。
    // 握れない周は駆動しない（新しい run id なので、ここで落ちるのは置き場を書けない周だけ）。
    let held = super::state_dir_of(args).ok().map(|state_dir| crate::pipe::Driver::hold(&state_dir, &id, policy));
    let _driver = match held {
        Some(None) => return super::broken(format!("run {id} の driver の札を握れない")),
        Some(found) => found,
        None => None,
    };
    // **run id は落ちた周も stdout に出す**。`resume` がこの id を要るためで、
    // ここで黙ると続きから引けない便が置き場に残る。
    // 索引を作れない周の尾（` index=unavailable:<語>`）は受付の本体が返した字をそのまま足す（write-set の欄と base の木の欄は足さない）。
    let mut lines = vec![format!("{}{index}", intake_line(args, &id))];
    // **段の通知は rc に依らず段の順で持つ**（設計 §21 (1)）: gate の `lens-input=…` のような行は
    // rc 0 の段が出すので、畳むときに捨てると連鎖で撃った周だけ理由が消える（`.286` の実測）。
    // 最初の通知は器の binary の留めの 1 行（審査の前に留め、同じ便の子は留めを撃つ・行 v-pin）。
    let mut notes: Vec<String> = super::state_dir_of(args).ok().map(|dir| crate::pipe::pin::note(&dir, &id)).into_iter().collect();
    // 審査が PASS でない周は spawn の前で止まる（構築点の呼出 0・AC22）。
    let spawned = review_then_launch(args, &id, &runner, manifest, policy);
    if let Some(stopped) = chain_noting(&mut lines, &mut notes, spawned) {
        return stopped;
    }
    // runner が口座の上限で止まった周は別口座で起こし直してから gate へ（設計 account-autonomy.md §4）。
    let ridden = ride_out_rate_limit(args, &id, &runner, manifest, policy);
    if let Some(stopped) = chain_noting(&mut lines, &mut notes, ridden) {
        return stopped;
    }
    let gated = gate_fixing(args, &id, Some(runner.as_str()), manifest, policy);
    if let Some(stopped) = chain_noting(&mut lines, &mut notes, gated) {
        return stopped;
    }
    let landed = land_fixing(args, &id, Some(runner.as_str()), manifest, policy);
    if let Some(stopped) = chain_noting(&mut lines, &mut notes, landed) {
        return stopped;
    }
    Outcome { out: lines, err: notes, rc: RC_OK }
}

/// gate の FAIL の直しの周の数え（設計 pipeline.md §73）: 便の `RunStage` のうち段 `Implemented` で detail が [`FIX_DETAIL`] で始まる
/// 記帳（直しの印）の数と、便の最後の `RunStage` がその印か（印の後に runner が起きる前に driver が死んだ便の判じ）。
///
/// 周の数は便の event から数え、process の記憶に持たない（FR1003 と同じ）。段 `Spawned` の `gate-fix:` の記帳（直しの周の起動）は数えない。
pub(super) fn fix_rounds(events: &[Event], run: &str) -> (u64, bool) {
    let is_mark = |event: &&Event| {
        event.stage == Some(Stage::Implemented) && event.detail.as_deref().is_some_and(|detail| detail.starts_with(FIX_DETAIL))
    };
    let mut own = events.iter().filter(|event| event.run == run && event.kind == EventKind::RunStage);
    let count = own.clone().filter(is_mark).count();
    (u64::try_from(count).unwrap_or(u64::MAX), own.next_back().is_some_and(|event| is_mark(&event)))
}

/// gate を撃った後の便の閉じた 3 値（[`fix_due`]）。
enum Due {
    /// 直しの周に入る（値は周の番号・1 始まり）。
    Round(u64),
    /// 上限に届いた（値は上限）。
    Exhausted(u64),
    /// 直しの周の対象でない。
    Off,
}

/// gate を撃った後の便が直しの周に入るか。`Off` は、runner の字が無い・行 `runner.gate_fix_rounds` を読めない（行の無い・不発効・型違いの
/// manifest）・event を読めない・便の最後の `RunStage` が段 `Gated` でないか detail が `verdict:FAIL` で始まらない（前の周の `Gated` の FAIL を
/// 読み直さない）・`verdict.json` の `verify_red` が 0 でないか読めない（verify の赤は審査役の所見が無い）の周である。
fn fix_due(args: &[String], id: &str, runner: Option<&str>, manifest: &Manifest) -> Due {
    let (Some(_), Ok(limit), Ok(state_dir)) = (runner, int_row(manifest, ROW_GATE_FIX_ROUNDS), state_dir_of(args)) else {
        return Due::Off;
    };
    let Ok(events) = store::read_all(&state_dir) else {
        return Due::Off;
    };
    let last = events.iter().rev().find(|event| event.run == id && event.kind == EventKind::RunStage);
    let failed = last.is_some_and(|event| {
        event.stage == Some(Stage::Gated) && event.detail.as_deref().is_some_and(|detail| detail.starts_with(FAIL_DETAIL))
    });
    if !failed || !verify_green(&state_dir, id) {
        return Due::Off;
    }
    match fix_rounds(&events, id).0 {
        done if done < limit => Due::Round(done + 1),
        _ => Due::Exhausted(limit),
    }
}

/// `verdict.json` の `verify_red` が 0 か（読めない・欄が無い周は偽）。
fn verify_green(state_dir: &Path, id: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(verdict_path(state_dir, id)) else {
        return false;
    };
    let Ok(pairs) = json_lite::parse_object(text.trim()) else {
        return false;
    };
    pairs.iter().find(|(key, _)| key == "verify_red").and_then(|(_, value)| value.as_num()) == Some(0)
}

/// 直しの印（段 `Implemented`・detail `gate-fix:<周>`）を 1 件記帳する（書けない周は rc 2）。記帳の門は [`emit`] の `NotStopped`。
fn mark_fix(args: &[String], id: &str, round: u64, policy: LockPolicy) -> Result<(), Outcome> {
    let state_dir = state_dir_of(args).map_err(refused)?;
    let state = current(&state_dir)
        .map_err(|errors| Outcome::failed(RC_BROKEN, errors.iter().map(ToString::to_string).collect()))?;
    let bead = state.runs.get(id).map(|run| run.bead.as_str()).unwrap_or_default();
    let entry = Emit {
        kind: EventKind::RunStage,
        run: id,
        bead,
        stage: Some(Stage::Implemented),
        seat: None,
        pid: None,
        detail: Some(format!("{FIX_DETAIL}{round}")),
    };
    emit(&state_dir, &entry, policy).map_err(|err| broken(err.to_string()))
}

/// 段の出力の前に、それまでの行と通知を積んだ形で返す（段の順・[`chain_noting`] と同じ畳み方で rc は後ろの段のもの）。
fn closing(mut lines: Vec<String>, mut notes: Vec<String>, last: Outcome) -> Outcome {
    lines.extend(last.out);
    notes.extend(last.err);
    Outcome { out: lines, err: notes, rc: last.rc }
}

/// gate を撃ち、審査役の FAIL の便は同じ worktree の runner を所見の節つきで起こし直してから gate を撃ち直す輪（設計 pipeline.md §73）。
///
/// gate の rc が [`RC_REFUSED`] で [`fix_due`] が `Round(n)` の周は、直しの印を記帳し、stdout に `run=<id> gate-fix=<n>/<上限>` を足し、runner を
/// 起こして上限の周の待ち（[`ride_out_rate_limit`]）まで済ませ、どちらかが rc 0 でなければそこまでの行とその rc で返る。`Exhausted(v)` の周は
/// 行の末に `run=<id> gate-fix=exhausted:<v>` を足して gate の rc のまま返る。`Off` の周と rc が [`RC_REFUSED`] でない周は gate の戻りのまま返る。
/// **輪は回数を数えない**——止めるのは [`fix_due`] の数え（便の event）である。
pub(super) fn gate_fixing(args: &[String], id: &str, runner: Option<&str>, manifest: &Manifest, policy: LockPolicy) -> Outcome {
    let (mut lines, mut notes) = (Vec::new(), Vec::new());
    loop {
        let mut gated = gate_run(args, id, manifest, policy);
        let due = if gated.rc == RC_REFUSED { fix_due(args, id, runner, manifest) } else { Due::Off };
        let (cmd, round) = match (runner, due) {
            (Some(cmd), Due::Round(round)) => (cmd, round),
            (Some(_), Due::Exhausted(limit)) => {
                gated.out.push(format!("run={id} gate-fix=exhausted:{limit}"));
                return closing(lines, notes, gated);
            }
            _ => return closing(lines, notes, gated),
        };
        lines.append(&mut gated.out);
        notes.append(&mut gated.err);
        let fixed = fix_round(args, id, (cmd, round), manifest, policy);
        if let Some(stopped) = chain_noting(&mut lines, &mut notes, fixed) {
            return stopped;
        }
    }
}

/// 直しの周 1 回: 直しの印を記帳し、stdout に `run=<id> gate-fix=<周>/<上限>` を足し、runner を起こして上限の周の待ち（[`ride_out_rate_limit`]）まで
/// 済ませる（[`gate_fixing`] と [`land_fixing`] が共有する）。どれかが rc 0 でなければそこまでの行とその rc で返る。`runner` は `(cmd, 周の番号)`。
fn fix_round(args: &[String], id: &str, (cmd, round): (&str, u64), manifest: &Manifest, policy: LockPolicy) -> Outcome {
    let (mut lines, mut notes) = (Vec::new(), Vec::new());
    // 行 `runner.gate_fix_rounds` は `fix_due` が読めた周だけここへ来る。
    let limit = int_row(manifest, ROW_GATE_FIX_ROUNDS).unwrap_or_default();
    if let Err(outcome) = mark_fix(args, id, round, policy) {
        return outcome;
    }
    lines.push(format!("run={id} gate-fix={round}/{limit}"));
    let launched = launch(args, id, cmd, policy, &[Stage::Implemented]);
    if let Some(stopped) = chain_noting(&mut lines, &mut notes, launched) {
        return stopped;
    }
    let ridden = ride_out_rate_limit(args, id, cmd, manifest, policy);
    chain_noting(&mut lines, &mut notes, ridden).unwrap_or(Outcome { out: lines, err: notes, rc: RC_OK })
}

/// land を撃ち、追随の再 gate が審査役の FAIL を返した便は gate の段と同じ直しの周に入れる輪（設計 pipeline.md §73・[`gate_fixing`] の land 版）。
///
/// land の前に便の verdict が PASS かを読み、PASS でない周は land の戻りのまま返る（Gated の FAIL の便に直しの周へ入らない）。PASS の周は land の rc が
/// [`RC_REFUSED`] で [`fix_due`] が `Round(n)` なら直しの周（[`fix_round`]）と直した後の gate（[`gate_fixing`]）を撃ってから land を撃ち直し、`Exhausted(v)` なら行の末に
/// `run=<id> gate-fix=exhausted:<v>` を足して land の rc のまま返る。`Off` の周と rc が [`RC_REFUSED`] でない周は land の戻りのまま返る。
/// 周の数えは gate の段と共有する（[`fix_rounds`]・段と再 gate を分けない）。**輪は回数を数えない**——止めるのは [`fix_due`] の数えである。
pub(super) fn land_fixing(args: &[String], id: &str, runner: Option<&str>, manifest: &Manifest, policy: LockPolicy) -> Outcome {
    let (mut lines, mut notes) = (Vec::new(), Vec::new());
    loop {
        let passing = state_dir_of(args).is_ok_and(|state_dir| verdict_of(&state_dir, id) == Some(Verdict::Pass));
        let mut landed = land_run(args, id, manifest, policy);
        let due = if passing && landed.rc == RC_REFUSED { fix_due(args, id, runner, manifest) } else { Due::Off };
        let (cmd, round) = match (runner, due) {
            (Some(cmd), Due::Round(round)) => (cmd, round),
            (Some(_), Due::Exhausted(limit)) => {
                landed.out.push(format!("run={id} gate-fix=exhausted:{limit}"));
                return closing(lines, notes, landed);
            }
            _ => return closing(lines, notes, landed),
        };
        lines.append(&mut landed.out);
        notes.append(&mut landed.err);
        let fixed = fix_round(args, id, (cmd, round), manifest, policy);
        if let Some(stopped) = chain_noting(&mut lines, &mut notes, fixed) {
            return stopped;
        }
        let gated = gate_fixing(args, id, Some(cmd), manifest, policy);
        if let Some(stopped) = chain_noting(&mut lines, &mut notes, gated) {
            return stopped;
        }
    }
}

/// 段の結果を畳む。rc≠0 ならそこまでの行を載せて**止める形**を返す。
///
/// 段の `err` は `notes` へ**段の順で**積む（設計 §21 (1)）。rc 0 の段の `err` をここで捨てると、
/// gate が `lens-input=<kind> reason=<語>` を出した周でも `pipe run` の stderr が 0 byte になる
/// ——連鎖で撃った便ほど理由が読めない（`.286`）。**stdout（`out`）の畳み方と rc の極性は不変**。
pub(super) fn chain_noting(
    lines: &mut Vec<String>,
    notes: &mut Vec<String>,
    outcome: Outcome,
) -> Option<Outcome> {
    let mut outcome = outcome;
    notes.append(&mut outcome.err);
    if outcome.rc == RC_OK {
        lines.extend(outcome.out);
        return None;
    }
    let mut stopped = outcome;
    let mut out = std::mem::take(lines);
    out.extend(stopped.out);
    stopped.out = out;
    stopped.err = std::mem::take(notes);
    Some(stopped)
}

/// 段の通知を継がない口（`resume` の 2 段・`cli.rs`）。
///
/// `resume` は段ごとに撃ち直す形で、返るのは**その周の段の stderr** だけである（従来の形・
/// `pipe run` の 4 段の連鎖とは別）。中身は [`chain_noting`] の 1 本で、`notes` を持たない呼び手が
/// 同じ畳み方を通るための薄い口である。
pub(super) fn chain(lines: &mut Vec<String>, outcome: Outcome) -> Option<Outcome> {
    let mut notes = Vec::new();
    chain_noting(lines, &mut notes, outcome)
}

#[cfg(test)]
mod tests {
    use super::fix_rounds;
    use crate::fleet::{EventKind, Stage};
    use crate::pipe::fixture::event;

    /// 便 `run` の `RunStage`（段と detail）の 1 件。
    fn stage(run: &str, stage: Stage, detail: &str) -> crate::fleet::Event {
        event(run, EventKind::RunStage, Some(stage), None, Some(detail))
    }

    /// 段 `Implemented` の直しの印の数と、便の最後の `RunStage` が印かを返す。ほかの便の記帳・段 `Spawned` の `gate-fix:` の記帳・
    /// 印の後の `Gated` は数えず、最後の `RunStage` が印でなくなる。
    #[test]
    fn gate_fix_rounds_count_the_markers_and_see_a_pending_one() {
        assert_eq!(fix_rounds(&[], "r1"), (0, false), "記帳が無い便は 0 と偽");
        let mut events = vec![
            stage("r1", Stage::Gated, "verdict:FAIL,rules:embedded"),
            stage("r1", Stage::Implemented, "gate-fix:1"),
        ];
        assert_eq!(fix_rounds(&events, "r1"), (1, true), "印の直後は 1 と真");
        events.push(stage("r2", Stage::Implemented, "gate-fix:1"));
        events.push(stage("r2", Stage::Implemented, "gate-fix:2"));
        assert_eq!(fix_rounds(&events, "r1"), (1, true), "ほかの便の記帳は数えず、最後の RunStage の判じにも入らない");
        assert_eq!(fix_rounds(&events, "r2"), (2, true), "ほかの便の側から見ても同じ");
        events.push(stage("r1", Stage::Spawned, "gate-fix:1"));
        assert_eq!(fix_rounds(&events, "r1"), (1, false), "段 Spawned の gate-fix: の記帳は数えず、最後の RunStage は印でない");
        events.push(stage("r1", Stage::Implemented, "gate-fix:2"));
        assert_eq!(fix_rounds(&events, "r1"), (2, true), "2 つ目の印");
        events.push(stage("r1", Stage::Gated, "verdict:FAIL,rules:embedded"));
        assert_eq!(fix_rounds(&events, "r1"), (2, false), "印の後の Gated は数えず、最後の RunStage は印でない");
        events.push(event("r1", EventKind::SeatSpawned, None, Some("r1"), None));
        assert_eq!(fix_rounds(&events, "r1"), (2, false), "RunStage でない event は最後の RunStage に入らない");
        events.push(stage("r1", Stage::Implemented, "rebase:a..b"));
        assert_eq!(fix_rounds(&events, "r1"), (2, false), "detail が gate-fix: で始まらない Implemented は印でない");
    }
}
