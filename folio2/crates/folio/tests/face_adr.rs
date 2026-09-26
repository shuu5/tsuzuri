//! 判断の記録の面（`folio face --face adr --id ADR-n`）の歯（便 25・docs/design/delivery-25.md §1 (g)）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/expected-adr.html）との byte 一致（P-10.1）・escape
//! - 実の正本の全本で `folio parts --check` に合格（AC14 の緑・NFR2）と逐語と件数の census（yaml-rust2 で正本を直に読む）
//! - check の 3 値・id の口 4 形・名札の表の外 2 つ・基の型・未解決の根拠の印・accepted と retired の状態・
//!   枝番付きの条 id・AC14 の赤の fixture・mode
//! - 図の章（便 33・FR15）: 写し（ADR-2・図 1 枚）の figure-panel と型の名札と根拠のリンクと cover-meta・図なしの面は
//!   図の章の外が byte で同じ・通らない図で 2 と前の面の保持・型外・道具の不在・実の正本の figure-panel の数
//! - 改訂の欄（便 137）: 表紙の札 2 つ・章 05 の空の断りの 1 段落・h3 の下の amends と revises の行・向きの表の外で 2・
//!   実の正本の全本で札の件数と revises の全行の逐語
//! - 受けた改訂の逆向きの行（便 148）: 写しの発効の ADR-1 の revises が ADR-2 の面の章 05 と表紙の札に出る・提案中と
//!   廃止は読まない・読めない改訂する側で 2・実の正本の全本で逆向きの札と行の逐語と順・ADR-18 と ADR-16 の実例
//!
//! 版管理の下の面は書き換えない（`--out` は必ず一時 dir の中）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use yaml_rust2::{Yaml, YamlLoader};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> PathBuf {
    repo_root().join("tests/fixtures/face")
}

fn design_intent() -> PathBuf {
    repo_root().join("design-intent")
}

fn vendor() -> PathBuf {
    repo_root().join("vendor/archify")
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-face-adr-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

/// fixture の正本 4 file と adr/ADR-1.yaml・adr/ADR-2.yaml を一時 dir の下の src/ へ、repo の vendor/archify/
/// （図の道具・便 33 の面は図ごとに撃つ）を親 dir の vendor/archify/ へ写す。戻り値 = (一時 dir, 写し)。
fn fixture_copy(case: &str) -> (PathBuf, PathBuf) {
    let td = temp_dir(case);
    let work = td.join("src");
    fs::create_dir_all(work.join("adr")).unwrap();
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "vocabulary.yaml",
        "srs.yaml",
        "ceiling.yaml",
        "adr/ADR-1.yaml",
        "adr/ADR-2.yaml",
    ] {
        fs::copy(fixture().join(name), work.join(name)).unwrap();
    }
    copy_dir(&vendor(), &td.join("vendor/archify"));
    (td, work)
}

/// `folio face --face <face> [--id <id>] --dir <dir> --out <out> <mode>`。
fn folio_face(face: &str, id: Option<&str>, dir: &Path, out: &Path, mode: &str) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_folio"));
    cmd.arg("face").arg("--face").arg(face);
    if let Some(id) = id {
        cmd.arg("--id").arg(id);
    }
    cmd.arg("--dir")
        .arg(dir)
        .arg("--out")
        .arg(out)
        .arg(mode)
        .output()
        .expect("folio を起動できない")
}

fn code(out: &Output, what: &str) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "{what} が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn edit(path: &Path, f: impl FnOnce(&str) -> String) {
    let before = fs::read_to_string(path).unwrap();
    let after = f(&before);
    assert_ne!(before, after, "変異が当たっていない: {}", path.display());
    fs::write(path, after).unwrap();
}

/// 5 字の escape（生成側の字面を使わず歯の側で持つ）。
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// 写しの ADR-2 に変異を当て、`--write` の結果と面の本文を返す（面が出来ていなければ本文は空）。
fn mutated(case: &str, mutate: impl FnOnce(&str) -> String) -> (Output, String) {
    let (td, work) = fixture_copy(case);
    edit(&work.join("adr/ADR-2.yaml"), mutate);
    let out = td.join("adr-2.html");
    let run = folio_face("adr", Some("ADR-2"), &work, &out, "--write");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    (run, html)
}

/// 写しの ADR-2 から面を組み、本文を返す（変異なし）。
fn fixture_html(case: &str) -> String {
    let (td, work) = fixture_copy(case);
    let out = td.join("adr-2.html");
    let run = folio_face("adr", Some("ADR-2"), &work, &out, "--write");
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    let html = fs::read_to_string(&out).unwrap();
    let _ = fs::remove_dir_all(&td);
    html
}

/// 写しの ADR-2 の context を差し替えて面を組む（章 01 の分かれ方を測る）。
fn with_context(case: &str, body: &str) -> String {
    let (run, html) = mutated(case, |t| {
        t.replacen(
            "context: 見本の面を出すのに、正本の欄だけで 1 枚を組めるかがまだ分からない。",
            &format!("context: {body}"),
            1,
        )
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    html
}

/// 変異が導出できない入力なら `--write` = 2 ∧「まだ分からない」∧ 文言。
fn unknown(case: &str, mutate: impl FnOnce(&str) -> String, wording: &str) {
    let (run, html) = mutated(case, mutate);
    assert_eq!(code(&run, "folio face"), 2, "{case}: {}", stderr(&run));
    assert!(
        stderr(&run).contains("まだ分からない"),
        "{case}: {}",
        stderr(&run)
    );
    assert!(
        stderr(&run).contains(wording),
        "{case}: 「{wording}」が無い: {}",
        stderr(&run)
    );
    assert!(html.is_empty(), "{case}: 導出できないのに面を書いた");
}

/// `a` から `b` の直前までを切り取る（a が無ければそのまま・a の後の最初の b・b が無ければ末尾まで）。
fn cut(html: &str, a: &str, b: &str) -> String {
    let Some(start) = html.find(a) else {
        return html.to_string();
    };
    let end = html[start..].find(b).map_or(html.len(), |e| start + e);
    format!("{}{}", &html[..start], &html[end..])
}

/// `a` で始まる行を改行ごと切り取る（a が無ければそのまま）。
fn cut_line(html: &str, a: &str) -> String {
    let Some(start) = html.find(a) else {
        return html.to_string();
    };
    let end = html[start..]
        .find('\n')
        .map_or(html.len(), |e| start + e + 1);
    format!("{}{}", &html[..start], &html[end..])
}

/// 図の本体の数（章の帯の kicker の絵記号 `<svg class="ico"` は数えない）。
fn svg_bodies(html: &str) -> usize {
    html.matches("<svg").count() - html.matches("<svg class=\"ico\"").count()
}

/// 写しの ADR-2 から図の節（figures 以降・末尾まで）を消した面（便 32 までの形）。
fn figureless_html(case: &str) -> String {
    let (run, html) = mutated(case, |t| {
        let at = t.find("\nfigures:\n").expect("figures が無い");
        format!("{}\n", &t[..at])
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    html
}

// ── 凍結 fixture ──

#[test]
fn face_adr_write_matches_the_frozen_fixture() {
    let (td, work) = fixture_copy("anchor");
    let out = td.join("adr-2.html");
    let run = folio_face("adr", Some("ADR-2"), &work, &out, "--write");
    let written = fs::read(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert_eq!(
        stdout(&run),
        format!("folio face: 書いた（{} byte）\n", written.len())
    );
    let frozen = fs::read(fixture().join("expected-adr.html")).unwrap();
    if written != frozen {
        let at = written
            .iter()
            .zip(&frozen)
            .position(|(a, b)| a != b)
            .unwrap_or(written.len().min(frozen.len()));
        let show = |b: &[u8]| {
            String::from_utf8_lossy(&b[at.saturating_sub(120)..(at + 200).min(b.len())])
                .into_owned()
        };
        panic!(
            "expected-adr.html と違う（{} byte ≠ {} byte・最初の差 {at} byte 目）\n--- folio\n{}\n--- 凍結\n{}",
            written.len(),
            frozen.len(),
            show(&written),
            show(&frozen)
        );
    }
}

#[test]
fn face_adr_escapes_values_from_the_source() {
    let (run, html) = mutated("escape", |t| {
        t.replacen(
            "title: 見本の判断の記録",
            "title: 見本の<b>判断</b>の記録",
            1,
        )
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(
        html.contains("&lt;b&gt;判断&lt;/b&gt;"),
        "title の山括弧が escape されていない"
    );
    assert!(
        !html.contains("<b>判断</b>"),
        "title の <b> が生のまま出ている"
    );
}

// ── 実の正本 ──

fn load_yaml_at(dir: &Path, name: &str) -> Yaml {
    let text = fs::read_to_string(dir.join(name)).unwrap();
    YamlLoader::load_from_str(&text).unwrap().remove(0)
}

/// 実の判断の記録の id を file 名から集める（古い順）。
fn real_ids() -> Vec<String> {
    let mut ids: Vec<String> = fs::read_dir(design_intent().join("adr"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter_map(|n| {
            n.strip_suffix(".yaml")
                .filter(|s| s.starts_with("ADR-"))
                .map(str::to_string)
        })
        .collect();
    ids.sort_by_key(|id| id["ADR-".len()..].parse::<u32>().unwrap());
    assert!(!ids.is_empty(), "実の判断の記録が 1 本も無い");
    ids
}

/// 実の正本 1 本から一時 file へ書く。
fn real_face(td: &Path, id: &str) -> (PathBuf, String) {
    let out = td.join(format!("{}.html", id.to_ascii_lowercase()));
    let run = folio_face("adr", Some(id), &design_intent(), &out, "--write");
    assert_eq!(
        code(&run, "folio face --write"),
        0,
        "{id}: {}",
        stderr(&run)
    );
    let html = fs::read_to_string(&out).unwrap();
    (out, html)
}

/// 面の下端の prevnext の（前の href・前の名・次の href・次の名）。
fn prevnext(html: &str) -> (String, String, String, String) {
    let line = html
        .lines()
        .find(|l| l.starts_with("<nav class=\"prevnext\">"))
        .expect("prevnext が無い");
    let parts: Vec<&str> = line.split("<a href=\"").skip(1).collect();
    assert_eq!(parts.len(), 2, "prevnext の a が 2 つでない: {line}");
    let split = |p: &str| {
        let (href, rest) = p.split_once('"').unwrap();
        let name = rest
            .split("</span>")
            .nth(1)
            .unwrap()
            .split("</a>")
            .next()
            .unwrap();
        (href.to_string(), name.to_string())
    };
    let ((ph, pn), (nh, nn)) = (split(parts[0]), split(parts[1]));
    (ph, pn, nh, nn)
}

// ── 前 / 次（便 65・delivery-65.md §1 (e)1）──

#[test]
fn neighbor_adr_links_go_to_the_adjacent_record() {
    let td = temp_dir("neighbor-adjacent");
    let (_, html) = real_face(&td, "ADR-5");
    let _ = fs::remove_dir_all(&td);
    let (ph, pn, nh, nn) = prevnext(&html);
    assert_eq!(ph, "adr-4.html", "前の href");
    assert!(pn.contains("ADR-4"), "前の名に ADR-4 が無い: {pn}");
    assert_eq!(nh, "adr-6.html", "次の href");
    assert!(nn.contains("ADR-6"), "次の名に ADR-6 が無い: {nn}");
    // 名は id + 半角空白 + 題（題は正本の逐語を escape したもの）
    let a4 = load_yaml_at(&design_intent().join("adr"), "ADR-4.yaml");
    assert_eq!(pn, format!("ADR-4 {}", esc(a4["title"].as_str().unwrap())));
}

#[test]
fn neighbor_adr_first_and_last_fall_back_to_the_entrance() {
    let ids = real_ids();
    let td = temp_dir("neighbor-ends");
    let (_, first) = real_face(&td, &ids[0]);
    let last_id = ids.last().unwrap();
    let (_, last) = real_face(&td, last_id);
    let _ = fs::remove_dir_all(&td);
    assert_eq!(ids[0], "ADR-1");
    let (ph, pn, nh, _) = prevnext(&first);
    assert_eq!(
        (ph.as_str(), pn.as_str()),
        ("index.html", "入口"),
        "ADR-1 の前"
    );
    assert_eq!(nh, "adr-2.html", "ADR-1 の次");
    let (ph, _, nh, nn) = prevnext(&last);
    assert_eq!(
        (nh.as_str(), nn.as_str()),
        ("index.html", "入口"),
        "{last_id} の次"
    );
    let before = &ids[ids.len() - 2];
    assert_eq!(
        ph,
        format!("{}.html", before.to_ascii_lowercase()),
        "{last_id} の前"
    );
}

#[test]
fn face_adr_on_the_real_sources_passes_parts_check() {
    let td = temp_dir("parts");
    for id in real_ids() {
        let (out, html) = real_face(&td, &id);
        // 図の枠の数は各正本の figures の数（便 33・実測 0）
        let a = load_yaml_at(&design_intent().join("adr"), &format!("{id}.yaml"));
        let figures = a["figures"].as_vec().map_or(0, Vec::len);
        assert_eq!(
            html.matches("data-component=\"figure-panel\"").count(),
            figures,
            "{id}: figure-panel の数が figures の数と違う"
        );
        assert_eq!(svg_bodies(&html), figures, "{id}: 図の本体の数");
        let check = Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("parts")
            .arg("--check")
            .arg("--dir")
            .arg(design_intent())
            .arg("--page")
            .arg(format!("adr={}", out.display()))
            .output()
            .unwrap();
        assert_eq!(
            code(&check, "folio parts --check"),
            0,
            "{id}: {}{}",
            stdout(&check),
            stderr(&check)
        );
        assert!(
            stdout(&check).contains("違反 0"),
            "{id}: {}",
            stdout(&check)
        );
    }
    let _ = fs::remove_dir_all(&td);
}

#[test]
fn face_adr_census_on_the_real_sources_counts_and_verbatims() {
    let td = temp_dir("census");
    for id in real_ids() {
        let (_, html) = real_face(&td, &id);
        let a = load_yaml_at(&design_intent().join("adr"), &format!("{id}.yaml"));
        let count = |needle: &str| html.matches(needle).count();

        // 逐語（h1 は短い名・副題が title の逐語）
        let title = esc(a["title"].as_str().unwrap());
        assert!(
            html.contains(&format!("<h1>判断の記録 {id}</h1>")),
            "{id}: h1 が「判断の記録 <id>」でない"
        );
        assert!(
            html.contains(&format!("<p class=\"sub-title\">{title}</p>")),
            "{id}: 副題が title の逐語でない"
        );

        // 件数（案・採用・根拠）
        let options = a["options"].as_vec().unwrap();
        assert_eq!(
            count("data-component=\"item-row\""),
            options.len(),
            "{id}: 案の数"
        );
        assert_eq!(
            count("<span class=\"pill\">採用</span>"),
            1,
            "{id}: 採用は 1 つ"
        );
        let basis = a["basis"].as_vec().unwrap();
        // 根拠のリンク = basis の数 + 図の節の refs の数（図の枠の「根拠:」も同じ xref・便 33）
        // + 改訂の欄の行の数（相手の面へのリンク・便 137）
        let figures = a["figures"].as_vec().map_or(0, Vec::len);
        let fig_refs: usize = a["figures"].as_vec().map_or(0, |v| {
            v.iter()
                .map(|f| f["refs"].as_vec().map_or(0, Vec::len))
                .sum()
        });
        let revises = a["revises"].as_vec().map_or(0, Vec::len);
        // + ほかの記録から受けた改訂の行の数（改訂する側の面へのリンク・便 148）
        let revised_by = real_revised_by(&id).len();
        assert_eq!(
            count("class=\"xref\""),
            basis.len() + fig_refs + revises + revised_by,
            "{id}: 根拠のリンクの数（basis + 図の refs + revises + 受けた改訂）"
        );
        // 行き先の無い印は id の直後に付く（face_adr.rs の link_text）。散文の「（まだ分からない）」は数えない
        let targets = basis
            .iter()
            .map(|b| b.as_str().unwrap().to_string())
            .chain(a["revises"].as_vec().into_iter().flatten().map(|r| {
                r["target"].as_str().unwrap().to_string()
            }));
        for b in targets {
            assert!(
                !html.contains(&format!("{b}（まだ分からない）")),
                "{id}: 行き先の無い根拠が在る（{b}）"
            );
        }
        // 根拠の群の一覧（li の数 = 根拠の数・li は行き先のリンクと題の span）+ 改訂の欄の行（便 137）
        // + 受けた改訂の行（便 148）
        assert_eq!(
            count("<li><a class=\"xref\""),
            basis.len() + revises + revised_by,
            "{id}: ul.basis と改訂の欄と受けた改訂の li の数"
        );
        assert_eq!(count("</a><span>"), basis.len(), "{id}: 題の span の数");

        // 章の帯（5 + 図の章の有無）と承認欄の帯
        assert_eq!(
            count("data-component=\"chapter-deck-band\""),
            5 + usize::from(figures > 0) + 1,
            "{id}: 章の帯の数"
        );

        // 状態の名札
        let label = match a["status"].as_str().unwrap() {
            "proposed" => "提案中・拘束力なし",
            "accepted" => "発効",
            "retired" => "廃止",
            other => panic!("{id}: 状態「{other}」は 3 つのどれでもない"),
        };
        let status_line = html
            .lines()
            .find(|l| l.contains("class=\"cover-status\""))
            .unwrap_or_else(|| panic!("{id}: 状態の行が無い"));
        assert!(
            status_line.contains(label),
            "{id}: 状態の行に名札「{label}」が無い: {status_line}"
        );
    }
    let _ = fs::remove_dir_all(&td);
}

// ── check の 3 値 ──

#[test]
fn face_adr_check_has_three_values() {
    let (td, work) = fixture_copy("check");
    let out = td.join("adr-2.html");

    let write = folio_face("adr", Some("ADR-2"), &work, &out, "--write");
    let ok = folio_face("adr", Some("ADR-2"), &work, &out, "--check");
    let mut bytes = fs::read(&out).unwrap();
    bytes[0] ^= 0x20;
    fs::write(&out, &bytes).unwrap();
    let drift = folio_face("adr", Some("ADR-2"), &work, &out, "--check");
    fs::remove_file(&out).unwrap();
    let missing = folio_face("adr", Some("ADR-2"), &work, &out, "--check");
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&write, "write"), 0, "{}", stderr(&write));
    assert_eq!(code(&ok, "check（一致）"), 0, "{}", stderr(&ok));
    assert!(stdout(&ok).contains("folio face: OK"), "{}", stdout(&ok));
    assert_eq!(code(&drift, "check（不一致）"), 1, "{}", stderr(&drift));
    assert!(stderr(&drift).contains("DRIFT"), "{}", stderr(&drift));
    assert_eq!(code(&missing, "check（無い）"), 2, "{}", stderr(&missing));
    assert!(
        stderr(&missing).contains("面が無い"),
        "{}",
        stderr(&missing)
    );
}

// ── id の口 ──

#[test]
fn face_adr_id_is_required_and_only_on_the_adr_face() {
    let (td, work) = fixture_copy("id");
    let out = td.join("never.html");
    let no_id = folio_face("adr", None, &work, &out, "--write");
    let missing = folio_face("adr", Some("ADR-9"), &work, &out, "--write");
    let on_srs = folio_face("srs", Some("ADR-2"), &work, &out, "--write");
    let bad_shape = folio_face("adr", Some("foo"), &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(code(&no_id, "--id 無し"), 2, "{}", stderr(&no_id));
    assert!(stderr(&no_id).contains("--id が無い"), "{}", stderr(&no_id));
    assert_eq!(code(&missing, "無い id"), 2, "{}", stderr(&missing));
    assert_eq!(code(&on_srs, "面 srs に --id"), 2, "{}", stderr(&on_srs));
    assert!(
        stderr(&on_srs).contains("--id は面 adr と note にだけ付く"),
        "{}",
        stderr(&on_srs)
    );
    assert_eq!(
        code(&bad_shape, "id の形でない"),
        2,
        "{}",
        stderr(&bad_shape)
    );
    assert!(
        stderr(&bad_shape).contains("id の形でない"),
        "{}",
        stderr(&bad_shape)
    );
    assert!(!exists, "導出できないのに出力先に書いた");
}

// ── 導出できない入力 ──

#[test]
fn face_adr_unknown_when_the_status_is_outside_the_table() {
    unknown(
        "status",
        |t| t.replacen("status: proposed", "status: draft", 1),
        "状態",
    );
}

#[test]
fn face_adr_unknown_when_a_verdict_is_outside_the_table() {
    unknown(
        "verdict",
        |t| t.replacen("verdict: rejected", "verdict: maybe", 1),
        "判定",
    );
}

#[test]
fn face_adr_unknown_when_basis_is_not_a_list() {
    unknown(
        "basis",
        |t| t.replacen("basis: [P-1, A-1, R-1, FR1, AC1, ADR-1]", "basis: P-1", 1),
        "一覧でない",
    );
}

// ── 行き先の無い根拠・状態 ──

#[test]
fn face_adr_marks_a_basis_id_without_a_target() {
    let (run, html) = mutated("unresolved", |t| {
        t.replacen(
            "basis: [P-1, A-1, R-1, FR1, AC1, ADR-1]",
            "basis: [P-1, A-1, R-1, FR1, AC1, ADR-1, FR9]",
            1,
        )
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(
        html.contains("FR9（まだ分からない）"),
        "行き先の無い id に印が無い"
    );
    assert!(
        !html.contains("srs.html#fr9"),
        "行き先の無い id をリンクにした"
    );
}

#[test]
fn face_adr_shows_the_accepted_and_retired_states() {
    let (accepted, html) = mutated("accepted", |t| {
        t.replacen("status: proposed", "status: accepted", 1)
            .replacen(
                "amends: []",
                "amends: []\napproval: {who: 持ち主, date: 2026-09-07, ruling: f2-648.40 notes 2026-09-07, verbatim: 承認する, surface: R-8}",
                1,
            )
    });
    assert_eq!(code(&accepted, "accepted"), 0, "{}", stderr(&accepted));
    assert!(
        html.contains("発効・拘束力あり（承認 2026-09-07）"),
        "accepted の状態の行が無い"
    );
    assert_eq!(
        html.matches("<div class=\"sign\">").count(),
        1,
        "承認欄の sign が 1 つでない"
    );

    let (retired, html) = mutated("retired", |t| {
        t.replacen("status: proposed", "status: retired", 1)
            .replacen("amends: []", "amends: []\nsuperseded_by: ADR-1", 1)
    });
    assert_eq!(code(&retired, "retired"), 0, "{}", stderr(&retired));
    assert!(html.contains("廃止 → 後継 "), "retired の状態の行が無い");
    assert!(html.contains("href=\"adr-1.html\""), "後継へのリンクが無い");
}

#[test]
fn face_adr_resolves_a_statement_id_to_its_article() {
    let (run, html) = mutated("statement", |t| {
        t.replacen("basis: [P-1,", "basis: [P-1.1, P-1.9, P-1,", 1)
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(
        html.contains("<a class=\"xref\" href=\"constitution.html#p-1\">P-1.1</a>"),
        "枝番付きの id が条の anchor へ跳んでいない"
    );
    assert!(
        html.contains("P-1.9（まだ分からない）"),
        "無い規範文に印が無い"
    );
}

// ── 列挙の分割（便 27 §1 (b)）──

#[test]
fn face_adr_splits_the_head_of_sentence_enumeration_into_items() {
    let html = with_context("items", "前置き。(1) あ。(2) い。");
    assert_eq!(
        html.matches("<p class=\"intro\">前置き。</p>").count(),
        1,
        "前置きの p が 1 つでない: {html}"
    );
    assert!(
        html.contains("<ol class=\"items\">\n<li>あ。</li>\n<li>い。</li>\n</ol>"),
        "列挙が li 2 つに分かれていない: {html}"
    );
}

#[test]
fn face_adr_keeps_an_enumeration_inside_one_sentence_as_a_paragraph() {
    let html = with_context("items-inline", "(1) あ、(2) い。");
    assert!(
        html.contains("<p>(1) あ、(2) い。</p>"),
        "文の中の列挙が段落のまま出ていない: {html}"
    );
    assert!(
        !html.contains("ol class=\"items\""),
        "文の中の列挙を分けた: {html}"
    );
}

#[test]
fn face_adr_keeps_a_skipping_enumeration_as_a_paragraph() {
    let html = with_context("items-skip", "前置き。(1) あ。(3) い。");
    assert!(
        html.contains("<p>前置き。(1) あ。(3) い。</p>"),
        "番号の飛ぶ列挙が段落のまま出ていない: {html}"
    );
    assert!(
        !html.contains("ol class=\"items\""),
        "印が 1 つなのに分けた: {html}"
    );
}

// ── 根拠の 4 群（便 27 §1 (c)）──

#[test]
fn face_adr_groups_the_basis_ids_into_four_cards_with_titles() {
    let html = fixture_html("groups");
    assert_eq!(
        html.matches("<div class=\"card\">").count(),
        4,
        "根拠の card が 4 枚でない: {html}"
    );
    assert!(
        html.contains("style=\"--band-n:4\""),
        "格子の列が 4 でない: {html}"
    );
    for cid in [
        "憲法の条（2）",
        "数値の表（1）",
        "要件書（2）",
        "判断の記録（1）",
    ] {
        assert!(
            html.contains(&format!("<div class=\"cid\">{cid}</div>")),
            "群の名札「{cid}」が無い: {html}"
        );
    }
    assert!(
        html.contains(
            "<li><a class=\"xref\" href=\"constitution.html#p-1\">P-1</a><span>判断する道具を作らない</span></li>"
        ),
        "条の題が無い: {html}"
    );
    assert!(
        html.contains(
            "<li><a class=\"xref\" href=\"constitution.html#r-1\">R-1</a><span>AI へ常時渡す説明文の合計</span></li>"
        ),
        "数値の表の題（what）が無い: {html}"
    );
    // title の欄を持たない stub（adr/ADR-1.yaml）はリンクだけ・題の span を出さず面も 2 にしない
    assert!(
        html.contains("<li><a class=\"xref\" href=\"adr-1.html\">ADR-1</a></li>"),
        "stub の判断の記録の li がリンクだけでない: {html}"
    );
    // 撤退条件の card は格子の外（callout を閉じた後）
    let lines: Vec<&str> = html.lines().collect();
    let callout = lines
        .iter()
        .position(|l| l.contains("data-component=\"section-lead-callout\""))
        .expect("callout が無い");
    let retreat = lines
        .iter()
        .position(|l| l.starts_with("<div class=\"card retreat\">"))
        .expect("撤退条件の card が無い");
    assert!(callout < retreat, "撤退条件が callout の前に在る");
    assert_eq!(
        lines[retreat - 1],
        "</div>",
        "撤退条件の card が callout の中に在る: {html}"
    );
}

#[test]
fn face_adr_shows_no_title_for_a_basis_id_without_a_target() {
    let (run, html) = mutated("group-unresolved", |t| {
        t.replacen(
            "basis: [P-1, A-1, R-1, FR1, AC1, ADR-1]",
            "basis: [P-1, A-1, R-1, FR1, AC1, ADR-1, FR9]",
            1,
        )
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(
        html.contains("<div class=\"cid\">要件書（3）</div>"),
        "要件書の群が 3 件でない: {html}"
    );
    assert!(
        html.contains("<li>FR9（まだ分からない）</li>"),
        "行き先の無い id の li が印だけでない: {html}"
    );
}

// ── 図の章（便 33・FR15）──

#[test]
fn face_adr_embeds_the_figure_in_a_figure_panel_with_label_and_refs() {
    let html = fixture_html("figure");
    assert!(
        html.contains(
            "<li><a href=\"#s6\"><span class=\"n\">06</span><span class=\"k\">図</span><span class=\"t\">1 枚</span></a></li>"
        ),
        "toc に 06「図」が無い: {html}"
    );
    assert_eq!(
        html.matches("<figure data-component=\"figure-panel\" data-role=\"diagram\" id=\"fig-1\">")
            .count(),
        1,
        "figure-panel が 1 つでない: {html}"
    );
    assert_eq!(svg_bodies(&html), 1, "図の本体が 1 つでない");
    assert!(
        html.contains("<div class=\"fig-title\"><span class=\"fn\">図 1</span>見本の図 <span class=\"fig-tools\">"),
        "fig-title に caption の逐語が無い: {html}"
    );
    assert!(
        html.contains(
            "<figcaption><span class=\"ver\">図 1 · 構成図（architecture） · fig-1 · 根拠: <a class=\"xref\" href=\"srs.html#fr1\">FR1</a></span></figcaption>"
        ),
        "figcaption に型の名札・id・根拠のリンクが無い: {html}"
    );
    assert!(
        html.contains(
            "<span class=\"m\"><span class=\"k\">図</span><span class=\"v\">1 枚</span></span>"
        ),
        "cover-meta に「図 1 枚」が無い: {html}"
    );
    // 章 06 の帯は「改訂と帰結」の後・承認欄の前
    let s5 = html.find("<section id=\"s5\"").expect("章 05 が無い");
    let s6 = html.find("<section id=\"s6\"").expect("章 06 が無い");
    let ap = html.find("<section id=\"approval\"").expect("承認欄が無い");
    assert!(s5 < s6 && s6 < ap, "章 06 の置き場が違う");
    assert!(
        html.contains("<h2>図 1 枚</h2>"),
        "章 06 の h2 が「図 1 枚」でない: {html}"
    );
    assert!(
        html.contains("<dt>figures</dt><dd>1</dd>"),
        "機械のための面に figures が無い: {html}"
    );
}

#[test]
fn face_adr_without_figures_has_no_figure_chapter() {
    let html = figureless_html("no-figures");
    assert_eq!(
        html.matches("data-component=\"figure-panel\"").count(),
        0,
        "図が無いのに figure-panel が在る"
    );
    assert!(
        !html.contains("<section id=\"s6\""),
        "図が無いのに図の章が在る"
    );
    assert_eq!(svg_bodies(&html), 0, "図が無いのに図の本体が在る");
    assert!(
        !html.contains("<li><a href=\"#s6\">") && !html.contains("<span class=\"k\">図</span>"),
        "図が無いのに toc か cover-meta に図の項が在る"
    );
    assert!(
        !html.contains("<dt>figures</dt>"),
        "図が無いのに機械のための面に figures が在る"
    );
    assert!(html.contains("全 6 章"), "図なしの面が全 6 章でない");
}

#[test]
fn face_adr_figure_chapter_is_the_only_difference_from_the_figureless_face() {
    let frozen = fs::read_to_string(fixture().join("expected-adr.html")).unwrap();
    let without = figureless_html("outside");
    // 図の章（s6 の帯から承認欄の帯の直前まで）・toc の 06・cover-meta の図・foot の figures を図ありの面から抜く
    let a = cut(&frozen, "<section id=\"s6\"", "<section id=\"approval\"");
    let a = cut_line(&a, "<li><a href=\"#s6\">");
    let a = cut_line(&a, "<span class=\"m\"><span class=\"k\">図</span>");
    let a = a.replace("<dt>figures</dt><dd>1</dd>", "");
    // 章の数から数える字（全 N 章・k/N）だけは正本から数えた数（γ）なので揃える
    let a = a
        .replace("全 7 章", "全 6 章")
        .replace("/7</span>", "/6</span>");
    if a != without {
        let at = a
            .bytes()
            .zip(without.bytes())
            .position(|(x, y)| x != y)
            .unwrap_or(a.len().min(without.len()));
        let show = |s: &str| {
            String::from_utf8_lossy(&s.as_bytes()[at.saturating_sub(120)..(at + 200).min(s.len())])
                .into_owned()
        };
        panic!(
            "図の章の外が違う（最初の差 {at} byte 目）\n--- 凍結（図を抜いた）\n{}\n--- 図なし\n{}",
            show(&a),
            show(&without)
        );
    }
}

#[test]
fn face_adr_unknown_when_a_figure_fails_the_tool_check_and_keeps_the_previous_face() {
    let (td, work) = fixture_copy("fig-fails");
    // layout を消すと道具の検査（showcase）に落ちる
    edit(&work.join("adr/ADR-2.yaml"), |t| {
        t.replacen(
            "      layout: {mode: grid, cols: 2, gapX: 70, gapY: 110, cellW: 160, cellH: 70}\n",
            "",
            1,
        )
    });
    let out = td.join("adr-2.html");
    let before = "<!DOCTYPE html>\n前の面\n".as_bytes().to_vec();
    fs::write(&out, &before).unwrap();
    let run = folio_face("adr", Some("ADR-2"), &work, &out, "--write");
    let after = fs::read(&out).unwrap();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{}", stderr(&run));
    assert!(
        stderr(&run).contains("図の道具の検査を通らない"),
        "{}",
        stderr(&run)
    );
    assert_eq!(after, before, "図が通らないのに前の面を上書きした");
}

#[test]
fn face_adr_unknown_when_a_figure_type_is_not_a_tool_type() {
    unknown(
        "fig-type",
        |t| t.replacen("type: archify-architecture", "type: pipeline-rail", 1),
        "図の道具の型でない",
    );
}

#[test]
fn face_adr_unknown_when_the_figure_tool_is_absent() {
    let (td, work) = fixture_copy("no-tool");
    fs::remove_file(td.join("vendor/archify/bin/archify.mjs")).unwrap();
    let out = td.join("adr-2.html");
    let run = folio_face("adr", Some("ADR-2"), &work, &out, "--write");
    let exists = out.exists();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("まだ分からない"), "{}", stderr(&run));
    assert!(stderr(&run).contains("図の道具が無い"), "{}", stderr(&run));
    assert!(!exists, "導出できないのに面を書いた");
}

// ── AC14 の赤 ──

#[test]
fn face_adr_parts_check_fails_on_the_red_fixture() {
    let page = fixture().join("adr/extra-class.html");
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("parts")
        .arg("--check")
        .arg("--dir")
        .arg(design_intent())
        .arg("--page")
        .arg(format!("adr={}", page.display()))
        .output()
        .unwrap();
    assert_eq!(code(&out, "folio parts --check"), 1, "{}", stdout(&out));
    assert!(
        stdout(&out).contains("部品目録に無い class「not-in-catalog」"),
        "{}",
        stdout(&out)
    );
}

// ── mode ──

#[test]
fn face_adr_mode_is_exactly_one() {
    for args in [&["--check", "--write"][..], &[][..]] {
        let out = Command::new(env!("CARGO_BIN_EXE_folio"))
            .arg("face")
            .arg("--face")
            .arg("adr")
            .arg("--id")
            .arg("ADR-2")
            .arg("--dir")
            .arg(fixture())
            .arg("--out")
            .arg(std::env::temp_dir().join("folio-face-adr-never-written.html"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
    }
}

// ── 表紙の撤退条件の 1 文（便 71・天井の 12 周目の読みやすさ F-11）──

/// 表紙の meta_span「撤退条件」の値。
fn cover_retreat(html: &str) -> String {
    let key = "<span class=\"m\"><span class=\"k\">撤退条件</span><span class=\"v\">";
    let at = html.find(key).expect("表紙に撤退条件が無い") + key.len();
    let end = html[at..].find("</span>").unwrap();
    html[at..at + end].to_string()
}

#[test]
fn label_fix_retreat_sentence_on_the_cover() {
    // 写しの ADR-2 の retreat.kind を measure にした面
    let (run, html) = mutated("label-retreat", |t| {
        t.replacen("retreat: {kind: ruling,", "retreat: {kind: measure,", 1)
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert_eq!(cover_retreat(&html), "数えた値が条件を超えたら捨てる");
    assert!(
        !html.contains("<span class=\"k\">撤退条件</span><span class=\"v\">数えた値</span>"),
        "表紙に種別の名だけの値が残る"
    );
    // 変異なしの写し（ruling）も 1 文
    assert_eq!(
        cover_retreat(&fixture_html("label-retreat-ruling")),
        "持ち主の裁定で捨てる"
    );
}

// ── 用語集への札（便 74・delivery-74.md §1 (b)(f)）──

/// 面の doc-locator の行の中の用語集の札の a 要素（札の span が 1 つだけ在ること・行き先を測る）。
fn glossary_chip_a(html: &str) -> String {
    let line = html
        .lines()
        .find(|l| l.starts_with("<p class=\"doc-locator\">"))
        .expect("doc-locator の行が無い");
    assert_eq!(
        line.matches("<span class=\"annex-chips\">").count(),
        1,
        "doc-locator の行に札の span が 1 つでない: {line}"
    );
    let chips = &line[line.find("<span class=\"annex-chips\">").unwrap()..];
    let start = chips.find("<a ").expect("札に a が無い");
    let end = chips[start..].find("</a>").expect("札の a が閉じていない") + start + "</a>".len();
    let a = chips[start..end].to_string();
    assert!(
        a.starts_with("<a href=\"constitution.html#s7\">"),
        "札の行き先が constitution.html#s7 でない: {a}"
    );
    a
}

#[test]
fn f74_adr_foot_links_to_the_glossary() {
    let a = glossary_chip_a(&fixture_html("f74-chip"));
    assert_eq!(
        a,
        "<a href=\"constitution.html#s7\">付録 語彙 <span class=\"cnt\">2 語 → 憲法 §7</span></a>"
    );
    // 語彙の正本に 1 語足すと数が 3 になる（数は正本から来る）
    let (td, work) = fixture_copy("f74-chip-3");
    edit(&work.join("vocabulary.yaml"), |t| {
        t.replacen(
            "\nfield_terms:",
            "  - id: third\n    term: 三つ目\n    en: null\n    short: 足した語\n    def: 足した語。\n\nfield_terms:",
            1,
        )
    });
    let out = td.join("adr-2.html");
    let run = folio_face("adr", Some("ADR-2"), &work, &out, "--write");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert!(
        glossary_chip_a(&html).contains("<span class=\"cnt\">3 語 → 憲法 §7</span>"),
        "語を足しても札の数が 3 にならない"
    );
}

#[test]
fn f74_glossary_chip_is_verbatim_the_index_chip() {
    let a = glossary_chip_a(&fixture_html("f74-verbatim"));
    let index = fs::read_to_string(fixture().join("expected-index.html")).unwrap();
    assert!(
        index.contains(&a),
        "札の a が入口の面の札と逐語で一致しない: {a}"
    );
}

// ── 章 05 の注の折りたたみ（便 82・docs/design/delivery-82.md §1 (c)・天井の 17 周目の読みやすさ F-3）──

const NOTE_FOLD: &str = "<details class=\"note\"><summary>注</summary><div><p>";

#[test]
fn f82_adr_note_is_folded() {
    let td = temp_dir("f82-fold");
    let (_, html) = real_face(&td, "ADR-8");
    let _ = fs::remove_dir_all(&td);
    let note = load_yaml_at(&design_intent(), "adr/ADR-8.yaml")["note"]
        .as_str()
        .expect("ADR-8 の note が文字列でない")
        .to_string();
    assert_eq!(
        html.matches(NOTE_FOLD).count(),
        1,
        "注の折りたたみが 1 つでない"
    );
    assert_eq!(html.matches("<h3>注</h3>").count(), 0, "h3 の注が残っている");
    let want = format!("{NOTE_FOLD}{}</p></div></details>", esc(&note));
    assert!(html.contains(&want), "折りたたみの中の字が note の逐語でない");
    // 章 05 の中に在る
    let ch5 = html.find("<div class=\"chapbody\">").is_some()
        && html[..html.find(NOTE_FOLD).unwrap()].contains("この判断で変わること");
    assert!(ch5, "注の折りたたみが章 05 の中に無い");
}

#[test]
fn f82_adr_without_a_note_has_no_fold() {
    let html = fixture_html("f82-nofold");
    assert_eq!(html.matches(NOTE_FOLD).count(), 0, "注の無い正本に折りたたみが在る");
    assert_eq!(html.matches("<summary>注</summary>").count(), 0);
    assert_eq!(html.matches("<h3>注</h3>").count(), 0, "注の無い正本に h3 の注が在る");
}

// ── 改訂の欄（便 137・docs/design/delivery-137.md §1 (c)・天井の 34 周目の読みやすさ F-1）──

/// 表紙の札 1 つの字面（名札と件数）。
fn cover_count(label: &str, n: usize) -> String {
    format!("<span class=\"m\"><span class=\"k\">{label}</span><span class=\"v\">{n} 件</span></span>")
}

/// 札が 2 つ（条文の改訂・判断の記録の改訂）で、名札「改訂」だけの札が無い。
fn assert_cover_counts(html: &str, amends: usize, revises: usize, what: &str) {
    assert_eq!(
        html.matches(&cover_count("条文の改訂", amends)).count(),
        1,
        "{what}: 札「条文の改訂 {amends} 件」が 1 つでない"
    );
    assert_eq!(
        html.matches(&cover_count("判断の記録の改訂", revises)).count(),
        1,
        "{what}: 札「判断の記録の改訂 {revises} 件」が 1 つでない"
    );
    assert!(
        !html.contains("<span class=\"k\">改訂</span>"),
        "{what}: 名札「改訂」だけの札が残る"
    );
}

/// 章 05（s5 の帯から次の帯の直前まで）。
fn chapter5(html: &str) -> &str {
    let s5 = html.find("<section id=\"s5\"").expect("章 05 が無い");
    let end = html[s5 + 1..]
        .find("<section id=")
        .map_or(html.len(), |e| s5 + 1 + e);
    &html[s5..end]
}

#[test]
fn f137_revises_rows_show_the_link_decision_kind_and_summary() {
    let (run, html) = mutated("f137-rows", |t| {
        t.replacen(
            "amends: []\n",
            "amends: []\nrevises:\n  - {target: ADR-1, decision: (2), kind: narrow, summary: 「<b>前</b>」を狭く読む}\n  - {target: ADR-9, decision: (1), kind: widen, summary: 相手の無い改訂}\n",
            1,
        )
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert_cover_counts(&html, 0, 2, "revises 2 行");
    assert_eq!(
        html.matches("<p>条文の改訂なし</p>").count(),
        1,
        "断りが「条文の改訂なし」の 1 つでない"
    );
    assert!(
        !html.contains("判断の記録の改訂なし"),
        "revises が在るのに断りが在る"
    );
    assert_eq!(
        html.matches("<h3>判断の記録の改訂</h3>").count(),
        1,
        "h3「判断の記録の改訂」が 1 つでない"
    );
    assert!(!html.contains("<h3>条文の改訂</h3>"), "amends が空なのに h3 が在る");
    let row1 = "<li><a class=\"xref\" href=\"adr-1.html\">ADR-1</a> の決定 (2) を狭める: 「&lt;b&gt;前&lt;/b&gt;」を狭く読む</li>";
    let row2 = "<li>ADR-9（まだ分からない） の決定 (1) を広げる: 相手の無い改訂</li>";
    let ch5 = chapter5(&html);
    let (Some(r1), Some(r2)) = (ch5.find(row1), ch5.find(row2)) else {
        panic!("章 05 に revises の行が逐語で無い: {ch5}");
    };
    assert!(r1 < r2, "revises の行が正本の順でない");
    assert!(
        ch5.find("<h3>判断の記録の改訂</h3>").unwrap() < r1,
        "revises の行が h3 の下に無い"
    );
    assert_eq!(html.matches(row1).count(), 1);
    assert_eq!(html.matches(row2).count(), 1);
    assert!(!html.contains("<b>前</b>"), "summary の山括弧が生のまま");
    assert!(
        !html.contains("href=\"adr-9.html\""),
        "行き先の無い相手をリンクにした"
    );
    let ap = html.find("<section id=\"approval\"").expect("承認欄が無い");
    assert!(html.find(row2).unwrap() < ap, "revises の行が承認欄の後");
    assert!(
        html.contains("<dt>amends</dt><dd>0</dd><dt>revises</dt><dd>2</dd>"),
        "機械のための面が amends 0 と revises 2 を数えない"
    );
}

#[test]
fn f137_a_record_without_revisions_says_both_are_none() {
    let html = fixture_html("f137-none");
    assert_cover_counts(&html, 0, 0, "改訂の無い記録");
    assert_eq!(
        chapter5(&html)
            .matches("<p>条文の改訂なし・判断の記録の改訂なし</p>")
            .count(),
        1,
        "断りが両方の 1 段落でない"
    );
    assert!(!html.contains("<p>条文の改訂なし</p>"), "片方だけの断りが在る");
    assert!(
        !html.contains("<p>判断の記録の改訂なし</p>"),
        "片方だけの断りが在る"
    );
    assert!(!html.contains("<h3>条文の改訂</h3>"), "空の欄に h3 が在る");
    assert!(!html.contains("<h3>判断の記録の改訂</h3>"), "空の欄に h3 が在る");
    assert!(
        html.contains("<dt>amends</dt><dd>0</dd><dt>revises</dt><dd>0</dd>"),
        "機械のための面が 0 と 0 を数えない"
    );
}

#[test]
fn f137_amends_go_under_their_own_heading() {
    let (run, html) = mutated("f137-amends", |t| {
        t.replacen(
            "amends: []\n",
            "amends:\n  - {version: v1.1, target: P-1, field: text, previous_text: 前の字, new_text: 後の<i>字</i>}\n",
            1,
        )
    });
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    assert_cover_counts(&html, 1, 0, "amends 1 行");
    assert_eq!(
        chapter5(&html).matches("<p>判断の記録の改訂なし</p>").count(),
        1,
        "断りが「判断の記録の改訂なし」の 1 つでない"
    );
    assert!(!html.contains("条文の改訂なし"), "amends が在るのに断りが在る");
    assert!(
        chapter5(&html).contains(
            "<h3>条文の改訂</h3>\n<ul>\n<li>v1.1 P-1.text: 「前の字」→「後の&lt;i&gt;字&lt;/i&gt;」</li>\n</ul>"
        ),
        "amends の行が h3「条文の改訂」の下に逐語で無い: {html}"
    );
    assert!(!html.contains("<h3>判断の記録の改訂</h3>"), "revises が無いのに h3 が在る");
    assert!(
        html.contains("<dt>amends</dt><dd>1</dd><dt>revises</dt><dd>0</dd>"),
        "機械のための面が amends 1 と revises 0 を数えない"
    );
}

#[test]
fn f137_unknown_when_a_revise_kind_is_outside_the_table() {
    unknown(
        "f137-kind",
        |t| {
            t.replacen(
                "amends: []\n",
                "amends: []\nrevises:\n  - {target: ADR-1, decision: (2), kind: shrink, summary: 向きが表の外}\n",
                1,
            )
        },
        "改訂の向き の表に無い値「shrink」",
    );
}

#[test]
fn f137_real_sources_draw_every_revises_row() {
    let td = temp_dir("f137-census");
    let mut drawn = 0;
    for id in real_ids() {
        let (_, html) = real_face(&td, &id);
        let a = load_yaml_at(&design_intent().join("adr"), &format!("{id}.yaml"));
        let amends = a["amends"].as_vec().map_or(0, Vec::len);
        let revises = a["revises"].as_vec().cloned().unwrap_or_default();
        assert_cover_counts(&html, amends, revises.len(), &id);
        assert_eq!(
            html.matches("<h3>判断の記録の改訂</h3>").count(),
            usize::from(!revises.is_empty()),
            "{id}: h3「判断の記録の改訂」の有無"
        );
        assert_eq!(
            html.matches("<h3>条文の改訂</h3>").count(),
            usize::from(amends > 0),
            "{id}: h3「条文の改訂」の有無"
        );
        let none = match (amends == 0, revises.is_empty()) {
            (true, true) => Some("条文の改訂なし・判断の記録の改訂なし"),
            (true, false) => Some("条文の改訂なし"),
            (false, true) => Some("判断の記録の改訂なし"),
            (false, false) => None,
        };
        if let Some(none) = none {
            assert_eq!(
                chapter5(&html).matches(&format!("<p>{none}</p>")).count(),
                1,
                "{id}: 断り「{none}」が 1 つでない"
            );
        }
        for r in &revises {
            let target = r["target"].as_str().unwrap();
            let kind = match r["kind"].as_str().unwrap() {
                "narrow" => "狭める",
                "widen" => "広げる",
                other => panic!("{id}: 向き「{other}」は 2 つのどれでもない"),
            };
            let row = format!(
                "<li><a class=\"xref\" href=\"{}.html\">{target}</a> の決定 {} を{kind}: {}</li>",
                target.to_ascii_lowercase(),
                esc(r["decision"].as_str().unwrap()),
                esc(r["summary"].as_str().unwrap())
            );
            assert_eq!(
                chapter5(&html).matches(&row).count(),
                1,
                "{id}: revises の行が逐語で 1 回ない: {row}"
            );
            drawn += 1;
        }
        assert!(
            html.contains(&format!(
                "<dt>amends</dt><dd>{amends}</dd><dt>revises</dt><dd>{}</dd>",
                revises.len()
            )),
            "{id}: 機械のための面の件数"
        );
    }
    let _ = fs::remove_dir_all(&td);
    assert!(drawn > 0, "実の正本の revises の行を 1 つも数えていない");
}

// ── 便 146: 鮮度の札と足の行の日付は承認欄の日付（提案中は読まない・無ければ記録の日付と名 生成・
// docs/design/delivery-146.md §1 (c) の 5・6）──

/// 鮮度の札の頭（名・日付・記録の id まで）。
fn adr_stamp(dated: &str, date: &str, id: &str) -> String {
    format!("<span data-component=\"freshness-stamp\">{dated} <b>{date}</b> · <b>{id}</b>（")
}

/// 判断の記録の面の足の行の日付の部分。
fn adr_foot(id: &str, dated: &str, date: &str) -> String {
    format!(" · 判断の記録 {id}（{dated} {date}）· 手で直さない</p>")
}

fn once_in(html: &str, want: &str, what: &str) {
    assert_eq!(html.matches(want).count(), 1, "{what}「{want}」がちょうど 1 つでない");
}

#[test]
fn f146_adr_stamp_and_foot_follow_the_approval() {
    let approval = "amends: []\napproval: {who: 持ち主, date: 2026-09-08, ruling: f2-648.219 notes, verbatim: 承認する, surface: R-8}";
    let date_cell = "<span class=\"m\"><span class=\"k\">日付</span><span class=\"v\">2026-09-06</span></span>";
    // (写しの名, 状態, 承認欄を足すか, 日付の名, 日付)
    let cases = [
        ("f146-proposed", "proposed", false, "生成", "2026-09-06"),
        ("f146-proposed-ap", "proposed", true, "生成", "2026-09-06"),
        ("f146-accepted", "accepted", true, "承認", "2026-09-08"),
        ("f146-retired-ap", "retired", true, "承認", "2026-09-08"),
        ("f146-retired", "retired", false, "生成", "2026-09-06"),
    ];
    for (case, status, with_approval, dated, date) in cases {
        let (run, html) = mutated(case, |t| {
            let mut t = t.replacen("status: proposed", &format!("status: {status}"), 1);
            if with_approval {
                t = t.replacen("amends: []", approval, 1);
            }
            if status == "retired" {
                t = t.replacen("amends: []", "amends: []\nsuperseded_by: ADR-1", 1);
            }
            if status == "proposed" && !with_approval {
                t.push_str("# 変異なし\n");
            }
            t
        });
        assert_eq!(code(&run, case), 0, "{case}: {}", stderr(&run));
        once_in(&html, &adr_stamp(dated, date, "ADR-2"), &format!("{case} の鮮度の札"));
        once_in(&html, &adr_foot("ADR-2", dated, date), &format!("{case} の足の行"));
        once_in(&html, date_cell, &format!("{case} の表紙の日付の枡"));
        if dated == "承認" {
            assert!(
                !html.contains("<span data-component=\"freshness-stamp\">生成 "),
                "{case}: 承認の日付の在る面に 生成 の札が在る"
            );
        }
    }
}

#[test]
fn f146_real_adr_faces_date_the_approval() {
    let td = temp_dir("f146-real");
    let mut approved = 0;
    for id in real_ids() {
        let (_, html) = real_face(&td, &id);
        let a = load_yaml_at(&design_intent().join("adr"), &format!("{id}.yaml"));
        // 歯の側の手書きの読み: 提案中でなく承認欄が表なら承認欄の date・無ければ記録の date と名 生成
        let status = a["status"].as_str().expect("status が無い");
        let (dated, date) = match a["approval"]["date"].as_str() {
            Some(d) if status != "proposed" && a["approval"].as_hash().is_some() => {
                approved += 1;
                ("承認", esc(d))
            }
            _ => ("生成", esc(a["date"].as_str().expect("date が無い"))),
        };
        once_in(&html, &adr_stamp(dated, &date, &id), &format!("{id} の鮮度の札"));
        once_in(&html, &adr_foot(&id, dated, &date), &format!("{id} の足の行"));
    }
    let _ = fs::remove_dir_all(&td);
    assert!(approved > 0, "承認欄を持つ実の判断の記録を 1 本も数えていない");
}

// ── 便 148: ほかの判断の記録から受けた改訂の逆向きの行（docs/design/delivery-148.md §1 (c) の 2〜6・天井の 40 周目の
// 読みやすさ F-3）──

/// 写しの ADR-1（改訂する側）の本文（状態と revises の行・行は yaml の一覧の要素の字面のまま）。
fn reviser(status: &str, rows: &str) -> String {
    format!("id: ADR-1\nstatus: {status}\nrevises:\n{rows}")
}

/// 写しの ADR-1 を `adr1` に書き換え、ADR-2 の面を `--write` で組む（面が出来ていなければ本文は空）。
fn revised_face(case: &str, adr1: &str) -> (Output, String) {
    let (td, work) = fixture_copy(case);
    fs::write(work.join("adr/ADR-1.yaml"), adr1).unwrap();
    let out = td.join("adr-2.html");
    let run = folio_face("adr", Some("ADR-2"), &work, &out, "--write");
    let html = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&td);
    (run, html)
}

/// 表紙の逆向きの札の字面。
fn revised_by_badge(n: usize) -> String {
    cover_count("ほかの判断の記録による改訂", n)
}

/// 逆向きの 1 行の字面（decision と summary は正本の逐語・escape は歯の側）。
fn revised_by_row(by: &str, decision: &str, kind: &str, summary: &str) -> String {
    format!(
        "<li><a class=\"xref\" href=\"{}.html\">{by}</a> がこの判断の決定 {} を{kind}: {}</li>",
        by.to_ascii_lowercase(),
        esc(decision),
        esc(summary)
    )
}

/// 歯の側の手書きの読み: 実の正本の発効の記録の revises のうち `id` を相手にする行を、改訂する側の id の数の順・
/// 正本の順に（改訂する側・decision・向きの名札・summary）。
fn real_revised_by(id: &str) -> Vec<(String, String, &'static str, String)> {
    let mut rows = Vec::new();
    for by in real_ids() {
        let a = load_yaml_at(&design_intent().join("adr"), &format!("{by}.yaml"));
        if by == id || a["status"].as_str() != Some("accepted") {
            continue;
        }
        for r in a["revises"].as_vec().into_iter().flatten() {
            if r["target"].as_str() != Some(id) {
                continue;
            }
            let kind = match r["kind"].as_str().unwrap() {
                "narrow" => "狭める",
                "widen" => "広げる",
                other => panic!("{by}: 向き「{other}」は 2 つのどれでもない"),
            };
            rows.push((
                by.clone(),
                r["decision"].as_str().unwrap().to_string(),
                kind,
                r["summary"].as_str().unwrap().to_string(),
            ));
        }
    }
    rows
}

/// 写しの ADR-1 の 3 行（ADR-2 の決定 (2) を狭める・相手の違う ADR-9・ADR-2 の決定 (1) を広げる）。
const F148_ROWS: &str = "  - {target: ADR-2, decision: (2), kind: narrow, summary: 「<b>前</b>」を狭く読む}\n  - {target: ADR-9, decision: (1), kind: widen, summary: 相手の違う行}\n  - {target: ADR-2, decision: (1), kind: widen, summary: 範囲を広げる}\n";

#[test]
fn f148_revised_by_rows_link_back_to_the_reviser() {
    let base = fixture_html("f148-base");
    let (run, html) = revised_face("f148-rows", &reviser("accepted", F148_ROWS));
    assert_eq!(code(&run, "folio face --write"), 0, "{}", stderr(&run));
    once_in(&html, &revised_by_badge(2), "表紙の逆向きの札");
    assert_eq!(
        html.matches("<span class=\"k\">ほかの判断の記録による改訂</span>").count(),
        1,
        "逆向きの札が 1 つでない"
    );
    // 正本の欄の札・断り・機械のための面は変わらない
    assert_cover_counts(&html, 0, 0, "受けた改訂 2 行");
    let ch5 = chapter5(&html);
    assert_eq!(
        ch5.matches("<p>条文の改訂なし・判断の記録の改訂なし</p>").count(),
        1,
        "章 05 の断りが変わった"
    );
    assert!(
        html.contains("<dt>amends</dt><dd>0</dd><dt>revises</dt><dd>0</dd><dt>figures</dt>"),
        "機械のための面が変わった"
    );
    let row1 = revised_by_row("ADR-1", "(2)", "狭める", "「<b>前</b>」を狭く読む");
    let row2 = revised_by_row("ADR-1", "(1)", "広げる", "範囲を広げる");
    let block = format!("<h3>ほかの判断の記録による改訂</h3>\n<ul>\n{row1}\n{row2}\n</ul>\n");
    assert_eq!(ch5.matches(&block).count(), 1, "章 05 に逆向きの一覧が逐語で無い: {ch5}");
    assert!(
        ch5.find(&block).unwrap() < ch5.find("<h3>この判断で変わること</h3>").unwrap(),
        "逆向きの一覧が帰結の後"
    );
    assert!(!html.contains("相手の違う行"), "相手の違う行が出た");
    assert!(!html.contains("<b>前</b>"), "summary の山括弧が生のまま");
    // 札の 1 行と h3 の一覧を除くと、改訂されていない面と byte で同じ
    let stripped = cut_line(&html, &revised_by_badge(2)).replacen(&block, "", 1);
    assert!(stripped == base, "札と一覧のほかが改訂されていない面と違う");
}

#[test]
fn f148_only_an_accepted_reviser_is_read() {
    let base = fixture_html("f148-status-base");
    for status in ["accepted", "proposed", "retired"] {
        let case = format!("f148-status-{status}");
        let (run, html) = revised_face(&case, &reviser(status, F148_ROWS));
        assert_eq!(code(&run, &case), 0, "{case}: {}", stderr(&run));
        if status == "accepted" {
            once_in(&html, &revised_by_badge(2), &format!("{case} の逆向きの札"));
        } else {
            assert!(html == base, "{case}: 発効でない改訂する側を読んだ");
        }
    }
}

/// 写しの ADR-1 が読めないなら ADR-2 の面は 2（まだ分からない）で、面を書かない。
fn reviser_unknown(case: &str, adr1: &str, wording: &str) {
    let (run, html) = revised_face(case, adr1);
    assert_eq!(code(&run, "folio face"), 2, "{case}: {}", stderr(&run));
    let err = stderr(&run);
    for want in ["まだ分からない", "adr/ADR-1.yaml", wording] {
        assert!(err.contains(want), "{case}: 「{want}」が無い: {err}");
    }
    assert!(html.is_empty(), "{case}: 導出できないのに面を書いた");
}

#[test]
fn f148_unknown_when_a_reviser_row_cannot_be_read() {
    reviser_unknown(
        "f148-kind",
        &reviser(
            "accepted",
            "  - {target: ADR-2, decision: (2), kind: shrink, summary: 向きが表の外}\n",
        ),
        "改訂の向き の表に無い値「shrink」",
    );
    reviser_unknown(
        "f148-not-list",
        "id: ADR-1\nstatus: accepted\nrevises: 一覧でない字\n",
        "adr/ADR-1.yaml.revises: 一覧でない",
    );
    reviser_unknown(
        "f148-target",
        &reviser(
            "accepted",
            "  - {target: adr two, decision: (2), kind: narrow, summary: 相手が id の形でない}\n",
        ),
        "adr/ADR-1.yaml.revises[0].target",
    );
    reviser_unknown(
        "f148-no-status",
        "id: ADR-1\nrevises:\n  - {target: ADR-2, decision: (2), kind: narrow, summary: 状態が無い}\n",
        "欄 status が無い",
    );
    reviser_unknown(
        "f148-unreadable",
        "id: ADR-1\nstatus: [accepted\n",
        "adr/ADR-1.yaml: 読めない",
    );
}

#[test]
fn f148_real_sources_draw_every_revised_by_row() {
    let td = temp_dir("f148-census");
    let mut drawn = 0;
    for id in real_ids() {
        let (_, html) = real_face(&td, &id);
        let rows = real_revised_by(&id);
        let has = usize::from(!rows.is_empty());
        assert_eq!(
            html.matches("<span class=\"k\">ほかの判断の記録による改訂</span>").count(),
            has,
            "{id}: 逆向きの札の有無"
        );
        if has == 1 {
            once_in(&html, &revised_by_badge(rows.len()), &format!("{id} の逆向きの札"));
        }
        let ch5 = chapter5(&html);
        assert_eq!(
            html.matches("<h3>ほかの判断の記録による改訂</h3>").count(),
            has,
            "{id}: h3 の有無"
        );
        assert_eq!(
            ch5.matches("<h3>ほかの判断の記録による改訂</h3>").count(),
            has,
            "{id}: 章 05 の h3 の有無"
        );
        let mut at = 0;
        for (by, decision, kind, summary) in &rows {
            let row = revised_by_row(by, decision, kind, summary);
            assert_eq!(ch5.matches(&row).count(), 1, "{id}: 逆向きの行が逐語で 1 回ない: {row}");
            let pos = ch5.find(&row).unwrap();
            assert!(pos >= at, "{id}: 逆向きの行が改訂する側の順でない: {row}");
            at = pos + row.len();
            drawn += 1;
        }
    }
    let _ = fs::remove_dir_all(&td);
    assert!(drawn > 0, "実の正本の逆向きの行を 1 つも数えていない");
}

#[test]
fn f148_adr18_and_adr16_link_back_to_their_revisers() {
    let td = temp_dir("f148-real");
    let (_, adr18) = real_face(&td, "ADR-18");
    let (_, adr16) = real_face(&td, "ADR-16");
    let _ = fs::remove_dir_all(&td);
    assert!(
        chapter5(&adr18).contains(
            "<li><a class=\"xref\" href=\"adr-24.html\">ADR-24</a> がこの判断の決定 (5) を狭める:"
        ),
        "ADR-18 の面に ADR-24 への行が無い"
    );
    for by in ["ADR-21", "ADR-22"] {
        assert!(
            chapter5(&adr16).contains(&format!(
                "<li><a class=\"xref\" href=\"{}.html\">{by}</a> がこの判断の決定 ",
                by.to_ascii_lowercase()
            )),
            "ADR-16 の面に {by} への行が無い"
        );
    }
}
