//! tz の口 code の台帳の契約の bead の歯（接頭辞 kcbead_・行 t-code-bead・要件 FR2）。
//! tz code が一時の git の repo の file の一覧と、偽の構文で探す道具の stream と、contracts/ の toml と、偽の bd（撃たれた引数を
//! file に書き見本の台帳を出す shell の script）の台帳から、契約の bead の write-set の file の定義を引き、数に足し、
//! 台帳が読めない時は出力の後に 1 行を標準エラーへ書いて rc 2 を返す（--file は rc を替えない）。
//! 撃った後に歯ごとの一時の根が一時の dir に残らないことも測る。
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

/// 見本の台帳（kb.1 は契約の行で write-set は b.rs と +d.rs・kb.2 は pointer の行だけ）。
const LEDGER: &str = r#"[{"id":"kb.1","title":"t kb.1","status":"open","issue_type":"task","acceptance_criteria":"[[contract]]\nid = \"t-one\"\nwrite-set = [\"b.rs\", \"+d.rs\"]"},{"id":"kb.2","title":"t kb.2","status":"open","issue_type":"task","acceptance_criteria":"design = contracts/nx.toml#a"}]"#;

/// 壊れた台帳（kb.1 の write-set の値が字の配列でなく b.rs の JSON の字）。
const BROKEN: &str = r#"[{"id":"kb.1","title":"t kb.1","status":"open","issue_type":"task","acceptance_criteria":"[[contract]]\nid = \"t-one\"\nwrite-set = \"b.rs\""},{"id":"kb.2","title":"t kb.2","status":"open","issue_type":"task","acceptance_criteria":"design = contracts/nx.toml#a"}]"#;

/// 台帳の読みに撃たれる引数（境界の BD_ARGS と同じ並び・1 行 1 語）。
const BD_WORDS: &str = "--readonly\nlist\n--all\n--limit\n0\n--json\n";

/// 歯ごとの一時の根（名に歯の字と process の id と時刻の ns）。
fn scratch(test: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let root = std::env::temp_dir().join(format!("kcbead-{test}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(root.join("repo/.config")).unwrap();
    fs::create_dir_all(root.join("repo/contracts")).unwrap();
    root
}

/// 実行できる shell の script を書く。
fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// 一時の git の repo（a.rs・b.rs・規則の file・contracts/n1.toml を index に置く）と、見本の stream を出す偽の道具と、
/// 台帳 `ledger` を出し rc を `bd_rc` で返す偽の bd（撃たれた引数を根の bd-args.txt に 1 行ずつ書く）。
fn place(test: &str, ledger: &str, bd_rc: u8) -> (PathBuf, PathBuf, PathBuf) {
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
    fs::write(root.join("ledger.json"), ledger).unwrap();
    let sg = root.join("fake-sg");
    script(&sg, &format!("cat '{}'\n", root.join("stream.jsonl").display()));
    let bd = root.join("fake-bd");
    script(
        &bd,
        &format!(
            "printf '%s\\n' \"$@\" > '{}'\ncat '{}'\nexit {bd_rc}\n",
            root.join("bd-args.txt").display(),
            root.join("ledger.json").display()
        ),
    );
    (root, sg, bd)
}

/// tz code を repo の置き場と偽の道具と偽の bd で撃つ（rc・標準出力・標準エラー）。
fn tz(root: &Path, sg: &Path, bd: &Path, args: &[&str]) -> (i32, String, String) {
    let repo = root.join("repo");
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

/// 一時の根を作り、`runs` の引数ごとに tz code を撃ち、偽の bd に撃たれた引数の file を読んでから根を消す
/// （断言は返した物に撃つので、断言が落ちた周も根は先に消えている・歯ごとの dir を一時の dir に残さない）。
/// 根の path と撃ちの結果の列と偽の bd に撃たれた引数（撃たれなければ無い）を返す。
fn shot<const N: usize>(
    test: &str,
    ledger: &str,
    bd_rc: u8,
    runs: [&[&str]; N],
) -> (PathBuf, [Shot; N], Option<String>) {
    let (root, sg, bd) = place(test, ledger, bd_rc);
    let outs = runs.map(|args| tz(&root, &sg, &bd, args));
    let args = fs::read_to_string(root.join("bd-args.txt")).ok();
    let _ = fs::remove_dir_all(&root);
    (root, outs, args)
}

/// b.rs の定義の行（`tz code --file b.rs` の出力と --row の def の行）。
const DEF_G: &str = "def\tb.rs#fn g\t1-1";

#[test]
fn kcbead_row_reads_bead_contract_write_set() {
    let (root, [(rc, out, err)], args) = shot("row", LEDGER, 0, [&["--row", "kb.1#t-one"]]);
    assert_eq!(rc, 0, "{err}");
    assert_eq!(out, format!("file\tb.rs\tb.rs\t1\n{DEF_G}\nnone\t+d.rs\n"));
    assert_eq!(args.as_deref(), Some(BD_WORDS));
    assert!(!root.exists(), "撃った後は無い: {}", root.display());
}

#[test]
fn kcbead_counts_add_bead_rows() {
    let (_, [(rc, out, err)], _) = shot("counts", LEDGER, 0, [&[]]);
    assert_eq!(rc, 0, "{err}");
    for want in ["rows\t3", "writes\t3", "unbound\t2"] {
        assert!(out.lines().any(|l| l == want), "{want}: {out}");
    }
}

#[test]
fn kcbead_unreadable_ledger_is_unknown() {
    let runs: [&[&str]; 3] = [&[], &["--row", "kb.1#t-one"], &["--file", "b.rs"]];
    let (_, [counts, row, file], args) = shot("unknown", LEDGER, 1, runs);
    assert_eq!(counts.0, 2, "{}", counts.2);
    assert!(counts.1.lines().any(|l| l == "rows\t2"), "{}", counts.1);
    assert_eq!(counts.2.lines().count(), 1, "{}", counts.2);
    assert!(counts.2.contains("台帳が読めない"), "{}", counts.2);
    assert_eq!(row.0, 2, "名指した行が bead の契約かもしれない: {}", row.2);
    assert!(row.2.contains("台帳が読めない"), "{}", row.2);
    assert_eq!((file.0, file.1.as_str()), (0, format!("{DEF_G}\n").as_str()));
    assert_eq!(args.as_deref(), Some(BD_WORDS));
}

#[test]
fn kcbead_malformed_bead_is_unknown() {
    let (_, [(rc, _, err)], _) = shot("broken", BROKEN, 0, [&[]]);
    assert_eq!(rc, 2, "{err}");
    assert!(err.contains("台帳が読めない"), "{err}");
}
