//! 突き合わせの歯（parity・便 0・docs/design/delivery-0.md §1）。
//! design-intent の写し全部（adr/ と anchors/ を含む）を一時 dir に作り、git init と 1 commit を行い
//! （tests/run_floor_cases.py と同じ作り）、変異を 1 つだけ当てて、day-1 の床 `python3 scripts/check_draft.py --dir <写し>` と
//! `folio check --dir <写し>` の終了コードが一致することを見る。期待の終了コードも §1 の値で pin する
//! （両方が同じ理由で起動できずに揃った、を緑にしない）。要件書と語彙には変異を当てない（床が数えないため）。

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), &to).unwrap();
        }
    }
}

/// git を呼ぶ。環境変数 GIT_* は継承しない（外の repo へ照合先をすげ替えない）。
fn git(cwd: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    let out = cmd
        .current_dir(cwd)
        .args([
            "-c",
            "user.email=fx@example",
            "-c",
            "user.name=fx",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .expect("git を起動できない");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn exit_code(cmd: &mut Command, what: &str) -> i32 {
    let out = cmd
        .output()
        .unwrap_or_else(|e| panic!("{what} を起動できない: {e}"));
    out.status.code().unwrap_or_else(|| {
        panic!(
            "{what} が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

/// 1 入力: 写しを作って commit し、変異を当て、床と folio を掛ける。
fn parity(case: &str, expected: i32, mutate: impl FnOnce(&Path)) {
    let root = repo_root();
    let td = std::env::temp_dir().join(format!("folio-parity-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    let work = td.join("design-intent");
    copy_tree(&root.join("design-intent"), &work);
    git(&td, &["init", "-q"]);
    git(&td, &["add", "-A"]);
    git(&td, &["commit", "-q", "--allow-empty", "-m", "fixture"]);
    mutate(&work);

    let floor = exit_code(
        Command::new("python3")
            .arg(root.join("scripts/check_draft.py"))
            .arg(OsStr::new("--dir"))
            .arg(&work),
        "床（python3 scripts/check_draft.py）",
    );
    let folio = exit_code(
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("check")
            .arg("--dir")
            .arg(&work),
        "folio check",
    );
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        (floor, folio),
        (expected, expected),
        "{case}: 床 {floor} / folio {folio}（期待 {expected}）"
    );
}

fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない: {}", path.display());
    fs::write(path, after).unwrap();
}

#[test]
fn parity_unmutated_passes() {
    parity("unmutated", 0, |_| {});
}

/// rules.yaml に重複キーを 1 つ足す = 行 R-1 を同じ id でもう 1 行書く（行の表のキー id の重複）。
/// YAML の同じ表に同じキーを 2 度書く形は床が「読めない」（2）に倒すので、終了コード 1 の入力はこの形になる。
#[test]
fn parity_rules_duplicate_key_fails() {
    parity("rules-duplicate-key", 1, |work| {
        edit(&work.join("rules.yaml"), |text| {
            let line = text
                .lines()
                .find(|l| l.starts_with("  - {id: R-1,"))
                .expect("行 R-1 が無い");
            text.replacen(line, &format!("{line}\n{line}"), 1)
        });
    });
}

#[test]
fn parity_constitution_unknown_section_fails() {
    parity("constitution-unknown-section", 1, |work| {
        edit(&work.join("constitution.yaml"), |text| {
            format!("{text}\nparity_unknown_section: schema の top_level に無い節\n")
        });
    });
}

#[test]
fn parity_constitution_empty_plain_fails() {
    parity("constitution-empty-plain", 1, |work| {
        edit(&work.join("constitution.yaml"), |text| {
            let article = text.find("\n  - id: P-1\n").expect("条 P-1 が無い");
            let start = article
                + text[article..]
                    .find("\n    plain: ")
                    .expect("P-1 の plain が無い")
                + 1;
            let end = start + text[start..].find('\n').unwrap();
            format!("{}    plain: \"\"{}", &text[..start], &text[end..])
        });
    });
}

#[test]
fn parity_constitution_missing_is_unknown() {
    parity("constitution-missing", 2, |work| {
        fs::remove_file(work.join("constitution.yaml")).unwrap();
    });
}
