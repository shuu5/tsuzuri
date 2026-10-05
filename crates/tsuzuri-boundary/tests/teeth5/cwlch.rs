//! 窓を起こす口の歯（接頭辞 cwlch_・設計ノート surface-wave27a 行 cs-launch・受入 AC19 の起こし）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に、git init した repo と state dir と起草の置き場と、偽の bd・bdw と、
//! 受けた argv と環境を記録する偽の tmux と偽の claude を置く（本物の tmux と claude は撃たない）。
//! 偽の tmux の記録と偽の claude の記録を、中核の `consult::launch` が同じ材料で組んだ argv と比べる。
//! 口座の置き場の symlink の先を解いて隠すことと、読めない置き場で起こさないことを見る（行 cs-cred-links）。
//! process の印が起こした時の口座の置き場（偽の環境の CLAUDE_CONFIG_DIR）を持つことを見る（行 cs-acct-mark）。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsuzuri_boundary::server::{events, ruling};
use tsuzuri_contract::consult::{FindingId, Form, ProcMark, Via, WindowId};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::launch::{Launch, PLUGIN_VERSION, argv, private_tmp};
use tsuzuri_core::consult::lines::{Event, Line, notice, read};
use tsuzuri_core::consult::quota::REFUSAL;

/// 台帳の写し（根の epic と memo と memo の下の問い・根の notes は歯が足す）。
const LEDGER: &str = r#"[
{"id":"fx-c","title":"根","status":"open","issue_type":"epic","updated_at":"2026-10-03T00:00:00Z","notes":"NOTES"},
{"id":"fx-c.1","title":"控え","status":"open","issue_type":"task","labels":["intake:memo"],"parent":"fx-c","updated_at":"2026-10-03T00:00:00Z"},
{"id":"fx-c.2","title":"問いの題名","status":"open","issue_type":"task","labels":["intake:question"],"parent":"fx-c.1","description":"問いの本文","updated_at":"2026-10-03T00:00:00Z"}
]"#;

const TZW: &str = "/x/tzw";

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(ok.success(), "git {args:?}");
}

/// argv を字 0x1f で区切り、撃たれた回ごとに 1 行を足す殻の字。
fn record(log: &Path) -> String {
    format!(
        "for a in \"$@\"; do printf '%s\\037' \"$a\"; done >> '{0}'\necho >> '{0}'",
        log.display()
    )
}

/// 歯ごとの置き場。
struct Fx {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
    drafts: PathBuf,
}

impl Fx {
    fn new(name: &str) -> Fx {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cwlch")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, state, drafts) = (root.join("repo"), root.join("state"), root.join("drafts"));
        for d in [
            &repo,
            &state.join("fleet"),
            &state.join("pipe"),
            &drafts,
            &root.join("bin"),
        ] {
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
        let fx = Fx {
            root,
            repo,
            state,
            drafts,
        };
        let (bin, r) = (fx.root.join("bin"), fx.root.display().to_string());
        script(&bin.join("bd"), &format!("exec cat '{r}/ledger.json'"));
        script(&bin.join("bdw"), &record(&fx.root.join("bdw.log")));
        let tmux = format!(
            "{}\n[ -f '{r}/tmux.fail' ] && exit 1\ncase \"$1\" in display-message) echo seat-s ;; new-window) echo '@7 4242' ;; esac",
            record(&fx.root.join("tmux.log"))
        );
        script(&bin.join("tmux"), &tmux);
        let claude = format!(
            "for a in \"$@\"; do printf '%s\\037' \"$a\"; done > '{r}/claude.args'\n\
             tr '\\000' '\\n' < /proc/$$/environ > '{r}/claude.environ'\n\
             {{ echo \"id=${{TZ_CONSULT_ID-}}\"; echo \"tmp=${{CLAUDE_CODE_TMPDIR-}}\"; echo \"tmux=${{TMUX-unset}}\"; \
             echo \"pane=${{TMUX_PANE-unset}}\"; echo \"cwd=$(pwd -P)\"; echo \"pid=$$\"; }} > '{r}/claude.env'\n\
             [ -f '{r}/answer' ] && echo '{{}}' > findings/cw1-1.json\necho '{{\"result\":\"ok\"}}'"
        );
        script(&bin.join("claude"), &claude);
        fx.notes("");
        fx
    }

    /// 根の notes を替える。
    fn notes(&self, notes: &str) {
        let text = LEDGER.replace("NOTES", &notes.replace('\n', "\\n"));
        fs::write(self.root.join("ledger.json"), text).unwrap();
    }

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
            .env("TZ_WRAPPER", TZW)
            .env("TMUX", "/tmp/fake-tmux,1,0")
            .env("TMUX_PANE", "%9")
            .env("CLAUDE_CONFIG_DIR", "/cfg/x")
            .env_remove("TZ_CONSULT_ID");
        c
    }

    fn tz(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    /// 窓を用意する（open の引数）。
    fn open(&self, extra: &[&str]) {
        let o = self.tz(&[&["consult", "open"][..], extra].concat());
        assert_eq!(
            o.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&o.stderr)
        );
    }

    fn ws(&self, n: u32) -> PathBuf {
        fs::canonicalize(self.drafts.join(format!("consult-cw{n}"))).unwrap()
    }

    /// 記録の file の撃たれた回ごとの argv。
    fn calls(&self, name: &str) -> Vec<Vec<String>> {
        fs::read_to_string(self.root.join(name))
            .unwrap_or_default()
            .lines()
            .map(|l| {
                l.split('\x1f')
                    .filter(|a| !a.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    }

    /// 偽の bdw が受けた行の字（--append-notes= の後）と置き場。
    fn written(&self) -> Vec<(String, String)> {
        self.calls("bdw.log")
            .into_iter()
            .map(|c| {
                (
                    c[1].clone(),
                    c[2].strip_prefix("--append-notes=").unwrap().to_string(),
                )
            })
            .collect()
    }

    /// 中核が同じ材料で組む材料。
    fn launch(&self, n: u32, form: Form, question: bool) -> Launch {
        let canon = |p: &Path| fs::canonicalize(p).unwrap().display().to_string();
        let ws = self.ws(n);
        let (repo, state) = (canon(&self.repo), canon(&self.state));
        Launch {
            form,
            window: WindowId::new(n).unwrap(),
            workspace: ws.display().to_string(),
            roots: vec![
                repo.clone(),
                format!("{state}/fleet"),
                format!("{state}/pipe"),
            ],
            repo,
            state,
            account_dirs: Vec::new(),
            tz: canon(Path::new(env!("CARGO_BIN_EXE_tz"))),
            uid: fs::metadata(&ws).unwrap().uid().to_string(),
            model: "fable".to_string(),
            effort: "xhigh".to_string(),
            question,
            resume: None,
        }
    }

    fn mark(&self, n: u32, k: u32) -> ProcMark {
        wire::decode(
            &fs::read_to_string(self.ws(n).join(format!(".consult/proc-{k}.json"))).unwrap(),
        )
        .unwrap()
    }
}

fn rc(o: &Output) -> i32 {
    o.status.code().unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}

fn err(o: &Output) -> String {
    String::from_utf8(o.stderr.clone()).unwrap()
}

/// 私用の temp を消す（歯の前と後）。
fn clear_tmp(l: &Launch) {
    let _ = fs::remove_dir_all(private_tmp(&l.workspace));
}

/// 開きの行を読み、欄の結果と撃ち直しと時刻を返す。
fn open_line(text: &str) -> (bool, bool, String) {
    let Some(Line::Open {
        opened, again, at, ..
    }) = read(text)
    else {
        panic!("{text}");
    };
    (opened, again, at)
}

/// 席の環境の番兵（窓に渡さない名と、窓では中核の値に替わる窓の id と私用の temp・行 cs-env-closed）。
const SEAT_CANARIES: [(&str, &str); 7] = [
    ("TERM", "seat-term"),
    ("CLAUDECODE", "1"),
    ("CLAUDE_CODE_MESSAGING_TOKEN", "seat-token"),
    ("SSH_AUTH_SOCK", "/seat/agent"),
    ("USER", "seat"),
    ("TZ_CONSULT_ID", "cw9"),
    ("CLAUDE_CODE_TMPDIR", "/seat/tmp"),
];

/// 席の環境に番兵と、窓に渡す足しの 3 つ（HOME は歯の置き場・SHELL・LANG）を置いた tz の命令。
fn seat(fx: &Fx, args: &[&str]) -> Command {
    let mut c = fx.command(args);
    c.env("HOME", &fx.root)
        .env("SHELL", "/bin/seat-sh")
        .env("LANG", "C.UTF-8")
        .envs(SEAT_CANARIES);
    c
}

/// 偽の claude が受けた環境（名の字の順・名と値）。
fn environ(fx: &Fx) -> Vec<(String, String)> {
    let text = fs::read_to_string(fx.root.join("claude.environ")).unwrap();
    let mut pairs: Vec<(String, String)> = text
        .lines()
        .map(|l| l.split_once('=').unwrap())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    pairs.sort();
    pairs
}

/// 偽の claude が受けた環境が `want`（名の字の順）と同じ（落ちた時に席の値を出さないよう、名を先に比べる）。
fn assert_environ(fx: &Fx, want: &[(String, String)]) {
    let got = environ(fx);
    let names = |xs: &[(String, String)]| xs.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>();
    assert_eq!(names(&got), names(want));
    assert_eq!(got, want);
}

/// 席の PATH（偽の道具の置き場を頭に置いた字）。
fn seat_path(fx: &Fx) -> String {
    format!(
        "{}:{}",
        fx.root.join("bin").display(),
        std::env::var("PATH").unwrap()
    )
}

/// `seat` の命令で起こした窓 cw1 が持つ閉じた列の名と値（`TALK_ENV` と `BASE_ENV` の順）。
fn closed(fx: &Fx, l: &Launch) -> Vec<(String, String)> {
    [
        ("CLAUDE_CONFIG_DIR", "/cfg/x".to_string()),
        ("PATH", seat_path(fx)),
        ("TZ_CONSULT_ID", "cw1".to_string()),
        ("CLAUDE_CODE_TMPDIR", private_tmp(&l.workspace)),
        ("HOME", fx.root.display().to_string()),
        ("SHELL", "/bin/seat-sh".to_string()),
        ("LANG", "C.UTF-8".to_string()),
    ]
    .map(|(k, v)| (k.to_string(), v))
    .to_vec()
}

/// 話す窓の new-window の -e の列と、claude を env -S で包む命令の頭（-P から claude まで）。
fn talk_env_args(fx: &Fx, l: &Launch) -> Vec<String> {
    let mut a: Vec<String> = closed(fx, l)
        .into_iter()
        .flat_map(|(k, v)| ["-e".to_string(), format!("{k}={v}")])
        .collect();
    let keep = "-i CLAUDE_CONFIG_DIR=${CLAUDE_CONFIG_DIR} PATH=${PATH} TZ_CONSULT_ID=${TZ_CONSULT_ID} CLAUDE_CODE_TMPDIR=${CLAUDE_CODE_TMPDIR} HOME=${HOME} SHELL=${SHELL} LANG=${LANG} TERM=${TERM} TERM_PROGRAM=${TERM_PROGRAM} TERM_PROGRAM_VERSION=${TERM_PROGRAM_VERSION} TMUX=${TMUX} TMUX_PANE=${TMUX_PANE}";
    let tail = [
        "-P",
        "-F",
        "#{window_id} #{pane_pid}",
        "--",
        "env",
        "-S",
        keep,
        "claude",
    ];
    a.extend(tail.map(String::from));
    a
}

/// 話す窓の 1 番目の process の印（偽の tmux の pane の pid と窓・偽の環境の口座の置き場）。
fn talk_mark(at: String) -> ProcMark {
    ProcMark {
        k: 1,
        form: Form::Talk,
        pid: 4242,
        at,
        again: false,
        tmux_window: Some("@7".into()),
        account: Some("/cfg/x".into()),
    }
}

#[test]
fn cwlch_talk_argv_and_env() {
    let fx = Fx::new("talk");
    fx.open(&["--by", "seat"]);
    let l = fx.launch(1, Form::Talk, false);
    clear_tmp(&l);
    let o = seat(&fx, &["consult", "launch", "cw1"]).output().unwrap();
    assert_eq!(rc(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "窓 cw1 を開いた（tmux の窓 consult-cw1）\n");
    let tmp = private_tmp(&l.workspace);
    let mut want: Vec<String> = [
        "new-window",
        "-d",
        "-t",
        "seat-s:",
        "-n",
        "consult-cw1",
        "-c",
    ]
    .map(String::from)
    .to_vec();
    want.push(l.workspace.clone());
    want.extend(talk_env_args(&fx, &l));
    want.extend(argv(&l));
    let display: Vec<String> = ["display-message", "-p", "-t", "%9", "#{session_name}"]
        .map(String::from)
        .to_vec();
    assert_eq!(fx.calls("tmux.log"), [display, want]);
    let mark = fx.mark(1, 1);
    let at = mark.at.clone();
    assert_eq!(mark, talk_mark(at.clone()));
    let meta = fs::symlink_metadata(&tmp).unwrap();
    assert!(meta.is_dir() && meta.mode() & 0o777 == 0o700);
    let line = format!(
        "相談の開き = cw1・形 = 話す・題 = 題なし・起こし手 = 席・model = fable・念入りさ = xhigh・結果 = 開いた・時刻 = {at}"
    );
    assert_eq!(fx.written(), [("fx-c".to_string(), line)]);
    assert!(!fx.root.join("claude.args").exists());
    clear_tmp(&l);
}

/// 話す窓の tmux の new-window の -- の後の命令を、tmux の server の環境（番兵）と tmux の置く TERM と TMUX と -e の値の上で
/// 撃つと、偽の claude は閉じた列の名と tmux の TERM だけを持ち、argv は中核の argv と同じ（行 cs-env-closed）。
#[test]
fn cwlch_talk_env_runs_closed() {
    let fx = Fx::new("talkenv");
    fx.open(&["--by", "seat"]);
    let l = fx.launch(1, Form::Talk, false);
    clear_tmp(&l);
    let o = seat(&fx, &["consult", "launch", "cw1"]).output().unwrap();
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let call = fx.calls("tmux.log").pop().unwrap();
    let at = call.iter().position(|a| a == "--").unwrap();
    let given: Vec<(&str, &str)> = call[..at]
        .windows(2)
        .filter(|w| w[0] == "-e")
        .map(|w| w[1].split_once('=').unwrap())
        .collect();
    let server = [
        ("SSH_AUTH_SOCK", "/srv/agent"),
        ("DBUS_SESSION_BUS_ADDRESS", "unix:path=/srv/bus"),
        ("SERVER_CANARY", "srv"),
        ("USER", "srv"),
        ("HOME", "/srv/home"),
        ("TERM", "tmux-256color"),
        ("TERM_PROGRAM", "tmux"),
        ("TERM_PROGRAM_VERSION", "3.6b"),
        ("TMUX", "/tmp/fake-tmux,1,0"),
        ("TMUX_PANE", "%3"),
    ];
    let status = Command::new(&call[at + 1])
        .args(&call[at + 2..])
        .current_dir(&l.workspace)
        .env_clear()
        .envs(server)
        .envs(given)
        .status()
        .unwrap();
    assert!(status.success());
    let mut want = closed(&fx, &l);
    want.extend(
        server[5..]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string())),
    );
    want.sort();
    assert_environ(&fx, &want);
    assert_eq!(fx.calls("claude.args"), [argv(&l)]);
    clear_tmp(&l);
}

/// 偽の claude が受けた環境（窓の id・私用の temp・tmux の 2 つが無い・cwd が作業場）と pid。
fn ask_env(fx: &Fx, l: &Launch) -> String {
    let env = fs::read_to_string(fx.root.join("claude.env")).unwrap();
    let pid = env
        .lines()
        .find_map(|l| l.strip_prefix("pid="))
        .unwrap()
        .to_string();
    let tmp = private_tmp(&l.workspace);
    let want = format!(
        "id=cw1\ntmp={tmp}\ntmux=unset\npane=unset\ncwd={}\npid={pid}\n",
        l.workspace
    );
    assert_eq!(env, want);
    pid
}

/// 偽の bdw が受けた開きの行が根の持ち主のチャットの行で、結果と撃ち直しが `want` の順。
fn assert_chat_opens(fx: &Fx, want: &[(bool, bool)]) {
    let written = fx.written();
    assert!(written.iter().all(|(home, _)| home == "fx-c"));
    let chat = "・起こし手 = 持ち主のチャット・発話 = 20261003T1410Z・";
    assert!(written.iter().all(|(_, t)| t.contains(chat)), "{written:?}");
    let got: Vec<(bool, bool)> = written
        .iter()
        .map(|(_, t)| open_line(t))
        .map(|(o, a, _)| (o, a))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn cwlch_ask_child_env_and_lines() {
    let fx = Fx::new("ask");
    fx.open(&[
        "--by",
        "chat",
        "--said",
        "20261003T1410Z",
        "--form",
        "ask",
        "--topic",
        "fx-c.2",
    ]);
    let l = fx.launch(1, Form::Ask, true);
    clear_tmp(&l);
    fs::write(fx.root.join("answer"), "").unwrap();
    let o = fx.tz(&["consult", "launch", "cw1"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let f = FindingId::new(WindowId::new(1).unwrap(), 1).unwrap();
    let done = Event::Finding {
        id: f,
        via: Via::Done,
    };
    assert_eq!(out(&o), notice(&done, TZW) + "\n");
    assert_eq!(fx.calls("claude.args"), [argv(&l)]);
    let pid = ask_env(&fx, &l);
    let answer = fs::read_to_string(fx.ws(1).join(".consult/ask-1.json")).unwrap();
    assert_eq!(answer, "{\"result\":\"ok\"}\n");
    let m = fx.mark(1, 1);
    assert_eq!(
        (m.form, m.pid.to_string(), m.again, m.tmux_window),
        (Form::Ask, pid, false, None)
    );
    assert_eq!(m.account.as_deref(), Some("/cfg/x"), "問う窓の印の口座");
    fs::remove_file(fx.root.join("answer")).unwrap();
    let o = fx.tz(&["consult", "launch", "cw1", "--again"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let stalled = Event::Stalled(WindowId::new(1).unwrap());
    assert_eq!(out(&o), notice(&stalled, TZW) + "\n");
    assert!(fx.mark(1, 2).again);
    assert_chat_opens(&fx, &[(true, false), (true, true)]);
    assert!(fx.calls("tmux.log").is_empty());
    clear_tmp(&l);
}

/// 問う窓の子は、席の環境の番兵を持たず、閉じた列の名だけを持ち（窓の id と私用の temp は中核の値）、席に無い名は
/// 空の値でも置かない（行 cs-env-closed）。
#[test]
fn cwlch_ask_env_closed() {
    let fx = Fx::new("askenv");
    fx.open(&["--by", "chat", "--said", "20261003T1410Z", "--form", "ask"]);
    let l = fx.launch(1, Form::Ask, false);
    clear_tmp(&l);
    let o = seat(&fx, &["consult", "launch", "cw1"]).output().unwrap();
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let mut want = closed(&fx, &l);
    want.sort();
    assert_environ(&fx, &want);
    let o = seat(&fx, &["consult", "launch", "cw1", "--again"])
        .env_remove("SHELL")
        .env_remove("LANG")
        .output()
        .unwrap();
    assert_eq!(rc(&o), 0, "{}", err(&o));
    want.retain(|(k, _)| k != "SHELL" && k != "LANG");
    assert_environ(&fx, &want);
    clear_tmp(&l);
}

/// 起こす前の断り（版のずれ・窓が無い・閉じた窓）は行を書かず tmux を撃たない。
fn refusals_before(fx: &Fx) {
    let launch = |c: &mut Command| c.output().unwrap();
    let o = launch(
        fx.command(&["consult", "launch", "cw1"])
            .env("TZ_PLUGIN_VERSION", "0.1.0"),
    );
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("TZ_PLUGIN_VERSION"));
    assert_eq!(rc(&fx.tz(&["consult", "launch", "cw2"])), 1);
    assert_eq!(rc(&fx.tz(&["consult", "launch"])), 1);
    fx.notes("相談の閉じ = cw1・起こし手 = 席・所見 = 0・時刻 = 20261003T1800Z");
    let o = fx.tz(&["consult", "launch", "cw1"]);
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("閉じた"), "{}", err(&o));
    assert!(fx.written().is_empty() && fx.calls("tmux.log").is_empty());
    fx.notes("");
}

#[test]
fn cwlch_refusals_without_launch() {
    let fx = Fx::new("refuse");
    fx.open(&["--by", "seat"]);
    let l = fx.launch(1, Form::Talk, false);
    clear_tmp(&l);
    refusals_before(&fx);
    fx.open(&["--by", "seat", "--model=--plugin-dir"]);
    let o = fx.tz(&["consult", "launch", "cw2"]);
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("検めの欠け"), "{}", err(&o));
    // 閉じの行は窓 cw2 のもの（窓 cw1 の次の 2 つの断りは TMUX_PANE と tmux の落ちだけ）。
    fx.notes("相談の閉じ = cw2・起こし手 = 席・所見 = 0・時刻 = 20261003T1800Z");
    let o = fx
        .command(&["consult", "launch", "cw1"])
        .env_remove("TMUX_PANE")
        .output()
        .unwrap();
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("TMUX_PANE"));
    fs::write(fx.root.join("tmux.fail"), "").unwrap();
    assert_eq!(rc(&fx.tz(&["consult", "launch", "cw1"])), 1);
    let heads: Vec<(String, bool)> = fx
        .written()
        .iter()
        .map(|(_, t)| (t.split('・').next().unwrap().to_string(), open_line(t).0))
        .collect();
    let want = [
        ("相談の開き = cw2", false),
        ("相談の開き = cw1", false),
        ("相談の開き = cw1", false),
    ];
    assert_eq!(heads, want.map(|(a, b)| (a.to_string(), b)));
    assert!(
        fx.calls("tmux.log")
            .iter()
            .all(|c| c[0] == "display-message")
    );
    assert!(!fx.ws(1).join(".consult/proc-1.json").exists());
    clear_tmp(&l);
    clear_tmp(&fx.launch(2, Form::Talk, false));
}

/// 窓 cw<n> の作業場に process の印 proc-1 を置く（pid だけを替えて生きている印と死んだ印を作る）。
fn put_mark(fx: &Fx, n: u32, pid: u32) {
    let dir = fx.drafts.join(format!("consult-cw{n}/.consult"));
    fs::create_dir_all(&dir).unwrap();
    let mark = ProcMark {
        k: 1,
        form: Form::Ask,
        pid,
        at: "20200101T0000Z".into(),
        again: false,
        tmux_window: None,
        account: None,
    };
    fs::write(dir.join("proc-1.json"), wire::encode(&mark).unwrap()).unwrap();
}

#[test]
fn cwlch_quota_r38() {
    let fx = Fx::new("quota");
    let today = ruling::minute(events::now());
    let opened = |n: u32, at: &str| {
        format!(
            "相談の開き = cw{n}・形 = 問う・題 = 題なし・起こし手 = 席・model = fable・念入りさ = xhigh・結果 = 開いた・時刻 = {at}"
        )
    };
    fx.notes(&[opened(5, &today), opened(6, &today), opened(7, &today)].join("\n"));
    fx.open(&["--by", "seat", "--form", "ask"]);
    let o = fx.tz(&["consult", "launch", "cw1"]);
    assert_eq!(rc(&o), 1);
    assert_eq!(err(&o), format!("tz consult launch: {REFUSAL}\n"));
    assert!(fx.written().is_empty() && !fx.root.join("claude.args").exists());
    let l = fx.launch(1, Form::Ask, false);
    clear_tmp(&l);
    let o = fx.tz(&["consult", "launch", "cw1", "--again"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    fx.open(&["--by", "chat", "--said", "20261003T1410Z", "--form", "ask"]);
    assert_eq!(rc(&fx.tz(&["consult", "launch", "cw2"])), 0);
    // 今日の 3 本の下でも、席の話す窓は数えずに起こす。
    fx.open(&["--by", "seat"]);
    assert_eq!(rc(&fx.tz(&["consult", "launch", "cw3"])), 0);
    fx.notes(&opened(9, "20200101T0000Z"));
    // 前の日の席の問う窓 cw9 は、process の印が死んでいれば数えずに起こし、生きていれば断る。
    put_mark(&fx, 9, u32::MAX);
    assert_eq!(rc(&fx.tz(&["consult", "launch", "cw1"])), 0);
    put_mark(&fx, 9, std::process::id());
    let o = fx.tz(&["consult", "launch", "cw1"]);
    assert_eq!(
        (rc(&o), err(&o)),
        (1, format!("tz consult launch: {REFUSAL}\n"))
    );
    assert_eq!(fx.written().len(), 4);
    clear_tmp(&l);
    clear_tmp(&fx.launch(2, Form::Ask, false));
    clear_tmp(&fx.launch(3, Form::Talk, false));
}

#[test]
fn cwlch_dry_run() {
    let fx = Fx::new("dry");
    fx.open(&["--by", "seat"]);
    let l = fx.launch(1, Form::Talk, false);
    clear_tmp(&l);
    // 台帳の写しを消す（--dry-run が台帳を読めば bd が落ちて rc 2 になる）。
    fs::remove_file(fx.root.join("ledger.json")).unwrap();
    let o = fx.tz(&["consult", "launch", "cw1", "--dry-run"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let want: Vec<String> = std::iter::once("claude".to_string())
        .chain(argv(&l))
        .collect();
    assert_eq!(out(&o).lines().collect::<Vec<_>>(), want);
    assert!(fx.written().is_empty() && fx.calls("tmux.log").is_empty());
    assert!(!Path::new(&private_tmp(&l.workspace)).exists());
    assert!(!fx.ws(1).join(".consult/proc-1.json").exists());
}

#[test]
fn cwlch_private_tmp_guard() {
    let fx = Fx::new("tmp");
    fx.open(&["--by", "seat"]);
    let l = fx.launch(1, Form::Talk, false);
    let tmp = private_tmp(&l.workspace);
    clear_tmp(&l);
    assert!(tmp.len() <= 30 && tmp.starts_with("/tmp/tzc-"));
    fs::create_dir(&tmp).unwrap();
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755)).unwrap();
    let o = fx.tz(&["consult", "launch", "cw1"]);
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("0700"), "{}", err(&o));
    assert!(fx.calls("tmux.log").is_empty());
    assert!(!open_line(&fx.written()[0].1).0);
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(rc(&fx.tz(&["consult", "launch", "cw1"])), 0);
    assert!(open_line(&fx.written()[1].1).0);
    clear_tmp(&l);
}

/// state dir の accounts に口座の置き場の形を置く（実体の dir・ふつうの dir・解けない link・相対の link・file・退役の link）。
fn put_accounts(fx: &Fx) -> Vec<String> {
    let (accounts, real) = (fx.state.join("accounts"), fx.root.join("real"));
    for d in [
        accounts.join(".retired"),
        accounts.join("b"),
        real.join("a"),
        real.join("d"),
        real.join("old"),
    ] {
        fs::create_dir_all(d).unwrap();
    }
    let gone = fx.root.join("gone");
    std::os::unix::fs::symlink(real.join("a"), accounts.join("a")).unwrap();
    std::os::unix::fs::symlink(&gone, accounts.join("c")).unwrap();
    std::os::unix::fs::symlink("../../real/d", accounts.join("d")).unwrap();
    std::os::unix::fs::symlink(real.join("old"), accounts.join(".retired/old.1")).unwrap();
    std::os::unix::fs::symlink(real.join("a"), accounts.join(".retired/a.2")).unwrap();
    fs::write(accounts.join("x.txt"), "x").unwrap();
    let canon = |p: PathBuf| fs::canonicalize(p).unwrap().display().to_string();
    let mut want = vec![
        canon(real.join("a")),
        gone.display().to_string(),
        canon(real.join("d")),
        canon(real.join("old")),
    ];
    want.sort();
    want
}

#[test]
fn cwlch_account_links_are_hidden() {
    let fx = Fx::new("links");
    fx.open(&["--by", "seat"]);
    let want = put_accounts(&fx);
    let mut l = fx.launch(1, Form::Talk, false);
    l.account_dirs = want.clone();
    assert_eq!(want.len(), 4);
    let o = fx.tz(&["consult", "launch", "cw1", "--dry-run"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let lines: Vec<String> = out(&o).lines().map(String::from).collect();
    let got: Vec<String> = std::iter::once("claude".to_string())
        .chain(argv(&l))
        .collect();
    assert_eq!(lines, got);
    let settings = lines
        .iter()
        .skip_while(|a| *a != "--settings")
        .nth(1)
        .unwrap();
    for d in &want {
        assert!(settings.contains(&format!("\"path\":\"{d}\"")), "{d}");
        assert!(settings.contains(&format!("\"Read(/{d}/**)\"")), "{d}");
    }
    assert!(
        !settings.contains("accounts/b\""),
        "ふつうの dir は accounts の覆いの中"
    );
    assert!(fx.written().is_empty() && fx.calls("tmux.log").is_empty());
}

#[test]
fn cwlch_unreadable_accounts_refuse() {
    let fx = Fx::new("noacct");
    fx.open(&["--by", "seat"]);
    put_accounts(&fx);
    let l = fx.launch(1, Form::Talk, false);
    clear_tmp(&l);
    for dir in [
        fx.state.join("accounts/.retired"),
        fx.state.join("accounts"),
    ] {
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o000)).unwrap();
        let o = fx.tz(&["consult", "launch", "cw1"]);
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        let shown = fs::canonicalize(&dir).unwrap().display().to_string();
        assert_eq!(rc(&o), 2, "{}", err(&o));
        assert!(
            err(&o).starts_with(&format!(
                "tz consult launch: 口座の置き場 {shown} が読めない: "
            )),
            "{}",
            err(&o)
        );
        assert!(
            err(&o).ends_with("（読める形に直してから起こし直す）\n"),
            "{}",
            err(&o)
        );
        assert!(fx.written().is_empty() && fx.calls("tmux.log").is_empty());
        assert!(!Path::new(&private_tmp(&l.workspace)).exists());
    }
    assert_eq!(rc(&fx.tz(&["consult", "launch", "cw1"])), 0);
    clear_tmp(&l);
}
