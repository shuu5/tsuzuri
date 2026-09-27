//! グラフの面（見本の map.html の graph）。中身は後の便がこの file に足す。この便は測れていないと理由の 1 行だけを返す。

use crate::project::Body;

/// この面がまだ無いときの理由。
pub const REASON: &str = "この面はまだ無い";

/// グラフの面の中身（この便は測れていない）。
pub fn body() -> Body<()> {
    Body::Unmeasured(REASON)
}
