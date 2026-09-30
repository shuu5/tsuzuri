//! 配達の hook の歯（行 f-deliver・接頭辞 fdlv_）。
//! 偽の bd は撃たれた回ごとの引数と cwd を記録の置き場に足してから作業場の out.json を出す script。
//! tz は作業場の根（偽の bd の在る dir）を PATH の頭に足して撃つ。歯の名の数えは中核の tests/fdlv.rs が持つ。
#![cfg(test)]

use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_boundary::hook::deliver::{self, Args, USAGE};
use tsuzuri_boundary::server::batch::BATCH_PREFIX;
use tsuzuri_boundary::server::ledger::{BD, BD_ARGS};
use tsuzuri_contract::board::Reading;
use tsuzuri_core::delivery::{BATCH_FIELD, context, pointed, said};

const D1: &str = "fx-d.1:20260930T0101Z-1";
const D2: &str = "fx-d.2:20260930T0110Z-1";
const BATCH: &str = "batch:20260930T0110Z-1";

/// 台帳の字（問い 3 本・fx-d.1 は 1 問の裁定・fx-d.2 と fx-d.3 は同じ束の裁定・逆斜線と n は改行）。
const LEDGER: &str = r#"[
{"id":"fx-d.1","labels":["intake:question"],"status":"closed","notes":"裁定 id = fx-d.1:20260930T0101Z-1・問い = fx-d.1・逐語 = 一行目\\n二行目・終わり"},
{"id":"fx-d.2","labels":["intake:question"],"status":"closed","notes":"裁定 id = fx-d.2:20260930T0110Z-1・問い = fx-d.2・束 = batch:20260930T0110Z-1・逐語 = 束の答え"},
{"id":"fx-d.3","labels":["intake:question"],"status":"closed","notes":"裁定 id = fx-d.3:20260930T0110Z-1・問い = fx-d.3・束 = batch:20260930T0110Z-1・逐語 = 束の答え"}
]"#;

/// 器の指し示しの行を prompt に持つ hook の入力。
fn input(id: &str) -> String {
    format!(
        r#"{{"session_id":"s-1","hook_event_name":"UserPromptSubmit","prompt":"scribe2 seat: 裁定 {id} が届いた（在りかは裁定面の記帳）"}}"#
    )
}

/// 歯ごとの作業場（空の repo・記録の置き場・偽の bd・out.json）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fdlv").join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, log) = (root.join("repo"), root.join("log"));
        fs::create_dir_all(&repo).expect("repo の置き場");
        fs::create_dir_all(&log).expect("記録の置き場");
        let bd = root.join("bd");
        fs::write(
            &bd,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}/bd.log'\npwd -P >> '{log}/bd.cwd'\nexec cat '{out}'\n",
                log = log.display(),
                out = root.join("out.json").display()
            ),
        )
        .expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
        let place = Place { root, repo, log };
        place.bd_returns(LEDGER);
        place
    }

    /// 偽の bd が返す字を置く。
    fn bd_returns(&self, text: &str) {
        fs::write(self.root.join("out.json"), text).expect("偽の bd の出力");
    }

    /// 偽の bd を落とす（出す file が無いので rc 1）。
    fn bd_fails(&self) {
        fs::remove_file(self.root.join("out.json")).expect("偽の bd の出力を消す");
    }

    /// 偽の bd が撃たれた回ごとの引数の行。
    fn bd_calls(&self) -> Vec<String> {
        fs::read_to_string(self.log.join("bd.log"))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// 偽の bd が撃たれた回ごとの cwd。
    fn bd_cwds(&self) -> Vec<String> {
        fs::read_to_string(self.log.join("bd.cwd"))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// hook deliver に --repo と偽の --bd を渡す引数。
    fn hook_args(&self, repo: &Path) -> Vec<OsString> {
        vec![
            "hook".into(),
            "deliver".into(),
            "--repo".into(),
            repo.into(),
            "--bd".into(),
            self.root.join("bd").into(),
        ]
    }

    /// tz を（作業場の根を PATH の頭に足して）撃ち、標準入力に payload を書いて閉じ、終わりまで待つ。
    fn tz(&self, args: &[OsString], payload: &str) -> Output {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut dirs = vec![self.root.clone()];
        dirs.extend(std::env::split_paths(&path));
        let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .env("PATH", std::env::join_paths(dirs).expect("PATH"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("tz を撃つ");
        let mut stdin = child.stdin.take().expect("標準入力");
        // tz が入力を読まずに終わる場では書きが落ちうるので捨てる。
        let _ = stdin.write_all(payload.as_bytes());
        drop(stdin);
        child.wait_with_output().expect("tz の終わり")
    }

    /// 標準入力に指し示しの入力を渡して撃つ。
    fn deliver(&self, id: &str) -> Output {
        self.tz(&self.hook_args(&self.repo), &input(id))
    }
}

fn args_of(list: &[&str]) -> Vec<OsString> {
    list.iter().map(OsString::from).collect()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 中核の said と context で組んだ答えの字（名指す id は payload の prompt から拾う）。
fn expected(payload: &str) -> String {
    let Reading::Known(found) = said(LEDGER, &pointed(payload)) else {
        panic!("歯の台帳が読めない");
    };
    format!("{}\n", context(&found).expect("答え"))
}

#[test]
fn fdlv_parse_args() {
    assert_eq!(BATCH_FIELD, BATCH_PREFIX);
    let want = |repo: &str, bd: &str| Args {
        repo: PathBuf::from(repo),
        bd: bd.into(),
    };
    assert_eq!(deliver::parse(&["--repo", "/r"]), Ok(want("/r", BD)));
    assert_eq!(
        deliver::parse(&["--repo=/r", "--bd", "b"]),
        Ok(want("/r", "b"))
    );
    for bad in [
        &[][..],
        &["--bd", "b"],
        &["--repo", "/r", "--bdw", "x"],
        &["--bdw", "x", "--repo", "/r"],
        &["--repo", "/r", "--repo", "/s"],
        &["--repo"],
        &["--repo="],
        &["--repo", "/r", "--bd="],
        &["/r"],
    ] {
        assert!(deliver::parse(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn fdlv_injects_named_verbatim() {
    for (name, id, count) in [("one", D1, 1), ("batch", BATCH, 2)] {
        let place = Place::new(name);
        let payload = input(id);
        let out = place.deliver(id);
        assert_eq!(out.status.code(), Some(0), "{name} の rc: {out:?}");
        assert_eq!(stdout(&out), expected(&payload), "{name} の標準出力");
        assert_eq!(stdout(&out).matches("の逐語:").count(), count, "{name} の裁定の数");
        assert!(stderr(&out).is_empty(), "{name} の標準エラー: {out:?}");
        assert_eq!(place.bd_calls(), [BD_ARGS.join(" ")], "{name} の bd の引数");
        let repo = place.repo.canonicalize().expect("repo の実体");
        assert_eq!(place.bd_cwds(), [repo.display().to_string()], "{name} の cwd");
    }
    // 束の中の 1 行の裁定の id だけを名指すときは、その 1 行だけを写す。
    let place = Place::new("member");
    let out = place.deliver(D2);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(stdout(&out).matches("の逐語:").count(), 1, "{out:?}");
}

#[test]
fn fdlv_quiet_without_pointer() {
    let place = Place::new("quiet");
    let plain = r#"{"session_id":"s-1","prompt":"ふつうの依頼"}"#;
    let stop_form = format!(r#"{{"prompt":"裁定 {D1} が届いた（問い fx-d.1）"}}"#);
    for payload in [plain, stop_form.as_str(), "not json", ""] {
        let out = place.tz(&place.hook_args(&place.repo), payload);
        assert_eq!(out.status.code(), Some(0), "{payload:?} の rc: {out:?}");
        assert!(out.stdout.is_empty(), "{payload:?} の標準出力: {out:?}");
        assert!(out.stderr.is_empty(), "{payload:?} の標準エラー: {out:?}");
    }
    assert!(place.bd_calls().is_empty(), "偽の bd が撃たれた");
}

#[test]
fn fdlv_unreadable_or_missing() {
    let cases: [(&str, &str, Option<&str>); 3] = [
        ("fails", D1, None),
        ("notjson", D1, Some("not json")),
        ("missing", "fx-d.9:20260930T0101Z-1", Some(LEDGER)),
    ];
    for (name, id, ledger) in cases {
        let place = Place::new(name);
        match ledger {
            Some(text) => place.bd_returns(text),
            None => place.bd_fails(),
        }
        let out = place.deliver(id);
        assert_eq!(out.status.code(), Some(0), "{name} の rc: {out:?}");
        assert!(out.stdout.is_empty(), "{name} の標準出力: {out:?}");
        let err = stderr(&out);
        assert!(err.starts_with("tz hook deliver: "), "{name}: {err:?}");
        assert!(err.ends_with('\n') && err.lines().count() == 1, "{name}: {err:?}");
        if name == "missing" {
            assert!(err.contains(id), "{name}: {err:?}");
        }
        assert_eq!(place.bd_calls().len(), 1, "{name} の bd の回数");
    }
}

#[test]
fn fdlv_usage_errors_are_one() {
    let place = Place::new("usage");
    let file = place.root.join("not-a-dir");
    fs::write(&file, "").expect("dir でない file");
    let bd = place.root.join("bd");
    let cases: Vec<(&str, Vec<OsString>)> = vec![
        ("bare", args_of(&["hook", "deliver"])),
        ("flag", {
            let mut a = place.hook_args(&place.repo);
            a.push("--nope".into());
            a
        }),
        ("file", place.hook_args(&file)),
        ("bd-only", {
            let mut a = args_of(&["hook", "deliver", "--bd"]);
            a.push(bd.into());
            a
        }),
    ];
    for (name, args) in cases {
        let out = place.tz(&args, &input(D1));
        assert_eq!(out.status.code(), Some(1), "{name} の rc: {out:?}");
        assert!(out.stdout.is_empty(), "{name} の標準出力: {out:?}");
        let err = stderr(&out);
        assert!(err.starts_with("tz hook deliver: "), "{name}: {err:?}");
        assert!(err.contains(USAGE), "{name}: {err:?}");
    }
    assert!(place.bd_calls().is_empty(), "偽の bd が撃たれた");
}
