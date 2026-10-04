//! bead の事実の一覧（起票の時刻・短い題・blocks の相手・判断の記録 ADR-27 決定 (9)・行 c-bead-facts）。
//! 入力は台帳の一覧の字（bd の読み取りの口が返す JSON の配列）だけで、file も子 process も時計も触らない。
//! 短い題は metadata の鍵 short の字か、無ければ題から機械で作り、どちらも `SHORT_MAX` の字数を越えれば頭の字と `ELLIPSIS` で切る
//! （規則の行 R-19・判断の記録 ADR-30 決定 (5)）。本文の概要は組まない（吹き出しは 1 本の引きの口から読む・行 c-fact-trim）。

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadFact, BeadFacts, BeadId};

use super::epoch_secs;
use crate::graph::build::{BdBead, metadata_ids, read_ledger};

/// 短い題を置く metadata の鍵。
pub const SHORT_KEY: &str = "short";

/// 契約の便の題の頭（後に契約の行 id を置く）。
pub const ROW_TITLE_PREFIX: &str = "便 ";

/// 題の区切りの字（前の字が短い題）。
pub const TITLE_DASH: char = '—';

/// 短い題の字数の上限（Unicode のスカラー値・規則の行 R-19）。
pub const SHORT_MAX: usize = 20;

/// 上限を越えて切った短い題の末の字。
pub const ELLIPSIS: char = '…';

/// 台帳の字から bead の事実の一覧を組む（字が空か JSON の配列として読めなければ行は `Unknown`）。
/// bd が消した bead（tombstone）と、id が bead の id の形でない bead は出さない。行は台帳の順。
pub fn facts(ledger: &str) -> BeadFacts {
    BeadFacts {
        rows: read_ledger(ledger).map_or(Reading::Unknown, |beads| {
            Reading::Known(beads.into_iter().filter_map(fact).collect())
        }),
    }
}

/// bead の 1 本の事実（tombstone か id の形が合わなければ None・blocks の先の id の形が合わなければその先だけ外す）。
fn fact(bead: BdBead) -> Option<BeadFact> {
    if bead.status.as_deref() == Some("tombstone") {
        return None;
    }
    let id = BeadId::new(bead.id).ok()?;
    let set = metadata_ids(&bead.metadata, SHORT_KEY).into_iter().next();
    let short_set = set.is_some();
    let blocks = bead
        .dependencies
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d.dep_type == "blocks")
        .filter_map(|d| BeadId::new(d.depends_on_id).ok())
        .collect();
    Some(BeadFact {
        id,
        created_at: bead.created_at.as_deref().and_then(epoch_secs),
        short: capped(&set.unwrap_or_else(|| short_of(bead.title.as_deref().unwrap_or_default()))),
        short_set,
        blocks,
    })
}

/// 題から機械で作る短い題。題の前後の空白を除き、`ROW_TITLE_PREFIX` で始まれば次の空白までの字（契約の行 id）、
/// ほかは `TITLE_DASH` の前の字の前後の空白を除いた字（区切りが無ければ題の全体）。作った字が空なら題の全体。
pub fn short_of(title: &str) -> String {
    let title = title.trim();
    let made = match title.strip_prefix(ROW_TITLE_PREFIX) {
        Some(rest) => rest.split_whitespace().next().unwrap_or_default(),
        None => title.split(TITLE_DASH).next().unwrap_or_default().trim(),
    };
    if made.is_empty() { title } else { made }.to_string()
}

/// 短い題を上限の字数に切る（`SHORT_MAX` 字以下ならそのまま・越えれば頭の `SHORT_MAX` − 1 字と `ELLIPSIS` の `SHORT_MAX` 字）。
pub fn capped(short: &str) -> String {
    if short.chars().count() <= SHORT_MAX {
        return short.to_string();
    }
    short
        .chars()
        .take(SHORT_MAX - 1)
        .chain([ELLIPSIS])
        .collect()
}
