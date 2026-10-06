//! 係の型と規律の手引きと門の設定の歯（接頭辞 agpl_・設計ノート surface-wave29b 行 ag-plugin・判断の記録 ADR-59 決定 (1)(2)(5)(6)・ADR-61 決定 (4)(9)・設計ノート surface-v4a 行 ag-designer・判断の記録 ADR-72 決定 (3)）。
//! workspace の根の plugin/ の file を読むだけで、tz も器も撃たない。hooks.json は stage の json で読み、口の名は境界の hook の
//! USAGE の字から、型と頭の行は中核の agent の spec の定数から組む。否定の見本は今の file から句を 1 つだけ崩して作る。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::hook::{agent_bind, agent_guard, agent_meter, agent_spawn, subagent_stop};
use tsuzuri_boundary::stage::json;
use tsuzuri_core::agent::spec::{HOLES, KEYS, TYPES, group};

/// 係の口の command の頭（runner の印の除き）と tzw の字。
const HEAD: &str =
    "[ -e \"$CLAUDE_PLUGIN_ROOT/scribe2-runner\" ] || \"$CLAUDE_PLUGIN_ROOT\"/bin/tzw hook ";

/// 係の口の command の末。
const TAIL: &str = " --repo \"$CLAUDE_PROJECT_DIR\"";

/// hooks.json で係の口を探す event の鍵。
const EVENTS: [&str; 7] = [
    "PostToolBatch",
    "PostToolUse",
    "PreToolUse",
    "Stop",
    "SubagentStart",
    "SubagentStop",
    "UserPromptSubmit",
];

/// 規律の手引きの置き場と名と、型が前置きする字。
const SKILL: &str = "plugin/skills/agent-discipline";
const SKILL_NAME: &str = "agent-discipline";
const PRELOAD: &str = "tsuzuri:agent-discipline";

/// 手引きの参照の file の名（本文が名指す）。
const REFERENCE: &str = "reference.md";

/// 手引きの本文の字数の上限（決定 (6)）。
const BODY_MAX: usize = 2000;

/// 説明の欄の末（引き金だけの 1 文）。
const TRIGGER_END: &str = "時に使う。";

/// 決まりの末の 3 つの形。
const CEILING: &str = "（天井だけ）";
const GATE: &str = "（門 ";
const VESSEL_GATE: &str = "（器の門 ";

/// 群の行の値の書き方。
const GROUP_HOLE: &str = "<群 id> <i>/<k>";

/// 設計係の型の file が字の中にちょうど 1 度ずつ持つ句の名と字。
const DESIGNER: [(&str, &str); 20] = [
    (
        "description",
        "description: tsuzuri の設計の席が、契約の行と節と判断の記録の下書きを設計だけで書かせる時に使う。",
    ),
    ("role", "あなたは tsuzuri の設計の席が起こした設計係である"),
    ("code-by-runner", "code と歯は便の実装役が書く"),
    ("input-code", "code の現物"),
    ("output-rows", "行と節の断片の file"),
    ("output-notes", "notes の file（notes.md）"),
    ("read-code", "1. code を読む。"),
    ("split", "2. 行を割る。"),
    ("fill", "3. 欄を埋める。"),
    ("floor", "4. 写しで床と preflight を撃ち"),
    ("coverage", "1. 要件の網羅。"),
    ("decided", "2. 何も決めない欄が無い。"),
    ("measured", "3. done の各項に歯か verify の行。"),
    ("agree", "4. 節と done の字の一致。"),
    ("by-value", "5. 環境や時刻に依る振る舞いは値で測る。"),
    ("done", "- DONE: "),
    ("concerns", "- DONE_WITH_CONCERNS: "),
    ("blocked", "- BLOCKED: "),
    ("context", "- NEEDS_CONTEXT: "),
    ("refused", "断りの字をそのまま席に知らせる"),
];

/// 設計係の型の file が持たない古い語の名と字（実装と道具の手順の語）。
const OLD_WORDS: [(&str, &str); 4] = [
    ("patch", "差の file"),
    ("bite", "噛み"),
    ("apply-script", "適用の script"),
    ("prototype", "試作"),
];

type Mouth =(String, Option<String>, String, Option<u64>);

/// 頭の --- の間の欄（鍵と値の列）。
type Fields = Vec<(String, Vec<String>)>;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// USAGE の字の tz hook の口の名。
fn mouth_name(usage: &str) -> &str {
    usage
        .strip_prefix("usage: tz hook ")
        .and_then(|rest| rest.split_whitespace().next())
        .expect("USAGE は usage と tz hook と口の名で始まる")
}

/// 境界の 5 つの係の口の名（起こしの門・結び・測り・係の門・終える前の門の順）。
fn gate_names() -> [&'static str; 5] {
    [
        agent_spawn::USAGE,
        agent_bind::USAGE,
        agent_meter::USAGE,
        agent_guard::USAGE,
        subagent_stop::USAGE,
    ]
    .map(mouth_name)
}

/// 係の口の結びの見込み（event・matcher・command・timeout）を並べ替えた列。
fn wanted() -> Vec<Mouth> {
    let [spawn, bind, meter, guard, stop] = gate_names();
    let spawn_matcher = format!("Agent|{}", group::WORKFLOW);
    let mut all: Vec<Mouth> = [
        ("PreToolUse", Some(spawn_matcher.as_str()), spawn, 30),
        ("PostToolUse", Some("Agent"), bind, 10),
        ("PostToolUse", None, meter, 10),
        ("PreToolUse", None, guard, 10),
        ("SubagentStop", None, stop, 120),
    ]
    .into_iter()
    .map(|(event, matcher, name, timeout)| {
        let command = format!("{HEAD}{name}{TAIL}");
        (
            event.to_owned(),
            matcher.map(str::to_owned),
            command,
            Some(timeout),
        )
    })
    .collect();
    all.sort();
    all
}

/// hooks.json の字の、command が字 tzw hook agent- を持つ hook の全部（event・matcher・command・timeout）を並べ替えた列。
fn agent_mouths(text: &str) -> Vec<Mouth> {
    let hooks = json::member(text, "hooks").expect("鍵 hooks");
    let mut all = Vec::new();
    for event in EVENTS {
        let entries = json::member(hooks, event)
            .and_then(json::items)
            .unwrap_or_default();
        for entry in entries {
            let matcher = json::member(entry, "matcher").and_then(json::unquote);
            let inner = json::member(entry, "hooks")
                .and_then(json::items)
                .unwrap_or_default();
            for hook in inner {
                let command = json::member(hook, "command")
                    .and_then(json::unquote)
                    .unwrap_or_default();
                if command.contains("tzw hook agent-") {
                    let timeout = json::member(hook, "timeout").and_then(|t| t.parse().ok());
                    all.push((event.to_owned(), matcher.clone(), command, timeout));
                }
            }
        }
    }
    all.sort();
    all
}

/// 頭の --- の間の欄（鍵と値の列・list の項は鍵ごとの値に足す）と本文。形が読めなければ None。
fn front(text: &str) -> Option<(Fields, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let (head, body) = rest.split_once("\n---\n")?;
    let mut fields: Fields = Vec::new();
    for line in head.lines() {
        if let Some(item) = line.strip_prefix("  - ") {
            fields.last_mut()?.1.push(item.to_owned());
        } else {
            let (key, value) = line.split_once(':')?;
            let value = value.trim();
            let values = if value.is_empty() {
                Vec::new()
            } else {
                vec![value.to_owned()]
            };
            fields.push((key.to_owned(), values));
        }
    }
    Some((fields, body))
}

/// 欄の値（無ければ空）。
fn field<'a>(fields: &'a [(String, Vec<String>)], key: &str) -> &'a [String] {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map_or(&[], |(_, v)| v.as_slice())
}

/// 説明の欄が引き金だけの 1 文か（句点が末の 1 つだけで、末が TRIGGER_END）。
fn trigger_only(values: &[String]) -> bool {
    matches!(values, [d] if d.ends_with(TRIGGER_END) && d.matches('。').count() == 1)
}

/// 型の file の欠け（name・description・model・skills の順）。
fn type_faults(stem: &str, text: &str) -> Vec<&'static str> {
    let Some((fields, _)) = front(text) else {
        return vec!["front"];
    };
    let mut faults = Vec::new();
    if field(&fields, "name") != [stem.to_owned()] {
        faults.push("name");
    }
    if !trigger_only(field(&fields, "description")) {
        faults.push("description");
    }
    if !matches!(field(&fields, "model"), [m] if m != "inherit") {
        faults.push("model");
    }
    if !field(&fields, "skills").iter().any(|s| s == PRELOAD) {
        faults.push("skills");
    }
    faults
}

/// 頭の 4 行の塊（KEYS と HOLES の順）。
fn head_block() -> String {
    KEYS.iter()
        .zip(HOLES)
        .map(|(k, h)| format!("{k}: {h}\n"))
        .collect()
}

/// 型の本文の頭の行の欠け（4 行の塊が 1 度だけ在るか・群の行が塊の直後に在るか、群でない型は群の鍵の行を持たないか）。
fn head_faults(body: &str, grouped: bool) -> Vec<&'static str> {
    let block = head_block();
    let group_line = format!("{}: {GROUP_HOLE}\n", group::KEY);
    let mut faults = Vec::new();
    if body.matches(&block).count() != 1 {
        faults.push("head");
    }
    let key = format!("{}:", group::KEY);
    let ok = if grouped {
        body.contains(&format!("{block}{group_line}"))
    } else {
        !body.lines().any(|l| l.starts_with(&key))
    };
    if !ok {
        faults.push("group");
    }
    faults
}

/// 手引きの file の欠け（name・description・本文の字数・参照の file の名指し）。
fn skill_faults(text: &str) -> Vec<&'static str> {
    let Some((fields, body)) = front(text) else {
        return vec!["front"];
    };
    let mut faults = Vec::new();
    if field(&fields, "name") != [SKILL_NAME.to_owned()] {
        faults.push("name");
    }
    if !trigger_only(field(&fields, "description")) {
        faults.push("description");
    }
    if body.chars().count() > BODY_MAX {
        faults.push("size");
    }
    if !body.contains(REFERENCE) {
        faults.push("reference");
    }
    faults
}

/// 設計係の型の file の欠け（字の数がちょうど 1 でない DESIGNER の句の名を表の順に、続けて字を持つ OLD_WORDS の語の名を表の順に）。
fn designer_faults(text: &str) -> Vec<&'static str> {
    let mut faults: Vec<&'static str> = DESIGNER
        .iter()
        .filter(|(_, words)| text.matches(words).count() != 1)
        .map(|(name, _)| *name)
        .collect();
    faults.extend(
        OLD_WORDS
            .iter()
            .filter(|(_, words)| text.contains(words))
            .map(|(name, _)| *name),
    );
    faults
}

/// 本文の決まり（字 - で始まる行）のうち、末が天井だけでも、門の名でも、在る器の門の名でもない行。
fn rule_faults(body: &str, gates: &[&str], vessel: impl Fn(&str) -> bool) -> Vec<String> {
    let ok = |rule: &str| {
        if rule.ends_with(CEILING) {
            return true;
        }
        let Some(open) = rule.strip_suffix('）') else {
            return false;
        };
        open.rsplit_once(VESSEL_GATE).map_or_else(
            || {
                open.rsplit_once(GATE)
                    .is_some_and(|(_, name)| gates.contains(&name))
            },
            |(_, name)| vessel(name),
        )
    };
    body.lines()
        .filter_map(|l| l.strip_prefix("- "))
        .filter(|rule| !ok(rule))
        .map(str::to_owned)
        .collect()
}

/// 決まりの末の形ごとの数（天井だけ・門・器の門）。
fn rule_kinds(body: &str) -> [usize; 3] {
    let rules: Vec<&str> = body.lines().filter_map(|l| l.strip_prefix("- ")).collect();
    let ends = |mark: &str| {
        rules
            .iter()
            .filter(|r| r.ends_with('）') && r.contains(mark))
            .count()
    };
    [ends(CEILING), ends(GATE), ends(VESSEL_GATE)]
}

/// 器の門の src の file が在るか。
fn vessel_gate(name: &str) -> bool {
    root()
        .join(format!("scribe2/crates/scribe2/src/hook/{name}.rs"))
        .is_file()
}

/// (1) hooks.json は係の口 5 つを、境界の USAGE の口の名で、起こしの門は matcher Agent と流れの道具、結びは matcher Agent、
/// 測りと係の門は matcher なし、終える前の門は SubagentStop で、runner の印の除きを頭に 1 度ずつ結ぶ。
#[test]
fn agpl_hooks_bind_the_five_agent_mouths_once() {
    let now = read("plugin/hooks/hooks.json");
    assert_eq!(agent_mouths(&now), wanted());
    let no_flow = now.replacen("\"Agent|Workflow\"", "\"Agent\"", 1);
    assert_ne!(
        agent_mouths(&no_flow),
        wanted(),
        "流れの道具の無い matcher の見本"
    );
    let renamed = now.replacen("hook agent-stop ", "hook agent-end ", 1);
    assert_ne!(agent_mouths(&renamed), wanted(), "口の名だけ替えた見本");
    let bare = now.replacen(
        "[ -e \\\"$CLAUDE_PLUGIN_ROOT/scribe2-runner\\\" ] || \\\"$CLAUDE_PLUGIN_ROOT\\\"/bin/tzw hook agent-guard",
        "\\\"$CLAUDE_PLUGIN_ROOT\\\"/bin/tzw hook agent-guard",
        1,
    );
    assert_ne!(bare, now, "印の除きの字");
    assert_ne!(agent_mouths(&bare), wanted(), "印の除きを外した見本");
    let slow = now.replacen(
        "agent-meter --repo \\\"$CLAUDE_PROJECT_DIR\\\"\",\n            \"timeout\": 10",
        "agent-meter --repo \\\"$CLAUDE_PROJECT_DIR\\\"\",\n            \"timeout\": 11",
        1,
    );
    assert_ne!(slow, now, "timeout の字");
    assert_ne!(agent_mouths(&slow), wanted(), "timeout だけ替えた見本");
}

/// (2) plugin/agents/ の file は中核の TYPES の 3 本（plugin の名と字 : の後の名に .md）だけで、どれも name が file の名、
/// 説明の欄が引き金だけの 1 文、model が inherit でなく、skills に規律の手引きを持つ。
#[test]
fn agpl_types_are_the_spec_types_with_the_discipline() {
    let mut stems: Vec<String> = fs::read_dir(root().join("plugin/agents"))
        .expect("plugin/agents を読む")
        .map(|e| e.expect("項").file_name().to_string_lossy().into_owned())
        .collect();
    stems.sort();
    let strip = |t: &str| t.strip_prefix("tsuzuri:").map(|s| format!("{s}.md"));
    let mut want: Vec<String> = TYPES.iter().filter_map(|t| strip(t)).collect();
    want.sort();
    assert_eq!((stems.len(), &stems), (TYPES.len(), &want));
    for name in &want {
        let text = read(&format!("plugin/agents/{name}"));
        assert!(
            type_faults(name.trim_end_matches(".md"), &text).is_empty(),
            "{name}"
        );
    }
    let now = read("plugin/agents/verifier.md");
    let bad = |from: &str, to: &str| type_faults("verifier", &now.replacen(from, to, 1));
    assert_eq!(bad("name: verifier", "name: checker"), ["name"]);
    assert_eq!(
        bad(TRIGGER_END, "時に使う。主張を照らす。"),
        ["description"]
    );
    assert_eq!(bad("model: opus", "model: inherit"), ["model"]);
    assert_eq!(
        bad(&format!("  - {PRELOAD}"), "  - tsuzuri:consult"),
        ["skills"]
    );
}

/// (3) 型の本文は頭の 4 行（中核の KEYS と HOLES の鍵と値の書き方）の塊を 1 度だけ持ち、検証役だけが塊の直後に群の行を持つ。
#[test]
fn agpl_type_bodies_carry_the_head_lines() {
    let body = |stem: &str| {
        let text = read(&format!("plugin/agents/{stem}.md"));
        front(&text).map(|(_, b)| b.to_owned()).expect("頭と本文")
    };
    for (stem, grouped) in [
        ("drafter", false),
        ("verifier", true),
        ("researcher", false),
    ] {
        assert!(head_faults(&body(stem), grouped).is_empty(), "{stem}");
    }
    let (block, line) = (head_block(), format!("{}: {GROUP_HOLE}\n", group::KEY));
    let drafter = body("drafter");
    let lack = drafter.replacen(&format!("{}: {}\n", KEYS[1], HOLES[1]), "", 1);
    assert_eq!(head_faults(&lack, false), ["head"]);
    let grouped = drafter.replacen(&block, &format!("{block}{line}"), 1);
    assert_eq!(head_faults(&grouped, false), ["group"]);
    assert_eq!(
        head_faults(&body("verifier").replacen(&line, "", 1), true),
        ["group"]
    );
}

/// (4) 規律の手引きは name が agent-discipline、説明の欄が引き金だけの 1 文、本文が 2,000 字以下で、同じ dir の参照の file を名指し、
/// その file が在る。
#[test]
fn agpl_skill_is_short_and_names_its_reference() {
    let now = read(&format!("{SKILL}/SKILL.md"));
    assert!(skill_faults(&now).is_empty(), "{:?}", skill_faults(&now));
    assert!(root().join(SKILL).join(REFERENCE).is_file(), "参照の file");
    let two = now.replacen(TRIGGER_END, "時に使う。席も読む。", 1);
    assert_eq!(skill_faults(&two), ["description"]);
    assert_eq!(
        skill_faults(&now.replacen(REFERENCE, "notes.md", 1)),
        ["reference"]
    );
    let (head, _) = now.split_at(now.find("\n---\n").expect("頭の末") + 5);
    let edge = format!(
        "{head}{REFERENCE}{}",
        "字".repeat(BODY_MAX - REFERENCE.len())
    );
    assert!(skill_faults(&edge).is_empty(), "2,000 字");
    assert_eq!(skill_faults(&format!("{edge}字")), ["size"], "2,001 字");
}

/// (5) 規律の手引きの本文の決まり 15 行は、どれも末が字（天井だけ）か、境界の係の口の名の（門 <名>）か、器の hook の src に
/// file の在る（器の門 <名>）で、数は天井だけ 7・門 6・器の門 2。
#[test]
fn agpl_skill_rules_end_with_a_gate_or_the_ceiling() {
    let text = read(&format!("{SKILL}/SKILL.md"));
    let (_, body) = front(&text).expect("頭と本文");
    let gates = gate_names();
    assert_eq!(rule_faults(body, &gates, vessel_gate), Vec::<String>::new());
    assert_eq!(body.lines().filter(|l| l.starts_with("- ")).count(), 15);
    assert_eq!(rule_kinds(body), [7, 6, 2]);
    let cut = |from: &str, to: &str| rule_faults(&body.replacen(from, to, 1), &gates, vessel_gate);
    let rule = "- 写しは起草の置き場の直下にだけ作る（器の門 drafts_guard）";
    assert!(body.contains(rule), "器の門の決まり");
    assert_eq!(
        cut("drafts_guard）", "drafts_ward）").len(),
        1,
        "無い器の門"
    );
    assert_eq!(
        cut("（門 agent-stop）", "（門 agent-end）").len(),
        1,
        "無い口の名"
    );
    assert_eq!(
        cut("で見る（天井だけ）", "で見る（天井だけ）。").len(),
        1,
        "末の句点"
    );
    assert_eq!(
        cut("（門 agent-guard）", "（agent-guard）").len(),
        1,
        "門の字の欠け"
    );
}

/// (6) 起草係の型の file は、頭の欄 effort が high の 1 つで、字の中に設計係の句の表の 20 句をどれもちょうど 1 度ずつ持ち、
/// 古い語の表の字を持たない。
#[test]
fn agpl_drafter_is_the_designer() {
    let text = read("plugin/agents/drafter.md");
    assert_eq!(designer_faults(&text), Vec::<&str>::new());
    let (fields, _) = front(&text).expect("頭と本文");
    assert_eq!(field(&fields, "effort"), ["high"]);
}

/// (7) 設計係の句を 1 つだけ外した見本と 2 度にした見本は、その句の名だけを名指し、古い語を file の末に 1 つ足した見本は、その語の
/// 名だけを名指す。
#[test]
fn agpl_drafter_clause_removed_doubled_or_old_word_added_is_named() {
    let text = read("plugin/agents/drafter.md");
    for (name, words) in DESIGNER {
        let removed = text.replacen(words, "", 1);
        assert_eq!(designer_faults(&removed), vec![name], "外した {name}");
        let doubled = text.replacen(words, &format!("{words}・{words}"), 1);
        assert_eq!(designer_faults(&doubled), vec![name], "重ねた {name}");
    }
    for (name, words) in OLD_WORDS {
        let added = format!("{text}\n- {words}\n");
        assert_eq!(designer_faults(&added), vec![name], "足した {name}");
    }
}
