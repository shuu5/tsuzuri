//! 歯の写し（insta の snapshot）の孤児の照らし（task insta-refs・行 v-ci・判断の記録 ADR-33 の決定 (5) と (14)）。
//! daily の段（行 t-daily-deny）では、check が入れ子の段の歯が読んだ snapshot の file の path を、環境変数 INSTA_SNAPSHOT_REFERENCES_FILE が
//! 名指す file に 1 行ずつ残し（insta 1.48.0 の memoize_snapshot_file）、段 insta-refs が役ごとの file を 1 つの dir から読む。この module が
//! 和を取って、追跡される scribe2/ の下の .snap のどれかが読まれていないか、追跡される .snap.new か .pending-snap が在れば落とす
//! （器の CI の job insta の --unreferenced reject と --check の残り・写しの食い違いは入れ子の段の nextest が落とす）。
//! crate:: の名を使わない（歯の file が #[path] で読む）。

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// 照らす持ち込んだ木（repo の根からの相対・字 / で終わる）。
pub const TREE: &str = "scribe2/";
/// 写しの file の名の終わり。
pub const SNAP: &str = ".snap";
/// 受け入れていない写しの名の終わり（追跡されていれば落とす）。
pub const PENDING: [&str; 2] = [".snap.new", ".pending-snap"];
/// insta が読んだ写しの path を足す file を名指す環境変数（daily の段では check が file に残し、段 insta-refs が照らす・行 t-daily-deny）。
pub const REFS_ENV: &str = "INSTA_SNAPSHOT_REFERENCES_FILE";

/// path の字の節 . を除き、節 .. を 1 つ前の節と打ち消す（insta は snapshot の置き場の設定の字 ../ をそのまま書く）。
/// 打ち消しが字 / の頭を越えると字 / で始まらない字になり、根の下とは読まれない。
pub fn normal(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

/// 読んだ path の行の列を、節 . と .. を解いた上で root（repo の根の絶対の path）からの相対の path の集合にする。
/// 空の行は飛ばし、root の下でない行が 1 つでも在れば Err。
pub fn referenced(root: &str, lines: &[String]) -> Result<BTreeSet<String>, String> {
    let head = format!("{}/", root.trim_end_matches('/'));
    let mut out = BTreeSet::new();
    for line in lines.iter().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        let Some(rel) = normal(line).strip_prefix(&head).map(str::to_string) else {
            return Err(format!("読んだ path {line} が根 {root} の下に無い"));
        };
        out.insert(rel);
    }
    Ok(out)
}

/// 判じ: 追跡される file（repo の根からの相対）のうち TREE の下で、名が SNAP で終わる file が全部 refs に在り、
/// 名が PENDING のどれかで終わる file が無ければ Ok（事実の 1 行）。そうでなければ Err（落ちた file ごとの行）。
/// TREE の下の写しが 0 本なら測れていないとして Err。
pub fn judge(tracked: &[String], refs: &BTreeSet<String>) -> Result<String, Vec<String>> {
    let mut bad = Vec::new();
    let mut snaps = 0;
    for path in tracked.iter().filter(|p| p.starts_with(TREE)) {
        if PENDING.iter().any(|end| path.ends_with(end)) {
            bad.push(format!("受け入れていない写しが追跡されている: {path}"));
        } else if path.ends_with(SNAP) {
            snaps += 1;
            if !refs.contains(path) {
                bad.push(format!("どの歯も読まない写し: {path}"));
            }
        }
    }
    if snaps == 0 {
        bad.push(format!("{TREE} の下に追跡される写しが無い"));
    }
    if bad.is_empty() {
        Ok(format!(
            "写し {snaps} 本・読まれた path {} 本・読まれない写し 0",
            refs.len()
        ))
    } else {
        Err(bad)
    }
}

/// dir の下の file の字を名の順に全部読み、行の列にする（下の dir も辿る）。
fn read_lines(dir: &Path, out: &mut Vec<String>) -> Result<usize, String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("{} を読めない: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    entries.sort();
    let mut files = 0;
    for path in entries {
        if path.is_dir() {
            files += read_lines(&path, out)?;
        } else {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("{} を読めない: {e}", path.display()))?;
            out.extend(text.lines().map(str::to_string));
            files += 1;
        }
    }
    Ok(files)
}

/// root の追跡される file（git ls-files・TREE の下だけ）。
fn tracked(root: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z", "--", TREE])
        .output()
        .map_err(|e| format!("git を起こせない: {e}"))?;
    if !out.status.success() {
        return Err(format!("git ls-files が rc {:?}", out.status.code()));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect())
}

/// task insta-refs: dir の下の役ごとの file の和を root の追跡される写しと照らし、(rc, 出す行) を返す。
/// rc は合格 0・落ち 1。dir に file が無い・読めない・根の下でない行が在る時も 1（測れていないを合格にしない）。
pub fn run(root: &Path, dir: &Path) -> (i32, Vec<String>) {
    let mut lines = Vec::new();
    let files = match read_lines(dir, &mut lines) {
        Ok(0) => {
            return (
                1,
                vec![format!(
                    "{} の下に {REFS_ENV} の file が無い",
                    dir.display()
                )],
            );
        }
        Ok(n) => n,
        Err(e) => return (1, vec![e]),
    };
    let refs = match referenced(&root.to_string_lossy(), &lines) {
        Ok(refs) => refs,
        Err(e) => return (1, vec![e]),
    };
    let tracked = match tracked(root) {
        Ok(t) => t,
        Err(e) => return (1, vec![e]),
    };
    match judge(&tracked, &refs) {
        Ok(fact) => (0, vec![format!("役の file {files} 本・{fact}")]),
        Err(bad) => (1, bad),
    }
}
