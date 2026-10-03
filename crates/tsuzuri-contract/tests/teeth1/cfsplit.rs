//! 群ごとの snapshot と歯の file の見張りの歯（接頭辞 cfsplit_・設計ノート surface-wave16a 行 t-cform-json と t-cform-split）。
//! 群の json（tests/snapshots の群の名の json）は群の module の見本だけを持ち、鍵は群どうしで重ならず、群の歯の file（tests/cform_ と群の名の .rs）は自分の群の json だけを比べる。
//! 鍵の和は割る前の snapshot の鍵を、群の file の歯の名は割る前の歯の名を含む（床・足す行は一覧を直さず・消すか名を替える行だけが直す）。
//! 共通の手は tests/teeth1/common/mod.rs に 1 度ずつだけ在り、tests の下の file はどれも受付の行数の上限以下。
//! 割る前の歯と snapshot の path の字は repo の crates と xtask の code に残らない（この file 自身は数えない）。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// 群の名と、群が見本を持つ module の列。
const GROUPS: [(&str, &[&str]); 7] = [
    ("surface", &["surface"]),
    ("ledger", &["ledger", "question"]),
    ("graph", &["graph"]),
    ("board", &["board"]),
    ("stats", &["stats"]),
    ("seat", &["seat"]),
    ("case", &["case"]),
];

/// 割る前の snapshot（tests/snapshots/contract_form.json）の鍵（割る前の順）。
const BEFORE_KEYS: [&str; 62] = [
    "surface::RulingId",
    "surface::SeatRole",
    "surface::SeatHealth",
    "surface::SeatView",
    "surface::SurfaceState",
    "surface::SurfaceEvent",
    "surface::QuestionNudge",
    "surface::RulingRequest",
    "surface::RulingResponse",
    "surface::BatchRequest",
    "surface::ItemOutcome",
    "surface::BatchResponse",
    "surface::PolicyRequest",
    "surface::PolicyResponse",
    "surface::Refusal",
    "surface::RefusalResponse",
    "ledger::BeadId",
    "ledger::ChildType",
    "ledger::Effect",
    "ledger::LedgerRow",
    "ledger::LedgerItem",
    "ledger::LedgerList",
    "ledger::LedgerChanged",
    "ledger::LedgerWrite",
    "graph::NodeKind",
    "graph::EdgeType",
    "graph::GraphNode",
    "graph::GraphEdge",
    "graph::GraphSource",
    "graph::BeadAttr",
    "graph::RunAttr",
    "graph::Verdict",
    "graph::InvariantCheck",
    "graph::SkippedEdges",
    "graph::GraphDoc",
    "graph::EdgeEnd",
    "graph::ViewNode",
    "graph::ViewEdge",
    "graph::GraphView",
    "graph::BoxFold",
    "graph::Fold",
    "graph::AroundRow",
    "graph::HubCut",
    "graph::AroundDoc",
    "board::Stage",
    "board::PipelineColumn",
    "board::PipelineCard",
    "board::PipelineBoard",
    "board::NextMove",
    "board::LedgerJudge",
    "board::AccountBoard",
    "stats::LedgerStats",
    "stats::NextStep",
    "stats::UnreflectedRow",
    "stats::UnreflectedList",
    "question::QuestionCard",
    "question::QuestionList",
    "seat::SeatState",
    "seat::QuotaUsed",
    "seat::SeatSpan",
    "seat::AccountMove",
    "seat::SeatCard",
];

/// 共通の手の定義の字（tests/teeth1/common/mod.rs に 1 度ずつだけ在り、群の file には無い）。
const HANDS: [&str; 10] = [
    "const AT:",
    "fn bead(",
    "fn ruling(",
    "struct Samples<",
    "trait Form",
    "fn form<",
    "fn snapshot_text(",
    "fn snapshot_matches(",
    "fn roundtrip_all(",
    "fn distinct<",
];

/// 割る前の歯の名と、その名の歯を持つ群の列。
const TEETH: [(&str, &[&str]); 8] = [
    (
        "contract_form_snapshot_matches",
        &["surface", "ledger", "graph", "board", "stats", "seat"],
    ),
    (
        "contract_form_roundtrip_all_types",
        &["surface", "ledger", "graph", "board", "stats", "seat"],
    ),
    ("contract_form_ledger_write_argv", &["ledger"]),
    ("contract_form_ids_refuse_bad_shape", &["ledger"]),
    ("contract_form_bd_line_reads", &["ledger"]),
    ("contract_form_ledger_row_labels", &["ledger"]),
    ("contract_form_ledger_item_digest", &["ledger"]),
    (
        "contract_form_closed_lists",
        &["graph", "board", "stats", "seat"],
    ),
];

/// 受付の 1 file の行数の上限。
const LIMIT: usize = 1500;

/// 割る前の歯と snapshot の path の字。
const OLD_PATHS: [&str; 2] = ["contract_form.rs", "snapshots/contract_form.json"];

fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// 群の歯の file と共通の手の置き場（判断の記録 ADR-32 の群 teeth1）。
fn group_dir() -> PathBuf {
    tests_dir().join("teeth1")
}

/// 群の json の object。
fn read(group: &str) -> Map<String, Value> {
    let path = tests_dir().join(format!("snapshots/{group}.json"));
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
    match serde_json::from_str::<Value>(&text).expect("群の json は JSON") {
        Value::Object(map) => map,
        other => panic!("{group}.json は object でない: {other}"),
    }
}

/// file の字。
fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

/// 属性 test の次の fn の名（属性・注・空の行は跨ぐ）。
fn test_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut pending = false;
    for line in text.lines().map(str::trim) {
        if line == "#[test]" {
            pending = true;
        } else if pending && !(line.is_empty() || line.starts_with("#[") || line.starts_with("//"))
        {
            if let Some(rest) = line.strip_prefix("fn ") {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                names.insert(name);
            }
            pending = false;
        }
    }
    names
}

/// dir の下の file（下の dir も辿る・名が target の dir と symlink は辿らない）。
fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries =
            std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display()));
        for entry in entries {
            let entry = entry.expect("dir の項");
            let kind = entry.file_type().expect("項の種類");
            let path = entry.path();
            if kind.is_dir() {
                if entry.file_name() != "target" {
                    stack.push(path);
                }
            } else if kind.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

#[test]
fn cfsplit_snapshots_by_group() {
    let mut seen = BTreeSet::new();
    for (group, modules) in GROUPS {
        let text = read_text(&group_dir().join(format!("cform_{group}.rs")));
        let call = format!("common::snapshot_matches(\"{group}\", &forms())");
        assert_eq!(
            text.matches(&call).count(),
            1,
            "cform_{group}.rs が {call} を 1 度だけ持たない"
        );
        let map = read(group);
        assert!(!map.is_empty(), "{group}.json が空");
        for key in map.keys() {
            let module = key.split("::").next().unwrap_or_default();
            assert!(
                key.contains("::") && modules.contains(&module),
                "{group}.json に群の module でない鍵 {key}"
            );
            assert!(seen.insert(key.clone()), "鍵 {key} が群どうしで重なる");
        }
    }
    assert!(
        !tests_dir().join("snapshots/contract_form.json").exists(),
        "割る前の snapshot が残る"
    );
    assert!(
        !tests_dir().join("contract_form.rs").exists(),
        "割る前の歯の file が残る"
    );
}

#[test]
fn cfsplit_snapshot_keys_floor() {
    let before: BTreeSet<&str> = BEFORE_KEYS.iter().copied().collect();
    assert_eq!(before.len(), BEFORE_KEYS.len(), "BEFORE_KEYS に重なり");
    let mut keys = BTreeSet::new();
    for (group, _) in GROUPS {
        for key in read(group).keys() {
            assert!(keys.insert(key.clone()), "鍵 {key} が群どうしで重なる");
        }
    }
    let lost: Vec<&str> = before
        .iter()
        .copied()
        .filter(|k| !keys.contains(*k))
        .collect();
    assert!(lost.is_empty(), "割る前の鍵が群の json に無い: {lost:?}");
}

#[test]
fn cfsplit_test_names_floor() {
    let names: Vec<(&str, BTreeSet<String>)> = GROUPS
        .iter()
        .map(|(group, _)| {
            let text = read_text(&group_dir().join(format!("cform_{group}.rs")));
            (*group, test_names(&text))
        })
        .collect();
    for (name, groups) in TEETH {
        for group in groups {
            // 2 つ以上の群に在る名は群の字を挟む（fn の名は package の中で一意・判断の記録 ADR-32）。
            let want = if groups.len() > 1 {
                name.replacen("contract_form_", &format!("contract_form_{group}_"), 1)
            } else {
                (*name).to_string()
            };
            let (_, have) = names
                .iter()
                .find(|(g, _)| g == group)
                .unwrap_or_else(|| panic!("TEETH の群 {group} が GROUPS に無い"));
            assert!(
                have.contains(&want),
                "cform_{group}.rs に歯 {want} が無い（在る歯: {have:?}）"
            );
        }
    }
}

#[test]
fn cfsplit_hands_in_common() {
    for (group, _) in GROUPS {
        let text = read_text(&group_dir().join(format!("cform_{group}.rs")));
        let mods = text.lines().filter(|l| l.trim() == "use crate::common;").count();
        assert_eq!(mods, 1, "cform_{group}.rs の行 use crate::common; が {mods} 個");
        for hand in HANDS {
            assert!(
                !text.contains(hand),
                "cform_{group}.rs が共通の手 {hand} を持つ"
            );
        }
    }
    let common = read_text(&group_dir().join("common/mod.rs"));
    for hand in HANDS {
        assert_eq!(
            common.matches(hand).count(),
            1,
            "common/mod.rs の共通の手 {hand} が 1 度でない"
        );
    }
}

#[test]
fn cfsplit_files_within_limit() {
    let files = files_under(&tests_dir());
    for path in &files {
        let lines = read_text(path).lines().count();
        assert!(
            lines <= LIMIT,
            "{} が {lines} 行（上限 {LIMIT}）",
            path.display()
        );
    }
    assert!(files.len() > 12, "tests の下の file が {} 個", files.len());
}

#[test]
fn cfsplit_no_old_paths() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .expect("workspace の root");
    let me = group_dir().join("cfsplit.rs");
    let mut files = Vec::new();
    for dir in ["crates", "xtask"] {
        files.extend(
            files_under(&root.join(dir))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|e| e == "rs" || e == "toml")),
        );
    }
    assert!(
        files.contains(&manifest.join("src/lib.rs")),
        "辿った file に契約の crate の src/lib.rs が無い"
    );
    for path in files.iter().filter(|p| **p != me) {
        let text = read_text(path);
        for old in OLD_PATHS {
            assert!(
                !text.contains(old),
                "{} に割る前の path の字 {old} が残る",
                path.display()
            );
        }
    }
}
