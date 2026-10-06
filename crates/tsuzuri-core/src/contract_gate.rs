//! 契約の bead の本文の散文の門（判断の記録 ADR-72 の決定 (2)）。
//! 入力は Claude Code の PreToolUse の hook の入力の字と台帳の一覧の字と呼び手が渡す撃ちの関数で、関数は file も子 process も時計も触らない。
//!
//! 台帳の書きの門（問いの起票の門と同じ hook）が、bd か bdw の create か update の一続きから契約の書き（`writes`）を拾う。
//! 甲は欄 acceptance の値が [[contract]] の行を持つ書きで、本文の形（file・file でない形・無し）を持つ。
//! 乙は欄 acceptance を持たず本文の旗だけを持つ update で、名指しの id を持つ。
//! 門は下書きを command の順に判じて最初に通さない答え（`judge`）を返し、止める答えは PreToolUse の deny の JSON（`output`）。
//! 甲の本文が file なら、撃ちの関数（file の path を受けて rc と標準出力の字を返す）が散文の門を撃つ。

use serde_json::Value;

use crate::gate::{CREATE, PROGRAMS, deny_json, is_assignment, segments};
use crate::graph::build::read_ledger;

/// 更新の語。
const UPDATE: &str = "update";

/// 欄 acceptance の旗。
const ACCEPTANCE: &str = "--acceptance";

/// 本文を持つ旗のうち file でない形の 3 つ。
const INLINE_FLAGS: [&str; 3] = ["--description", "-d", "--stdin"];

/// 本文の file の旗。
const BODY_FILE: &str = "--body-file";

/// 欄 acceptance の契約の行（前後の空白を落とした字が一致する行）。
const CONTRACT_LINE: &str = "[[contract]]";

/// 散文の門の口の要約の行の頭（違反の行から外す）。
const SUMMARY_HEAD: &str = "folio check --prose:";

/// 本文の形。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    /// --body-file の値（最後の値）が / で始まる絶対 path。
    File(String),
    /// --body-file の値が - か相対 path の時と、--description・-d・--stdin の時。
    NotFile,
    /// 本文の旗が無い。
    Absent,
}

/// 契約の書き。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractWrite {
    /// 甲: 契約の書き（欄 acceptance の値が [[contract]] の行を 1 つ以上持つ）。
    Contract { body: Body },
    /// 乙: 本文だけの直し（語 update を持ち、--acceptance を持たず、本文の旗を持つ・名指しの id は語 update の後の - で始まらない語）。
    BodyOnly { ids: Vec<String> },
}

/// 通さない理由（閉じた 4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractWhy {
    /// 契約の書きの本文が file でないか無い・契約の bead の本文だけの直し。
    BodyForm,
    /// 本文の file が散文の門で落ちる。
    Prose,
    /// 散文の門が まだ分からない（rc が 0 と 1 のほか）。
    ProseUnknown,
    /// 台帳が読めない。
    LedgerUnread,
}

impl ContractWhy {
    pub const ALL: [ContractWhy; 4] = [
        ContractWhy::BodyForm,
        ContractWhy::Prose,
        ContractWhy::ProseUnknown,
        ContractWhy::LedgerUnread,
    ];

    /// 理由の語。
    pub fn word(self) -> &'static str {
        match self {
            ContractWhy::BodyForm => "contract-body-form",
            ContractWhy::Prose => "contract-prose",
            ContractWhy::ProseUnknown => "contract-prose-unknown",
            ContractWhy::LedgerUnread => "contract-ledger-unread",
        }
    }

    /// 答えの字の次の一手の 1 文。
    fn next_move(self) -> &'static str {
        match self {
            ContractWhy::BodyForm => {
                "契約の行の --acceptance と本文の --body-file の絶対 path を 1 つの bd update に並べて書き直す。"
            }
            ContractWhy::Prose => {
                "本文の印を持つ文に参照 id を足すか参照 id の外の数と単位を外して書き直す。"
            }
            ContractWhy::ProseUnknown | ContractWhy::LedgerUnread => {
                "規則の表と台帳が読めるようになってから書き直す。"
            }
        }
    }
}

/// 門の答え（違反の行は `lines_of` の字・無ければ空）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractGate {
    Allow,
    Deny { why: ContractWhy, lines: Vec<String> },
    Unknown { why: ContractWhy, lines: Vec<String> },
}

/// 字が契約の行（前後の空白を落とした字が [[contract]]）を持つか。
fn has_contract_line(text: &str) -> bool {
    text.lines().any(|l| l.trim() == CONTRACT_LINE)
}

/// 旗 `flag` を持つか（`旗` か `旗=値`）。
fn has_flag(words: &[String], flag: &str) -> bool {
    words
        .iter()
        .any(|w| w == flag || w.strip_prefix(flag).is_some_and(|r| r.starts_with('=')))
}

/// 旗 `flag` の値（`旗 値` か `旗=値`・何度在っても最後の値）。
fn last_value(words: &[String], flag: &str) -> Option<String> {
    let mut last = None;
    for (i, w) in words.iter().enumerate() {
        if w == flag {
            last = words.get(i + 1).cloned().or(last);
        } else if let Some(v) = w.strip_prefix(flag).and_then(|r| r.strip_prefix('=')) {
            last = Some(v.to_string());
        }
    }
    last
}

/// 一続きの本文の形（本文の旗が無ければ無し）。
fn body_of(words: &[String]) -> Body {
    if !has_flag(words, BODY_FILE) && !INLINE_FLAGS.iter().any(|f| has_flag(words, f)) {
        return Body::Absent;
    }
    if INLINE_FLAGS.iter().any(|f| has_flag(words, f)) {
        return Body::NotFile;
    }
    match last_value(words, BODY_FILE) {
        Some(path) if path.starts_with('/') => Body::File(path),
        _ => Body::NotFile,
    }
}

/// bd か bdw の一続き（頭の代入の語の後の program が台帳の program で、語 create か update を持つもの）の語 `rest`（program の後）。
fn ledger_words(seg: &[String]) -> Option<&[String]> {
    let start = seg.iter().position(|w| !is_assignment(w))?;
    let (head, rest) = seg.get(start..)?.split_first()?;
    let program = head.rsplit('/').next().unwrap_or(head);
    let writes = rest.iter().any(|w| w == CREATE || w == UPDATE);
    (PROGRAMS.contains(&program) && writes).then_some(rest)
}

/// 一続きの契約の書き（甲か乙・どちらでもなければ None）。
fn classify(rest: &[String]) -> Option<ContractWrite> {
    if has_flag(rest, ACCEPTANCE) {
        let acceptance = last_value(rest, ACCEPTANCE)?;
        return has_contract_line(&acceptance).then(|| ContractWrite::Contract { body: body_of(rest) });
    }
    let update = rest.iter().position(|w| w == UPDATE)?;
    if body_of(rest) == Body::Absent {
        return None;
    }
    let ids = rest.get(update + 1..)?;
    Some(ContractWrite::BodyOnly {
        ids: ids.iter().filter(|w| !w.starts_with('-')).cloned().collect(),
    })
}

/// PreToolUse の hook の入力から契約の書きを command の順に拾う（Bash の呼び出しでなければ空）。
pub fn writes(payload: &str) -> Vec<ContractWrite> {
    let Ok(Value::Object(input)) = serde_json::from_str::<Value>(payload) else {
        return Vec::new();
    };
    if input.get("tool_name").and_then(Value::as_str) != Some("Bash") {
        return Vec::new();
    }
    let Some(command) = input
        .get("tool_input")
        .and_then(Value::as_object)
        .and_then(|t| t.get("command"))
        .and_then(Value::as_str)
    else {
        return Vec::new();
    };
    segments(command)
        .iter()
        .filter_map(|seg| classify(ledger_words(seg)?))
        .collect()
}

/// 散文の門の口の標準出力の字から、答えに写す行（前後の空白を落とした空でない行のうち、要約の行でないもの）。
pub fn lines_of(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with(SUMMARY_HEAD))
        .map(str::to_string)
        .collect()
}

/// 甲の本文の file を撃ちの関数で判じる（rc 0 は通す・rc 1 は止める・ほかは まだ分からない）。
fn by_prose(path: &str, fire: &impl Fn(&str) -> (u8, String)) -> ContractGate {
    let (rc, stdout) = fire(path);
    let lines = lines_of(&stdout);
    match rc {
        0 => ContractGate::Allow,
        1 => ContractGate::Deny { why: ContractWhy::Prose, lines },
        _ => ContractGate::Unknown { why: ContractWhy::ProseUnknown, lines },
    }
}

/// 乙を台帳の字で判じる（名指しの id のどれかが契約の bead なら止め・台帳が読めなければ まだ分からない）。
fn by_ledger(ids: &[String], ledger: &str) -> ContractGate {
    let Some(beads) = read_ledger(ledger) else {
        return ContractGate::Unknown { why: ContractWhy::LedgerUnread, lines: Vec::new() };
    };
    let contract = beads.iter().any(|b| {
        ids.contains(&b.id)
            && b.acceptance_criteria
                .as_deref()
                .is_some_and(has_contract_line)
    });
    if contract {
        ContractGate::Deny { why: ContractWhy::BodyForm, lines: Vec::new() }
    } else {
        ContractGate::Allow
    }
}

/// 契約の書きを command の順に判じ、最初に通さない答えを返す（全部通せば Allow・通さない答えの後の書きは撃たない）。
/// `fire` は甲の本文の file の path を受けて散文の門の rc と標準出力の字を返す。`ledger` は台帳の一覧の字（乙だけが読む）。
pub fn judge(
    writes: &[ContractWrite],
    fire: impl Fn(&str) -> (u8, String),
    ledger: &str,
) -> ContractGate {
    writes
        .iter()
        .map(|w| match w {
            ContractWrite::Contract { body: Body::File(path) } => by_prose(path, &fire),
            ContractWrite::Contract { .. } => {
                ContractGate::Deny { why: ContractWhy::BodyForm, lines: Vec::new() }
            }
            ContractWrite::BodyOnly { ids } => by_ledger(ids, ledger),
        })
        .find(|answer| *answer != ContractGate::Allow)
        .unwrap_or(ContractGate::Allow)
}

/// 答えの hook の JSON の字（通す時は None・止める時と まだ分からない時は PreToolUse の deny）。
pub fn output(answer: &ContractGate) -> Option<String> {
    let (head, why, lines) = match answer {
        ContractGate::Allow => return None,
        ContractGate::Deny { why, lines } => ("契約の本文の散文の門は止める（", why, lines),
        ContractGate::Unknown { why, lines } => ("契約の本文の散文の門は まだ分からない（", why, lines),
    };
    let shown = if lines.is_empty() {
        String::new()
    } else {
        format!(" {}", lines.join(" / "))
    };
    Some(deny_json(format!(
        "{head}{}）{shown} 次の一手 = {}",
        why.word(),
        why.next_move()
    )))
}
