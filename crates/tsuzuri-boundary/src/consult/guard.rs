//! tz consult guard（窓の守りの hook の口・設計ノート surface-wave27a 行 cs-answer が中身を置く・行 cs-open は骨だけを置く）。

use super::FAIL;
use crate::out::emit_err;

/// 口の中身はまだ無い（rc 1）。
pub fn run(_rest: &[&str]) -> u8 {
    emit_err("tz consult guard: まだ無い（行 cs-answer）");
    FAIL
}
