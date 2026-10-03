//! 面の file の応答の頭 Cache-Control の歯（接頭辞 ecache_・設計ノート surface-wave11b 行 e-cache の完了の条件）。
//! 名に hash を持つ file は 1 年持たせ（`files::IMMUTABLE`）、index.html とほかの file は毎回確かめさせる（`files::NO_CACHE`）。
//! server は同じ process の thread で 127.0.0.1 の空き port に立て、要求は素の TCP で撃つ。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use tsuzuri_boundary::server::files::{IMMUTABLE, NO_CACHE, hashed, respond};
use tsuzuri_boundary::server::http::{Response, read_request};
use tsuzuri_boundary::server::{Config, Server};

const INDEX: &str = "<!doctype html><title>tz</title>";
const CSS: &str = "style-13a5b5bf849cd686.css";
const WASM: &str = "tsuzuri-surface-2e993b6ea1850f19_bg.wasm";
const JS: &str = "tsuzuri-surface-2e993b6ea1850f19.js";

#[test]
fn ecache_hashed_names() {
    for name in [
        WASM,
        JS,
        CSS,
        "style-0123abcd.css",
        "a-0123456789abcdef.js",
    ] {
        assert!(hashed(name), "{name}");
    }
    for name in [
        "index.html",
        "app.js",
        "tsuzuri-surface.js",
        "tsuzuri-surface_bg.wasm",
        "style-0123abc.css",
        "style-0123456789abcdef0.css",
        "style-13A5B5BF849CD686.css",
        "style-13a5b5bf849cd68g.css",
        "-13a5b5bf849cd686.css",
        "style-13a5b5bf849cd686",
        "",
    ] {
        assert!(!hashed(name), "{name}");
    }
}

/// 歯ごとの作業場（repo の置き場と面の file の置き場と置き場の外の秘密と偽の bd）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("ecache")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let files = root.join("files");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(files.join("sub")).expect("面の file の置き場");
        fs::write(files.join("index.html"), INDEX).expect("index.html");
        fs::write(files.join(CSS), "body{}").expect("css");
        fs::write(files.join(WASM), "\0asm").expect("wasm");
        fs::write(files.join(JS), "export{}").expect("js");
        fs::write(files.join("sub/app.js"), "let a;").expect("sub/app.js");
        fs::write(root.join("secret.txt"), "SECRET").expect("secret.txt");
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

/// 名が Cache-Control の頭の値の列。
fn cache(resp: &Response) -> Vec<&str> {
    resp.headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("Cache-Control"))
        .map(|(_, value)| value.as_str())
        .collect()
}

#[test]
fn ecache_respond_heads() {
    let place = Place::new("respond");
    let root = place.files.canonicalize().expect("置き場の実体");
    let get = |path: &str| {
        let raw = format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n");
        let req = read_request(raw.as_bytes()).expect(path);
        respond(&req, &root)
    };
    for (path, content_type, body) in [
        ("/", "text/html; charset=utf-8", INDEX.as_bytes()),
        ("/index.html", "text/html; charset=utf-8", INDEX.as_bytes()),
        (
            "/sub/app.js?v=1",
            "text/javascript; charset=utf-8",
            b"let a;".as_slice(),
        ),
    ] {
        let resp = get(path);
        assert_eq!(resp.status, 200, "{path}");
        assert_eq!(resp.content_type, content_type, "{path}");
        assert_eq!(resp.body, body, "{path}");
        assert_eq!(cache(&resp), [NO_CACHE], "{path}");
    }
    for (path, content_type) in [
        ("/style-13a5b5bf849cd686.css", "text/css; charset=utf-8"),
        (
            "/tsuzuri-surface-2e993b6ea1850f19_bg.wasm?v=1",
            "application/wasm",
        ),
        (
            "/tsuzuri-surface-2e993b6ea1850f19.js",
            "text/javascript; charset=utf-8",
        ),
    ] {
        let resp = get(path);
        assert_eq!(resp.status, 200, "{path}");
        assert_eq!(resp.content_type, content_type, "{path}");
        assert_eq!(cache(&resp), [IMMUTABLE], "{path}");
    }
    for (path, status, body) in [
        ("/nope-13a5b5bf849cd686.wasm", 404, "no-file"),
        ("/sub/", 404, "no-file"),
        ("/../secret.txt", 403, "outside"),
        ("/%2e%2e/secret.txt", 403, "outside"),
    ] {
        let resp = get(path);
        assert_eq!(resp.status, status, "{path}");
        assert_eq!(resp.body, body.as_bytes(), "{path}");
        assert_eq!(resp.content_type, "text/plain; charset=utf-8", "{path}");
        assert!(cache(&resp).is_empty(), "{path}");
    }
}

/// GET を撃ち、応答の頭の行（1 行目を含む）と本文を返す（読みの上限 5 秒）。
fn get(addr: SocketAddr, path: &str) -> (Vec<String>, Vec<u8>) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let end = out
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("頭と本文");
    let head = String::from_utf8(out[..end].to_vec()).expect("頭の字");
    let lines = head.split("\r\n").map(str::to_owned).collect();
    (lines, out[end + 4..].to_vec())
}

/// 頭の行のうち名が Cache-Control の行の値。
fn cache_lines(head: &[String]) -> Vec<&str> {
    head.iter()
        .filter_map(|l| l.split_once(':'))
        .filter(|(name, _)| name.trim().eq_ignore_ascii_case("Cache-Control"))
        .map(|(_, value)| value.trim())
        .collect()
}

#[test]
fn ecache_served_over_tcp() {
    let place = Place::new("tcp");
    let addr = place.serve();

    let (head, body) = get(addr, "/");
    assert_eq!(head[0], "HTTP/1.1 200 OK");
    assert_eq!(body, INDEX.as_bytes());
    assert_eq!(cache_lines(&head), ["no-cache"]);

    let (head, _) = get(addr, &format!("/{WASM}"));
    assert_eq!(head[0], "HTTP/1.1 200 OK");
    assert!(
        head.iter().any(|l| l == "Content-Type: application/wasm"),
        "{head:?}"
    );
    assert_eq!(cache_lines(&head), [IMMUTABLE]);

    let (head, _) = get(addr, "/nope.wasm");
    assert_eq!(head[0], "HTTP/1.1 404 Not Found");
    assert!(cache_lines(&head).is_empty(), "{head:?}");

    let (head, _) = get(addr, "/api/ledger");
    assert_eq!(head[0], "HTTP/1.1 200 OK");
    assert_eq!(cache_lines(&head), ["no-store"]);
}

#[test]
fn ecache_handle_wiring_text() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = fs::read_to_string(dir.join("src/server/mod.rs")).expect("mod.rs");
    let handle = src.find("fn handle(").expect("fn handle(");
    let wired = handle
        + src[handle..]
            .find("None => files::respond(&req, &shared.files),")
            .expect("配布の 1 行");
    assert!(src[wired..].contains("fn guarded<"), "fn guarded< が後に無い");
    assert_eq!(src.matches("files::respond(").count(), 1);
    for gone in [
        "Served",
        "files::serve(",
        "Response::new(200",
        "Cache-Control",
        "\"outside\"",
        "\"no-file\"",
    ] {
        assert!(!src.contains(gone), "mod.rs が {gone} を持つ");
    }
    let files = fs::read_to_string(dir.join("src/server/files.rs")).expect("files.rs");
    assert!(
        files
            .lines()
            .any(|l| l == "pub fn respond(req: &Request, root: &Path) -> Response {"),
        "respond の宣言の行"
    );
    for once in ["\"outside\"", "\"no-file\"", "\"Cache-Control\""] {
        assert_eq!(files.matches(once).count(), 1, "files.rs の {once}");
    }
}

/// 着地済みの行の verify の filter の語と、後の行の接頭辞（新しい歯の名はこのどれも含まない）。
const WORDS: &[&str] = &[
    "aaround_", "accept_", "account_", "acctcore_", "acctdoc_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "afocus_", "aord_", "apop_", "askcard_", "athr_", "batchpanel_", "board_min_", "bport_",
    "brand_", "btuck_", "cadopt_", "cgdom_", "cmark_", "contract_form_", "cround_", "csled_",
    "denv_", "flight_", "fmark_", "frame_", "fserve_", "fstop_", "gapspage_", "gfresh_", "ghb_",
    "glabel_", "gnav_", "graph_", "gsum_", "gtuck_", "gview_", "hbconf_", "hbpost_", "hbproc_",
    "hbroute_", "hcard_", "hcled_", "hcnx_", "hcproj_", "hcsess_", "hfig_", "hook_", "hruling_",
    "hsblock_", "hsderive_", "hspage_", "hsym_", "iclose_", "ilink_", "kcli_", "klink_",
    "lcard_", "ledgerblock_", "lhome_", "lspark_", "mapview_", "mkeys_", "mlink_", "mstore_",
    "mtree_", "nact_", "nbatch_", "ncard_", "nextstep_", "nodepage_", "nstall_", "nsum_",
    "nsumw_", "ntime_", "nxact_", "parts_", "pclosed_", "pfold_", "pipe_", "plimit_", "pmore_",
    "project_", "ptitle_", "pwhole_", "qblock_", "qgate_", "qkey_", "question_", "rhold_",
    "runsdoc_", "saxis_", "seatblock_", "seatcard_", "server_", "sesplit_", "shb_", "skeleton_",
    "smore_", "stage_", "stats_", "steady_", "sxaxis_", "ticker_", "tipx_", "topbar_", "tz_",
    "urpanel_", "uword_", "wstrip_", "pgz_", "pquest_", "sclosed_",
];

#[test]
fn ecache_own_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/ecache.rs"))
        .expect("ecache.rs");
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
        let rest = name.strip_prefix("ecache_").expect(name);
        for word in WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
