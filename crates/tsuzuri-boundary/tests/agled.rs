//! 係の門の台帳と器の state の検めの歯（接頭辞 agled_・判断の記録 ADR-59 決定 (3)(4)・条 N-2）。
//! 中核の `ledger_word` と `ledger_refusal` を直に撃ち、tz の binary の tz hook agent-guard に係の呼びの入力の形（会話の id と path は伏せた）を
//! 標準入力で渡し、歯ごとの置き場（CARGO_TARGET_TMPDIR の下）を --drafts で渡す。測りの札は置かない（使った量 0 で予算の前の段）。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_core::agent::guard::{ledger_refusal, ledger_word};

/// 歯ごとの置き場（前の撃ちの残りを消して作る）の drafts/。
fn place(name: &str) -> PathBuf {
    let drafts = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("agled")
        .join(name)
        .join("drafts");
    let _ = fs::remove_dir_all(&drafts);
    fs::create_dir_all(&drafts).unwrap();
    drafts
}

/// 係 w218a の札（予算 1000・組み `build`・出す物 notes.md）と、係の id a77 から名への結び。
fn bind(drafts: &Path, build: &str) {
    let dir = drafts.join("w218a");
    fs::create_dir_all(&dir).unwrap();
    let spec = format!(
        r#"{{"name":"w218a","type":"tsuzuri:drafter","budget":1000,"build":"{build}","target":"t3-hub.87","outputs":["notes.md"],"spawned":1,"agent_id":"a77","ended":null}}"#
    );
    fs::write(dir.join("spec.json"), spec).unwrap();
    fs::create_dir_all(drafts.join(".agents")).unwrap();
    fs::write(drafts.join(".agents/a77"), "w218a\n").unwrap();
}

/// 係の id `head`（空なら席の呼び）の道具 `tool` の呼びの前の入力（tool_input は `input` の JSON の字）。
fn input(head: &str, tool: &str, input: &str) -> String {
    format!(
        r#"{{"session_id":"00000000-0000-4000-8000-000000000000","transcript_path":"/T/s.jsonl","cwd":"/W","permission_mode":"bypassPermissions",{head}"hook_event_name":"PreToolUse","tool_name":"{tool}","tool_input":{input},"tool_use_id":"toolu_X"}}"#
    )
}

/// tz hook agent-guard に `payload` を渡した結果。
fn guard(drafts: &Path, payload: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", "agent-guard", "--repo"])
        .arg(drafts)
        .arg("--drafts")
        .arg(drafts)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(payload.as_bytes());
    child.wait_with_output().unwrap()
}

/// 係の呼びの頭の欄（係の id a77）。
const SUB: &str = r#""agent_id":"a77","agent_type":"tsuzuri:drafter","#;

/// deny の答えの理由の頭（PreToolUse の deny の JSON の頭と、断りの字の頭の `why`）。
fn deny_head(why: &str) -> String {
    format!(
        r#"{{"hookSpecificOutput":{{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"{why}"#
    )
}

/// 何も出さずに rc 0 で通る。
fn passed(out: &Output, what: &str) {
    assert_eq!(
        (out.status.code(), out.stdout.as_slice()),
        (Some(0), &b""[..]),
        "{what} {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// deny の答えで、理由が `why` で始まる。
fn denied(out: &Output, why: &str, what: &str) {
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{what} {text}");
    assert!(text.starts_with(&deny_head(why)), "{what} {text}");
}

/// Bash の呼びの tool_input。
fn bash(command: &str) -> String {
    format!(r#"{{"command":"{command}"}}"#)
}

/// 台帳と器の state の検めの断りの理由の頭（撃たない語 `word`）。
fn why(word: &str) -> String {
    format!("係の門は止める（係の Bash で、台帳か器の state を書く語 {word} は撃たない）")
}

/// 台帳と器の state を書く語は、command の頭の語の列から読む。
#[test]
fn agled_ledger_word_names_the_writes() {
    let agent = Path::new("/D/w218a");
    for (command, word) in [
        ("bdw show t3-hub.1", "bdw"),
        ("bd update t3-hub.1 --notes x", "bd update"),
        ("bd --readonly update t3-hub.1", "bd update"),
        ("cd /R && bd dep add t3-hub.1 t3-hub.2", "bd dep"),
        ("BEADS_DIR=/R/.beads bd create x", "bd create"),
        ("bash -c 'bd note t3-hub.1 x'", "bd note"),
        ("/h/.local/bin/bd sql x", "bd sql"),
        ("tz hook agent-bind --repo /R", "tz hook"),
        ("tzw stage notify", "tzw stage"),
        ("tz consult open", "tz consult"),
        ("tz surface serve --repo /R", "tz surface"),
        ("scribe2 fleet ls", "scribe2 fleet"),
        ("scribe2 pipe land --repo /R", "scribe2 pipe"),
        (
            "scribe2 pipe preflight --contract /D/w218a/w/contract/a.toml --state-dir /S/pf",
            "scribe2 pipe",
        ),
        (
            "scribe2 pipe preflight --contract /D/w218a/w/contract/a.toml",
            "scribe2 pipe",
        ),
        (
            "scribe2 pipe preflight --state-dir /D/w218a/pf --state-dir /S/pf",
            "scribe2 pipe",
        ),
        (
            "scribe2 pipe preflight --state-dir /D/w218a/../w219a/pf",
            "scribe2 pipe",
        ),
        (
            "scribe2 pipe preflight --state-dir /D/w218ax/pf",
            "scribe2 pipe",
        ),
        ("scribe2 pipe preflight --state-dir", "scribe2 pipe"),
    ] {
        assert_eq!(ledger_word(command, agent).as_deref(), Some(word), "{command}");
    }
}

/// 読みの口と係の dir の下の state dir の preflight と、語を頭に持たない command は通す。
#[test]
fn agled_ledger_word_lets_the_reads_through() {
    let agent = Path::new("/D/w218a");
    for command in [
        "bd --readonly show t3-hub.1 --json",
        "bd list --status open",
        "bd -C /R --readonly show t3-hub.1",
        "bd --db /R/.beads/x.db show t3-hub.1",
        "bd --version",
        "grep -n bd notes.md",
        "echo bdw",
        "tz check --dir /T/design-intent --prose /D/w218a/w/contract/a.md",
        "tz derive --dir . --out ../contracts --write",
        "scribe2 pipe preflight --contract /D/w218a/w/contract/a.toml --bead t3-hub.1 --repo /T --state-dir /D/w218a/pf-state",
        "scribe2 pipe preflight --state-dir=/D/w218a/w/st",
        "scribe2 --version",
        "cargo nextest run",
    ] {
        assert_eq!(ledger_word(command, agent), None, "{command}");
    }
}

/// 断りの字は、読みの手と state dir の置き場と notes.md の場所を示す。
#[test]
fn agled_refusal_names_the_way_round() {
    assert_eq!(
        ledger_refusal("bd update", Path::new("/D/w218a"), Path::new("/D/w218a/w")),
        "係の門は止める（係の Bash で、台帳か器の state を書く語 bd update は撃たない） \
         次の一手 = 台帳は bd --readonly show か list で読み、preflight は scribe2 pipe preflight --state-dir /D/w218a/ の下の dir で撃ち、書きの要る事は /D/w218a/w/notes.md に書いて席に頼む"
    );
}

/// 台帳と器の state の検めは、組みによらず予算の前でも書きの語を止める。
#[test]
fn agled_gate_denies_ledger_writes_before_the_budget() {
    let light = place("light");
    bind(&light, "軽");
    let bash_in = |drafts: &Path, command: &str| guard(drafts, &input(SUB, "Bash", &bash(command)));
    let out = bash_in(&light, "bd update t3-hub.1 --notes x");
    denied(&out, &why("bd update"), "軽 の bd update");
    let out = bash_in(&light, "bdw show t3-hub.1");
    denied(&out, &why("bdw"), "軽 の bdw");
    let preflight = |agent: &str, state: &str| {
        format!(
            "scribe2 pipe preflight --contract {}/w218a/w/contract/a.toml --state-dir {}/{agent}/{state}",
            light.display(),
            light.display()
        )
    };
    let out = bash_in(&light, &preflight("w219a", "pf"));
    denied(&out, &why("scribe2 pipe"), "ほかの係の dir の state");
    for command in [
        "bd --readonly show t3-hub.1".to_string(),
        preflight("w218a", "pf-state"),
        "tz check --dir /T/design-intent".to_string(),
    ] {
        passed(&bash_in(&light, &command), &command);
    }
    let none = place("none");
    bind(&none, "なし");
    let out = bash_in(&none, "bd close t3-hub.1");
    denied(&out, &why("bd close"), "なし の bd close");
}

/// 係の id の無い席の呼びは、台帳の書きも器の口も通し、門は置き場に何も書かない。
#[test]
fn agled_calls_without_an_agent_id_pass() {
    let drafts = place("seat");
    for command in [
        "bd update t3-hub.1 --notes x",
        "bdw create x",
        "scribe2 pipe land --repo /R",
    ] {
        let out = guard(&drafts, &input("", "Bash", &bash(command)));
        passed(&out, command);
    }
    assert_eq!(fs::read_dir(&drafts).unwrap().count(), 0);
}
