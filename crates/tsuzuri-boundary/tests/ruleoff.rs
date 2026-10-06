//! 接頭辞 ruleoff_・設計ノートを退ける行 t-derive-retire の歯（判断の記録 ADR-74 決定 (5)）。
//! 根の design-intent/ の写しで規則の表の欄 status の 廃止 が読む口を閉じる（tz の binary を撃つ・歯ごとに写しを作る）。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 退ける 5 行（章の上限・計画のノートの名札・生きたノートの本数・1 本の契約表の行・計画だけの行）。
const ROWS: [&str; 5] = ["R-26", "R-27", "R-31", "R-32", "R-33"];

/// 歯ごとの写し（前の撃ちの残りを消して作る）。根の design-intent/ の木と契約表の欄の決まりを写す。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ruleoff").join(case);
        let _ = fs::remove_dir_all(&root);
        copy_tree(&repo.join("design-intent"), &root.join("design-intent"));
        let schema = root.join("contracts/field-schema");
        fs::create_dir_all(&schema).unwrap();
        fs::copy(repo.join("contracts/field-schema/schema.toml"), schema.join("schema.toml")).unwrap();
        Work { root }
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    /// 写しの file の字を直す（今の値を問わずに値を置く）。
    fn edit(&self, rel: &str, f: impl Fn(&str) -> String) {
        let path = self.dir().join(rel);
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, f(&text)).unwrap();
    }

    /// 規則の表の 5 行の状態を `status` に置く。
    fn rule_status(&self, status: &str) {
        self.edit("rules.yaml", |t| set_rows_status(t, status));
    }

    /// surface-plan を退ける（meta.status を retired・meta.superseded_by を ADR-74 に）。
    fn retire_plan(&self) {
        self.edit("design-note/surface-plan.yaml", retire_meta);
    }

    /// 生成区間を書き（tz schema --write）、置き場を git の commit に固め、tz check を撃つ。
    fn check(&self) -> Output {
        let dir = self.dir();
        let dir = dir.to_str().unwrap();
        let wrote = tz(&["schema", "--dir", dir, "--write"]);
        assert!(wrote.status.success(), "tz schema --write: {}", said(&wrote));
        git(&self.root, &["init", "-q"]);
        git(&self.root, &["add", "-A"]);
        git(&self.root, &["commit", "-q", "-m", "fixture"]);
        tz(&["check", "--dir", dir])
    }
}

fn tz(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tz")).args(args).output().expect("tz を起動できない")
}

/// 環境の GIT_ を外して git を撃つ（commit が無いと床は 版管理（git）が無いか読めない を まだ分からない で出す）。
fn git(cwd: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    let out = cmd
        .current_dir(cwd)
        .args(["-c", "user.email=fx@example", "-c", "user.name=fx", "-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .expect("git を起動できない");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
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

/// 標準出力と標準エラーを 1 本の字にする。
fn said(out: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

/// 規則の表の行（行頭が 2 つの空白と `- {id: R-<n>,`）のうち ROWS の 5 行の欄 status の字を置き替える。
fn set_rows_status(text: &str, status: &str) -> String {
    const OPEN: &str = "status: \"";
    let mut hit = 0;
    let out: String = text
        .split_inclusive('\n')
        .map(|line| {
            let is_row = ROWS.iter().any(|id| line.starts_with(&format!("  - {{id: {id},")));
            let Some(at) = line.find(OPEN).filter(|_| is_row) else {
                return line.to_string();
            };
            let from = at + OPEN.len();
            let len = line[from..].find('"').expect("欄 status の二重引用符が閉じていない");
            hit += 1;
            format!("{}{status}{}", &line[..from], &line[from + len..])
        })
        .collect();
    assert_eq!(hit, ROWS.len(), "欄 status を置いた行の数");
    out
}

/// meta の直下の行頭 2 つの空白の status の行を retired に置き、同じ meta の superseded_by の行を ADR-74 に置く（無ければ
/// status の行の次に足す）。
fn retire_meta(text: &str) -> String {
    let mut in_meta = false;
    let mut have_status = false;
    let mut have_by = false;
    let mut lines: Vec<String> = Vec::new();
    for line in text.split_inclusive('\n') {
        if line == "meta:\n" {
            in_meta = true;
        } else if !line.starts_with("  ") {
            in_meta = false;
        }
        if in_meta && line.starts_with("  status:") && !have_status {
            have_status = true;
            lines.push("  status: retired\n".to_string());
        } else if in_meta && line.starts_with("  superseded_by:") {
            have_by = true;
            lines.push("  superseded_by: ADR-74\n".to_string());
        } else {
            lines.push(line.to_string());
        }
    }
    assert!(have_status, "meta の status の行が無い");
    if !have_by {
        let at = lines.iter().position(|l| l == "  status: retired\n").unwrap();
        lines.insert(at + 1, "  superseded_by: ADR-74\n".to_string());
    }
    lines.concat()
}

/// 行 R-31 の行の中の字 `, key: live-notes` を消す（行と id は残す）。
fn drop_live_notes_key(text: &str) -> String {
    let row = text.split_inclusive('\n').find(|l| l.starts_with("  - {id: R-31,")).expect("行 R-31 が無い");
    assert_eq!(row.matches(", key: live-notes").count(), 1);
    text.replacen(row, &row.replacen(", key: live-notes", "", 1), 1)
}

#[test]
fn ruleoff_real_place_after_retire() {
    let w = Work::new("ruleoff_real_place_after_retire");
    w.retire_plan();
    w.rule_status("廃止");
    let out = w.check();
    let text = said(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(text.contains("合格（違反 0・まだ分からない 0）"), "{text}");
}

#[test]
fn ruleoff_live_rows_still_count() {
    let w = Work::new("ruleoff_live_rows_still_count");
    w.retire_plan();
    w.rule_status("仮");
    let out = w.check();
    let text = said(&out);
    assert_eq!(out.status.code(), Some(2), "{text}");
    assert!(text.contains("計画のノートの状態が effective でない"), "{text}");

    let w = Work::new("ruleoff_live_rows_still_count_key");
    w.edit("rules.yaml", drop_live_notes_key);
    let out = w.check();
    let text = said(&out);
    assert_eq!(out.status.code(), Some(2), "{text}");
    assert!(text.contains("欄 key が live-notes の閾値の行が無い"), "{text}");
}
