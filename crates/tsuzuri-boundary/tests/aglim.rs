//! 接頭辞 aglim_・設計ノート surface-v4a 行 t-stop-limit の歯。
//! 終える前の門の契約の file の撃ち直しの上限の歯（1 本ごとの上限と全部の上限・偽の scribe2 を PATH の頭に置き、tz hook agent-stop を撃つ）。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use tsuzuri_boundary::hook::subagent_stop::contract::shot_limit;

/// 係の記録（頼みの行だけ・SendMessage の呼びは無い）。
fn asked() -> String {
    r#"{"type":"user","message":{"content":"頼み"}}"#.to_string() + "\n"
}

/// 歯ごとの置き場（前の撃ちの残りを消して作る）。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("aglim")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("drafts")).unwrap();
    fs::create_dir_all(root.join("s/subagents")).unwrap();
    root
}

/// 欠けの無い終わりの置き場: 係 w218a の札（出す物 notes.md）と係の id a77 の結び・係の記録・w/notes.md（要点の見出し付き）。
/// 撃ちごとに 3 秒眠ってから記録の行を足して rc 0 で終わる偽の scribe2（置き場の bin）も置く。
fn whole(name: &str) -> PathBuf {
    let root = place(name);
    let dir = root.join("drafts/w218a");
    fs::create_dir_all(dir.join("w")).unwrap();
    fs::write(
        dir.join("spec.json"),
        r#"{"name":"w218a","type":"tsuzuri:drafter","budget":1000,"build":"なし","target":"t3-hub.87","outputs":["notes.md"],"spawned":1,"agent_id":"a77","ended":null}"#,
    )
    .unwrap();
    fs::write(dir.join("w/notes.md"), "# 要点\n分かった所\n").unwrap();
    fs::create_dir_all(root.join("drafts/.agents")).unwrap();
    fs::write(root.join("drafts/.agents/a77"), "w218a\n").unwrap();
    fs::write(root.join("s/subagents/agent-a77.jsonl"), asked()).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    let fake = root.join("bin/scribe2");
    let script = format!(
        "#!/bin/sh\nsleep 3\necho \"$*\" >> {calls}\nexit 0\n",
        calls = root.join("calls.txt").display()
    );
    fs::write(&fake, script).unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    root
}

/// 係の出力の dir（出す物と契約の file の置き場）。
fn w(root: &Path) -> PathBuf {
    root.join("drafts/w218a/w")
}

/// 最後の答え（1 行目 DONE・要点 1 行・最後の行に出力の dir の path・JSON の字のまま改行は \n の 2 字）。
fn said(root: &Path) -> String {
    format!("DONE\\n要点 1 行\\n{}", w(root).display())
}

/// tz hook agent-stop に、係の id a77 の 1 度目の終わりの入力を、偽の scribe2 を頭に置いた PATH で渡した結果。
fn stop(root: &Path) -> Output {
    let path = format!(
        "{}:{}",
        root.join("bin").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let input = format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"{}","cwd":"/W","permission_mode":"bypassPermissions","agent_id":"a77","agent_type":"tsuzuri:drafter","hook_event_name":"SubagentStop","stop_hook_active":false,"agent_transcript_path":"{}","last_assistant_message":"{}"}}"#,
        root.join("s.jsonl").display(),
        root.join("s/subagents/agent-a77.jsonl").display(),
        said(root)
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", "agent-stop", "--repo"])
        .arg(root)
        .arg("--drafts")
        .arg(root.join("drafts"))
        .env("PATH", path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}

/// 契約 1 本ごとの上限は 8 秒で、全部の上限 50 秒の残りが 8 秒より短ければその残りになり、50 秒以上は撃たない。
#[test]
fn aglim_shot_limit_gives_each_contract_its_own_limit() {
    let secs = Duration::from_secs;
    let millis = Duration::from_millis;
    assert_eq!(shot_limit(secs(0)), Some(secs(8)));
    assert_eq!(shot_limit(millis(8530)), Some(secs(8)));
    assert_eq!(shot_limit(secs(42)), Some(secs(8)));
    assert_eq!(shot_limit(secs(45)), Some(secs(5)));
    assert_eq!(shot_limit(millis(49999)), Some(millis(1)));
    assert_eq!(shot_limit(secs(50)), None);
    assert_eq!(shot_limit(secs(51)), None);
}

/// 撃ちごとに 3 秒眠る 3 本の契約（和は 8 秒を越える）が 1 度目の終わりで全部撃たれて通る。
#[test]
fn aglim_three_slow_contracts_pass_on_the_first_end() {
    let root = whole("slow");
    let dir = w(&root).join("contract");
    fs::create_dir_all(&dir).unwrap();
    for name in ["a.toml", "b.toml", "c.toml"] {
        fs::write(dir.join(name), "schema = 1\n").unwrap();
    }
    let out = stop(&root);
    assert_eq!((out.status.code(), out.stderr.len()), (Some(0), 0));
    let calls: Vec<String> = fs::read_to_string(root.join("calls.txt"))
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(calls.len(), 3, "{calls:?}");
    for (call, name) in calls.iter().zip(["a.toml", "b.toml", "c.toml"]) {
        assert!(
            call.contains(&format!("--contract {}", dir.join(name).display())),
            "{call}"
        );
    }
}
