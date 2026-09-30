//! 裁定 id の書き出しの歯（便 186・docs/design/delivery-186.md §1 (c)・判断の記録 ADR-31 決定 (4)・要件書 FR26・AC28 の後半）。
//! 土台は凍結した写し（tests/fixtures/floor_base/design-intent/）か folio2 の正本の写し全部を一時 dir の `design-intent/` に、
//! 器の導出 file を写しの根の `contracts/` に作って git の 1 commit にし、字を足すか変えて `folio check --emit-rulings` と
//! 素の `folio check` を撃つ（土台の写しの素の床は合格 0）。期待の行は歯の側の手書きで、土台の数（欄 55・行 67）と行の番号と
//! 欄の道は独立の実装（起草の記録の count-186.py）の数。
#![cfg(test)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use folio::yaml_rust2::{Yaml, YamlLoader};

const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";
const REAL: &str = "design-intent";
const KEYS: [&str; 7] = ["ruling", "form", "bead", "node", "file", "line", "field"];
const THREE: &str = "t3-hub.56:20260927T2259Z-1・f2-648 notes 2026-09-28 07:18 JST（t3-hub.1）";

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
        .args(["-c", "user.email=fx@example", "-c", "user.name=fx", "-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .expect("git を起動できない");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

/// 置き場の写しの一時 dir（根・歯の終わりに消す）。
struct Work(PathBuf);

/// 1 回の起動の結果（終了コード・標準出力の行・標準エラーの字）。
struct Run {
    code: i32,
    out: Vec<String>,
    err: String,
}

impl Work {
    fn new(case: &str, src: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-emit-rulings-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../folio2");
        copy_tree(&repo.join(src), &root.join("design-intent"));
        fs::create_dir_all(root.join("contracts")).unwrap();
        fs::copy(repo.join("contracts/schema.toml"), root.join("contracts/schema.toml")).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "fixture"]);
        Work(root)
    }

    fn dir(&self) -> PathBuf {
        self.0.join("design-intent")
    }

    fn run(&self, flags: &[&str]) -> Run {
        let o = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(["check", "--dir"])
            .arg(self.dir())
            .args(flags)
            .output()
            .expect("folio を起動できない");
        Run {
            code: o.status.code().unwrap(),
            out: String::from_utf8(o.stdout).unwrap().lines().map(str::to_string).collect(),
            err: String::from_utf8(o.stderr).unwrap(),
        }
    }

    fn emit(&self) -> Run {
        self.run(&["--emit-rulings"])
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.dir().join(rel)).unwrap()
    }

    fn write(&self, rel: &str, text: &str) {
        fs::write(self.dir().join(rel), text).unwrap();
    }

    /// `marker` を含むちょうど 1 行の、流れの形の欄 `key` の値を `value` に替える。
    fn set(&self, rel: &str, marker: &str, key: &str, value: &str) {
        let text = self.read(rel);
        let hits: Vec<&str> = text.lines().filter(|l| l.contains(marker)).collect();
        assert_eq!(hits.len(), 1, "{rel}: 「{marker}」の行が 1 つでない");
        let line = hits[0];
        let start = line.find(&format!("{key}: ")).unwrap();
        let v = start + key.len() + 2;
        let end = if line[v..].starts_with('"') {
            v + 2 + line[v + 1..].find('"').unwrap()
        } else {
            v + line[v..].find([',', '}']).unwrap()
        };
        let new = format!("{}{key}: {value}{}", &line[..start], &line[end..]);
        self.write(rel, &text.replacen(line, &new, 1));
    }

    /// 写しの中の全 file の字（書き出しが置き場を書き換えないことを見る）。
    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(&path, out);
                } else {
                    out.insert(path.clone(), fs::read(&path).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(&self.dir(), &mut out);
        out
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 手書きの期待の 1 行（欄の順・空白なし）。
fn line(ruling: &str, form: &str, bead: &str, node: Option<&str>, file: &str, at: usize, field: &str) -> String {
    let node = node.map_or("null".to_string(), |n| format!("\"{n}\""));
    format!("{{\"ruling\":\"{ruling}\",\"form\":\"{form}\",\"bead\":\"{bead}\",\"node\":{node},\"file\":\"{file}\",\"line\":{at},\"field\":\"{field}\"}}")
}

/// JSON の 1 行を欄の名と値の字に割る（値は文字列なら引用符を外した字・null と数はそのまま）。書き出しの値は入れ子を持たない。
fn fields(json: &str) -> Vec<(String, String)> {
    let body = json.strip_prefix('{').and_then(|s| s.strip_suffix('}')).unwrap_or_else(|| panic!("表でない: {json}"));
    let mut out = Vec::new();
    let mut chars = body.chars().peekable();
    let string = |chars: &mut std::iter::Peekable<std::str::Chars>| {
        assert_eq!(chars.next(), Some('"'), "{json}");
        let mut s = String::new();
        while let Some(c) = chars.next() {
            match c {
                '"' => return s,
                '\\' => s.push(chars.next().unwrap()),
                c => s.push(c),
            }
        }
        panic!("字が閉じない: {json}")
    };
    loop {
        let key = string(&mut chars);
        assert_eq!(chars.next(), Some(':'), "{json}");
        let value = if chars.peek() == Some(&'"') {
            string(&mut chars)
        } else {
            let mut v = String::new();
            while chars.peek().is_some_and(|&c| c != ',') {
                v.push(chars.next().unwrap());
            }
            v
        };
        out.push((key, value));
        match chars.next() {
            Some(',') => {}
            None => return out,
            c => panic!("{c:?}: {json}"),
        }
    }
}

/// 欄の道（`a.b[0].c`）で木を引く（床の読み手とは別の yaml-rust2 の高水準の読み手）。
fn follow<'a>(doc: &'a Yaml, path: &str) -> &'a Yaml {
    let mut at = doc;
    for part in path.split('.') {
        let (key, rest) = part.split_once('[').unwrap_or((part, ""));
        at = &at[key];
        for n in rest.split('[').filter(|s| !s.is_empty()) {
            at = &at[n.trim_end_matches(']').parse::<usize>().unwrap()];
        }
    }
    at
}

/// 歯 1（AC28 の後半）: 規則の表の行 R-10 の裁定の欄に器の問いの形・notes の日時の形・台帳の id だけの 3 つを書くと、
/// 書き出しがその 3 つを書かれた字のまま、形の種類をつけて 3 行で出す（ほかの欄の行はそのまま）。
#[test]
fn f186_one_field_with_three_forms_gives_three_lines() {
    let base = Work::new("three-base", FLOOR_BASE).emit();
    assert_eq!(base.out.len(), 70);
    let w = Work::new("three", FLOOR_BASE);
    w.set("rules.yaml", "{id: R-10,", "ruling", &format!("\"{THREE}\""));
    let got = w.emit();
    let r10: Vec<&String> = got.out.iter().filter(|l| l.contains("\"node\":\"R-10\"")).collect();
    let at = |ruling, form, bead| line(ruling, form, bead, Some("R-10"), "rules.yaml", 38, "thresholds[9].ruling");
    assert_eq!(
        r10,
        [
            &at("t3-hub.56:20260927T2259Z-1", "question", "t3-hub.56"),
            &at("f2-648 notes 2026-09-28 07:18 JST", "notes-time", "f2-648"),
            &at("t3-hub.1", "bead", "t3-hub.1"),
        ]
    );
    let was = at("f2-648.1 notes 2026-09-12 20:2x", "notes-time", "f2-648.1");
    let rest: Vec<&String> = got.out.iter().filter(|l| !l.contains("\"node\":\"R-10\"")).collect();
    let before: Vec<&String> = base.out.iter().filter(|l| **l != was).collect();
    assert_eq!((got.out.len(), rest), (72, before));
}

/// 歯 2: どの行も 7 つの欄をこの順で持つ。node は条・規則の表の行・判断の記録の欄では その id、承認欄の行（憲法の発効の
/// 承認・5 正本の stamp・設計ノートの承認欄）と判断の表の行では null。一覧の外の欄（条の前の版との対応 supersedes_v1）は
/// 台帳の id を書いても出さない。
#[test]
fn f186_every_line_has_the_seven_fields_and_the_null_node() {
    let w = Work::new("seven", FLOOR_BASE);
    let c = w.read("constitution.yaml");
    let anchor = "    relations: {reqs: [FR1, FR2, AC1]}\n";
    let add = "    amended_by: [{adr: ADR-2, date: 2026-09-13, ruling: f2-648.2 notes 2026-09-13 09:35}]\n";
    w.write("constitution.yaml", &c.replacen(anchor, &format!("{anchor}{add}"), 1));
    w.set("constitution.yaml", "ruling: \"裁定 #4\"", "ruling", "\"f2-648 notes 2026-09-12 08:55 JST（裁定 #4）\"");
    w.write("graph.yaml", "meta:\n  approval:\n    - {role: 作成, stamp: 起草}\n    - {role: 承認, stamp: \"t3-hub.1（裁定 id = t3-hub.58:20260927T2357Z-1）\"}\n");
    let note = "meta:\n  id: decide\n  title: 判断のノート\n  version: v0.1\n  status: effective\n  generated: 2026-09-28\n  profile: design-note\n  approval:\n    - {who: 持ち主, date: 2026-09-28, ruling: f2-648 notes 2026-09-28 10:29 JST, verbatim: 承認する, surface: R-8}\nsections:\n  - n: 1\n    type: prose\n    title: 目的\n    body: 判断の見本。\n  - n: 2\n    type: decision-table\n    title: 判断\n    rows:\n      - {id: d1, text: 行を足す, ruling: t3-hub.57.1:20260927T2357Z-1}\n";
    w.write("design-note/decide.yaml", note);
    let got = w.emit();
    for want in [
        line("f2-648.1 notes 2026-09-12 20:2x", "notes-time", "f2-648.1", None, "constitution.yaml", 80, "meta.approval.ruling"),
        line("f2-648.2 notes 2026-09-13 09:35", "notes-time", "f2-648.2", Some("P-1"), "constitution.yaml", 130, "articles[0].amended_by[0].ruling"),
        line("f2-648.1 notes 2026-09-12 20:2x", "notes-time", "f2-648.1", Some("D-8"), "rules.yaml", 58, "discipline[7].ruling"),
        line("f2-648.2 notes 2026-09-13 09:35", "notes-time", "f2-648.2", Some("ADR-2"), "adr/ADR-2.yaml", 16, "approval.ruling"),
        line("f2-648 notes 2026-09-28 10:29 JST", "notes-time", "f2-648", None, "design-note/decide.yaml", 9, "meta.approval[0].ruling"),
        line("t3-hub.57.1:20260927T2357Z-1", "question", "t3-hub.57.1", None, "design-note/decide.yaml", 19, "sections[1].rows[0].ruling"),
        line("f2-648.1 notes 20:2x", "notes-time", "f2-648.1", None, "srs.yaml", 22, "meta.approval[2].stamp"),
        line("t3-hub.1", "bead", "t3-hub.1", None, "graph.yaml", 4, "meta.approval[1].stamp"),
        line("t3-hub.58:20260927T2357Z-1", "question", "t3-hub.58", None, "graph.yaml", 4, "meta.approval[1].stamp"),
    ] {
        assert!(got.out.contains(&want), "{want}\n{:#?}", got.out);
    }
    assert_eq!(got.out.len(), 70 + 5);
    assert!(!got.out.iter().any(|l| l.contains("supersedes_v1") || l.contains("08:55")));
    for l in &got.out {
        let f = fields(l);
        assert_eq!(f.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(), KEYS, "{l}");
        let field = &f[6].1;
        let held = field.starts_with("articles[") || field.starts_with("thresholds[") || field.starts_with("discipline[") || field == "approval.ruling";
        assert_eq!(f[3].1 != "null", held, "{l}");
        assert!(["question", "notes-time", "bead"].contains(&f[1].1.as_str()), "{l}");
        assert!(f[0].1.starts_with(&f[2].1) && f[5].1.parse::<usize>().unwrap() > 0, "{l}");
    }
}

/// 歯 3: file・line・field が正本の所在を指す（土台と folio2 の正本）。field を別の読み手で引くと裁定 id を含む字で、
/// line の行は欄の鍵を持つ。node は field の行の id（条・規則の表の行）か判断の記録の id。土台では裁定 id も line の行に在る。
#[test]
fn f186_file_line_and_field_point_at_the_source() {
    for (src, same_line) in [(FLOOR_BASE, true), (REAL, false)] {
        let w = Work::new("where", src);
        let got = w.emit();
        assert!(!got.out.is_empty(), "{src}");
        let mut docs: BTreeMap<String, (Vec<String>, Yaml)> = BTreeMap::new();
        for l in &got.out {
            let f: BTreeMap<String, String> = fields(l).into_iter().collect();
            let (text, doc) = docs.entry(f["file"].clone()).or_insert_with(|| {
                let t = w.read(&f["file"]);
                let d = YamlLoader::load_from_str(&t).unwrap().remove(0);
                (t.lines().map(str::to_string).collect(), d)
            });
            let field = &f["field"];
            let value = follow(doc, field).as_str().unwrap_or_else(|| panic!("{src}: {l}"));
            assert!(value.contains(&f["ruling"]), "{src}: {l}");
            let at = &text[f["line"].parse::<usize>().unwrap() - 1];
            let key = field.rsplit('.').next().unwrap();
            assert!(at.contains(&format!("{key}: ")), "{src}: {l}");
            if same_line {
                assert!(at.contains(&f["ruling"]), "{src}: {l}");
            }
            let holder = field.rsplit_once('.').unwrap().0;
            let id = match holder.split_once(".amended_by") {
                Some((article, _)) => follow(doc, &format!("{article}.id")).as_str(),
                None if holder == "approval" => doc["id"].as_str(),
                None if f["node"] != "null" => follow(doc, &format!("{holder}.id")).as_str(),
                None => None,
            };
            assert_eq!(id.unwrap_or("null"), f["node"], "{src}: {l}");
        }
    }
}

/// 歯 4: 終了コードは素の床と同じ（合格 0・違反 1・骨格の印の まだ分からない 2・索引の床〔folio graph が組めるか〕だけが
/// 落ちる写しも 1）で、標準出力は JSON の行だけ（「#」の注・違反・要約は標準エラー）。違反のある写しでも切り出せた欄の行は
/// 出る。書き出しは置き場を書き換えない。旗は他の旗と同時に撃てない。
#[test]
fn f186_the_exit_code_is_the_plain_floor_and_stdout_is_json_only() {
    let json = |r: &Run| r.out.iter().all(|l| l.starts_with("{\"ruling\":\"") && l.ends_with("\"}") && !l.starts_with('#'));
    let base = Work::new("rc-base", FLOOR_BASE);
    let before = base.snapshot();
    let (plain, emit) = (base.run(&[]), base.emit());
    assert_eq!((plain.code, emit.code), (0, 0));
    assert_eq!(base.snapshot(), before);
    assert!(json(&emit) && emit.out.len() == 70, "{:?}", emit.out);
    let summary = plain.out.last().unwrap();
    assert_eq!(summary, "folio check: 合格（違反 0・まだ分からない 0）");
    assert!(emit.err.lines().any(|l| l == summary) && emit.err.lines().any(|l| l.starts_with("# ")), "{}", emit.err);
    assert!(!plain.out.iter().any(|l| l.starts_with('{')));

    let w = Work::new("rc-violation", FLOOR_BASE);
    w.set("rules.yaml", "{id: R-10,", "ruling", "G16=A（受入 (f)）");
    let (plain, emit) = (w.run(&[]), w.emit());
    assert_eq!((plain.code, emit.code), (1, 1));
    let violation = "[裁定 id] rules.yaml: 行 R-10 の ruling「G16=A（受入 (f)）」に台帳 id が無い（決定の欄・形は adr/schema.yaml の ruling_pattern）";
    assert!(plain.out.iter().any(|l| l == violation), "{:?}", plain.out);
    assert!(emit.err.lines().any(|l| l == violation) && emit.err.lines().any(|l| l == plain.out.last().unwrap()));
    assert!(json(&emit) && emit.out.len() == 69 && !emit.out.iter().any(|l| l.contains("\"node\":\"R-10\"")));

    let w = Work::new("rc-mark", FLOOR_BASE);
    w.set("rules.yaml", "{id: R-10,", "ruling", "未記入");
    let (plain, emit) = (w.run(&[]), w.emit());
    assert_eq!((plain.code, emit.code, emit.out.len()), (2, 2, 69));

    // 索引の床だけが落ちる写し（要件 FR1 の id を単引用符にすると、索引が行の逐語で切れない）
    let w = Work::new("rc-index", FLOOR_BASE);
    let (from, to) = ("\n  - id: FR1\n", "\n  - id: 'FR1'\n");
    let srs = w.read("srs.yaml");
    assert_eq!(srs.matches(from).count(), 1);
    w.write("srs.yaml", &srs.replacen(from, to, 1));
    let (plain, emit) = (w.run(&[]), w.emit());
    assert_eq!((plain.code, emit.code, emit.out.len()), (1, 1, 70));
    let index: Vec<&String> = plain.out.iter().filter(|l| l.starts_with('[')).collect();
    assert!(index.len() == 1 && index[0].starts_with("[索引の節点] srs.yaml: 索引の節点 FR1 の行を"), "{index:?}");
    assert!(json(&emit) && emit.err.lines().any(|l| l == index[0]), "{}", emit.err);

    let real = Work::new("rc-real", REAL);
    let (plain, emit) = (real.run(&[]), real.emit());
    assert_eq!(plain.code, emit.code);
    assert!(json(&emit) && !emit.out.is_empty());

    for other in ["--emit-amends", "--freeze-anchor", "--freeze-ids", "--freeze-start", "--freeze-adrs"] {
        let r = base.run(&["--emit-rulings", other]);
        assert!(r.code != 0 && r.out.is_empty() && r.err.contains("cannot be used with"), "{other}: {}", r.err);
    }
}

/// 決定の欄を持つ file の、語頭の台帳の id の頭の字を全部大字にする（歯の側の別の走査）。
fn upper_heads(text: &str) -> String {
    let mut b = text.as_bytes().to_vec();
    for i in 0..b.len().saturating_sub(3) {
        let head = i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b"_-.".contains(&b[i - 1]));
        let id = b[i].is_ascii_lowercase() && b[i + 1].is_ascii_digit() && b[i + 2] == b'-';
        if head && id && (b[i + 3].is_ascii_lowercase() || b[i + 3].is_ascii_digit()) {
            b[i] = b[i].to_ascii_uppercase();
        }
    }
    String::from_utf8(b).unwrap()
}

/// 歯 5: 書き出しの欄（file と field の組）は床が数える決定の欄と同じ母集団で全数。台帳の id の頭を全部大字にした写しで、
/// 種別 裁定 id の違反は file ごとに書き出しの欄の数と同じ（土台と folio2 の正本）。土台の数は独立の実装の数（欄 55・
/// 行 67 = notes-time 46・bead 21・node が null の行 18）。
#[test]
fn f186_every_decision_field_is_written_out() {
    for src in [FLOOR_BASE, REAL] {
        let w = Work::new("all", src);
        let got = w.emit();
        let mut fields_of: BTreeMap<String, usize> = BTreeMap::new();
        let mut seen = std::collections::BTreeSet::new();
        for l in &got.out {
            let f: BTreeMap<String, String> = fields(l).into_iter().collect();
            if seen.insert((f["file"].clone(), f["field"].clone())) {
                *fields_of.entry(f["file"].clone()).or_default() += 1;
            }
        }
        if src == FLOOR_BASE {
            let count = |p: &str| got.out.iter().filter(|l| l.contains(p)).count();
            assert_eq!(
                (seen.len(), got.out.len(), count("\"form\":\"notes-time\""), count("\"form\":\"bead\""), count("\"node\":null")),
                (58, 70, 49, 21, 18)
            );
        }
        let mut files: Vec<String> = ["constitution", "rules", "srs", "index", "ceiling", "intake", "graph"].map(|f| format!("{f}.yaml")).to_vec();
        for dir in ["adr", "design-note"] {
            for e in fs::read_dir(w.dir().join(dir)).unwrap() {
                let name = e.unwrap().file_name().into_string().unwrap();
                if name.ends_with(".yaml") && name != "schema.yaml" {
                    files.push(format!("{dir}/{name}"));
                }
            }
        }
        for f in files.iter().filter(|f| w.dir().join(f).exists()) {
            w.write(f, &upper_heads(&w.read(f)));
        }
        let mut violations: BTreeMap<String, usize> = BTreeMap::new();
        for l in w.run(&[]).out.iter().filter_map(|l| l.strip_prefix("[裁定 id] ")) {
            *violations.entry(l.split(": ").next().unwrap().to_string()).or_default() += 1;
        }
        assert_eq!(violations, fields_of, "{src}");
        assert!(w.emit().out.is_empty(), "{src}");
    }
}
