//! 受付の断りの記帳（設計 docs/design/dispatcher.md §32・契約表の行 ag・FR68 / AC60・ADR-0088 (6)）。
//!
//! 起こす側の周（[`super::fire`]）だけが、起こし終えた後に受付が断った契約ごとに断りの名と bead を
//! `IntakeRefused` 1 件に残す。書くのは断りに入った周と名が変わった周だけ（その bead の最後の行が同じ名の
//! `IntakeRefused` なら書かない）で、同じ断りが続く間は log を伸ばさない。観測の口（[`super::turn`]）は撃たない。

use super::{Candidate, Input, WaitReason, MARK, SPAWN};
use crate::fleet::store;
use crate::fleet::{Case, Event, EventKind, SCHEMA};

/// 記帳する (bead, 断りの名) を候補の順に選ぶ（**pure**・形 1）。
///
/// 入るのは理由が [`WaitReason::Admission`] で名が列自身の 2 語（[`MARK`]・[`SPAWN`]）でないもの＝受付の断りの名と
/// 受付札の `slot`。`host-busy` と他の待ちの理由は受付でないので入らない。
fn chosen(candidates: &[Candidate]) -> Vec<(&str, &'static str)> {
    candidates
        .iter()
        .filter_map(|candidate| match candidate.reason {
            Some(WaitReason::Admission { reason, .. }) if reason != MARK && reason != SPAWN => {
                Some((candidate.bead.as_str(), reason))
            }
            _ => None,
        })
        .collect()
}

/// (bead, 名) を書く周か（**pure**・形 2）: 周が読んだ event の列のうち**その bead を持つ最後の行**が同じ名の
/// `IntakeRefused` でなければ書く。event log を読めなかった周（`None`）は書かない（読めないを「行が無い」に
/// 読み替えない・C10）。
fn due(events: Option<&[Event]>, bead: &str, name: &str) -> bool {
    let Some(events) = events else {
        return false;
    };
    !events.iter().rev().find(|event| event.bead == bead).is_some_and(|last| {
        last.kind == EventKind::IntakeRefused && matches!(&last.case, Some(Case::Refused { refuse }) if refuse == name)
    })
}

/// 断りを記帳する（形 3・追記は fleet の 1 本 [`store::append`]・lock の待ち方は起こした印と同じ manifest から読む）。
///
/// policy を読めない周と書けない周は何もしない（列の結果・rc・行を 1 つも変えない・次の周が同じ判定で書き直す）。
pub(super) fn record(input: &Input<'_>, candidates: &[Candidate], events: Option<&[Event]>) {
    let Ok(policy) = store::LockPolicy::from_rules(input.manifest) else {
        return;
    };
    for (bead, name) in chosen(candidates) {
        if due(events, bead, name) {
            // 断りの 1 行の理由は detail に載せる（判定には使わない・局面の出力の契約の部品の欄 why に写る）。
            let event = Event { detail: reason_line(candidates, bead), ..refusal(bead, name) };
            let _ = store::append(input.state_dir, &event, policy);
        }
    }
}

/// 候補 `bead` の受付の断りの 1 行の理由（[`WaitReason::Admission`] の `why`・無ければ `None`）。
fn reason_line(candidates: &[Candidate], bead: &str) -> Option<String> {
    candidates.iter().find(|candidate| candidate.bead == bead).and_then(|candidate| match candidate.reason {
        Some(WaitReason::Admission { ref why, .. }) => why.clone(),
        _ => None,
    })
}

/// 書く 1 行（kind `IntakeRefused`・refuse = 断りの名・actor は kind の既定・detail なし＝理由は [`record`] が足す）。
fn refusal(bead: &str, name: &str) -> Event {
    let kind = EventKind::IntakeRefused;
    Event {
        schema: SCHEMA,
        ts: crate::fleet::cli::now_utc(),
        kind,
        run: String::new(),
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
        case: Some(Case::Refused { refuse: name.to_owned() }),
    }
}

#[cfg(test)]
mod tests {
    use super::{chosen, due, reason_line, refusal, Candidate, WaitReason};
    use crate::fleet::Event;

    /// 断られた bead（歯の fixture の既定）。
    const BEAD: &str = "s2-ref.1";

    /// 理由だけを呼び手が選ぶ候補 1 件。
    fn candidate(bead: &str, reason: Option<WaitReason>) -> Candidate {
        Candidate { bead: bead.to_owned(), priority: None, mark: None, reason }
    }

    /// JSON の 1 行から組む event（kind の字は歯が自分で書く）。
    fn line(json: &str) -> Event {
        Event::from_line(json).expect("fixture の行は読める")
    }

    /// `bead` に断りの名 `name` を記帳した行。
    fn refused(bead: &str, name: &str) -> Event {
        line(&format!(
            r#"{{"schema":1,"ts":"2026-09-29T00:00:00Z","kind":"IntakeRefused","bead":"{bead}","refuse":"{name}","host":"h","actor":"machine"}}"#
        ))
    }

    /// `bead` の `release` の印の行。
    fn released(bead: &str) -> Event {
        line(&format!(
            r#"{{"schema":1,"ts":"2026-09-29T00:00:01Z","kind":"DispatchMark","bead":"{bead}","mark":"release","host":"h","actor":"human"}}"#
        ))
    }

    /// (a) 7 候補（Admission の cap-headroom・slot・mark・spawn、Dependency・Hold・理由なし）から選ばれるのは cap-headroom と slot の
    /// 2 件だけで、候補の順。
    #[test]
    fn refusal_record_chooses_only_intake_refusals_in_candidate_order() {
        let admission = |reason: &'static str| Some(WaitReason::Admission { reason, why: None });
        let candidates = [
            candidate("s2-ref.1", admission("cap-headroom")),
            candidate("s2-ref.2", admission(super::MARK)),
            candidate("s2-ref.3", Some(WaitReason::Dependency { on: vec!["s2-ref.9".to_owned()] })),
            candidate("s2-ref.4", admission(super::super::SLOT)),
            candidate("s2-ref.5", admission(super::SPAWN)),
            candidate("s2-ref.6", Some(WaitReason::Hold { since: "2026-09-29T00:00:00Z".to_owned(), why: None })),
            candidate("s2-ref.7", None),
        ];
        assert_eq!(
            chosen(&candidates),
            vec![("s2-ref.1", "cap-headroom"), ("s2-ref.4", "slot")],
            "母集団 {} 候補のうち受付の断りの 2 件だけ",
            candidates.len()
        );
    }

    /// (b) その bead の最後の行が同じ名の IntakeRefused なら書かない。
    #[test]
    fn refusal_record_skips_when_the_last_line_is_the_same_refusal() {
        let events = [refused(BEAD, "cap-headroom")];
        assert!(!due(Some(&events), BEAD, "cap-headroom"));
    }

    /// (c) 最後の行が別の名の IntakeRefused なら書く。
    #[test]
    fn refusal_record_writes_when_the_name_changed() {
        let events = [refused(BEAD, "slot")];
        assert!(due(Some(&events), BEAD, "cap-headroom"));
    }

    /// (d) 同じ名の IntakeRefused の後にその bead の release の行が在れば書く。
    #[test]
    fn refusal_record_writes_after_a_release_of_the_same_bead() {
        let events = [refused(BEAD, "cap-headroom"), released(BEAD)];
        assert!(due(Some(&events), BEAD, "cap-headroom"));
    }

    /// (e) 同じ名の IntakeRefused の後に別の bead の行だけなら書かない。
    #[test]
    fn refusal_record_ignores_lines_of_other_beads() {
        let events = [refused(BEAD, "cap-headroom"), released("s2-ref.2"), refused("s2-ref.3", "slot")];
        assert!(!due(Some(&events), BEAD, "cap-headroom"));
    }

    /// (f) その bead の行が無ければ書く（書く行は kind の既定の actor・detail なしで読み返せる）。
    #[test]
    fn refusal_record_writes_when_the_bead_has_no_line() {
        let events = [released("s2-ref.2")];
        assert!(due(Some(&events), BEAD, "cap-headroom"));
        let written = refusal(BEAD, "cap-headroom");
        assert_eq!(line(&written.to_line()), written, "書く行は from_line で同じ値に読み返せる");
        assert_eq!(written.detail, None);
    }

    /// 断りの記録の detail は候補の受付の断りの 1 行の理由で、理由の無い断り・受付でない理由・候補に無い bead は持たない（行 v-refuse-why）。
    #[test]
    fn vhdref_record_detail_is_the_one_line_reason_of_the_candidate() {
        let candidates = [
            candidate("s2-ref.1", Some(WaitReason::Admission { reason: "cap-headroom", why: Some("余地が足りない 1 行".to_owned()) })),
            candidate("s2-ref.2", Some(WaitReason::Admission { reason: "contract-table", why: None })),
            candidate("s2-ref.3", Some(WaitReason::Hold { since: "2026-09-29T00:00:00Z".to_owned(), why: Some("止め".to_owned()) })),
        ];
        assert_eq!(reason_line(&candidates, "s2-ref.1").as_deref(), Some("余地が足りない 1 行"), "受付の断りの 1 行");
        assert_eq!(reason_line(&candidates, "s2-ref.2"), None, "理由の無い断り");
        assert_eq!(reason_line(&candidates, "s2-ref.3"), None, "受付でない理由の字は写さない");
        assert_eq!(reason_line(&candidates, "s2-ref.9"), None, "候補に無い bead");
    }

    /// (g) event log を読めない周は書かない。
    #[test]
    fn refusal_record_writes_nothing_when_the_log_is_unreadable() {
        assert!(!due(None, BEAD, "cap-headroom"));
    }
}
