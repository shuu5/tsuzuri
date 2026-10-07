//! folio の歯の群 tz1（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/tz1/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

mod adr;
mod anchor;
mod badge;
mod bundle;
mod ceiling;
mod check;
mod common;
mod constitution_range;
mod emit_rulings;
mod entrance;
mod face;
mod face_adr;
