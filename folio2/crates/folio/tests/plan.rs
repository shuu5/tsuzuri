//! 計画の設計ノートの歯（便 183・docs/design/delivery-183.md §1 (c)・判断の記録 ADR-31 決定 (2)・要件書 FR27・AC29）。
//! 土台は凍結した写し（tests/fixtures/floor_base/design-intent/）の写し全部を一時 dir の `design-intent/` に、器の導出 file を
//! 写しの根の `contracts/` に作り、歯の中の最小の手書き（計画のノート plan.yaml と、規則の表の名札の行 R-27〔欄 key plan-note〕と
//! 条 P-2 の関係の行）を足して素の folio check と folio derive を撃つ。もう 1 本の設計ノートは土台の見本 example.yaml（契約表の行 a）。
//! 版管理は作らない（土台の写しの床は版管理が まだ分からない）＝歯は種別 note と 裁定 id の違反の行と、まだ分からない の行の増減を数える。
//! 便 207（docs/design/delivery-207.md §1 (c)）の歯 f207_ は、土台の数の上限の行（値 99）の計画だけの行の上限（欄 key plan-rows）を下げ、
//! 計画だけの行を置き場の合計で数えることを見る。

use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::process::{Command, Output, Stdio};

const FLOOR_BASE: &str = "tests/fixtures/floor_base/design-intent";
const BEGIN: &str = "# folio:rows:begin — 生成区間・手で直さない・正本は置き場の契約表（folio derive --write が書く）";
const END: &str = "# folio:rows:end";
const RULING: &str = "f2-648 notes 2026-09-28 10:29 JST";
const R19: &str = "ruled_at: 2026-09-28, stage: post, key: note-chapters}\n";
const RELATION: &str = "rules: [R-3, R-5, R-19]";

fn row27(id: &str, value: &str) -> String {
    format!(
        "  - {{id: {id}, article: P-2, what: 計画のノートの名札, value: {value}, kind: build-check, status: 仮, ruling: {RULING}, ruled_at: 2026-09-28, stage: post, key: plan-note}}\n"
    )
}

/// 計画のノート（行の索引は土台の見本の行 a・計画だけの行 b と c・判断の表 d1）。
fn plan_note() -> String {
    format!(
        "meta:
  id: plan
  title: 計画のノート
  version: v0.1
  status: effective
  generated: 2026-09-28
  profile: design-note
  approval:
    - {{who: 持ち主, date: 2026-09-28, ruling: {RULING}, verbatim: 承認する, surface: R-8}}
sections:
  - n: 1
    type: prose
    title: 目的
    body: 計画の見本。
  - n: 2
    type: row-index
    title: 行の索引
    rows:
      {BEGIN}
      - {{id: a, doc: example}}
      {END}
  - n: 3
    type: row-plan
    title: 計画だけの行
    rows:
      - {{id: b, what: 次の行, size: S, files: [crates/x.rs], depends: [a]}}
      - {{id: c, what: その次の行, depends: [b], ruling: {RULING}, note: 注}}
  - n: 4
    type: decision-table
    title: 判断
    rows:
      - {{id: d1, text: 行を足す, ruling: {RULING}}}
"
    )
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

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// 写しの一時 dir（歯の終わりに消す）。
struct Work(PathBuf);

impl Work {
    /// 名札の行と計画のノートを持つ写し（`with_row` が偽なら名札の行も関係の行も足さない）。
    fn new(case: &str, with_row: bool) -> Work {
        let root = std::env::temp_dir().join(format!("folio-plan-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_tree(&repo().join(FLOOR_BASE), &root.join("design-intent"));
        fs::create_dir_all(root.join("contracts")).unwrap();
        fs::copy(repo().join("contracts/schema.toml"), root.join("contracts/schema.toml")).unwrap();
        let w = Work(root);
        fs::write(w.path("design-note/plan.yaml"), plan_note()).unwrap();
        if with_row {
            w.edit("rules.yaml", R19, &format!("{R19}{}", row27("R-27", "plan")));
            w.edit("constitution.yaml", RELATION, "rules: [R-3, R-5, R-19, R-27]");
        }
        w
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.0.join("design-intent").join(rel)
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.path(rel)).unwrap()
    }

    /// 字の 1 か所の置き換え。
    fn edit(&self, rel: &str, from: &str, to: &str) {
        let t = self.read(rel);
        assert_eq!(t.matches(from).count(), 1, "{rel}: 「{from}」が 1 か所でない");
        fs::write(self.path(rel), t.replacen(from, to, 1)).unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .args(args)
            .arg("--dir")
            .arg(self.0.join("design-intent"))
            .output()
            .expect("folio を起動できない")
    }

    /// 素の check の、種別 note と 裁定 id の違反の行（標準出力）と、まだ分からない の行（標準エラー）。
    fn check(&self) -> (Vec<String>, Vec<String>) {
        let out = self.run(&["check"]);
        let pick = |b: &[u8], ps: &[&str]| {
            String::from_utf8_lossy(b)
                .lines()
                .filter(|l| ps.iter().any(|p| l.starts_with(p)))
                .map(str::to_string)
                .collect::<Vec<_>>()
        };
        (
            pick(&out.stdout, &["[note] ", "[裁定 id] ", "[schema] "]),
            pick(&out.stderr, &["# まだ分からない: "]),
        )
    }

    /// 編集時の口（便 199）: `rel` に `text` を書こうとしているときの終了と、止める行と つながりの行（接頭辞を外した字）。
    fn propose(&self, rel: &str, text: &str) -> (i32, Vec<String>, Vec<String>) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_folio"))
            .args(["check", "--proposed", rel, "--dir"])
            .arg(self.0.join("design-intent"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("folio を起動できない");
        child.stdin.take().unwrap().write_all(text.as_bytes()).unwrap();
        let out = child.wait_with_output().unwrap();
        let lines: Vec<String> = String::from_utf8_lossy(&out.stdout).lines().map(str::to_string).collect();
        let stops = lines.iter().filter(|l| l.starts_with('[')).cloned().collect();
        let links = lines.iter().filter_map(|l| l.strip_prefix("# つながり（編集は止めない・事後の床が数える）: ")).map(str::to_string).collect();
        (out.status.code().unwrap(), stops, links)
    }

    fn derive(&self, flag: &str) -> Output {
        self.run(&["derive", flag, "--out", "../contracts"])
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn text(out: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

const ROW_A: &str = "      - {id: a, title: 図の生成の口 3 つを 1 便で置く,";

/// 歯 1（AC29 の 1 つ目）: 名札の行と計画のノートを持つ写しで床が合格し、契約表に行を 1 つ足すと違反 1（導出と違う）、
/// folio derive --write の後に合格する。derive --check は書く前に差分 1・書いた後に一致。器の導出 file が無ければ書かない。
#[test]
fn f183_a_new_contract_row_is_one_violation_until_derive_write() {
    let base = Work::new("base", true);
    let (v, p) = base.check();
    assert!(v.is_empty(), "{v:?}");
    let w = Work::new("add", true);
    let row = w.read("design-note/example.yaml").lines().find(|l| l.starts_with(ROW_A)).unwrap().to_string();
    w.edit("design-note/example.yaml", &format!("{row}\n"), &format!("{row}\n{}\n", row.replacen("{id: a,", "{id: a2,", 1)));
    let (v2, p2) = w.check();
    assert_eq!(v2, ["[note] design-note/plan.yaml: 行の索引の生成区間が契約表からの導出と違う（folio derive --write で書き直す）"]);
    assert_eq!(p2, p);
    let drift = w.derive("--check");
    assert_eq!(drift.status.code(), Some(1), "{}", text(&drift));
    assert!(text(&drift).contains("DRIFT: design-note/plan.yaml（行の索引の生成区間が契約表からの導出と違う"), "{}", text(&drift));
    // 器の導出 file が無い置き場は書けない（まだ分からない・計画のノートは変えない）
    let schema = w.0.join("contracts/schema.toml");
    let keep = fs::read(&schema).unwrap();
    fs::remove_file(&schema).unwrap();
    let before = w.read("design-note/plan.yaml");
    assert_eq!(w.derive("--write").status.code(), Some(2));
    assert_eq!(w.read("design-note/plan.yaml"), before);
    fs::write(&schema, keep).unwrap();
    let wrote = w.derive("--write");
    assert_eq!(wrote.status.code(), Some(0), "{}", text(&wrote));
    assert!(w.read("design-note/plan.yaml").contains("      - {id: a, doc: example}\n      - {id: a2, doc: example}\n      # folio:rows:end\n"));
    assert_eq!(w.check().0, Vec::<String>::new());
    assert_eq!(w.derive("--check").status.code(), Some(0));
}

/// 歯 2（AC29 の 2 つ目と 3 つ目）: 計画だけの行に索引の id を足すと違反 1、依存にどこにも無い id を書くと違反 1、
/// 計画だけの行の id を重ねると違反 1。
#[test]
fn f183_a_planned_row_in_the_index_a_dangling_depends_or_a_twin_is_one_violation() {
    let file = "design-note/plan.yaml";
    let cases = [
        ("{id: c, what: その次の行,", "{id: a, what: その次の行,", "計画だけの行「a」が行の索引に在る（契約の行が在る＝計画だけの行の節から外す）"),
        ("depends: [b], ruling", "depends: [zz], ruling", "計画だけの行「c」の depends「zz」が行の索引にも計画だけの行にも無い（宙に浮いた依存）"),
        ("{id: c, what: その次の行,", "{id: b, what: その次の行,", "計画だけの行 id「b」が 2 度在る"),
    ];
    for (from, to, want) in cases {
        let w = Work::new("row", true);
        w.edit(file, from, to);
        assert_eq!(w.check().0, [format!("[note] {file}: {want}")], "{to}");
    }
}

/// 歯 3（AC29 の 4 つ目ほか）: 計画のノートを消すと まだ分からない 1。下書き・名札の行 2 本・値が文書 id の形でない、も
/// まだ分からない 1 で、計画の違反は出ない（2 本は欄 key の床が種別 schema の違反を別に出す）。
#[test]
fn f183_no_plan_note_a_draft_two_rows_or_a_bad_value_is_unknown() {
    let (_, base) = Work::new("base3", true).check();
    let gone = Work::new("gone", true);
    fs::remove_file(gone.path("design-note/plan.yaml")).unwrap();
    let draft = Work::new("draft", true);
    draft.edit("design-note/plan.yaml", "status: effective", "status: draft");
    let two = Work::new("two", true);
    two.edit("rules.yaml", "key: plan-note}\n", &format!("key: plan-note}}\n{}", row27("R-28", "plan")));
    two.edit("constitution.yaml", "R-19, R-27]", "R-19, R-27, R-28]");
    let bad = Work::new("bad", true);
    bad.edit("rules.yaml", "value: plan,", "value: Plan,");
    // 設計ノートが 1 本も無い（欄の決まりの schema.yaml だけ）置き場と、設計ノートの置き場が無い置き場でも、名札の行が在れば黙らない（改訂 a）
    let none = Work::new("none", true);
    for n in ["example.yaml", "figures.yaml", "plan.yaml"] {
        fs::remove_file(none.path(&format!("design-note/{n}"))).unwrap();
    }
    let nodir = Work::new("nodir", true);
    fs::remove_dir_all(nodir.path("design-note")).unwrap();
    for (w, why, schema) in [
        (&gone, "rules.yaml: 計画の名札の行が名指す計画のノート design-note/plan.yaml が無い", 0),
        (&draft, "design-note/plan.yaml: 計画のノートの状態が effective でない（draft）＝計画の床を数えない", 0),
        (&two, "rules.yaml: 計画のノートの名札の行が読めない: 欄 key が plan-note の閾値の行が 2 本ある", 1),
        (&bad, "rules.yaml: 計画のノートの名札の行が読めない: 行 R-27 の value「Plan」が文書 id の形でない", 0),
        (&none, "rules.yaml: 計画の名札の行が名指す計画のノート design-note/plan.yaml が無い", 0),
        (&nodir, "rules.yaml: 計画の名札の行が名指す計画のノート design-note/plan.yaml が無い", 0),
    ] {
        let (v, p) = w.check();
        assert_eq!(v.iter().filter(|l| l.starts_with("[schema] ")).count(), schema, "{v:?}");
        assert!(!v.iter().any(|l| l.starts_with("[note] ")), "{v:?}");
        assert_eq!(p.len(), base.len() + 1, "{p:?}");
        assert!(p.contains(&format!("# まだ分からない: {why}")), "{p:?}");
    }
}

/// 歯 4（AC29 の 5 つ目）: 名札の行を消し、索引の節を残した写しでは違反 1（名札の行が無くても置き場の決まりは掛かる）。
/// 計画だけの行の節も残せば違反 2。名札の行が在っても、名指されないノートに索引の節を置けば違反 1。
#[test]
fn f183_the_index_outside_the_plan_note_is_a_violation_with_or_without_the_row() {
    let only_index = Work::new("noindex", false);
    let plan = only_index.read("design-note/plan.yaml");
    let cut = &plan[plan.find("  - n: 3\n").unwrap()..plan.find("  - n: 4\n").unwrap()];
    only_index.edit("design-note/plan.yaml", cut, "");
    let head = "[note] design-note/plan.yaml: §2: 節の型 row-index は計画の名札の行（欄 key が plan-note の閾値の行）が名指す計画のノートにだけ置ける（名札の行が無い）";
    assert_eq!(only_index.check().0, [head]);
    let both = Work::new("noboth", false).check().0;
    assert_eq!(both.len(), 2, "{both:?}");
    assert_eq!(both[0], head);
    assert!(both[1].contains("§3: 節の型 row-plan は"), "{both:?}");
    let other = Work::new("other", true);
    other.edit(
        "design-note/example.yaml",
        "\nfigures:\n",
        "  - n: 7\n    type: row-index\n    title: 行の索引\n    rows: []\n\nfigures:\n",
    );
    let v = other.check().0;
    assert_eq!(v.len(), 1, "{v:?}");
    assert!(v[0].contains("design-note/example.yaml: §7: 節の型 row-index は") && v[0].ends_with("（名指すのは plan）"), "{v:?}");
}

/// 歯 5: 計画だけの行の size は字・files と depends は字の一覧（値は器の語と照らさない）。判断の表の ruling は決定の欄の
/// 床が裁定 id の形を数え（便 A の関数）、判断の表はどの設計ノートにも置ける。
#[test]
fn f183_size_files_shapes_and_the_decision_ruling_are_read() {
    let file = "design-note/plan.yaml";
    for (from, to, want) in [
        ("size: S,", "size: [S],", "[note] design-note/plan.yaml: §3 の行 b: 欄「size」が字の形でない"),
        ("files: [crates/x.rs],", "files: crates/x.rs,", "[note] design-note/plan.yaml: §3 の行 b: 欄「files」が字の一覧の形でない"),
        ("size: S,", "size: XL,", ""),
        ("{id: d1, text: 行を足す, ruling: f2-648 notes 2026-09-28 10:29 JST}", "{id: d1, text: 行を足す, ruling: 持ち主の裁定}",
         "[裁定 id] design-note/plan.yaml: §4 の行 d1 の ruling「持ち主の裁定」に台帳 id が無い（決定の欄・形は adr/schema.yaml の ruling_pattern）"),
    ] {
        let w = Work::new("shape", true);
        w.edit(file, from, to);
        let want: Vec<&str> = if want.is_empty() { vec![] } else { vec![want] };
        assert_eq!(w.check().0, want, "{to}");
    }
    // 判断の表は名指されないノートにも置け、その行の ruling も数える
    let w = Work::new("table", true);
    w.edit(
        "design-note/example.yaml",
        "\nfigures:\n",
        "  - n: 7\n    type: decision-table\n    title: 判断\n    rows:\n      - {id: e1, text: 見本の判断, ruling: 見本}\n\nfigures:\n",
    );
    assert_eq!(
        w.check().0,
        ["[裁定 id] design-note/example.yaml: §7 の行 e1 の ruling「見本」に台帳 id が無い（決定の欄・形は adr/schema.yaml の ruling_pattern）"]
    );
}

/// 歯 6: 行の索引は file 名の順（文書 id の順ではない・ex-b.yaml が ex.yaml より先）・表の中の順（id の字の順ではない）に並び、folio derive --write が書いた字を 2 度目は変えない。
#[test]
fn f183_the_index_follows_file_names_then_table_order() {
    let w = Work::new("order", true);
    let row = w.read("design-note/example.yaml").lines().find(|l| l.starts_with(ROW_A)).unwrap().to_string();
    let wave = format!(
        "meta:\n  id: a-wave\n  title: 波\n  version: v0.1\n  status: example\n  generated: 2026-09-28\n  profile: design-note\nsections:\n  - n: 1\n    type: prose\n    title: 目的\n    body: 波の見本。\n  - n: 2\n    type: contract-table\n    title: 契約表\n    rows:\n{}\n{}\n",
        row.replacen("{id: a,", "{id: z,", 1),
        row.replacen("{id: a,", "{id: y,", 1)
    );
    fs::write(w.path("design-note/a-wave.yaml"), &wave).unwrap();
    // file 名の順（ex-b.yaml が ex.yaml より先）と文書 id の順（ex が ex-b より先）が割れる組（改訂 a）
    for (id, rid) in [("ex", "x1"), ("ex-b", "x2")] {
        let body = wave.replacen("id: a-wave", &format!("id: {id}"), 1).replacen("{id: z,", &format!("{{id: {rid},"), 1);
        let body = body.lines().filter(|l| !l.starts_with("      - {id: y,")).collect::<Vec<_>>().join("\n") + "\n";
        fs::write(w.path(&format!("design-note/{id}.yaml")), body).unwrap();
    }
    assert_eq!(w.derive("--write").status.code(), Some(0));
    let plan = w.read("design-note/plan.yaml");
    let want = format!(
        "      {BEGIN}\n      - {{id: z, doc: a-wave}}\n      - {{id: y, doc: a-wave}}\n      - {{id: x2, doc: ex-b}}\n      - {{id: x1, doc: ex}}\n      - {{id: a, doc: example}}\n      {END}\n"
    );
    assert!(plan.contains(&want), "{plan}");
    let again = w.derive("--write");
    assert!(text(&again).contains("書いた 0 file"), "{}", text(&again));
    assert_eq!(w.read("design-note/plan.yaml"), plan);
    assert!(w.check().0.is_empty());
}

/// 歯 7（改訂 a）: 印を散文の節の body の中に置き（区間の字は導出と合う）、行の索引の節の rows を空にした写しで、床は違反 1、
/// folio derive --check は 1 と同じ理由の DRIFT の行を出す（床と --check が同じ関数で比べる・判断の記録 ADR-31 決定 (2)(ウ)）。
#[test]
fn f183_markers_outside_the_section_fail_the_floor_and_derive_check_alike() {
    let w = Work::new("outside", true);
    let file = "design-note/plan.yaml";
    let region = format!("    rows:\n      {BEGIN}\n      - {{id: a, doc: example}}\n      {END}\n");
    w.edit(file, &region, "    rows: []\n");
    w.edit(file, "    body: 計画の見本。\n", &format!("    body: |\n      計画の見本。\n      {BEGIN}\n      - {{id: a, doc: example}}\n      {END}\n"));
    let why = "行の索引の節の行が生成区間の導出と違う（印が節の rows の外に在る・folio derive --write で書き直す）";
    assert_eq!(w.check().0, [format!("[note] {file}: {why}")]);
    let out = w.derive("--check");
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(text(&out).contains(&format!("DRIFT: {file}（{why}）")), "{}", text(&out));
}

/// 便 199 (c) 4（ADR-33 決定 (2)）: 計画の床の突き合わせの字（契約表からの導出との違い・索引に在る計画だけの行・宙に浮いた依存）は
/// 口が止めず（0）つながりに名指し、素の床は同じ字で数える。計画だけの行の id の重なりと生成区間の印の崩れ（1 つの file の形）は止める（1）。
#[test]
fn f199_plan_cross_checks_are_links_but_shapes_stop() {
    let file = "design-note/plan.yaml";
    let w = Work::new("mouth", true);
    let example = w.read("design-note/example.yaml");
    let row = example.lines().find(|l| l.starts_with(ROW_A)).unwrap().to_string();
    let added = example.replacen(&format!("{row}\n"), &format!("{row}\n{}\n", row.replacen("{id: a,", "{id: a2,", 1)), 1);
    let plan = w.read(file);
    let cases = [
        ("design-note/example.yaml", added, 0, "行の索引の生成区間が契約表からの導出と違う（folio derive --write で書き直す）"),
        (file, plan.replacen("{id: c, what: その次の行,", "{id: a, what: その次の行,", 1), 0, "計画だけの行「a」が行の索引に在る（契約の行が在る＝計画だけの行の節から外す）"),
        (file, plan.replacen("depends: [b], ruling", "depends: [zz], ruling", 1), 0, "計画だけの行「c」の depends「zz」が行の索引にも計画だけの行にも無い（宙に浮いた依存）"),
        (file, plan.replacen("{id: c, what: その次の行,", "{id: b, what: その次の行,", 1), 1, "計画だけの行 id「b」が 2 度在る"),
        (file, plan.replacen("      # folio:rows:end\n", "", 1), 1, "行の索引の生成区間の印が 1 対でない（begin 1・end 0）"),
    ];
    for (rel, text, code, want) in cases {
        let line = format!("[note] {file}: {want}");
        let (got, stops, links) = w.propose(rel, &text);
        assert_eq!(got, code, "{want}: {stops:?} {links:?}");
        let (hit, miss) = if code == 0 { (&links, &stops) } else { (&stops, &links) };
        assert_eq!(hit, std::slice::from_ref(&line), "{want}");
        assert!(miss.is_empty(), "{want}: {miss:?}");
        let keep = w.read(rel);
        fs::write(w.path(rel), &text).unwrap();
        assert!(w.check().0.contains(&line), "素の床に無い: {line}");
        fs::write(w.path(rel), keep).unwrap();
    }
}

/// 便 207 (c)10（AC33・ADR-35 決定 (1)(イ)）: 土台の欄 key が `key` の行（値 99）の値を置き換える。
fn set_cap(w: &Work, key: &str, value: &str) {
    let t = w.read("rules.yaml");
    let line = t.lines().find(|l| l.ends_with(&format!("key: {key}}}"))).expect("欄 key の行が無い");
    w.edit("rules.yaml", line, &line.replacen("\"99 行 以下\"", &format!("\"{value}\""), 1));
}

/// 便 207 (c)10（AC33・ADR-35 決定 (1)(イ)）: 計画だけの行は置き場の生きたノートの合計で数える。計画のノートの 2 行は上限 1 で違反 1
/// （置き場を名指し、今の数は # の行）、上限 2 で合格。もう 1 本の下書きに計画だけの行を 1 つ足すと（名札の外の違反も出る）合計 3 で
/// 上限 2 を超える。見本に足した計画だけの行は数えない。
#[test]
fn f207_plan_rows_are_summed_over_the_place() {
    let over = |cap: usize| format!("[note] design-note/: 計画だけの行が多すぎる（plan-rows の上限 {cap} 行 以下）");
    let w = Work::new("f207-plan-rows", true);
    set_cap(&w, "plan-rows", "1 行 以下");
    let (v, u) = w.check();
    assert_eq!(v, [over(1)], "{u:?}");
    assert!(text(&w.run(&["check"])).contains("# 今の数: design-note/: 計画だけの行の今の数 2（plan-rows の上限 1 行 以下）"));
    let w = Work::new("f207-plan-rows-2", true);
    set_cap(&w, "plan-rows", "2 行 以下");
    assert_eq!(w.check().0, Vec::<String>::new());
    let plan = "  - n: 7\n    type: row-plan\n    title: 計画だけの行\n    rows:\n      - {id: x1, what: 見本の計画}\n";
    w.edit("design-note/example.yaml", "\nfigures:\n", &format!("\n{plan}figures:\n"));
    assert!(!w.check().0.contains(&over(2)), "見本の計画だけの行を数えた");
    fs::write(
        w.path("design-note/extra.yaml"),
        format!("meta:\n  id: extra\n  title: もう 1 本\n  version: v0.1\n  status: draft\n  generated: 2026-09-29\n  profile: design-note\nsections:\n{}", plan.replace("n: 7", "n: 1").replace("x1", "y1")),
    )
    .unwrap();
    let out = w.run(&["check"]);
    assert!(w.check().0.contains(&over(2)), "{}", text(&out));
    assert!(text(&out).contains("# 今の数: design-note/: 計画だけの行の今の数 3（plan-rows の上限 2 行 以下）"), "{}", text(&out));
}
