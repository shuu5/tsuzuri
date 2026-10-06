//! memo の起票の門（要件 FR4・判断の記録 ADR-72 の決定 (5)）。
//! 入力は Claude Code の PreToolUse の hook の入力の字と導出グラフで、関数は file も子 process も時計も触らない。
//!
//! 席は memo を bd か bdw の create で置く（label `intake:memo`・名指した節点は metadata の touches・
//! 似た memo や便への辺は --deps の relates-to か discovered-from）。
//! 門は memo の起票の下書き（`drafts`）ごとに、touches の各節点の近傍の候補（`memo_needs`・8 種の節点）の全部に
//! 処分（touches か not-relevant か links）が在り、metadata の digest が今の束の要約値（`memo_digest`）と同じときだけ通す（`judge`）。
//! 止める答えは PreToolUse の deny の JSON（`output`）。問いの起票の門（`gate`）と同じ作りで、前提の方針の欄は見ない。

use std::collections::BTreeSet;

use serde_json::{Map, Value};
use tsuzuri_contract::graph::{EdgeType, NodeKind};
use tsuzuri_contract::ledger::MEMO_LABEL;

use crate::gate::{
    CREATE, DIGEST, HUB_DEGREE, NEEDS_CAP, PROGRAMS, deny_json, digest_of, disposed_of,
    is_assignment, label_values, metadata_value, segments, sorted, unread_sources,
};
use crate::graph::Graph;
use crate::question::touches;

/// 候補の節点の種類（条・規則行・判断の記録・要件・設計ノートの行・memo・契約・問い）。
pub const CANDIDATE_KINDS: [NodeKind; 8] = [
    NodeKind::Article,
    NodeKind::Rule,
    NodeKind::Adr,
    NodeKind::Req,
    NodeKind::NoteRow,
    NodeKind::Memo,
    NodeKind::Task,
    NodeKind::Question,
];

/// 旗 --deps の項の型のうち処分に数える 2 つ。
const LINK_TYPES: [&str; 2] = ["relates-to", "discovered-from"];

/// memo の起票の下書き（呼び出しの 1 続きの create のうち label に `intake:memo` を持つもの）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoDraft {
    /// --metadata の値（何度在っても最後の値・無ければ無し）。
    pub metadata: Option<String>,
    /// --deps の項のうち型が relates-to か discovered-from の項の id（書いた順）。
    pub links: Vec<String>,
}

/// 旗 --deps の値（`--deps 値` か `--deps=値`・何度でも）。
fn deps_values(words: &[String]) -> Vec<&str> {
    let mut out = Vec::new();
    for (i, w) in words.iter().enumerate() {
        if w == "--deps" {
            out.extend(words.get(i + 1).map(String::as_str));
        } else if let Some(v) = w.strip_prefix("--deps=") {
            out.push(v);
        }
    }
    out
}

/// --deps の値をコンマで割った項のうち、型と id をコロンで割った型が `LINK_TYPES` の項の id。
fn links(words: &[String]) -> Vec<String> {
    deps_values(words)
        .into_iter()
        .flat_map(|v| v.split(','))
        .filter_map(|item| {
            let (kind, id) = item.trim().split_once(':')?;
            let id = id.trim();
            (LINK_TYPES.contains(&kind.trim()) && !id.is_empty()).then(|| id.to_string())
        })
        .collect()
}

/// PreToolUse の hook の入力から memo の起票の下書きを command の順に拾う（Bash の呼び出しでなければ空）。
pub fn drafts(payload: &str) -> Vec<MemoDraft> {
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
            let (head, rest) = seg.get(start..)?.split_first()?;
            let program = head.rsplit('/').next().unwrap_or(head);
            let memo = label_values(rest)
                .iter()
                .any(|v| v.split(',').any(|l| l.trim() == MEMO_LABEL));
            (PROGRAMS.contains(&program) && rest.iter().any(|w| w == CREATE) && memo).then(|| {
                MemoDraft {
                    metadata: metadata_value(rest),
                    links: links(rest),
                }
            })
        })
        .collect()
}

/// 処分の要る候補（2 段・natural_cmp の順）。1 段目は touches の各 id の辺の他方の端のうち、種類が `CANDIDATE_KINDS` で
/// 次数が `HUB_DEGREE` 以下で、台帳の bead なら（memo は開閉を問わず、ほかの種類は）開いたもの。
/// 2 段目は 1 段目の設計ノートの行ごとに、その行へ型 design の辺を持つ次数が `HUB_DEGREE` 以下の開いた契約の bead。
pub fn memo_needs(graph: &Graph, touches: &[String]) -> Vec<String> {
    let index = graph.index();
    let degrees = graph.degrees();
    let kind_of = |id: &str| index.get(id).map(|n| n.kind);
    let small = |id: &str| degrees.get(id).copied().unwrap_or(0) <= HUB_DEGREE;
    let is_open = |id: &str| graph.beads.get(id).is_some_and(|b| b.is_open());
    let first = |id: &str| {
        let Some(kind) = kind_of(id) else {
            return false;
        };
        let live = kind == NodeKind::Memo || graph.beads.get(id).is_none_or(|b| b.is_open());
        CANDIDATE_KINDS.contains(&kind) && small(id) && live
    };
    let both = |from: &str, to: &str| from != to && index.contains_key(from) && index.contains_key(to);
    let mut out: BTreeSet<String> = BTreeSet::new();
    for e in graph.edges.iter().filter(|e| both(&e.from, &e.to)) {
        for (end, other) in [(&e.from, &e.to), (&e.to, &e.from)] {
            if touches.contains(end) && first(other) {
                out.insert(other.clone());
            }
        }
    }
    let rows: Vec<String> = out
        .iter()
        .filter(|id| kind_of(id) == Some(NodeKind::NoteRow))
        .cloned()
        .collect();
    for e in graph.edges.iter().filter(|e| both(&e.from, &e.to)) {
        if e.edge_type == EdgeType::Design
            && rows.contains(&e.to)
            && kind_of(&e.from) == Some(NodeKind::Task)
            && small(&e.from)
            && is_open(&e.from)
        {
            out.insert(e.from.clone());
        }
    }
    sorted(out)
}

/// 束の要約値（touches のうちグラフに在る id と `memo_needs` の和・式は問いの門の `bundle_digest` と同じ）。
pub fn memo_digest(graph: &Graph, touches: &[String]) -> String {
    let bundle = sorted(
        touches
            .iter()
            .filter(|id| graph.node(id).is_some())
            .cloned()
            .chain(memo_needs(graph, touches)),
    );
    digest_of(graph, &bundle)
}

/// 通さない理由（閉じた 8）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoWhy {
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
    /// 処分の要る候補が `NEEDS_CAP` を越える。
    TooMany,
    /// 処分の無い候補が在る。
    Undisposed,
    /// metadata の digest が今の束の要約値と違う。
    Stale,
}

impl MemoWhy {
    pub const ALL: [MemoWhy; 8] = [
        MemoWhy::Args,
        MemoWhy::Metadata,
        MemoWhy::NoTouches,
        MemoWhy::Unread,
        MemoWhy::UnknownId,
        MemoWhy::TooMany,
        MemoWhy::Undisposed,
        MemoWhy::Stale,
    ];

    /// 理由の語。
    pub fn word(self) -> &'static str {
        match self {
            MemoWhy::Args => "args",
            MemoWhy::Metadata => "metadata",
            MemoWhy::NoTouches => "no-touches",
            MemoWhy::Unread => "unread",
            MemoWhy::UnknownId => "unknown-id",
            MemoWhy::TooMany => "too-many",
            MemoWhy::Undisposed => "undisposed",
            MemoWhy::Stale => "stale",
        }
    }

    /// 答えの字の次の一手の 1 文。
    fn next_move(self) -> &'static str {
        match self {
            MemoWhy::Args => "席の設定の tz hook question-gate の引数を直す。",
            MemoWhy::Metadata => "--metadata に JSON の object の字を渡して置き直す。",
            MemoWhy::NoTouches => "metadata の touches に名指す節点を 1 つ以上書いて置き直す。",
            MemoWhy::Unread => "設計の索引と台帳が読めるようになってから置き直す。",
            MemoWhy::UnknownId => "touches から索引と台帳に無い節点を除くか直して置き直す。",
            MemoWhy::TooMany => "touches を絞った memo に分けて置き直す。",
            MemoWhy::Undisposed => {
                "名指した節点を touches か not-relevant か --deps の relates-to か discovered-from に足し、答えの要約値を metadata の digest に写して置き直す。同じ問題の memo が名指しに在れば、起票をやめてその memo の notes に [再発] の行を書く。"
            }
            MemoWhy::Stale => "答えの要約値を metadata の digest に写して置き直す。",
        }
    }
}

/// 門の答え。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoGate {
    Allow,
    Deny {
        why: MemoWhy,
        ids: Vec<String>,
        digest: Option<String>,
    },
    Unknown {
        why: MemoWhy,
        ids: Vec<String>,
        digest: Option<String>,
    },
}

/// 1 つの下書きを判じる。
fn judge_one(draft: &MemoDraft, graph: &Graph) -> MemoGate {
    let deny = |why: MemoWhy, ids: Vec<String>, digest: Option<String>| MemoGate::Deny {
        why,
        ids,
        digest,
    };
    let object = match &draft.metadata {
        None => Map::new(),
        Some(text) => match serde_json::from_str::<Value>(text) {
            Ok(Value::Object(o)) => o,
            _ => return deny(MemoWhy::Metadata, Vec::new(), None),
        },
    };
    let touches = touches(&Value::Object(object.clone()));
    if touches.is_empty() {
        return deny(MemoWhy::NoTouches, Vec::new(), None);
    }
    let unread = unread_sources(graph);
    if !unread.is_empty() {
        return MemoGate::Unknown {
            why: MemoWhy::Unread,
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
        return deny(MemoWhy::UnknownId, sorted(missing), None);
    }
    let required = memo_needs(graph, &touches);
    let mut disposed = disposed_of(&touches, &object);
    disposed.extend(draft.links.iter().cloned());
    let undisposed: Vec<String> = required
        .iter()
        .filter(|id| !disposed.contains(*id))
        .cloned()
        .collect();
    let digest = memo_digest(graph, &touches);
    if required.len() > NEEDS_CAP {
        return MemoGate::Unknown {
            why: MemoWhy::TooMany,
            ids: undisposed,
            digest: Some(digest),
        };
    }
    if !undisposed.is_empty() {
        return deny(MemoWhy::Undisposed, undisposed, Some(digest));
    }
    if object.get(DIGEST).and_then(Value::as_str) != Some(digest.as_str()) {
        return deny(MemoWhy::Stale, Vec::new(), Some(digest));
    }
    MemoGate::Allow
}

/// 下書きを順に判じ、最初に通さなかった下書きの答えを返す（全部が通るか下書きが 0 なら Allow）。
pub fn judge(drafts: &[MemoDraft], graph: &Graph) -> MemoGate {
    drafts
        .iter()
        .map(|d| judge_one(d, graph))
        .find(|g| *g != MemoGate::Allow)
        .unwrap_or(MemoGate::Allow)
}

/// hook の答えの JSON の字（Allow は None・止めるときは PreToolUse の deny と理由）。
pub fn output(gate: &MemoGate) -> Option<String> {
    let (head, why, ids, digest) = match gate {
        MemoGate::Allow => return None,
        MemoGate::Deny { why, ids, digest } => ("memo の起票の門は止める（", why, ids, digest),
        MemoGate::Unknown { why, ids, digest } => {
            ("memo の起票の門は まだ分からない（", why, ids, digest)
        }
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
    Some(deny_json(reason))
}
