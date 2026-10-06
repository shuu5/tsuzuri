//! tz の口 code の歯（接頭辞 kcode_・判断の記録 ADR-46 の決定 (1)〜(4)）。
//! tz code が一時の git の repo の file の一覧と、偽の構文で探す道具（--sg・撃たれた引数を file に書き見本の stream を出す
//! shell の script）の stream と、contracts/ の toml から、数と行の触る定義と file の定義を出し、規則の file の無い repo を
//! 道具を撃たずに 1 行で断る。撃った後に歯ごとの一時の根が一時の dir に残らないことも測る。
#![cfg(test)]
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 見本の stream（a.rs に struct S と fn f・b.rs に fn g・中核が読む欄だけ）。
const STREAM: &str = r#"{"ruleId":"struct","file":"a.rs","language":"Rust","range":{"byteOffset":{"start":0,"end":9},"start":{"line":0},"end":{"line":0}},"metaVariables":{"single":{"NAME":{"text":"S"}}}}
{"ruleId":"fn","file":"a.rs","language":"Rust","range":{"byteOffset":{"start":10,"end":20},"start":{"line":1},"end":{"line":2}},"metaVariables":{"single":{"NAME":{"text":"f"}}}}
{"ruleId":"fn","file":"b.rs","language":"Rust","range":{"byteOffset":{"start":0,"end":9},"start":{"line":0},"end":{"line":0}},"metaVariables":{"single":{"NAME":{"text":"g"}}}}
"#;

/// 見本の契約表の導出物（行 a は a.rs と新しい c.rs と置き場だけの b.rs・行 b は b.rs）。
const TOML: &str = "schema = 1\n\n[[contract]]\nid = \"a\"\nwrite-set = [\"a.rs\", \"+c.rs\", \"=b.rs\"]\n\n[[contract]]\nid = \"b\"\nwrite-set = [\"b.rs\"]\n";

/// 歯ごとの一時の根（名に歯の字と process の id と時刻の ns）。
fn scratch(test: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let root = std::env::temp_dir().join(format!("kcode-{test}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(root.join("repo/.config")).unwrap();
    fs::create_dir_all(root.join("repo/contracts")).unwrap();
    root
}

/// 一時の git の repo（a.rs・b.rs・規則の file・contracts/n1.toml を index に置く）と、rc を `rc` で返す偽の道具と、
/// 空の台帳（空の配列の字）を出す偽の bd（根の fake-bd）。
fn place(test: &str, rc: u8) -> (PathBuf, PathBuf) {
    let root = scratch(test);
    let repo = root.join("repo");
    for (name, text) in [
        ("a.rs", "struct S;\nfn f() {}\n"),
        ("b.rs", "fn g() {}\n"),
        (".config/code-defs.yml", "id: fn\n"),
        ("contracts/n1.toml", TOML),
    ] {
        fs::write(repo.join(name), text).unwrap();
    }
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&repo)
                .output()
                .unwrap()
                .status
                .success()
        )
    };
    git(&["init", "-q"]);
    git(&["add", "-A"]);
    fs::write(root.join("stream.jsonl"), STREAM).unwrap();
    let sg = root.join("fake-sg");
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\ncat '{}'\nexit {rc}\n",
        root.join("args.txt").display(),
        root.join("stream.jsonl").display()
    );
    fs::write(&sg, script).unwrap();
    fs::set_permissions(&sg, fs::Permissions::from_mode(0o755)).unwrap();
    let bd = root.join("fake-bd");
    fs::write(&bd, "#!/bin/sh\nprintf '[]'\n").unwrap();
    fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).unwrap();
    (root, sg)
}

/// tz code を repo の置き場と偽の道具と偽の bd で撃つ（rc・標準出力・標準エラー）。
fn tz(root: &Path, sg: &Path, args: &[&str]) -> (i32, String, String) {
    let repo = root.join("repo");
    let bd = root.join("fake-bd");
    let mut all = vec![
        "code",
        "--repo",
        repo.to_str().unwrap(),
        "--sg",
        sg.to_str().unwrap(),
        "--bd",
        bd.to_str().unwrap(),
    ];
    all.extend_from_slice(args);
    let out = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(&all)
        .output()
        .unwrap();
    let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
    (
        out.status.code().unwrap_or(-1),
        text(&out.stdout),
        text(&out.stderr),
    )
}

/// 撃ちの結果（rc・標準出力・標準エラー）。
type Shot = (i32, String, String);

/// 一時の根を作って `prep` で手を入れ、`runs` の引数ごとに tz code を撃ち、撃たれた引数の file を読んでから根を消す
/// （断言は返した物に撃つので、断言が落ちた周も根は先に消えている・歯ごとの dir を一時の dir に残さない・memo t3-hub.74.49.10）。
/// 根の path と撃ちの結果の列と撃たれた引数（道具が撃たれなければ無い）を返す。
fn shot<const N: usize>(
    test: &str,
    rc: u8,
    prep: fn(&Path),
    runs: [&[&str]; N],
) -> (PathBuf, [Shot; N], Option<String>) {
    let (root, sg) = place(test, rc);
    prep(&root);
    let outs = runs.map(|args| tz(&root, &sg, args));
    let args = fs::read_to_string(root.join("args.txt")).ok();
    let _ = fs::remove_dir_all(&root);
    (root, outs, args)
}

#[test]
fn kcode_row_lists_defs_of_write_set_files_only() {
    let (_, [(rc, out, err)], args) = shot("row", 0, |_| {}, [&["--row", "n1#a"]]);
    assert_eq!(rc, 0, "{err}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines,
        [
            "file\ta.rs\ta.rs\t2",
            "def\ta.rs#struct S\t1-1",
            "def\ta.rs#fn f\t2-3",
            "none\t+c.rs"
        ]
    );
    assert_eq!(
        args.as_deref(),
        Some("scan\n--rule\n.config/code-defs.yml\n--json=stream\n")
    );
}

#[test]
fn kcode_missing_rule_file_refuses_in_one_line() {
    let norule = |root: &Path| fs::remove_file(root.join("repo/.config/code-defs.yml")).unwrap();
    let (_, [(rc, out, err)], args) = shot("norule", 0, norule, [&["--row", "n1#a"]]);
    assert_eq!(rc, 1);
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(
        err.contains(".config/code-defs.yml") && err.contains("が無い"),
        "{err}"
    );
    assert!(out.is_empty());
    assert!(args.is_none(), "規則の file が無い時に道具を撃った");
}

#[test]
fn kcode_counts_and_file_lines() {
    let (_, [(rc, out, _), file], _) = shot("counts", 0, |_| {}, [&[], &["--file", "b.rs"]]);
    assert_eq!(rc, 0);
    for want in [
        "files\t4",
        "defs\t3",
        "kind\tfn\t2",
        "kind\tstruct\t1",
        "rows\t2",
        "writes\t2",
        "unbound\t1",
        "skipped\t0",
    ] {
        assert!(out.lines().any(|l| l == want), "{want}: {out}");
    }
    assert_eq!((file.0, file.1.as_str()), (0, "def\tb.rs#fn g\t1-1\n"));
}

#[test]
fn kcode_unreadable_and_named_errors() {
    let (_, [errs], _) = shot("errs", 1, |_| {}, [&[]]);
    assert_eq!(errs.0, 2, "道具の rc が 0 でない時はまだ分からない");
    let runs = [
        &["--row", "n1#zz"][..],
        &["--file", "zz.rs"],
        &["--row", "n1#a", "--file", "a.rs"],
        &["--what"],
    ];
    let (_, outs, _) = shot("named", 0, |_| {}, runs);
    for (args, (rc, out, err)) in runs.iter().zip(outs) {
        assert_eq!((rc, out.as_str()), (1, ""), "{args:?}: {err}");
    }
}

/// 撃つ間は根が在り（tz code が rc 0 を返し、撃たれた引数の file が読める）、撃った後は根の dir が無い。
#[test]
fn kcode_scratch_is_gone_after_the_shot() {
    let (root, [(rc, _, err)], args) = shot("gone", 0, |_| {}, [&["--row", "n1#a"]]);
    assert_eq!(rc, 0, "根が在る間に撃った: {err}");
    assert!(args.is_some(), "根が在る間に道具が引数の file を書いた");
    assert!(!root.exists(), "撃った後は無い: {}", root.display());
}
