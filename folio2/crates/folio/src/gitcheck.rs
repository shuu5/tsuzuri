//! `folio check` の凍結 anchor の列の版管理（git）との照合（便 8 (a)・docs/design/delivery-8.md §1）。
//! day-1 の床 `scripts/check_draft.py` の anchor の節の版管理の照合を同じ手順で写す。git は外部 crate でなく
//! 命令 `git` を子 process で撃つ（環境変数 GIT_* は渡さない・各命令の待ち上限 20 秒）。
//! 見えない（git が無い・commit が無い・浅い写し・読めない）は「無い」でなく「まだ分からない」（測れない）。
//! 列の真偽は索引と digest が受け持ち、版管理は「消された・差し替えられた anchor」を早く止める補助。

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::adr;
use crate::anchor;
use crate::verdict::Report;
use crate::yaml::{self, Value};

/// 各命令の待ち上限。
const TIMEOUT: Duration = Duration::from_secs(20);

const NO_GIT: &str = "版管理（git）が無いか読めない＝anchor の削除を版管理と照合できない（まだ分からない）。写しで回すときも git init + commit の中で回す";

fn floor(path: &[&str]) -> &'static str {
    adr::floor_val(path).unwrap_or_default()
}

/// 命令 1 本の結果（終了コードと標準出力）。
struct Out {
    code: Option<i32>,
    stdout: Vec<u8>,
}

impl Out {
    fn ok(&self) -> bool {
        self.code == Some(0)
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
}

/// `git -C <cwd> <args>` を撃つ。起動できない・待ち上限を超えたは None。
fn git<S: AsRef<OsStr>>(cwd: &Path, args: &[S]) -> Option<Out> {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    let mut child = cmd
        .arg("-C")
        .arg(cwd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut pipe = child.stdout.take()?;
    let reader = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        buf
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if start.elapsed() < TIMEOUT => thread::sleep(Duration::from_millis(2)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return None;
            }
        }
    };
    let stdout = reader.join().ok()?;
    Some(Out {
        code: status.code(),
        stdout,
    })
}

/// 版管理と照合できた anchors/ の集合（未追跡の検査は列の最新の版が決まってから回す）。
pub(crate) struct Tracked {
    tracked: BTreeSet<String>,
    ever: BTreeSet<String>,
    present: BTreeSet<String>,
}

impl Tracked {
    /// 版管理の HEAD か履歴に anchors/ の file が在った（便 9 の凍結が列の始め直しを断るのに読む）。
    pub(crate) fn seen(&self) -> bool {
        !self.tracked.is_empty() || !self.ever.is_empty()
    }

    /// 作業ツリーに在って HEAD に無い anchor file（索引を除く）のうち最新の版の file でないもの。
    pub(crate) fn untracked(&self, newest: Option<&str>, report: &mut Report) {
        let index_file = floor(&["anchor", "index_file"]);
        let newest_name = newest.map(|v| floor(&["anchor", "file_name"]).replace("<version>", v));
        for name in self
            .present
            .iter()
            .filter(|n| !self.tracked.contains(*n) && *n != index_file)
        {
            if newest_name.as_deref() != Some(name.as_str()) {
                report.violation(
                    "anchor",
                    format!(
                        "anchors/{name} が版管理に追跡されていない（凍結した anchor は commit する。最新の版 {} 以外の未追跡の anchor は列の差し替え）",
                        newest.unwrap_or("None")
                    ),
                );
            }
        }
    }
}

/// path の file 名（無ければ path そのもの）。
fn base_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map_or_else(|| path.to_string(), |n| n.to_string_lossy().into_owned())
}

fn is_commit_line(line: &str) -> bool {
    line.len() == 40 && line.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// `log --name-status` の行（状態の字・数字・タブ・path・〔タブ・path〕）を (状態, 右の path) に割る。
fn status_line(line: &str) -> Option<(char, &str)> {
    let mut chars = line.chars();
    let st = chars.next().filter(char::is_ascii_uppercase)?;
    let rest = chars
        .as_str()
        .trim_start_matches(|c: char| c.is_ascii_digit());
    let rest = rest.strip_prefix('\t')?;
    // 床の `(.+?)(?:\t(.+))?`: 1 字目より後の最初のタブの右が空でなければ右の path
    let path = match rest.char_indices().skip(1).find(|(_, c)| *c == '\t') {
        Some((i, _)) if i + 1 < rest.len() => &rest[i + 1..],
        _ => rest,
    };
    (!path.is_empty()).then_some((st, path))
}

/// 履歴の anchor が床の今の形式（固定の欄・digest の方式・写しの取り方）と同じか。
fn format_same(doc: &Value) -> bool {
    let Some(map) = doc.as_map() else {
        return false;
    };
    let keys: BTreeSet<String> = map.iter().map(|(k, _)| k.py_str()).collect();
    let want: BTreeSet<String> = adr::floor_strs(&["anchor", "file_keys"])
        .iter()
        .map(|k| k.to_string())
        .collect();
    if keys != want
        || doc.get("digest_algo").and_then(Value::as_str) != Some(floor(&["anchor", "digest_algo"]))
    {
        return false;
    }
    let empty = Value::Map(Vec::new());
    let pj = match doc.get("projection") {
        Some(p @ Value::Map(_)) => p,
        _ => &empty,
    };
    anchor::str_list_eq(
        pj.get("article_fields"),
        adr::floor_strs(&["anchor", "projection_article_fields"]),
    ) && anchor::str_list_eq(
        pj.get("statement_fields"),
        adr::floor_strs(&["anchor", "statement_fields"]),
    )
}

/// 本文を床の読み手と同じく型付きで読む（重複キー・読めない本文は None）。
fn read_blob(bytes: &[u8]) -> Option<Value> {
    let text = std::str::from_utf8(bytes).ok()?;
    let doc = yaml::parse(text).ok()?;
    if !doc.duplicates.is_empty() {
        return None;
    }
    yaml::parse_typed(text).ok()
}

/// (a) 版管理との照合。照合を終えられたら未追跡の検査の材料を返す。
pub(crate) fn check_git(dir: &Path, report: &mut Report) -> Option<Tracked> {
    let Ok(here) = fs::canonicalize(dir) else {
        report.pending(NO_GIT);
        return None;
    };
    let anch = here.join(floor(&["anchor", "dir"]));
    let present: BTreeSet<String> = if anch.is_dir() {
        fs::read_dir(&anch)
            .map(|rd| {
                rd.filter_map(Result::ok)
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .filter(|n| n.ends_with(".yaml"))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        BTreeSet::new()
    };
    let top = match git(&here, &["rev-parse", "--show-toplevel"]) {
        Some(o) if o.ok() => o.text(),
        _ => {
            report.pending(NO_GIT);
            return None;
        }
    };
    let Ok(top) = fs::canonicalize(PathBuf::from(top.trim_end_matches(['\n', '\r']))) else {
        report.pending(NO_GIT);
        return None;
    };
    if top == here {
        report.violation(
            "anchor",
            format!(
                "design-intent 自体が版管理の根（{}）＝床の script と別の版管理では照合にならない",
                top.display()
            ),
        );
        return None;
    }
    if !here.starts_with(&top) {
        report.violation(
            "anchor",
            format!(
                "版管理の根 {} が design-intent の上に無い（照合先が違う）",
                top.display()
            ),
        );
        return None;
    }
    match git(&top, &["rev-parse", "--verify", "-q", "HEAD"]) {
        Some(o) if o.ok() => {}
        Some(_) => {
            report.pending("版管理に commit が 1 つも無い（HEAD 無し）＝anchor を版管理と照合できない（まだ分からない）。git init だけでなく commit してから回す");
            return None;
        }
        None => {
            report.pending(NO_GIT);
            return None;
        }
    }
    let Some(shallow) = git(&top, &["rev-parse", "--is-shallow-repository"]) else {
        report.pending(NO_GIT);
        return None;
    };
    if shallow.text().trim() == "true" && present.is_empty() {
        report.pending("版管理が浅い写し（shallow）で anchor が 1 本も無い＝履歴を照合できない（まだ分からない）。完全な写しで回す");
        return None;
    }
    let rel = anch
        .strip_prefix(&top)
        .unwrap_or(&anch)
        .to_string_lossy()
        .into_owned();
    // 追跡済みでも ignore の規則が当たれば落とす（以後の anchor が版管理に入らなくなる）
    for p in std::iter::once(rel.clone()).chain(present.iter().map(|n| format!("{rel}/{n}"))) {
        let Some(o) = git(&top, &["check-ignore", "-q", "--no-index", &p]) else {
            report.pending(NO_GIT);
            return None;
        };
        if o.ok() {
            report.violation(
                "anchor",
                format!(
                    "{p} が版管理から除外（ignore）されている（追跡済みでも規則が当たれば落とす）"
                ),
            );
        }
    }
    let (Some(ls), Some(lg)) = (
        git(&top, &["ls-tree", "-r", "--name-only", "HEAD", "--", &rel]),
        git(
            &top,
            &["log", "--all", "--format=%H", "--name-status", "--", &rel],
        ),
    ) else {
        report.pending(NO_GIT);
        return None;
    };
    if !ls.ok() || !lg.ok() {
        report.pending(
            "版管理を読めない（ls-tree / log が失敗）＝anchor を版管理と照合できない（まだ分からない）",
        );
        return None;
    }
    let tracked: BTreeSet<String> = ls.text().split_whitespace().map(base_name).collect();
    let mut ever: BTreeSet<String> = BTreeSet::new();
    // anchor 名 → [(commit の頭 7 字, 本文)]（全 ref の履歴で追加・変更された anchor）
    let mut hist: BTreeMap<String, Vec<(String, Vec<u8>)>> = BTreeMap::new();
    let mut commit: Option<String> = None;
    for line in lg.text().lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if is_commit_line(line) {
            commit = Some(line.to_string());
            continue;
        }
        let (Some(cm), Some((st, path))) = (&commit, status_line(line)) else {
            continue;
        };
        let name = base_name(path);
        if st != 'D' {
            ever.insert(name.clone());
        }
        if "AMRC".contains(st) && name.starts_with("constitution-") {
            let spec = format!("{cm}:{path}");
            let Some(show) = git(&top, &["show", &spec]) else {
                report.pending(NO_GIT);
                return None;
            };
            if show.ok() {
                hist.entry(name)
                    .or_default()
                    .push((cm[..7].to_string(), show.stdout));
            }
        }
    }
    for name in tracked.difference(&present) {
        report.violation(
            "anchor",
            format!(
                "anchors/{name} は版管理（HEAD）にあるが作業ツリーに無い（anchor と索引は消さない）"
            ),
        );
    }
    for name in ever
        .iter()
        .filter(|n| !tracked.contains(*n) && !present.contains(*n))
    {
        report.violation(
            "anchor",
            format!(
                "anchors/{name} は版管理の履歴に在ったが作業ツリーに無い（削除を commit しても列の始め直しは認めない・移行は床の外の手順で行う）"
            ),
        );
    }
    // 履歴に在った同じ形式の anchor と中身が違えば差し替え（凍結物は書き換えない・同じ版は凍結し直さない）
    for (name, blobs) in hist.iter().filter(|(n, _)| present.contains(*n)) {
        let Ok(cur) = fs::read(anch.join(name)) else {
            continue;
        };
        for (cm, blob) in blobs {
            if *blob == cur {
                continue;
            }
            let Some(doc) = read_blob(blob) else {
                continue;
            };
            if format_same(&doc) {
                report.violation(
                    "anchor",
                    format!(
                        "anchors/{name} が版管理の履歴の同じ形式の anchor と中身が違う（{cm}・凍結物の差し替え・書き換え。同じ版は凍結し直さない）"
                    ),
                );
                break;
            }
        }
    }
    Some(Tracked {
        tracked,
        ever,
        present,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gitcheck_status_lines_take_the_right_path() {
        assert_eq!(status_line("A\tx/a.yaml"), Some(('A', "x/a.yaml")));
        assert_eq!(
            status_line("R100\tx/a.yaml\tx/b.yaml"),
            Some(('R', "x/b.yaml"))
        );
        assert_eq!(status_line("D\tx/c.yaml"), Some(('D', "x/c.yaml")));
        assert_eq!(status_line("a\tx"), None);
        assert_eq!(status_line("M"), None);
        assert!(is_commit_line(&"0123456789abcdef".repeat(3)[..40]));
        assert!(!is_commit_line("0123"));
    }
}
