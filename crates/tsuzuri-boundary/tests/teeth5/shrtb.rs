//! 問いの起票の門の hook が短い題の断りを先に出す歯（接頭辞 shrtb_・設計ノート surface-wave27b 行 c-short-gate・規則の行 R-39）。
//! 境界の歯は JSON を読まないので hook の入力は字で組む。tz hook question-gate を偽の bd と偽の設計の道具（撃たれたら記録の file に 1 行を足して rc 1）で撃つ。作業場は CARGO_TARGET_TMPDIR の下。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tsuzuri_core::gate::{ShortWhy, short_output};

fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("shrtb")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("repo")).expect("repo");
    for program in ["bd", "folio"] {
        let path = root.join(program);
        fs::write(
            &path,
            format!("#!/bin/sh\necho x >> '{}/calls'\nexit 1\n", root.display()),
        )
        .expect("偽の program");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("権限");
    }
    root
}

/// tz hook question-gate を `command` の Bash の hook の入力で撃ち、rc と標準出力と、偽の program が撃たれた回を返す。
fn gate(root: &Path, command: &str) -> (Option<i32>, String, usize) {
    let command = command.replace('\\', "\\\\").replace('"', "\\\"");
    let input = format!(
        r#"{{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":"{command}"}}}}"#
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", "question-gate", "--repo"])
        .arg(root.join("repo"))
        .arg("--bd")
        .arg(root.join("bd"))
        .arg("--folio")
        .arg(root.join("folio"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("tz を撃つ");
    child
        .stdin
        .take()
        .expect("標準入力")
        .write_all(input.as_bytes())
        .expect("hook の入力");
    let out = child.wait_with_output().expect("tz の終わり");
    let calls = fs::read_to_string(root.join("calls"))
        .unwrap_or_default()
        .lines()
        .count();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        calls,
    )
}

#[test]
fn shrtb_deny_without_short() {
    let root = place("deny");
    let memo = "bdw create --parent=fx-q --labels=intake:memo --metadata='{\"touches\":[\"FR4\"]}' 門の歯のメモ";
    let want = format!("{}\n", short_output(ShortWhy::Missing));
    assert_eq!(gate(&root, memo), (Some(0), want.clone(), 0), "memo の起票");
    // 問いの起票も、問いの判じ（台帳と設計の索引の読み）より先に短い題で断る。
    let question = "bdw create --parent=fx-q --labels=intake:question --metadata='{\"touches\":[\"FR4\"]}' 門の歯の問い";
    assert_eq!(gate(&root, question), (Some(0), want, 0), "問いの起票");
    let long = format!(
        "bd create --metadata='{{\"short\":\"{}\"}}' 長い題",
        "長".repeat(21)
    );
    assert_eq!(
        gate(&root, &long),
        (Some(0), format!("{}\n", short_output(ShortWhy::Long)), 0)
    );
}

#[test]
fn shrtb_pass_with_short() {
    let root = place("pass");
    let created = format!(
        "bdw create --parent=fx-q --no-inherit-labels --metadata='{{\"short\":\"{}\"}}' 門の歯のメモ",
        "短".repeat(20)
    );
    assert_eq!(gate(&root, &created), (Some(0), String::new(), 0));
    assert_eq!(
        gate(&root, "bd update fx-q.1 --notes x"),
        (Some(0), String::new(), 0)
    );
}
