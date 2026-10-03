//! tsuzuri-core の歯の群 teeth3（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/teeth3/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

mod cwarg;
mod cwfnd;
mod cwlin;
mod cwquo;
