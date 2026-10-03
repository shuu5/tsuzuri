//! 行 g-win-url の歯（接頭辞 bvwurl_・判断の記録 ADR-27 決定 (4)）: 窓を開く URL（query の win と問いの id）で頁を開いて
//! 窓を開いた後、頁の URL を history の置き替えで win と問いの id の無い URL にする（読み直しても同じ窓は開かない）。
//! 窓の開け閉めは history を足さない（戻るは前の頁へ）。DOM は host で撃てないので、src の字で見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::frame::{Mode, PageId};
use tsuzuri_surface::topbar::{Win, settled_query, win_href, win_of_href};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// home の頁では win と問いの id を外し、ほかの値と順は残し、残りが無ければ空の字。外した後の URL は窓を開かない。
/// 節点の頁の id は頁の中心なので残す。
#[test]
fn bvwurl_drops_win_and_id() {
    let opened = format!("{}&id=t3-hub.5", win_href(Win::Ask, Mode::Expert));
    assert_eq!(opened, "?mode=expert&win=ask&id=t3-hub.5");
    let settled = settled_query(&opened, PageId::Home);
    assert_eq!(settled, "?mode=expert");
    assert_eq!(win_of_href(&settled), None);
    assert_eq!(settled_query("?win=gaps", PageId::Home), "");
    assert_eq!(
        settled_query("?id=q.1&mode=beginner&win=seat&x=1", PageId::Home),
        "?mode=beginner&x=1"
    );
    assert_eq!(settled_query("?mode=expert", PageId::Home), "?mode=expert");
    assert_eq!(
        settled_query("?page=node&id=t3-hub.5&win=ask&mode=beginner", PageId::Node),
        "?page=node&id=t3-hub.5&mode=beginner"
    );
}

/// board.rs の行頭の関数ごとの名と字（名の後から次の行頭の閉じ括弧まで）。
fn top_fns(src: &str) -> Vec<(&str, &str)> {
    src.match_indices("\nfn ")
        .chain(src.match_indices("\npub fn "))
        .map(|(at, head)| {
            let rest = &src[at + head.len()..];
            let name = &rest[..rest.find(['(', '<']).expect("関数の名")];
            (name, &rest[..rest.find("\n}\n").expect("関数の閉じ")])
        })
        .collect()
}

fn body_of<'a>(fns: &[(&'a str, &'a str)], name: &str) -> &'a str {
    fns.iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("board.rs に関数 {name} が無い"))
        .1
}

/// App は URL の win で窓を開いた直後に settle_url を呼び（呼ぶのはそこだけ）、settle_url は今の path と外した query と
/// 井桁の後をこの順でつないだ URL を history の置き替えに渡す。頁の中の link で窓を開く口（open_link）は URL と history に
/// 触れず、それらに触れる board.rs の関数も呼ばない。board.rs は history を足さない。字はそれぞれの関数の中で照らす。
#[test]
fn bvwurl_board_settles_after_open_text() {
    let src = read("src/board.rs");
    let fns = top_fns(&src);
    let app = body_of(&fns, "App");
    let open = app.find("win.open(w, false);").expect("URL の win で開く");
    let settle = app
        .find("settle_url(page, &query);")
        .expect("URL の置き替えの呼び");
    let after = open + "win.open(w, false);".len();
    assert!(
        after <= settle && app[after..settle].trim().is_empty(),
        "置き替えが窓を開いた直後に無い"
    );
    assert_eq!(
        src.matches("settle_url(").count(),
        2,
        "置き替えの呼びは App の 1 所だけ"
    );
    let body = body_of(&fns, "settle_url");
    let parts = [
        "let url = format!(",
        "\"{}{}{}\",",
        "loc.pathname().unwrap_or_default(),",
        "topbar::settled_query(query, page),",
        "loc.hash().unwrap_or_default()",
        "history.replace_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, \"\", Some(&url))",
    ]
    .map(|n| {
        body.find(n)
            .unwrap_or_else(|| panic!("settle_url に {n} が無い"))
    });
    assert!(parts.is_sorted(), "URL のつなぎの並びが違う: {parts:?}");
    let link = body_of(&fns, "open_link");
    let touch: Vec<&str> = fns
        .iter()
        .filter(|(_, b)| b.contains("history") || b.contains("location()"))
        .map(|(n, _)| *n)
        .collect();
    assert!(touch.contains(&"settle_url") && touch.contains(&"keep_mode_in_url"));
    for word in ["history", "location()"] {
        assert!(!link.contains(word), "open_link が {word} に触れる");
    }
    for name in &touch {
        assert!(
            !link.contains(&format!("{name}(")),
            "open_link が URL か history に触れる関数 {name} を呼ぶ"
        );
    }
    assert!(!src.contains("push_state"), "board.rs が history を足す");
}
