//! 席の手元へ要の写しを出した暫定の hook を外した後の守りの歯（接頭辞 seathk_・行 t-hook-drop・判断の記録 ADR-38 決定 (6)(7)）。
//! workspace の根の plugin/hooks/hooks.json を JSON として読み（境界の crate は serde_json に直に依存しないので中核の crate に置く）、
//! event の鍵に SessionStart が無く、どの command も要の写しを出す語を持たないことと、runner の印の除きを持つ event の全部の command の
//! 頭の印の名が器の名の定数から成ることを見る（係の口の command も印を持つ・行 ag-plugin）。否定の見本は今の hooks.json から句を 1 つだけ崩して作る。
#![cfg(test)]

use crate::common::read;
use serde_json::Value;

/// runner の印の除きの字（command の頭・印の名は器の名の定数と尾の字から成る）。
const EXCLUDE_HEAD: &str = "[ -e \"$CLAUDE_PLUGIN_ROOT/";

/// 印の名の尾（器の pipe/spawn.rs の定数の字）。
const MARK_SUFFIX: &str = "-runner";

/// 器の名の定数の行の頭と、印の名の尾の定数の行（器の src の字）。
const NAME_LINE: &str = "pub const NAME: &str = \"";
const SUFFIX_LINE: &str = "const RUNNER_MARK_SUFFIX: &str = \"-runner\";";

/// hooks.json の event の鍵（字の順）。
const EVENTS: [&str; 6] = [
    "PostToolBatch",
    "PostToolUse",
    "PreToolUse",
    "Stop",
    "SubagentStop",
    "UserPromptSubmit",
];

/// runner の印の除きを持つ command の event（Stop は除きを持たない）。
const MARKED: [&str; 5] = [
    "PostToolBatch",
    "PostToolUse",
    "PreToolUse",
    "SubagentStop",
    "UserPromptSubmit",
];

/// 外した暫定の hook の event の鍵。
const DROPPED: &str = "SessionStart";

/// 要の写しを出す hook の語（器の binary への問いの語と、要の写しの置き場の字）。
const COPY_WORDS: [&str; 2] = ["seat-constitution", "contracts/seat/"];

fn hooks() -> Value {
    serde_json::from_str(&read("plugin/hooks/hooks.json"))
        .unwrap_or_else(|e| panic!("hooks.json は JSON でない: {e}"))
}

/// event の鍵（字の順）。
fn events(hooks: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = hooks["hooks"]
        .as_object()
        .unwrap_or_else(|| panic!("hooks は object でない: {hooks}"))
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

/// event の全部の要素の全部の hook の command の字（要素の順）。
fn commands<'a>(hooks: &'a Value, event: &str) -> Vec<&'a str> {
    let all = |v: &'a Value| v.as_array().map_or(&[][..], Vec::as_slice);
    let list: Vec<&str> = all(&hooks["hooks"][event])
        .iter()
        .flat_map(|entry| all(&entry["hooks"]))
        .map(|hook| hook["command"].as_str().unwrap_or_default())
        .collect();
    assert!(!list.is_empty(), "{event} に command が無い");
    list
}

/// 見本を作るために、event の最初の要素の最初の command の字を `edit` で書き替えた写し。
fn edited(hooks: &Value, event: &str, edit: impl Fn(&str) -> String) -> Value {
    let mut copy = hooks.clone();
    let text = edit(commands(hooks, event)[0]);
    copy["hooks"][event][0]["hooks"][0]["command"] = Value::String(text);
    copy
}

/// 見本を作るために、event の最後の要素の最初の command の字を `edit` で書き替えた写し。
fn edited_last(hooks: &Value, event: &str, edit: impl Fn(&str) -> String) -> Value {
    let mut copy = hooks.clone();
    let last = hooks["hooks"][event].as_array().map_or(0, Vec::len).saturating_sub(1);
    let text = edit(hooks["hooks"][event][last]["hooks"][0]["command"].as_str().unwrap_or_default());
    copy["hooks"][event][last]["hooks"][0]["command"] = Value::String(text);
    copy
}

/// 要の写しを出す語を持つ command の event（字の順）。
fn copying(hooks: &Value) -> Vec<String> {
    events(hooks)
        .into_iter()
        .filter(|event| commands(hooks, event).iter().any(|c| COPY_WORDS.iter().any(|w| c.contains(w))))
        .map(str::to_owned)
        .collect()
}

/// 器の名の定数の字（器の src/name.rs の 1 行から）。
fn vessel_name() -> String {
    let src = read("scribe2/crates/scribe2/src/name.rs");
    let line = src
        .lines()
        .find(|l| l.starts_with(NAME_LINE))
        .expect("器の名の定数の行");
    line.strip_prefix(NAME_LINE)
        .and_then(|rest| rest.strip_suffix("\";"))
        .expect("器の名の定数の字")
        .to_owned()
}

/// 印の名（器の名の定数の字と、器の src/pipe/spawn.rs の尾の定数の字をつないだ字）。
fn mark_name() -> String {
    let spawn = read("scribe2/crates/scribe2/src/pipe/spawn.rs");
    assert_eq!(
        spawn.lines().filter(|l| *l == SUFFIX_LINE).count(),
        1,
        "器の印の名の尾は {MARK_SUFFIX}"
    );
    format!("{}{MARK_SUFFIX}", vessel_name())
}

/// 印の除きの頭が `mark` の名で始まらないか、除きを 1 度だけ持たない command を持つ event（MARKED の順）。
fn unmarked(hooks: &Value, mark: &str) -> Vec<String> {
    let head = format!("{EXCLUDE_HEAD}{mark}\" ] || ");
    MARKED
        .iter()
        .filter(|event| {
            commands(hooks, event)
                .iter()
                .any(|text| !text.starts_with(&head) || text.matches(EXCLUDE_HEAD).count() != 1)
        })
        .map(|event| (*event).to_owned())
        .collect()
}

#[test]
fn seathk_session_start_is_gone() {
    let now = hooks();
    assert_eq!(
        events(&now),
        EVENTS,
        "event の鍵は 6 つで SessionStart が無い"
    );
    assert!(
        copying(&now).is_empty(),
        "要の写しを出す語を持つ command が無い"
    );
    let mut back = now.clone();
    back["hooks"][DROPPED] = now["hooks"]["Stop"].clone();
    assert_ne!(events(&back), EVENTS, "SessionStart を戻した見本");
    let cat = edited(&now, "UserPromptSubmit", |c| {
        format!("{c} || cat \"$CLAUDE_PROJECT_DIR/contracts/seat/brief.txt\"")
    });
    assert_eq!(events(&cat), EVENTS, "鍵は替えない見本");
    assert_eq!(copying(&cat), ["UserPromptSubmit"], "要の写しの置き場の字");
    let ask = edited(&now, "PreToolUse", |c| {
        format!("{c} || scribe2 vessel seat-constitution")
    });
    assert_eq!(copying(&ask), ["PreToolUse"], "器への問いの語");
}

#[test]
fn seathk_marked_heads_follow_the_vessel_name() {
    let now = hooks();
    let mark = mark_name();
    assert_eq!(
        unmarked(&now, &mark),
        Vec::<String>::new(),
        "印を持つ event の全部の command の頭が印の除き"
    );
    let other = edited(&now, "PreToolUse", |c| c.replacen(&mark, "other-runner", 1));
    assert_eq!(
        unmarked(&other, &mark),
        ["PreToolUse"],
        "印の名だけ替えた見本"
    );
    let bare = edited(&now, "PostToolUse", |c| {
        c.split_once(" ] || ")
            .map(|(_, rest)| rest.to_owned())
            .expect("除きの後")
    });
    assert_eq!(unmarked(&bare, &mark), ["PostToolUse"], "除きを外した見本");
    let gate = edited_last(&now, "PreToolUse", |c| c.replacen(&mark, "other-runner", 1));
    assert_eq!(unmarked(&gate, &mark), ["PreToolUse"], "最後の要素の印の名だけ替えた見本");
    let twice = edited(&now, "UserPromptSubmit", |c| {
        format!("{EXCLUDE_HEAD}{mark}\" ] || {c}")
    });
    assert_eq!(
        unmarked(&twice, &mark),
        ["UserPromptSubmit"],
        "除きを 2 度持つ見本"
    );
}
