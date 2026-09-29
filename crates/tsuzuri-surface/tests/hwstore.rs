//! 行 h-win-store の歯（接頭辞 hwstore_）: account board の窓の一覧を browser の保存に残す（要件 FR1・持ち主の裁定 t3-hub.52.16）。
//! 保存の字と戻し・生きている handle の重ね・開いたはずの窓の移り方と語・閉じの知らせの字と送り手は host の純粋な関数で撃ち、
//! 保存と event と知らせを撃つ所の配線は source の字で見る。

use std::path::PathBuf;

use tsuzuri_surface::account::windows::{
    CLOSED_MSG, FRONT, Opened, WINS_KEY, Win, WinState, after_close, after_open, button_text,
    closed_message, closed_project, row_class, state_mark, state_word, win_name, wins_from,
    wins_text, with_live,
};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} が読めない: {e}", path.display()))
}

fn win(project: &str, state: WinState) -> Win {
    Win {
        project: project.to_string(),
        name: win_name(project),
        state,
    }
}

fn states(wins: &[Win]) -> Vec<(&str, WinState)> {
    wins.iter().map(|w| (w.project.as_str(), w.state)).collect()
}

/// 関数の本文（`fn name(` から、字下げ `indent` の次の `fn `・`pub fn ` の行の前まで・`from` の字の後で探す）。
fn function<'a>(text: &'a str, from: &str, name: &str, indent: &str) -> &'a str {
    let base = text.find(from).unwrap_or_else(|| panic!("{from} が在る"));
    let text = &text[base..];
    let start = text
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("fn {name} が在る"));
    let rest = &text[start..];
    let end = [format!("\n{indent}fn "), format!("\n{indent}pub fn ")]
        .iter()
        .filter_map(|m| rest[1..].find(m.as_str()).map(|i| i + 1))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

/// (1) 保存の鍵・開いたはずの語と記号と class と button の字・stylesheet の規則。
#[test]
fn hwstore_key_and_nohandle_words() {
    assert_eq!(WINS_KEY, "tz-wins");
    assert_eq!(
        state_word(WinState::NoHandle),
        "開いたはず（この窓の reload で handle なし）"
    );
    assert_eq!(state_mark(WinState::NoHandle), "unknown");
    assert_eq!(row_class(WinState::NoHandle), "w-nohandle");
    assert_eq!(button_text(Some(WinState::NoHandle)), FRONT);
    // 着地済みの 2 つの状態の字は変わらない。
    assert_eq!(state_word(WinState::Open), "開いている");
    assert_eq!(row_class(WinState::Closed), "w-closed");
    assert_ne!(button_text(Some(WinState::Closed)), FRONT);
    let css = read("style.css");
    assert!(css.contains(".wins > li.w-nohandle"), "stylesheet に開いたはずの行の規則が無い");
}

/// (2) 保存の字は 1 項 1 行の tab 区切りで、戻すと open の項は開いたはず・closed の項は閉じた。
#[test]
fn hwstore_text_and_restore() {
    let list = vec![
        win("proj-a", WinState::Open),
        win("proj-b", WinState::Closed),
        win("proj-c", WinState::NoHandle),
    ];
    let text = wins_text(&list);
    assert_eq!(text, "proj-a\topen\nproj-b\tclosed\nproj-c\topen\n");
    let back = wins_from(&text);
    assert_eq!(
        back,
        vec![
            win("proj-a", WinState::NoHandle),
            win("proj-b", WinState::Closed),
            win("proj-c", WinState::NoHandle),
        ]
    );
    for w in &back {
        assert_eq!(w.name, win_name(&w.project));
    }
    assert_eq!(wins_text(&[]), "");
    assert_eq!(wins_from(""), Vec::<Win>::new());
    // 戻した一覧を書き直しても同じ字。
    assert_eq!(wins_text(&back), text);
}

/// (3) 壊れた行と同じ project の 2 度目の行は読み捨てる。
#[test]
fn hwstore_restore_skips_broken_lines() {
    let text = [
        "proj-a\topen",
        "no-tab-line",
        "\tclosed",
        "proj-x\tbogus",
        "proj-a\tclosed",
        "proj-y\topen\textra",
        "proj-b\tclosed",
    ]
    .join("\n");
    assert!(!text.ends_with('\n'));
    assert_eq!(
        states(&wins_from(&text)),
        vec![("proj-a", WinState::NoHandle), ("proj-b", WinState::Closed)]
    );
}

/// (4) 生きている handle の名の項は開いている・ほかはそのまま・列だけの名は足さない。
#[test]
fn hwstore_live_handles_overlay() {
    let saved = vec![
        win("proj-a", WinState::NoHandle),
        win("proj-b", WinState::Closed),
        win("proj-c", WinState::NoHandle),
    ];
    let live = vec!["proj-c".to_string(), "proj-z".to_string()];
    assert_eq!(
        states(&with_live(&saved, &live)),
        vec![
            ("proj-a", WinState::NoHandle),
            ("proj-b", WinState::Closed),
            ("proj-c", WinState::Open)
        ]
    );
    let closed_live = vec!["proj-b".to_string()];
    assert_eq!(
        states(&with_live(&saved, &closed_live)),
        vec![
            ("proj-a", WinState::NoHandle),
            ("proj-b", WinState::Open),
            ("proj-c", WinState::NoHandle)
        ]
    );
    assert_eq!(with_live(&saved, &[]), saved);
    assert_eq!(with_live(&[], &live), Vec::<Win>::new());
}

/// (5) 開いたはずの窓を開くと位置を変えずに開いているにして前面へ・閉じると閉じた。
#[test]
fn hwstore_nohandle_open_and_close() {
    let list = vec![
        win("proj-a", WinState::Open),
        win("proj-b", WinState::NoHandle),
        win("proj-c", WinState::Closed),
    ];
    let (next, how) = after_open(&list, "proj-b");
    assert_eq!(how, Opened::Front);
    assert_eq!(
        states(&next),
        vec![
            ("proj-a", WinState::Open),
            ("proj-b", WinState::Open),
            ("proj-c", WinState::Closed)
        ]
    );
    assert_eq!(
        states(&after_close(&list, "proj-b")),
        vec![
            ("proj-a", WinState::Open),
            ("proj-b", WinState::Closed),
            ("proj-c", WinState::Closed)
        ]
    );
}

/// (6) 閉じの知らせの字と、送り手の origin の hostname を見る受け取り。
#[test]
fn hwstore_closed_message_and_sender() {
    assert_eq!(CLOSED_MSG, "tz-closed:");
    assert_eq!(closed_message("tz-proj-a"), "tz-closed:tz-proj-a");
    let msg = closed_message(&win_name("proj-a"));
    let some = Some("proj-a".to_string());
    for (origin, hostname) in [
        ("http://127.0.0.1:40001", "127.0.0.1"),
        ("https://127.0.0.1:40001", "127.0.0.1"),
        ("http://127.0.0.1", "127.0.0.1"),
        ("http://localhost:4173", "localhost"),
        ("https://board.example", "board.example"),
        ("http://[::1]:40001", "[::1]"),
        ("http://[::1]", "[::1]"),
    ] {
        assert_eq!(closed_project(&msg, origin, hostname), some, "{origin} と {hostname}");
    }
    for (origin, hostname) in [
        ("http://127.0.0.2:40001", "127.0.0.1"),
        ("http://127.0.0.1.evil:40001", "127.0.0.1"),
        ("http://127.0.0.10", "127.0.0.1"),
        ("http://localhost.evil", "localhost"),
        ("http://[::2]:40001", "[::1]"),
        ("null", "127.0.0.1"),
        ("", "127.0.0.1"),
        ("", ""),
        ("file://", ""),
    ] {
        assert_eq!(closed_project(&msg, origin, hostname), None, "{origin} と {hostname}");
    }
    let origin = "http://127.0.0.1:40001";
    for data in ["", "proj-a", "tz-proj-a", "tz-closed", "closed:tz-proj-a", "tz-closed:proj-a", "tz-closed:tz-"] {
        assert_eq!(closed_project(data, origin, "127.0.0.1"), None, "{data:?}");
    }
    assert_eq!(
        closed_project("tz-closed:tz-proj-b", origin, "127.0.0.1"),
        Some("proj-b".to_string())
    );
}

const WASM: &str = "#[cfg(target_arch = \"wasm32\")]";

/// (7) 保存と event と知らせを撃つ所の字（windows.rs・store.rs・board.rs）。
#[test]
fn hwstore_wasm_wiring() {
    let src = read("src/account/windows.rs");
    for w in [
        "store::get(WINS_KEY)",
        "store::set(WINS_KEY, &list.with_untracked(|l| wins_text(l)))",
        "store::on_change(WINS_KEY,",
        "window_event_listener(leptos::ev::message, closed_by_message)",
        "closed_project(&data, &e.origin(), &hostname)",
        "with_live(&saved, &live)",
    ] {
        assert!(src.contains(w), "windows.rs に {w} が無い");
    }
    for w in ["local_storage", "get_item(", "set_item("] {
        assert!(!src.contains(w), "windows.rs に {w} が在る");
    }
    let open = function(&src, "mod dom {", "open", "    ");
    assert!(
        open.contains("change(|l| *l = after_open(l, project).0)"),
        "open に change の移しが無い: {open}"
    );
    let sweep = function(&src, "mod dom {", "sweep", "    ");
    assert!(sweep.contains("change(|l|"), "sweep に change の移しが無い: {sweep}");

    let store = read("src/store.rs");
    let lines: Vec<&str> = store.lines().map(str::trim).collect();
    assert!(
        lines.windows(2).any(|w| w[0] == WASM
            && w[1].starts_with("pub fn on_change(key: &'static str, f: impl Fn() + 'static)")),
        "store.rs の on_change が wasm の target のときだけの行の次に無い"
    );
    let change = function(&store, "pub fn on_change(", "on_change", "");
    for w in ["leptos::ev::storage", "e.key().as_deref() == Some(key)", "f()"] {
        assert!(change.contains(w), "on_change に {w} が無い: {change}");
    }

    let board = read("src/board.rs");
    let back = function(&board, "fn back_to_board(", "back_to_board", "");
    let arm = back
        .split("BackStep::Front =>")
        .nth(1)
        .and_then(|s| s.split("BackStep::OpenNew").next())
        .expect("back_to_board に BackStep::Front の腕");
    assert!(
        arm.contains("closed_message(&me.name().unwrap_or_default())"),
        "Front の腕に閉じの知らせの字が無い: {arm}"
    );
    let post = arm
        .find("post_message(&msg.into(), \"*\")")
        .unwrap_or_else(|| panic!("Front の腕に post_message が無い: {arm}"));
    let focus = arm.find("focus()").expect("Front の腕に focus()");
    assert!(post < focus, "post_message が focus() より前に無い");
}
