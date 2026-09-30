//! 3 つの字からグラフを組む。読めない字はその出所を `Graph::unread` に挙げ、ほかの出所は組む。
//! 設計の索引の節点と辺は表の行を写す。種類の読めない節点の行と型の読めない辺の行は、その行だけを組まずに数える
//! （飛ばした節点の行の id を端に持つ辺の行も組まずに数える）。bead の種類は epic・memo・問い・契約の順に決める。
//! 裁定と受けと方針は notes の定型行から導く。走行は event log の RunCreated から導く。
//! 設計ノートの行は索引の `NOTE_ROW_KIND` の節点の行から組み、design の辺は pointer の行が指す行の節点へ組む。
//! ruled_by の辺は build が組まず、build の後に `add_rulings` が裁定の書き出し（folio check --emit-rulings）から組む。
//! 台帳の bead の 2 つの概要は build が description の定型行（「概要 = 」「技術 = 」）から写す（行は無し）。
//! 設計の節点の行と 2 つの概要は組まず（無し）、build の後に `add_summary` が folio の要約の字から写す。
//! 更新の時刻は、bead は台帳の updated_at、走行は event log の読める ts の最後の値を epoch 秒で読む（読めなければ無し）。
//! 設計の索引の節点と notes の定型行から導く節点は時刻を持たない（無し）。
//! 索引の表と notes の定型行は時刻の欄を持たず、tsuzuri は設計文書も git も読まないため。
//! ほかの project の台帳の一覧は節点にせず、`outside` が bead の id と族と notes の裁定の定型行の id だけを読む（行 c-g3-extern）。

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::graph::{EdgeType, GraphEdge, GraphNode, NodeKind, title36};
pub(crate) use tsuzuri_contract::ledger::bead_kind;

use super::{BeadAttr, Graph, Inputs, Outside, PolicyAttr, RulingRow, RunAttr, Source};
use crate::ledger::epoch_secs;
use crate::question::{ENG_PREFIX, PLAIN_PREFIX, typed};

/// 設計の索引の節点の種類の数（`NodeKind::ALL` の先頭の 12 = 設計文書の 11 種と設計ノートの行）。
pub const DESIGN_KINDS: usize = 12;

/// 索引の設計ノートの行の種類の語（契約の型の serde の名とは違う）。
pub const NOTE_ROW_KIND: &str = "設計ノートの行";

/// pointer の先の file の名の終わり（folio derive が書く導出物）。
const DERIVED_EXT: &str = ".toml";

/// 設計の索引の辺の型の数（`EdgeType::ALL` の先頭の 19 = 設計文書の 17 型と設計ノートの行の 2 型）。
pub const DESIGN_EDGE_TYPES: usize = 19;

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

/// 方針の定型行の 2 つ目の欄の頭（範囲の欄）。
pub const SCOPE_FIELD: &str = "範囲 = ";

/// 方針の範囲が全体のときの字（範囲の欄が無いか空のときも）。
pub const SCOPE_ALL: &str = "all";

/// 問いの metadata の前提の鍵（型 premises の辺の先の id）。
pub const PREMISES_KEY: &str = "premises";

/// 3 つの字から導出グラフを組む。
pub fn build(inputs: &Inputs) -> Graph {
    let mut g = Graph::default();
    match read_design(inputs.design_index) {
        Some(design) => {
            g.nodes.extend(design.nodes);
            g.edges.extend(design.edges);
            g.skipped.design_nodes = design.skipped_nodes;
            g.skipped.design_edges = design.skipped_edges;
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

/// 裁定の書き出しの字（1 行 1 つの JSON の object）を読む（行 c-g3g7）。
/// 字が空か、JSON の object として読めないか `RulingRow` の形に合わない行が 1 つでも在れば None。空の行は読み捨てる。
pub fn read_rulings(text: &str) -> Option<Vec<RulingRow>> {
    if text.trim().is_empty() {
        return None;
    }
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str::<Value>(l)
                .ok()
                .filter(Value::is_object)
                .and_then(|v| serde_json::from_value::<RulingRow>(v).ok())
        })
        .collect()
}

/// 設計ノートの承認欄の裁定の欄の道か（`meta.approval[` と数字と `].ruling`）。
fn note_approval(field: &str) -> bool {
    field
        .strip_prefix("meta.approval[")
        .and_then(|rest| rest.strip_suffix("].ruling"))
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()))
}

/// 裁定の書き出しの字を節点へ結び、ruled_by の辺を組む（行 c-g3g7）。
/// 読めなければ g を変えずに偽を返す。読めたら上から順に、node が在れば id が node で file が同じ字の節点 1 つへ、
/// node が無く設計ノートの承認欄の行なら file が同じ字の設計ノートの行の節点の全部へ結び、ほかの行は結ばない。
/// 辺は節点の id の順・行の順に、先の在る行（`Graph::holds`）だけ、節点から `RulingRow::target` へ build の辺の後に足す
/// （同じ組は 1 本）。結んだ表を `Graph::rulings` に置いて真を返す。
pub fn add_rulings(g: &mut Graph, text: &str) -> bool {
    let Some(rows) = read_rulings(text) else {
        return false;
    };
    let mut joined: BTreeMap<String, Vec<RulingRow>> = BTreeMap::new();
    for row in rows {
        let file = Some(row.file.as_str());
        let ids: Vec<String> = match &row.node {
            Some(id) => g
                .nodes
                .iter()
                .find(|n| n.id == *id && n.file.as_deref() == file)
                .map(|n| n.id.clone())
                .into_iter()
                .collect(),
            None if note_approval(&row.field) => g
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::NoteRow && n.file.as_deref() == file)
                .map(|n| n.id.clone())
                .collect(),
            None => Vec::new(),
        };
        for id in ids {
            joined.entry(id).or_default().push(row.clone());
        }
    }
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut ruled: Vec<GraphEdge> = Vec::new();
    for (id, rows) in &joined {
        for row in rows.iter().filter(|r| g.holds(r)) {
            if seen.insert((id.clone(), row.target().to_string())) {
                ruled.push(edge(id, row.target(), EdgeType::RuledBy));
            }
        }
    }
    g.edges.extend(ruled);
    g.rulings = Some(joined);
    true
}

/// bead の id の族（最初の「.」の前の字）。
pub(crate) fn family(bead: &str) -> &str {
    bead.split('.').next().unwrap_or(bead)
}

/// 外の台帳の bead の 1 本のうち読む 2 欄（ほかの欄は読み捨てる）。
#[derive(Debug, Deserialize)]
struct OutsideBead {
    id: String,
    notes: Option<String>,
}

/// ほかの project の台帳の一覧（bd の読み取りの口が返す JSON の配列）を読む（行 c-g3-extern）。
/// 字が空か JSON の配列として読めない（欄 id の無い行が在る）なら None。読めたら bead の id と族と、notes の
/// 裁定の定型行（`TYPED_LINES` の裁定の種類）の id の集まり（配列が空なら既定の `Outside`）。
pub fn outside(ledger: &str) -> Option<Outside> {
    if ledger.trim().is_empty() {
        return None;
    }
    let beads: Vec<OutsideBead> = serde_json::from_str(ledger).ok()?;
    let mut out = Outside::default();
    for bead in beads {
        let notes = bead.notes.as_deref().unwrap_or_default();
        for (kind, id, _) in typed_lines(notes) {
            if kind == NodeKind::Ruling {
                out.rulings.insert(id);
            }
        }
        out.families.insert(family(&bead.id).to_string());
        out.beads.insert(bead.id);
    }
    Some(out)
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

/// 設計の索引から組んだ節点と辺と、組まずに数えた節点の行と辺の行の数。
struct Design {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    skipped_nodes: usize,
    skipped_edges: usize,
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
/// `#` で始まる行と空の行は読み捨てる。字が空か、列の数が合わない行が在れば読めない（None）。
/// 種類の読めない節点の行と型の読めない辺の行は組まずに数え、組んだ辺のうち端が飛ばした節点の行の id の
/// 辺も除いて辺の行に数える。
fn read_design(text: &str) -> Option<Design> {
    if text.trim().is_empty() {
        return None;
    }
    let mut design = Design {
        nodes: Vec::new(),
        edges: Vec::new(),
        skipped_nodes: 0,
        skipped_edges: 0,
    };
    let mut dropped: BTreeSet<&str> = BTreeSet::new();
    for line in text.lines().map(|l| l.trim_end_matches('\r')) {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        match line.split('\t').collect::<Vec<_>>()[..] {
            [id, kind, file, digest, title] => {
                let Some(kind) = design_kind(kind) else {
                    design.skipped_nodes += 1;
                    dropped.insert(id);
                    continue;
                };
                design.nodes.push(GraphNode {
                    id: id.to_string(),
                    kind,
                    file: Some(file.to_string()),
                    digest: Some(digest.to_string()),
                    title: title.to_string(),
                    line: None,
                    plain: None,
                    eng: None,
                    updated: None,
                });
            }
            [from, to, edge_type] => {
                match named::<EdgeType>(edge_type)
                    .filter(|t| EdgeType::ALL[..DESIGN_EDGE_TYPES].contains(t))
                {
                    Some(t) => design.edges.push(edge(from, to, t)),
                    None => design.skipped_edges += 1,
                }
            }
            _ => return None,
        }
    }
    let built = design.edges.len();
    design
        .edges
        .retain(|e| !dropped.contains(e.from.as_str()) && !dropped.contains(e.to.as_str()));
    design.skipped_edges += built - design.edges.len();
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

/// metadata の欄の id（欄は字 1 つか字の配列・metadata は object か、object を JSON にした字）。
pub(crate) fn metadata_ids(metadata: &Value, key: &str) -> Vec<String> {
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

/// 方針の定型行の範囲（「・」で割った 2 つ目の欄が `SCOPE_FIELD` で始まれば、その後の前後の空白を除いた字・
/// 欄が無いか空なら `SCOPE_ALL`）。
fn policy_scope(line: &str) -> String {
    line.split(ID_END)
        .nth(1)
        .and_then(|field| field.strip_prefix(SCOPE_FIELD))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(SCOPE_ALL)
        .to_string()
}

/// 台帳の bead から節点と辺を組む。design の辺は pointer の行が指す設計ノートの行の節点が在るときだけ組む。
/// 方針の属性（範囲）は同じ方針の id の最初の行から読む。
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
            updated: bead.updated_at.as_deref().and_then(epoch_secs),
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
                    updated: None,
                });
                if line_kind == NodeKind::Policy {
                    g.policies.insert(
                        id.clone(),
                        PolicyAttr {
                            scope: policy_scope(line),
                        },
                    );
                }
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
            for to in metadata_ids(&bead.metadata, PREMISES_KEY) {
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
    let stamp = at.as_bytes().split_first_chunk::<8>().is_some_and(|(date, rest)| {
        matches!(rest, [b'T', time @ .., b'Z']
            if time.len() == 6 && date.iter().chain(time).all(u8::is_ascii_digit))
    });
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
    // 走行ごとの、読める ts を持つ最後の event の時刻。
    let mut times: BTreeMap<String, EpochSecs> = BTreeMap::new();
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
        if let Some(at) = event.get("ts").and_then(Value::as_str).and_then(epoch_secs) {
            times.insert(run.to_string(), at);
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
            updated: times.get(&run).copied(),
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
