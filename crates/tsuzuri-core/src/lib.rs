//! 中核: 導出グラフと不変条件・台帳の指標と次の一手の純粋な関数。
//! 導出グラフと不変条件と近傍は便 c が置く（`graph`）。台帳の指標と未反映の一覧（`ledger`）・
//! pipeline の板（`pipeline`）・次の一手（`next_step`）は便 d が置く。問いの一覧（`question`）は便 e-ask が置く。
//! 席の card（`seat`）と、席の card から判じる次の一手の 2 種は便 e-seat が置く。
//! account board の電文を組む関数（`account`）は便 e-acct-host と e-acct-proj が置く。
//! 配達済みの印と未配達の裁定と停止の hook の入出力の JSON（`delivery`）は行 f-mark が置く。
//! 問いの起票の門の下書きの読みと判じと答えの JSON（`gate`）は行 f-gate が置く。
//! どの関数も file も子 process も時計も触らない（今の時刻は引数で受ける）。

pub mod account;
pub mod delivery;
pub mod gate;
pub mod graph;
pub mod ledger;
pub mod next_step;
pub mod pipeline;
pub mod question;
pub mod seat;
