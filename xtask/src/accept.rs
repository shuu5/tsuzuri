//! accept は受入 12 条の runner の入口（行 j-runner・要件 NFR1・規則の行 R-23）。
//! 席の目の headless の Chrome（境界の crate の relay の Eyes・行 i-4）で全画面 × 2 幅 × 2 mode を測り
//! （境界の crate の audit の sweep）、report を書いて違反の計を標準誤りに出す。
//! server は起こさない（board の URL を旗で受ける・現物の server は席が tz surface serve で起こす）。
//! rc は違反が在れば 1・無ければ 0・撃てなければ 2。まだ分からない画面（読みの印が待ちの上限の後も残る）が
//! 在れば、違反の数に依らず report を書いて 2（行 g-accept-runner）。

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::emit_err;
use tsuzuri_boundary::audit;
use tsuzuri_boundary::stage::relay::{CHROME, Eyes};
use tsuzuri_boundary::stage::tunnel;

/// accept の撃ち方。
pub const USAGE: &str =
    "usage: cargo run -q -p xtask -- accept --url <board の URL> [--chrome <program>] [--out <path>]";

/// 面の語彙表（workspace の root から）。
const VOCAB: &str = "crates/tsuzuri-surface/vocab.json";

/// 面の語彙の部品の dir（workspace の root から・行 g-vocab-parts）。
const VOCAB_PARTS: &str = "crates/tsuzuri-surface/vocab";

/// --out を省いた時の report の置き場（workspace の root から）。
const REPORT: &str = "target/accept/report.txt";

/// 席の目の Chrome の 1 つの応答と読み込みを待つ上限。
const TIMEOUT: Duration = Duration::from_secs(30);

/// 読んだ旗。
struct Flags {
    url: String,
    chrome: OsString,
    out: PathBuf,
}

/// 旗を読む（--url は要る・同じ旗は 1 度・知らない旗と値の無い旗は Err）。
fn flags(args: &[String], root: &Path) -> Result<Flags, String> {
    let (mut url, mut chrome, mut out) = (None, None, None);
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        let slot = match flag.as_str() {
            "--url" => &mut url,
            "--chrome" => &mut chrome,
            "--out" => &mut out,
            _ => return Err(format!("知らない旗 {flag}")),
        };
        let value = rest.next().ok_or_else(|| format!("旗 {flag} に値が無い"))?;
        if slot.replace(value.clone()).is_some() {
            return Err(format!("旗 {flag} は 1 度だけ"));
        }
    }
    Ok(Flags {
        url: url.ok_or("旗 --url が要る")?,
        chrome: chrome.map_or_else(|| OsString::from(CHROME), OsString::from),
        out: out.map_or_else(|| root.join(REPORT), PathBuf::from),
    })
}

/// 旗を読んで全画面を測り、report を書いて rc を返す。
pub fn run(args: &[String], root: &Path) -> i32 {
    let flags = match flags(args, root) {
        Ok(flags) => flags,
        Err(e) => {
            emit_err(&format!("xtask accept: {e}\n{USAGE}"));
            return 2;
        }
    };
    match sweep(&flags, root) {
        Ok((total, 0)) => {
            emit_err(&format!(
                "xtask accept: 違反 計 {total}（report は {}）",
                flags.out.display()
            ));
            i32::from(total > 0)
        }
        Ok((total, unknown)) => {
            emit_err(&format!(
                "xtask accept: 違反 計 {total}・まだ分からない画面 {unknown}（report は {}）",
                flags.out.display()
            ));
            2
        }
        Err(e) => {
            emit_err(&format!("xtask accept: {e}"));
            2
        }
    }
}

/// 語彙表を読み、tz の口と同じ口座の dir の下で席の目の Chrome を起こして sweep を撃ち、report を書く
/// （違反の和とまだ分からない画面の数を返す）。
fn sweep(flags: &Flags, root: &Path) -> Result<(usize, usize), String> {
    let vocab = vocab_text(root)?;
    let board = flags.url.split('?').next().unwrap_or(&flags.url);
    let base = tunnel::user_dir(&env::temp_dir())?;
    let (_eyes, mut session) = Eyes::open(&flags.chrome, &base, board, TIMEOUT)?;
    let (report, total, unknown) = audit::sweep(&mut session, board, &vocab)?;
    let _ = session.close();
    if let Some(dir) = flags.out.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir).map_err(|e| format!("{} を作れない: {e}", dir.display()))?;
    }
    fs::write(&flags.out, report)
        .map_err(|e| format!("report {} を書けない: {e}", flags.out.display()))?;
    Ok((total, unknown))
}

/// 基の語彙表と部品の dir の .json を名の順に読み、面の vocab() と同じ和の 1 つの JSON の字にする（行 g-vocab-parts）。
fn vocab_text(root: &Path) -> Result<String, String> {
    let dir = root.join(VOCAB_PARTS);
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("語彙の部品の dir {VOCAB_PARTS} を読めない: {e}"))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    files.insert(0, root.join(VOCAB));
    let mut texts = Vec::new();
    for file in &files {
        texts.push(
            fs::read_to_string(file)
                .map_err(|e| format!("語彙表 {} を読めない: {e}", file.display()))?,
        );
    }
    audit::merge_vocab(&texts.iter().map(String::as_str).collect::<Vec<_>>())
}
