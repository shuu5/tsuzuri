//! 起動の口の argv と設定の歯（接頭辞 cwarg_・設計ノート surface-wave27a 行 cs-argv・受入 AC19 の argv と設定）。
//! fixture: tests/fixtures/consult/launch-argv.yaml（JSON の字・置き字 /W /R /S /T と uid の U・host の path を書かない）。
//! 組みが fixture と同じこと、plugin の置き場の旗を足した形と囲いの設定を 1 つ欠いた形と広い許しを足した形を検めが断ることを見る。
//! Claude の口座の資格の file を囲いと読む道具の両方から隠し、どちらを欠いても検めが断ることを見る（行 cs-cred-claude）。
#![cfg(test)]

use std::path::Path;

use serde_json::{Value, json};
use tsuzuri_contract::consult::{DRAFT_FIELDS, Form, WindowId};
use tsuzuri_core::consult::launch::{
    ASK_DROP, CREDENTIALS, Launch, PLUGIN_VERSION, TALK_ENV, argv, audit, brief, env, private_tmp,
    prompt, read_roots, settings, version_ok,
};

fn fixture() -> Value {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/consult/launch-argv.yaml");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
    serde_json::from_str(&text).expect("JSON の字")
}

fn launch(inputs: &Value) -> Launch {
    let s = |k: &str| inputs[k].as_str().expect(k).to_string();
    Launch {
        form: if s("form") == "ask" {
            Form::Ask
        } else {
            Form::Talk
        },
        window: WindowId::parse(&s("window")).expect("窓 id"),
        workspace: s("workspace"),
        repo: s("repo"),
        state: s("state"),
        roots: inputs["roots"]
            .as_array()
            .expect("roots")
            .iter()
            .map(|r| r.as_str().expect("root").to_string())
            .collect(),
        tz: s("tz"),
        uid: s("uid"),
        model: s("model"),
        effort: s("effort"),
        question: inputs["question"].as_bool().expect("question"),
    }
}

/// argv の `--settings` の次の字を設定の値に読み、その字を `<settings>` に替える。
fn split_settings(mut a: Vec<String>) -> (Vec<String>, Value) {
    let at = a
        .iter()
        .position(|x| x == "--settings")
        .expect("--settings")
        + 1;
    let set = serde_json::from_str(&a[at]).expect("設定の JSON");
    a[at] = "<settings>".into();
    (a, set)
}

/// 設定を `edit` で替えた argv。
fn with_settings(l: &Launch, edit: impl Fn(&mut Value)) -> Vec<String> {
    let mut a = argv(l);
    let at = a
        .iter()
        .position(|x| x == "--settings")
        .expect("--settings")
        + 1;
    let mut set = settings(l);
    edit(&mut set);
    a[at] = set.to_string();
    a
}

#[test]
fn cwarg_matches_fixture() {
    let fx = fixture();
    for form in ["talk", "ask"] {
        let l = launch(&fx[form]["inputs"]);
        let a = argv(&l);
        assert!(audit(&a, &l).is_empty(), "{form}: {:?}", audit(&a, &l));
        let (rest, set) = split_settings(a);
        let want: Vec<String> = serde_json::from_value(fx[form]["argv"].clone()).expect("argv");
        assert_eq!(rest, want, "{form} の argv");
        assert_eq!(set, fx["settings"], "{form} の設定");
        let got: Vec<Value> = env(&l).iter().map(|(k, v)| json!([k, v])).collect();
        assert_eq!(Value::from(got), fx[form]["env"], "{form} の環境");
    }
    assert_eq!(Value::from(TALK_ENV.to_vec()), fx["talk_env"]);
    assert_eq!(Value::from(ASK_DROP.to_vec()), fx["ask_drop"]);
    assert_eq!(read_roots("/R", "/S"), ["/R", "/S/fleet", "/S/pipe"]);
}

#[test]
fn cwarg_audit_refuses_flags() {
    let fx = fixture();
    let talk = launch(&fx["talk"]["inputs"]);
    let ask = launch(&fx["ask"]["inputs"]);
    let add = |l: &Launch, extra: &[&str]| {
        let mut a = argv(l);
        a.extend(extra.iter().map(|x| x.to_string()));
        audit(&a, l)
    };
    for extra in [
        &["--plugin-dir", "/p"][..],
        &["--plugin-dir=/p"],
        &["--dangerously-skip-permissions"],
        &["--mcp-config", "/m.json"],
        &["--append-system-prompt", "x"],
        &["--add-dir", "/S"],
        &["--add-dir", "/elsewhere"],
    ] {
        assert!(!add(&talk, extra).is_empty(), "talk {extra:?}");
        assert!(!add(&ask, extra).is_empty(), "ask {extra:?}");
    }
    let swap = |l: &Launch, from: &str, to: &str| {
        let a: Vec<String> = argv(l)
            .into_iter()
            .map(|x| if x == from { to.to_string() } else { x })
            .collect();
        audit(&a, l)
    };
    for (from, to) in [
        ("dontAsk", "bypassPermissions"),
        ("dontAsk", "acceptEdits"),
        ("--restricted", "--verbose"),
        ("--strict-mcp-config", "--verbose"),
        ("mcp__*", "mcp__x"),
        (
            "Read,Grep,Glob,Bash,Edit,Write,WebSearch,WebFetch",
            "Read,Bash,Task",
        ),
        ("consult-cw3", "consult-cw9"),
        ("-p", "--print-x"),
        ("--output-format", "--verbose"),
    ] {
        let l = if ["-p", "--output-format"].contains(&from) {
            &ask
        } else {
            &talk
        };
        assert!(!swap(l, from, to).is_empty(), "{from} → {to}");
    }
    let mut talk_p = argv(&talk);
    talk_p.insert(0, "-p".into());
    assert!(!audit(&talk_p, &talk).is_empty());
    let no_settings: Vec<String> = split_settings(argv(&talk)).0;
    assert!(!audit(&no_settings, &talk).is_empty());
}

#[test]
fn cwarg_audit_refuses_each_missing_key() {
    let fx = fixture();
    let l = launch(&fx["talk"]["inputs"]);
    let leaves = [
        "/sandbox/enabled",
        "/sandbox/autoAllowBashIfSandboxed",
        "/sandbox/allowUnsandboxedCommands",
        "/sandbox/failIfUnavailable",
        "/sandbox/filesystem",
        "/sandbox/credentials",
        "/sandbox/network",
        "/permissions/allow",
        "/permissions/deny",
        "/env",
        "/hooks",
    ];
    for pointer in leaves {
        let (parent, key) = pointer.rsplit_once('/').expect("pointer");
        let a = with_settings(&l, |set| {
            let p = if parent.is_empty() {
                &mut *set
            } else {
                set.pointer_mut(parent).expect("親")
            };
            p.as_object_mut().expect("object").remove(key);
        });
        assert!(!audit(&a, &l).is_empty(), "{pointer} を除いた");
    }
    for (list, n) in [
        ("/sandbox/credentials/files", 9),
        ("/sandbox/filesystem/denyWrite", 3),
        ("/permissions/deny", 3),
    ] {
        for i in 0..n {
            let a = with_settings(&l, |set| {
                set.pointer_mut(list)
                    .and_then(Value::as_array_mut)
                    .expect("列")
                    .remove(i);
            });
            assert!(!audit(&a, &l).is_empty(), "{list} の {i} を除いた");
        }
    }
    for key in [
        "UV_CACHE_DIR",
        "PIP_CACHE_DIR",
        "CARGO_HOME",
        "npm_config_cache",
    ] {
        let a = with_settings(&l, |set| {
            set["env"].as_object_mut().expect("env").remove(key);
        });
        assert!(!audit(&a, &l).is_empty(), "env の {key} を除いた");
    }
}

/// 設定の替えの名と替え方。
type Edit = (&'static str, fn(&mut Value));

fn push(set: &mut Value, pointer: &str, item: Value) {
    set.pointer_mut(pointer)
        .and_then(Value::as_array_mut)
        .expect("列")
        .push(item);
}

fn refused(edits: &[Edit]) {
    let fx = fixture();
    let l = launch(&fx["talk"]["inputs"]);
    for (name, edit) in edits {
        let a = with_settings(&l, edit);
        assert!(!audit(&a, &l).is_empty(), "{name}");
    }
}

#[test]
fn cwarg_audit_refuses_wide_permissions() {
    refused(&[
        ("WebFetch(domain:*)", |s| {
            push(s, "/permissions/allow", json!("WebFetch(domain:*)"))
        }),
        ("Edit(//R/**)", |s| {
            push(s, "/permissions/allow", json!("Edit(//R/**)"))
        }),
        ("Bash(podman run *)", |s| {
            push(s, "/permissions/allow", json!("Bash(podman run *)"))
        }),
        ("defaultMode", |s| {
            s["permissions"]["defaultMode"] = json!("bypassPermissions")
        }),
        ("domain *", |s| {
            push(s, "/sandbox/network/allowedDomains", json!("*"))
        }),
        ("github.com", |s| {
            push(s, "/sandbox/network/allowedDomains", json!("github.com"))
        }),
    ]);
}

#[test]
fn cwarg_audit_refuses_wide_sandbox() {
    refused(&[
        ("excludedCommands", |s| {
            s["sandbox"]["excludedCommands"] = json!(["podman *"])
        }),
        ("allowUnixSockets", |s| {
            s["sandbox"]["network"]["allowUnixSockets"] = json!(["/tmp/x.sock"])
        }),
        ("allowAllUnixSockets", |s| {
            s["sandbox"]["network"]["allowAllUnixSockets"] = json!(true)
        }),
        ("allowUnsandboxedCommands", |s| {
            s["sandbox"]["allowUnsandboxedCommands"] = json!(true)
        }),
        ("enabled", |s| s["sandbox"]["enabled"] = json!(false)),
        ("CARGO_HOME の外", |s| {
            s["env"]["CARGO_HOME"] = json!("/home/x/.cargo")
        }),
        ("hook の命令", |s| {
            s["hooks"]["PreToolUse"][0]["hooks"][0]["command"] = json!("/T consult guard")
        }),
        ("hook の matcher", |s| {
            s["hooks"]["PreToolUse"][0]["matcher"] = json!("Bash")
        }),
        ("credentials の allow", |s| {
            s["sandbox"]["credentials"]["files"][1]["mode"] = json!("allow")
        }),
    ]);
}

#[test]
fn cwarg_hides_the_claude_credentials() {
    let fx = fixture();
    let file = "~/.claude/.credentials.json";
    let rule = "Read(~/.claude/.credentials.json)";
    assert_eq!(CREDENTIALS.last(), Some(&file));
    for form in ["talk", "ask"] {
        let l = launch(&fx[form]["inputs"]);
        let set = settings(&l);
        let files = set["sandbox"]["credentials"]["files"]
            .as_array()
            .expect("列");
        let hidden = |x: &&Value| x["path"] == json!(file);
        assert_eq!(
            files.iter().filter(hidden).collect::<Vec<_>>(),
            [&json!({"path": file, "mode": "deny"})],
            "{form}"
        );
        let deny = set["permissions"]["deny"].as_array().expect("列");
        assert_eq!(
            deny.iter().filter(|r| **r == json!(rule)).count(),
            1,
            "{form}"
        );
        let gaps = |edit: &dyn Fn(&mut Value)| audit(&with_settings(&l, edit), &l);
        let drop = |pointer: &'static str, want: &'static str| {
            move |s: &mut Value| {
                s.pointer_mut(pointer)
                    .and_then(Value::as_array_mut)
                    .expect("列")
                    .retain(|x| x != &json!(want) && x["path"] != json!(want));
            }
        };
        let files_gap = ["/sandbox/credentials/files".to_string()];
        let deny_gap = ["/permissions/deny".to_string()];
        assert_eq!(gaps(&drop("/sandbox/credentials/files", file)), files_gap);
        assert_eq!(gaps(&drop("/permissions/deny", rule)), deny_gap);
        let allow = |s: &mut Value| {
            let xs = s["sandbox"]["credentials"]["files"]
                .as_array_mut()
                .expect("列");
            xs.iter_mut()
                .filter(|x| x["path"] == json!(file))
                .for_each(|x| x["mode"] = json!("allow"));
        };
        assert_eq!(gaps(&allow), files_gap, "{form} の allow");
        let wide = |s: &mut Value| {
            let xs = s["permissions"]["deny"].as_array_mut().expect("列");
            xs.iter_mut()
                .filter(|r| **r == json!(rule))
                .for_each(|r| *r = json!("Read(~/.claude/**/x)"));
        };
        assert_eq!(gaps(&wide), deny_gap, "{form} の別の字");
    }
}

#[test]
fn cwarg_tmp_and_version() {
    assert_eq!(private_tmp("/W"), "/tmp/tzc-07d671");
    let long = format!(
        "/{}/seat/x_orchestrator/drafts/consult-cw12",
        "a".repeat(120)
    );
    let t = private_tmp(&long);
    assert!(t.len() <= 30 && t.starts_with("/tmp/tzc-"), "{t}");
    assert_ne!(private_tmp("/W/a"), private_tmp("/W/b"));
    assert_eq!(PLUGIN_VERSION, "0.2.0");
    assert!(version_ok(Some("0.2.0")));
    assert!(!version_ok(Some("0.1.0")));
    assert!(!version_ok(None));
    let fx = fixture();
    let mut talk = launch(&fx["talk"]["inputs"]);
    talk.question = true;
    assert!(prompt(&talk).contains("bundle/question.md"));
    assert!(
        argv(&talk)
            .last()
            .is_some_and(|p| p.contains("bundle/question.md"))
    );
}

#[test]
fn cwarg_brief_text() {
    let w = WindowId::parse("cw3").expect("窓 id");
    let text = brief("/T", w);
    for part in [
        "相談の窓 cw3",
        "/T consult answer",
        "/T consult bundle cw3",
        "notes.md",
        "findings/",
        "drafts/",
        "work/",
        "bundle/ledger.json",
        "sandbox_denials",
    ] {
        assert!(text.contains(part), "{part}");
    }
    for field in DRAFT_FIELDS {
        assert!(text.contains(field), "{field}");
    }
    for verb in [
        "open", "launch", "show", "dispose", "close", "watch", "list", "guard",
    ] {
        assert!(!text.contains(&format!("consult {verb}")), "{verb}");
    }
}
