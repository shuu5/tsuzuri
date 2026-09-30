//! 設計文書の索引（便 94・docs/design/delivery-94.md §1）。正本 4 種（憲法・規則の表・要件書・判断の記録）の
//! id を持つ行を節点に、型付きの欄が指す id を辺にして、2 つの表と要約の 1 行を標準出力へ出す（`folio graph --print`）。
//! 索引は導出物で repo へは書かない（ADR-13 決定 (4)・P-6.3）。組めなければ表を出さずに「まだ分からない」（P-4.2）。
//! 節点の種類と辺の型の閉じた一覧の正本はこの file の定数（P-5.1・ADR-13 決定 (1)(2)）。欄の値を読み、散文は走査しない。
//! 便 180（docs/design/delivery-180.md §1・要件 FR14 第 1.52 版）: `--print --summary` は表の代わりに、節点ごとに
//! 所属 file の中の id の行の番号・平易文の欄の字・技術の要約の字（受入基準は題の全文）を添えた 1 行の JSON（JSON Lines）を出す。
//! 便 185（docs/design/delivery-185.md §1・判断の記録 ADR-32・要件 FR14 第 1.54 版）: 設計ノートの契約表の節の行も節点にする
//! （種類 設計ノートの行・id は meta の id と行 id を「#」でつないだ字・辺は req と depends）。読み手は床と導出と同じ note.rs の load_notes。
//! 同じ id の節点を 2 度組んだ索引は、どの口（--print・--summary・--digest・folio hello）も まだ分からない にする（P-4.1）。
//! 便 208（docs/design/delivery-208.md §1・判断の記録 ADR-35 決定 (2)・要件 FR31）: `--summary` の 1 行の末尾に欄 status を置く。
//! 値は所属 file が 1 つの状態を持つ文書（判断の記録の status・設計ノートの meta.status）の字をそのまま、ほかは null。

use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::floor::Floor;
use crate::floor_note::CONTRACT_TABLE;
use crate::mentions;
use crate::note;
use crate::prose;
use crate::refs;
use crate::sha256;
use crate::verdict::{Report, Verdict};
use crate::yaml::{self, Node, json_str};

/// 節点の種類（12・順も固定）。
pub const NODE_KINDS: [&str; 12] = [
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
    "設計ノートの行",
];

/// 辺の型（19・順も固定）。
pub const EDGE_TYPES: [&str; 19] = [
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
    "req",
    "depends",
];

/// 辺の欄の閉じた一覧（正本の file ごと・欄の名・判断の記録は adr・設計ノートの契約表の節の行は design-note・便 99・便 185・
/// ADR-13 決定 (8)）。節点の要約値はこの欄を
/// 落とした本文だけを数える。一覧は落とす側で持つ＝正本に増えた欄は本文に入って周を呼ぶ（黙って通さない・P-4.1）。
/// `verify.ac` は欄 verify の値の中の対 ac を指す。
pub const EDGE_FIELDS: [(&str, &[&str]); 5] = [
    ("constitution.yaml", &["relations"]),
    ("rules.yaml", &["article", "refs"]),
    ("srs.yaml", &["basis", "goals", "rules", "adrs", "verifies", "verify.ac"]),
    ("adr", &["basis", "produced"]),
    ("design-note", &["req", "depends"]),
];

/// 索引の欄の決まりの正本 `graph.yaml` の最上位の節の閉じた一覧（便 95）。
const GRAPH_TOP_LEVEL: [&str; 2] = ["meta", "schema"];

/// `graph.yaml` の schema 節（生成区間）の床の木（便 95・docs/design/delivery-95.md §1 (c)(d)）。欄の順と字面は凍結
/// anchor tests/fixtures/schema/graph-region.txt のとおり。閉じた一覧 2 本の葉は上の定数そのもの（同じ一覧を 2 回書かない）。
/// 末尾の ids と mentions（便 195）は参照 id の空間（refs.rs・prose.rs）と行 R-17 の読み（mentions.rs）の定数の写し（P-5.6・行 D-11）。
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
            "索引の節点 1 つの欄。id は設計文書の全体で 1 つに定まる id（設計ノートの行は文書 id と行 id を「#」でつないだ字）、kind は node_kinds の値、file は正本の置き場からの相対の path、digest は節点の本文の要約値（式は digest_note）、title は空白を 1 つに畳んで Unicode の字で 36 に切った 1 行の題（設計ノートの行は行の section が指す節の題）",
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
            (EDGE_FIELDS[4].0, Floor::Strs(EDGE_FIELDS[4].1)),
        ]),
    ),
    (
        "edge_fields_note",
        Floor::Val(
            "辺の欄の閉じた一覧（正本の file ごと・欄の名・design-note は設計ノートの契約表の節の行の欄）。節点の要約値はこの欄を落とした本文だけを数えるので、この欄に id を足すだけの変更は節点の要約値を動かさない。正本は実装の型付きの定数 crates/folio/src/graph.rs の EDGE_FIELDS で、この節はその写しである（P-5.1・P-5.6）。verify.ac は要件の verify の中の対を指す。図の欄（figures）と改訂の来歴の欄（amended_by・amends）は散文を持つので本文の側に残す",
        ),
    ),
    (
        "digest_note",
        Floor::Val(
            "節点の要約値の式（正本の読み口に依らず、行の逐語の byte で決まる）。① 節点の block は、その id を持つ行から、空行でなく字下げが頭の行以下である最初の行の直前まで（判断の記録は file の全行・設計ノートの行は契約表の rows の中の字下げ 6 の「- 」の行から）。② block から、入れ子の節点の block と 辺の欄の行（その行より深い続きの行も）を落とし、流れの形の行からは辺の欄の対を落とす。③ 末尾の空行を落とし、残った行を改行ごと連結した byte の sha256 の先頭 8 字が要約値",
        ),
    ),
    (
        "ids",
        Floor::Map(&[
            ("rule_sections", Floor::Strs(&refs::RULE_SECTIONS)),
            ("srs_sections", Floor::Strs(&refs::SRS_ID_SECTIONS)),
            ("relation_namespaces", Floor::Strs(&refs::RELATION_NAMESPACES)),
            (
                "prefixes",
                Floor::Map(&[
                    ("article", Floor::Strs(&prose::ARTICLE)),
                    ("rule", Floor::Strs(&prose::RULE)),
                    ("srs", Floor::Strs(&refs::SRS_ID_PREFIXES)),
                ]),
            ),
        ]),
    ),
    (
        "ids_note",
        Floor::Val(
            "参照 id の空間の閉じた一覧。参照 id の解決の母集団（行 R-4）と散文の門と散文の言及の読み（行 R-17）が同じ定数を引く。rule_sections は規則の表の行を持つ節、srs_sections は要件書の id を持つ節、relation_namespaces は憲法の条の relations の名前空間である。prefixes は参照 id の頭で、article の頭の id は枝番 .<数> を取れ、rule と srs の頭の id は取れない。正本は実装の型付きの定数 crates/folio/src/refs.rs と prose.rs で、この節はその写しである（P-5.1・P-5.6・行 D-11）",
        ),
    ),
    (
        "mentions",
        Floor::Map(&[
            ("targets", Floor::Strs(&mentions::TARGETS)),
            ("typed", Floor::Strs(&mentions::TYPED)),
            ("provenance", Floor::Strs(&mentions::PROVENANCE)),
            ("top_skipped", Floor::Strs(&mentions::TOP_SKIPPED)),
            ("excluded", Floor::Strs(&mentions::EXCLUDED)),
            (
                "srs_kinds",
                Floor::Map(&[
                    (mentions::SRS_SECTIONS[0].0, Floor::Val(mentions::SRS_SECTIONS[0].1.name())),
                    (mentions::SRS_SECTIONS[1].0, Floor::Val(mentions::SRS_SECTIONS[1].1.name())),
                    (mentions::SRS_SECTIONS[2].0, Floor::Val(mentions::SRS_SECTIONS[2].1.name())),
                    (mentions::SRS_SECTIONS[3].0, Floor::Val(mentions::SRS_SECTIONS[3].1.name())),
                    (mentions::SRS_SECTIONS[4].0, Floor::Val(mentions::SRS_SECTIONS[4].1.name())),
                    (mentions::SRS_SECTIONS[5].0, Floor::Val(mentions::SRS_SECTIONS[5].1.name())),
                    (mentions::SRS_SECTIONS[6].0, Floor::Val(mentions::SRS_SECTIONS[6].1.name())),
                ]),
            ),
            (
                "receives",
                Floor::Map(&[
                    (mentions::RECEIVES[0].0, Floor::Strs(mentions::RECEIVES[0].1)),
                    (mentions::RECEIVES[1].0, Floor::Strs(mentions::RECEIVES[1].1)),
                    (mentions::RECEIVES[2].0, Floor::Strs(mentions::RECEIVES[2].1)),
                    (mentions::RECEIVES[3].0, Floor::Strs(mentions::RECEIVES[3].1)),
                    (mentions::RECEIVES[4].0, Floor::Strs(mentions::RECEIVES[4].1)),
                    (mentions::RECEIVES[5].0, Floor::Strs(mentions::RECEIVES[5].1)),
                    (mentions::RECEIVES[6].0, Floor::Strs(mentions::RECEIVES[6].1)),
                    (mentions::RECEIVES[7].0, Floor::Strs(mentions::RECEIVES[7].1)),
                    (mentions::RECEIVES[8].0, Floor::Strs(mentions::RECEIVES[8].1)),
                    (mentions::RECEIVES[9].0, Floor::Strs(mentions::RECEIVES[9].1)),
                ]),
            ),
        ]),
    ),
    (
        "mentions_note",
        Floor::Val(
            "散文の言及の歯（行 R-17）の機械の読みを閉じる 5 つの閉じた一覧。targets は対象の file（名が / で終わるものは dir の中の記録）、typed は型付きの辺として読む欄、provenance は来歴の欄、top_skipped は読まない最上位の節で、散文の欄はこの 3 つの補集合である。srs_kinds は要件書の節ごとの行の種類で、id を持つ行の種類は条・規範文・規則行・判断の記録とこの値である。receives は受け皿の表で、鍵は出所の行の種類、値はその行の型付きの欄が受けられる指す先の種類であり、値に無い組（空の一覧の行は全部）は数えない。excluded は数えない言及の語形で、言及を含む 1 文にどれかが在れば数えない。正本は実装の型付きの定数 crates/folio/src/mentions.rs で、この節はその写しである（P-5.6・行 D-11）",
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

/// 技術の要約の欄（規範文・本文・what・決定）。節点の行にこの順で最初に在る字を採る（条はその最初の規範文の字・
/// 受入基準は題の全文・便 180・設計ノートの行は行の題の全文・便 185）。
const ENG_FIELDS: [&str; 4] = ["shall", "text", "what", "decision"];

/// 設計ノートの置き場（正本の dir の直下・便 185）。
const NOTE_DIR: &str = "design-note";

/// 要件書の 7 節・行の種類・題の欄。
const SRS_SECTIONS: [(&str, &str, &str); 7] = [
    ("goals", NODE_KINDS[3], "title"),
    ("actors", NODE_KINDS[8], "name"),
    ("outputs", NODE_KINDS[9], "name"),
    ("requirements", NODE_KINDS[4], "title"),
    ("nonfunctional", NODE_KINDS[5], "title"),
    ("acceptance", NODE_KINDS[6], "title"),
    ("constraints", NODE_KINDS[7], "title"),
];

/// 要件書の行の型付きの欄と辺の型（verify.ac は別に読む）。
const SRS_FIELDS: [(&str, &str); 6] = [
    ("basis", EDGE_TYPES[8]),
    ("goals", EDGE_TYPES[9]),
    ("rules", EDGE_TYPES[10]),
    ("adrs", EDGE_TYPES[11]),
    ("figures", EDGE_TYPES[14]),
    ("verifies", EDGE_TYPES[13]),
];

/// 憲法の relations の 4 名前空間と辺の型。
const RELATIONS: [(&str, &str); 4] = [
    ("articles", EDGE_TYPES[1]),
    ("reqs", EDGE_TYPES[2]),
    ("rules", EDGE_TYPES[3]),
    ("sections", EDGE_TYPES[4]),
];

/// 欄が指した参照の 3 つ組（端・端・型）。
type Ref = (String, String, &'static str);

/// 索引: 節点（id → 種類・file・題）と、欄が指した参照と、行の逐語から組んだ節点の要約値と id の行の番号（--print
/// だけが組む）と、節点の平易文と技術の要約の字（無ければ None・便 180）と、所属 file の文書の状態の字（便 208）と、
/// 2 度組もうとした節点の id（便 185）。
#[derive(Default)]
struct Index {
    nodes: BTreeMap<String, (&'static str, String, String)>,
    refs: BTreeSet<Ref>,
    digests: BTreeMap<String, String>,
    lines: BTreeMap<String, usize>,
    texts: BTreeMap<String, (Option<String>, Option<String>)>,
    states: BTreeMap<String, String>,
    notes: Vec<String>,
    twice: BTreeSet<String>,
}

impl Index {
    fn node(&mut self, id: &str, kind: &'static str, file: &str, title: Option<&Node>) {
        let title = fold(title.and_then(Node::as_str).unwrap_or_default());
        if let Entry::Vacant(e) = self.nodes.entry(id.to_string()) {
            e.insert((kind, file.to_string(), title));
        } else {
            self.twice.insert(id.to_string());
        }
    }

    /// 節点の平易文（行の欄 plain）と技術の要約 `eng` の字を覚える。
    fn texts(&mut self, id: &str, row: &Node, eng: Option<&str>) {
        let plain = row.get("plain").and_then(Node::as_str).map(str::to_string);
        self.texts
            .entry(id.to_string())
            .or_insert((plain, eng.map(str::to_string)));
    }

    /// 節点の所属 file の文書の状態の欄の字を覚える（字でなければ覚えない＝--summary で null・便 208）。
    fn state(&mut self, id: &str, status: Option<&Node>) {
        if let Some(text) = status.and_then(Node::as_str) {
            self.states.entry(id.to_string()).or_insert_with(|| text.to_string());
        }
    }

    fn edge(&mut self, from: &str, to: &str, ty: &'static str) {
        self.refs.insert((from.to_string(), to.to_string(), ty));
    }

    /// 欄の値から id を取って辺を足す。
    fn field(&mut self, from: &str, value: Option<&Node>, ty: &'static str) {
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

    /// 節点ごとの 1 行の JSON（--print --summary・便 180）。欄は id・kind・file・line・title・plain・eng・status の順で空白を
    /// 挟まない。題は表と同じ字・line は所属 file の中でその id が書かれた行（1 始まり）・plain と eng は無ければ null・
    /// status は判断の記録と設計ノートの行の所属 file の状態の字で、ほかの節点と字でない状態は null（便 208）。
    fn jsonl(&self) -> String {
        let mut out = String::new();
        for (id, (kind, file, title)) in &self.nodes {
            let (plain, eng) = self.texts.get(id).cloned().unwrap_or_default();
            let line = self.lines.get(id).copied().unwrap_or_default();
            for (key, value) in [("{\"id\":", id.as_str()), (",\"kind\":", *kind), (",\"file\":", file.as_str())] {
                out.push_str(key);
                json_str(value, &mut out);
            }
            out.push_str(&format!(",\"line\":{line},\"title\":"));
            json_str(title, &mut out);
            let state = self.states.get(id).cloned();
            for (key, value) in [(",\"plain\":", plain), (",\"eng\":", eng), (",\"status\":", state)] {
                out.push_str(key);
                match value {
                    Some(text) => json_str(&text, &mut out),
                    None => out.push_str("null"),
                }
            }
            out.push_str("}\n");
        }
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

/// 行の技術の要約: ENG_FIELDS のうち最初に字の値を持つ欄の字（空の値の欄は飛ばす）。
fn eng(row: &Node) -> Option<&str> {
    ENG_FIELDS.iter().find_map(|k| row.get(k).and_then(Node::as_str))
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
        index.node(aid, NODE_KINDS[0], file, article.get("title"));
        let first = section(article, "statements").iter().find(|st| id_of(st).is_some());
        index.texts(aid, article, first.and_then(eng));
        for st in section(article, "statements") {
            if let Some(sid) = id_of(st) {
                index.node(sid, NODE_KINDS[1], file, st.get("text"));
                index.texts(sid, st, eng(st));
                index.edge(aid, sid, EDGE_TYPES[0]);
                index.edge(sid, aid, EDGE_TYPES[0]);
            }
        }
        if let Some(rel) = article.get("relations") {
            for (key, ty) in RELATIONS {
                index.field(aid, rel.get(key), ty);
            }
        }
        index.field(aid, article.get("amended_by"), EDGE_TYPES[5]);
    }
}

/// 規則の表: 2 節の行・article・refs。
fn rules(index: &mut Index, root: &Node) {
    for name in refs::RULE_SECTIONS {
        for row in section(root, name) {
            let Some(rid) = id_of(row) else {
                continue;
            };
            index.node(rid, NODE_KINDS[2], "rules.yaml", row.get("what"));
            index.texts(rid, row, eng(row));
            index.field(rid, row.get("article"), EDGE_TYPES[6]);
            index.field(rid, row.get("refs"), EDGE_TYPES[7]);
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
            // 受入基準（種類 6）は 4 つの欄を持たないので、技術の要約は題の全文（表の 36 字で切らない字）
            let text = if kind == NODE_KINDS[6] { row.get(title).and_then(Node::as_str) } else { eng(row) };
            index.texts(id, row, text);
            for (key, ty) in SRS_FIELDS {
                index.field(id, row.get(key), ty);
            }
            index.field(id, row.get("verify").and_then(|v| v.get("ac")), EDGE_TYPES[12]);
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
        index.node(id, NODE_KINDS[10], &file, root.get("title"));
        index.texts(id, &root, eng(&root));
        index.state(id, root.get("status"));
        index.field(id, root.get("basis"), EDGE_TYPES[8]);
        index.field(id, root.get("produced"), EDGE_TYPES[15]);
        index.field(id, root.get("figures"), EDGE_TYPES[14]);
        for item in section(&root, "amends") {
            if let Some(target) = item.get("target").and_then(Node::as_str) {
                let head = target
                    .split(|c: char| c.is_whitespace() || matches!(c, '.' | '（' | '('))
                    .next()
                    .unwrap_or_default();
                index.edge(id, head, EDGE_TYPES[16]);
            }
        }
    }
    Ok(())
}

/// 設計ノート（便 185・判断の記録 ADR-32）: 契約表の節の行・req・depends。id は meta の id と行 id を「#」でつないだ字、
/// 題は行の section が指す節の題、技術の要約は行の題の全文、状態はノートの meta.status（便 208）。読み手は床と導出と同じ `note::load_notes`（置き場が無ければ
/// 0 本・dir でない・読めない file が在れば Err＝索引を組まない・P-4.1）。
fn notes(index: &mut Index, dir: &Path) -> Result<(), String> {
    let nd = dir.join(NOTE_DIR);
    if !nd.exists() {
        return Ok(());
    }
    if nd.is_symlink() || !nd.is_dir() {
        return Err(format!("{NOTE_DIR}/ が dir でない"));
    }
    let mut report = Report::default();
    let docs = note::load_notes(&nd, &mut report);
    if let Some(why) = report.unknowns.first() {
        return Err(why.clone());
    }
    for doc in docs {
        let file = format!("{NOTE_DIR}/{}", doc.file);
        index.notes.push(file.clone());
        let Some(meta) = doc.root.get("meta").and_then(id_of) else {
            continue;
        };
        let status = doc.root.get("meta").and_then(|m| m.get("status"));
        let sections = section(&doc.root, "sections");
        let tables = sections
            .iter()
            .filter(|s| s.get("type").and_then(Node::as_str) == Some(CONTRACT_TABLE));
        for row in tables.flat_map(|s| section(s, "rows")) {
            let Some(rid) = id_of(row) else {
                continue;
            };
            let id = format!("{meta}#{rid}");
            let head = row
                .get("section")
                .and_then(Node::as_str)
                .and_then(|n| sections.iter().find(|s| s.get("n").and_then(Node::as_str) == Some(n)));
            index.node(&id, NODE_KINDS[11], &file, head.and_then(|s| s.get("title")));
            index.texts(&id, row, row.get("title").and_then(Node::as_str));
            index.state(&id, status);
            index.field(&id, row.get("req"), EDGE_TYPES[17]);
            for to in row.get("depends").map(ids).unwrap_or_default() {
                index.edge(&id, &format!("{meta}#{to}"), EDGE_TYPES[18]);
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
    notes(&mut index, dir)?;
    if index.nodes.is_empty() {
        return Err("節点が 1 つも無い".to_string());
    }
    Ok(index)
}

/// 口が使う索引: 同じ id の節点を 2 度組んだら黙って 1 つに数えず Err（床は行の逐語の側で違反に数える・便 185）。
fn whole(dir: &Path) -> Result<Index, String> {
    let index = build(dir)?;
    match index.twice.first() {
        Some(id) => Err(format!("索引に節点 {id} が 2 度ある")),
        None => Ok(index),
    }
}

// ── 節点の要約値（便 99・docs/design/delivery-99.md §1 (b)）──
// 正本を YAML として読まず、行の逐語の byte だけで切り分ける。凍結 anchor は folio に依らない独立の実装
// tests/fixtures/schema/node-digest.py の出力（P-10.2）。

/// 索引の正本の file（置き場からの相対・設計ノートは索引が読めたもの）。
fn source_files(dir: &Path, index: &Index) -> Result<Vec<String>, String> {
    let mut files: Vec<String> = ["constitution.yaml", "rules.yaml", "srs.yaml"].map(str::to_string).to_vec();
    files.extend(adr_names(dir)?.into_iter().map(|n| format!("adr/{n}")));
    files.extend(index.notes.iter().cloned());
    Ok(files)
}

fn read_text(dir: &Path, name: &str) -> Result<String, String> {
    fs::read_to_string(dir.join(name)).map_err(|e| format!("{name} を読めない: {e}"))
}

/// 節点の節（正本の file ごと）。
fn node_sections(name: &str) -> Vec<&'static str> {
    match name {
        "constitution.yaml" => vec!["articles"],
        "rules.yaml" => refs::RULE_SECTIONS.to_vec(),
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
    if flow {
        return i + 1;
    }
    let deeper = lines
        .iter()
        .skip(i + 1)
        .take_while(|l| is_blank(l) || indent_of(l) > depth);
    i + 1 + deeper.count()
}

/// 二重引用符の開き `i` から、閉じの次の位置（逆斜線は次の 1 字を逃がす）。
fn skip_quoted(b: &[u8], i: usize) -> usize {
    let mut j = i + 1;
    while let Some(&c) = b.get(j) {
        match c {
            b'\\' => j += 2,
            b'"' => return j + 1,
            _ => j += 1,
        }
    }
    b.len()
}

/// 流れの形の行から欄 `names` の対を区切りごと落とす（対の頭は `{` か `, ` の直後・二重引用符の中は跳ばす）。
fn drop_pairs(line: &str, names: &[&str]) -> String {
    let mut line = line.to_string();
    let mut i = 0;
    while let Some(&head) = line.as_bytes().get(i) {
        let b = line.as_bytes();
        if head == b'"' {
            i = skip_quoted(b, i);
            continue;
        }
        let opened = i > 0 && b.get(i - 1) == Some(&b'{');
        let comma = i > 1 && b.get(i - 2..i).is_some_and(|w| w == b", ");
        let here = b.get(i..).unwrap_or_default();
        let name = names.iter().find(|n| {
            (opened || comma)
                && here.starts_with(n.as_bytes())
                && here.get(n.len()..).is_some_and(|r| r.starts_with(b": "))
        });
        let Some(name) = name else {
            i += 1;
            continue;
        };
        let (mut j, mut depth) = (i + name.len() + 2, 0usize);
        while let Some(&c) = b.get(j) {
            match c {
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
        if !comma && b.get(end..).is_some_and(|r| r.starts_with(b", ")) {
            end += 2;
        }
        line.replace_range(start..end, "");
        i = start;
    }
    line
}

/// 行の逐語で切り分けた節点の要約値（id → 16 進 8 字）と、id が書かれた行の番号（id → 1 始まり・便 180）。
#[derive(Default)]
struct Scan {
    nodes: BTreeMap<String, String>,
    lines: BTreeMap<String, usize>,
}

impl Scan {
    /// 正本 1 file を切り分けて節点を足す。返りは残差（本文にも辺の欄にも属さない行を file の順に連結した字）。
    fn file(&mut self, name: &str, text: &str) -> Result<String, String> {
        let lines: Vec<&str> = text.split_inclusive('\n').collect();
        let mut owned = vec![false; lines.len()];
        if name.starts_with("adr/") {
            let (at, id) = lines
                .iter()
                .enumerate()
                .find_map(|(n, l)| l.strip_prefix("id: ").map(|id| (n, id.trim())))
                .filter(|(_, id)| !id.is_empty())
                .ok_or_else(|| format!("{name} に id の行が無い"))?;
            let all: Vec<usize> = (0..lines.len()).collect();
            self.cut(&lines, &mut owned, all, 0, EDGE_FIELDS[3].1, id)?;
            self.lines.insert(id.to_string(), at + 1);
        } else if name.starts_with(NOTE_DIR) {
            self.note(&lines, &mut owned)?;
        } else {
            let fields = EDGE_FIELDS.iter().find(|(f, _)| *f == name).map_or(&[][..], |(_, f)| *f);
            let sections = node_sections(name);
            let (mut section, mut i) = (false, 0);
            while let Some(&line) = lines.get(i) {
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
                    let sub = lines.get(k).and_then(|l| head_id(l, 6));
                    match sub.filter(|_| name == "constitution.yaml") {
                        Some((sub, sub_flow)) => {
                            let sub_end = block_end(&lines, k, 6, sub_flow);
                            self.cut(&lines, &mut owned, (k..sub_end).collect(), 8, fields, sub)?;
                            self.lines.insert(sub.to_string(), k + 1);
                            k = sub_end;
                        }
                        None => {
                            kept.push(k);
                            k += 1;
                        }
                    }
                }
                self.cut(&lines, &mut owned, kept, 4, fields, id)?;
                self.lines.insert(id.to_string(), i + 1);
                i = end;
            }
        }
        Ok(lines.iter().zip(&owned).filter(|(_, o)| !**o).map(|(l, _)| *l).collect())
    }

    /// 設計ノート 1 本（便 185）: meta の中の字下げ 2 の `id: ` の字と、sections の字下げ 2 の頭の行の block のうち字下げ 4 に
    /// `type: contract-table` を持つ節の、字下げ 4 の `rows:` の block の中の字下げ 6 の `- ` の行ごとに節点を 1 つ作る。行 id は
    /// 流れの形なら表の一番上の段の欄 id、塊の形なら頭の行か字下げ 8 の `id: ` の行の字（欄の順を問わない・辺の欄は字下げ 8）。
    fn note(&mut self, lines: &[&str], owned: &mut [bool]) -> Result<(), String> {
        let Some(meta) = meta_id(lines) else {
            return Ok(());
        };
        let (mut part, mut i) = (None, 0);
        while let Some(&line) = lines.get(i) {
            if !is_blank(line) && indent_of(line) == 0 {
                part = field_key(line);
            }
            if part != Some("sections") || !line.starts_with("  - ") {
                i += 1;
                continue;
            }
            let end = block_end(lines, i, 2, false);
            let at4 = |k: usize| {
                let l = lines.get(k)?;
                if k == i { l.get(4..) } else { l.strip_prefix("    ") }
            };
            let field = |k: usize, key: &str| at4(k).filter(|s| !s.starts_with(' ')).and_then(field_key) == Some(key);
            let table = (i..end).any(|k| at4(k).is_some_and(|s| s.trim_end() == format!("type: {CONTRACT_TABLE}")));
            if let Some(r) = (i..end).find(|k| table && field(*k, "rows")) {
                let (mut k, rows_end) = (r + 1, block_end(lines, r, 4, false).min(end));
                while k < rows_end {
                    let Some(rest) = lines.get(k).and_then(|l| l.strip_prefix("      - ")) else {
                        k += 1;
                        continue;
                    };
                    let flow = rest.starts_with('{');
                    let row_end = block_end(lines, k, 6, flow).min(rows_end);
                    let found = match flow {
                        true => flow_value(rest, "id").map(|id| (id, k)),
                        false => (k..row_end).find_map(|j| {
                            let pad = if j == k { "      - id: " } else { "        id: " };
                            lines.get(j)?.strip_prefix(pad).map(|id| (id.trim(), j))
                        }),
                    };
                    if let Some((rid, at)) = found {
                        let id = format!("{meta}#{rid}");
                        self.cut(lines, owned, (k..row_end).collect(), 8, EDGE_FIELDS[4].1, &id)?;
                        self.lines.insert(id, at + 1);
                    }
                    k = row_end;
                }
            }
            i = end;
        }
        Ok(())
    }

    /// 節点 1 つ: 行の番号の列 `idx`（入れ子を除いた block）から辺の欄（字下げ `depth` の行・より深い続きの行・
    /// 流れの形の対）と末尾の空行を落とし、残りを連結した byte の sha256 の先頭 8 字を要約値にする。
    fn cut(
        &mut self, lines: &[&str], owned: &mut [bool], mut idx: Vec<usize>, depth: usize, fields: &[&str], id: &str,
    ) -> Result<(), String> {
        while idx.pop_if(|n| lines.get(*n).is_some_and(|l| is_blank(l))).is_some() {}
        let whole: Vec<&str> = fields.iter().copied().filter(|f| !f.contains('.')).collect();
        let mut body: Vec<(usize, String)> = Vec::new();
        let mut rest = idx.into_iter().peekable();
        while let Some(n) = rest.next() {
            let Some(&line) = lines.get(n) else {
                continue;
            };
            mark(owned, n, true);
            let key = field_key(line);
            if !is_blank(line) && indent_of(line) == depth && key.is_some_and(|k| whole.contains(&k)) {
                // 辺の欄の行と、その後の空行・より深い続きの行（block の末尾の空行は先に落としてある）
                let deeper = |m: &usize| lines.get(*m).is_some_and(|l| is_blank(l) || indent_of(l) > depth);
                while let Some(m) = rest.next_if(deeper) {
                    mark(owned, m, true);
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
            body.push((n, text));
        }
        while let Some((n, _)) = body.pop_if(|(_, l)| is_blank(l)) {
            mark(owned, n, false);
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

/// 行の番号 `at` の持ち主の印を書く（範囲の外は何もしない）。
fn mark(owned: &mut [bool], at: usize, value: bool) {
    if let Some(o) = owned.get_mut(at) {
        *o = value;
    }
}

/// 流れの形の行の表の一番上の段の欄 `name` の値（頭は `{` か `, ` の直後・二重引用符の中は跳ばす・便 185）。
fn flow_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let (b, key) = (line.as_bytes(), format!("{name}: "));
    let (mut i, mut depth) = (0, 0usize);
    while let Some(&c) = b.get(i) {
        match c {
            b'"' => {
                i = skip_quoted(b, i);
                continue;
            }
            b'[' | b'{' => depth += 1,
            b']' | b'}' => depth = depth.saturating_sub(1),
            _ if depth == 1
                && (i > 0 && b.get(i - 1) == Some(&b'{') || b.get(..i).is_some_and(|h| h.ends_with(b", ")))
                && b.get(i..).is_some_and(|r| r.starts_with(key.as_bytes())) =>
            {
                let rest = &line[i + key.len()..];
                return Some(rest[..rest.find([',', '}']).unwrap_or(rest.len())].trim());
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// 設計ノートの meta の id（最上位の `meta:` の block の中の字下げ 2 の `id: ` の行の字・無ければ None・便 185）。
fn meta_id<'a>(lines: &[&'a str]) -> Option<&'a str> {
    let at = lines.iter().position(|l| field_key(l) == Some("meta") && indent_of(l) == 0)?;
    let end = block_end(lines, at, 0, false);
    lines
        .get(at + 1..end)
        .unwrap_or_default()
        .iter()
        .find_map(|l| l.strip_prefix("  id: "))
        .map(str::trim)
        .filter(|id| !id.is_empty())
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

/// 床の口（便 136 §1 (b) の 1）: `graph --print` と同じ build と Scan で節点の集合を組み、食い違いを
/// 種類 `INDEX_KIND` の違反に数える（索引が組めない置き場を床が合格と言わない・P-4.1）。索引を組めないときは、
/// 床がほかに何も数えていなければ「まだ分からない」を 1 件足す（読めない正本は床が先に数えている＝同じ原因を 2 度数えない）。
pub fn check_index(dir: &Path, report: &mut Report) {
    let silent = report.violations.is_empty() && report.unknowns.is_empty() && report.pendings.is_empty();
    let files = build(dir).and_then(|index| {
        let files = source_files(dir, &index)?;
        Ok((index, files))
    });
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
                    "{file}: 索引の節点 {id} の行を行の逐語で切れない（id か節の見出しの key が引用符つきか裸の形でない＝folio graph --print が組めない）"
                ),
            );
        }
    }
    for (id, file) in &scanned {
        // 設計ノートの行（便 185）は「#」の前後を別に外す（引用符つきの行 id を 2 度数えない）
        let bare = match id.split_once('#').filter(|_| file.starts_with(NOTE_DIR)) {
            Some((doc, row)) => format!("{}#{}", bare_id(doc), bare_id(row)),
            None => bare_id(id).to_string(),
        };
        if !index.nodes.contains_key(id) && !index_only.contains(bare.as_str()) {
            report.violation(
                INDEX_KIND,
                format!("{file}: 行の逐語の節点 {id} が索引の節点に無い（id が引用符つきか裸の形でない）"),
            );
        }
    }
}

/// 節点の数と表に出た辺の数だけを返す口（`folio hello` の 1 行が使う・組み方を 2 面に増やさない・便 96）。
pub fn counts(dir: &Path) -> Result<(usize, usize), String> {
    let index = whole(dir)?;
    let edges = index.split().0.len();
    Ok((index.nodes.len(), edges))
}

/// `folio graph --print` / `--digest` の結果。
pub struct Outcome {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub verdict: Verdict,
}

/// 組めたら標準出力の字（`digest` なら短い出力・`summary` なら節点ごとの 1 行の JSON・でなければ索引）と 合格、組めなければ
/// 表を出さずに「まだ分からない」。索引は節点の要約値の欄を持つ（便 99）: 索引の節点と行の逐語から切り出した節点が
/// 食い違えば表を出さない（P-4.1）。節点ごとの 1 行の id の行の番号も同じ行の逐語から取る（便 180）。
pub fn run(dir: &Path, digest: bool, summary: bool) -> Outcome {
    let built = whole(dir).and_then(|mut index| {
        if !digest {
            let mut scan = Scan::default();
            for name in source_files(dir, &index)? {
                scan.file(&name, &read_text(dir, &name)?)?;
            }
            index.lines = std::mem::take(&mut scan.lines);
            index.digests = scan.agree(&index)?;
        }
        Ok(index)
    });
    match built {
        Ok(index) => Outcome {
            stdout: Some(if digest {
                index.digest()
            } else if summary {
                index.jsonl()
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

    /// 便 195 の歯 5: 外の置き場の名で導出した生成区間でも ids と mentions の型付きの欄は folio2 の置き場と同じ字で、2 つの注は
    /// folio2 の番号の印を持つ括弧だけが落ちて残る（床の導出の規則 7・字面の置き換えで作った期待と比べる）。
    #[test]
    fn f195_abroad_region_keeps_the_lists_and_the_unmarked_notes() {
        use crate::floor::{HOME, derive_for, ids_in};
        let tail = |text: String| -> Vec<String> {
            text.lines().skip_while(|l| *l != "  ids:").map(str::to_string).collect()
        };
        let home = tail(derive_for(&FLOOR, Some(HOME)));
        let abroad = tail(derive_for(&FLOOR, Some("x-constitution")));
        assert_eq!(home.len(), 62, "{home:?}");
        assert_eq!(abroad.len(), home.len());
        for (h, a) in home.iter().zip(&abroad) {
            let Some((key, text)) = h.split_once(": ").filter(|(k, _)| k.ends_with("_note")) else {
                assert_eq!(h, a);
                continue;
            };
            let want = match key {
                "  ids_note" => text.replace("（行 R-4）", "").replace("（行 R-17）", "").replace("（P-5.1・P-5.6・行 D-11）", ""),
                "  mentions_note" => text.replace("（行 R-17）", "").replace("（P-5.6・行 D-11）", ""),
                other => panic!("注の欄 {other}"),
            };
            assert_eq!(*a, format!("{key}: {want}"));
            assert!(ids_in(a).is_empty() && !a.contains("決定 ("), "{a}");
            assert!(ids_in(h).len() >= 2, "folio2 の置き場の注が番号を持たない: {h}");
        }
    }

    /// 歯 6（便 195・検証役の非 blocking N1・変異 V3）: 実の graph.yaml の生成区間の ids と mentions の一覧は、正本の定数そのものと
    /// 字も順も同じ（床の木が定数を引かずに字を手で持つと、定数を変えた途端に落ちる）。
    #[test]
    fn f195_the_real_region_equals_the_constants() {
        let text = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../design-intent/graph.yaml")).unwrap();
        let root = yaml::parse(&text).unwrap().root;
        let schema = root.get("schema").unwrap();
        let strs = |node: &Node| -> Vec<String> {
            node.as_seq().unwrap().iter().map(|n| n.as_str().unwrap().to_string()).collect()
        };
        let ids = schema.get("ids").unwrap();
        let prefixes = ids.get("prefixes").unwrap();
        let m = schema.get("mentions").unwrap();
        let pairs: [(&Node, &[&str]); 11] = [
            (ids.get("rule_sections").unwrap(), &refs::RULE_SECTIONS),
            (ids.get("srs_sections").unwrap(), &refs::SRS_ID_SECTIONS),
            (ids.get("relation_namespaces").unwrap(), &refs::RELATION_NAMESPACES),
            (prefixes.get("article").unwrap(), &prose::ARTICLE),
            (prefixes.get("rule").unwrap(), &prose::RULE),
            (prefixes.get("srs").unwrap(), &refs::SRS_ID_PREFIXES),
            (m.get("targets").unwrap(), &mentions::TARGETS),
            (m.get("typed").unwrap(), &mentions::TYPED),
            (m.get("provenance").unwrap(), &mentions::PROVENANCE),
            (m.get("top_skipped").unwrap(), &mentions::TOP_SKIPPED),
            (m.get("excluded").unwrap(), &mentions::EXCLUDED),
        ];
        for (node, want) in pairs {
            assert_eq!(strs(node), want.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        }
    }
}
