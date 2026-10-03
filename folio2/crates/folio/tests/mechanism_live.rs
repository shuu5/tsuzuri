//! 機構がまだ無い条の 1 行の歯（便 131・docs/design/delivery-131.md §1 (c)・ADR-23 決定 (3)・FR5）。folio は実行 file の crate なので
//! 命令を撃つ。写しは一時 dir に design-intent/ として作り、置き場の親の contracts/ に器の導出 file を写し、git init と 1 commit を行う
//! （tests/freeze_root.rs と同じ作り方・歯の終わりに消す）。
//! 凍結 anchor（P-10.1）は土台の一覧の定数 FLOOR_BASE_LIST（手で書く・folio の code から組まない）。folio2 自身の憲法の一覧の定数
//! FOLIO2_LIST は、条の機構の段の欄の変更を審査に引き出すための意図した 2 つ目の写しである（実の正本に依らない歯の向きの例外・ADR-23 決定 (3)）。
//! 1. 土台の 14 本の行と要約の直前と種別の絞り／2. 0 本の写しと骨格で行が無い／3. 断りの道で行が無い／4. folio2 自身の 6 本。
//!
//! 便 156（docs/design/delivery-156.md §1 (c)）: 行 R-17 が無い置き場で数えなかった知らせの 1 行と、名札が置き場の規則の表に在る行だけを
//! 名指すこと（f156_ の 2 本）。便 203 で、外の置き場の名札 R-9〜R-11 は行を足しても検査の名、folio2 の置き場は行 id（f156_ の 2 本目）。土台は行 R-17 を持たないので、f131_ の 2 本の標準エラーにも知らせが機構の行の前に出る。
//! 便 200（docs/design/delivery-200.md §1 (d)）: 土台と骨格は欄 key が in-loop-min の行も持たないので、下限を数えなかった知らせが
//! 行 R-17 の知らせの次に出る（歯は tests/polarity.rs）。
//! 便 202（docs/design/delivery-202.md §1 (e)）: 骨格（外の置き場）の知らせは folio2 の条の番号の項が落ちた字（IN_LOOP_OFF_ABROAD）。
//! 便 203（docs/design/delivery-203.md §1 (c)）: 骨格では、違反の名札（素の床・編集時の口・folio parts）が folio2 の id を名指さず
//! 検査の名になり（置き場が同じ id の条か行を持っても同じ）、まだ分からない の行から folio2 の番号の片が落ちる。同じ中身で名だけ
//! folio2 にした写しは今の字（f203_ の 1 本）。
#![cfg(test)]

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
/// 欄 key が in-loop-min の行が無くて編集時の止めの下限を数えなかった知らせ（delivery-200.md §1 (b)・手で書く）。
const IN_LOOP_OFF: &str = "# 欄 key が in-loop-min の閾値の行が規則の表に無い＝編集時の止めの本数の下限は数えていない（床の判定の外・条 P-18.4）";
/// 骨格（外の置き場）の同じ知らせ（delivery-202.md §1 (b)・手で書く・folio2 の条の番号の項だけが落ちる）。
const IN_LOOP_OFF_ABROAD: &str = "# 欄 key が in-loop-min の閾値の行が規則の表に無い＝編集時の止めの本数の下限は数えていない（床の判定の外）";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../folio2")
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
        fs::create_dir_all(root.join("contracts/field-schema")).unwrap();
        fs::copy(
            repo_root().join("contracts/schema.toml"),
            root.join("contracts/field-schema/schema.toml"),
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
    Command::new(env!("CARGO_BIN_EXE_tz"))
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
    assert_eq!(lines(&out.stderr), [OFF, IN_LOOP_OFF, FLOOR_BASE_LIST], "{}", show(&out));

    let amends = w.check(&["--emit-amends"]);
    assert_eq!(amends.status.code(), Some(0), "{}", show(&amends));
    assert_eq!(lines(&amends.stderr), [OFF, IN_LOOP_OFF, FLOOR_BASE_LIST, PASS], "{}", show(&amends));
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
    assert_eq!(lines(&out.stderr), [OFF, IN_LOOP_OFF], "{}", show(&out));

    let sk = Work::skeleton("skeleton");
    let out = sk.check(&[]);
    assert_eq!(out.status.code(), Some(2), "{}", show(&out));
    assert_eq!(
        lines(&out.stdout),
        [SKELETON_SUMMARY],
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

/// 骨格の要約（違反 0・まだ分からない 6 = 凍結の基準の不在 2 と骨格が書く決定の欄の骨格の印 4・便 181）。
const SKELETON_SUMMARY: &str = "folio check: まだ分からない（違反 0・まだ分からない 6）";

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
    let err = lines(&out.stderr);
    assert_eq!(err[err.len().saturating_sub(2)..], [OFF, IN_LOOP_OFF_ABROAD], "{}", show(&out));
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

/// 便 203 で改めた: 外の置き場（骨格・名は 未記入）の名札 R-9〜R-11 は、規則の表に行 R-9〜R-11 を足しても検査の名（床は行の値も
/// 対象も読まない・tsuzuri の同じ id は別の意味）。同じ中身で名だけ folio2 の置き場の名にした写しは行 id（便 156 と同じ字）。
#[test]
fn f156_abroad_labels_name_the_checks_even_with_the_rows() {
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
    let names = [
        format!("[平易文] {PLAIN}"),
        format!("[強度と文末] {POLARITY}"),
        format!("[語彙] {VOCAB}"),
    ];

    let w = Work::skeleton("f156-names");
    break_p1(&w.dir());
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert_eq!(violations(&out), names, "{}", show(&out));
    // 違反の在る置き場でも知らせは出る
    assert_eq!(off_count(&out), 1, "{}", show(&out));

    // 閾値の節に行 R-9〜R-11 を足しても、外の置き場の名札は検査の名
    add_rows(&w.dir(), "thresholds", &rows(|id, what| threshold(id, what, "0 件")));
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert_eq!(violations(&out), names, "{}", show(&out));

    // 作法の節に置いても同じ
    let d = Work::skeleton("f156-discipline");
    break_p1(&d.dir());
    add_rows(&d.dir(), "discipline", &rows(discipline));
    d.commit();
    let out = d.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert_eq!(violations(&out), names, "{}", show(&out));

    // 名だけ folio2 の置き場の名にした写しは行 id（folio2 の置き場は行 R-9〜R-11 を持ち、字は便 156 のまま）
    let h = Work::skeleton("f156-home");
    break_p1(&h.dir());
    add_rows(&h.dir(), "thresholds", &rows(|id, what| threshold(id, what, "0 件")));
    edit(&h.dir().join("constitution.yaml"), |t| t.replacen("  id: 未記入\n", "  id: folio2-constitution\n", 1));
    h.commit();
    let out = h.check(&[]);
    // 骨格の欄の決まりの file（adr/schema.yaml・design-note/schema.yaml）は外の置き場の字で書かれ、folio2 の置き場の床の定数と違う
    // （[adr]・[note] の行）ので、崩した 3 つの字の行だけを見る
    let broken: Vec<String> = violations(&out)
        .into_iter()
        .filter(|l| [PLAIN, POLARITY, VOCAB].iter().any(|m| l.ends_with(m)))
        .collect();
    assert_eq!(
        broken,
        [
            format!("[R-10] {PLAIN}"),
            format!("[R-11] {POLARITY}"),
            format!("[R-9] {VOCAB}"),
        ],
        "{}",
        show(&out)
    );
}

// ── 便 203（delivery-203.md §1 (c)）──

/// 骨格の判断の記録 ADR-1 を発効にした字（承認欄は無いまま）。
const ADR1_ACCEPTED: &str = "ADR-1: accepted なのに approval（逐語・日付・裁定 id・対話面）が無い";
/// 骨格の印の行（決定の欄 4 つ・便 181）の頭。
const MARKED: [&str; 4] = [
    "constitution.yaml: meta.approval.ruling",
    "rules.yaml: 行 R-2 の ruling",
    "rules.yaml: 行 R-8 の ruling",
    "rules.yaml: 行 R-16 の ruling",
];

/// 骨格の ADR-1 を発効にする。
fn accept_adr1(dir: &Path) {
    edit(&dir.join("adr/ADR-1.yaml"), |s| s.replacen("status: proposed\n", "status: accepted\n", 1));
}

/// 標準出力と標準エラーの、条と規範文の id の形（P- / N- / A- と数・前の字が英字でない）。骨格の条は P-1 だけで出力には出ない。
fn article_ids(out: &Output) -> Vec<String> {
    let all: Vec<char> = show(out).chars().collect();
    all.iter()
        .enumerate()
        .filter(|&(i, c)| {
            matches!(c, 'P' | 'N' | 'A')
                && (i == 0 || !all[i - 1].is_ascii_alphabetic())
                && all.get(i + 1) == Some(&'-')
                && all.get(i + 2).is_some_and(char::is_ascii_digit)
        })
        .map(|(i, _)| all[i..(i + 7).min(all.len())].iter().collect())
        .collect()
}

/// まだ分からない の行（床は標準エラーに出し、編集時の口は標準出力に出す）。
fn unknown_lines(bytes: &[u8]) -> Vec<String> {
    lines(bytes)
        .into_iter()
        .filter_map(|l| l.strip_prefix("# まだ分からない: ").map(str::to_string))
        .collect()
}

#[test]
fn f203_abroad_labels_and_unknowns_name_no_folio2_number() {
    // (a) 骨格（名は 未記入＝外の置き場）の ADR-1 を発効にした写し: 名札は検査の名、まだ分からない の行は番号の片が落ちる
    let w = Work::skeleton("f203-accepted");
    accept_adr1(&w.dir());
    w.commit();
    let out = w.check(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert_eq!(violations(&out), [format!("[改訂の承認] {ADR1_ACCEPTED}")], "{}", show(&out));
    let unknowns = unknown_lines(&out.stderr);
    assert!(
        unknowns.iter().any(|l| l.starts_with("凍結 anchor が 0 本（")
            && l.ends_with("）＝差分検査は「まだ分からない」。発効版で --freeze-anchor を実行する")),
        "{}",
        show(&out)
    );
    assert!(
        unknowns.contains(&"anchors/adr-seals.yaml（判断の記録の封の一覧）が無い＝発効した判断の記録 1 本の本文の凍結を測れない（folio check --freeze-adrs で封を書き、commit する）".to_string()),
        "{}",
        show(&out)
    );
    for at in MARKED {
        assert!(unknowns.contains(&format!("{at} が 未記入（骨格の印）")), "{at}: {}", show(&out));
    }
    assert_eq!(article_ids(&out), Vec::<String>::new(), "{}", show(&out));

    // 編集時の口: 発効にした ADR-1 の中身を渡すと、同じ名札の違反と封の まだ分からない が新しく出る
    let s = Work::skeleton("f203-proposed");
    let accepted = fs::read_to_string(w.dir().join("adr/ADR-1.yaml")).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["check", "--dir"])
        .arg(s.dir())
        .args(["--proposed", "adr/ADR-1.yaml"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("folio を起動できない");
    std::io::Write::write_all(child.stdin.as_mut().unwrap(), accepted.as_bytes()).unwrap();
    let proposed = child.wait_with_output().unwrap();
    assert_eq!(violations(&proposed), [format!("[改訂の承認] {ADR1_ACCEPTED}")], "{}", show(&proposed));
    assert!(
        unknown_lines(&proposed.stdout).iter().any(|l| l.ends_with("（folio check --freeze-adrs で封を書き、commit する）")),
        "{}",
        show(&proposed)
    );
    assert_eq!(article_ids(&proposed), Vec::<String>::new(), "{}", show(&proposed));

    retreat_and_parts();
    index_and_home();
}

/// 撤退条件を空にした写しと folio parts --check の部品目録に無い class で、名札が検査の名なのを見る。
fn retreat_and_parts() {
    // (b) 撤退条件を空にした写し: 名札は検査の名（字の中の P-8.1 は本便の外）
    let r = Work::skeleton("f203-retreat");
    edit(&r.dir().join("adr/ADR-1.yaml"), |t| {
        t.replacen("retreat: {kind: ruling, condition: 未記入}\n", "retreat: {}\n", 1)
    });
    // 置き場が同じ id の条 P-8（別の意味）を持っても、名札は検査の名（tsuzuri の条 P-8 は値の型の区別）
    edit(&r.dir().join("constitution.yaml"), |t| {
        t.replacen("  counts: {always: 1,", "  counts: {always: 2,", 1).replacen(
            "    relations: {rules: [R-2, R-8, R-16]}\n",
            "    relations: {rules: [R-2, R-8, R-16]}\n  - id: P-8\n    title: 値の型\n    tier: always\n    binds: both\n    statements:\n      - {id: P-8.1, pattern: ubiquitous, strength: must, text: 宣言した値と測った値を型で区別する。}\n    plain: 宣言した値と測った値を型で区別する。\n    rationale: []\n    mechanism: {kind: none, live: now, note: 判断の規則であって機械では検査できない。}\n    relations: {rules: []}\n",
            1,
        )
    });
    r.commit();
    let out = r.check(&[]);
    assert_eq!(
        violations(&out),
        [
            "[撤退条件] ADR-1.retreat: 必須欄が無い: kind・condition",
            "[撤退条件] ADR-1: retreat.kind が値域外: （無い）",
            "[撤退条件] ADR-1: 撤退条件が空（P-8.1）",
        ],
        "{}",
        show(&out)
    );

    // (c) folio parts --check: 部品目録に無い class の名札は検査の名で、置き場に行 R-3 を足しても同じ（床は行 R-3 を引かない・面は 1 度だけ組む）
    let p = Work::skeleton("f203-parts");
    let site = p.root.join("site");
    let built = folio(&["build"], &p.dir(), &["--out", site.to_str().unwrap(), "--write"]);
    // 骨格の床は まだ分からない（2）でも、面は書く
    assert!(text(&built.stdout).contains("folio build: 書いた"), "{}", show(&built));
    edit(&site.join("index.html"), |t| t.replacen("class=\"", "class=\"zz-unknown ", 1));
    let page = format!("index={}", site.join("index.html").display());
    let css = site.join("folio.css");
    let parts = || {
        let out = folio(&["parts", "--check"], &p.dir(), &["--css", css.to_str().unwrap(), "--page", &page]);
        assert_eq!(out.status.code(), Some(1), "{}", show(&out));
        violations(&out)
    };
    assert_eq!(parts(), ["[部品目録] index.html: 部品目録に無い class「zz-unknown」"]);
    add_rows(&p.dir(), "thresholds", &[("R-3", threshold("R-3", "部品目録に無い型の数", "0 件"))]);
    assert_eq!(parts(), ["[部品目録] index.html: 部品目録に無い class「zz-unknown」"]);
}

/// 凍結 anchor の索引が空か版の file が無い骨格と、名だけ folio2 の置き場の名にした写しの まだ分からない の行を見る。
fn index_and_home() {
    // (d) 凍結 anchor の索引の entries が空・索引の版の anchor file が無い骨格: まだ分からない の行は（P-10.3）の片が落ちる
    let x = Work::skeleton("f203-index");
    let index = x.dir().join("anchors/index.yaml");
    fs::create_dir_all(index.parent().unwrap()).unwrap();
    for (entries, want) in [
        (
            "entries: []\n",
            "index.yaml: 索引はあるが entries が空＝比較元が立たない（まだ分からない）。索引と anchor は消さない・空にしない",
        ),
        (
            "entries:\n- version: v1.0\n  previous: null\n  digest: acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed\n",
            "anchor の列が切れている: 索引にある版 v1.0 の anchor file が無いか読めない＝差分検査は「まだ分からない」。anchors/ は消さない",
        ),
    ] {
        fs::write(&index, format!("kind: constitution-anchor-index\n{entries}")).unwrap();
        x.commit();
        let out = x.check(&[]);
        assert!(unknown_lines(&out.stderr).contains(&want.to_string()), "{}", show(&out));
    }

    // (e) 同じ中身で名だけ folio2 の置き場の名にした写しは今の字（名札 N-4・A-2 / N-4 と P-10.3・条 P-17.3）
    let h = Work::skeleton("f203-home");
    accept_adr1(&h.dir());
    edit(&h.dir().join("constitution.yaml"), |t| t.replacen("  id: 未記入\n", "  id: folio2-constitution\n", 1));
    h.commit();
    let out = h.check(&[]);
    assert!(violations(&out).contains(&format!("[N-4] {ADR1_ACCEPTED}")), "{}", show(&out));
    let unknowns = unknown_lines(&out.stderr);
    assert!(
        unknowns.iter().any(|l| l.ends_with("）＝A-2 / N-4 の差分検査は「まだ分からない」（P-10.3）。発効版で --freeze-anchor を実行する")),
        "{}",
        show(&out)
    );
    for at in MARKED {
        assert!(unknowns.contains(&format!("{at} が 未記入（骨格の印・裁定の前＝条 P-17.3）")), "{at}: {}", show(&out));
    }
}
