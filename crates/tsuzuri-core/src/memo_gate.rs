//! memo の起票の門（要件 FR4・判断の記録 ADR-72 の決定 (5)）。
//! 入力は Claude Code の PreToolUse の hook の入力の字と導出グラフで、関数は file も子 process も時計も触らない。
//!
//! 席は memo を bd か bdw の create で置く（label `intake:memo`・名指した節点は metadata の touches・
//! 似た memo や便への辺は --deps の relates-to か discovered-from）。
//! 門は memo の起票の下書き（`drafts`）ごとに、touches の各節点の近傍の候補（`memo_needs`・8 種の節点）と、
//! 題と本文の字が似た memo（`similar`）と、本文の名指す code の file と関数（`code_needs`）の全部に
//! 処分（touches か not-relevant か links か metadata の code）が在り、metadata の digest が今の束の要約値と同じときだけ通す（`judge`）。
//! 止める答えは PreToolUse の deny の JSON（`output`）。問いの起票の門（`gate`）と同じ作りで、前提の方針の欄は見ない。

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};
use tsuzuri_contract::graph::{EdgeType, NodeKind, natural_cmp};
use tsuzuri_contract::ledger::MEMO_LABEL;

use crate::gate::{
    CREATE, DIGEST, HUB_DEGREE, NEEDS_CAP, PROGRAMS, deny_json, digest_of, disposed_of,
    is_assignment, label_values, metadata_value, segments, sorted, unread_sources,
};
use crate::graph::build::{metadata_ids, read_ledger};
use crate::graph::code::{CodeGraph, DefKind};
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

/// 似ている比の下限（百分率）。
pub const SIMILAR_MIN: usize = 30;

/// 似た memo の候補の上限。
pub const SIMILAR_CAP: usize = 5;

/// code の候補の上限（越えれば まだ分からない）。
pub const CODE_CAP: usize = 10;

/// 1 つの名を定義する file の数の上限（越える名は広すぎて候補にしない）。
pub const CODE_SPREAD: usize = 3;

/// 関わる code を名指す metadata の鍵（path か `path#名` の id の字か字の配列）。
pub const CODE_KEY: &str = "code";

/// memo の起票の下書き（呼び出しの 1 続きの create のうち label に `intake:memo` を持つもの）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoDraft {
    /// --metadata の値（何度在っても最後の値・無ければ無し）。
    pub metadata: Option<String>,
    /// --deps の項のうち型が relates-to か discovered-from の項の id（書いた順）。
    pub links: Vec<String>,
    /// 題と本文の字（--title の値・位置の題・-d か --description の値を command の順に改行でつないだ字・
    /// 本文の file を読めなければ None）。
    pub text: Option<String>,
    /// 最後の --body-file の値。
    pub body_file: Option<String>,
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

/// 旗の値の組（旗の位置と値・`旗 値` の旗は `space` の名、`旗=値` の旗は `equals` の頭の字・command の順）。
fn flag_values<'a>(words: &'a [String], space: &[&str], equals: &[&str]) -> Vec<(usize, &'a str)> {
    let mut out = Vec::new();
    for (i, w) in words.iter().enumerate() {
        if space.contains(&w.as_str()) {
            out.extend(words.get(i + 1).map(|v| (i, v.as_str())));
        } else if let Some(v) = equals.iter().find_map(|p| w.strip_prefix(p)) {
            out.push((i, v));
        }
    }
    out
}

/// 位置の題（語 create の後で - で始まらない最初の語のうち、前の語が - で始まらないか、- で始まり等号を持つもの）。
fn positional_title(words: &[String]) -> Option<(usize, &str)> {
    let create = words.iter().position(|w| w == CREATE)?;
    words
        .iter()
        .enumerate()
        .skip(create)
        .zip(words.iter().skip(create + 1))
        .find(|((_, prev), word)| {
            !word.starts_with('-') && (!prev.starts_with('-') || prev.contains('='))
        })
        .map(|((i, _), word)| (i + 1, word.as_str()))
}

/// 題と本文の字（旗 --title の値・位置の題・旗 -d と --description の値を command の順に改行でつないだ字）。
fn draft_text(words: &[String]) -> String {
    let mut parts = flag_values(words, &["--title"], &["--title="]);
    parts.extend(flag_values(words, &["-d", "--description"], &["--description="]));
    parts.extend(positional_title(words));
    parts.sort_by_key(|(i, _)| *i);
    let values: Vec<&str> = parts.into_iter().map(|(_, v)| v).collect();
    values.join("\n")
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
                    text: Some(draft_text(rest)),
                    body_file: flag_values(rest, &["--body-file"], &["--body-file="])
                        .last()
                        .map(|(_, v)| v.to_string()),
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
    digest_with(graph, touches, memo_needs(graph, touches))
}

/// touches のうちグラフに在る id と `extra` の和の束の要約値（code の候補はグラフの節点でないので `digest_of` が飛ばす）。
fn digest_with(graph: &Graph, touches: &[String], extra: Vec<String>) -> String {
    let bundle = sorted(
        touches
            .iter()
            .filter(|id| graph.node(id).is_some())
            .cloned()
            .chain(extra),
    );
    digest_of(graph, &bundle)
}

/// hook の入力の鍵 cwd の字（無いか字でなければ None）。
pub fn payload_cwd(payload: &str) -> Option<String> {
    let Ok(Value::Object(input)) = serde_json::from_str::<Value>(payload) else {
        return None;
    };
    input.get("cwd").and_then(Value::as_str).map(str::to_string)
}

/// 台帳の一覧の字から、label に `intake:memo` を持つ bead（開閉を問わない）の id と、題と改行と本文をつないだ字の写し
/// （字が空か JSON の配列でなければ空の写し）。
pub fn memo_texts(ledger: &str) -> BTreeMap<String, String> {
    read_ledger(ledger)
        .unwrap_or_default()
        .into_iter()
        .filter(|b| {
            b.labels
                .as_ref()
                .is_some_and(|ls| ls.iter().any(|l| l == MEMO_LABEL))
        })
        .map(|b| {
            let text = format!(
                "{}\n{}",
                b.title.unwrap_or_default(),
                b.description.unwrap_or_default()
            );
            (b.id, text)
        })
        .collect()
}

/// 字の 2 字組の集合（頭の空白の後の 1 字目が # の行を除き、英数字だけを残して ASCII を小文字にし、隣る 2 字の組）。
fn bigrams(text: &str) -> BTreeSet<(char, char)> {
    let chars: Vec<char> = text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .flat_map(str::chars)
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    chars
        .iter()
        .zip(chars.iter().skip(1))
        .map(|(a, b)| (*a, *b))
        .collect()
}

/// 似ている比（2 字組の集合の共通の数の 100 倍を和の数で割った整数・どちらかが空なら 0）。
fn ratio(a: &BTreeSet<(char, char)>, b: &BTreeSet<(char, char)>) -> usize {
    let common = a.intersection(b).count();
    let union = a.len() + b.len() - common;
    (100 * common).checked_div(union).unwrap_or(0)
}

/// 下書きの字に似た memo（比が `SIMILAR_MIN` 以上を比の大きい順・同じ比は natural_cmp の順に頭の `SIMILAR_CAP` 本・
/// natural_cmp の順で返す・閉じた memo も入る）。
pub fn similar(memos: &BTreeMap<String, String>, text: &str) -> Vec<String> {
    let draft = bigrams(text);
    let mut hits: Vec<(usize, &String)> = memos
        .iter()
        .map(|(id, memo)| (ratio(&draft, &bigrams(memo)), id))
        .filter(|(r, _)| *r >= SIMILAR_MIN)
        .collect();
    hits.sort_by(|(ra, a), (rb, b)| rb.cmp(ra).then_with(|| natural_cmp(a, b)));
    sorted(hits.into_iter().take(SIMILAR_CAP).map(|(_, id)| id.clone()))
}

/// 語が path の語か（/ を持ち、最後の / の後に . を持つ）。
fn is_path_word(word: &str) -> bool {
    word.rsplit_once('/').is_some_and(|(_, last)| last.contains('.'))
}

/// 語の末の `:数字` か `:数字-数字` を除く。
fn without_line(word: &str) -> &str {
    let Some((head, tail)) = word.rsplit_once(':') else {
        return word;
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit());
    let range = match tail.split_once('-') {
        Some((a, b)) => digits(a) && digits(b),
        None => digits(tail),
    };
    if range { head } else { word }
}

/// 識別子の語か（下線を持ち、英字か下線で始まり、ASCII の英数字と下線だけの 4 字以上）。
fn is_ident_word(word: &str) -> bool {
    word.len() >= 4
        && word.contains('_')
        && word.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && word.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

/// 字から code の語（path の語と識別子の語）を重ねずに出た順に返す。
pub fn code_words(text: &str) -> Vec<String> {
    let run = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | ':' | '-');
    let mut out: Vec<String> = Vec::new();
    for raw in text.split(|c: char| !run(c)).filter(|w| !w.is_empty()) {
        let trimmed = raw.trim_end_matches(['.', ',', ':']);
        let word = without_line(trimmed);
        let word = if is_path_word(word) {
            word
        } else {
            word.rsplit("::").next().unwrap_or(word)
        };
        if (is_path_word(word) || is_ident_word(word)) && !out.iter().any(|w| w == word) {
            out.push(word.to_string());
        }
    }
    out
}

/// code の候補（path の語が files に在ればその path・識別子の語と名が同じ定義（mod と impl でないもの）を持つ file が
/// 1 以上 `CODE_SPREAD` 以下なら file ごとの `path#名`・重ねずに natural_cmp の順）。
pub fn code_needs(code: &CodeGraph, words: &[String]) -> Vec<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    for word in words {
        if is_path_word(word) {
            if code.files.contains(word) {
                out.insert(word.clone());
            }
            continue;
        }
        let files: BTreeSet<&str> = code
            .defs
            .iter()
            .filter(|d| d.name == *word && !matches!(d.kind, DefKind::Mod | DefKind::Impl))
            .map(|d| d.file.as_str())
            .collect();
        if (1..=CODE_SPREAD).contains(&files.len()) {
            out.extend(files.into_iter().map(|f| format!("{f}#{word}")));
        }
    }
    sorted(out)
}

/// 判じの材料（グラフと、memo の題と本文の写しと、読んだ code の層・読んでいなければ None）。
#[derive(Debug, Clone)]
pub struct Seen<'a> {
    pub graph: &'a Graph,
    pub memos: BTreeMap<String, String>,
    pub code: Option<CodeGraph>,
}

impl<'a> Seen<'a> {
    /// memo の写しも code の層も持たない材料。
    pub fn of(graph: &'a Graph) -> Seen<'a> {
        Seen {
            graph,
            memos: BTreeMap::new(),
            code: None,
        }
    }
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
            MemoWhy::Unread => {
                "名指した読みの元（design は設計の索引・ledger は台帳・body は --body-file の file・code は code の層）が読めるようになってから置き直す。"
            }
            MemoWhy::UnknownId => "touches から索引と台帳に無い節点を除くか直して置き直す。",
            MemoWhy::TooMany => "touches を絞るか、本文の名指す code を絞った memo に分けて置き直す。",
            MemoWhy::Undisposed => {
                "名指した節点を touches か not-relevant か --deps の relates-to か discovered-from に、名指した code を metadata の code か not-relevant に足し、答えの要約値を metadata の digest に写して置き直す。同じ問題の memo が名指しに在れば、起票をやめてその memo の notes に [再発] の行を書く。"
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

/// 読めない元の名（設計か台帳が読めていなければ design か ledger・欄 text が None なら body・
/// 欄 text が code の語を持ち code の層が無ければ code）。
fn unread_of(draft: &MemoDraft, seen: &Seen, words: &[String]) -> Vec<String> {
    let mut unread = unread_sources(seen.graph);
    if draft.text.is_none() {
        unread.push("body".to_string());
    }
    if !words.is_empty() && seen.code.is_none() {
        unread.push("code".to_string());
    }
    unread
}

/// 処分の要る候補の 3 つ組（近傍の `memo_needs`・本文の名指す `code_needs`・2 つと似た memo の和）。
fn candidates(
    draft: &MemoDraft,
    seen: &Seen,
    touches: &[String],
    words: &[String],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let near = memo_needs(seen.graph, touches);
    let named = seen
        .code
        .as_ref()
        .map(|code| code_needs(code, words))
        .unwrap_or_default();
    let like = similar(&seen.memos, draft.text.as_deref().unwrap_or_default());
    let all = sorted(near.iter().chain(&like).chain(&named).cloned());
    (near, named, all)
}

/// 1 つの下書きを判じる。
fn judge_one(draft: &MemoDraft, seen: &Seen) -> MemoGate {
    let graph = seen.graph;
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
    let words = draft.text.as_deref().map(code_words).unwrap_or_default();
    let unread = unread_of(draft, seen, &words);
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
    let (near, named, required) = candidates(draft, seen, &touches, &words);
    let mut disposed = disposed_of(&touches, &object);
    disposed.extend(draft.links.iter().cloned());
    disposed.extend(metadata_ids(&Value::Object(object.clone()), CODE_KEY));
    let undisposed: Vec<String> = required
        .iter()
        .filter(|id| !disposed.contains(*id))
        .cloned()
        .collect();
    let digest = digest_with(graph, &touches, required.clone());
    if near.len() > NEEDS_CAP || named.len() > CODE_CAP {
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
pub fn judge(drafts: &[MemoDraft], seen: &Seen) -> MemoGate {
    drafts
        .iter()
        .map(|d| judge_one(d, seen))
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
