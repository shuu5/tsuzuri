//! project board の頁（行 hs-pages・判断の記録 ADR-13）: 1 頁 1 file。各 file は頁の定義（`PAGE`）を持つ。
//! 頁の module の宣言と列挙 `PageId` は組み立ての script（build.rs）が dir の file から生成する
//! （頁を足すのは file を 1 つ置くだけ）。

include!(concat!(env!("OUT_DIR"), "/pages.rs"));
