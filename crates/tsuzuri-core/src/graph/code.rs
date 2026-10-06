//! code の層（判断の記録 ADR-46 の決定 (1)〜(4)）。git の file の一覧と、構文で探す道具（ast-grep）が規則の file で
//! 読んだ定義の stream と、契約表の導出物の write-set と、台帳の契約の bead の write-set の 4 つの字から、file と定義の
//! 節点と、行 → file の辺（宣言・write-set の項）を組む。file → 定義は定義の file の欄（測り・構文の範囲）で、行の触る定義は 2 つを辿って引く。
//! 定義の名前は path と入れ子と名と種類（`Def::id`）で、同じ名の関数と型を 1 つに潰さない。
//! 呼び手と参照の目と、commit と歯の節点はまだ持たない。字を読んで子 process を撃つ側は境界の crate が持つ。

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::build::read_ledger;

/// 定義の種類（閉じた 11・規則の file の rule の id の語と同じ字）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DefKind {
    Fn,
    Struct,
    Enum,
    Union,
    Trait,
    Type,
    Const,
    Static,
    Macro,
    Mod,
    Impl,
}

impl DefKind {
    pub const ALL: [DefKind; 11] = [
        DefKind::Fn,
        DefKind::Struct,
        DefKind::Enum,
        DefKind::Union,
        DefKind::Trait,
        DefKind::Type,
        DefKind::Const,
        DefKind::Static,
        DefKind::Macro,
        DefKind::Mod,
        DefKind::Impl,
    ];

    /// 種類の語（rule の id と名前の段の頭）。
    pub fn word(self) -> &'static str {
        match self {
            DefKind::Fn => "fn",
            DefKind::Struct => "struct",
            DefKind::Enum => "enum",
            DefKind::Union => "union",
            DefKind::Trait => "trait",
            DefKind::Type => "type",
            DefKind::Const => "const",
            DefKind::Static => "static",
            DefKind::Macro => "macro",
            DefKind::Mod => "mod",
            DefKind::Impl => "impl",
        }
    }

    /// rule の id の種類（`IMPL_FOR` も impl・知らない id は None）。
    pub fn of_rule(id: &str) -> Option<DefKind> {
        if id == IMPL_FOR {
            return Some(DefKind::Impl);
        }
        DefKind::ALL.into_iter().find(|k| k.word() == id)
    }
}

/// trait の impl の rule の id（名は「trait の名 for 型の名」）。
pub const IMPL_FOR: &str = "impl-for";

/// 名を捕える meta 変数の名。
pub const NAME_VAR: &str = "NAME";

/// trait の impl の trait の名を捕える meta 変数の名。
pub const TRAIT_VAR: &str = "TRAIT";

/// 読む言語の表（ast-grep の言語の語）。言語を足す時は規則の file の rule とこの表の行を足す。
pub const LANGS: [&str; 1] = ["Rust"];

/// 名前の段の区切り（外の定義から順）。
pub const NEST_SEP: char = '/';

/// file の path と名前の段の区切り。
pub const PATH_SEP: char = '#';

/// 定義の節点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Def {
    /// 名前（`<file>#<段>/<段>`・段は「種類の語 名」で、外の定義から自分まで）。
    pub id: String,
    pub file: String,
    pub kind: DefKind,
    pub name: String,
    /// 1 から数える行の範囲（頭と末・同じ名前の範囲が 2 つ以上在れば file の順に全部）。
    pub spans: Vec<(u32, u32)>,
}

/// stream から読んだ定義と、組まずに数えた行。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Defs {
    pub defs: Vec<Def>,
    /// 言語が表に無い・rule の id が種類に無い・捕えた名が無い行の数。
    pub skipped: usize,
}

/// write-set の項の印（素の path・`+` 新しい file・`-` 縮むが残る・`~` 着地で消える・`=` 置き場だけ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mark {
    Plain,
    New,
    Shrink,
    Gone,
    Place,
}

/// 行 → file の辺（宣言・write-set の項から）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Writes {
    pub row: String,
    pub file: String,
    pub mark: Mark,
    /// 辺を組んだ write-set の項の字（印と末の `/` を含む）。
    pub item: String,
}

/// code の層（repo に書かない・毎回組み直す）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeGraph {
    pub files: BTreeSet<String>,
    /// 定義の節点（file の字の順・file の中は頭の行の順）。
    pub defs: Vec<Def>,
    pub writes: Vec<Writes>,
    /// file の節点に当たらない write-set の項（行の id と項の字・置き場だけの項は数えない）。
    pub unbound: Vec<(String, String)>,
    /// stream の組まずに数えた行と、file の一覧に無い file の定義の数。
    pub skipped: usize,
}

impl CodeGraph {
    /// file の定義（頭の行の順）。
    pub fn file_defs(&self, file: &str) -> Vec<&Def> {
        self.defs.iter().filter(|d| d.file == file).collect()
    }

    /// 行の辺（write-set の項の順）。
    pub fn row_writes(&self, row: &str) -> Vec<&Writes> {
        self.writes.iter().filter(|w| w.row == row).collect()
    }

    /// 行の触る定義（行 → file → 定義・file の字の順）。
    pub fn row_defs(&self, row: &str) -> Vec<&Def> {
        let files: BTreeSet<&str> = self
            .row_writes(row)
            .iter()
            .map(|w| w.file.as_str())
            .collect();
        self.defs
            .iter()
            .filter(|d| files.contains(d.file.as_str()))
            .collect()
    }
}

/// ast-grep の stream の 1 行のうち読む欄（ほかの欄は読み捨てる）。
#[derive(Deserialize)]
struct Match {
    #[serde(rename = "ruleId")]
    rule_id: String,
    file: String,
    language: String,
    range: Range,
    #[serde(rename = "metaVariables")]
    meta: Meta,
}

#[derive(Deserialize)]
struct Range {
    #[serde(rename = "byteOffset")]
    bytes: Offsets,
    start: Pos,
    end: Pos,
}

#[derive(Deserialize)]
struct Offsets {
    start: usize,
    end: usize,
}

#[derive(Deserialize)]
struct Pos {
    line: u32,
}

#[derive(Deserialize)]
struct Meta {
    single: BTreeMap<String, Var>,
}

#[derive(Deserialize)]
struct Var {
    text: String,
}

/// 1 つの file の中の定義の候補（byte の範囲で入れ子を決める）。
struct Item {
    start: usize,
    end: usize,
    lines: (u32, u32),
    kind: DefKind,
    name: String,
}

/// 捕えた名（`IMPL_FOR` は「trait の名 for 型の名」・名が無ければ None）。
fn caught(m: &Match) -> Option<String> {
    let name = &m.meta.single.get(NAME_VAR)?.text;
    if m.rule_id != IMPL_FOR {
        return Some(name.clone());
    }
    let tr = &m.meta.single.get(TRAIT_VAR)?.text;
    Some(format!("{tr} for {name}"))
}

/// 1 つの file の候補を頭の順（同じ頭なら長い方が先）に並べ、包む候補を外の段にして名前を決める。
/// 同じ名前の候補は 1 つの節点に範囲を足す。
fn nest(file: &str, mut items: Vec<Item>) -> Vec<Def> {
    items.sort_by(|a, b| a.start.cmp(&b.start).then(b.end.cmp(&a.end)));
    let mut defs: Vec<Def> = Vec::new();
    let mut outer: Vec<(usize, String)> = Vec::new();
    let mut at: BTreeMap<String, usize> = BTreeMap::new();
    for item in items {
        while outer.last().is_some_and(|(end, _)| *end < item.end) {
            outer.pop();
        }
        let seg = format!("{} {}", item.kind.word(), item.name);
        let mut path: Vec<&str> = outer.iter().map(|(_, s)| s.as_str()).collect();
        path.push(&seg);
        let id = format!("{file}{PATH_SEP}{}", path.join(&NEST_SEP.to_string()));
        match at.get(&id).and_then(|&i| defs.get_mut(i)) {
            Some(def) => def.spans.push(item.lines),
            None => {
                at.insert(id.clone(), defs.len());
                defs.push(Def {
                    id,
                    file: file.to_string(),
                    kind: item.kind,
                    name: item.name,
                    spans: vec![item.lines],
                });
            }
        }
        outer.push((item.end, seg));
    }
    defs
}

/// ast-grep の scan の stream（1 行 1 つの JSON）から定義を読む。空の行は読み捨て、読めない行が 1 つでも在れば None。
/// 言語が `LANGS` に無い行・rule の id が種類に無い行・捕えた名の無い行は組まずに数える。
/// 入れ子は同じ file の byte の範囲の包みで決め（同じ頭なら長い方が外）、同じ名前の行は 1 つの節点に範囲を足す。
pub fn read_defs(stream: &str) -> Option<Defs> {
    let mut by_file: BTreeMap<String, Vec<Item>> = BTreeMap::new();
    let mut skipped = 0;
    for line in stream.lines().filter(|l| !l.trim().is_empty()) {
        let m: Match = serde_json::from_str(line).ok()?;
        let kind = DefKind::of_rule(&m.rule_id).filter(|_| LANGS.contains(&m.language.as_str()));
        let Some((kind, name)) = kind.zip(caught(&m)) else {
            skipped += 1;
            continue;
        };
        by_file.entry(m.file).or_default().push(Item {
            start: m.range.bytes.start,
            end: m.range.bytes.end,
            lines: (m.range.start.line + 1, m.range.end.line + 1),
            kind,
            name,
        });
    }
    let defs = by_file
        .into_iter()
        .flat_map(|(file, items)| nest(&file, items))
        .collect();
    Some(Defs { defs, skipped })
}

/// 契約表の導出物の字（folio derive が書く toml）から、行の id と write-set の項を表の順に読む。
/// 行は `[[contract]]` の行から次の `[[contract]]` の行の前までで、頭が `id = ` の行の字と、頭が `write-set = ` の行の
/// 値（字の配列の JSON として読む）を持つ。id の無い行か、write-set の値が字の配列でない行が在れば None。
pub fn read_write_sets(toml: &str) -> Option<Vec<(String, Vec<String>)>> {
    let mut rows: Vec<(Option<String>, Vec<String>)> = Vec::new();
    for line in toml.lines() {
        if line.trim_end() == "[[contract]]" {
            rows.push((None, Vec::new()));
        } else if let Some(row) = rows.last_mut() {
            if let Some(v) = line.strip_prefix("id = ") {
                row.0 = Some(serde_json::from_str(v).ok()?);
            } else if let Some(v) = line.strip_prefix("write-set = ") {
                row.1 = serde_json::from_str(v).ok()?;
            }
        }
    }
    rows.into_iter()
        .map(|(id, items)| Some((id?, items)))
        .collect()
}

/// 台帳の字（bd の読み取りの口の JSON の配列）から、契約の bead の行の id（`<bead の id>#<契約の id>`）と write-set の項を
/// 台帳の順に読む。bead の欄 acceptance の字（無ければ空の字）を `read_write_sets` で読み、契約の行を持たない bead は
/// 何も足さない。開閉は問わない。台帳が読めないか、読めない契約の行が 1 つでも在れば None。
pub fn bead_write_sets(ledger: &str) -> Option<Vec<(String, Vec<String>)>> {
    let mut out = Vec::new();
    for bead in read_ledger(ledger)? {
        let rows = read_write_sets(bead.acceptance_criteria.as_deref().unwrap_or(""))?;
        out.extend(
            rows.into_iter()
                .map(|(id, items)| (format!("{}{PATH_SEP}{id}", bead.id), items)),
        );
    }
    Some(out)
}

/// write-set の項の印と path（頭の 1 字が印の字なら印・ほかは素の path）。
pub fn mark(item: &str) -> (Mark, &str) {
    let marks = [
        ('+', Mark::New),
        ('-', Mark::Shrink),
        ('~', Mark::Gone),
        ('=', Mark::Place),
    ];
    for (c, m) in marks {
        if let Some(path) = item.strip_prefix(c) {
            return (m, path);
        }
    }
    (Mark::Plain, item)
}

/// file の一覧（git ls-files -z の字・NUL で区切る）と定義と行の write-set から code の層を組む。
/// 行の id は `<ノート>#<行>`。write-set の項は、末が `/` なら dir の下の file の全部、ほかは同じ path の file へ
/// 行 → file の辺を組む。置き場だけの項（`=`）は辺を組まない。file に当たらない項は `unbound` に数える。
/// 一覧に無い file の定義は組まずに数える。
pub fn build(files: &str, defs: Defs, rows: &[(String, Vec<String>)]) -> CodeGraph {
    let files: BTreeSet<String> = files
        .split('\0')
        .filter(|f| !f.is_empty())
        .map(str::to_string)
        .collect();
    let before = defs.defs.len();
    let kept: Vec<Def> = defs
        .defs
        .into_iter()
        .filter(|d| files.contains(&d.file))
        .collect();
    let mut g = CodeGraph {
        skipped: defs.skipped + before - kept.len(),
        defs: kept,
        ..CodeGraph::default()
    };
    for (row, items) in rows {
        for item in items {
            let (m, path) = mark(item);
            if m == Mark::Place {
                continue;
            }
            let hits: Vec<&String> = match path.strip_suffix('/') {
                Some(_) => files.iter().filter(|f| f.starts_with(path)).collect(),
                None => files.get(path).into_iter().collect(),
            };
            if hits.is_empty() {
                g.unbound.push((row.clone(), item.clone()));
            }
            for file in hits {
                g.writes.push(Writes {
                    row: row.clone(),
                    file: file.clone(),
                    mark: m,
                    item: item.clone(),
                });
            }
        }
    }
    g.files = files;
    g
}
