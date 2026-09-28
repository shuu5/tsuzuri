//! 3 つの字からグラフを組む。読めない字はその出所を `Graph::unread` に挙げ、ほかの出所は組む。
//! 設計の索引の節点と辺は表の行を写す。bead の種類は epic・memo・問い・契約の順に決める。
//! 裁定と受けと方針は notes の定型行から導く。走行は event log の RunCreated から導く。
//! 設計ノートの行は索引の `NOTE_ROW_KIND` の節点の行から組み、design の辺は pointer の行が指す行の節点へ組む。
//! ruled_by の辺は組まない（索引に裁定の欄が無い）。
//! 台帳の bead の 2 つの概要は build が description の定型行（「概要 = 」「技術 = 」）から写す（行は無し）。
//! 設計の節点の行と 2 つの概要は組まず（無し）、build の後に `add_summary` が folio の要約の字から写す。

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tsuzuri_contract::graph::{EdgeType, GraphEdge, GraphNode, NodeKind, title36};
use tsuzuri_contract::ledger::{MEMO_LABEL, QUESTION_LABEL};

use super::{BeadAttr, Graph, Inputs, RunAttr, Source};
use crate::question::{ENG_PREFIX, PLAIN_PREFIX, typed};

/// 設計の索引の節点の種類の数（`NodeKind::ALL` の先頭の 12 = 設計文書の 11 種と設計ノートの行）。
pub const DESIGN_KINDS: usize = 12;

/// 索引の設計ノートの行の種類の語（契約の型の serde の名とは違う）。
pub const NOTE_ROW_KIND: &str = "設計ノートの行";

/// pointer の先の file の名の終わり（folio derive が書く導出物）。
const DERIVED_EXT: &str = ".toml";

/// 設計文書の辺の型の数（`EdgeType::ALL` の先頭の 17）。
pub const DESIGN_EDGE_TYPES: usize = 17;

/// 台帳の辺の型（bd の依存の種類のうち使う 4 つ）。
pub const LEDGER_EDGE_TYPES: [EdgeType; 4] = [
    EdgeType::ParentChild,
    EdgeType::Blocks,
    EdgeType::RelatesTo,
    EdgeType::DiscoveredFrom,
];

/// 契約の pointer の行の頭（acceptance の中）。
pub const POINTER_PREFIX: &str = "design = ";

/// notes の定型行の頭と、導く節点の種類（裁定・受け・方針）。
pub const TYPED_LINES: [(&str, NodeKind); 3] = [
    ("裁定 id = ", NodeKind::Ruling),
    ("受け id = ", NodeKind::Receipt),
    ("方針 id = ", NodeKind::Policy),
];

/// 定型行の id の終わりの字。
const ID_END: char = '・';

/// 3 つの字から導出グラフを組む。
pub fn build(inputs: &Inputs) -> Graph {
    let mut g = Graph::default();
    match read_design(inputs.design_index) {
        Some(design) => {
            g.nodes.extend(design.nodes);
            g.edges.extend(design.edges);
            g.skipped.design_edges = design.skipped;
        }
        None => g.unread.push(Source::Design),
    }
    match read_ledger(inputs.ledger) {
        Some(beads) => add_ledger(&mut g, beads),
        None => g.unread.push(Source::Ledger),
    }
    match read_events(inputs.events) {
        Some(events) => add_runs(&mut g, &events),
        None => g.unread.push(Source::Runs),
    }
    g
}

/// folio の要約の 1 行のうち写す欄（鍵 kind と title とほかの鍵は読み捨てる・鍵が無ければ無し）。
#[derive(Debug, Deserialize)]
struct SummaryRow {
    id: String,
    file: Option<String>,
    line: Option<u32>,
    plain: Option<String>,
    eng: Option<String>,
}

/// folio の要約の字（1 行 1 つの JSON の object）から、節点の行と 2 つの概要を写す（要件 FR15）。
/// 字が空か、形の合わない行が 1 つでも在れば読めず、g を変えずに偽を返す。
/// 読めたら、file を持つ節点のうち id と file が同じ行の在る節点の line・plain・eng をその行の値にして真を返す。
/// 行の無い節点と file の無い節点（台帳と走行）と、節点のほかの欄と辺と属性は触らない。
pub fn add_summary(g: &mut Graph, summary: &str) -> bool {
    if summary.trim().is_empty() {
        return false;
    }
    let mut rows: BTreeMap<(String, String), SummaryRow> = BTreeMap::new();
    for line in summary.lines().filter(|l| !l.trim().is_empty()) {
        let Some(row) = serde_json::from_str::<Value>(line)
            .ok()
            .filter(Value::is_object)
            .and_then(|v| serde_json::from_value::<SummaryRow>(v).ok())
        else {
            return false;
        };
        if let Some(file) = row.file.clone() {
            rows.entry((row.id.clone(), file)).or_insert(row);
        }
    }
    for node in &mut g.nodes {
        let Some(file) = &node.file else {
            continue;
        };
        if let Some(row) = rows.get(&(node.id.clone(), file.clone())) {
            node.line = row.line;
            node.plain = row.plain.clone();
            node.eng = row.eng.clone();
        }
    }
    true
}

/// 語から閉じた enum の値を読む（契約の型の crate の serde の名が正本）。
fn named<T: DeserializeOwned>(name: &str) -> Option<T> {
    serde_json::from_value(Value::String(name.to_string())).ok()
}

fn edge(from: &str, to: &str, edge_type: EdgeType) -> GraphEdge {
    GraphEdge {
        from: from.to_string(),
        to: to.to_string(),
        edge_type,
    }
}

/// 設計の索引から組んだ節点と辺と、組まずに数えた辺の行の数。
struct Design {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    skipped: usize,
}

/// 索引の種類の語を読む。語が `NOTE_ROW_KIND` なら設計ノートの行、ほかは契約の型の serde の名で読んだ
/// 種類が設計ノートの行でなく `NodeKind::ALL` の先頭の `DESIGN_KINDS` に在るときだけ（ほかは None）。
fn design_kind(word: &str) -> Option<NodeKind> {
    if word == NOTE_ROW_KIND {
        return Some(NodeKind::NoteRow);
    }
    named::<NodeKind>(word)
        .filter(|k| *k != NodeKind::NoteRow && NodeKind::ALL[..DESIGN_KINDS].contains(k))
}

/// pointer の行が指す設計ノートの行の id の候補。`POINTER_PREFIX` を除いた残りの前後の空白を除き、
/// 最初の `#` で path と行 id に分け、path の最後の `/` の後の字が `.toml` で終わるとき、
/// その前の字（文書 id）と `#` と行 id をつないだ字（`#` が無いか `.toml` で終わらなければ None）。
fn pointer_row(line: &str) -> Option<String> {
    let rest = line.strip_prefix(POINTER_PREFIX)?.trim();
    let (path, row) = rest.split_once('#')?;
    let name = path.rsplit('/').next().unwrap_or(path);
    let doc = name.strip_suffix(DERIVED_EXT)?;
    Some(format!("{doc}#{row}"))
}

/// 設計の索引を読む。節点の行は 5 列（id・種類・file・要約値・題）、辺の行は 3 列（端・端・型）。
/// `#` で始まる行と空の行は読み捨てる。字が空か、列の数か種類が合わない行が在れば読めない（None）。
fn read_design(text: &str) -> Option<Design> {
    if text.trim().is_empty() {
        return None;
    }
    let mut design = Design {
        nodes: Vec::new(),
        edges: Vec::new(),
        skipped: 0,
    };
    for line in text.lines().map(|l| l.trim_end_matches('\r')) {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        match line.split('\t').collect::<Vec<_>>()[..] {
            [id, kind, file, digest, title] => {
                let kind = design_kind(kind)?;
                design.nodes.push(GraphNode {
                    id: id.to_string(),
                    kind,
                    file: Some(file.to_string()),
                    digest: Some(digest.to_string()),
                    title: title.to_string(),
                    line: None,
                    plain: None,
                    eng: None,
                });
            }
            [from, to, edge_type] => {
                match named::<EdgeType>(edge_type)
                    .filter(|t| EdgeType::ALL[..DESIGN_EDGE_TYPES].contains(t))
                {
                    Some(t) => design.edges.push(edge(from, to, t)),
                    None => design.skipped += 1,
                }
            }
            _ => return None,
        }
    }
    Some(design)
}

/// bd の読み取りの口が返す配列の 1 本のうち、グラフと台帳の指標と pipeline の板が読む欄（知らない欄は読み捨てる）。
/// bd は空の欄を省くので、省ける欄は Option で読む。時刻は RFC 3339 の字のまま。
#[derive(Debug, Deserialize)]
pub(crate) struct BdBead {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) issue_type: Option<String>,
    pub(crate) labels: Option<Vec<String>>,
    pub(crate) notes: Option<String>,
    #[serde(alias = "acceptance")]
    pub(crate) acceptance_criteria: Option<String>,
    #[serde(default)]
    pub(crate) metadata: Value,
    pub(crate) parent: Option<String>,
    pub(crate) created_at: Option<String>,
    pub(crate) updated_at: Option<String>,
    pub(crate) closed_at: Option<String>,
    /// 閉じた理由（bd は閉じた bead にだけ出す）。
    pub(crate) close_reason: Option<String>,
    pub(crate) dependencies: Option<Vec<BdDependency>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BdDependency {
    pub(crate) depends_on_id: String,
    #[serde(rename = "type")]
    pub(crate) dep_type: String,
}

/// 台帳の一覧を読む（字が空か JSON の配列として読めなければ None）。
pub(crate) fn read_ledger(text: &str) -> Option<Vec<BdBead>> {
    if text.trim().is_empty() {
        return None;
    }
    serde_json::from_str(text).ok()
}

/// bead の種類（epic・memo・問い・契約の順に決める・どの bead も 1 つに当たる）。
pub(crate) fn bead_kind(issue_type: &str, labels: &[String]) -> NodeKind {
    let has = |label: &str| labels.iter().any(|l| l == label);
    if issue_type == "epic" {
        NodeKind::Epic
    } else if has(MEMO_LABEL) {
        NodeKind::Memo
    } else if has(QUESTION_LABEL) {
        NodeKind::Question
    } else {
        NodeKind::Task
    }
}

/// metadata の欄の id（欄は字 1 つか字の配列・metadata は object か、object を JSON にした字）。
fn metadata_ids(metadata: &Value, key: &str) -> Vec<String> {
    let parsed;
    let object = match metadata {
        Value::String(s) => {
            parsed = serde_json::from_str::<Value>(s).unwrap_or(Value::Null);
            &parsed
        }
        other => other,
    };
    match object.get(key) {
        Some(Value::String(s)) if !s.is_empty() => vec![s.clone()],
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

/// notes の定型行（種類・id・行）。id は頭の字の後から最初の「・」までの字。
fn typed_lines(notes: &str) -> Vec<(NodeKind, String, &str)> {
    notes
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter_map(|line| {
            TYPED_LINES.iter().find_map(|(prefix, kind)| {
                let rest = line.strip_prefix(prefix)?;
                let id = rest.split(ID_END).next().unwrap_or(rest).trim();
                (!id.is_empty()).then(|| (*kind, id.to_string(), line))
            })
        })
        .collect()
}

/// 台帳の bead から節点と辺を組む。design の辺は pointer の行が指す設計ノートの行の節点が在るときだけ組む。
fn add_ledger(g: &mut Graph, beads: Vec<BdBead>) {
    let rows: BTreeSet<String> = g
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::NoteRow)
        .map(|n| n.id.clone())
        .collect();
    let mut derived: BTreeSet<(NodeKind, String)> = BTreeSet::new();
    for bead in beads {
        let labels = bead.labels.unwrap_or_default();
        let kind = bead_kind(bead.issue_type.as_deref().unwrap_or_default(), &labels);
        let description = bead.description.as_deref().unwrap_or_default();
        g.nodes.push(GraphNode {
            id: bead.id.clone(),
            kind,
            file: None,
            digest: None,
            title: title36(bead.title.as_deref().unwrap_or_default()),
            line: None,
            plain: typed(description, PLAIN_PREFIX),
            eng: typed(description, ENG_PREFIX),
        });
        for dep in bead.dependencies.unwrap_or_default() {
            match named::<EdgeType>(&dep.dep_type).filter(|t| LEDGER_EDGE_TYPES.contains(t)) {
                Some(t) => g.edges.push(edge(&bead.id, &dep.depends_on_id, t)),
                None => g.skipped.ledger_edges += 1,
            }
        }
        for (line_kind, id, line) in typed_lines(bead.notes.as_deref().unwrap_or_default()) {
            if derived.insert((line_kind, id.clone())) {
                g.nodes.push(GraphNode {
                    id: id.clone(),
                    kind: line_kind,
                    file: None,
                    digest: None,
                    title: title36(line),
                    line: None,
                    plain: None,
                    eng: None,
                });
            }
            if line_kind == NodeKind::Ruling && kind == NodeKind::Question {
                g.edges.push(edge(&id, &bead.id, EdgeType::Answers));
            }
        }
        let touches = metadata_ids(&bead.metadata, "touches");
        if kind == NodeKind::Question {
            for to in &touches {
                g.edges.push(edge(&bead.id, to, EdgeType::Touches));
            }
            for to in metadata_ids(&bead.metadata, "premises") {
                g.edges.push(edge(&bead.id, &to, EdgeType::Premises));
            }
        }
        if kind == NodeKind::Memo {
            for to in metadata_ids(&bead.metadata, "source") {
                g.edges.push(edge(&bead.id, &to, EdgeType::Source));
            }
        }
        let pointers = bead
            .acceptance_criteria
            .as_deref()
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim_end_matches('\r'))
            .filter(|l| l.starts_with(POINTER_PREFIX))
            .map(str::to_string)
            .collect::<Vec<_>>();
        let mut pointed: BTreeSet<String> = BTreeSet::new();
        for row in pointers.iter().filter_map(|p| pointer_row(p)) {
            if rows.contains(&row) && pointed.insert(row.clone()) {
                g.edges.push(edge(&bead.id, &row, EdgeType::Design));
            }
        }
        g.beads.insert(
            bead.id,
            BeadAttr {
                kind,
                status: bead.status.unwrap_or_default(),
                labels,
                pointers,
                touches,
            },
        );
    }
}

/// event log を読む（字が空か、JSON の object として読めない行が在れば None）。
pub(crate) fn read_events(text: &str) -> Option<Vec<Value>> {
    if text.trim().is_empty() {
        return None;
    }
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str::<Value>(l)
                .ok()
                .filter(Value::is_object)
        })
        .collect()
}

/// run の id の前半（時刻の前の字）。後半が器の時刻の形（`20260927T071348Z`）でなければ None。
pub fn run_bead(run: &str) -> Option<&str> {
    let (bead, at) = run.rsplit_once('-')?;
    let b = at.as_bytes();
    let stamp = b.len() == 16
        && b[8] == b'T'
        && b[15] == b'Z'
        && b[..8].iter().chain(&b[9..15]).all(u8::is_ascii_digit);
    (stamp && !bead.is_empty()).then_some(bead)
}

/// event の口座（欄 account か、detail の `account:<名>` の札）。
fn event_account(event: &Value) -> Option<String> {
    if let Some(a) = event.get("account").and_then(Value::as_str) {
        return Some(a.to_string());
    }
    event
        .get("detail")
        .and_then(Value::as_str)?
        .split([',', ' '])
        .find_map(|tok| tok.strip_prefix("account:"))
        .filter(|a| !a.is_empty())
        .map(str::to_string)
}

/// event log から走行の節点と run_of・raised の辺を組む。段と口座と答えの無い問いの数は走行の属性に持つ。
/// raised の辺は QuestionRaised が問いの id の欄（question）を持つときだけ組む（器の実物は持たない）。
/// 同じ run の RunCreated の 2 件目は節点を足さない。RunCreated の無い run の event は読み捨てる。
fn add_runs(g: &mut Graph, events: &[Value]) {
    let mut order: Vec<String> = Vec::new();
    let mut attrs: BTreeMap<String, RunAttr> = BTreeMap::new();
    let mut raised: Vec<GraphEdge> = Vec::new();
    // 走行ごとの、最後の問いがまだ答えを持たないか。
    let mut open: BTreeMap<String, bool> = BTreeMap::new();
    for event in events {
        let kind = event
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(run) = event.get("run").and_then(Value::as_str) else {
            continue;
        };
        if kind == "RunCreated" && !attrs.contains_key(run) {
            order.push(run.to_string());
        }
        let attr = attrs.entry(run.to_string()).or_default();
        if let Some(stage) = event.get("stage").and_then(Value::as_str) {
            attr.stage = Some(stage.to_string());
        }
        if let Some(account) = event_account(event) {
            attr.account = Some(account);
        }
        if kind == "QuestionRaised" {
            attr.unanswered += 1;
            open.insert(run.to_string(), true);
            if let Some(q) = event.get("question").and_then(Value::as_str) {
                raised.push(edge(run, q, EdgeType::Raised));
            }
        }
        if kind == "QuestionAnswered" && open.insert(run.to_string(), false) == Some(true) {
            attr.unanswered -= 1;
        }
    }
    for run in order {
        g.nodes.push(GraphNode {
            id: run.clone(),
            kind: NodeKind::Run,
            file: None,
            digest: None,
            title: title36(&run),
            line: None,
            plain: None,
            eng: None,
        });
        if let Some(bead) = run_bead(&run) {
            g.edges.push(edge(&run, bead, EdgeType::RunOf));
        }
        let attr = attrs.remove(&run).unwrap_or_default();
        g.runs.insert(run, attr);
    }
    let runs = &g.runs;
    let raised: Vec<GraphEdge> = raised
        .into_iter()
        .filter(|e| runs.contains_key(&e.from))
        .collect();
    g.edges.extend(raised);
}

#[cfg(test)]
mod tests {
    use super::{metadata_ids, run_bead, typed_lines};
    use serde_json::json;
    use tsuzuri_contract::graph::NodeKind;

    #[test]
    fn graph_run_bead_reads_the_front() {
        assert_eq!(run_bead("t3-hub.2-20260927T071348Z"), Some("t3-hub.2"));
        assert_eq!(run_bead("t3-hub.2"), None);
        assert_eq!(run_bead("free-run"), None);
        assert_eq!(run_bead("-20260927T071348Z"), None);
        assert_eq!(run_bead("x-20260927X071348Z"), None);
    }

    #[test]
    fn graph_typed_lines_cut_at_the_dot() {
        let notes = "見本\n裁定 id = user 2026-09-27T03:19Z・束 b1・よい\n受け id = r-1\n 方針 id = p-1・頭に空白\n方針 id = ・空";
        let got: Vec<(NodeKind, String)> = typed_lines(notes)
            .into_iter()
            .map(|(k, id, _)| (k, id))
            .collect();
        assert_eq!(
            got,
            vec![
                (NodeKind::Ruling, "user 2026-09-27T03:19Z".to_string()),
                (NodeKind::Receipt, "r-1".to_string()),
            ]
        );
    }

    #[test]
    fn graph_metadata_ids_read_object_and_string() {
        assert_eq!(
            metadata_ids(&json!({"touches": ["A-1", "FR2"]}), "touches"),
            vec!["A-1", "FR2"]
        );
        assert_eq!(
            metadata_ids(&json!("{\"source\":\"r-1\"}"), "source"),
            vec!["r-1"]
        );
        assert!(metadata_ids(&json!(null), "touches").is_empty());
        assert!(metadata_ids(&json!({"touches": []}), "touches").is_empty());
    }
}
