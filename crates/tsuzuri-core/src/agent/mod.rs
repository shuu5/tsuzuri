//! 席が起こす係（判断の記録 ADR-59・要件 FR21）。起こしの門と結びの口の判じと係の札の字は `spec` が持つ。
//! 係の記録の使用量の測りと残りの注ぎは `meter` が持つ。
//! 予算を越えた係の呼びの係の門の判じは `guard` が持つ。
//! 係の終える前の門の判じは `stop` が持つ。

pub mod guard;
pub mod meter;
pub mod spec;
pub mod stop;
