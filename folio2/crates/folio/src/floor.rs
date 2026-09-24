//! 欄の決まりの床の機械（便 106・docs/design/delivery-106.md §1 (b)・ADR-15 決定 (2)(3)・責務の層 1 読む）。
//! 床の定数の木の型 `Floor`・欄の集合を木に写すマクロ `keys_floor`・注釈の欄を落とす `strip_notes`・正本の木と
//! 突き合わせる `floor_diff`・木から生成区間の本文を組む `derive` を持つ。`schema.rs`（便 45）から字を変えずに降ろした。
//! 正本の形だけを知り、何も書かない。床の検査の側（`adr.rs`・`note.rs` ほか）と `folio schema` の口が使う。
//! 生成区間の印と区間を取る口は `schema.rs` に残る。
//!
//! 導出の体裁（§1 (c)・W = 100 字・字数は Unicode の字の数・行は字下げ込み）:
//! 1. 表の block の子は「<字下げ><キー>: <値>」・字下げは深さ × 2・キーの順は FLOOR の順。
//! 2. 文字列と数の一覧: flow の 1 行が W 以内なら flow、超えれば block（各項「<字下げ + 2>- <値>」）。空は []。
//! 3. 表: 子が全部 文字列・数・真偽 か その一覧 で flow の 1 行が W 以内なら flow、さもなくば block。空は {}。
//! 4. 表の一覧: 各項を 3 の flow で「<字下げ + 2>- {…}」、W を超える項は block（1 つ目のキーを「- 」の後ろに）。
//! 5. 値の字面: 数と真偽は裸。文字列は yaml で裸にできない形のときだけ単引用符（中の単引用符は 2 つ重ねる）。
//! 6. 置き場の名で行を選ぶ表（`Floor::Pick`・便 121・ADR-16 決定 (2)(ア)）: 写すのは鍵が置き場の名と等しい行だけ。
//!    行が無ければ {}、在れば flow の 1 行が W 以内なら flow、超えれば block（欄名の行と「<字下げ + 2><名>: <値>」の行）。

use crate::yaml::Node;

/// 1 行の幅の上限（Unicode の字の数）。
const WIDTH: usize = 100;

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
    /// 置き場の名で行を選ぶ表（鍵 = 置き場の名・値 = 字）。写しと突き合わせは鍵が置き場の名と等しい行だけ
    /// （0 行か 1 行・名が無いか表に無ければ空の表）。判断の記録の列の根の表（便 121）が使う
    Pick(&'static [(&'static str, &'static str)]),
}

/// 置き場の名で選んだ行（0 行か 1 行）。
fn picked(
    rows: &'static [(&'static str, &'static str)],
    name: Option<&str>,
) -> Vec<(&'static str, &'static str)> {
    rows.iter()
        .filter(|(k, _)| Some(*k) == name)
        .copied()
        .collect()
}

/// 欄の集合（required / optional）を床の木の表に写す。
macro_rules! keys_floor {
    ($keys:expr) => {
        $crate::floor::Floor::Map(&[
            ("required", $crate::floor::Floor::Strs($keys.required)),
            ("optional", $crate::floor::Floor::Strs($keys.optional)),
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
/// 置き場の名を持たない口＝名で行を選ぶ表は空の表と突き合わせる（`floor_diff_for` の名なし）。
pub(crate) fn floor_diff(data: &Node, floor: &Floor, path: &str, out: &mut Vec<String>) {
    floor_diff_for(data, floor, None, path, out);
}

/// 名つきの突き合わせ（便 121）。`name` は検査される置き場の憲法の名（meta.id）で、名で行を選ぶ表はその行だけと比べる。
/// 余分な行・欠けた行・値の違い・表でない値はどれも表の道 1 つ。
pub(crate) fn floor_diff_for(
    data: &Node,
    floor: &Floor,
    name: Option<&str>,
    path: &str,
    out: &mut Vec<String>,
) {
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
                    (Some((_, f)), Some(d)) => floor_diff_for(d, f, name, &p, out),
                }
            }
        }
        Floor::Seq(items) => match data.as_seq() {
            Some(seq) if seq.len() == items.len() => {
                for (i, (d, f)) in seq.iter().zip(items.iter()).enumerate() {
                    floor_diff_for(d, f, name, &format!("{path}[{i}]"), out);
                }
            }
            _ => out.push(path.to_string()),
        },
        Floor::Pick(rows) => {
            let want = picked(rows, name);
            let same = data.as_map().is_some_and(|entries| {
                entries.len() == want.len()
                    && want
                        .iter()
                        .all(|(k, v)| data.get(k).and_then(Node::as_str) == Some(*v))
            });
            if !same {
                out.push(path.to_string());
            }
        }
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

/// 木を flow の 1 行に（`name` は置き場の名＝名で行を選ぶ表の行を選ぶ）。
fn flow(node: &Floor, name: Option<&str>) -> String {
    match node {
        Floor::Val(v) => quoted(v, true),
        Floor::Num(n) => n.to_string(),
        Floor::Strs(items) => {
            let items: Vec<String> = items.iter().map(|v| quoted(v, true)).collect();
            format!("[{}]", items.join(", "))
        }
        Floor::Seq(items) => {
            let items: Vec<String> = items.iter().map(|x| flow(x, name)).collect();
            format!("[{}]", items.join(", "))
        }
        Floor::Map(fields) => {
            let fields: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{k}: {}", flow(v, name)))
                .collect();
            format!("{{{}}}", fields.join(", "))
        }
        Floor::Pick(rows) => {
            let rows: Vec<String> = picked(rows, name)
                .iter()
                .map(|(k, v)| format!("{k}: {}", quoted(v, true)))
                .collect();
            format!("{{{}}}", rows.join(", "))
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
fn block_map(fields: &[(&str, Floor)], indent: usize, name: Option<&str>, out: &mut Vec<String>) {
    let pad = " ".repeat(indent);
    for (key, value) in fields {
        match value {
            Floor::Val(v) => out.push(format!("{pad}{key}: {}", quoted(v, false))),
            Floor::Num(n) => out.push(format!("{pad}{key}: {n}")),
            Floor::Strs(items) => {
                let line = format!("{pad}{key}: {}", flow(value, name));
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
                let line = format!("{pad}{key}: {}", flow(value, name));
                if sub.is_empty() || (flat(sub) && fits(&line)) {
                    out.push(line);
                } else {
                    out.push(format!("{pad}{key}:"));
                    block_map(sub, indent + 2, name, out);
                }
            }
            Floor::Seq(items) => {
                if items.is_empty() {
                    out.push(format!("{pad}{key}: []"));
                } else {
                    out.push(format!("{pad}{key}:"));
                    for item in items.iter() {
                        block_item(item, indent + 2, name, out);
                    }
                }
            }
            Floor::Pick(rows) => {
                let rows = picked(rows, name);
                let line = format!("{pad}{key}: {}", flow(value, name));
                if rows.is_empty() || fits(&line) {
                    out.push(line);
                } else {
                    out.push(format!("{pad}{key}:"));
                    for (k, v) in rows {
                        out.push(format!("{pad}  {k}: {}", quoted(v, false)));
                    }
                }
            }
        }
    }
}

/// 規則 4: 一覧の項 1 つ（「- 」の行から）。表の項は flow が W に収まればその 1 行、さもなくば block
/// （1 つ目のキーを「- 」の後ろに、残りのキーを同じ列に）。
fn block_item(item: &Floor, indent: usize, name: Option<&str>, out: &mut Vec<String>) {
    let pad = " ".repeat(indent);
    match item {
        Floor::Map(sub) if !sub.is_empty() => {
            let line = format!("{pad}- {}", flow(item, name));
            if flat(sub) && fits(&line) {
                out.push(line);
                return;
            }
            let first = out.len();
            block_map(sub, indent + 2, name, out);
            out[first].replace_range(indent..indent + 2, "- ");
        }
        Floor::Val(v) => out.push(format!("{pad}- {}", quoted(v, false))),
        other => out.push(format!("{pad}- {}", flow(other, name))),
    }
}

/// 床の定数 → 生成区間の本文（決定的）。「schema:」の行 + 規則で組んだ本体・各行の末尾は改行 1 つ。
/// 置き場の名を持たない口＝名で行を選ぶ表は空の表（`derive_for` の名なしと同じ字）。
/// 命令の口（`schema.rs`）は名つきの導出を呼ぶので、この口を読むのは各床の単体の歯だけ。
#[cfg_attr(not(test), allow(dead_code))]
pub fn derive(floor: &Floor) -> String {
    derive_for(floor, None)
}

/// 名つきの導出（便 121・ADR-16 決定 (2)(ア)）。`name` は置き場の憲法の名（meta.id）で、名で行を選ぶ表はその行だけを写す。
pub fn derive_for(floor: &Floor, name: Option<&str>) -> String {
    let mut lines = vec!["schema:".to_string()];
    match floor {
        Floor::Map(fields) => block_map(fields, 2, name, &mut lines),
        other => lines[0] = format!("schema: {}", flow(other, name)),
    }
    lines.iter().map(|l| format!("{l}\n")).collect()
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

    /// 便 121 の歯 8: 置き場の名で行を選ぶ表の導出と突き合わせ。行 a は flow の 1 行・行 b は幅を超えて block の 2 行・
    /// 表に無い名と名なしは空の表の 1 行（名なしは `derive` と同じ字）。突き合わせは名の行だけと比べる。
    #[test]
    fn f121_pick_table_derives_and_diffs_only_the_named_row() {
        const WIDE: &str =
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        const F: Floor = Floor::Map(&[
            ("n", Floor::Num(1)),
            ("roots", Floor::Pick(&[("a", "short"), ("b", WIDE)])),
        ]);
        assert_eq!(
            derive_for(&F, Some("a")),
            "schema:\n  n: 1\n  roots: {a: short}\n"
        );
        assert_eq!(
            derive_for(&F, Some("b")),
            format!("schema:\n  n: 1\n  roots:\n    b: {WIDE}\n")
        );
        let empty = "schema:\n  n: 1\n  roots: {}\n";
        assert_eq!(derive_for(&F, Some("c")), empty);
        assert_eq!(derive_for(&F, None), empty);
        assert_eq!(derive(&F), empty);

        let diff = |text: &str, name: Option<&str>| {
            let doc = yaml::parse(text).unwrap();
            let mut out = Vec::new();
            floor_diff_for(&doc.root, &F, name, "", &mut out);
            out
        };
        assert!(diff("n: 1\nroots: {a: short}\n", Some("a")).is_empty());
        assert!(diff(&format!("n: 1\nroots:\n  b: {WIDE}\n"), Some("b")).is_empty());
        assert!(diff("n: 1\nroots: {}\n", Some("c")).is_empty());
        assert!(diff("n: 1\nroots: {}\n", None).is_empty());
        for (text, name) in [
            ("n: 1\nroots: {a: short}\n", Some("c")),
            ("n: 1\nroots: {a: short}\n", None),
            ("n: 1\nroots: {a: long}\n", Some("a")),
            ("n: 1\nroots: {a: short, b: x}\n", Some("a")),
            ("n: 1\nroots: {}\n", Some("a")),
            ("n: 1\nroots: short\n", Some("a")),
        ] {
            assert_eq!(diff(text, name), ["roots"], "{text} {name:?}");
        }
    }
}
