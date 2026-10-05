//! 中核: 導出グラフと不変条件・台帳の指標と次の一手の純粋な関数。
//! 導出グラフと不変条件と近傍（`graph`）。台帳の指標と未反映の一覧（`ledger`）・
//! pipeline の板（`pipeline`）・次の一手（`next_step`）。問いの一覧（`question`）。
//! 席の card（`seat`）と、席の card から判じる次の一手の 2 種。
//! account board の電文を組む関数（`account`）。
//! 配達済みの印と未配達の裁定と停止の hook の入出力の JSON（`delivery`）。
//! 問いの起票の門の下書きの読みと判じと答えの JSON（`gate`）。
//! 器の局面の出力の読み（`case`）。
//! 相談の窓の所見の検めと台帳の行と数えと起動の組み（`consult`）。
//! 席が起こす係の起こしの門と結びの口の判じと係の札（`agent`）。
//! どの関数も file も子 process も時計も触らない（今の時刻は引数で受ける）。

pub mod account;
pub mod agent;
pub mod case;
pub mod consult;
pub mod delivery;
pub mod gate;
pub mod graph;
pub mod ledger;
pub mod next_step;
pub mod pipeline;
pub mod question;
pub mod seat;
