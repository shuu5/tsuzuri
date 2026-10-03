//! folio の歯の群 teeth1（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/teeth1/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

#[expect(dead_code, reason = "取り込んだ build.rs の fn main と導出の fn の一部は使わない")]
#[path = "../../build.rs"]
mod build;
mod constitution_enums;
mod deps;
mod face_labels;
mod face_style;
mod fdrop;
mod modules;
mod parts_catalog;
