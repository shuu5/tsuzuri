//! 口の列を src/server/routes の下の file から生成する（判断の記録 ADR-13・案 M2・標準 library だけ）。
//! 名（拡張子 rs を除く字）を byte の順に並べ、名ごとの path の属性と mod の宣言と、列挙 `Route` と
//! その impl を、組み立ての出力の dir の routes.rs に書く（server の mod.rs が include で取り込む）。
//! 生成する字は include の macro の中に置くので、内側の doc と内側の属性を持たない。

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// 口の file の置き場（package の根から）。
const ROUTES: &str = "src/server/routes";

fn main() -> Result<(), String> {
    println!("cargo:rerun-if-changed=src/server/routes");
    println!("cargo:rerun-if-changed=build.rs");
    let manifest = env::var("CARGO_MANIFEST_DIR").map_err(|e| format!("CARGO_MANIFEST_DIR: {e}"))?;
    let out_dir = env::var("OUT_DIR").map_err(|e| format!("OUT_DIR: {e}"))?;
    let names = names(&Path::new(&manifest).join(ROUTES))?;
    let text = generate(&manifest, &names)?;
    let out = Path::new(&out_dir).join("routes.rs");
    if fs::read_to_string(&out).ok().as_deref() != Some(text.as_str()) {
        fs::write(&out, text).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    Ok(())
}

/// dir の下の拡張子 rs の file の名（byte の順）。英小文字で始まり英小文字と数字と下線だけでない名は Err。
fn names(dir: &Path) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut names = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|name| well_formed(name))
            .ok_or_else(|| format!("口の file の名が読めない: {}", path.display()))?;
        names.push(name.to_string());
    }
    names.sort();
    Ok(names)
}

fn well_formed(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// 変種の名（先頭と下線の後の字を大文字にし、下線を外す）。
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

fn generate(manifest: &str, names: &[String]) -> Result<String, String> {
    let mut s = String::new();
    let w = |r: std::fmt::Result| r.map_err(|e| format!("生成: {e}"));
    for name in names {
        let path = format!("{manifest}/{ROUTES}/{name}.rs");
        w(writeln!(s, "#[path = {path:?}]\nmod {name};"))?;
    }
    w(writeln!(
        s,
        "\n/// 口の列（src/server/routes の下の file ごとに 1 つ・名の順）。"
    ))?;
    w(writeln!(s, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]"))?;
    w(writeln!(s, "pub enum Route {{"))?;
    for name in names {
        w(writeln!(s, "    {},", variant(name)))?;
    }
    w(writeln!(s, "}}\n\nimpl Route {{"))?;
    w(writeln!(s, "    /// 全部の口（宣言の順）。"))?;
    w(writeln!(s, "    pub const ALL: [Route; {}] = [", names.len()))?;
    for name in names {
        w(writeln!(s, "        Route::{},", variant(name)))?;
    }
    w(writeln!(s, "    ];\n"))?;
    for (sig, doc, body) in [
        (
            "pub fn name(self) -> &'static str",
            "口の file の名。",
            None,
        ),
        (
            "pub fn key(self) -> crate::server::route::Key",
            "口の鍵。",
            Some("ROUTE.key"),
        ),
        (
            "pub(in crate::server) fn entry(self) -> crate::server::route::Entry",
            "口の本体。",
            Some("ROUTE"),
        ),
    ] {
        w(writeln!(s, "    /// {doc}\n    {sig} {{\n        match self {{"))?;
        for name in names {
            let arm = match body {
                None => format!("{name:?}"),
                Some(item) => format!("{name}::{item}"),
            };
            w(writeln!(s, "            Route::{} => {arm},", variant(name)))?;
        }
        w(writeln!(s, "        }}\n    }}\n"))?;
    }
    w(writeln!(s, "}}"))?;
    Ok(s)
}
