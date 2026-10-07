//! `pipe dispatch` の歯。**本体は `dispatch.rs`** で、ここには test だけが在る。
//!
//! 分けたのは憲法 C4（1 file の上限）である——`dispatch.rs` が上限の 98.7% まで育ち、同じ file へ
//! 行を足す契約を受けられなくなった（`t3-hub.92.10.5`）。親が `mod tests;` で取り込むので、
//! module path は `pipe::dispatch::tests` のまま＝歯の名前は 1 つも変わらない。

// 純粋な移動（`dispatch.rs` の inline の `mod tests` から歯を足さずに写した・t3-hub.92.10.5）。
// flip-check: moved t3-hub.92.10.5

// flip-check: moved s2-07l.531
// 行頭の `#[cfg(test)]` は歯の区間の始まり（器の読み手は `tests` dir の外の file をこの印から末まで読む・file 全体が歯なのでここに 1 つ）。
#[cfg(test)]
use super::{
    admits_gated, advance, digits_of, handoff, hold_line, launch_of, marks_of, order, rank, released_after, requeues,
    review_unmeasured, revive_of, section_keyed, tools, why_of, Advance, Candidate, Handoff, Input, Pointer, WaitReason,
    DRIVE, HANDOFFS, WAIT_REASONS,
};
use super::{floor, reserve, Verdict};
use crate::fleet::{Event, EventKind, Mark, Stage, SCHEMA, STAGES};
use crate::rules::manifest::Manifest;
use std::path::Path;

/// 候補 1 件（印と priority だけを呼び手が選ぶ）。
fn candidate(bead: &str, priority: Option<u64>, mark: Option<Mark>) -> Candidate {
    Candidate { bead: bead.to_owned(), priority, mark, reason: None }
}

/// 印の行 1 件。
fn marked(ts: &str, bead: &str, mark: Mark) -> Event {
    Event {
        schema: SCHEMA,
        ts: ts.to_owned(),
        kind: EventKind::DispatchMark,
        run: String::new(),
        bead: bead.to_owned(),
        host: "h".to_owned(),
        actor: EventKind::DispatchMark.default_actor().to_owned(),
        stage: None,
        seat: None,
        pid: None,
        detail: None,
        allowance: None,
        registration: None,
        mark: Some(mark),
        account: None,
        cost: None,
        rule: None,
        case: None,
    }
}

/// 便の記帳 1 件（段の行・印を持たない）。
fn staged(ts: &str, run: &str, bead: &str, stage: Stage) -> Event {
    Event {
        schema: SCHEMA,
        ts: ts.to_owned(),
        kind: EventKind::RunStage,
        run: run.to_owned(),
        bead: bead.to_owned(),
        host: "h".to_owned(),
        actor: EventKind::RunStage.default_actor().to_owned(),
        stage: Some(stage),
        seat: None,
        pid: None,
        detail: None,
        allowance: None,
        registration: None,
        mark: None,
        account: None,
        cost: None,
        rule: None,
        case: None,
    }
}

/// `release` で列へ戻す段は **`Failed` / `Stopped` / `Gated`** の 3 つで、`Landed` と審査 FAIL の
/// `Reviewed` は戻さない（設計 §12・母集団 = [`STAGES`] の全段・網羅の match 1 本）。
#[test]
fn pipe_dispatch_release_requeues_failed_stopped_and_gated_but_not_landed_or_reviewed() {
    let back: Vec<Stage> = STAGES.iter().copied().filter(|stage| requeues(*stage)).collect();
    assert_eq!(
        back,
        vec![Stage::Gated, Stage::Stopped, Stage::Failed],
        "母集団 {} 段のうち戻すのは終端の 3 段（宣言順）",
        STAGES.len()
    );
    assert!(!requeues(Stage::Landed), "Landed は済んでいる（起こし直すと同じ変更をもう一度作る）");
    assert!(!requeues(Stage::Reviewed), "審査 FAIL は中身が変わるまで列に入らない（FR49）");
}

/// 列外の鍵に § の本文を含める段は **`Reviewed` だけ**で、`Landed` と `release` が戻す 3 段は含めない
/// （設計 §16・母集団 = [`STAGES`] の全段・網羅の match 1 本・[`requeues`] とは交わらない）。
#[test]
fn pipe_dispatch_section_key_applies_to_reviewed_only() {
    let keyed: Vec<Stage> = STAGES.iter().copied().filter(|stage| section_keyed(*stage)).collect();
    assert_eq!(keyed, vec![Stage::Reviewed], "母集団 {} 段のうち § を鍵に入れるのは審査の段だけ", STAGES.len());
    assert!(!section_keyed(Stage::Landed), "Landed は済んでいる（§ を直しても起こし直さない）");
    assert!(
        STAGES.iter().all(|stage| !(section_keyed(*stage) && requeues(*stage))),
        "§ の鍵と release の印は同じ段を持たない（審査の終端は印で戻さない・FR49）"
    );
}

/// `release` が効くのは**便の最後の記帳より後**の 1 件だけで、位置で引く（終端より前の印・別の
/// bead への印・`release` でない印は効かない・pure・設計 §12）。
#[test]
fn pipe_dispatch_release_requeues_only_when_the_mark_follows_the_last_record_of_the_run() {
    let run = "s2-a-1";
    let after = vec![
        staged("t1", run, "s2-a", Stage::Intake),
        staged("t2", run, "s2-a", Stage::Failed),
        marked("t3", "s2-a", Mark::Release),
    ];
    assert!(released_after(&after, run, "s2-a"), "終端の後の release は効く");
    let before = vec![
        staged("t1", run, "s2-a", Stage::Intake),
        marked("t2", "s2-a", Mark::Release),
        staged("t3", run, "s2-a", Stage::Failed),
    ];
    assert!(!released_after(&before, run, "s2-a"), "終端より前の release は効かない");
    let other = vec![staged("t1", run, "s2-a", Stage::Failed), marked("t2", "s2-b", Mark::Release)];
    assert!(!released_after(&other, run, "s2-a"), "別の bead への release は効かない");
    let held = vec![staged("t1", run, "s2-a", Stage::Failed), marked("t2", "s2-a", Mark::Hold)];
    assert!(!released_after(&held, run, "s2-a"), "release でない印は効かない");
    // 起こし直した便の記帳が release の後に並べば、その便から見て release は**前**に戻る（印は 1 回）。
    let again = "s2-a-2";
    let relaunched = vec![
        staged("t1", run, "s2-a", Stage::Failed),
        marked("t2", "s2-a", Mark::Release),
        staged("t3", again, "s2-a", Stage::Failed),
    ];
    assert!(!released_after(&relaunched, again, "s2-a"), "起こし直した便の終端の後には release が無い");
    assert!(!released_after(&[], run, "s2-a"), "記帳の無い便は「後」を測れない（効かない側）");
}

/// 判定で引く戻しが真になるのは **`INCONCLUSIVE` ∧ `Unparsed` の対だけ**で、`FAIL`（kind を問わず）・
/// `INCONCLUSIVE` で kind が他の 6 語・`PASS` は偽（設計 §22・母集団 = [`crate::pipe::review::FINDING_KINDS`] の 7 語 × 3 値）。
#[test]
fn pipe_dispatch_release_unparsed_is_the_only_judgement_that_requeues() {
    use crate::pipe::gate::Verdict;
    use crate::pipe::review::{FindingKind, Judgement, FINDING_KINDS};
    assert_eq!(FINDING_KINDS.len(), 7, "前提: 理由の型は 7 語");
    let mut back = Vec::new();
    for verdict in [Verdict::Pass, Verdict::Fail, Verdict::Inconclusive] {
        for kind in FINDING_KINDS {
            if review_unmeasured(&Judgement { verdict, kind: Some(*kind), at: Vec::new() }) {
                back.push((verdict, *kind));
            }
        }
        assert!(!review_unmeasured(&Judgement { verdict, kind: None, at: Vec::new() }), "kind の無い判定は戻さない");
    }
    assert_eq!(
        back,
        vec![(Verdict::Inconclusive, FindingKind::Unparsed)],
        "母集団 {} 対のうち戻すのは測れなかった 1 対だけ",
        FINDING_KINDS.len() * 3
    );
    let unparsed = |verdict| Judgement { verdict, kind: Some(FindingKind::Unparsed), at: Vec::new() };
    assert!(!review_unmeasured(&unparsed(Verdict::Fail)), "FAIL は kind を問わず戻さない（FR49）");
    assert!(review_unmeasured(&unparsed(Verdict::Inconclusive)), "INCONCLUSIVE ∧ unparsed は戻す");
}


/// 関門が開いた待ちの便の候補の選別（**pure**・設計 §13）: driver でない周は絞らず、driver の周は
/// 段の前進の 3 値のうち**前進だけ**が候補をそのまま起こし、同じ段のままと段が戻った周は 0 本にする
/// （空撃ちの連鎖を塞ぐ・母集団 = 3 値 + driver でない周）。
#[test]
fn pipe_dispatch_waiting_gate_admits_only_forward_drivers_and_every_non_driver() {
    assert!(admits_gated(None), "driver でない周（手動の 1 周・印・回答や承認の直後）は絞らない");
    assert!(admits_gated(Some(Advance::Forward)), "段を前へ進めた driver の周は候補をそのまま起こす");
    assert!(!admits_gated(Some(Advance::Same)), "同じ段のままの driver の周は 0 本");
    assert!(!admits_gated(Some(Advance::Backward)), "段が戻った driver の周は 0 本");
    let listed = [Advance::Forward, Advance::Same, Advance::Backward];
    let admitted = listed.iter().filter(|moved| admits_gated(Some(**moved))).count();
    assert_eq!(admitted, 1, "母集団 {} 値のうち起こすのは前進の 1 値だけ", listed.len());
}

/// 列が起こす便には**常に** `--drive` が付く（起こす側・起こし直す側の両方）。道具の
/// pass-through（[`tools`]）とは**別の定数**である——渡された道具に混ぜると、`--rules` 等を
/// 渡していない呼び手の周だけ自走しなくなる（「全部か皆無か」の列に載せない理由）。
#[test]
fn pipe_dispatch_drive_is_added_to_every_run_the_queue_starts() {
    let manifest = Manifest::embedded().expect("埋め込みの manifest を読める");
    let input = Input {
        state_dir: Path::new("s"),
        repo: Path::new("r"),
        manifest: &manifest,
        bd: "bd",
        bd_flag: None,
        rules: None,
        lens: None,
        curl: None,
        runner: Some("true"),
        driving: None,
        driven: None,
    };
    let pointer = Pointer { path: "docs/design/toy.md".to_owned(), id: "a".to_owned() };
    let started = launch_of(&input, "s2-toy.1", &pointer).argv;
    let revived = revive_of(&input, "s2-toy.1-1").argv;
    assert_eq!(started.last().map(String::as_str), Some(DRIVE), "起こす側: {started:?}");
    assert_eq!(revived.last().map(String::as_str), Some(DRIVE), "起こし直す側: {revived:?}");
    // **道具の列には入らない**（母集団 = 渡された道具 1 件 `--runner`）。
    let passed = tools(&input);
    assert_eq!(passed, vec!["--runner".to_owned(), "true".to_owned()], "道具は渡された分だけ");
    assert!(!passed.contains(&DRIVE.to_owned()), "自走の flag は道具の pass-through ではない: {passed:?}");
}

/// 段の位置は [`STAGES`] の宣言から導く（順序の宣言は 1 か所・C2）。**全 variant が母集団に在る**
/// ことをここで測る——1 つでも欠けると [`rank`] の `unwrap_or` が同じ値を 2 つの段に配り、
/// [`advance`] が静かに「同じ段」を返す。
#[test]
fn pipe_dispatch_drive_ranks_every_stage_from_the_declared_order() {
    let ranks: Vec<usize> = STAGES.iter().map(|stage| rank(*stage)).collect();
    assert_eq!(ranks, (0..STAGES.len()).collect::<Vec<usize>>(), "母集団 {} 段が宣言順の位置を持つ", STAGES.len());
    assert!(ranks.iter().all(|found| *found < STAGES.len()), "外れ値（母集団の外）の段は無い: {ranks:?}");
}

/// 段の動きは**閉じた 3 形**（pure・設計 §5）: 入口に段が無い周（`pipe run`）は前進・
/// 宣言順に進めば前進・同じ段は `Same`・戻れば `Backward`。
#[test]
fn pipe_dispatch_drive_advance_is_forward_same_or_backward() {
    assert_eq!(advance(None, Stage::Intake), Advance::Forward, "便を作った周は入口に段が無い");
    assert_eq!(advance(Some(Stage::Implemented), Stage::Gated), Advance::Forward, "Implemented → Gated");
    assert_eq!(advance(Some(Stage::Gated), Stage::Gated), Advance::Same, "同じ段");
    assert_eq!(advance(Some(Stage::Gated), Stage::Implemented), Advance::Backward, "追随で戻った段");
}

/// 渡すのは **前進 ∧ 待ちの段でない ∧ 終端でない** 周だけで、渡さない周は理由を名乗る
/// （**閉じた 5 値**・C10）。生死が読めない周を「終端」に読み替えない。
#[test]
fn pipe_dispatch_drive_hands_off_only_on_forward_and_names_the_reason() {
    let live = Some(true);
    assert_eq!(handoff(Advance::Forward, Stage::Gated, live), Handoff::Pass, "前進・生きている・待ちでない");
    assert_eq!(handoff(Advance::Same, Stage::Gated, live), Handoff::NoProgress, "段が動かなかった");
    assert_eq!(handoff(Advance::Backward, Stage::Implemented, live), Handoff::NoProgress, "戻った段");
    assert_eq!(handoff(Advance::Forward, Stage::Blocked, live), Handoff::Waiting, "承認待ち");
    assert_eq!(handoff(Advance::Forward, Stage::Questioned, live), Handoff::Waiting, "回答待ち");
    assert_eq!(handoff(Advance::Forward, Stage::Landed, Some(false)), Handoff::Settled, "終端");
    assert_eq!(handoff(Advance::Forward, Stage::Gated, None), Handoff::Unmeasured, "生死を読めない");
}

/// `drive=` の値は [`HANDOFFS`] と 1 対 1（宣言順・字面は 1 か所）。
#[test]
fn pipe_dispatch_drive_tokens_are_the_closed_five() {
    let listed =
        [Handoff::Pass, Handoff::Waiting, Handoff::Settled, Handoff::NoProgress, Handoff::Unmeasured];
    let names: Vec<&str> = listed.iter().map(|found| found.as_str()).collect();
    assert_eq!(names, HANDOFFS, "母集団 {} 値（宣言順）", HANDOFFS.len());
}

/// 順序は (1) `first` (2) priority (3) 起票順（設計 dispatcher.md §2）。**散文の順序を持たない**ので、
/// 3 つの軸が同時に食い違う列で 1 関数の順だけを測る。
#[test]
fn pipe_dispatch_order_puts_first_before_priority_then_the_issue_number() {
    let listed = vec![
        candidate("s2-07l.100", Some(0), None),
        candidate("s2-07l.99", Some(4), Some(Mark::First)),
        candidate("s2-07l.20", Some(2), None),
        candidate("s2-07l.3", Some(0), None),
        candidate("s2-07l.7", None, None),
    ];
    let sorted: Vec<String> = order(listed.clone()).into_iter().map(|found| found.bead).collect();
    assert_eq!(
        sorted,
        vec!["s2-07l.99", "s2-07l.3", "s2-07l.100", "s2-07l.20", "s2-07l.7"],
        "母集団 {} 件（first → P0 の 2 件は起票順 → P2 → priority を読めない 1 件は最後尾）",
        listed.len()
    );
}

/// 起票順の鍵は id の**数字の列**である（辞書順だと `.100` が `.20` より前に来る）。
#[test]
fn pipe_dispatch_order_reads_the_issue_number_as_digits_not_text() {
    assert_eq!(digits_of("s2-07l.345"), vec![2, 7, 345], "数字の列");
    assert_eq!(digits_of("s2-07l.479.2"), vec![2, 7, 479, 2], "小数点の先も 1 つの数");
    assert_eq!(digits_of("s2-abc"), vec![2], "数字を持たない部分は鍵に入らない");
    assert!(digits_of("s2-07l.20") < digits_of("s2-07l.100"), "辞書順なら逆になる 2 件");
}

/// 印は bead ごとに**最後の 1 件**が効き、`release` は印を外す（event log の往復・設計 §4）。
#[test]
fn pipe_dispatch_marks_keep_the_last_one_and_release_removes_it() {
    let events = vec![
        marked("t1", "s2-a", Mark::First),
        marked("t2", "s2-b", Mark::Hold),
        marked("t3", "s2-a", Mark::Hold),
        marked("t4", "s2-b", Mark::Release),
    ];
    let found = marks_of(&events).order;
    assert_eq!(found.get("s2-a").map(|(mark, _)| *mark), Some(Mark::Hold), "後の印が勝つ");
    assert_eq!(found.get("s2-a").map(|(_, ts)| ts.clone()), Some("t3".to_owned()), "ts は勝った印の行");
    assert_eq!(found.get("s2-b"), None, "release は印を外す");
    assert_eq!(found.len(), 1, "母集団 {} 行のうち残る印は 1 つ", events.len());
}

/// 起こした事実の印（`launched`）は介入の印と**独立の値**で最新の 1 つを持ち、同じ bead の `RunCreated` と
/// `release` が外す（別の bead の `RunCreated` は外さない・pure・設計 §17）。
#[test]
fn pipe_dispatch_launched_marks_are_cleared_by_run_created_or_release() {
    let mut created = staged("t4", "s2-b-1", "s2-b", Stage::Intake);
    created.kind = EventKind::RunCreated;
    let events = vec![
        marked("t1", "s2-a", Mark::Hold),
        marked("t2", "s2-a", Mark::Launched),
        marked("t3", "s2-b", Mark::Launched),
        created,
        marked("t5", "s2-c", Mark::Launched),
        marked("t6", "s2-c", Mark::Release),
        marked("t7", "s2-d", Mark::Launched),
        marked("t8", "s2-d", Mark::Launched),
    ];
    let found = marks_of(&events);
    assert_eq!(found.order.get("s2-a").map(|(mark, _)| *mark), Some(Mark::Hold), "launched は hold を上書きしない");
    assert_eq!(found.launched.get("s2-a").map(String::as_str), Some("t2"), "受付に届いていない bead");
    assert_eq!(found.launched.get("s2-b"), None, "RunCreated が外す");
    assert_eq!(found.launched.get("s2-c"), None, "release が外す");
    assert_eq!(found.launched.get("s2-d").map(String::as_str), Some("t8"), "最新の 1 つ");
    assert_eq!(found.launched.len(), 2, "母集団 4 bead のうち残るのは 2 つ");
}

/// 止めの印の行 1 件（detail に理由の字）。
fn held_with(ts: &str, bead: &str, detail: Option<&str>) -> Event {
    let mut event = marked(ts, bead, Mark::Hold);
    event.detail = detail.map(str::to_owned);
    event
}

/// 止めの理由は最後の印が理由つきの `hold` の bead だけが持ち、`first`・`release`・理由の無い `hold`・頭が `reason:` でない
/// detail の `hold` は持たない（pure・行 v-hold-why）。
#[test]
fn vhold_marks_keep_the_reason_of_the_last_hold_only() {
    let events = vec![
        held_with("t1", "s2-a", Some("reason:設計の行を直す")),
        held_with("t2", "s2-b", Some("reason:x")),
        marked("t3", "s2-b", Mark::First),
        held_with("t4", "s2-c", Some("reason:y")),
        marked("t5", "s2-c", Mark::Release),
        held_with("t6", "s2-d", Some("reason:z")),
        held_with("t7", "s2-d", None),
        held_with("t8", "s2-e", Some("why:w")),
        held_with("t9", "s2-f", Some("reason:前")),
        held_with("t10", "s2-f", Some("reason:後")),
    ];
    let found = marks_of(&events);
    assert_eq!(found.why.get("s2-a").map(String::as_str), Some("設計の行を直す"), "理由つきの hold は理由を持つ");
    assert_eq!(found.why.get("s2-b"), None, "後の first が外す");
    assert_eq!(found.why.get("s2-c"), None, "release が外す");
    assert_eq!(found.why.get("s2-d"), None, "後の理由の無い hold が外す");
    assert_eq!(found.why.get("s2-e"), None, "頭が reason: でない detail は理由でない");
    assert_eq!(found.why.get("s2-f").map(String::as_str), Some("後"), "最後の hold の理由");
    assert_eq!(found.why.len(), 2, "母集団 6 bead のうち理由を持つのは 2 つ");
    assert_eq!(found.order.get("s2-d"), Some(&(Mark::Hold, "t7".to_owned())), "理由の無い hold も印は効く");
}

/// `hold` は 1 行の理由を要り detail を `reason:<理由>` にし、ほかの印は `--reason` を断り理由なしは detail なし（pure・行 v-hold-why）。
#[test]
fn vhold_why_of_needs_one_line_on_hold_and_refuses_it_on_the_other_marks() {
    assert_eq!(why_of(Mark::Hold, Some("設計の行 を直す")), Ok(Some("reason:設計の行 を直す".to_owned())), "通る理由");
    assert_eq!(why_of(Mark::Hold, None), Err("hold は --reason の理由を要る".to_owned()), "理由なし");
    assert_eq!(why_of(Mark::Hold, Some(" \t ")), Err("--reason の理由が空である".to_owned()), "空白だけ");
    assert_eq!(why_of(Mark::Hold, Some("前\n後")), Err("--reason の理由が改行を含む".to_owned()), "改行");
    assert_eq!(why_of(Mark::Hold, Some("前\r後")), Err("--reason の理由が改行を含む".to_owned()), "復帰");
    for mark in [Mark::First, Mark::Release, Mark::Launched] {
        assert_eq!(why_of(mark, Some("x")), Err(format!("--reason は hold だけが受ける（{}）", mark.as_str())), "{mark:?}");
        assert_eq!(why_of(mark, None), Ok(None), "{mark:?} は理由なしで通る");
    }
}

/// 止めの行は理由が `hold` の候補だけに立ち、bead と印の時刻と理由（理由の無い古い印は `-`）を名指す（pure・行 v-hold-why）。
#[test]
fn vhold_hold_line_names_bead_since_and_why_only_for_the_hold_reason() {
    let with = |reason: Option<WaitReason>| Candidate { reason, ..candidate("s2-a", Some(2), Some(Mark::Hold)) };
    let hold = |why: Option<&str>| with(Some(WaitReason::Hold { since: "t1".to_owned(), why: why.map(str::to_owned) }));
    assert_eq!(hold_line(&hold(Some("直す 間"))), Some("[DISPATCH-HOLD] bead=s2-a since=t1 why=直す 間".to_owned()), "理由つき");
    assert_eq!(hold_line(&hold(None)), Some("[DISPATCH-HOLD] bead=s2-a since=t1 why=-".to_owned()), "理由の無い古い印");
    let behind = with(Some(WaitReason::Dependency { on: vec!["s2-z".to_owned()] }));
    assert_eq!(hold_line(&behind), None, "印が hold でも理由が依存の候補は立たない");
    assert_eq!(hold_line(&with(None)), None, "理由の無い候補は立たない");
}

// flip-check: retroactive s2-07l.738.37.2
// flip-check: retroactive s2-07l.738.37.5
/// 理由の名は [`WAIT_REASONS`] と 1 対 1 で、値を持つ variant は値も描く（`dispatch ls` の `reason=`）。
#[test]
fn pipe_dispatch_wait_reasons_render_the_name_and_the_value() {
    let listed = vec![
        WaitReason::Dependency { on: vec!["s2-x".to_owned(), "s2-y".to_owned()] },
        WaitReason::Overlap { with: "r1".to_owned(), files: vec!["src/a.rs".to_owned(), "src/b/".to_owned()], verdict: None },
        WaitReason::Overlap { with: "r2".to_owned(), files: vec!["src/a.rs".to_owned()], verdict: Some(Verdict::SameName) },
        WaitReason::Admission { reason: "cap-headroom", why: Some("x".to_owned()) },
        WaitReason::HostBusy,
        WaitReason::Hold { since: "t1".to_owned(), why: Some("x".to_owned()) },
        WaitReason::Launched { since: "t2".to_owned() },
        WaitReason::Settled { sha: "abc".to_owned(), stage: Stage::Landed },
        WaitReason::NoDesignPointer,
        WaitReason::UnreflectedRuling { id: "s2-q:20260930T0000Z-1".to_owned() },
        WaitReason::Floor(floor_judged(floor::Word::Fail, Some(2), None, "")),
        WaitReason::Floor(floor_judged(floor::Word::Unfireable, None, Some("path"), "")),
        WaitReason::Floor(floor_judged(floor::Word::Timeout, None, None, "")),
        WaitReason::Reserved(reserve::Held { by: "s2-b".to_owned(), files: 2, unset: false }),
        WaitReason::Reserved(reserve::Held { by: "s2-b".to_owned(), files: 1, unset: true }),
        WaitReason::Sibling("s2-b".to_owned()),
        WaitReason::Sibling("s2-b/unset".to_owned()),
        WaitReason::RunCap("r3".to_owned()),
    ];
    let mut names: Vec<&str> = listed.iter().map(WaitReason::as_str).collect();
    names.dedup();
    assert_eq!(names, WAIT_REASONS, "母集団 {} 件（宣言順）", WAIT_REASONS.len());
    let rendered: Vec<String> = listed.iter().map(WaitReason::render).collect();
    assert_eq!(
        rendered,
        vec![
            "dependency:s2-x,s2-y",
            "overlap:r1/2",
            "overlap:r2/1/same-name",
            "admission:cap-headroom",
            "host-busy",
            "hold:t1",
            "launched:t2",
            "settled:abc/Landed",
            "no-design-pointer",
            "unreflected-ruling:s2-q:20260930T0000Z-1",
            "floor:2",
            "floor:unfireable",
            "floor:timeout",
            "reserved:s2-b/2",
            "reserved:s2-b/1/unset",
            "sibling:s2-b",
            "sibling:s2-b/unset",
            "run-cap:r3",
        ],
        "値を持つ 10 件は値も描き、床は 3 形（rc・unfireable・timeout）・行の予約と兄弟の待ちは 2 形（末尾 /unset）"
    );
}

/// 自分自身の起動は起動の記述を通る（設計 core-boundary.md §9 行 d）: 記録の program は [`super::myself`]・引数は
/// `pipe` に続く argv の逐語。記録する stub は spawn を断るので、起動は偽（呼び手は「起こせなかった」と読む）。
/// argv は子が起きても何も撃たない形（歯の binary の該当 0 本の filter）にする。
#[test]
fn invocation_pipe_flow_dispatch_self_launch_failure_is_false() {
    use crate::pipe::fixture::{exited, scratch, Stub};
    let state = scratch("dispatch-self-launch");
    let argv = vec!["--exact".to_owned(), "no-such-test-invocation-dispatch".to_owned()];
    let stub = Stub::install(|_| exited(0, b""));
    assert!(!super::spawn_self(&state, &argv), "stub の断りは起動の失敗＝偽");
    let calls = stub.calls();
    assert_eq!(calls.len(), 1, "起動は 1 回: {calls:?}");
    let found: Vec<(String, Vec<String>)> = calls.into_iter().map(|call| (call.program, call.args)).collect();
    let expected: Vec<String> = std::iter::once(super::PIPE.to_owned()).chain(argv.iter().cloned()).collect();
    assert_eq!(found, vec![(super::myself(), expected)], "program は自分・引数は pipe と argv");
    let _ = std::fs::remove_dir_all(&state);
}

/// 床の判定の 1 件（歯の fixture・sha は 40 字）。
fn floor_judged(word: floor::Word, rc: Option<i32>, why: Option<&str>, summary: &str) -> floor::Judged {
    floor::Judged { sha: "0123456789abcdef0123456789abcdef01234567".to_owned(), word, rc, why: why.map(str::to_owned), summary: summary.to_owned() }
}

/// 床の今の判定の file の本文は同じ reader で読み戻せ（引用符・逆斜線・日本語も）、doctor の行は逐語の形（rc と理由が無い欄は `-`・
/// 要約が空なら `-`）。読めない本文は `None`。
#[test]
fn pipe_dispatch_floor_current_body_round_trips_and_renders_the_doctor_line() {
    use super::floor::Word;
    for found in [
        floor_judged(Word::Pass, Some(0), None, "ok \"quoted\" \\ done"),
        floor_judged(Word::Fail, Some(-1), None, ""),
        floor_judged(Word::Unfireable, None, Some("path"), ""),
        floor_judged(Word::Timeout, None, None, "日本語 summary"),
    ] {
        assert_eq!(floor::judged_of(&found.body()), Some(found.clone()), "{}", found.body());
    }
    assert_eq!(floor_judged(Word::Pass, Some(0), None, "ok").line(), "floor=pass rc=0 sha=0123456 why=- summary=ok");
    assert_eq!(floor_judged(Word::Unfireable, None, Some("path"), "").line(), "floor=unfireable rc=- sha=0123456 why=path summary=-");
    assert_eq!(floor::judged_of("not json"), None);
}

/// 床の要約は最後の空でない行・制御文字を除いた頭の 200 字で、空白だけの行は空に数え、何も無ければ `None`。
#[test]
fn pipe_dispatch_floor_summary_is_the_last_nonblank_line_without_controls() {
    assert_eq!(floor::summary_of(b"a\nb\n\n  \n"), Some("b".to_owned()));
    let long = format!("x\t\u{1b}{}", "y".repeat(300));
    assert_eq!(floor::summary_of(long.as_bytes()).map(|found| found.chars().count()), Some(200));
    assert_eq!(floor::summary_of(b"\n \t\n"), None);
    assert_eq!(floor::summary_of(b""), None);
}

/// 床の行の 3 つの検査は (i) form → (ii) denied → (iii) metachar の順に最初の語を名乗り、当たらない行は通る。
#[test]
fn pipe_dispatch_floor_fault_names_the_first_of_form_denied_metachar() {
    let manifest = Manifest::embedded().expect("埋め込みは読める");
    for (row, want) in [
        ("a\nb; c", Some("form")),
        ("a\tb", Some("form")),
        ("a {base}", Some("form")),
        ("git push --force; ls", Some("denied")),
        ("git branch -D x", Some("denied")),
        ("ls | wc", Some("metachar")),
        ("cargo check --workspace", None),
    ] {
        assert_eq!(floor::fault(&manifest, row), want, "{row:?}");
    }
}

/// 待つ側が呼ぶ読み手は file だけを読む: 置き場の無い state dir は何も持たず、sha の結果の file が今の判定に先立ち、今の判定は
/// 同じ sha のときだけ返る。
#[test]
fn pipe_dispatch_floor_judgement_reads_the_result_then_the_current_of_the_same_sha() {
    use crate::pipe::fixture::scratch;
    let state = scratch("floor-judgement");
    let (sha, other) = ("0123456789abcdef0123456789abcdef01234567", "89abcdef0123456789abcdef0123456789abcdef");
    assert_eq!((floor::doctor_line(&state), floor::judgement(&state, sha)), (None, None), "置き場の無い周");
    let dir = state.join("pipe").join("floor");
    std::fs::create_dir_all(&dir).expect("floor の dir を作れる");
    let timed = floor_judged(floor::Word::Timeout, None, None, "");
    std::fs::write(dir.join("current"), timed.body()).expect("今の判定を置ける");
    assert_eq!(floor::judgement(&state, sha), Some(timed.clone()), "結果が無ければ同じ sha の今の判定");
    assert_eq!(floor::judgement(&state, other), None, "別の sha の今の判定は返さない");
    assert_eq!(floor::doctor_line(&state), Some(timed.line()), "doctor の行");
    std::fs::write(dir.join(format!("{sha}.result")), "{\"schema\":1,\"rc\":2,\"summary\":\"boom\"}\n").expect("結果を置ける");
    let failed = floor::judgement(&state, sha).expect("結果を読める");
    assert_eq!((failed.word, failed.rc, failed.summary.as_str()), (floor::Word::Fail, Some(2), "boom"), "結果の file が先");
    std::fs::write(dir.join(format!("{other}.result")), "{\"schema\":1,\"rc\":0,\"summary\":\"\"}\n").expect("結果を置ける");
    assert_eq!(floor::judgement(&state, other).map(|found| found.word), Some(floor::Word::Pass), "rc 0 は pass");
    let _ = std::fs::remove_dir_all(&state);
}
// flip-check: moved s2-07l.736.33.22
