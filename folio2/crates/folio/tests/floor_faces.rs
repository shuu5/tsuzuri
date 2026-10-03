//! 床が面と同じ関数で面を組む歯（便 187・docs/design/delivery-187.md §1 (c)・FR5・FR4・FR9）。binary 経由。
//! 床（folio check）が合格なのに面の生成（folio build）だけが止まっていた所を、床が build と同じ `build_all` を図の道具を
//! 撃たずに回して、面の字のまま違反 1 件に数えることを測る。写しの土台は凍結した床の土台（tests/fixtures/floor_base/）に、
//! 提案中の判断の記録 1 本（ADR-4 の字から id と状態だけ替える・封の外）と発効した設計ノート 1 本を歯の中で足し、
//! 一時 dir の根で git の init と 1 commit をした置き場（素の床 = 合格 0 / 0）。字を当てるのは作業ツリーだけ。
//! 期待の字（面の字）は歯の側の手書き（P-10.1）。版管理の下の正本と面は書き換えない（写しと配信先は必ず一時 dir の中）。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../folio2")
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-floor-faces-{case}-{}", std::process::id()));
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

/// git を呼ぶ。環境変数 GIT_* は継承しない（tests/init.rs と同じ形）。
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
    Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(args)
        .output()
        .expect("folio を起動できない")
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// 提案中の判断の記録（床の土台の ADR-4 の字から id と状態だけ替え、承認欄を外す）。
fn proposed_adr(dir: &Path) {
    let t = fs::read_to_string(dir.join("adr/ADR-4.yaml")).unwrap();
    let mut out = String::new();
    for line in t.split_inclusive('\n') {
        match line {
            "id: ADR-4\n" => out.push_str("id: ADR-11\n"),
            "status: accepted\n" => out.push_str("status: proposed\n"),
            l if l.starts_with("approval: ") => {}
            l => out.push_str(l),
        }
    }
    assert!(out.contains("id: ADR-11\n") && out.contains("status: proposed\n"));
    fs::write(dir.join("adr/ADR-11.yaml"), out).unwrap();
}

/// 発効した設計ノート（承認欄 1 行・散文の節と判断の表の節）。
const DECIDE: &str = "meta:\n  id: decide\n  title: 判断のノート\n  version: v0.1\n  status: effective\n  generated: 2026-09-28\n  profile: design-note\n  approval:\n    - {who: 持ち主, date: 2026-09-28, ruling: f2-648 notes 2026-09-28 10:29 JST, verbatim: 承認する, surface: R-8}\nsections:\n  - n: 1\n    type: prose\n    title: 目的\n    body: 判断の見本。\n  - n: 2\n    type: decision-table\n    title: 判断\n    rows:\n      - {id: d1, text: 行を足す, ruling: t3-hub.57.1:20260927T2357Z-1}\n";

/// 写しの置き場（根・design-intent）。素の床が合格 0 / 0 であることを確かめてから返す。
fn place(case: &str) -> (PathBuf, PathBuf) {
    let td = temp_dir(case);
    let dir = td.join("design-intent");
    copy_dir(&repo_root().join("tests/fixtures/floor_base/design-intent"), &dir);
    fs::create_dir_all(td.join("contracts/field-schema")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/field-schema/schema.toml"),
    )
    .unwrap();
    proposed_adr(&dir);
    fs::write(dir.join("design-note/decide.yaml"), DECIDE).unwrap();
    git(&td, &["init", "-q"]);
    git(&td, &["add", "-A"]);
    git(&td, &["commit", "-q", "-m", "base"]);
    (td, dir)
}

fn check(dir: &Path) -> Output {
    folio(&["check", "--dir", dir.to_str().unwrap()])
}

/// file の字をちょうど 1 か所当てる（当て先がちょうど 1 つでなければ落とす）。
fn edit(dir: &Path, file: &str, from: &str, to: &str) {
    let p = dir.join(file);
    let t = fs::read_to_string(&p).unwrap();
    assert_eq!(t.matches(from).count(), 1, "{file}: 当て先「{from}」が 1 つでない");
    fs::write(&p, t.replacen(from, to, 1)).unwrap();
}

/// `anchor` の後で最初の `from` を `to` に替える（`anchor` はちょうど 1 つ）。
fn edit_after(dir: &Path, file: &str, anchor: &str, from: &str, to: &str) {
    let p = dir.join(file);
    let t = fs::read_to_string(&p).unwrap();
    assert_eq!(t.matches(anchor).count(), 1, "{file}: 目印「{anchor}」が 1 つでない");
    let at = t.find(anchor).unwrap();
    let rel = t[at..].find(from).expect("目印の後に当て先が無い");
    let mut out = t.clone();
    out.replace_range(at + rel..at + rel + from.len(), to);
    fs::write(&p, out).unwrap();
}

/// 一覧の承認欄（`  approval:` と続く `    - ` の行）を表 1 つの行に替える。
fn approval_as_one_table(dir: &Path, file: &str, table: &str) {
    let p = dir.join(file);
    let t = fs::read_to_string(&p).unwrap();
    let lines: Vec<&str> = t.split('\n').collect();
    let i = lines.iter().position(|l| *l == "  approval:").expect("承認欄が無い");
    let mut j = i + 1;
    while lines[j].starts_with("    - ") {
        j += 1;
    }
    assert!(j > i + 1, "{file}: 承認欄の行が無い");
    let mut out: Vec<&str> = lines[..i].to_vec();
    out.push(table);
    out.extend_from_slice(&lines[j..]);
    fs::write(&p, out.join("\n")).unwrap();
}

/// 表の 1 行: 割れの写し（床の土台に当てる字）と、面が言う字（歯の側の手書き）。
struct Row {
    id: &'static str,
    apply: fn(&Path),
    face: &'static str,
    said: &'static str,
}

const TOOL: &str = "  - {id: folio-v2, name: folio v2, role: 道具}\n";
const OUT3: &str = "  - {id: verdict, name: 検査の結果（3 値）, from: FR5}\n";
const RAIL7: &str = "  - {n: 7, who: folio, what: 組み立てて、見せる, reqs: [FR7]}\n";

const SRS_ROWS: &[Row] = &[
    Row {
        id: "tool-0",
        apply: |d| edit(d, "srs.yaml", TOOL, "  - {id: folio-v2, name: folio v2, role: 作る}\n"),
        face: "srs",
        said: "srs.yaml.actors: role が「道具」の actor が 0 で 1 つでない",
    },
    Row {
        id: "tool-2",
        apply: |d| {
            edit(
                d,
                "srs.yaml",
                TOOL,
                "  - {id: folio-v2, name: folio v2, role: 道具}\n  - {id: folio-v3, name: folio v3, role: 道具}\n",
            )
        },
        face: "srs",
        said: "srs.yaml.actors: role が「道具」の actor が 2 で 1 つでない",
    },
    Row {
        id: "band-in-5",
        apply: |d| {
            edit(
                d,
                "srs.yaml",
                TOOL,
                "  - {id: folio-v2, name: folio v2, role: 道具}\n  - {id: reader, name: 読み手, role: 読む}\n",
            )
        },
        face: "srs",
        said: "srs.yaml.actors: 入れる側の帯が 5 で上限 4（部品目録の context-band の max_per_band）を超える",
    },
    Row {
        id: "band-out-5",
        apply: |d| {
            edit(
                d,
                "srs.yaml",
                OUT3,
                "  - {id: verdict, name: 検査の結果（3 値）, from: FR5}\n  - {id: out-4, name: 4 つ目, from: FR5}\n  - {id: out-5, name: 5 つ目, from: FR5}\n",
            )
        },
        face: "srs",
        said: "srs.yaml.outputs: 出る側の帯が 5 で上限 4（部品目録の context-band の max_per_band）を超える",
    },
    Row {
        id: "out-from-ac",
        apply: |d| {
            edit(
                d,
                "srs.yaml",
                OUT3,
                "  - {id: verdict, name: 検査の結果（3 値）, from: AC3}\n",
            )
        },
        face: "srs",
        said: "srs.yaml.outputs[2].from: 要件 id「AC3」が無い",
    },
    Row {
        id: "rail-8",
        apply: |d| {
            edit(
                d,
                "srs.yaml",
                RAIL7,
                "  - {n: 7, who: folio, what: 組み立てて、見せる, reqs: [FR7]}\n  - {n: 8, who: folio, what: 余分の段, reqs: [FR7]}\n",
            )
        },
        face: "srs",
        said: "srs.yaml.rail: 段が 8 で上限 7（部品目録の pipeline-rail の max_nodes）を超える",
    },
    Row {
        id: "verdicts-5",
        apply: |d| {
            edit(
                d,
                "srs.yaml",
                "\nrequirements:\n",
                "  - {id: v4, name: 4 つ目, tone: ok, cond: 余分。}\n  - {id: v5, name: 5 つ目, tone: ok, cond: 余分。}\n\nrequirements:\n",
            )
        },
        face: "srs",
        said: "srs.yaml.verdicts: 答えが 5 で上限 4（部品目録の state-strip の max_nodes）を超える",
    },
    Row {
        id: "basis-adr",
        apply: |d| {
            edit_after(
                d,
                "srs.yaml",
                "    title: 相談窓口（intake）\n",
                "    basis: [P-1]\n",
                "    basis: [ADR-1]\n",
            )
        },
        face: "srs",
        said: "srs.yaml.requirements[0].basis[0]: 条 id「ADR-1」が無い",
    },
    Row {
        id: "goal-text",
        apply: |d| {
            edit(
                d,
                "srs.yaml",
                "  - {id: GOAL1, title: 相談して決められる, text: 「どの文書が要るか分からない」を無くす。最初と途中の両方に窓口がある。}\n",
                "  - {id: GOAL1, title: 相談して決められる}\n",
            )
        },
        face: "srs",
        said: "srs.yaml.goals[0]: 欄 text が無い",
    },
    Row {
        id: "srs-approval-table",
        apply: |d| {
            approval_as_one_table(
                d,
                "srs.yaml",
                "  approval: {role: 作成, who: 書き手, when: 2026-09-12, stamp: 起草}",
            )
        },
        face: "srs",
        said: "srs.yaml.meta.approval: 一覧でない",
    },
];

const OTHER_ROWS: &[Row] = &[
    Row {
        id: "index-approval-table",
        apply: |d| {
            approval_as_one_table(
                d,
                "index.yaml",
                "  approval: {role: 作成, who: 書き手, when: 2026-09-17, stamp: 起草}",
            )
        },
        face: "index",
        said: "index.yaml.meta.approval: 一覧でない",
    },
    Row {
        id: "amend-steps-8",
        apply: |d| {
            edit(
                d,
                "constitution.yaml",
                "  effective_step: {n: 0,",
                "    - {n: 6, who: 機械, what: 余分 6, article: [A-2]}\n    - {n: 7, who: 機械, what: 余分 7, article: [A-2]}\n    - {n: 8, who: 機械, what: 余分 8, article: [A-2]}\n  effective_step: {n: 0,",
            )
        },
        face: "constitution",
        said: "constitution.yaml.amendment: 改訂の段が 8 で上限 7（部品目録の pipeline-rail の max_nodes）を超える",
    },
    Row {
        id: "amend-no-effective-step",
        apply: |d| {
            let p = d.join("constitution.yaml");
            let t = fs::read_to_string(&p).unwrap();
            let line = t
                .split_inclusive('\n')
                .find(|l| l.starts_with("  effective_step: {n: 0,"))
                .expect("改訂の欄 effective_step が無い")
                .to_string();
            fs::write(&p, t.replacen(&line, "", 1)).unwrap();
        },
        face: "constitution",
        said: "constitution.yaml.amendment: 欄 effective_step が無い",
    },
    Row {
        id: "article-no-mechanism",
        apply: |d| {
            let p = d.join("constitution.yaml");
            let t = fs::read_to_string(&p).unwrap();
            let at = t.find("  - id: P-1\n").expect("条 P-1 が無い");
            let rel = t[at..].find("    mechanism: {").expect("条 P-1 に mechanism が無い");
            let end = at + rel + t[at + rel..].find('\n').unwrap() + 1;
            let mut out = t.clone();
            out.replace_range(at + rel..end, "");
            fs::write(&p, out).unwrap();
        },
        face: "constitution",
        said: "constitution.yaml.articles[0]: 欄 mechanism が無い",
    },
    Row {
        id: "adr-consequence-map",
        apply: |d| edit(d, "adr/ADR-11.yaml", "  - 語彙に語を足す（", "  - 語彙に語を足す, id: （"),
        face: "adr:ADR-11",
        said: "adr/ADR-11.yaml.consequences[1]: 文字列でない",
    },
    Row {
        id: "note-approval-list",
        apply: |d| {
            edit(
                d,
                "design-note/decide.yaml",
                "{who: 持ち主, date: 2026-09-28,",
                "{who: [持ち主], date: 2026-09-28,",
            )
        },
        face: "note:decide",
        said: "design-note/decide.yaml.meta.approval[0].who: 文字列でない",
    },
    // 形の誤り（重複キー）は読めないでない＝面だけが読む file でも違反 [面]（床のほかの段の重複キーと同じ・便 187 改訂 a）
    Row {
        id: "intake-duplicate-key",
        apply: |d| {
            let p = d.join("intake-sheet.yaml");
            fs::write(&p, format!("{}x: 1\nx: 2\n", fs::read_to_string(&p).unwrap())).unwrap();
        },
        face: "index",
        said: "intake-sheet.yaml: 重複キー「x」（52 行）",
    },
];

/// 1 行を撃つ: 床が不合格（違反 1 件・[面] の字が面の字）・build は何も書かず 1・面の口そのものも同じ字で止まる。
fn assert_row(row: &Row) {
    let (td, dir) = place(row.id);
    let before = text(&check(&dir));
    assert!(
        before.contains("folio check: 合格（違反 0・まだ分からない 0）"),
        "{}: 写しの土台の床が合格でない: {before}",
        row.id
    );
    (row.apply)(&dir);
    let out = check(&dir);
    let told = text(&out);
    assert_eq!(out.status.code(), Some(1), "{}: {told}", row.id);
    assert!(
        told.contains(&format!("[面] {}\n", row.said)),
        "{}: 床が面の字を名指さない: {told}",
        row.id
    );
    assert!(
        told.contains("folio check: 不合格（違反 1・まだ分からない 0）"),
        "{}: {told}",
        row.id
    );
    let site = td.join("site");
    let build = folio(&[
        "build",
        "--dir",
        dir.to_str().unwrap(),
        "--out",
        site.to_str().unwrap(),
        "--write",
    ]);
    let said = text(&build);
    assert_eq!(build.status.code(), Some(1), "{}: {said}", row.id);
    assert!(
        said.contains("folio build: 床 = 不合格（違反 1・まだ分からない 0）・書かない"),
        "{}: {said}",
        row.id
    );
    assert!(!site.exists(), "{}: 床が不合格なのに配信先を作った", row.id);
    // 面の口（床を回さない）も同じ字で止まる＝床の字は面の字そのもの
    let mut args = vec!["face".to_string()];
    match row.face.split_once(':') {
        Some((face, id)) => args.extend(["--face".into(), face.into(), "--id".into(), id.into()]),
        None => args.extend(["--face".into(), row.face.into()]),
    }
    let page = td.join("page.html");
    args.extend([
        "--dir".into(),
        dir.to_str().unwrap().into(),
        "--out".into(),
        page.to_str().unwrap().into(),
        "--write".into(),
    ]);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let face = folio(&args);
    let face_said = text(&face);
    assert_eq!(face.status.code(), Some(2), "{}: {face_said}", row.id);
    assert!(face_said.contains(row.said), "{}: 面の字が違う: {face_said}", row.id);
    let _ = fs::remove_dir_all(&td);
}

#[test]
fn f187_srs_splits_fail_the_floor_with_the_face_words() {
    for row in SRS_ROWS {
        assert_row(row);
    }
}

#[test]
fn f187_other_splits_fail_the_floor_with_the_face_words() {
    for row in OTHER_ROWS {
        assert_row(row);
    }
}

/// 床がほかに何かを数えているときは面を組まない（同じ原因を 2 度数えない・床が決めきれない所を決めつけない）。
/// 違反が在る写しは違反の数が 1 のまま・版管理の無い写し（測れない）は まだ分からない のまま、どちらも [面] を出さない。
/// 対の緑: 同じ面の欠けだけの写しは床が [面] で落とす（base では床が合格＝RED）。
#[test]
fn f187_the_face_stage_runs_only_on_an_otherwise_silent_floor() {
    let face_gap = |d: &Path| edit(d, "srs.yaml", TOOL, "  - {id: folio-v2, name: folio v2, role: 作る}\n");
    // 面の欠けだけ → [面]
    let (td, dir) = place("silent-only");
    face_gap(&dir);
    let told = text(&check(&dir));
    assert!(told.contains("[面] "), "{told}");
    let _ = fs::remove_dir_all(&td);
    // ほかの違反 1 件（最上位の閉じた一覧に無い節）+ 面の欠け → 違反 1 件のまま・[面] なし
    let (td, dir) = place("with-violation");
    face_gap(&dir);
    let p = dir.join("srs.yaml");
    fs::write(&p, format!("{}extra_top: 1\n", fs::read_to_string(&p).unwrap())).unwrap();
    let told = text(&check(&dir));
    assert!(
        told.contains("folio check: 不合格（違反 1・まだ分からない 0）") && !told.contains("[面] "),
        "{told}"
    );
    let _ = fs::remove_dir_all(&td);
    // 版管理の無い写し（測れない 1 件）+ 面の欠け → まだ分からない のまま・[面] なし
    let td = temp_dir("no-git");
    let dir = td.join("design-intent");
    copy_dir(&repo_root().join("tests/fixtures/floor_base/design-intent"), &dir);
    fs::create_dir_all(td.join("contracts/field-schema")).unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/field-schema/schema.toml"),
    )
    .unwrap();
    face_gap(&dir);
    let out = check(&dir);
    let told = text(&out);
    assert_eq!(out.status.code(), Some(2), "{told}");
    assert!(!told.contains("[面] ") && told.contains("違反 0・"), "{told}");
    let _ = fs::remove_dir_all(&td);
}

/// 床の段の順（便 187 改訂 a）: 面の段は索引の段の後・凍結の後始末の前。索引が数えた置き場では面を組まない（check も
/// build --write も違反 1 件のまま・[面] なし）。面が組めない置き場では --freeze-adrs が封を足さない（対の緑: 面の欠けを
/// 戻すと同じ口が封を足す＝歯の写しが凍結の道を通る）。
#[test]
fn f187_the_face_stage_runs_after_the_index_and_before_the_freeze() {
    let face_gap = |d: &Path| edit(d, "srs.yaml", TOOL, "  - {id: folio-v2, name: folio v2, role: 作る}\n");
    let face_back = |d: &Path| edit(d, "srs.yaml", "  - {id: folio-v2, name: folio v2, role: 作る}\n", TOOL);
    // 索引の節点の違反（FR1 の id を一重の引用符で）+ 面の欠け
    let (td, dir) = place("after-index");
    face_gap(&dir);
    edit(&dir, "srs.yaml", "  - id: FR1\n", "  - id: 'FR1'\n");
    let told = text(&check(&dir));
    assert!(
        told.contains("[索引の節点] srs.yaml: 索引の節点 FR1 ")
            && told.contains("folio check: 不合格（違反 1・まだ分からない 0）")
            && !told.contains("[面] "),
        "{told}"
    );
    let site = td.join("site");
    let build = folio(&[
        "build",
        "--dir",
        dir.to_str().unwrap(),
        "--out",
        site.to_str().unwrap(),
        "--write",
    ]);
    let said = text(&build);
    assert!(
        said.contains("folio build: 床 = 不合格（違反 1・まだ分からない 0）・書かない"),
        "{said}"
    );
    let _ = fs::remove_dir_all(&td);
    // 発効して封の無い判断の記録（ADR-4 の字から id だけ替える）+ 面の欠け → 封を足さない
    let (td, dir) = place("before-freeze");
    let t = fs::read_to_string(dir.join("adr/ADR-4.yaml")).unwrap();
    fs::write(dir.join("adr/ADR-12.yaml"), t.replacen("id: ADR-4\n", "id: ADR-12\n", 1)).unwrap();
    git(&td, &["add", "-A"]);
    git(&td, &["commit", "-q", "-m", "ADR-12"]);
    face_gap(&dir);
    let seals = dir.join("anchors/adr-seals.yaml");
    let before = fs::read(&seals).unwrap();
    let freeze = |dir: &Path| folio(&["check", "--freeze-adrs", "--dir", dir.to_str().unwrap()]);
    let out = freeze(&dir);
    let told = text(&out);
    assert_eq!(out.status.code(), Some(1), "{told}");
    assert!(
        told.contains("[面] srs.yaml.actors: role が「道具」の actor が 0 で 1 つでない") && !told.contains("封を足した"),
        "{told}"
    );
    assert_eq!(fs::read(&seals).unwrap(), before, "面が組めないのに封を足した");
    face_back(&dir);
    let out = freeze(&dir);
    let told = text(&out);
    assert_eq!(out.status.code(), Some(0), "{told}");
    assert!(told.contains("封を足した: ") && told.contains("足した行 ADR-12"), "{told}");
    let _ = fs::remove_dir_all(&td);
}

/// 面だけが読む file を壊す行（case の名・写しを壊す手・床が言う字）。
type Case = (&'static str, fn(&Path), &'static str);

/// 面だけが読む file が読めない（相談窓口の支度表が YAML として読めない・dir・UTF-8 でない・型付きの木に読めない、様式 2 本が
/// dir、天井の印が dir・YAML として読めない）は、床のほかの段と同じく まだ分からない 1 件（違反 [面] にしない・P-4.2）。
/// build --write も まだ分からない で何も書かない。
#[test]
fn f187_unreadable_face_files_stay_unknown() {
    let rows: [Case; 8] = [
        (
            "intake-syntax",
            |d| {
                let p = d.join("intake-sheet.yaml");
                fs::write(&p, format!("{}k: [\n", fs::read_to_string(&p).unwrap())).unwrap();
            },
            "# まだ分からない: intake-sheet.yaml: 読めない: ",
        ),
        (
            "intake-dir",
            |d| {
                fs::remove_file(d.join("intake-sheet.yaml")).unwrap();
                fs::create_dir_all(d.join("intake-sheet.yaml")).unwrap();
            },
            "# まだ分からない: intake-sheet.yaml: 読めない: ",
        ),
        (
            "intake-utf8",
            |d| {
                let p = d.join("intake-sheet.yaml");
                let mut bytes = fs::read(&p).unwrap();
                bytes.extend_from_slice(b"k: \xff\n");
                fs::write(&p, bytes).unwrap();
            },
            "# まだ分からない: intake-sheet.yaml: UTF-8 でない",
        ),
        (
            "intake-scalar",
            |d| {
                let p = d.join("intake-sheet.yaml");
                fs::write(&p, format!("{}zz: 0o17\n", fs::read_to_string(&p).unwrap())).unwrap();
            },
            "# まだ分からない: intake-sheet.yaml: 正規化できない scalar「0o17」",
        ),
        (
            "style-dir",
            |d| fs::create_dir_all(d.join("preview/folio.css")).unwrap(),
            "preview/folio.css: 読めない: ",
        ),
        (
            "ui-dir",
            |d| fs::create_dir_all(d.join("preview/folio-ui.js")).unwrap(),
            "preview/folio-ui.js: 読めない: ",
        ),
        (
            "stamp-dir",
            |d| fs::create_dir_all(d.join("preview/ceiling-stamp.yaml")).unwrap(),
            "# まだ分からない: preview/ceiling-stamp.yaml: 読めない: ",
        ),
        (
            "stamp-syntax",
            |d| {
                fs::create_dir_all(d.join("preview")).unwrap();
                fs::write(d.join("preview/ceiling-stamp.yaml"), "at: [\n").unwrap();
            },
            "# まだ分からない: preview/ceiling-stamp.yaml: parse できない: ",
        ),
    ];
    check_unreadable(rows);
}

/// 行ごとに写しを壊し、床が まだ分からない 1 件で build --write も何も書かないことを見る。
fn check_unreadable(rows: [Case; 8]) {
    for (case, apply, said) in rows {
        let (td, dir) = place(case);
        apply(&dir);
        let out = check(&dir);
        let told = text(&out);
        assert_eq!(out.status.code(), Some(2), "{case}: {told}");
        assert!(
            told.contains(said)
                && told.contains("folio check: まだ分からない（違反 0・まだ分からない 1）")
                && !told.contains("[面] "),
            "{case}: {told}"
        );
        let site = td.join("site");
        let build = folio(&[
            "build",
            "--dir",
            dir.to_str().unwrap(),
            "--out",
            site.to_str().unwrap(),
            "--write",
        ]);
        let said = text(&build);
        assert_eq!(build.status.code(), Some(2), "{case}: {said}");
        assert!(
            said.contains("folio build: 床 = まだ分からない（違反 0・まだ分からない 1）"),
            "{case}: {said}"
        );
        assert!(!site.exists(), "{case}: 床が まだ分からない なのに配信先を作った");
        let _ = fs::remove_dir_all(&td);
    }
}

/// 床の面の段は図の道具を撃たない: git だけを置いた PATH（Node が無い）でも、図を持つ写しの床は合格のまま（道具の
/// 答えは build が まだ分からない で知らせる・床は Node に依らない）。対の RED: 同じ PATH で面の欠けは床が [面] で落とす。
#[test]
fn f187_the_floor_does_not_run_the_figure_tool() {
    let (td, dir) = place("no-node");
    assert!(
        fs::read_to_string(dir.join("srs.yaml")).unwrap().contains("\nfigures:\n"),
        "床の土台の要件書に図が無い"
    );
    let empty = td.join("git-only-path");
    fs::create_dir_all(&empty).unwrap();
    let path = std::env::var_os("PATH").expect("PATH が無い");
    let real_git = std::env::split_paths(&path)
        .map(|d| d.join("git"))
        .find(|g| g.is_file())
        .expect("PATH に git が無い");
    std::os::unix::fs::symlink(&real_git, empty.join("git")).unwrap();
    assert!(!empty.join("node").exists());
    let run = |dir: &Path| {
        Command::new(env!("CARGO_BIN_EXE_tz"))
            .env("PATH", &empty)
            .args(["check", "--dir", dir.to_str().unwrap()])
            .output()
            .expect("folio を起動できない")
    };
    let out = run(&dir);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    edit(&dir, "srs.yaml", RAIL7, "  - {n: 7, who: folio, what: 組み立てて、見せる, reqs: [FR7]}\n  - {n: 8, who: folio, what: 余分の段, reqs: [FR7]}\n");
    let out = run(&dir);
    let told = text(&out);
    assert_eq!(out.status.code(), Some(1), "{told}");
    assert!(told.contains("[面] srs.yaml.rail: 段が 8 で上限 7"), "{told}");
    let _ = fs::remove_dir_all(&td);
}

/// 契約表の行の verify と done は器の導出 file で conditional（在るときだけ出す）: 2 欄の無い行でも床は合格のまま、
/// 設計ノートの面が書ける（base では面が「欄 done が無い」で止まり、床の面の段も同じ字で落とす）。
#[test]
fn f187_contract_rows_without_verify_and_done_build() {
    let (td, dir) = place("conditional");
    let p = dir.join("design-note/example.yaml");
    let t = fs::read_to_string(&p).unwrap();
    let row = t
        .lines()
        .find(|l| l.starts_with("      - {id: a, title: 図の生成の口 3 つを 1 便で置く,"))
        .expect("契約表の行 a が無い")
        .to_string();
    let verify = row.find(", verify: [").unwrap();
    let size = row.find(", size: M").unwrap();
    let done = row.find(", done: ").unwrap();
    let bare = format!("{}{}}}", &row[..verify], &row[size..done]);
    assert!(!bare.contains("verify") && !bare.contains("done"), "{bare}");
    fs::write(&p, t.replacen(&row, &bare, 1)).unwrap();
    let told = text(&check(&dir));
    assert!(told.contains("folio check: 合格（違反 0・まだ分からない 0）"), "{told}");
    let page = td.join("note.html");
    let out = folio(&[
        "face",
        "--face",
        "note",
        "--id",
        "example",
        "--dir",
        dir.to_str().unwrap(),
        "--out",
        page.to_str().unwrap(),
        "--write",
    ]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let html = fs::read_to_string(&page).unwrap();
    let at = html.find("図の生成の口 3 つを 1 便で置く").expect("行 a の題が面に無い");
    let article = &html[at..at + html[at..].find("</article>").unwrap()];
    assert!(!article.contains("class=\"norm\""), "done の段落が出た: {article}");
    assert!(!article.contains(">検証<"), "検証の札が出た: {article}");
    let _ = fs::remove_dir_all(&td);
}
