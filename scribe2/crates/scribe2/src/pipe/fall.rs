//! 便の落ちの型（判断の記録 ADR-77 の決定 (7)・条 P-10.2・契約表の行 77-4 の前半）。
//!
//! 便の落ちは段の記帳の自由文の `detail` に散っている（審査の `Reviewed`・gate の `Gated` と verify の記録・`Failed`・質問）。
//! 本 file は落ちを**閉じた型の 1 語**に決める 1 本（[`fall_of`]・pure）と、その語の往復（[`Fall::as_str`] / [`Fall::parse`]）を持つ。
//! 書き手は器の列の周（[`super::dispatch`] の `fell`）だけで、記録の kind は [`crate::fleet::EventKind::RunFell`]。
//! 数えの側（条 P-10.2・行 77-4 の後半）は event の列から型の語ごとに累計する [`counted`] と、止めて書き直しの便を選ぶ [`rewritten`]。

use super::gate::{Check, Verdict, CHECKS};
use super::review::{read_detail, FindingKind};
use crate::fleet::{Case, Event, EventKind, Stage};

/// gate の FAIL の周に、verify の記録の赤い段が無く lens が落とした段の名（呼び手が [`fall_of`] へ渡す字）。
pub const LENS_STEP: &str = "lens";

/// 便の落ちの型（**閉じた 15 語**）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fall {
    /// 審査の FAIL・verify の歯が write-set の外の file に在る。
    ReviewTeethOutsideWriteSet,
    /// 審査の FAIL・goal と done が矛盾する。
    ReviewGoalDoneContradiction,
    /// 審査の FAIL・歯が空虚で done を測れない。
    ReviewVacuousAssert,
    /// 審査の FAIL・契約の字面が設計の節・要件の現物と合わない。
    ReviewLiteralMismatch,
    /// 審査の FAIL・設計の節や要件の材料が欠けている。
    ReviewSectionMaterialMissing,
    /// 審査の FAIL・上のどれでもない。
    ReviewOther,
    /// 審査の FAIL・理由の型を読めなかった（受け皿）。
    ReviewUnparsed,
    /// gate の FAIL・write-set の照合の段が赤い。
    GateWriteSet,
    /// gate の FAIL・共通 verify の段が赤い。
    GateCommon,
    /// gate の FAIL・契約の verify の段が赤い。
    GateContract,
    /// gate の FAIL・verify は緑で lens が落とした。
    GateLens,
    /// `Failed` で終端した便のうち審査と gate の型を持たない落ち（runner の rc・箱の中の死・着地の側の落ち）。
    Terminal,
    /// 質問で止まった便・問いが write-set に関する。
    QuestionWriteSet,
    /// 質問で止まった便・ほかの問い。
    QuestionOther,
    /// 審査の前に止めた便の後に、契約か節を替えた起こし直し（語だけを置く・書き手は受付）。
    Rewritten,
}

/// [`Fall`] の全 variant（宣言順・`enum-slices` が集合完全性を測る）。
pub const FALLS: &[Fall] = &[
    Fall::ReviewTeethOutsideWriteSet,
    Fall::ReviewGoalDoneContradiction,
    Fall::ReviewVacuousAssert,
    Fall::ReviewLiteralMismatch,
    Fall::ReviewSectionMaterialMissing,
    Fall::ReviewOther,
    Fall::ReviewUnparsed,
    Fall::GateWriteSet,
    Fall::GateCommon,
    Fall::GateContract,
    Fall::GateLens,
    Fall::Terminal,
    Fall::QuestionWriteSet,
    Fall::QuestionOther,
    Fall::Rewritten,
];

impl Fall {
    /// event の `fall` に書く字面。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReviewTeethOutsideWriteSet => "review-teeth-outside-write-set",
            Self::ReviewGoalDoneContradiction => "review-goal-done-contradiction",
            Self::ReviewVacuousAssert => "review-vacuous-assert",
            Self::ReviewLiteralMismatch => "review-literal-mismatch",
            Self::ReviewSectionMaterialMissing => "review-section-material-missing",
            Self::ReviewOther => "review-other",
            Self::ReviewUnparsed => "review-unparsed",
            Self::GateWriteSet => "gate-write-set",
            Self::GateCommon => "gate-common",
            Self::GateContract => "gate-contract",
            Self::GateLens => "gate-lens",
            Self::Terminal => "terminal",
            Self::QuestionWriteSet => "question-write-set",
            Self::QuestionOther => "question-other",
            Self::Rewritten => "rewritten",
        }
    }

    /// 字面から引く。15 語の外は `None`。
    pub fn parse(text: &str) -> Option<Self> {
        FALLS.iter().copied().find(|found| found.as_str() == text)
    }
}

/// 審査の理由の型に対応する落ちの型（網羅 `match`・理由の型を足した便は compile が止める）。
fn review_fall(kind: FindingKind) -> Fall {
    match kind {
        FindingKind::TeethOutsideWriteSet => Fall::ReviewTeethOutsideWriteSet,
        FindingKind::GoalDoneContradiction => Fall::ReviewGoalDoneContradiction,
        FindingKind::VacuousAssert => Fall::ReviewVacuousAssert,
        FindingKind::LiteralMismatch => Fall::ReviewLiteralMismatch,
        FindingKind::SectionMaterialMissing => Fall::ReviewSectionMaterialMissing,
        FindingKind::Other => Fall::ReviewOther,
        FindingKind::Unparsed => Fall::ReviewUnparsed,
    }
}

/// gate の FAIL の周に落ちた段の名に対応する落ちの型（赤い段の名か [`LENS_STEP`]・ほかの名は `None`）。
fn gate_fall(step: &str) -> Option<Fall> {
    if step == LENS_STEP {
        return Some(Fall::GateLens);
    }
    let check = CHECKS.iter().copied().find(|found| found.as_str() == step)?;
    match check {
        Check::WriteSet => Some(Fall::GateWriteSet),
        Check::Common => Some(Fall::GateCommon),
        Check::Contract => Some(Fall::GateContract),
        Check::Detection => None,
    }
}

/// `Gated` の detail（`verdict:<V>,…`）の判定の語。
fn gated_verdict(detail: &str) -> Option<Verdict> {
    detail.split(|glyph: char| glyph == ',' || glyph.is_whitespace()).find_map(|word| word.strip_prefix("verdict:")).and_then(Verdict::parse)
}

/// 便 1 本の event の列（log の順）から落ちの型を決める（**pure**）。
///
/// `gate_step` は gate の FAIL の周に落ちた段の名（呼び手が便の dir の verify の記録から読む・読めない周は `None`＝
/// [`Fall::GateLens`] に倒さず型を返さない）。決め方は最後の段で引く: `Landed` と終端でない段は `None`、`Reviewed` の verdict が
/// PASS でなければ審査の理由の型の語（[`read_detail`] の対の読み）、`Gated` の verdict が FAIL なら落ちた段の語、`Failed` は
/// [`Fall::Terminal`]、`Questioned` は問いの about が write-set かで 2 語のどちらか。
pub fn fall_of(events: &[Event], gate_step: Option<&str>) -> Option<Fall> {
    let (stage, detail) = events.iter().rev().find_map(|event| event.stage.map(|stage| (stage, event.detail.as_deref().unwrap_or_default())))?;
    match stage {
        Stage::Reviewed => {
            let (verdict, kind) = read_detail(detail);
            (verdict != Some(Verdict::Pass)).then(|| review_fall(kind))
        }
        Stage::Gated => {
            if gated_verdict(detail) == Some(Verdict::Fail) {
                gate_step.and_then(gate_fall)
            } else {
                None
            }
        }
        Stage::Failed => Some(Fall::Terminal),
        Stage::Questioned => Some(if detail.strip_prefix("about:") == Some("write-set") { Fall::QuestionWriteSet } else { Fall::QuestionOther }),
        Stage::Landed
        | Stage::Intake
        | Stage::Blocked
        | Stage::Spawned
        | Stage::RateLimited
        | Stage::Implemented
        | Stage::Stopped => None,
    }
}

/// 数えない受付の断りの名（**閉じた 12 語**・器の容量・索引の待ち・条 P-10.3 の断り・器の壊れ）。[`counted`] はこの名の
/// `IntakeRefused` を引数に依らず数えない。
pub const UNCOUNTED_REFUSALS: &[&str] = &[
    "max-live",
    "slot",
    "run-cap",
    "cap-headroom",
    "index-building",
    "code-facts-unmeasured",
    "same-kind-repeated",
    "not-a-repo",
    "rules",
    "store",
    "declaration",
    "generated",
];

/// bead ごとの断りに数えない型の語（**閉じた 4 語**・受け皿の型と便の問いの止まり・条 P-10.3）。受付が [`counted`] の 3 つ目の引数に渡す。
/// bead をまたぐ数えは空の列を渡して受け皿の型も数える。
pub const UNCOUNTED_FOR_BEAD: &[&str] = &["review-other", "review-unparsed", "question-write-set", "question-other"];

/// 型の語 1 つの数え（[`counted`] の返り値の 1 要素）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tally {
    /// 型の語（[`Fall::as_str`] の語か、受付の断りの `intake-<断りの名>`）。
    pub kind: String,
    /// その型の落ちの証（新しい順・`RunFell` は便の run id、`IntakeRefused` は `<bead>@<ts>`）。
    pub proofs: Vec<String>,
}

/// 型の語の並び（[`FALLS`] の宣言順・受付の断りの型はその後に名の辞書順）の鍵。
fn kind_order(kind: &str) -> (usize, &str) {
    (FALLS.iter().position(|fall| fall.as_str() == kind).unwrap_or(FALLS.len()), kind)
}

/// event の列から `beads` の bead の落ちを型の語ごとに累計する（**pure**・判断の記録 ADR-77 の決定 (7)・条 P-10.2 の数えの側）。
///
/// 読むのは列の最初の `RunFell` の行から後の 2 種類だけ（`RunFell` の無い列は空）: `RunFell` は欄 `fall` の語が型の語で証は便の run id、
/// 数えない名（[`UNCOUNTED_REFUSALS`]）でない `IntakeRefused` は `intake-` と断りの名を繋いだ字が型の語で証は `<bead>@<ts>`。`beads` に無い
/// bead の行と `skip` に在る型の語の行は読まない。数えは累計で、PASS の便も契約か節の字の替えも別の型の落ちを挟むことも戻さない。
/// 返りは型の語の宣言順（[`FALLS`]・その後に断りの型の名の辞書順）で、証は新しい順。
pub fn counted(events: &[Event], beads: &[&str], skip: &[&str]) -> Vec<Tally> {
    let Some(start) = events.iter().position(|event| event.kind == EventKind::RunFell) else {
        return Vec::new();
    };
    let mut tallies: Vec<Tally> = Vec::new();
    for event in events.iter().skip(start).filter(|event| beads.contains(&event.bead.as_str())) {
        let (kind, proof) = match (event.kind, event.case.as_ref()) {
            (EventKind::RunFell, Some(Case::Fell { fall })) => (fall.as_str().to_owned(), event.run.clone()),
            (EventKind::IntakeRefused, Some(Case::Refused { refuse })) if !UNCOUNTED_REFUSALS.contains(&refuse.as_str()) => {
                (format!("intake-{refuse}"), format!("{}@{}", event.bead, event.ts))
            }
            _ => continue,
        };
        if skip.contains(&kind.as_str()) {
            continue;
        }
        match tallies.iter_mut().find(|found| found.kind == kind) {
            Some(found) => found.proofs.push(proof),
            None => tallies.push(Tally { kind, proofs: vec![proof] }),
        }
    }
    for found in &mut tallies {
        found.proofs.reverse();
    }
    tallies.sort_by(|left, right| kind_order(&left.kind).cmp(&kind_order(&right.kind)));
    tallies
}

/// 数える bead の便 1 本の材料（[`rewritten`] が読む）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    /// run id。
    pub run: String,
    /// 審査の前に止めた便か（段が `Stopped` で `review.json` が無い）。
    pub unreviewed: bool,
    /// 契約 file の写しの字。
    pub contract: String,
    /// `design.txt` の写しの字（無ければ `None`）。
    pub design: Option<String>,
}

/// 止めて書き直しの便を選ぶ（**pure**）。`runs` は古い順で、審査の前に止めた便ごとに、次の便（最後の便なら `today`＝今回の契約 file と節の本文の対）と
/// 契約 file の写しが違うか、両方に `design.txt` の写しが在りその字が違う時、その便の run id を新しい順に返す（写しの無い側を違うと読まない）。
pub fn rewritten(runs: &[Attempt], today: (&str, &str)) -> Vec<String> {
    runs.iter()
        .enumerate()
        .filter(|(_, run)| run.unreviewed)
        .filter(|(index, run)| {
            let (contract, design) = match runs.get(index.saturating_add(1)) {
                Some(next) => (next.contract.as_str(), next.design.as_deref()),
                None => (today.0, Some(today.1)),
            };
            run.contract != contract || run.design.as_deref().zip(design).is_some_and(|(before, after)| before != after)
        })
        .map(|(_, run)| run.run.clone())
        .rev()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{counted, fall_of, rewritten, Attempt, Fall, Tally, FALLS, LENS_STEP, UNCOUNTED_FOR_BEAD, UNCOUNTED_REFUSALS};
    use crate::fleet::Event;
    use crate::order::is_declaration_order;

    /// 便の段の記帳 1 行（stage と detail だけを歯が選ぶ）。
    fn staged(stage: &str, detail: Option<&str>) -> Event {
        let detail = detail.map(|text| format!(r#","detail":"{text}""#)).unwrap_or_default();
        Event::from_line(&format!(
            r#"{{"schema":1,"ts":"2026-10-07T00:00:00Z","kind":"RunStage","run":"r-1","bead":"b-1","host":"h","actor":"machine","stage":"{stage}"{detail}}}"#
        ))
        .expect("fixture の行は読める")
    }

    /// 便が `Intake` から `stage` まで来た列（最後の記帳だけ detail を持つ）。
    fn run_at(stage: &str, detail: Option<&str>) -> Vec<Event> {
        vec![staged("Intake", None), staged(stage, detail)]
    }

    /// 落ちの型は 15 語が宣言順に並び、字面と往復し、列の外の語は引けない。
    #[test]
    fn fallen_words_are_closed_and_round_trip_in_declaration_order() {
        let words: Vec<&str> = FALLS.iter().map(|fall| fall.as_str()).collect();
        assert_eq!(
            words,
            [
                "review-teeth-outside-write-set",
                "review-goal-done-contradiction",
                "review-vacuous-assert",
                "review-literal-mismatch",
                "review-section-material-missing",
                "review-other",
                "review-unparsed",
                "gate-write-set",
                "gate-common",
                "gate-contract",
                "gate-lens",
                "terminal",
                "question-write-set",
                "question-other",
                "rewritten",
            ],
            "15 語・宣言順"
        );
        assert!(is_declaration_order(FALLS, |fall| fall as usize), "FALLS は宣言順: {FALLS:?}");
        for fall in FALLS {
            assert_eq!(Fall::parse(fall.as_str()), Some(*fall), "{} は往復する", fall.as_str());
        }
        for outside in ["", "Terminal", "gate-detection", "review-", "question"] {
            assert_eq!(Fall::parse(outside), None, "列の外の語 {outside:?} は引けない");
        }
    }

    /// 型の決まる枝（審査 7 語・gate の 4 語・terminal・問いの 2 語）と、型を返さない枝（着地・終端でない便・Stopped・PASS・
    /// 判定が FAIL でない gate・落ちた段を読めない gate の FAIL）。
    #[test]
    fn fallen_kind_of_names_review_gate_step_terminal_and_asked_branch() {
        names_review_kinds();
        names_gate_steps();
        names_terminal_and_asked_and_untyped();
    }

    /// 審査の枝: PASS でない判定は理由の型の語（kind の無い・detail の無い審査は unparsed）、PASS は型を持たない。
    fn names_review_kinds() {
        let review = [
            ("teeth-outside-write-set", Fall::ReviewTeethOutsideWriteSet),
            ("goal-done-contradiction", Fall::ReviewGoalDoneContradiction),
            ("vacuous-assert", Fall::ReviewVacuousAssert),
            ("literal-mismatch", Fall::ReviewLiteralMismatch),
            ("section-material-missing", Fall::ReviewSectionMaterialMissing),
            ("other", Fall::ReviewOther),
            ("unparsed", Fall::ReviewUnparsed),
        ];
        for (kind, want) in review {
            let events = run_at("Reviewed", Some(&format!("verdict:FAIL kind:{kind}")));
            assert_eq!(fall_of(&events, None), Some(want), "審査 FAIL の kind:{kind}");
        }
        assert_eq!(fall_of(&run_at("Reviewed", Some("verdict:INCONCLUSIVE kind:other")), None), Some(Fall::ReviewOther), "PASS でない判定");
        assert_eq!(fall_of(&run_at("Reviewed", Some("verdict:FAIL")), None), Some(Fall::ReviewUnparsed), "kind の無い detail は unparsed");
        assert_eq!(fall_of(&run_at("Reviewed", None), None), Some(Fall::ReviewUnparsed), "detail の無い審査は PASS に化けない");
        assert_eq!(fall_of(&run_at("Reviewed", Some("verdict:PASS")), None), None, "審査 PASS は落ちでない");
    }

    /// gate の枝: FAIL は落ちた段の語（読めない周と検出線は型を返さない）、INCONCLUSIVE と PASS は型を持たない。
    fn names_gate_steps() {
        let gate = [("write-set", Fall::GateWriteSet), ("common", Fall::GateCommon), ("contract", Fall::GateContract), (LENS_STEP, Fall::GateLens)];
        for (step, want) in gate {
            let events = run_at("Gated", Some("verdict:FAIL,rules:embedded"));
            assert_eq!(fall_of(&events, Some(step)), Some(want), "gate FAIL の段 {step}");
        }
        let failed_gate = run_at("Gated", Some("verdict:FAIL,account:a,rules:embedded"));
        assert_eq!(fall_of(&failed_gate, None), None, "落ちた段を読めない周は gate-lens に倒さない");
        assert_eq!(fall_of(&failed_gate, Some("detection")), None, "検出線は gate の落ちの段でない");
        assert_eq!(fall_of(&run_at("Gated", Some("verdict:INCONCLUSIVE,rules:embedded")), Some("common")), None, "INCONCLUSIVE は落ちでない");
        assert_eq!(fall_of(&run_at("Gated", Some("verdict:PASS,rules:embedded")), Some("common")), None, "PASS は落ちでない");
    }

    /// 終端・問い・型を持たない枝（着地・終端でない便・Stopped・段の無い列・最後の段が着地の便）。
    fn names_terminal_and_asked_and_untyped() {
        assert_eq!(fall_of(&run_at("Failed", Some("runner-rc:1")), None), Some(Fall::Terminal), "Failed の終端");
        assert_eq!(fall_of(&run_at("Questioned", Some("about:write-set")), None), Some(Fall::QuestionWriteSet), "write-set の問い");
        assert_eq!(fall_of(&run_at("Questioned", Some("about:verify")), None), Some(Fall::QuestionOther), "ほかの問い");
        assert_eq!(fall_of(&run_at("Questioned", None), None), Some(Fall::QuestionOther), "about の無い問い");

        assert_eq!(fall_of(&run_at("Landed", Some("closed")), None), None, "着地した便");
        assert_eq!(fall_of(&run_at("Implemented", None), Some("common")), None, "終端でない便");
        assert_eq!(fall_of(&run_at("Stopped", None), None), None, "止めた便は型を持たない");
        assert_eq!(fall_of(&[], None), None, "段の無い列");
        let revived = [staged("Reviewed", Some("verdict:FAIL kind:other")), staged("Spawned", None), staged("Landed", None)];
        assert_eq!(fall_of(&revived, None), None, "最後の段が着地なら過去の審査 FAIL は数えない");
    }

    /// 落ちの型 1 件の記帳（`RunFell`）。
    fn fell(run: &str, bead: &str, fall: &str) -> Event {
        Event::from_line(&format!(
            r#"{{"schema":1,"ts":"2026-10-07T00:00:00Z","kind":"RunFell","run":"{run}","bead":"{bead}","host":"h","actor":"machine","fall":"{fall}"}}"#
        ))
        .expect("fixture の行は読める")
    }

    /// 受付の断り 1 件の記帳（`IntakeRefused`）。
    fn refused(bead: &str, name: &str, ts: &str) -> Event {
        Event::from_line(&format!(
            r#"{{"schema":1,"ts":"{ts}","kind":"IntakeRefused","bead":"{bead}","refuse":"{name}","host":"h","actor":"machine"}}"#
        ))
        .expect("fixture の行は読める")
    }

    /// 数えの 1 要素。
    fn tally(kind: &str, proofs: &[&str]) -> Tally {
        Tally { kind: kind.to_owned(), proofs: proofs.iter().map(|proof| (*proof).to_owned()).collect() }
    }

    /// 数えは型の語ごとの累計で、証は新しい順・型の語は Fall の宣言順・間に別の型や PASS の便を挟んでも戻らず、列に無い bead の行は読まない。
    #[test]
    fn fallen_count_sums_each_kind_newest_first_across_other_kinds() {
        let events = [
            fell("r1", "b-1", "review-literal-mismatch"),
            fell("r2", "b-1", "gate-common"),
            staged("Reviewed", Some("verdict:PASS")),
            fell("r3", "b-1", "review-literal-mismatch"),
            fell("r4", "b-2", "review-literal-mismatch"),
            fell("r5", "b-1", "terminal"),
        ];
        let want = [tally("review-literal-mismatch", &["r3", "r1"]), tally("gate-common", &["r2"]), tally("terminal", &["r5"])];
        assert_eq!(counted(&events, &["b-1"], &[]), want, "b-1 だけを型の語ごとに新しい順");
        let both = counted(&events, &["b-1", "b-2"], &[]);
        assert_eq!(both.first(), Some(&tally("review-literal-mismatch", &["r4", "r3", "r1"])), "数える bead の列に在る bead は全部足す");
        assert!(counted(&events, &["b-3"], &[]).is_empty(), "列に無い bead の行は読まない");
    }

    /// 数えない受付の断りの名は 12 語で引数に依らず数えず、bead ごとに数えない型の語は 4 語で、渡すと数えず空を渡すと数える。
    #[test]
    fn fallen_count_skips_receptacles_and_uncounted_refusals_for_the_bead_stop() {
        let names = "max-live slot run-cap cap-headroom index-building code-facts-unmeasured same-kind-repeated not-a-repo rules store declaration generated";
        assert_eq!(UNCOUNTED_REFUSALS, names.split(' ').collect::<Vec<&str>>(), "数えない断りの名は 12 語");
        assert_eq!(UNCOUNTED_FOR_BEAD, ["review-other", "review-unparsed", "question-write-set", "question-other"], "bead ごとに数えない型は 4 語");
        assert!(UNCOUNTED_FOR_BEAD.iter().all(|word| Fall::parse(word).is_some()), "4 語は落ちの型の語");
        let mut events = vec![
            fell("r1", "b", "review-other"),
            fell("r2", "b", "review-unparsed"),
            fell("r3", "b", "question-write-set"),
            fell("r4", "b", "question-other"),
            fell("r5", "b", "review-vacuous-assert"),
        ];
        events.extend(UNCOUNTED_REFUSALS.iter().map(|name| refused("b", name, "2026-10-07T00:00:01Z")));
        assert_eq!(counted(&events, &["b"], UNCOUNTED_FOR_BEAD), [tally("review-vacuous-assert", &["r5"])], "受け皿の型と数えない断りは数えない");
        let all: Vec<String> = counted(&events, &["b"], &[]).into_iter().map(|found| found.kind).collect();
        assert_eq!(
            all,
            ["review-vacuous-assert", "review-other", "review-unparsed", "question-write-set", "question-other"],
            "空の列を渡すと 4 語の型も宣言順に数える（数えない断りは引数に依らず数えない）"
        );
    }

    /// 数えない名でない受付の断りは型の語 `intake-<名>`・証は `<bead>@<ts>` で、最初の `RunFell` より前の行は数えず、`RunFell` の無い列は空。
    #[test]
    fn fallen_count_reads_intake_refusals_only_after_the_first_run_fell() {
        let before = refused("b", "write-set-overlap", "2026-10-06T00:00:00Z");
        assert!(counted(std::slice::from_ref(&before), &["b"], &[]).is_empty(), "RunFell の無い列は空");
        let events = [
            before,
            fell("r1", "b", "gate-lens"),
            refused("b", "write-set-overlap", "2026-10-07T00:00:02Z"),
            refused("b", "max-live", "2026-10-07T00:00:03Z"),
            refused("b", "contract-table", "2026-10-07T00:00:04Z"),
            refused("b-2", "write-set-overlap", "2026-10-07T00:00:05Z"),
            refused("b", "write-set-overlap", "2026-10-07T00:00:06Z"),
        ];
        let want = [
            tally("gate-lens", &["r1"]),
            tally("intake-contract-table", &["b@2026-10-07T00:00:04Z"]),
            tally("intake-write-set-overlap", &["b@2026-10-07T00:00:06Z", "b@2026-10-07T00:00:02Z"]),
        ];
        assert_eq!(counted(&events, &["b"], &[]), want, "落ちの型の後に断りの型が名の辞書順・最初の RunFell より前の行は数えない");
    }

    /// 書き直しの便の材料 1 本。
    fn attempt(run: &str, unreviewed: bool, contract: &str, design: Option<&str>) -> Attempt {
        Attempt { run: run.to_owned(), unreviewed, contract: contract.to_owned(), design: design.map(str::to_owned) }
    }

    /// 止めて書き直しの便は、審査の前に止めた便のうち次の便（最後は今回）と契約の写しが違うか両方に在る設計の写しが違う便（新しい順）で、写しの無い側は違うと読まない。
    #[test]
    fn fallen_count_rewritten_names_stopped_unreviewed_runs_followed_by_changed_materials() {
        let runs = [
            attempt("a", true, "c1", Some("d1")),
            attempt("b", true, "c2", Some("d1")),
            attempt("c", true, "c2", None),
            attempt("d", false, "c2", Some("d2")),
            attempt("e", true, "c2", Some("d2")),
        ];
        assert_eq!(rewritten(&runs, ("c2", "d3")), ["e", "a"], "a は契約の写しが違い e は今回の節が違う・b と c は写しの無い側を違うと読まない・d は審査に届いた便");
        assert_eq!(rewritten(&runs, ("c2", "d2")), ["a"], "最後の便は今回の材料と同じなら書き直しでない");
        assert_eq!(rewritten(&runs, ("c3", "d2")), ["e", "a"], "今回の契約 file が違えば最後の便も書き直し");
        assert_eq!(rewritten(&[attempt("x", true, "c1", None)], ("c1", "d")), Vec::<String>::new(), "設計の写しの無い側は違うと読まない");
        assert!(rewritten(&[], ("c1", "d")).is_empty(), "便の無い列は空");
    }
}
