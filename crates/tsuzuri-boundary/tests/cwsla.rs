//! 窓の状態の行に口座のふだんの statusline を出す歯（接頭辞 cwsla_・行 cs-status-seat・判断の記録 ADR-67）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に窓の作業場と偽の口座の置き場（設定 file の statusLine の命令は殻の 1 行）を置き、
//! tz consult statusline に session の JSON を渡して、口座の命令の出力の行が先で窓の 1 行が次の行に出ることと、出せない周は
//! 窓の 1 行の末に訳の語だけを添えることと、口座の行の色の制御の並びだけを通してほかの制御の並びと字を落とし切ることを見る。否定の見本は正しい見本から 1 つだけを替える。
#![cfg(test)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use tsuzuri_contract::consult::{Form, Starter, WindowFile, WindowId};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::status::SEP;

const INPUT: &str = r#"{"context_window":{"used_percentage":42.4}}"#;

/// 正しい見本の命令（環境の CWSLA_TAG と標準入力を写して 1 行を出す）。
const ECHO: &str = r#"printf '%s:' "$CWSLA_TAG"; cat"#;

/// 窓の 1 行（控えだけを持つ作業場・所見の dir と束は置かない）。
fn window_line() -> String {
    ["窓 cw3", "題 t3-hub.88", "fable・xhigh", "文脈 42%"].join(SEP)
}

/// 歯ごとの置き場に作業場 `consult-cw3` と口座の置き場 `acct` を作る（前の撃ちの残りを消す）。
fn place(name: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cwsla-{name}"));
    let _ = fs::remove_dir_all(&root);
    let ws = root.join("consult-cw3");
    fs::create_dir_all(ws.join(".consult")).unwrap();
    let w = WindowFile {
        id: WindowId::parse("cw3").unwrap(),
        form: Form::Talk,
        topic: Some("t3-hub.88".into()),
        model: "fable".into(),
        effort: "xhigh".into(),
        starter: Starter::Chat,
        uttered: Some("20261006T0204Z".into()),
        request: None,
        made: "20261006T0204Z".into(),
    };
    fs::write(ws.join(".consult/window.json"), wire::encode(&w).unwrap()).unwrap();
    let acct = root.join("acct");
    fs::create_dir_all(&acct).unwrap();
    (ws, acct)
}

/// 命令 `command` を statusLine に持つ設定の字（JSON の字の囲みと逆斜線と二重引用符だけを逃がす）。
fn settings_with(command: &str) -> String {
    let quoted = command.replace('\\', "\\\\").replace('"', "\\\"");
    format!(r#"{{"statusLine":{{"type":"command","command":"{quoted}","refreshInterval":10}}}}"#)
}

/// 口座の置き場を環境に置いて口を撃つ（`edit` で環境を足すか除く）。
fn shoot(ws: &Path, acct: &Path, edit: impl Fn(&mut Command)) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_tz"));
    c.current_dir(ws)
        .args(["consult", "statusline", ws.to_str().unwrap()])
        .env("CLAUDE_CONFIG_DIR", acct)
        .env("CWSLA_TAG", "acct")
        .env_remove("TZ_CONSULT_STATUSLINE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    edit(&mut c);
    let mut child = c.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(INPUT.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn text(out: &Output) -> String {
    String::from_utf8(out.stdout.clone())
        .unwrap()
        .trim_end_matches('\n')
        .to_string()
}

/// 口座の設定の statusLine の命令を、同じ標準入力と環境で殻に撃った出力が 1 行目、窓の 1 行が 2 行目に出る。
#[test]
fn cwsla_account_line_comes_first() {
    let (ws, acct) = place("first");
    fs::write(acct.join("settings.json"), settings_with(ECHO)).unwrap();
    let got = text(&shoot(&ws, &acct, |_| {}));
    let want = [format!("acct:{INPUT}"), window_line()];
    assert_eq!(got.lines().collect::<Vec<_>>(), want);
}

/// 設定 file が無い・JSON でない・置き場の環境が無い・鍵が無い・命令が rc 1・時間切れ・入れ子の各々で、窓の 1 行の末に
/// 欄 `口座 <訳の語>` だけを足した 1 行を出し、落ちた命令の出力は出さず、入れ子では命令を撃たない。
#[test]
fn cwsla_misses_add_only_the_word() {
    type Case = (&'static str, Option<String>, fn(&mut Command), &'static str);
    let mark = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cwsla-nested-ran");
    let _ = fs::remove_file(&mark);
    let nested = settings_with(&format!("touch {}; echo ran", mark.display()));
    let cases: [Case; 7] = [
        ("設定 file が無い", None, |_| {}, "読めない"),
        ("JSON でない", Some("{".into()), |_| {}, "読めない"),
        (
            "置き場の環境が無い",
            Some(settings_with(ECHO)),
            |c| {
                c.env_remove("CLAUDE_CONFIG_DIR");
            },
            "読めない",
        ),
        (
            "鍵が無い",
            Some(r#"{"statusLine":{"type":"command","refreshInterval":10}}"#.into()),
            |_| {},
            "鍵なし",
        ),
        (
            "命令が rc 1",
            Some(settings_with("echo leak; exit 1")),
            |_| {},
            "落ちた",
        ),
        (
            "時間切れ",
            Some(settings_with("echo late; sleep 5")),
            |_| {},
            "時間切れ",
        ),
        (
            "入れ子",
            Some(nested),
            |c| {
                c.env("TZ_CONSULT_STATUSLINE", "1");
            },
            "入れ子",
        ),
    ];
    for (name, settings, edit, word) in cases {
        let (ws, acct) = place("miss");
        if let Some(s) = settings {
            fs::write(acct.join("settings.json"), s).unwrap();
        }
        let start = Instant::now();
        let got = text(&shoot(&ws, &acct, edit));
        assert!(start.elapsed() < Duration::from_secs(2), "{name}");
        assert_eq!(got, format!("{}{SEP}口座 {word}", window_line()), "{name}");
    }
    assert!(!mark.exists(), "入れ子で命令を撃った");
}

/// 口座の行の色の制御の並び（SGR）は通して行の末に色の戻しを足し、ほかの ESC の並び（m 以外で終わる ESC [ の並び・
/// 数と ; でない字を持つ m の並び・ESC ] の並び・単独の ESC）は丸ごと落として残骸を出さず、制御の字を落とす。見本は句ごとに 1 つ。
#[test]
fn cwsla_keeps_colors_and_drops_other_controls() {
    let cases: [(&str, &str, &str); 6] = [
        (
            "色",
            r"\033[01;32mgreen\033[0m",
            "\u{1b}[01;32mgreen\u{1b}[0m\u{1b}[0m",
        ),
        ("画面の消し", r"a\033[2Jb", "ab"),
        ("数でない m の並び", r"a\033[1:2mb", "ab"),
        ("題の書き換え", r"a\033]0;title\007b", "ab"),
        ("単独の ESC", r"a\033", "a"),
        ("制御の字", r"a\001b", "ab"),
    ];
    for (name, body, want) in cases {
        let (ws, acct) = place("ctl");
        let command = format!("printf '{body}'");
        fs::write(acct.join("settings.json"), settings_with(&command)).unwrap();
        let got = text(&shoot(&ws, &acct, |_| {}));
        assert_eq!(got, format!("{want}\n{}", window_line()), "{name}");
    }
}

/// 口座の行の間の空の行は残し、末の空の行を除く。
#[test]
fn cwsla_keeps_inner_empty_lines() {
    let (ws, acct) = place("empty");
    let command = r"printf 'first\r\n\nsecond\n\n'";
    fs::write(acct.join("settings.json"), settings_with(command)).unwrap();
    let got = text(&shoot(&ws, &acct, |_| {}));
    let want = [
        "first".to_string(),
        String::new(),
        "second".to_string(),
        window_line(),
    ];
    assert_eq!(got.split('\n').collect::<Vec<_>>(), want);
}

/// 口座の行は頭の 8 行だけを、1 行は頭の 400 字だけを出す（縁の 8 行と 9 行目・400 字と 401 字目）。
#[test]
fn cwsla_cuts_lines_and_length() {
    let (ws, acct) = place("cut");
    let long = "あ".repeat(400) + "い";
    let command = format!("printf '{long}\\n'; for i in 2 3 4 5 6 7 8 9; do echo line$i; done");
    fs::write(acct.join("settings.json"), settings_with(&command)).unwrap();
    let got = text(&shoot(&ws, &acct, |_| {}));
    let lines: Vec<&str> = got.lines().collect();
    assert_eq!(lines.len(), 9, "{got}");
    assert_eq!(lines[0], "あ".repeat(400));
    assert_eq!(lines[7], "line8");
    assert!(!got.contains("line9"), "{got}");
    assert_eq!(lines[8], window_line());
}
