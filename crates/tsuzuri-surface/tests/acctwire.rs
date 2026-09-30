//! project board の「戻る」の歯（接頭辞 acctwire_・行 h-wire の完了の条件 (5)〜(7)）。

use std::path::{Path, PathBuf};

use tsuzuri_surface::account::windows::ACCOUNT_WIN;
use tsuzuri_surface::frame::{
    self, ACCOUNT_URL, BACK, BACK_WRAP, BackStep, HEADER, back_steps,
};
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// (5) header は HEADER の 4 つの前に「戻る」を持ち、HEADER は変わらず、snapshot の字は file と 1 字も違わない。
#[test]
fn acctwire_back_part_before_header() {
    assert_eq!(
        (BACK.part, BACK.key, BACK.class, BACK.items),
        ("back", "acct_back", "upto", &[] as &[&str])
    );
    assert_eq!(BACK_WRAP, "backwrap");
    let parts: Vec<&str> = HEADER.iter().map(|h| h.part).collect();
    assert_eq!(parts, ["brand", "nav", "updated", "mode"], "HEADER は変わらない");
    assert_eq!(frame::nav_keys(), ["home", "questions", "map", "gaps"]);
    let want = read("tests/snapshots/header.json");
    let got = frame::header_snapshot();
    assert!(got == want, "header の値が snapshot と違う。今の値:\n{got}");
    let lines: Vec<&str> = got.lines().collect();
    assert_eq!(
        lines[2],
        "    {\"part\": \"back\", \"key\": \"acct_back\", \"class\": \"upto\", \"items\": []},"
    );
    assert!(lines[3].contains("\"part\": \"brand\""), "{}", lines[3]);
    // 語の鍵と class は語の辞書と stylesheet に在る。
    let term = vocab().term(BACK.key).expect("acct_back が語の辞書に在る");
    assert!(!term.label.is_empty());
    let css = read("style.css");
    for class in [".upto", ".backwrap"] {
        assert!(css.contains(class), "stylesheet に {class} が無い");
    }
}

/// (6) tz-account の窓が在れば前面へ、無ければ ?board=account を新しい窓で開き、どちらの後も自分の窓を閉じる。
#[test]
fn acctwire_back_steps() {
    assert_eq!(back_steps(true), [BackStep::Front, BackStep::CloseSelf]);
    assert_eq!(
        back_steps(false),
        [BackStep::OpenNew("?board=account"), BackStep::CloseSelf]
    );
    assert_eq!(ACCOUNT_URL, "?board=account");
    assert!(
        tsuzuri_surface::account::selects(ACCOUNT_URL),
        "account board の入口が選ぶ"
    );
    assert_eq!(ACCOUNT_WIN, "tz-account");
    // 描画は frame の段の列に従い、window の open は空の URL と tz-account を渡す（wasm の target だけ）。
    let board = read("src/board.rs");
    for word in [
        "frame::back_steps(",
        "open_named(\"\", ACCOUNT_WIN)",
        "BackStep::CloseSelf",
        "me.close()",
        "class=BACK_WRAP",
        "class=BACK.class",
    ] {
        assert!(board.contains(word), "board.rs に {word} が無い");
    }
    let lib = read("src/lib.rs");
    assert!(lib.contains("#[cfg(target_arch = \"wasm32\")]\npub mod board;"));
    let frame_src = read("src/frame.rs");
    assert!(!frame_src.contains("web_sys"), "frame は window を撃たない");
}

/// (7) 面の crate の依存の数は変わらない（Cargo.toml は触らない）。
#[test]
fn acctwire_no_new_dependencies() {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("Cargo.toml");
    assert!(!text.contains("url ="), "{text}");
    assert!(text.contains("\"Location\""), "web-sys の Location の feature");
    assert!(text.contains("\"Window\""), "web-sys の Window の feature");
}
