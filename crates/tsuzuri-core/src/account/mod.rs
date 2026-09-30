//! account board の電文を組む純粋な関数（設計ノート surface-base 便 e-acct-host・e-acct-proj・規則の行 R-22）。
//! host の下に口座の列と群の枠と移動の列（`host`）を置く。project の下に project の行と session の行と
//! 電文の組み立て（`project`）を置く（便 e-acct-proj）。
//! 入力は字と今の時刻だけで、どの関数も file も子 process も時計も触らない。状態の判定は器の値を写すだけで、
//! 閾値を持たない。読めない字はその字から組む部分だけを「まだ分からない」にする。

pub mod host;
pub mod project;

use std::sync::Arc;

use serde::{Deserialize, Deserializer};

/// 字か null の JSON の値を、共有の字の `Option<Arc<str>>` に読む（`serde` の rc の機能は使わない）。
pub(crate) fn shared_text<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Arc<str>>, D::Error> {
    Ok(Option::<String>::deserialize(deserializer)?.map(Arc::from))
}

/// 行の `鍵=値` の欄（空白で区切った字のうち `=` を持つもの）。
pub(crate) fn fields(line: &str) -> impl Iterator<Item = (&str, &str)> {
    line.split_whitespace().filter_map(|t| t.split_once('='))
}

/// 行の最初の `鍵=値` の値。
pub(crate) fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    fields(line).find(|(k, _)| *k == key).map(|(_, v)| v)
}

/// 行の最初の `鍵=値` の値のうち空でないもの。
pub(crate) fn value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    field(line, key).filter(|v| !v.is_empty())
}

/// 末尾の「/」を除いて同じ path か。
pub(crate) fn same_path(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

/// anchor の path の最後の名（project の名）。
pub fn project_name(anchor: &str) -> String {
    anchor
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_string()
}
