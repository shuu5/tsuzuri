//! `folio render` の歯（便 11・docs/design/delivery-11.md §1 (c)）。binary 経由。
//! 入力は parity の歯と同じ作り（`design-intent/` を丸ごと一時 dir へ写す・preview/ 込み・git は要らない）。
//! 版管理の `design-intent/preview/readable.html` は書き換えない（`--out` は必ず一時 file か写しの中）。
//! - 凍結 anchor との一致（P-10.1）: 写しに --write → 版管理の readable.html と byte 一致
//! - check の 3 値: 一致 0 ／ 不一致 1（DRIFT）／ 無い 2 → --write 0 → --check 0
//! - script との突き合わせ（oracle・P-10.2 により唯一の判定にしない）: 変異 4 つで script と folio の出力が byte 一致し、変異前と違う
//! - 導出できない入力 4 つ: 2「まだ分からない」で出力先に 1 byte も書かない

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

/// 写し 1 つ（一時 dir の下の design-intent）。戻り値 = (一時 dir, 写しの design-intent)。
fn fresh_copy(case: &str) -> (PathBuf, PathBuf) {
    let td = std::env::temp_dir().join(format!("folio-render-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    let work = td.join("design-intent");
    copy_tree(&repo_root().join("design-intent"), &work);
    (td, work)
}

fn folio_render(dir: &Path, out: Option<&Path>, mode: &str) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("render").arg("--dir").arg(dir);
    if let Some(out) = out {
        cmd.arg("--out").arg(out);
    }
    cmd.arg(mode).output().expect("folio を起動できない")
}

/// day-1 の script（oracle）。--dir も --out も絶対 path で渡す（script は --dir へ移ってから開く）。
fn script_render(dir: &Path, out: &Path) -> Output {
    assert!(dir.is_absolute() && out.is_absolute());
    Command::new("python3")
        .arg(repo_root().join("scripts/render_preview.py"))
        .arg("--dir")
        .arg(dir)
        .arg("--out")
        .arg(out)
        .arg("--write")
        .output()
        .expect("python3 を起動できない（突き合わせの歯は飛ばさない）")
}

fn code(out: &Output, what: &str) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "{what} が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn frozen_readable() -> Vec<u8> {
    fs::read(repo_root().join("design-intent/preview/readable.html")).unwrap()
}

fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない: {}", path.display());
    fs::write(path, after).unwrap();
}

/// 凍結 anchor との一致（P-10.1）: 写しに --write → 終了 0 ∧ 出力が版管理の readable.html と byte 一致。
#[test]
fn render_write_matches_frozen_readable() {
    let (td, work) = fresh_copy("anchor");
    let out = td.join("out.html");
    let run = folio_render(&work, Some(&out), "--write");
    let written = fs::read(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio render --write"), 0, "{}", stderr(&run));
    assert_eq!(
        stdout(&run),
        format!("rendered bytes {}\n", written.len()),
        "{}",
        stderr(&run)
    );
    let frozen = frozen_readable();
    if written != frozen {
        let at = written
            .iter()
            .zip(&frozen)
            .position(|(a, b)| a != b)
            .unwrap_or(written.len().min(frozen.len()));
        let show = |b: &[u8]| {
            String::from_utf8_lossy(&b[at.saturating_sub(120)..(at + 200).min(b.len())])
                .into_owned()
        };
        panic!(
            "readable.html と違う（{} byte ≠ {} byte・最初の差 {at} byte 目）\n--- folio\n{}\n--- 凍結\n{}",
            written.len(),
            frozen.len(),
            show(&written),
            show(&frozen)
        );
    }
}

/// check の 3 値: 一致 0 ／ title の 1 字を変えて 1（DRIFT）／ 読み物を消して 2 ／ --write 0 → --check 0。
#[test]
fn render_check_has_three_values() {
    let (td, work) = fresh_copy("check");
    let readable = work.join("preview/readable.html");

    let ok = folio_render(&work, None, "--check");
    edit(&work.join("srs.yaml"), |text| {
        text.replacen("  title: 要件書（M0", "  title: 要件集（M0", 1)
    });
    let drift = folio_render(&work, None, "--check");
    fs::remove_file(&readable).unwrap();
    let missing = folio_render(&work, None, "--check");
    let write = folio_render(&work, None, "--write");
    let after = folio_render(&work, None, "--check");
    let regenerated = fs::read(&readable).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&ok, "check（一致）"), 0, "{}", stderr(&ok));
    assert!(stdout(&ok).contains("render: OK"), "{}", stdout(&ok));
    assert_eq!(code(&drift, "check（不一致）"), 1, "{}", stderr(&drift));
    assert!(stderr(&drift).contains("DRIFT"), "{}", stderr(&drift));
    assert_eq!(code(&missing, "check（無い）"), 2, "{}", stderr(&missing));
    assert!(
        stderr(&missing).contains("読み物が無い"),
        "{}",
        stderr(&missing)
    );
    assert_eq!(code(&write, "write"), 0, "{}", stderr(&write));
    assert_eq!(code(&after, "check（再生成後）"), 0, "{}", stderr(&after));
    assert!(!regenerated.is_empty());
    assert_ne!(
        regenerated,
        frozen_readable(),
        "title の変異が出力に出ていない"
    );
}

/// --write と --check の両方無し・両方有りは clap の使い方の誤り（終了 2）。
#[test]
fn render_mode_is_exactly_one() {
    let dir = repo_root().join("design-intent");
    for args in [&["--check", "--write"][..], &[][..]] {
        let out = Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("render")
            .arg("--dir")
            .arg(&dir)
            .arg("--out")
            .arg(std::env::temp_dir().join("folio-render-never-written.html"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
    }
}

/// script との突き合わせ（oracle）: 写しに変異を 1 つ当て、script と folio の出力が byte 一致し、変異前の出力（凍結 anchor）と違う。
fn oracle(case: &str, mutate: impl FnOnce(&Path)) {
    let (td, work) = fresh_copy(&format!("oracle-{case}"));
    mutate(&work);
    let a = td.join("script.html");
    let b = td.join("folio.html");
    let script = script_render(&work, &a);
    let folio = folio_render(&work, Some(&b), "--write");
    let script_out = fs::read(&a).unwrap_or_default();
    let folio_out = fs::read(&b).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(
        code(&script, "script"),
        0,
        "{case}: script\n{}",
        stderr(&script)
    );
    assert_eq!(
        code(&folio, "folio"),
        0,
        "{case}: folio\n{}",
        stderr(&folio)
    );
    assert_eq!(stdout(&script), stdout(&folio), "{case}: 標準出力");
    assert_ne!(
        script_out,
        frozen_readable(),
        "{case}: 変異が効いていない（変異前の出力と同じ）"
    );
    if script_out != folio_out {
        let at = script_out
            .iter()
            .zip(&folio_out)
            .position(|(x, y)| x != y)
            .unwrap_or(script_out.len().min(folio_out.len()));
        let show = |b: &[u8]| {
            String::from_utf8_lossy(&b[at.saturating_sub(120)..(at + 200).min(b.len())])
                .into_owned()
        };
        panic!(
            "{case}: script と folio の出力が違う（{} byte ≠ {} byte・最初の差 {at} byte 目）\n--- script\n{}\n--- folio\n{}",
            script_out.len(),
            folio_out.len(),
            show(&script_out),
            show(&folio_out)
        );
    }
}

/// (1) 判断の記録 1 本（ADR-1）に amends の 1 要素（version・target・field・previous_text・new_text）を置く。
#[test]
fn render_oracle_adr_amends_entry() {
    oracle("adr-amends", |work| {
        edit(&work.join("adr/ADR-1.yaml"), |text| {
            text.replacen(
                "\namends: []\n",
                "\namends: [{target: P-1, field: title, version: v1.1, previous_text: 判断する道具を作らない, new_text: \"判断する道具を作らない（改訂）<b>\"}]\n",
                1,
            )
        });
    });
}

/// (2) `anchors/index.yaml` を消す（凍結 anchor の列 = 「なし」の文）。
#[test]
fn render_oracle_anchor_index_removed() {
    oracle("anchor-index-removed", |work| {
        fs::remove_file(work.join("anchors/index.yaml")).unwrap();
    });
}

/// (3) srs.yaml の meta の status_note の値を変える（正本 v1.2 は status_note を既に持つ＝足すと重複キーで 2 になるので足さない）。
/// status_note が出る行の字面が変わり、変異前の出力と違うことは oracle が確かめる（escape の要る字を含める）。
#[test]
fn render_oracle_srs_status_note_value_changed() {
    oracle("srs-status-note", |work| {
        edit(&work.join("srs.yaml"), |text| {
            let at = text
                .find("\n  status_note: ")
                .expect("meta の status_note が無い")
                + 1;
            let end = at + text[at..].find('\n').unwrap();
            format!(
                "{}  status_note: \"変異の注（v1.2 の後） & <x>\"{}",
                &text[..at],
                &text[end..]
            )
        });
    });
}

/// (4) 判断の記録 1 本（ADR-1）に superseded_by を足す。
#[test]
fn render_oracle_adr_superseded_by() {
    oracle("adr-superseded-by", |work| {
        edit(&work.join("adr/ADR-1.yaml"), |text| {
            format!("{text}superseded_by: ADR-2\n")
        });
    });
}

/// 導出できない入力: `--out <一時 file> --write` = 2 ∧ 標準エラーに「まだ分からない」∧ 一時 file が出来ていない。
fn unknown(case: &str, mutate: impl FnOnce(&Path)) {
    let (td, work) = fresh_copy(&format!("unknown-{case}"));
    mutate(&work);
    let out = td.join("never.html");
    let run = folio_render(&work, Some(&out), "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio render"), 2, "{case}: {}", stderr(&run));
    assert!(
        stderr(&run).contains("まだ分からない"),
        "{case}: {}",
        stderr(&run)
    );
    assert!(!exists, "{case}: 導出できないのに出力先に書いた");
}

/// (1) rules.yaml に重複キーを足す（top level の節 discipline をもう 1 度書く = 同じ表に同じキーを 2 度）。
#[test]
fn render_unknown_rules_duplicate_key() {
    unknown("rules-duplicate-key", |work| {
        edit(&work.join("rules.yaml"), |text| {
            format!("{text}\ndiscipline: []\n")
        });
    });
}

/// (2) constitution.yaml の north_star を消す。
#[test]
fn render_unknown_north_star_removed() {
    unknown("north-star-removed", |work| {
        edit(&work.join("constitution.yaml"), |text| {
            let at = text.find("\nnorth_star:\n").expect("north_star が無い") + 1;
            let end = at + text[at..].find("\n\n").unwrap() + 1;
            format!("{}{}", &text[..at], &text[end..])
        });
    });
}

/// (3) 条 P-1 の tier を表に無い値にする。
#[test]
fn render_unknown_article_tier_out_of_table() {
    unknown("tier-out-of-table", |work| {
        edit(&work.join("constitution.yaml"), |text| {
            text.replacen(
                "  - id: P-1\n    title: 判断する道具を作らない\n    tier: always\n",
                "  - id: P-1\n    title: 判断する道具を作らない\n    tier: sometimes\n",
                1,
            )
        });
    });
}

/// (4) 要件書 meta の changes_from_v1_1 の表の値に逆斜線を足す（repr の狭い写しが断る形）。
#[test]
fn render_unknown_changes_from_backslash() {
    unknown("changes-backslash", |work| {
        edit(&work.join("srs.yaml"), |text| {
            text.replacen(
                "    - CON1（順序）を改訂した: 「実装本体は",
                "    - CON1（順序）を改訂した: \\「実装本体は",
                1,
            )
        });
    });
}
