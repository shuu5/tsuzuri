//! 判断の記録の封（`anchors/adr-seals.yaml`）と `folio check --freeze-adrs` の歯（便 170・docs/design/delivery-170.md §1 (c) の 2〜7・
//! 判断の記録 ADR-30 決定 (3)・便 170 を割った便 172 が運ぶ）と、(c) の 1 の改訂の欄の歯（ADR-30 決定 (2)・便 173 の f173_）。
//! 土台は凍結した写し（tests/fixtures/floor_base/design-intent/・封の一覧 10 行）の写し全部を一時 dir に作り git init と 1 commit を行う。
//! 新しい発効した記録は最小の手書き tests/fixtures/adr/seal-ADR-11.yaml で、その要約値（outside の欄を除く記録の木の json の
//! sha256）は歯が字で持つ。(7) だけは folio2 自身の design-intent/ の写し。
#![cfg(test)]

use crate::common::{FLOOR_BASE, copy_tree, git, repo_root};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const SEALS: &str = "anchors/adr-seals.yaml";
/// 手書きの ADR-11（tests/fixtures/adr/seal-ADR-11.yaml）の要約値。
const ADR11_SUM: &str = "8f4958791a5a475a20a96640c7d30e5f111dc00f8fe35fbaf6e060ecf2eb802e";
/// 同じ ADR-11 に `supersedes: ADR-2` の 1 行を足した記録の要約値。
const ADR11_SUPERSEDES_SUM: &str = "68b74190d773205be9aebb05db9417d30ea23c1c27c6a34a5cb9abf34ecc8105";
/// 手書きの ADR-11 から status を除いた木の json を手で写した凍結の字（Python の json.dumps(sort_keys・区切り「,」「:」・
/// ensure_ascii なし) と同じ形）。`{SUPERSEDES}` の場所に supersedes の欄が入る（retreat と title の間）。
const ADR11_JSON: &str = "{\"approval\":{\"date\":\"2026-09-27\",\"ruling\":\"f2-648.254 notes 2026-09-27\",\"surface\":\"R-8\",\"verbatim\":\"承認する\",\"who\":\"持ち主\"},\"basis\":[\"A-2\"],\"context\":\"封の一覧に行の無い発効した判断の記録を足す場合を作る。\",\"date\":\"2026-09-27\",\"decision\":\"封の一覧の末尾に行を足す。\",\"id\":\"ADR-11\",\"options\":[{\"id\":\"a\",\"name\":\"足す\",\"reason\":\"在る行を変えずに済む。\",\"text\":\"封の一覧の末尾に行を足す。\",\"verdict\":\"adopted\"},{\"id\":\"b\",\"name\":\"足さない\",\"reason\":\"発効した記録が封の外に残る。\",\"text\":\"封の一覧をそのままにする。\",\"verdict\":\"rejected\"}],\"plain\":\"封の一覧に新しい行を足します。\",\"retreat\":{\"condition\":\"持ち主が取り消したら元へ戻す。\",\"kind\":\"ruling\"},{SUPERSEDES}\"title\":\"封の歯のための新しい判断\"}";

/// 写しの一時 dir（歯の終わりに消す）。器（scribe2）の導出 file は写しの根の contracts/ に置く。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(case: &str, src: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-seal-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(&repo_root().join(src), &root.join("design-intent"));
        fs::create_dir_all(root.join("contracts/field-schema")).unwrap();
        fs::copy(
            repo_root().join("contracts/schema.toml"),
            root.join("contracts/field-schema/schema.toml"),
        )
        .unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "fixture"]);
        Work { root }
    }

    fn dir(&self) -> PathBuf {
        self.root.join("design-intent")
    }

    fn check(&self, flags: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tz"))
            .arg("check")
            .arg("--dir")
            .arg(self.dir())
            .args(flags)
            .output()
            .expect("folio を起動できない")
    }

    fn seals(&self) -> String {
        fs::read_to_string(self.dir().join(SEALS)).unwrap()
    }

    /// 写しの file の字面の変異（1 か所だけ）。
    fn edit(&self, rel: &str, from: &str, to: &str) {
        let path = self.dir().join(rel);
        let before = fs::read_to_string(&path).unwrap();
        assert_eq!(
            before.matches(from).count(),
            1,
            "{rel}: 変異の当て先が 1 か所でない: {from:?}"
        );
        fs::write(&path, before.replacen(from, to, 1)).unwrap();
    }

    /// 手書きの ADR-11 に `extra` の行を足して写しの adr/ に置く。
    fn add_adr11(&self, extra: &str) {
        let base = fs::read_to_string(repo_root().join("tests/fixtures/adr/seal-ADR-11.yaml")).unwrap();
        fs::write(self.dir().join("adr/ADR-11.yaml"), format!("{base}{extra}")).unwrap();
    }

    /// 写しの ADR-3 の決定の行の末尾に「。」を 1 字足す。
    fn one_char_in_adr3(&self) {
        let path = self.dir().join("adr/ADR-3.yaml");
        let t = fs::read_to_string(&path).unwrap();
        let at = t.find("\ndecision: ").expect("ADR-3 に決定の行が無い");
        let end = at + 1 + t[at + 1..].find('\n').unwrap();
        fs::write(&path, format!("{}。{}", &t[..end], &t[end..])).unwrap();
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って測る（歯は crate の中を読めない・tests/tz4/schema.rs と同じ形）。
fn sha256_hex(bytes: &[u8]) -> String {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("sha256sum を起動できない");
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "sha256sum が失敗した");
    text(&out.stdout)
        .split_whitespace()
        .next()
        .unwrap()
        .to_string()
}

/// 歯の持つ ADR-11 の要約値 2 つは、手で写した json の字を `sha256sum` で測った値と同じ（folio の外で測った字）。
fn assert_sums_measured_apart() {
    assert_eq!(sha256_hex(ADR11_JSON.replace("{SUPERSEDES}", "").as_bytes()), ADR11_SUM);
    assert_eq!(
        sha256_hex(
            ADR11_JSON
                .replace("{SUPERSEDES}", "\"supersedes\":\"ADR-2\",")
                .as_bytes()
        ),
        ADR11_SUPERSEDES_SUM
    );
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn show(out: &Output) -> String {
    format!("{}{}", text(&out.stdout), text(&out.stderr))
}

/// 違反の行（`[種類] …`）だけを拾う。
fn violations(out: &Output) -> Vec<String> {
    text(&out.stdout)
        .lines()
        .filter(|l| l.starts_with('['))
        .map(str::to_string)
        .collect()
}

/// 本文が封と違う の行の頭。
fn differs(id: &str) -> String {
    format!("[adr] {id}: 発効した判断の記録の本文が封（{SEALS}）の行と違う")
}

/// 行が無い の行の頭。
fn missing(id: &str) -> String {
    format!("[adr] {id}: 発効しているのに封の行が無い")
}

/// rc 1・違反はちょうど `heads` の数で、各行が `heads` の順に頭で合い、まだ分からない は 0。
fn assert_violations(out: &Output, heads: &[&str], what: &str) {
    assert_eq!(out.status.code(), Some(1), "{what}: {}", show(out));
    let v = violations(out);
    assert_eq!(v.len(), heads.len(), "{what}: {v:?}");
    for head in heads {
        assert_eq!(
            v.iter().filter(|l| l.starts_with(head)).count(),
            1,
            "{what}: 「{head}」が 1 行でない: {v:?}"
        );
    }
    let summary = format!("不合格（違反 {}・まだ分からない 0）", heads.len());
    assert!(text(&out.stdout).contains(&summary), "{what}: {}", show(out));
}

fn assert_pass(out: &Output, what: &str) {
    assert_eq!(out.status.code(), Some(0), "{what}: {}", show(out));
}

/// `before` の digest の行より前が `after` の頭に字のまま在り、その後に ADR-11 の行 1 つと digest の行だけが続く。
fn assert_appended(before: &str, after: &str, sum: &str) {
    let head = &before[..before.find("\n\"digest\": \"").expect("digest の行が無い") + 1];
    assert!(after.starts_with(head), "前の行が字のまま残っていない:\n{after}");
    let rest = &after[head.len()..];
    let row = format!("  - \"id\": \"ADR-11\"\n    \"sum\": \"{sum}\"\n\"digest\": \"");
    assert!(rest.starts_with(&row), "末尾が ADR-11 の行でない:\n{rest}");
    assert_eq!(rest.len(), row.len() + 64 + 2, "digest の後に字が在る:\n{rest}");
    assert_ne!(before, after);
}

#[test]
fn f173_a_record_with_revises_is_an_unknown_field() {
    let w = Work::new("revises", FLOOR_BASE);
    let path = w.dir().join("adr/ADR-2.yaml");
    let t = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        format!("{t}revises:\n  - {{target: ADR-1, decision: (1), kind: narrow, summary: 狭く読む}}\n"),
    )
    .unwrap();
    assert_violations(
        &w.check(&[]),
        &["[adr] ADR-2: 未知の欄（N-3）: revises", &differs("ADR-2")],
        "ADR-2 に revises",
    );
}

#[test]
fn f170_one_char_in_an_effective_record_is_caught() {
    let w = Work::new("one-char", FLOOR_BASE);
    assert_pass(&w.check(&[]), "土台");
    w.one_char_in_adr3();
    assert_violations(&w.check(&[]), &[&differs("ADR-3")], "ADR-3 に「。」");
}

#[test]
fn f170_retiring_changes_only_status_and_superseded_by() {
    let w = Work::new("retiring", FLOOR_BASE);
    let before = w.seals();
    w.add_adr11("supersedes: ADR-2\n");
    w.edit(
        "adr/ADR-2.yaml",
        "\nstatus: accepted\n",
        "\nstatus: retired\nsuperseded_by: ADR-11\n",
    );
    assert_violations(&w.check(&[]), &[&missing("ADR-11")], "ADR-2 を退役");
    let frozen = w.check(&["--freeze-adrs"]);
    assert_pass(&frozen, "--freeze-adrs");
    assert!(text(&frozen.stderr).contains("封を足した: "), "{}", show(&frozen));
    assert_appended(&before, &w.seals(), ADR11_SUPERSEDES_SUM);
    assert_pass(&w.check(&[]), "封を足した後");
}

#[test]
fn f170_a_missing_row_is_caught_and_the_freeze_appends_it() {
    assert_sums_measured_apart();
    let w = Work::new("missing", FLOOR_BASE);
    let before = w.seals();
    w.add_adr11("");
    assert_violations(&w.check(&[]), &[&missing("ADR-11")], "ADR-11 を足した");
    let frozen = w.check(&["--freeze-adrs"]);
    assert_pass(&frozen, "--freeze-adrs");
    assert!(
        text(&frozen.stderr).contains("封を足した: ")
            && text(&frozen.stderr).contains("（足した行 ADR-11・在る行 10 は変えない）"),
        "{}",
        show(&frozen)
    );
    let after = w.seals();
    assert_appended(&before, &after, ADR11_SUM);
    assert_pass(&w.check(&[]), "封を足した後");
    let again = w.check(&["--freeze-adrs"]);
    assert_pass(&again, "2 度目の --freeze-adrs");
    assert!(text(&again.stderr).contains("足す封の行は無い"), "{}", show(&again));
    assert_eq!(w.seals(), after, "2 度目で封の一覧を書いた");
}

#[test]
fn f170_the_freeze_does_not_rewrite_a_row() {
    let w = Work::new("rewrite", FLOOR_BASE);
    let before = w.seals();
    w.one_char_in_adr3();
    let out = w.check(&["--freeze-adrs"]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert!(
        show(&out).contains(&format!("{SEALS} の在る行（ADR-3）は書き換えない")),
        "{}",
        show(&out)
    );
    assert_eq!(w.seals(), before, "本文が違うのに封の一覧を書いた");
    drop(w);

    // 足す行が在っても、ほかの違反が在れば書かない
    let w = Work::new("rewrite-other", FLOOR_BASE);
    let before = w.seals();
    w.add_adr11("foo: 1\n");
    let out = w.check(&["--freeze-adrs"]);
    assert_eq!(out.status.code(), Some(1), "{}", show(&out));
    assert!(show(&out).contains("凍結しない"), "{}", show(&out));
    assert_eq!(w.seals(), before, "ほかの違反が在るのに封の一覧を書いた");
}

#[test]
fn f170_a_sealed_record_stays_effective_and_the_list_is_not_hand_edited() {
    let w = Work::new("proposed", FLOOR_BASE);
    w.edit("adr/ADR-4.yaml", "\nstatus: accepted\n", "\nstatus: proposed\n");
    assert_violations(
        &w.check(&[]),
        &[&format!("[adr] ADR-4: 封（{SEALS}）に行が在るのに発効した判断の記録が無い")],
        "ADR-4 を proposed に",
    );
    drop(w);

    let w = Work::new("hand-edit", FLOOR_BASE);
    let t = w.seals();
    let at = t.find("    \"sum\": \"").unwrap() + "    \"sum\": \"".len();
    let digit = if &t[at..=at] == "0" { "1" } else { "0" };
    fs::write(
        w.dir().join(SEALS),
        format!("{}{digit}{}", &t[..at], &t[at + 1..]),
    )
    .unwrap();
    assert_violations(
        &w.check(&[]),
        &[&format!("[adr] {SEALS}: digest が中身と合わない")],
        "要約値の 1 桁を手で変えた",
    );
}

#[test]
fn f170_folio2_itself_is_sealed() {
    let dir = repo_root().join("design-intent");
    let seals = fs::read_to_string(dir.join(SEALS)).unwrap();
    let mut rows: Vec<String> = seals
        .lines()
        .filter_map(|l| l.strip_prefix("  - \"id\": \""))
        .map(|l| l.trim_end_matches('"').to_string())
        .collect();
    rows.sort();
    let mut effective: Vec<String> = Vec::new();
    for entry in fs::read_dir(dir.join("adr")).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        let Some(id) = name.strip_suffix(".yaml").filter(|n| n.starts_with("ADR-")) else {
            continue;
        };
        let body = fs::read_to_string(dir.join("adr").join(&name)).unwrap();
        if body
            .lines()
            .any(|l| l == "status: accepted" || l == "status: retired")
        {
            effective.push(id.to_string());
        }
    }
    effective.sort();
    assert!(!effective.is_empty());
    assert_eq!(rows, effective, "封の行の id の集合が発効した記録の file の集合と違う");

    let w = Work::new("itself", "design-intent");
    assert_pass(&w.check(&[]), "folio2 自身の写しの素の床");
    let before = w.seals();
    let again = w.check(&["--freeze-adrs"]);
    assert_pass(&again, "--freeze-adrs");
    assert!(text(&again.stderr).contains("足す封の行は無い"), "{}", show(&again));
    assert_eq!(w.seals(), before, "足す行が無いのに封の一覧を書いた");
}
