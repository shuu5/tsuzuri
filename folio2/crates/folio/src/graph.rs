//! 設計文書の索引（便 94・docs/design/delivery-94.md §1）。正本 4 種（憲法・規則の表・要件書・判断の記録）の
//! id を持つ行を節点に、型付きの欄が指す id を辺にして、2 つの表と要約の 1 行を標準出力へ出す（`folio graph --print`）。
//! 索引は導出物で repo へは書かない（ADR-13 決定 (4)・P-6.3）。組めなければ表を出さずに「まだ分からない」（P-4.2）。
//! 節点の種類と辺の型の閉じた一覧の正本はこの file の定数（P-5.1・ADR-13 決定 (1)(2)）。欄の値を読み、散文は走査しない。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::schema::Floor;
use crate::verdict::Verdict;
use crate::yaml::{self, Node};

/// 節点の種類（11・順も固定）。
pub const NODE_KINDS: [&str; 11] = [
    "条",
    "規範文",
    "規則行",
    "目的",
    "要件",
    "非機能要件",
    "受入基準",
    "制約",
    "登場人物",
    "出力",
    "判断の記録",
];

/// 辺の型（17・順も固定）。
pub const EDGE_TYPES: [&str; 17] = [
    "in-article",
    "relations.articles",
    "relations.reqs",
    "relations.rules",
    "relations.sections",
    "amended_by",
    "article",
    "refs",
    "basis",
    "goals",
    "rules",
    "adrs",
    "verify.ac",
    "verifies",
    "figures",
    "produced",
    "amends",
];

/// 索引の欄の決まりの正本 `graph.yaml` の最上位の節の閉じた一覧（便 95）。
const GRAPH_TOP_LEVEL: [&str; 2] = ["meta", "schema"];

/// `graph.yaml` の schema 節（生成区間）の床の木（便 95・docs/design/delivery-95.md §1 (c)(d)）。欄の順と字面は凍結
/// anchor tests/fixtures/schema/graph-region.txt のとおり。閉じた一覧 2 本の葉は上の定数そのもの（同じ一覧を 2 回書かない）。
pub(crate) const FLOOR: Floor = Floor::Map(&[
    ("top_level", Floor::Strs(&GRAPH_TOP_LEVEL)),
    (
        "top_level_note",
        Floor::Val(
            "最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。meta は人が書き、schema は生成区間。索引の中身そのものはこの file に置かない＝毎回 folio graph --print が正本から組み直す導出物である（判断の記録 ADR-13 決定 (4)・P-6.3 / P-6.4）",
        ),
    ),
    (
        "node",
        Floor::Map(&[("required", Floor::Strs(&["id", "kind", "file", "title"]))]),
    ),
    (
        "node_note",
        Floor::Val(
            "索引の節点 1 つの欄。id は設計文書の全体で 1 つに定まる id、kind は node_kinds の値、file は正本の置き場からの相対の path、title は空白を 1 つに畳んで Unicode の字で 36 に切った 1 行の題。欄の要約値は天井の印と同じ便で足す（ADR-13 決定 (1)(8)）",
        ),
    ),
    ("node_kinds", Floor::Strs(&NODE_KINDS)),
    (
        "node_kinds_note",
        Floor::Val(
            "節点の種類の閉じた一覧（順も固定・増減は判断の記録が要る＝P-2.4 と同じ扱い）。正本は実装の型付きの定数 crates/folio/src/graph.rs の NODE_KINDS で、この節はその写しである（P-5.1・P-5.6）",
        ),
    ),
    (
        "edge",
        Floor::Map(&[("required", Floor::Strs(&["from", "to", "type"]))]),
    ),
    (
        "edge_note",
        Floor::Val(
            "索引の辺 1 つの欄。from と to は節点の id、type は edge_types の値。両端が節点のときだけ表に出し、端が節点でない参照（図の名・改訂の範囲の節名）は数だけ要約の 1 行に出す（P-4.2）",
        ),
    ),
    ("edge_types", Floor::Strs(&EDGE_TYPES)),
    (
        "edge_types_note",
        Floor::Val(
            "辺の型の閉じた一覧（順も固定・ADR-13 決定 (2)）。正本は実装の型付きの定数 crates/folio/src/graph.rs の EDGE_TYPES で、この節はその写しである。figures は伝播に使わない（決定 (2)）。観点が読む文書の欄（reads）と入口の棚の関係は、文書を節点にする便で足す",
        ),
    ),
]);

/// 題の字数の上限（Unicode の字）。
const TITLE_CHARS: usize = 36;

/// 2 つの表の見出し。
const NODES_HEAD: &str = "# 節点（1 行 = id / 種類 / file / 題 36 字・タブ区切り）";
const EDGES_HEAD: &str = "# 辺（1 行 = 端 / 端 / 型・タブ区切り）";

/// 規則の表の 2 節。
const RULE_SECTIONS: [&str; 2] = ["thresholds", "discipline"];

/// 要件書の 7 節・行の種類・題の欄。
const SRS_SECTIONS: [(&str, usize, &str); 7] = [
    ("goals", 3, "title"),
    ("actors", 8, "name"),
    ("outputs", 9, "name"),
    ("requirements", 4, "title"),
    ("nonfunctional", 5, "title"),
    ("acceptance", 6, "title"),
    ("constraints", 7, "title"),
];

/// 要件書の行の型付きの欄と辺の型（verify.ac は別に読む）。
const SRS_FIELDS: [(&str, usize); 6] = [
    ("basis", 8),
    ("goals", 9),
    ("rules", 10),
    ("adrs", 11),
    ("figures", 14),
    ("verifies", 13),
];

/// 憲法の relations の 4 名前空間と辺の型。
const RELATIONS: [(&str, usize); 4] = [("articles", 1), ("reqs", 2), ("rules", 3), ("sections", 4)];

/// 索引: 節点（id → 種類・file・題）と、欄が指した参照の 3 つ組（端・端・型）。
#[derive(Default)]
struct Index {
    nodes: BTreeMap<String, (&'static str, String, String)>,
    refs: BTreeSet<(String, String, &'static str)>,
}

impl Index {
    fn node(&mut self, id: &str, kind: usize, file: &str, title: Option<&Node>) {
        let title = fold(title.and_then(Node::as_str).unwrap_or_default());
        self.nodes
            .entry(id.to_string())
            .or_insert((NODE_KINDS[kind], file.to_string(), title));
    }

    fn edge(&mut self, from: &str, to: &str, ty: usize) {
        self.refs.insert((from.to_string(), to.to_string(), EDGE_TYPES[ty]));
    }

    /// 欄の値から id を取って辺を足す。
    fn field(&mut self, from: &str, value: Option<&Node>, ty: usize) {
        for to in value.map(ids).unwrap_or_default() {
            self.edge(from, to, ty);
        }
    }

    /// 2 つの表と要約の 1 行。辺は両端が節点のときだけ表に出し、端が節点でない参照は数だけ出す。
    fn render(&self) -> String {
        let mut out = format!("{NODES_HEAD}\n");
        for (id, (kind, file, title)) in &self.nodes {
            out.push_str(&format!("{id}\t{kind}\t{file}\t{title}\n"));
        }
        out.push_str(&format!("{EDGES_HEAD}\n"));
        let (edges, dangling): (Vec<_>, Vec<_>) = self
            .refs
            .iter()
            .partition(|(_, to, _)| self.nodes.contains_key(to));
        for (from, to, ty) in &edges {
            out.push_str(&format!("{from}\t{to}\t{ty}\n"));
        }
        let types: BTreeSet<&str> = edges.iter().map(|(_, _, ty)| *ty).collect();
        out.push_str(&format!(
            "# 節点 {}・辺 {}・型 {}・端が節点でない参照 {}\n",
            self.nodes.len(),
            edges.len(),
            types.len(),
            dangling.len()
        ));
        out
    }
}

/// 題を 1 行にする: 空白の連なりを 1 つに畳み、前後を落とし、Unicode の字で上限に切る。
fn fold(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(TITLE_CHARS)
        .collect()
}

/// 欄の値から id の一覧を取る（一覧なら各項・1 つの字ならそれ 1 つ・表なら adr の欄）。
fn ids(node: &Node) -> Vec<&str> {
    match node {
        Node::Null => Vec::new(),
        Node::Scalar(s) => vec![s.as_str()],
        Node::Seq(items) => items.iter().flat_map(ids).collect(),
        Node::Map(_) => node.get("adr").and_then(Node::as_str).into_iter().collect(),
    }
}

fn section<'a>(node: &'a Node, key: &str) -> &'a [Node] {
    node.get(key).and_then(Node::as_seq).unwrap_or_default()
}

fn id_of(node: &Node) -> Option<&str> {
    node.get("id").and_then(Node::as_str)
}

/// 正本 1 file を読む。読めない・最上位が欄の表でないは Err。
fn load(path: &Path) -> Result<Node, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{} を読めない: {e}", path.display()))?;
    let root = yaml::parse(&text)
        .map_err(|e| format!("{} を読めない: {e}", path.display()))?
        .root;
    if root.as_map().is_none() {
        return Err(format!("{} の最上位が欄の表でない", path.display()));
    }
    Ok(root)
}

/// 憲法: 条と規範文・構造の辺 in-article（両向き）・relations の 4 名前空間・amended_by。
fn constitution(index: &mut Index, root: &Node) {
    let file = "constitution.yaml";
    for article in section(root, "articles") {
        let Some(aid) = id_of(article) else {
            continue;
        };
        index.node(aid, 0, file, article.get("title"));
        for st in section(article, "statements") {
            if let Some(sid) = id_of(st) {
                index.node(sid, 1, file, st.get("text"));
                index.edge(aid, sid, 0);
                index.edge(sid, aid, 0);
            }
        }
        if let Some(rel) = article.get("relations") {
            for (key, ty) in RELATIONS {
                index.field(aid, rel.get(key), ty);
            }
        }
        index.field(aid, article.get("amended_by"), 5);
    }
}

/// 規則の表: 2 節の行・article・refs。
fn rules(index: &mut Index, root: &Node) {
    for name in RULE_SECTIONS {
        for row in section(root, name) {
            let Some(rid) = id_of(row) else {
                continue;
            };
            index.node(rid, 2, "rules.yaml", row.get("what"));
            index.field(rid, row.get("article"), 6);
            index.field(rid, row.get("refs"), 7);
        }
    }
}

/// 要件書: 7 節の行・basis / goals / rules / adrs / figures / verify.ac / verifies。
fn srs(index: &mut Index, root: &Node) {
    for (name, kind, title) in SRS_SECTIONS {
        for row in section(root, name) {
            let Some(id) = id_of(row) else {
                continue;
            };
            index.node(id, kind, "srs.yaml", row.get(title));
            for (key, ty) in SRS_FIELDS {
                index.field(id, row.get(key), ty);
            }
            index.field(id, row.get("verify").and_then(|v| v.get("ac")), 12);
        }
    }
}

/// 判断の記録: 記録・basis・produced・figures・amends の target（最初の区切りの前まで）。
fn adr(index: &mut Index, dir: &Path) -> Result<(), String> {
    let ad = dir.join("adr");
    let entries = fs::read_dir(&ad).map_err(|e| format!("{} を読めない: {e}", ad.display()))?;
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("ADR-") && n.ends_with(".yaml"))
        .collect();
    names.sort();
    for name in names {
        let root = load(&ad.join(&name))?;
        let Some(id) = id_of(&root) else {
            continue;
        };
        let file = format!("adr/{name}");
        index.node(id, 10, &file, root.get("title"));
        index.field(id, root.get("basis"), 8);
        index.field(id, root.get("produced"), 15);
        index.field(id, root.get("figures"), 14);
        for item in section(&root, "amends") {
            if let Some(target) = item.get("target").and_then(Node::as_str) {
                let head = target
                    .split(|c: char| c.is_whitespace() || matches!(c, '.' | '（' | '('))
                    .next()
                    .unwrap_or_default();
                index.edge(id, head, 16);
            }
        }
    }
    Ok(())
}

/// 正本の置き場から索引を組む。
fn build(dir: &Path) -> Result<Index, String> {
    let mut index = Index::default();
    constitution(&mut index, &load(&dir.join("constitution.yaml"))?);
    rules(&mut index, &load(&dir.join("rules.yaml"))?);
    srs(&mut index, &load(&dir.join("srs.yaml"))?);
    adr(&mut index, dir)?;
    if index.nodes.is_empty() {
        return Err("節点が 1 つも無い".to_string());
    }
    Ok(index)
}

/// `folio graph --print` の結果。
pub struct Outcome {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub verdict: Verdict,
}

/// 組めたら標準出力の字と 合格、組めなければ表を出さずに「まだ分からない」。
pub fn run(dir: &Path) -> Outcome {
    match build(dir) {
        Ok(index) => Outcome {
            stdout: Some(index.render()),
            stderr: None,
            verdict: Verdict::Pass,
        },
        Err(msg) => Outcome {
            stdout: None,
            stderr: Some(format!("まだ分からない（{msg}）")),
            verdict: Verdict::Unknown,
        },
    }
}
