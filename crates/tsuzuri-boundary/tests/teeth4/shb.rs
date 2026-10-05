//! project board の停止の切り替えの口の歯（接頭辞 shb_・設計ノート surface-wave8b 行 e-seat-hb の完了の条件・裁定 t3-hub.52.29）。
//! 偽の器と偽の git は、受けた argv を記録の置き場（repo と state dir の外）に 1 行ずつ足す script。
//! 器は argv の頭で doctor か seat heartbeat かを見分け、doctor は作業場の out の下の決めた字を出す。
//! seat heartbeat は off で席の dir に停止の記録を置き、on でそれを消す（器の側の書き）。
//! rc・crash の下に同じ名の印が在れば、rc 3 で返る・自分を KILL で落とす。
//! git は -C の次の path の最後の区切りで字を選ぶ。anchor は作業場の work と other の下の dir で、
//! 字の中の path は実行の時に組む（行 D-4）。server は同じ process の thread で 127.0.0.1 の空き port に立てる。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::common::{refused, tree};
use tsuzuri_boundary::acct::Acct;
use tsuzuri_boundary::accthb::{BAD_BODY, NO_PROJECT, NO_SEAT, VESSEL_FAILED, accept, accept_own};
use tsuzuri_boundary::server::route::{Key, Match};
use tsuzuri_boundary::server::{Config, Route, Server};
use tsuzuri_contract::account::{HEARTBEAT_PATH, Heartbeat, HeartbeatResponse};
use tsuzuri_contract::seathb::{PATH, SeatHeartbeatRequest};
use tsuzuri_contract::wire;

/// 着地済みの行と第 3 波〜第 8 波のほかの行の verify の filter の語（この行の接頭辞 shb_ は並べない）。
const FILTERS: [&str; 106] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
    "kcli_",
    "qgate_",
    "hcard_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
    "fstop_",
    "nsum_",
    "nsumw_",
    "fmark_",
    "fserve_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
    "wsteady_",
    "gsum_",
    "nstall_",
    "pfold_",
    "uword_",
    "cround_",
    "cgdom_",
    "csled_",
    "lhome_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
];

/// 読めない 9 つの本文（向きだけの形に合わない字）。
const UNREADABLE: [&str; 9] = [
    "",
    "{",
    "[]",
    "{}",
    "\"off\"",
    "{\"to\":\"OFF\"}",
    "{\"to\":\"pause\"}",
    "{\"to\":true}",
    "{\"to\":\"off\",\"project\":\"proj-a\"}",
];

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    work: PathBuf,
    other: PathBuf,
    states: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("shb").join(name);
        let _ = fs::remove_dir_all(&root);
        let place = Place {
            repo: root.join("repo"),
            files: root.join("files"),
            work: root.join("work"),
            other: root.join("other"),
            states: root.join("states"),
            root,
        };
        for dir in ["bin", "out", "git", "rc", "crash", "log"] {
            fs::create_dir_all(place.root.join(dir)).expect("置き場");
        }
        fs::create_dir_all(&place.repo).expect("repo");
        fs::create_dir_all(&place.files).expect("面の file の置き場");
        fs::write(place.files.join("index.html"), "tz").expect("index.html");
        fs::create_dir_all(place.work.join("proj-a").join("inner")).expect("anchor の下の dir");
        for p in ["proj-b", "proj-c", "proj-d"] {
            fs::create_dir_all(place.work.join(p)).expect("anchor");
        }
        fs::create_dir_all(place.other.join("proj-a")).expect("同じ名の anchor");
        for s in ["state-h", "state-a", "state-b"] {
            fs::create_dir_all(place.state(s)).expect("state dir");
        }
        std::os::unix::fs::symlink(place.work.join("proj-a"), place.link()).expect("symlink");
        let root = place.root.display().to_string();
        script(
            &place.program("scribe2"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/scribe2'\n\
              case \"$1\" in doctor) f=\"doctor-${3##*/}\" ;; seat) f=\"hb-${5##*/}\" ;; *) exit 2 ;; esac\n\
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
        script(&place.program("bd"), "echo '[]'");
        place.lay();
        place
    }

    fn program(&self, name: &str) -> PathBuf {
        self.root.join("bin").join(name)
    }

    /// work の proj-a を指す symlink（byte の比べの 4 つの dir の外）。
    fn link(&self) -> PathBuf {
        self.root.join("link-a")
    }

    fn proj(&self, project: &str) -> PathBuf {
        self.work.join(project)
    }

    fn anchor(&self, project: &str) -> String {
        self.proj(project).display().to_string()
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
    }

    /// 群の宣言の置き場を state-h にした読み（cwd は作業場の repo）。
    fn acct(&self) -> Acct {
        self.acct_at(self.program("scribe2"), self.state("state-h"), &self.repo)
    }

    fn acct_at(&self, scribe2: PathBuf, host: PathBuf, cwd: &Path) -> Acct {
        Acct::new(scribe2, self.program("git"), self.program("bd"), host, cwd)
    }

    /// 偽の器の `name` の出力に印を置く（`kind` は rc・crash）。
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

    /// 器と git が受けた argv の組を読んで消す。
    fn take_calls(&self) -> (Vec<String>, Vec<String>) {
        let calls = (self.calls("scribe2"), self.calls("git"));
        self.clear_calls();
        calls
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

    /// 偽の git と偽の器と偽の bd で server を立てる（`state_dir` が偽なら --state-dir の無い server）。
    fn config(&self, state_dir: bool) -> Config {
        Config {
            bd: self.program("bd").into(),
            state_dir: state_dir.then(|| self.state("state-h")),
            scribe2: self.program("scribe2").into(),
            ..Config::new(
                self.proj("proj-a"),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    fn serve(&self, state_dir: bool) -> SocketAddr {
        let server = Server::bind_with(&self.config(state_dir), self.program("git").as_os_str())
            .expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

fn to(to: &str) -> String {
    format!("{{\"to\":\"{to}\"}}")
}

fn named(project: &str, to: &str) -> String {
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

/// dir の下の拡張子 rs の file（再帰）。
fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.expect("dir の項").path();
        if path.is_dir() {
            rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn shb_wire_takes_direction_only() {
    assert_eq!(PATH, "/api/seat/heartbeat");
    assert_eq!(
        wire::encode(&SeatHeartbeatRequest { to: Heartbeat::Off }).expect("電文の字"),
        "{\"to\":\"off\"}"
    );
    assert_eq!(
        wire::decode::<SeatHeartbeatRequest>("{\"to\":\"on\"}").expect("値 on を読む"),
        SeatHeartbeatRequest { to: Heartbeat::On }
    );
    for text in UNREADABLE {
        assert!(
            wire::decode::<SeatHeartbeatRequest>(text).is_err(),
            "{text} が読める"
        );
    }
    // 引用符で囲んだ口の path は契約の型の crate の seathb.rs に 1 度だけ。
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates の dir");
    let needle = format!("\"{PATH}\"");
    let mut found = Vec::new();
    for krate in ["tsuzuri-contract", "tsuzuri-boundary", "tsuzuri-surface"] {
        let mut files = Vec::new();
        rs_files(&crates.join(krate).join("src"), &mut files);
        for file in files {
            let text = fs::read_to_string(&file).expect("rs の file");
            for _ in text.matches(&needle) {
                found.push(file.clone());
            }
        }
    }
    assert_eq!(
        found,
        [crates.join("tsuzuri-contract/src/seathb.rs")],
        "引用符で囲んだ口の path の在りか"
    );
}

#[test]
fn shb_route_listed() {
    let seathb = Route::ALL
        .iter()
        .find(|r| r.name() == "seathb")
        .expect("Route の ALL に seathb が無い");
    let key = seathb.key();
    assert_eq!(
        key,
        Key {
            method: "POST",
            path: Match::Exact(PATH),
        }
    );
    assert!(key.matches("POST", PATH));
    assert!(!key.matches("GET", PATH));
    assert!(!key.matches("POST", HEARTBEAT_PATH));
    let heartbeat = Route::ALL
        .iter()
        .find(|r| r.name() == "heartbeat")
        .expect("Route の ALL に heartbeat が無い");
    assert_eq!(
        heartbeat.key(),
        Key {
            method: "POST",
            path: Match::Exact(HEARTBEAT_PATH),
        }
    );
}

#[test]
fn shb_own_anchor_by_path() {
    let place = Place::new("own");
    let acct = place.acct();
    let sa = place.state("state-a").display().to_string();
    let off = place.off_file("state-a", "proj-a_0.1");

    let got = accept_own(&acct, &place.proj("proj-a"), &to("off"));
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
        )]
    );
    assert_eq!(
        fs::read_to_string(&off).ok().as_deref(),
        Some("ts=1790510400\n"),
        "器が停止の記録を置く"
    );

    place.clear_calls();
    assert_eq!(
        accept_own(&acct, &place.proj("proj-a"), &to("on")),
        ok("proj-a:0.1", Heartbeat::On)
    );
    assert!(!off.exists(), "器が停止の記録を消す");

    own_by_other_paths(place, acct);
}

/// 別の字の path・最後の名が同じ anchor・末尾の「/」の宣言でも、名でなく dir で anchor を引くことを見る。
fn own_by_other_paths(place: Place, acct: Acct) {
    // 同じ dir を指す別の字の path。
    for repo in [place.proj("proj-a").join("inner").join(".."), place.link()] {
        assert_eq!(
            accept_own(&acct, &repo, &to("off")),
            ok("proj-a:0.1", Heartbeat::Off),
            "{}",
            repo.display()
        );
    }

    // 最後の名が同じ anchor は、名でなく dir で引く。
    assert_eq!(
        accept_own(&acct, &place.other.join("proj-a"), &to("off")),
        ok("proj-z:0.1", Heartbeat::Off)
    );
    assert_eq!(
        accept(&acct, &named("proj-a", "off")),
        ok("proj-a:0.1", Heartbeat::Off)
    );

    // 宣言の末尾の「/」は同じ dir。
    place.clear_calls();
    let sb = place.state("state-b").display().to_string();
    assert_eq!(
        accept_own(&acct, &place.proj("proj-b"), &to("off")),
        ok("proj-b:0.1", Heartbeat::Off)
    );
    assert_eq!(
        place.calls("scribe2"),
        [
            format!("doctor --state-dir {sb}"),
            format!("seat heartbeat off --state-dir {sb} --target proj-b:0.1"),
        ]
    );
}

#[test]
fn shb_same_steps_as_named_body() {
    let place = Place::new("same");
    let acct = place.acct();
    for project in ["proj-a", "proj-b", "proj-c", "proj-d"] {
        for dir in ["off", "on"] {
            place.clear_calls();
            let own = accept_own(&acct, &place.proj(project), &to(dir));
            let own_calls = place.take_calls();
            let by_name = accept(&acct, &named(project, dir));
            let name_calls = place.take_calls();
            assert_eq!(own, by_name, "{project} {dir}");
            assert_eq!(own_calls, name_calls, "{project} {dir}");
            match project {
                "proj-c" => {
                    assert_eq!(own, refused(404, NO_SEAT));
                    assert!(own_calls.0.is_empty(), "proj-c で器を撃つ");
                }
                "proj-d" => {
                    assert_eq!(own, refused(404, NO_SEAT));
                    assert_eq!(
                        own_calls.0,
                        [format!(
                            "doctor --state-dir {}",
                            place.state("state-a").display()
                        )]
                    );
                }
                _ => assert_eq!(own.0, 200, "{project} {dir}"),
            }
        }
    }
}

#[test]
fn shb_refusals_before_vessel() {
    let place = Place::new("refuse");
    let acct = place.acct();
    let a = place.proj("proj-a");
    let evil = "{\"to\":\"off\",\"target\":\"evil:9.9\",\"state_dir\":\"/tmp/evil\"}";
    for text in UNREADABLE.into_iter().chain([evil]) {
        assert_eq!(accept_own(&acct, &a, text), refused(400, BAD_BODY), "{text}");
    }
    for repo in [place.repo.clone(), PathBuf::from("/"), place.proj("proj-x")] {
        assert_eq!(
            accept_own(&acct, &repo, &to("off")),
            refused(404, NO_PROJECT),
            "{}",
            repo.display()
        );
    }
    // 群の宣言が読めなければ、どの repo も無い。
    let none = place.acct_at(place.program("scribe2"), place.state("state-a"), &place.repo);
    assert_eq!(
        accept_own(&none, &a, &to("off")),
        refused(404, NO_PROJECT)
    );
    assert!(place.calls("scribe2").is_empty(), "器を撃つ");
    assert!(place.calls("git").is_empty(), "git を撃つ");
}

#[test]
fn shb_vessel_failures_are_502() {
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
            accept_own(&place.acct(), &place.proj("proj-a"), &to("off")),
            refused(502, VESSEL_FAILED),
            "{kind} {name}"
        );
        let heartbeats = place
            .calls("scribe2")
            .iter()
            .filter(|c| c.starts_with("seat heartbeat "))
            .count();
        assert_eq!(heartbeats, usize::from(name.starts_with("hb-")), "{kind} {name}");
    }
    // 器が撃てない。
    let place = Place::new("fail-no-vessel");
    let none = place.acct_at(
        PathBuf::from("/nonexistent/tz-no-such-scribe2"),
        place.state("state-h"),
        &place.repo,
    );
    assert_eq!(
        accept_own(&none, &place.proj("proj-a"), &to("off")),
        refused(502, VESSEL_FAILED)
    );
}

#[test]
fn shb_bytes_only_vessel_file() {
    let place = Place::new("bytes");
    let acct = place.acct();
    let off = place.off_file("state-a", "proj-a_0.1");
    let before = place.snapshot();
    // 断りは何も変えない。
    for repo in [place.repo.clone(), place.proj("proj-c"), place.proj("proj-d")] {
        accept_own(&acct, &repo, &to("off"));
        assert!(
            before == place.snapshot(),
            "{} の断りで byte が変わる",
            repo.display()
        );
    }
    accept_own(&acct, &place.proj("proj-a"), "{");
    assert!(before == place.snapshot(), "bad-body で byte が変わる");
    // off は器の書く停止の記録だけが増える。
    assert_eq!(
        accept_own(&acct, &place.proj("proj-a"), &to("off")).0,
        200
    );
    let after: Vec<_> = place
        .snapshot()
        .into_iter()
        .filter(|(p, _)| *p != off)
        .collect();
    assert!(before == after, "停止の記録の外の byte が変わる");
    assert!(off.exists());
    // on は器が消し、元の byte に戻る。
    assert_eq!(
        accept_own(&acct, &place.proj("proj-a"), &to("on")).0,
        200
    );
    assert!(before == place.snapshot(), "on の後に元の byte に戻らない");
}

/// 応答（状態の code・頭・本文）。
struct Reply {
    status: u16,
    head: String,
    body: String,
}

fn post(addr: SocketAddr, heads: &str, body: &str) -> Reply {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(
        format!(
            "POST {PATH} HTTP/1.1\r\nHost: {addr}\r\n{heads}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .as_bytes(),
    )
    .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse().ok())
        .expect("状態の code");
    Reply {
        status,
        head: head.to_string(),
        body: body.to_string(),
    }
}

#[test]
fn shb_http_guard_and_reply() {
    let place = Place::new("http");
    let addr = place.serve(true);
    let port = addr.port();
    for origin in [
        format!("http://evil.example:{port}"),
        format!("http://127.0.0.1:{}", port.wrapping_add(1)),
        "null".to_string(),
    ] {
        let reply = post(addr, &format!("Origin: {origin}\r\n"), &to("off"));
        assert_eq!((reply.status, reply.body.as_str()), (403, "origin"), "{origin}");
    }
    let big = format!("{{\"to\":\"off\",\"pad\":\"{}\"}}", "x".repeat(65_536));
    let reply = post(addr, "", &big);
    assert_eq!((reply.status, reply.body.as_str()), (413, "too-large"));
    assert!(place.calls("scribe2").is_empty(), "守りで器を撃つ");
    assert!(place.calls("git").is_empty(), "守りで git を撃つ");

    // 通った本文は同じ作業場の受付の関数の状態の数と本文をそのまま返す。
    let a = place.proj("proj-a");
    let direct = place.acct_at(place.program("scribe2"), place.state("state-h"), &a);
    for (heads, text) in [
        (format!("Origin: http://{addr}\r\n"), to("off")),
        (String::new(), to("on")),
        (String::new(), "{".to_string()),
        (
            String::new(),
            "{\"to\":\"off\",\"project\":\"proj-b\"}".to_string(),
        ),
    ] {
        let reply = post(addr, &heads, &text);
        let want = accept_own(&direct, &a, &text);
        assert_eq!((reply.status, reply.body.clone()), want, "{text}");
        let kind = if reply.status == 200 {
            "application/json"
        } else {
            "text/plain"
        };
        assert!(reply.head.contains(kind), "{text}: {}", reply.head);
    }
    let reply = post(addr, "", &to("off"));
    let got: HeartbeatResponse = wire::decode(&reply.body).expect("応答の電文");
    assert_eq!(
        got,
        HeartbeatResponse {
            target: "proj-a:0.1".to_string(),
            to: Heartbeat::Off
        }
    );
    let sa = place.state("state-a").display().to_string();
    assert_eq!(
        place.calls("scribe2").last().map(String::as_str),
        Some(format!("seat heartbeat off --state-dir {sa} --target proj-a:0.1").as_str())
    );

    // state dir の無い server は受付の関数を呼ばずに 404 no-project。
    let bare = place.serve(false);
    place.clear_calls();
    let reply = post(bare, "", &to("off"));
    assert_eq!((reply.status, reply.body.as_str()), (404, NO_PROJECT));
    assert!(place.calls("scribe2").is_empty());
    assert!(place.calls("git").is_empty());
}

#[test]
fn shb_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth4/shb.rs");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut names = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let head = lines.next().expect("test の属性の後の行");
        let name = head
            .trim()
            .strip_prefix("fn ")
            .and_then(|rest| rest.split_once('('))
            .map(|(name, _)| name)
            .unwrap_or_else(|| panic!("test の属性の後が fn でない: {head}"));
        names.push(name.to_string());
    }
    assert_eq!(names.len(), 9, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("shb_")
            .unwrap_or_else(|| panic!("{name} が shb_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
