//! 席の停止の hook の plugin の登録の歯（行 f-stop・接頭辞 fstop_）。
//! workspace の根の plugin の 2 つの file（.claude-plugin/plugin.json と hooks/hooks.json）を JSON として読む
//! （境界の crate は serde_json に直に依存しないので、この歯は中核の crate に置く）。
#![cfg(test)]

use std::path::Path;

use serde_json::Value;

/// 行 f-stop の外で決めた filter の語（この行の接頭辞 fstop_ は並べない）。
const FILTER_WORDS: [&str; 92] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
    "kcli_",
    "hcard_",
    "qgate_",
    "nsum_",
    "nsumw_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
    "fmark_",
    "fserve_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
];

/// hooks.json の command の語（二重引用符を除いた字）。
const COMMAND_WORDS: [&str; 5] = [
    "$CLAUDE_PROJECT_DIR/target/debug/tz",
    "hook",
    "stop",
    "--repo",
    "$CLAUDE_PROJECT_DIR",
];

/// 名の字を持つ環境変数（command の中で二重引用符に挟む）。
const PROJECT_DIR: &str = "$CLAUDE_PROJECT_DIR";

fn read_json(rel: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel} は JSON でない: {e}"))
}

/// object の鍵を並べ替えた列。
fn keys(v: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = v
        .as_object()
        .unwrap_or_else(|| panic!("object でない: {v}"))
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

/// 要素 1 つの配列のその要素。
fn only(v: &Value) -> &Value {
    match v.as_array().map(Vec::as_slice) {
        Some([one]) => one,
        _ => panic!("要素 1 つの配列でない: {v}"),
    }
}

/// 空白で分けて二重引用符を除いた語。
fn words(text: &str) -> Vec<String> {
    text.split_whitespace().map(|w| w.replace('"', "")).collect()
}

/// 入れ子を全部たどった字の値。
fn strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
        Value::Object(o) => o.values().for_each(|x| strings(x, out)),
        _ => {}
    }
}

#[test]
fn fstop_plugin_stop_entry() {
    let plugin = read_json("plugin/.claude-plugin/plugin.json");
    assert_eq!(keys(&plugin), ["description", "name", "version"]);
    let brand = env!("CARGO_PKG_NAME")
        .strip_suffix("-core")
        .expect("中核の crate の名は -core で終わる");
    assert_eq!(plugin["name"], brand, "plugin の名は器の名");
    for key in ["version", "description"] {
        let s = plugin[key].as_str().unwrap_or_else(|| panic!("{key} は字"));
        assert!(!s.is_empty(), "{key} が空");
    }

    let hooks = read_json("plugin/hooks/hooks.json");
    assert_eq!(keys(&hooks), ["hooks"]);
    let entry = only(&hooks["hooks"]["Stop"]);
    assert_eq!(keys(entry), ["hooks"], "matcher を持たない");
    let hook = only(&entry["hooks"]);
    assert_eq!(keys(hook), ["command", "timeout", "type"]);
    assert_eq!(hook["type"], "command");
    let timeout = hook["timeout"].as_u64().expect("timeout は整数");
    assert!(timeout >= 30, "timeout {timeout} は 30 秒より短い");
    let command = hook["command"].as_str().expect("command は字");
    assert_eq!(words(command), COMMAND_WORDS, "{command}");
    let quoted = format!("\"{PROJECT_DIR}\"");
    assert_eq!(command.matches(PROJECT_DIR).count(), 2, "{command}");
    assert_eq!(command.matches(&quoted).count(), 2, "{command}");

    for (name, doc) in [("plugin.json", &plugin), ("hooks.json", &hooks)] {
        let mut all = Vec::new();
        strings(doc, &mut all);
        for s in &all {
            for w in words(s) {
                assert!(
                    !w.starts_with('/') && !w.starts_with('~'),
                    "{name} に絶対の path: {w}"
                );
            }
        }
    }
}

#[test]
fn fstop_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fstop.rs");
    let src = std::fs::read_to_string(&path).expect("tests/fstop.rs を読む");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 2, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("fstop_")
            .unwrap_or_else(|| panic!("{name} は fstop_ で始まらない"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
