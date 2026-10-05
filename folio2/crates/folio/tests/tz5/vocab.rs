//! `folio check` の語彙の検査の歯（便 4・docs/design/delivery-4.md §1）。
//! tests/fixtures/vocab/ の 2 組（最小の手書き 4 file に変異 1 つ）で 不合格 1。
//! 各組の違反はちょうど 1 件で、名札と語・場所まで見る（別の理由で落ちた組・免除の形を数えた組を緑にしない）。
//! 2 組の写しの名 fixture-constitution は外の置き場なので、名札は検査の名 [語彙]（便 156・便 203）。
//! 名の読めない置き場と、名が folio2 の置き場で行 R-9〜R-11 を持たない置き場は外でないので、名札は行 id（便 203・歯 f203_）。
#![cfg(test)]

use crate::common::{repo_root, stdout};
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn folio_check(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tz"))
        .arg("check")
        .arg("--dir")
        .arg(dir)
        .output()
        .expect("folio を起動できない")
}

/// 違反の行（`[種類] …`）だけを拾う。
fn violations(out: &Output) -> Vec<String> {
    stdout(out)
        .lines()
        .filter(|l| l.starts_with('['))
        .map(str::to_string)
        .collect()
}

fn assert_single_unknown_word(name: &str, at: &str, word: &str) {
    let out = folio_check(&repo_root().join("tests/fixtures/vocab").join(name));
    assert_eq!(
        out.status.code(),
        Some(1),
        "{name}: {}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = violations(&out);
    assert_eq!(v.len(), 1, "{name}: 違反は変異の 1 件だけのはず: {v:?}");
    assert!(
        v[0].starts_with(&format!("[語彙] {at}")) && v[0].contains(&format!("「{word}」")),
        "{name}: {v:?}"
    );
    assert!(stdout(&out).contains("不合格"), "{name}");
}

#[test]
fn vocab_unknown_word_fails() {
    assert_single_unknown_word("unknown-word", "P-1 plain", "widget");
}

#[test]
fn vocab_exemptions_count_only_the_unknown_word() {
    assert_single_unknown_word("exemptions", "P-1 plain", "widget");
}

/// 便 203: 骨格（行 R-9〜R-11 を持たない）の名を消した置き場と、名だけ folio2 の置き場の名にした置き場は外の置き場でないので、
/// 条 P-1 の 3 つの崩しの名札は行 id（便 156 の行の在否の形は畳んだ＝行が無くても検査の名にならない）。
#[test]
fn f203_places_not_abroad_show_the_ids_even_without_the_rows() {
    const BROKEN: [(&str, &str); 3] = [
        ("R-10", "P-1: plain が無い"),
        ("R-11", "P-1: P-1.1: strength must-not と文末が合わない（must-not ⇔ 〜ない。）"),
        ("R-9", "P-1 title: 語彙に無い英字の語「foobar」"),
    ];
    for (case, meta) in [("nameless", "meta:\n"), ("home", "meta:\n  id: folio2-constitution\n")] {
        let root = std::env::temp_dir().join(format!("folio-f203-vocab-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let dir = root.join("design-intent");
        let init = Command::new(env!("CARGO_BIN_EXE_tz"))
            .arg("init")
            .arg("--dir")
            .arg(&dir)
            .output()
            .expect("folio を起動できない");
        assert_eq!(init.status.code(), Some(0), "{case}: {}", String::from_utf8_lossy(&init.stderr));
        let path = dir.join("constitution.yaml");
        let text = fs::read_to_string(&path)
            .unwrap()
            .replacen("meta:\n  id: 未記入\n", meta, 1)
            .replacen("    title: 承認の受け方と検査の値\n", "    title: 承認の受け方と検査の値 foobar\n", 1)
            .lines()
            .filter(|l| !l.starts_with("    plain: 承認は決まった対話面"))
            .map(|l| format!("{l}\n"))
            .collect::<String>()
            .replacen("strength: must, text:", "strength: must-not, text:", 1);
        fs::write(&path, text).unwrap();
        let out = folio_check(&dir);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(out.status.code(), Some(1), "{case}: {}", stdout(&out));
        let got: Vec<String> = violations(&out)
            .into_iter()
            .filter(|l| BROKEN.iter().any(|(_, msg)| l.ends_with(msg)))
            .collect();
        let want: Vec<String> = BROKEN.iter().map(|(id, msg)| format!("[{id}] {msg}")).collect();
        assert_eq!(got, want, "{case}: {}", stdout(&out));
    }
}
