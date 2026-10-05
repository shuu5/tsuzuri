//! 相談の窓の中核の純な関数（判断の記録 ADR-29・要件 FR19・設計ノート surface-wave27a）。
//! 口座の信頼の印の字（`trust`）と撃ち直しで続ける会話の id の選び（`resume`）は行 cs-trust（ノート surface-wave29b）が置く。
//! 所見の欄の検め（`finding`）は行 cs-finding、台帳の相談の行の字と置き場（`lines`）は行 cs-lines、
//! 席が自分で開く問う窓の数え（`quota`・規則の行 R-38）は行 cs-quota、
//! 起動の argv と設定と検め（`launch`）は行 cs-argv、窓の守りの hook の判じ（`guard`）は行 cs-answer、
//! 席の hook の相談の拾いの字（`pickup`）は行 cs-hooks（ノート surface-wave27b）、窓の状態の 1 行の字（`status`）は
//! 行 cs-status-text（ノート surface-wave29b）が置く。
//! どの関数も file も子 process も時計も触らない（今の時刻は分の字で受ける）。

pub mod finding;
pub mod guard;
pub mod launch;
pub mod lines;
pub mod pickup;
pub mod quota;
pub mod resume;
pub mod stamp;
pub mod status;
pub mod trust;
