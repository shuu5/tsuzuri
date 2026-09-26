//! 機構がまだ無い条の 1 行の歯（便 131・docs/design/delivery-131.md §1 (c)・ADR-23 決定 (3)・FR5）。folio は実行 file の crate なので
//! 命令を撃つ。写しは一時 dir に design-intent/ として作り、置き場の親の contracts/ に器の導出 file を写し、git init と 1 commit を行う
//! （tests/freeze_root.rs と同じ作り方・歯の終わりに消す）。
//! 凍結 anchor（P-10.1）は土台の一覧の定数 FLOOR_BASE_LIST（手で書く・folio の code から組まない）。folio2 自身の憲法の一覧の定数
//! FOLIO2_LIST は、条の機構の段の欄の変更を審査に引き出すための意図した 2 つ目の写しである（実の正本に依らない歯の向きの例外・ADR-23 決定 (3)）。
//! 1. 土台の 14 本の行と要約の直前と種別の絞り／2. 0 本の写しと骨格で行が無い／3. 断りの道で行が無い／4. folio2 自身の 6 本。
//!
//! 便 156（docs/design/delivery-156.md §1 (c)）: 行 R-17 が無い置き場で数えなかった知らせの 1 行と、名札が置き場の規則の表に在る行だけを
//! 名指すこと（f156_ の 2 本）。土台は行 R-17 を持たないので、f131_ の 2 本の標準エラーにも知らせが機構の行の前に出る。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";
const HEAD: &str = "# 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: ";
const PASS: &str = "folio check: 合格（違反 0・まだ分からない 0）";
/// 床の凍結の土台（第 1.0 版）の一覧（delivery-131.md §1 (a) の 4）。
const FLOOR_BASE_LIST: &str = "# 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: P-1（M0）・P-3（M0）・P-4（M0）・P-7（delivery-0）・P-10（M0）・P-11（delivery-0）・P-12（M0）・P-13（M0）・P-15（M1）・P-17（delivery-0）・P-18（delivery-0）・A-3（M1）・N-1（M0）・N-5（M0）";
/// folio2 自身の憲法（第 1.4 版）の一覧（delivery-131.md §1 (b) の 3）。
const FOLIO2_LIST: &str = "# 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: P-11（M1）・P-13（M1）・P-15（M1）・P-17（M1）・P-18（M1）・A-3（M1）";
/// 行 R-17 が無くて散文の言及の歯が数えなかった知らせ（delivery-156.md §1 (b) の 1・手で書く）。
const OFF: &str = "# 行 R-17 が規則の表に無い＝散文の言及の歯は数えていない（床の判定の外・値 0 件 の行 R-17 を足して撃ち直すと数える）";

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

/// 写しの一時 dir（歯の終わりに消す）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn root(case: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("folio-f131-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    /// 土台を design-intent/ として写し、器の導出 file を写し、`prep` を当ててから git init と 1 commit。
    fn floor_base(case: &str, prep: impl FnOnce(&Path)) -> Work {
        let root = Work::root(case);
        copy_tree(&repo_root().join(FLOOR_BASE), &root.join("design-intent"));
        fs::create_dir_all(root.join("contracts")).unwrap();
        fs::copy(
            repo_root().join("contracts/schema.toml"),
            root.join("contracts/schema.toml"),
        )
        .unwrap();
        prep(&root.join("design-intent"));
        let w = Work { root };
        w.commit();
        w
    }

    /// folio init の骨格を design-intent/ に作り、git init と 1 commit。
    fn skeleton(case: &str) -> Work {
        let root = Work::root(case);
        fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        let w = Work { root };
        let out = folio(&["init"], &w.dir(), &[]);
        assert_eq!(out.status.code(), Some(0), "{}", show(&out));
        w.commit();
        w
    }

    fn commit(&self) {
        git(&self.root, &["init", "-q"]);
        git(&self.root, &["add", "-A"]);
        git(&self.root, &["commit", "-q", "--allow-empty", "-m", "fixture"]);
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn check(&self, flags: &[&str]) -> Output {
        folio(&["check"], &self.dir(), flags)
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn folio(head: &[&str], dir: &Path, tail: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(head)
        .arg("--dir")
        .arg(dir)
        .args(tail)
        .output()
        .expect("folio を起動できない")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn show(out: &Output) -> String {
    format!("{}\n{}", text(&out.stdout), text(&out.stderr))
}

fn lines(bytes: &[u8]) -> Vec<String> {
    text(bytes).lines().map(str::to_string).collect()
}

/// 標準エラーの一覧の行（頭の字で始まる）。
fn listed(out: &Output) -> Vec<String> {
    lines(&out.stderr)
        .into_iter()
        .filter(|l| l.starts_with(HEAD))
        .collect()
}

fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "置き換える字が無い（前提が崩れた）: {}", path.display());
    fs::write(path, after).unwrap();
}

#[test]
fn f131_floor_base_lists_the_articles_before_the_summary() {
    let w = Work::floor_base("base", |_| {});
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert_eq!(lines(&out.stdout), [PASS], "{}", show(&out));
    // 土台は行 R-17 を持たないので、数えなかった知らせが機構の行の前に出る（便 156）
    assert_eq!(lines(&out.stderr), [OFF, FLOOR_BASE_LIST], "{}", show(&out));

    let amends = w.check(&["--emit-amends"]);
    assert_eq!(amends.status.code(), Some(0), "{}", show(&amends));
    assert_eq!(lines(&amends.stderr), [OFF, FLOOR_BASE_LIST, PASS], "{}", show(&amends));
    assert!(!text(&amends.stdout).contains(HEAD), "{}", show(&amends));

    // 種別の絞り: human-review の最初の条（P-16）の live を M1 にしても一覧は 14 本のまま
    let review = Work::floor_base("review", |dir| {
        edit(&dir.join("constitution.yaml"), |s| {
            s.replacen("{kind: human-review, live: now,", "{kind: human-review, live: M1,", 1)
        });
    });
    let out = review.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert_eq!(lines(&out.stdout), [PASS], "{}", show(&out));
    assert_eq!(listed(&out), [FLOOR_BASE_LIST], "{}", show(&out));
}

#[test]
fn f131_no_line_when_no_article_waits_for_its_mechanism() {
    let w = Work::floor_base("now", |dir| {
        edit(&dir.join("constitution.yaml"), |s| {
            let mut t = s.to_string();
            for live in ["M0", "delivery-0", "M1", "adr"] {
                t = t.replace(&format!(", live: {live},"), ", live: now,");
            }
            t
        });
    });
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", show(&out));
    assert_eq!(lines(&out.stdout), [PASS], "{}", show(&out));
    assert_eq!(lines(&out.stderr), [OFF], "{}", show(&out));

    let sk = Work::skeleton("skeleton");
    let out = sk.check(&[]);
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert_eq!(
        lines(&out.stdout),
        ["folio check: まだ分からない（違反 0・まだ分からない 2）"],
        "{}",
        show(&out)
    );
    assert!(listed(&out).is_empty(), "{}", show(&out));
}

#[test]
fn f131_a_refused_freeze_prints_no_line() {
    let w = Work::floor_base("refused", |_| {});
    let out = w.check(&["--freeze-anchor"]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert!(out.stdout.is_empty(), "{}", show(&out));
    assert!(listed(&out).is_empty(), "{}", show(&out));
    assert!(text(&out.stderr).contains("同じ版は上書きしない"), "{}", show(&out));
}

#[test]
fn f131_folio2_constitution_lists_the_six_articles() {
    let out = folio(&["check"], &repo_root().join("design-intent"), &[]);
    assert_eq!(listed(&out), [FOLIO2_LIST], "{}", show(&out));
}

// ── 便 156（delivery-156.md §1 (c)）──

/// 骨格の要約（違反 0・まだ分からない 2）。
const SKELETON_SUMMARY: &str = "folio check: まだ分からない（違反 0・まだ分からない 2）";

/// 閾値の行 1 つ（条 P-1 に結ぶ・骨格の行と同じ欄）。
fn threshold(id: &str, what: &str, value: &str) -> String {
    format!(
        "{{id: {id}, article: P-1, what: {what}, value: {value}, kind: build-check, status: 仮, ruling: 未記入, ruled_at: 未記入, stage: post}}"
    )
}

/// 作法の行 1 つ（条 P-1 に結ぶ）。
fn discipline(id: &str, what: &str) -> String {
    format!(
        "{{id: {id}, article: P-1, what: {what}, kind: human-review, status: 仮, ruling: 未記入, ruled_at: 未記入}}"
    )
}

/// 骨格の規則の表の節 `section`（thresholds か discipline）に行を足し、条 P-1 の relations.rules に行 id を足す。
fn add_rows(dir: &Path, section: &str, rows: &[(&str, String)]) {
    let body: String = rows.iter().map(|(_, row)| format!("  - {row}\n")).collect();
    edit(&dir.join("rules.yaml"), |s| match section {
        "thresholds" => s.replacen("thresholds:\n", &format!("thresholds:\n{body}"), 1),
        _ => s.replacen("discipline: []\n", &format!("discipline:\n{body}"), 1),
    });
    let ids: String = rows.iter().map(|(id, _)| format!(", {id}")).collect();
    edit(&dir.join("constitution.yaml"), |s| {
        s.replacen(
            "relations: {rules: [R-2, R-8, R-16]}",
            &format!("relations: {{rules: [R-2, R-8, R-16{ids}]}}"),
            1,
        )
    });
}

/// 違反の行（`[名札] …`）。
fn violations(out: &Output) -> Vec<String> {
    lines(&out.stdout)
        .into_iter()
        .filter(|l| l.starts_with('['))
        .collect()
}

fn off_count(out: &Output) -> usize {
    lines(&out.stderr).iter().filter(|l| *l == OFF).count()
}

#[test]
fn f156_a_place_without_r17_says_the_mentions_are_not_counted() {
    let w = Work::skeleton("f156-off");
    edit(&w.dir().join("adr/ADR-1.yaml"), |s| {
        s.replacen("発効します。\n", "発効します。承認は行 R-8 の対話面で受ける。\n", 1)
    });
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert_eq!(lines(&out.stdout), [SKELETON_SUMMARY], "{}", show(&out));
    assert_eq!(lines(&out.stderr).last().map(String::as_str), Some(OFF), "{}", show(&out));
    assert_eq!(off_count(&out), 1, "{}", show(&out));

    // 値 0 件 の行 R-17 を足すと言及を数える
    add_rows(&w.dir(), "thresholds", &[("R-17", threshold("R-17", "散文の言及の数", "0 件"))]);
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert_eq!(
        violations(&out),
        ["[R-17] adr/ADR-1.yaml: ADR-1 の散文が R-8 を指すのに、ADR-1 の型付きの欄にも R-8 の型付きの欄にも無い（型付きの欄へ書き写す）"],
        "{}",
        show(&out)
    );
    assert_eq!(off_count(&out), 0, "{}", show(&out));

    // 値が 0 件 でなければ まだ分からない で、知らせは出ない
    edit(&w.dir().join("rules.yaml"), |s| s.replacen("value: 0 件,", "value: 1 件,", 1));
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert!(violations(&out).is_empty(), "{}", show(&out));
    assert!(
        text(&out.stderr).contains("R-17 の値「1 件」が 0 件 でない"),
        "{}",
        show(&out)
    );
    assert_eq!(off_count(&out), 0, "{}", show(&out));

    let own = folio(&["check"], &repo_root().join("design-intent"), &[]);
    assert_eq!(off_count(&own), 0, "{}", show(&own));
}

/// 骨格の条 P-1 を 3 通り崩す（見出しに foobar・平易文を消す・強度を must-not）。
fn break_p1(dir: &Path) {
    edit(&dir.join("constitution.yaml"), |s| {
        s.replacen(
            "    title: 承認の受け方と検査の値\n",
            "    title: 承認の受け方と検査の値 foobar\n",
            1,
        )
        .lines()
        .filter(|l| !l.starts_with("    plain: 承認は決まった対話面"))
        .map(|l| format!("{l}\n"))
        .collect::<String>()
        .replacen("strength: must, text:", "strength: must-not, text:", 1)
    });
}

#[test]
fn f156_labels_name_only_rows_the_place_has() {
    const PLAIN: &str = "P-1: plain が無い";
    const POLARITY: &str = "P-1: P-1.1: strength must-not と文末が合わない（must-not ⇔ 〜ない。）";
    const VOCAB: &str = "P-1 title: 語彙に無い英字の語「foobar」";
    let rows = |row: fn(&str, &str) -> String| {
        [
            ("R-9", row("R-9", "語彙の検査")),
            ("R-10", row("R-10", "条の平易文の有無")),
            ("R-11", row("R-11", "強度と文末の一致")),
        ]
    };

    let w = Work::skeleton("f156-names");
    break_p1(&w.dir());
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert_eq!(
        violations(&out),
        [
            format!("[平易文] {PLAIN}"),
            format!("[強度と文末] {POLARITY}"),
            format!("[語彙] {VOCAB}"),
        ],
        "{}",
        show(&out)
    );
    // 違反の在る置き場でも知らせは出る
    assert_eq!(off_count(&out), 1, "{}", show(&out));

    let ids = [
        format!("[R-10] {PLAIN}"),
        format!("[R-11] {POLARITY}"),
        format!("[R-9] {VOCAB}"),
    ];
    add_rows(&w.dir(), "thresholds", &rows(|id, what| threshold(id, what, "0 件")));
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert_eq!(violations(&out), ids, "{}", show(&out));

    // 作法の節に置いても行 id で名指す
    let d = Work::skeleton("f156-discipline");
    break_p1(&d.dir());
    add_rows(&d.dir(), "discipline", &rows(discipline));
    d.commit();
    let out = d.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert_eq!(violations(&out), ids, "{}", show(&out));
}
