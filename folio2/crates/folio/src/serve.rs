//! `folio serve`（便 17・docs/design/delivery-17.md §1 (b)）。`folio build` の配信先を tailnet の内側だけで
//! 見せる小さな配信器。bind 先が tailnet の外なら起動を拒む（N-6.1・fail-closed）。台帳や版管理が在る dir は
//! 配信先にしない（N-6.2）。標準 library の net と fs だけで書く（外部 crate も正規表現も使わない）。
//! 接続は 1 つずつ順に処理し（並列にしない）、1 接続 = 1 要求（Connection: close）・GET と HEAD だけ。

use std::fs;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::verdict::Verdict;

/// 要求の頭（空行まで）の上限。
const MAX_HEAD: usize = 8 * 1024;
/// 読みと書きの timeout。
const TIMEOUT: Duration = Duration::from_secs(5);
/// tailscale の名前引きの決まった住所（packet は送らない・経路の出口を OS に問うだけ）。
const TAILSCALE_DNS: (Ipv4Addr, u16) = (Ipv4Addr::new(100, 100, 100, 100), 53);
/// 平文の短い応答の Content-Type。
const TEXT: &str = "text/plain; charset=utf-8";

/// 拡張子 → Content-Type（表に無い拡張子は OCTET）。
const TYPES: [(&str, &str); 9] = [
    ("html", "text/html; charset=utf-8"),
    ("css", "text/css; charset=utf-8"),
    ("js", "text/javascript; charset=utf-8"),
    ("json", "application/json"),
    ("svg", "image/svg+xml"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("txt", "text/plain; charset=utf-8"),
];
const OCTET: &str = "application/octet-stream";

// ── 命令の口 ──

/// 起動の順に判定し、最初に当たったもので終わる。配信に入れば持ち主の Ctrl-C まで戻らない。
pub fn run(dir: &Path, host: Option<&str>, port: u16) -> Verdict {
    // 1. 配信先
    if !dir.is_dir() {
        return refuse(Verdict::Unknown, "まだ分からない: 配信先が無い".to_string());
    }
    // 2. 入口（入口の無い dir は見せない・P-2.2）
    if !dir.join("index.html").is_file() {
        return refuse(
            Verdict::Unknown,
            "まだ分からない: 入口 index.html が無い".to_string(),
        );
    }
    // 3. 版管理か台帳（中身を晒さない・N-6.2）
    if [".git", ".beads"]
        .iter()
        .any(|name| fs::symlink_metadata(dir.join(name)).is_ok())
    {
        return refuse(
            Verdict::Fail,
            "拒否 — 配信先に版管理か台帳が在る".to_string(),
        );
    }
    let docroot = match fs::canonicalize(dir) {
        Ok(p) => p,
        Err(_) => return refuse(Verdict::Unknown, "まだ分からない: 配信先が無い".to_string()),
    };
    // 4. bind 先
    let ip = match bind_host(host) {
        Ok(ip) => ip,
        Err((verdict, msg)) => return refuse(verdict, msg),
    };
    // 5. bind
    let listener = match TcpListener::bind(SocketAddr::from((ip, port))) {
        Ok(l) => l,
        Err(e) => {
            return refuse(
                Verdict::Unknown,
                format!("まだ分からない: bind できない: {e}"),
            );
        }
    };
    let bound = match listener.local_addr() {
        Ok(a) => a.port(),
        Err(e) => {
            return refuse(
                Verdict::Unknown,
                format!("まだ分からない: bind できない: {e}"),
            );
        }
    };
    // 6. 1 行だけ標準出力へ（以後 stdout には何も書かない）
    println!(
        "folio serve: http://{ip}:{bound}/index.html（配信先 {}・止めるには Ctrl-C）",
        docroot.display()
    );
    let _ = std::io::stdout().flush();
    for stream in listener.incoming() {
        match stream {
            Ok(s) => serve_one(s, &docroot),
            // 受け取れなかった接続は落として次へ（自分では終わらない）
            Err(_) => continue,
        }
    }
    Verdict::Pass
}

fn refuse(verdict: Verdict, message: String) -> Verdict {
    eprintln!("folio serve: {message}");
    verdict
}

// ── bind 先の判定 ──

/// tailnet の範囲（100.64.0.0/10）。
pub fn in_tailnet(ip: Ipv4Addr) -> bool {
    let [a, b, _, _] = ip.octets();
    a == 100 && (64..128).contains(&b)
}

/// bind してよい住所（tailnet の内側・loopback は同じ端末の中だけなので内側と扱う）。
pub fn inside(ip: Ipv4Addr) -> bool {
    ip.is_loopback() || in_tailnet(ip)
}

/// `--host` が在ればその住所を検査し、無ければ tailnet の住所を自分で解く（fail-closed）。
fn bind_host(host: Option<&str>) -> Result<Ipv4Addr, (Verdict, String)> {
    match host {
        Some(s) => {
            let ip: Ipv4Addr = s.parse().map_err(|_| {
                (
                    Verdict::Unknown,
                    format!("まだ分からない: bind 先が IPv4 でない: {s}"),
                )
            })?;
            if inside(ip) {
                Ok(ip)
            } else {
                Err((
                    Verdict::Fail,
                    format!("拒否 — bind 先 {ip} は tailnet の外"),
                ))
            }
        }
        None => tailnet_address().ok_or((Verdict::Fail, "拒否 — tailnet の住所が無い".to_string())),
    }
}

/// 経路の出口の住所を OS に問い、tailnet の範囲の中ならその住所（FR7 の確かめ方）。
fn tailnet_address() -> Option<Ipv4Addr> {
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    sock.connect(TAILSCALE_DNS).ok()?;
    match sock.local_addr().ok()? {
        SocketAddr::V4(a) if in_tailnet(*a.ip()) => Some(*a.ip()),
        _ => None,
    }
}

// ── 配信器 ──

/// 1 接続 = 1 要求。応答を書けなければ（相手が切った）無視して次へ。
fn serve_one(mut stream: TcpStream, docroot: &Path) {
    let _ = stream.set_read_timeout(Some(TIMEOUT));
    let _ = stream.set_write_timeout(Some(TIMEOUT));
    let head = match read_head(&mut stream) {
        Head::Text(h) => h,
        Head::TooLong => return fail(&mut stream, 400, "-", "-"),
        Head::Broken => return,
    };
    let Some((method, target)) = request_line(&head) else {
        return fail(&mut stream, 400, "-", "-");
    };
    if method != "GET" && method != "HEAD" {
        return fail(&mut stream, 405, method, target);
    }
    if !target.starts_with('/') {
        return fail(&mut stream, 400, method, target);
    }
    let Some(rel) = rel_path(target) else {
        return fail(&mut stream, 404, method, target);
    };
    // docroot の下に閉じ込める（docroot の外を指す symlink も落とす）
    let Ok(real) = fs::canonicalize(docroot.join(rel)) else {
        return fail(&mut stream, 404, method, target);
    };
    if !real.starts_with(docroot) {
        return fail(&mut stream, 404, method, target);
    }
    let Ok(meta) = fs::metadata(&real) else {
        return fail(&mut stream, 404, method, target);
    };
    // 通常の file だけ（dir は一覧を出さずに 404）
    if !meta.is_file() {
        return fail(&mut stream, 404, method, target);
    }
    let ctype = content_type(&real);
    if method == "HEAD" {
        eprintln!("200 {method} {target}");
        let _ = respond(&mut stream, 200, "OK", ctype, meta.len(), None);
        return;
    }
    let Ok(body) = fs::read(&real) else {
        return fail(&mut stream, 404, method, target);
    };
    eprintln!("200 {method} {target}");
    let _ = respond(
        &mut stream,
        200,
        "OK",
        ctype,
        body.len() as u64,
        Some(&body),
    );
}

/// 400・404・405 の短い応答（本文は平文）。
fn fail(stream: &mut TcpStream, status: u16, method: &str, target: &str) {
    let (reason, body) = match status {
        400 => ("Bad Request", "400 読めない要求"),
        405 => ("Method Not Allowed", "405 GET と HEAD だけ"),
        _ => ("Not Found", "404 無い"),
    };
    eprintln!("{status} {method} {target}");
    let _ = respond(
        stream,
        status,
        reason,
        TEXT,
        body.len() as u64,
        Some(body.as_bytes()),
    );
}

fn respond(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    ctype: &str,
    len: u64,
    body: Option<&[u8]>,
) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(head.as_bytes())?;
    if let Some(body) = body {
        stream.write_all(body)?;
    }
    stream.flush()
}

/// 要求の頭の読みの結果。
enum Head {
    Text(String),
    /// 8 KiB を超えた（400）
    TooLong,
    /// 読めない（応答もしない）
    Broken,
}

fn read_head(stream: &mut TcpStream) -> Head {
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                if head_ends(&buf) {
                    break;
                }
                if buf.len() > MAX_HEAD {
                    return Head::TooLong;
                }
            }
            Err(_) => return Head::Broken,
        }
    }
    if buf.is_empty() {
        return Head::Broken;
    }
    Head::Text(String::from_utf8_lossy(&buf).into_owned())
}

/// 空行（頭の終わり）まで来たか。
fn head_ends(buf: &[u8]) -> bool {
    buf.windows(4).any(|w| w == b"\r\n\r\n") || buf.windows(2).any(|w| w == b"\n\n")
}

/// 要求行の method と path（path は `?` と `#` から後ろを捨てる）。読めなければ None（400）。
pub fn request_line(head: &str) -> Option<(&str, &str)> {
    let line = head.lines().next()?;
    let mut parts = line.split(' ');
    let method = parts.next()?;
    let target = parts.next()?;
    if method.is_empty() || target.is_empty() {
        return None;
    }
    let end = target.find(['?', '#']).unwrap_or(target.len());
    Some((method, &target[..end]))
}

/// 要求の path → docroot からの相対 path。見せられない形は None（404）。
/// `/` だけなら入口。各節は 空・`.`・`..`・先頭 `.`・`%` か `\` を含む・ASCII 以外 を受けない。
pub fn rel_path(target: &str) -> Option<PathBuf> {
    let target = if target == "/" { "/index.html" } else { target };
    let mut rel = PathBuf::new();
    for seg in target.strip_prefix('/')?.split('/') {
        if !safe_segment(seg) {
            return None;
        }
        rel.push(seg);
    }
    Some(rel)
}

/// path の 1 節が見せてよい形か。
pub fn safe_segment(seg: &str) -> bool {
    !seg.is_empty()
        && !seg.starts_with('.')
        && !seg.contains('%')
        && !seg.contains('\\')
        && seg.is_ascii()
}

/// 拡張子の表（表に無ければ application/octet-stream）。
pub fn content_type(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    TYPES
        .iter()
        .find(|(e, _)| *e == ext)
        .map_or(OCTET, |(_, t)| *t)
}

#[cfg(test)]
mod serve_tests {
    use super::*;

    #[test]
    fn serve_binds_only_inside_the_tailnet_or_loopback() {
        for inside_ip in ["127.0.0.1", "100.64.0.0", "100.127.255.255"] {
            assert!(
                inside(inside_ip.parse().unwrap()),
                "{inside_ip} は内側のはず"
            );
        }
        for outside in ["100.128.0.0", "0.0.0.0", "10.0.0.1", "192.168.0.1"] {
            assert!(!inside(outside.parse().unwrap()), "{outside} は外のはず");
        }
        assert!(in_tailnet("100.64.0.1".parse().unwrap()));
        assert!(!in_tailnet("127.0.0.1".parse().unwrap()));
    }

    #[test]
    fn serve_rejects_path_segments_that_escape_or_hide() {
        assert_eq!(rel_path("/"), Some(PathBuf::from("index.html")));
        assert_eq!(
            rel_path("/sub/a.txt"),
            Some(PathBuf::from("sub").join("a.txt"))
        );
        for bad in [
            "/../Cargo.toml",
            "/./a.txt",
            "/.hidden.txt",
            "//a.txt",
            "/sub/",
            "/a%2e.txt",
            "/a\\b.txt",
            "/日本語.html",
        ] {
            assert_eq!(rel_path(bad), None, "{bad} は見せない");
        }
        assert!(safe_segment("index.html"));
        assert!(!safe_segment(""));
        assert!(!safe_segment(".."));
    }

    #[test]
    fn serve_content_type_table_is_closed() {
        for (name, want) in [
            ("a.html", "text/html; charset=utf-8"),
            ("a.css", "text/css; charset=utf-8"),
            ("a.js", "text/javascript; charset=utf-8"),
            ("a.json", "application/json"),
            ("a.svg", "image/svg+xml"),
            ("a.png", "image/png"),
            ("a.jpg", "image/jpeg"),
            ("a.jpeg", "image/jpeg"),
            ("a.txt", "text/plain; charset=utf-8"),
            ("a.wasm", OCTET),
            ("a", OCTET),
        ] {
            assert_eq!(content_type(Path::new(name)), want, "{name}");
        }
    }

    #[test]
    fn serve_reads_the_request_line() {
        assert_eq!(
            request_line("GET /a.html?x=1 HTTP/1.1\r\nHost: h\r\n\r\n"),
            Some(("GET", "/a.html"))
        );
        assert_eq!(
            request_line("HEAD /a.html#top HTTP/1.1\r\n\r\n"),
            Some(("HEAD", "/a.html"))
        );
        assert_eq!(
            request_line("GET index.html HTTP/1.1\r\n"),
            Some(("GET", "index.html"))
        );
        assert_eq!(
            request_line("POST /a.html HTTP/1.1\r\n"),
            Some(("POST", "/a.html"))
        );
        assert_eq!(request_line("GET\r\n"), None);
        assert_eq!(request_line(""), None);
    }
}
