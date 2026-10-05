//! tz の口 context の歯（行 k-ctx-cli・接頭辞 kctx_・設計ノート surface-wave29a・判断の記録 ADR-51 の決定 (4) と帰結）。
//! tz context --row <ノート>#<行> [--dir <dir>] が、folio の lib の入口 folio::entry::context（行 f-ctx-build）と同じ塊の字を
//! 標準出力へ出し、組めない理由の終了 code（1 と 2）を入口のまま返し、使い方の誤りを 1 で断る。
//! 見本の置き場は一時の dir に書く正本 3 file と判断の記録 1 本と設計ノート 1 本（置き場の名の字 tag で 2 つを分ける）。
#![cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 見本のノート n1 の行 a（req は FR1・題と節の本文と done が R-1 と P-1.1 と ADR-1 決定 (1) を名指す）。
const NOTE: &str = "sections:
  - {n: 1, type: prose, title: 節, body: 本文の R-1}
  - n: 2
    type: contract-table
    rows:
      - {id: a, title: 題の P-1.1, req: [FR1], section: '1', done: 測る ADR-1 決定 (1)}
";

/// 置き場 `dir` に見本を書く（字 tag を本文の字に混ぜて 2 つの置き場を分ける）。
fn place(dir: &Path, tag: &str) {
    fs::create_dir_all(dir.join("adr")).unwrap();
    fs::create_dir_all(dir.join("design-note")).unwrap();
    let files = [
        (
            "constitution.yaml",
            format!(
                "articles:\n  - id: P-1\n    title: 条一 {tag}\n    statements:\n      - {{id: P-1.1, text: 規範 {tag}}}\n"
            ),
        ),
        (
            "rules.yaml",
            format!("thresholds:\n  - {{id: R-1, article: P-1, what: 何 {tag}, value: 3 本}}\n"),
        ),
        (
            "srs.yaml",
            format!(
                "requirements:\n  - {{id: FR1, title: 要件 {tag}, shall: 本文 {tag}, basis: [P-1]}}\nacceptance:\n  - {{id: AC1, title: 受入 {tag}, verifies: [FR1]}}\n"
            ),
        ),
        (
            "adr/ADR-1.yaml",
            format!("title: 記録 {tag}\ndecision: |\n  (1) 項一 {tag}\n  (2) 項二\n"),
        ),
        ("design-note/n1.yaml", NOTE.to_string()),
    ];
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
}

/// 歯ごとの一時の根（名に歯の字と process の id と時刻の ns）。
fn scratch(test: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let root = std::env::temp_dir().join(format!("kctx-{test}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    root
}

/// tz を `cwd` で撃ち、(終了 code, 標準出力, 標準エラー)。
fn tz(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_tz"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("tz を起動できない");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn kctx_prints_the_same_blocks_as_the_folio_entry() {
    let root = scratch("same");
    let dir = root.join("di");
    place(&dir, "根");
    let want = folio::entry::context(&dir, "n1#a").expect("入口が組む");
    assert!(
        want.starts_with("- FR1 要件\n") && want.contains("- ADR-1 判断の記録\n"),
        "{want}"
    );
    let d = dir.to_str().unwrap();
    for args in [
        vec!["context", "--row", "n1#a", "--dir", d],
        vec!["context", "--dir", d, "--row=n1#a"],
        vec!["context", "--row=n1#a", &format!("--dir={d}")],
    ] {
        assert_eq!(
            tz(&root, &args),
            (0, want.clone(), String::new()),
            "{args:?}"
        );
    }
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn kctx_dir_defaults_to_design_intent_under_the_cwd() {
    let root = scratch("cwd");
    place(&root.join("design-intent"), "根");
    place(&root.join("folio2/design-intent"), "持ち込み");
    let want = folio::entry::context(&root.join("design-intent"), "n1#a").expect("入口が組む");
    let (rc, out, err) = tz(&root, &["context", "--row", "n1#a"]);
    assert_eq!((rc, err.as_str()), (0, ""), "{err}");
    assert_eq!(out, want);
    assert!(
        out.contains("要件 根") && !out.contains("持ち込み"),
        "{out}"
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn kctx_usage_errors_are_one() {
    let root = scratch("usage");
    for (args, why) in [
        (vec!["context"], "--row が無い"),
        (vec!["context", "--row"], "--row の値が無い"),
        (vec!["context", "--row="], "--row の値が空"),
        (
            vec!["context", "--row", "n1#a", "--row", "n1#a"],
            "--row が 2 度ある",
        ),
        (
            vec!["context", "--row", "n1#a", "--dir", "a", "--dir=b"],
            "--dir が 2 度ある",
        ),
        (
            vec!["context", "--row", "n1#a", "--check"],
            "知らない引数 --check",
        ),
    ] {
        let (rc, out, err) = tz(&root, &args);
        assert_eq!((rc, out.as_str()), (1, ""), "{args:?}");
        assert_eq!(
            err,
            format!("tz context: {why}\nusage: tz context --row <ノート>#<行> [--dir <dir>]\n"),
            "{args:?}"
        );
    }
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn kctx_refusals_keep_the_entry_code() {
    let root = scratch("refuse");
    let dir = root.join("di");
    place(&dir, "根");
    let d = dir.to_str().unwrap();
    for (row, code) in [("n1", 1), ("n9#a", 1), ("n1#z", 1)] {
        let (want_code, why) = folio::entry::context(&dir, row).unwrap_err();
        assert_eq!(want_code, code, "{row}");
        assert_eq!(
            tz(&root, &["context", "--row", row, "--dir", d]),
            (code.into(), String::new(), format!("tz context: {why}\n")),
            "{row}"
        );
    }
    fs::remove_file(dir.join("srs.yaml")).unwrap();
    let (rc, out, err) = tz(&root, &["context", "--row", "n1#a", "--dir", d]);
    assert_eq!((rc, out.as_str()), (2, ""));
    assert!(
        err.starts_with("tz context: ") && err.contains("srs.yaml を読めない"),
        "{err}"
    );
    fs::remove_dir_all(&root).unwrap();
}
