//! 台帳 open の一覧の行の hover で板の札に印を付ける歯（行 g-list-hover・接頭辞 lhov_・判断の記録 ADR-30 決定 (8)）。
//! 印を置く pointer の種類と輪が先の決まりを純な関数で断言し、DOM（wasm の枝）の配線の字と stylesheet の規則を照らす。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::ledgerlist::{HOVER_POINTER, hover_lit, hover_on};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// file の字の start の字から end の字の前まで（どちらも在ることを断言する）。
fn span(src: &str, start: &str, end: &str) -> String {
    let at = src.find(start).unwrap_or_else(|| panic!("{start} が無い"));
    let rest = &src[at..];
    let to = rest.find(end).unwrap_or_else(|| panic!("{end} が無い"));
    rest[..to].to_string()
}

#[test]
fn lhov_pointer_mouse_only() {
    assert_eq!(HOVER_POINTER, "mouse");
    assert_eq!(hover_on("mouse", "fx-h.1"), Some("fx-h.1".to_string()));
    assert_eq!(
        hover_on("touch", "fx-h.1"),
        None,
        "指で触る pointer は印を置かない"
    );
    assert_eq!(hover_on("pen", "fx-h.1"), None, "ペンはマウスでない");
    assert_eq!(
        hover_on("", "fx-h.1"),
        None,
        "種類の無い pointer は置かない"
    );
}

#[test]
fn lhov_lit_ring_first() {
    assert!(hover_lit(None, Some("fx-h.1"), "fx-h.1"));
    assert!(
        hover_lit(Some("fx-h.2"), Some("fx-h.1"), "fx-h.1"),
        "ほかの bead の吹き出しが開いていても印"
    );
    assert!(
        !hover_lit(Some("fx-h.1"), Some("fx-h.1"), "fx-h.1"),
        "吹き出しの開いている札は輪だけ（輪が先）"
    );
    assert!(
        !hover_lit(None, Some("fx-h.2"), "fx-h.1"),
        "hover の id と違う札"
    );
    assert!(!hover_lit(None, None, "fx-h.1"), "hover の無い間は印なし");
}

#[test]
fn lhov_row_enter_leave() {
    let list = read("src/ledgerlist.rs");
    let dom = &list[list.find("mod dom {").expect("wasm の枝")..];
    assert!(
        dom.contains("hover: RwSignal::new(None),"),
        "SelCtx の hover は None で始まる"
    );
    let row = span(dom, "fn row_view(", "\n    }\n");
    assert!(
        row.contains(
            "on:click=press\n                on:pointerenter=enter on:pointerleave=leave>"
        ),
        "一覧の行の tag に入りと出の口が無い"
    );
    let enter = span(&row, "let enter = {", "let leave = ");
    assert!(
        enter.contains("s.hover.set(hover_on(&e.pointer_type(), &id));"),
        "入りは pointer の種類で hover を置く"
    );
    assert!(!enter.contains("press"), "入りは吹き出しを開かない");
    let leave = span(&row, "let leave = ", "};\n");
    assert!(leave.contains("s.hover.set(None);"), "出は hover を外す");
    for width in ["inner_width", "match_media", "max-width", "outer_width"] {
        assert!(!row.contains(width), "行の hover が窓の幅 {width} で分ける");
    }
    assert!(
        dom.contains("<div class=\"ll-body\" on:pointerleave=out>"),
        "一覧を出た pointer は hover を外す"
    );
}

#[test]
fn lhov_card_marked() {
    let pipe = read("src/project/pipeline.rs");
    let pick = span(&pipe, "fn pick_view(", "\n    }\n");
    assert!(
        pick.contains(
            "<div class=\"kpick\" class:dim=dimmed class:ring=ringed class:hov=hovered>{inner}</div>"
        ),
        "札の包みに hover の class が無い"
    );
    assert!(
        pick.contains(".with(|h| hover_lit(shown.as_deref(), h.as_deref(), &id))"),
        "札の hover の印は hover_lit で決める"
    );
}

#[test]
fn lhov_style_rule() {
    let css = read("style.css");
    let rule = ".kpick.hov > .kcard { box-shadow: 0 0 0 2px color-mix(in srgb, var(--ink) 35%, transparent); }";
    assert_eq!(
        css.matches(".kpick.hov > .kcard").count(),
        1,
        "hover の印の規則は 1 つ"
    );
    let at = css.find(rule).expect("hover の印の規則の字");
    let before = &css[..at];
    assert_eq!(
        before.matches('{').count(),
        before.matches('}').count(),
        "hover の印の規則は @media の外（窓の幅で分けない）"
    );
    assert!(
        css.contains(".kpick.ring > .kcard { box-shadow: 0 0 0 2px var(--ink); }"),
        "選びの輪は濃いまま"
    );
    assert!(
        css.contains(".kpick.dim.hov { opacity: 1; }"),
        "薄い札も印は見せる"
    );
}
