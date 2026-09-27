//! 停止の切り替えの受付の歯（接頭辞 server_hb_・設計ノート surface-base 便 e-acct-hb の完了の条件）。
//! 偽の器と偽の git は、受けた argv を記録の置き場（repo と state dir の外）に 1 行ずつ足す script。
//! 器は argv の頭で doctor か seat heartbeat かを見分け、doctor は作業場の out の下の決めた字を出す。
//! seat heartbeat は off で席の dir に停止の記録を置き、on でそれを消す（器の側の書き）。
//! slow・rc・crash の下に同じ名の印が在れば、8 秒眠る・rc 3 で返る・自分を KILL で落とす。
//! git は -C の次の path の最後の区切りで字を選ぶ。anchor は作業場の work と other の下の dir で、
//! 字の中の path は実行の時に組む（行 D-4）。

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tsuzuri_boundary::acct::Acct;
use tsuzuri_boundary::accthb::{
    BAD_BODY, HEARTBEAT_ARGS, NO_PROJECT, NO_SEAT, VESSEL_FAILED, accept, word,
};
use tsuzuri_contract::account::{Heartbeat, HeartbeatResponse};
use tsuzuri_contract::wire;

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    work: PathBuf,
    other: PathBuf,
    states: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_hb")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let place = Place {
            repo: root.join("repo"),
            work: root.join("work"),
            other: root.join("other"),
            states: root.join("states"),
            root,
        };
        for dir in ["bin", "out", "git", "slow", "rc", "crash", "log"] {
            fs::create_dir_all(place.root.join(dir)).expect("置き場");
        }
        fs::create_dir_all(&place.repo).expect("repo");
        for p in ["proj-a", "proj-b", "proj-c", "proj-d"] {
            fs::create_dir_all(place.work.join(p)).expect("anchor");
        }
        fs::create_dir_all(place.other.join("proj-a")).expect("同じ名の anchor");
        fs::create_dir_all(place.state("state-h")).expect("引数の state dir");
        let root = place.root.display().to_string();
        script(
            &place.program("scribe2"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/scribe2'\n\
              case \"$1\" in doctor) f=\"doctor-${3##*/}\" ;; seat) f=\"hb-${5##*/}\" ;; *) exit 2 ;; esac\n\
              if [ -e 'ROOT/slow/'\"$f\" ]; then exec sleep 8; fi\n\
              if [ -e 'ROOT/rc/'\"$f\" ]; then exit 3; fi\n\
              if [ -e 'ROOT/crash/'\"$f\" ]; then kill -KILL $$; fi\n\
              if [ \"$1\" = seat ]; then\n\
                d=\"$5/seat/${7%%:*}_${7#*:}\"\n\
                mkdir -p \"$d\"\n\
                case \"$3\" in off) printf 'ts=1790510400\\n' > \"$d/heartbeat-off\" ;; on) rm -f \"$d/heartbeat-off\" ;; *) exit 2 ;; esac\n\
                exit 0\n\
              fi\n\
              exec cat 'ROOT/out/'\"$f\""
                .replace("ROOT", &root),
        );
        script(
            &place.program("git"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/git'\n\
              f=\"${2%/}\"; f=\"${f##*/}\"\n\
              exec cat 'ROOT/git/'\"$f\""
                .replace("ROOT", &root),
        );
        place.lay();
        place
    }

    fn program(&self, name: &str) -> PathBuf {
        self.root.join("bin").join(name)
    }

    fn anchor(&self, project: &str) -> String {
        self.work.join(project).display().to_string()
    }

    fn state(&self, name: &str) -> PathBuf {
        self.states.join(name)
    }

    fn put(&self, path: PathBuf, text: &str) {
        fs::create_dir_all(path.parent().expect("親")).expect("親の dir");
        fs::write(&path, text).expect("file を置く");
    }

    /// 字を組んで置く（anchor の path は実行の時の作業場から）。
    /// proj-b は宣言に末尾の「/」付きで書き、doctor には付けずに書く。other の proj-a は work の proj-a の後に書く。
    /// proj-c は git が落ち、proj-d は doctor に orchestrator の席の行が無い。
    fn lay(&self) {
        let [a, b, c, d] = ["proj-a", "proj-b", "proj-c", "proj-d"].map(|p| self.anchor(p));
        let other_a = self.other.join("proj-a").display().to_string();
        self.put(
            self.state("state-h").join("host.toml"),
            &format!(
                "[[account]]\nlabel = \"acct-1\"\n\n\
                 [[account-group]]\nname = \"g-a\"\nanchors = [\"{a}\", \"{b}/\"]\naccounts = [\"acct-1\"]\n\n\
                 [[account-group]]\nname = \"g-b\"\nanchors = [\n  \"{other_a}\",\n  \"{c}\",\n  \"{d}\",\n]\n\
                 accounts = [\"acct-1\"]\n"
            ),
        );
        self.put(
            self.root.join("out/doctor-state-a"),
            &format!(
                "doctor: state dir ok\n\
                 seat: role=pipeline anchor={a} target=proj-a:1.1 account=acct-1 model=opus\n\
                 seat: role=orchestrator anchor={other_a} target=proj-z:0.1 account=acct-1 model=opus\n\
                 seat: role=orchestrator anchor={a}/ target=proj-a:0.1 account=acct-1 model=opus\n\
                 seat: role=pipeline anchor={d} target=proj-d:1.1 account=acct-1 model=opus\n"
            ),
        );
        self.put(
            self.root.join("out/doctor-state-b"),
            &format!(
                "doctor: state dir ok\n\
                 seat: role=orchestrator anchor={b} target=proj-b:0.1 account=acct-1 model=sonnet\n"
            ),
        );
        let (sa, sb) = (self.state("state-a"), self.state("state-b"));
        self.put(
            self.root.join("git/proj-a"),
            &format!("  {}  \n", sa.display()),
        );
        self.put(self.root.join("git/proj-b"), &format!("{}\n", sb.display()));
        self.put(self.root.join("git/proj-d"), &format!("{}\n", sa.display()));
        for (dir, seat) in [
            (&sa, "proj-a_0.1"),
            (&sa, "proj-a_1.1"),
            (&sa, "proj-z_0.1"),
            (&sb, "proj-b_0.1"),
        ] {
            self.put(
                dir.join("seat").join(seat).join("state.jsonl"),
                "{\"schema\":1,\"state\":\"idle\",\"event\":\"stop\",\"ts\":1790440000,\"sid\":\"s-1\"}\n",
            );
            self.put(
                dir.join("seat").join(seat).join("tick-last"),
                "ts=1790510390 decision=noop reason=seat-busy\n",
            );
        }
        self.put(sa.join("fleet/events.jsonl"), "");
    }

    fn acct(&self) -> Acct {
        Acct::new(
            self.program("scribe2"),
            self.program("git"),
            "/nonexistent/tz-no-such-bd",
            self.state("state-h"),
            &self.repo,
        )
    }

    /// 偽の器の `name` の出力に印を置く（`kind` は slow・rc・crash）。
    fn mark(&self, kind: &str, name: &str) {
        fs::write(self.root.join(kind).join(name), "").expect("印");
    }

    /// 偽の program が受けた argv（1 回 1 行・撃った順）。
    fn calls(&self, program: &str) -> Vec<String> {
        fs::read_to_string(self.root.join("log").join(program))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn clear_calls(&self) {
        for program in ["scribe2", "git"] {
            let _ = fs::remove_file(self.root.join("log").join(program));
        }
    }

    /// 器が書く停止の記録。
    fn off_file(&self, state: &str, seat: &str) -> PathBuf {
        self.state(state)
            .join("seat")
            .join(seat)
            .join("heartbeat-off")
    }

    /// repo・anchor・state dir の全 file の path と byte。
    fn snapshot(&self) -> Vec<(PathBuf, Vec<u8>)> {
        [&self.repo, &self.work, &self.other, &self.states]
            .into_iter()
            .flat_map(|d| tree(d))
            .collect()
    }
}

/// dir の中の file の path と byte の一覧。
fn tree(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).expect("dir を読む") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                out.push((path.clone(), Vec::new()));
                stack.push(path);
            } else {
                out.push((path.clone(), fs::read(&path).expect("file を読む")));
            }
        }
    }
    out.sort();
    out
}

fn body(project: &str, to: &str) -> String {
    format!("{{\"project\":\"{project}\",\"to\":\"{to}\"}}")
}

fn ok(target: &str, to: Heartbeat) -> (u16, String) {
    let json = wire::encode(&HeartbeatResponse {
        target: target.to_string(),
        to,
    })
    .expect("応答の字");
    (200, json)
}

fn refused(status: u16, text: &str) -> (u16, String) {
    (status, text.to_string())
}

#[test]
fn server_hb_off_then_on_shoots_seat() {
    let place = Place::new("off-on");
    let acct = place.acct();
    let sa = place.state("state-a").display().to_string();
    let off = place.off_file("state-a", "proj-a_0.1");

    let got = accept(&acct, &body("proj-a", "off"));
    assert_eq!(got, ok("proj-a:0.1", Heartbeat::Off));
    let response: HeartbeatResponse = wire::decode(&got.1).expect("応答は HeartbeatResponse");
    assert_eq!(response.target, "proj-a:0.1");
    assert_eq!(
        place.calls("scribe2"),
        [
            format!("doctor --state-dir {sa}"),
            format!("seat heartbeat off --state-dir {sa} --target proj-a:0.1"),
        ]
    );
    assert_eq!(
        place.calls("git"),
        [format!(
            "-C {} config --get scribe2.statedir",
            place.anchor("proj-a")
        )],
        "宣言の最初の同じ名の anchor だけを引く"
    );
    assert_eq!(
        fs::read_to_string(&off).ok().as_deref(),
        Some("ts=1790510400\n"),
        "器が停止の記録を置く"
    );

    place.clear_calls();
    let got = accept(&acct, &body("proj-a", "on"));
    assert_eq!(got, ok("proj-a:0.1", Heartbeat::On));
    assert_eq!(
        place.calls("scribe2"),
        [
            format!("doctor --state-dir {sa}"),
            format!("seat heartbeat on --state-dir {sa} --target proj-a:0.1"),
        ]
    );
    assert!(!off.exists(), "器が停止の記録を消す");

    // 宣言の末尾の「/」と doctor の anchor の違いは同じ path。
    place.clear_calls();
    let sb = place.state("state-b").display().to_string();
    assert_eq!(
        accept(&acct, &body("proj-b", "off")),
        ok("proj-b:0.1", Heartbeat::Off)
    );
    assert_eq!(
        place.calls("scribe2"),
        [
            format!("doctor --state-dir {sb}"),
            format!("seat heartbeat off --state-dir {sb} --target proj-b:0.1"),
        ]
    );
    assert_eq!(HEARTBEAT_ARGS, ["seat", "heartbeat"]);
    assert_eq!(word(Heartbeat::On), "on");
    assert_eq!(word(Heartbeat::Off), "off");
}

#[test]
fn server_hb_bad_body_shoots_nothing() {
    let place = Place::new("bad-body");
    let acct = place.acct();
    for text in [
        "",
        "{",
        "[]",
        "{\"project\":\"proj-a\"}",
        "{\"to\":\"off\"}",
        "{\"project\":\"proj-a\",\"to\":\"pause\"}",
        "{\"project\":\"proj-a\",\"to\":\"OFF\"}",
        "{\"project\":\"proj-a\",\"to\":true}",
        "{\"project\":7,\"to\":\"off\"}",
    ] {
        assert_eq!(accept(&acct, text), refused(400, BAD_BODY), "{text}");
    }
    assert_eq!(BAD_BODY, "bad-body");
    assert!(place.calls("scribe2").is_empty(), "器を撃たない");
    assert!(place.calls("git").is_empty(), "git を撃たない");
}

#[test]
fn server_hb_no_project_shoots_nothing() {
    let place = Place::new("no-project");
    let acct = place.acct();
    for project in [
        "proj-x",
        "",
        "work",
        "other",
        "--target",
        "proj-a:0.1",
        "seat",
    ] {
        assert_eq!(
            accept(&acct, &body(project, "off")),
            refused(404, NO_PROJECT),
            "{project}"
        );
    }
    // 群の宣言が読めなければ、どの project も無い。
    let none = Acct::new(
        place.program("scribe2"),
        place.program("git"),
        "/nonexistent/tz-no-such-bd",
        place.state("state-a"),
        &place.repo,
    );
    assert_eq!(
        accept(&none, &body("proj-a", "off")),
        refused(404, NO_PROJECT)
    );
    assert_eq!(NO_PROJECT, "no-project");
    assert!(place.calls("scribe2").is_empty(), "器を撃たない");
    assert!(place.calls("git").is_empty(), "git を撃たない");
}

#[test]
fn server_hb_no_seat_does_not_shoot_heartbeat() {
    let place = Place::new("no-seat");
    let acct = place.acct();
    // git が落ちる project は器を撃たない。
    assert_eq!(accept(&acct, &body("proj-c", "off")), refused(404, NO_SEAT));
    assert!(place.calls("scribe2").is_empty());
    // doctor に orchestrator の席の行が無い project は doctor だけを撃つ。
    assert_eq!(accept(&acct, &body("proj-d", "off")), refused(404, NO_SEAT));
    assert_eq!(
        place.calls("scribe2"),
        [format!(
            "doctor --state-dir {}",
            place.state("state-a").display()
        )]
    );
    // git が撃てなければ state dir は引けない。
    let no_git = Acct::new(
        place.program("scribe2"),
        "/nonexistent/tz-no-such-git",
        "/nonexistent/tz-no-such-bd",
        place.state("state-h"),
        &place.repo,
    );
    assert_eq!(
        accept(&no_git, &body("proj-a", "off")),
        refused(404, NO_SEAT)
    );
    assert_eq!(NO_SEAT, "no-seat");
    assert!(
        place
            .calls("scribe2")
            .iter()
            .all(|c| !c.starts_with("seat ")),
        "seat heartbeat を撃たない"
    );
}

#[test]
fn server_hb_vessel_failures() {
    for (kind, name) in [
        ("rc", "doctor-state-a"),
        ("crash", "doctor-state-a"),
        ("missing", "doctor-state-a"),
        ("rc", "hb-state-a"),
        ("crash", "hb-state-a"),
    ] {
        let place = Place::new(&format!("fail-{kind}-{name}"));
        if kind == "missing" {
            fs::remove_file(place.root.join("out").join(name)).expect("出力を消す");
        } else {
            place.mark(kind, name);
        }
        assert_eq!(
            accept(&place.acct(), &body("proj-a", "off")),
            refused(502, VESSEL_FAILED),
            "{kind} {name}"
        );
        let heartbeats = place
            .calls("scribe2")
            .iter()
            .filter(|c| c.starts_with("seat heartbeat "))
            .count();
        let want = usize::from(name.starts_with("hb-"));
        assert_eq!(heartbeats, want, "{kind} {name}");
    }
    // 器が撃てない。
    let place = Place::new("fail-no-vessel");
    let none = Acct::new(
        "/nonexistent/tz-no-such-scribe2",
        place.program("git"),
        "/nonexistent/tz-no-such-bd",
        place.state("state-h"),
        &place.repo,
    );
    assert_eq!(
        accept(&none, &body("proj-a", "off")),
        refused(502, VESSEL_FAILED)
    );
    assert_eq!(VESSEL_FAILED, "vessel-failed");
}

fn slow(name: &str) {
    let place = Place::new(&format!("slow-{name}"));
    place.mark("slow", name);
    let from = Instant::now();
    let got = accept(&place.acct(), &body("proj-a", "off"));
    let took = from.elapsed();
    assert_eq!(got, refused(502, VESSEL_FAILED), "{name}");
    assert!(took >= Duration::from_secs(4), "5 秒待たずに返る: {took:?}");
    assert!(
        took < Duration::from_secs(8),
        "眠りの終わりまで待つ: {took:?}"
    );
}

#[test]
fn server_hb_slow_doctor_fails() {
    slow("doctor-state-a");
}

#[test]
fn server_hb_slow_heartbeat_fails() {
    slow("hb-state-a");
}

#[test]
fn server_hb_argv_from_vessel_not_body() {
    let place = Place::new("argv");
    let sa = place.state("state-a").display().to_string();
    let text = "{\"project\":\"proj-a\",\"to\":\"off\",\"target\":\"evil:9.9\",\
                \"state_dir\":\"/tmp/evil\",\"anchor\":\"/tmp/evil\"}";
    assert_eq!(
        accept(&place.acct(), text),
        ok("proj-a:0.1", Heartbeat::Off)
    );
    let calls = place.calls("scribe2");
    assert_eq!(
        calls.last().map(String::as_str),
        Some(format!("seat heartbeat off --state-dir {sa} --target proj-a:0.1").as_str())
    );
    assert!(calls.iter().all(|c| !c.contains("evil")), "{calls:?}");
    assert!(place.calls("git").iter().all(|c| !c.contains("evil")));
}

#[test]
fn server_hb_bytes_only_vessel_file() {
    let place = Place::new("bytes");
    let acct = place.acct();
    let off = place.off_file("state-a", "proj-a_0.1");
    let before = place.snapshot();
    // 断りは何も変えない。
    for (project, to) in [("proj-x", "off"), ("proj-c", "off"), ("proj-d", "off")] {
        accept(&acct, &body(project, to));
        assert!(
            before == place.snapshot(),
            "{project} の断りで byte が変わる"
        );
    }
    accept(&acct, "{");
    assert!(before == place.snapshot());
    // off は器の書く停止の記録だけが増える。
    assert_eq!(accept(&acct, &body("proj-a", "off")).0, 200);
    let after: Vec<_> = place
        .snapshot()
        .into_iter()
        .filter(|(p, _)| *p != off)
        .collect();
    assert!(before == after, "停止の記録の外の byte が変わる");
    assert!(off.exists());
    // on は器が消し、元の byte に戻る。
    assert_eq!(accept(&acct, &body("proj-a", "on")).0, 200);
    assert!(before == place.snapshot(), "on の後に元の byte に戻らない");
}

#[test]
fn server_hb_no_new_dependencies() {
    let deps = |krate: &str| -> Vec<String> {
        let text = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join(krate)
                .join("Cargo.toml"),
        )
        .expect("Cargo.toml");
        let mut names = Vec::new();
        let mut inside = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                inside = line.ends_with("dependencies]");
                continue;
            }
            if inside
                && let Some((name, _)) = line.split_once('=')
                && !line.starts_with('#')
            {
                let name = name.trim();
                names.push(name.strip_suffix(".workspace").unwrap_or(name).to_string());
            }
        }
        names
    };
    assert_eq!(
        deps("tsuzuri-boundary"),
        ["tsuzuri-contract", "tsuzuri-core"]
    );
}
