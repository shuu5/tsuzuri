//! xtask の歯の群 teeth1（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/teeth1/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

mod carry;
mod cishard;
mod deny;
mod kfold;
mod klint;
mod klintf;
mod ntmo;
mod retire;
mod tlic;
mod vcij;
mod vdaily;
mod vjprep;
mod vretb;
mod vskip;
mod vsplit;
