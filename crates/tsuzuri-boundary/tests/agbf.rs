//! 起こしの予算の下限と予算の記録の歯（接頭辞 agbf_・設計ノート surface-wave29c 行 ag-budget-floor・判断の記録 ADR-63 の決定 (3)(9)）。
//! 中核の `agent::spec` の頭の読みと断りの理由の見本と、`agent::meter` の係の記録の数えと予算の記録を直に撃ち、
//! tz の binary の tz hook agent-spawn と agent-stop に係の試しの門の入力の形（会話の id と path は伏せた）を標準入力で渡し、
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）の drafts/ を --drafts で渡す。否定の見本は正しい見本から句を 1 つだけ外す。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_core::agent::meter::{TALLY, Tally, spent};
use tsuzuri_core::agent::spec::group::MEMBER_NEW;
use tsuzuri_core::agent::spec::{BUDGET_MIN, Call, HeadError, Refusal, Spec, head, judge, reason};

/// 頼みの頭の 4 行（予算の値 `budget`・組みの値 `build`）と空の行と本文。
fn prompt(budget: &str, build: &str) -> String {
    format!(
        "予算: {budget}\n組み: {build}\n対象: ag-budget-floor\n出す物: notes.md\n\n行を起草する。"
    )
}

/// 直した頭の見本の後ろの 3 行（組み・対象・出す物）。
const REST: &str = "\n組み: なし\n対象: ag-budget-floor\n出す物: notes.md";

/// 下限: 下限より 1 小さい予算と予算 1 は下限より小さいと断り、下限ちょうどの 150000 は通す。
#[test]
fn agbf_head_refuses_a_budget_below_the_floor() {
    assert_eq!(
        head(&prompt("token 149999", "なし")),
        Err(HeadError::Low(149_999))
    );
    assert_eq!(head(&prompt("token 1", "なし")), Err(HeadError::Low(1)));
    let top = head(&prompt("token 150000", "なし")).map(|h| h.budget);
    assert_eq!(top, Ok(150_000));
}

/// 下限は群の係ごとの上限以下で、上限ちょうどの予算の頭は下限を通る。
#[test]
fn agbf_floor_is_at_most_the_member_cap() {
    let (floor, cap) = (BUDGET_MIN, MEMBER_NEW);
    assert!(
        floor <= cap,
        "下限 {floor} が群の係ごとの上限 {cap} を越える"
    );
    let at_cap = head(&prompt(&format!("token {cap}"), "なし")).map(|h| h.budget);
    assert_eq!(at_cap, Ok(cap));
}

/// 起こしの判じは下限より小さい予算を頭の断りにし、理由は値と下限を名指して、直した頭の見本の予算の行だけを
/// 下限から上の書き方に替える。形の違う予算の行は今までの書き方、下限ちょうどの予算の行は値のまま。
#[test]
fn agbf_reason_fixes_only_the_low_budget_line() {
    let low = prompt("token 100000", "なし");
    let call = Call {
        kind: Some("tsuzuri:drafter".into()),
        name: Some("w238a".into()),
        prompt: low.clone(),
    };
    let r = judge(&call, &[], 1).unwrap_err();
    assert_eq!(r, Refusal::Head(HeadError::Low(100_000)));
    assert_eq!(
        reason(&r, &low),
        format!(
            "係の起こしの門は止める（予算の値 100000 が下限 150000 より小さい） 次の一手 = prompt の頭に次の 4 行を置き、空の行の後に頼みを書く\n予算: token <150000 以上の数>{REST}"
        )
    );
    let form = prompt("token 10万", "なし");
    let r = Refusal::Head(HeadError::Budget("token 10万".into()));
    assert!(
        reason(&r, &form).ends_with(&format!("\n予算: token <数>{REST}")),
        "{}",
        reason(&r, &form)
    );
    let edge = prompt("token 150000", "大");
    let r = Refusal::Head(HeadError::Build("大".into()));
    assert!(
        reason(&r, &edge).ends_with(
            "\n予算: token 150000\n組み: なし|軽|重\n対象: ag-budget-floor\n出す物: notes.md"
        ),
        "{}",
        reason(&r, &edge)
    );
}

/// 係の記録の assistant の行（応答の id の欄 `id`・usage の input・cache_creation・output・cache_read）。
fn turn(id: &str, input: u64, create: u64, output: u64, read: u64) -> String {
    format!(
        r#"{{"type":"assistant","message":{{{id}"content":[{{"type":"text","text":"調べ"}}],"usage":{{"input_tokens":{input},"cache_creation_input_tokens":{create},"output_tokens":{output},"cache_read_input_tokens":{read}}}}}}}"#
    )
}

/// 応答の id の欄の字。
fn id(id: &str) -> String {
    format!(r#""id":"{id}","#)
}

/// 席の道具と同じ数え: 同じ応答の id の行は離れていても最後の行だけ、id の無い行は 1 行ずつ数え、cache の読みと
/// user の行と JSON でない行は数えず、改行で終わらない最後の行も数える。
#[test]
fn agbf_spent_counts_like_the_seat_tool() {
    let lines = [
        r#"{"type":"user","message":{"content":"頼み"}}"#.to_string(),
        turn(&id("A"), 10, 100, 1, 1000),
        turn(&id("A"), 10, 100, 5, 1000),
        turn(&id("B"), 3, 0, 7, 500),
        turn(&id("A"), 10, 100, 9, 1000),
        r#"{"type":"user","message":{"content":"結果","usage":{"input_tokens":50}}}"#.to_string(),
        "{".to_string(),
        turn("", 2, 0, 2, 0),
        turn("", 1, 0, 1, 0),
    ];
    let record = lines.join("\n");
    assert_eq!(spent(record.as_bytes()), 119 + 10 + 4 + 2);
    assert_eq!(spent(format!("{record}\n").as_bytes()), 135);
    assert_eq!(spent(b""), 0);
}

/// 札（係 w238a・予算 150000・終えの印 `ended`）。
fn card(ended: Option<u64>) -> Spec {
    Spec {
        name: "w238a".into(),
        kind: "tsuzuri:drafter".into(),
        budget: 150_000,
        build: "重".into(),
        target: "ag-budget-floor".into(),
        outputs: vec!["notes.md".into()],
        spawned: 100,
        agent_id: Some("a77".into()),
        ended,
    }
}

/// 予算の記録の字（名・型・対象・予算・新しい量・倍率・時刻の順）。
fn rendered(used: u64, ratio: &str, at: u64) -> String {
    format!(
        "{{\n  \"name\": \"w238a\",\n  \"type\": \"tsuzuri:drafter\",\n  \"target\": \"ag-budget-floor\",\n  \"budget\": 150000,\n  \"used\": {used},\n  \"ratio\": {ratio},\n  \"at\": {at}\n}}\n"
    )
}

/// 予算の記録は札の名・型・対象・予算と、記録を数えた新しい量と、新しい量を予算で割って小数 3 桁に丸めた倍率と、
/// 札の終えの印の時刻（無ければ起こしの時刻）を持ち、file の名は usage.json。
#[test]
fn agbf_tally_holds_name_type_target_budget_used_ratio_and_time() {
    let record = turn(&id("A"), 100_000, 99_000, 1_000, 7);
    let t = Tally::of(&card(Some(200)), record.as_bytes());
    assert_eq!(t.render(), rendered(200_000, "1.333", 200));
    let t = Tally::of(&card(None), record.as_bytes());
    assert_eq!(t.render(), rendered(200_000, "1.333", 100));
    let small = turn(&id("A"), 1, 0, 0, 0);
    assert_eq!(
        Tally::of(&card(None), small.as_bytes()).render(),
        rendered(1, "0.0", 100)
    );
    assert_eq!(TALLY, "usage.json");
}

/// 歯ごとの置き場（前の撃ちの残りを消して作る）。drafts/ と、係の記録を置く親の記録の dir。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("agbf")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("drafts")).unwrap();
    fs::create_dir_all(root.join("s/subagents")).unwrap();
    root
}

/// tz hook <verb> --repo <root> --drafts <root>/drafts に `input` を渡した結果。
fn hook(verb: &str, root: &Path, input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", verb, "--repo"])
        .arg(root)
        .arg("--drafts")
        .arg(root.join("drafts"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}

/// 席の起こしの入力（型 tsuzuri:drafter・名 w238a・頼みの予算の値 `budget`・JSON の字の中の形）。
fn spawn_in(budget: &str) -> String {
    format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"/T/s.jsonl","cwd":"/W","permission_mode":"bypassPermissions","hook_event_name":"PreToolUse","tool_name":"Agent","tool_input":{{"description":"起草","prompt":"予算: {budget}\n組み: なし\n対象: ag-budget-floor\n出す物: notes.md\n\n行を起草する。","subagent_type":"tsuzuri:drafter","name":"w238a"}},"tool_use_id":"toolu_X"}}"#
    )
}

/// 起こしの門: 下限より 1 小さい予算の起こしを deny の答えで断って頼みと札を書かず、下限ちょうどは通して札の予算に書く。
#[test]
fn agbf_spawn_hook_denies_a_budget_below_the_floor() {
    let root = place("spawn");
    let low = hook("agent-spawn", &root, &spawn_in("token 149999"));
    let text = String::from_utf8_lossy(&low.stdout);
    assert_eq!(low.status.code(), Some(0));
    assert!(
        text.starts_with(r#"{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"係の起こしの門は止める（予算の値 149999 が下限 150000 より小さい） 次の一手 = "#),
        "{text}"
    );
    assert!(
        text.contains(r"予算: token <150000 以上の数>\n組み: なし"),
        "{text}"
    );
    assert!(!root.join("drafts/w238a").exists());
    let ok = hook("agent-spawn", &root, &spawn_in("token 150000"));
    assert_eq!((ok.status.code(), ok.stdout.len()), (Some(0), 0));
    let card = fs::read_to_string(root.join("drafts/w238a/spec.json")).unwrap();
    assert_eq!(Spec::parse(&card).map(|s| s.budget), Some(150_000));
}

/// SendMessage の呼びを持たず、usage の新しい量が 300000 の係の記録。
fn record() -> String {
    let said = r#"{"type":"assistant","message":{"id":"M2","content":[{"type":"text","text":"済み"}],"usage":{"input_tokens":1000,"cache_creation_input_tokens":49000,"output_tokens":0,"cache_read_input_tokens":9}}}"#;
    [
        r#"{"type":"user","message":{"content":"頼み"}}"#.to_string(),
        turn(&id("M1"), 50_000, 199_000, 1_000, 5),
        said.to_string(),
    ]
    .join("\n")
        + "\n"
}

/// 欠けの無い終わりの置き場: 係 w238a の札（予算 150000・出す物 notes.md）と係の id a77 の結び・係の記録・w/notes.md。
fn whole(name: &str) -> PathBuf {
    let root = place(name);
    let dir = root.join("drafts/w238a");
    fs::create_dir_all(dir.join("w")).unwrap();
    fs::write(dir.join("spec.json"), card(None).render()).unwrap();
    fs::write(dir.join("w/notes.md"), "# 要点\n分かった所\n").unwrap();
    fs::create_dir_all(root.join("drafts/.agents")).unwrap();
    fs::write(root.join("drafts/.agents/a77"), "w238a\n").unwrap();
    fs::write(root.join("s/subagents/agent-a77.jsonl"), record()).unwrap();
    root
}

/// tz hook agent-stop に係 a77 の終わりの入力（止めた後の終わりか `again`・最後の答えは 3 行の形で最後の行が出力の dir の path）を渡した結果。
fn stop(root: &Path, again: bool) -> Output {
    let w = root.join("drafts/w238a/w");
    let input = format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"{}","cwd":"/W","permission_mode":"bypassPermissions","agent_id":"a77","agent_type":"tsuzuri:drafter","hook_event_name":"SubagentStop","stop_hook_active":{again},"agent_transcript_path":"{}","last_assistant_message":"DONE\n要点 1 行\n{}"}}"#,
        root.join("s.jsonl").display(),
        root.join("s/subagents/agent-a77.jsonl").display(),
        w.display()
    );
    hook("agent-stop", root, &input)
}

/// 札の終えの印。
fn ended(root: &Path) -> Option<u64> {
    let card = fs::read_to_string(root.join("drafts/w238a/spec.json")).unwrap();
    Spec::parse(&card).and_then(|s| s.ended)
}

/// 終える前の門は、通す終わりに係の記録（入力の agent_transcript_path）を数えた予算の記録を係の dir の usage.json に書き、
/// 時刻は札に書いた終えの印。欠けの在る 2 度目の終わりも通すので書く。
#[test]
fn agbf_stop_hook_writes_the_tally_when_it_lets_the_agent_go() {
    let root = whole("pass");
    let out = stop(&root, false);
    assert_eq!(
        (out.status.code(), out.stdout.len(), out.stderr.len()),
        (Some(0), 0, 0)
    );
    let at = ended(&root).unwrap();
    let tally = fs::read_to_string(root.join("drafts/w238a").join(TALLY)).unwrap();
    assert_eq!(tally, rendered(300_000, "2.0", at));
    let again = whole("again");
    fs::remove_file(again.join("drafts/w238a/w/notes.md")).unwrap();
    let out = stop(&again, true);
    assert_eq!(out.status.code(), Some(0));
    let at = ended(&again).unwrap();
    let tally = fs::read_to_string(again.join("drafts/w238a").join(TALLY)).unwrap();
    assert_eq!(tally, rendered(300_000, "2.0", at));
}

/// 1 度目の終わりを止める時（出す物の欠け）は予算の記録を書かず、係の記録が読めない通す終わりは書かずに標準エラーに書く。
#[test]
fn agbf_stop_hook_skips_the_tally_on_a_hold_and_an_unread_record() {
    let held = whole("held");
    fs::remove_file(held.join("drafts/w238a/w/notes.md")).unwrap();
    let out = stop(&held, false);
    assert_eq!(out.status.code(), Some(2));
    assert!(!held.join("drafts/w238a").join(TALLY).exists());
    let unread = whole("unread");
    fs::remove_file(unread.join("s/subagents/agent-a77.jsonl")).unwrap();
    let out = stop(&unread, true);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0));
    assert!(ended(&unread).is_some());
    assert!(!unread.join("drafts/w238a").join(TALLY).exists());
    assert!(
        err.contains(
            "tz hook agent-stop: 係の記録が読めないか空なので予算の記録を書かない（通す）"
        ),
        "{err}"
    );
}
