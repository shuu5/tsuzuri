//! 計画の memo の上限の門（要件 FR4・規則の行 `CAP_ROW`）。
//! 入力は Claude Code の PreToolUse の hook の入力の字と導出グラフと規則の表の字で、関数は file も子 process も時計も触らない。
//!
//! 計画の memo は label `memo:plan` を持つ bead で、席は label `intake:memo` と並べて起票の時に置く。
//! 門は bd か bdw の呼び出しの一続きから、開いた計画の memo を増やす書き（`opens`）を command の順に拾い、
//! 開いた計画の memo の本数（label `memo:plan` を持ち `BeadAttr::is_open` が真の bead）との和が
//! 規則の表の上限（`cap`）を越えれば止める（`judge`）。止める答えは PreToolUse の deny の JSON（`output`）。
//! 上限の行が読めない間は、印の在る書きを まだ分からない で止める（値の読めない間は止める側に倒す常の口）。
//! 印は起票の時だけ置き、後から足す書きは本数を問わず止める。

use serde_json::Value;

use crate::gate::{CREATE, PROGRAMS, deny_json, is_assignment, label_values, segments};
use crate::graph::{BeadAttr, Graph, Source};

/// 計画の memo の印の label。
pub const PLAN_LABEL: &str = "memo:plan";

/// 開いた計画の memo の本数の上限の行の id（規則の表）。
pub const CAP_ROW: &str = "R-45";

/// 親の label を子に継がせない旗。
const NO_INHERIT: &str = "--no-inherit-labels";

/// 上限の行の value の末尾（`<0 以上の整数> 本 以下`）。
const VALUE_TAIL: &str = " 本 以下";

/// 開いた計画の memo を増やすか印の置き方を破る書き 1 つ（呼び出しの 1 続きごと）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Open {
    /// 印の起票（create で label に `memo:plan` を持つ）。
    Marked,
    /// 継ぐ起票の候補（create で label に印が無く、--parent を持ち、--no-inherit-labels を持たない・値は親の id）。
    Inherits(String),
    /// 起こし直しの候補（reopen か、--status が closed でない update・値は一続きの語）。
    Revives(Vec<String>),
    /// 後からの印（update の --add-label か --set-labels・label add・tag）。
    Late,
}

/// 通さない理由（閉じた 3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanWhy {
    /// 開いた計画の memo の本数と足す本数の和が上限を越える。
    Cap,
    /// 印を起票の後から足す書き。
    LateLabel,
    /// 上限の行か台帳が読めない。
    Unread,
}

impl PlanWhy {
    pub const ALL: [PlanWhy; 3] = [PlanWhy::Cap, PlanWhy::LateLabel, PlanWhy::Unread];

    /// 理由の語。
    pub fn word(self) -> &'static str {
        match self {
            PlanWhy::Cap => "plan-cap",
            PlanWhy::LateLabel => "plan-late-label",
            PlanWhy::Unread => "plan-cap-unread",
        }
    }

    /// 答えの字の次の一手の 1 文。
    fn next_move(self) -> &'static str {
        match self {
            PlanWhy::Cap => "済んだ計画の memo を閉じるか契約の bead へ移してから置き直す。",
            PlanWhy::LateLabel => "印 memo:plan は起票の時に label で置き、後から足さない。",
            PlanWhy::Unread => "規則の表の上限の行と台帳が読めるようになってから置き直す。",
        }
    }
}

/// 上限を越える時の数（開いた本数・足す本数・上限の値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub open: usize,
    pub add: usize,
    pub cap: usize,
}

/// 門の答え。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanGate {
    Allow,
    Deny { why: PlanWhy, counts: Option<Counts> },
    Unknown { why: PlanWhy },
}

/// 規則の表の字から開いた計画の memo の本数の上限を読む（行 `CAP_ROW` がちょうど 1 本で、
/// value が `<0 以上の整数> 本 以下` の形の時だけ・ほかは Err）。
pub fn cap(rules: &str) -> Result<usize, String> {
    let head = format!("- {{id: {CAP_ROW},");
    let mut rows = rules.lines().map(str::trim).filter(|l| l.starts_with(&head));
    let (Some(row), None) = (rows.next(), rows.next()) else {
        return Err(format!("規則の行 {CAP_ROW} がちょうど 1 本でない"));
    };
    let value = row
        .split_once(" value: \"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(value, _)| value)
        .ok_or_else(|| format!("規則の行 {CAP_ROW} に value が無い"))?;
    let number = value
        .strip_suffix(VALUE_TAIL)
        .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .ok_or_else(|| format!("規則の行 {CAP_ROW} の value の形が違う: {value}"))?;
    number
        .parse()
        .map_err(|e| format!("規則の行 {CAP_ROW} の value の数が読めない: {e}"))
}

/// 旗 `name` の値を command の順に全部拾う（`--名 値` か `--名=値`・旗が在って値が無ければ空の字）。
fn flag_values<'a>(words: &'a [String], name: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    for (i, w) in words.iter().enumerate() {
        if w == name {
            out.push(words.get(i + 1).map_or("", String::as_str));
        } else if let Some(v) = w.strip_prefix(name).and_then(|v| v.strip_prefix('=')) {
            out.push(v);
        }
    }
    out
}

/// コンマ区切りの値のどれかの項が印か。
fn has_plan(value: &str) -> bool {
    value.split(',').any(|l| l.trim() == PLAN_LABEL)
}

/// PreToolUse の hook の入力から bd か bdw の呼び出しの一続きの書きを command の順に拾う
/// （Bash の呼び出しでなければ空・語 list・show ほかの読みと label を外す旗は拾わない）。
pub fn opens(payload: &str) -> Vec<Open> {
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
    let mut out = Vec::new();
    for seg in segments(command) {
        let Some(start) = seg.iter().position(|w| !is_assignment(w)) else {
            continue;
        };
        let Some((head, rest)) = seg.get(start..).and_then(<[String]>::split_first) else {
            continue;
        };
        let program = head.rsplit('/').next().unwrap_or(head);
        if PROGRAMS.contains(&program) {
            segment_opens(rest, &mut out);
        }
    }
    out
}

/// bd か bdw の一続き 1 つ（頭の語を除いた語）の書きを `out` に足す。
fn segment_opens(rest: &[String], out: &mut Vec<Open>) {
    let has = |word: &str| rest.iter().any(|w| w == word);
    if has(CREATE) {
        if label_values(rest).into_iter().any(has_plan) {
            out.push(Open::Marked);
        } else if !has(NO_INHERIT)
            && let Some(parent) = flag_values(rest, "--parent").last()
        {
            out.push(Open::Inherits((*parent).to_string()));
        }
    }
    let revived = has("update")
        && flag_values(rest, "--status")
            .last()
            .is_some_and(|status| *status != "closed");
    if has("reopen") || revived {
        out.push(Open::Revives(rest.to_vec()));
    }
    let late_flag = has("update")
        && ["--add-label", "--set-labels"]
            .iter()
            .any(|flag| flag_values(rest, flag).into_iter().any(has_plan));
    let late_word = (has("label") && has("add") || has("tag")) && has(PLAN_LABEL);
    if late_flag || late_word {
        out.push(Open::Late);
    }
}

/// 台帳の bead が計画の memo か（label `memo:plan` を持つ）。
fn is_plan(attr: &BeadAttr) -> bool {
    attr.labels.iter().any(|l| l == PLAN_LABEL)
}

/// 開いた計画の memo の本数（台帳の bead のうち label `memo:plan` を持ち `BeadAttr::is_open` が真の物）。
fn open_plans(graph: &Graph) -> usize {
    graph
        .beads
        .values()
        .filter(|b| is_plan(b) && b.is_open())
        .count()
}

/// 開いた計画の memo を増やす本数（台帳が読めない時は継ぐ起票の候補を全部数える・同じ id の起こし直しは 1 度）。
fn added(opens: &[Open], graph: &Graph) -> usize {
    let unread = !graph.is_read(Source::Ledger);
    let mut revived: Vec<&str> = Vec::new();
    let mut count = 0;
    for open in opens {
        match open {
            Open::Marked => count += 1,
            Open::Inherits(parent) => {
                if unread || graph.beads.get(parent).is_some_and(is_plan) {
                    count += 1;
                }
            }
            Open::Revives(words) => {
                for word in words {
                    let closed_plan = graph
                        .beads
                        .get(word)
                        .is_some_and(|b| is_plan(b) && !b.is_open());
                    if closed_plan && !revived.contains(&word.as_str()) {
                        revived.push(word);
                        count += 1;
                    }
                }
            }
            Open::Late => {}
        }
    }
    count
}

/// 拾った書きを判じる（後からの印は止める・足す本数が 0 なら通す・上限か台帳が読めなければ まだ分からない・
/// 開いた本数と足す本数の和が上限を越えれば止める）。`cap` は規則の表から読んだ上限（読めなければ None）。
pub fn judge(opens: &[Open], graph: &Graph, cap: Option<usize>) -> PlanGate {
    if opens.contains(&Open::Late) {
        return PlanGate::Deny {
            why: PlanWhy::LateLabel,
            counts: None,
        };
    }
    let add = added(opens, graph);
    if add == 0 {
        return PlanGate::Allow;
    }
    let Some(cap) = cap.filter(|_| graph.is_read(Source::Ledger)) else {
        return PlanGate::Unknown { why: PlanWhy::Unread };
    };
    let open = open_plans(graph);
    if open + add > cap {
        return PlanGate::Deny {
            why: PlanWhy::Cap,
            counts: Some(Counts { open, add, cap }),
        };
    }
    PlanGate::Allow
}

/// hook の答えの JSON の字（通すときは None・止めるときは PreToolUse の deny と理由）。
pub fn output(gate: &PlanGate) -> Option<String> {
    let (head, why, counts) = match gate {
        PlanGate::Allow => return None,
        PlanGate::Deny { why, counts } => ("計画の memo の上限の門は止める（", why, counts),
        PlanGate::Unknown { why } => ("計画の memo の上限の門は まだ分からない（", why, &None),
    };
    let mut reason = format!("{head}{}）", why.word());
    if let Some(Counts { open, add, cap }) = counts {
        reason.push_str(&format!(" 開いた計画の memo {open}・足す {add}・上限 {cap}"));
    }
    reason.push_str(" 次の一手 = ");
    reason.push_str(why.next_move());
    Some(deny_json(reason))
}
