//! 便 h-win の歯: 窓の名・開く URL・窓の一覧の移り方・button の字・保存の口の字が無い・
//! 窓の名を付ける所と開く手順（wasm の枝の字）・Cargo.toml と語の辞書と stylesheet を触らない。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::windows::{
    self, ACCOUNT_WIN, EMPTY, FRONT, NOT_YET_KEY, OPEN_NEW_KEY, Opened, Win, WinState, after_close,
    after_open, button_text, open_url, row_class, state_mark, state_of, state_word, win_name,
};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::{Body, NO_CONTENT};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_text() -> String {
    read("../../tests/fixtures/account/acct-doc.json")
}

fn fixture() -> AccountDoc {
    wire::decode(&fixture_text()).expect("fixture が AccountDoc として読める")
}

fn win(project: &str, state: WinState) -> Win {
    Win {
        project: project.to_string(),
        name: format!("tz-{project}"),
        state,
    }
}

fn projects(wins: &[Win]) -> Vec<(&str, WinState)> {
    wins.iter().map(|w| (w.project.as_str(), w.state)).collect()
}

/// (1) 窓の名は tz-<project の名>・account board の窓の名は tz-account。
#[test]
fn acctwin_names() {
    assert_eq!(win_name("tsuzuri"), "tz-tsuzuri");
    assert_eq!(win_name("proj-a"), "tz-proj-a");
    assert_eq!(ACCOUNT_WIN, "tz-account");
    // project の名 account の窓は account board の窓と同じ名になるが、窓の名の決まりはそのまま。
    assert_eq!(win_name("account"), ACCOUNT_WIN);
}

/// (2) 開く URL の host の後ろの字は電文の board の port に mode を足した字・board が無い project は無し。
#[test]
fn acctwin_open_url_from_board_and_mode() {
    let doc = fixture();
    let got: Vec<(&str, Option<String>, Option<String>)> = doc
        .projects
        .iter()
        .map(|p| {
            (
                p.name.as_str(),
                open_url(p, Mode::Beginner),
                open_url(p, Mode::Expert),
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            (
                "proj-a",
                Some(":40001/?mode=beginner".to_string()),
                Some(":40001/?mode=expert".to_string())
            ),
            (
                "proj-b",
                Some(":40002/?mode=beginner".to_string()),
                Some(":40002/?mode=expert".to_string())
            ),
            ("proj-c", None, None),
        ]
    );
    // URL の字は電文の board の port から取る（project の名から組まない）。
    let mut row = doc.projects[0].clone();
    row.board = Some(1);
    assert_eq!(
        open_url(&row, Mode::Expert).as_deref(),
        Some(":1/?mode=expert")
    );
    row.board = None;
    assert_eq!(open_url(&row, Mode::Expert), None);
    // mode の字は frame の Mode の key。
    for mode in Mode::ALL {
        assert_eq!(
            open_url(&doc.projects[1], mode),
            Some(format!(":40002/?mode={}", mode.key()))
        );
    }
}

/// (3) 一覧に無い窓を開くと末尾に開いているで足し（新しい）・開いている窓を開くと前面へ（位置も並びも変えない）。
#[test]
fn acctwin_after_open_adds_and_fronts() {
    let (l1, how) = after_open(&[], "proj-b");
    assert_eq!(how, Opened::New);
    assert_eq!(l1, vec![win("proj-b", WinState::Open)]);
    assert_eq!(l1[0].name, "tz-proj-b");

    let (l2, how) = after_open(&l1, "proj-a");
    assert_eq!(how, Opened::New);
    assert_eq!(
        projects(&l2),
        vec![("proj-b", WinState::Open), ("proj-a", WinState::Open)]
    );

    let (l3, how) = after_open(&l2, "proj-b");
    assert_eq!(how, Opened::Front);
    assert_eq!(l3, l2);

    let (l4, how) = after_open(&l3, "proj-c");
    assert_eq!(how, Opened::New);
    assert_eq!(
        projects(&l4),
        vec![
            ("proj-b", WinState::Open),
            ("proj-a", WinState::Open),
            ("proj-c", WinState::Open)
        ]
    );
    // 移り方は純粋（元の一覧を変えない）。
    assert_eq!(l1, vec![win("proj-b", WinState::Open)]);
}

/// (3) handle の closed が真になった窓は閉じた・閉じた窓を開くと新しい（位置を変えずに開いているにする）。
#[test]
fn acctwin_after_close_then_reopen_keeps_place() {
    let list = vec![
        win("proj-a", WinState::Open),
        win("proj-b", WinState::Open),
        win("proj-c", WinState::Open),
    ];
    let closed = after_close(&list, "proj-b");
    assert_eq!(
        projects(&closed),
        vec![
            ("proj-a", WinState::Open),
            ("proj-b", WinState::Closed),
            ("proj-c", WinState::Open)
        ]
    );
    assert_eq!(state_of(&closed, "proj-b"), Some(WinState::Closed));
    // 2 度閉じても同じ・一覧に無い project を閉じても一覧は変わらない。
    assert_eq!(after_close(&closed, "proj-b"), closed);
    assert_eq!(after_close(&closed, "proj-z"), closed);
    assert_eq!(after_close(&[], "proj-a"), Vec::<Win>::new());

    let (reopened, how) = after_open(&closed, "proj-b");
    assert_eq!(how, Opened::New);
    assert_eq!(reopened, list);

    // 全部閉じてから開き直しても開いた順（初めて足した順）のまま。
    let mut all = list.clone();
    for p in ["proj-c", "proj-a", "proj-b"] {
        all = after_close(&all, p);
    }
    assert!(all.iter().all(|w| w.state == WinState::Closed));
    for p in ["proj-c", "proj-a"] {
        let (next, how) = after_open(&all, p);
        assert_eq!(how, Opened::New, "{p}");
        all = next;
    }
    assert_eq!(
        projects(&all),
        vec![
            ("proj-a", WinState::Open),
            ("proj-b", WinState::Closed),
            ("proj-c", WinState::Open)
        ]
    );
    assert_eq!(state_of(&all, "proj-z"), None);
}

/// (4) button の字は開いている窓は前面へ・ほかは語の鍵 open_new。行の語と印と class。
#[test]
fn acctwin_button_text_and_row_words() {
    assert_eq!(FRONT, "前面へ");
    assert_eq!(button_text(Some(WinState::Open)), "前面へ");
    let open_new = vocab().term(OPEN_NEW_KEY).expect("鍵 open_new が語の辞書に在る");
    assert!(!open_new.label.is_empty());
    assert_eq!(button_text(Some(WinState::Closed)), open_new.label);
    assert_eq!(button_text(None), open_new.label);
    assert_ne!(open_new.label, FRONT);
    assert_eq!(OPEN_NEW_KEY, "open_new");
    assert_eq!(NOT_YET_KEY, "not_yet");
    assert!(vocab().term(NOT_YET_KEY).is_some());
    assert!(vocab().term(windows::BLOCK.heading).is_some());

    // 一覧の button の字は一覧の状態から引く。
    let list = after_close(&after_open(&[], "proj-a").0, "proj-a");
    let (list, _) = after_open(&list, "proj-b");
    assert_eq!(
        button_text(state_of(&list, "proj-a")),
        vocab().label("open_new")
    );
    assert_eq!(button_text(state_of(&list, "proj-b")), "前面へ");
    assert_eq!(
        button_text(state_of(&list, "proj-c")),
        vocab().label("open_new")
    );

    assert_eq!(state_word(WinState::Open), "開いている");
    assert_eq!(state_word(WinState::Closed), "閉じた");
    assert_eq!(state_mark(WinState::Open), "run");
    assert_eq!(state_mark(WinState::Closed), "wait");
    assert_eq!(row_class(WinState::Open), "");
    assert_eq!(row_class(WinState::Closed), "w-closed");
    assert!(!EMPTY.is_empty());
    let css = read("style.css");
    for class in ["wins", "items", "w-closed", "btn", "ttl"] {
        assert!(css.contains(&format!(".{class}")), "stylesheet に {class}");
    }
}

/// 着地済みの枠は変えない: block の枠と body（読めたら中身はまだ無い）。
#[test]
fn acctwin_frame_unchanged() {
    assert_eq!(
        (
            windows::BLOCK.id,
            windows::BLOCK.heading,
            windows::BLOCK.class
        ),
        ("winsp", "open_windows", "panel")
    );
    assert_eq!(
        windows::body(&Fetched::Body(fixture_text())),
        Body::Unmeasured(NO_CONTENT)
    );
}

/// (5) src の account の下の file に保存の口の字が無い。窓の名を付ける所と開く手順（wasm の枝）。
#[test]
fn acctwin_no_storage_and_wasm_wiring() {
    let dir = crate_dir().join("src/account");
    let mut n = 0;
    for entry in std::fs::read_dir(&dir).expect("src/account").flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        n += 1;
        let text = std::fs::read_to_string(&path).expect("src の file");
        for word in ["localStorage", "sessionStorage", "cookie"] {
            assert!(
                !text.contains(word),
                "{} に {word} の字が在る",
                path.display()
            );
        }
    }
    assert!(n >= 7);

    let board = read("src/account/board.rs");
    let mount = board
        .split("pub fn mount()")
        .nth(1)
        .and_then(|s| s.split("\n}").next())
        .expect("board.rs の mount");
    assert!(mount.contains("set_name(windows::ACCOUNT_WIN)"));

    let src = read("src/account/windows.rs");
    for part in [
        "pub fn open(project: &str, url: &str)",
        "open_named(\"\", &win_name(project))",
        "\"about:blank\"",
        "set_href(url)",
        "focus()",
        "closed()",
        "after_open(",
        "after_close(",
        "dom::list_view()",
        "class=\"items wins\"",
    ] {
        assert!(src.contains(part), "windows.rs に {part} が無い");
    }
}

/// (6) Cargo.toml は外の依存を足さない（Window と Location の feature は着地済み）。
#[test]
fn acctwin_manifest_features_already_there() {
    let manifest = read("Cargo.toml");
    for feature in ["\"Window\"", "\"Location\""] {
        assert!(manifest.contains(feature), "Cargo.toml に {feature}");
    }
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|s| s.split("[target.").next())
        .expect("dependencies の節");
    let names: Vec<&str> = deps
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split('=').next())
        .map(str::trim)
        .collect();
    assert_eq!(names, vec!["tsuzuri-contract"]);
}
