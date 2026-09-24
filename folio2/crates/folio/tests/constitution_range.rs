//! 便 122（docs/design/delivery-122.md §1 (c)・FR25・ADR-16 決定 (2)(ウ)）: 置き場の憲法の値域の節の各鍵が、道具を組み立てた版の
//! 同じ鍵の値の部分集合かを床が数え、条の値を置き場の値域で引くことの歯。
//! 土台は design-intent の写し（git の 1 commit・tests/check.rs の Work と同じ形）で、写しの constitution.yaml の字面に変異を当てて
//! `folio check` を撃つ。値域の節は憲法の改訂の差分の範囲（schema）に入るので、値域を変える変異では改憲の違反（種別 N-4）がちょうど
//! 1 件出る。歯は N-4 がちょうど 1 件であることを確かめてから、それを除いた違反の行と「まだ分からない」の行を全部数える。
//! 凍結 anchor（P-10.1）は手書きの tests/fixtures/check/enum-range-anchor.yaml（組み立てた値域 10 鍵）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

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

/// 値域の節の見出しの行（前の改行を含む）。
const ENUMS_HEAD: &str = "\n  enums:\n";
/// 条 P-1 の機構の頭（note の頭まで・写しの中で 1 か所）。
const P1_MECHANISM: &str =
    "{kind: build-check, live: now, stage: post, polarity: fail-closed, note: 公開する命令";
/// 同じ頭の live を両方の値域の外の値にしたもの。
const P1_MECHANISM_ZZ: &str =
    "{kind: build-check, live: zz-live, stage: post, polarity: fail-closed, note: 公開する命令";
/// 違反の行の頭（値域の違反・便 55 の字）。
const RANGE_HIT: &str = "が憲法の値域 schema.enums.";

/// design-intent の写しの一時 dir（歯の終わりに消す）。器（scribe2）の導出 file は写しの根の contracts/ に置く。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str) -> Work {
        let root =
            std::env::temp_dir().join(format!("folio-range-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(
            &repo_root().join("design-intent"),
            &root.join("design-intent"),
        );
        fs::create_dir_all(root.join("contracts")).unwrap();
        fs::copy(
            repo_root().join("contracts/schema.toml"),
            root.join("contracts/schema.toml"),
        )
        .unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "fixture"]);
        Work { root }
    }

    fn constitution(&self) -> PathBuf {
        self.root.join("design-intent/constitution.yaml")
    }

    fn text(&self) -> String {
        fs::read_to_string(self.constitution()).unwrap()
    }

    /// 写しの constitution.yaml の字面の変異（当て先はちょうど 1 か所）。
    fn mutate(&self, from: &str, to: &str) {
        let before = self.text();
        assert_eq!(
            before.matches(from).count(),
            1,
            "変異の当て先が 1 か所でない: {from:?}"
        );
        fs::write(self.constitution(), before.replacen(from, to, 1)).unwrap();
    }

    /// 値域の節の中身（見出しの次の行から、4 字下げの行が続く限り）の位置。
    fn enums_span(&self) -> (usize, usize, usize) {
        let text = self.text();
        assert_eq!(text.matches(ENUMS_HEAD).count(), 1, "値域の節の見出しが 1 つでない");
        let head = text.find(ENUMS_HEAD).unwrap();
        let start = head + ENUMS_HEAD.len();
        let mut end = start;
        for line in text[start..].split_inclusive('\n') {
            if !line.starts_with("    ") {
                break;
            }
            end += line.len();
        }
        (head + 1, start, end)
    }

    /// 値域の節の中身を `body`（4 字下げ済み・行ごとに改行）に差し替える。
    fn set_enums(&self, body: &str) {
        let (_, start, end) = self.enums_span();
        let text = self.text();
        fs::write(
            self.constitution(),
            format!("{}{body}{}", &text[..start], &text[end..]),
        )
        .unwrap();
    }

    /// 値域の節を見出しごと消す。
    fn drop_enums(&self) {
        let (head, _, end) = self.enums_span();
        let text = self.text();
        fs::write(
            self.constitution(),
            format!("{}{}", &text[..head], &text[end..]),
        )
        .unwrap();
    }

    fn check(&self) -> Run {
        let out = Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("check")
            .arg("--dir")
            .arg(self.root.join("design-intent"))
            .output()
            .expect("folio を起動できない");
        Run::of(&out)
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// `folio check` の 1 回の結果を数えたもの。
struct Run {
    code: Option<i32>,
    /// 改憲の違反（種別 N-4）の行。
    n4: Vec<String>,
    /// N-4 を除いた違反の行。
    others: Vec<String>,
    /// 「まだ分からない」の行（接頭辞を剥がした字）。
    pendings: Vec<String>,
    /// 判定の行。
    verdict: String,
    all: String,
}

impl Run {
    fn of(out: &Output) -> Run {
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        let violations: Vec<String> = stdout
            .lines()
            .filter(|l| l.starts_with('['))
            .map(str::to_string)
            .collect();
        Run {
            code: out.status.code(),
            n4: violations
                .iter()
                .filter(|l| l.starts_with("[N-4] "))
                .cloned()
                .collect(),
            others: violations
                .iter()
                .filter(|l| !l.starts_with("[N-4] "))
                .cloned()
                .collect(),
            pendings: stderr
                .lines()
                .filter_map(|l| l.strip_prefix("# まだ分からない: "))
                .map(str::to_string)
                .collect(),
            verdict: stdout
                .lines()
                .find(|l| l.starts_with("folio check: "))
                .unwrap_or("")
                .to_string(),
            all: format!("{stdout}{stderr}"),
        }
    }

    /// 改憲の違反がちょうど 1 件（値域の節の変更）。
    fn one_amendment(&self) {
        assert_eq!(self.n4.len(), 1, "N-4 がちょうど 1 件のはず: {}", self.all);
    }

    /// 値域の違反（便 55 の字）の行。
    fn range_hits(&self) -> Vec<&String> {
        self.others
            .iter()
            .filter(|l| l.contains(RANGE_HIT))
            .collect()
    }
}

/// 凍結 anchor の行（注と空行を除く）。
fn anchor_lines() -> Vec<String> {
    fs::read_to_string(repo_root().join("tests/fixtures/check/enum-range-anchor.yaml"))
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// 行の一覧を 4 字下げて値域の節の中身の字にする。
fn indent(lines: &[String]) -> String {
    lines.iter().map(|l| format!("    {l}\n")).collect()
}

fn built_outside(key: &str, values: &str) -> String {
    format!(
        "constitution.yaml: schema.enums.{key}: 組み立て時の値域に無い値がある（{values}・値域を置き場ごとに広げる口は無い・FR25）"
    )
}

/// 歯 1: 部分集合なら床が続き、条の値は置き場の値域で引かれる。
#[test]
fn f122_subset_range_is_silent_and_article_values_are_looked_up_in_the_place_range() {
    let w = Work::new("subset");
    w.mutate(
        "mechanism_live: [now, M0, delivery-0, M1, adr]",
        "mechanism_live: [adr, M1, delivery-0, now]",
    );
    let r = w.check();
    r.one_amendment();
    assert!(r.others.is_empty(), "{}", r.all);
    assert!(r.pendings.is_empty(), "{}", r.all);
    assert_eq!(r.verdict, "folio check: 不合格（違反 1・まだ分からない 0）", "{}", r.all);

    w.mutate(
        P1_MECHANISM,
        "{kind: build-check, live: M0, stage: post, polarity: fail-closed, note: 公開する命令",
    );
    let r = w.check();
    r.one_amendment();
    assert_eq!(
        r.others,
        ["[schema] constitution.yaml: 条 P-1 の mechanism.live の値「M0」が憲法の値域 schema.enums.mechanism_live に無い"],
        "{}",
        r.all
    );
    assert!(r.pendings.is_empty(), "{}", r.all);
    assert_eq!(r.code, Some(1), "{}", r.all);
}

/// 歯 2: 組み立てた値域の外の値は「まだ分からない」で合格にならず、置き場の値域に在る値は違反にならない。
#[test]
fn f122_value_outside_the_built_range_is_pending_and_never_pass() {
    let w = Work::new("outside");
    w.mutate("strength: [must, must-not, should]", "strength: [must, must-not, should, may]");
    let pending = built_outside("strength", "「may」");
    let r = w.check();
    r.one_amendment();
    assert!(r.others.is_empty(), "{}", r.all);
    assert_eq!(r.pendings, std::slice::from_ref(&pending), "{}", r.all);
    assert_eq!(r.verdict, "folio check: 不合格（違反 1・まだ分からない 1）", "{}", r.all);
    assert_ne!(r.code, Some(0), "{}", r.all);

    w.mutate(
        "{id: P-1.2, pattern: ubiquitous, strength: must-not, text:",
        "{id: P-1.2, pattern: ubiquitous, strength: may, text:",
    );
    let r = w.check();
    assert!(r.range_hits().is_empty(), "{}", r.all);
    assert_eq!(r.pendings, std::slice::from_ref(&pending), "{}", r.all);
    assert_ne!(r.code, Some(0), "{}", r.all);

    w.mutate(
        "{id: P-2.1, pattern: ubiquitous, strength: must, text:",
        "{id: P-2.1, pattern: ubiquitous, strength: never-heard, text:",
    );
    let r = w.check();
    r.one_amendment();
    assert_eq!(
        r.others,
        ["[schema] constitution.yaml: 条 P-2 の statements の P-2.1 の strength の値「never-heard」が憲法の値域 schema.enums.strength に無い"],
        "{}",
        r.all
    );
    assert_eq!(r.pendings, [pending], "{}", r.all);
    assert_eq!(r.verdict, "folio check: 不合格（違反 2・まだ分からない 1）", "{}", r.all);
}

/// 歯 3: 引けない鍵は「まだ分からない」で、黙るのはその鍵の条の値だけ（ほかの鍵の条の値は数え続ける）。
/// 足す鍵と一覧でない鍵は節の先頭に置く＝引けない鍵の後ろに数え続ける鍵が在る。
#[test]
fn f122_unreadable_keys_are_pending_and_only_their_article_values_go_silent() {
    let polarity_line = "    polarity: [fail-open, fail-closed]                 # 仕掛けが壊れたとき 通す / 止める\n";
    let live_hit = "[schema] constitution.yaml: 条 P-1 の mechanism.live の値「zz-live」が憲法の値域 schema.enums.mechanism_live に無い";
    let cases: [(&str, &str); 4] = [
        (
            "unbuilt-key",
            "constitution.yaml: schema.enums.colour: 組み立て時の値域に無い値がある（組み立てた版に無い鍵・値域を置き場ごとに広げる口は無い・FR25）",
        ),
        (
            "missing-key",
            "constitution.yaml: schema.enums に鍵 polarity が無い＝条の polarity の値を置き場の値域で引けない（FR25）",
        ),
        (
            "not-a-list",
            "constitution.yaml: schema.enums.polarity が文字列の一覧でない＝条の値を置き場の値域で引けない（FR25）",
        ),
        (
            "no-section",
            "constitution.yaml: schema.enums（置き場の憲法の値域の節）が表でない＝条の値を置き場の値域で引けない（FR25）",
        ),
    ];
    for (case, pending) in cases {
        let w = Work::new(case);
        match case {
            "unbuilt-key" => w.mutate("  enums:\n", "  enums:\n    colour: [red, blue]\n"),
            "missing-key" => w.mutate(polarity_line, ""),
            "not-a-list" => {
                w.mutate(polarity_line, "");
                w.mutate("  enums:\n", "  enums:\n    polarity: fail-closed\n");
            }
            _ => w.drop_enums(),
        }
        w.mutate(P1_MECHANISM, P1_MECHANISM_ZZ);
        let r = w.check();
        assert!(
            r.pendings.iter().any(|p| p == pending),
            "{case}: 「まだ分からない」の行が無い: {}",
            r.all
        );
        assert!(
            !r.others
                .iter()
                .any(|l| l.contains("が憲法の値域 schema.enums.polarity")),
            "{case}: {}",
            r.all
        );
        let hits = r.range_hits();
        if case == "no-section" {
            assert!(hits.is_empty(), "{case}: {}", r.all);
        } else {
            assert_eq!(hits, [live_hit], "{case}: {}", r.all);
        }
        assert_ne!(r.code, Some(0), "{case}: {}", r.all);
    }
}

/// 歯 4: 凍結 anchor に差し替えた写しは合格で、anchor の全部の鍵に外の値を足すと鍵ごとに「まだ分からない」1 件と、
/// 判断の記録の床の撤退条件の種類の 1 件が並ぶ。
#[test]
fn f122_frozen_anchor_passes_and_outside_values_are_pending_per_key() {
    let lines = anchor_lines();
    let w = Work::new("anchor");
    w.set_enums(&indent(&lines));
    let r = w.check();
    assert_eq!(r.code, Some(0), "{}", r.all);
    assert_eq!(r.verdict, "folio check: 合格（違反 0・まだ分からない 0）", "{}", r.all);

    // 各鍵の値の末尾に zz-outside を 1 つずつ足す
    let mut widened: Vec<String> = Vec::new();
    let mut keys: Vec<String> = Vec::new();
    let mut retreat: Vec<String> = Vec::new();
    for line in &lines {
        if !line.starts_with(' ') {
            if !keys.is_empty() {
                widened.push("  - zz-outside".to_string());
            }
            keys.push(line.trim_end_matches(':').to_string());
        } else if keys.last().is_some_and(|k| k == "retreat_kind") {
            retreat.push(line.trim().trim_start_matches("- ").to_string());
        }
        widened.push(line.clone());
    }
    widened.push("  - zz-outside".to_string());
    assert!(!keys.is_empty() && !retreat.is_empty());
    let w = Work::new("anchor-widened");
    w.set_enums(&indent(&widened));
    let r = w.check();
    r.one_amendment();
    assert!(r.others.is_empty(), "{}", r.all);
    let mut expected: Vec<String> = keys
        .iter()
        .map(|k| built_outside(k, "「zz-outside」"))
        .collect();
    expected.push(format!(
        "constitution.yaml: schema.enums.retreat_kind の「zz-outside」が判断の記録の床の撤退条件の種類 [{}] に無い＝判断の記録の撤退条件を置き場の値域で数えられない（FR25）",
        retreat.join(", ")
    ));
    assert_eq!(r.pendings, expected, "{}", r.all);
    assert_ne!(r.code, Some(0), "{}", r.all);
}

/// 歯 5: 判断の記録の床も撤退条件の種類を部分集合で数える（値を外した・並べ替えただけの一覧は黙る）。
#[test]
fn f122_narrowed_or_reordered_retreat_kind_is_silent() {
    for (case, to) in [
        ("retreat-narrowed", "retreat_kind: [ruling, spike]"),
        ("retreat-reordered", "retreat_kind: [ruling, measure, spike]"),
    ] {
        let w = Work::new(case);
        w.mutate("retreat_kind: [spike, measure, ruling]", to);
        let r = w.check();
        r.one_amendment();
        assert!(r.others.is_empty(), "{case}: {}", r.all);
        assert!(r.pendings.is_empty(), "{case}: {}", r.all);
        assert_eq!(
            r.verdict, "folio check: 不合格（違反 1・まだ分からない 0）",
            "{case}: {}",
            r.all
        );
    }
}
