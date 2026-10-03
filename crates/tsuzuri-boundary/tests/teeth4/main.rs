//! tsuzuri-boundary の歯の群 teeth4（1 つの test target・nextest の歯の名は <module>::<fn>・判断の記録 ADR-32）。
//! この file は mod の行だけを持つ（tests/teeth4/ の下の .rs は全部ここで名指す・xtask の歯 kfold_ が照らす）。
#![cfg(test)]

mod server_hb;
mod server_min;
mod server_read;
mod server_reap;
mod server_seat;
mod server_src;
mod server_view;
mod shb;
mod stbp;
mod stcdp;
mod stcli;
mod stdenv;
mod stlaunch;
mod stnfy;
