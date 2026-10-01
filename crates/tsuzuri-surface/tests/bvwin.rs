//! 行 g-win-frame の歯: 窓の枠（判断の記録 ADR-27 決定 (4)・見本 board-v2 の窓）の閉じる判定と窓の積みと幅の字と、
//! 幕の id と窓の class と全画面の規則が stylesheet に在り全画面の幅の上限が規則の行 R-35 の値であることと、語の鍵と、
//! 層の DOM（wasm の枝）の配線の字。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::modal::{
    BACK_KEY, BACK_MARK, CLASSES, CLOSE_KEY, ESC, FULL_MAX_PX, Hit, SCRIM, Stack, closes,
    width_style,
};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// × と外の click と取り消しの鍵で閉じ、中の click とほかの鍵と変換の途中の取り消しの鍵では閉じない。
#[test]
fn bvwin_close_on_esc_outside_x() {
    assert_eq!(ESC, "Escape");
    assert!(closes(Hit::X));
    assert!(closes(Hit::Outside));
    assert!(closes(Hit::Key {
        key: "Escape",
        composing: false
    }));
    assert!(!closes(Hit::Inside));
    assert!(!closes(Hit::Key {
        key: "Escape",
        composing: true
    }));
    for key in ["Enter", "Esc", "x", " ", "Tab", "Backspace"] {
        assert!(
            !closes(Hit::Key {
                key,
                composing: false
            }),
            "{key}"
        );
    }
}

/// 窓の積み: 窓の中の口から開くと積み、戻ると前の窓、外から開くと前の窓を全部閉じ、閉じると空。
#[test]
fn bvwin_stack_back_to_previous() {
    let mut s: Stack<u8> = Stack::default();
    assert_eq!((s.top(), s.can_back()), (None, false));
    s.open(1, false);
    assert_eq!((s.top(), s.can_back()), (Some(1), false));
    s.open(2, true);
    assert_eq!((s.top(), s.can_back()), (Some(2), true));
    s.open(3, true);
    assert_eq!((s.top(), s.can_back()), (Some(3), true));
    s.back();
    assert_eq!((s.top(), s.can_back()), (Some(2), true));
    s.back();
    assert_eq!((s.top(), s.can_back()), (Some(1), false));
    s.back();
    assert_eq!((s.top(), s.can_back()), (None, false));
    s.back();
    assert_eq!(s.top(), None);
    s.open(1, false);
    s.open(2, true);
    s.open(4, false);
    assert_eq!((s.top(), s.can_back()), (Some(4), false));
    s.back();
    assert_eq!(s.top(), None);
    s.open(1, false);
    s.open(2, true);
    s.close();
    assert_eq!((s.top(), s.can_back()), (None, false));
}

/// 窓の幅の style は見本の drawModal の min(<幅>px, 94vw)。
#[test]
fn bvwin_width_style() {
    assert_eq!(width_style(760), "width:min(760px, 94vw)");
    assert_eq!(width_style(520), "width:min(520px, 94vw)");
}

/// 幕の id と窓の 5 つの class の規則が stylesheet に在り、600 px 以下の media の規則で窓が全画面になり、
/// その値は規則の行 R-35 のスマホの形の境（窓は全画面）と同じ。
#[test]
fn bvwin_classes_in_stylesheet() {
    assert_eq!(SCRIM, "scrim");
    assert_eq!(CLASSES, ["modal", "mh", "mb", "back", "x"]);
    let css = read("style.css");
    for rule in [
        "#scrim { position: fixed; inset: 0;",
        ".modal { display: flex;",
        ".mh {",
        ".mb {",
        ".mh .back {",
        ".mh .x {",
    ] {
        assert!(css.contains(rule), "stylesheet に {rule} が無い");
    }
    assert_eq!(FULL_MAX_PX, 600);
    let head = format!("@media (max-width: {FULL_MAX_PX}px) {{\n  #scrim {{ padding: 0;");
    let at = css.find(&head).expect("窓の全画面の media の規則");
    let block = &css[at..];
    let block = &block[..block.find("\n}\n").expect("media の規則の終わり")];
    assert!(
        block.contains(
            ".modal { width: 100% !important; height: 100%; max-height: none; border-radius: 0; }"
        ),
        "{block}"
    );
    let rules = read("../../design-intent/rules.yaml");
    let row = rules
        .lines()
        .find(|l| l.contains("id: R-35,"))
        .expect("規則の行 R-35 が在る");
    assert!(
        row.contains(&format!("・{FULL_MAX_PX} px 以下はスマホの形")),
        "{row}"
    );
    assert!(row.contains("窓は全画面"), "{row}");
}

/// × の語の鍵 mclose の語は 窓を閉じる、戻る口の鍵 mback の語は 戻る で、戻る口の頭の印は ‹。
#[test]
fn bvwin_words_in_vocab() {
    let v = vocab();
    assert_eq!(CLOSE_KEY, "mclose");
    assert_eq!(BACK_KEY, "mback");
    assert_eq!(v.term(CLOSE_KEY).map(|t| t.label.as_str()), Some("窓を閉じる"));
    assert_eq!(v.term(BACK_KEY).map(|t| t.label.as_str()), Some("戻る"));
    assert_eq!(BACK_MARK, "‹");
    for key in [CLOSE_KEY, BACK_KEY] {
        let t = v.term(key).expect("語");
        assert!(!t.note.is_empty() && !t.internal.is_empty(), "{key}");
    }
}

/// 層の DOM（wasm の枝）の配線の字: 取り消しの鍵は window の keydown で変換の途中を渡し、幕の click は幕そのものかで
/// 外と中を分け、× は Hit::X、戻る口は積みが 2 枚以上の時だけで、窓は role dialog と aria-modal と幅の style を持つ。
#[test]
fn bvwin_dom_wiring_text() {
    let src = read("src/widgets/modal.rs");
    let dom = &src[src.find("mod dom {").expect("wasm の枝")..];
    for needle in [
        "window_event_listener(ev::keydown",
        "composing: e.is_composing()",
        "e.target() == e.current_target()",
        "hit(ctx, Hit::X)",
        "can_back.then(",
        "<div id=SCRIM",
        "role=\"dialog\" aria-modal=\"true\" style=width_style(f.width)",
    ] {
        assert!(dom.contains(needle), "wasm の枝に {needle} が無い");
    }
    let widgets = read("src/widgets/mod.rs");
    assert!(widgets.contains("\npub mod modal;\n"));
}
