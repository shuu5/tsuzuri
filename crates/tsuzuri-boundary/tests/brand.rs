//! header の題の project の名の口の歯（接頭辞 brand_・設計ノート surface-wave3b 行 g-brand の完了の条件）。
//! server は同じ process の thread で 127.0.0.1 の空き port に立て、GET を素の TCP で撃つ。
//! 3 つの crate の src と自分の file の字は CARGO_MANIFEST_DIR から読む。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use tsuzuri_boundary::server::route::{Key, Match};
use tsuzuri_boundary::server::{Config, Route, Server};
use tsuzuri_contract::project::{PATH, ProjectName};
use tsuzuri_contract::wire;

/// 着地済みの行とこの波の行の verify の filter の語（この行の接頭辞は並べない）。
const FILTERS: [&str; 65] = [
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
    "runsdoc_",
    "nbatch_",
];

/// 名 proj-kiri の電文の字。
const KIRI_WIRE: &str = r#"{"name":"proj-kiri"}"#;

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 歯ごとの作業場（dir proj-kiri とその下の inner・面の file の置き場・偽の bd）。
struct Place {
    root: PathBuf,
    files: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("brand")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let files = root.join("files");
        for dir in [root.join("proj-kiri/inner"), files.clone()] {
            fs::create_dir_all(dir).expect("置き場");
        }
        fs::write(files.join("index.html"), "tz").expect("index.html");
        let bd = root.join("bd");
        fs::write(&bd, "#!/bin/sh\necho '[]'\n").expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
        Place { root, files }
    }

    /// repo を `repo` にした server を立てて口の住所を返す。
    fn serve(&self, repo: PathBuf) -> SocketAddr {
        let config = Config {
            bd: self.root.join("bd").into(),
            ..Config::new(
                repo,
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        };
        let server = Server::bind(&config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

/// GET を 1 つ撃ち、（状態の code・本文）を返す。
fn get(addr: SocketAddr, path: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
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
    (status, body.to_string())
}

/// dir の下の拡張子 rs の file（深さを問わない）。
fn rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).unwrap_or_else(|e| panic!("{}: {e}", d.display())) {
            let path = entry.expect("dir の項").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn brand_name_is_repo_dir() {
    let place = Place::new("kiri");
    let addr = place.serve(place.root.join("proj-kiri").join("inner").join(".."));
    let (status, body) = get(addr, PATH);
    assert_eq!(status, 200, "{body}");
    assert_eq!(body, KIRI_WIRE);
    let got: ProjectName = wire::decode(&body).expect("名の電文の形");
    assert_eq!(got.name, "proj-kiri");
}

#[test]
fn brand_no_name_is_503() {
    let place = Place::new("root");
    let addr = place.serve(PathBuf::from("/"));
    assert_eq!(get(addr, PATH), (503, "no-name".to_string()));
}

#[test]
fn brand_route_is_listed() {
    let route = Route::ALL
        .iter()
        .find(|r| r.name() == "project")
        .expect("Route の ALL に名 project の口が無い");
    assert_eq!(
        route.key(),
        Key {
            method: "GET",
            path: Match::Exact(PATH),
        }
    );
}

#[test]
fn brand_path_and_wire_words() {
    assert_eq!(PATH, "/api/project");
    let name = ProjectName {
        name: "proj-kiri".to_string(),
    };
    assert_eq!(wire::encode(&name).expect("電文"), KIRI_WIRE);
    assert_eq!(wire::decode::<ProjectName>(KIRI_WIRE).expect("読む"), name);
}

#[test]
fn brand_path_written_once_in_contract() {
    let quoted = format!("\"{PATH}\"");
    let mut found = Vec::new();
    for krate in ["tsuzuri-contract", "tsuzuri-boundary", "tsuzuri-surface"] {
        let src = manifest("..").join(krate).join("src");
        for file in rs_files(&src) {
            let n = read(&file).matches(&quoted).count();
            if n > 0 {
                found.push((
                    file.strip_prefix(manifest(".."))
                        .expect("crates の下")
                        .display()
                        .to_string(),
                    n,
                ));
            }
        }
    }
    assert_eq!(
        found,
        [("tsuzuri-contract/src/project.rs".to_string(), 1)],
        "引用符の {quoted} の在りか"
    );
}

#[test]
fn brand_own_names_clean() {
    let text = read(&manifest("tests/brand.rs"));
    let mut names = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines.next().expect("test の属性の次の行");
        let name = decl
            .trim()
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("fn の宣言でない: {decl}"));
        names.push(name.to_string());
    }
    assert!(names.len() >= 6, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("brand_")
            .unwrap_or_else(|| panic!("{name} が brand_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
