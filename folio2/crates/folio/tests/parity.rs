//! 突き合わせの歯（parity・便 0・docs/design/delivery-0.md §1）。
//! design-intent の写し全部（adr/ と anchors/ を含む）を一時 dir に作り、git init と 1 commit を行い
//! （tests/run_floor_cases.py と同じ作り）、変異を 1 つだけ当てて、day-1 の床 `python3 scripts/check_draft.py --dir <写し>` と
//! `folio check --dir <写し>` の終了コードが一致することを見る。期待の終了コードも §1 の値で pin する
//! （両方が同じ理由で起動できずに揃った、を緑にしない）。語彙には変異を当てない。要件書への変異は便 1 以降（参照 id・語彙の検査）。
//! 判断の記録（adr/）への変異は便 5 以降（欄の決まりの検査）。判断の記録と正本 4 file の突き合わせの変異は便 6（(18)〜(22)）。

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

/// `text` の中で、`block` の行から始まる区間の最初の `field` 行を `f` で書き換える（便 1 の変異の共通形）。
fn edit_line_in_block(
    text: &str,
    block: &str,
    field: &str,
    f: impl FnOnce(&str) -> String,
) -> String {
    let at = text
        .find(block)
        .unwrap_or_else(|| panic!("区間 {block:?} が無い"));
    let start = at
        + text[at..]
            .find(field)
            .unwrap_or_else(|| panic!("{block:?} の {field:?} が無い"))
        + 1;
    let end = start + text[start..].find('\n').unwrap();
    format!("{}{}{}", &text[..start], f(&text[start..end]), &text[end..])
}

/// (6) 要件書 FR1 の basis に実在しない条 id を 1 つ足す（参照 id の解決・便 1）。
#[test]
fn parity_srs_dangling_article_id_fails() {
    parity("srs-dangling-article-id", 1, |work| {
        edit(&work.join("srs.yaml"), |text| {
            edit_line_in_block(text, "\n  - id: FR1\n", "\n    basis: [", |line| {
                line.replacen("basis: [", "basis: [P-99, ", 1)
            })
        });
    });
}

/// (7) rules 行 R-3 の article を実在しない条 id にする。
#[test]
fn parity_rules_article_dangling_fails() {
    parity("rules-article-dangling", 1, |work| {
        edit(&work.join("rules.yaml"), |text| {
            edit_line_in_block(text, "\n  - {id: R-3,", "\n  - {id: R-3,", |line| {
                let from =
                    line.find("article: ").expect("R-3 の article が無い") + "article: ".len();
                let to = from + line[from..].find(',').unwrap();
                format!("{}P-99{}", &line[..from], &line[to..])
            })
        });
    });
}

/// (8) 憲法 P-2 の relations.rules から R-3 を外す（逆参照・便 1）。
#[test]
fn parity_constitution_orphan_rule_fails() {
    parity("constitution-orphan-rule", 1, |work| {
        edit(&work.join("constitution.yaml"), |text| {
            edit_line_in_block(text, "\n  - id: P-2\n", "\n    relations: ", |line| {
                line.replacen("R-3, ", "", 1)
            })
        });
    });
}

/// (9) 憲法 meta.counts.always を実数と違う値にする（件数・便 1）。
#[test]
fn parity_constitution_bad_counts_fails() {
    parity("constitution-bad-counts", 1, |work| {
        edit(&work.join("constitution.yaml"), |text| {
            edit_line_in_block(text, "\n  counts:\n", "\n    always: ", |line| {
                let n: usize = line["    always: ".len()..]
                    .trim()
                    .parse()
                    .expect("always が数でない");
                format!("    always: {}", n + 1)
            })
        });
    });
}

/// (10) 憲法 P-1 の plain の末尾に語彙に無い英字の語 zzzz を足す（語彙の検査 R-9・便 4）。
#[test]
fn parity_constitution_unknown_word_fails() {
    parity("constitution-unknown-word", 1, |work| {
        edit(&work.join("constitution.yaml"), |text| {
            edit_line_in_block(text, "\n  - id: P-1\n", "\n    plain: ", |line| {
                format!("{line}zzzz")
            })
        });
    });
}

/// (11) 要件書 FR1 の shall の末尾に zzzz を足す。
#[test]
fn parity_srs_unknown_word_fails() {
    parity("srs-unknown-word", 1, |work| {
        edit(&work.join("srs.yaml"), |text| {
            edit_line_in_block(text, "\n  - id: FR1\n", "\n    shall: ", |line| {
                format!("{line}zzzz")
            })
        });
    });
}

/// (12) rules 行 R-2 の what の末尾に zzzz を足す。
#[test]
fn parity_rules_unknown_word_fails() {
    parity("rules-unknown-word", 1, |work| {
        edit(&work.join("rules.yaml"), |text| {
            edit_line_in_block(text, "\n  - {id: R-2,", "\n  - {id: R-2,", |line| {
                let from = line.find("what: ").expect("R-2 の what が無い");
                let to = from + line[from..].find(',').unwrap();
                format!("{}zzzz{}", &line[..to], &line[to..])
            })
        });
    });
}

/// (13) 憲法 P-1 の plain の末尾に「型付きの表（zzzz）」を足す（「日本語（原語）」の形は免除 → 0）。
#[test]
fn parity_constitution_glossed_word_passes() {
    parity("constitution-glossed-word", 0, |work| {
        edit(&work.join("constitution.yaml"), |text| {
            edit_line_in_block(text, "\n  - id: P-1\n", "\n    plain: ", |line| {
                format!("{line}型付きの表（zzzz）")
            })
        });
    });
}

/// (14) 判断の記録 ADR-1 の retreat の condition を空にする（撤退条件の非空・便 5）。
#[test]
fn parity_adr_empty_retreat_condition_fails() {
    parity("adr-empty-retreat-condition", 1, |work| {
        edit(&work.join("adr/ADR-1.yaml"), |text| {
            edit_line_in_block(text, "\nretreat: ", "\nretreat: ", |line| {
                let at = line
                    .find("condition: ")
                    .expect("retreat の condition が無い")
                    + "condition: ".len();
                format!("{}\"\"}}", &line[..at])
            })
        });
    });
}

/// (15) ADR-1 の rejected の案 1 つを adopted にする（採用の案は 1 つ）。
#[test]
fn parity_adr_second_adopted_option_fails() {
    parity("adr-second-adopted-option", 1, |work| {
        edit(&work.join("adr/ADR-1.yaml"), |text| {
            text.replacen("verdict: rejected", "verdict: adopted", 1)
        });
    });
}

/// (16) adr/schema.yaml の options_rule の min を 3 にする（欄の決まりは床の定数の写し）。
#[test]
fn parity_adr_schema_options_min_drift_fails() {
    parity("adr-schema-options-min-drift", 1, |work| {
        edit(&work.join("adr/schema.yaml"), |text| {
            text.replacen("options_rule: {min: 2,", "options_rule: {min: 3,", 1)
        });
    });
}

/// (17) ADR-1 の approval の行を消す（accepted のまま＝発効に承認欄が無い）。
#[test]
fn parity_adr_accepted_without_approval_fails() {
    parity("adr-accepted-without-approval", 1, |work| {
        edit(&work.join("adr/ADR-1.yaml"), |text| {
            text.lines()
                .filter(|l| !l.starts_with("approval: "))
                .map(|l| format!("{l}\n"))
                .collect()
        });
    });
}

/// (18) 憲法の schema.enums.retreat_kind から ruling を外す（床の定数と食い違う・便 6）。
#[test]
fn parity_constitution_retreat_kind_drift_fails() {
    parity("constitution-retreat-kind-drift", 1, |work| {
        edit(&work.join("constitution.yaml"), |text| {
            edit_line_in_block(text, "\n  enums:\n", "\n    retreat_kind: ", |line| {
                line.replacen(", ruling]", "]", 1)
            })
        });
    });
}

/// (19) ADR-1 の basis に実在しない判断の記録の id ADR-9 を足す。
#[test]
fn parity_adr_basis_dangling_adr_id_fails() {
    parity("adr-basis-dangling-adr-id", 1, |work| {
        edit(&work.join("adr/ADR-1.yaml"), |text| {
            text.replacen("\nbasis: [", "\nbasis: [ADR-9, ", 1)
        });
    });
}

/// (20) ADR-1 の title の末尾に語彙に無い英字の語 zzzz を足す（判断の記録の本文の英字語）。
#[test]
fn parity_adr_title_unknown_word_fails() {
    parity("adr-title-unknown-word", 1, |work| {
        edit(&work.join("adr/ADR-1.yaml"), |text| {
            edit_line_in_block(text, "\ntitle: ", "\ntitle: ", |line| format!("{line}zzzz"))
        });
    });
}

/// (21) 要件書 FR1 の plain の末尾に実在しない判断の記録の id ADR-9 を足す（正本 4 file の判断の記録の参照）。
#[test]
fn parity_srs_dangling_adr_id_fails() {
    parity("srs-dangling-adr-id", 1, |work| {
        edit(&work.join("srs.yaml"), |text| {
            edit_line_in_block(text, "\n  - id: FR1\n", "\n    plain: ", |line| {
                format!("{line}ADR-9")
            })
        });
    });
}

/// (22) ADR-1 の amends に実在しない条 P-99 を対象にした項を 1 つ足す（amends の対象）。
#[test]
fn parity_adr_amends_unknown_target_fails() {
    parity("adr-amends-unknown-target", 1, |work| {
        edit(&work.join("adr/ADR-1.yaml"), |text| {
            text.replacen(
                "\namends: []\n",
                "\namends: [{target: P-99, field: title, version: v1.1, previous_text: 前, new_text: 今}]\n",
                1,
            )
        });
    });
}
