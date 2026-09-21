//! 判断の記録の面（`folio face --face adr --id ADR-n`）の歯（便 25・docs/design/delivery-25.md §1 (g)）。binary 経由。
//! - 凍結 fixture（tests/fixtures/face/expected-adr.html）との byte 一致（P-10.1）・escape
//! - 実の正本の全本で `folio parts --check` に合格（AC14 の緑・NFR2）と逐語と件数の census（yaml-rust2 で正本を直に読む）
//! - check の 3 値・id の口 4 形・名札の表の外 2 つ・基の型・未解決の根拠の印・accepted と retired の状態・
//!   枝番付きの条 id・AC14 の赤の fixture・mode
//! - 図の章（便 33・FR15）: 写し（ADR-2・図 1 枚）の figure-panel と型の名札と根拠のリンクと cover-meta・図なしの面は
//!   図の章の外が byte で同じ・通らない図で 2 と前の面の保持・型外・道具の不在・実の正本の figure-panel の数
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
        let figures = a["figures"].as_vec().map_or(0, Vec::len);
        let fig_refs: usize = a["figures"].as_vec().map_or(0, |v| {
            v.iter()
                .map(|f| f["refs"].as_vec().map_or(0, Vec::len))
                .sum()
        });
        assert_eq!(
            count("class=\"xref\""),
            basis.len() + fig_refs,
            "{id}: 根拠のリンクの数（basis + 図の refs）"
        );
        assert!(
            !html.contains("（まだ分からない）"),
            "{id}: 行き先の無い根拠が在る"
        );
        // 根拠の群の一覧（li の数 = 根拠の数・li は行き先のリンクと題の span）
        assert_eq!(
            count("<li><a class=\"xref\""),
            basis.len(),
            "{id}: ul.basis の li の数"
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
