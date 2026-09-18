//! `folio hello` の歯（便 21・docs/design/delivery-21.md §1 (c)）。binary 経由だけで測る（unit は置かない）。
//! 3 状態の行数・1 回きり・印を書けない・1 行の中身・判定の順と印の名を見る。
//! `--state` は必ず一時 dir（持ち主の印の置き場を触らない）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn folio_hello(dir: &Path, state: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("hello")
        .arg("--dir")
        .arg(dir)
        .arg("--state")
        .arg(state)
        .output()
        .expect("folio を起動できない")
}

/// 印の名の一覧（印の置き場がまだ無ければ空）。
fn marks(state: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(state.join("greeted")) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// 一時 dir の中の「プロジェクトの根」と「印の置き場」（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-hello-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let w = Work { root };
        fs::create_dir_all(w.project()).unwrap();
        w
    }

    /// プロジェクトの根（design-intent の親）。
    fn project(&self) -> PathBuf {
        self.root.join("project")
    }

    /// 正本の置き場（作らなければ未整備）。
    fn dir(&self) -> PathBuf {
        self.project().join("design-intent")
    }

    /// 印の置き場。
    fn state(&self) -> PathBuf {
        self.root.join("state")
    }

    /// 整備済みにする（constitution.yaml だけ置く）。
    fn prepare(&self) {
        fs::create_dir_all(self.dir()).unwrap();
        fs::write(self.dir().join("constitution.yaml"), "version: v0.1\n").unwrap();
    }

    /// 止める設定を根の直下に置く。
    fn quiet(&self) {
        fs::write(self.project().join(".folio-quiet"), "").unwrap();
    }

    fn hello(&self) -> Output {
        folio_hello(&self.dir(), &self.state())
    }

    fn marks(&self) -> Vec<String> {
        marks(&self.state())
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 標準出力の行数。
fn lines(out: &Output) -> usize {
    stdout(out).lines().count()
}

/// 1 行だけ出て終了 0。出た 1 行を返す。
fn assert_greets(out: &Output) -> String {
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(out), stderr(out));
    assert_eq!(lines(out), 1, "{}", stdout(out));
    assert!(stdout(out).ends_with('\n'), "改行 1 つで終わらない");
    stdout(out).trim_end_matches('\n').to_string()
}

/// 何も出ずに終了 0。
fn assert_silent(out: &Output) {
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(out), stderr(out));
    assert_eq!(lines(out), 0, "{}", stdout(out));
}

#[test]
fn hello_unprepared_greets_and_marks() {
    let w = Work::new("unprepared");
    assert_greets(&w.hello());
    assert_eq!(w.marks().len(), 1, "{:?}", w.marks());
}

#[test]
fn hello_prepared_says_nothing() {
    let w = Work::new("prepared");
    w.prepare();
    assert_silent(&w.hello());
    assert!(
        w.marks().is_empty(),
        "整備済みで印が出来た: {:?}",
        w.marks()
    );
}

/// 止める設定は未整備でも効く（判定の順で最初）。
#[test]
fn hello_quiet_file_says_nothing_even_when_unprepared() {
    let w = Work::new("quiet");
    w.quiet();
    assert_silent(&w.hello());
    assert!(
        w.marks().is_empty(),
        "止める設定で印が出来た: {:?}",
        w.marks()
    );
}

#[test]
fn hello_greets_only_once_per_project() {
    let w = Work::new("once");
    assert_greets(&w.hello());
    let after_first = w.marks();
    assert_silent(&w.hello());
    assert_eq!(w.marks(), after_first, "2 回目で印が増えた");
    assert_eq!(after_first.len(), 1, "{after_first:?}");
}

/// 別の根なら改めて 1 行（印の名も別）。
#[test]
fn hello_greets_again_in_another_root() {
    let w = Work::new("another-root");
    assert_greets(&w.hello());
    let other = w.root.join("other-project");
    fs::create_dir_all(&other).unwrap();
    assert_greets(&folio_hello(&other.join("design-intent"), &w.state()));
    let names = w.marks();
    assert_eq!(names.len(), 2, "{names:?}");
    assert_ne!(names[0], names[1], "別の根で同じ名の印: {names:?}");
}

/// 印の名は 64 桁の 16 進 1 本。
#[test]
fn hello_mark_name_is_hex_digest() {
    let w = Work::new("mark-name");
    assert_greets(&w.hello());
    let names = w.marks();
    assert_eq!(names.len(), 1, "{names:?}");
    assert_eq!(names[0].len(), 64, "{names:?}");
    assert!(
        names[0]
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
        "16 進でない: {names:?}"
    );
}

/// 印を書けなくても 1 行は出したまま「まだ分からない」2 で終わる（次回も出る・P-4.1）。
#[test]
fn hello_greets_even_when_the_mark_cannot_be_written() {
    let w = Work::new("unwritable");
    let state = w.root.join("state-is-a-file");
    fs::write(&state, "").unwrap();
    let out = folio_hello(&w.dir(), &state);
    assert_eq!(lines(&out), 1, "{}", stdout(&out));
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("folio hello: まだ分からない: 印を書けない: "),
        "{}",
        stderr(&out)
    );
}

#[test]
fn hello_line_names_intake_and_the_quiet_file() {
    let w = Work::new("line");
    let line = assert_greets(&w.hello());
    assert!(line.contains("folio intake"), "{line}");
    assert!(line.contains(".folio-quiet"), "{line}");
}
