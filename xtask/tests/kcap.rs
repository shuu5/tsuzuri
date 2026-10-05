//! task kfold-cap（xtask の src/kcap.rs・判断の記録 ADR-63 の決定 (8) の (a)）の歯（接頭辞 kcap_）: 一時の dir の toy の木
//! （member 2 つ・群 3 つ・main.rs を持たない dir 1 つ・段 3 本）で、名指した群の越えだけが rc 1 になり、ほかの群の越えと段の本数は
//! 出力に名指すだけで、群の行が群の dir の直下の .rs（main.rs を除く）の行の和で、上限が束ねの表の file の定数の行の字から読まれることを測り、
//! xtask の binary の task kfold-cap の撃ちの rc と出力を測る。
#![cfg(test)]

#[path = "../src/kcap.rs"]
mod kcap;

use std::path::{Path, PathBuf};
use std::process::Command;

/// toy の群の行の上限の定数の行（値 10 を字 1_0 で書く）。
const GROUP_LINE: &str = "const GROUP_LINES_CAP: usize = 1_0;";
/// toy の段の本数の上限の定数の行。
const STAGE_LINE: &str = "const STAGE_FILES_CAP: usize = 2;";
/// 群でない字の出力の行の末。
const NOT_GROUP: &str =
    " は群でない（member の tests/ の直下で main.rs を持つ dir を根からの相対で名指す）";

fn put(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().expect("親の dir")).expect(rel);
    std::fs::write(&path, body).expect(rel);
}

/// 改行で終わる n 行の字。
fn lines(n: usize) -> String {
    "//\n".repeat(n)
}

/// toy の木（名に pid と字 tag）。members は a と xtask。群 a/tests/g1 は main.rs 50 行と x.rs 6 行、
/// 群 a/tests/g2 は main.rs 2 行と y.rs 11 行（最後の行は改行で終わらない）と common/mod.rs 30 行と notes.txt 30 行、
/// 群 xtask/tests/teeth1 は main.rs と kfold.rs 3 行（頭の 1 行と group_line と STAGE_LINE）。a/tests/plain は main.rs を持たず z.rs 4 行。
/// 段は a/tests/s1.rs・s2.rs・s3.rs。
fn toy(tag: &str, group_line: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("tsuzuri-kcap-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    put(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\n    \"a\",\n    \"xtask\",\n]\n",
    );
    put(&root, "a/tests/g1/main.rs", &lines(50));
    put(&root, "a/tests/g1/x.rs", &lines(6));
    put(&root, "a/tests/g2/main.rs", &lines(2));
    put(&root, "a/tests/g2/y.rs", &format!("{}//", lines(10)));
    put(&root, "a/tests/g2/common/mod.rs", &lines(30));
    put(&root, "a/tests/g2/notes.txt", &lines(30));
    put(&root, "a/tests/plain/z.rs", &lines(4));
    for stage in ["s1", "s2", "s3"] {
        put(&root, &format!("a/tests/{stage}.rs"), &lines(1));
    }
    put(&root, "xtask/tests/teeth1/main.rs", "mod kfold;\n");
    put(
        &root,
        "xtask/tests/teeth1/kfold.rs",
        &format!("//! toy\n{group_line}\n{STAGE_LINE}\n"),
    );
    root
}

fn run(root: &Path, named: &[&str]) -> (i32, Vec<String>) {
    let named: Vec<String> = named.iter().map(|s| (*s).to_string()).collect();
    kcap::run(root, &named)
}

fn has(out: &[String], line: &str) -> bool {
    out.iter().any(|l| l == line)
}

#[test]
fn kcap_named_group_over_cap_fails() {
    let root = toy("over", GROUP_LINE);
    let (rc, out) = run(&root, &["a/tests/g2"]);
    assert_eq!(rc, 1, "{out:?}");
    assert!(
        has(&out, "群 a/tests/g2 11 行・名指し・越え（落とす）"),
        "{out:?}"
    );
    assert_eq!(
        out.last().map(String::as_str),
        Some("名指した群 1 のうち越え 1")
    );
}

#[test]
fn kcap_unnamed_group_over_cap_is_named_only() {
    let root = toy("unnamed", GROUP_LINE);
    let (rc, out) = run(&root, &["a/tests/g1"]);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(
        out,
        [
            "上限 群 10 行・段 2 本（xtask/tests/teeth1/kfold.rs の定数）",
            "群 a/tests/g1 6 行・名指し",
            "群 a/tests/g2 11 行・越え（名指さないので落とさない）",
            "群 xtask/tests/teeth1 3 行",
            "段 a/tests 3 本",
            "段の合計 3 本（上限 2・段の本数では落とさない）・上限に届いた",
            "名指した群 1 のうち越え 0",
        ]
    );
}

#[test]
fn kcap_group_lines_skip_main_rs() {
    let root = toy("main", GROUP_LINE);
    let (rc, out) = run(&root, &["a/tests/g1"]);
    assert_eq!(rc, 0, "{out:?}");
    assert!(has(&out, "群 a/tests/g1 6 行・名指し"), "{out:?}");
}

#[test]
fn kcap_group_lines_count_direct_rs_only() {
    let root = toy("direct", GROUP_LINE);
    let (_, out) = run(&root, &["a/tests/g1"]);
    assert!(
        has(
            &out,
            "群 a/tests/g2 11 行・越え（名指さないので落とさない）"
        ),
        "{out:?}"
    );
    let groups: Vec<&String> = out.iter().filter(|l| l.starts_with("群 ")).collect();
    assert_eq!(groups.len(), 3, "{out:?}");
}

#[test]
fn kcap_caps_read_from_table_lines() {
    let narrow = toy("narrow", GROUP_LINE);
    assert_eq!(run(&narrow, &["a/tests/g2"]).0, 1);
    let wide = toy("wide", "const GROUP_LINES_CAP: usize = 11;");
    let (rc, out) = run(&wide, &["a/tests/g2"]);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(
        out.first().map(String::as_str),
        Some("上限 群 11 行・段 2 本（xtask/tests/teeth1/kfold.rs の定数）")
    );
    assert!(has(&out, "群 a/tests/g2 11 行・名指し"), "{out:?}");

    let public = toy("public", "pub const GROUP_LINES_CAP: usize = 11;");
    assert_eq!(
        run(&public, &["a/tests/g2"]),
        (
            2,
            vec![
                "kfold.rs に行 const GROUP_LINES_CAP: usize = <数>; がちょうど 1 行無い（0 行）"
                    .to_string()
            ]
        )
    );
    let sum = toy("sum", "const GROUP_LINES_CAP: usize = 5 + 6;");
    assert_eq!(
        run(&sum, &["a/tests/g2"]),
        (
            2,
            vec!["kfold.rs の GROUP_LINES_CAP の値 5 + 6 が数字と _ だけの字でない".to_string()]
        )
    );
}

#[test]
fn kcap_stage_files_are_named_only() {
    let root = toy("stage", GROUP_LINE);
    let (rc, out) = run(&root, &["a/tests/g1"]);
    assert_eq!(rc, 0, "{out:?}");
    assert!(has(&out, "段 a/tests 3 本"), "{out:?}");
    assert!(
        has(
            &out,
            "段の合計 3 本（上限 2・段の本数では落とさない）・上限に届いた"
        ),
        "{out:?}"
    );
}

#[test]
fn kcap_refuses_non_group_and_empty_names() {
    let root = toy("refuse", GROUP_LINE);
    assert_eq!(
        run(&root, &["a/tests/g1", "a/tests/plain"]),
        (2, vec![format!("a/tests/plain{NOT_GROUP}")])
    );
    assert_eq!(run(&root, &[]), (2, vec![kcap::USAGE.to_string()]));
}

#[test]
fn kcap_runs_as_the_xtask_task() {
    let out = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["kfold-cap", "no/such/tests/g"])
        .output()
        .expect("xtask を撃つ");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        format!("xtask kfold-cap: no/such/tests/g{NOT_GROUP}\n")
    );
}
