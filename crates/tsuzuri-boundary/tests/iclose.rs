//! 1 byte も送らない接続を黙って閉じる歯（接頭辞 iclose_・設計ノート surface-wave10b 行 e-idle-close の完了の条件）。
//! server は同じ process の thread で 127.0.0.1 の空き port に立て、要求は素の TCP で撃つ。
//! 10 秒の読みの時間切れは待たない（時間切れの道は読み手の WouldBlock と TimedOut と mod.rs の字で見る）。

use std::collections::VecDeque;
use std::fs;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use tsuzuri_boundary::server::http::{no_bytes, read_request};
use tsuzuri_boundary::server::{Config, Server};

const INDEX: &str = "<!doctype html><title>tz</title>";

/// 読み手の 1 歩（byte の塊か誤りの種類）。
type Step<'a> = Result<&'a [u8], ErrorKind>;

/// 順に byte の塊か誤りを返し、尽きたら 0 byte を返す読み手。
struct Script(VecDeque<Result<Vec<u8>, ErrorKind>>);

impl Script {
    fn new(steps: Vec<Step>) -> Script {
        Script(steps.into_iter().map(|s| s.map(<[u8]>::to_vec)).collect())
    }
}

impl Read for Script {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.0.pop_front() {
            None => Ok(0),
            Some(Err(kind)) => Err(io::Error::from(kind)),
            Some(Ok(mut bytes)) => {
                let n = bytes.len().min(buf.len());
                buf[..n].copy_from_slice(&bytes[..n]);
                if n < bytes.len() {
                    self.0.push_front(Ok(bytes.split_off(n)));
                }
                Ok(n)
            }
        }
    }
}

#[test]
fn iclose_no_bytes_kinds() {
    let silent: Vec<(&str, Vec<Step>)> = vec![
        ("尽きる", vec![]),
        ("WouldBlock", vec![Err(ErrorKind::WouldBlock)]),
        ("TimedOut", vec![Err(ErrorKind::TimedOut)]),
        ("ConnectionReset", vec![Err(ErrorKind::ConnectionReset)]),
    ];
    for (what, steps) in silent {
        let e = read_request(Script::new(steps)).expect_err(what);
        assert!(no_bytes(&e), "{what}: {e:?}");
    }
    let loud: Vec<(&str, Vec<Step>)> = vec![
        (
            "GET / HT の後に WouldBlock",
            vec![Ok(b"GET / HT"), Err(ErrorKind::WouldBlock)],
        ),
        ("GARBAGE", vec![Ok(b"GARBAGE\r\n\r\n")]),
        ("CR LF だけ", vec![Ok(b"\r\n")]),
        ("要求の行だけ", vec![Ok(b"GET / HTTP/1.1\r\n")]),
        ("UTF-8 でない byte", vec![Ok(&[0xFF, 0xFE])]),
        (
            "本文の切れ",
            vec![Ok(b"POST / HTTP/1.1\r\nContent-Length: 5\r\n\r\n{}")],
        ),
    ];
    for (what, steps) in loud {
        let e = read_request(Script::new(steps)).expect_err(what);
        assert!(!no_bytes(&e), "{what}: {e:?}");
    }
    let req = read_request(Script::new(vec![
        Err(ErrorKind::Interrupted),
        Ok(b"GET / HTTP/1.1\r\n\r\n"),
    ]))
    .expect("Interrupted は読み直す");
    assert_eq!(req.method, "GET");
    assert_eq!(req.path(), "/");
    for outside in [
        io::Error::from(ErrorKind::WouldBlock),
        io::Error::from(ErrorKind::UnexpectedEof),
        io::Error::new(ErrorKind::InvalidData, "外で作った誤り"),
    ] {
        assert!(!no_bytes(&outside), "{outside:?}");
    }
}

/// 歯ごとの作業場（repo の置き場と面の file の置き場と偽の bd）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("iclose")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let files = root.join("files");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::write(files.join("index.html"), INDEX).expect("index.html");
        let bd = root.join("bd");
        fs::write(&bd, "#!/bin/sh\necho '[]'\n").expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
        Place { root, repo, files }
    }

    fn config(&self) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    /// server を立てて口の住所を返す。
    fn serve(&self) -> SocketAddr {
        let server = Server::bind(&self.config()).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

/// 字を書き（`close` なら書きの側を閉じ）、接続が閉じるまで応答を読む（読みの上限 5 秒）。
fn exchange(addr: SocketAddr, raw: &[u8], close: bool) -> Vec<u8> {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout");
    s.write_all(raw).expect("要求を書く");
    if close {
        s.shutdown(Shutdown::Write).expect("書きの側を閉じる");
    }
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    out
}

#[test]
fn iclose_eof_writes_nothing() {
    let place = Place::new("eof");
    let addr = place.serve();
    let out = exchange(addr, b"", true);
    assert!(out.is_empty(), "{}", String::from_utf8_lossy(&out));
    let reply = String::from_utf8(exchange(addr, b"GET / HTTP/1.1\r\nHost: x\r\n\r\n", false))
        .expect("応答の字");
    let (head, body) = reply.split_once("\r\n\r\n").expect("頭と本文");
    assert!(head.starts_with("HTTP/1.1 200 OK\r\n"), "{head}");
    assert_eq!(body, INDEX);
}

#[test]
fn iclose_broken_keeps_400() {
    let place = Place::new("broken");
    let addr = place.serve();
    for (raw, close) in [
        (&b"GARBAGE\r\n\r\n"[..], false),
        (b"GET / HTTP/1.1\r\n", true),
        (b"G", true),
        (
            b"POST /api/ruling HTTP/1.1\r\nContent-Length: 5\r\n\r\n{}",
            true,
        ),
    ] {
        let what = String::from_utf8_lossy(raw);
        let reply = String::from_utf8(exchange(addr, raw, close)).expect("応答の字");
        let (head, body) = reply.split_once("\r\n\r\n").expect("頭と本文");
        assert_eq!(
            head.lines().next(),
            Some("HTTP/1.1 400 Bad Request"),
            "{what}"
        );
        assert_eq!(body, "bad-request", "{what}");
    }
}

#[test]
fn iclose_handle_wiring_text() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = fs::read_to_string(dir.join("src/server/mod.rs")).expect("mod.rs");
    let bad = "Response::text(400, \"bad-request\")";
    let handle = src.find("fn handle(").expect("fn handle(");
    let wired = handle + src[handle..].find("http::no_bytes(").expect("http::no_bytes(");
    let answer = wired + src[wired..].find(bad).expect("400 の腕");
    assert!(src[answer..].contains("fn guarded<"), "fn guarded< が後に無い");
    assert_eq!(src.matches("http::no_bytes(").count(), 1);
    assert_eq!(src.matches("\"bad-request\"").count(), 1);
    let timeout = src
        .lines()
        .find(|l| l.starts_with("const READ_TIMEOUT"))
        .expect("const READ_TIMEOUT の行");
    assert!(timeout.contains("Duration::from_secs(10);"), "{timeout}");
    let toml = fs::read_to_string(dir.join("Cargo.toml")).expect("Cargo.toml");
    let deps: Vec<&str> = toml
        .lines()
        .map(str::trim)
        .skip_while(|l| *l != "[dependencies]")
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split('=').next().unwrap_or("").trim())
        .collect();
    assert_eq!(deps, ["tsuzuri-contract", "tsuzuri-core"]);
}

/// 着地済みの行の verify の filter の語と、後の行の接頭辞（新しい歯の名はこのどれも含まない）。
const WORDS: &[&str] = &[
    "aaround_", "accept_", "account_", "acctcore_", "acctdoc_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "afocus_", "aord_", "apop_", "askcard_", "athr_", "batchpanel_", "board_min_", "bport_",
    "brand_", "btuck_", "cadopt_", "cgdom_", "contract_form_", "cround_", "csled_", "flight_",
    "fmark_", "frame_", "fserve_", "fstop_", "gapspage_", "ghb_", "gnav_", "graph_", "gsum_",
    "gtuck_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hcard_", "hcproj_",
    "hcsess_", "hfig_", "hook_", "hruling_", "hsblock_", "hsderive_", "hspage_", "hsym_",
    "ilink_", "kcli_", "klink_", "lcard_", "ledgerblock_", "lhome_", "lspark_", "mapview_",
    "mkeys_", "mlink_", "mstore_", "mtree_", "nact_", "nbatch_", "ncard_", "nextstep_",
    "nodepage_", "nstall_", "nsum_", "nsumw_", "ntime_", "nxact_", "parts_", "pclosed_",
    "pfold_", "pgz_", "pipe_", "plimit_", "pmore_", "project_", "ptitle_", "pwhole_", "qblock_",
    "qgate_", "qkey_", "question_", "rhold_", "runsdoc_", "saxis_", "seatblock_", "seatcard_",
    "server_", "sesplit_", "shb_", "skeleton_", "smore_", "stage_", "stats_", "steady_",
    "sxaxis_", "ticker_", "tipx_", "topbar_", "tz_", "urpanel_", "uword_", "wstrip_",
    "glabel_",
];

#[test]
fn iclose_own_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/iclose.rs"))
        .expect("iclose.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .map(|w| {
            w[1].strip_prefix("fn ")
                .and_then(|l| l.split_once('('))
                .map(|(name, _)| name)
                .expect("test の属性の次の fn")
        })
        .collect();
    assert!(names.len() >= 5, "{names:?}");
    for name in names {
        let rest = name.strip_prefix("iclose_").expect(name);
        for word in WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
