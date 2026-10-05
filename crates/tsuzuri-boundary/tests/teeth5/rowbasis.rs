//! 契約表の行の欄 basis の歯（設計の道具・接頭辞 rowbasis_・設計ノート surface-wave29a 行 f-row-basis・判断の記録 ADR-44 段 2・
//! ADR-47）。行の欄 basis（行の根拠の条・規範文・規則行・判断の記録の id の列）を、設計の索引が型 basis の辺（行 → id）にし、
//! 床が参照 id の母集団で解き、導出が契約表の写しに運ぶ。欄 basis は辺の欄なので行の要約値を替えない。
//! 土台は folio2 の凍結の土台 floor_base の design-intent の写し（git の 1 commit）と、器の欄の写しに basis を足した file。
//! 否定の見本は正しい見本から 1 句だけ替える。
#![cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 行 a の流れの形の末尾（正しい見本はこの後ろに欄 basis を足す）。
const ROW_END: &str = ", depends: []}";

/// 器の欄の写しに足す欄 basis の塊。
const BASIS_FIELD: &str = "\n[[field]]\nname = \"basis\"\nneed = \"optional\"\nshape = \"list\"\n";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("写しの dir");
    for entry in fs::read_dir(src).expect("土台を読む") {
        let entry = entry.expect("土台の項");
        let to = dst.join(entry.file_name());
        if entry.file_type().expect("項の種類").is_dir() {
            copy_tree(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), &to).expect("写す");
        }
    }
}

/// git を呼ぶ（環境変数 GIT_* は継承しない）。
fn git(cwd: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    let out = cmd
        .current_dir(cwd)
        .args(["-c", "user.email=fx@example", "-c", "user.name=fx"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .expect("git を起動する");
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

/// 土台の写し（`basis` は行 a の欄 basis の流れの形の値・None なら欄を足さない・`field` が偽なら欄の写しに basis を置かない・
/// 根の欄の写しが既に basis を持っていても 1 度外してから足す）。
struct Work {
    dir: PathBuf,
}

impl Work {
    fn new(name: &str, basis: Option<&str>, field: bool) -> Work {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("rowbasis")
            .join(name);
        let _ = fs::remove_dir_all(&dir);
        copy_tree(
            &root().join("folio2/tests/fixtures/floor_base/design-intent"),
            &dir.join("design-intent"),
        );
        let schema = fs::read_to_string(root().join("contracts/field-schema/schema.toml"))
            .expect("器の欄の写しを読む");
        let schema = schema.replace(BASIS_FIELD, "");
        let schema = if field {
            format!("{schema}{BASIS_FIELD}")
        } else {
            schema
        };
        fs::create_dir_all(dir.join("contracts/field-schema")).expect("欄の写しの dir");
        fs::write(dir.join("contracts/field-schema/schema.toml"), schema).expect("欄の写し");
        let note = dir.join("design-intent/design-note/example.yaml");
        let text = fs::read_to_string(&note).expect("見本のノート");
        assert_eq!(text.matches(ROW_END).count(), 1, "行 a の末尾は 1 つ");
        let tail = basis.map_or(ROW_END.to_string(), |b| {
            format!(", depends: [], basis: {b}}}")
        });
        fs::write(&note, text.replace(ROW_END, &tail)).expect("見本のノートを書く");
        git(&dir, &["init", "-q"]);
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "fx"]);
        Work { dir }
    }

    fn tz(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tz"))
            .current_dir(self.dir.join("design-intent"))
            .args(args)
            .output()
            .expect("tz を撃つ")
    }

    /// 索引の行 a の節点の行と、行 a から出る辺の行。
    fn index_of_a(&self) -> Vec<String> {
        let out = self.tz(&["graph", "--print", "--dir", "."]);
        assert_eq!(out.status.code(), Some(0), "{out:?}");
        stdout(&out)
            .lines()
            .filter(|l| l.starts_with("example#a\t"))
            .map(str::to_string)
            .collect()
    }

    /// 床の違反の行（`[` で始まる標準出力の行）と終了 code。
    fn check(&self) -> (Option<i32>, Vec<String>) {
        let out = self.tz(&["check", "--dir", "."]);
        let lines = stdout(&out)
            .lines()
            .filter(|l| l.starts_with('['))
            .map(str::to_string)
            .collect();
        (out.status.code(), lines)
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("標準出力の字")
}

#[test]
fn rowbasis_index_makes_basis_edges_and_keeps_the_digest() {
    let with = Work::new("index-with", Some("[ADR-4, P-10]"), true);
    let got = with.index_of_a();
    assert_eq!(got.len(), 4, "{got:?}");
    assert!(got[0].starts_with("example#a\t設計ノートの行\tdesign-note/example.yaml\t"));
    assert_eq!(
        got[1..].to_vec(),
        vec![
            "example#a\tADR-4\tbasis",
            "example#a\tFR15\treq",
            "example#a\tP-10\tbasis"
        ]
    );
    let without = Work::new("index-without", None, true);
    let bare = without.index_of_a();
    assert_eq!(bare[0], got[0], "欄 basis は行の要約値を替えない");
    assert_eq!(bare[1..].to_vec(), vec!["example#a\tFR15\treq"]);
}

#[test]
fn rowbasis_floor_resolves_basis_ids() {
    let good = Work::new("floor-good", Some("[ADR-4, P-10]"), true);
    assert_eq!(good.check(), (Some(0), Vec::new()));
    let unknown = Work::new("floor-unknown", Some("[ADR-4, ADR-99]"), true);
    assert_eq!(
        unknown.check(),
        (
            Some(1),
            vec![
                "[note] design-note/example.yaml: §6 の行 a の basis[1]: id「ADR-99」が実在しない"
                    .to_string()
            ]
        )
    );
}

#[test]
fn rowbasis_floor_needs_the_field_in_the_vessel_schema() {
    let missing = Work::new("floor-no-field", Some("[ADR-4, P-10]"), false);
    assert_eq!(
        missing.check(),
        (
            Some(1),
            vec!["[note] design-note/example.yaml: §6 の行 a: 契約表の欄「basis」が器の導出 file に無い".to_string()]
        )
    );
}

#[test]
fn rowbasis_derive_carries_basis_to_the_table() {
    let work = Work::new("derive", Some("[ADR-4, P-10]"), true);
    let out = work.tz(&["derive", "--dir", ".", "--out", "../contracts", "--write"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let table = fs::read_to_string(work.dir.join("contracts/example.toml")).expect("導出の写し");
    let lines: Vec<&str> = table
        .lines()
        .filter(|l| l.starts_with("basis = "))
        .collect();
    assert_eq!(lines, vec!["basis = [\"ADR-4\", \"P-10\"]"]);
}
