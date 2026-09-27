//! account board の HOME の block（便 h-frame は枠だけ）: 各 project の次の一手・群の枠・口座 × 窓・移動。
//! 見本は account/index.html の render の home の枝（`#nxall`・`.gtop`・口座 × 窓の panel・`details#moves`）。
//! 中身は後の便（h-home）が描く。この便は口の読みの結果から中身の有無だけを出す。

use crate::frame::Block;
use crate::project::Body;
use crate::view::Fetched;

/// 各 project の次の一手（1 段目の左）。
pub const NXALL: Block = Block {
    id: "nxall",
    heading: "next_all",
    class: "panel",
};

/// 群の枠（2 段目・見本の `.gtop`）。
pub const GROUPS: Block = Block {
    id: "groups",
    heading: "group",
    class: "gtop",
};

/// 口座 × 窓（3 段目）。
pub const ALLOWANCE: Block = Block {
    id: "allowance",
    heading: "allowance",
    class: "panel",
};

/// 移動（4 段目・畳める段）。
pub const MOVES: Block = Block {
    id: "moves",
    heading: "moves",
    class: "panel fold mvp",
};

/// この module が描く block（HOME の段の順）。
pub const BLOCKS: [Block; 4] = [NXALL, GROUPS, ALLOWANCE, MOVES];

/// block の中身（中身の関数がまだ無い: 測れていない）。
pub fn body(fetched: &Fetched) -> Body<()> {
    super::pending(fetched)
}

/// block の DOM（移動は畳める段・ほかは section）。
#[cfg(target_arch = "wasm32")]
pub fn view(block: Block) -> leptos::prelude::AnyView {
    if block.id == MOVES.id {
        super::fold_view(block)
    } else {
        super::section_view(block)
    }
}
