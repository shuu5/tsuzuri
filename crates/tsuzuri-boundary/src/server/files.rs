//! 面の file の配布（API の口でない GET）。置き場の外の path は断る。
//! `..` と `.` の区切りを断り、実体の path（symlink を解いた先）が置き場の中に在ることを確かめる。

use std::path::{Path, PathBuf};

/// 配布の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Served {
    File {
        content_type: &'static str,
        body: Vec<u8>,
    },
    /// 置き場の外を指す path（`..`・symlink の外・読めない %）。
    Outside,
    /// 置き場の中に無い。
    Missing,
}

/// URL の path を置き場の中の file に解く。`root` は canonicalize 済みの置き場。
pub fn resolve(root: &Path, url_path: &str) -> Result<PathBuf, Served> {
    let decoded = percent_decode(url_path).ok_or(Served::Outside)?;
    if decoded.contains(['\0', '\\']) {
        return Err(Served::Outside);
    }
    let mut target = root.to_path_buf();
    for seg in decoded.split('/').filter(|s| !s.is_empty()) {
        if seg == "." || seg == ".." {
            return Err(Served::Outside);
        }
        target.push(seg);
    }
    if target.is_dir() {
        target.push("index.html");
    }
    let real = target.canonicalize().map_err(|_| Served::Missing)?;
    if !real.starts_with(root) {
        return Err(Served::Outside);
    }
    if !real.is_file() {
        return Err(Served::Missing);
    }
    Ok(real)
}

/// URL の path の file を読む。
pub fn serve(root: &Path, url_path: &str) -> Served {
    let path = match resolve(root, url_path) {
        Ok(path) => path,
        Err(refused) => return refused,
    };
    match std::fs::read(&path) {
        Ok(body) => Served::File {
            content_type: content_type(&path),
            body,
        },
        Err(_) => Served::Missing,
    }
}

/// `%XX` を解く（解けない % や UTF-8 でない字は None）。
fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = s.get(i + 1..i + 3)?;
            if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return None;
            }
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::percent_decode;

    #[test]
    fn server_min_percent_decode() {
        assert_eq!(percent_decode("/a%2e%2E/b").as_deref(), Some("/a../b"));
        assert_eq!(percent_decode("/%E9%9D%A2").as_deref(), Some("/面"));
        assert_eq!(percent_decode("/%2"), None);
        assert_eq!(percent_decode("/%zz"), None);
        assert_eq!(percent_decode("/%+1"), None);
        assert_eq!(percent_decode("/%ff"), None);
    }
}
