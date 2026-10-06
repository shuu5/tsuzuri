//! account board の HOME の host の負荷と書きの block の歯（接頭辞 hacct_）。
//! 置き場（HOME の 3 段目・口座 × 窓の後）と枠の値と、帯の host の窓と同じ中身の関数と本文を描く配線の字を測る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::account::{self, Tab, home, hostblock, stage};
use tsuzuri_surface::frame::STACK;
use tsuzuri_surface::hostwin::HOST_KEY;
use tsuzuri_surface::topbar::Win;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::wins::frame_of;

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// block の枠は id hostld・見出しの語は帯の host の窓の題と同じ鍵（host_load・語は「host の負荷」）・class panel で、
/// HOME の 3 段目（STACK）の口座 × 窓のすぐ後・移動の前に 1 度だけ置き、ほかの tab と段には置かない。
#[test]
fn hacct_block_after_allowance() {
    let b = hostblock::BLOCK;
    assert_eq!((b.id, b.heading, b.class), ("hostld", HOST_KEY, "panel"));
    assert_eq!(b.heading, frame_of(Win::Host).1);
    assert_eq!(vocab().label(b.heading), "host の負荷");
    let home_page = account::page(Tab::Home);
    let third = &home_page.rows[2];
    assert_eq!(third.class, STACK);
    assert_eq!(
        third.blocks,
        [home::GROUPS, home::ALLOWANCE, b, home::MOVES, stage::BLOCK]
    );
    let all: Vec<&str> = account::pages()
        .iter()
        .flat_map(|p| p.block_ids())
        .filter(|id| *id == b.id)
        .collect();
    assert_eq!(all, ["hostld"], "host の block は頁の全部で 1 度だけ");
    assert!(
        !home::BLOCKS.contains(&b),
        "home の module の block に入れない"
    );
}

/// block の DOM（wasm の枝・host で組めないので字で測る）は見出しの section に帯の host の窓と同じ本文
/// （hostwin の body）を置くだけで、account board の口を読まない。account board の block の選びは id で hostblock を描く。
#[test]
fn hacct_draws_the_window_body() {
    let src = read("src/account/hostblock.rs");
    let at = src.find("pub fn view()").expect("block の DOM");
    let view = &src[at..];
    assert_eq!(
        view.matches("crate::project::section(BLOCK, ().into_any(), crate::hostwin::body())")
            .count(),
        1
    );
    for word in ["crate::net::read", "account::PATH", "AccountDoc", "view!"] {
        assert!(!src.contains(word), "hostblock.rs に {word}");
    }
    assert!(src.contains("use crate::hostwin::HOST_KEY;"));
    let board = read("src/account/board.rs");
    let pick = &board[board
        .find("fn block_view(block: Block)")
        .expect("block の選び")..];
    let pick = &pick[..pick.find("\n}\n").expect("選びの閉じ")];
    assert_eq!(
        pick.matches("id if id == hostblock::BLOCK.id => hostblock::view(),")
            .count(),
        1
    );
    assert!(read("src/account/mod.rs").contains("\npub mod hostblock;\n"));
}
