//! 問いの一覧（設計ノート surface-base 便 e-ask）。
//! 入力は台帳の一覧の字（bd の読み取りの口が返す JSON の配列）で、関数は file も子 process も触らない。
//! open の問い（label `intake:question` を持ち、状態が closed でも tombstone でもない bead）を、
//! 作られた時刻の古い順（同じなら id の順）に card にする。字が読めなければ「まだ分からない」。
//!
//! 問いの本文の形: 席は問いを置くとき、本文の行頭に定型の字を書く（1 つの定型は 1 行・同じ定型が 2 行在れば
//! 最初の行を取る・定型の無い欄は None）。A-1 の印は label のどれかが「A-1」で始まること。
//! 名指した節点は bead の metadata の touches の欄（字 1 つか字の配列・metadata は object か、object を JSON にした字）。
//! 止めている task は、台帳の bead のうち種類 blocks の依存の先がその問いの bead（状態で選ばない・便 c-q-blocking）。

use serde_json::Value;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerItem, LedgerRow, QUESTION_LABEL};
use tsuzuri_contract::question::{QuestionCard, QuestionList};

use crate::graph::build::{BdBead, read_ledger};
use crate::ledger::epoch_secs;

/// 非エンジニア向けの概要の行の頭。
pub const PLAIN_PREFIX: &str = "概要 = ";

/// エンジニア向けの概要の行の頭。
pub const ENG_PREFIX: &str = "技術 = ";

/// 理由の行の頭。
pub const REASON_PREFIX: &str = "理由 = ";

/// 推奨の行の頭。
pub const RECOMMEND_PREFIX: &str = "推奨 = ";

/// A-1 の印の label の頭。
pub const A1_PREFIX: &str = "A-1";

/// 問いの答えを待って止まった task を数える台帳の依存の種類（見本の material の辺 blocks）。
pub const BLOCKS_TYPE: &str = "blocks";

/// open の問いの 1 本（card と、裁定の id の数を決める notes）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenQuestion {
    pub card: QuestionCard,
    pub notes: String,
}

/// 問いの一覧（口 GET /api/questions の出力）。
pub fn list(ledger: &str) -> QuestionList {
    QuestionList {
        cards: match open_questions(ledger) {
            Reading::Known(qs) => Reading::Known(qs.into_iter().map(|q| q.card).collect()),
            Reading::Unknown => Reading::Unknown,
        },
    }
}

/// open の問いを作られた時刻の古い順（同じなら id の順）に返す。
/// 字が読めないか、open の問いの id が bead の id の形でなければ Unknown。
pub fn open_questions(ledger: &str) -> Reading<Vec<OpenQuestion>> {
    let Some(beads) = read_ledger(ledger) else {
        return Reading::Unknown;
    };
    // 種類 blocks の依存の（依存の先・依存する bead）の組（台帳の順・状態を問わない）。
    let blocks: Vec<(String, String)> = beads
        .iter()
        .flat_map(|b| {
            b.dependencies
                .iter()
                .flatten()
                .filter(|d| d.dep_type == BLOCKS_TYPE)
                .map(|d| (d.depends_on_id.clone(), b.id.clone()))
        })
        .collect();
    let mut out = Vec::new();
    for bead in beads.into_iter().filter(is_open_question) {
        let blocking = waiting(&blocks, &bead.id);
        let Some(q) = open_question(bead, blocking) else {
            return Reading::Unknown;
        };
        out.push(q);
    }
    out.sort_by(|a, b| (a.card.posted_at, &a.card.id).cmp(&(b.card.posted_at, &b.card.id)));
    Reading::Known(out)
}

fn is_open_question(bead: &BdBead) -> bool {
    let status = bead.status.as_deref().unwrap_or_default();
    let labels = bead.labels.as_deref().unwrap_or_default();
    labels.iter().any(|l| l == QUESTION_LABEL) && status != "closed" && status != "tombstone"
}

/// 問いの答えを待つ bead の id（`blocks` の組のうち依存の先が `id` の組の bead・台帳の順に 1 度ずつ）。
fn waiting(blocks: &[(String, String)], id: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (to, from) in blocks {
        if to == id && !out.contains(from) {
            out.push(from.clone());
        }
    }
    out
}

/// bead を card にする（id が bead の id の形でなければ None）。
/// 作られた時刻が読めなければ更新時刻、それも無ければ 0。
fn open_question(bead: BdBead, blocking: Vec<String>) -> Option<OpenQuestion> {
    let id = BeadId::new(bead.id).ok()?;
    let labels = bead.labels.unwrap_or_default();
    let description = bead.description.unwrap_or_default();
    let posted_at = [&bead.created_at, &bead.updated_at]
        .into_iter()
        .find_map(|t| t.as_deref().and_then(epoch_secs))
        .unwrap_or(0);
    // 要約値は契約の型の関数で付ける（欄は id・題・状態・本文・notes だけを使う）。
    let item = LedgerItem {
        row: LedgerRow {
            id,
            kind: bead.issue_type.unwrap_or_default(),
            title: bead.title.unwrap_or_default(),
            status: bead.status.unwrap_or_default(),
            updated_at: posted_at,
            parent: None,
            labels,
        },
        description,
        notes: bead.notes.unwrap_or_default(),
    };
    let digest = item.digest();
    let field = |prefix: &str| typed(&item.description, prefix);
    let card = QuestionCard {
        plain: field(PLAIN_PREFIX),
        eng: field(ENG_PREFIX),
        reason: field(REASON_PREFIX),
        recommend: field(RECOMMEND_PREFIX),
        a1: item.row.labels.iter().any(|l| l.starts_with(A1_PREFIX)),
        touches: touches(&bead.metadata),
        blocking,
        digest,
        posted_at,
        title: item.row.title,
        id: item.row.id,
    };
    Some(OpenQuestion {
        card,
        notes: item.notes,
    })
}

/// 本文の定型行の値（行頭が `prefix` の最初の行の残り・前後の空白を除く・空なら None）。
pub fn typed(description: &str, prefix: &str) -> Option<String> {
    let rest = description
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .find_map(|l| l.strip_prefix(prefix))?
        .trim();
    (!rest.is_empty()).then(|| rest.to_string())
}

/// metadata の touches の欄の id（欄は字 1 つか字の配列・metadata は object か、object を JSON にした字）。
pub fn touches(metadata: &Value) -> Vec<String> {
    let parsed;
    let object = match metadata {
        Value::String(s) => {
            parsed = serde_json::from_str::<Value>(s).unwrap_or(Value::Null);
            &parsed
        }
        other => other,
    };
    match object.get("touches") {
        Some(Value::String(s)) if !s.is_empty() => vec![s.clone()],
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}
