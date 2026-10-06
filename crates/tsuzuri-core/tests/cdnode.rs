//! code の層の歯（接頭辞 cdnode_・判断の記録 ADR-46 の決定 (2)(3)(4)）。
//! git の file の一覧と ast-grep の stream と契約表の write-set の字から、file と定義の節点と 行 → file の辺を組み、
//! 行の触る定義を引く。見本の stream は ast-grep 0.45 の scan --json=stream の行のうち中核が読む欄だけを持つ。
#![cfg(test)]

use serde_json::json;
use tsuzuri_core::graph::code::{
    CodeGraph, DefKind, IMPL_FOR, LANGS, Mark, build, read_defs, read_write_sets,
};

/// stream の 1 行（rule の id・file・0 から数える頭と末の行と byte の範囲・名と trait の名・言語）。
fn line(
    rule: &str,
    file: &str,
    at: [usize; 4],
    names: (Option<&str>, Option<&str>),
    lang: &str,
) -> String {
    let [l0, l1, b0, b1] = at;
    let (name, tr) = names;
    let mut single = serde_json::Map::new();
    if let Some(n) = name {
        single.insert("NAME".into(), json!({"text": n}));
    }
    if let Some(t) = tr {
        single.insert("TRAIT".into(), json!({"text": t}));
    }
    json!({
        "text": "略", "ruleId": rule, "file": file, "language": lang,
        "range": {"byteOffset": {"start": b0, "end": b1}, "start": {"line": l0, "column": 0}, "end": {"line": l1, "column": 1}},
        "metaVariables": {"single": single, "multi": {}, "transformed": {}}
    })
    .to_string()
}

/// Rust の名の在る行（0 から数える頭と末の行と byte の範囲）。
fn rs(rule: &str, file: &str, lines: (usize, usize), bytes: (usize, usize), name: &str) -> String {
    line(
        rule,
        file,
        [lines.0, lines.1, bytes.0, bytes.1],
        (Some(name), None),
        "Rust",
    )
}

/// 見本の file の一覧（git ls-files -z の字）。
const FILES: &str = "a.rs\0b.rs\0docs/x.md\0docs/y.md\0";

/// 見本の stream（a.rs に struct S・impl S と中の fn new・fn f、b.rs に fn g）。
fn stream() -> String {
    [
        rs("fn", "a.rs", (5, 6), (60, 80), "f"),
        rs("struct", "a.rs", (0, 0), (0, 9), "S"),
        rs("impl", "a.rs", (1, 3), (10, 50), "S"),
        rs("fn", "a.rs", (2, 2), (20, 40), "new"),
        rs("fn", "b.rs", (0, 0), (0, 9), "g"),
    ]
    .join("\n")
}

/// 見本の契約表の導出物（行 a は a.rs と docs/x.md・行 b は b.rs）。
const TOML: &str = "schema = 1\n\n[[contract]]\nid = \"a\"\ntitle = \"id = \\\"z\\\" の題\"\nwrite-set = [\"a.rs\", \"docs/x.md\"]\n\n[[contract]]\nid = \"b\"\nwrite-set = [\"b.rs\"]\n";

fn toy() -> CodeGraph {
    let rows: Vec<(String, Vec<String>)> = read_write_sets(TOML)
        .unwrap()
        .into_iter()
        .map(|(id, items)| (format!("n1#{id}"), items))
        .collect();
    build(FILES, read_defs(&stream()).unwrap(), &rows)
}

fn ids(defs: &[&tsuzuri_core::graph::code::Def]) -> Vec<String> {
    defs.iter().map(|d| d.id.clone()).collect()
}

#[test]
fn cdnode_toy_repo_builds_file_and_def_nodes_and_row_edges() {
    let g = toy();
    assert_eq!(g.files.len(), 4);
    let all: Vec<&str> = g.defs.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(
        all,
        [
            "a.rs#struct S",
            "a.rs#impl S",
            "a.rs#impl S/fn new",
            "a.rs#fn f",
            "b.rs#fn g"
        ]
    );
    let files: Vec<&str> = g
        .row_writes("n1#a")
        .iter()
        .map(|w| w.file.as_str())
        .collect();
    assert_eq!(files, ["a.rs", "docs/x.md"]);
    assert_eq!(
        ids(&g.row_defs("n1#a")),
        [
            "a.rs#struct S",
            "a.rs#impl S",
            "a.rs#impl S/fn new",
            "a.rs#fn f"
        ]
    );
    assert_eq!(ids(&g.row_defs("n1#b")), ["b.rs#fn g"]);
    let new = g.defs.iter().find(|d| d.name == "new").unwrap();
    assert_eq!(
        (new.file.as_str(), new.kind, new.spans.clone()),
        ("a.rs", DefKind::Fn, vec![(3, 3)])
    );
    assert_eq!((g.unbound.len(), g.skipped), (0, 0));
}

#[test]
fn cdnode_same_name_fn_and_struct_are_two_nodes() {
    let text = [
        rs("struct", "a.rs", (0, 0), (0, 9), "S"),
        rs("fn", "a.rs", (1, 1), (10, 20), "S"),
    ]
    .join("\n");
    let defs = read_defs(&text).unwrap().defs;
    let got: Vec<(&str, DefKind)> = defs.iter().map(|d| (d.id.as_str(), d.kind)).collect();
    assert_eq!(
        got,
        [
            ("a.rs#struct S", DefKind::Struct),
            ("a.rs#fn S", DefKind::Fn)
        ]
    );
    // 否定の見本: 種類も同じ名の 2 行は 1 つの節点に範囲を 2 つ持つ。
    let twice = [
        rs("fn", "a.rs", (0, 0), (0, 9), "S"),
        rs("fn", "a.rs", (4, 5), (30, 40), "S"),
    ]
    .join("\n");
    let one = read_defs(&twice).unwrap().defs;
    assert_eq!(one.len(), 1);
    assert_eq!(one.first().unwrap().spans, vec![(1, 1), (5, 6)]);
}

#[test]
fn cdnode_files_outside_write_set_have_no_edge() {
    let g = toy();
    assert!(g.row_writes("n1#a").iter().all(|w| w.file != "b.rs"));
    assert!(!ids(&g.row_defs("n1#a")).contains(&"b.rs#fn g".to_string()));
    assert!(g.writes.iter().all(|w| w.file != "docs/y.md"));
    assert_eq!(g.writes.len(), 3);
    assert!(g.row_writes("n1#c").is_empty() && g.row_defs("n1#c").is_empty());
}

#[test]
fn cdnode_nest_names_container_and_kind() {
    let text = [
        rs("mod", "a.rs", (0, 3), (0, 50), "tests"),
        rs("fn", "a.rs", (1, 1), (10, 20), "helper"),
        line(
            IMPL_FOR,
            "a.rs",
            [4, 6, 51, 90],
            (Some("S"), Some("fmt::Display")),
            "Rust",
        ),
        rs("fn", "a.rs", (5, 5), (60, 80), "fmt"),
        rs("fn", "a.rs", (7, 7), (91, 99), "after"),
    ]
    .join("\n");
    let defs = read_defs(&text).unwrap().defs;
    let got: Vec<&str> = defs.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(
        got,
        [
            "a.rs#mod tests",
            "a.rs#mod tests/fn helper",
            "a.rs#impl fmt::Display for S",
            "a.rs#impl fmt::Display for S/fn fmt",
            "a.rs#fn after"
        ]
    );
}

#[test]
fn cdnode_marks_bind_new_gone_place_and_dir() {
    let rows = vec![(
        "n1#m".to_string(),
        ["+c.rs", "~a.rs", "-b.rs", "=b.rs", "docs/"]
            .map(String::from)
            .to_vec(),
    )];
    let g = build(FILES, read_defs("").unwrap(), &rows);
    let got: Vec<(&str, Mark, &str)> = g
        .writes
        .iter()
        .map(|w| (w.file.as_str(), w.mark, w.item.as_str()))
        .collect();
    assert_eq!(
        got,
        [
            ("a.rs", Mark::Gone, "~a.rs"),
            ("b.rs", Mark::Shrink, "-b.rs"),
            ("docs/x.md", Mark::Plain, "docs/"),
            ("docs/y.md", Mark::Plain, "docs/")
        ]
    );
    assert_eq!(g.unbound, [("n1#m".to_string(), "+c.rs".to_string())]);
}

#[test]
fn cdnode_unreadable_stream_or_write_set_is_none() {
    let good = stream();
    assert!(
        read_defs(&format!("{good}\n{{\"ruleId\": \"fn\"}}")).is_none(),
        "欄の欠けた行"
    );
    assert!(
        read_defs(&format!("{good}\nnot json")).is_none(),
        "JSON でない行"
    );
    assert_eq!(read_defs("\n\n").unwrap().defs.len(), 0);
    assert!(
        read_write_sets("[[contract]]\nid = \"a\"\nwrite-set = \"a.rs\"\n").is_none(),
        "配列でない write-set"
    );
    assert!(
        read_write_sets("[[contract]]\nwrite-set = [\"a.rs\"]\n").is_none(),
        "id の無い行"
    );
    assert_eq!(
        read_write_sets("[[contract]]\nid = \"a\"\nwrite-set = [\"a.rs\"]\n")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn cdnode_skipped_lines_are_counted() {
    let lines = [
        line("fn", "a.rs", [0, 0, 0, 9], (Some("p"), None), "Python"),
        rs("closure", "a.rs", (1, 1), (10, 19), "c"),
        line("fn", "a.rs", [2, 2, 20, 29], (None, None), "Rust"),
        line(IMPL_FOR, "a.rs", [3, 3, 30, 39], (Some("S"), None), "Rust"),
    ];
    for one in &lines {
        let read = read_defs(one).unwrap();
        assert_eq!((read.defs.len(), read.skipped), (0, 1), "{one}");
    }
    assert_eq!(LANGS, ["Rust"]);
    let outside = build(
        "b.rs\0",
        read_defs(&rs("fn", "a.rs", (0, 0), (0, 9), "f")).unwrap(),
        &[],
    );
    assert_eq!((outside.defs.len(), outside.skipped), (0, 1));
}

/// 規則の file は外の道具の data で、CI に道具が無く振る舞いで測れないので、rule の id と言語と捕える名の字を照らす。
#[test]
fn cdnode_rule_file_names_every_kind() {
    let text = include_str!("../../../.config/code-defs.yml");
    let docs: Vec<&str> = text.split("\n---\n").collect();
    let mut rules: Vec<&str> = docs
        .iter()
        .filter_map(|d| d.lines().find_map(|l| l.strip_prefix("id: ")))
        .collect();
    rules.sort_unstable();
    let mut want: Vec<&str> = DefKind::ALL
        .iter()
        .map(|k| k.word())
        .chain([IMPL_FOR])
        .collect();
    want.sort_unstable();
    assert_eq!(rules, want);
    for d in &docs {
        assert!(
            d.lines().any(|l| l == "language: Rust") && d.contains("pattern: $NAME"),
            "{d}"
        );
    }
    let imp = docs.iter().find(|d| d.contains("id: impl-for")).unwrap();
    assert!(imp.contains("pattern: $TRAIT"));
}
