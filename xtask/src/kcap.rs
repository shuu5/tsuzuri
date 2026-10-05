//! 歯の群の行と段の file の本数の上限の照らし（task kfold-cap・判断の記録 ADR-63 の決定 (8) の (a)・ADR-32 の決定 (2) と (4)）。
//! 上限は束ねの表の file（xtask の tests/ の下の群の dir の kfold.rs）の 2 つの定数の行の字から読む（値の正本はその 2 行だけ）。
//! 群は member の tests/ の直下で main.rs を持つ dir、群の行は群の dir の直下の .rs（main.rs を除く）の行の和（畳みの道具と同じ数え）、
//! 段は member の tests/ の直下の .rs。名指した群の越えだけを落とし、ほかの群の越えと段の本数は出力に名指すだけにする。
//! crate:: の名を使わない（歯の file が #[path] で読む）。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// 引数の無い撃ちが返す使い方。
pub const USAGE: &str = "usage: cargo run -q -p xtask -- kfold-cap <群の dir（根からの相対）> ...（畳みの行が書く群と作る群）";

/// 束ねの表の file の名（xtask の tests/ の下の群の dir に 1 本）。
const KFOLD: &str = "kfold.rs";
/// 群の行の上限を持つ定数の名。
const GROUP_CAP: &str = "GROUP_LINES_CAP";
/// 段の file の本数の上限を持つ定数の名。
const STAGE_CAP: &str = "STAGE_FILES_CAP";

/// 2 つの上限。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps {
    /// 群の行の上限（越えれば落とす・ちょうどは通す）。
    pub group_lines: usize,
    /// 段の file の本数の上限（届いても落とさない）。
    pub stage_files: usize,
}

/// kfold.rs の字から 2 つの上限を読む。定数ごとに、行頭からの行 `const <名>: usize = <数字と _>;` がちょうど 1 行要る。
pub fn caps(text: &str) -> Result<Caps, String> {
    Ok(Caps {
        group_lines: cap(text, GROUP_CAP)?,
        stage_files: cap(text, STAGE_CAP)?,
    })
}

fn cap(text: &str, name: &str) -> Result<usize, String> {
    let head = format!("const {name}: usize = ");
    let values: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix(head.as_str())?.strip_suffix(';'))
        .collect();
    let [value] = values.as_slice() else {
        return Err(format!(
            "{KFOLD} に行 {head}<数>; がちょうど 1 行無い（{} 行）",
            values.len()
        ));
    };
    if !value.chars().all(|c| c.is_ascii_digit() || c == '_') {
        return Err(format!(
            "{KFOLD} の {name} の値 {value} が数字と _ だけの字でない"
        ));
    }
    value
        .replace('_', "")
        .parse()
        .map_err(|e| format!("{KFOLD} の {name} の値 {value} を読めない: {e}"))
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{} を読めない: {e}", path.display()))
}

/// 根の Cargo.toml の members の dir（書かれた順）。
pub fn members(root: &Path) -> Result<Vec<String>, String> {
    let text = read(&root.join("Cargo.toml"))?;
    let table = text
        .find("members = [")
        .and_then(|start| text.get(start..))
        .and_then(|rest| rest.get(..rest.find(']')?))
        .ok_or_else(|| "根の Cargo.toml に members の表が無い".to_string())?;
    Ok(table
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect())
}

/// dir の直下の項目の path（読めない dir は空・path の順）。
fn entries(dir: &Path) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = read.filter_map(Result::ok).map(|e| e.path()).collect();
    out.sort();
    out
}

/// dir の直下の名が .rs で終わる file。
fn rs_files(dir: &Path) -> Vec<PathBuf> {
    entries(dir)
        .into_iter()
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "rs"))
        .collect()
}

/// member の tests/ の直下で main.rs を持つ dir（群・path の順）。
pub fn groups(root: &Path, member: &str) -> Vec<PathBuf> {
    entries(&root.join(member).join("tests"))
        .into_iter()
        .filter(|p| p.join("main.rs").is_file())
        .collect()
}

/// 字の行の数（改行の数と、改行で終わらない最後の行）。
fn line_count(bytes: &[u8]) -> usize {
    let breaks = bytes.iter().filter(|&&b| b == b'\n').count();
    breaks + usize::from(bytes.last().is_some_and(|&b| b != b'\n'))
}

/// 群の行（群の dir の直下の .rs の行の和・main.rs を除く）。
fn group_lines(dir: &Path) -> Result<usize, String> {
    let mut sum = 0;
    for path in rs_files(dir) {
        if path.file_name().is_some_and(|n| n == "main.rs") {
            continue;
        }
        let bytes =
            std::fs::read(&path).map_err(|e| format!("{} を読めない: {e}", path.display()))?;
        sum += line_count(&bytes);
    }
    Ok(sum)
}

/// 根からの相対の path の字。
fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

/// 測った木（上限と、その字の file・群ごとの行・member ごとの段の本数）。
struct Tree {
    caps: Caps,
    kfold: String,
    groups: Vec<(String, usize)>,
    stages: Vec<(String, usize)>,
}

/// xtask の tests/ の下の群の dir の kfold.rs（ちょうど 1 本）。
fn kfold_file(root: &Path) -> Result<PathBuf, String> {
    let found: Vec<PathBuf> = entries(&root.join("xtask").join("tests"))
        .into_iter()
        .map(|dir| dir.join(KFOLD))
        .filter(|p| p.is_file())
        .collect();
    match found.as_slice() {
        [one] => Ok(one.clone()),
        _ => Err(format!(
            "xtask/tests/ の下の群の dir に {KFOLD} がちょうど 1 本無い（{} 本）",
            found.len()
        )),
    }
}

fn measure(root: &Path) -> Result<Tree, String> {
    let kfold = kfold_file(root)?;
    let caps = caps(&read(&kfold)?)?;
    let mut groups_seen = Vec::new();
    let mut stages = Vec::new();
    for member in members(root)? {
        for dir in groups(root, &member) {
            groups_seen.push((rel(root, &dir), group_lines(&dir)?));
        }
        let n = rs_files(&root.join(&member).join("tests")).len();
        if n > 0 {
            stages.push((format!("{member}/tests"), n));
        }
    }
    Ok(Tree {
        caps,
        kfold: rel(root, &kfold),
        groups: groups_seen,
        stages,
    })
}

/// 判じ: 名指した群が全部群なら、群ごと・段ごとの行と合計を出し、名指した群に上限を越える物が在れば rc 1、無ければ rc 0。
/// 名指した字に群でない物が在れば rc 2（その字ごとの行だけ）。
fn judge(tree: &Tree, named: &[String]) -> (i32, Vec<String>) {
    let named: BTreeSet<&str> = named.iter().map(String::as_str).collect();
    let unknown: Vec<String> = named
        .iter()
        .filter(|n| !tree.groups.iter().any(|(g, _)| g == *n))
        .map(|n| format!("{n} は群でない（member の tests/ の直下で main.rs を持つ dir を根からの相対で名指す）"))
        .collect();
    if !unknown.is_empty() {
        return (2, unknown);
    }
    let Caps {
        group_lines,
        stage_files,
    } = tree.caps;
    let mut out = vec![format!(
        "上限 群 {group_lines} 行・段 {stage_files} 本（{} の定数）",
        tree.kfold
    )];
    let mut over = 0;
    for (group, n) in &tree.groups {
        let (is_named, is_over) = (named.contains(group.as_str()), *n > group_lines);
        let mark = match (is_named, is_over) {
            (true, true) => "・名指し・越え（落とす）",
            (true, false) => "・名指し",
            (false, true) => "・越え（名指さないので落とさない）",
            (false, false) => "",
        };
        over += usize::from(is_named && is_over);
        out.push(format!("群 {group} {n} 行{mark}"));
    }
    out.extend(tree.stages.iter().map(|(s, n)| format!("段 {s} {n} 本")));
    let total: usize = tree.stages.iter().map(|(_, n)| n).sum();
    let reached = if total >= stage_files {
        "・上限に届いた"
    } else {
        ""
    };
    out.push(format!(
        "段の合計 {total} 本（上限 {stage_files}・段の本数では落とさない）{reached}"
    ));
    out.push(format!("名指した群 {} のうち越え {over}", named.len()));
    (i32::from(over > 0), out)
}

/// task kfold-cap: root（repo の根）の木を測り、named（根からの相対の群の dir）を判じて rc と出力の行を返す。
/// 名指しが無い・木を読めない・上限の行を読めない時は rc 2。
pub fn run(root: &Path, named: &[String]) -> (i32, Vec<String>) {
    if named.is_empty() {
        return (2, vec![USAGE.to_string()]);
    }
    match measure(root) {
        Ok(tree) => judge(&tree, named),
        Err(e) => (2, vec![e]),
    }
}
