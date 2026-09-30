//! `folio ceiling --gate`（便 73・docs/design/delivery-73.md §1 (d)(e)）の歯。binary 経由。
//! 正本は便 38 の凍結 fixture の束の source/（tests/fixtures/ceiling/bundle/source/）を一時 dir の design-intent/ へ写したもの。
//! 印は凍結 fixture stamp-pass / stamp-fail / stamp-unknown.yaml を写し、「同じ要約値」の場合は sources の仮の値（64 字の 0）を
//! 写しの正本から sha256sum（子の処理）で測った値に置き換え、「古い」の場合は仮の値のまま置く。
//! 命令は一時 dir を今の dir にして `--dir design-intent` で撃つ（write-set は repo の根からの相対）。
//!
//! 便 126（docs/design/delivery-126.md §1 (e)）: 印の trigger の仮の値も、fresh のときは引き金の要約値の独立の実装
//! （凍結 anchor ceiling-region.txt の trigger の欄と写しの文書を yaml-rust2 で読み、正規化の json を sha256sum で測る・
//! folio の code を呼ばない）で測った値に置き換える。節点の数を見る歯は folio graph --print の節点の行から nodes の表を組み、
//! rest の仮の値と一緒に印の末尾に足す。
//! 便 129（docs/design/delivery-129.md §1 (e) の 1）: 独立の実装は憲法を凍結 anchor の trigger.constitution の scope の範囲で写す。
//! 便 151（docs/design/delivery-151.md §1 (c)(e)）: 独立の実装は判断の記録を状態を問わず全部写す（anchor から status の葉が消えた）。
//! 向きが逆になる既存の 2 通り（ADR-2 の発効・ADR-2 の決定の字）は f151_ の 2 本へ移した。
//! 便 169（docs/design/delivery-169.md §1 (c)(e)）: 門は印の古さで止めない。古さと節点の数で答えを縛った歯（便 126・129・151）
//! は外し、f169_ の 5 本に置き換えた。印の refutes の行は歯の中で書き替えて作る（trigger の独立の実装は印の仮の値に使う）。
//!
//! 便 175（docs/design/delivery-175.md §1 (b)(c)）: 引き金の要約値の独立の実装（trigger_hex）を外した。fixture の印の trigger の行は
//! 仮の値のまま置く（門は読まない＝便 175 の前に書いた印〔trigger・rest・nodes を持つ〕も同じ答えで読む見張り）。
//!
//! 版管理の下の file は書き換えない（`--dir` は必ず一時 dir の中）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// fixture の印の sources の仮の値。
const PLACEHOLDER: &str =
    "sources: sha256 0000000000000000000000000000000000000000000000000000000000000000\n";

/// 天井の正本の観点の reads が指す文書の file（bundle fixture の ceiling.yaml・file 形と dir 形）。
const READ_FILES: [&str; 6] = [
    "index.yaml",
    "constitution.yaml",
    "rules.yaml",
    "srs.yaml",
    "adr/",
    "design-note/",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn findings_fixture(name: &str) -> String {
    fs::read_to_string(
        repo_root()
            .join("tests/fixtures/ceiling/findings")
            .join(name),
    )
    .unwrap()
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or_else(|| {
        panic!(
            "folio が signal で終わった: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 要約値（sha256）は命令 `sha256sum` を子の処理で撃って測る（tests/stamp.rs と同じ形）。
fn sha256_hex(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("sha256sum を起動できない: {e}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "sha256sum の標準入力が無い".to_string())?
        .write_all(bytes)
        .map_err(|e| format!("sha256sum へ書けない: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("sha256sum を待てない: {e}"))?;
    if !out.status.success() {
        return Err("sha256sum が失敗した".to_string());
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let hex = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    if hex.len() != 64 {
        return Err(format!("sha256sum の出力が 16 進 64 字でない: {text}"));
    }
    Ok(hex)
}

// ── 引き金の要約値の独立の実装（便 126・§1 (e)・folio の code を呼ばない）──

/// 一時 dir（design-intent/ = 正本の写し）。
struct Repo {
    td: PathBuf,
}

impl Repo {
    fn new(case: &str) -> Repo {
        let td = std::env::temp_dir().join(format!("folio-gate-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&td);
        copy_tree(
            &repo_root().join("tests/fixtures/ceiling/bundle/source"),
            &td.join("design-intent"),
        );
        Repo { td }
    }

    fn dir(&self) -> PathBuf {
        self.td.join("design-intent")
    }

    /// 写しの正本の要約値（読む文書の file を design-intent からの相対 path の byte 順に連結した sha256）。
    fn sources_hex(&self) -> String {
        let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        for file in READ_FILES {
            let path = self.dir().join(file);
            if let Some(sub) = file.strip_suffix('/') {
                for entry in fs::read_dir(&path).unwrap() {
                    let name = entry.unwrap().file_name().into_string().unwrap();
                    if name.ends_with(".yaml") {
                        files.insert(
                            format!("{sub}/{name}"),
                            fs::read(path.join(&name)).unwrap(),
                        );
                    }
                }
            } else {
                files.insert(file.to_string(), fs::read(&path).unwrap());
            }
        }
        let bytes: Vec<u8> = files.into_values().flatten().collect();
        sha256_hex(&bytes).expect("要約値を測れない")
    }

    /// 印を置く。`fresh` なら sources を今の正本の要約値に合わせる（trigger の行は仮の値のまま・門は読まない）。
    fn put_stamp(&self, name: &str, fresh: bool) {
        let mut text = findings_fixture(name);
        assert!(text.contains(PLACEHOLDER), "{name}: 仮の値が無い");
        if fresh {
            text = text.replace(
                PLACEHOLDER,
                &format!("sources: sha256 {}\n", self.sources_hex()),
            );
        }
        self.write_stamp(&text);
    }

    /// fresh の `name` の印の refutes の節を `rows`（字下げ込みの行・空なら `refutes: []`）に替えて置く（便 169）。
    fn put_stamp_with_stops(&self, name: &str, rows: &[&str]) {
        self.put_stamp(name, true);
        let text = self.read_stamp();
        let start = text.find("\nrefutes:").expect("印に refutes が無い") + 1;
        let end = text.find("\nreads: ").expect("印に reads が無い") + 1;
        let body = if rows.is_empty() {
            "refutes: []\n".to_string()
        } else {
            format!("refutes:\n{}", rows.iter().map(|r| format!("{r}\n")).collect::<String>())
        };
        self.write_stamp(&format!("{}{body}{}", &text[..start], &text[end..]));
    }

    /// 印の観点 `id` の行（改行込み）。
    fn stamp_row(&self, id: &str) -> String {
        let head = format!("  - {{id: {id}, ");
        let line = self
            .read_stamp()
            .lines()
            .find(|l| l.starts_with(&head))
            .unwrap_or_else(|| panic!("印に観点 {id} の行が無い"))
            .to_string();
        format!("{line}\n")
    }

    /// 印の観点 `id` の行の字 `from` を 1 か所だけ `to` に替える（便 169）。
    fn edit_stamp_row(&self, id: &str, from: &str, to: &str) {
        let line = self.stamp_row(id);
        assert_eq!(line.matches(from).count(), 1, "{id} の行に「{from}」が 1 か所でない");
        let text = self.read_stamp();
        self.write_stamp(&text.replacen(&line, &line.replacen(from, to, 1), 1));
    }

    /// 印の観点 `id` の行を外す（便 169）。
    fn drop_stamp_row(&self, id: &str) {
        let line = self.stamp_row(id);
        let text = self.read_stamp();
        self.write_stamp(&text.replacen(&line, "", 1));
    }

    fn read_stamp(&self) -> String {
        fs::read_to_string(self.dir().join("preview/ceiling-stamp.yaml")).unwrap()
    }

    fn write_stamp(&self, text: &str) {
        fs::create_dir_all(self.dir().join("preview")).unwrap();
        fs::write(self.dir().join("preview/ceiling-stamp.yaml"), text).unwrap();
    }

    /// 写しの正本 `file` の字 `from` を 1 か所だけ `to` に置き換える（1 か所でなければ歯を落とす）。
    fn edit(&self, file: &str, from: &str, to: &str) {
        let path = self.dir().join(file);
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text.matches(from).count(), 1, "{file}: 「{from}」が 1 か所でない");
        fs::write(&path, text.replacen(from, to, 1)).unwrap();
    }

    /// `folio ceiling --gate --dir design-intent --write-set <paths…>`（今の dir = 一時 dir）。
    fn gate(&self, write_set: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .current_dir(&self.td)
            .args(["ceiling", "--gate", "--dir", "design-intent", "--write-set"])
            .args(write_set)
            .output()
            .expect("folio を起動できない")
    }

    /// `folio ceiling --gate --dir <dir> --write-set <paths…>`（今の dir = `cwd`・便 142）。
    fn gate_at(&self, cwd: &Path, dir: &Path, write_set: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_folio"))
            .current_dir(cwd)
            .args(["ceiling", "--gate", "--dir"])
            .arg(dir)
            .arg("--write-set")
            .args(write_set)
            .output()
            .expect("folio を起動できない")
    }

    /// 一時 dir を本流の一番上に、その .worktrees/x/ を作業ツリーの一番上に見立て、置き場（印を含む）を写す（便 142）。
    /// 作業ツリーの一番上の絶対 path（symlink を解いた字）を返す。
    fn above_the_worktree(&self) -> PathBuf {
        let top = self.td.join(".worktrees/x");
        copy_tree(&self.dir(), &top.join("design-intent"));
        fs::canonicalize(&top).unwrap()
    }

    fn done(self) {
        let _ = fs::remove_dir_all(&self.td);
    }
}

// ── 1. 設計文書の正本を書き換えない便 ──

#[test]
fn gate_passes_a_delivery_that_touches_no_design_intent() {
    let repo = Repo::new("no-design");
    let run = repo.gate(&[
        "crates/folio/src/gate.rs",
        "+crates/folio/tests/gate.rs",
        "docs/design/delivery-73.md",
    ]);
    repo.done();
    assert_eq!(code(&run), 0, "{}", stdout(&run));
    assert!(stdout(&run).contains("通す"), "{}", stdout(&run));
}

// ── 2. 4 観点合格・要約値が同じ ──

#[test]
fn gate_passes_when_the_stamp_is_all_pass_and_fresh() {
    let repo = Repo::new("pass");
    repo.put_stamp("stamp-pass.yaml", true);
    let run = repo.gate(&["design-intent/srs.yaml", "crates/folio/src/gate.rs"]);
    repo.done();
    assert_eq!(code(&run), 0, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(out.contains("通す") && out.contains("合格"), "{out}");
}

// ── 3. 不合格の観点 ──

#[test]
fn gate_stops_on_a_failed_viewpoint() {
    let repo = Repo::new("fail");
    repo.put_stamp("stamp-fail.yaml", true);
    let run = repo.gate(&["design-intent/srs.yaml"]);
    repo.done();
    assert_eq!(code(&run), 1, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(out.contains("止める") && out.contains("coherence"), "{out}");
}

// ── 5. まだ分からない観点 ──

#[test]
fn gate_is_unknown_on_an_unknown_viewpoint() {
    let repo = Repo::new("unknown");
    repo.put_stamp("stamp-unknown.yaml", true);
    let run = repo.gate(&["+design-intent/adr/ADR-3.yaml"]);
    repo.done();
    assert_eq!(code(&run), 2, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(out.contains("まだ分からない") && out.contains("reality"), "{out}");
}

// ── 6. 印が無い ──

#[test]
fn gate_is_unknown_without_a_stamp() {
    let repo = Repo::new("no-stamp");
    let run = repo.gate(&["design-intent/srs.yaml"]);
    repo.done();
    assert_eq!(code(&run), 2, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(out.contains("まだ分からない") && out.contains("印が無い"), "{out}");
}

// ── 7. preview の生成物と retired は正本でない ──

#[test]
fn gate_ignores_preview_and_retired_paths() {
    let repo = Repo::new("preview-retired");
    let run = repo.gate(&[
        "design-intent/preview/ceiling-stamp.yaml",
        "design-intent/preview/retired/readable.html",
        "design-intent/adr/retired/ADR-0.yaml",
    ]);
    repo.done();
    assert_eq!(code(&run), 0, "{}", stdout(&run));
    let out = stdout(&run);
    assert!(
        out.contains("通す") && out.contains("設計文書の正本を書き換えない便"),
        "{out}"
    );
}

// ── 便 142: 作業ツリーの一番上以外から撃つと まだ分からない（docs/design/delivery-142.md §1 (c) の 2・3） ──

#[test]
fn f142_the_gate_is_unknown_from_above_the_worktree() {
    let repo = Repo::new("f142-above");
    repo.put_stamp("stamp-pass.yaml", true);
    let top = repo.above_the_worktree();
    let main = fs::canonicalize(&repo.td).unwrap();
    let write_set = ["design-intent/srs.yaml", "crates/folio/src/gate.rs"];
    let runs = [
        (repo.gate_at(&main, Path::new(".worktrees/x/design-intent"), &write_set), 2),
        (repo.gate_at(&main, &top.join("design-intent"), &write_set), 2),
        (repo.gate_at(&top, Path::new("design-intent"), &write_set), 0),
        (repo.gate_at(&top, &top.join("design-intent"), &write_set), 0),
    ];
    let code_only = repo.gate_at(
        &main,
        Path::new(".worktrees/x/design-intent"),
        &["crates/folio/src/gate.rs"],
    );
    repo.done();
    for (i, (run, want)) in runs.iter().enumerate() {
        let out = stdout(run);
        assert_eq!(code(run), *want, "撃ち方 {i}: {out}");
        if *want == 2 {
            assert!(
                out.contains("まだ分からない")
                    && out.contains("--dir と write-set の根が違う＝design-intent/srs.yaml")
                    && out.contains("作業ツリーの一番上から撃つ"),
                "撃ち方 {i}: {out}"
            );
        } else {
            assert!(out.contains("通す") && out.contains("印の後の変更は審査していない"), "撃ち方 {i}: {out}");
        }
    }
    let out = stdout(&code_only);
    assert_eq!(code(&code_only), 0, "{out}");
    assert!(out.contains("設計文書の正本を書き換えない便"), "{out}");
}

#[test]
fn f142_the_gate_is_unknown_when_the_dir_is_outside_the_cwd() {
    let repo = Repo::new("f142-outside");
    repo.put_stamp("stamp-pass.yaml", true);
    let top = repo.above_the_worktree();
    let other = repo.td.join("other");
    let crates = top.join("crates");
    fs::create_dir_all(&other).unwrap();
    fs::create_dir_all(&crates).unwrap();
    let place = top.join("design-intent");
    let srs = ["design-intent/srs.yaml"];
    let outside = [
        repo.gate_at(&other, &place, &srs),
        repo.gate_at(&other, Path::new("../.worktrees/x/design-intent"), &srs),
        repo.gate_at(&crates, Path::new("../design-intent"), &srs),
        repo.gate_at(&other, &place, &["crates/folio/src/gate.rs"]),
    ];
    let abs = place.join("srs.yaml");
    let absolute = repo.gate_at(&top, Path::new("design-intent"), &[abs.to_str().unwrap()]);
    repo.done();
    for (i, run) in outside.iter().enumerate() {
        let out = stdout(run);
        assert_eq!(code(run), 2, "撃ち方 {i}: {out}");
        assert!(
            out.contains("まだ分からない")
                && out.contains("--dir が今の dir の下に無い")
                && out.contains("作業ツリーの一番上から撃つ"),
            "撃ち方 {i}: {out}"
        );
    }
    let out = stdout(&absolute);
    assert_eq!(code(&absolute), 2, "{out}");
    assert!(out.contains("--dir と write-set の根が違う"), "{out}");
}

// ── 便 150: --dir が設計文書の置き場でなければ まだ分からない（docs/design/delivery-150.md §1 (c) の 2・3） ──

#[test]
fn f150_the_gate_is_unknown_when_the_dir_is_not_a_place() {
    let repo = Repo::new("f150-not-a-place");
    repo.put_stamp("stamp-pass.yaml", true);
    fs::create_dir_all(repo.td.join("empty")).unwrap();
    let cwd = fs::canonicalize(&repo.td).unwrap();
    let missing = cwd.join("design-intnet");
    let both = ["design-intent/srs.yaml", "crates/folio/src/gate.rs"];
    let dirs: [(&Path, &[&str]); 6] = [
        (Path::new("design-intnet"), &both),
        (Path::new("design-intent/adr"), &both),
        (Path::new("empty"), &both),
        (Path::new("design-intent/srs.yaml"), &both),
        (&missing, &both),
        (Path::new("design-intnet"), &["crates/folio/src/gate.rs"]),
    ];
    let runs: Vec<(String, Output)> = dirs
        .iter()
        .map(|(dir, ws)| (dir.display().to_string(), repo.gate_at(&cwd, dir, ws)))
        .collect();
    let outside = repo.gate_at(&cwd, Path::new("../nope"), &both);
    repo.done();
    for (dir, run) in &runs {
        let out = stdout(run);
        assert_eq!(code(run), 2, "{dir}: {out}");
        let reason = format!(
            "--dir が設計文書の置き場でない（{dir}・constitution.yaml が無い）・作業ツリーの一番上から撃つ"
        );
        assert!(out.contains("まだ分からない") && out.contains(&reason), "{dir}: {out}");
    }
    let out = stdout(&outside);
    assert_eq!(code(&outside), 2, "{out}");
    assert!(
        out.contains("--dir が今の dir の下に無い") && !out.contains("置き場でない"),
        "{out}"
    );
}

#[test]
fn f150_the_gate_reads_the_place_as_before() {
    let repo = Repo::new("f150-place");
    let cwd = fs::canonicalize(&repo.td).unwrap();
    let place = Path::new("design-intent");
    let code_only = repo.gate_at(&cwd, place, &["crates/folio/src/gate.rs"]);
    let no_stamp = repo.gate_at(&cwd, place, &["design-intent/srs.yaml"]);
    repo.put_stamp("stamp-pass.yaml", true);
    let absolute = cwd.join("design-intent");
    let fresh: Vec<Output> = [place, Path::new("./design-intent/"), &absolute]
        .iter()
        .map(|dir| repo.gate_at(&cwd, dir, &["design-intent/srs.yaml"]))
        .collect();
    repo.done();
    let out = stdout(&code_only);
    assert_eq!(code(&code_only), 0, "{out}");
    assert!(out.contains("通す") && out.contains("設計文書の正本を書き換えない便"), "{out}");
    let out = stdout(&no_stamp);
    assert_eq!(code(&no_stamp), 2, "{out}");
    assert!(out.contains("まだ分からない") && out.contains("印が無い"), "{out}");
    for (i, run) in fresh.iter().enumerate() {
        let out = stdout(run);
        assert_eq!(code(run), 0, "撃ち方 {i}: {out}");
        assert!(out.contains("通す") && out.contains("印の後の変更は審査していない"), "撃ち方 {i}: {out}");
    }
}

// ── 便 169: 門は書き換える file に反証で支持された 止める が在るかだけを見て、印の古さで止めない（docs/design/delivery-169.md §1 (c)） ──

/// 通す理由の頭と末尾（印の周・判定は印の字）。
const PASS_HEAD: &str = "folio ceiling: 通す（印の周 gate-case（判定 ";
const UNREVIEWED: &str = "印の後の変更は審査していない";

/// 写しの要件 FR1 の規範文（引き金の中）。
const FR1_SHALL: (&str, &str) = (
    "    shall: folio は易しい質問を推奨回答つきで出す。\n",
    "    shall: folio は易しい質問を推奨回答つきで必ず出す。\n",
);

#[test]
fn f169_a_stale_stamp_without_an_upheld_stop_passes() {
    let stale = Repo::new("f169-stale");
    stale.put_stamp("stamp-pass.yaml", false);
    stale.edit("srs.yaml", FR1_SHALL.0, FR1_SHALL.1);
    let run = stale.gate(&["design-intent/srs.yaml", "+design-intent/adr/ADR-3.yaml"]);
    stale.done();
    let out = stdout(&run);
    assert_eq!(code(&run), 0, "{out}");
    assert!(
        out.starts_with(&format!("{PASS_HEAD}合格）に、")) && out.contains(UNREVIEWED),
        "{out}"
    );

    // 反証待ちだけの まだ分からない 観点（wait: 反証）は file ごとの判定に任せる
    let waiting = Repo::new("f169-stale-wait");
    waiting.put_stamp("stamp-unknown.yaml", false);
    waiting.edit_stamp_row("reality", "}\n", ", wait: 反証}\n");
    let run = waiting.gate(&["design-intent/srs.yaml"]);
    waiting.done();
    let out = stdout(&run);
    assert_eq!(code(&run), 0, "{out}");
    assert!(
        out.starts_with(&format!("{PASS_HEAD}まだ分からない）に、")) && out.contains(UNREVIEWED),
        "{out}"
    );
}

#[test]
fn f169_the_gate_stops_only_on_the_file_of_an_upheld_stop() {
    let repo = Repo::new("f169-upheld");
    repo.put_stamp_with_stops(
        "stamp-pass.yaml",
        &[
            "  - {viewpoint: coherence, finding: C-9, refute: 退けた, at: articles.A-1, file: constitution.yaml}",
            "  - {viewpoint: fidelity, finding: F-1, refute: 支持, at: requirements.FR1.plain, file: srs.yaml}",
            "  - {viewpoint: reality, finding: R-1, refute: 支持, at: ADR-1.decision, file: adr/ADR-1.yaml}",
            "  - {viewpoint: coherence, finding: C-2, refute: 支持, at: sections.x, file: design-note/}",
        ],
    );
    // （write-set・終了コード・標準出力に要る字）
    let cases: [(&[&str], i32, &str); 7] = [
        (&["design-intent/srs.yaml", "crates/folio/src/gate.rs"], 1, "srs.yaml（fidelity F-1）"),
        (&["~./design-intent/adr/ADR-1.yaml"], 1, "adr/ADR-1.yaml（reality R-1）"),
        (&["+design-intent/design-note/new.yaml"], 1, "design-note/（coherence C-2）"),
        (&["design-intent/adr/ADR-2.yaml"], 0, UNREVIEWED),
        (&["design-intent/constitution.yaml"], 0, UNREVIEWED),
        (&["design-intent/rules.yaml", "design-intent/preview/ceiling-stamp.yaml"], 0, UNREVIEWED),
        (&["crates/folio/src/gate.rs"], 0, "設計文書の正本を書き換えない便"),
    ];
    let runs: Vec<Output> = cases.iter().map(|(ws, _, _)| repo.gate(ws)).collect();
    repo.done();
    for ((ws, want, word), run) in cases.iter().zip(&runs) {
        let out = stdout(run);
        assert_eq!(code(run), *want, "{ws:?}: {out}");
        assert!(out.contains(word), "{ws:?}: {out}");
        if *want == 1 {
            assert!(
                out.contains("止める（反証で支持された 止める の場所の file を書き換える: "),
                "{ws:?}: {out}"
            );
        }
    }
}

#[test]
fn f169_an_unrefuted_stop_file_is_unknown() {
    let repo = Repo::new("f169-unrefuted");
    repo.put_stamp_with_stops(
        "stamp-unknown.yaml",
        &[
            "  - {viewpoint: coherence, finding: C-1, refute: 支持, at: requirements.FR1.shall, file: srs.yaml}",
            "  - {viewpoint: reality, finding: R-2, at: requirements.FR2.shall, file: srs.yaml}",
            "  - {viewpoint: fidelity, finding: F-2, refute: まだ分からない, at: thresholds.R-1, file: rules.yaml}",
        ],
    );
    repo.edit_stamp_row("reality", "}\n", ", wait: 反証}\n");
    let srs = repo.gate(&["design-intent/srs.yaml"]);
    let rules = repo.gate(&["design-intent/rules.yaml"]);
    let other = repo.gate(&["design-intent/constitution.yaml"]);
    repo.done();
    let head = "まだ分からない（反証の済んでいない 止める の場所の file を書き換える: ";
    let out = stdout(&srs);
    assert_eq!(code(&srs), 2, "{out}");
    assert!(out.contains(&format!("{head}srs.yaml（reality R-2）")) && !out.contains("C-1"), "{out}");
    let out = stdout(&rules);
    assert_eq!(code(&rules), 2, "{out}");
    assert!(out.contains(&format!("{head}rules.yaml（fidelity F-2）")), "{out}");
    let out = stdout(&other);
    assert_eq!(code(&other), 0, "{out}");
    assert!(out.contains(UNREVIEWED), "{out}");
}

#[test]
fn f169_the_gate_is_unknown_without_a_readable_stamp() {
    let missing = "まだ分からない（印の観点の結果が欠けている: ";
    let mut runs: Vec<(&str, Output, String)> = Vec::new();

    let repo = Repo::new("f169-none");
    runs.push(("印が無い", repo.gate(&["design-intent/srs.yaml"]), "印が無い".to_string()));
    repo.done();

    let repo = Repo::new("f169-dropped");
    repo.put_stamp("stamp-pass.yaml", true);
    repo.drop_stamp_row("reality");
    runs.push(("観点の行が無い", repo.gate(&["design-intent/srs.yaml"]), format!("{missing}reality（無い）")));
    repo.done();

    let repo = Repo::new("f169-no-wait");
    repo.put_stamp("stamp-unknown.yaml", true);
    runs.push((
        "wait の無い まだ分からない",
        repo.gate(&["design-intent/srs.yaml"]),
        format!("{missing}reality（まだ分からない）"),
    ));
    repo.done();

    let repo = Repo::new("f169-dropped-upheld");
    repo.put_stamp("stamp-fail.yaml", true);
    repo.drop_stamp_row("reality");
    runs.push((
        "支持の 止める の file と欠けた観点",
        repo.gate(&["design-intent/srs.yaml"]),
        format!("{missing}reality（無い）"),
    ));
    repo.done();

    let repo = Repo::new("f169-not-three");
    repo.put_stamp("stamp-pass.yaml", true);
    repo.edit_stamp_row("reality", "verdict: 合格,", "verdict: 合格？,");
    runs.push(("3 値でない", repo.gate(&["design-intent/srs.yaml"]), format!("{missing}reality（無い）")));
    repo.done();

    let repo = Repo::new("f169-other-wait");
    repo.put_stamp("stamp-unknown.yaml", true);
    repo.edit_stamp_row("reality", "}\n", ", wait: 反証か}\n");
    runs.push((
        "wait が 反証 でない",
        repo.gate(&["design-intent/srs.yaml"]),
        format!("{missing}reality（まだ分からない）"),
    ));
    repo.done();

    let repo = Repo::new("f169-old-rows");
    repo.put_stamp_with_stops("stamp-fail.yaml", &["  - {viewpoint: coherence, finding: C-1, refute: 支持}"]);
    runs.push((
        "行に file が無い",
        repo.gate(&["design-intent/srs.yaml"]),
        "印が読めない: refutes が読めない".to_string(),
    ));
    repo.done();

    for (case, run, word) in &runs {
        let out = stdout(run);
        assert_eq!(code(run), 2, "{case}: {out}");
        assert!(out.contains("まだ分からない") && out.contains(word.as_str()), "{case}: {out}");
    }
}

#[test]
fn f169_a_dir_item_covers_the_stop_files_under_it() {
    let repo = Repo::new("f169-dir-item");
    repo.put_stamp_with_stops(
        "stamp-pass.yaml",
        &[
            "  - {viewpoint: coherence, finding: C-2, refute: 支持, at: ADR-1.decision, file: adr/ADR-1.yaml}",
            "  - {viewpoint: reality, finding: R-2, at: sections.x, file: design-note/}",
        ],
    );
    let cases: [(&str, i32, &str); 4] = [
        ("design-intent/adr/", 1, "adr/ADR-1.yaml（coherence C-2）"),
        ("~./design-intent/adr", 1, "adr/ADR-1.yaml（coherence C-2）"),
        ("design-intent/design-note/", 2, "design-note/（reality R-2）"),
        ("design-intent/adr/ADR-2.yaml", 0, UNREVIEWED),
    ];
    let runs: Vec<Output> = cases.iter().map(|(p, _, _)| repo.gate(&[p])).collect();
    repo.done();
    for ((path, want, word), run) in cases.iter().zip(&runs) {
        let out = stdout(run);
        assert_eq!(code(run), *want, "{path}: {out}");
        assert!(out.contains(word), "{path}: {out}");
    }
}

// ── 便 178: 置き場そのものか置き場を下に持つ dir の項目は置き場の file を全部書き換える（docs/design/delivery-178.md §1 (c)） ──

/// 置き場を名指す項目の 4 形（`--dir design-intent`・今の dir = 一時 dir）。
const PLACE_ITEMS: [&str; 4] = [".", "design-intent", "design-intent/", "./design-intent"];

/// 設計文書の正本を書き換えない便の理由。
const NO_SOURCE: &str = "設計文書の正本を書き換えない便";

#[test]
fn f178_a_place_item_stops_on_an_upheld_stop() {
    let repo = Repo::new("f178-upheld");
    repo.put_stamp_with_stops(
        "stamp-pass.yaml",
        &[
            "  - {viewpoint: coherence, finding: C-9, refute: 退けた, at: articles.A-1, file: constitution.yaml}",
            "  - {viewpoint: reality, finding: R-1, refute: 支持, at: ADR-1.decision, file: adr/ADR-1.yaml}",
        ],
    );
    let stop = "止める（反証で支持された 止める の場所の file を書き換える: adr/ADR-1.yaml（reality R-1）";
    // （write-set・終了コード・標準出力に要る字）: 4 形と接頭辞の形は 1、置き場を名指さない項目は今どおり
    let mut cases: Vec<(Vec<&str>, i32, &str)> =
        PLACE_ITEMS.iter().map(|p| (vec![*p, "crates/folio/src/gate.rs"], 1, stop)).collect();
    cases.extend([
        (vec!["~./design-intent/"], 1, stop),
        (vec!["+."], 1, stop),
        (vec!["design-intent/adr/"], 1, stop),
        (vec!["design-intent/adr/ADR-2.yaml"], 0, UNREVIEWED),
        (vec!["design-intent/preview/", "design-intent/adr/retired/"], 0, NO_SOURCE),
        (vec!["design-intent-x", "design", "crates", "docs/"], 0, NO_SOURCE),
    ]);
    let runs: Vec<Output> = cases.iter().map(|(ws, _, _)| repo.gate(ws)).collect();
    // 作業ツリーの一番上の上から撃つ（便 142）: 置き場を下に持つ dir は 1、根の違う path と置き場でない --dir は今どおり 2
    let top = repo.above_the_worktree();
    let main = fs::canonicalize(&repo.td).unwrap();
    let deep = Path::new(".worktrees/x/design-intent");
    let above: Vec<(&str, Output)> = [".", ".worktrees", ".worktrees/x/", "./.worktrees/x/design-intent"]
        .iter()
        .map(|p| (*p, repo.gate_at(&main, deep, &[*p])))
        .collect();
    let other_root = repo.gate_at(&main, deep, &[".", "design-intent/srs.yaml"]);
    let not_a_place = repo.gate_at(&top, Path::new("design-intnet"), &["."]);
    repo.done();
    for ((ws, want, word), run) in cases.iter().zip(&runs) {
        let out = stdout(run);
        assert_eq!(code(run), *want, "{ws:?}: {out}");
        assert!(out.contains(word), "{ws:?}: {out}");
    }
    for (p, run) in &above {
        let out = stdout(run);
        assert_eq!(code(run), 1, "{p}: {out}");
        assert!(out.contains(stop), "{p}: {out}");
    }
    let out = stdout(&other_root);
    assert_eq!(code(&other_root), 2, "{out}");
    assert!(out.contains("--dir と write-set の根が違う＝design-intent/srs.yaml"), "{out}");
    let out = stdout(&not_a_place);
    assert_eq!(code(&not_a_place), 2, "{out}");
    assert!(out.contains("--dir が設計文書の置き場でない（design-intnet・"), "{out}");
}

#[test]
fn f178_a_place_item_passes_when_no_stop_is_upheld() {
    let mut runs: Vec<(String, Output)> = Vec::new();
    for (case, rows) in [
        ("f178-none", &[][..]),
        (
            "f178-refuted",
            &["  - {viewpoint: coherence, finding: C-9, refute: 退けた, at: articles.A-1, file: constitution.yaml}"][..],
        ),
    ] {
        let repo = Repo::new(case);
        repo.put_stamp_with_stops("stamp-pass.yaml", rows);
        for p in PLACE_ITEMS {
            runs.push((format!("{case} {p}"), repo.gate(&[p])));
        }
        repo.done();
    }
    for (case, run) in &runs {
        let out = stdout(run);
        assert_eq!(code(run), 0, "{case}: {out}");
        assert!(
            out.starts_with(&format!("{PASS_HEAD}合格）に、")) && out.contains(UNREVIEWED) && !out.contains(NO_SOURCE),
            "{case}: {out}"
        );
    }
}

#[test]
fn f178_a_place_item_is_unknown_on_an_unrefuted_stop() {
    let repo = Repo::new("f178-unrefuted");
    repo.put_stamp_with_stops(
        "stamp-unknown.yaml",
        &[
            "  - {viewpoint: coherence, finding: C-1, refute: 支持, at: requirements.FR1.shall, file: srs.yaml}",
            "  - {viewpoint: reality, finding: R-2, at: thresholds.R-1, file: rules.yaml}",
        ],
    );
    repo.edit_stamp_row("reality", "}\n", ", wait: 反証}\n");
    let runs: Vec<Output> = PLACE_ITEMS.iter().map(|p| repo.gate(&[p])).collect();
    repo.done();
    let word = "まだ分からない（反証の済んでいない 止める の場所の file を書き換える: rules.yaml（reality R-2）";
    for (p, run) in PLACE_ITEMS.iter().zip(&runs) {
        let out = stdout(run);
        assert_eq!(code(run), 2, "{p}: {out}");
        assert!(out.contains(word) && !out.contains("C-1"), "{p}: {out}");
    }
}

#[test]
fn f178_a_place_item_is_unknown_without_a_stamp() {
    let mut runs: Vec<(String, Output, &str)> = Vec::new();
    let repo = Repo::new("f178-no-stamp");
    for p in PLACE_ITEMS {
        runs.push((format!("印が無い {p}"), repo.gate(&[p]), "まだ分からない（印が無い）"));
    }
    repo.done();
    let repo = Repo::new("f178-dropped");
    repo.put_stamp("stamp-pass.yaml", true);
    repo.drop_stamp_row("reality");
    for p in PLACE_ITEMS {
        let word = "まだ分からない（印の観点の結果が欠けている: reality（無い）";
        runs.push((format!("観点の行が無い {p}"), repo.gate(&[p]), word));
    }
    repo.done();
    for (case, run, word) in &runs {
        let out = stdout(run);
        assert_eq!(code(run), 2, "{case}: {out}");
        assert!(out.contains(word), "{case}: {out}");
    }
}
