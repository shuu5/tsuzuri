//! HTTP/1.1 の最小の読み書き（標準 library だけ）。要求は 1 接続 1 つで、応答の後に接続を閉じる。

use std::io::{self, BufRead, BufReader, Read, Write};

/// 要求の頭の上限（byte）。
const HEAD_MAX: usize = 16 * 1024;

/// 受ける本文の上限（byte）。越える本文は持たずに読み捨てる（口は 413 を返す）。
pub const BODY_MAX: u64 = 65_536;

/// 読み捨てる本文の上限（byte）。越える本文は読まずに接続を閉じる。
const BODY_DRAIN_MAX: u64 = 1024 * 1024;

/// 要求（頭の Host と Origin と Accept-Encoding・Content-Length・`BODY_MAX` 以下の本文）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub target: String,
    pub host: Option<String>,
    pub origin: Option<String>,
    /// 頭 Accept-Encoding の値（2 行在れば後の行・面の file の gzip の写しを選ぶ・行 g-gz）。
    pub accept_encoding: Option<String>,
    /// 頭の Content-Length（無ければ 0）。
    pub content_length: u64,
    /// 本文（Content-Length が `BODY_MAX` を越えれば空）。
    pub body: Vec<u8>,
}

impl Request {
    /// target の path の部分（`?` の前）。
    pub fn path(&self) -> &str {
        self.target
            .split_once('?')
            .map_or(self.target.as_str(), |(p, _)| p)
    }

    /// target の query（`?` の後）で鍵 `key` の最初の値（鍵と値は `+` を空白に、`%XX` を byte に戻して読む）。
    /// 鍵が無ければ None。`=` の無い鍵の値は空の字。形の悪い `%` はそのまま残し、UTF-8 でない byte は置き換える。
    pub fn query(&self, key: &str) -> Option<String> {
        let (_, query) = self.target.split_once('?')?;
        query
            .split('&')
            .map(|pair| pair.split_once('=').unwrap_or((pair, "")))
            .find(|(k, _)| unescape(k) == key)
            .map(|(_, v)| unescape(v))
    }

    /// 本文が `BODY_MAX` を越える。
    pub fn too_large(&self) -> bool {
        self.content_length > BODY_MAX
    }

    /// Origin の頭が無いか、その host と port が Host の頭と同じ（Origin が在って Host が無ければ違う扱い）。
    pub fn same_origin(&self) -> bool {
        let Some(origin) = &self.origin else {
            return true;
        };
        let Some((scheme, rest)) = origin.trim().split_once("://") else {
            return false;
        };
        let default_port = match scheme.to_ascii_lowercase().as_str() {
            "http" => 80,
            "https" => 443,
            _ => return false,
        };
        let from = authority(rest, default_port);
        let to = self
            .host
            .as_deref()
            .and_then(|h| authority(h.trim(), default_port));
        from.is_some() && from == to
    }
}

/// query の字を戻す（`+` は空白・`%XX` は byte・形の悪い `%` はそのまま）。
fn unescape(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let hex = |at: usize| bytes.get(at).and_then(|b| (*b as char).to_digit(16));
    while i < bytes.len() {
        match (bytes[i], hex(i + 1), hex(i + 2)) {
            (b'+', _, _) => out.push(b' '),
            (b'%', Some(h), Some(l)) => {
                out.push((h * 16 + l) as u8);
                i += 2;
            }
            (b, _, _) => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `host[:port]` を（小文字の host・port）にする（port を省けば `default_port`・IPv6 は `[…]` の形）。
fn authority(s: &str, default_port: u16) -> Option<(String, u16)> {
    let (host, port) = if s.starts_with('[') {
        let end = s.find(']')?;
        match &s[end + 1..] {
            "" => (&s[..=end], None),
            rest => (&s[..=end], Some(rest.strip_prefix(':')?)),
        }
    } else {
        match s.rsplit_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (s, None),
        }
    };
    if host.is_empty() || host.contains(['/', '@', ' ']) {
        return None;
    }
    let port = match port {
        Some(p) => p.parse().ok()?,
        None => default_port,
    };
    Some((host.to_ascii_lowercase(), port))
}

/// 要求の最初の 1 byte が届く前に相手が閉じたか読みが終わった（時間切れ・切断ほか）印。
#[derive(Debug)]
struct NoBytes;

impl std::fmt::Display for NoBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("要求の 1 byte も届かない")
    }
}

impl std::error::Error for NoBytes {}

/// `read_request` が最初の 1 byte を読む前に返した誤り（接続には何も書かずに閉じる）。
/// byte の届いた後の誤りと、`read_request` の外で作った誤りでは偽。
pub fn no_bytes(e: &io::Error) -> bool {
    e.get_ref().is_some_and(|inner| inner.is::<NoBytes>())
}

/// 要求の頭を読み、Content-Length の本文を `BODY_MAX` まで持ち、越える本文は上限まで読み捨てる
/// （読み残しの在る接続を閉じると応答が相手に届かないことがある）。本文が切れていれば誤り。
/// 最初の 1 byte が届く前に閉じたか読みが終わった接続は `no_bytes` が真の誤り。
pub fn read_request(stream: impl Read) -> io::Result<Request> {
    let mut reader = BufReader::new(stream);
    loop {
        match reader.fill_buf() {
            Ok([]) => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, NoBytes)),
            Ok(_) => break,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(io::Error::new(e.kind(), NoBytes)),
        }
    }
    let mut head = 0usize;
    let mut line = String::new();
    let mut next_line = |reader: &mut BufReader<_>, line: &mut String| -> io::Result<()> {
        line.clear();
        let n = reader.by_ref().take(HEAD_MAX as u64).read_line(line)?;
        head += n;
        if n == 0 || head > HEAD_MAX || !line.ends_with('\n') {
            return Err(bad("要求の頭が切れているか長すぎる"));
        }
        Ok(())
    };
    next_line(&mut reader, &mut line)?;
    let mut parts = line.split_ascii_whitespace();
    let (Some(method), Some(target), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(bad("要求の行の形"));
    };
    if !version.starts_with("HTTP/1.") || !target.starts_with('/') {
        return Err(bad("要求の行の形"));
    }
    let mut request = Request {
        method: method.to_string(),
        target: target.to_string(),
        host: None,
        origin: None,
        accept_encoding: None,
        content_length: 0,
        body: Vec::new(),
    };
    loop {
        next_line(&mut reader, &mut line)?;
        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        let Some((k, v)) = header.split_once(':') else {
            continue;
        };
        let (k, v) = (k.trim(), v.trim());
        if k.eq_ignore_ascii_case("content-length") {
            request.content_length = v.parse().map_err(|_| bad("Content-Length の形"))?;
        } else if k.eq_ignore_ascii_case("host") {
            request.host = Some(v.to_string());
        } else if k.eq_ignore_ascii_case("origin") {
            request.origin = Some(v.to_string());
        } else if k.eq_ignore_ascii_case("accept-encoding") {
            request.accept_encoding = Some(v.to_string());
        }
    }
    let length = request.content_length;
    if length <= BODY_MAX {
        reader.take(length).read_to_end(&mut request.body)?;
        if request.body.len() as u64 != length {
            return Err(bad("本文が切れている"));
        }
    } else if length <= BODY_DRAIN_MAX {
        io::copy(&mut reader.take(length), &mut io::sink())?;
    }
    Ok(request)
}

fn bad(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, what)
}

/// 状態の code の語。
fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Content Too Large",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Internal Server Error",
    }
}

/// 応答（本文を持ち、接続を閉じる）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
    /// 足す頭の行（名と値）。
    pub headers: Vec<(&'static str, String)>,
}

impl Response {
    pub fn new(status: u16, content_type: &'static str, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            content_type,
            body: body.into(),
            headers: Vec::new(),
        }
    }

    /// JSON の応答（API の口は cache させない）。
    pub fn json(status: u16, body: String) -> Self {
        Self::new(status, "application/json; charset=utf-8", body)
            .header("Cache-Control", "no-store")
    }

    /// 字の応答（断りの理由の 1 語など）。
    pub fn text(status: u16, body: &str) -> Self {
        Self::new(status, "text/plain; charset=utf-8", body)
    }

    pub fn header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }

    pub fn write_to(&self, mut w: impl Write) -> io::Result<()> {
        let mut head = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n",
            self.status,
            reason(self.status),
            self.content_type,
            self.body.len()
        );
        for (name, value) in &self.headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        head.push_str("\r\n");
        w.write_all(head.as_bytes())?;
        w.write_all(&self.body)?;
        w.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::{Response, read_request};

    #[test]
    fn server_min_http_reads_request() {
        let raw = b"POST /api/ledger?x=1 HTTP/1.1\r\nHost: a\r\nContent-Length: 3\r\n\r\nabc";
        let req = read_request(&raw[..]).expect("要求");
        assert_eq!(req.method, "POST");
        assert_eq!(req.path(), "/api/ledger");
        for bad in [
            &b"GET / HTTP/1.1\r\n"[..],
            b"GET\r\n\r\n",
            b"GET http://x/ HTTP/1.1\r\n\r\n",
            b"GET / HTTP/1.1\r\nContent-Length: x\r\n\r\n",
        ] {
            assert!(
                read_request(bad).is_err(),
                "{:?}",
                String::from_utf8_lossy(bad)
            );
        }
    }

    #[test]
    fn server_ask_http_keeps_body_and_heads() {
        let raw = b"POST /api/ruling HTTP/1.1\r\nhost: 127.0.0.1:8080\r\nOrigin: http://127.0.0.1:8080\r\nContent-Length: 2\r\n\r\n{}";
        let req = read_request(&raw[..]).expect("要求");
        assert_eq!(req.body, b"{}");
        assert_eq!(req.host.as_deref(), Some("127.0.0.1:8080"));
        assert!(req.same_origin() && !req.too_large());
        let cut = b"POST /api/ruling HTTP/1.1\r\nContent-Length: 5\r\n\r\n{}";
        assert!(read_request(&cut[..]).is_err(), "本文が切れている");
        let big = format!(
            "POST /api/ruling HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
            super::BODY_MAX + 1,
            "x".repeat(super::BODY_MAX as usize + 1)
        );
        let req = read_request(big.as_bytes()).expect("要求");
        assert!(req.too_large() && req.body.is_empty());
    }

    #[test]
    fn server_ask_http_same_origin() {
        let with = |host: Option<&str>, origin: Option<&str>| super::Request {
            method: "POST".into(),
            target: "/".into(),
            host: host.map(str::to_string),
            origin: origin.map(str::to_string),
            accept_encoding: None,
            content_length: 0,
            body: Vec::new(),
        };
        for (host, origin) in [
            (Some("a:1"), None),
            (None, None),
            (Some("a:1"), Some("http://a:1")),
            (Some("A"), Some("http://a:80")),
            (Some("a:443"), Some("https://a")),
            (Some("[::1]:9"), Some("http://[::1]:9")),
        ] {
            assert!(with(host, origin).same_origin(), "{host:?} {origin:?}");
        }
        for (host, origin) in [
            (Some("a:1"), Some("http://a:2")),
            (Some("a:1"), Some("http://b:1")),
            (None, Some("http://a:1")),
            (Some("a:1"), Some("null")),
            (Some("a:1"), Some("ftp://a:1")),
            (Some("[::1]:9"), Some("http://[::2]:9")),
        ] {
            assert!(!with(host, origin).same_origin(), "{host:?} {origin:?}");
        }
    }

    #[test]
    fn server_view_http_query_unescapes() {
        let with = |target: &str| super::Request {
            method: "GET".into(),
            target: target.into(),
            host: None,
            origin: None,
            accept_encoding: None,
            content_length: 0,
            body: Vec::new(),
        };
        let req =
            with("/api/around?id=R%2D25&k=&fold&q=a+b%20c&id=x&%6B2=%E8%A8%AD&bad=%4&odd=%zz%");
        assert_eq!(req.path(), "/api/around");
        assert_eq!(req.query("id").as_deref(), Some("R-25"), "最初の値");
        assert_eq!(req.query("k").as_deref(), Some(""));
        assert_eq!(req.query("fold").as_deref(), Some(""), "`=` の無い鍵");
        assert_eq!(req.query("q").as_deref(), Some("a b c"));
        assert_eq!(req.query("k2").as_deref(), Some("設"), "鍵も戻す");
        assert_eq!(req.query("bad").as_deref(), Some("%4"));
        assert_eq!(req.query("odd").as_deref(), Some("%zz%"));
        assert_eq!(req.query("steps"), None);
        assert_eq!(with("/api/around").query("id"), None);
        assert_eq!(
            with("/api/around?id=%FF").query("id").as_deref(),
            Some("\u{FFFD}")
        );
    }

    #[test]
    fn server_min_http_writes_response() {
        let mut out = Vec::new();
        Response::text(405, "method")
            .header("Allow", "GET")
            .write_to(&mut out)
            .expect("書く");
        let text = String::from_utf8(out).expect("字");
        assert!(
            text.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"),
            "{text}"
        );
        assert!(text.contains("Content-Length: 6\r\n"), "{text}");
        assert!(text.contains("Allow: GET\r\n"), "{text}");
        assert!(text.ends_with("\r\n\r\nmethod"), "{text}");
    }
}
