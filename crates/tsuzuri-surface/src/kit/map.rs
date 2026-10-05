//! グラフの口（/api/graph）の読み（地図の block から移した）: 本文を契約の型の GraphDoc に読む。
//! 地図の頁は消したが、節点の card（問い・台帳・pipeline・次の一手）と抜けの検査と問いの頁はこの口を読む。
//! kit の下に置くので block でなく（Module の列に入らない）、project の mod.rs の glob の再公開で crate::project::map の
//! path のまま使える（読む側の字は替えない）。

use tsuzuri_contract::graph::GraphDoc;
use tsuzuri_contract::wire;

use super::{NO_CONTENT, NOT_READ};
use crate::view::Fetched;

/// 読みの口（便 g-parts）。
pub const PATH: &str = "/api/graph";

/// 口が読めないときの理由。
pub const REASON: &str =
    "地図の 7 つの出所を組む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 口の本文を電文に読む（まだ読んでいない・読めない・電文の型として読めないは理由）。
/// 電文が読めない本文の理由は、中身の無い block と同じ（便 g-parts の歯が 3 値の理由を pin する）。
pub fn doc(fetched: &Fetched) -> Result<GraphDoc, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(REASON),
        Fetched::Body(text) => wire::decode::<GraphDoc>(text).map_err(|_| NO_CONTENT),
    }
}
