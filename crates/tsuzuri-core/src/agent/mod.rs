//! 席が起こす係（判断の記録 ADR-59・要件 FR21）。起こしの門と結びの口の判じと係の札の字（`spec`）は行 ag-spec が置く。
//! 係の記録の使用量の測りと残りの注ぎ（`meter`）は行 ag-meter が置く。
//! 係の終える前の門の判じ（`stop`）は行 ag-stop が置く。

pub mod meter;
pub mod spec;
pub mod stop;
