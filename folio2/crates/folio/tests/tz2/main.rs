//! folio の歯の群 tz2（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/tz2/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

mod common;
mod face_constitution;
mod face_index;
mod face_index_sheet;
mod face_index_shelf;
mod face_note;
mod face_srs;
mod face_srs_adrs;
mod face_srs_body;
mod face_srs_figure;
mod figure;
