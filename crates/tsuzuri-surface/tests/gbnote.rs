//! 行 g-back-note の歯（接頭辞 gbnote_）: 戻るで窓を探した結果と閉じられない窓の注記の字（frame の back_how と back_note）・
//! board の戻るの字の並び・開いた窓の一覧の閉じの印（着地済みの account の windows）・この file の歯の名。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::account::windows::{
    Win, WinState, after_close, row_class, state_mark, state_word, win_name,
};
use tsuzuri_surface::frame::{
    self, ACCOUNT_URL, BACK_NOTE, BACK_NOTE_KEY, BACK_NOTE_MS, BLANK, BackHow, BackStep,
    back_steps,
};
use tsuzuri_surface::vocab::vocab;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 関数の本文（`fn name(` から、字下げ `indent` の次の `fn `・`pub fn ` の行の前まで）。
fn function<'a>(text: &'a str, name: &str, indent: &str) -> &'a str {
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

/// 型が Copy と PartialEq と Eq と Debug を持つ（組めることで見る）。
fn traits<T: Copy + PartialEq + Eq + std::fmt::Debug>(_: T) {}

/// (1) frame の BLANK・BackHow・back_how・注記の値と字、back_steps との繋ぎ、語の鍵と stylesheet の規則。
#[test]
fn gbnote_how_and_note() {
    assert_eq!(BLANK, "about:blank");
    traits(BackHow::Front);
    let how_fn: fn(bool, Option<&str>) -> BackHow = frame::back_how;
    let note_fn: fn(BackHow) -> String = frame::back_note;
    assert_eq!(how_fn(false, None), BackHow::Blocked);
    assert_eq!(how_fn(false, Some(BLANK)), BackHow::Blocked);
    assert_eq!(how_fn(true, Some(BLANK)), BackHow::Opened);
    assert_eq!(
        how_fn(true, Some("http://127.0.0.1:4173/?board=account")),
        BackHow::Front
    );
    assert_eq!(how_fn(true, None), BackHow::Front, "href が読めない窓は前面へ");

    let steps = |how: BackHow| back_steps(how == BackHow::Front);
    assert_eq!(steps(BackHow::Front), [BackStep::Front, BackStep::CloseSelf]);
    for how in [BackHow::Opened, BackHow::Blocked] {
        assert_eq!(
            steps(how),
            [BackStep::OpenNew(ACCOUNT_URL), BackStep::CloseSelf],
            "{how:?}"
        );
    }

    let ms: u64 = BACK_NOTE_MS;
    assert_eq!(ms, 300);
    let (class, key): (&str, &str) = (BACK_NOTE, BACK_NOTE_KEY);
    assert_eq!((class, key), ("winnote", "win_close_hand"));
    let tail = "・この窓は script が開いた窓でないので閉じられない";
    for (how, head) in [
        (BackHow::Front, "account board の窓は前面に出した"),
        (BackHow::Opened, "account board を新しい窓で開いた"),
        (
            BackHow::Blocked,
            "account board の窓を開けなかった（popup の許可が要る）",
        ),
    ] {
        assert_eq!(note_fn(how), format!("{head}{tail}"), "{how:?}");
    }
    word_and_css();
}

/// 注記の語の鍵が語の辞書に在り、stylesheet に注記の規則が在ることを見る。
fn word_and_css() {
    let term = vocab().term(BACK_NOTE_KEY).expect("win_close_hand が語の辞書に在る");
    assert_eq!(term.label, "この窓は手で閉じてください");
    let css = read("style.css");
    for rule in [".winnote", ".small", ".muted"] {
        assert!(css.contains(rule), "stylesheet に {rule} が無い");
    }
}

/// (2) board の back_to_board は窓を探した結果で段を選び、close の後に閉じたかを見て注記を置く・
/// top は header の直後に注記を出し、back_note は注記の DOM を描く。
#[test]
fn gbnote_board_text() {
    let board = read("src/board.rs");
    let back = function(&board, "back_to_board", "");
    for w in [
        "fn back_to_board(note: RwSignal<Option<BackHow>>)",
        ".href().ok()",
        "frame::back_how(win.is_some(), href.as_deref())",
        "frame::back_steps(how == BackHow::Front)",
        "window().closed()",
        "frame::BACK_NOTE_MS",
        "note.set(Some(how))",
    ] {
        assert!(back.contains(w), "back_to_board に {w} が無い: {back}");
    }
    assert!(!back.contains("is_ok_and("), "{back}");
    let close = back.find("me.close()").expect("back_to_board に me.close() が在る");
    let timer = back
        .find("set_timeout(")
        .expect("back_to_board に set_timeout( が在る");
    assert!(close < timer, "me.close() が set_timeout( の前に無い");

    let top = function(&board, "top", "");
    for w in [
        "back_to_board(note)",
        "</header>{move || note.get().map(back_note)}",
    ] {
        assert!(top.contains(w), "top に {w} が無い: {top}");
    }

    let note = function(&board, "back_note", "");
    for w in [
        "class=frame::BACK_NOTE",
        "<b>{label(frame::BACK_NOTE_KEY)}</b>",
        "frame::back_note(how)",
        "role=\"status\"",
        "class=\"small muted\"",
    ] {
        assert!(note.contains(w), "back_note に {w} が無い: {note}");
    }

    for w in ["tz-wins", "local_storage", "store::set("] {
        assert!(!board.contains(w), "board.rs に {w} が在る");
    }
    assert!(!read("src/frame.rs").contains("web_sys"), "frame は window を撃たない");
}

/// (3) 開いた窓の一覧の閉じの印は着地済みの account board の窓の一覧が出す（handle の closed を 1 秒ごとに見る）。
#[test]
fn gbnote_list_closed_mark() {
    let open = Win {
        project: "p".to_string(),
        name: win_name("p"),
        state: WinState::Open,
    };
    assert_eq!(open.name, "tz-p");
    let next = after_close(&[open], "p");
    let states: Vec<WinState> = next.iter().map(|w| w.state).collect();
    assert_eq!(states, [WinState::Closed]);
    assert_eq!(state_word(WinState::Closed), "閉じた");
    assert_eq!(row_class(WinState::Closed), "w-closed");
    assert_eq!(state_mark(WinState::Closed), "wait");
    let css = read("style.css");
    assert!(css.contains(".wins > li.w-closed .ttl"), "stylesheet に閉じた行の規則が無い");

    let text = read("src/account/windows.rs");
    assert!(text.contains("const SWEEP"), "windows.rs に const SWEEP が無い");
    assert!(
        text.contains(": Duration = Duration::from_secs(1);"),
        "SWEEP が 1 秒でない"
    );
    let sweep = function(&text, "sweep", "    ");
    for w in [".closed()", "after_close("] {
        assert!(sweep.contains(w), "sweep に {w} が無い: {sweep}");
    }
    let open_fn = function(&text, "open", "    ");
    assert!(
        open_fn.contains("set_interval(sweep, SWEEP)"),
        "open に set_interval(sweep, SWEEP) が無い: {open_fn}"
    );
}

/// (5) この file の歯の名はちょうど 4 で、どれも gbnote_ で始まり、残りの字は filter の語を含まない。
#[test]
fn gbnote_own_names_clean() {
    let text = read("tests/gbnote.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    let words = [words_front(), words_middle(), words_back()].concat();
    names_avoid(names, &words);
}

/// filter の語の表の前の部分（44 語）。
fn words_front() -> &'static [&'static str] {
    &[
        "aaround_",
        "accept_",
        "account_",
        "acctcore_",
        "acctdoc_",
        "accthb_",
        "accthome_",
        "acctled_",
        "acctlook_",
        "acctpcore_",
        "acctproj_",
        "acctsess_",
        "acctwin_",
        "acctwire_",
        "afocus_",
        "aord_",
        "apop_",
        "askcard_",
        "athr_",
        "batchpanel_",
        "bhalf_",
        "board_min_",
        "bport_",
        "brand_",
        "btuck_",
        "cadopt_",
        "cgdom_",
        "cmark_",
        "contract_form_",
        "cround_",
        "csled_",
        "denv_",
        "ecache_",
        "flight_",
        "fmark_",
        "frame_",
        "fserve_",
        "fstop_",
        "gapspage_",
        "gfresh_",
        "ghb_",
        "glabel_",
        "gnav_",
        "gpill_",
    ]
}

/// filter の語の表の中の部分（44 語）。
fn words_middle() -> &'static [&'static str] {
    &[
        "gpulse_",
        "graph_",
        "gsum_",
        "gtuck_",
        "gview_",
        "hacols_",
        "hbconf_",
        "hbpost_",
        "hbproc_",
        "hbroute_",
        "hcard_",
        "hcled_",
        "hcnx_",
        "hcproj_",
        "hcsess_",
        "hfig_",
        "hnunk_",
        "hook_",
        "hruling_",
        "hsblock_",
        "hsderive_",
        "hspage_",
        "hsym_",
        "iclose_",
        "ilink_",
        "kcli_",
        "klink_",
        "launch_",
        "lcard_",
        "ledgerblock_",
        "lhome_",
        "lspark_",
        "mapview_",
        "mkeys_",
        "mlink_",
        "mstore_",
        "mtree_",
        "nact_",
        "nbatch_",
        "ncard_",
        "nextstep_",
        "nodepage_",
        "nstall_",
        "nsum_",
    ]
}

/// filter の語の表の後の部分（44 語）。
fn words_back() -> &'static [&'static str] {
    &[
        "nsumw_",
        "ntime_",
        "nxact_",
        "parts_",
        "pclosed_",
        "pfold_",
        "pgz_",
        "pipe_",
        "plimit_",
        "pmore_",
        "pquest_",
        "project_",
        "ptitle_",
        "punmap_",
        "pwhole_",
        "qblock_",
        "qgate_",
        "qkey_",
        "question_",
        "relay_",
        "rhold_",
        "runsdoc_",
        "rvk_",
        "saxis_",
        "sclosed_",
        "seatblock_",
        "seatcard_",
        "server_",
        "sesplit_",
        "shb_",
        "skeleton_",
        "smore_",
        "stage_",
        "stats_",
        "stcli_",
        "steady_",
        "sxaxis_",
        "ticker_",
        "tipx_",
        "topbar_",
        "tz_",
        "urpanel_",
        "uword_",
        "wstrip_",
    ]
}

/// 歯の名の残りの字が filter の語を含まないことを見る。
fn names_avoid(names: Vec<&str>, words: &[&str]) {
    for name in names {
        let rest = name
            .strip_prefix("gbnote_")
            .unwrap_or_else(|| panic!("{name} が gbnote_ で始まらない"));
        for w in words {
            assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
        }
    }
}
