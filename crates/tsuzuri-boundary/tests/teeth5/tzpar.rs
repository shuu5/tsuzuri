//! tz の口と folio の入口の一致の歯（行 k-tz-parity・受入 AC14・接頭辞 tzpar_）。
//! fixture tests/fixtures/tz/parity/cases.tsv の case を、tz の binary と folio の lib の入口 folio::entry::run に渡し、
//! 終了 code・標準出力・標準エラー・撃った後の作業場の file の表を照らす。
#![cfg(test)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common::{FILTER_WORDS, root};
use tsuzuri_boundary::cli::folio::{INDEX_FLAGS, NAMES};

/// fixture の 1 行（1 case）。
struct Case {
    mouth: String,
    name: String,
    want: u8,
    args: String,
}

/// 撃った 1 回の結果（作業場の path は字 {T} に戻した後）。
struct Shot {
    rc: i32,
    out: String,
    err: String,
    files: BTreeMap<String, Vec<u8>>,
}

/// 持ち込んだ folio の置き場 folio2。
fn folio_home() -> String {
    root().join("folio2").display().to_string()
}

fn fixture_path() -> PathBuf {
    root().join("tests/fixtures/tz/parity/cases.tsv")
}

/// fixture を読む（井桁で始まる行と空の行は飛ばす・4 列でない行はここで落とす）。
fn cases() -> Vec<Case> {
    let text = fs::read_to_string(fixture_path()).expect("cases.tsv を読む");
    text.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let cols: Vec<&str> = l.split('\t').collect();
            assert_eq!(cols.len(), 4, "4 列でない行: {l}");
            Case {
                mouth: cols[0].to_string(),
                name: cols[1].to_string(),
                want: cols[2].parse().unwrap_or_else(|_| panic!("期待の code: {l}")),
                args: cols[3].to_string(),
            }
        })
        .collect()
}

/// 撃つ側ごとの空の作業場を作る。
fn workspace(group: &str, case: &Case, side: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("tzpar-{}", std::process::id()))
        .join(format!("{group}-{}-{}-{side}", case.mouth, case.name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("作業場を作る");
    dir
}

/// 引数の字の {F} と {T} を替えて空白で割る。
fn argv(case: &Case, work: &Path) -> Vec<String> {
    case.args
        .replace("{F}", &folio_home())
        .replace("{T}", &work.display().to_string())
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// 作業場の下の file の相対の path と byte の表。
fn table(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).expect("作業場を読む") {
            let path = entry.expect("作業場の項").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path.strip_prefix(dir).expect("作業場の下").display().to_string();
                files.insert(rel, fs::read(&path).expect("file を読む"));
            }
        }
    }
    files
}

/// 作業場の path を字 {T} に戻す。
fn untemp(text: &str, work: &Path) -> String {
    text.replace(&work.display().to_string(), "{T}")
}

/// tz の binary を撃つ。usage の行の命令の名 tz は folio に替える。
fn shoot_tz(group: &str, case: &Case) -> Shot {
    let work = workspace(group, case, "tz");
    let out = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(argv(case, &work))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("tz を撃つ");
    let fix = |bytes: Vec<u8>| {
        untemp(&String::from_utf8(bytes).expect("UTF-8"), &work).replace("Usage: tz ", "Usage: folio ")
    };
    Shot {
        rc: out.status.code().expect("終了 code"),
        out: fix(out.stdout),
        err: fix(out.stderr),
        files: table(&work),
    }
}

/// folio の lib の入口を撃つ（頭の命令の名は folio・書き先は Vec）。
fn shoot_entry(group: &str, case: &Case) -> Shot {
    let work = workspace(group, case, "entry");
    let args = std::iter::once("folio".to_string()).chain(argv(case, &work));
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let rc = folio::entry::run(args, &mut out, &mut err);
    let fix = |bytes: Vec<u8>| untemp(&String::from_utf8(bytes).expect("UTF-8"), &work);
    Shot {
        rc: i32::from(rc),
        out: fix(out),
        err: fix(err),
        files: table(&work),
    }
}

/// 口が `mouths` のどれかの case を全部、tz と入口に撃って照らす。
fn same_as_entry(group: &str, mouths: &[&str]) {
    let picked: Vec<Case> = cases()
        .into_iter()
        .filter(|c| mouths.contains(&c.mouth.as_str()))
        .collect();
    assert!(!picked.is_empty(), "{group}: case が 1 つも無い");
    for case in &picked {
        let label = format!("{} {}", case.mouth, case.name);
        let want = shoot_entry(group, case);
        let got = shoot_tz(group, case);
        assert_eq!(want.rc, i32::from(case.want), "{label}: 入口の code\n{}", want.err);
        assert_eq!(got.rc, want.rc, "{label}: tz の code\n{}", got.err);
        assert_eq!(got.out, want.out, "{label}: 標準出力");
        assert_eq!(got.err, want.err, "{label}: 標準エラー");
        assert_eq!(got.files, want.files, "{label}: 作業場の file");
    }
    let _ = fs::remove_dir_all(
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tzpar-{}", std::process::id())),
    );
}

#[test]
fn tzpar_cases_cover_every_mouth() {
    let text = fs::read_to_string(fixture_path()).expect("cases.tsv を読む");
    let rows: Vec<Vec<&str>> = text
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split('\t').collect())
        .collect();
    assert!(!rows.is_empty(), "case が無い");
    let mut seen = Vec::new();
    for row in &rows {
        assert_eq!(row.len(), 4, "4 列でない行: {row:?}");
        assert!(row[2].parse::<u8>().is_ok(), "期待の code: {row:?}");
        assert_eq!(row[3].split_whitespace().next(), Some(row[0]), "引数の頭の語が口の名でない: {row:?}");
        assert!(!seen.contains(&(row[0], row[1])), "口と case の名の組が重なる: {row:?}");
        seen.push((row[0], row[1]));
    }
    let mouths: Vec<&str> = rows.iter().map(|r| r[0]).collect();
    for name in NAMES.iter().chain(["graph"].iter()) {
        assert!(mouths.contains(name), "{name} の case が無い");
    }
    for flag in INDEX_FLAGS {
        assert!(
            rows.iter().any(|r| r[0] == "graph" && r[3].split_whitespace().any(|a| a == flag)),
            "graph の case に {flag} が無い"
        );
    }
    for code in ["0", "1", "2"] {
        assert!(rows.iter().any(|r| r[2] == code), "期待の code {code} の case が無い");
    }
}

#[test]
fn tzpar_floor_mouths_match() {
    same_as_entry("floor", &["check", "schema", "derive", "ceiling"]);
}

#[test]
fn tzpar_face_mouths_match() {
    same_as_entry("face", &["parts", "face", "figure", "build"]);
}

#[test]
fn tzpar_start_mouths_match() {
    same_as_entry("start", &["init", "intake", "hello"]);
}

#[test]
fn tzpar_index_views_match() {
    same_as_entry("index", &["graph"]);
}

#[test]
fn tzpar_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 325, "filter の語の数");
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth5/tzpar.rs"))
        .expect("tests/teeth5/tzpar.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 6, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("tzpar_")
            .unwrap_or_else(|| panic!("{name} は tzpar_ で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
