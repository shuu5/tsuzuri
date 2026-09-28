//! 問いの起票の門（設計ノート surface-wavef 行 f-gate・要件 FR4・受入 AC6）。
//! 入力は Claude Code の PreToolUse の hook の入力の字と導出グラフで、関数は file も子 process も時計も触らない。
//!
//! 席は問いを bd か bdw の create で置く（label `intake:question`・名指した節点は metadata の touches）。
//! 門は問いの起票の下書き（`drafts`）ごとに、touches の各節点の 1 段の近傍のうち 4 種（条・規則行・判断の記録・要件）で
//! hub でない節点（`needs`）の全部に処分（touches か not-relevant）が在り、metadata の digest が
//! 今の束の要約値（`bundle_digest`）と同じときだけ通す（`judge`）。止める答えは PreToolUse の deny の JSON（`output`）。
//! 門は allow を出さない（通すことは止めないことで、ほかの許可の仕組みはそのまま効く）。

use std::collections::BTreeSet;

use serde_json::{Map, Value};
use tsuzuri_contract::graph::{NodeKind, natural_cmp};
use tsuzuri_contract::ledger::{QUESTION_LABEL, fnv1a64};

use crate::graph::{Graph, Source};
use crate::question::touches;

/// 次数がこれを越える節点が hub（処分の要る節点から除く）。
pub const HUB_DEGREE: usize = 30;

/// 処分の要る節点がこれを越える問いは まだ分からない（touches を絞った問いに分けさせる）。
pub const NEEDS_CAP: usize = 30;

/// 処分の要る節点の種類（条・規則行・判断の記録・要件）。
pub const KINDS: [NodeKind; 4] = [NodeKind::Article, NodeKind::Rule, NodeKind::Adr, NodeKind::Req];

/// 処分の鍵（id から理由の 1 句の字への object）。
pub const NOT_RELEVANT: &str = "not-relevant";

/// 束の要約値の鍵（門の答えの要約値を写す）。
pub const DIGEST: &str = "digest";

/// 台帳の program の名（呼び出しの最初の語の最後の斜線の後の字）。
const PROGRAMS: [&str; 2] = ["bd", "bdw"];

/// 起票の語。
const CREATE: &str = "create";

/// 問いの起票の下書き（呼び出しの 1 続きの create のうち label に `intake:question` を持つもの）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    /// --metadata の値（何度在っても最後の値・無ければ無し）。
    pub metadata: Option<String>,
}

/// 通さない理由（閉じた 8）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// 門の引数の誤り（境界が組む）。
    Args,
    /// metadata の字が JSON の object として読めない。
    Metadata,
    /// touches が 1 つも無い。
    NoTouches,
    /// 設計の索引か台帳が読めない。
    Unread,
    /// touches の id がグラフに無い。
    UnknownId,
    /// 処分の要る節点が `NEEDS_CAP` を越える。
    TooMany,
    /// 処分の無い節点が在る。
    Undisposed,
    /// metadata の digest が今の束の要約値と違う。
    Stale,
}

impl Why {
    pub const ALL: [Why; 8] = [
        Why::Args,
        Why::Metadata,
        Why::NoTouches,
        Why::Unread,
        Why::UnknownId,
        Why::TooMany,
        Why::Undisposed,
        Why::Stale,
    ];

    /// 理由の語。
    pub fn word(self) -> &'static str {
        match self {
            Why::Args => "args",
            Why::Metadata => "metadata",
            Why::NoTouches => "no-touches",
            Why::Unread => "unread",
            Why::UnknownId => "unknown-id",
            Why::TooMany => "too-many",
            Why::Undisposed => "undisposed",
            Why::Stale => "stale",
        }
    }

    /// 答えの字の次の一手の 1 文。
    fn next_move(self) -> &'static str {
        match self {
            Why::Args => "席の設定の tz hook question-gate の引数を直す。",
            Why::Metadata => "--metadata に JSON の object の字を渡して置き直す。",
            Why::NoTouches => "metadata の touches に名指す節点を 1 つ以上書いて置き直す。",
            Why::Unread => "設計の索引と台帳が読めるようになってから置き直す。",
            Why::UnknownId => "touches から索引と台帳に無い節点を除くか直して置き直す。",
            Why::TooMany => "touches を絞った問いに分けて置き直す。",
            Why::Undisposed => {
                "名指した節点を touches か not-relevant に足し、答えの要約値を metadata の digest に写して置き直す。"
            }
            Why::Stale => "答えの要約値を metadata の digest に写して置き直す。",
        }
    }
}

/// 門の答え。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    Allow,
    Deny {
        why: Why,
        ids: Vec<String>,
        digest: Option<String>,
    },
    Unknown {
        why: Why,
        ids: Vec<String>,
        digest: Option<String>,
    },
}

/// command を一続きの語の列に分ける（器の起票の門と同じ分け方）。
/// 引用符の外では空白で語を切り、セミコロン・アンパサンド・縦棒・改行で語と一続きを切り、逆斜線は次の 1 字を足す。
/// 一重引用符の中は字のまま、二重引用符の中は逆斜線が次の 1 字を足す。引用符だけの空の語も 1 語。
/// 閉じない引用符は command の終わりまで続く。語の無い一続きは捨てる。
pub fn segments(command: &str) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = Vec::new();
    let mut seg: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut in_word = false;
    let mut chars = command.chars();
    let end_word = |seg: &mut Vec<String>, word: &mut String, in_word: &mut bool| {
        if *in_word {
            seg.push(std::mem::take(word));
            *in_word = false;
        }
    };
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                in_word = true;
                for c in chars.by_ref() {
                    if c == '\'' {
                        break;
                    }
                    word.push(c);
                }
            }
            '"' => {
                in_word = true;
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => word.extend(chars.next()),
                        _ => word.push(c),
                    }
                }
            }
            '\\' => {
                in_word = true;
                word.extend(chars.next());
            }
            ';' | '&' | '|' | '\n' => {
                end_word(&mut seg, &mut word, &mut in_word);
                if !seg.is_empty() {
                    out.push(std::mem::take(&mut seg));
                }
            }
            c if c.is_whitespace() => end_word(&mut seg, &mut word, &mut in_word),
            c => {
                in_word = true;
                word.push(c);
            }
        }
    }
    end_word(&mut seg, &mut word, &mut in_word);
    if !seg.is_empty() {
        out.push(seg);
    }
    out
}

/// 頭の代入の語か（等号を持ち、等号の前が空でなく英数字と下線だけ）。
fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

/// label の旗の値（`--labels 値`・`--label 値`・`-l 値`・`--labels=値`・`--label=値`・`-l=値`・`-l値`）。
fn label_values(words: &[String]) -> Vec<&str> {
    let mut out = Vec::new();
    for (i, w) in words.iter().enumerate() {
        let w = w.as_str();
        if matches!(w, "--labels" | "--label" | "-l") {
            out.extend(words.get(i + 1).map(String::as_str));
        } else if let Some(v) = ["--labels=", "--label=", "-l="]
            .iter()
            .find_map(|p| w.strip_prefix(p))
        {
            out.push(v);
        } else if !w.starts_with("--") && w.chars().count() >= 3 {
            out.extend(w.strip_prefix("-l"));
        }
    }
    out
}

/// --metadata の値（`--metadata 値` か `--metadata=値`・何度在っても最後の値）。
fn metadata_value(words: &[String]) -> Option<String> {
    let mut last = None;
    for (i, w) in words.iter().enumerate() {
        if w == "--metadata" {
            if let Some(v) = words.get(i + 1) {
                last = Some(v.clone());
            }
        } else if let Some(v) = w.strip_prefix("--metadata=") {
            last = Some(v.to_string());
        }
    }
    last
}

/// PreToolUse の hook の入力から問いの起票の下書きを command の順に拾う（Bash の呼び出しでなければ空）。
pub fn drafts(payload: &str) -> Vec<Draft> {
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
        .into_iter()
        .filter_map(|seg| {
            let start = seg.iter().position(|w| !is_assignment(w))?;
            let head = &seg[start];
            let program = head.rsplit('/').next().unwrap_or(head);
            let rest = &seg[start + 1..];
            let question = PROGRAMS.contains(&program)
                && rest.iter().any(|w| w == CREATE)
                && label_values(rest)
                    .iter()
                    .any(|v| v.split(',').any(|l| l.trim() == QUESTION_LABEL));
            question.then(|| Draft {
                metadata: metadata_value(rest),
            })
        })
        .collect()
}

/// 重ねずに natural_cmp の順に並べる。
fn sorted(ids: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out: Vec<String> = ids.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
    out.sort_by(|a, b| natural_cmp(a, b));
    out
}

/// 処分の要る節点（touches の各 id の 1 段の隣のうち、種類が `KINDS` で次数が `HUB_DEGREE` 以下のもの・natural_cmp の順）。
pub fn needs(graph: &Graph, touches: &[String]) -> Vec<String> {
    let index = graph.index();
    let degrees = graph.degrees();
    let mut out: BTreeSet<String> = BTreeSet::new();
    for e in &graph.edges {
        if e.from == e.to || !index.contains_key(e.from.as_str()) || !index.contains_key(e.to.as_str())
        {
            continue;
        }
        for (end, other) in [(&e.from, &e.to), (&e.to, &e.from)] {
            if !touches.contains(end) {
                continue;
            }
            let kind_ok = index
                .get(other.as_str())
                .is_some_and(|n| KINDS.contains(&n.kind));
            let hub = degrees.get(other.as_str()).copied().unwrap_or(0) > HUB_DEGREE;
            if kind_ok && !hub {
                out.insert(other.clone());
            }
        }
    }
    sorted(out)
}

/// 束の要約値（touches のうちグラフに在る id と `needs` の和を natural_cmp の順に 1 節点 1 行で
/// 「id・要約値・状態・題」をタブでつなぎ、行を改行でつないだ字の FNV-1a 64 bit・16 字の 16 進の小文字）。
pub fn bundle_digest(graph: &Graph, touches: &[String]) -> String {
    let bundle = sorted(
        touches
            .iter()
            .filter(|id| graph.node(id).is_some())
            .cloned()
            .chain(needs(graph, touches)),
    );
    let text = bundle
        .iter()
        .filter_map(|id| graph.node(id))
        .map(|n| {
            [
                n.id.as_str(),
                n.digest.as_deref().unwrap_or_default(),
                &graph.status(n).unwrap_or_default(),
                &n.title,
            ]
            .join("\t")
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{:016x}", fnv1a64(text.as_bytes()))
}

fn deny(why: Why, ids: Vec<String>, digest: Option<String>) -> Gate {
    Gate::Deny { why, ids, digest }
}

/// 1 つの下書きを判じる。
fn judge_one(draft: &Draft, graph: &Graph) -> Gate {
    let object = match &draft.metadata {
        None => Map::new(),
        Some(text) => match serde_json::from_str::<Value>(text) {
            Ok(Value::Object(o)) => o,
            _ => return deny(Why::Metadata, Vec::new(), None),
        },
    };
    let touches = touches(&Value::Object(object.clone()));
    if touches.is_empty() {
        return deny(Why::NoTouches, Vec::new(), None);
    }
    // 走行の出所は読まない（走行の節点は 4 種の外で、走行の辺は設計の節点の次数に入らない）。
    let unread: Vec<String> = graph
        .unread
        .iter()
        .filter_map(|s| match s {
            Source::Design => Some("design".to_string()),
            Source::Ledger => Some("ledger".to_string()),
            Source::Runs => None,
        })
        .collect();
    if !unread.is_empty() {
        return Gate::Unknown {
            why: Why::Unread,
            ids: sorted(unread),
            digest: None,
        };
    }
    let missing: Vec<String> = touches
        .iter()
        .filter(|id| graph.node(id).is_none())
        .cloned()
        .collect();
    if !missing.is_empty() {
        return deny(Why::UnknownId, sorted(missing), None);
    }
    let required = needs(graph, &touches);
    let mut disposed: BTreeSet<String> = touches.iter().cloned().collect();
    if let Some(Value::Object(nr)) = object.get(NOT_RELEVANT) {
        disposed.extend(
            nr.iter()
                .filter(|(_, v)| v.as_str().is_some_and(|s| !s.trim().is_empty()))
                .map(|(k, _)| k.clone()),
        );
    }
    let undisposed: Vec<String> = required
        .iter()
        .filter(|id| !disposed.contains(*id))
        .cloned()
        .collect();
    let digest = bundle_digest(graph, &touches);
    if required.len() > NEEDS_CAP {
        return Gate::Unknown {
            why: Why::TooMany,
            ids: undisposed,
            digest: Some(digest),
        };
    }
    if !undisposed.is_empty() {
        return deny(Why::Undisposed, undisposed, Some(digest));
    }
    if object.get(DIGEST).and_then(Value::as_str) != Some(digest.as_str()) {
        return deny(Why::Stale, Vec::new(), Some(digest));
    }
    Gate::Allow
}

/// 下書きを順に判じ、最初に通さなかった下書きの答えを返す（全部が通るか下書きが 0 なら Allow）。
pub fn judge(drafts: &[Draft], graph: &Graph) -> Gate {
    drafts
        .iter()
        .map(|d| judge_one(d, graph))
        .find(|g| *g != Gate::Allow)
        .unwrap_or(Gate::Allow)
}

/// hook の答えの JSON の字（Allow は None・止めるときは PreToolUse の deny と理由）。
pub fn output(gate: &Gate) -> Option<String> {
    let (head, why, ids, digest) = match gate {
        Gate::Allow => return None,
        Gate::Deny { why, ids, digest } => ("問いの起票の門は止める（", why, ids, digest),
        Gate::Unknown { why, ids, digest } => ("問いの起票の門は まだ分からない（", why, ids, digest),
    };
    let mut reason = format!("{head}{}）", why.word());
    if !ids.is_empty() {
        reason.push_str(" id =");
        for id in ids {
            reason.push(' ');
            reason.push_str(id);
        }
    }
    if let Some(d) = digest {
        reason.push_str(" 要約値 = ");
        reason.push_str(d);
    }
    reason.push_str(" 次の一手 = ");
    reason.push_str(why.next_move());
    let mut inner = Map::new();
    inner.insert(
        "hookEventName".to_string(),
        Value::String("PreToolUse".to_string()),
    );
    inner.insert(
        "permissionDecision".to_string(),
        Value::String("deny".to_string()),
    );
    inner.insert(
        "permissionDecisionReason".to_string(),
        Value::String(reason),
    );
    let mut answer = Map::new();
    answer.insert("hookSpecificOutput".to_string(), Value::Object(inner));
    Some(Value::Object(answer).to_string())
}
