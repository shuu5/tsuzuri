//! 相談の窓の命令の骨と open と bundle の歯（接頭辞 cwopn_・設計ノート surface-wave27a 行 cs-open）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）に、git init した repo（鍵 scribe2.statedir と tsuzuri.draftsdir）と
//! state dir と起草の置き場と、偽の bd（置き場の ledger.json を出す・無ければ rc 1）と偽の bdw（argv を記録する）を置き、
//! tz を偽の program の dir を PATH の頭に足して撃つ。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsuzuri_boundary::consult::{self, bundle};
use tsuzuri_boundary::server::{events, ruling};
use tsuzuri_contract::consult::{
    FindingId, Form, ProcMark, RequestId, Starter, Via, WindowFile, WindowId,
};
use tsuzuri_contract::ledger::fnv1a64;
use tsuzuri_contract::wire;
use tsuzuri_core::consult::launch::{PLUGIN_VERSION, brief};
use tsuzuri_core::consult::lines::{Line, Subject, read};

/// 台帳の写し（根の epic・memo・memo の下の問い・頼みの行を持つ memo）。
const LEDGER: &str = r#"[
{"id":"fx-c","title":"根","status":"open","issue_type":"epic","updated_at":"2026-10-03T00:00:00Z"},
{"id":"fx-c.1","title":"控え","status":"open","issue_type":"task","labels":["intake:memo"],"parent":"fx-c","updated_at":"2026-10-03T00:00:00Z","notes":"相談の頼み = rq-20261003T1412Z-1・題 = fx-c.2・形 = 問う・model = opus・起こし手 = 持ち主の button・時刻 = 20261003T1412Z"},
{"id":"fx-c.2","title":"問いの題名","status":"open","issue_type":"task","labels":["intake:question"],"parent":"fx-c.1","description":"問いの本文","updated_at":"2026-10-03T00:00:00Z"}
]"#;

const RQ: &str = "rq-20261003T1412Z-1";

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

/// 歯ごとの置き場。
struct Fx {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
    drafts: PathBuf,
}

impl Fx {
    /// 鍵を 2 つとも置いた置き場（`keys` が偽なら鍵を置かない）。
    fn new(name: &str, keys: bool) -> Fx {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cwopn")
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
        let fx = Fx {
            root,
            repo,
            state,
            drafts,
        };
        if keys {
            fx.key("scribe2.statedir", &fx.state);
            fx.key("tsuzuri.draftsdir", &fx.drafts);
        }
        let (bin, log) = (fx.root.join("bin"), fx.root.join("bdw.log"));
        script(
            &bin.join("bd"),
            &format!(
                "echo \"$*\" >> '{}'\n[ -f '{}' ] || exit 1\nexec cat '{}'",
                fx.root.join("bd.log").display(),
                fx.root.join("ledger.json").display(),
                fx.root.join("ledger.json").display()
            ),
        );
        script(
            &bin.join("bdw"),
            &format!(
                "for a in \"$@\"; do printf '%s\\037' \"$a\"; done >> '{}'\necho >> '{}'",
                log.display(),
                log.display()
            ),
        );
        fx.ledger(LEDGER);
        fx
    }

    fn key(&self, name: &str, value: &Path) {
        git(&self.repo, &["config", name, &value.display().to_string()]);
    }

    fn ledger(&self, text: &str) {
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
            .env_remove("TZ_CONSULT_ID");
        c
    }

    fn tz(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    /// 偽の bdw が受けた argv の列（撃たれた回の順）。
    fn writes(&self) -> Vec<Vec<String>> {
        fs::read_to_string(self.root.join("bdw.log"))
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

    fn ws(&self, n: u32) -> PathBuf {
        self.drafts.join(format!("consult-cw{n}"))
    }

    fn window(&self, n: u32) -> WindowFile {
        wire::decode(&fs::read_to_string(self.ws(n).join(".consult/window.json")).unwrap()).unwrap()
    }

    fn entries(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.drafts)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    }
}

fn out(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}

fn err(o: &Output) -> String {
    String::from_utf8(o.stderr.clone()).unwrap()
}

fn rc(o: &Output) -> i32 {
    o.status.code().unwrap()
}

fn tz_exe() -> String {
    fs::canonicalize(env!("CARGO_BIN_EXE_tz"))
        .unwrap()
        .display()
        .to_string()
}

/// 鍵 tsuzuri.draftsdir が普通の file を指せば rc 2 で字 dir でない（その file に触れない）。
fn refuses_plain_drafts(fx: &Fx) {
    let plain = fx.root.join("plain");
    fs::write(&plain, "").unwrap();
    fx.key("tsuzuri.draftsdir", &plain);
    let o = fx.tz(&["consult", "open", "--by", "seat"]);
    assert_eq!(rc(&o), 2);
    assert!(err(&o).contains("dir でない"), "{}", err(&o));
    assert!(fs::read(&plain).unwrap().is_empty());
}

/// open の 14 の悪い形（どれも 1 つの欠けだけを持つ）と、その欠けを照らす断りの字。
fn bad_starters() -> Vec<(Vec<&'static str>, &'static str)> {
    let said = "--said は --by chat の時だけ";
    let request = "--request は --by button の時だけ";
    let by = "--by は seat・chat・button のどれか";
    vec![
        (vec!["--by", "chat"], said),
        (vec!["--by", "chat", "--said", "20261003T1412"], said),
        (vec!["--by", "seat", "--said", "20261003T1410Z"], said),
        (vec!["--by", "button"], request),
        (vec!["--by", "seat", "--request", RQ], request),
        (
            vec!["--by", "button", "--request", "rq-1"],
            "--request が頼みの id の形でない",
        ),
        (vec!["--by", "owner"], by),
        (vec!["--form", "talk"], by),
        (
            vec!["--by", "seat", "--form", "chat"],
            "--form は talk か ask",
        ),
        (
            vec!["--by", "seat", "--via", "完了"],
            "--via は 見張り・hook・一覧 のどれか",
        ),
        (vec!["--by", "seat", "--by", "seat"], "--by が 2 度ある"),
        (vec!["--by", "seat", "--topic="], "--topic の値が空"),
        (vec!["--by", "seat", "--name", "x"], "知らない引数 --name"),
        (vec!["--by", "seat", "extra"], "知らない引数 extra"),
    ]
}

/// 悪い形はどれも rc 1 で、標準エラーにその欠けの断りの字を出す（前の検めで rc 1 になる空振りを除く）。
fn refuses_bad_starters(fx: &Fx) {
    for (bad, why) in bad_starters() {
        let mut args = vec!["consult", "open"];
        args.extend(bad.iter().copied());
        let o = fx.tz(&args);
        assert_eq!(rc(&o), 1, "{bad:?} {}", err(&o));
        assert!(err(&o).contains(why), "{bad:?} {}", err(&o));
    }
}

#[test]
fn cwopn_keys_or_rc2() {
    let fx = Fx::new("keys", false);
    let o = fx.tz(&["consult", "open", "--by", "seat"]);
    assert_eq!(rc(&o), 2, "{}", err(&o));
    assert!(err(&o).contains("scribe2.statedir"), "{}", err(&o));
    fx.key("scribe2.statedir", &fx.state);
    let o = fx.tz(&["consult", "open", "--by", "seat"]);
    assert_eq!(rc(&o), 2);
    assert!(err(&o).contains("tsuzuri.draftsdir"), "{}", err(&o));
    fx.key("tsuzuri.draftsdir", &fx.root.join("nowhere"));
    let o = fx.tz(&["consult", "open", "--by", "seat"]);
    assert_eq!(rc(&o), 2);
    assert!(err(&o).contains("dir でない"), "{}", err(&o));
    assert!(fx.entries().is_empty() && !fx.root.join("nowhere").exists());
    refuses_plain_drafts(&fx);
    fx.key("tsuzuri.draftsdir", &fx.drafts);
    fs::remove_file(fx.root.join("ledger.json")).unwrap();
    let o = fx.tz(&["consult", "open", "--by", "seat"]);
    assert_eq!(rc(&o), 2);
    assert!(err(&o).contains("台帳が読めない"), "{}", err(&o));
    assert!(fx.entries().is_empty());
    fx.ledger(LEDGER);
    let o = fx.tz(&["consult", "open", "--by", "seat"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let o = fx.tz(&["consult", "nothing"]);
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("usage: tz consult"));
}

/// 用意した作業場の dir と file の形。
fn assert_tree(ws: &Path) {
    assert!(ws.join(".git").is_dir());
    let dirs = [".consult", "bundle", "drafts", "findings", "work"];
    assert!(dirs.iter().all(|d| ws.join(d).is_dir()));
    let files = [
        ".consult/window.json",
        "notes.md",
        "bundle/brief.md",
        "bundle/reads.md",
        "bundle/ledger.json",
        "bundle/digest",
    ];
    assert!(files.iter().all(|f| ws.join(f).is_file()));
    assert!(!ws.join("bundle/question.md").exists());
    let notes = fs::read_to_string(ws.join("notes.md")).unwrap();
    assert_eq!(notes, "# 窓 cw1 の控え\n");
}

#[test]
fn cwopn_ids_and_tree() {
    let fx = Fx::new("tree", true);
    let o = fx.tz(&["consult", "open", "--by", "seat"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let ws = fs::canonicalize(fx.ws(1)).unwrap();
    assert_eq!(out(&o), format!("窓 cw1 を用意した\n{}\n", ws.display()));
    assert_tree(&ws);
    let w = fx.window(1);
    assert_eq!(w.id, WindowId::new(1).unwrap());
    let got = (
        w.form, w.topic, w.model, w.effort, w.starter, w.uttered, w.request,
    );
    let want = (
        Form::Talk,
        None,
        "fable".into(),
        "xhigh".into(),
        Starter::Seat,
        None,
        None,
    );
    assert_eq!(got, want);
    let args = ["--form", "ask", "--model", "opus", "--effort", "high"];
    let o = fx.tz(&[&["consult", "open", "--by", "seat"][..], &args].concat());
    assert_eq!(out(&o).lines().next(), Some("窓 cw2 を用意した"));
    let w = fx.window(2);
    assert_eq!(
        (w.form, w.model, w.effort),
        (Form::Ask, "opus".into(), "high".into())
    );
    fs::create_dir(fx.drafts.join("retired-consult-cw7")).unwrap();
    fs::create_dir(fx.drafts.join("consult-x")).unwrap();
    let o = fx.tz(&["consult", "open", "--by", "seat"]);
    assert_eq!(out(&o).lines().next(), Some("窓 cw8 を用意した"));
    let names = [
        "consult-cw1",
        "consult-cw2",
        "consult-cw8",
        "consult-x",
        "retired-consult-cw7",
    ];
    assert_eq!(fx.entries(), names);
    assert!(fx.writes().is_empty());
}

#[test]
fn cwopn_version_and_starters() {
    let fx = Fx::new("starters", true);
    let o = fx
        .command(&["consult", "open", "--by", "seat"])
        .env_remove("TZ_PLUGIN_VERSION")
        .output()
        .unwrap();
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("TZ_PLUGIN_VERSION"), "{}", err(&o));
    let o = fx
        .command(&["consult", "open", "--by", "seat"])
        .env("TZ_PLUGIN_VERSION", "0.1.0")
        .output()
        .unwrap();
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("TZ_PLUGIN_VERSION"), "{}", err(&o));
    let o = fx.tz(&["--version"]);
    assert_eq!(rc(&o), 0);
    assert_eq!(
        out(&o),
        format!("tz {} consult 0.2.0\n", env!("CARGO_PKG_VERSION"))
    );
    refuses_bad_starters(&fx);
    assert!(fx.entries().is_empty());
    let o = fx.tz(&[
        "consult",
        "open",
        "--by",
        "chat",
        "--said",
        "20261003T1410Z",
        "--topic",
        "自由な題",
    ]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let w = fx.window(1);
    assert_eq!(w.starter, Starter::Chat);
    assert_eq!(w.uttered.as_deref(), Some("20261003T1410Z"));
    assert_eq!(w.topic.as_deref(), Some("自由な題"));
    assert!(fx.writes().is_empty());
}

/// 束の 4 つの file の名と字を順に FNV-1a 64 に通した要約値。
fn digest_of(dir: &Path) -> String {
    let mut bytes = Vec::new();
    for name in ["brief.md", "question.md", "reads.md", "ledger.json"] {
        bytes.extend_from_slice(name.as_bytes());
        bytes.push(0);
        bytes.extend(fs::read(dir.join(name)).unwrap());
        bytes.push(0);
    }
    format!("{:016x}", fnv1a64(&bytes))
}

/// 題 fx-c.2 で開いた窓の束の file の字。
fn assert_bundle(fx: &Fx, dir: &Path) {
    let read = |f: &str| fs::read_to_string(dir.join(f)).unwrap();
    assert_eq!(
        read("brief.md"),
        brief(&tz_exe(), WindowId::new(1).unwrap())
    );
    assert_eq!(
        read("question.md"),
        "# 題 fx-c.2\n\n問いの題名\n\n問いの本文\n"
    );
    assert_eq!(read("ledger.json"), LEDGER);
    let reads = read("reads.md");
    let wants = [
        format!("- {}\n", fx.repo.display()),
        format!("- {}/fleet\n", fx.state.display()),
        format!("- {}/pipe\n", fx.state.display()),
        "- bundle/ledger.json".to_string(),
    ];
    assert!(wants.iter().all(|w| reads.contains(w)), "{reads}");
}

/// 字を選んで要約値の頭の 3 字が 0 になる束（要約値は頭の 0 を残して 16 字）。
fn assert_padded_digest(fx: &Fx) {
    let pad = fx.root.join("pad");
    fs::create_dir_all(&pad).unwrap();
    let files = [
        ("brief.md", "手引き\n"),
        ("question.md", "問い 7248\n"),
        ("reads.md", "読む物\n"),
        ("ledger.json", "[]"),
    ];
    for (name, text) in files {
        fs::write(pad.join(name), text).unwrap();
    }
    assert_eq!(bundle::digest(&pad), "000ed594a174a89c");
    assert_eq!(digest_of(&pad), "000ed594a174a89c");
}

#[test]
fn cwopn_bundle_files_and_digest() {
    let fx = Fx::new("bundle", true);
    let o = fx.tz(&["consult", "open", "--by", "seat", "--topic", "fx-c.2"]);
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let dir = fx.ws(1).join("bundle");
    let read = |f: &str| fs::read_to_string(dir.join(f)).unwrap();
    assert_bundle(&fx, &dir);
    let want = digest_of(&dir);
    assert_eq!(
        (read("digest"), bundle::digest(&dir)),
        (format!("{want}\n"), want.clone())
    );
    assert_padded_digest(&fx);
    let o = fx.tz(&["consult", "bundle", "cw1"]);
    assert_eq!((rc(&o), out(&o)), (0, format!("束 {want}\n")));
    let newer = LEDGER.replace("問いの本文", "替えた本文");
    fx.ledger(&newer);
    let o = fx.tz(&["consult", "bundle", "cw1"]);
    assert_eq!((rc(&o), read("ledger.json")), (0, newer));
    assert_eq!(
        read("question.md"),
        "# 題 fx-c.2\n\n問いの題名\n\n問いの本文\n"
    );
    let o2 = fx.tz(&["consult", "bundle", "cw1", "--topic", "fx-c.2"]);
    assert_eq!(
        read("question.md"),
        "# 題 fx-c.2\n\n問いの題名\n\n替えた本文\n"
    );
    assert_ne!(out(&o), out(&o2));
    assert_eq!(out(&o2), format!("束 {}\n", digest_of(&dir)));
    fs::remove_file(fx.root.join("ledger.json")).unwrap();
    assert_eq!(rc(&fx.tz(&["consult", "bundle", "cw1"])), 2);
    fx.ledger(LEDGER);
    let rcs: Vec<i32> = [vec!["cw9"], vec!["w1"], vec![], vec!["cw1", "cw1"]]
        .iter()
        .map(|a| rc(&fx.tz(&[&["consult", "bundle"][..], a].concat())))
        .collect();
    assert_eq!(rcs, [1, 1, 1, 1]);
}

#[test]
fn cwopn_bundle_inside_window() {
    let fx = Fx::new("inside", true);
    assert_eq!(rc(&fx.tz(&["consult", "open", "--by", "seat"])), 0);
    let ws = fx.ws(1);
    let calls = fs::read_to_string(fx.root.join("bd.log"))
        .unwrap()
        .lines()
        .count();
    fx.ledger(&LEDGER.replace("問いの本文", "窓から見えない本文"));
    fs::write(ws.join("drafts/q.md"), "窓の問い\n").unwrap();
    let o = fx
        .command(&["consult", "bundle", "cw1", "--question", "drafts/q.md"])
        .current_dir(&ws)
        .env("TZ_CONSULT_ID", "cw1")
        .output()
        .unwrap();
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let dir = ws.join("bundle");
    assert_eq!(fs::read_to_string(dir.join("ledger.json")).unwrap(), LEDGER);
    assert_eq!(
        fs::read_to_string(dir.join("question.md")).unwrap(),
        "窓の問い\n"
    );
    assert_eq!(out(&o), format!("束 {}\n", bundle::digest(&dir)));
    let o = fx
        .command(&["consult", "bundle", "cw1", "--topic", "fx-c.2"])
        .current_dir(&ws)
        .env("TZ_CONSULT_ID", "cw1")
        .output()
        .unwrap();
    assert_eq!(rc(&o), 0);
    assert_eq!(
        fs::read_to_string(dir.join("question.md")).unwrap(),
        "# 題 fx-c.2\n\n問いの題名\n\n問いの本文\n"
    );
    let after = fs::read_to_string(fx.root.join("bd.log"))
        .unwrap()
        .lines()
        .count();
    assert_eq!(calls, after, "窓の中の束は bd を撃たない");
    inside_only_own_window(&fx);
}

/// 窓の中の束は環境の窓の id が束の窓の id で cwd がその窓の作業場の時だけ（作業場でない cwd と
/// ほかの窓の作業場は断り、環境がほかの窓の id なら席の撃ちで台帳の写しを替える）。
fn inside_only_own_window(fx: &Fx) {
    let bundle_at = |cwd: &Path, id: &str| {
        fx.command(&["consult", "bundle", "cw1"])
            .current_dir(cwd)
            .env("TZ_CONSULT_ID", id)
            .output()
            .unwrap()
    };
    let o = bundle_at(&fx.drafts, "cw1");
    assert_eq!(rc(&o), 1, "cwd が作業場でなければ断る");
    assert!(err(&o).contains("窓 cw1 の作業場が無い"), "{}", err(&o));
    assert_eq!(rc(&fx.tz(&["consult", "open", "--by", "seat"])), 0);
    let o = bundle_at(&fx.ws(2), "cw1");
    assert_eq!(rc(&o), 1, "ほかの窓の作業場は断る");
    assert!(
        err(&o).contains("窓の id が cw2 で cw1 でない"),
        "{}",
        err(&o)
    );
    let o = bundle_at(&fx.repo, "cw2");
    assert_eq!(rc(&o), 0, "{}", err(&o));
    let copy = fs::read_to_string(fx.ws(1).join("bundle/ledger.json")).unwrap();
    assert!(
        copy.contains("窓から見えない本文"),
        "ほかの窓の id は席の撃ち"
    );
}

/// 頼みで開いた窓の控えと、偽の bdw が受けた受けの行（1 本だけ）の字。
fn assert_first_receipt(fx: &Fx, before: &str, after: &str) {
    let w = fx.window(1);
    let rq = RequestId::parse(RQ).unwrap();
    assert_eq!(
        (w.form, w.model, w.topic),
        (Form::Ask, "opus".into(), Some("fx-c.2".into()))
    );
    assert_eq!((w.starter, w.request), (Starter::Button, Some(rq.clone())));
    let writes = fx.writes();
    let [verb, id, line] = writes[0].as_slice() else {
        panic!("{writes:?}");
    };
    assert_eq!(
        (writes.len(), verb.as_str(), id.as_str()),
        (1, "update", "fx-c.1")
    );
    let text = line.strip_prefix("--append-notes=").unwrap();
    let Some(Line::Receipt { subject, via, at }) = read(text) else {
        panic!("{text}");
    };
    assert_eq!((subject, via), (Subject::Request(rq), Via::Watch));
    assert!(before <= at.as_str() && at.as_str() <= after, "{at}");
    assert_eq!(
        text,
        format!("相談の受け = {RQ}・経路 = 見張り・時刻 = {at}")
    );
}

#[test]
fn cwopn_request_receipt() {
    let fx = Fx::new("request", true);
    let open = |extra: &[&str]| {
        let base = ["consult", "open", "--by", "button", "--request"];
        fx.tz(&[&base[..], extra].concat())
    };
    let before = ruling::minute(events::now());
    let o = open(&[RQ]);
    let after = ruling::minute(events::now());
    assert_eq!(rc(&o), 0, "{}", err(&o));
    assert_first_receipt(&fx, &before, &after);
    let o = open(&["rq-20261003T1412Z-2"]);
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("台帳に無い"));
    let tail = "時刻 = 20261003T1412Z\\n相談の受け = rq-20261003T1412Z-1・経路 = hook・時刻 = 20261003T1413Z\"";
    fx.ledger(&LEDGER.replace("時刻 = 20261003T1412Z\"", tail));
    let o = open(&[RQ, "--via", "hook"]);
    assert_eq!(rc(&o), 1);
    assert!(err(&o).contains("もう受けた"), "{}", err(&o));
    let other = tail.replace("Z-1・経路", "Z-2・経路");
    fx.ledger(&LEDGER.replace("時刻 = 20261003T1412Z\"", &other));
    let o = open(&[RQ, "--via", "一覧", "--topic", "別の題"]);
    assert_eq!(rc(&o), 0, "ほかの頼みの受けは断りでない {}", err(&o));
    assert_eq!(fx.window(2).topic.as_deref(), Some("別の題"));
    let writes = fx.writes();
    assert_eq!((writes.len(), writes[1][1].as_str()), (2, "fx-c.1"));
    assert!(writes[1][2].contains("・経路 = 一覧・"), "{:?}", writes[1]);
    assert_eq!(fx.entries(), ["consult-cw1", "consult-cw2"]);
}

fn mark(k: u32, pid: u32) -> ProcMark {
    ProcMark {
        k,
        form: Form::Ask,
        pid,
        at: "20261003T1412Z".to_string(),
        again: k > 1,
        tmux_window: None,
        account: None,
    }
}

/// 窓の id と所見の id の読み。
fn readers_windows_findings(fx: &Fx) {
    for n in [1, 3] {
        fs::create_dir_all(fx.ws(n).join("findings")).unwrap();
        fs::create_dir_all(fx.ws(n).join(".consult")).unwrap();
    }
    fs::create_dir(fx.drafts.join("retired-consult-cw2")).unwrap();
    fs::create_dir(fx.drafts.join("retired-consult-cw10")).unwrap();
    fs::create_dir(fx.drafts.join("consult-cw0")).unwrap();
    fs::create_dir(fx.drafts.join("cw5")).unwrap();
    fs::write(fx.drafts.join("consult-cw4"), "file").unwrap();
    let w = |n| WindowId::new(n).unwrap();
    let want = [(w(1), false), (w(2), true), (w(3), false), (w(10), true)];
    assert_eq!(consult::windows(&fx.drafts), want);
    let ws = fx.ws(1);
    for name in [
        "cw1-2.json",
        "cw1-1.json",
        "cw3-1.json",
        "cw1-4.txt",
        "cw1-3",
        "cw1-0.json",
        "cw1-10.json",
    ] {
        fs::write(ws.join("findings").join(name), "{}").unwrap();
    }
    let f = |k| FindingId::new(w(1), k).unwrap();
    assert_eq!(consult::findings(&ws, w(1)), [f(1), f(2), f(10)]);
    assert!(consult::findings(&fx.ws(3), w(3)).is_empty());
}

/// process の印の読みと生きているかの判じ。
fn readers_procs(fx: &Fx) {
    let ws = fx.ws(1);
    let me = std::process::id();
    let put = |k: u32, pid: u32| {
        let text = wire::encode(&mark(k, pid)).unwrap();
        fs::write(consult::proc_path(&ws, k), text).unwrap();
    };
    put(2, me);
    put(1, 0);
    fs::write(ws.join(".consult/proc-3.json"), "{\"k\":3}").unwrap();
    let named = wire::encode(&mark(5, me)).unwrap();
    fs::write(ws.join(".consult/proc-5.txt"), named).unwrap();
    assert_eq!(consult::procs(&ws), [mark(1, 0), mark(2, me)]);
    assert!(consult::alive(me) && consult::alive(1) && !consult::alive(0));
    assert!(consult::live(&ws) && !consult::live(&fx.ws(3)));
    put(10, 0);
    assert_eq!(consult::procs(&ws), [mark(1, 0), mark(2, me), mark(10, 0)]);
    assert!(!consult::live(&ws));
}

/// 引数の読み。
fn readers_flags() {
    let (values, switches, many) = (&["--via"][..], &["--dry-run"][..], &["--keep"][..]);
    let args = [
        "cw1",
        "--keep",
        "b",
        "--keep=a",
        "--via",
        "hook",
        "--dry-run",
    ];
    let parsed = consult::flags(&args, values, switches, many).unwrap();
    assert_eq!(parsed.pos, ["cw1"]);
    assert_eq!(parsed.all("--keep"), ["b", "a"]);
    assert_eq!(parsed.get("--via"), Some("hook"));
    assert!(parsed.has("--dry-run") && !parsed.has("--again"));
    let bad: [(&[&str], &str); 5] = [
        (&["--via", "a", "--via", "b"], "--via が 2 度ある"),
        (&["--dry-run", "--dry-run"], "--dry-run が 2 度ある"),
        (&["--other", "x"], "知らない引数 --other"),
        (&["--via="], "--via の値が空"),
        (&["--via"], "--via の値が無い"),
    ];
    for (b, why) in bad {
        let e = consult::flags(b, values, switches, many).unwrap_err();
        assert_eq!(e, (1, why.to_string()), "{b:?}");
    }
}

#[test]
fn cwopn_workspace_readers() {
    let fx = Fx::new("readers", true);
    readers_windows_findings(&fx);
    readers_procs(&fx);
    readers_flags();
}
