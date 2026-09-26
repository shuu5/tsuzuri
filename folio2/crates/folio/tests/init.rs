//! `folio init` の歯（便 125・docs/design/delivery-125.md §1 (e)・FR22 / FR3・受入基準 AC19 / AC26 / AC27）。
//! binary 経由だけで測る。土台は一時 dir の根で git の init をし、置き場を根の下の design-intent/folio2 に取る
//! （利用者の形・版管理の外の「まだ分からない」を混ぜない）。印の置き場（hello の --state）も一時 dir の下。
//! 凍結 anchor（P-10.1）は 2 つで、どちらも生成器から独立した手書き: 禁止字の一覧 tests/fixtures/schema/init-forbidden.txt と、
//! 歯の中に字で持つ書く file 11 本の閉じた一覧 `FILES`。括弧の落としと id の形は歯の側で自前に書く（init.rs の関数を呼ばない）。

#[path = "../src/sha256.rs"]
#[allow(dead_code)]
mod sha256;
#[path = "../src/yaml.rs"]
#[allow(dead_code)]
mod yaml;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use yaml::Value;

/// 書く file の閉じた一覧（凍結 anchor・置き場からの相対）。
const FILES: [&str; 11] = [
    "constitution.yaml",
    "rules.yaml",
    "vocabulary.yaml",
    "srs.yaml",
    "index.yaml",
    "intake.yaml",
    "ceiling.yaml",
    "graph.yaml",
    "adr/schema.yaml",
    "adr/ADR-1.yaml",
    "design-note/schema.yaml",
];

/// 断りの名 11 個（正本 7・graph.yaml・adr/・design-note/・anchors/）。dir の名は末尾に / を付けて書く。
const REFUSE: [&str; 11] = [
    "constitution.yaml",
    "rules.yaml",
    "vocabulary.yaml",
    "srs.yaml",
    "index.yaml",
    "intake.yaml",
    "ceiling.yaml",
    "graph.yaml",
    "adr/",
    "design-note/",
    "anchors/",
];

/// 骨格自身の番号。
const OWN: [&str; 6] = ["ADR-1", "P-1", "P-1.1", "R-2", "R-8", "R-16"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn folio2() -> PathBuf {
    repo_root().join("design-intent")
}

/// git を呼ぶ。環境変数 GIT_* は継承しない。
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

fn folio(args: &[&str], place: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(args)
        .arg("--dir")
        .arg(place)
        .output()
        .expect("folio を起動できない")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn both(out: &Output) -> String {
    format!("{}{}", stdout(out), stderr(out))
}

/// 一時 dir の根（git の init 済み）と、その下の置き場 design-intent/folio2（親まで作る・置き場は作らない）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-init-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("design-intent")).unwrap();
        git(&root, &["init", "-q"]);
        Work { root }
    }

    fn place(&self) -> PathBuf {
        self.root.join("design-intent/folio2")
    }

    fn state(&self) -> PathBuf {
        self.root.join("state")
    }

    fn init(&self) -> Output {
        folio(&["init"], &self.place())
    }

    fn init_ok(&self) -> Output {
        let out = self.init();
        assert_eq!(out.status.code(), Some(0), "{}", both(&out));
        out
    }

    fn hello(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("hello")
            .arg("--dir")
            .arg(self.place())
            .arg("--state")
            .arg(self.state())
            .output()
            .expect("folio を起動できない")
    }

    fn read(&self, file: &str) -> String {
        fs::read_to_string(self.place().join(file)).unwrap()
    }

    fn typed(&self, file: &str) -> Value {
        yaml::parse_typed(&self.read(file)).unwrap_or_else(|e| panic!("{file}: {e}"))
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// dir の下の全部の項（.git の下は数えない）の相対 path と、種類と中身の要約値。
fn snapshot(dir: &Path) -> BTreeMap<String, String> {
    fn walk(base: &Path, at: &Path, out: &mut BTreeMap<String, String>) {
        let Ok(entries) = fs::read_dir(at) else {
            return;
        };
        for e in entries {
            let p = e.unwrap().path();
            let rel = p.strip_prefix(base).unwrap().to_string_lossy().into_owned();
            if rel == ".git" {
                continue;
            }
            let meta = fs::symlink_metadata(&p).unwrap();
            if meta.file_type().is_symlink() {
                out.insert(rel, format!("link {}", fs::read_link(&p).unwrap().display()));
            } else if meta.is_dir() {
                out.insert(rel, "dir".to_string());
                walk(base, &p, out);
            } else {
                out.insert(rel, sha256::hex(&fs::read(&p).unwrap()));
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

// ── 数える範囲（歯の側の式）──

/// 行頭が空白でない空でない行（最上位の節か注釈の始まり）。
fn is_boundary(line: &str) -> bool {
    !line.is_empty() && !line.starts_with(' ')
}

/// 生成区間の印の間（印の行も）を落とした行。
fn without_region(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("# folio:schema:begin") {
            inside = true;
            continue;
        }
        if line.starts_with("# folio:schema:end") {
            inside = false;
            continue;
        }
        if !inside {
            out.push(line);
        }
    }
    out
}

/// 最上位の節 `keys` を落とした行（節の頭の鍵は引用符付きも数える）。
fn without_sections<'a>(lines: Vec<&'a str>, keys: &[&str]) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut skip = false;
    for line in lines {
        if is_boundary(line) {
            let key = line.split(':').next().unwrap_or("").trim_matches('"');
            skip = keys.contains(&key) && line.contains(':');
        }
        if !skip {
            out.push(line);
        }
    }
    out
}

/// 禁止字の数える範囲（生成区間と、相談窓口の 4 節を除いた字）。
fn forbidden_range(dir: &Path, file: &str) -> String {
    let text = fs::read_to_string(dir.join(file)).unwrap();
    let mut lines = without_region(&text);
    if file == "intake.yaml" {
        lines = without_sections(lines, &["answers", "targets", "questions", "sheet"]);
    }
    lines.join("\n")
}

/// id の数える範囲（生成区間と、憲法の schema の節を除いた字）。
fn id_range(dir: &Path, file: &str) -> String {
    let text = fs::read_to_string(dir.join(file)).unwrap();
    let mut lines = without_region(&text);
    if file == "constitution.yaml" {
        lines = without_sections(lines, &["schema"]);
    }
    lines.join("\n")
}

/// id の形の字（前の字が英字でないもの）を全部取る。
fn ids(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let num = |k: usize| -> usize { chars.iter().skip(k).take_while(|c| c.is_ascii_digit()).count() };
    let starts = |k: usize, p: &str| -> bool {
        p.chars().enumerate().all(|(n, ch)| chars.get(k + n) == Some(&ch))
    };
    let mut found = Vec::new();
    let mut k = 0;
    while k < chars.len() {
        let free = k == 0 || !chars[k - 1].is_ascii_alphabetic();
        let mut take = 0;
        if free {
            if starts(k, "ADR-") && num(k + 4) > 0 {
                take = 4 + num(k + 4);
            } else if chars[k] == '便' {
                let gap = chars.iter().skip(k + 1).take_while(|c| **c == ' ').count();
                if num(k + 1 + gap) > 0 {
                    take = 1 + gap + num(k + 1 + gap);
                }
            } else {
                for p in ["NFR", "CON", "FR", "AC"] {
                    if starts(k, p) && num(k + p.len()) > 0 {
                        take = p.len() + num(k + p.len());
                        break;
                    }
                }
                if take == 0 && "PNARD".contains(chars[k]) && chars.get(k + 1) == Some(&'-') {
                    let n = num(k + 2);
                    if n > 0 {
                        take = 2 + n;
                        if "PNA".contains(chars[k])
                            && chars.get(k + take) == Some(&'.')
                            && num(k + take + 1) > 0
                        {
                            take += 1 + num(k + take + 1);
                        }
                    }
                }
            }
        }
        if take > 0 {
            found.push(chars[k..k + take].iter().filter(|c| **c != ' ').collect());
            k += take;
        } else {
            k += 1;
        }
    }
    found
}

fn foreign(text: &str) -> Vec<String> {
    ids(text)
        .into_iter()
        .filter(|id| !OWN.contains(&id.as_str()))
        .collect()
}

/// 全角の括弧（入れ子は数える）のうち中に骨格自身の番号でない id を持つものを中身ごと落とす（歯の側の式）。
fn drop_parens(text: &str) -> String {
    let mut out = String::new();
    let mut group = String::new();
    let mut depth = 0usize;
    for ch in text.chars() {
        if depth == 0 && ch != '（' {
            out.push(ch);
            continue;
        }
        group.push(ch);
        match ch {
            '（' => depth += 1,
            '）' => depth -= 1,
            _ => {}
        }
        if depth == 0 {
            if foreign(&group).is_empty() {
                out.push_str(&group);
            }
            group.clear();
        }
    }
    out.push_str(&group);
    out
}

/// 値の中の字の全部に括弧の落としを当てる。
fn dropped(v: &Value) -> Value {
    match v {
        Value::Str(s) => Value::Str(drop_parens(s)),
        Value::Seq(items) => Value::Seq(items.iter().map(dropped).collect()),
        Value::Map(entries) => Value::Map(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), dropped(v)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn folio2_typed(file: &str) -> Value {
    yaml::parse_typed(&fs::read_to_string(folio2().join(file)).unwrap()).unwrap()
}

fn seq<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_seq)
        .unwrap_or_else(|| panic!("{key} が一覧でない"))
}

fn str_of<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{key} が字でない"))
}

fn strs(v: &Value) -> Vec<&str> {
    v.as_seq()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect()
}

/// 1970-01-01 からの日数を年-月-日へ（歯の側の数え）。
fn ymd(mut days: i64) -> String {
    let mut y = 1970;
    loop {
        let len = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
        if days < len {
            break;
        }
        days -= len;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let months = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0;
    while days >= months[m] {
        days -= months[m];
        m += 1;
    }
    format!("{y:04}-{:02}-{:02}", m + 1, days + 1)
}

/// 撃った日の UTC と前後 1 日。
fn dates_around_now() -> Vec<String> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let today = secs / 86_400;
    vec![ymd(today - 1), ymd(today), ymd(today + 1)]
}

// ── 歯 ──

/// 歯 1（AC19 の前半）: 空の置き場に書くと 11 本がちょうど書かれ、床は凍結の基準の不在だけを「まだ分からない」にする。
#[test]
fn f125_init_writes_the_skeleton_and_the_floor_passes() {
    let w = Work::new("writes");
    let out = w.init_ok();
    let files: Vec<String> = snapshot(&w.root)
        .into_iter()
        .filter(|(_, kind)| kind != "dir")
        .map(|(rel, _)| rel)
        .collect();
    let want: Vec<String> = {
        let mut v: Vec<String> = FILES
            .iter()
            .map(|f| format!("design-intent/folio2/{f}"))
            .collect();
        v.sort();
        v
    };
    assert_eq!(files, want, "置き場の外か 11 本の外に file が在る");
    assert!(!w.root.join("CLAUDE.md").exists());
    let text = stdout(&out);
    let last = text.lines().last().unwrap_or("");
    assert!(last.contains("commit") && last.contains("folio check"), "{text}");

    git(&w.root, &["add", "-A"]);
    git(&w.root, &["commit", "-q", "-m", "skeleton"]);
    let check = folio(&["check"], &w.place());
    assert_eq!(check.status.code(), Some(2), "{}", both(&check));
    assert!(
        !stdout(&check).lines().any(|l| l.starts_with('[')),
        "{}",
        both(&check)
    );
    assert!(stdout(&check).contains("違反 0"), "{}", both(&check));
    let unknowns: Vec<String> = stderr(&check)
        .lines()
        .filter(|l| l.starts_with("# まだ分からない: "))
        .map(str::to_string)
        .collect();
    assert_eq!(unknowns.len(), 2, "{unknowns:?}");
    assert!(unknowns.iter().any(|l| l.contains("凍結 anchor が 0 本")), "{unknowns:?}");
    assert!(
        unknowns.iter().any(|l| l.contains("anchors/ids-*.yaml）が無い")),
        "{unknowns:?}"
    );

    let schema = folio(&["schema", "--check"], &w.place());
    assert_eq!(schema.status.code(), Some(0), "{}", both(&schema));

    let intake = folio(&["intake", "--print"], &w.place());
    assert_eq!(intake.status.code(), Some(0), "{}", both(&intake));
    let asked: Vec<String> = stdout(&intake)
        .lines()
        .filter_map(|l| l.split_once('.').map(|(q, _)| q.to_string()))
        .filter(|q| q.starts_with('q'))
        .collect();
    assert_eq!(asked, ["q1", "q2", "q3", "q4", "q5"], "{}", stdout(&intake));

    let hello = w.hello();
    assert_eq!(hello.status.code(), Some(0), "{}", both(&hello));
    let line = stdout(&hello);
    assert_eq!(line.lines().count(), 1, "{line}");
    assert!(line.contains("節点") && line.contains("辺"), "{line}");
}

/// 歯 2（AC19 の後半）: 断りの名が 1 つでも在れば何も書かずに 1。ほかの file だけなら 0。親が無ければ 2。
#[test]
fn f125_init_refuses_when_any_source_exists() {
    for name in REFUSE {
        let w = Work::new("refuse");
        fs::create_dir_all(w.place()).unwrap();
        match name.strip_suffix('/') {
            Some(d) => fs::create_dir(w.place().join(d)).unwrap(),
            None => fs::write(w.place().join(name), "x").unwrap(),
        }
        let before = snapshot(&w.place());
        let out = w.init();
        assert_eq!(out.status.code(), Some(1), "{name}: {}", both(&out));
        let named = w.place().join(name.trim_end_matches('/'));
        assert!(
            stderr(&out).contains(&named.display().to_string()),
            "{name}: {}",
            stderr(&out)
        );
        assert_eq!(snapshot(&w.place()), before, "{name}: 置き場が変わった");
    }

    let w = Work::new("refuse-broken-link");
    fs::create_dir_all(w.place()).unwrap();
    std::os::unix::fs::symlink("no-such-target", w.place().join("constitution.yaml")).unwrap();
    let before = snapshot(&w.place());
    let out = w.init();
    assert_eq!(out.status.code(), Some(1), "{}", both(&out));
    assert!(stderr(&out).contains("constitution.yaml"), "{}", stderr(&out));
    assert_eq!(snapshot(&w.place()), before, "壊れた symlink の置き場が変わった");

    let w = Work::new("other-file");
    fs::create_dir_all(w.place()).unwrap();
    fs::write(w.place().join("README"), "読んでね\n").unwrap();
    w.init_ok();
    assert_eq!(fs::read(w.place().join("README")).unwrap(), "読んでね\n".as_bytes());

    let w = Work::new("no-parent");
    let place = w.root.join("nowhere/folio2");
    let out = folio(&["init"], &place);
    assert_eq!(out.status.code(), Some(2), "{}", both(&out));
    assert!(!w.root.join("nowhere").exists(), "置き場の親を作った");
}

/// 歯 3（AC26）: 禁止字の一覧が自己検査を通り、骨格の数える範囲に 0 回・folio2 自身には 1 回以上。
#[test]
fn f125_forbidden_words_stay_out_of_the_skeleton() {
    let list = fs::read_to_string(repo_root().join("tests/fixtures/schema/init-forbidden.txt")).unwrap();
    let forbidden: Vec<&str> = list.lines().collect();
    assert_eq!(forbidden.len(), 28, "{forbidden:?}");

    // (i) 各行が folio2 の承認欄の行（鍵 who か verbatim）に在る
    fn yamls(at: &Path, out: &mut Vec<PathBuf>) {
        for e in fs::read_dir(at).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n != "preview") {
                    yamls(&p, out);
                }
            } else if p.extension().is_some_and(|x| x == "yaml") {
                out.push(p);
            }
        }
    }
    let mut paths = Vec::new();
    yamls(&folio2(), &mut paths);
    let is_key_line = |line: &str| {
        ["who", "verbatim"].iter().any(|key| {
            [format!("{key}:"), format!("\"{key}\":"), format!("'{key}':")]
                .iter()
                .any(|pat| {
                    line.match_indices(pat.as_str()).any(|(at, _)| {
                        line[..at]
                            .chars()
                            .next_back()
                            .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
                    })
                })
        })
    };
    let approval_lines: Vec<String> = paths
        .iter()
        .flat_map(|p| {
            fs::read_to_string(p)
                .unwrap()
                .lines()
                .filter(|l| is_key_line(l))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect();
    let init_rs = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/init.rs")).unwrap();
    for word in &forbidden {
        assert!(
            approval_lines.iter().any(|l| l.contains(word)),
            "承認欄の行に無い: {word}"
        );
        // (ii) 雛形の定数（init.rs）に無い
        assert!(!init_rs.contains(word), "init.rs に在る: {word}");
    }

    let w = Work::new("forbidden");
    w.init_ok();
    let mut in_folio2 = 0;
    for file in FILES {
        let body = forbidden_range(&w.place(), file);
        for word in &forbidden {
            assert!(!body.contains(word), "{file} に禁止字「{word}」");
        }
        let own = forbidden_range(&folio2(), file);
        in_folio2 += forbidden.iter().map(|word| own.matches(word).count()).sum::<usize>();
    }
    assert!(in_folio2 > 0, "folio2 自身の数える範囲に 1 回も出ない＝数えが効いていない");
}

/// 歯 4: 2 回撃った骨格は、日付の字（撃った日の UTC と前後 1 日）を固定の字に替えた後で byte 一致する。
#[test]
fn f125_two_runs_differ_only_by_the_date() {
    let a = Work::new("twice-a");
    let b = Work::new("twice-b");
    a.init_ok();
    b.init_ok();
    let dates = dates_around_now();
    let norm = |text: String| {
        dates
            .iter()
            .fold(text, |t, d| t.replace(d.as_str(), "YYYY-MM-DD"))
    };
    for file in FILES {
        assert_eq!(norm(a.read(file)), norm(b.read(file)), "{file}");
        assert!(
            without_region(&a.read(file))
                .iter()
                .filter(|l| dates.iter().any(|d| l.contains(d.as_str())))
                .all(|l| l.contains("generated:") || l.contains("date:")),
            "{file}: 日付の欄の外に日付の字が在る"
        );
    }
    let adr = a.typed("adr/ADR-1.yaml");
    assert!(dates.iter().any(|d| adr.get("date") == Some(&Value::Date(d.clone()))), "ADR-1.date");
}

/// 歯 5: 雛形の出所が folio2 の正本と揃う（入口・天井・相談窓口は folio2 の値に括弧の落としを歯の側で当てた字と比べる）。
#[test]
fn f125_sources_match_folio2() {
    let w = Work::new("sources");
    w.init_ok();

    // 憲法の schema の節
    let mine = w.typed("constitution.yaml");
    let theirs = folio2_typed("constitution.yaml");
    assert_eq!(mine.get("schema"), theirs.get("schema"), "憲法の schema の値");
    let text = w.read("constitution.yaml");
    let schema_text: String = text
        .lines()
        .skip_while(|l| !l.starts_with("\"schema\":") && !l.starts_with("schema:"))
        .skip(1)
        .take_while(|l| !is_boundary(l))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!schema_text.is_empty(), "憲法の schema の節が無い");
    assert!(!schema_text.contains('#'), "憲法の schema の節に # が在る");

    // 条は P-1 の 1 本で行 R-2・R-8・R-16 を縛る
    let articles = seq(&mine, "articles");
    assert_eq!(articles.len(), 1);
    assert_eq!(str_of(&articles[0], "id"), "P-1");
    let rules = articles[0].get("relations").and_then(|r| r.get("rules")).unwrap();
    assert_eq!(strs(rules), ["R-2", "R-8", "R-16"]);

    // 行 R-2 の what・value・kind と行 R-16 の what と value（行の状態と裁定は写さない）
    let row = |v: &Value, id: &str| {
        seq(v, "thresholds")
            .iter()
            .find(|r| r.get("id").and_then(Value::as_str) == Some(id))
            .cloned()
            .unwrap_or_else(|| panic!("行 {id} が無い"))
    };
    let (mr, tr) = (w.typed("rules.yaml"), folio2_typed("rules.yaml"));
    let (m2, t2) = (row(&mr, "R-2"), row(&tr, "R-2"));
    for key in ["what", "value", "kind"] {
        assert_eq!(m2.get(key), t2.get(key), "行 R-2 の {key}");
    }
    let (m16, t16) = (row(&mr, "R-16"), row(&tr, "R-16"));
    assert_eq!(m16.get("what"), t16.get("what"));
    assert_eq!(m16.get("value"), t16.get("value"));
    for r in seq(&mr, "thresholds") {
        assert_eq!(str_of(r, "status"), "仮", "{r:?}");
        assert_eq!(str_of(r, "ruling"), "未記入", "{r:?}");
    }
    assert!(
        seq(&w.typed("rules.yaml"), "thresholds")
            .iter()
            .all(|r| str_of(r, "id") != "R-17")
    );

    // 相談窓口
    let (mi, ti) = (w.typed("intake.yaml"), folio2_typed("intake.yaml"));
    for key in ["answers", "questions"] {
        assert_eq!(mi.get(key), ti.get(key).map(dropped).as_ref(), "intake.{key}");
    }
    let (mt, tt) = (seq(&mi, "targets"), seq(&ti, "targets"));
    assert_eq!(mt.len(), tt.len());
    for (m, t) in mt.iter().zip(tt) {
        let keys: Vec<&str> = m.as_map().unwrap().iter().map(|(k, _)| k.as_str().unwrap()).collect();
        assert_eq!(keys, ["id", "type", "with"]);
        for key in keys {
            assert_eq!(m.get(key), t.get(key).map(dropped).as_ref(), "targets.{key}");
        }
    }
    let (ms, ts) = (mi.get("sheet").unwrap(), ti.get("sheet").unwrap());
    let keys: Vec<&str> = ms.as_map().unwrap().iter().map(|(k, _)| k.as_str().unwrap()).collect();
    assert_eq!(keys, ["file", "title", "explain"]);
    assert_eq!(ms.get("file"), ts.get("file"));
    assert_eq!(ms.get("title"), ts.get("title"));
    assert_eq!(ms.get("explain"), ts.get("explain").map(dropped).as_ref());

    // 天井
    let (mc, tc) = (w.typed("ceiling.yaml"), folio2_typed("ceiling.yaml"));
    let pairs = |v: &Value| -> Vec<(String, String)> {
        seq(v, "documents")
            .iter()
            .map(|d| (str_of(d, "id").to_string(), str_of(d, "file").to_string()))
            .collect()
    };
    assert_eq!(pairs(&mc), pairs(&tc));
    let (mv, tv) = (seq(&mc, "viewpoints"), seq(&tc, "viewpoints"));
    assert_eq!(mv.len(), tv.len());
    for (m, t) in mv.iter().zip(tv) {
        for key in ["id", "name", "reader", "reads", "question"] {
            assert_eq!(m.get(key), t.get(key).map(dropped).as_ref(), "viewpoints.{key}");
        }
    }

    // 承認欄は空・欄の決まりは ADR-1 が決め、ADR-1 は提案中
    for file in ["srs.yaml", "index.yaml", "intake.yaml", "ceiling.yaml", "graph.yaml"] {
        let approval = w.typed(file).get("meta").and_then(|m| m.get("approval")).cloned();
        assert_eq!(approval, Some(Value::Seq(Vec::new())), "{file}");
    }
    for file in ["adr/schema.yaml", "design-note/schema.yaml"] {
        let decided = w.typed(file).get("meta").and_then(|m| m.get("decided_by")).cloned().unwrap();
        assert_eq!(strs(&decided), ["ADR-1"], "{file}");
    }
    assert_eq!(str_of(&w.typed("adr/ADR-1.yaml"), "status"), "proposed");
}

/// 歯 6（AC27）: 憲法の正本が無い置き場で案内の 1 行は folio init を名指し folio intake を名指さない。init の後は整備済みの 1 行。
#[test]
fn f125_hello_names_init_until_the_skeleton_exists() {
    let w = Work::new("hello");
    let before = w.hello();
    assert_eq!(before.status.code(), Some(0), "{}", both(&before));
    let line = stdout(&before);
    assert_eq!(line.lines().count(), 1, "{line}");
    assert!(line.contains("folio init"), "{line}");
    assert!(!line.contains("folio intake"), "{line}");

    w.init_ok();
    let after = w.hello();
    assert_eq!(after.status.code(), Some(0), "{}", both(&after));
    let line2 = stdout(&after);
    assert_eq!(line2.lines().count(), 1, "{line2}");
    assert!(
        line2.contains("節点") && line2.contains("辺") && line2.contains("folio graph --digest"),
        "{line2}"
    );
    assert_ne!(line2, line);
}

/// 歯 7: 骨格の id の数える範囲に骨格自身の番号でない id が 0 個・folio2 自身には 1 個以上。
#[test]
fn f125_no_folio2_ids_in_the_skeleton() {
    let w = Work::new("ids");
    w.init_ok();
    let mut in_folio2 = 0;
    for file in FILES {
        let found = foreign(&id_range(&w.place(), file));
        assert!(found.is_empty(), "{file}: {found:?}");
        let adr: Vec<String> = ids(&id_range(&w.place(), file))
            .into_iter()
            .filter(|id| id.starts_with("ADR-"))
            .collect();
        assert!(adr.iter().all(|id| id == "ADR-1"), "{file}: {adr:?}");
        in_folio2 += foreign(&id_range(&folio2(), file)).len();
    }
    assert!(in_folio2 > 0, "folio2 自身に 1 個も無い＝数えが効いていない");

    let index = w.typed("index.yaml");
    let rows = seq(index.get("lanes").unwrap(), "rows");
    let adr_stops: Vec<&str> = rows
        .iter()
        .flat_map(|r| seq(r, "stops"))
        .filter(|s| s.get("doc").and_then(Value::as_str) == Some("adr"))
        .map(|s| str_of(s, "at"))
        .collect();
    assert!(!adr_stops.is_empty());
    assert!(adr_stops.iter().all(|at| *at == "ADR-1"), "{adr_stops:?}");
}

/// 歯 8（便 152・群 A の通し）: 根の直下の design-intent に init して commit した後、手直しなしで床・注入・欄の決まり・
/// 組み立て・4 面が答える。便 153（delivery-153.md §1 (c) の 1）: 組み立ては置き場に様式が無いので焼いた様式を出し、
/// 6 file を書いて床の まだ分からない だけで 2・--check は 0・配信先の 4 面の部品の検査は 0（置き場に preview/ を作らない）。
#[test]
fn f152_the_skeleton_runs_every_command_without_hand_edits() {
    let w = Work::new("run-all");
    let place = w.root.join("design-intent");
    let init = folio(&["init"], &place);
    assert_eq!(init.status.code(), Some(0), "{}", both(&init));
    git(&w.root, &["add", "-A"]);
    git(&w.root, &["commit", "-q", "-m", "skeleton"]);

    let check = folio(&["check"], &place);
    assert_eq!(check.status.code(), Some(2), "{}", both(&check));
    assert!(
        stdout(&check).contains("違反 0・まだ分からない 2"),
        "{}",
        both(&check)
    );

    let md = w.root.join("CLAUDE.md");
    fs::write(
        &md,
        "# 一時の CLAUDE.md\n\n<!-- constitution:begin -->\n<!-- constitution:end -->\n",
    )
    .unwrap();
    let md_arg = md.to_str().unwrap();
    for mode in ["--write", "--check"] {
        let inject = folio(&["inject", "--claude-md", md_arg, mode], &place);
        assert_eq!(inject.status.code(), Some(0), "inject {mode}: {}", both(&inject));
    }
    let injected = fs::read_to_string(&md).unwrap();
    assert!(
        injected.contains("P-1.1") && injected.contains("R-2"),
        "{injected}"
    );

    let schema = folio(&["schema", "--check"], &place);
    assert_eq!(schema.status.code(), Some(0), "{}", both(&schema));

    let site = w.root.join("site");
    let build = folio(
        &["build", "--write", "--out", site.to_str().unwrap()],
        &place,
    );
    assert_eq!(build.status.code(), Some(2), "{}", both(&build));
    assert!(
        stdout(&build).contains("床 = まだ分からない（違反 0・まだ分からない 2）"),
        "{}",
        both(&build)
    );
    assert!(stdout(&build).contains("書いた（6 file・"), "{}", both(&build));
    assert!(stderr(&build).is_empty(), "{}", both(&build));
    for name in ["folio.css", "folio-ui.js"] {
        let got = fs::read(site.join(name)).unwrap();
        let want = fs::read(folio2().join("preview").join(name)).unwrap();
        assert!(got == want, "{name} が repo の正本と byte で違う");
    }
    assert!(!place.join("preview").exists(), "置き場に preview/ を作った");
    let build_check = folio(
        &["build", "--check", "--out", site.to_str().unwrap()],
        &place,
    );
    assert_eq!(build_check.status.code(), Some(0), "{}", both(&build_check));

    for (face, id) in [
        ("adr", Some("ADR-1")),
        ("constitution", None),
        ("srs", None),
        ("index", None),
    ] {
        let out = w.root.join(format!("face-{face}.html"));
        let mut args = vec!["face", "--face", face, "--out", out.to_str().unwrap(), "--write"];
        if let Some(id) = id {
            args.extend(["--id", id]);
        }
        let run = folio(&args, &place);
        assert_eq!(run.status.code(), Some(0), "face {face}: {}", both(&run));
        let html = fs::read_to_string(&out).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"), "face {face}");
    }

    let pages: Vec<String> = [
        ("index", "index.html"),
        ("constitution", "constitution.html"),
        ("srs", "srs.html"),
        ("adr", "adr-1.html"),
    ]
    .iter()
    .map(|(face, file)| format!("{face}={}", site.join(file).display()))
    .collect();
    let mut args = vec!["parts", "--check"];
    for page in &pages {
        args.extend(["--page", page.as_str()]);
    }
    let parts = folio(&args, &place);
    assert_eq!(parts.status.code(), Some(0), "{}", both(&parts));
    assert!(stdout(&parts).contains("違反 0"), "{}", both(&parts));
    assert!(!place.join("preview").exists(), "置き場に preview/ を作った");
}

/// 歯 9（便 152・台帳 .196）: 骨格の憲法の雛形は外の憲法を指さず、自分を正本とする形（名 未記入・根拠は空・改訂は表）。
#[test]
fn f152_the_constitution_template_stands_on_its_own() {
    let w = Work::new("constitution");
    w.init_ok();
    let range = id_range(&w.place(), "constitution.yaml");
    for word in ["本来の憲法", "scribe2", "床の受付の宣言", "floor-declaration"] {
        assert!(!range.contains(word), "憲法の雛形に「{word}」");
    }
    let c = w.typed("constitution.yaml");
    assert_eq!(str_of(c.get("meta").unwrap(), "id"), "未記入");

    let precedence = c.get("precedence").unwrap();
    assert!(seq(precedence, "rationale").is_empty(), "前文の根拠");
    assert_eq!(
        str_of(precedence, "text"),
        "段どうしが衝突したら「絶対にやらない ＞ 確認してから ＞ いつも守る」の順で解く。"
    );
    let articles = seq(&c, "articles");
    assert!(!articles.is_empty());
    for article in articles {
        assert!(seq(article, "rationale").is_empty(), "{article:?}");
        for statement in seq(article, "statements") {
            let text = str_of(statement, "text");
            assert!(!text.contains("憲法"), "{text}");
        }
    }

    let amendment = c.get("amendment").unwrap();
    let keys: Vec<&str> = amendment
        .as_map()
        .expect("amendment が表でない")
        .iter()
        .map(|(k, _)| k.as_str().unwrap())
        .collect();
    assert_eq!(keys, ["declaration", "steps", "effective_step"]);
    let step = amendment.get("effective_step").unwrap();
    assert_eq!(step.get("n"), Some(&Value::Int("0".to_string())));
    assert_eq!(str_of(step, "who"), "持ち主");
}

/// 歯 10（便 152）: 骨格の数える範囲（生成区間と憲法の schema の節を除く）に folio2 固有の字が 0 回・folio2 自身には 1 回以上。
#[test]
fn f152_no_folio2_words_in_the_skeleton() {
    let w = Work::new("words");
    w.init_ok();
    let words = ["folio2", "f2-", "scribe2"];
    let mut theirs = [0usize; 3];
    for file in FILES {
        let mine = id_range(&w.place(), file);
        let own = id_range(&folio2(), file);
        for (k, word) in words.iter().enumerate() {
            assert!(!mine.contains(word), "{file} に「{word}」");
            theirs[k] += own.matches(word).count();
        }
    }
    assert!(
        theirs.iter().all(|n| *n > 0),
        "folio2 自身に 0 回の語が在る＝数えが効いていない: {theirs:?}"
    );
}
