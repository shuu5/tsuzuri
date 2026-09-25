//! 設計文書の索引（便 94・docs/design/delivery-94.md §1）。正本 4 種（憲法・規則の表・要件書・判断の記録）の
//! id を持つ行を節点に、型付きの欄が指す id を辺にして、2 つの表と要約の 1 行を標準出力へ出す（`folio graph --print`）。
//! 索引は導出物で repo へは書かない（ADR-13 決定 (4)・P-6.3）。組めなければ表を出さずに「まだ分からない」（P-4.2）。
//! 節点の種類と辺の型の閉じた一覧の正本はこの file の定数（P-5.1・ADR-13 決定 (1)(2)）。欄の値を読み、散文は走査しない。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::ceiling_src::Ceiling;
use crate::floor::Floor;
use crate::gate;
use crate::sha256;
use crate::verdict::{Report, Verdict};
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

/// 辺の欄の閉じた一覧（正本の file ごと・欄の名・判断の記録は adr・便 99・ADR-13 決定 (8)）。節点の要約値はこの欄を
/// 落とした本文だけを数える。一覧は落とす側で持つ＝正本に増えた欄は本文に入って周を呼ぶ（黙って通さない・P-4.1）。
/// `verify.ac` は欄 verify の値の中の対 ac を指す。
pub const EDGE_FIELDS: [(&str, &[&str]); 4] = [
    ("constitution.yaml", &["relations"]),
    ("rules.yaml", &["article", "refs"]),
    ("srs.yaml", &["basis", "goals", "rules", "adrs", "verifies", "verify.ac"]),
    ("adr", &["basis", "produced"]),
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
            "最上位の節の閉じた一覧（ほかの節は床が落とす）。meta は人が書き、schema は生成区間。索引の中身そのものはこの file に置かない＝毎回 folio graph --print が正本から組み直す導出物である（判断の記録 ADR-13 決定 (4)・P-6.3 / P-6.4）",
        ),
    ),
    (
        "node",
        Floor::Map(&[("required", Floor::Strs(&["id", "kind", "file", "digest", "title"]))]),
    ),
    (
        "node_note",
        Floor::Val(
            "索引の節点 1 つの欄。id は設計文書の全体で 1 つに定まる id、kind は node_kinds の値、file は正本の置き場からの相対の path、digest は節点の本文の要約値（式は digest_note）、title は空白を 1 つに畳んで Unicode の字で 36 に切った 1 行の題",
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
            "辺の型の閉じた一覧（順も固定・ADR-13 決定 (2)）。正本は実装の型付きの定数 crates/folio/src/graph.rs の EDGE_TYPES で、この節はその写しである。figures は伝播に使わない（決定 (2)）。観点が読む文書と入口の棚の関係は辺にしない＝文書そのものを節点にしないので（判断の記録 ADR-14 決定 (1)）、その関係は天井の正本の読む欄の表と入口の棚から引く",
        ),
    ),
    (
        "edge_fields",
        Floor::Map(&[
            (EDGE_FIELDS[0].0, Floor::Strs(EDGE_FIELDS[0].1)),
            (EDGE_FIELDS[1].0, Floor::Strs(EDGE_FIELDS[1].1)),
            (EDGE_FIELDS[2].0, Floor::Strs(EDGE_FIELDS[2].1)),
            (EDGE_FIELDS[3].0, Floor::Strs(EDGE_FIELDS[3].1)),
        ]),
    ),
    (
        "edge_fields_note",
        Floor::Val(
            "辺の欄の閉じた一覧（正本の file ごと・欄の名）。節点の要約値はこの欄を落とした本文だけを数えるので、この欄に id を足すだけの変更は節点の要約値を動かさない。周の引き金になるかは天井の正本の引き金の一覧が決め、この欄のうち受入基準の verifies・要件の verify.ac・規則の表の行の article は規範の欄として引き金に入る（判断の記録 ADR-18 決定 (1)・ADR-20）。ほかの辺の欄に id を足すだけの変更は周の引き金にならない（ADR-13 決定 (8)）。正本は実装の型付きの定数 crates/folio/src/graph.rs の EDGE_FIELDS で、この節はその写しである（P-5.1・P-5.6）。verify.ac は要件の verify の中の対を指す。図の欄（figures）と改訂の来歴の欄（amended_by・amends）は散文を持つので本文の側に残す",
        ),
    ),
    (
        "digest_note",
        Floor::Val(
            "節点の要約値の式（正本の読み口に依らず、行の逐語の byte で決まる）。① 節点の block は、その id を持つ行から、空行でなく字下げが頭の行以下である最初の行の直前まで（判断の記録は file の全行）。② block から、入れ子の節点の block と 辺の欄の行（その行より深い続きの行も）を落とし、流れの形の行からは辺の欄の対を落とす。③ 末尾の空行を落とし、残った行を改行ごと連結した byte の sha256 の先頭 8 字が要約値。天井の印はこの要約値の表と、節点にも辺の欄にも属さない残りの byte の要約値（残差）を持つ（ADR-13 決定 (8)）",
        ),
    ),
]);

/// 題の字数の上限（Unicode の字）。
const TITLE_CHARS: usize = 36;

/// 2 つの表の見出し。
const NODES_HEAD: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）";
const EDGES_HEAD: &str = "# 辺（1 行 = 端 / 端 / 型・タブ区切り）";

/// 短い出力の 3 つの表の見出しと、最後に置く次の口の 1 行（便 96）。
const KINDS_HEAD: &str = "# 種類ごとの節点（1 行 = 種類 / 数・タブ区切り・閉じた一覧の順）";
const TYPES_HEAD: &str =
    "# 型ごとの辺（1 行 = 型 / 表に出た数 / 端が節点でない数・タブ区切り・閉じた一覧の順）";
const FILES_HEAD: &str = "# file ごとの節点（1 行 = file / 数・タブ区切り・file の名の byte 順）";
const NEXT_LINE: &str = "# 索引そのもの（節点と辺の全行）は folio graph --print";

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

/// 欄が指した参照の 3 つ組（端・端・型）。
type Ref = (String, String, &'static str);

/// 索引: 節点（id → 種類・file・題）と、欄が指した参照と、行の逐語から組んだ節点の要約値（--print だけが組む）。
#[derive(Default)]
struct Index {
    nodes: BTreeMap<String, (&'static str, String, String)>,
    refs: BTreeSet<Ref>,
    digests: BTreeMap<String, String>,
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

    /// 参照を 表に出る辺（両端が節点）と 端が節点でない参照 に分ける。
    fn split(&self) -> (Vec<&Ref>, Vec<&Ref>) {
        self.refs
            .iter()
            .partition(|(_, to, _)| self.nodes.contains_key(to))
    }

    /// 要約の 1 行（--print の最後の行・--digest の要約の 1 行目）。
    fn summary(&self) -> String {
        let (edges, dangling) = self.split();
        let types: BTreeSet<&str> = edges.iter().map(|(_, _, ty)| *ty).collect();
        format!(
            "# 節点 {}・辺 {}・型 {}・端が節点でない参照 {}\n",
            self.nodes.len(),
            edges.len(),
            types.len(),
            dangling.len()
        )
    }

    /// 2 つの表と要約の 1 行。辺は両端が節点のときだけ表に出し、端が節点でない参照は数だけ出す。
    fn render(&self) -> String {
        let mut out = format!("{NODES_HEAD}\n");
        for (id, (kind, file, title)) in &self.nodes {
            let digest = self.digests.get(id).map_or("", String::as_str);
            out.push_str(&format!("{id}\t{kind}\t{file}\t{digest}\t{title}\n"));
        }
        out.push_str(&format!("{EDGES_HEAD}\n"));
        for (from, to, ty) in self.split().0 {
            out.push_str(&format!("{from}\t{to}\t{ty}\n"));
        }
        out.push_str(&self.summary());
        out
    }

    /// 短い出力（便 96）: 種類ごとの節点・型ごとの辺（表に出た数 / 端が節点でない数）・file ごとの節点の 3 表と
    /// 要約の 2 行。1 表と 2 表は閉じた一覧の全数をその順で出す（数が 0 の行も出す）。
    fn digest(&self) -> String {
        let mut out = format!("{KINDS_HEAD}\n");
        for kind in NODE_KINDS {
            let n = self.nodes.values().filter(|(k, _, _)| *k == kind).count();
            out.push_str(&format!("{kind}\t{n}\n"));
        }
        out.push_str(&format!("{TYPES_HEAD}\n"));
        let (edges, dangling) = self.split();
        for ty in EDGE_TYPES {
            let shown = edges.iter().filter(|(_, _, t)| *t == ty).count();
            let loose = dangling.iter().filter(|(_, _, t)| *t == ty).count();
            out.push_str(&format!("{ty}\t{shown}\t{loose}\n"));
        }
        out.push_str(&format!("{FILES_HEAD}\n"));
        let mut files: BTreeMap<&str, usize> = BTreeMap::new();
        for (_, file, _) in self.nodes.values() {
            *files.entry(file.as_str()).or_default() += 1;
        }
        for (file, n) in files {
            out.push_str(&format!("{file}\t{n}\n"));
        }
        out.push_str(&self.summary());
        out.push_str(&format!("{NEXT_LINE}\n"));
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
    for name in adr_names(dir)? {
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

/// 判断の記録の file の名（`ADR-` で始まり `.yaml` で終わる・名の byte 順）。欄の決まりの file は節点でない。
fn adr_names(dir: &Path) -> Result<Vec<String>, String> {
    let ad = dir.join("adr");
    let entries = fs::read_dir(&ad).map_err(|e| format!("{} を読めない: {e}", ad.display()))?;
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("ADR-") && n.ends_with(".yaml"))
        .collect();
    names.sort();
    Ok(names)
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

// ── 節点の要約値（便 99・docs/design/delivery-99.md §1 (b)）──
// 正本を YAML として読まず、行の逐語の byte だけで切り分ける。凍結 anchor は folio に依らない独立の実装
// tests/fixtures/schema/node-digest.py の出力（P-10.2）。

/// 索引の正本の file（置き場からの相対）。
fn source_files(dir: &Path) -> Result<Vec<String>, String> {
    let mut files: Vec<String> = ["constitution.yaml", "rules.yaml", "srs.yaml"].map(str::to_string).to_vec();
    files.extend(adr_names(dir)?.into_iter().map(|n| format!("adr/{n}")));
    Ok(files)
}

fn read_text(dir: &Path, name: &str) -> Result<String, String> {
    fs::read_to_string(dir.join(name)).map_err(|e| format!("{name} を読めない: {e}"))
}

/// 節点の節（正本の file ごと）。
fn node_sections(name: &str) -> Vec<&'static str> {
    match name {
        "constitution.yaml" => vec!["articles"],
        "rules.yaml" => RULE_SECTIONS.to_vec(),
        "srs.yaml" => SRS_SECTIONS.iter().map(|(s, _, _)| *s).collect(),
        _ => Vec::new(),
    }
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// 字下げの後の `key:`（その後が空白か行の終わり）の key。
fn field_key(line: &str) -> Option<&str> {
    let s = line.trim_start_matches(' ');
    let k = s.find(':').filter(|k| *k > 0)?;
    if !matches!(s.as_bytes().get(k + 1), None | Some(b' ' | b'\n')) {
        return None;
    }
    let key = &s[..k];
    key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-').then_some(key)
}

/// 字下げ `depth` の節点の頭の行なら（id・流れの形か）。
fn head_id(line: &str, depth: usize) -> Option<(&str, bool)> {
    let rest = line.strip_prefix(" ".repeat(depth).as_str())?;
    if let Some(id) = rest.strip_prefix("- id: ") {
        return Some((id.trim(), false));
    }
    let rest = rest.strip_prefix("- {id: ")?;
    let end = rest.find([',', '}'])?;
    Some((rest[..end].trim(), true))
}

/// 頭の行 `i` の block の終わり（含まない）。流れの形の頭は 1 行だけ。
fn block_end(lines: &[&str], i: usize, depth: usize, flow: bool) -> usize {
    let mut j = i + 1;
    while !flow && j < lines.len() && (is_blank(lines[j]) || indent_of(lines[j]) > depth) {
        j += 1;
    }
    j
}

/// 二重引用符の開き `i` から、閉じの次の位置（逆斜線は次の 1 字を逃がす）。
fn skip_quoted(b: &[u8], mut i: usize) -> usize {
    i += 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    b.len()
}

/// 流れの形の行から欄 `names` の対を区切りごと落とす（対の頭は `{` か `, ` の直後・二重引用符の中は跳ばす）。
fn drop_pairs(line: &str, names: &[&str]) -> String {
    let mut line = line.to_string();
    let mut i = 0;
    while i < line.len() {
        let b = line.as_bytes();
        if b[i] == b'"' {
            i = skip_quoted(b, i);
            continue;
        }
        let opened = i > 0 && b[i - 1] == b'{';
        let comma = i > 1 && &b[i - 2..i] == b", ";
        let name = names.iter().find(|n| {
            (opened || comma) && b[i..].starts_with(n.as_bytes()) && b[i + n.len()..].starts_with(b": ")
        });
        let Some(name) = name else {
            i += 1;
            continue;
        };
        let (mut j, mut depth) = (i + name.len() + 2, 0usize);
        while j < b.len() {
            match b[j] {
                b'"' => {
                    j = skip_quoted(b, j);
                    continue;
                }
                b'[' | b'{' => depth += 1,
                b']' | b'}' if depth == 0 => break,
                b']' | b'}' => depth -= 1,
                b',' if depth == 0 => break,
                _ => {}
            }
            j += 1;
        }
        let (start, mut end) = if comma { (i - 2, j) } else { (i, j) };
        if !comma && b[end..].starts_with(b", ") {
            end += 2;
        }
        line.replace_range(start..end, "");
        i = start;
    }
    line
}

/// 行の逐語で切り分けた節点の要約値（id → 16 進 8 字）。
#[derive(Default)]
struct Scan {
    nodes: BTreeMap<String, String>,
}

impl Scan {
    /// 正本 1 file を切り分けて節点を足す。返りは残差（本文にも辺の欄にも属さない行を file の順に連結した字）。
    fn file(&mut self, name: &str, text: &str) -> Result<String, String> {
        let lines: Vec<&str> = text.split_inclusive('\n').collect();
        let mut owned = vec![false; lines.len()];
        if name.starts_with("adr/") {
            let id = lines
                .iter()
                .find_map(|l| l.strip_prefix("id: "))
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .ok_or_else(|| format!("{name} に id の行が無い"))?;
            let all: Vec<usize> = (0..lines.len()).collect();
            self.cut(&lines, &mut owned, all, 0, EDGE_FIELDS[3].1, id)?;
        } else {
            let fields = EDGE_FIELDS.iter().find(|(f, _)| *f == name).map_or(&[][..], |(_, f)| *f);
            let sections = node_sections(name);
            let (mut section, mut i) = (false, 0);
            while i < lines.len() {
                let line = lines[i];
                if !is_blank(line) && indent_of(line) == 0 {
                    section = field_key(line).is_some_and(|k| sections.contains(&k));
                }
                let Some((id, flow)) = head_id(line, 2).filter(|_| section) else {
                    i += 1;
                    continue;
                };
                let end = block_end(&lines, i, 2, flow);
                let mut kept = vec![i];
                let mut k = i + 1;
                while k < end {
                    match head_id(lines[k], 6).filter(|_| name == "constitution.yaml") {
                        Some((sub, sub_flow)) => {
                            let sub_end = block_end(&lines, k, 6, sub_flow);
                            self.cut(&lines, &mut owned, (k..sub_end).collect(), 8, fields, sub)?;
                            k = sub_end;
                        }
                        None => {
                            kept.push(k);
                            k += 1;
                        }
                    }
                }
                self.cut(&lines, &mut owned, kept, 4, fields, id)?;
                i = end;
            }
        }
        Ok(lines.iter().zip(&owned).filter(|(_, o)| !**o).map(|(l, _)| *l).collect())
    }

    /// 節点 1 つ: 行の番号の列 `idx`（入れ子を除いた block）から辺の欄（字下げ `depth` の行・より深い続きの行・
    /// 流れの形の対）と末尾の空行を落とし、残りを連結した byte の sha256 の先頭 8 字を要約値にする。
    fn cut(
        &mut self, lines: &[&str], owned: &mut [bool], mut idx: Vec<usize>, depth: usize, fields: &[&str], id: &str,
    ) -> Result<(), String> {
        while idx.last().is_some_and(|n| is_blank(lines[*n])) {
            idx.pop();
        }
        let whole: Vec<&str> = fields.iter().copied().filter(|f| !f.contains('.')).collect();
        let mut body: Vec<(usize, String)> = Vec::new();
        let mut k = 0;
        while k < idx.len() {
            let line = lines[idx[k]];
            owned[idx[k]] = true;
            k += 1;
            let key = field_key(line);
            if !is_blank(line) && indent_of(line) == depth && key.is_some_and(|k| whole.contains(&k)) {
                // 辺の欄の行と、その後の空行・より深い続きの行（block の末尾の空行は先に落としてある）
                while k < idx.len() && (is_blank(lines[idx[k]]) || indent_of(lines[idx[k]]) > depth) {
                    owned[idx[k]] = true;
                    k += 1;
                }
                continue;
            }
            let s = line.trim_start_matches(' ');
            let text = if s.starts_with("- {") {
                drop_pairs(line, &whole)
            } else if let Some(key) =
                key.filter(|key| s.get(key.len() + 2..).is_some_and(|v| v.starts_with('{')))
            {
                let prefix = format!("{key}.");
                let mut names = whole.clone();
                names.extend(fields.iter().filter_map(|f| f.strip_prefix(prefix.as_str())));
                drop_pairs(line, &names)
            } else {
                line.to_string()
            };
            body.push((idx[k - 1], text));
        }
        while body.last().is_some_and(|(_, l)| is_blank(l)) {
            owned[body.pop().map_or(0, |(n, _)| n)] = false;
        }
        let text: String = body.into_iter().map(|(_, l)| l).collect();
        let digest = sha256::hex(text.as_bytes())[..8].to_string();
        if self.nodes.insert(id.to_string(), digest).is_some() {
            return Err(format!("行の逐語に節点 {id} が 2 度ある"));
        }
        Ok(())
    }

    /// 索引の節点と行の逐語の節点が同じ集合なら要約値の表、食い違えば Err（P-4.1）。
    fn agree(self, index: &Index) -> Result<BTreeMap<String, String>, String> {
        fn only<V, W>(a: &BTreeMap<String, V>, b: &BTreeMap<String, W>) -> String {
            let ids: Vec<&str> = a.keys().filter(|k| !b.contains_key(*k)).map(String::as_str).collect();
            ids.join("・")
        }
        let (index_only, scan_only) = (only(&index.nodes, &self.nodes), only(&self.nodes, &index.nodes));
        if index_only.is_empty() && scan_only.is_empty() {
            return Ok(self.nodes);
        }
        Err(format!(
            "索引の節点と行の逐語の節点が食い違う（索引だけ: {index_only}・行だけ: {scan_only}）"
        ))
    }
}

/// 天井の印の 2 欄（便 99・§1 (d)）: 残差の要約値（「sha256 <16 進>」）と節点ごとの要約値の表（id の byte 順）。
/// 母集団 = 索引の正本と、観点の reads が指す文書の file（dir 形は直下の .yaml）の和集合を相対 path の byte 順に。
pub(crate) fn stamp_table(dir: &Path, ceiling: &Ceiling) -> Result<(String, Vec<(String, String)>), String> {
    let index = build(dir)?;
    let sources = source_files(dir)?;
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for name in &sources {
        files.insert(name.clone(), read_text(dir, name)?.into_bytes());
    }
    let documents = gate::documents(dir)?;
    for (doc, _) in ceiling.viewpoints.iter().flat_map(|vp| &vp.reads) {
        let (_, file) =
            documents.iter().find(|(id, _)| id == doc).ok_or_else(|| format!("{doc}: 文書の一覧に無い"))?;
        gate::collect(dir, file, &mut files)?;
    }
    let mut scan = Scan::default();
    let mut rest: Vec<u8> = Vec::new();
    for (name, bytes) in &files {
        if sources.contains(name) {
            let text = std::str::from_utf8(bytes).map_err(|e| format!("{name} を読めない: {e}"))?;
            rest.extend(scan.file(name, text)?.into_bytes());
        } else {
            rest.extend(bytes);
        }
    }
    let nodes = scan.agree(&index)?;
    Ok((format!("sha256 {}", sha256::hex(&rest)), nodes.into_iter().collect()))
}

/// 床の違反の種類（便 136）。入口の正本の違反の種類 index と読み違えない字。
pub const INDEX_KIND: &str = "索引の節点";

/// 行の逐語の id を突き合わせる字にする: 行の末尾の注釈（空白と `#` 以降）と前後の空白と引用符を外す。
fn bare_id(id: &str) -> &str {
    let cut = id
        .char_indices()
        .find(|&(i, c)| c == '#' && id[..i].ends_with(char::is_whitespace))
        .map_or(id, |(i, _)| &id[..i]);
    cut.trim().trim_matches(['"', '\'']).trim()
}

/// 床の口（便 136 §1 (b) の 1）: `graph --print` と `stamp_table` と同じ build と Scan で節点の集合を組み、食い違いを
/// 種類 `INDEX_KIND` の違反に数える（索引と印が組めない置き場を床が合格と言わない・P-4.1）。索引を組めないときは、
/// 床がほかに何も数えていなければ「まだ分からない」を 1 件足す（読めない正本は床が先に数えている＝同じ原因を 2 度数えない）。
pub fn check_index(dir: &Path, report: &mut Report) {
    let silent = report.violations.is_empty() && report.unknowns.is_empty() && report.pendings.is_empty();
    let files = build(dir).and_then(|index| Ok((index, source_files(dir)?)));
    let (index, files) = match files {
        Ok(built) => built,
        Err(e) => {
            if silent {
                report.unknown(format!("索引を組めない: {e}"));
            }
            return;
        }
    };
    // file ごとに切り分ける（切れない file はその file の違反 1 件で、その file の節点は数えない）
    let mut scanned: BTreeMap<String, String> = BTreeMap::new();
    let mut cut: Vec<String> = Vec::new();
    for name in files {
        let mut scan = Scan::default();
        let result = read_text(dir, &name).and_then(|text| scan.file(&name, &text)).and_then(|_| {
            match scan.nodes.keys().find(|id| scanned.contains_key(*id)) {
                Some(id) => Err(format!("行の逐語に節点 {id} が 2 度ある")),
                None => Ok(()),
            }
        });
        if let Err(e) = result {
            report.violation(
                INDEX_KIND,
                format!("{name}: 行の逐語で切れない（{e}・id の key か値が引用符つきか裸の形でない）"),
            );
            continue;
        }
        scanned.extend(scan.nodes.into_keys().map(|id| (id, name.clone())));
        cut.push(name);
    }
    let mut index_only: BTreeSet<&str> = BTreeSet::new();
    for (id, (_, file, _)) in &index.nodes {
        if cut.contains(file) && !scanned.contains_key(id) {
            index_only.insert(id);
            report.violation(
                INDEX_KIND,
                format!(
                    "{file}: 索引の節点 {id} の行を行の逐語で切れない（id か節の見出しの key が引用符つきか裸の形でない＝folio graph --print と天井の印が組めない）"
                ),
            );
        }
    }
    for (id, file) in &scanned {
        if !index.nodes.contains_key(id) && !index_only.contains(bare_id(id)) {
            report.violation(
                INDEX_KIND,
                format!("{file}: 行の逐語の節点 {id} が索引の節点に無い（id が引用符つきか裸の形でない）"),
            );
        }
    }
}

/// 節点の数と表に出た辺の数だけを返す口（`folio hello` の 1 行が使う・組み方を 2 面に増やさない・便 96）。
pub fn counts(dir: &Path) -> Result<(usize, usize), String> {
    let index = build(dir)?;
    let edges = index.split().0.len();
    Ok((index.nodes.len(), edges))
}

/// `folio graph --print` / `--digest` の結果。
pub struct Outcome {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub verdict: Verdict,
}

/// 組めたら標準出力の字（`digest` なら短い出力・でなければ索引）と 合格、組めなければ表を出さずに「まだ分からない」。
/// 索引は節点の要約値の欄を持つ（便 99）: 索引の節点と行の逐語から切り出した節点が食い違えば表を出さない（P-4.1）。
pub fn run(dir: &Path, digest: bool) -> Outcome {
    let built = build(dir).and_then(|mut index| {
        if !digest {
            let mut scan = Scan::default();
            for name in source_files(dir)? {
                scan.file(&name, &read_text(dir, &name)?)?;
            }
            index.digests = scan.agree(&index)?;
        }
        Ok(index)
    });
    match built {
        Ok(index) => Outcome {
            stdout: Some(if digest {
                index.digest()
            } else {
                index.render()
            }),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 便 136 §1 (c) の 1-2: 索引を組めないとき、床がほかに何も数えていなければ まだ分からない を 1 件足し、
    /// 違反か まだ分からない が在れば足さない（同じ原因を 2 度数えず、不合格を まだ分からない に変えない）。
    #[test]
    fn f136_unbuildable_index_is_unknown_only_when_the_floor_is_silent() {
        let td = std::env::temp_dir().join(format!("folio-graph-f136-{}", std::process::id()));
        fs::create_dir_all(&td).unwrap();
        let mut silent = Report::default();
        check_index(&td, &mut silent);
        assert!(silent.violations.is_empty());
        assert_eq!(silent.unknowns.len(), 1);
        assert!(silent.unknowns[0].starts_with("索引を組めない: "), "{:?}", silent.unknowns);
        let mut failed = Report::default();
        failed.violation("adr", "読めない");
        check_index(&td, &mut failed);
        assert!(failed.unknowns.is_empty());
        assert_eq!((failed.violations.len(), failed.verdict()), (1, Verdict::Fail));
        let mut unknown = Report::default();
        unknown.unknown("読めない");
        check_index(&td, &mut unknown);
        assert_eq!((unknown.violations.len(), unknown.unknowns.len()), (0, 1));
        fs::remove_dir_all(&td).unwrap();
    }
}
