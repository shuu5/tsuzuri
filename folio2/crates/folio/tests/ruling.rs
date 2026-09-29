//! 決定の欄の裁定 id の形の床の歯（便 181・docs/design/delivery-181.md §1 (c)・判断の記録 ADR-31 決定 (1)(3)・AC28 の前半）。
//! 土台は凍結した写し（tests/fixtures/floor_base/design-intent/）の写し全部を一時 dir に作り、字を 1 か所ずつ変えて素の
//! folio check を撃つ。版管理は作らない（土台の写しの床は器の導出 file と版管理の 2 つが まだ分からない）＝歯は種別 裁定 id の
//! 違反の行と、未記入 の まだ分からない の行を数える。数の 55 と内訳は独立の実装（起草の記録の fields.py）の数。
//! 便 182（docs/design/delivery-182.md §1 (c)）の歯 f182_ は、判断の記録の欄の決まりの生成区間の文法と一覧の写しを見る。
//! 便 204（docs/design/delivery-204.md §1 (c)）の歯 f204_ は、規則の表の行の裁定の時刻（ruled_at）の在ることと形と、閾値の行の
//! 種別（human-review を置けない）を見る。期待の字は歯の中の手書き。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";
const NO_ID: &str = "に台帳 id が無い（決定の欄・形は adr/schema.yaml の ruling_pattern）";
const MARK: &str = "が 未記入（骨格の印・裁定の前＝条 P-17.3）";

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

/// 土台の写しの一時 dir（歯の終わりに消す）。
struct Work(PathBuf);

impl Work {
    fn new(case: &str) -> Work {
        let root = std::env::temp_dir().join(format!("folio-ruling-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(FLOOR_BASE), &root);
        Work(root)
    }

    /// 素の check の、種別 裁定 id の違反の行（標準出力）と、まだ分からない の行（標準エラー）。
    fn check(&self) -> (Vec<String>, Vec<String>) {
        let out = Command::new(env!("CARGO_BIN_EXE_folio"))
            .args(["check", "--dir"])
            .arg(&self.0)
            .output()
            .expect("folio を起動できない");
        let pick = |b: Vec<u8>, p: &str| {
            let t = String::from_utf8(b).unwrap();
            t.lines().filter(|l| l.starts_with(p)).map(str::to_string).collect()
        };
        (pick(out.stdout, "[裁定 id] "), pick(out.stderr, "# まだ分からない: "))
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.0.join(rel)).unwrap()
    }

    /// `marker` を含むちょうど 1 行の、流れの形の欄 `key` の値を `value` に替える（None なら欄ごと外す）。
    fn set(&self, rel: &str, marker: &str, key: &str, value: Option<&str>) {
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
        let new = match value {
            Some(s) => format!("{}{key}: {s}{}", &line[..start], &line[end..]),
            None => format!("{}{}", &line[..start], line[end..].trim_start_matches(", ")),
        };
        fs::write(self.0.join(rel), text.replacen(line, &new, 1)).unwrap();
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const R10: (&str, &str, &str) = ("rules.yaml", "{id: R-10,", "ruling");

/// 歯 1（AC28 の前半）: 規則の表の 1 行の裁定の欄から台帳の id を消すと違反がちょうど 1 つ増え、骨格の印にすると違反は
/// 増えず まだ分からない がちょうど 1 つ増える。語頭でない台帳の id の形（folio2-648）は数えない。
#[test]
fn f181_a_rules_row_without_a_ledger_id_is_one_violation() {
    let base = Work::new("base").check();
    assert!(base.0.is_empty(), "{base:?}");
    let (file, marker, key) = R10;
    for (value, want) in [("G16=A（受入 (f)）", "G16=A（受入 (f)）"), ("G16=A・folio2-648", "G16=A・folio2-648")] {
        let w = Work::new("r10");
        w.set(file, marker, key, Some(value));
        let (v, p) = w.check();
        assert_eq!(v, [format!("[裁定 id] rules.yaml: 行 R-10 の ruling「{want}」{NO_ID}")]);
        assert_eq!(p, base.1);
    }
    let w = Work::new("r10-mark");
    w.set(file, marker, key, Some("未記入"));
    let (v, p) = w.check();
    assert!(v.is_empty(), "{v:?}");
    assert_eq!(p.len(), base.1.len() + 1, "{p:?}");
    assert!(p.contains(&format!("# まだ分からない: rules.yaml: 行 R-10 の ruling {MARK}")), "{p:?}");
}

/// 歯 2: 決定の欄の閉じた一覧の各種類（憲法の発効の承認と改訂来歴・閾値と開発規律の行・判断の記録の承認欄・5 正本の
/// 承認欄の行の stamp）の 1 つから台帳の id を消すと、その欄の違反がちょうど 1 つ出る。設計ノートは tests/note.rs の f161_。
#[test]
fn f181_each_kind_of_decision_field_is_read() {
    let cases = [
        ("constitution.yaml", "20:2x（発効承認）", "ruling", "constitution.yaml: meta.approval.ruling「発効承認」"),
        ("rules.yaml", "{id: D-8,", "ruling", "rules.yaml: 行 D-8 の ruling「発効承認」"),
        ("adr/ADR-2.yaml", "09-13 09:35,", "ruling", "adr/ADR-2.yaml: approval.ruling「発効承認」"),
        ("srs.yaml", "（f2-648.1 notes 20:2x）", "stamp", "srs.yaml: meta.approval[2].stamp「発効承認」"),
        ("index.yaml", "2026-09-18 00:2x JST", "stamp", "index.yaml: meta.approval[2].stamp「発効承認」"),
        ("ceiling.yaml", "2026-09-19 13:1x JST", "stamp", "ceiling.yaml: meta.approval[1].stamp「発効承認」"),
        ("intake.yaml", "2026-09-18 11:4x JST", "stamp", "intake.yaml: meta.approval[1].stamp「発効承認」"),
    ];
    for (file, marker, key, at) in cases {
        let w = Work::new("kind");
        w.set(file, marker, key, Some("発効承認"));
        assert_eq!(w.check().0, [format!("[裁定 id] {at}{NO_ID}")], "{file}");
    }
    // 改訂来歴と、床の 7 本の外の索引の欄の決まり（在れば読む）
    let w = Work::new("amended");
    let c = w.read("constitution.yaml");
    let at = "    relations: {reqs: [FR1, FR2, AC1]}\n";
    let add = "    amended_by: [{adr: ADR-2, date: 2026-09-13, ruling: 持ち主の裁定}]\n";
    fs::write(w.0.join("constitution.yaml"), c.replacen(at, &format!("{at}{add}"), 1)).unwrap();
    let head = "meta:\n  approval:\n    - {role: 作成, stamp: 起草}\n";
    fs::write(w.0.join("graph.yaml"), format!("{head}    - {{role: 承認, stamp: 発効}}\n")).unwrap();
    assert_eq!(
        w.check().0,
        [
            format!("[裁定 id] constitution.yaml: 条 P-1 の amended_by[0].ruling「持ち主の裁定」{NO_ID}"),
            format!("[裁定 id] graph.yaml: meta.approval[1].stamp「発効」{NO_ID}"),
        ]
    );
}

/// 歯 3: 役が 作成 と レビュー の行（土台の srs.yaml の「起草」と review/summary.md）と、憲法の条 P-2 の前の版との対応
/// （supersedes_v1 の「裁定 #4」）は数えない。役を 作成 から替えた行は数える。
#[test]
fn f181_rows_without_a_decision_are_not_read() {
    let w = Work::new("skip");
    assert!(w.read("constitution.yaml").contains("ruling: \"裁定 #4\""));
    w.set("srs.yaml", "stamp: review/summary.md", "stamp", Some("未記入"));
    assert!(w.check().0.is_empty());
    w.set("srs.yaml", "when: 2026-09-12, stamp: 起草}", "role", Some("確認"));
    assert_eq!(w.check().0, [format!("[裁定 id] srs.yaml: meta.approval[0].stamp「起草」{NO_ID}")]);
}

/// 歯 4: 骨格の印（未記入）が まだ分からない になるのは骨格が書く欄（憲法の発効の承認・規則の表の行）だけで、判断の記録と
/// 5 正本の承認欄の未記入は違反のまま。欄が無い・値が一覧は違反。
#[test]
fn f181_the_mark_waits_only_in_the_skeleton_fields() {
    let w = Work::new("mark");
    w.set("constitution.yaml", "20:2x（発効承認）", "ruling", Some("未記入"));
    w.set("rules.yaml", "{id: D-8,", "ruling", Some("未記入"));
    w.set("adr/ADR-2.yaml", "09-13 09:35,", "ruling", Some("未記入"));
    w.set("srs.yaml", "（f2-648.1 notes 20:2x）", "stamp", Some("未記入"));
    let (v, p) = w.check();
    assert_eq!(
        v,
        [
            format!("[裁定 id] adr/ADR-2.yaml: approval.ruling「未記入」{NO_ID}"),
            format!("[裁定 id] srs.yaml: meta.approval[2].stamp「未記入」{NO_ID}"),
        ]
    );
    for at in ["constitution.yaml: meta.approval.ruling", "rules.yaml: 行 D-8 の ruling"] {
        assert!(p.contains(&format!("# まだ分からない: {at} {MARK}")), "{at}: {p:?}");
    }
    let (file, marker, key) = R10;
    for (value, want) in [(Some("[f2-648.1]"), "が字でない（一覧か表）＝台帳 id を切り出せない"), (None, "が無い＝台帳 id が無い")] {
        let w = Work::new("shape");
        w.set(file, marker, key, value);
        assert_eq!(w.check().0, [format!("[裁定 id] rules.yaml: 行 R-10 の ruling {want}")]);
    }
}

/// 歯 5: 決定の欄を持つ 7 本の台帳の id（f2- と s2-）を全部大字にすると、種別 裁定 id の違反は独立の実装の数と同じ 55
/// （憲法 1・規則の表 27・判断の記録 10・要件書 9・入口 4・天井の正本 3・相談窓口 1）。
#[test]
fn f181_every_decision_field_of_the_base_is_counted() {
    let w = Work::new("upper");
    let files = ["constitution", "rules", "srs", "index", "ceiling", "intake"].map(|f| format!("{f}.yaml"));
    for f in files.into_iter().chain((1..=10).map(|n| format!("adr/ADR-{n}.yaml"))) {
        let t = w.read(&f).replace("f2-", "F2-").replace("s2-", "S2-");
        fs::write(w.0.join(&f), t).unwrap();
    }
    let v = w.check().0;
    let n = |p: &str| v.iter().filter(|l| l.starts_with(&format!("[裁定 id] {p}"))).count();
    let got = ["constitution", "rules", "adr/", "srs", "index", "ceiling", "intake"].map(n);
    assert_eq!((v.len(), got), (55, [1, 27, 10, 9, 4, 3, 1]), "{v:?}");
}

/// 便 182 の歯の手書きの字: 判断の記録の欄の決まりの生成区間の、形の種類・決定の欄・骨格の欄・数えない役の 4 欄（写しの字）。
const LISTS: &str = "  ruling_forms: [question, notes-time, bead]
  ruling_fields:
    - constitution.yaml meta.approval.ruling
    - constitution.yaml articles[].amended_by[].ruling
    - rules.yaml thresholds[].ruling
    - rules.yaml discipline[].ruling
    - adr/ADR-*.yaml approval.ruling
    - design-note/*.yaml meta.approval[].ruling
    - srs.yaml meta.approval[].stamp
    - index.yaml meta.approval[].stamp
    - ceiling.yaml meta.approval[].stamp
    - intake.yaml meta.approval[].stamp
    - graph.yaml meta.approval[].stamp
    - design-note/*.yaml sections[decision-table].rows[].ruling
  ruling_skeleton:
    - constitution.yaml meta.approval.ruling
    - rules.yaml thresholds[].ruling
    - rules.yaml discipline[].ruling
  ruling_skip_roles: [作成, レビュー]
";
const GRAMMAR: &str = r"(?<![0-9A-Za-z_.-])[a-z][0-9]-[0-9a-z]+(\.[0-9]+)*(:[0-9]{8}T[0-9]{4}Z-[0-9]+| notes( [0-9]{4}-[0-9]{2}-[0-9]{2}( [0-9]{2}:[0-9][0-9x])?| [0-9]{2}:[0-9][0-9x])( JST)?)?";

/// 歯 6（便 182）: folio2 の正本と床の土台の欄の決まりが、文法の字面と 4 欄を手書きの字のとおりに持つ。
#[test]
fn f182_the_region_holds_the_grammar_and_the_lists() {
    let real = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../design-intent/adr/schema.yaml")).unwrap();
    let copy = Work::new("region").read("adr/schema.yaml");
    for (text, pattern) in [(&real, format!("  ruling_pattern: {GRAMMAR}\n")), (&copy, format!("  ruling_pattern: '{GRAMMAR}'\n"))] {
        assert!(text.contains(&pattern) && text.contains(LISTS), "{pattern}");
    }
}

/// 歯 7（便 182）: 土台の写しの欄の決まりから 4 欄の 1 つを消すか、文法の字面を前の字（便 181 まで）に戻すと、写しの床が
/// その欄の食い違いを違反にする（欄の決まりは床の定数の写し）。
#[test]
fn f182_a_copy_that_drops_a_list_drifts() {
    for (from, to, key) in [
        ("  ruling_skip_roles: [作成, レビュー]\n", "", "schema.ruling_skip_roles（欠落）"),
        (GRAMMAR, r"[a-z]\d-[0-9a-z]+(\.\d+)?", "schema.ruling_pattern"),
    ] {
        let w = Work::new("drift");
        let t = w.read("adr/schema.yaml");
        assert_eq!(t.matches(from).count(), 1, "{from}");
        fs::write(w.0.join("adr/schema.yaml"), t.replacen(from, to, 1)).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_folio")).args(["check", "--dir"]).arg(&w.0).output().unwrap();
        let text = String::from_utf8(out.stdout).unwrap();
        let v: Vec<&str> = text.lines().filter(|l| l.starts_with('[')).collect();
        assert_eq!(v, [format!("[adr] adr/schema.yaml {key} が床の定数と違う（欄の決まりの閾値・値域・置き場は床の定数の写し＝data 側で動かせない・N-3.1）")], "{key}");
    }
}

/// 便 204 の歯の手書きの字: 裁定の時刻の違反の末尾。
const NO_TIME: &str = "が無い＝裁定の時刻が無い";
const BAD_TIME: &str = "が裁定の時刻の形でない（年-月-日か UTC の分・形は rules.yaml の ruled_at_format）";

impl Work {
    /// 素の check の標準出力のうち `prefix` で始まる行。
    fn lines(&self, prefix: &str) -> Vec<String> {
        let out = Command::new(env!("CARGO_BIN_EXE_folio")).args(["check", "--dir"]).arg(&self.0).output().unwrap();
        let text = String::from_utf8(out.stdout).unwrap();
        text.lines().filter(|l| l.starts_with(prefix)).map(str::to_string).collect()
    }
}

/// 歯 8（便 204）: 閾値の行と開発規律の行の 1 行から裁定の時刻の欄を外すと、種別 裁定 id の違反がちょうど 1 つ出る（面の床の
/// 手前で数える）。
#[test]
fn f204_a_rules_row_without_a_time_is_one_violation() {
    for (marker, id) in [("{id: R-10,", "R-10"), ("{id: D-8,", "D-8")] {
        let w = Work::new("f204-none");
        w.set("rules.yaml", marker, "ruled_at", None);
        assert!(w.read("rules.yaml").lines().any(|l| l.contains(marker) && !l.contains("ruled_at")), "{id}");
        assert_eq!(w.check().0, [format!("[裁定 id] rules.yaml: 行 {id} の ruled_at {NO_TIME}")], "{id}");
    }
}

/// 歯 9（便 204）: 裁定の時刻の形の違う字（区切りが「/」・空の字・JST の時刻付き・時が 1 桁・秒付き・Z の無い分）と、字でない
/// 値（一覧・表・null）は、どれも違反がちょうど 1 つ。
#[test]
fn f204_a_time_out_of_form_is_one_violation() {
    let (file, marker, _) = R10;
    let bad = [
        ("\"2026/09/12\"", "「2026/09/12」"),
        ("\"\"", "「」"),
        ("\"2026-09-12 20:25 JST\"", "「2026-09-12 20:25 JST」"),
        ("\"2026-09-12T1:25Z\"", "「2026-09-12T1:25Z」"),
        ("\"2026-09-12T11:25:00Z\"", "「2026-09-12T11:25:00Z」"),
        ("\"2026-09-12T11:25\"", "「2026-09-12T11:25」"),
    ];
    for (value, shown) in bad {
        let w = Work::new("f204-form");
        w.set(file, marker, "ruled_at", Some(value));
        assert_eq!(w.check().0, [format!("[裁定 id] rules.yaml: 行 R-10 の ruled_at{shown}{BAD_TIME}")], "{value}");
    }
    let listed = "が字でない（一覧か表）＝裁定の時刻が無い";
    for (value, want) in [("[2026-09-12]", listed), ("{at: 2026-09-12}", listed), ("null", NO_TIME)] {
        let w = Work::new("f204-shape");
        w.set(file, marker, "ruled_at", Some(value));
        assert_eq!(w.check().0, [format!("[裁定 id] rules.yaml: 行 R-10 の ruled_at {want}")], "{value}");
    }
}

/// 歯 10（便 204）: 年-月-日と UTC の分（tsuzuri の規則の表の形）は違反 0。骨格の印（未記入）は、裁定の欄も骨格の印なら
/// 数えず（裁定の欄の まだ分からない だけ）、裁定の欄が埋まっていれば違反。
#[test]
fn f204_the_two_forms_pass_and_the_mark_waits_only_with_the_ruling() {
    let (file, marker, key) = R10;
    for value in ["2026-09-28", "\"2026-09-24T22:39Z\""] {
        let w = Work::new("f204-ok");
        w.set(file, marker, "ruled_at", Some(value));
        assert!(w.check().0.is_empty(), "{value}");
    }
    let base = Work::new("f204-base").check();
    let w = Work::new("f204-both");
    w.set(file, marker, key, Some("未記入"));
    w.set(file, marker, "ruled_at", Some("未記入"));
    let (v, p) = w.check();
    assert!(v.is_empty(), "{v:?}");
    assert_eq!(p.len(), base.1.len() + 1, "{p:?}");
    assert!(p.contains(&format!("# まだ分からない: rules.yaml: 行 R-10 の ruling {MARK}")), "{p:?}");
    let w = Work::new("f204-half");
    w.set(file, marker, "ruled_at", Some("未記入"));
    let (v, p) = w.check();
    assert_eq!(v, [format!("[裁定 id] rules.yaml: 行 R-10 の ruled_at「未記入」{BAD_TIME}")]);
    assert_eq!(p, base.1);
}

/// 歯 11（便 204）: 閾値の行の種別を human-review にすると種別 schema の違反がちょうど 1 つ出て、開発規律の行の human-review と
/// 閾値の行のほかの 3 つの種別は違反 0。
#[test]
fn f204_a_threshold_row_of_human_review_is_one_violation() {
    let w = Work::new("f204-kind");
    w.set("rules.yaml", "{id: R-10,", "kind", Some("human-review"));
    assert_eq!(
        w.lines("["),
        ["[schema] rules.yaml: 行 R-10 の kind「human-review」は閾値の行に置けない（kind_map_to_constitution の左辺に無い＝開発規律の行の作法）"]
    );
    for kind in ["deny", "build-check", "detect"] {
        let w = Work::new("f204-kind-ok");
        w.set("rules.yaml", "{id: R-10,", "kind", Some(kind));
        assert!(w.lines("[").is_empty(), "{kind}");
    }
    assert!(Work::new("f204-d").read("rules.yaml").lines().any(|l| l.contains("{id: D-8,") && l.contains("kind: human-review")));
}
