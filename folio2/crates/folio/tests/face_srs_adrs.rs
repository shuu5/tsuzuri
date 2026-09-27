//! 要件書の面が要件の行の adrs（判断の記録）を根拠の札に出す歯（便 166・docs/design/delivery-166.md §1 (c)・FR4）。binary 経由。
//! 口 skeleton: 一時 dir の根で git の init → `folio init` の骨格の `requirements: []`（と `nonfunctional: []`）を要件の行に
//! 替える（骨格が counts を持てば行の数に揃える）→ commit。期待する字（hint の札・リンク）は歯の側で組む（P-10.1）。
//! 1. 面の在る判断の記録は根拠の札の中に番号だけのリンクで出る／2. 記録の無い id は「（まだ分からない）」で正本の順のまま／
//! 3. 判断の記録の id の形でない項は床と同じく面を止める。
//!
//! 版管理の下の file は書き換えない（置き場と配信先は必ず一時 dir の中）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

fn folio(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(args)
        .output()
        .expect("folio を起動できない")
}

fn both(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn code(out: &Output, what: &str) -> i32 {
    out.status
        .code()
        .unwrap_or_else(|| panic!("{what} が signal で終わった: {}", both(out)))
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

/// 要件の行 1 本（字の欄は固定・basis / rules / adrs は引数のまま）。
fn row(id: &str, basis: &str, rules: Option<&str>, adrs: &str) -> String {
    let rules = rules.map_or(String::new(), |r| format!("    rules: {r}\n"));
    format!(
        "  - id: {id}\n    title: 判断の記録を根拠に持つ要件\n    pattern: ubiquitous\n    strength: must\n    when: つねに\n    shall: folio は面を出す。\n    plain: 面を出します。\n    goals: []\n    basis: {basis}\n{rules}    adrs: {adrs}\n    verify: {{method: test, how: 面を見て比べる, ac: []}}\n    figures: []\n"
    )
}

/// 一時 dir（歯の終わりに消す）と置き場。
struct Work {
    root: PathBuf,
}

impl Work {
    fn place(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    /// 口 skeleton: git の init → `folio init` → 要件の行（機能 `fr`・非機能 `nfr`）を足す → commit。
    fn skeleton(case: &str, fr: &[String], nfr: &[String]) -> Work {
        let root = std::env::temp_dir().join(format!("folio-f166-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let w = Work { root };
        git(&w.root, &["init", "-q"]);
        let init = folio(&["init", "--dir", s(&w.place())]);
        assert_eq!(code(&init, "folio init"), 0, "{}", both(&init));
        let path = w.place().join("srs.yaml");
        let mut text = fs::read_to_string(&path).unwrap();
        for (key, rows) in [("requirements", fr), ("nonfunctional", nfr)] {
            if rows.is_empty() {
                continue;
            }
            let from = format!("\n{key}: []\n");
            assert_eq!(text.matches(&from).count(), 1, "骨格に「{key}: []」が 1 か所でない");
            text = text.replacen(&from, &format!("\n{key}:\n{}", rows.concat()), 1);
        }
        // 骨格が counts を持つ間は行の数に揃える（持たなければ当たらない）
        text = text.replacen(
            "counts: {fr: 0, nfr: 0,",
            &format!("counts: {{fr: {}, nfr: {},", fr.len(), nfr.len()),
            1,
        );
        fs::write(&path, text).unwrap();
        git(&w.root, &["add", "-A"]);
        git(&w.root, &["commit", "-q", "-m", "fixture"]);
        w
    }

    fn check(&self) -> Output {
        folio(&["check", "--dir", s(&self.place())])
    }

    fn build(&self) -> Output {
        folio(&[
            "build",
            "--dir",
            s(&self.place()),
            "--out",
            s(&self.root.join("site")),
            "--write",
        ])
    }

    fn face(&self) -> Output {
        folio(&[
            "face",
            "--face",
            "srs",
            "--dir",
            s(&self.place()),
            "--out",
            s(&self.root.join("srs.html")),
            "--write",
        ])
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

const CHIP: &str = "<span class=\"hint-btn\">根拠</span></label><span class=\"hint-body\">";

/// 行 `id` の根拠の札の中身（札が無ければ None）。
fn chip<'a>(html: &'a str, id: &str) -> Option<&'a str> {
    let open = format!(" id=\"{}\">", id.to_ascii_lowercase());
    let start = html.find(&open).unwrap_or_else(|| panic!("行 {id} が面に無い"));
    let art = &html[start..];
    let art = &art[..art.find("</article>").unwrap()];
    let body = &art[art.find(CHIP)? + CHIP.len()..];
    Some(&body[..body.find("</span></span>").unwrap()])
}

fn adr_link(id: &str) -> String {
    format!(
        "<a class=\"xref\" href=\"{}.html\">{id}</a>",
        id.to_ascii_lowercase()
    )
}

#[test]
fn f166_adrs_link_to_the_adr_face_in_the_basis_chip() {
    let w = Work::skeleton(
        "link",
        &[
            row("FR1", "[]", None, "[ADR-1]"),
            row("FR2", "[P-1]", Some("[R-2]"), "[ADR-1]"),
        ],
        &[row("NFR1", "[]", None, "[ADR-1]")],
    );
    let check = w.check();
    assert_eq!(code(&check, "folio check"), 2, "{}", both(&check));
    assert!(both(&check).contains("違反 0"), "{}", both(&check));
    let build = w.build();
    assert_eq!(code(&build, "folio build --write"), 2, "{}", both(&build));
    let site = w.root.join("site");
    assert!(site.join("adr-1.html").is_file(), "adr-1.html を書いていない");
    let html = fs::read_to_string(site.join("srs.html")).unwrap();

    let adr = format!("判断の記録 {}", adr_link("ADR-1"));
    for id in ["FR1", "NFR1"] {
        assert_eq!(chip(&html, id), Some(adr.as_str()), "{id} の根拠の札");
    }
    let fr2 = chip(&html, "FR2").expect("FR2 に根拠の札が無い");
    assert!(
        fr2.starts_with("<a class=\"xref\" href=\"constitution.html#p-1\">"),
        "FR2 の札が条のリンクで始まらない: {fr2}"
    );
    let tail = format!(
        "<a class=\"xref\" href=\"constitution.html#r-2\">rules R-2</a>・{adr}"
    );
    assert!(fr2.ends_with(&tail), "FR2 の札の末尾: {fr2}");
    assert_eq!(html.matches("href=\"adr-1.html\"").count(), 3, "adr-1.html へのリンクの数");
    assert_eq!(html.matches(CHIP).count(), 3, "根拠の札の数");
}

#[test]
fn f166_adrs_without_a_record_are_not_yet_known() {
    let w = Work::skeleton("unknown", &[row("FR1", "[]", None, "[ADR-9, ADR-1]")], &[]);
    let check = w.check();
    assert_eq!(code(&check, "folio check"), 1, "{}", both(&check));
    assert!(
        both(&check)
            .contains("[A-2] srs.yaml: requirements[0].adrs[0]: 判断の記録 ADR-9 が実在しない"),
        "{}",
        both(&check)
    );
    let build = w.build();
    assert_eq!(code(&build, "folio build --write"), 1, "{}", both(&build));
    assert!(!w.root.join("site").exists(), "床が不合格なのに配信先を作った");

    let face = w.face();
    assert_eq!(code(&face, "folio face"), 0, "{}", both(&face));
    let html = fs::read_to_string(w.root.join("srs.html")).unwrap();
    let want = format!(
        "判断の記録 ADR-9（まだ分からない）・判断の記録 {}",
        adr_link("ADR-1")
    );
    assert_eq!(chip(&html, "FR1"), Some(want.as_str()), "FR1 の根拠の札");
    assert!(!html.contains("adr-9.html"), "記録の無い ADR-9 をリンクにした");
}

#[test]
fn f166_adrs_outside_the_adr_form_stop_the_face_like_the_floor() {
    for bad in ["P-1", "ADR-01", "R-2"] {
        let w = Work::skeleton(
            &format!("form-{bad}"),
            &[row("FR1", "[]", None, &format!("[ADR-1, {bad}]"))],
            &[],
        );
        let said = format!("adrs「{bad}」が判断の記録の id の形でない");
        let check = w.check();
        assert_eq!(code(&check, "folio check"), 1, "{bad}: {}", both(&check));
        assert!(
            both(&check).contains(&format!("の FR1 の {said}")),
            "{bad}: {}",
            both(&check)
        );
        let face = w.face();
        assert_eq!(code(&face, "folio face"), 2, "{bad}: {}", both(&face));
        assert!(
            both(&face).contains(&format!("requirements[0].adrs[1]: {said}")),
            "{bad}: {}",
            both(&face)
        );
        assert!(!w.root.join("srs.html").exists(), "{bad}: 面を書いた");
    }
}
