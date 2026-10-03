//! 席の背景の見張りの歯（接頭辞 cwwat_・設計ノート surface-wave27a 行 cs-watch・受入 AC16 の見張りの 1 行）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に、git init した repo と state dir と起草の置き場と偽の bd を置き、
//! 窓の作業場（窓の控え・process の印・所見）を歯が直に書いて、tz consult watch を短い上限で撃つ。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant, SystemTime};

use tsuzuri_contract::consult::{
    Finding, FindingId, Form, ProcMark, Starter, WindowFile, WindowId,
};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::finding::check;

/// 台帳の写し（根の epic・根の notes は歯が足す）。
const LEDGER: &str = r#"[{"id":"fx-c","title":"根","status":"open","issue_type":"epic","updated_at":"2026-10-03T00:00:00Z","notes":"NOTES"}]"#;

const RQ_LINE: &str = "相談の頼み = rq-20261003T1412Z-1・題 = 題なし・形 = 話す・model = fable・起こし手 = 持ち主の button・時刻 = 20261003T1412Z";

const TAIL: &str = "・「/x/tzw consult watch」を背景で置き直す";

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(ok.success(), "git {args:?}");
}

/// 歯ごとの置き場。
struct Fx {
    root: PathBuf,
    repo: PathBuf,
    drafts: PathBuf,
}

impl Fx {
    fn new(name: &str) -> Fx {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cwwat")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, state, drafts) = (root.join("repo"), root.join("state"), root.join("drafts"));
        for d in [&repo, &state, &drafts, &root.join("bin")] {
            fs::create_dir_all(d).unwrap();
        }
        git(&repo, &["init", "-q"]);
        git(
            &repo,
            &["config", "scribe2.statedir", &state.display().to_string()],
        );
        git(
            &repo,
            &["config", "tsuzuri.draftsdir", &drafts.display().to_string()],
        );
        let bd = root.join("bin/bd");
        fs::write(
            &bd,
            format!("#!/bin/sh\nexec cat '{}/ledger.json'\n", root.display()),
        )
        .unwrap();
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).unwrap();
        let fx = Fx { root, repo, drafts };
        fx.notes(&[]);
        fx
    }

    fn notes(&self, lines: &[&str]) {
        let text = LEDGER.replace("NOTES", &lines.join("\\n"));
        fs::write(self.root.join("ledger.json"), text).unwrap();
    }

    fn command(&self, args: &[&str]) -> Command {
        let path = format!(
            "{}:{}",
            self.root.join("bin").display(),
            std::env::var("PATH").unwrap()
        );
        let mut c = Command::new(env!("CARGO_BIN_EXE_tz"));
        c.args(["consult", "watch"])
            .args(args)
            .current_dir(&self.repo)
            .env("PATH", path)
            .env("TZ_WRAPPER", "/x/tzw");
        c
    }

    fn watch(&self, max: &str) -> Output {
        self.command(&["--max", max]).output().unwrap()
    }

    /// 窓の作業場（控えと、在れば最後の process の印）。
    fn window(&self, n: u32, form: Form, pid: Option<u32>) -> PathBuf {
        let ws = self.drafts.join(format!("consult-cw{n}"));
        for d in [".consult", "findings"] {
            fs::create_dir_all(ws.join(d)).unwrap();
        }
        let w = WindowFile {
            id: WindowId::new(n).unwrap(),
            form,
            topic: None,
            model: "fable".into(),
            effort: "xhigh".into(),
            starter: Starter::Seat,
            uttered: None,
            request: None,
            made: "20261003T1400Z".into(),
        };
        fs::write(ws.join(".consult/window.json"), wire::encode(&w).unwrap()).unwrap();
        if let Some(pid) = pid {
            proc_mark(&ws, 1, form, pid);
        }
        ws
    }

    fn alive(&self) -> PathBuf {
        self.drafts.join("consult-watch.alive")
    }
}

/// 作業場に k 番目の process の印を置く（話す窓は tmux の窓 @7 を持つ・2 番目からは起こし直し）。
fn proc_mark(ws: &Path, k: u32, form: Form, pid: u32) {
    let tmux_window = (form == Form::Talk).then(|| "@7".to_string());
    let m = ProcMark {
        k,
        form,
        pid,
        at: "20261003T1401Z".into(),
        again: k > 1,
        tmux_window,
    };
    let path = ws.join(format!(".consult/proc-{k}.json"));
    fs::write(path, wire::encode(&m).unwrap()).unwrap();
}

fn rc(o: &Output) -> i32 {
    o.status.code().unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}

/// 窓 cw3 の作業場に、fixture complete.json の草稿に id と時刻を足した揃った所見 cw3-1 を置く。
fn answer(ws: &Path) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/consult/finding-2/complete.json");
    let draft = check(&fs::read_to_string(fixture).unwrap()).unwrap();
    let id = FindingId::new(WindowId::new(3).unwrap(), 1).unwrap();
    let f = Finding::new(id, "20261003T1500Z", draft);
    fs::write(ws.join("findings/cw3-1.json"), wire::encode(&f).unwrap()).unwrap();
}

#[test]
fn cwwat_finding_line_once() {
    let fx = Fx::new("finding");
    let ws = fx.window(3, Form::Talk, Some(std::process::id()));
    answer(&ws);
    let o = fx.watch("30");
    assert_eq!(rc(&o), 0);
    let want = format!(
        "相談: 窓 cw3 の所見 cw3-1 が届いた（/x/tzw consult show cw3-1 --via 見張り）{TAIL}\n"
    );
    assert_eq!(out(&o), want);
    assert!(!fx.alive().exists());
    fx.notes(&["相談の受け = cw3-1・経路 = 見張り・時刻 = 20261003T1530Z"]);
    let start = Instant::now();
    let o = fx.watch("1");
    assert_eq!(
        (rc(&o), out(&o)),
        (0, format!("相談: 見張りが上限で終わった{TAIL}\n"))
    );
    assert!(start.elapsed() >= Duration::from_secs(1));
    assert!(!fx.alive().exists());
}

#[test]
fn cwwat_request_stalled_gone_order() {
    let fx = Fx::new("order");
    // 前の process は生きていて最後の process だけが無い問う窓 cw4。
    let ws4 = fx.window(4, Form::Ask, Some(std::process::id()));
    proc_mark(&ws4, 2, Form::Ask, 0);
    // 最後の process の無い問う窓で、受けた所見 cw2-1 を持つ cw2（止まったとしない）。
    let ws2 = fx.window(2, Form::Ask, Some(0));
    fs::write(ws2.join("findings/cw2-1.json"), "{}").unwrap();
    fx.window(5, Form::Talk, Some(0));
    fx.window(8, Form::Ask, None);
    let retired = fx.drafts.join("retired-consult-cw6/findings");
    fs::create_dir_all(&retired).unwrap();
    fs::write(retired.join("cw6-1.json"), "{}").unwrap();
    let live = fx.window(7, Form::Ask, Some(std::process::id()));
    fs::write(live.join("findings/cw7-1.json"), "{}").unwrap();
    // 所見の無い生きている問う窓と話す窓（止まった・閉じられたとしない）。
    fx.window(9, Form::Ask, Some(std::process::id()));
    fx.window(10, Form::Talk, Some(std::process::id()));
    let got7 = "相談の受け = cw7-1・経路 = 一覧・時刻 = 20261003T1500Z";
    let got2 = "相談の受け = cw2-1・経路 = 一覧・時刻 = 20261003T1500Z";
    fx.notes(&[RQ_LINE, got2, got7]);
    let want = "相談: 頼み rq-20261003T1412Z-1 が届いた（/x/tzw consult open --request rq-20261003T1412Z-1 --by button）";
    assert_eq!(out(&fx.watch("30")), format!("{want}{TAIL}\n"));
    let got_rq = "相談の受け = rq-20261003T1412Z-1・経路 = hook・時刻 = 20261003T1413Z";
    fx.notes(&[RQ_LINE, got2, got7, got_rq]);
    let want = "相談: 問う窓 cw4 が所見なしで止まった（/x/tzw consult launch cw4 --again）";
    assert_eq!(out(&fx.watch("30")), format!("{want}{TAIL}\n"));
    let closed4 = "相談の閉じ = cw4・起こし手 = 席・所見 = 0・時刻 = 20261003T1600Z";
    fx.notes(&[RQ_LINE, got2, got7, got_rq, closed4]);
    let want = "相談: 話す窓 cw5 が閉じられた（/x/tzw consult close cw5 --by chat）";
    assert_eq!(out(&fx.watch("30")), format!("{want}{TAIL}\n"));
    let closed5 = "相談の閉じ = cw5・起こし手 = 持ち主のチャット・所見 = 0・時刻 = 20261003T1601Z";
    fx.notes(&[RQ_LINE, got2, got7, got_rq, closed4, closed5]);
    assert_eq!(
        out(&fx.watch("1")),
        format!("相談: 見張りが上限で終わった{TAIL}\n")
    );
    fs::write(live.join("findings/cw7-2.json"), "{}").unwrap();
    fx.notes(&[got2, got7]);
    let o = fx.watch("30");
    assert!(
        out(&o).starts_with("相談: 窓 cw7 の所見 cw7-2 が届いた"),
        "{}",
        out(&o)
    );
}

#[test]
fn cwwat_alive_mark() {
    let fx = Fx::new("alive");
    fs::write(fx.alive(), "pid = 1・始まり = 20261003T1400Z\n").unwrap();
    let o = fx.watch("30");
    assert_eq!(
        (rc(&o), out(&o)),
        (0, "相談: 見張りはもう居る\n".to_string())
    );
    assert_eq!(
        fs::read_to_string(fx.alive()).unwrap(),
        "pid = 1・始まり = 20261003T1400Z\n"
    );
    let old = SystemTime::now() - Duration::from_secs(120);
    fs::File::options()
        .write(true)
        .open(fx.alive())
        .unwrap()
        .set_modified(old)
        .unwrap();
    let child = fx
        .command(&["--max", "3"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(1500));
    let text = fs::read_to_string(fx.alive()).unwrap();
    assert!(
        text.starts_with(&format!("pid = {}・始まり = ", child.id())),
        "{text}"
    );
    let second = fx.watch("30");
    assert_eq!(out(&second), "相談: 見張りはもう居る\n");
    let first = child.wait_with_output().unwrap();
    assert_eq!(out(&first), format!("相談: 見張りが上限で終わった{TAIL}\n"));
    assert!(!fx.alive().exists());
}

#[test]
fn cwwat_ledger_reread_on_mark() {
    let fx = Fx::new("reread");
    let child = fx
        .command(&["--max", "6"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(1000));
    fx.notes(&[RQ_LINE]);
    let quiet = child.wait_with_output().unwrap();
    assert_eq!(
        out(&quiet),
        format!("相談: 見張りが上限で終わった{TAIL}\n"),
        "印が同じなら読み直さない"
    );
    fx.notes(&[]);
    let child = fx
        .command(&["--max", "20"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(1000));
    fx.notes(&[RQ_LINE]);
    fs::create_dir_all(fx.repo.join(".beads")).unwrap();
    fs::write(fx.repo.join(".beads/issues.jsonl"), "{}\n").unwrap();
    let start = Instant::now();
    let seen = child.wait_with_output().unwrap();
    assert!(
        out(&seen).starts_with("相談: 頼み rq-20261003T1412Z-1 が届いた"),
        "{}",
        out(&seen)
    );
    assert!(start.elapsed() < Duration::from_secs(10));
}

#[test]
fn cwwat_refusals() {
    let fx = Fx::new("refuse");
    for bad in [
        &["--max", "0"][..],
        &["--max", "x"],
        &["extra", "--max", "1"],
        &["--max", "1", "--max", "2"],
    ] {
        let o = fx.command(bad).output().unwrap();
        assert_eq!(rc(&o), 1, "{bad:?}");
    }
    fs::remove_file(fx.root.join("ledger.json")).unwrap();
    assert_eq!(rc(&fx.watch("1")), 2);
    assert!(!fx.alive().exists(), "台帳が読めなければ印を置かない");
    // 台帳を読める形に戻し、鍵の欠けだけの見本にする。
    fx.notes(&[]);
    git(&fx.repo, &["config", "--unset", "tsuzuri.draftsdir"]);
    let o = fx.watch("1");
    assert_eq!(rc(&o), 2);
    let e = String::from_utf8(o.stderr.clone()).unwrap();
    assert!(e.contains("鍵 tsuzuri.draftsdir が無い"), "{e}");
}
