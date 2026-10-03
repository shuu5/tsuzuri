//! folio の歯の群 tz3（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/tz3/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

mod findings;
mod floor_cases;
mod floor_faces;
mod freeze;
mod freeze_root;
mod gate;
mod gitcheck;
mod graph;
mod graph_notes;
mod graph_summary;
mod hello;
mod help;
mod ids;
mod init;
mod intake;
mod link;
mod mechanism_live;
