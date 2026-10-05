//! 係の札の結びの道（判断の記録 ADR-59 決定 (4) と帰結の (1)・要件 FR21）。
//! 名を付けた起こしが team の形になる周の結果（status teammate_spawned）は、係の呼びの入力の係の id を持たない。
//! 係の門と測りと終える前の門は、係の呼びの入力の係の id と親の記録の path を持つので、係の記録の隣の meta.json
//! （`subagents/agent-<係の id>.meta.json`・Claude Code が係ごとに書く）の欄 name の名の札に、その係の id を結ぶ。
//! 係の id の字の形から名を切り出さない。結べない訳は記帳と標準エラーの 1 行に書く（門は通す）。

use serde_json::Value;

use super::{Spec, safe};

/// 係の記録の隣の meta.json の path（親の記録 `transcript_path` の .jsonl を除いた dir の下の subagents/agent-<係の id>.meta.json）。
pub fn meta_path(parent: &str, agent_id: &str) -> Option<String> {
    let stem = parent.strip_suffix(".jsonl")?;
    Some(format!("{stem}/subagents/agent-{agent_id}.meta.json"))
}

/// meta.json の字の名（欄 name・英数で始まり英数と `-` と `_` だけの 64 字以内でなければ None）。
pub fn meta_name(text: &str) -> Option<String> {
    let meta: Value = serde_json::from_str(text).ok()?;
    meta.get("name")?
        .as_str()
        .filter(|n| safe(n, 64))
        .map(str::to_string)
}

/// 係の id を札に結べない訳。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Miss {
    /// meta.json を読めない（親の記録の path が .jsonl で終わらないか、file が無い）。
    Meta,
    /// meta.json に名が無い（名を付けない起こし・Claude Code の内の係）。
    Name,
    /// その名の札が無いか読めない。
    Spec(String),
    /// 札が別の係の id に結ばれている。
    Taken { name: String, id: String },
    /// 札に終えの印が在る。
    Ended(String),
    /// 結びを書けない。
    Write(String),
}

impl Miss {
    /// 記帳の欄 cause と標準エラーの 1 行の字。
    pub fn text(&self) -> String {
        match self {
            Self::Meta => "係の記録の隣の meta.json が読めない".to_string(),
            Self::Name => "meta.json に名が無い".to_string(),
            Self::Spec(name) => format!("名 {name} の札が無い"),
            Self::Taken { name, id } => format!("名 {name} の札は別の係の id {id} に結ばれている"),
            Self::Ended(name) => format!("名 {name} の札に終えの印が在る"),
            Self::Write(e) => format!("結びを書けない: {e}"),
        }
    }
}

/// 札にこの係の id を結べるか（札の係の id が無いか同じで、終えの印が無い）。
pub fn claim(spec: &Spec, agent_id: &str) -> Result<(), Miss> {
    if let Some(id) = spec.agent_id.as_deref().filter(|id| *id != agent_id) {
        return Err(Miss::Taken {
            name: spec.name.clone(),
            id: id.to_string(),
        });
    }
    if spec.ended.is_some() {
        return Err(Miss::Ended(spec.name.clone()));
    }
    Ok(())
}
