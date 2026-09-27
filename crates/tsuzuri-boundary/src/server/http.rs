//! HTTP/1.1 の最小の読み書き（標準 library だけ）。要求は 1 接続 1 つで、応答の後に接続を閉じる。

use std::io::{self, BufRead, BufReader, Read, Write};

/// 要求の頭の上限（byte）。
const HEAD_MAX: usize = 16 * 1024;

/// 読み捨てる本文の上限（byte）。越える本文は読まずに接続を閉じる。
const BODY_DRAIN_MAX: u64 = 1024 * 1024;

/// 要求（本文は読み捨てる・書く口を持たないので使わない）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub target: String,
}

impl Request {
    /// target の path の部分（`?` の前）。
    pub fn path(&self) -> &str {
        self.target
            .split_once('?')
            .map_or(self.target.as_str(), |(p, _)| p)
    }
}

/// 要求の頭を読み、Content-Length の本文を上限まで読み捨てる
/// （読み残しの在る接続を閉じると応答が相手に届かないことがある）。
pub fn read_request(stream: impl Read) -> io::Result<Request> {
    let mut reader = BufReader::new(stream);
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
    let request = Request {
        method: method.to_string(),
        target: target.to_string(),
    };
    let mut body = 0u64;
    loop {
        next_line(&mut reader, &mut line)?;
        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        if let Some((k, v)) = header.split_once(':')
            && k.trim().eq_ignore_ascii_case("content-length")
        {
            body = v.trim().parse().map_err(|_| bad("Content-Length の形"))?;
        }
    }
    if body <= BODY_DRAIN_MAX {
        io::copy(&mut reader.take(body), &mut io::sink())?;
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
