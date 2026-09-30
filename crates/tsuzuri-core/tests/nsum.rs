//! 節点の行と 2 つの概要の歯（行 c-summary-design・要件 FR15）。
//! 契約の型の節点に欄 line・plain・eng が在り、中核の `add_summary` が folio の要約の字（JSON Lines）から写す。
//! build は 3 つの欄を組まない（無し）。節の字は歯の中で組み、workspace の根の fixture は読むだけで書かない。
#![cfg(test)]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tsuzuri_contract::graph::{AroundDoc, GraphDoc, GraphNode, GraphView, NodeKind};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::build::add_summary;
use tsuzuri_core::graph::{Graph, Inputs, build};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

fn read_root(rel: &str) -> String {
    read(&root().join(rel))
}

/// 節の節点 N1。
fn n1() -> GraphNode {
    GraphNode {
        id: "FR1".into(),
        kind: NodeKind::Req,
        file: Some("srs.yaml".into()),
        digest: Some("00000000".into()),
        title: "面は 2 つ".into(),
        line: Some(2),
        plain: Some("画面は 2 つです。".into()),
        eng: Some("面は 2 つとする。".into()),
        updated: None,
    }
}

/// 節の索引の字 IDX（タブ区切りの 5 列の 5 行）。
const IDX: &str = "FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ\n\
FR404\t要件\tsrs.yaml\t00000000\t要約の行が無い\n\
owner\t登場人物\tsrs.yaml\t00000000\t持ち主\n\
ADR-90\t判断の記録\tadr/ADR-90.yaml\t00000000\t見本\n\
A-9\t条\tconstitution.yaml\t00000000\tfile の違う行\n";

/// 節の台帳の字 LED（bead 1 本）。
const LED: &str =
    r#"[{"id":"t-1","title":"台帳の行","issue_type":"task","status":"open"}]"#;

/// 要約の 1 行（鍵は id・kind・file・line・title・plain・eng の順）。
#[expect(
    clippy::too_many_arguments,
    reason = "引数が規則の行 R-4 の 5 を越える・R-4 の歯の行が直してこの属性を外す"
)]
fn sum_line(
    id: &str,
    kind: &str,
    file: &str,
    line: u32,
    title: &str,
    plain: Option<&str>,
    eng: Option<&str>,
) -> String {
    format!(
        "{{\"id\":{},\"kind\":{},\"file\":{},\"line\":{line},\"title\":{},\"plain\":{},\"eng\":{}}}",
        json!(id),
        json!(kind),
        json!(file),
        json!(title),
        json!(plain),
        json!(eng),
    )
}

/// 節の要約の字 SUM（6 行と、3 行目と 4 行目の間の空の行）。
fn sum() -> String {
    [
        sum_line(
            "FR1",
            "要件",
            "srs.yaml",
            2,
            "面は 2 つ",
            Some("画面は 2 つです。"),
            Some("面は 2 つとする。"),
        ),
        sum_line(
            "ADR-90",
            "判断の記録",
            "adr/ADR-90.yaml",
            3,
            "見本",
            Some("一つ目です。\n二つ目です。"),
            Some("一つ目を決める。"),
        ),
        sum_line("owner", "登場人物", "srs.yaml", 37, "持ち主", None, None),
        String::new(),
        sum_line(
            "A-9",
            "条",
            "rules.yaml",
            7,
            "file の違う行",
            Some("違う file です。"),
            None,
        ),
        sum_line(
            "t-1",
            "要件",
            "srs.yaml",
            9,
            "台帳の行",
            Some("台帳です。"),
            Some("台帳。"),
        ),
        sum_line(
            "X-1",
            "要件",
            "srs.yaml",
            11,
            "無い節点",
            Some("無いです。"),
            Some("無い。"),
        ),
    ]
    .join("\n")
}

/// 節のグラフ G（IDX と LED・event log は空の字）。
fn g() -> Graph {
    build(&Inputs {
        design_index: IDX,
        ledger: LED,
        events: "",
    })
}

fn node<'a>(g: &'a Graph, id: &str) -> &'a GraphNode {
    g.node(id).unwrap_or_else(|| panic!("節点 {id} が在る"))
}

fn fields(n: &GraphNode) -> (Option<u32>, Option<&str>, Option<&str>) {
    (n.line, n.plain.as_deref(), n.eng.as_deref())
}

fn cleared(g: &Graph) -> Graph {
    let mut g = g.clone();
    for n in &mut g.nodes {
        n.line = None;
        n.plain = None;
        n.eng = None;
    }
    g
}

#[test]
fn nsum_wire_keys_in_order() {
    let text = wire::encode(&n1()).expect("encode");
    let keys = [
        "id", "kind", "file", "digest", "title", "line", "plain", "eng", "updated",
    ];
    let mut last = None;
    for key in keys {
        let pat = format!("\"{key}\":");
        assert_eq!(text.matches(&pat).count(), 1, "{pat} は 1 度: {text}");
        let at = text.find(&pat).unwrap();
        assert!(last.is_none_or(|l| l < at), "{pat} は前の鍵の後: {text}");
        last = Some(at);
    }
    let v: Value = serde_json::from_str(&text).expect("JSON");
    let obj = v.as_object().expect("object");
    let mut got: Vec<&str> = obj.keys().map(String::as_str).collect();
    got.sort_unstable();
    let mut want = keys.to_vec();
    want.sort_unstable();
    assert_eq!(got, want);
    assert_eq!(v["line"], json!(2));
    assert_eq!(v["plain"], json!("画面は 2 つです。"));
    assert_eq!(v["eng"], json!("面は 2 つとする。"));
    assert_eq!(wire::decode::<GraphNode>(&text).expect("decode"), n1());

    let bare = GraphNode {
        line: None,
        plain: None,
        eng: None,
        ..n1()
    };
    let v: Value = serde_json::from_str(&wire::encode(&bare).expect("encode")).expect("JSON");
    for key in ["line", "plain", "eng"] {
        assert_eq!(v.get(key), Some(&Value::Null), "鍵 {key} は null で在る");
    }
}

#[test]
fn nsum_old_text_reads_none() {
    let old = r#"{"id":"FR1","kind":"要件","file":"srs.yaml","digest":"00000000","title":"面は 2 つ"}"#;
    let n: GraphNode = wire::decode(old).expect("鍵の無い節点を読む");
    assert_eq!(fields(&n), (None, None, None));
    assert_eq!(n.id, "FR1");

    let doc: GraphDoc = wire::decode(&read_root("tests/fixtures/surface/graph-doc.json"))
        .expect("graph-doc.json");
    let around: AroundDoc = wire::decode(&read_root("tests/fixtures/surface/around-doc.json"))
        .expect("around-doc.json");
    let view: GraphView = wire::decode(&read_root("tests/fixtures/surface/graph-view.json"))
        .expect("graph-view.json");
    let nodes: Vec<&GraphNode> = doc
        .nodes
        .iter()
        .chain(around.rows.iter().map(|r| &r.node))
        .chain(view.nodes.iter().map(|v| &v.node))
        .collect();
    assert!(!nodes.is_empty());
    for n in nodes {
        assert_eq!(fields(n), (None, None, None), "{} の 3 つの欄は無し", n.id);
    }
}

/// 入れ子の全部から、鍵 id と kind と digest と title を持つ object を集める。
fn node_objects<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(map) => {
            if ["id", "kind", "digest", "title"]
                .iter()
                .all(|k| map.contains_key(*k))
            {
                out.push(v);
            }
            for child in map.values() {
                node_objects(child, out);
            }
        }
        Value::Array(items) => {
            for child in items {
                node_objects(child, out);
            }
        }
        _ => {}
    }
}

#[test]
fn nsum_snapshot_carries_fields() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tsuzuri-contract/tests/snapshots/graph.json");
    let v: Value = serde_json::from_str(&read(&path)).expect("snapshot は JSON");
    let mut objs = Vec::new();
    node_objects(&v, &mut objs);
    assert_eq!(objs.len(), 13);
    let first = &v["graph::GraphNode"][0];
    assert_eq!(first["id"], json!("ADR-7"));
    assert_eq!(first["line"], json!(3));
    assert_eq!(first["plain"], json!("決定の画面の形を決めます。"));
    assert_eq!(first["eng"], json!("面は 2 つとする。"));
    let mut nulls = 0;
    for o in &objs {
        for key in ["line", "plain", "eng"] {
            assert!(o.get(key).is_some(), "鍵 {key} が在る: {o}");
        }
        if std::ptr::eq(*o, first) {
            continue;
        }
        for key in ["line", "plain", "eng"] {
            assert_eq!(o[key], Value::Null, "鍵 {key} は null: {o}");
        }
        nulls += 1;
    }
    assert_eq!(nulls, 12);
}

#[test]
fn nsum_core_reads_no_files() {
    let f: fn(&mut Graph, &str) -> bool = add_summary;
    let mut g = Graph::default();
    assert!(!f(&mut g, ""));
    let src = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/graph/build.rs"));
    for word in ["std::fs", "std::process", "SystemTime"] {
        assert!(!src.contains(word), "build.rs に {word} が無い");
    }
}

#[test]
fn nsum_fills_matching_nodes() {
    let before = g();
    let mut g = before.clone();
    assert!(add_summary(&mut g, &sum()));
    assert_eq!(
        fields(node(&g, "FR1")),
        (Some(2), Some("画面は 2 つです。"), Some("面は 2 つとする。"))
    );
    assert_eq!(
        fields(node(&g, "ADR-90")),
        (
            Some(3),
            Some("一つ目です。\n二つ目です。"),
            Some("一つ目を決める。")
        )
    );
    assert_eq!(fields(node(&g, "owner")), (Some(37), None, None));
    for id in ["A-9", "FR404", "t-1"] {
        assert_eq!(fields(node(&g, id)), (None, None, None), "{id} は無しのまま");
    }
    assert_eq!(cleared(&g), before);
}

#[test]
fn nsum_missing_keys_are_null() {
    let before = g();
    let mut g = before.clone();
    assert!(add_summary(
        &mut g,
        r#"{"id":"FR1","file":"srs.yaml","line":2,"extra":1}"#
    ));
    assert_eq!(fields(node(&g, "FR1")), (Some(2), None, None));
    for n in g.nodes.iter().filter(|n| n.id != "FR1") {
        assert_eq!(fields(n), (None, None, None), "{} は無しのまま", n.id);
    }
    assert_eq!(cleared(&g), before);

    let mut g = before.clone();
    assert!(add_summary(&mut g, r#"{"id":"X-1","file":"srs.yaml"}"#));
    assert_eq!(g, before);
}

#[test]
fn nsum_unreadable_changes_nothing() {
    let first = sum().lines().next().unwrap().to_string();
    let bad_line = format!("{first}\n{}", r#"{"id":"FR404","file":"srs.yaml","line":-1}"#);
    let no_id = r#"{"file":"srs.yaml","line":2}"#.to_string();
    let tsv = read_root("tests/fixtures/graph/real/design-index.tsv");
    let before = g();
    for text in [String::new(), "\n\n".to_string(), tsv, bad_line, no_id] {
        let mut g = before.clone();
        assert!(!add_summary(&mut g, &text), "読めない: {text:?}");
        assert_eq!(g, before, "変えない: {text:?}");
    }
}

#[test]
fn nsum_every_index_node_filled() {
    let tsv = read_root("tests/fixtures/graph/real/design-index.tsv");
    let mut g = build(&Inputs {
        design_index: &tsv,
        ledger: "",
        events: "",
    });
    assert!(!g.nodes.is_empty());
    let text: Vec<String> = g
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            json!({
                "id": n.id,
                "file": n.file.as_deref().expect("設計の節点は file を持つ"),
                "line": i + 1,
                "plain": n.id,
                "eng": n.title,
            })
            .to_string()
        })
        .collect();
    let want: Vec<(String, String)> = g
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.title.clone()))
        .collect();
    assert!(add_summary(&mut g, &text.join("\n")));
    for (i, (n, (id, title))) in g.nodes.iter().zip(&want).enumerate() {
        assert_eq!(n.line, Some(u32::try_from(i + 1).unwrap()), "{id} の行");
        assert_eq!(n.plain.as_deref(), Some(id.as_str()), "{id} の plain");
        assert_eq!(n.eng.as_deref(), Some(title.as_str()), "{id} の eng");
    }
}

#[test]
fn nsum_build_leaves_fields_empty() {
    let tsv = read_root("tests/fixtures/graph/real/design-index.tsv");
    let ledger = read_root("tests/fixtures/ledger/bd-list-8.json");
    let events = read_root("tests/fixtures/graph/real/events.jsonl");
    let g = build(&Inputs {
        design_index: &tsv,
        ledger: &ledger,
        events: &events,
    });
    assert!(g.unread.is_empty());
    assert!(!g.nodes.is_empty());
    for n in &g.nodes {
        assert_eq!(fields(n), (None, None, None), "{} の 3 つの欄は無し", n.id);
    }
}

/// 着地済みの行と第 3 波から第 7 波の行の verify の語（92 語）。
const FILTER_WORDS: [&str; 92] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
    "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_",
    "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_",
    "pmore_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "qgate_", "hcard_",
    "ntime_", "ptitle_", "ncard_", "hsym_", "smore_", "lcard_", "aaround_", "hruling_",
    "sxaxis_", "plimit_", "bport_", "fstop_", "nsumw_", "fmark_", "fserve_", "cadopt_", "tipx_",
    "hcsess_", "hcproj_", "sesplit_", "mstore_", "athr_", "qblock_",
];

#[test]
fn nsum_own_names_clean() {
    let src = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/nsum.rs"));
    let lines: Vec<&str> = src.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .map(|w| {
            let rest = w[1].strip_prefix("fn ").expect("test の属性の次の行は fn");
            rest.split('(').next().unwrap()
        })
        .collect();
    assert_eq!(names.len(), 10);
    for name in names {
        let tail = name
            .strip_prefix("nsum_")
            .unwrap_or_else(|| panic!("{name} は nsum_ で始まる"));
        for word in FILTER_WORDS {
            assert!(!tail.contains(word), "{name} は {word} を含まない");
        }
    }
}
