//! 凍結 fixture `tests/floor_cases.yaml`（134 case・f2-648.2 の受入・P-10.1）を `folio check` で回す歯（便 10・docs/design/delivery-10.md §1）。
//! 規則は Python の runner `tests/run_floor_cases.py` の写し: case ごとに design-intent/ を一時 dir へ写し、既定で git の 1 commit にし、
//! mutate の段を順に当てて folio を回し、終了コード・出力の語・file の有無・違反件数を照合する。fixture は読むだけで書かない。
//! 1 本の歯が全 case を回し、落ちた case を名指す。型付きの読み書きと digest は src の yaml.rs / sha256.rs を取り込んで使う。

#[path = "../src/sha256.rs"]
#[allow(dead_code)]
mod sha256;
#[path = "../src/yaml.rs"]
#[allow(dead_code)]
mod yaml;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use yaml::Value;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// ── 型付きの木の小道具 ──

/// Python の真偽（None・False・0・空の文字列・空の一覧・空の表は偽）。
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Int(s) => s.trim_start_matches('-') != "0",
        Value::Float(f) => *f != 0.0,
        Value::Date(_) => true,
        Value::Str(s) => !s.is_empty(),
        Value::Seq(s) => !s.is_empty(),
        Value::Map(m) => !m.is_empty(),
    }
}

/// `mu.get(key)` の真偽。
fn flag(mu: &Value, key: &str) -> bool {
    mu.get(key).is_some_and(truthy)
}

fn int_of(v: &Value) -> Result<i64, String> {
    match v {
        Value::Int(s) => s.parse().map_err(|e| format!("整数でない「{s}」: {e}")),
        Value::Bool(b) => Ok(i64::from(*b)),
        other => Err(format!("整数でない「{}」", other.py_str())),
    }
}

/// `mu[key]`（無ければ KeyError の写し）。
fn need<'a>(mu: &'a Value, key: &str) -> Result<&'a Value, String> {
    mu.get(key)
        .ok_or_else(|| format!("KeyError: 欄 {key} が無い"))
}

/// file の path に使う文字列（`str(x)`）。
fn text(mu: &Value, key: &str) -> Result<String, String> {
    Ok(need(mu, key)?.py_str())
}

/// 表の欄を置き換える（無ければ末尾に足す）。
fn map_set(v: &mut Value, key: &str, value: Value) -> Result<(), String> {
    let Value::Map(m) = v else {
        return Err(format!("表でないものに欄 {key} を置けない"));
    };
    match m.iter_mut().find(|(k, _)| k.as_str() == Some(key)) {
        Some(e) => e.1 = value,
        None => m.push((Value::Str(key.to_string()), value)),
    }
    Ok(())
}

enum Step {
    Name(String),
    Index(i64),
    /// 一覧から欄の字面が値に等しい要素を選ぶ
    Find(String, String),
}

/// path の文法 = 「.」区切りの欄名・「[<数>]」の添字・「[<欄>=<値>]」（runner の SEG の写し）。
fn steps(path: &str) -> Result<Vec<Step>, String> {
    let chars: Vec<char> = path.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '.' | ']' => i += 1,
            '[' => match chars[i + 1..].iter().position(|&c| c == ']') {
                Some(n) if n > 0 => {
                    let t: String = chars[i + 1..i + 1 + n].iter().collect();
                    out.push(match t.split_once('=') {
                        Some((k, v)) => Step::Find(k.to_string(), v.to_string()),
                        None => Step::Index(
                            t.parse()
                                .map_err(|e| format!("添字「{t}」が整数でない: {e}"))?,
                        ),
                    });
                    i += n + 2;
                }
                _ => i += 1,
            },
            _ => {
                let start = i;
                while i < chars.len() && !matches!(chars[i], '[' | ']' | '.') {
                    i += 1;
                }
                out.push(Step::Name(chars[start..i].iter().collect()));
            }
        }
    }
    Ok(out)
}

fn norm_index(n: i64, len: usize) -> Result<usize, String> {
    let i = if n < 0 { n + len as i64 } else { n };
    if i < 0 || i as usize >= len {
        return Err(format!("IndexError: 添字 {n} が一覧（{len} 要素）の外"));
    }
    Ok(i as usize)
}

fn step_mut<'a>(v: &'a mut Value, st: &Step) -> Result<&'a mut Value, String> {
    match (v, st) {
        (Value::Map(m), Step::Name(k)) => m
            .iter_mut()
            .find(|(key, _)| key.as_str() == Some(k))
            .map(|(_, x)| x)
            .ok_or_else(|| format!("KeyError: 欄 {k} が無い")),
        (Value::Seq(s), Step::Index(n)) => {
            let i = norm_index(*n, s.len())?;
            Ok(&mut s[i])
        }
        (Value::Seq(s), Step::Find(k, want)) => {
            for x in s.iter_mut() {
                if !matches!(x, Value::Map(_)) {
                    return Err(format!("[{k}={want}]: 一覧の要素が表でない"));
                }
                if x.get(k).unwrap_or(&Value::Null).py_str() == *want {
                    return Ok(x);
                }
            }
            Err(format!("StopIteration: [{k}={want}] に当たる要素が無い"))
        }
        _ => Err("path が解けない（表・一覧の形が違う）".to_string()),
    }
}

fn walk<'a>(doc: &'a mut Value, st: &[Step]) -> Result<&'a mut Value, String> {
    let mut cur = doc;
    for s in st {
        cur = step_mut(cur, s)?;
    }
    Ok(cur)
}

/// path の欄を置き換える（欄が無ければ add のときだけ新設・一覧の末尾の次への add は追加）。
fn mutate(doc: &mut Value, path: &str, value: Value, add: bool) -> Result<(), String> {
    let st = steps(path)?;
    let (last, init) = st
        .split_last()
        .ok_or_else(|| format!("path「{path}」が空"))?;
    let cur = walk(doc, init)?;
    match (cur, last) {
        (_, Step::Find(..)) => Err(format!("末尾の段は欄名か添字: {path}")),
        (Value::Map(m), Step::Name(k)) => {
            match m.iter_mut().find(|(key, _)| key.as_str() == Some(k)) {
                Some(e) => e.1 = value,
                None if add => m.push((Value::Str(k.clone()), value)),
                None => return Err(format!("欄 {path} が無い（新設なら add: true）")),
            }
            Ok(())
        }
        (Value::Seq(s), Step::Index(n)) => {
            if *n >= 0 && *n as usize == s.len() {
                if !add {
                    return Err(format!("{path} は一覧の末尾の次（新設なら add: true）"));
                }
                s.push(value);
            } else {
                let i = norm_index(*n, s.len())?;
                s[i] = value;
            }
            Ok(())
        }
        _ => Err(format!("path「{path}」が解けない（表・一覧の形が違う）")),
    }
}

fn load(path: &Path) -> Result<Value, String> {
    let t = fs::read_to_string(path).map_err(|e| format!("{}: 読めない: {e}", path.display()))?;
    yaml::parse_typed(&t).map_err(|e| format!("{}: 読めない: {e}", path.display()))
}

fn dump(path: &Path, v: &Value) -> Result<(), String> {
    let t = yaml::write(v, "# floor_cases")?;
    fs::write(path, t).map_err(|e| format!("{}: 書けない: {e}", path.display()))
}

/// 便 7 の正規化 + sha256（digest を除く全欄・runner の digest_of）。
fn digest_of(doc: &Value) -> Result<String, String> {
    let body: Vec<(Value, Value)> = doc
        .as_map()
        .ok_or("anchor が表でない")?
        .iter()
        .filter(|(k, _)| k.as_str() != Some("digest"))
        .cloned()
        .collect();
    Ok(sha256::hex(yaml::canonical(&Value::Map(body))?.as_bytes()))
}

// ── file・git・folio ──

fn io<T>(r: std::io::Result<T>, what: &str) -> Result<T, String> {
    r.map_err(|e| format!("{what}: {e}"))
}

fn copy_tree(src: &Path, dst: &Path) -> Result<(), String> {
    io(fs::create_dir_all(dst), "dir を作れない")?;
    for entry in io(fs::read_dir(src), "dir を読めない")? {
        let entry = io(entry, "dir を読めない")?;
        let to = dst.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &to)?;
        } else {
            io(fs::copy(entry.path(), &to), "file を写せない")?;
        }
    }
    Ok(())
}

/// git を呼ぶ（環境変数 GIT_* は継承しない・失敗は例外）。
fn git_raw(cwd: &Path, args: &[&str]) -> Result<(), String> {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    let out = io(
        cmd.current_dir(cwd).args(args).output(),
        "git を起動できない",
    )?;
    if !out.status.success() {
        return Err(format!(
            "git {args:?} が失敗: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

fn gitc(cwd: &Path, args: &[&str]) -> Result<(), String> {
    let mut all = vec![
        "-c",
        "user.email=fx@example",
        "-c",
        "user.name=fx",
        "-c",
        "commit.gpgsign=false",
    ];
    all.extend_from_slice(args);
    git_raw(cwd, &all)
}

fn git_commit(cwd: &Path) -> Result<(), String> {
    gitc(cwd, &["add", "-A"])?;
    gitc(cwd, &["commit", "-q", "--allow-empty", "-m", "fixture"])
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn folio(dir: &Path, flags: &[&str], env: &[(&str, PathBuf)]) -> Result<Run, String> {
    let out = io(
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("check")
            .arg("--dir")
            .arg(dir)
            .args(flags)
            .envs(env.iter().map(|(k, v)| (*k, v)))
            .output(),
        "folio を起動できない",
    )?;
    Ok(Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

// ── case ──

/// 段の適用の状態。
struct State<'a> {
    td: &'a Path,
    work: PathBuf,
    env: Vec<(&'static str, PathBuf)>,
    last: Option<Run>,
    note: String,
}

impl State<'_> {
    fn apply(&mut self, muts: &[&Value]) -> Result<(), String> {
        let td = self.td;
        for (i, mu) in muts.iter().enumerate() {
            let mu = *mu;
            if flag(mu, "freeze_anchor") {
                let r = folio(&self.work, &["--freeze-anchor"], &self.env)?;
                if i + 1 != muts.len() {
                    let want = mu.get("freeze_rc").map_or(Ok(0), int_of)?;
                    if i64::from(r.code) != want {
                        self.note += &format!(" ／ 途中の凍結が rc {}", r.code);
                    }
                    if r.code == 0
                        && mu.get("commit").is_none_or(truthy)
                        && td.join(".git").exists()
                    {
                        git_commit(td)?;
                    }
                }
                self.last = Some(r);
                continue;
            }
            self.last = None;
            if flag(mu, "emit_amends_into") {
                let em = folio(&self.work, &["--emit-amends"], &self.env)?;
                if em.code == 2 {
                    self.note += &format!(" ／ emit が rc {}（読めない）", em.code);
                    continue;
                }
                let body: Vec<&str> = em.stdout.lines().filter(|l| !l.starts_with('#')).collect();
                let lst = match yaml::parse_typed(&body.join("\n")) {
                    Ok(v) if truthy(&v) => v,
                    Ok(_) => Value::Seq(Vec::new()),
                    Err(e) if e == "文書が空" => Value::Seq(Vec::new()),
                    Err(e) => return Err(format!("emit の出力が読めない: {e}")),
                };
                let f = self.work.join(text(mu, "emit_amends_into")?);
                let mut doc = load(&f)?;
                map_set(&mut doc, "amends", lst)?;
                dump(&f, &doc)?;
                continue;
            }
            if flag(mu, "git_snapshot") || flag(mu, "git_commit") {
                if !td.join(".git").exists() {
                    gitc(td, &["init", "-q"])?;
                }
                git_commit(td)?;
                continue;
            }
            if flag(mu, "git_ignore_anchors") {
                io(
                    fs::write(td.join(".gitignore"), "design-intent/anchors/\n"),
                    ".gitignore を書けない",
                )?;
                gitc(td, &["rm", "-r", "-q", "--cached", "design-intent/anchors"])?;
                git_commit(td)?;
                continue;
            }
            if flag(mu, "git_nested") {
                gitc(&self.work, &["init", "-q"])?;
                git_commit(&self.work)?;
                continue;
            }
            if flag(mu, "git_ignore_anchors_keep_tracked") {
                io(
                    fs::write(td.join(".gitignore"), "design-intent/anchors/\n"),
                    ".gitignore を書けない",
                )?;
                git_commit(td)?;
                continue;
            }
            if flag(mu, "git_ignore_pattern") {
                let pattern = text(mu, "git_ignore_pattern")?;
                io(
                    fs::write(td.join(".gitignore"), format!("{pattern}\n")),
                    ".gitignore を書けない",
                )?;
                git_commit(td)?;
                continue;
            }
            if flag(mu, "git_reinit_no_commit") {
                io(fs::remove_dir_all(td.join(".git")), ".git を消せない")?;
                gitc(td, &["init", "-q"])?;
                continue;
            }
            if flag(mu, "git_orphan_drop_anchors") {
                gitc(td, &["checkout", "-q", "--orphan", "clean"])?;
                gitc(td, &["rm", "-r", "-q", "--cached", "design-intent/anchors"])?;
                io(
                    fs::remove_dir_all(self.work.join("anchors")),
                    "anchors/ を消せない",
                )?;
                git_commit(td)?;
                continue;
            }
            if flag(mu, "git_shallow_clone") {
                let sh = td.join("shallow");
                let from = format!("file://{}", td.display());
                let to = sh.display().to_string();
                git_raw(td, &["clone", "-q", "--depth", "1", &from, &to])?;
                self.work = sh.join("design-intent");
                continue;
            }
            if flag(mu, "env_git_dir_empty") {
                let em = td.join("empty-repo");
                io(fs::create_dir(&em), "empty-repo を作れない")?;
                gitc(&em, &["init", "-q"])?;
                self.env = vec![
                    ("GIT_DIR", em.join(".git")),
                    ("GIT_WORK_TREE", td.to_path_buf()),
                ];
                continue;
            }
            if flag(mu, "refreeze_in_fresh_repo") {
                let fr = td.join("fresh");
                let fresh = fr.join("design-intent");
                if fresh.exists() {
                    return Err("fresh/design-intent が既に在る".to_string());
                }
                copy_tree(&self.work, &fresh)?;
                let _ = fs::remove_dir_all(fresh.join("anchors"));
                gitc(&fr, &["init", "-q"])?;
                git_commit(&fr)?;
                let fz = folio(&fresh, &["--freeze-anchor"], &self.env)?;
                if fz.code == 0 {
                    self.note += " ／ 別の写しでの凍結が rc 0（列の始め直しが通った）";
                }
                if fresh.join("anchors").is_dir() {
                    for entry in io(fs::read_dir(fresh.join("anchors")), "anchors/ を読めない")?
                    {
                        let entry = io(entry, "anchors/ を読めない")?;
                        let name = entry.file_name();
                        if name.to_string_lossy().ends_with(".yaml") {
                            io(
                                fs::copy(entry.path(), self.work.join("anchors").join(&name)),
                                "anchor を持ち帰れない",
                            )?;
                        }
                    }
                }
                continue;
            }
            if let Some(af) = mu.get("anchor_forge") {
                let f = self.work.join(text(af, "file")?);
                let mut doc = load(&f)?;
                mutate(
                    &mut doc,
                    &text(af, "path")?,
                    need(af, "value")?.clone(),
                    flag(af, "add"),
                )?;
                let digest = digest_of(&doc)?;
                map_set(&mut doc, "digest", Value::Str(digest.clone()))?;
                dump(&f, &doc)?;
                let ix = self.work.join("anchors/index.yaml");
                let mut idx = load(&ix)?;
                let version = doc.get("version").unwrap_or(&Value::Null).py_str();
                if let Ok(Value::Seq(entries)) = walk(&mut idx, &[Step::Name("entries".into())]) {
                    for e in entries.iter_mut() {
                        if !matches!(e, Value::Map(_)) {
                            return Err("索引の項が表でない".to_string());
                        }
                        if e.get("version").unwrap_or(&Value::Null).py_str() == version {
                            map_set(e, "digest", Value::Str(digest.clone()))?;
                        }
                    }
                }
                dump(&ix, &idx)?;
                continue;
            }
            if flag(mu, "symlink_dir") || flag(mu, "symlink_file") {
                let rel = if flag(mu, "symlink_dir") {
                    text(mu, "symlink_dir")?
                } else {
                    text(mu, "symlink_file")?
                };
                let src = if rel == "." {
                    self.work.clone()
                } else {
                    self.work.join(&rel)
                };
                let outside = td.join("outside");
                io(fs::create_dir_all(&outside), "outside を作れない")?;
                let name = src.file_name().ok_or("symlink の対象に名前が無い")?;
                let dst = outside.join(name);
                io(fs::rename(&src, &dst), "対象を外へ動かせない")?;
                io(std::os::unix::fs::symlink(&dst, &src), "symlink を置けない")?;
                continue;
            }
            if flag(mu, "delete_dir") {
                io(
                    fs::remove_dir_all(self.work.join(text(mu, "dir")?)),
                    "dir を消せない",
                )?;
                continue;
            }
            let f = self.work.join(text(mu, "file")?);
            if flag(mu, "delete") {
                io(fs::remove_file(&f), "file を消せない")?;
                continue;
            }
            if let Some(tree) = mu.get("create") {
                if let Some(parent) = f.parent() {
                    io(fs::create_dir_all(parent), "親 dir を作れない")?;
                }
                dump(&f, tree)?;
                continue;
            }
            if let Some(raw) = mu.get("write_text") {
                let raw = raw.as_str().ok_or("write_text が文字列でない")?;
                if let Some(parent) = f.parent() {
                    io(fs::create_dir_all(parent), "親 dir を作れない")?;
                }
                io(fs::write(&f, raw), "file を書けない")?;
                continue;
            }
            let mut doc = load(&f)?;
            if let Some(pair) = mu.get("swap") {
                let pair = pair.as_seq().ok_or("swap が一覧でない")?;
                let (Some(a), Some(b)) = (pair.first(), pair.get(1)) else {
                    return Err("swap の要素が 2 つでない".to_string());
                };
                let (a, b) = (int_of(a)?, int_of(b)?);
                let Value::Seq(s) = walk(&mut doc, &steps(&text(mu, "path")?)?)? else {
                    return Err("swap の path が一覧でない".to_string());
                };
                let (a, b) = (norm_index(a, s.len())?, norm_index(b, s.len())?);
                s.swap(a, b);
            } else if let Some(key) = mu.get("raw_key") {
                let key = key.py_str();
                let value = need(mu, "value")?.clone();
                map_set(walk(&mut doc, &steps(&text(mu, "path")?)?)?, &key, value)?;
            } else if flag(mu, "pop") {
                let Value::Seq(s) = walk(&mut doc, &steps(&text(mu, "path")?)?)? else {
                    return Err("pop の path が一覧でない".to_string());
                };
                s.pop().ok_or("IndexError: 空の一覧から pop")?;
            } else {
                mutate(
                    &mut doc,
                    &text(mu, "path")?,
                    need(mu, "value")?.clone(),
                    flag(mu, "add"),
                )?;
            }
            dump(&f, &doc)?;
        }
        Ok(())
    }
}

fn str_field(cs: &Value, key: &str) -> Option<String> {
    cs.get(key).filter(|v| truthy(v)).map(Value::py_str)
}

/// 1 case を回す。期待どおりなら None、違えば落ちた case の文言。
fn run_case(cs: &Value, td: &Path) -> Option<String> {
    let id = cs.get("id").map(Value::py_str).unwrap_or_default();
    let why = cs.get("why").map(Value::py_str).unwrap_or_default();
    let expect_rc = cs.get("expect_rc").map(int_of);
    let Some(Ok(expect_rc)) = expect_rc else {
        return Some(format!("FAIL {id}: expect_rc が読めない"));
    };
    let mut muts: Vec<&Value> = Vec::new();
    for m in cs.get("mutate").and_then(Value::as_seq).unwrap_or_default() {
        match m {
            Value::Seq(items) => muts.extend(items.iter()),
            other => muts.push(other),
        }
    }
    let mut st = State {
        td,
        work: td.join("design-intent"),
        env: Vec::new(),
        last: None,
        note: String::new(),
    };
    let prepared = copy_tree(&repo_root().join("design-intent"), &st.work).and_then(|()| {
        if flag(cs, "no_git") {
            Ok(())
        } else {
            gitc(td, &["init", "-q"]).and_then(|()| git_commit(td))
        }
    });
    if let Err(e) = prepared.and_then(|()| st.apply(&muts)) {
        return Some(format!("FAIL {id}: fixture の適用で例外 {e}"));
    }
    let pr = match st.last.take() {
        Some(pr) if pr.code == 0 => match folio(&st.work, &[], &st.env) {
            Ok(pr2) if pr2.code != 0 => {
                st.note += &format!(" ／ 凍結後の素の床が rc {}", pr2.code);
                pr2
            }
            Ok(_) => pr,
            Err(e) => return Some(format!("FAIL {id}: {e}")),
        },
        Some(pr) => pr,
        None => match folio(&st.work, &[], &st.env) {
            Ok(pr) => pr,
            Err(e) => return Some(format!("FAIL {id}: {e}")),
        },
    };
    let all = format!("{}{}", pr.stdout, pr.stderr);
    let mut ok = i64::from(pr.code) == expect_rc;
    if let Some(m) = str_field(cs, "expect_msg")
        && !all.contains(&m)
    {
        ok = false;
    }
    if let Some(m) = str_field(cs, "expect_stderr")
        && !pr.stderr.contains(&m)
    {
        ok = false;
    }
    if let Some(m) = str_field(cs, "expect_not_msg")
        && all.contains(&m)
    {
        ok = false;
    }
    if let Some(n) = cs.get("expect_n") {
        let got = pr.stdout.lines().filter(|l| l.starts_with('[')).count() as i64;
        if int_of(n).ok() != Some(got) {
            ok = false;
        }
    }
    ok = ok && st.note.is_empty();
    for rel in cs
        .get("expect_no_file")
        .and_then(Value::as_seq)
        .unwrap_or_default()
    {
        let rel = rel.py_str();
        if st.work.join(&rel).exists() {
            ok = false;
            st.note += &format!(" ／ {rel} が書かれている");
        }
    }
    for rel in cs
        .get("expect_file")
        .and_then(Value::as_seq)
        .unwrap_or_default()
    {
        let rel = rel.py_str();
        if !st.work.join(&rel).exists() {
            ok = false;
            st.note += &format!(" ／ {rel} が無い");
        }
    }
    if ok {
        return None;
    }
    let mut msg = format!(
        "FAIL {id}: rc={}（期待 {expect_rc}） — {why}{}",
        pr.code, st.note
    );
    for ln in all.trim().lines().take(10) {
        msg.push_str("\n    ");
        msg.push_str(ln);
    }
    Some(msg)
}

#[test]
fn floor_cases_all_pass_with_folio() {
    let path = repo_root().join("tests/floor_cases.yaml");
    let fx = yaml::parse_typed(&fs::read_to_string(&path).unwrap())
        .unwrap_or_else(|e| panic!("fixture が読めない: {e}"));
    let cases: Vec<&Value> = fx
        .as_map()
        .expect("fixture が表でない")
        .iter()
        .filter(|(k, _)| k.as_str().is_some_and(|k| k.starts_with("cases")))
        .flat_map(|(k, v)| {
            v.as_seq()
                .unwrap_or_else(|| panic!("{} が一覧でない", k.py_str()))
                .iter()
        })
        .collect();
    let expected = fx.get("expected_cases").map(int_of);
    assert_eq!(
        expected,
        Some(Ok(cases.len() as i64)),
        "case 件数 {} が expected_cases と違う（fixture が欠けたか増えた）",
        cases.len()
    );
    for cs in &cases {
        let id = cs.get("id").map(Value::py_str).unwrap_or_default();
        let rc = cs.get("expect_rc").map(int_of);
        assert!(matches!(rc, Some(Ok(_))), "{id}: expect_rc が無い");
        assert!(
            rc == Some(Ok(0)) || flag(cs, "expect_msg") || flag(cs, "expect_stderr"),
            "{id}: rc 非 0 の case には expect_msg か expect_stderr が要る"
        );
    }

    let next = AtomicUsize::new(0);
    let failures: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let n = next.fetch_add(1, Ordering::SeqCst);
                    let Some(cs) = cases.get(n) else { break };
                    let td = std::env::temp_dir()
                        .join(format!("folio-floor-cases-{}-{n}", std::process::id()));
                    let _ = fs::remove_dir_all(&td);
                    let result = run_case(cs, &td);
                    let _ = fs::remove_dir_all(&td);
                    if let Some(msg) = result {
                        failures.lock().unwrap().push((n, msg));
                    }
                }
            });
        }
    });
    let mut failures = failures.into_inner().unwrap();
    failures.sort_by_key(|(n, _)| *n);
    let report: Vec<String> = failures.into_iter().map(|(_, m)| m).collect();
    assert!(
        report.is_empty(),
        "{} / {} case が期待どおり\n{}",
        cases.len() - report.len(),
        cases.len(),
        report.join("\n")
    );
}
