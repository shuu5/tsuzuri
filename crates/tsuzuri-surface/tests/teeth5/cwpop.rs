//! 札と行の吹き出しの相談の口の歯（行 cs-pop・接頭辞 cwpop_・判断の記録 ADR-29 決定 (4)）。
//! 口の先の URL（純な関数）を host で撃ち、吹き出しの層の DOM（wasm の枝）は配線の字を読む。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::consultwin::{POP_KEY, consult_href};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::topbar::{Win, win_of_href};
use tsuzuri_surface::vocab::vocab;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 口の先は相談の窓を開いた home の頁の URL で、bead の id を query の id に置き（%XX で包む）、窓と id に読み戻る。
#[test]
fn cwpop_href_opens_consult_with_id() {
    for mode in Mode::ALL {
        assert_eq!(
            consult_href("t3-hub.78", mode),
            format!("?mode={}&win=consult&id=t3-hub.78", mode.key())
        );
        for id in ["t3-hub.78", "e.2:x", "a b&c"] {
            assert_eq!(
                win_of_href(&consult_href(id, mode)),
                Some((Win::Consult, Some(id.to_string()))),
                "{id}"
            );
        }
    }
}

/// どの段の吹き出しも、末の口の並び（class foot の段）の個別の頁への口の後に相談の口を 1 つだけ持ち、口は hover の card を
/// 付けない（押した時だけ開く）。窓を開く link の押しは、頁の層が id を題の初めの字に渡して窓を開く。
#[test]
fn cwpop_foot_link_once_without_hover() {
    let src = read("src/widgets/pop.rs");
    let dom = &src[src.find("mod dom {").expect("wasm の枝")..];
    let link =
        "<a class=link href=consult_href(&p.id, mode)>{format!(\"{} ›\", label(POP_KEY))}</a>";
    assert_eq!(
        dom.matches("consult_href(").count(),
        1,
        "相談の口が 1 つでない"
    );
    let foot = &dom[dom.find("<div class=foot>").expect("末の段")..];
    let foot = &foot[..foot.find("</div>").expect("末の段の終わり")];
    let page = foot.find("label(PAGE_KEY)").expect("個別の頁への口");
    let at = foot.find(link).expect("末の段に相談の口が無い");
    assert!(page < at, "相談の口が個別の頁への口の前");
    assert!(!foot.contains("use:attach"), "末の段の口に hover の card");
    let board = read("src/board.rs");
    assert!(board.contains("focus.0.set(id);\n    win.open(w, from_win);"));
}

/// 口の語は語の辞書の「相談する」で、注釈が在る。
#[test]
fn cwpop_word_key() {
    assert_eq!(POP_KEY, "pop_consult");
    let term = vocab().term(POP_KEY).expect("語の辞書の pop_consult");
    assert_eq!(term.label, "相談する");
    assert!(!term.note.is_empty(), "注釈が空");
}
