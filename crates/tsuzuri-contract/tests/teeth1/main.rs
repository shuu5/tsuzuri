//! tsuzuri-contract の歯の群 teeth1（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/teeth1/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

mod acctdoc;
mod cform_board;
mod cform_case;
mod cform_graph;
mod cform_ledger;
mod cform_seat;
mod cform_stats;
mod cform_surface;
mod cfsplit;
mod common;
mod ctick;
mod cwty;
mod fxpre;
mod lresume;
mod sumtw;
