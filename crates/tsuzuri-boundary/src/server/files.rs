//! 面の file の配布（API の口でない GET）。置き場の外の path は断る。
//! `..` と `.` の区切りを断り、実体の path（symlink を解いた先）が置き場の中に在ることを確かめる。
//! 名に中身の hash を持つ file（trunk が付ける）は頭 Cache-Control で 1 年持たせ（`IMMUTABLE`）、
//! index.html とほかの file は毎回確かめさせる（`NO_CACHE`）。
//! 隣に gzip の写し（名に `GZ_SUFFIX`・xtask の surface-build が書く）の在る file には頭 Vary を足し、
//! 要求の Accept-Encoding が gzip を受ければ写しの中身を頭 Content-Encoding と返す。
//! 写しの無い file の頭は写しの無かった時と同じで、写しを名指す要求はほかの file と同じ配り。

use std::path::{Path, PathBuf};

use super::http::{Request, Response};

/// 名に hash を持つ file の Cache-Control。
pub const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// ほかの file の Cache-Control。
pub const NO_CACHE: &str = "no-cache";

/// gzip の写しの名に足す字（xtask の gz の `SUFFIX` と同じ字）。
pub const GZ_SUFFIX: &str = ".gz";

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

/// file の名が trunk の付ける中身の hash を持つか（最初の `.` より前の字から末の `_bg` を除き、
/// 最後の `-` の後が 8〜16 字の小文字の 16 進で、`-` の前が 1 字以上）。
pub fn hashed(name: &str) -> bool {
    let Some((stem, _)) = name.split_once('.') else {
        return false;
    };
    let stem = stem.strip_suffix("_bg").unwrap_or(stem);
    let Some((base, hash)) = stem.rsplit_once('-') else {
        return false;
    };
    !base.is_empty()
        && (8..=16).contains(&hash.len())
        && hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// 頭 Accept-Encoding の字が gzip を受けるか。`,` で分けた項ごとに `;` の前の名の前後の空白を除き、
/// 大小の字を問わず gzip の項が在り、その項の引数に q が無いか q の値が 0 より大きい数なら真。
/// x-gzip と `*` と、q の値が数として読めない項は受けない側に倒す。
pub fn accepts_gzip(header: Option<&str>) -> bool {
    header.is_some_and(|h| {
        h.split(',').any(|item| {
            let mut parts = item.split(';');
            let name = parts.next().unwrap_or("").trim();
            name.eq_ignore_ascii_case("gzip")
                && parts.all(|param| {
                    let (k, v) = param.split_once('=').unwrap_or((param, ""));
                    !k.trim().eq_ignore_ascii_case("q")
                        || v.trim().parse::<f64>().is_ok_and(|q| q > 0.0)
                })
        })
    })
}

/// 実体の path（`resolve` の出力）の隣の gzip の写し（名に `GZ_SUFFIX` を足した file の実体）。
/// 無いか、実体が置き場の外か、file でなければ None。
pub fn gz_copy(root: &Path, real: &Path) -> Option<PathBuf> {
    let mut name = real.as_os_str().to_owned();
    name.push(GZ_SUFFIX);
    let copy = PathBuf::from(name).canonicalize().ok()?;
    (copy.starts_with(root) && copy.is_file()).then_some(copy)
}

/// 面の file の GET の応答（file なら頭 Cache-Control を 1 つ足し、写しが在れば Vary を、写しを返せば
/// Content-Encoding を足す・断りは頭を足さない・読めない file は無い file と同じ 404）。
pub fn respond(req: &Request, root: &Path) -> Response {
    let path = req.path();
    let found = resolve(root, path).and_then(|real| {
        let copy = gz_copy(root, &real);
        let gzip = copy
            .clone()
            .filter(|_| accepts_gzip(req.accept_encoding.as_deref()));
        let body = std::fs::read(gzip.as_ref().unwrap_or(&real)).map_err(|_| Served::Missing)?;
        Ok((content_type(&real), body, copy.is_some(), gzip.is_some()))
    });
    match found {
        Ok((content_type, body, vary, gzip)) => {
            let name = path.rsplit('/').next().unwrap_or("");
            let cache = if hashed(name) { IMMUTABLE } else { NO_CACHE };
            let mut resp = Response::new(200, content_type, body).header("Cache-Control", cache);
            if vary {
                resp = resp.header("Vary", "Accept-Encoding");
            }
            if gzip {
                resp = resp.header("Content-Encoding", "gzip");
            }
            resp
        }
        Err(Served::Outside) => Response::text(403, "outside"),
        Err(_) => Response::text(404, "no-file"),
    }
}

/// `%XX` を解く（解けない % や UTF-8 でない字は None）。
fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&byte) = bytes.get(i) {
        if byte == b'%' {
            let hex = s.get(i + 1..i + 3)?;
            if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return None;
            }
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(byte);
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
