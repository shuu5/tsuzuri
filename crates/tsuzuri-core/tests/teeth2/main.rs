//! tsuzuri-core の歯の群 teeth2（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/teeth2/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

mod common;
mod gins;
mod gquestion;
mod graph;
mod gview;
mod harestcore;
mod lresume;
mod nbatch;
mod nstall;
mod nsum;
mod pci;
mod pclosed;
mod pkac;
mod plimit;
mod pmisfit;
mod pquest;
mod pqueue;
mod punmap;
mod qblock;
mod qgate;
mod question;
mod runsdoc;
mod sclosed;
mod seat;
mod sgrace;
mod stats;
