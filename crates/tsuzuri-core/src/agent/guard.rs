//! 係の門の判じ（判断の記録 ADR-59 決定 (3)(4)・要件 FR21）。
//! 結んだ係の使った量（測りの札の `used`）が札の予算以上の時は書き終えの段で、係の呼びのうち席への知らせ（SendMessage）と、
//! 書きの道具（Write・Edit・NotebookEdit）で係の出力の dir `<起草の置き場>/<名>/w` の下を書く呼びだけを通す。
//! path の下の判じ（`under`）は、`..` の成分を持たず dir の成分で始まる時だけ下と読む（ほかの係の dir と `..` を持つ path は断る）。
//! 群の係の割りの読む path の判じ（`meter::group::inside`）も同じ `under` で比べる。

use std::path::{Component, Path};

use serde_json::Value;

/// 書き終えの段でも通す席への知らせの道具。
pub const NOTIFY: &str = "SendMessage";

/// 書き終えの段で出力の dir の下への書きだけを通す道具と、その tool_input の path の鍵。
pub const WRITES: [(&str, &str); 3] = [
    ("Write", "file_path"),
    ("Edit", "file_path"),
    ("NotebookEdit", "notebook_path"),
];

/// 使った量 `used` が予算 `budget` 以上か（書き終えの段）。
pub fn spent(used: u64, budget: u64) -> bool {
    used >= budget
}

/// 係の呼びの入力の書きの道具の path（書きの道具でないか、path の鍵が字でなければ None）。
pub fn write_path(payload: &str) -> Option<String> {
    let input: Value = serde_json::from_str(payload).ok()?;
    let tool = input.get("tool_name")?.as_str()?;
    let (_, key) = WRITES.iter().find(|(t, _)| *t == tool)?;
    input
        .get("tool_input")?
        .get(key)?
        .as_str()
        .map(str::to_string)
}

/// path `path` が dir `dir` と同じか、その下か（成分で比べ、`..` の成分を持つ path は外と読む）。
pub fn under(path: &Path, dir: &Path) -> bool {
    !path.components().any(|c| c == Component::ParentDir) && path.starts_with(dir)
}

/// 書き終えの段でも通す呼びか（席への知らせか、書きの道具の出力の dir `out` の下への書き）。
pub fn open(tool: &str, path: Option<&str>, out: &Path) -> bool {
    tool == NOTIFY
        || (WRITES.iter().any(|(t, _)| *t == tool)
            && path.is_some_and(|p| under(Path::new(p), out)))
}

/// 断りの理由の字（使った量・予算・通さない道具と、通す 2 つの次の一手）。
pub fn refusal(tool: &str, used: u64, budget: u64, out: &Path) -> String {
    format!(
        "係の門は止める（使った量 {used} が予算 {budget} 以上の書き終えの段で、道具 {tool} は通さない） \
         次の一手 = 分かった所を {}/ の下に Write か Edit で書き、最後の答えを 5 行の形にして終える",
        out.display()
    )
}
