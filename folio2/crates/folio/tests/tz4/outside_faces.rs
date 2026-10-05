//! 外の置き場の面の歯（便 165・docs/design/delivery-165.md §1 (c) の 1〜3・6・7・FR4 / FR22）。binary 経由。
//! 床は合格なのに面だけが止まっていた 4 か所（要件書の meta.counts・meta.promise・範囲の節 scope / scope_m1・憲法の
//! amendment が字）を、面の側で寛容にしたことを測る。在るのに形が違う欄は今どおり まだ分からない（保つ歯）。
//! fixture の dir を足さない: 口 `Place` が一時 dir の根で git の init をし、`folio init --dir <根>/design-intent` の骨格を
//! 書き、字を当てて commit し、素の check（骨格と同じ床 = 違反 0・まだ分からない 6〔便 181 で骨格の決定の欄 4 つ〕）と build --write を撃つ。
//! 字の amendment の改訂の例と版ごとの変更点は面の凍結 fixture（tests/fixtures/face/）の写しで測る。
//! 版管理の下の正本と面は書き換えない（写しと配信先は必ず一時 dir の中）。
#![cfg(test)]

use crate::common::{both, copy_dir, fixture};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-outside-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

/// git を呼ぶ。環境変数 GIT_* は継承しない（tests/tz3/init.rs と同じ形）。
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
    Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(args)
        .arg("--dir")
        .arg(place)
        .output()
        .expect("folio を起動できない")
}

/// 5 字の escape（生成側の字面を使わず歯の側で持つ）。
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// dir の直下の file の数（dir が無ければ 0）。
fn files_in(dir: &Path) -> usize {
    fs::read_dir(dir).map_or(0, |entries| entries.count())
}

/// `line`（改行を含む 1 行）をちょうど 1 つ落とす。
fn drop_lines(text: &str, lines: &[&str]) -> String {
    let mut out = text.to_string();
    for line in lines {
        assert_eq!(out.matches(line).count(), 1, "当て先「{line}」が 1 つでない");
        out = out.replacen(line, "", 1);
    }
    out
}

/// 最上位の節 `key` を（次の最上位の鍵の手前まで）`with` に替える。
fn replace_section(text: &str, key: &str, with: &str) -> String {
    let head = format!("\n{key}:");
    assert_eq!(text.matches(&head).count(), 1, "写しに {key} の節が 1 つでない");
    let start = text.find(&head).unwrap() + 1;
    let rest = &text[start..];
    let mut end = rest.find('\n').unwrap() + 1;
    for line in rest[end..].split_inclusive('\n') {
        if !line.trim().is_empty() && !line.starts_with(' ') {
            break;
        }
        end += line.len();
    }
    format!("{}{with}{}", &text[..start], &rest[end..])
}

/// 骨格の要件書に 目標・要件・受入基準 を 1 つずつ書く（床は骨格と同じ・面は導出できる最小の形）。
fn one_requirement(srs: &str) -> String {
    let mut out = srs.to_string();
    for (from, to) in [
        (
            "goals: []\n",
            "goals:\n  - {id: GOAL1, title: 目標, text: 目標の文。}\n",
        ),
        (
            "requirements: []\n",
            "requirements:\n  - id: FR1\n    title: 結果を返す\n    pattern: ubiquitous\n    strength: must\n    when: つねに\n    shall: 道具は結果を返す。\n    plain: 結果を返します。\n    basis: [P-1]\n    verify: {method: test, how: 動かして比べる, ac: [AC1]}\n    goals: [GOAL1]\n    figures: [図1]\n",
        ),
        (
            "acceptance: []\n",
            "acceptance:\n  - {id: AC1, title: 結果が出る, plain: 結果が出ます。, verifies: [FR1], red_test: {sentence: 結果が出る, fixture: tests/fixtures/one.yaml}}\n",
        ),
    ] {
        assert_eq!(out.matches(from).count(), 1, "骨格に「{from}」が 1 つでない");
        out = out.replacen(from, to, 1);
    }
    out
}

/// 一時 dir の根（git の init 済み）と、その下の置き場 design-intent（`folio init` の骨格）。
struct Place {
    root: PathBuf,
}

impl Place {
    fn new(case: &str) -> Place {
        let root = temp_dir(case);
        git(&root, &["init", "-q"]);
        let place = Place { root };
        let init = folio(&["init"], &place.dir());
        assert_eq!(init.status.code(), Some(0), "{case}: {}", both(&init));
        place
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn read(&self, file: &str) -> String {
        fs::read_to_string(self.dir().join(file)).unwrap()
    }

    /// 骨格の `file` に字を当てる（当たらなければ落ちる）。
    fn edit(&self, file: &str, f: impl FnOnce(&str) -> String) {
        let before = self.read(file);
        let after = f(&before);
        assert_ne!(before, after, "変異が当たっていない: {file}");
        fs::write(self.dir().join(file), after).unwrap();
    }

    /// commit して素の check が骨格と同じ床（違反 0・まだ分からない 6）で答えることを確かめる。
    fn commit(&self, what: &str) {
        git(&self.root, &["add", "-A"]);
        git(&self.root, &["commit", "-q", "--allow-empty", "-m", what]);
        let check = folio(&["check"], &self.dir());
        assert_eq!(check.status.code(), Some(2), "{what}: {}", both(&check));
        assert!(
            both(&check).contains("違反 0・まだ分からない 6"),
            "{what}: 床が骨格と違う: {}",
            both(&check)
        );
    }

    fn build(&self, site: &str) -> (Output, PathBuf) {
        let out = self.root.join(site);
        let run = folio(&["build", "--write", "--out", out.to_str().unwrap()], &self.dir());
        (run, out)
    }

    /// build --write が床の まだ分からない だけで 2 を返し 6 file を書く。配信先を返す。
    fn built(&self, site: &str) -> PathBuf {
        let (run, out) = self.build(site);
        assert_eq!(run.status.code(), Some(2), "{site}: {}", both(&run));
        assert!(
            both(&run).contains("書いた（6 file・"),
            "{site}: {}",
            both(&run)
        );
        assert_eq!(files_in(&out), 6, "{site}: 配信先の file が 6 でない");
        out
    }

    /// build --write が まだ分からない（`said` を含む）で 2 を返し、配信先に 1 file も書かない。
    fn stop(&self, site: &str, said: &str) {
        let (run, out) = self.build(site);
        let told = both(&run);
        assert_eq!(run.status.code(), Some(2), "{site}: {told}");
        assert!(told.contains("まだ分からない"), "{site}: {told}");
        assert!(told.contains(said), "{site}: 「{said}」が無い: {told}");
        assert_eq!(files_in(&out), 0, "{site}: 導出できないのに配信先に書いた");
    }
}

impl Drop for Place {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn page(site: &Path, file: &str) -> String {
    fs::read_to_string(site.join(file)).unwrap_or_else(|e| panic!("{file} が読めない: {e}"))
}

/// 章 `n`（帯 s<n> から次の帯の前まで）。
fn chapter(html: &str, n: usize) -> &str {
    let start = html
        .find(&format!("<section id=\"s{n}\""))
        .unwrap_or_else(|| panic!("章 {n} が無い"));
    let end = html
        .find(&format!("<section id=\"s{}\"", n + 1))
        .unwrap_or_else(|| panic!("章 {} が無い", n + 1));
    &html[start..end]
}

// ── 要件書（(c) の 1・2）──

const PROMISE_LAB: &str = "この文書が約束すること（1 文）";
const SCOPE_H3: &str = "<h3>作るもの / 作らないもの</h3>";

/// 表紙の件数の札（歯の側の手書き）。
fn cover_count(name: &str, href: &str, text: &str) -> String {
    format!("<span class=\"m\"><span class=\"k\">{name}</span><span class=\"v\"><a href=\"#{href}\">{text}</a></span></span>")
}

#[test]
fn f165_srs_without_counts_promise_and_scope_builds() {
    let p = Place::new("f165-srs");
    let srs = p.read("srs.yaml");
    assert!(!srs.contains("counts"), "骨格の要件書に counts の行が在る");
    p.commit("skeleton");
    // 緑の対: 骨格のままなら要約の札・小見出し・M0 と M1 の card が 1 つずつ
    let site = p.built("site-skeleton");
    let html = page(&site, "srs.html");
    for want in [
        PROMISE_LAB,
        SCOPE_H3,
        "<div class=\"cid\">M0 で作る</div>",
        "<div class=\"cid\">M1 で作る</div>",
    ] {
        assert_eq!(html.matches(want).count(), 1, "骨格の面に「{want}」が 1 つでない");
    }

    p.edit("srs.yaml", |t| {
        drop_lines(
            t,
            &[
                "  promise: 未記入\n",
                "scope: {build: [], not_build: []}\n",
                "scope_m1: {build: [], not_build: []}\n",
            ],
        )
    });
    p.commit("drop promise and scope");
    let site = p.built("site-dropped");
    let html = page(&site, "srs.html");
    for gone in [PROMISE_LAB, "summary-card", SCOPE_H3, "で作る</div>", "では作らない"] {
        assert_eq!(html.matches(gone).count(), 0, "消した欄の面に「{gone}」が在る");
    }
    assert_eq!(
        html.matches(&cover_count("機能要件", "s3", "0 件")).count(),
        1,
        "表紙の機能要件が 0 件でない"
    );
    let index = page(&site, "index.html");
    assert_eq!(
        index.matches("機能 0 · 非機能 0 · 受入基準 0").count(),
        1,
        "入口の要件書のカードが数えた数でない"
    );
}

#[test]
fn f165_counts_are_counted_not_read() {
    let p = Place::new("f165-counts");
    p.edit("srs.yaml", one_requirement);
    p.commit("one requirement");
    let promise = "  promise: 未記入\n";
    let mut faces = Vec::new();
    for (case, counts) in [
        ("none", None),
        ("shifted", Some("  counts: {fr: 0, nfr: 5, ac: 0, con: 5}\n")),
        ("matching", Some("  counts: {fr: 1, nfr: 0, ac: 1, con: 0}\n")),
    ] {
        if let Some(line) = counts {
            p.edit("srs.yaml", |t| {
                let without = t
                    .lines()
                    .filter(|l| !l.starts_with("  counts: "))
                    .map(|l| format!("{l}\n"))
                    .collect::<String>();
                without.replacen(promise, &format!("{promise}{line}"), 1)
            });
            p.commit(case);
        }
        let site = p.built(&format!("site-{case}"));
        faces.push((case, page(&site, "srs.html"), page(&site, "index.html")));
    }
    let (_, srs, index) = &faces[0];
    for want in [
        cover_count("機能要件", "s3", "1 件（FR1–FR1）"),
        cover_count("受入基準", "s5", "1 件（AC1–AC1）"),
    ] {
        assert_eq!(srs.matches(&want).count(), 1, "表紙に「{want}」が無い");
    }
    assert_eq!(
        index.matches("機能 1 · 非機能 0 · 受入基準 1").count(),
        1,
        "入口の要件書のカードが数えた数でない"
    );
    for (case, s, i) in &faces[1..] {
        assert!(s == srs, "{case}: 要件書の面が counts の無い面と byte で違う");
        assert!(i == index, "{case}: 入口の面が counts の無い面と byte で違う");
    }
}

// ── 憲法の amendment が字（(c) の 3・7）──

/// 散文の 2 行（「<」「&」と ASCII の二重引用符を含む）。
const PROSE_1: &str = "改訂は \"持ち主\" が承認し & 版 <v1.1> を上げる。";
const PROSE_2: &str = "判断の記録を 1 本起こしてから条を書き換える。";

/// 空行を挟む 2 行の散文の amendment の節。
fn prose_amendment() -> String {
    format!("amendment: |\n  {PROSE_1}\n\n  {PROSE_2}\n")
}

#[test]
fn f165_prose_amendment_is_the_chapter_body() {
    let p = Place::new("f165-prose");
    p.commit("skeleton");
    // 緑の対: 骨格のまま（表・段 0）なら図 1 と「 — 0 段」
    let site = p.built("site-table");
    let html = page(&site, "constitution.html");
    assert_eq!(html.matches("id=\"fig-amend-flow\"").count(), 1, "表の面に図 1 が無い");
    assert_eq!(
        html.matches("<h2>変えるときの手続き — 0 段</h2>").count(),
        1,
        "表の面の章 06 の h2 が 0 段でない"
    );

    p.edit("constitution.yaml", |t| replace_section(t, "amendment", &prose_amendment()));
    p.commit("prose amendment");
    let site = p.built("site-prose");
    let html = page(&site, "constitution.html");
    let ch = chapter(&html, 6);
    let (p1, p2) = (format!("<p>{}</p>", esc(PROSE_1)), format!("<p>{}</p>", esc(PROSE_2)));
    assert!(p1.contains("&quot;") && p1.contains("&amp;") && p1.contains("&lt;"), "{p1}");
    for want in [&p1, &p2] {
        assert_eq!(html.matches(want.as_str()).count(), 1, "散文の行「{want}」が 1 回でない");
        assert_eq!(ch.matches(want.as_str()).count(), 1, "散文の行「{want}」が章 06 に無い");
    }
    assert_eq!(ch.matches("<p>").count(), 2, "章 06 の p が 2 つでない: {ch}");
    assert!(
        ch.contains(&format!("{p1}\n{p2}\n</div>\n")),
        "章 06 が散文の 2 行の後で閉じない: {ch}"
    );
    assert!(ch.contains("<h2>変えるときの手続き</h2>"), "章 06 の h2 が段の数の無い字でない: {ch}");
    assert!(!ch.contains("<p class=\"lead\">"), "章 06 に lead が在る: {ch}");
    assert!(
        html.contains("<span class=\"t\">変えるときの手続き</span>"),
        "目次の 06 の字が帯の h2 と違う"
    );
    for gone in ["fig-amend-flow", "data-component=\"stepper\"", "変えるときの手続き — "] {
        assert_eq!(html.matches(gone).count(), 0, "散文の面に「{gone}」が在る");
    }
}

#[test]
fn f165_prose_amendment_keeps_the_examples_and_changes() {
    let td = temp_dir("f165-examples");
    let work = td.join("src");
    copy_dir(&fixture(), &work);
    let path = work.join("constitution.yaml");
    let before = fs::read_to_string(&path).unwrap();
    fs::write(&path, replace_section(&before, "amendment", &prose_amendment())).unwrap();
    let out = td.join("constitution.html");
    let run = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["face", "--face", "constitution", "--dir"])
        .arg(&work)
        .arg("--out")
        .arg(&out)
        .arg("--write")
        .output()
        .expect("folio を起動できない");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(run.status.code(), Some(0), "{}", both(&run));

    let frozen = fs::read_to_string(fixture().join("expected.html")).unwrap();
    let (ch, fch) = (chapter(&html, 6), chapter(&frozen, 6));
    // 散文の行の後 = 凍結 fixture の段の一覧の後（改訂の例・版ごとの変更点・章の閉じ）
    let last = format!("<p>{}</p>\n", esc(PROSE_2));
    let after = &ch[ch.find(&last).expect("散文の最後の行が無い") + last.len()..];
    let stepper = fch.find("<ol data-component=\"stepper\">").expect("凍結 fixture に段の一覧が無い");
    let fafter = &fch[stepper..];
    let fafter = &fafter[fafter.find("</ol>\n").unwrap() + "</ol>\n".len()..];
    assert!(fafter.contains("data-component=\"amendment-example\""), "凍結 fixture に改訂の例が無い");
    assert!(fafter.contains("からの変更（"), "凍結 fixture に版ごとの変更点が無い");
    assert_eq!(after, fafter, "散文の行の後が凍結 fixture の段の一覧の後と違う");
    // 章 06 の外は目次の 06 の行の段の数のほかは凍結 fixture と同じ
    let toc = |t: &str| format!("<span class=\"t\">変えるときの手続き{t}</span>");
    let outside = html.replacen(ch, "", 1);
    let foutside = frozen.replacen(fch, "", 1).replacen(&toc(" — 2 段"), &toc(""), 1);
    assert!(outside == foutside, "章 06 の外が凍結 fixture と違う");
}

// ── 在るのに形が違う欄（(c) の 6・保つ歯）──

#[test]
fn f165_a_present_field_in_a_wrong_shape_still_stops() {
    // (写しの名, file, 当てる前の行, 当てた後の行, まだ分からない の字)
    let cases = [
        (
            "promise",
            "srs.yaml",
            "  promise: 未記入\n",
            "  promise: [未記入]\n",
            "srs.yaml.meta.promise: 文字列でない",
        ),
        (
            "scope",
            "srs.yaml",
            "scope: {build: [], not_build: []}\n",
            "scope: 未記入\n",
            "srs.yaml.scope: 表でない",
        ),
        (
            "scope_m1",
            "srs.yaml",
            "scope_m1: {build: [], not_build: []}\n",
            "scope_m1: [未記入]\n",
            "srs.yaml.scope_m1: 表でない",
        ),
    ];
    for (case, file, from, to, said) in cases {
        let p = Place::new(&format!("f165-shape-{case}"));
        p.edit(file, |t| {
            assert_eq!(t.matches(from).count(), 1, "{case}: 当て先が 1 つでない");
            t.replacen(from, to, 1)
        });
        p.commit(case);
        p.stop("site", said);
    }
    let p = Place::new("f165-shape-amendment");
    p.edit("constitution.yaml", |t| {
        replace_section(t, "amendment", "amendment:\n  - 未記入\n")
    });
    p.commit("amendment");
    p.stop("site", "constitution.yaml.amendment: 表でない");
}
