//! 行 v-daily の歯: 日に 1 度の全部の撃ち（xtask の src/daily.rs・判断の記録 ADR-34 の決定 (6)）を、一時の git の repo の fixture と
//! 偽の cargo と偽の bdw（shell の script）で撃ち、記録の行と memo の書きを照らし、systemd の雛形と main.rs の呼びを字で読む。
#![cfg(test)]

#[path = "../../src/daily.rs"]
mod daily;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common::{read_root, strings};
use daily::{CHECK, CHECKOUT, Daily, FETCH, FORCE, NICE, NOTES_MAX, Outcome, SHOW, Seen, USAGE};

/// 2026-10-03T09:00Z の epoch 秒。
const NOW: u64 = 1_791_018_000;

/// 一時の dir（名に pid と字 tag）を作り直す。
fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tsuzuri-vdaily-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("一時の dir");
    dir
}

/// fixture の git（家と system の git の設定を読まない）。
fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "fx")
        .env("GIT_AUTHOR_EMAIL", "fx@example.invalid")
        .env("GIT_COMMITTER_NAME", "fx")
        .env("GIT_COMMITTER_EMAIL", "fx@example.invalid")
        .output()
        .expect("git を起動する");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn commit(dir: &Path, file: &str, body: &str) {
    std::fs::write(dir.join(file), body).expect(file);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", file]);
}

/// 実行の許しを付けた shell の script を書く。
fn script(path: &Path, body: &str) {
    std::fs::write(path, body).expect("script");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
}

/// fixture: origin の repo と、その写し（撃つ置き場）と、偽の cargo と偽の bdw と台帳の dir。
struct Fx {
    dir: PathBuf,
    origin: PathBuf,
    daily: Daily,
}

impl Fx {
    fn new(tag: &str) -> Fx {
        let dir = fresh(tag);
        let origin = dir.join("origin");
        std::fs::create_dir_all(&origin).expect("origin");
        git(&origin, &["init", "-q", "-b", "main"]);
        commit(&origin, "a.txt", "a\n");
        let place = dir.join("place");
        git(
            &dir,
            &["clone", "-q", origin.to_str().expect("path"), "place"],
        );
        commit(&origin, "b.txt", "b\n");
        let ledger = dir.join("ledger");
        std::fs::create_dir_all(&ledger).expect("ledger");
        let d = dir.display();
        script(
            &dir.join("cargo"),
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$(pwd -P)\" \"$TSUZURI_CHECK_NESTED_ALL\" \"$*\" > {d}/cargo.seen\necho check-output\nexit $(cat {d}/cargo.rc)\n"
            ),
        );
        script(
            &dir.join("bdw"),
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$(pwd -P)\" \"$@\" --- >> {d}/bdw.seen\nif [ \"$2\" = show ]; then cat {d}/bdw.show; exit $(cat {d}/bdw.show.rc); fi\nif [ \"$1\" = create ]; then cat {d}/bdw.id; fi\nexit $(cat {d}/bdw.rc)\n"
            ),
        );
        std::fs::write(dir.join("bdw.rc"), "0").expect("bdw.rc");
        std::fs::write(dir.join("bdw.id"), "fx-9\n").expect("bdw.id");
        std::fs::write(dir.join("bdw.show"), shown("fx-9", "open", "n")).expect("bdw.show");
        std::fs::write(dir.join("bdw.show.rc"), "0").expect("bdw.show.rc");
        let daily = Daily {
            root: place,
            log: dir.join("daily.log"),
            out: dir.join("daily.out"),
            ledger,
            parent: "fx-hub".to_string(),
            cargo: dir.join("cargo").display().to_string(),
            bdw: dir.join("bdw").display().to_string(),
        };
        Fx { dir, origin, daily }
    }

    /// 偽の cargo の rc を置いて撃つ。
    fn shoot(&self, rc: i32) -> Outcome {
        std::fs::write(self.dir.join("cargo.rc"), rc.to_string()).expect("cargo.rc");
        daily::run(&self.daily, NOW)
    }

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.dir.join(name)).unwrap_or_default()
    }

    fn sha12(&self) -> String {
        git(&self.origin, &["rev-parse", "HEAD"])
            .get(..12)
            .expect("sha")
            .to_string()
    }
}

impl Drop for Fx {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// bd の show --json の字（bead 1 本の配列・欄は台帳の読みの型 BdLine が要る物と label）。
fn shown(id: &str, status: &str, notes: &str) -> String {
    format!(
        "[{{\"id\":\"{id}\",\"title\":\"memo\",\"status\":\"{status}\",\"updated_at\":\"2026-10-03T09:00:00Z\",\"notes\":\"{notes}\",\"labels\":[\"intake:memo\"]}}]"
    )
}

/// 段 check の撃ちの材料（秒は 5）。
fn seen(stamp: &str, sha: &str, rc: i32) -> Seen {
    Seen {
        stamp: stamp.to_string(),
        sha: sha.to_string(),
        step: "check",
        rc,
        secs: 5,
    }
}

/// 記録の行の頭（時刻・sha・段・rc）と末（memo）が見込みどおりか。
fn assert_line(line: &str, sha: &str, step: &str, rc: i32, memo: &str) {
    let head = format!("2026-10-03T09:00Z sha={sha} step={step} rc={rc} secs=");
    assert!(line.starts_with(&head), "{line}");
    assert!(line.ends_with(&format!(" memo={memo}")), "{line}");
}

#[test]
fn vdaily_consts_fixed() {
    assert_eq!(FORCE, ("TSUZURI_CHECK_NESTED_ALL", "1"));
    assert!(
        read_root("xtask/src/nested/skip.rs")
            .contains("pub const FORCE_ENV: &str = \"TSUZURI_CHECK_NESTED_ALL\";"),
        "行 v-skip の環境変数と同じ名"
    );
    assert_eq!(
        FETCH,
        [
            "fetch",
            "-q",
            "origin",
            "+refs/heads/main:refs/remotes/origin/main"
        ]
    );
    assert_eq!(
        CHECKOUT,
        ["checkout", "-q", "--detach", "refs/remotes/origin/main"]
    );
    assert_eq!(NICE, ("nice", &["-n", "10"][..]));
    assert_eq!(CHECK, ["run", "-q", "-p", "xtask", "--", "check"]);
    assert_eq!(SHOW, ["--readonly", "show"]);
    assert_eq!(NOTES_MAX, 8192);
    assert_eq!(
        USAGE,
        "usage: cargo run -q -p xtask -- daily --log <記録の file> --out <出力の file> --ledger <台帳の作業木> --parent <memo の親の bead> [--bdw <program>]"
    );
    assert_eq!(
        daily::record(&seen("T", "s", 0), "-"),
        "T sha=s step=check rc=0 secs=5 memo=-"
    );
    assert_eq!(
        daily::recur_note(&seen("T", "s", 3)),
        "[再発] T origin/main s の段 check が rc 3"
    );
}

#[test]
fn vdaily_parse_flags() {
    let root = Path::new("/r");
    let full = strings(&[
        "--log", "/l", "--out", "/o", "--ledger", "/g", "--parent", "fx-1",
    ]);
    assert_eq!(
        daily::parse(&full, root, "cargo"),
        Ok(Daily {
            root: PathBuf::from("/r"),
            log: PathBuf::from("/l"),
            out: PathBuf::from("/o"),
            ledger: PathBuf::from("/g"),
            parent: "fx-1".to_string(),
            cargo: "cargo".to_string(),
            bdw: "bdw".to_string(),
        })
    );
    let mut with_bdw = full.clone();
    with_bdw.extend(strings(&["--bdw", "/x/bdw"]));
    assert_eq!(
        daily::parse(&with_bdw, root, "cargo").map(|d| d.bdw),
        Ok("/x/bdw".to_string())
    );
    for (i, flag) in ["--log", "--out", "--ledger", "--parent"]
        .iter()
        .enumerate()
    {
        let mut short = full.clone();
        short.drain(i * 2..i * 2 + 2);
        assert_eq!(
            daily::parse(&short, root, "cargo"),
            Err(format!("{flag} が無い"))
        );
    }
    let mut twice = full.clone();
    twice.extend(strings(&["--log", "/l2"]));
    assert_eq!(
        daily::parse(&twice, root, "c"),
        Err("--log が 2 度在る".to_string())
    );
    let mut unknown = full.clone();
    unknown.push("--dry".to_string());
    assert_eq!(
        daily::parse(&unknown, root, "c"),
        Err("知らない引数: --dry".to_string())
    );
    // 値の欠けと空の値の見本は、full から値の 1 つだけを外した形。
    let mut bare = full.clone();
    bare.pop();
    assert_eq!(
        daily::parse(&bare, root, "c"),
        Err("--parent の値が無い".to_string())
    );
    let empty = strings(&[
        "--log", "", "--out", "/o", "--ledger", "/g", "--parent", "fx-1",
    ]);
    assert_eq!(
        daily::parse(&empty, root, "c"),
        Err("--log の値が無い".to_string())
    );
}

#[test]
fn vdaily_stamp_utc() {
    assert_eq!(daily::stamp(0), "1970-01-01T00:00Z");
    assert_eq!(daily::stamp(NOW), "2026-10-03T09:00Z");
    assert_eq!(daily::stamp(1_709_251_140), "2024-02-29T23:59Z");
    assert_eq!(daily::stamp(951_868_800), "2000-03-01T00:00Z");
    assert!(daily::now() > NOW - 86_400 * 365, "今の epoch 秒");
}

#[test]
fn vdaily_pass_records_no_memo() {
    let fx = Fx::new("pass");
    let out = fx.shoot(0);
    let sha = fx.sha12();
    assert_eq!(out.rc, 0);
    assert_line(&out.line, &sha, "check", 0, "-");
    assert_eq!(fx.read("daily.log"), format!("{}\n", out.line));
    assert_eq!(
        git(&fx.daily.root, &["rev-parse", "HEAD"]).get(..12),
        Some(sha.as_str())
    );
    assert_eq!(
        git(&fx.daily.root, &["status", "--porcelain", "--branch"]),
        "## HEAD (no branch)"
    );
    let place = fx.daily.root.canonicalize().expect("place");
    assert_eq!(
        fx.read("cargo.seen"),
        format!("{}\n1\nrun -q -p xtask -- check\n", place.display())
    );
    assert_eq!(fx.read("daily.out"), "check-output\n");
    assert_eq!(fx.read("bdw.seen"), "", "通った時は台帳に書かない");
}

#[test]
fn vdaily_fail_memo_then_recur() {
    let fx = Fx::new("fail");
    let first = fx.shoot(3);
    let sha = fx.sha12();
    assert_eq!(first.rc, 3);
    assert_line(&first.line, &sha, "check", 3, "fx-9");
    let ledger = fx.daily.ledger.canonicalize().expect("ledger");
    let body = fx.dir.join("daily.memo.md");
    let mut want = vec![ledger.display().to_string()];
    let argv = daily::memo_argv("fx-hub", &body, "2026-10-03T09:00Z", &sha);
    want.extend(argv.clone());
    want.push("---".to_string());
    assert_eq!(fx.read("bdw.seen"), format!("{}\n", want.join("\n")));
    let text = fx.read("daily.memo.md");
    let heads: Vec<&str> = text.lines().filter(|l| l.starts_with("### ")).collect();
    assert_eq!(heads, ["### 出所", "### 観測", "### 候補", "### 昇格条件"]);
    assert!(text.contains("\n引き金: 再発 2\n"), "{text}");
    assert!(text.contains(&format!("origin/main {sha} の木")), "{text}");
    assert!(text.contains("file daily.out") && !text.contains(&fx.dir.display().to_string()));
    assert!(
        argv.iter().any(|a| a == "--labels=intake:memo")
            && argv.iter().any(|a| a == "--parent=fx-hub")
            && argv
                .iter()
                .any(|a| *a == format!("--body-file={}", body.display()))
            && argv.last().is_some_and(|t| t.starts_with("memo "))
    );
    std::fs::write(fx.dir.join("bdw.seen"), "").expect("空にする");
    let second = fx.shoot(100);
    assert_line(&second.line, &sha, "check", 100, "fx-9");
    let note = daily::recur_note(&seen("2026-10-03T09:00Z", &sha, 100));
    assert_eq!(
        fx.read("bdw.seen"),
        format!(
            "{l}\n--readonly\nshow\nfx-9\n--json\n---\n{l}\nupdate\nfx-9\n--append-notes={note}\n---\n",
            l = ledger.display()
        )
    );
    std::fs::write(fx.dir.join("bdw.seen"), "").expect("空にする");
    let third = fx.shoot(0);
    assert_line(&third.line, &sha, "check", 0, "-");
    assert_eq!(fx.read("bdw.seen"), "");
    let fourth = fx.shoot(1);
    assert_line(&fourth.line, &sha, "check", 1, "fx-9");
    assert!(
        fx.read("bdw.seen").contains("\ncreate\n"),
        "通った後の落ちは新しい memo"
    );
    assert_eq!(fx.read("daily.log").lines().count(), 4);
}

#[test]
fn vdaily_fetch_and_bdw_failures() {
    let fx = Fx::new("fetch");
    let gone = fx.dir.join("origin-gone");
    std::fs::rename(&fx.origin, &gone).expect("origin を退ける");
    let out = fx.shoot(0);
    assert_ne!(out.rc, 0);
    assert!(out.line.contains(" step=fetch rc="), "{}", out.line);
    assert!(out.line.ends_with(" memo=fx-9"), "{}", out.line);
    assert_eq!(
        fx.read("cargo.seen"),
        "",
        "fetch が落ちたら check を撃たない"
    );
    std::fs::rename(&gone, &fx.origin).expect("戻す");
    std::fs::write(fx.dir.join("bdw.rc"), "1").expect("bdw.rc");
    std::fs::write(fx.dir.join("daily.log"), "").expect("記録を空に");
    let failed = fx.shoot(2);
    assert_line(&failed.line, &fx.sha12(), "check", 2, "failed");
    assert_eq!(daily::open_memo(&fx.read("daily.log")), None);
}

#[test]
fn vdaily_open_memo_reads_last_line() {
    let line = |rc: i32, memo: &str| daily::record(&seen("T", "s", rc), memo);
    assert_eq!(daily::open_memo(""), None);
    assert_eq!(
        daily::open_memo(&format!("{}\n", line(3, "fx-2"))),
        Some("fx-2".to_string())
    );
    assert_eq!(
        daily::open_memo(&format!("{}\n\n", line(3, "fx-2"))),
        Some("fx-2".to_string()),
        "末の空の行は読み飛ばす"
    );
    // 否定の見本は、id を返す見本から句の 1 つだけを替えた形（最後の行・rc・memo の字）。
    assert_eq!(
        daily::open_memo(&format!("{}\n{}\n", line(3, "fx-2"), line(0, "-"))),
        None,
        "最後の行が通り"
    );
    assert_eq!(
        daily::open_memo(&format!("{}\n{}\n", line(0, "-"), line(1, "fx-3"))),
        Some("fx-3".to_string())
    );
    assert_eq!(daily::open_memo(&format!("{}\n", line(1, "failed"))), None);
    assert_eq!(
        daily::open_memo(&format!("{}\n", line(0, "fx-4"))),
        None,
        "通りの行は memo を開いたままにしない"
    );
    assert_eq!(daily::open_memo(&format!("{}\n", line(1, "-"))), None);
}

#[test]
fn vdaily_units_template() {
    let service = read_root("xtask/systemd/tsuzuri-daily.service");
    let timer = read_root("xtask/systemd/tsuzuri-daily.timer");
    let body = |text: &str| -> Vec<String> {
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect()
    };
    assert_eq!(
        body(&service),
        strings(&[
            "[Unit]",
            "Description=tsuzuri daily full check (ADR-34)",
            "[Service]",
            "Type=oneshot",
            "Environment=PATH=%h/.cargo/bin:%h/.local/bin:/usr/local/bin:/usr/bin:/bin",
            "ExecStart=%h/.cargo/bin/cargo run -q --manifest-path ${TSUZURI_DAILY_PLACE}/Cargo.toml -p xtask -- daily --log ${TSUZURI_DAILY_LOG} --out ${TSUZURI_DAILY_OUT} --ledger ${TSUZURI_DAILY_LEDGER} --parent ${TSUZURI_DAILY_PARENT}",
            "TimeoutStartSec=2h",
        ])
    );
    for name in [
        "TSUZURI_DAILY_PLACE",
        "TSUZURI_DAILY_LOG",
        "TSUZURI_DAILY_OUT",
        "TSUZURI_DAILY_LEDGER",
        "TSUZURI_DAILY_PARENT",
        "CARGO_TARGET_DIR",
        "BDW_BD_BIN",
    ] {
        assert!(
            service
                .lines()
                .any(|l| l.starts_with(&format!("#   {name} "))),
            "注に host の値 {name} の説明"
        );
    }
    assert_eq!(
        body(&timer),
        strings(&[
            "[Unit]",
            "Description=tsuzuri daily full check timer (ADR-34)",
            "[Timer]",
            "OnCalendar=daily",
            "RandomizedDelaySec=30min",
            "Persistent=true",
            "[Install]",
            "WantedBy=timers.target",
        ])
    );
    assert!(
        !service.contains("/home/") && !timer.contains("/home/"),
        "host の path を持たない"
    );
}

#[test]
fn vdaily_main_wiring() {
    let main = read_root("xtask/src/main.rs");
    assert_eq!(main.lines().filter(|l| *l == "mod daily;").count(), 1);
    assert!(main.contains(
        "        Some(\"daily\") => exit_code(daily_task(args.get(1..).unwrap_or_default())),\n"
    ));
    let (_, task) = main.split_once("fn daily_task(").expect("fn daily_task");
    let (task, _) = task.split_once("\n}\n").expect("fn daily_task の終わり");
    let order = [
        "daily::parse(args, &workspace_root(), &cargo)",
        "return 2;",
        "daily::run(&d, daily::now())",
        "emit_err(&format!(\"xtask daily: {}\", outcome.line));",
        "outcome.rc",
    ];
    let at: Vec<usize> = order.iter().map(|s| task.find(s).expect(s)).collect();
    assert!(at.windows(2).all(|w| w[0] < w[1]), "{at:?}");
}

#[test]
fn vdaily_appendable_clauses() {
    let note = "[再発] T origin/main s の段 check が rc 1";
    let room = NOTES_MAX - 1 - note.len();
    let fits = "n".repeat(room);
    for status in ["open", "in_progress"] {
        assert!(
            daily::appendable("fx-9", Some(&shown("fx-9", status, "n")), note),
            "{status}"
        );
    }
    assert!(
        daily::appendable("fx-9", Some(&shown("fx-9", "open", &fits)), note),
        "ちょうど上限"
    );
    // 否定の見本は、足してよい見本から句の 1 つだけを外した形。
    let one = shown("fx-9", "open", "n");
    let cases = [
        ("読めない", None),
        ("JSON でない", Some(one.trim_end_matches(']').to_string())),
        ("0 本", Some("[]".to_string())),
        (
            "2 本",
            Some(format!(
                "{},{}",
                one.trim_end_matches(']'),
                one.trim_start_matches('[')
            )),
        ),
        ("id が違う", Some(shown("fx-8", "open", "n"))),
        ("閉じた", Some(shown("fx-9", "closed", "n"))),
        ("先送り", Some(shown("fx-9", "deferred", "n"))),
        ("満ちる", Some(shown("fx-9", "open", &format!("{fits}n")))),
    ];
    for (why, text) in cases {
        assert!(!daily::appendable("fx-9", text.as_deref(), note), "{why}");
    }
}

#[test]
fn vdaily_closed_memo_starts_new() {
    let fx = Fx::new("closed");
    let ledger = fx.daily.ledger.canonicalize().expect("ledger");
    let show = format!(
        "{}\n--readonly\nshow\nfx-9\n--json\n---\n",
        ledger.display()
    );
    let last = daily::record(&seen("T", "s", 1), "fx-9");
    for (why, status, show_rc, new) in [
        ("閉じた", "closed", "0", "fx-10"),
        ("読めない", "open", "1", "fx-11"),
    ] {
        std::fs::write(fx.dir.join("daily.log"), format!("{last}\n")).expect("記録");
        std::fs::write(fx.dir.join("bdw.show"), shown("fx-9", status, "n")).expect("show");
        std::fs::write(fx.dir.join("bdw.show.rc"), show_rc).expect("show.rc");
        std::fs::write(fx.dir.join("bdw.id"), format!("{new}\n")).expect("id");
        std::fs::write(fx.dir.join("bdw.seen"), "").expect("空にする");
        let out = fx.shoot(2);
        assert!(
            out.line.ends_with(&format!(" memo={new}")),
            "{why}: {}",
            out.line
        );
        let seen = fx.read("bdw.seen");
        assert!(seen.starts_with(&show), "{why}: {seen}");
        assert!(
            seen[show.len()..].starts_with(&format!("{}\ncreate\n", ledger.display())),
            "{why}: {seen}"
        );
        assert!(!seen.contains("\nupdate\n"), "{why}");
        let body = fx.read("daily.memo.md");
        assert!(
            body.contains("\n前の memo fx-9 は open でも in_progress でもないか"),
            "{why}: {body}"
        );
    }
    std::fs::write(fx.dir.join("daily.log"), "").expect("記録を空に");
    fx.shoot(2);
    assert!(
        !fx.read("daily.memo.md").contains("前の memo"),
        "前の memo が無い時は書かない"
    );
}
