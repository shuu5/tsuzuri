//! 撃ち直しで会話を続ける口の歯（接頭辞 cwres_・設計ノート surface-wave29b 行 cs-resume・判断の記録 ADR-55 決定 (1)(3)）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に、git init した repo と state dir と起草の置き場と口座の置き場と、偽の bd・bdw と
//! 受けた argv を記録する偽の tmux を置く（本物の tmux と claude は撃たず、本物の口座の置き場は読まない）。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsuzuri_boundary::consult::launch::STAMPS_MAX;
use tsuzuri_contract::consult::{Form, ProcMark, Stamp, StampEvent};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::launch::{PLUGIN_VERSION, private_tmp};
use tsuzuri_core::consult::lines::{Line, read};
use tsuzuri_core::consult::trust::trusted;

const SID: &str = "bf1f3252-2ac9-42ce-bfb7-b5897fa42308";
const SID2: &str = "772c5b4c-0000-4000-8000-00000000abcd";

/// 束に問いの無い話す窓の最初の指示。
const PROMPT: &str = "bundle/brief.md を読み、notes.md と findings/ が在れば続きから始めて";

/// 在り得ない pid（pid の上限より大きい・偽の tmux の pane の pid に使う）。
const DEAD: u32 = 4_294_967_294;

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// 歯ごとの置き場（口座の置き場 acct は dir だけを作る）。
struct Fx {
    root: PathBuf,
    repo: PathBuf,
    acct: PathBuf,
}

impl Fx {
    fn new(name: &str) -> Fx {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cwres")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, acct) = (root.join("repo"), root.join("acct"));
        for d in ["repo", "state/fleet", "drafts", "acct", "bin"] {
            fs::create_dir_all(root.join(d)).unwrap();
        }
        let r = root.display().to_string();
        for args in [
            vec!["init", "-q"],
            vec!["config", "scribe2.statedir", &format!("{r}/state")],
            vec!["config", "tsuzuri.draftsdir", &format!("{r}/drafts")],
        ] {
            let ok = Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args(args)
                .status()
                .unwrap();
            assert!(ok.success());
        }
        let ledger = r#"[{"id":"fx-c","title":"根","status":"open","issue_type":"epic","updated_at":"2026-10-03T00:00:00Z"}]"#;
        fs::write(root.join("ledger.json"), ledger).unwrap();
        let rec = |log: &str| {
            format!(
                "for a in \"$@\"; do printf '%s\\037' \"$a\"; done >> '{r}/{log}'\necho >> '{r}/{log}'"
            )
        };
        script(&root.join("bin/bd"), &format!("exec cat '{r}/ledger.json'"));
        script(&root.join("bin/bdw"), &rec("bdw.log"));
        let tmux = format!(
            "{}\ncase \"$1\" in display-message) echo seat-s ;; new-window) echo '@8 {DEAD}' ;; esac",
            rec("tmux.log")
        );
        script(&root.join("bin/tmux"), &tmux);
        Fx { root, repo, acct }
    }

    /// 口座の置き場を環境に置いた tz の命令。
    fn command(&self, args: &[&str]) -> Command {
        let path = format!(
            "{}:{}",
            self.root.join("bin").display(),
            std::env::var("PATH").unwrap()
        );
        let mut c = Command::new(env!("CARGO_BIN_EXE_tz"));
        c.args(args)
            .current_dir(&self.repo)
            .env("PATH", path)
            .env("TZ_PLUGIN_VERSION", PLUGIN_VERSION)
            .env("TMUX_PANE", "%9")
            .env("CLAUDE_CONFIG_DIR", &self.acct)
            .env_remove("TZ_CONSULT_ID");
        c
    }

    fn tz(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    /// 窓 cw<n> を用意し、process の印 proc-1（pid・話す窓は tmux の窓 @7）を置く。
    fn window(&self, n: u32, form: Form, pid: u32) -> PathBuf {
        let word = if form == Form::Ask { "ask" } else { "talk" };
        let o = self.tz(&["consult", "open", "--by", "seat", "--form", word]);
        assert_eq!(o.status.code(), Some(0));
        let ws = fs::canonicalize(self.root.join(format!("drafts/consult-cw{n}"))).unwrap();
        let tmux_window = (form == Form::Talk).then(|| "@7".to_string());
        let mark = ProcMark {
            k: 1,
            form,
            pid,
            at: "20261005T0000Z".into(),
            again: false,
            tmux_window,
            account: None,
        };
        let text = wire::encode(&mark).unwrap();
        fs::write(ws.join(".consult/proc-1.json"), text).unwrap();
        let _ = fs::remove_dir_all(private_tmp(&ws.display().to_string()));
        ws
    }

    /// 撃たれた回ごとの argv（記録の file の名）。
    fn calls(&self, log: &str) -> Vec<Vec<String>> {
        let text = fs::read_to_string(self.root.join(log)).unwrap_or_default();
        text.lines()
            .map(|l| {
                l.split('\x1f')
                    .filter(|a| !a.is_empty())
                    .map(String::from)
                    .collect()
            })
            .collect()
    }

    /// 偽の tmux の new-window の argv の末の 2 つ。
    fn opened(&self) -> Vec<Vec<String>> {
        let calls = self
            .calls("tmux.log")
            .into_iter()
            .filter(|c| c[0] == "new-window");
        calls.map(|c| c[c.len() - 2..].to_vec()).collect()
    }

    /// 偽の bdw が受けた開きの行の結果と撃ち直し。
    fn opens(&self) -> Vec<(bool, bool)> {
        let lines = self
            .calls("bdw.log")
            .into_iter()
            .filter_map(|c| read(c[2].strip_prefix("--append-notes=")?));
        lines
            .filter_map(|l| {
                if let Line::Open { opened, again, .. } = l {
                    Some((opened, again))
                } else {
                    None
                }
            })
            .collect()
    }

    fn trust(&self) -> PathBuf {
        self.acct.join(".claude.json")
    }
}

/// 作業場の会話の印の file に、会話の始まり・持ち主の入力・turn の終わりの 3 行を足す。
fn stamps(ws: &Path, sid: &str) {
    let path = ws.join(".consult/stamps.jsonl");
    let mut text = fs::read_to_string(&path).unwrap_or_default();
    for (at, event) in (1_000..).zip([StampEvent::Start, StampEvent::Prompt, StampEvent::Stop]) {
        let source = (event == StampEvent::Start).then(|| "startup".to_string());
        let stamp = Stamp {
            at,
            event,
            sid: sid.to_string(),
            source,
        };
        text.push_str(&(wire::encode(&stamp).unwrap() + "\n"));
    }
    fs::write(path, text).unwrap();
}

fn rc_err(o: &Output) -> (i32, String) {
    let err = String::from_utf8_lossy(&o.stderr).into_owned();
    (o.status.code().unwrap(), err)
}

/// 話す窓の --again は会話の印の最後の id（消し直しの後の新しい id）を続きの旗で渡して最初の指示を置かず、起こす前に口座の置き場の
/// 設定 file に作業場の行の信頼の印を置き（中核の trusted の字）、撃ち直しの印と開きの行を書く。環境に口座の置き場が無ければ、
/// 起こさず行も書かずに rc 1 で断る。会話の印か途中の段が symlink・印が fifo か上限を越える時は、辿らず開かずに rc 2 で断る。
#[test]
fn cwres_again_continues_the_last_conversation() {
    let fx = Fx::new("again");
    let ws = fx.window(1, Form::Talk, DEAD);
    stamps(&ws, SID2);
    stamps(&ws, SID);
    let refused = |odd: &str| {
        let (rc, err) = rc_err(&fx.tz(&["consult", "launch", "cw1", "--again"]));
        assert!(rc == 2 && err.contains("会話の印"), "{odd}: {err}");
    };
    let dot = ws.join(".consult");
    let (real, kept) = (dot.join("stamps.jsonl"), ws.join("kept"));
    fs::rename(&dot, ws.join("dot")).unwrap();
    std::os::unix::fs::symlink(ws.join("dot"), &dot).unwrap();
    refused("途中の段の symlink");
    fs::remove_file(&dot).unwrap();
    fs::rename(ws.join("dot"), &dot).unwrap();
    fs::rename(&real, &kept).unwrap();
    std::os::unix::fs::symlink(&kept, &real).unwrap();
    refused("symlink");
    fs::remove_file(&real).unwrap();
    fs::write(&real, vec![b'\n'; STAMPS_MAX as usize + 1]).unwrap();
    refused("上限を越える");
    fs::remove_file(&real).unwrap();
    let made = Command::new("mkfifo").arg(&real).status().unwrap();
    assert!(made.success());
    let f = real.clone();
    let writer = std::thread::spawn(move || fs::OpenOptions::new().write(true).open(f));
    refused("fifo");
    assert!(!writer.is_finished(), "fifo を開いた");
    fs::remove_file(&real).unwrap();
    fs::rename(&kept, &real).unwrap();
    let mut bare = fx.command(&["consult", "launch", "cw1", "--again"]);
    let (rc, err) = rc_err(&bare.env_remove("CLAUDE_CONFIG_DIR").output().unwrap());
    assert!(rc == 1 && err.contains("CLAUDE_CONFIG_DIR が無い"), "{err}");
    assert!(fx.opened().is_empty() && fx.opens().is_empty());
    let before = r#"{"projects":{"/other":{"hasTrustDialogAccepted":false}}}"#;
    fs::write(fx.trust(), before).unwrap();
    let (rc, err) = rc_err(&fx.tz(&["consult", "launch", "cw1", "--again"]));
    assert_eq!(rc, 0, "{err}");
    assert_eq!(fx.opened(), [["--resume", SID]]);
    let want = trusted(Some(before), &ws.display().to_string()).unwrap();
    assert_eq!(fs::read_to_string(fx.trust()).ok(), want);
    let m: ProcMark =
        wire::decode(&fs::read_to_string(ws.join(".consult/proc-2.json")).unwrap()).unwrap();
    assert!(m.again && m.account.as_deref() == fx.acct.to_str());
    assert_eq!(fx.opens(), [(true, true)]);
}

/// 会話の印を持たない話す窓は、席が --again と一緒に名指した id で続け、口座の置き場に信頼の印の file を作る。--again の無い
/// 名指し・uuid の形でない名指し・問う窓への名指し・印を持つ窓への名指しは、起こさず行も書かず信頼の印も置かずに rc 1 で断る。
#[test]
fn cwres_session_names_an_unstamped_window() {
    let fx = Fx::new("session");
    let ws = fx.window(1, Form::Talk, DEAD);
    fx.window(2, Form::Ask, DEAD);
    let refusals = [
        ("cw1", &["--session", SID][..], "--again と一緒"),
        (
            "cw1",
            &["--again", "--session", "bf1f3252-2ac9"][..],
            "会話の id の形（uuid）でない",
        ),
        (
            "cw2",
            &["--again", "--session", SID][..],
            "問う窓は会話を続けない",
        ),
    ];
    let shoot =
        |id: &str, args: &[&str]| rc_err(&fx.tz(&[&["consult", "launch", id][..], args].concat()));
    for (id, args, word) in refusals {
        let (rc, err) = shoot(id, args);
        assert!(rc == 1 && err.contains(word), "{args:?}: {err}");
    }
    assert!(fx.opened().is_empty() && fx.opens().is_empty() && !fx.trust().exists());
    assert_eq!(shoot("cw1", &["--again", "--session", SID]).0, 0);
    assert_eq!(fx.opened(), [["--resume", SID]]);
    assert_eq!(
        fs::read_to_string(fx.trust()).ok(),
        trusted(None, &ws.display().to_string()).unwrap()
    );
    stamps(&ws, SID2);
    let (rc, err) = shoot("cw1", &["--again", "--session", SID]);
    assert!(rc == 1 && err.contains("会話の印を持つ"), "{err}");
    assert_eq!(fx.opens(), [(true, true)]);
}

/// 前の process が生きている話す窓の --again は、起こさず行も書かず信頼の印も置かずに rc 1 で断る。前の process が無く会話の印も
/// 無い窓の --again は、続きの旗を置かずに最初の指示で起こし、信頼の印の file を作らない。
#[test]
fn cwres_live_window_refuses_again() {
    let fx = Fx::new("live");
    let ws = fx.window(1, Form::Talk, std::process::id());
    stamps(&ws, SID);
    let (rc, err) = rc_err(&fx.tz(&["consult", "launch", "cw1", "--again"]));
    assert!(
        rc == 1 && err.contains("前の process が生きている"),
        "{err}"
    );
    assert!(fx.opened().is_empty() && fx.opens().is_empty() && !fx.trust().exists());
    fx.window(2, Form::Talk, DEAD);
    assert_eq!(
        rc_err(&fx.tz(&["consult", "launch", "cw2", "--again"])).0,
        0
    );
    let call = fx.opened().pop().unwrap();
    assert_eq!(call[1], PROMPT);
    assert!(call[0] != "--resume" && !fx.trust().exists());
}
