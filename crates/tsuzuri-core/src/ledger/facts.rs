//! bead の事実の一覧（起票の時刻・短い題・概要・blocks の相手・判断の記録 ADR-27 決定 (9)・行 c-bead-facts）。
//! 入力は台帳の一覧の字（bd の読み取りの口が返す JSON の配列）だけで、file も子 process も時計も触らない。
//! 短い題は metadata の鍵 short の字か、無ければ題から機械で作る。概要は本文の 1 行を Unicode の字で 120 に切る。

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

/// 概要を取る見出しの行。
pub const OBSERVATION_HEADING: &str = "### 観測";

/// 概要の行から外す頭の字。
pub const SUMMARY_LABEL: &str = "概要 =";

/// 概要の字数の上限（Unicode の字）。
pub const SUMMARY_MAX: usize = 120;

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
        short: set.unwrap_or_else(|| short_of(bead.title.as_deref().unwrap_or_default())),
        short_set,
        summary: summary_of(bead.description.as_deref().unwrap_or_default()),
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

/// 本文から作る概要。見出しの行 `OBSERVATION_HEADING` が在ればその下の最初の空でない行、無ければ空の行と
/// 字 # で始まる行を飛ばした最初の行（どれも前後の空白を除く）から、頭の `SUMMARY_LABEL` とその後の空白を外し、
/// Unicode の字で `SUMMARY_MAX` に切る。行が無ければ空の字。
pub fn summary_of(description: &str) -> String {
    let lines: Vec<&str> = description.lines().map(str::trim).collect();
    let line = match lines.iter().position(|l| *l == OBSERVATION_HEADING) {
        Some(at) => lines
            .get(at + 1..)
            .unwrap_or_default()
            .iter()
            .find(|l| !l.is_empty()),
        None => lines.iter().find(|l| !l.is_empty() && !l.starts_with('#')),
    };
    let line = line.copied().unwrap_or_default();
    line.strip_prefix(SUMMARY_LABEL)
        .map_or(line, str::trim_start)
        .chars()
        .take(SUMMARY_MAX)
        .collect()
}
