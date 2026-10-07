//! host-guard の 7 つ目の種類 self-match（判断の記録 ADR-44 の決定 (3)・束 B05・rules 行 host_guard.self_match・SRS FR1056）。
//!
//! `pgrep -f` と `pkill -f` は型を process の command 行の全体に当てる。撃った shell の command 行も型の字を持つので、型が
//! その字に当たる形では待ちが終わらず、`pkill` は撃った shell を止める。呼びの読み（[`calls`]）は command 行を段に分け
//! （[`levels`]・単引用符の外の置換と subshell の本文を別の段へ）、段ごとに起票の門の分割（[`segments`]）で切った segment の
//! 頭（制御の語と `NAME=value` を読み飛ばし、launcher を剥いだ動詞・[`verb_of`]）が pgrep か pkill の呼びを getopt の形で読む。
//! 型の判じ（[`verdict`]）は字だけで決まる形だけを断る: 変数や置換で字の決まらない型と、型の字が command 行に字のまま在り
//! `|` で割ったどれかの枝が字と `.` と `.*` と `.+` だけの型（自分の字に当たる）。角括弧などを持つ型は決めずに通す。子 process
//! を撃たず、fs も読まない。

use super::{verb_of, Kind, Refusal, Subject, SELF_MATCH_ROW};
use crate::hook::ledger_guard::{is_assignment, segments};
use crate::rules::manifest::Manifest;
use crate::rules::RuleValue;

/// 呼びの動詞（launcher を剥いだ頭の語の basename）。
const VERBS: [&str; 2] = ["pgrep", "pkill"];
/// segment の頭で読み飛ばす制御の語（後ろの語が command の頭）。
const KEYWORDS: [&str; 10] = ["while", "until", "if", "elif", "then", "do", "else", "!", "time", "{"];
/// full の長い flag。
const FULL: &str = "--full";
/// full の短い字。
const FULL_SHORT: char = 'f';
/// 値を取る短い flag の字（束の残りか、束の末なら次の語が値・大文字の F は pidfile）。
const SHORT_VALUED: [char; 12] = ['d', 'g', 'G', 'O', 'P', 'q', 's', 't', 'u', 'U', 'F', 'r'];
/// 値を取る長い flag（`=` が無ければ次の語が値）。
const LONG_VALUED: [&str; 16] = [
    "--delimiter", "--pgroup", "--group", "--older", "--parent", "--session", "--signal", "--terminal", "--euid", "--uid",
    "--pidfile", "--runstates", "--cgroup", "--ns", "--nslist", "--queue",
];
/// flag の読みを終える語（後ろは全部が非 flag）。
const END: &str = "--";
/// 型の判じを決めない字（角括弧・丸括弧・波括弧・`^`・逆斜線・`$`〔名の前の `$` は先に解けない形へ〕）。
const OPAQUE: [char; 9] = ['[', ']', '(', ')', '{', '}', '^', '\\', '$'];
/// 自分の字に当たる繰り返しの字（`.` の直後だけ）。
const REPEATS: [char; 2] = ['*', '+'];
/// 型の枝の区切り。
const BRANCH: char = '|';

/// pgrep / pkill の呼び 1 つ。
#[derive(Debug, Clone, PartialEq, Eq)]
struct Call {
    /// 動詞（[`VERBS`] の 1 つ）。
    verb: &'static str,
    /// `--full` か、値を取る字より前に `f` を持つ短い束を持つ。
    full: bool,
    /// 最初の非 flag の語（無ければ `None`）。
    pattern: Option<String>,
}

/// 型の判じ（閉じた 3 値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// 型が自分の command 行の字に当たる。
    SelfMatch,
    /// 変数か置換で型の字が決まらない。
    Unresolved,
    /// 当たらないか、字だけでは決めない。
    Other,
}

/// 判定（宣言の末の種類）: Bash の command 行に full の呼びが無ければ行を読まずに通す（編集系の道具の周は command 行が空）。在れば行 [`SELF_MATCH_ROW`] を読み、無い・列でない
/// 周は no-row で断り（FailClosed）、`enabled = false` の周は通す。行の語列を順に見て、頭の語と同じ動詞の full の呼びの型が
/// 自分に当たる形か字の決まらない形なら、hit をその語列にして断る。
pub(super) fn judge(kind: Kind, subject: &Subject, manifest: &Manifest) -> Option<Refusal> {
    let found: Vec<Call> = calls(subject.command).into_iter().filter(|call| call.full).collect();
    if found.is_empty() {
        return None;
    }
    let Some(row) = manifest.get(SELF_MATCH_ROW) else {
        return Some(Refusal::no_row(kind, SELF_MATCH_ROW));
    };
    if !row.enabled {
        return None;
    }
    let RuleValue::List(ref sequences) = row.value else {
        return Some(Refusal::no_row(kind, SELF_MATCH_ROW));
    };
    let refused = |call: &Call| call.pattern.as_deref().is_some_and(|pattern| verdict(pattern, subject.command) != Verdict::Other);
    let hit = sequences.iter().find(|sequence| {
        let verb = sequence.split_whitespace().next();
        found.iter().any(|call| verb == Some(call.verb) && refused(call))
    })?;
    Some(Refusal { kind, hit: hit.clone(), row: SELF_MATCH_ROW, ruling: row.ruling.clone(), route: kind.route() })
}

/// command 行の pgrep と pkill の呼び（[`levels`] の段ごとに [`segments`] で切り、segment ごとに [`call_of`] で読む）。
fn calls(command: &str) -> Vec<Call> {
    levels(command).iter().flat_map(|level| segments(level)).filter_map(|words| call_of(&words)).collect()
}

/// segment 1 つの呼び: 頭の制御の語と `NAME=value` を読み飛ばし、launcher を剥いだ動詞が pgrep か pkill なら後ろの語を読む。
fn call_of(words: &[String]) -> Option<Call> {
    let at = words.iter().position(|word| !KEYWORDS.contains(&word.as_str()) && !is_assignment(word))?;
    let (verb, rest) = verb_of(words.get(at..)?)?;
    let verb = VERBS.into_iter().find(|found| *found == verb)?;
    let (full, pattern) = read_args(rest);
    Some(Call { verb, full, pattern })
}

/// 呼びの後ろの語を getopt の形で読む（flag の順は問わない・`--` の後ろは全部が非 flag）: `--full` か、値を取る字より前に
/// `f` を持つ短い束を full と読み、値を取る flag の値（束の残りか次の語・長い名は `=` が無ければ次の語）を飛ばし、最初の
/// 非 flag の語を型にする。
fn read_args(rest: &[String]) -> (bool, Option<String>) {
    let (mut full, mut pattern, mut ended) = (false, None::<String>, false);
    let mut words = rest.iter();
    while let Some(word) = words.next() {
        let valued = if ended || word.len() < 2 || !word.starts_with('-') {
            pattern.get_or_insert_with(|| word.clone());
            false
        } else if word == END {
            ended = true;
            false
        } else if word.starts_with(END) {
            full |= word == FULL;
            LONG_VALUED.contains(&word.as_str())
        } else {
            let (bundled, takes_next) = bundle(word);
            full |= bundled;
            takes_next
        };
        if valued {
            words.next();
        }
    }
    (full, pattern)
}

/// 短い flag の束（頭の `-` の後ろの字）: 値を取る字より前に `f` を持つかと、値を取る字が束の末で次の語を値に取るか。
fn bundle(word: &str) -> (bool, bool) {
    let letters: Vec<char> = word.chars().skip(1).collect();
    let valued = letters.iter().position(|letter| SHORT_VALUED.contains(letter));
    let full = letters.iter().take(valued.unwrap_or(letters.len())).any(|letter| *letter == FULL_SHORT);
    (full, valued.is_some_and(|at| at.saturating_add(1) == letters.len()))
}

/// command 行の段: 1 つ目は command 行で、単引用符の外の置換と subshell（`$(…)`・`` `…` ``・`(…)`）を中の無い対（`$()`・
/// ``` `` ```・`()`）に替えた字（中の字を外の segment の語に混ぜない）、後ろは対の中の本文ごとの段（入れ子も同じ形・閉じない
/// 本文は末まで）。
fn levels(command: &str) -> Vec<String> {
    let chars: Vec<char> = command.chars().collect();
    let (mut flat, mut inner) = (String::new(), Vec::new());
    let (mut at, mut single, mut double) = (0_usize, false, false);
    while let Some(found) = chars.get(at) {
        if let Some((open, close, empty)) = opening(&chars, at, single, double) {
            let start = at.saturating_add(open);
            let end = closing(&chars, start, close);
            inner.extend(levels(&chars.get(start..end).unwrap_or_default().iter().collect::<String>()));
            flat.push_str(empty);
            at = end.saturating_add(1);
            continue;
        }
        let step = match found {
            '\'' if !double => {
                single = !single;
                1
            }
            '"' if !single => {
                double = !double;
                1
            }
            '\\' if !single => 2,
            _ => 1,
        };
        let next = at.saturating_add(step).min(chars.len());
        flat.extend(chars.get(at..next).unwrap_or_default());
        at = next;
    }
    std::iter::once(flat).chain(inner).collect()
}

/// `at` の字が置換か subshell を開くなら（開きの字の数・閉じの字・外の段に残す対）。単引用符の中は開かず、二重引用符の中の
/// `(` は字（subshell でない）。
fn opening(chars: &[char], at: usize, single: bool, double: bool) -> Option<(usize, char, &'static str)> {
    if single {
        return None;
    }
    match (chars.get(at)?, chars.get(at.saturating_add(1))) {
        ('`', _) => Some((1, '`', "``")),
        ('$', Some('(')) => Some((2, ')', "$()")),
        ('(', _) if !double => Some((1, ')', "()")),
        _ => None,
    }
}

/// 本文の始まり `start` から閉じの字の位置（閉じなければ末）。`)` は引用符の外の丸括弧の入れ子を数え、逆斜線の次の字は読まない。
fn closing(chars: &[char], start: usize, close: char) -> usize {
    let (mut at, mut depth, mut quote) = (start, 0_usize, None::<char>);
    while let Some(found) = chars.get(at) {
        match (quote, *found) {
            (Some(open), now) if now == open => quote = None,
            (Some('\''), _) => {}
            (_, '\\') => at = at.saturating_add(1),
            (Some(_), _) => {}
            (None, now @ ('\'' | '"')) if close == ')' => quote = Some(now),
            (None, '(') if close == ')' => depth = depth.saturating_add(1),
            (None, now) if now == close && depth == 0 => return at,
            (None, ')') => depth = depth.saturating_sub(1),
            (None, _) => {}
        }
        at = at.saturating_add(1);
    }
    chars.len()
}

/// 型の判じ: 逆引用符か名の前の `$` を持てば解けない、[`OPAQUE`] の字を持つか型の字が command 行に字のまま無ければ決めない
/// （通す）、`|` で割った枝のどれかが自分の字に当たる形（[`own`]）なら自分に当たる。
fn verdict(pattern: &str, command: &str) -> Verdict {
    if pattern.contains('`') || named(pattern) {
        Verdict::Unresolved
    } else if pattern.contains(OPAQUE) || !command.contains(pattern) || !pattern.split(BRANCH).any(own) {
        Verdict::Other
    } else {
        Verdict::SelfMatch
    }
}

/// `$` の後に英字か `_` か波括弧か丸括弧が続く字を持つか（変数と置換）。
fn named(pattern: &str) -> bool {
    pattern.split('$').skip(1).any(|rest| rest.starts_with(|next: char| next.is_ascii_alphabetic() || matches!(next, '_' | '{' | '(')))
}

/// 枝が自分の字に当たる形か: `?` を持たず、[`REPEATS`] の字が全部 `.` の直後（字と `.` と `.*` と `.+` だけ）。
fn own(branch: &str) -> bool {
    let chars: Vec<char> = branch.chars().collect();
    let after_dot = |at: usize| at.checked_sub(1).and_then(|before| chars.get(before)) == Some(&'.');
    !chars.contains(&'?') && chars.iter().enumerate().all(|(at, found)| !REPEATS.contains(found) || after_dot(at))
}

#[cfg(test)]
mod tests {
    use super::super::{HostGuardDecision, Kind, Refusal, Scene, Subject, BASH};
    use super::{calls, judge, verdict, Call, Verdict, KEYWORDS};
    use crate::name::NAME;
    use crate::rules::manifest::Manifest;
    use std::path::Path;

    /// 呼び 1 つ（型あり）。
    fn call(verb: &'static str, full: bool, pattern: &str) -> Call {
        Call { verb, full, pattern: Some(pattern.to_owned()) }
    }

    /// full の呼びを 1 つだけ読む見本（command・動詞・型）。
    fn full_samples() -> Vec<(String, &'static str, &'static str)> {
        let mut samples: Vec<(String, &str, &str)> = [
            ("pgrep -f x", "pgrep", "x"),
            ("pkill -f x", "pkill", "x"),
            ("while pgrep -f 'x y'; do sleep 1; done", "pgrep", "x y"),
            ("until pgrep --full x; do :; done", "pgrep", "x"),
            ("X=1 pgrep -f x", "pgrep", "x"),
            ("sudo pgrep -af x", "pgrep", "x"),
            ("/usr/bin/pgrep -f x", "pgrep", "x"),
            ("kill $(pgrep -f x)", "pgrep", "x"),
            ("echo \"$(pgrep -f x)\"", "pgrep", "x"),
            ("n=`pkill -fc x`", "pkill", "x"),
            ("(sleep 1; pgrep -f x)", "pgrep", "x"),
            ("(cd a; pgrep -f '[m]y')", "pgrep", "[m]y"),
            ("pgrep -u root -f x", "pgrep", "x"),
            ("pgrep -d , -f x", "pgrep", "x"),
            ("pgrep --parent 1 -f x", "pgrep", "x"),
            ("pkill --signal=KILL -f x", "pkill", "x"),
            ("pgrep x -f", "pgrep", "x"),
            ("pgrep -f -- -x", "pgrep", "-x"),
            ("echo $( (cd a); pgrep -f x)", "pgrep", "x"),
            ("kill $(pgrep -f 'a)b')", "pgrep", "a)b"),
        ]
        .iter()
        .map(|(command, verb, pattern)| ((*command).to_owned(), *verb, *pattern))
        .collect();
        samples.extend(KEYWORDS.iter().map(|word| (format!("{word} pgrep -f x"), "pgrep", "x")));
        samples
    }

    /// (1) 呼びの読み: segment の頭・制御の語の後・置換と subshell の段の中の pgrep と pkill を、getopt の形で full と型に読む。
    /// full でない見本は full の見本 `pgrep -f x` から 1 句だけ外し（flag 無し・大文字の F・-u の値の f・-- の後の -f）、
    /// 呼びでない見本は動詞が頭に無いか単引用符の中か名が違う。
    #[test]
    fn vselfm_finds_pgrep_and_pkill_with_the_full_flag() {
        for (command, verb, pattern) in full_samples() {
            assert_eq!(calls(&command), vec![call(verb, true, pattern)], "{command}");
        }
        for (command, pattern) in [("pgrep x", "x"), ("pgrep -F f x", "x"), ("pgrep -uf x", "x"), ("pgrep -u f x", "x"), ("pgrep -- -f x", "-f")] {
            assert_eq!(calls(command), vec![call("pgrep", false, pattern)], "{command}");
        }
        for command in ["echo pgrep -f x", "grep -f x pgrep", "echo '$(pgrep -f x)'", "pgrepx -f x", "echo '(pkill -f x)'", "echo \"(pkill -f x)\""] {
            assert_eq!(calls(command), Vec::new(), "{command}");
        }
    }

    /// (2) 型の判じ: 型の字が command 行に字のまま在り、どれかの枝が字と `.` と `.*` と `.+` だけなら自分に当たり、自分に当たる
    /// 見本 `merge.sh` に字を 1 つ足した型（判じを決めない字・`.` でない字の後の繰り返し・`?`）と、字のまま無い command は通す。
    #[test]
    fn vselfm_bracket_or_caret_patterns_pass() {
        let quoted = |pattern: &str| format!("pgrep -f '{pattern}'");
        for pattern in ["merge.sh", "merge-scribe2.sh 5eaf4426", "a.*b", "x.+y", "cargo|rustc", "ab*c|merge.sh"] {
            assert_eq!(verdict(pattern, &quoted(pattern)), Verdict::SelfMatch, "{pattern}");
        }
        let opaque = ["merge[.sh", "merge.sh]", "merge(.sh", "merge.sh)", "merge{.sh", "merge.sh}", "^merge.sh", "merge\\.sh", "merge.sh$"];
        for pattern in opaque.into_iter().chain(["mer*ge.sh", "merg+e.sh", "merge.s?h", "ab*c|x?y", "[m]erge.sh"]) {
            assert_eq!(verdict(pattern, &quoted(pattern)), Verdict::Other, "{pattern}");
        }
        assert_eq!(verdict("merge.sh", "pgrep -f merge\\.sh"), Verdict::Other, "command 行に字のまま無い");
    }

    /// 行 host_guard.self_match（値は語列の列・`enabled` は引数）の本文。
    fn self_row(value: &str, enabled: bool) -> String {
        format!("\n[[rule]]\nid = \"host_guard.self_match\"\nkind = \"HostGuardDeniedCommands\"\nvalue = {value}\nenabled = {enabled}\nruling = \"r\"\nruled_at = \"d\"\n")
    }

    /// `rules`（行の本文）の manifest で Bash の command を判じた断りの 1 行（通せば `None`）。
    fn judged(command: &str, rules: &str) -> Option<String> {
        let manifest = Manifest::parse(&format!("schema = 1\n{rules}")).unwrap_or_else(|errors| panic!("fixture を読める: {errors:?}"));
        let host = Manifest::default();
        let scene = Scene { cwd: Path::new("/nonexistent"), state_dir: Path::new("/nonexistent/state"), git: Path::new("git"), gh: Path::new("gh"), host: &host };
        let subject = Subject { tool: BASH, segments: Vec::new(), command, edited: None, scene: &scene };
        judge(Kind::SelfMatch, &subject, &manifest).map(Refusal::decision).map(|decision| match decision {
            HostGuardDecision::Deny { what, line } => format!("{what} | {line}"),
            HostGuardDecision::Allow => panic!("断りは Deny に写る"),
        })
    }

    /// 変数と置換の型は解けない（`$` の後が数字か無い型は決めない）。
    fn unresolved_verdicts() {
        for pattern in ["$P", "${P}", "$(cat f)", "`cat f`", "x$_y"] {
            assert_eq!(verdict(pattern, &format!("pgrep -f \"{pattern}\"")), Verdict::Unresolved, "{pattern}");
        }
        for pattern in ["x$1", "x$"] {
            assert_eq!(verdict(pattern, &format!("pgrep -f '{pattern}'")), Verdict::Other, "{pattern}");
        }
    }

    /// (3) 変数と置換の型は解けずに断り（`$` の後が数字か無い型は決めない）、断りの hit は当たった動詞の行の語列で、行に無い
    /// 動詞の呼びと full でない呼びと自分に当たらない型は通し、行が無い・列でない周は no-row で断り、`enabled = false` の周と
    /// full の呼びが無い周（行が無くても）は通す。
    // flip-check: retroactive t3-hub.92.10.34
    #[test]
    fn vselfm_unresolved_patterns_are_refused() {
        unresolved_verdicts();
        let both = self_row("[\"pgrep -f\", \"pkill -f\"]", true);
        let refusal = |hit: &str, ruling: &str| {
            format!("host-guard-deny self-match | {NAME}: host-guard deny kind=self-match hit={hit} row=host_guard.self_match ruling={ruling} — {}", Kind::SelfMatch.route())
        };
        assert_eq!(judged("while pgrep -f 'merge.sh 5'; do sleep 15; done", &both), Some(refusal("pgrep -f", "r")), "待ち");
        assert_eq!(judged("pkill -f merge.sh", &both), Some(refusal("pkill -f", "r")), "止め");
        assert_eq!(judged("pgrep -f \"$P\"", &both), Some(refusal("pgrep -f", "r")), "解けない型");
        assert_eq!(judged("pgrep -f '[m]erge.sh'", &both), None, "自分に当たらない型");
        assert_eq!(judged("pgrep merge.sh", &both), None, "full でない呼び");
        let pkill_only = self_row("[\"pkill -f\"]", true);
        assert_eq!(judged("pgrep -f merge.sh", &pkill_only), None, "行に無い動詞");
        assert_eq!(judged("pgrep -f merge.sh; pkill -f merge.sh", &pkill_only), Some(refusal("pkill -f", "r")), "行に在る動詞");
        assert_eq!(judged("pgrep -f merge.sh", &self_row("[\"pgrep -f\"]", false)), None, "切った行");
        assert_eq!(judged("pgrep -f merge.sh", ""), Some(refusal("no-row", "-")), "行が無い");
        let not_list = "\n[[rule]]\nid = \"host_guard.self_match\"\nkind = \"ModuleLines\"\nvalue = 1\nenabled = true\nruling = \"r\"\nruled_at = \"d\"\n";
        assert_eq!(judged("pgrep -f merge.sh", not_list), Some(refusal("no-row", "-")), "列でない行");
        assert_eq!(judged("pgrep merge.sh", ""), None, "full の呼びが無い周は行を読まない");
    }
}
