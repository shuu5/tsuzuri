//! 係の口と係の門の形の畳みの歯（接頭辞 agshp_・設計ノート surface-wave29c 行 t-shape-ag・判断の記録 ADR-63 決定 (8) の (d)）。
//! 振る舞いは tz の binary の係の口 5 つに係の呼びの入力を標準入力で渡し、引数の誤りと置き場の解けない時の標準エラーの字を測る。
//! 形（deny の答えの JSON の字・係の出力の dir の名・path の下の判じ・係の口の引数の読み・係の門の置き場が 1 か所に在る事）は、
//! 写しが 2 つでも同じ字を出すので振る舞いでは測れず、中核と境界の src の字を関数の範囲（宣言の行から行の頭の閉じ括弧の行の前まで）で数える。
//! 否定の見本は本物の src から 1 句だけ崩した写しで、崩した句だけを名指す。
#![cfg(test)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use tsuzuri_boundary::hook::{agent_bind, agent_guard, agent_meter, agent_spawn, subagent_stop};

const GATE: &str = "tsuzuri-core/src/gate.rs";
const SPEC: &str = "tsuzuri-core/src/agent/spec.rs";
const STOP: &str = "tsuzuri-core/src/agent/stop.rs";
const AGENT_MOD: &str = "tsuzuri-core/src/agent/mod.rs";
const METER: &str = "tsuzuri-core/src/agent/meter.rs";
const GUARD: &str = "tsuzuri-core/src/agent/guard.rs";
const OLD_GUARD: &str = "tsuzuri-core/src/agent/meter/guard.rs";
const GROUP: &str = "tsuzuri-core/src/agent/meter/group.rs";
const HOOK: &str = "tsuzuri-boundary/src/hook/";
const ARGS: &str = "tsuzuri-boundary/src/hook/agent_args.rs";

/// deny の答えの JSON の鍵の字。
const DENY_KEY: &str = "\"permissionDecision\"";
/// gate.rs の deny_json の中の鍵の行と、ほかの関数 short_output の宣言の行（置き場だけ動かす見本）。
const KEY_LINE: &str = "        \"permissionDecision\".to_string(),\n";
const SHORT_HEAD: &str = "pub fn short_output(why: ShortWhy) -> String {\n";
/// 係の出力の dir の名の定義の頭と、その 1 行。
const OUT_DEF: &str = "const OUT: &str";
const OUT_LINE: &str = "pub const OUT: &str = \"w\";";
/// path の下の判じの `..` の成分の字。
const PARENT: &str = "Component::ParentDir";
/// 係の口の USAGE の定義の頭。
const AGENT_USAGE: &str = "pub const USAGE: &str = \"usage: tz hook agent-";
/// 置き場の解けない時の訳の字と、使い方の誤りの USAGE の行の書式の字。
const MISS: &str = "起草の置き場を解けない";
const USAGE_LINE: &str = "\\n{USAGE}";

/// 係の口の file（hook の dir の下）。
const MOUTHS: [&str; 5] = [
    "agent_bind.rs",
    "agent_guard.rs",
    "agent_meter.rs",
    "agent_spawn.rs",
    "subagent_stop.rs",
];

/// 中核と境界の src の下の .rs の全部（`<crate>/src/<相対>` から字へ）。
fn sources() -> BTreeMap<String, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates の dir");
    let mut out = BTreeMap::new();
    for krate in ["tsuzuri-core", "tsuzuri-boundary"] {
        walk(
            &root.join(krate).join("src"),
            &format!("{krate}/src"),
            &mut out,
        );
    }
    out
}

fn walk(dir: &Path, rel: &str, out: &mut BTreeMap<String, String>) {
    for entry in fs::read_dir(dir).expect("src の dir を読む").flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            walk(&path, &format!("{rel}/{name}"), out);
        } else if name.ends_with(".rs") {
            let text = fs::read_to_string(&path).expect("src の file を読む");
            out.insert(format!("{rel}/{name}"), text);
        }
    }
}

/// 関数 `name` の範囲（宣言 `fn <name>(` の行から、行の頭の閉じ括弧の行の前まで・無ければ空）。
fn body<'a>(files: &'a BTreeMap<String, String>, file: &str, name: &str) -> &'a str {
    let Some(src) = files.get(file) else {
        return "";
    };
    let Some(at) = src.find(&format!("fn {name}(")) else {
        return "";
    };
    let start = src[..at].rfind('\n').map_or(0, |i| i + 1);
    let end = src[start..].find("\n}\n").map_or(src.len(), |i| start + i);
    &src[start..end]
}

/// 全部の file の中の字 `needle` の数。
fn total(files: &BTreeMap<String, String>, needle: &str) -> usize {
    files.values().map(|t| t.matches(needle).count()).sum()
}

/// deny の鍵の字の全部の数と、gate.rs の deny_json の範囲の中の数。
fn deny_shape(files: &BTreeMap<String, String>) -> (usize, usize) {
    let inner = body(files, GATE, "deny_json").matches(DENY_KEY).count();
    (total(files, DENY_KEY), inner)
}

/// 出力の dir の名の定義の全部の数と、spec.rs の定義の 1 行の数。
fn out_shape(files: &BTreeMap<String, String>) -> (usize, usize) {
    let spec = files.get(SPEC).map_or(0, |t| t.matches(OUT_LINE).count());
    (total(files, OUT_DEF), spec)
}

/// `..` の成分の字の全部の数・guard.rs の under の中の数・open の中の字 under( の数・group.rs の inside の中の字 under( の数。
fn under_shape(files: &BTreeMap<String, String>) -> [usize; 4] {
    [
        total(files, PARENT),
        body(files, GUARD, "under").matches(PARENT).count(),
        body(files, GUARD, "open").matches("under(").count(),
        body(files, GROUP, "inside").matches("under(").count(),
    ]
}

/// 係の口の file の名と run の中の字 begin( の数の列・口の file の中の訳の字と USAGE の行の書式の字の数の和・
/// agent_args.rs の begin の中の字 parse(rest)・drafts(&args)・訳の字の数。
fn begin_shape(files: &BTreeMap<String, String>) -> (Vec<(String, usize)>, usize, [usize; 3]) {
    let mut mouths = Vec::new();
    let mut copies = 0;
    for (path, text) in files {
        let Some(name) = path.strip_prefix(HOOK) else {
            continue;
        };
        if !text.contains(AGENT_USAGE) {
            continue;
        }
        let run = body(files, path, "run").matches("begin(").count();
        mouths.push((name.to_string(), run));
        copies += text.matches(MISS).count() + text.matches(USAGE_LINE).count();
    }
    let begin = body(files, ARGS, "begin");
    let reader = [
        begin.matches("parse(rest)").count(),
        begin.matches("drafts(&args)").count(),
        begin.matches(MISS).count(),
    ];
    (mouths, copies, reader)
}

/// agent/guard.rs の在り・agent/meter/guard.rs の在り・agent/mod.rs の字 pub mod guard; の数・meter.rs の字 mod guard; の数。
fn guard_shape(files: &BTreeMap<String, String>) -> (bool, bool, usize, usize) {
    let count = |f: &str, s: &str| files.get(f).map_or(0, |t| t.matches(s).count());
    (
        files.contains_key(GUARD),
        files.contains_key(OLD_GUARD),
        count(AGENT_MOD, "pub mod guard;"),
        count(METER, "mod guard;"),
    )
}

/// 口の file の名と run の中の begin( の数の見込み（5 つの口が 1 度ずつ）。
fn mouths_once() -> Vec<(String, usize)> {
    MOUTHS.iter().map(|m| ((*m).to_string(), 1)).collect()
}

/// 崩れた句の名（deny・out・under・begin・guard の順・崩れが無ければ空）。
fn broken(files: &BTreeMap<String, String>) -> Vec<&'static str> {
    let ok = [
        ("deny", deny_shape(files) == (1, 1)),
        ("out", out_shape(files) == (1, 1)),
        ("under", under_shape(files) == [1, 1, 1, 1]),
        ("begin", begin_shape(files) == (mouths_once(), 0, [1, 1, 1])),
        ("guard", guard_shape(files) == (true, false, 1, 0)),
    ];
    ok.iter().filter(|(_, ok)| !ok).map(|(n, _)| *n).collect()
}

/// 係の口を入力 `payload` と引数 `args` で撃ち、(rc, 標準出力, 標準エラー) を返す。
fn mouth(name: &str, args: &[&str], payload: &str) -> (Option<i32>, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["hook", name])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tz を撃つ");
    child
        .stdin
        .take()
        .expect("標準入力")
        .write_all(payload.as_bytes())
        .expect("入力を書く");
    let out = child.wait_with_output().expect("終わりを待つ");
    let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
    (out.status.code(), text(&out.stdout), text(&out.stderr))
}

#[test]
fn agshp_five_mouths_keep_the_usage_and_the_drafts_lines() {
    let seat = r#"{"tool_name":"Agent","tool_input":{"subagent_type":"tsuzuri:drafter","name":"w1","prompt":"p"}}"#;
    let bound =
        r#"{"tool_name":"Agent","tool_input":{"name":"w1"},"tool_response":{"agentId":"a1"}}"#;
    let sub = r#"{"agent_id":"a1","hook_event_name":"PreToolUse","tool_name":"Read"}"#;
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("agshp");
    fs::create_dir_all(&root).expect("歯の置き場");
    let none = root.join("none");
    let none = none.to_str().expect("path の字");
    let repo = root.to_str().expect("path の字");
    for (name, usage, payload, miss) in [
        ("agent-spawn", agent_spawn::USAGE, seat, "通す"),
        ("agent-bind", agent_bind::USAGE, bound, "書かない"),
        ("agent-meter", agent_meter::USAGE, sub, "通す"),
        ("agent-guard", agent_guard::USAGE, sub, "通す"),
        ("agent-stop", subagent_stop::USAGE, sub, "通す"),
    ] {
        let wrong = mouth(name, &["--repo", repo, "--bogus", "x"], payload);
        let line = format!("tz hook {name}: 知らない引数 --bogus\n{usage}\n");
        assert_eq!(wrong, (Some(1), String::new(), line), "{name}");
        let lost = mouth(name, &["--repo", repo, "--drafts", none], payload);
        let line = format!("tz hook {name}: {MISS}（{miss}）\n");
        assert_eq!(lost, (Some(0), String::new(), line), "{name}");
    }
}

#[test]
fn agshp_deny_answer_is_built_in_one_place() {
    assert_eq!(deny_shape(&sources()), (1, 1));
}

#[test]
fn agshp_out_dir_name_is_defined_once() {
    assert_eq!(out_shape(&sources()), (1, 1));
}

#[test]
fn agshp_path_under_is_judged_in_one_place() {
    assert_eq!(under_shape(&sources()), [1, 1, 1, 1]);
}

#[test]
fn agshp_agent_mouths_begin_through_one_reader() {
    assert_eq!(begin_shape(&sources()), (mouths_once(), 0, [1, 1, 1]));
}

#[test]
fn agshp_guard_sits_beside_meter_spec_and_stop() {
    assert_eq!(guard_shape(&sources()), (true, false, 1, 0));
}

/// 本物の src の写しの file `file` の、ちょうど 1 度在る字 `old` を `new` に替える。
fn swap(files: &mut BTreeMap<String, String>, file: &str, old: &str, new: &str) {
    let text = files.get_mut(file).expect("写しの file");
    assert_eq!(text.matches(old).count(), 1, "{file} の {old}");
    *text = text.replacen(old, new, 1);
}

/// 本物の src の写しへの 1 句の直し。
type Edit = fn(&mut BTreeMap<String, String>);

/// deny と出力の dir の名と path の下の判じの句を 1 つだけ崩す見本（句の名と直し）。
fn core_mutants() -> [(&'static str, Edit); 6] {
    [
        ("deny", |f| {
            let copy = "\nfn deny_copy() -> &'static str {\n    \"permissionDecision\"\n}\n";
            f.get_mut(SPEC).expect("spec").push_str(copy);
        }),
        ("deny", |f| {
            swap(f, GATE, KEY_LINE, "");
            swap(f, GATE, SHORT_HEAD, &format!("{SHORT_HEAD}{KEY_LINE}"));
        }),
        ("out", |f| {
            swap(
                f,
                STOP,
                "\npub mod floor;\n",
                &format!("\npub mod floor;\n{OUT_LINE}\n"),
            );
        }),
        ("out", |f| {
            swap(f, SPEC, OUT_LINE, "");
            swap(
                f,
                AGENT_MOD,
                "pub mod guard;\n",
                &format!("pub mod guard;\n{OUT_LINE}\n"),
            );
        }),
        ("under", |f| {
            let inline = "!Path::new(path).components().any(|c| c == Component::ParentDir) && Path::new(path).starts_with(a)";
            swap(f, GROUP, "under(Path::new(path), Path::new(a))", inline);
        }),
        ("under", |f| {
            let old = "path.is_some_and(|p| under(Path::new(p), out))";
            swap(
                f,
                GUARD,
                old,
                "path.is_some_and(|p| Path::new(p).starts_with(out))",
            );
        }),
    ]
}

/// 係の口の引数の読みと係の門の置き場の句を 1 つだけ崩す見本（句の名と直し）。
fn mouth_mutants() -> [(&'static str, Edit); 4] {
    [
        ("begin", |f| {
            let text = "pub const USAGE: &str = \"usage: tz hook agent-extra --repo <dir>\";\n\
                        pub fn run(rest: &[&str]) -> u8 {\n    parse(rest).map_or(1, |_| 0)\n}\n";
            f.insert(format!("{HOOK}agent_extra.rs"), text.to_string());
        }),
        ("begin", |f| {
            let new = "Err(rc) => {\n            emit_err(\"tz hook agent-meter: 起草の置き場を解けない（通す）\");\n            return rc;\n        }";
            swap(
                f,
                &format!("{HOOK}agent_meter.rs"),
                "Err(rc) => return rc,",
                new,
            );
        }),
        ("guard", |f| {
            swap(f, AGENT_MOD, "pub mod guard;\n", "");
            swap(
                f,
                METER,
                "pub mod group;\n",
                "pub mod group;\npub mod guard;\n",
            );
        }),
        ("guard", |f| {
            f.insert(OLD_GUARD.to_string(), "//! 写し\n".to_string());
        }),
    ]
}

#[test]
fn agshp_one_clause_mutants_are_named() {
    let real = sources();
    assert!(broken(&real).is_empty(), "{:?}", broken(&real));
    for (i, (clause, edit)) in core_mutants()
        .into_iter()
        .chain(mouth_mutants())
        .enumerate()
    {
        let mut files = real.clone();
        edit(&mut files);
        assert_eq!(broken(&files), vec![clause], "{i} 番の見本");
    }
}
