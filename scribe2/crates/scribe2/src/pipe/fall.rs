//! 便の落ちの型（判断の記録 ADR-77 の決定 (7)・条 P-10.2・契約表の行 77-4 の前半）。
//!
//! 便の落ちは段の記帳の自由文の `detail` に散っている（審査の `Reviewed`・gate の `Gated` と verify の記録・`Failed`・質問）。
//! 本 file は落ちを**閉じた型の 1 語**に決める 1 本（[`fall_of`]・pure）と、その語の往復（[`Fall::as_str`] / [`Fall::parse`]）を持つ。
//! 書き手は器の列の周（[`super::dispatch`] の `fell`）だけで、記録の kind は [`crate::fleet::EventKind::RunFell`]。

use super::gate::{Check, Verdict, CHECKS};
use super::review::{read_detail, FindingKind};
use crate::fleet::{Event, Stage};

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

#[cfg(test)]
mod tests {
    use super::{fall_of, Fall, FALLS, LENS_STEP};
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
}
