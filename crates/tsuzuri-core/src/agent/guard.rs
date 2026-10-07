//! 係の門の判じ（判断の記録 ADR-59 決定 (3)(4)・要件 FR21）。
//! 結んだ係の使った量（測りの札の `used`）が札の予算以上の時は書き終えの段で、係の呼びのうち席への知らせ（SendMessage）と、
//! 書きの道具（Write・Edit・NotebookEdit）で係の出力の dir `<起草の置き場>/<名>/w` の下を書く呼びだけを通す。
//! path の下の判じ（`under`）は、`..` の成分を持たず dir の成分で始まる時だけ下と読む（ほかの係の dir と `..` を持つ path は断る）。
//! 群の係の割りの読む path の判じ（`meter::group::inside`）も同じ `under` で比べる。
//! 全時間の 2 つの検め（条 N-2）: 組みの検め（頼みの頭の組みが なし の係の Bash の command の頭の語が cargo なら断る・`cargo_word`）と、
//! 書きの置き場の検め（書きの道具が出力の dir と写しの dir の外へ書くなら断る・`fenced`）。置換の中の語と Bash の書きは見ない。

use std::path::{Component, Path};

use serde_json::Value;

use crate::gate::{is_assignment, segments};

/// 書き終えの段でも通す席への知らせの道具。
pub const NOTIFY: &str = "SendMessage";

/// 写しの dir の名の頭（写しは起草の置き場の直下の `try-<名>`）。
pub const CLONE: &str = "try-";

/// command の頭の語の前に在って、続く語が command の頭になる包みの語。
pub const WRAPS: [&str; 8] = [
    "env", "time", "nice", "nohup", "timeout", "command", "exec", "xargs",
];

/// `-c` の次の語を command として読む shell の名。
pub const SHELLS: [&str; 4] = ["sh", "bash", "zsh", "dash"];

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

/// 係の呼びの入力が Bash の呼びなら、tool_input の command の字（Bash でないか command が字でなければ None）。
pub fn bash_command(payload: &str) -> Option<String> {
    let input: Value = serde_json::from_str(payload).ok()?;
    if input.get("tool_name")?.as_str()? != "Bash" {
        return None;
    }
    input
        .get("tool_input")?
        .get("command")?
        .as_str()
        .map(str::to_string)
}

/// 包みの語の後ろに続く旗（`-` で始まる語）・代入・数（数字と `.` だけで、末に s・m・h・d の 1 字が在ってもよい）の語か。
fn wrap_arg(word: &str) -> bool {
    let digits = word.strip_suffix(['s', 'm', 'h', 'd']).unwrap_or(word);
    word.starts_with('-')
        || is_assignment(word)
        || (!digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit() || c == '.'))
}

/// command の一続きごとの頭の語が cargo か cargo- で始まる名なら、その名（頭の代入と包みの語と旗と数は除き、頭の字 `({$` と backtick と
/// 斜線の前は除く・shell の `-c` の次の語は command として読み直す・当たらなければ None）。
pub fn cargo_word(command: &str) -> Option<String> {
    segments(command).iter().find_map(|seg| {
        let mut words = seg.iter().map(String::as_str).peekable();
        while words.next_if(|w| is_assignment(w)).is_some() {}
        while words.next_if(|w| WRAPS.contains(w)).is_some() {
            while words.next_if(|w| wrap_arg(w)).is_some() {}
        }
        let head = words.next()?.trim_start_matches(['(', '{', '$', '`']);
        let name = head.rsplit('/').next().unwrap_or(head);
        if name == "cargo" || name.starts_with("cargo-") {
            return Some(name.to_string());
        }
        if !SHELLS.contains(&name) {
            return None;
        }
        let rest: Vec<&str> = words.collect();
        let at = rest
            .iter()
            .position(|w| w.starts_with('-') && !w.starts_with("--") && w.contains('c'))?;
        cargo_word(rest.get(at + 1)?)
    })
}

/// 書きの道具の呼びが出力の dir `out` と写しの dir `clone` のどちらの下でもない所へ書くか（path が無い書きも外と読む）。
pub fn fenced(tool: &str, path: Option<&str>, out: &Path, clone: &Path) -> bool {
    WRITES.iter().any(|(t, _)| *t == tool)
        && path.is_none_or(|p| !under(Path::new(p), out) && !under(Path::new(p), clone))
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

/// 組みの検めの断りの理由の字（撃たない語と、床の 2 つと席に頼む次の一手）。
pub fn build_refusal(word: &str, out: &Path) -> String {
    format!(
        "係の門は止める（頼みの頭の組みが なし の係の Bash で、語 {word} は撃たない） \
         次の一手 = 床は tz check と scribe2 pipe preflight で撃ち、組みの要る照らしは {}/notes.md に書いて席に頼む",
        out.display()
    )
}

/// 書きの置き場の検めの断りの理由の字（道具と path〔無ければ字 -〕と、出す物と写しの次の一手）。
pub fn fence_refusal(tool: &str, path: Option<&str>, out: &Path, clone: &Path) -> String {
    format!(
        "係の門は止める（書きの道具 {tool} の path {} は出力の dir {}/ と写し {}/ の下でない） \
         次の一手 = 出す物は {}/ の下に、写しの中の書きは {}/ の下に書く",
        path.unwrap_or("-"),
        out.display(),
        clone.display(),
        out.display(),
        clone.display()
    )
}

/// 断りの理由の字（使った量・予算・通さない道具と、通す 2 つの次の一手）。
pub fn refusal(tool: &str, used: u64, budget: u64, out: &Path) -> String {
    format!(
        "係の門は止める（使った量 {used} が予算 {budget} 以上の書き終えの段で、道具 {tool} は通さない） \
         次の一手 = 分かった所を {}/ の下に Write か Edit で書き、最後の答えを 5 行の形にして終える",
        out.display()
    )
}
