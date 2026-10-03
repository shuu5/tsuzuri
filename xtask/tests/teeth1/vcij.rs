//! 行 v-ci の歯: 器 scribe2 の CI にだけ在った門を tsuzuri の CI に載せたこと（判断の記録 ADR-33 の決定 (5) と (14)）。
//! 依存の監査は job scribe2-deny が器の ci.yml の job deny と同じ字で scribe2/ を根にして撃ち、歯の写しの孤児の門は job check の役が
//! 残す読んだ写しの path の和を job insta-refs が task insta-refs（xtask の src/snaprefs.rs）で照らす。器の ci.yml の job の全部が、
//! 写した・和で照らす・入れ子の段が撃つ・後の行へ回したのどれか 1 つにだけ振られていることも字で読む。
#![cfg(test)]

#[path = "../../src/snaprefs.rs"]
mod snaprefs;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// tsuzuri の ci.yml と、持ち込んだ器の ci.yml（repo の根からの相対）。
const OURS: &str = ".github/workflows/ci.yml";
const VESSEL: &str = "scribe2/.github/workflows/ci.yml";
/// 写した job の頭に置く 3 行（job の既定の撃つ dir を scribe2/ にする）。
const DEFAULTS: [&str; 3] = ["defaults:", "run:", "working-directory: scribe2"];
/// 写した job（器の job の名・tsuzuri の job の名）。
const PORTED: [(&str, &str); 1] = [("deny", "scribe2-deny")];
/// 読んだ写しの path の和で照らす器の job（tsuzuri の job insta-refs）。
const SUMMED: [&str; 1] = ["insta"];
/// 入れ子の段（xtask の check の nested・build と歯と clippy と器の xtask の check）が撃つ器の job。
const NESTED: [&str; 3] = ["nextest", "clippy", "xtask-check"];
/// 入口の赤と main の出所を測る門で、後の行 v-gates へ回した器の job（入れ子では tsuzuri の根を測る）。
const LATER: [&str; 2] = ["flip-check", "main-provenance"];
/// 比べる行の頭（job の既定・uses の action と版・tool・run）。
const KEYS: [&str; 6] = [
    "defaults:",
    "run:",
    "working-directory: ",
    "- uses: ",
    "tool: ",
    "- run: ",
];
/// job check と job insta-refs の、読んだ写しの path の受け渡しの字（この順に job の中に在る）。
const CHECK_REFS: [&str; 5] = [
    "- run: cargo run -q -p xtask -- check",
    "INSTA_SNAPSHOT_REFERENCES_FILE: ${{ runner.temp }}/insta-refs.txt",
    "- uses: actions/upload-artifact@",
    "name: insta-refs-${{ matrix.shard }}",
    "path: ${{ runner.temp }}/insta-refs.txt",
];
const SUM_REFS: [&str; 5] = [
    "needs: check",
    "- uses: actions/download-artifact@",
    "pattern: insta-refs-*",
    "path: ${{ runner.temp }}/insta-refs",
    "- run: cargo run -q -p xtask -- insta-refs \"${{ runner.temp }}/insta-refs\"",
];

fn read_root(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 行 jobs: の後の、2 字下げの名と字 : だけの行の名（字の順）。
fn job_names(ci: &str) -> Vec<&str> {
    ci.lines()
        .skip_while(|l| *l != "jobs:")
        .skip(1)
        .filter_map(|l| l.strip_prefix("  ")?.strip_suffix(':'))
        .filter(|n| !n.is_empty() && !n.starts_with([' ', '#']))
        .collect()
}

/// job の本体の行（名の行の次から 4 字下げか空か注の行が続く間・注の行と空の行を除き、行の末の注と前後の空白を除く・字の順）。
fn body(ci: &str, name: &str) -> Vec<String> {
    let head = format!("  {name}:");
    ci.lines()
        .skip_while(|l| *l != head)
        .skip(1)
        .take_while(|l| {
            l.starts_with("    ") || l.trim().is_empty() || l.trim_start().starts_with('#')
        })
        .map(|l| l.split("  #").next().unwrap_or(l).trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// job の本体のうち比べる行（KEYS のどれかで始まる行）。
fn shape(ci: &str, name: &str) -> Vec<String> {
    body(ci, name)
        .into_iter()
        .filter(|l| KEYS.iter().any(|k| l.starts_with(k)))
        .collect()
}

/// 写した job の字の判じ: tsuzuri の job は、頭の DEFAULTS の 3 行に続けて、器の job の比べる行をそのまま持つ。
fn judge(ours: &str, vessel: &str) -> Result<(), String> {
    for (theirs, mine) in PORTED {
        let want = shape(vessel, theirs);
        if want.is_empty() {
            return Err(format!("器の ci.yml に job {theirs} が無い"));
        }
        let expect: Vec<String> = DEFAULTS.iter().map(|s| s.to_string()).chain(want).collect();
        let got = shape(ours, mine);
        if got != expect {
            return Err(format!(
                "job {mine} の字が器の job {theirs} と違う: {got:?}"
            ));
        }
    }
    Ok(())
}

/// lines の中に want の字の行（字 @ で終わる字は、続く 40 字の 16 進の SHA だけの行）がこの順に 1 つずつ在り、
/// 字 - run: で始まる行は want のものだけ。
fn in_order(lines: &[String], want: &[&str]) -> Result<(), String> {
    let mut at = 0;
    for w in want {
        let hit = |l: &String| match w.strip_suffix('@').map(|_| l.strip_prefix(*w)) {
            Some(Some(sha)) => sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
            Some(None) => false,
            None => l == w,
        };
        let Some(found) = lines.iter().skip(at).position(hit) else {
            return Err(format!("字 {w} の行が順に無い: {lines:?}"));
        };
        at += found + 1;
    }
    let runs = lines.iter().filter(|l| l.starts_with("- run: ")).count();
    let wanted = want.iter().filter(|w| w.starts_with("- run: ")).count();
    if runs == wanted {
        Ok(())
    } else {
        Err(format!("run の行が {runs} 個"))
    }
}

/// 読んだ写しの path の受け渡しの判じ: job check と job insta-refs の字と、ci.yml の中の環境変数の字の数。
fn refs_wiring(ci: &str) -> Result<(), String> {
    in_order(&body(ci, "check"), &CHECK_REFS)?;
    in_order(&body(ci, "insta-refs"), &SUM_REFS)?;
    let n = ci.matches(snaprefs::REFS_ENV).count();
    if n == 1 {
        Ok(())
    } else {
        Err(format!("ci.yml の中で {} が {n} 度", snaprefs::REFS_ENV))
    }
}

/// job name の中の最初の old を new に替えた字（job の名の行より前の字は替えない）。
fn swap(ci: &str, name: &str, old: &str, new: &str) -> String {
    let at = ci
        .find(&format!("\n  {name}:\n"))
        .unwrap_or_else(|| panic!("job {name} が無い"));
    let rel = ci[at..]
        .find(old)
        .unwrap_or_else(|| panic!("job {name} に {old} が無い"));
    format!("{}{new}{}", &ci[..at + rel], &ci[at + rel + old.len()..])
}

/// 正しい ci.yml から 1 つの句だけを外す、写した job の見本（見本の名・job の名・元の字・替える字）。
fn bad_ports() -> [(&'static str, &'static str, &'static str, &'static str); 5] {
    [
        (
            "撃つ dir の無い deny",
            "scribe2-deny",
            "        working-directory: scribe2\n",
            "",
        ),
        (
            "撃つ dir の違う deny",
            "scribe2-deny",
            "working-directory: scribe2",
            "working-directory: scribe2/crates",
        ),
        (
            "run の字の違う deny",
            "scribe2-deny",
            "- run: cargo deny check",
            "- run: cargo deny check -A unmaintained",
        ),
        (
            "tool の版の違う deny",
            "scribe2-deny",
            "tool: cargo-deny@0.20.2",
            "tool: cargo-deny@0.20.1",
        ),
        (
            "action の版の違う deny",
            "scribe2-deny",
            "setup-rust-toolchain@166cdcfd",
            "setup-rust-toolchain@266cdcfd",
        ),
    ]
}

/// 正しい ci.yml から 1 つの句だけを外す、受け渡しの見本。
fn bad_refs() -> [(&'static str, &'static str, &'static str, &'static str); 7] {
    [
        (
            "環境変数の file の違う check",
            "check",
            "${{ runner.temp }}/insta-refs.txt\n      #",
            "${{ runner.temp }}/insta-refs.txt.old\n      #",
        ),
        (
            "SHA で固定しない上げ",
            "check",
            "upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
            "upload-artifact@v7",
        ),
        (
            "役の番号の無い artifact の名",
            "check",
            "name: insta-refs-${{ matrix.shard }}",
            "name: insta-refs",
        ),
        ("check を待たない和", "insta-refs", "    needs: check\n", ""),
        (
            "名の違う artifact の下ろし",
            "insta-refs",
            "pattern: insta-refs-*",
            "pattern: refs-*",
        ),
        (
            "下ろした dir と違う dir の照らし",
            "insta-refs",
            "-- insta-refs \"${{ runner.temp }}/insta-refs\"",
            "-- insta-refs \"${{ runner.temp }}\"",
        ),
        (
            "run の行の 2 つ目",
            "insta-refs",
            "      - run: cargo run -q -p xtask -- insta-refs",
            "      - run: true\n      - run: cargo run -q -p xtask -- insta-refs",
        ),
    ]
}

#[test]
fn vcij_ports_vessel_jobs() {
    let ours = read_root(OURS);
    let vessel = read_root(VESSEL);
    judge(&ours, &vessel).expect("写した job は器の job と同じ字");
    for (label, name, old, new) in bad_ports() {
        let text = swap(&ours, name, old, new);
        assert_ne!(text, ours, "{label} の見本");
        assert!(judge(&text, &vessel).is_err(), "{label} を通す");
    }
    let renamed = ours.replace("\n  scribe2-deny:\n", "\n  scribe2-audit:\n");
    assert!(
        judge(&renamed, &vessel).is_err(),
        "名の違う deny の job を通す"
    );
    let moved = swap(
        &vessel,
        "deny",
        "- run: cargo deny check",
        "- run: cargo deny check --all-features",
    );
    assert!(
        judge(&ours, &moved).is_err(),
        "器の job の字が替わったのに通す"
    );
}

#[test]
fn vcij_vessel_jobs_classified() {
    let mut named: Vec<&str> = PORTED
        .iter()
        .map(|(theirs, _)| *theirs)
        .chain(SUMMED)
        .chain(NESTED)
        .chain(LATER)
        .collect();
    named.sort_unstable();
    let vessel = read_root(VESSEL);
    let mut jobs = job_names(&vessel);
    jobs.sort_unstable();
    assert_eq!(
        jobs, named,
        "器の ci.yml の job は 4 つの組のどれか 1 つにだけ在る"
    );
    let ours = read_root(OURS);
    let mine: Vec<&str> = job_names(&ours)
        .into_iter()
        .filter(|n| n.starts_with("scribe2-") || n.starts_with("insta"))
        .collect();
    assert_eq!(
        mine,
        ["scribe2-deny", "insta-refs"],
        "tsuzuri の ci.yml の器の job"
    );
    let fixture = "on:\n  push:\njobs:\n  a:\n    # 注\n  b-c:\n    steps:\n";
    assert_eq!(
        job_names(fixture),
        ["a", "b-c"],
        "jobs: の後の 2 字下げの名だけ"
    );
}

#[test]
fn vcij_refs_wiring() {
    let ours = read_root(OURS);
    refs_wiring(&ours).expect("読んだ写しの path の受け渡し");
    for (label, name, old, new) in bad_refs() {
        let text = swap(&ours, name, old, new);
        assert_ne!(text, ours, "{label} の見本");
        assert!(refs_wiring(&text).is_err(), "{label} を通す");
    }
    let twice = ours.replace(
        "    runs-on: ubuntu-latest\n    defaults:",
        "    env:\n      INSTA_SNAPSHOT_REFERENCES_FILE: /tmp/x\n    runs-on: ubuntu-latest\n    defaults:",
    );
    assert!(refs_wiring(&twice).is_err(), "環境変数の 2 度目を通す");
    let main = read_root("xtask/src/main.rs");
    assert_eq!(
        main.matches("\nmod snaprefs;\n").count(),
        1,
        "main.rs の mod の行"
    );
    assert_eq!(
        main.matches("Some(\"insta-refs\") => exit_code(insta_refs(")
            .count(),
        1,
        "main.rs の task insta-refs の腕"
    );
}

fn strings(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn vcij_refs_judge() {
    let tracked = strings(&[
        "scribe2/a/snapshots/x.snap",
        "scribe2/b/snapshots/y.snap",
        "crates/c/snapshots/z.snap",
        "scribe2/a/x.rs",
    ]);
    let lines = strings(&[
        "/r/scribe2/a/snapshots/x.snap",
        "",
        "/r/scribe2/b/tests/./../snapshots/y.snap",
        "/r/scribe2/a/snapshots/x.snap",
    ]);
    let refs = snaprefs::referenced("/r", &lines).expect("根の下の行");
    assert_eq!(refs.len(), 2, "重なる行は 1 つ");
    assert_eq!(
        snaprefs::judge(&tracked, &refs),
        Ok("写し 2 本・読まれた path 2 本・読まれない写し 0".to_string()),
        "scribe2/ の外の写しと .rs は見ない"
    );
    let one: BTreeSet<String> = refs.iter().take(1).cloned().collect();
    assert!(
        snaprefs::judge(&tracked, &one).is_err(),
        "読まれない写しを通す"
    );
    for extra in [
        "scribe2/a/snapshots/x.snap.new",
        "scribe2/a/.x.pending-snap",
    ] {
        let mut more = tracked.clone();
        more.push(extra.to_string());
        assert!(snaprefs::judge(&more, &refs).is_err(), "{extra} を通す");
    }
    let none = strings(&["crates/c/snapshots/z.snap"]);
    assert!(snaprefs::judge(&none, &refs).is_err(), "写しの無い木を通す");
    assert!(
        snaprefs::referenced("/r", &strings(&["/other/scribe2/a/snapshots/x.snap"])).is_err(),
        "根の下でない行を通す"
    );
    assert!(
        snaprefs::referenced("/r", &strings(&["/rx/scribe2/a/snapshots/x.snap"])).is_err(),
        "根の名を頭に持つだけの行を通す"
    );
    assert!(
        snaprefs::referenced("/r", &strings(&["/r/../r/scribe2/a/snapshots/x.snap"])).is_ok(),
        "節 .. を解いて根の下"
    );
    assert!(
        snaprefs::referenced("/r", &strings(&["/r/../../r/scribe2/a/snapshots/x.snap"])).is_err(),
        "根より上へ打ち消す行を通す"
    );
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(ok, "git {args:?}");
}

fn put(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("親の dir")).expect("dir を作る");
    std::fs::write(path, text).expect("file を書く");
}

#[test]
fn vcij_refs_run() {
    let root = std::env::temp_dir().join(format!("vcij-refs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for rel in ["scribe2/a/snapshots/x.snap", "scribe2/b/snapshots/y.snap"] {
        put(&root.join(rel), "---\n");
    }
    git(&root, &["init", "-q"]);
    git(&root, &["add", "-A"]);
    let refs = root.join("refs");
    let top = root.to_string_lossy().to_string();
    assert_eq!(snaprefs::run(&root, &refs).0, 1, "下ろした dir が無い");
    put(
        &refs.join("p1/insta-refs.txt"),
        &format!("{top}/scribe2/a/snapshots/x.snap\n"),
    );
    let (rc, lines) = snaprefs::run(&root, &refs);
    assert_eq!(rc, 1, "役 1 つの和は y を読まない: {lines:?}");
    assert_eq!(lines, ["どの歯も読まない写し: scribe2/b/snapshots/y.snap"]);
    put(
        &refs.join("p2/insta-refs.txt"),
        &format!("{top}/scribe2/b/snapshots/y.snap\n"),
    );
    let (rc, lines) = snaprefs::run(&root, &refs);
    assert_eq!(rc, 0, "2 つの役の和: {lines:?}");
    assert_eq!(
        lines,
        ["役の file 2 本・写し 2 本・読まれた path 2 本・読まれない写し 0"]
    );
    put(&root.join("scribe2/c/snapshots/w.snap"), "---\n");
    assert_eq!(snaprefs::run(&root, &refs).0, 0, "追跡されない写しは見ない");
    git(&root, &["add", "-A"]);
    assert_eq!(
        snaprefs::run(&root, &refs).0,
        1,
        "追跡された孤児の写しを通す"
    );
    let _ = std::fs::remove_dir_all(&root);
}
