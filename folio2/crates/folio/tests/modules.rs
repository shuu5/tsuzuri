//! 便 106（docs/design/delivery-106.md §1 (e)・ADR-15 決定 (4)）: 実装の区切りの境界の歯。
//! 層の割り当ての表と、まだ降ろしていない層が上がる辺の凍結した一覧を型付きデータで持ち、`src/` の実測と
//! ちょうど一致することを数える。集合の一致なので辺を足しても減らしても落ちる。無効化の旗も 今回だけ の口も持たない
//! （N-3.1）。この表と一覧は実装と同じ便が書き直せるので、独立した凍結 anchor ではない（凍結した実測・§1 (h) の 1）。
//!
//! 走査の式（設計ノート docs/design/module-map-2026-09-23.md の冒頭）: 注釈（`//` と `/* */`）と字面（文字列・素の
//! 文字列・1 文字）を剥がしてから、`crate::<区切り名>` と `use crate::{a, b}` の括り書きを解いて相異なる名を取る。
//! 区切りを宣言する `lib.rs` だけは接頭辞の無い `<区切り名>::` も数える。自分自身は数えない。
#![cfg(test)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// 層の割り当て（責務・ADR-15 決定 (2)・設計ノート §1・§2）。新しい区切りは必ずここに 1 行を持つ。
/// 0 土台・1 読む・2 検査する・3 導出する・4 面に出す・5 配る。file は自分より下の層だけを名指す（同じ層は許す）。
const LAYERS: &[(&str, u8)] = &[
    ("sha256", 0),
    ("verdict", 0),
    ("catalog", 1),
    ("ceiling_src", 1),
    ("constitution_enums", 1),
    ("cursor", 1),
    ("floor", 1),
    ("floor_adr", 1),
    ("floor_note", 1),
    ("phase", 1),
    ("ruling", 1),
    ("shelf", 1),
    ("yaml", 1),
    ("adr", 2),
    ("anchor", 2),
    ("ceiling", 2),
    ("check", 2),
    ("entrance", 2),
    ("findings", 2),
    ("gate", 2),
    ("gitcheck", 2),
    ("ids", 2),
    ("intake", 2),
    ("lineage", 2),
    ("link", 2),
    ("mentions", 2),
    ("note", 2),
    ("parts", 2),
    ("plan", 2),
    ("polarity", 2),
    ("prose", 2),
    ("refs", 2),
    ("rules", 2),
    ("seal", 2),
    ("vocab", 2),
    ("bundle", 3),
    ("derive", 3),
    ("freeze", 3),
    ("graph", 3),
    ("init", 3),
    ("inject", 3),
    ("proposed", 3),
    ("schema", 3),
    ("sheet", 3),
    ("stamp", 3),
    ("face", 4),
    ("face_adr", 4),
    ("face_constitution", 4),
    ("face_constitution_read", 4),
    ("face_index", 4),
    ("face_index_read", 4),
    ("face_labels", 4),
    ("face_note", 4),
    ("face_srs", 4),
    ("face_srs_items", 4),
    ("face_srs_rtm", 4),
    ("figure", 4),
    ("entry", 5),
    ("hello", 5),
    ("lib", 5),
    ("main", 5),
    ("serve", 5),
    ("site", 5),
];

/// まだ降ろしていない層が上がる辺（名指す側, 名指される側）。凍結した実測であって許可ではない。
/// 行を足す書き換えは認めない（足すなら ADR-15 を改訂する）。ADR-15 の列（便 107〜112）が 1 本ずつ消す。
const REMAINING_UPWARD: &[(&str, &str)] = &[];

fn src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read(name: &str) -> String {
    let p = src_dir().join(format!("{name}.rs"));
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} を読めない: {e}", p.display()))
}

fn is_ident(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// 注釈（`//`・`/* */`〔入れ子も〕）と字面（文字列・素の文字列・1 文字）を剥がす。剥がした所は空白 1 つにする。
/// 寿命の印（`'a`）は字面でないので残す。
fn strip(src: &str) -> String {
    let c: Vec<char> = src.chars().collect();
    let at = |k: usize| c.get(k).copied();
    let mut out = String::new();
    let mut i = 0;
    while i < c.len() {
        if c[i] == '/' && at(i + 1) == Some('/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c[i] == '/' && at(i + 1) == Some('*') {
            let mut depth = 0;
            while i < c.len() {
                if c[i] == '/' && at(i + 1) == Some('*') {
                    depth += 1;
                    i += 2;
                } else if c[i] == '*' && at(i + 1) == Some('/') {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            out.push(' ');
            continue;
        }
        let raw_head = c[i] == 'r'
            && (i == 0
                || !is_ident(c[i - 1])
                || (c[i - 1] == 'b' && (i < 2 || !is_ident(c[i - 2]))));
        if raw_head {
            let mut j = i + 1;
            let mut hashes = 0;
            while at(j) == Some('#') {
                hashes += 1;
                j += 1;
            }
            if at(j) == Some('"') {
                j += 1;
                while j < c.len() {
                    if c[j] == '"' && (1..=hashes).all(|h| at(j + h) == Some('#')) {
                        j += 1 + hashes;
                        break;
                    }
                    j += 1;
                }
                i = j;
                out.push(' ');
                continue;
            }
        }
        if c[i] == '"' {
            i += 1;
            while i < c.len() && c[i] != '"' {
                if c[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            out.push(' ');
            continue;
        }
        if c[i] == '\'' {
            if at(i + 1) == Some('\\') {
                let mut j = i + 3;
                while j < c.len() && c[j] != '\'' {
                    j += 1;
                }
                i = j + 1;
                out.push(' ');
                continue;
            }
            if at(i + 2) == Some('\'') {
                i += 3;
                out.push(' ');
                continue;
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// 剥がした本文から名指す区切りの名を取る。`crate::<名>` と `crate::{a, b::{…}}` の括り書きの頭の名。
/// `bare` のとき（`lib.rs`）は接頭辞の無い `<名>::` も数える。`modules` に無い名は数えない。
fn named(code: &str, bare: bool, modules: &BTreeSet<&str>) -> BTreeSet<String> {
    let c: Vec<char> = code.chars().collect();
    let mut names = BTreeSet::new();
    let mut i = 0;
    while i < c.len() {
        if !is_ident(c[i]) || (i > 0 && is_ident(c[i - 1])) {
            i += 1;
            continue;
        }
        let start = i;
        while i < c.len() && is_ident(c[i]) {
            i += 1;
        }
        let word: String = c[start..i].iter().collect();
        if c.get(i) != Some(&':') || c.get(i + 1) != Some(&':') {
            continue;
        }
        if word == "crate" {
            let mut j = i + 2;
            while j < c.len() && c[j].is_whitespace() {
                j += 1;
            }
            if c.get(j) == Some(&'{') {
                let mut depth = 0;
                let mut expect = true;
                while j < c.len() {
                    match c[j] {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        ',' if depth == 1 => expect = true,
                        ch if is_ident(ch) && expect && depth == 1 => {
                            let s = j;
                            while j < c.len() && is_ident(c[j]) {
                                j += 1;
                            }
                            names.insert(c[s..j].iter().collect::<String>());
                            expect = false;
                            continue;
                        }
                        _ => {}
                    }
                    j += 1;
                }
            } else {
                let s = j;
                while j < c.len() && is_ident(c[j]) {
                    j += 1;
                }
                names.insert(c[s..j].iter().collect::<String>());
            }
        } else if bare && (start == 0 || c[start - 1] != ':') {
            names.insert(word);
        }
    }
    names.retain(|n| modules.contains(n.as_str()));
    names
}

/// `lib.rs` の区切りの宣言（`mod <名>;`と`pub mod <名>;`）の集合。
fn declared() -> BTreeSet<String> {
    strip(&read("lib"))
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            let l = l.strip_prefix("pub ").unwrap_or(l);
            l.strip_prefix("mod ")?.strip_suffix(';').map(str::to_string)
        })
        .collect()
}

/// `src/` の `*.rs` の file 名（拡張子を除く）の集合。
fn files() -> BTreeSet<String> {
    fs::read_dir(src_dir())
        .expect("src/ を読めない")
        .map(|e| e.expect("src/ の項を読めない").path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect()
}

fn layer_of(name: &str) -> u8 {
    LAYERS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, l)| *l)
        .unwrap_or_else(|| panic!("層が無い: {name}"))
}

/// 2 つの集合の差を「<左の名>: …」「<右の名>: …」の行に。一致なら空。
fn gaps(left: &str, a: &BTreeSet<String>, right: &str, b: &BTreeSet<String>) -> Vec<String> {
    let mut out: Vec<String> = a.difference(b).map(|x| format!("{left}: {x}")).collect();
    out.extend(b.difference(a).map(|x| format!("{right}: {x}")));
    out
}

#[test]
fn p106_layers_cover_every_module() {
    let table: BTreeSet<String> = LAYERS.iter().map(|(n, _)| n.to_string()).collect();
    assert_eq!(table.len(), LAYERS.len(), "層の割り当ての表に同じ名の行が 2 つある");

    let mut modules = declared();
    modules.insert("lib".to_string());
    modules.insert("main".to_string());
    let miss = gaps("層が無い", &modules, "宣言が無い", &table);
    assert!(miss.is_empty(), "入口の区切りの宣言と表が食い違う: {miss:?}");

    let miss = gaps("層が無い", &files(), "file が無い", &table);
    assert!(miss.is_empty(), "src/ の file と表が食い違う: {miss:?}");
}

#[test]
fn p106_edges_point_down() {
    let modules: BTreeSet<&str> = LAYERS.iter().map(|(n, _)| *n).collect();
    let mut upward = BTreeSet::new();
    for from in &modules {
        let code = strip(&read(from));
        for to in named(&code, *from == "lib", &modules) {
            if to != *from && layer_of(from) < layer_of(&to) {
                upward.insert(format!("{from} と {to} の対"));
            }
        }
    }
    let frozen: BTreeSet<String> = REMAINING_UPWARD
        .iter()
        .map(|(a, b)| format!("{a} と {b} の対"))
        .collect();
    assert_eq!(frozen.len(), REMAINING_UPWARD.len(), "層が上がる辺の一覧に同じ対が 2 つある");
    let miss = gaps("一覧に無い", &upward, "src に無い", &frozen);
    assert!(miss.is_empty(), "層が上がる辺が凍結した一覧と食い違う: {miss:?}");
}

// ── 便 195（行 gp・P-5.6・P-6.3・行 D-11）: 参照 id の空間の閉じた一覧は refs.rs の 1 枚 ──

/// 参照 id の空間の閉じた一覧 4 本の定数名（base 7528256 では RULE_SECTIONS が 6 枚・SRS_ID_SECTIONS が 3 枚）。
const F195_ONE_SHEET: [&str; 4] = [
    "RULE_SECTIONS",
    "SRS_ID_SECTIONS",
    "SRS_ID_PREFIXES",
    "RELATION_NAMESPACES",
];

/// file の `#[cfg(test)]` より前の行のうち、定数 `name` を宣言する行（可視性の印は問わない）の数。
fn f195_declares(src: &str, name: &str) -> usize {
    let head = src.split("#[cfg(test)]").next().unwrap_or_default();
    let want = format!("const {name}:");
    head.lines()
        .map(str::trim_start)
        .map(|l| l.strip_prefix("pub(crate) ").or_else(|| l.strip_prefix("pub ")).unwrap_or(l))
        .filter(|l| l.starts_with(&want))
        .count()
}

/// 歯 2: 4 本を宣言するのは refs.rs だけで各 1 回。散文の門（prose.rs）は要件 id の頭を自分で持たず refs.rs の 1 枚を引く。
#[test]
fn f195_the_id_space_lists_are_declared_only_in_refs() {
    for name in F195_ONE_SHEET {
        let at: Vec<String> = files()
            .into_iter()
            .flat_map(|f| {
                let n = f195_declares(&read(&f), name);
                std::iter::repeat_n(f, n)
            })
            .collect();
        assert_eq!(at, ["refs"], "{name} を宣言する file");
    }
    let prose = read("prose");
    let head = prose.split("#[cfg(test)]").next().unwrap_or_default();
    assert!(!head.contains("\"GOAL\""), "prose.rs が要件 id の頭を自分で持つ");
    assert!(head.contains("refs::SRS_ID_PREFIXES"), "prose.rs が refs.rs の要件 id の頭を引かない");
}

/// 正本の file と、参照 id の空間の一覧の字（検証役の非 blocking N1）。
const F195_SPELLED: [(&str, &str, &[&str]); 6] = [
    ("refs", "RULE_SECTIONS", &["thresholds", "discipline"]),
    (
        "refs",
        "SRS_ID_SECTIONS",
        &["goals", "requirements", "nonfunctional", "acceptance", "constraints", "actors", "outputs"],
    ),
    ("refs", "RELATION_NAMESPACES", &["reqs", "rules", "articles", "sections"]),
    ("refs", "SRS_ID_PREFIXES", &["FR", "NFR", "AC", "CON", "GOAL"]),
    ("prose", "ARTICLE", &["P-", "A-", "N-"]),
    ("prose", "RULE", &["R-", "D-"]),
];

/// 歯 2 の 2（検証役の変異 V4）: 1 枚に寄せた 6 file（refs・link・note・prose・mentions・graph）のうち、`#[cfg(test)]` より前で
/// 参照 id の空間の一覧を字で持つ（空白を除いて数える・式の中の 2 枚目も数える）のは正本の file の 1 回だけ。
/// ほかの file に残る 2 枚目（vocab.rs・adr.rs・rules.rs・面の 4 file）は本便の外（台帳 f2-648.76 に残す）。
#[test]
fn f195_the_merged_files_spell_no_id_space_list() {
    for (owner, name, items) in F195_SPELLED {
        let list = items.iter().map(|i| format!("\"{i}\"")).collect::<Vec<_>>().join(",");
        let at: Vec<&str> = ["refs", "link", "note", "prose", "mentions", "graph"]
            .into_iter()
            .flat_map(|f| {
                let src = read(f);
                let head = src.split("#[cfg(test)]").next().unwrap_or_default();
                let flat: String = head.chars().filter(|c| !c.is_whitespace()).collect();
                std::iter::repeat_n(f, flat.matches(&list).count())
            })
            .collect();
        assert_eq!(at, [owner], "{name} の字を持つ file");
    }
}
