//! 相談の窓の作業場の残りの読み書きを `plain` へ移した歯（接頭辞 cwrst_・行 cs-plain-rest・判断の記録 ADR-55）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に、git init した repo と state dir と起草の置き場と偽の bd・bdw と、
//! 作業場の外の file（窓の symlink の指す先）を置く。作業場の file を symlink（外の file を指す）か fifo か途中の段の symlink に
//! 替えた見本で、移した口が外の file を書き替えず、fifo で止まらず（口は timeout 10 で包み rc 124 を落ちにする）、1 行で名指して
//! 断ることを見る。否定の見本は正しい見本から 1 つの file だけを替える。
#![cfg(test)]

use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::mpsc;
use std::time::Duration;
use std::{fs, thread};

use tsuzuri_boundary::consult::plain::{
    LINKED, MID, NOT_PLAIN, NOT_UNDER, TOO_BIG, create_plain, odd, odd_dir, plain_names, read_text,
    write_plain,
};
use tsuzuri_boundary::consult::{
    findings, odd_files, odd_line, odd_lines, plain_ws, procs, read_window,
};
use tsuzuri_contract::consult::{
    Finding, FindingId, Form, ProcMark, Starter, WindowFile, WindowId,
};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::finding::check;
use tsuzuri_core::consult::launch::{PLUGIN_VERSION, VERSION_ENV};

const LEDGER: &str = r#"[{"id":"fx-c","title":"根","status":"open","issue_type":"epic","updated_at":"2026-10-03T00:00:00Z","notes":"NOTES"}]"#;

/// 外の file の字（どの口の後も替わらないこと）。
const OUTSIDE: &str = "外の字\n";

fn w(n: u32) -> WindowId {
    WindowId::new(n).unwrap()
}

fn err(o: &Output) -> String {
    String::from_utf8(o.stderr.clone()).unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git").arg("-C").arg(dir).args(args).status();
    assert!(ok.unwrap().success(), "git {args:?}");
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn fifo(path: &Path) {
    let made = Command::new("mkfifo").arg(path).status().unwrap();
    assert!(made.success());
}

/// `f` を別の thread で撃ち、5 秒の内に返した値（fifo で止まれば落ちる）。
fn within<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || tx.send(f()).unwrap());
    rx.recv_timeout(Duration::from_secs(5))
        .expect("5 秒の内に返らない")
}

/// 歯ごとの置き場。
struct Fx {
    root: PathBuf,
    repo: PathBuf,
    ws: PathBuf,
    outside: PathBuf,
}

impl Fx {
    /// 置き場と、窓 cw1（問う窓・process の印 1・所見 cw1-1）の作業場と外の file を作る。
    fn new(name: &str) -> Fx {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cwrst")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, state, drafts) = (root.join("repo"), root.join("state"), root.join("drafts"));
        for d in [&repo, &state, &drafts, &root.join("bin"), &root.join("out")] {
            fs::create_dir_all(d).unwrap();
        }
        git(&repo, &["init", "-q"]);
        git(
            &repo,
            &["config", "scribe2.statedir", state.to_str().unwrap()],
        );
        git(
            &repo,
            &["config", "tsuzuri.draftsdir", drafts.to_str().unwrap()],
        );
        let r = root.display().to_string();
        script(&root.join("bin/bd"), &format!("exec cat '{r}/ledger.json'"));
        script(
            &root.join("bin/bdw"),
            &format!("echo \"$@\" >> '{r}/bdw.log'"),
        );
        script(&root.join("bin/tmux"), "exit 0");
        let ws = drafts.join("consult-cw1");
        for d in [".consult", "bundle", "findings", "work"] {
            fs::create_dir_all(ws.join(d)).unwrap();
        }
        let outside = root.join("out/f");
        fs::write(&outside, OUTSIDE).unwrap();
        let fx = Fx {
            root,
            repo,
            ws,
            outside,
        };
        fx.notes(&[]);
        fx.put(".consult/window.json", &wire::encode(&window()).unwrap());
        fx.put(".consult/proc-1.json", &wire::encode(&mark(1)).unwrap());
        fx.put("findings/cw1-1.json", &wire::encode(&finding()).unwrap());
        fx.put("work/report.md", "調べのレポート\n");
        fx.put("work/probe.py", "print(1)\n");
        fx
    }

    fn notes(&self, lines: &[&str]) {
        let text = LEDGER.replace("NOTES", &lines.join("\\n"));
        fs::write(self.root.join("ledger.json"), text).unwrap();
    }

    fn put(&self, rel: &str, text: &str) {
        fs::write(self.ws.join(rel), text).unwrap();
    }

    /// 作業場の `rel` を外の file への symlink に替える（外の file に `text` を書く）。
    fn link(&self, rel: &str, text: &str) {
        fs::write(&self.outside, text).unwrap();
        let _ = fs::remove_file(self.ws.join(rel));
        symlink(&self.outside, self.ws.join(rel)).unwrap();
    }

    /// 作業場の `rel` を fifo に替える。
    fn pipe(&self, rel: &str) {
        let _ = fs::remove_file(self.ws.join(rel));
        fifo(&self.ws.join(rel));
    }

    fn outside(&self) -> String {
        fs::read_to_string(&self.outside).unwrap()
    }

    /// tz consult を timeout 10 で包んで撃つ（fifo で止まれば rc 124）。
    fn tz(&self, args: &[&str]) -> Output {
        let path = format!(
            "{}:{}",
            self.root.join("bin").display(),
            std::env::var("PATH").unwrap()
        );
        Command::new("timeout")
            .arg("10")
            .arg(env!("CARGO_BIN_EXE_tz"))
            .arg("consult")
            .args(args)
            .current_dir(&self.repo)
            .env("PATH", path)
            .env_remove("TZ_CONSULT_ID")
            .env(VERSION_ENV, PLUGIN_VERSION)
            .output()
            .unwrap()
    }

    fn written(&self) -> String {
        fs::read_to_string(self.root.join("bdw.log")).unwrap_or_default()
    }
}

fn window() -> WindowFile {
    WindowFile {
        id: w(1),
        form: Form::Ask,
        topic: None,
        model: "fable".into(),
        effort: "xhigh".into(),
        starter: Starter::Seat,
        uttered: None,
        request: None,
        made: "20261003T1400Z".into(),
    }
}

fn mark(k: u32) -> ProcMark {
    ProcMark {
        k,
        form: Form::Ask,
        pid: 0,
        at: "20261003T1401Z".into(),
        again: false,
        tmux_window: None,
        account: None,
    }
}

/// 所見 cw1-1（fixture complete.json の草稿・添え物 work/report.md と work/probe.py）。
fn finding() -> Finding {
    let text = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/consult/finding-2/complete.json");
    let draft = check(&fs::read_to_string(text).unwrap()).unwrap();
    Finding::new(FindingId::new(w(1), 1).unwrap(), "20261003T1500Z", draft)
}

/// 断りの 1 行（口の名と `odd_line` の字）だけを標準エラーに持つか。
fn refused_once(o: &Output, verb: &str, rel: &str, why: &str) {
    let want = format!("tz consult {verb}: {}\n", odd_line(w(1), rel, why));
    assert_eq!(err(o), want);
}

#[test]
fn cwrst_read_text_reads_only_plain_files() {
    let fx = Fx::new("read");
    let ws = fx.ws.clone();
    fx.put(".consult/a", "あ\n");
    assert_eq!(
        read_text(&ws, ".consult/a", 64).unwrap().as_deref(),
        Some("あ\n")
    );
    assert_eq!(read_text(&ws, ".consult/none", 64).unwrap(), None);
    assert_eq!(read_text(&ws, "none/a", 64).unwrap(), None);
    let why = |rel: &str, max: u64| read_text(&ws, rel, max).unwrap_err().to_string();
    assert_eq!(why(".consult/a", 3), TOO_BIG);
    assert_eq!(why("../consult-cw1/.consult/a", 64), NOT_UNDER);
    fx.link(".consult/a", "外の控え\n");
    assert_eq!(why(".consult/a", 64), NOT_PLAIN);
    symlink(ws.join(".consult"), ws.join("dl")).unwrap();
    fx.put("findings/a", "あ\n");
    assert_eq!(
        read_text(&ws, "findings/a", 64).unwrap().as_deref(),
        Some("あ\n")
    );
    assert_eq!(why("dl/proc-1.json", 64), MID);
    fx.pipe("findings/a");
    let ws2 = ws.clone();
    let got = within(move || read_text(&ws2, "findings/a", 64).map_err(|e| e.to_string()));
    assert_eq!(got, Err(NOT_PLAIN.to_string()));
}

#[test]
fn cwrst_write_plain_never_writes_outside() {
    let fx = Fx::new("write");
    let ws = fx.ws.clone();
    write_plain(&ws, "bundle/x", b"long text\n").unwrap();
    write_plain(&ws, "bundle/x", b"s\n").unwrap();
    assert_eq!(fs::read_to_string(ws.join("bundle/x")).unwrap(), "s\n");
    let why = |rel: &str| write_plain(&ws, rel, b"w\n").unwrap_err().to_string();
    fx.link("bundle/l", OUTSIDE);
    assert_eq!(why("bundle/l"), NOT_PLAIN);
    fs::hard_link(&fx.outside, ws.join("bundle/h")).unwrap();
    assert_eq!(why("bundle/h"), LINKED);
    symlink(fx.root.join("out"), ws.join("bl")).unwrap();
    assert_eq!(why("bl/f"), MID);
    assert_eq!(why("../../out/f"), NOT_UNDER);
    fx.pipe("bundle/p");
    let ws2 = ws.clone();
    let got = within(move || {
        create_plain(&ws2, "bundle/p")
            .map(|_| ())
            .map_err(|e| e.to_string())
    });
    assert_eq!(got, Err(NOT_PLAIN.to_string()));
    assert_eq!(fx.outside(), OUTSIDE);
    fs::remove_file(ws.join("bundle/h")).unwrap();
    fs::hard_link(ws.join("bundle/x"), ws.join("bundle/h")).unwrap();
    fs::remove_file(ws.join("bundle/x")).unwrap();
    write_plain(&ws, "bundle/h", b"one\n").unwrap();
    assert_eq!(fs::read_to_string(ws.join("bundle/h")).unwrap(), "one\n");
}

#[test]
fn cwrst_odd_names_links_fifos_and_dirs() {
    let fx = Fx::new("odd");
    let ws = fx.ws.clone();
    assert_eq!(odd(&ws, ".consult/window.json"), None);
    assert_eq!(odd(&ws, ".consult/none"), None);
    assert_eq!(odd(&ws, "none/x"), None);
    assert_eq!(odd(&ws, "findings").as_deref(), Some(NOT_PLAIN));
    fx.link(".consult/l", OUTSIDE);
    fx.pipe(".consult/p");
    assert_eq!(odd(&ws, ".consult/l").as_deref(), Some(NOT_PLAIN));
    assert_eq!(odd(&ws, ".consult/p").as_deref(), Some(NOT_PLAIN));
    assert_eq!(odd_dir(&ws, "bundle"), None);
    assert_eq!(odd_dir(&ws, "none"), None);
    fs::remove_dir(ws.join("bundle")).unwrap();
    symlink(fx.root.join("out"), ws.join("bundle")).unwrap();
    assert_eq!(odd_dir(&ws, "bundle").as_deref(), Some(MID));
    assert_eq!(odd(&ws, "bundle/f").as_deref(), Some(MID));
    assert_eq!(plain_names(&ws, ".consult"), ["proc-1.json", "window.json"]);
    assert!(plain_names(&ws, "bundle").is_empty());
}

#[test]
fn cwrst_readers_skip_links_and_fifos() {
    let fx = Fx::new("readers");
    let ws = fx.ws.clone();
    assert_eq!(read_window(&ws), Some(window()));
    fx.put(".consult/proc-2.json", &wire::encode(&mark(2)).unwrap());
    assert_eq!(procs(&ws).iter().map(|p| p.k).collect::<Vec<_>>(), [1, 2]);
    fx.link(".consult/proc-2.json", &wire::encode(&mark(2)).unwrap());
    fx.pipe(".consult/proc-3.json");
    let ws2 = ws.clone();
    let ks = within(move || procs(&ws2).iter().map(|p| p.k).collect::<Vec<_>>());
    assert_eq!(ks, [1]);
    let id = FindingId::new(w(1), 2).unwrap();
    symlink(
        ws.join("findings/cw1-1.json"),
        ws.join(format!("findings/{id}.json")),
    )
    .unwrap();
    assert_eq!(findings(&ws, w(1)), [FindingId::new(w(1), 1).unwrap()]);
    let odd = [
        ".consult/proc-2.json",
        ".consult/proc-3.json",
        "findings/cw1-2.json",
    ];
    let want: Vec<(String, String)> = odd.map(|r| (r.to_string(), NOT_PLAIN.to_string())).to_vec();
    assert_eq!(odd_files(&ws), want);
    let line = odd_line(w(1), ".consult/proc-2.json", NOT_PLAIN);
    assert_eq!(plain_ws(&ws, w(1)), Err((1, line)));
    assert_eq!(odd_lines(ws.parent().unwrap()).len(), 3);
    fx.link(".consult/window.json", &wire::encode(&window()).unwrap());
    assert_eq!(read_window(&ws), None);
    assert_eq!(fx.outside(), wire::encode(&window()).unwrap());
}

#[test]
fn cwrst_found_dir_link_is_not_listed() {
    let fx = Fx::new("fdir");
    let ws = fx.ws.clone();
    assert_eq!(plain_ws(&ws, w(1)), Ok(()));
    let out = fx.root.join("out/findings");
    fs::create_dir_all(&out).unwrap();
    fs::rename(ws.join("findings/cw1-1.json"), out.join("cw1-1.json")).unwrap();
    fs::remove_dir(ws.join("findings")).unwrap();
    symlink(&out, ws.join("findings")).unwrap();
    assert!(findings(&ws, w(1)).is_empty());
    assert_eq!(odd_files(&ws), [("findings".to_string(), MID.to_string())]);
}

#[test]
fn cwrst_launch_refuses_an_odd_workspace() {
    let fx = Fx::new("launch");
    let o = fx.tz(&["launch", "cw1", "--dry-run"]);
    assert!(!err(&o).contains("読まず書かない"), "{}", err(&o));
    fx.link(".consult/ask-2.json", OUTSIDE);
    let o = fx.tz(&["launch", "cw1"]);
    assert_eq!(o.status.code(), Some(1));
    refused_once(&o, "launch", ".consult/ask-2.json", NOT_PLAIN);
    assert_eq!(
        (fx.outside(), fx.written()),
        (OUTSIDE.to_string(), String::new())
    );
    fs::remove_file(fx.ws.join(".consult/ask-2.json")).unwrap();
    fx.pipe(".consult/proc-2.json");
    let o = fx.tz(&["launch", "cw1", "--dry-run"]);
    assert_eq!(o.status.code(), Some(1));
    refused_once(&o, "launch", ".consult/proc-2.json", NOT_PLAIN);
    assert!(out(&o).is_empty());
}

#[test]
fn cwrst_bundle_refuses_and_keeps_outside() {
    let fx = Fx::new("bundle");
    let o = fx.tz(&["bundle", "cw1"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).starts_with("束 "));
    fx.link("bundle/ledger.json", OUTSIDE);
    let o = fx.tz(&["bundle", "cw1"]);
    assert_eq!(o.status.code(), Some(1));
    refused_once(&o, "bundle", "bundle/ledger.json", NOT_PLAIN);
    assert_eq!(fx.outside(), OUTSIDE);
    fs::remove_file(fx.ws.join("bundle/ledger.json")).unwrap();
    fx.pipe("bundle/digest");
    let o = fx.tz(&["bundle", "cw1"]);
    assert_eq!(o.status.code(), Some(1));
    refused_once(&o, "bundle", "bundle/digest", NOT_PLAIN);
}

#[test]
fn cwrst_show_and_dispose_refuse_links() {
    let fx = Fx::new("show");
    fx.notes(&["相談の受け = cw1-1・経路 = 見張り・時刻 = 20261003T1500Z"]);
    fx.link("work/report.md", OUTSIDE);
    let into = fx.root.join("into");
    fs::create_dir_all(&into).unwrap();
    let args = [
        "dispose",
        "cw1-1",
        "--verdict",
        "一部採る",
        "--reason",
        "理由",
        "--keep",
        "work/report.md",
        "--drop",
        "work/probe.py",
        "--into",
        into.to_str().unwrap(),
    ];
    let o = fx.tz(&args);
    assert_eq!(o.status.code(), Some(1));
    let line = odd_line(w(1), "work/report.md", NOT_PLAIN);
    assert_eq!(err(&o), format!("tz consult dispose: {line}\n"));
    assert!(!into.join("docs").exists() && fx.written().is_empty());
    fs::remove_file(fx.ws.join("work/report.md")).unwrap();
    fx.put("work/report.md", "調べのレポート\n");
    let o = fx.tz(&args);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let kept = into.join("docs/consult/kept/cw1-1/work/report.md");
    assert_eq!(fs::read_to_string(kept).unwrap(), "調べのレポート\n");
    let text = fs::read_to_string(fx.ws.join("findings/cw1-1.json")).unwrap();
    fx.link("findings/cw1-1.json", &text);
    let o = fx.tz(&["show", "cw1-1", "--via", "見張り"]);
    assert_eq!(o.status.code(), Some(1));
    refused_once(&o, "show", "findings/cw1-1.json", NOT_PLAIN);
}

#[test]
fn cwrst_list_close_watch_name_and_go_on() {
    let fx = Fx::new("scan");
    fx.pipe(".consult/proc-2.json");
    let o = fx.tz(&["list"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    refused_once(&o, "list", ".consult/proc-2.json", NOT_PLAIN);
    assert!(out(&o).contains("cw1"));
    let o = fx.tz(&["watch", "--max", "1"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    refused_once(&o, "watch", ".consult/proc-2.json", NOT_PLAIN);
    let o = fx.tz(&["close", "cw1", "--by", "seat"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    refused_once(&o, "close", ".consult/proc-2.json", NOT_PLAIN);
    assert!(fx.written().contains("相談の閉じ"));
}
