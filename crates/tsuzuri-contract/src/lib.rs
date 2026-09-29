//! 契約の型: server と面と hook が共有する型（設計ノート surface §20・surface-base 便 b）。
//! 面の側に電文の写しを書かない。電文の形は JSON で、字の形は `wire` の 2 関数だけが決める。
//! 型の外形は群ごとの歯 `tests/cform_*.rs` と snapshot `tests/snapshots/*.json` が pin する（共通の手は `tests/common/mod.rs`）。

pub mod account;
pub mod board;
pub mod graph;
pub mod ledger;
pub mod notice;
pub mod project;
pub mod question;
pub mod runs;
pub mod seat;
pub mod seathb;
pub mod stats;
pub mod surface;

/// 時刻（UTC の epoch 秒）。器の tick-last と heartbeat-off と同じ単位。
pub type EpochSecs = u64;

/// id の字の形の検査（ASCII の英数字と `extra` の記号だけの 1 語・先頭は英数字・64 byte 以下）。
/// 先頭を英数字に限るので、id が argv の旗に化けない。
pub(crate) fn id_shape(s: &str, extra: &[u8]) -> Result<(), IdError> {
    let bytes = s.as_bytes();
    match bytes.first() {
        None => Err(IdError::Empty),
        _ if bytes.len() > ID_MAX => Err(IdError::Long),
        Some(b) if !b.is_ascii_alphanumeric() => Err(IdError::Shape),
        _ if bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || extra.contains(b)) =>
        {
            Ok(())
        }
        _ => Err(IdError::Shape),
    }
}

/// id の長さの上限（byte）。器の seat deliver の id-shape と同じ。
pub const ID_MAX: usize = 64;

/// id を断る理由（器の seat deliver の id-empty・id-shape・id-long と同じ 3 語）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdError {
    Empty,
    Shape,
    Long,
}

impl std::fmt::Display for IdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            IdError::Empty => "id-empty",
            IdError::Shape => "id-shape",
            IdError::Long => "id-long",
        })
    }
}

impl std::error::Error for IdError {}

/// 電文の JSON の読み書き（server と面と hook はこの 2 関数だけで電文を字にする）。
pub mod wire {
    pub use serde_json::Error;

    /// 値を電文の字にする。
    pub fn encode<T: serde::Serialize>(value: &T) -> Result<String, Error> {
        serde_json::to_string(value)
    }

    /// 電文の字を値に読む。
    pub fn decode<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, Error> {
        serde_json::from_str(text)
    }
}

#[cfg(test)]
mod tests {
    use super::{IdError, id_shape};

    #[test]
    fn skeleton_contract_crate_is_in_workspace() {
        assert_eq!(env!("CARGO_PKG_NAME"), "tsuzuri-contract");
    }

    #[test]
    fn contract_form_id_shape_refuses() {
        assert_eq!(id_shape("t3-hub.5", b".-_"), Ok(()));
        assert_eq!(id_shape("", b".-_"), Err(IdError::Empty));
        assert_eq!(id_shape("-x", b".-_"), Err(IdError::Shape));
        assert_eq!(id_shape("a b", b".-_"), Err(IdError::Shape));
        assert_eq!(id_shape("a:b", b".-_"), Err(IdError::Shape));
        assert_eq!(id_shape(&"a".repeat(64), b""), Ok(()));
        assert_eq!(id_shape(&"a".repeat(65), b""), Err(IdError::Long));
    }
}
