//! 組み立ての script（便 13 (a)・P-6.4）。部品目録 `design-intent/preview/parts.json` を読み、
//! 部品・図の型・棚の型・行内の様式に許す性質 の閉じた一覧（enum）を Rust の source 1 本として `OUT_DIR` に書く。
//! 人は型の一覧を書かない。導出できない部品目録は組み立てを失敗させる（黙って空の一覧にしない）。

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

use yaml_rust2::{Yaml, YamlLoader};

/// 部品目録の path（crate から見た相対）。
const CATALOG: &str = "../../design-intent/preview/parts.json";

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let path = Path::new(&manifest).join(CATALOG);
    println!("cargo:rerun-if-changed={}", path.display());
    let source = match derive(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("部品目録 {} から導出できない: {e}", path.display());
            std::process::exit(1);
        }
    };
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    fs::write(Path::new(&out_dir).join("parts_catalog.rs"), source).expect("OUT_DIR へ書けない");
}

/// 部品目録を読んで Rust の source を組む。
fn derive(path: &Path) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("読めない（{e}）"))?;
    let docs = YamlLoader::load_from_str(&text).map_err(|e| format!("読めない（{e}）"))?;
    let root = match docs.as_slice() {
        [doc] => doc,
        _ => return Err("文書が 1 つでない".to_string()),
    };

    let components = root
        .as_hash()
        .and_then(|m| m.get(&Yaml::String("components".to_string())))
        .and_then(Yaml::as_hash)
        .ok_or_else(|| "components が表でない".to_string())?;
    let mut parts: Vec<(String, Vec<String>)> = Vec::with_capacity(components.len());
    for (name, body) in components.iter() {
        let name = name
            .as_str()
            .ok_or_else(|| "components のキーが文字列でない".to_string())?;
        let faces = body
            .as_hash()
            .and_then(|m| m.get(&Yaml::String("faces".to_string())))
            .and_then(Yaml::as_vec)
            .ok_or_else(|| format!("部品「{name}」が faces の一覧を持たない"))?;
        let faces = faces
            .iter()
            .map(|f| {
                f.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| format!("部品「{name}」の faces に文字列でない値"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        parts.push((name.to_string(), faces));
    }

    let figure_types = string_list(root, "figure_type_enum")?;
    let shelf_types = string_list(root, "shelf_type_enum")?;
    let style_props = string_list(root, "style_props_allowed")?;

    let mut out = String::new();
    out.push_str("// 組み立て時に build.rs が部品目録（design-intent/preview/parts.json）から導出した。人は書かない。\n");
    let names: Vec<&str> = parts.iter().map(|(n, _)| n.as_str()).collect();
    let variants = write_enum(
        &mut out,
        "部品（部品目録の components・目録の順）",
        "Component",
        &names,
    )?;
    // 部品だけが持つ faces
    out.push_str("impl Component {\n    /// 置ける面（部品目録の faces）\n    pub fn faces(self) -> &'static [&'static str] {\n        match self {\n");
    for ((_, faces), v) in parts.iter().zip(&variants) {
        let lits: Vec<String> = faces.iter().map(|f| format!("{f:?}")).collect();
        out.push_str(&format!(
            "            Component::{v} => &[{}],\n",
            lits.join(", ")
        ));
    }
    out.push_str("        }\n    }\n}\n");
    write_enum(
        &mut out,
        "図の型（部品目録の figure_type_enum・目録の順）",
        "FigureType",
        &refs(&figure_types),
    )?;
    write_enum(
        &mut out,
        "棚の型（部品目録の shelf_type_enum・目録の順）",
        "ShelfType",
        &refs(&shelf_types),
    )?;
    write_enum(
        &mut out,
        "行内の様式（属性 style）に許す性質の名（部品目録の style_props_allowed・目録の順）",
        "StyleProp",
        &refs(&style_props),
    )?;
    Ok(out)
}

fn refs(v: &[String]) -> Vec<&str> {
    v.iter().map(String::as_str).collect()
}

/// 根の表の `key` を文字列の一覧として読む。
fn string_list(root: &Yaml, key: &str) -> Result<Vec<String>, String> {
    let seq = root
        .as_hash()
        .and_then(|m| m.get(&Yaml::String(key.to_string())))
        .and_then(Yaml::as_vec)
        .ok_or_else(|| format!("{key} が一覧でない"))?;
    seq.iter()
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{key} に文字列でない値"))
        })
        .collect()
}

/// 部品目録の名を型の名にする（「-」で割り、各片の先頭を大文字にして繋ぐ・pipeline-rail は PipelineRail）。
/// ASCII の英小文字と数字と「-」以外を含む・先頭が数字・型の名にならない、は Err。
fn variant(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("空の名".to_string());
    }
    if let Some(c) = name
        .chars()
        .find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-'))
    {
        return Err(format!("名「{name}」に使えない字「{c}」"));
    }
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(format!("名「{name}」の先頭が数字"));
    }
    let mut out = String::with_capacity(name.len());
    for piece in name.split('-') {
        let mut chars = piece.chars();
        if let Some(head) = chars.next() {
            out.push(head.to_ascii_uppercase());
            out.extend(chars);
        }
    }
    if out.is_empty() || out.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(format!("名「{name}」が型の名にならない"));
    }
    Ok(out)
}

/// 閉じた一覧 1 つ: enum・全部を並べた定数 ALL・目録の名を返す name・名から引く from_name。
/// 戻り値は各名の型の名（目録の順・呼び手が同じ名を 2 度計算しないため）。
fn write_enum(
    out: &mut String,
    doc: &str,
    ty: &str,
    names: &[&str],
) -> Result<Vec<String>, String> {
    let mut seen = HashSet::new();
    let mut variants = Vec::with_capacity(names.len());
    for name in names {
        let v = variant(name)?;
        if !seen.insert(v.clone()) {
            return Err(format!("2 つの名が同じ型の名「{v}」に潰れる（{ty}）"));
        }
        variants.push(v);
    }
    out.push_str(&format!(
        "/// {doc}。\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum {ty} {{\n"
    ));
    for (name, v) in names.iter().zip(&variants) {
        out.push_str(&format!("    /// {name}\n    {v},\n"));
    }
    out.push_str("}\n");
    out.push_str(&format!("impl {ty} {{\n"));
    out.push_str(&format!(
        "    /// 全部（部品目録の順）\n    pub const ALL: [{ty}; {}] = [\n",
        variants.len()
    ));
    for v in &variants {
        out.push_str(&format!("        {ty}::{v},\n"));
    }
    out.push_str("    ];\n");
    out.push_str(
        "    /// 部品目録の名\n    pub fn name(self) -> &'static str {\n        match self {\n",
    );
    for (name, v) in names.iter().zip(&variants) {
        out.push_str(&format!("            {ty}::{v} => {name:?},\n"));
    }
    out.push_str("        }\n    }\n");
    out.push_str(&format!(
        "    /// 部品目録の名から引く（無ければ None）\n    pub fn from_name(name: &str) -> Option<{ty}> {{\n        match name {{\n"
    ));
    for (name, v) in names.iter().zip(&variants) {
        out.push_str(&format!("            {name:?} => Some({ty}::{v}),\n"));
    }
    out.push_str("            _ => None,\n        }\n    }\n}\n");
    Ok(variants)
}
