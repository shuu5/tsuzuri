//! 口座の信頼の印の字（設計ノート surface-wave29b 行 cs-trust・判断の記録 ADR-55 決定 (3)）。
//! 起こす口座の置き場の設定 file（`TRUST_FILE`）の字に、作業場 1 つだけの信頼の印を置いた字を組む（`trusted`）。
//! 置く印は `projects.<作業場の絶対 path>.hasTrustDialogAccepted = true` の 1 つだけで、作業場の親と起草の置き場には置かない。

use serde_json::{Map, Value};

/// 口座の置き場の設定 file（口座の置き場からの相対・器が席の anchor の信頼の印を置く file と同じ）。
pub const TRUST_FILE: &str = ".claude.json";

/// 作業場の行の信頼の印の鍵。
pub const TRUST_KEY: &str = "hasTrustDialogAccepted";

/// 口座の置き場の設定 file の字（file が無ければ None）に、作業場 `workspace` の信頼の印を置いた字（断りは理由の字）。
/// 既に印が真なら None（書かない）。file が無い時は印だけの字を組む。末の改行は元の字に合わせ、無い file は改行で終える。
/// 根・projects・作業場の行が object でない字と、印が真偽でない字は断る。鍵の並びは字の順になる（JSON の値は同じ）。
pub fn trusted(text: Option<&str>, workspace: &str) -> Result<Option<String>, &'static str> {
    let mut root: Value = match text {
        Some(t) => serde_json::from_str(t).map_err(|_| "JSON の字でない")?,
        None => Value::Object(Map::new()),
    };
    let row = root
        .as_object_mut()
        .and_then(|r| {
            r.entry("projects")
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
        })
        .and_then(|p| {
            p.entry(workspace)
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
        })
        .ok_or("根か projects か作業場の行が object でない")?;
    match row.get(TRUST_KEY) {
        Some(Value::Bool(true)) => return Ok(None),
        Some(Value::Bool(false)) | None => {}
        Some(_) => return Err("作業場の行の信頼の印が真偽でない"),
    }
    row.insert(TRUST_KEY.to_string(), Value::Bool(true));
    let body = serde_json::to_string_pretty(&root).map_err(|_| "JSON の字にできない")?;
    let newline = text.is_none_or(|t| t.ends_with('\n'));
    Ok(Some(if newline { body + "\n" } else { body }))
}
