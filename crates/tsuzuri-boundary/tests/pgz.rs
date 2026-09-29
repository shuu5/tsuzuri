//! 面の file の gzip の写しの配りの歯（接頭辞 pgz_・設計ノート surface-wave19d 行 g-gz の完了の条件）。
//! 写しの在る file には頭 Vary を足し、要求の Accept-Encoding が gzip を受ければ写しの中身を頭 Content-Encoding と返す。

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::server::files::{
    GZ_SUFFIX, IMMUTABLE, NO_CACHE, accepts_gzip, gz_copy, respond,
};
use tsuzuri_boundary::server::http::{Response, read_request};

#[test]
fn pgz_accepts_gzip_table() {
    for header in [
        "gzip",
        "GZIP",
        "gzip, deflate, br, zstd",
        "gzip;q=0.001",
        "gzip;q=1.0",
        " gzip ; Q=0.5 ",
        "gzip;q=0, gzip",
        "identity;q=1, gzip;level=9",
    ] {
        assert!(accepts_gzip(Some(header)), "{header:?}");
    }
    assert!(!accepts_gzip(None), "字の無い頭");
    for header in [
        "",
        "deflate, br",
        "gzip;q=0",
        "gzip; q=0.000",
        "gzip;q=abc",
        "br;q=1, gzip;q=0",
        "x-gzip",
        "*",
        "gzipx, deflate",
    ] {
        assert!(!accepts_gzip(Some(header)), "{header:?}");
    }
}

#[test]
fn pgz_request_reads_encoding() {
    for (line, want) in [
        ("Accept-Encoding:  gzip, br ", Some("gzip, br")),
        ("accept-encoding: gzip;q=0", Some("gzip;q=0")),
        ("Accept-Language: ja", None),
    ] {
        let raw = format!("GET /x.wasm HTTP/1.1\r\nHost: h\r\n{line}\r\n\r\n");
        let req = read_request(raw.as_bytes()).expect(line);
        assert_eq!(req.accept_encoding.as_deref(), want, "{line}");
        assert_eq!(req.host.as_deref(), Some("h"), "{line}");
    }
}

const INDEX: &[u8] = b"<html></html>";
const WASM: &[u8] = b"\0asm wasm";
const WASM_GZ: &[u8] = b"\x1f\x8b wasm copy";
const JS: &[u8] = b"export {}";
const CSS: &[u8] = b"body{}";
const CSS_GZ: &[u8] = b"\x1f\x8b css copy";
const EVIL: &[u8] = b"let evil;";

/// 歯の置き場（面の file の置き場と、置き場の外の file）。
fn place(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("pgz").join(name);
    let _ = fs::remove_dir_all(&dir);
    let files = dir.join("files");
    fs::create_dir_all(&files).expect("置き場");
    for (file, body) in [
        ("index.html", INDEX),
        ("app-0123abcd_bg.wasm", WASM),
        ("app-0123abcd_bg.wasm.gz", WASM_GZ),
        ("app-0123abcd.js", JS),
        ("style.css", CSS),
        ("style.css.gz", CSS_GZ),
        ("evil.js", EVIL),
    ] {
        fs::write(files.join(file), body).expect(file);
    }
    fs::write(dir.join("secret.gz"), "SECRET").expect("置き場の外の file");
    symlink(dir.join("secret.gz"), files.join("evil.js.gz")).expect("外を指す写し");
    files.canonicalize().expect("置き場の実体")
}

fn get(root: &Path, path: &str, encoding: Option<&str>) -> Response {
    let line = encoding.map_or(String::new(), |e| format!("Accept-Encoding: {e}\r\n"));
    let raw = format!("GET {path} HTTP/1.1\r\nHost: x\r\n{line}\r\n");
    respond(&read_request(raw.as_bytes()).expect(path), root)
}

/// 要求（path と頭 Accept-Encoding）と応答（Content-Type・本文・頭の名と値の列）。
type Case = (
    &'static str,
    Option<&'static str>,
    &'static str,
    &'static [u8],
    Vec<(&'static str, String)>,
);

#[test]
fn pgz_respond_picks_copy() {
    assert_eq!(GZ_SUFFIX, ".gz");
    let root = place("respond");
    let wasm = "/app-0123abcd_bg.wasm";
    let html = "text/html; charset=utf-8";
    let js = "text/javascript; charset=utf-8";
    let css = "text/css; charset=utf-8";
    let cc = |v: &str| ("Cache-Control", v.to_string());
    let vary = ("Vary", "Accept-Encoding".to_string());
    let ce = ("Content-Encoding", "gzip".to_string());
    let cases: [Case; 10] = [
        (
            wasm,
            Some("gzip, br"),
            "application/wasm",
            WASM_GZ,
            vec![cc(IMMUTABLE), vary.clone(), ce.clone()],
        ),
        (
            wasm,
            None,
            "application/wasm",
            WASM,
            vec![cc(IMMUTABLE), vary.clone()],
        ),
        (
            wasm,
            Some("gzip;q=0"),
            "application/wasm",
            WASM,
            vec![cc(IMMUTABLE), vary.clone()],
        ),
        (
            wasm,
            Some("br"),
            "application/wasm",
            WASM,
            vec![cc(IMMUTABLE), vary.clone()],
        ),
        (
            "/style.css",
            Some("gzip"),
            css,
            CSS_GZ,
            vec![cc(NO_CACHE), vary.clone(), ce.clone()],
        ),
        (
            "/app-0123abcd.js",
            Some("gzip"),
            js,
            JS,
            vec![cc(IMMUTABLE)],
        ),
        ("/", Some("gzip"), html, INDEX, vec![cc(NO_CACHE)]),
        ("/index.html", Some("gzip"), html, INDEX, vec![cc(NO_CACHE)]),
        ("/evil.js", Some("gzip"), js, EVIL, vec![cc(NO_CACHE)]),
        (
            "/app-0123abcd_bg.wasm.gz",
            Some("gzip"),
            "application/octet-stream",
            WASM_GZ,
            vec![cc(IMMUTABLE)],
        ),
    ];
    for (path, encoding, content_type, body, headers) in cases {
        let resp = get(&root, path, encoding);
        let what = format!("{path} {encoding:?}");
        assert_eq!(resp.status, 200, "{what}");
        assert_eq!(resp.content_type, content_type, "{what}");
        assert_eq!(resp.body, body, "{what}");
        assert_eq!(resp.headers, headers, "{what}");
    }
    for (path, status, body) in [
        ("/nope.wasm", 404, "no-file"),
        ("/../secret.gz", 403, "outside"),
    ] {
        let resp = get(&root, path, Some("gzip"));
        assert_eq!(resp.status, status, "{path}");
        assert_eq!(resp.body, body.as_bytes(), "{path}");
        assert!(resp.headers.is_empty(), "{path}");
    }
    assert_eq!(
        gz_copy(&root, &root.join("style.css")),
        Some(root.join("style.css.gz"))
    );
    assert_eq!(gz_copy(&root, &root.join("app-0123abcd.js")), None);
    assert_eq!(gz_copy(&root, &root.join("evil.js")), None);
}
