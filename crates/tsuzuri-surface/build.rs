//! project board の block の列を src/project の dir から生成する（行 hs-blocks・判断の記録 ADR-13）。
//! mod.rs でない .rs の file ごとに path の属性と pub mod の宣言を置き、列挙 Module と ALL・name・block・view を足す。
//! 字は組み立ての出力の dir の project_blocks.rs に書く（今の字と同じなら書き直さない）。trunk の wasm の組み立ても同じ字を得る。
//! project board の頁も src/pages の dir から同じ書き方で pages.rs に生成する（列挙 PageId と ALL・id・def・行 hs-pages）。

use std::fmt::Write as _;
use std::path::Path;

fn main() -> Result<(), String> {
    println!("cargo:rerun-if-changed=src/project");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/pages");
    let manifest = std::env::var("CARGO_MANIFEST_DIR").map_err(|e| format!("CARGO_MANIFEST_DIR: {e}"))?;
    let out_dir = std::env::var("OUT_DIR").map_err(|e| format!("OUT_DIR: {e}"))?;
    let dir = Path::new(&manifest).join("src").join("project");
    let names = block_names(&dir)?;
    let text = generate(&dir, &names)?;
    write_if_changed(&Path::new(&out_dir).join("project_blocks.rs"), &text)?;
    let dir = Path::new(&manifest).join("src").join("pages");
    let names = block_names(&dir)?;
    let text = generate_pages(&dir, &names)?;
    write_if_changed(&Path::new(&out_dir).join("pages.rs"), &text)?;
    Ok(())
}

/// 今の字と違うときだけ書く。
fn write_if_changed(out: &Path, text: &str) -> Result<(), String> {
    if std::fs::read_to_string(out).ok().as_deref() != Some(text) {
        std::fs::write(out, text).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    Ok(())
}

/// dir の下の mod.rs でない .rs の file の名（拡張子を除く・byte の順）。名の字が外れれば file の名を書いた Err。
fn block_names(dir: &Path) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
        let file = entry.file_name();
        let Some(file) = file.to_str() else {
            return Err(format!("file の名が UTF-8 でない: {file:?}"));
        };
        let Some(name) = file.strip_suffix(".rs") else {
            continue;
        };
        if file == "mod.rs" {
            continue;
        }
        if !name_ok(name) {
            return Err(format!(
                "{} の file の名は英小文字で始まり英小文字と数字と下線だけ: {file}",
                dir.display()
            ));
        }
        names.push(name.to_string());
    }
    names.sort();
    Ok(names)
}

/// 名が英小文字で始まり、英小文字と数字と下線だけか。
fn name_ok(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// 変種の名（先頭と下線の後の字を大文字にして下線を外す・askpage は Askpage）。
fn variant(name: &str) -> String {
    let mut out = String::new();
    let mut upper = true;
    for c in name.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// 名ごとの path の属性と pub mod の宣言。
fn modules(s: &mut String, dir: &Path, names: &[String]) -> Result<(), String> {
    let w = |e: std::fmt::Error| e.to_string();
    for name in names {
        let path = dir.join(format!("{name}.rs"));
        let Some(path) = path.to_str() else {
            return Err(format!("path が UTF-8 でない: {}", path.display()));
        };
        writeln!(s, "#[path = {path:?}]").map_err(w)?;
        writeln!(s, "pub mod {name};").map_err(w)?;
    }
    Ok(())
}

/// 頁の生成する字（内側の doc と内側の属性を置かない）。
fn generate_pages(dir: &Path, names: &[String]) -> Result<String, String> {
    let mut s = String::new();
    let w = |e: std::fmt::Error| e.to_string();
    modules(&mut s, dir, names)?;
    writeln!(s).map_err(w)?;
    writeln!(s, "/// project board の頁（src/pages の file ごとに 1 つ・名の順）。").map_err(w)?;
    writeln!(s, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]").map_err(w)?;
    writeln!(s, "pub enum PageId {{").map_err(w)?;
    for name in names {
        writeln!(s, "    {},", variant(name)).map_err(w)?;
    }
    writeln!(s, "}}").map_err(w)?;
    writeln!(s).map_err(w)?;
    writeln!(s, "impl PageId {{").map_err(w)?;
    writeln!(s, "    /// 全部の頁（名の順・nav の順は frame の nav）。").map_err(w)?;
    writeln!(s, "    pub const ALL: [PageId; {}] = [", names.len()).map_err(w)?;
    for name in names {
        writeln!(s, "        PageId::{},", variant(name)).map_err(w)?;
    }
    writeln!(s, "    ];").map_err(w)?;
    writeln!(s).map_err(w)?;
    writeln!(s, "    /// 頁の id（file の名から拡張子を除いた字・URL の query の page の値）。").map_err(w)?;
    writeln!(s, "    pub fn id(self) -> &'static str {{").map_err(w)?;
    writeln!(s, "        match self {{").map_err(w)?;
    for name in names {
        writeln!(s, "            PageId::{} => {name:?},", variant(name)).map_err(w)?;
    }
    writeln!(s, "        }}").map_err(w)?;
    writeln!(s, "    }}").map_err(w)?;
    writeln!(s).map_err(w)?;
    writeln!(s, "    /// 頁の定義（その頁の PAGE）。").map_err(w)?;
    writeln!(s, "    pub fn def(self) -> crate::frame::PageDef {{").map_err(w)?;
    writeln!(s, "        match self {{").map_err(w)?;
    for name in names {
        writeln!(s, "            PageId::{} => {name}::PAGE,", variant(name)).map_err(w)?;
    }
    writeln!(s, "        }}").map_err(w)?;
    writeln!(s, "    }}").map_err(w)?;
    writeln!(s, "}}").map_err(w)?;
    Ok(s)
}

/// 生成する字（内側の doc と内側の属性を置かない）。
fn generate(dir: &Path, names: &[String]) -> Result<String, String> {
    let mut s = String::new();
    let w = |e: std::fmt::Error| e.to_string();
    modules(&mut s, dir, names)?;
    writeln!(s).map_err(w)?;
    writeln!(s, "/// project board の block の module（src/project の file ごとに 1 つ・名の順）。").map_err(w)?;
    writeln!(s, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]").map_err(w)?;
    writeln!(s, "pub enum Module {{").map_err(w)?;
    for name in names {
        writeln!(s, "    {},", variant(name)).map_err(w)?;
    }
    writeln!(s, "}}").map_err(w)?;
    writeln!(s).map_err(w)?;
    writeln!(s, "impl Module {{").map_err(w)?;
    writeln!(s, "    /// 全部の module（名の順）。").map_err(w)?;
    writeln!(s, "    pub const ALL: [Module; {}] = [", names.len()).map_err(w)?;
    for name in names {
        writeln!(s, "        Module::{},", variant(name)).map_err(w)?;
    }
    writeln!(s, "    ];").map_err(w)?;
    writeln!(s).map_err(w)?;
    writeln!(s, "    /// module の名（file の名から拡張子を除いた字）。").map_err(w)?;
    writeln!(s, "    pub fn name(self) -> &'static str {{").map_err(w)?;
    writeln!(s, "        match self {{").map_err(w)?;
    for name in names {
        writeln!(s, "            Module::{} => {name:?},", variant(name)).map_err(w)?;
    }
    writeln!(s, "        }}").map_err(w)?;
    writeln!(s, "    }}").map_err(w)?;
    writeln!(s).map_err(w)?;
    writeln!(s, "    /// module の枠の値（その module の BLOCK）。").map_err(w)?;
    writeln!(s, "    pub fn block(self) -> crate::frame::Block {{").map_err(w)?;
    writeln!(s, "        match self {{").map_err(w)?;
    for name in names {
        writeln!(s, "            Module::{} => {name}::BLOCK,", variant(name)).map_err(w)?;
    }
    writeln!(s, "        }}").map_err(w)?;
    writeln!(s, "    }}").map_err(w)?;
    writeln!(s).map_err(w)?;
    writeln!(s, "    /// module の中身を描く（wasm の target のときだけ）。").map_err(w)?;
    writeln!(s, "    #[cfg(target_arch = \"wasm32\")]").map_err(w)?;
    writeln!(s, "    pub fn view(self) -> leptos::prelude::AnyView {{").map_err(w)?;
    writeln!(s, "        match self {{").map_err(w)?;
    for name in names {
        writeln!(s, "            Module::{} => {name}::view(),", variant(name)).map_err(w)?;
    }
    writeln!(s, "        }}").map_err(w)?;
    writeln!(s, "    }}").map_err(w)?;
    writeln!(s, "}}").map_err(w)?;
    Ok(s)
}
