//! account board の電文の歯（接頭辞 acctdoc_・設計ノート surface-base 便 b-acct）。
//! fixture は tests/fixtures/account/acct-doc.json（口座 3・群 2・project 3・移動 2・session 4）。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tsuzuri_contract::account::{
    AccountDoc, HEARTBEAT_PATH, Heartbeat, HeartbeatRequest, HeartbeatResponse, PATH,
};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;

fn crate_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_path() -> PathBuf {
    crate_dir().join("../../tests/fixtures/account/acct-doc.json")
}

fn fixture_text() -> String {
    std::fs::read_to_string(fixture_path()).expect("fixture を読む")
}

fn fixture() -> AccountDoc {
    wire::decode(&fixture_text()).expect("fixture は AccountDoc として読める")
}

/// dir の下の全部の `.rs` の file（path の順）。
fn rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("dir を読む")
        .map(|e| e.expect("dir の中身").path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            out.extend(rs_files(&p));
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    out
}

/// 値の中で字 `unknown` が立つ所の path（`.` と `[n]` でつなぐ）。
fn unknown_paths(v: &Value, at: &str, out: &mut BTreeSet<String>) {
    match v {
        Value::String(s) if s == "unknown" => {
            out.insert(at.to_string());
        }
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                unknown_paths(item, &format!("{at}[{i}]"), out);
            }
        }
        Value::Object(map) => {
            for (k, item) in map {
                let next = if at.is_empty() {
                    k.clone()
                } else {
                    format!("{at}.{k}")
                };
                unknown_paths(item, &next, out);
            }
        }
        _ => {}
    }
}

#[test]
fn acctdoc_paths_once_in_account_module() {
    assert_eq!(PATH, "/api/account");
    assert_eq!(HEARTBEAT_PATH, "/api/account/heartbeat");
    let src = crate_dir().join("src");
    let account = src.join("account.rs");
    for word in ["\"/api/account\"", "\"/api/account/heartbeat\""] {
        for file in rs_files(&src) {
            let text = std::fs::read_to_string(&file).expect("src を読む");
            let want = usize::from(file == account);
            assert_eq!(
                text.matches(word).count(),
                want,
                "{word} の字の数 {}",
                file.display()
            );
        }
    }
}

#[test]
fn acctdoc_fixture_roundtrip() {
    let doc = fixture();
    let text = wire::encode(&doc).expect("電文に戻す");
    let back: AccountDoc = wire::decode(&text).expect("読み直す");
    assert_eq!(back, doc);
    assert_eq!(wire::encode(&back).expect("電文に戻す"), text);
    // 数は契約の通り（口座 3・群 2・project 3・移動 2・session 4・休止は空）。
    let Reading::Known(accounts) = &doc.accounts else {
        panic!("口座の列が読めない");
    };
    assert_eq!(accounts.len(), 3);
    assert_eq!(accounts.iter().filter(|a| a.retired).count(), 1);
    let Reading::Known(groups) = &doc.groups else {
        panic!("群の列が読めない");
    };
    let names: Vec<&str> = groups.iter().map(|g| g.row.group.as_str()).collect();
    assert_eq!(names, ["Tier1", "Tier2"]);
    let Reading::Known(moves) = &doc.moves else {
        panic!("移動の列が読めない");
    };
    assert_eq!(moves.len(), 2);
    assert_eq!(doc.projects.len(), 3);
    assert_eq!(
        doc.projects.iter().filter(|p| !p.state_dir_known).count(),
        1
    );
    assert_eq!(doc.sessions.len(), 4);
    assert!(doc.dormant.is_empty());
}

#[test]
fn acctdoc_unknown_only_where_marked() {
    let text = fixture_text();
    let value: Value = serde_json::from_str(&text).expect("JSON");
    let mut got = BTreeSet::new();
    unknown_paths(&value, "", &mut got);
    let want: BTreeSet<String> = [
        "accounts.known[2].usage",
        "groups.known[0].members[1].matches",
        "projects[1].seat.known.usage",
        "projects[1].ledger",
        "projects[1].next",
        "projects[2].seat",
        "projects[2].runs",
        "projects[2].ledger",
        "projects[2].next",
        "sessions[1].spans",
        // 席の無い行の状態（席の状態の語 unknown）と稼働の記録。
        "sessions[3].state",
        "sessions[3].spans",
    ]
    .map(String::from)
    .into();
    assert_eq!(got, want, "unknown の字の在りか");

    // 部分ごとの読み: Unknown にした部分だけが Unknown で、ほかは値のまま。
    let doc = fixture();
    let known = |p: usize| {
        let row = &doc.projects[p];
        [
            matches!(row.seat, Reading::Known(_)),
            matches!(row.runs, Reading::Known(_)),
            matches!(row.ledger, Reading::Known(_)),
            matches!(row.next, Reading::Known(_)),
        ]
    };
    assert_eq!(known(0), [true, true, true, true]);
    assert_eq!(known(1), [true, true, false, false]);
    assert_eq!(known(2), [false, false, false, false]);
    assert!(
        matches!(doc.projects[0].runs, Reading::Known(r) if (r.wait, r.run, r.stop, r.land) == (1, 2, 0, 3))
    );
    assert_eq!(doc.projects[1].move_until, Some(doc.at + 1101));

    // 部分を 1 つずつ Unknown にすると、その部分だけが変わる。
    type Part = (&'static str, Option<usize>, fn(&mut AccountDoc));
    let parts: [Part; 7] = [
        ("accounts", None, |d| d.accounts = Reading::Unknown),
        ("groups", None, |d| d.groups = Reading::Unknown),
        ("moves", None, |d| d.moves = Reading::Unknown),
        ("seat", Some(0), |d| d.projects[0].seat = Reading::Unknown),
        ("runs", Some(0), |d| d.projects[0].runs = Reading::Unknown),
        ("ledger", Some(0), |d| {
            d.projects[0].ledger = Reading::Unknown
        }),
        ("next", Some(0), |d| d.projects[0].next = Reading::Unknown),
    ];
    for (key, project, set) in parts {
        let mut marked = value.clone();
        let slot = match project {
            None => &mut marked[key],
            Some(p) => &mut marked["projects"][p][key],
        };
        *slot = Value::String("unknown".into());
        let got: AccountDoc =
            serde_json::from_value(marked).expect("Unknown にした fixture を読む");
        let mut want = doc.clone();
        set(&mut want);
        assert_eq!(got, want, "{key} だけを Unknown にした読み");
        assert_ne!(got, doc, "{key} が変わらない");
    }
}

#[test]
fn acctdoc_heartbeat_closed_two() {
    let words: Vec<String> = Heartbeat::ALL
        .iter()
        .map(|h| wire::encode(h).expect("語"))
        .collect();
    assert_eq!(words, ["\"on\"", "\"off\""]);
    for (body, to) in [
        (r#"{"project":"proj-a","to":"on"}"#, Heartbeat::On),
        (r#"{"project":"proj-a","to":"off"}"#, Heartbeat::Off),
    ] {
        let req: HeartbeatRequest = wire::decode(body).expect("要求を読む");
        assert_eq!(
            req,
            HeartbeatRequest {
                project: "proj-a".into(),
                to
            }
        );
        assert_eq!(wire::encode(&req).expect("字"), body);
    }
    for bad in [
        r#"{"project":"proj-a","to":"On"}"#,
        r#"{"project":"proj-a","to":"OFF"}"#,
        r#"{"project":"proj-a","to":"stop"}"#,
        r#"{"project":"proj-a","to":""}"#,
        r#"{"project":"proj-a","to":true}"#,
        r#"{"project":"proj-a","to":null}"#,
        r#"{"project":"proj-a"}"#,
        r#"{"to":"on"}"#,
    ] {
        assert!(
            wire::decode::<HeartbeatRequest>(bad).is_err(),
            "{bad} を読む"
        );
    }
    let resp = HeartbeatResponse {
        target: "proj-a-orch".into(),
        to: Heartbeat::Off,
    };
    let text = wire::encode(&resp).expect("字");
    assert_eq!(text, r#"{"target":"proj-a-orch","to":"off"}"#);
    assert_eq!(
        wire::decode::<HeartbeatResponse>(&text).expect("読む"),
        resp
    );
}

#[test]
fn acctdoc_fixture_names_and_size() {
    let text = fixture_text();
    assert!(text.len() <= 20_000, "fixture が {} byte", text.len());
    let doc = fixture();
    let mut names: Vec<String> = Vec::new();
    if let Reading::Known(accounts) = &doc.accounts {
        names.extend(accounts.iter().map(|a| a.label.clone()));
    }
    if let Reading::Known(groups) = &doc.groups {
        for g in groups {
            names.push(g.row.account.clone());
            names.extend(g.row.candidates.iter().cloned());
            names.extend(g.row.next_account.iter().cloned());
            names.extend(g.previous.iter().cloned());
            names.extend(g.members.iter().filter_map(|m| m.seat_account.clone()));
        }
    }
    if let Reading::Known(moves) = &doc.moves {
        for m in moves {
            names.extend(m.from.iter().cloned());
            names.push(m.to.clone());
        }
    }
    for p in &doc.projects {
        if let Reading::Known(seat) = &p.seat {
            names.extend(seat.account.iter().cloned());
            if let Reading::Known(g) = &seat.group {
                names.push(g.account.clone());
                names.extend(g.candidates.iter().cloned());
                names.extend(g.next_account.iter().cloned());
            }
            if let Reading::Known(moves) = &seat.moves {
                for m in moves {
                    names.extend(m.from.iter().cloned());
                    names.push(m.to.clone());
                }
            }
        }
    }
    names.extend(doc.sessions.iter().filter_map(|s| s.account.clone()));
    assert!(names.len() > 20, "口座の名を拾えていない {names:?}");
    for n in &names {
        assert!(n.starts_with("acct-"), "acct- で始まらない口座の名 {n}");
    }
    let labels: BTreeSet<&str> = names.iter().map(String::as_str).collect();
    assert_eq!(labels, BTreeSet::from(["acct-1", "acct-2", "acct-3"]));

    // host の名と住所と home の path を書かない（行 D-4）。
    let lower = text.to_ascii_lowercase();
    for word in [
        "/home/",
        "~/",
        "/users/",
        "/root",
        "localhost",
        "tailnet",
        ".ts.net",
        "http:",
        "https:",
        "host",
    ] {
        assert!(!lower.contains(word), "fixture に {word} が在る");
    }
    // 数 4 つを点でつないだ住所の形と、IPv6 の `::` の形が無い。
    let bytes = text.as_bytes();
    for (i, _) in text.match_indices('.') {
        let digits_before = bytes[..i]
            .iter()
            .rev()
            .take_while(|b| b.is_ascii_digit())
            .count();
        let digits_after = bytes[i + 1..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        if digits_before == 0 || digits_after == 0 {
            continue;
        }
        let run = &text[i - digits_before..];
        let dotted = run
            .split(|c: char| !(c.is_ascii_digit() || c == '.'))
            .next()
            .unwrap_or("");
        assert!(dotted.split('.').count() < 4, "住所の形 {dotted}");
    }
    assert!(!text.contains("::"), "IPv6 の形");
}

#[test]
fn acctdoc_no_new_dependencies() {
    let manifest = std::fs::read_to_string(crate_dir().join("Cargo.toml")).expect("Cargo.toml");
    let deps: Vec<&str> = manifest
        .split("[dependencies]")
        .nth(1)
        .expect("[dependencies] の節")
        .lines()
        .map(str::trim)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| !l.is_empty())
        .filter_map(|l| l.split(['.', '=', ' ']).next())
        .collect();
    assert_eq!(deps, ["serde", "serde_json"]);
    assert!(!manifest.contains("[dev-dependencies]"));
}
