//! 便の落ちの型の記帳（判断の記録 ADR-77 の決定 (7)・契約表の行 77-4 の前半・条 P-10.2）。
//!
//! 起こす側の周（[`super::fire`]）だけが、型の決まった便ごとに `RunFell` 1 件を event log に残す（型は [`crate::pipe::fall::fall_of`]
//! の 1 本が決める）。同じ便の `RunFell` が既に在れば書かない。観測の口（[`super::turn`]）は撃たない。
//! gate の FAIL の便だけは、落ちた段を便の dir の verify の記録から読む。

use super::Input;
use crate::fleet::json_lite::{self, Value};
use crate::fleet::store::{self, LockPolicy};
use crate::fleet::{Case, Event, EventKind, Shape, SCHEMA};
use crate::pipe::fall::{fall_of, Fall, LENS_STEP};
use crate::pipe::verify_log_path;
use std::collections::BTreeMap;
use std::path::Path;

/// その便の `RunFell` を書く周か（**pure**）: 周が読んだ event の列にその便の `RunFell` が在れば書かない。event log を読めなかった周
/// （`None`）は書かない（読めないを「行が無い」に読み替えない・C10）。
fn due(events: Option<&[Event]>, run: &str) -> bool {
    events.is_some_and(|events| !events.iter().any(|event| event.kind == EventKind::RunFell && event.run == run))
}

/// 便ごとの段の記帳の列（log の順・便に紐づく kind だけ・便 id の順）。
fn runs_of(events: &[Event]) -> BTreeMap<&str, Vec<Event>> {
    let mut runs: BTreeMap<&str, Vec<Event>> = BTreeMap::new();
    for event in events.iter().filter(|event| event.kind.shape() == Shape::Run) {
        runs.entry(event.run.as_str()).or_default().push(event.clone());
    }
    runs
}

/// gate の FAIL の周に落ちた段の名（便の dir の `verify.jsonl` の最後の周の最初の赤い段・赤が 1 つも無ければ lens の落ち）。
///
/// 最後の周は最後の `kind=write-set` の record から終わりまで（gate の周は write-set の照合から始まる）。file を読めない・record を
/// 読めない・周の頭が無い周は `None`（lens の落ちに倒さない）。撃たなかった段の record は `rc` を持たない＝赤でない。
fn gate_step_of(state_dir: &Path, run: &str) -> Option<String> {
    let text = std::fs::read_to_string(verify_log_path(state_dir, run)).ok()?;
    let mut records: Vec<Vec<(String, Value)>> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        records.push(json_lite::parse_object(line).ok()?);
    }
    let kind_of = |record: &[(String, Value)]| record.iter().find(|(key, _)| key == "kind").and_then(|(_, value)| value.as_str().map(str::to_owned));
    let head = records.iter().rposition(|record| kind_of(record).as_deref() == Some("write-set"))?;
    let red = records.get(head..)?.iter().find(|record| record.iter().any(|(key, value)| key == "rc" && value.as_num().is_some_and(|rc| rc != 0)));
    Some(red.and_then(|record| kind_of(record)).unwrap_or_else(|| LENS_STEP.to_owned()))
}

/// 便の落ちの型（gate の FAIL の便だけ verify の記録を読む・ほかの便は記録を読まない）。
fn fall_in(state_dir: &Path, run: &str, events: &[Event]) -> Option<Fall> {
    match fall_of(events, Some(LENS_STEP)) {
        Some(Fall::GateLens) => fall_of(events, gate_step_of(state_dir, run).as_deref()),
        other => other,
    }
}

/// 落ちの型を記帳する（追記は fleet の 1 本 [`store::append`]・lock の待ち方は起こした印と同じ manifest から読む）。
///
/// policy を読めない周と書けない周は何もしない（列の結果・rc・行を 1 つも変えない・次の周が同じ判定で書き直す）。
pub(super) fn record(input: &Input<'_>, events: Option<&[Event]>) {
    let Ok(policy) = store::LockPolicy::from_rules(input.manifest) else {
        return;
    };
    write(input.state_dir, policy, events);
}

/// 型の決まった便ごとに、まだ記帳の無い便へ 1 件ずつ書く（便 id の順）。
fn write(state_dir: &Path, policy: LockPolicy, events: Option<&[Event]>) {
    let Some(all) = events else {
        return;
    };
    for (run, own) in runs_of(all) {
        if !due(events, run) {
            continue;
        }
        let Some(fall) = fall_in(state_dir, run, &own) else {
            continue;
        };
        let bead = own.last().map(|last| last.bead.as_str()).unwrap_or_default();
        let _ = store::append(state_dir, &fell(run, bead, fall), policy);
    }
}

/// 書く 1 行（kind `RunFell`・run = 便・bead = 契約・fall = 型の語・actor は kind の既定・detail なし）。
fn fell(run: &str, bead: &str, fall: Fall) -> Event {
    let kind = EventKind::RunFell;
    Event {
        schema: SCHEMA,
        ts: crate::fleet::cli::now_utc(),
        kind,
        run: run.to_owned(),
        bead: bead.to_owned(),
        host: crate::fleet::cli::host(),
        actor: kind.default_actor().to_owned(),
        stage: None,
        seat: None,
        pid: None,
        detail: None,
        allowance: None,
        registration: None,
        mark: None,
        account: None,
        cost: None,
        rule: None,
        case: Some(Case::Fell { fall }),
    }
}

#[cfg(test)]
mod tests {
    use super::{due, fell, gate_step_of, write};
    use crate::fleet::store::{self, LockPolicy};
    use crate::fleet::{Case, Event, EventKind};
    use crate::pipe::fall::Fall;
    use crate::pipe::fixture::scratch;
    use crate::pipe::run_dir;

    /// 歯の lock の待ち方。
    const POLICY: LockPolicy = LockPolicy { retry_ms: 2000, stale_ms: 60_000 };

    /// 便 `run` の段の記帳 1 行。
    fn staged(run: &str, stage: &str, detail: &str) -> Event {
        Event::from_line(&format!(
            r#"{{"schema":1,"ts":"2026-10-07T00:00:00Z","kind":"RunStage","run":"{run}","bead":"b-{run}","host":"h","actor":"machine","stage":"{stage}","detail":"{detail}"}}"#
        ))
        .expect("fixture の行は読める")
    }

    /// 積んだ event log の `RunFell` の (run, bead, 型)（log の順）。
    fn fells(state: &std::path::Path) -> Vec<(String, String, Fall)> {
        let events = store::read_all(state).expect("log を読める");
        events
            .into_iter()
            .filter_map(|event| match event.case {
                Some(Case::Fell { fall }) if event.kind == EventKind::RunFell => Some((event.run, event.bead, fall)),
                _ => None,
            })
            .collect()
    }

    /// 型の決まった便ごとに `RunFell` を 1 件書き、同じ便の `RunFell` が既に在る周と event log を読めない周は書かない。
    #[test]
    fn fallen_ledger_writes_one_row_per_fallen_run_and_not_twice() {
        let state = scratch("fell");
        let events = vec![
            staged("r1", "Failed", "runner-rc:1"),
            staged("r2", "Reviewed", "verdict:FAIL kind:vacuous-assert"),
            staged("r3", "Landed", "closed"),
            staged("r4", "Questioned", "about:write-set"),
            staged("r5", "Implemented", ""),
        ];
        write(&state, POLICY, None);
        assert!(fells(&state).is_empty(), "event log を読めない周は書かない");

        write(&state, POLICY, Some(&events));
        let want = [
            ("r1".to_owned(), "b-r1".to_owned(), Fall::Terminal),
            ("r2".to_owned(), "b-r2".to_owned(), Fall::ReviewVacuousAssert),
            ("r4".to_owned(), "b-r4".to_owned(), Fall::QuestionWriteSet),
        ];
        assert_eq!(fells(&state), want, "型の決まった 3 本だけに 1 件ずつ（着地と終端でない便は書かない）");

        let written = store::read_all(&state).expect("log を読める");
        write(&state, POLICY, Some(&written));
        assert_eq!(fells(&state), want, "同じ便の RunFell が在る周は書かない");
        assert!(!due(Some(&written), "r1"), "記帳の在る便");
        assert!(due(Some(&written), "r3"), "記帳の無い便（型が決まるかは別）");
        assert!(!due(None, "r3"), "読めない周は書かない");
    }

    /// gate の FAIL の便は、verify の記録の最後の周の最初の赤い段の語を型にし、赤が無ければ lens・記録を読めなければ書かない。
    #[test]
    fn fallen_ledger_reads_the_failed_gate_step_from_the_verify_record() {
        let state = scratch("fell-gate");
        let gated = |run: &str| staged(run, "Gated", "verdict:FAIL,rules:embedded");
        let record = |run: &str, body: &str| {
            let dir = run_dir(&state, run);
            std::fs::create_dir_all(&dir).expect("dir を作れる");
            std::fs::write(dir.join("verify.jsonl"), body).expect("record を書ける");
        };
        record("g1", "{\"schema\":1,\"n\":1,\"rc\":0,\"kind\":\"write-set\"}\n{\"schema\":1,\"n\":2,\"rc\":0,\"kind\":\"common\"}\n{\"schema\":1,\"n\":3,\"rc\":1,\"kind\":\"contract\"}\n");
        record("g2", "{\"schema\":1,\"n\":1,\"rc\":0,\"kind\":\"write-set\"}\n{\"schema\":1,\"n\":2,\"rc\":0,\"kind\":\"contract\"}\n");
        record("g3", "{\"schema\":1,\"n\":1,\"rc\":1,\"kind\":\"common\"}\n{\"schema\":1,\"n\":2,\"rc\":0,\"kind\":\"write-set\"}\n{\"schema\":1,\"n\":3,\"rc\":0,\"kind\":\"common\"}\n");
        record("g4", "壊れた行\n");
        let events = vec![gated("g1"), gated("g2"), gated("g3"), gated("g4"), gated("g5")];
        write(&state, POLICY, Some(&events));
        assert_eq!(
            fells(&state),
            [
                ("g1".to_owned(), "b-g1".to_owned(), Fall::GateContract),
                ("g2".to_owned(), "b-g2".to_owned(), Fall::GateLens),
                ("g3".to_owned(), "b-g3".to_owned(), Fall::GateLens),
            ],
            "赤い段・赤の無い周は lens・前の周の赤は数えない・壊れた記録と記録の無い便は書かない"
        );
        assert_eq!(gate_step_of(&state, "g1").as_deref(), Some("contract"));
        assert_eq!(gate_step_of(&state, "g4"), None);
        assert_eq!(gate_step_of(&state, "g5"), None);
        let row = fell("g1", "b-g1", Fall::GateLens);
        assert_eq!((row.actor.as_str(), row.detail.as_ref()), ("machine", None), "既定の actor・detail なし");
        assert_eq!(Event::from_line(&row.to_line()).expect("書いた行は読める"), row, "行に直して読み返すと同じ値");
    }
}
