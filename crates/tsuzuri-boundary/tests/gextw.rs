//! 不変条件 g-3 の外の台帳の境界の歯（接頭辞 gext_・行 c-g3-extern）: tz graph --check の --project と、
//! 口 GET /api/graph の Config の欄 projects。歯ごとの作業場（CARGO_TARGET_TMPDIR の下の gextw の dir）に自分の repo と
//! 3 つの置き場（far・cut・gone）と偽の bd（撃たれた cwd の ledger.json を出す・無ければ rc が 0 でない）と
//! 偽の設計の道具（引数に --emit-rulings が在れば書き出しの字・--summary が在れば rc 1・ほかは索引の字を返す）を置く。
//! 書き出しは fixture の書き出しに、族 s9-far を引く ADR-8 の行（bead の形で s9-far.2）を足した字。
//! 歯の名の検査は tsuzuri-core の tests/gext.rs が持つ。

use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use tsuzuri_boundary::cli::graph::{NEXT, OUTSIDE_HEAD};
use tsuzuri_boundary::server::board::{self, Texts};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::graph::{GraphDoc, Verdict};
use tsuzuri_contract::wire;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/graph/unruled")
        .join(name)
}

fn read(name: &str) -> String {
    fs::read_to_string(fixture(name)).unwrap_or_else(|e| panic!("{name} を読む: {e}"))
}

/// 置き場 far の台帳（s9-far.2 を持つ）。
const FAR: &str = r#"[
{"id": "s9-far", "title": "外の根", "status": "open", "issue_type": "epic"},
{"id": "s9-far.2", "title": "外の契約", "status": "closed", "issue_type": "task"}
]"#;

/// 置き場 cut の台帳（族 s9-far の bead を持つが、s9-far.2 を持たない）。
const CUT: &str = r#"[
{"id": "s9-far", "title": "外の根", "status": "open", "issue_type": "epic"},
{"id": "s9-far.3", "title": "外の契約", "status": "closed", "issue_type": "task"}
]"#;

/// 書き出しに node ADR-8・form bead・bead s9-far.2 の行を足した字。
fn rulings() -> String {
    format!(
        "{}{{\"ruling\":\"s9-far.2\",\"form\":\"bead\",\"bead\":\"s9-far.2\",\"node\":\"ADR-8\",\"file\":\"adr/ADR-8.yaml\",\"line\":9,\"field\":\"approval.ruling\"}}\n",
        read("rulings.jsonl")
    )
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
    far: PathBuf,
    cut: PathBuf,
    gone: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("gextw")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, state) = (root.join("repo"), root.join("state"));
        let (far, cut, gone) = (root.join("far"), root.join("cut"), root.join("gone"));
        for dir in [&repo, &far, &cut, &gone] {
            fs::create_dir_all(dir.join(".beads")).expect("置き場");
            fs::write(dir.join(".beads/issues.jsonl"), "{}\n").expect("印の file");
        }
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::write(state.join("fleet/events.jsonl"), read("events.jsonl")).expect("event log");
        fs::write(repo.join("ledger.json"), read("ledger.json")).expect("自分の台帳");
        fs::write(far.join("ledger.json"), FAR).expect("far の台帳");
        fs::write(cut.join("ledger.json"), CUT).expect("cut の台帳");
        fs::write(root.join("rulings.jsonl"), rulings()).expect("書き出しの字");
        script(&root.join("bd"), "exec cat ledger.json");
        script(
            &root.join("folio"),
            &format!(
                "for a in \"$@\"; do\n\
                 [ \"$a\" = --emit-rulings ] && {{ cat '{}'; exit 0; }}\n\
                 [ \"$a\" = --summary ] && exit 1\n\
                 done\n\
                 exec cat '{}'",
                root.join("rulings.jsonl").display(),
                fixture("index.tsv").display()
            ),
        );
        Place {
            root,
            repo,
            state,
            far,
            cut,
            gone,
        }
    }

    /// tz graph --check を、引数（--project ほか）を足して撃つ。
    fn check(&self, extra: &[OsString]) -> Output {
        let mut args: Vec<OsString> = vec![
            "graph".into(),
            "--check".into(),
            "--repo".into(),
            self.repo.clone().into(),
            "--bd".into(),
            self.root.join("bd").into(),
            "--folio".into(),
            self.root.join("folio").into(),
            "--state-dir".into(),
            self.state.clone().into(),
        ];
        args.extend(extra.iter().cloned());
        Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .output()
            .expect("tz を撃つ")
    }

    /// server を同じ process の thread で立てて口の住所を返す。
    fn serve(&self, projects: Vec<PathBuf>) -> SocketAddr {
        let files = self.root.join("files");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        let config = Config {
            bd: self.root.join("bd").into(),
            folio: self.root.join("folio").into(),
            state_dir: Some(self.state.clone()),
            projects,
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                files,
            )
        };
        let server = Server::bind(&config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

fn project(dir: &Path) -> OsString {
    OsString::from(format!("--project={}", dir.display()))
}

fn pair(dir: &Path) -> Vec<OsString> {
    vec!["--project".into(), dir.into()]
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("標準出力の字")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("標準エラーの字")
}

fn next(id: &str) -> &'static str {
    NEXT.iter()
        .find(|(i, _)| *i == id)
        .map(|(_, n)| *n)
        .expect("次の 1 手")
}

/// 口 GET /api/graph の電文。
fn graph_doc(addr: SocketAddr) -> GraphDoc {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(30)))
        .expect("timeout");
    s.write_all(
        format!("GET /api/graph HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n").as_bytes(),
    )
    .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");
    wire::decode(body).expect("導出グラフの電文")
}

/// (4) tz graph --check の --project。
#[test]
fn gext_check_takes_projects() {
    let place = Place::new("check");
    let head = format!("# まだ分からない: [g-3] {OUTSIDE_HEAD} 1（族 s9-far）");
    let unverified = |out: &Output| stderr(out).lines().any(|l| l == head);

    let out = place.check(&[]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(unverified(&out), "{}", stderr(&out));
    assert!(!stdout(&out).contains("[g-3]"), "{out:?}");

    let out = place.check(&pair(&place.far));
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(!stdout(&out).contains("[g-3]"), "{out:?}");
    assert!(!stderr(&out).contains("[g-3]"), "{out:?}");

    let out = place.check(&[project(&place.gone), project(&place.far)]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(!stdout(&out).contains("[g-3]"), "{out:?}");
    assert!(!stderr(&out).contains("[g-3]"), "{out:?}");

    let out = place.check(&[project(&place.gone)]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(unverified(&out), "{}", stderr(&out));

    let out = place.check(&pair(&place.cut));
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(
        stdout(&out).starts_with(&format!("[g-3] 違反 1（ADR-8） next={}\n", next("g-3"))),
        "{out:?}"
    );
    assert!(!stderr(&out).contains("[g-3]"), "{out:?}");

    let out = place.check(&[project(Path::new(""))]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(stderr(&out).contains("--project の値が空"), "{out:?}");
}

/// (5) 口 GET /api/graph の Config の欄 projects と、境界の board の graph。
#[test]
fn gext_route_reads_projects() {
    let place = Place::new("route");
    let g3 = |projects: Vec<PathBuf>| {
        let doc = graph_doc(place.serve(projects));
        let inv = doc
            .invariants
            .iter()
            .find(|i| i.id == "g-3")
            .expect("g-3")
            .clone();
        (inv.verdict, inv.ids)
    };
    let (place_far, place_cut, place_gone) =
        (place.far.clone(), place.cut.clone(), place.gone.clone());
    let none: Vec<String> = Vec::new();
    assert_eq!(g3(vec![]), (Verdict::Unknown, none.clone()));
    assert_eq!(g3(vec![place_gone.clone()]), (Verdict::Unknown, none.clone()));
    assert_eq!(g3(vec![place_far.clone()]), (Verdict::Pass, none.clone()));
    assert_eq!(g3(vec![place_gone, place_far]), (Verdict::Pass, none));
    assert_eq!(
        g3(vec![place_cut]),
        (Verdict::Violation, vec!["ADR-8".to_string()])
    );

    let texts = Texts {
        design: read("index.tsv"),
        ledger: read("ledger.json"),
        events: read("events.jsonl"),
        rulings: rulings(),
        summary: String::new(),
    };
    assert_eq!(board::graph(&texts), board::graph_outside(&texts, Vec::new()));
}
