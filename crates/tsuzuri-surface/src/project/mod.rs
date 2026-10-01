//! project board の block（便 g-frame）: 1 つの block に 1 つの module。
//! 各 module は枠の値（`BLOCK`）と中身の純粋な関数を持ち、DOM は wasm の target のときだけ組み立てる。
//! 問いの頁（便 g-ask）は ask（問いの card の列・口 /api/questions）と askpage（これまでの決定・台帳の一覧の口）の 2 つ。
//! 抜けの検査の頁（便 g-gaps）は gaps（不変条件の 12 本の判定・口 /api/graph の定数は map の module の 1 本を使う）。
//! 各 block は自分の読みの口の path を module の定数に持ち、通信（net）の同じ関数で読む（便 g-parts）。
//! 中身の関数は読みの結果（3 値）を受けて中身を返す純粋な関数。next・pipeline・seat・map と ledger の指標の段は、
//! 3 値のどれを受けても測れていないの印と理由の 1 行を返す（本文を読んで中身を返すのは後の block の便）。
//! 畳める段（details）の開き閉じは鍵ごとの記録（`Folds`）から読み、toggle で書き戻す（組み直しの後も保つ・便 g-steady）。
//! 節点の頁（便 g-node）は node（頭と概要）と nodearound（つながり・口の path の頭 /api/around に query を付けて読む）の 2 つ。
//! 2 つの block は nodearound の module が持つ 1 つの読みを分ける。
//! 問いの頁の右の列（便 g-batch）は batch（まとめて承認・口 /api/batch）と policy（全体への指示・口 /api/policy）の 2 つ。
//! 2 つとも問いの一覧を ask の module の口から読む（同じ path の読みは 1 つの signal を分ける）。
//! block に共通の部品は kit に置き、ここで全部を再公開する（行 hs-blocks）。block の module の宣言と列挙 `Module` は
//! 組み立ての script（build.rs）が dir の file から生成する（block を足すのは file を 1 つ置くだけ・判断の記録 ADR-13）。

pub use crate::kit::*;

include!(concat!(env!("OUT_DIR"), "/project_blocks.rs"));
