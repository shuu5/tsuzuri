//! 配布の file の gzip の写し（行 g-gz・裁定 t3-hub.52.40:20260928T0156Z-1）。
//! surface-build は dist の下（子の dir も）の wasm・js・css の file ごとに、隣へ名に `.gz` を足した写しを書く。
//! 枠（RFC 1952）と CRC-32 はここで書き、deflate だけを miniz_oxide に任せる。頭の時刻は 0 に置き、同じ中身から同じ byte の写しを作る。

use std::path::{Path, PathBuf};

/// 写しを書く file の拡張子。
pub const EXTENSIONS: [&str; 3] = ["wasm", "js", "css"];

/// 写しの名に足す字（server の files の `GZ_SUFFIX` と同じ字）。
pub const SUFFIX: &str = ".gz";

/// deflate の段。
pub const LEVEL: u8 = 9;

/// gzip の頭（ID 1f 8b・CM 8・FLG 0・MTIME 0・XFL 2（最も縮める）・OS 3（Unix））。
pub const HEADER: [u8; 10] = [0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x03];

/// CRC-32（反転の多項式 0xEDB88320・bit ごとに計算し表を持たない）。
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// 中身の gzip（`HEADER`・生の deflate・CRC-32・長さの下位 32 bit を little endian で）。
pub fn gzip(bytes: &[u8]) -> Vec<u8> {
    let deflated = miniz_oxide::deflate::compress_to_vec(bytes, LEVEL);
    let mut out = Vec::with_capacity(HEADER.len() + deflated.len() + 8);
    out.extend_from_slice(&HEADER);
    out.extend_from_slice(&deflated);
    out.extend_from_slice(&crc32(bytes).to_le_bytes());
    // ISIZE は長さを 2^32 で割った余り（RFC 1952・`as` の切り捨てがそれに当たる）。
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out
}

/// 写しを書く file か（拡張子が `EXTENSIONS` のどれか）。
pub fn wanted(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e))
}

/// dist の下を子の dir まで名の順に歩き、`wanted` の file ごとに隣へ名に `SUFFIX` を足した写しを書く（在れば書き直す）。
/// 書いた写しの path を名の順に返す。読めない dir と file・書けない写しはその path を名指す誤り。
pub fn write_copies(dist: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    walk(dist, &mut files)?;
    let mut copies = Vec::new();
    for file in files.into_iter().filter(|f| wanted(f)) {
        let body =
            std::fs::read(&file).map_err(|e| format!("{} を読めない: {e}", file.display()))?;
        let mut name = file.clone().into_os_string();
        name.push(SUFFIX);
        let copy = PathBuf::from(name);
        std::fs::write(&copy, gzip(&body))
            .map_err(|e| format!("{} を書けない: {e}", copy.display()))?;
        copies.push(copy);
    }
    Ok(copies)
}

/// dir の下の file を名の順に集める（子の dir はその名の位置で潜る）。
fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let unreadable = |e: std::io::Error| format!("{} を読めない: {e}", dir.display());
    let mut entries = std::fs::read_dir(dir)
        .map_err(unreadable)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(unreadable)?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|e| format!("{} を読めない: {e}", path.display()))?;
        if kind.is_dir() {
            walk(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{EXTENSIONS, HEADER, LEVEL, crc32, gzip, write_copies};

    #[test]
    fn pgz_crc32_check_values() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
        assert_eq!(crc32(b"a"), 0xE8B7_BE43);
    }

    #[test]
    fn pgz_gzip_round_trip() {
        assert_eq!(HEADER, [0x1f, 0x8b, 0x08, 0x00, 0, 0, 0, 0, 0x02, 0x03]);
        assert_eq!(LEVEL, 9);
        let long: Vec<u8> = (0..70_000u32).map(|i| (i % 251) as u8).collect();
        for input in [&b""[..], b"tsuzuri", &long] {
            let out = gzip(input);
            assert!(out.len() >= 18, "{}", out.len());
            assert_eq!(out[..10], HEADER);
            let (middle, tail) = out[10..].split_at(out.len() - 18);
            assert_eq!(tail[..4], crc32(input).to_le_bytes());
            assert_eq!(tail[4..], (input.len() as u32).to_le_bytes());
            let back = miniz_oxide::inflate::decompress_to_vec(middle).expect("inflate");
            assert_eq!(back, input);
            assert_eq!(gzip(input), out, "同じ入力から同じ byte");
        }
        let out = gzip(&long);
        assert!(out.len() < long.len() / 10, "{}", out.len());
    }

    #[test]
    fn pgz_write_copies_only_assets() {
        assert_eq!(EXTENSIONS, ["wasm", "js", "css"]);
        let dist = std::env::temp_dir().join(format!("tsuzuri-pgz-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dist);
        std::fs::create_dir_all(dist.join("snippets/x-1")).expect("dist");
        let files: [(&str, &[u8]); 7] = [
            ("index.html", b"<html></html>"),
            ("app-0123abcd_bg.wasm", b"\0asm wasm"),
            ("app-0123abcd.js", b"export {}"),
            ("style-0123abcd.css", b"body{}"),
            ("snippets/x-1/inline0.js", b"export const a = 1;"),
            ("favicon.ico", b"ico"),
            ("old.wasm.gz", b"stale"),
        ];
        for (name, body) in files {
            std::fs::write(dist.join(name), body).expect(name);
        }
        let copies = write_copies(&dist).expect("写し");
        let rel: Vec<String> = copies
            .iter()
            .map(|p| {
                p.strip_prefix(&dist)
                    .expect("dist の下")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(
            rel,
            [
                "app-0123abcd.js.gz",
                "app-0123abcd_bg.wasm.gz",
                "snippets/x-1/inline0.js.gz",
                "style-0123abcd.css.gz",
            ]
        );
        for (name, body) in files {
            let copy = dist.join(format!("{name}.gz"));
            if rel.iter().any(|r| *r == format!("{name}.gz")) {
                assert_eq!(std::fs::read(&copy).expect(name), gzip(body), "{name}");
            } else {
                assert!(!copy.exists(), "{name} の写しを書かない");
            }
        }
        let missing = write_copies(&dist.join("none")).expect_err("無い dir");
        assert!(missing.contains("none"), "{missing}");
        let _ = std::fs::remove_dir_all(&dist);
    }
}
