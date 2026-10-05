//! 群の係の測りと係の門の字（行 ag-gwatch・判断の記録 ADR-61 決定 (4)(5)(7)・要件 FR22）。
//! 群の係は、係の dir に群の席の札（`group.json`・行 ag-gspawn が書く）を持つ係である。群の係は文脈の読み直し（cache_read）も
//! 上限 `MEMBER_READ` で数え、50・75・90% の注ぎを両方の欄で出し、どちらかの欄が上限以上なら書き終えの段に入る（決定 (7) の (イ)）。
//! 係の門は、群の係の読みの道具（Read）の割りの読む path の外の読みと、係を起こす道具（Agent）の呼び（入れ子）を断り、
//! 命令（Bash）と検索（Grep・Glob）の道具は見ない（決定 (4)(5)）。測りの口は、群の係の Read の path を群の id の dir の
//! `reads.jsonl` に足し、同じ群のほかの係が先に読んだ path なら `twice.jsonl` にも 1 行足す（記帳だけで断らない・決定 (5)）。

use std::path::{Component, Path};

use serde_json::{Value, json};
use tsuzuri_contract::EpochSecs;

use super::Meter;
use crate::agent::spec::group::{MEMBER_READ, Seat};

/// 読みの道具の名。
pub const READ: &str = "Read";

/// 係を起こす道具の名（群の係の呼びは入れ子）。
pub const NEST: &str = "Agent";

/// 群の id の dir の、群の係の読みの道具の path の記帳。
pub const READS: &str = "reads.jsonl";

/// 群の id の dir の、同じ path の 2 度の読み（同じ群のほかの係が先に読んだ path）の記帳。
pub const TWICE: &str = "twice.jsonl";

/// 呼びの入力の読みの道具の path（Read の呼びで tool_input の file_path が字の時だけ）。
pub fn read_path(payload: &str) -> Option<String> {
    let input: Value = serde_json::from_str(payload).ok()?;
    if input.get("tool_name")?.as_str()? != READ {
        return None;
    }
    input
        .pointer("/tool_input/file_path")?
        .as_str()
        .map(str::to_string)
}

/// path が割りの読む path のどれかと同じか、その dir の下か（成分で比べ、`..` の成分を持つ path は外と読む）。
pub fn inside(path: &str, paths: &[String]) -> bool {
    let p = Path::new(path);
    !p.components().any(|c| c == Component::ParentDir) && paths.iter().any(|a| p.starts_with(a))
}

/// 群の係の呼びの断りの理由（係を起こす道具の呼びと、読みの道具の割りの外の読み・ほかの呼びは None）。
pub fn fence(tool: &str, path: Option<&str>, seat: &Seat) -> Option<String> {
    let why = match tool {
        NEST => format!(
            "群 {} の係は係を起こす道具 {NEST} を呼ばない（入れ子）",
            seat.group
        ),
        READ if !path.is_some_and(|p| inside(p, &seat.paths)) => format!(
            "群 {} の係の読みの道具 {READ} の path {} は割りの読む path の外",
            seat.group,
            path.unwrap_or_default()
        ),
        _ => return None,
    };
    Some(format!(
        "係の門は止める（{why}・判断の記録 ADR-61 の決定 (4)） \
         次の一手 = 割りの外の証拠か調べが要る主張は、分からない（U）のまま要る path を w/ に書いて席に返す"
    ))
}

/// 群の係の読み直しが上限以上の書き終えの段の断りの理由（読み直しと上限・通さない道具と、通す 2 つの次の一手）。
pub fn read_refusal(tool: &str, read: u64, out: &Path) -> String {
    format!(
        "係の門は止める（群の係の読み直し {read} が上限 {MEMBER_READ} 以上の書き終えの段で、道具 {tool} は通さない・判断の記録 ADR-61 の決定 (7)） \
         次の一手 = 分かった所を {}/ の下に Write か Edit で書き、SendMessage で席に知らせて終える",
        out.display()
    )
}

/// 群の係の注ぎの字（新しく越えた欄と印・両方の欄の量と上限の残り・次の一手）。
pub fn member_notice(new: Option<u64>, read: Option<u64>, meter: &Meter, budget: u64) -> String {
    let marks: Vec<String> = [("新しい量", new), ("読み直し", read)]
        .into_iter()
        .filter_map(|(field, mark)| mark.map(|m| format!("{field} {m}%")))
        .collect();
    format!(
        "係の測り（群の係）: {} を越えた（新しい量 {} は予算 {budget} の残り {}・読み直し {} は上限 {MEMBER_READ} の残り {}・判断の記録 ADR-61 の決定 (7)）。\
         次の一手 = 新しい調べを広げず、分かった所を w/ に書く。どちらかの欄が 100% を越えると、係の門は自分の w/ への書きと SendMessage だけを通す",
        marks.join("・"),
        meter.used,
        budget.saturating_sub(meter.used),
        meter.cache_read,
        MEMBER_READ.saturating_sub(meter.cache_read)
    )
}

/// 読みの記帳の 1 行（時刻・係の名・path の JSON・改行なし）。
pub fn read_line(name: &str, path: &str, at: EpochSecs) -> String {
    json!({"at": at, "name": name, "path": path}).to_string()
}

/// 読みの記帳の字 `reads` で、係 `name` がまだ読まず、同じ群のほかの係が先に読んだ path なら、最初に読んだ係の名（2 度の読み）。
pub fn first_reader(reads: &str, name: &str, path: &str) -> Option<String> {
    let readers: Vec<String> = reads
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v.get("path").and_then(Value::as_str) == Some(path))
        .filter_map(|v| v.get("name").and_then(Value::as_str).map(str::to_string))
        .collect();
    if readers.iter().any(|n| n == name) {
        return None;
    }
    readers.into_iter().next()
}

/// 2 度の読みの記帳の 1 行（時刻・係の名・path・先に読んだ係の名の JSON・改行なし）。
pub fn twice_line(name: &str, path: &str, first: &str, at: EpochSecs) -> String {
    json!({"at": at, "name": name, "path": path, "first": first}).to_string()
}
