//! `folio schema`（便 45・docs/design/delivery-45.md §1・ADR-9・FR19）。欄の決まりの file の schema 節（生成区間）を
//! 床の定数から導出して書く（--write）・検査する（--check）。憲法の生成区間（`inject.rs`）と同じ型 = 印 2 本・区間を取る
//! 関数・導出 1 関数を --write / --check が共有し、終了コードは 合格 0 / 不合格 1 / まだ分からない 2。
//!
//! 床の機械（床の木の型 `Floor`・マクロ `keys_floor`・`strip_notes`・`floor_diff`）もここに置き、`adr.rs` と
//! `note.rs`（便 46）が使う。`folio check` の置き場ごとの検査は印を見ない（fixture の写しは印を持たない）。
//!
//! 導出の体裁（§1 (c)・W = 100 字・字数は Unicode の字の数・行は字下げ込み）:
//! 1. 表の block の子は「<字下げ><キー>: <値>」・字下げは深さ × 2・キーの順は FLOOR の順。
//! 2. 文字列と数の一覧: flow の 1 行が W 以内なら flow、超えれば block（各項「<字下げ + 2>- <値>」）。空は []。
//! 3. 表: 子が全部 文字列・数・真偽 か その一覧 で flow の 1 行が W 以内なら flow、さもなくば block。空は {}。
//! 4. 表の一覧: 各項を 3 の flow で「<字下げ + 2>- {…}」、W を超える項は block（1 つ目のキーを「- 」の後ろに）。
//! 5. 値の字面: 数と真偽は裸。文字列は yaml で裸にできない形のときだけ単引用符（中の単引用符は 2 つ重ねる）。

use std::fs;
use std::path::Path;

use crate::verdict::Verdict;
use crate::yaml::Node;

/// 生成区間の印（行の全部がこの字面・前後に空白なし）。
pub const BEGIN: &str = "# folio:schema:begin — 生成区間・手で直さない・正本は実装の定数（folio schema --write が書く）";
pub const END: &str = "# folio:schema:end";

/// 1 行の幅の上限（Unicode の字の数）。
const WIDTH: usize = 100;

/// 命令が扱う file と、その schema 節の正本（床の定数）。この順に見て、最初に合格でない file で返す。
/// 3 本目は天井の正本（便 48・ADR-11 決定 (4)①・生成区間は file の末尾に 1 対）。
/// 4 本目は規則の表（便 53・ADR-11 決定 (4)②・生成区間は file の先頭の注釈の次に 1 対）。
const TARGETS: &[(&str, &Floor)] = &[
    ("adr/schema.yaml", &crate::adr::FLOOR),
    ("design-note/schema.yaml", &crate::note::FLOOR),
    ("ceiling.yaml", &crate::ceiling::FLOOR),
    ("rules.yaml", &crate::rules::FLOOR),
];

// ── 床の機械 ──

/// 床の定数の木（欄の決まりの file の schema 節と同じ形）。
#[derive(Debug)]
pub(crate) enum Floor {
    /// 値（yaml の値の字面・引用符を除く）
    Val(&'static str),
    /// 数（字面で比べる＝2.0 と 2 は違う）
    Num(usize),
    /// 値の一覧
    Strs(&'static [&'static str]),
    /// 木の一覧（順も比べる）。設計ノートの側（landing.verdict_cases）が使う
    Seq(&'static [Floor]),
    /// 表（欄の順は schema 節の順）
    Map(&'static [(&'static str, Floor)]),
}

/// 欄の集合（required / optional）を床の木の表に写す。
macro_rules! keys_floor {
    ($keys:expr) => {
        $crate::schema::Floor::Map(&[
            ("required", $crate::schema::Floor::Strs($keys.required)),
            ("optional", $crate::schema::Floor::Strs($keys.optional)),
        ])
    };
}
pub(crate) use keys_floor;

/// 名前が `_note` で終わる欄を（入れ子の表の中も含めて）落とす。
pub(crate) fn strip_notes(node: &Node) -> Node {
    match node {
        Node::Map(entries) => Node::Map(
            entries
                .iter()
                .filter(|(k, _)| !k.ends_with("_note"))
                .map(|(k, v)| (k.clone(), strip_notes(v)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// 写しと床の定数の違いを欄の道で並べる。床の側の `_note` の欄（説明の注）は突き合わせの外（data 側は strip_notes 済み）。
pub(crate) fn floor_diff(data: &Node, floor: &Floor, path: &str, out: &mut Vec<String>) {
    match floor {
        Floor::Map(fields) => {
            let Some(entries) = data.as_map() else {
                out.push(format!(
                    "{}（欄の表でない）",
                    if path.is_empty() { "schema" } else { path }
                ));
                return;
            };
            let mut keys: Vec<&str> = entries
                .iter()
                .map(|(k, _)| k.as_str())
                .chain(
                    fields
                        .iter()
                        .map(|(k, _)| *k)
                        .filter(|k| !k.ends_with("_note")),
                )
                .collect();
            keys.sort_unstable();
            keys.dedup();
            for key in keys {
                let p = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };
                match (fields.iter().find(|(k, _)| *k == key), data.get(key)) {
                    (None, _) => out.push(format!(
                        "{p}（未知の欄＝機械が読まない欄は *_note で終える）"
                    )),
                    (Some(_), None) => out.push(format!("{p}（欠落）")),
                    (Some((_, f)), Some(d)) => floor_diff(d, f, &p, out),
                }
            }
        }
        Floor::Seq(items) => match data.as_seq() {
            Some(seq) if seq.len() == items.len() => {
                for (i, (d, f)) in seq.iter().zip(items.iter()).enumerate() {
                    floor_diff(d, f, &format!("{path}[{i}]"), out);
                }
            }
            _ => out.push(path.to_string()),
        },
        Floor::Strs(items) => match data.as_seq() {
            Some(seq) if seq.len() == items.len() => {
                for (i, (d, v)) in seq.iter().zip(items.iter()).enumerate() {
                    if d.as_str() != Some(v) {
                        out.push(format!("{path}[{i}]"));
                    }
                }
            }
            _ => out.push(path.to_string()),
        },
        Floor::Val(v) => {
            if data.as_str() != Some(v) {
                out.push(path.to_string());
            }
        }
        Floor::Num(n) => {
            if data.as_str() != Some(n.to_string().as_str()) {
                out.push(path.to_string());
            }
        }
    }
}

// ── 導出（§1 (c)） ──

/// 規則 5: 文字列の字面。裸にできない形のときだけ単引用符で囲む（`flow` = flow の中では読点・括弧も囲む理由）。
fn quoted(s: &str, flow: bool) -> String {
    let quote = s.is_empty()
        || s.starts_with(char::is_whitespace)
        || s.ends_with(char::is_whitespace)
        || s.starts_with([
            '[', '{', '#', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`',
        ])
        || s.starts_with("- ")
        || s.starts_with("? ")
        || s.starts_with(": ")
        || s.contains(": ")
        || s.contains(" #")
        || s.ends_with(':')
        || (flow && s.contains([',', '[', ']', '{', '}']));
    if quote {
        format!("'{}'", s.replace('\'', "''"))
    } else {
        s.to_string()
    }
}

/// 木を flow の 1 行に。
fn flow(node: &Floor) -> String {
    match node {
        Floor::Val(v) => quoted(v, true),
        Floor::Num(n) => n.to_string(),
        Floor::Strs(items) => {
            let items: Vec<String> = items.iter().map(|v| quoted(v, true)).collect();
            format!("[{}]", items.join(", "))
        }
        Floor::Seq(items) => {
            let items: Vec<String> = items.iter().map(flow).collect();
            format!("[{}]", items.join(", "))
        }
        Floor::Map(fields) => {
            let fields: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{k}: {}", flow(v)))
                .collect();
            format!("{{{}}}", fields.join(", "))
        }
    }
}

/// 規則 3 の flow の条件: 子が全部 文字列・数・真偽 か その一覧。
fn flat(fields: &[(&str, Floor)]) -> bool {
    fields
        .iter()
        .all(|(_, v)| matches!(v, Floor::Val(_) | Floor::Num(_) | Floor::Strs(_)))
}

fn fits(line: &str) -> bool {
    line.chars().count() <= WIDTH
}

/// 表を block で書く（各子を 1 行以上・字下げ `indent`）。
fn block_map(fields: &[(&str, Floor)], indent: usize, out: &mut Vec<String>) {
    let pad = " ".repeat(indent);
    for (key, value) in fields {
        match value {
            Floor::Val(v) => out.push(format!("{pad}{key}: {}", quoted(v, false))),
            Floor::Num(n) => out.push(format!("{pad}{key}: {n}")),
            Floor::Strs(items) => {
                let line = format!("{pad}{key}: {}", flow(value));
                if items.is_empty() || fits(&line) {
                    out.push(line);
                } else {
                    out.push(format!("{pad}{key}:"));
                    for item in items.iter() {
                        out.push(format!("{pad}  - {}", quoted(item, false)));
                    }
                }
            }
            Floor::Map(sub) => {
                let line = format!("{pad}{key}: {}", flow(value));
                if sub.is_empty() || (flat(sub) && fits(&line)) {
                    out.push(line);
                } else {
                    out.push(format!("{pad}{key}:"));
                    block_map(sub, indent + 2, out);
                }
            }
            Floor::Seq(items) => {
                if items.is_empty() {
                    out.push(format!("{pad}{key}: []"));
                } else {
                    out.push(format!("{pad}{key}:"));
                    for item in items.iter() {
                        block_item(item, indent + 2, out);
                    }
                }
            }
        }
    }
}

/// 規則 4: 一覧の項 1 つ（「- 」の行から）。表の項は flow が W に収まればその 1 行、さもなくば block
/// （1 つ目のキーを「- 」の後ろに、残りのキーを同じ列に）。
fn block_item(item: &Floor, indent: usize, out: &mut Vec<String>) {
    let pad = " ".repeat(indent);
    match item {
        Floor::Map(sub) if !sub.is_empty() => {
            let line = format!("{pad}- {}", flow(item));
            if flat(sub) && fits(&line) {
                out.push(line);
                return;
            }
            let first = out.len();
            block_map(sub, indent + 2, out);
            out[first].replace_range(indent..indent + 2, "- ");
        }
        Floor::Val(v) => out.push(format!("{pad}- {}", quoted(v, false))),
        other => out.push(format!("{pad}- {}", flow(other))),
    }
}

/// 床の定数 → 生成区間の本文（決定的）。「schema:」の行 + 規則で組んだ本体・各行の末尾は改行 1 つ。
pub fn derive(floor: &Floor) -> String {
    let mut lines = vec!["schema:".to_string()];
    match floor {
        Floor::Map(fields) => block_map(fields, 2, &mut lines),
        other => lines[0] = format!("schema: {}", flow(other)),
    }
    lines.iter().map(|l| format!("{l}\n")).collect()
}

// ── 区間と命令 ──

/// 生成区間（begin の行の改行の次の byte から end の行の先頭まで）。begin と end の行がそれぞれちょうど 1 本で
/// begin が先のときだけ定まる。印は行の全部が字面のとおりの行だけを数える。
pub fn region_of(text: &str) -> Option<(usize, usize)> {
    let mut begins = Vec::new();
    let mut ends = Vec::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        if body == BEGIN {
            begins.push(at + line.len());
        }
        if body == END {
            ends.push(at);
        }
        at += line.len();
    }
    match (begins.as_slice(), ends.as_slice()) {
        ([b], [e]) if b <= e => Some((*b, *e)),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Write,
    Check,
}

/// 1 回の実行の結果。`stdout` は file ごとの 1 行、`stderr` は理由（「folio schema: 」は口が付ける）。
#[derive(Debug)]
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Vec<String>,
    pub stderr: Option<String>,
}

impl Outcome {
    fn refused(verdict: Verdict, msg: impl Into<String>) -> Self {
        Outcome {
            verdict,
            stdout: Vec::new(),
            stderr: Some(msg.into()),
        }
    }
}

/// 対象の file 1 本を --write / --check に掛ける。合格なら標準出力の 1 行を返す。
fn run_one(dir: &Path, file: &str, floor: &Floor, mode: Mode) -> Result<String, Outcome> {
    let path = dir.join(file);
    if path.is_symlink() || !path.is_file() {
        return Err(Outcome::refused(
            Verdict::Unknown,
            format!("{file}: 読めない"),
        ));
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| Outcome::refused(Verdict::Unknown, format!("{file}: 読めない: {e}")))?;
    let Some((start, end)) = region_of(&text) else {
        return Err(Outcome::refused(
            Verdict::Unknown,
            format!("{file}: 印が 1 対でない"),
        ));
    };
    let want = derive(floor);
    let cur = &text[start..end];
    let size = want.len();
    if cur == want {
        return Ok(match mode {
            Mode::Check => format!("folio schema: 一致（{file}・{size} byte）"),
            Mode::Write => format!("folio schema: 変わらない（{file}・{size} byte）"),
        });
    }
    if mode == Mode::Check {
        return Err(Outcome::refused(
            Verdict::Fail,
            format!("{file}: 生成区間 {} byte ≠ 導出 {size} byte", cur.len()),
        ));
    }
    let new = format!("{}{want}{}", &text[..start], &text[end..]);
    fs::write(&path, new)
        .map_err(|e| Outcome::refused(Verdict::Unknown, format!("{file}: 書けない: {e}")))?;
    Ok(format!("folio schema: 書いた（{file}・{size} byte）"))
}

pub fn run(dir: &Path, mode: Mode) -> Outcome {
    let mut stdout = Vec::new();
    for (file, floor) in TARGETS {
        match run_one(dir, file, floor, mode) {
            Ok(line) => stdout.push(line),
            Err(outcome) => return outcome,
        }
    }
    Outcome {
        verdict: Verdict::Pass,
        stdout,
        stderr: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml;

    #[test]
    fn schema_floor_diff_compares_literally_and_skips_floor_notes() {
        let doc = yaml::parse("min: '2'\nadopted: 2.0\nextra: x\n").unwrap();
        let mut out = Vec::new();
        floor_diff(
            &doc.root,
            &Floor::Map(&[
                ("min", Floor::Num(2)),
                ("min_note", Floor::Val("説明")),
                ("adopted", Floor::Num(1)),
                ("x", Floor::Strs(&[])),
            ]),
            "options_rule",
            &mut out,
        );
        assert_eq!(
            out,
            [
                "options_rule.adopted",
                "options_rule.extra（未知の欄＝機械が読まない欄は *_note で終える）",
                "options_rule.x（欠落）"
            ]
        );
    }

    #[test]
    fn schema_quoting_follows_rule_5() {
        assert_eq!(quoted("持ち主", false), "持ち主");
        assert_eq!(quoted("[a-z]", false), "'[a-z]'");
        assert_eq!(quoted("", false), "''");
        assert_eq!(quoted("a: b", false), "'a: b'");
        assert_eq!(quoted("a #b", false), "'a #b'");
        assert_eq!(quoted("a:", false), "'a:'");
        assert_eq!(quoted("- a", false), "'- a'");
        assert_eq!(quoted("it's", true), "it's");
        assert_eq!(quoted("'it's", true), "'''it''s'");
        assert_eq!(quoted("a, b", false), "a, b");
        assert_eq!(quoted("a, b", true), "'a, b'");
        assert_eq!(quoted("x {y}", false), "x {y}");
        assert_eq!(quoted("x {y}", true), "'x {y}'");
    }

    #[test]
    fn schema_layout_switches_flow_and_block_at_the_width() {
        const LONG: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz";
        const F: Floor = Floor::Map(&[
            ("n", Floor::Num(2)),
            ("s", Floor::Strs(&["a", "b"])),
            ("e", Floor::Strs(&[])),
            ("m", Floor::Map(&[])),
            ("wide", Floor::Strs(&[LONG, LONG])),
            (
                "nest",
                Floor::Map(&[("k", Floor::Val("v")), ("t", Floor::Map(&[]))]),
            ),
            (
                "rows",
                Floor::Seq(&[
                    Floor::Map(&[("a", Floor::Val("1")), ("b", Floor::Num(2))]),
                    Floor::Map(&[("a", Floor::Val(LONG)), ("b", Floor::Val(LONG))]),
                ]),
            ),
        ]);
        let want = format!(
            "schema:\n  n: 2\n  s: [a, b]\n  e: []\n  m: {{}}\n  wide:\n    - {LONG}\n    - {LONG}\n  nest:\n    k: v\n    t: {{}}\n  rows:\n    - {{a: 1, b: 2}}\n    - a: {LONG}\n      b: {LONG}\n"
        );
        assert_eq!(derive(&F), want);
    }

    #[test]
    fn schema_region_needs_one_pair_of_marker_lines_in_order() {
        let text = format!("meta: x\n{BEGIN}\nschema:\n  a: 1\n{END}\nplain: y\n");
        let (s, e) = region_of(&text).unwrap();
        assert_eq!(&text[s..e], "schema:\n  a: 1\n");
        assert_eq!(region_of(&format!("{END}\n{BEGIN}\n")), None);
        assert_eq!(region_of(&format!("{BEGIN}\n{END}\n{END}\n")), None);
        assert_eq!(region_of(&format!(" {BEGIN}\n{END}\n")), None);
        assert_eq!(region_of(&format!("{BEGIN}\n")), None);
        assert_eq!(
            region_of(&format!("{BEGIN}\n{END}")),
            Some((BEGIN.len() + 1, BEGIN.len() + 1))
        );
    }
}
