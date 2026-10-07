//! 係の門の判じ（判断の記録 ADR-59 決定 (3)(4)・要件 FR21）。
//! 結んだ係の使った量（測りの札の `used`）が札の予算以上の時は書き終えの段で、係の呼びのうち席への知らせ（SendMessage）と、
//! 書きの道具（Write・Edit・NotebookEdit）で係の出力の dir `<起草の置き場>/<名>/w` の下を書く呼びだけを通す。
//! path の下の判じ（`under`）は、`..` の成分を持たず dir の成分で始まる時だけ下と読む（ほかの係の dir と `..` を持つ path は断る）。
//! 群の係の割りの読む path の判じ（`meter::group::inside`）も同じ `under` で比べる。
//! 全時間の 3 つの検め（条 N-2）: 組みの検め（頼みの頭の組みが なし の係の Bash の command の頭の語が cargo なら断る・`cargo_word`）と、
//! 台帳と器の state の検め（係の Bash の command の頭の語が台帳か器の state を書く語なら組みによらず断る・`ledger_word`）と、
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

/// command の一続きごとの頭の語の列（頭が名・後ろが引数）。頭の代入と包みの語と旗と数は除き、頭の字 `({$` と backtick と斜線の前は除く。
/// shell の `-c` の次の語が在れば、それを command として読み直した列を shell の列の代わりに並べる。除いた後に語が残らない一続きは並べない。
fn heads(command: &str) -> Vec<Vec<String>> {
    segments(command)
        .iter()
        .flat_map(|seg| {
            let mut words = seg.iter().map(String::as_str).peekable();
            while words.next_if(|w| is_assignment(w)).is_some() {}
            while words.next_if(|w| WRAPS.contains(w)).is_some() {
                while words.next_if(|w| wrap_arg(w)).is_some() {}
            }
            let Some(head) = words.next() else {
                return Vec::new();
            };
            let head = head.trim_start_matches(['(', '{', '$', '`']);
            let name = head.rsplit('/').next().unwrap_or(head);
            let rest: Vec<&str> = words.collect();
            if SHELLS.contains(&name)
                && let Some(at) = rest
                    .iter()
                    .position(|w| w.starts_with('-') && !w.starts_with("--") && w.contains('c'))
                && let Some(inner) = rest.get(at + 1)
            {
                return heads(inner);
            }
            let line = std::iter::once(name).chain(rest).map(str::to_string).collect();
            vec![line]
        })
        .collect()
}

/// command の一続きごとの頭の語が cargo か cargo- で始まる名なら、その名（当たらなければ None・頭の語の割りは `heads`）。
pub fn cargo_word(command: &str) -> Option<String> {
    heads(command)
        .into_iter()
        .map(|line| line.into_iter().next().unwrap_or_default())
        .find(|name| name == "cargo" || name.starts_with("cargo-"))
}

/// 台帳の書きの口を持つ名の bdw（読みも通さず止める）と、台帳の名 bd。
const LEDGER_WRAP: &str = "bdw";
const LEDGER: &str = "bd";

/// bd の読みの口（bdw の読みの素通しの一覧と同じ字）。
const BD_READS: [&str; 22] = [
    "show",
    "list",
    "ready",
    "blocked",
    "search",
    "query",
    "count",
    "status",
    "stats",
    "info",
    "context",
    "history",
    "children",
    "state",
    "statuses",
    "types",
    "graph",
    "version",
    "prime",
    "human",
    "quickstart",
    "help",
];

/// bd の頭の旗のうち、次の語を値に取る旗。
const BD_VALUED: [&str; 5] = ["--actor", "--db", "--directory", "-C", "--dolt-auto-commit"];

/// tz の名と、その口のうち器の state を書く口。
const TZ_NAMES: [&str; 2] = ["tz", "tzw"];
const TZ_WRITES: [&str; 4] = ["surface", "hook", "stage", "consult"];

/// 器の名と、器の state の dir を渡す旗。
const VESSEL: &str = "scribe2";
const STATE_FLAG: &str = "--state-dir";

/// 引数の列の `--state-dir` の値（次の語・無ければ空の字）と `--state-dir=` の後ろの値。
fn states(args: &[String]) -> Vec<String> {
    let mut values = Vec::new();
    for (at, word) in args.iter().enumerate() {
        if word == STATE_FLAG {
            values.push(args.get(at + 1).cloned().unwrap_or_default());
        } else if let Some(value) = word
            .strip_prefix(STATE_FLAG)
            .and_then(|rest| rest.strip_prefix('='))
        {
            values.push(value.to_string());
        }
    }
    values
}

/// 頭の名 `name` と引数 `args` の一続きが台帳か器の state を書く語か（書く語の字・読みなら None）。
fn writes(name: &str, args: &[String], agent: &Path) -> Option<String> {
    if name == LEDGER_WRAP {
        return Some(name.to_string());
    }
    if name == LEDGER {
        let mut words = args.iter();
        let mouth = loop {
            match words.next() {
                Some(w) if BD_VALUED.contains(&w.as_str()) => {
                    words.next();
                }
                Some(w) if w.starts_with('-') => {}
                other => break other,
            }
        };
        return mouth
            .filter(|m| !BD_READS.contains(&m.as_str()))
            .map(|m| format!("{LEDGER} {m}"));
    }
    if TZ_NAMES.contains(&name) {
        return args
            .first()
            .filter(|w| TZ_WRITES.contains(&w.as_str()))
            .map(|w| format!("{name} {w}"));
    }
    if name != VESSEL {
        return None;
    }
    let first = args.first()?;
    if first == "--version" || first == "help" {
        return None;
    }
    if first == "pipe" && args.get(1).is_some_and(|w| w == "preflight") {
        let values = states(args);
        if !values.is_empty() && values.iter().all(|v| under(Path::new(v), agent)) {
            return None;
        }
    }
    Some(format!("{VESSEL} {first}"))
}

/// command の一続きごとの頭の語のうち、台帳か器の state を書く最初の語の字（`bdw`・`bd <口>`・`tz <口>`・`scribe2 <口>`・どれも当たらなければ None）。
/// `agent` は係の dir で、`scribe2 pipe preflight` は `--state-dir` の値が全部その下の時だけ通す。
pub fn ledger_word(command: &str, agent: &Path) -> Option<String> {
    heads(command)
        .iter()
        .find_map(|line| line.split_first().and_then(|(name, args)| writes(name, args, agent)))
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

/// 台帳と器の state の検めの断りの理由の字（撃たない語と、読みの手・state dir の置き場・席に頼む次の一手）。
pub fn ledger_refusal(word: &str, agent: &Path, out: &Path) -> String {
    format!(
        "係の門は止める（係の Bash で、台帳か器の state を書く語 {word} は撃たない） \
         次の一手 = 台帳は bd --readonly show か list で読み、preflight は scribe2 pipe preflight --state-dir {}/ の下の dir で撃ち、書きの要る事は {}/notes.md に書いて席に頼む",
        agent.display(),
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
