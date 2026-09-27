//! 中核: 導出グラフと不変条件・台帳の指標と次の一手の純粋な関数。
//! 導出グラフと不変条件と近傍は便 c が置く（`graph`）。台帳の指標と未反映の一覧（`ledger`）・
//! pipeline の板（`pipeline`）・次の一手（`next_step`）は便 d が置く。問いの一覧（`question`）は便 e-ask が置く。
//! どの関数も file も子 process も時計も触らない（今の時刻は引数で受ける）。

pub mod graph;
pub mod ledger;
pub mod next_step;
pub mod pipeline;
pub mod question;
