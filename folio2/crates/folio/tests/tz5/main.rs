//! folio の歯の群 tz5（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/tz5/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

mod stamp;
mod unknown_fields;
mod vocab;
