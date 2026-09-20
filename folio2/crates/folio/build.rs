//! 組み立ての script（便 13 (a)・P-6.4）。部品目録 `design-intent/preview/parts.json` を読み、
//! 部品・図の型・棚の型・行内の様式に許す性質 の閉じた一覧（enum）を Rust の source 1 本として `OUT_DIR` に書く。
//! 便 49（ADR-11 決定 (4)②）から憲法の正本 `design-intent/constitution.yaml` の schema.enums も同じ型で導出し、
//! 値域の閉じた一覧をもう 1 本 `OUT_DIR` に書く（鍵の一覧は file から・人は鍵も値も書かない）。
//! 人は型の一覧を書かない。導出できない部品目録・憲法は組み立てを失敗させる（黙って空の一覧にしない）。

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

use yaml_rust2::{Yaml, YamlLoader};

/// 部品目録の path（crate から見た相対）。
const CATALOG: &str = "../../design-intent/preview/parts.json";
/// 憲法の正本の path（crate から見た相対）。
const CONSTITUTION: &str = "../../design-intent/constitution.yaml";

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let out_dir = Path::new(&out_dir);

    let path = Path::new(&manifest).join(CATALOG);
    println!("cargo:rerun-if-changed={}", path.display());
    let source = match derive(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("部品目録 {} から導出できない: {e}", path.display());
            std::process::exit(1);
        }
    };
    fs::write(out_dir.join("parts_catalog.rs"), source).expect("OUT_DIR へ書けない");

    let path = Path::new(&manifest).join(CONSTITUTION);
    println!("cargo:rerun-if-changed={}", path.display());
    let source = fs::read_to_string(&path)
        .map_err(|e| format!("読めない（{e}）"))
        .and_then(|text| constitution_enums(&text));
    let source = match source {
        Ok(s) => s,
        Err(e) => {
            eprintln!("憲法の正本 {} から導出できない: {e}", path.display());
            std::process::exit(1);
        }
    };
    fs::write(out_dir.join("constitution_enums.rs"), source).expect("OUT_DIR へ書けない");
}

/// 憲法の正本の文字列を受け、schema.enums の表に在る鍵を file の順に全部、閉じた一覧（enum）の Rust の source に組む
/// （便 49 (a)）。鍵の名を書き並べない。鍵の名は「_」で割って各片の先頭を大文字にして繋いだ型の名になる
/// （retreat_kind は RetreatKind）。文書が 1 つでない・schema.enums が表でない・表が空・ある鍵の値が一覧でない・
/// 一覧が空・値が文字列でない・同じ値が 2 度在る・2 つの値が同じ型の中の名に潰れる・2 つの鍵が同じ型の名に潰れる・
/// 型の名にならない字、は Err（理由の文に鍵の名を入れる）。
pub fn constitution_enums(text: &str) -> Result<String, String> {
    let docs = YamlLoader::load_from_str(text).map_err(|e| format!("読めない（{e}）"))?;
    let root = match docs.as_slice() {
        [doc] => doc,
        _ => return Err("文書が 1 つでない".to_string()),
    };
    let enums = root
        .as_hash()
        .and_then(|m| m.get(&Yaml::String("schema".to_string())))
        .and_then(Yaml::as_hash)
        .and_then(|m| m.get(&Yaml::String("enums".to_string())))
        .and_then(Yaml::as_hash)
        .ok_or_else(|| "schema.enums が表でない".to_string())?;
    if enums.is_empty() {
        return Err("schema.enums が空".to_string());
    }

    let mut out = String::new();
    out.push_str("// 組み立て時に build.rs が憲法の正本（design-intent/constitution.yaml）の schema.enums から導出した。人は書かない。\n");
    let mut types = HashSet::new();
    // 鍵の名と型の名の対（定数 ENUMS の素・file の順）
    let mut keys: Vec<(&str, String)> = Vec::with_capacity(enums.len());
    for (key, body) in enums.iter() {
        let key = key
            .as_str()
            .ok_or_else(|| "schema.enums の鍵が文字列でない".to_string())?;
        let ty = type_name(key).map_err(|e| format!("schema.enums.{key}: {e}"))?;
        if !types.insert(ty.clone()) {
            return Err(format!(
                "schema.enums.{key}: 2 つの鍵が同じ型の名「{ty}」に潰れる"
            ));
        }
        let values = body
            .as_vec()
            .ok_or_else(|| format!("schema.enums.{key} が一覧でない"))?;
        if values.is_empty() {
            return Err(format!("schema.enums.{key} が空"));
        }
        let mut names: Vec<&str> = Vec::with_capacity(values.len());
        for v in values {
            let s = v
                .as_str()
                .ok_or_else(|| format!("schema.enums.{key} に文字列でない値"))?;
            if names.contains(&s) {
                return Err(format!("schema.enums.{key} に同じ値「{s}」が 2 度在る"));
            }
            names.push(s);
        }
        write_enum(
            &mut out,
            "憲法の正本",
            &format!("憲法の値域 {key}（憲法の正本の schema.enums.{key}・file の順）"),
            &ty,
            &names,
        )
        .map_err(|e| format!("schema.enums.{key}: {e}"))?;
        keys.push((key, ty));
    }
    // 鍵の名と NAMES の対の列（便 50 (d)・面の生成器が読んでいる置き場の値域と組み立てた版のずれを鍵ごとに見る）
    out.push_str(&format!(
        "/// 憲法の値域の鍵の名と NAMES の対（憲法の正本の schema.enums の鍵・file の順）。\npub const ENUMS: [(&str, &[&str]); {}] = [\n",
        keys.len()
    ));
    for (key, ty) in &keys {
        out.push_str(&format!("    ({key:?}, &{ty}::NAMES),\n"));
    }
    out.push_str("];\n");
    Ok(out)
}

/// 憲法の鍵の名を型の名にする（「_」で割り、各片の先頭を大文字にして繋ぐ・retreat_kind は RetreatKind・tier は Tier）。
/// ASCII の英字と数字と「_」以外を含む・先頭が数字・型の名にならない、は Err。
fn type_name(key: &str) -> Result<String, String> {
    if key.is_empty() {
        return Err("空の鍵".to_string());
    }
    if let Some(c) = key
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || *c == '_'))
    {
        return Err(format!("鍵「{key}」に使えない字「{c}」"));
    }
    let mut out = String::with_capacity(key.len());
    for piece in key.split('_') {
        let mut chars = piece.chars();
        if let Some(head) = chars.next() {
            out.push(head.to_ascii_uppercase());
            out.extend(chars);
        }
    }
    if out.is_empty() || out.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(format!("鍵「{key}」が型の名にならない"));
    }
    Ok(out)
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
        "部品目録",
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
        "部品目録",
        "図の型（部品目録の figure_type_enum・目録の順）",
        "FigureType",
        &refs(&figure_types),
    )?;
    write_enum(
        &mut out,
        "部品目録",
        "棚の型（部品目録の shelf_type_enum・目録の順）",
        "ShelfType",
        &refs(&shelf_types),
    )?;
    write_enum(
        &mut out,
        "部品目録",
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

/// 目録・憲法の値の名を型の中の名にする（「-」で割り、各片の先頭を大文字にして繋ぐ・pipeline-rail は PipelineRail・
/// M0 は M0・delivery-0 は Delivery0・v1-incident は V1Incident・must-not は MustNot）。
/// ASCII の英字と数字と「-」以外を含む・先頭が数字・型の名にならない、は Err。
fn variant(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("空の名".to_string());
    }
    if let Some(c) = name
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || *c == '-'))
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

/// 閉じた一覧 1 つ: enum・全部を並べた定数 ALL・名の字面を並べた定数 NAMES・名を返す name（const fn）・名から引く from_name。
/// `source` は doc に書く出どころ（部品目録 / 憲法の正本）。
/// 戻り値は各名の型の名（出どころの順・呼び手が同じ名を 2 度計算しないため）。
fn write_enum(
    out: &mut String,
    source: &str,
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
        "    /// 全部（{source}の順）\n    pub const ALL: [{ty}; {}] = [\n",
        variants.len()
    ));
    for v in &variants {
        out.push_str(&format!("        {ty}::{v},\n"));
    }
    out.push_str("    ];\n");
    out.push_str(&format!(
        "    /// 名の字面（{source}の順・長さは値の数）\n    pub const NAMES: [&str; {}] = [\n",
        names.len()
    ));
    for name in names {
        out.push_str(&format!("        {name:?},\n"));
    }
    out.push_str("    ];\n");
    out.push_str(&format!(
        "    /// {source}の名\n    pub const fn name(self) -> &'static str {{\n        match self {{\n"
    ));
    for (name, v) in names.iter().zip(&variants) {
        out.push_str(&format!("            {ty}::{v} => {name:?},\n"));
    }
    out.push_str("        }\n    }\n");
    out.push_str(&format!(
        "    /// {source}の名から引く（無ければ None）\n    pub fn from_name(name: &str) -> Option<{ty}> {{\n        match name {{\n"
    ));
    for (name, v) in names.iter().zip(&variants) {
        out.push_str(&format!("            {name:?} => Some({ty}::{v}),\n"));
    }
    out.push_str("            _ => None,\n        }\n    }\n}\n");
    Ok(variants)
}
