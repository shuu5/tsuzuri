//! tz consult launch（窓を起こす口（話す窓は tmux の窓・問う窓は背景の子）・設計ノート surface-wave27a 行 cs-launch が中身を置く・行 cs-open は骨だけを置く）。

use super::FAIL;
use crate::out::emit_err;

/// 口の中身はまだ無い（rc 1）。
pub fn run(_rest: &[&str]) -> u8 {
    emit_err("tz consult launch: まだ無い（行 cs-launch）");
    FAIL
}
