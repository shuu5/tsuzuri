//! 所見の欄の検め（設計ノート surface-wave27a 行 cs-finding・判断の記録 ADR-10 決定 (3) と ADR-29 決定 (1)(ウ)・(8)・受入 AC16）。
//! 入力は窓が書いた草稿の字（JSON）。欄の欠け（鍵が無い・null・空白だけの字）は `DRAFT_FIELDS` の順の名の列で断り、
//! 知らない鍵を断り、候補は 2 つ以上で採るのは 1 つ、添え物の path は作業場からの相対（空・絶対・`~`・`..` の段・
//! 逆斜線と制御の字と行の区切りの字「・」「、」を断る）に限る。path が作業場の下に実在するかは所見の口が確かめる。

use serde_json::Value;
use tsuzuri_contract::consult::{Adoption, DRAFT_FIELDS, FindingDraft};

/// 草稿を受けない理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// JSON の object でない。
    NotObject,
    /// 欠けた欄の名（`DRAFT_FIELDS` の順）。
    Missing(Vec<&'static str>),
    /// 知らない鍵（字の順）。
    Unknown(Vec<String>),
    /// 欄は揃うが値の形が違う（serde の理由の字）。
    Shape(String),
    /// 候補が 2 つ未満か、採る候補が 1 つでない。
    Options,
    /// 作業場からの相対の形でない添え物の path。
    Path(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::NotObject => f.write_str("所見の草稿が JSON の object でない"),
            Refusal::Missing(names) => write!(f, "欠けた欄: {}", names.join(", ")),
            Refusal::Unknown(keys) => write!(f, "知らない欄: {}", keys.join(", ")),
            Refusal::Shape(why) => write!(f, "欄の形が違う: {why}"),
            Refusal::Options => f.write_str("候補は 2 つ以上で、採る（adopted）のは 1 つ"),
            Refusal::Path(path) => write!(f, "添え物の path が作業場からの相対でない: {path}"),
        }
    }
}

/// 欄の値が欠けか（null か空白だけの字）。
fn blank(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.trim().is_empty(),
        _ => false,
    }
}

/// 草稿の字を検めて読む。
pub fn check(text: &str) -> Result<FindingDraft, Refusal> {
    let value: Value = serde_json::from_str(text).map_err(|_| Refusal::NotObject)?;
    let object = value.as_object().ok_or(Refusal::NotObject)?;
    let missing: Vec<&'static str> = DRAFT_FIELDS
        .into_iter()
        .filter(|k| object.get(*k).is_none_or(blank))
        .collect();
    if !missing.is_empty() {
        return Err(Refusal::Missing(missing));
    }
    let unknown: Vec<String> = object
        .keys()
        .filter(|k| !DRAFT_FIELDS.contains(&k.as_str()))
        .cloned()
        .collect();
    if !unknown.is_empty() {
        return Err(Refusal::Unknown(unknown));
    }
    let draft: FindingDraft =
        serde_json::from_value(value).map_err(|e| Refusal::Shape(e.to_string()))?;
    let adopted = draft
        .options
        .iter()
        .filter(|o| o.verdict == Adoption::Adopted)
        .count();
    if draft.options.len() < 2 || adopted != 1 {
        return Err(Refusal::Options);
    }
    if let Some(bad) = draft.attachments.iter().find(|a| !path_ok(&a.path)) {
        return Err(Refusal::Path(bad.path.clone()));
    }
    Ok(draft)
}

/// 添え物の path が作業場からの相対の形か。
pub fn path_ok(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('~')
        && path.split('/').all(|seg| seg != "..")
        && !path
            .chars()
            .any(|c| c.is_control() || matches!(c, '\\' | '・' | '、'))
}
