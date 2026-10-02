//! 幅の 3 段とスマホの段の tile（行 g-layout・判断の記録 ADR-27 決定 (5)・規則の行 R-35・見本 board-v2 の applyLayout と
//! renderPipe の mpipe と openTile）: 幅 1200 px 以上は横並び（左に一覧・右に pipeline）、601〜1199 px は縦積み（上に pipeline の
//! 5 列・下に一覧・pipeline の面の中だけを横に scroll させて 5 列を保つ）、600 px 以下はスマホの形（頁は縦に scroll し、
//! pipeline は 5 つの段の tile と開いた段の札の並び）。並べ方は stylesheet の media の規則が決め、境の値はここの定数と同じ字。
//! tile は段ごとの札の数と色を出し、既定で開くのは札の在る段のうち一番急ぐ段（止まり → Blocked → Queued → Running / Gated →
//! 着地）。tile を押すとその段を開き、開いている段を押すと畳む。tile の組みと開く段は純粋な関数にして host で試し、DOM は
//! pipeline の block の DOM が描く（札は板と同じ部品）。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::PipelineColumn;

use crate::project::pipeline::{Column, Lane};
use crate::widgets::keyline::QUEUED_WARN_S;

/// 横並びの幅の下限（px・規則の行 R-35）。
pub const WIDE_MIN_PX: u32 = 1200;

/// 縦積みの幅の下限と上限（px・規則の行 R-35）。
pub const STACK_MIN_PX: u32 = 601;
pub const STACK_MAX_PX: u32 = 1199;

/// スマホの形の幅の上限（px・規則の行 R-35・窓の枠の全画面の上限と同じ）。
pub const PHONE_MAX_PX: u32 = crate::widgets::modal::FULL_MAX_PX;

/// 縦積みの板の 1 列の幅の下限（px・狭い時は pipeline の面の中だけを横に scroll させて 5 列を保つ・見本の minmax(112px,1fr)）。
pub const STACK_COL_MIN_PX: u32 = 112;

/// 急ぐ段の順（止まり → Blocked → Queued → Running / Gated → 着地・見本の URGENT）。
pub const URGENT: [PipelineColumn; 5] = [
    PipelineColumn::QuestionedFailedStopped,
    PipelineColumn::Blocked,
    PipelineColumn::Queued,
    PipelineColumn::RunningGated,
    PipelineColumn::Landed,
];

/// 段の tile の 1 つ（列・札の数・開いているか・Queued の札に注意の札が在るか）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub lane: Lane,
    pub n: usize,
    pub open: bool,
    pub warn: bool,
}

/// 既定で開く段（札の在る段のうち `URGENT` の順で最初の段・札が 1 枚も無ければ None）。
pub fn default_open(cols: &[Column]) -> Option<PipelineColumn> {
    URGENT.into_iter().find(|c| {
        cols.iter()
            .any(|col| col.lane.column == *c && !col.cards.is_empty())
    })
}

/// 開く段（tile を押した記録が在ればその値〔None は全部畳む〕・押していなければ既定）。
pub fn open_of(pressed: Option<Option<PipelineColumn>>, cols: &[Column]) -> Option<PipelineColumn> {
    pressed.unwrap_or_else(|| default_open(cols))
}

/// tile を押した後に開く段（開いている段を押すと畳んで None・ほかはその段・見本の act.tile）。
pub fn press(open: Option<PipelineColumn>, tile: PipelineColumn) -> Option<PipelineColumn> {
    (open != Some(tile)).then_some(tile)
}

/// 5 つの tile（板の列の順・数は列の札の全部・Queued の tile は列に在る長さが `QUEUED_WARN_S` を越えた札が在れば注意）。
pub fn tiles(cols: &[Column], open: Option<PipelineColumn>, now: EpochSecs) -> Vec<Tile> {
    cols.iter()
        .map(|c| Tile {
            lane: c.lane,
            n: c.cards.len(),
            open: open == Some(c.lane.column),
            warn: c.lane.column == PipelineColumn::Queued
                && c.cards.iter().any(|k| {
                    k.since
                        .is_some_and(|s| now.saturating_sub(s) > QUEUED_WARN_S)
                }),
        })
        .collect()
}
