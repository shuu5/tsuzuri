//! 係の口 5 つ（起こしの門・結びの口・測り・係の門・終える前の門）の引数の読みと起草の置き場の解き（判断の記録 ADR-59 決定 (4)・要件 FR21）。
//! 引数は `--名 値` か `--名=値` の --repo と省ける --drafts で、起草の置き場は --drafts か repo の git config の鍵 `tsuzuri.draftsdir` から解く。
//! 口の頭の段（`begin`）は、引数の誤りを口の名（USAGE の字の `usage: ` の後の最初の ` --` の前）と訳と USAGE の 2 行で
//! 標準エラーに書いて rc 1 を、置き場を解けなければ口の名と訳を標準エラーに書いて rc 0 を、どちらも Err で返す。

use std::ffi::OsStr;
use std::path::PathBuf;

use crate::acct::GIT;
use crate::consult::{DRAFTS_ARGS, GIT_TIMEOUT};
use crate::out::emit_err;
use crate::server::proc;

/// 使い方の誤り。
const FAIL: u8 = 1;

/// 読んだ引数（repo の置き場・省ける起草の置き場）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub repo: PathBuf,
    pub drafts: Option<PathBuf>,
}

/// `--名 値` か `--名=値` の --repo と、省ける --drafts を読む（空の値と 2 度目は断る・--repo は省けない）。
pub fn parse(rest: &[&str]) -> Result<Args, String> {
    let (mut repo, mut drafts) = (None, None);
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, v),
            None => (*arg, *it.next().ok_or_else(|| format!("{arg} の値が無い"))?),
        };
        let slot = match name {
            "--repo" => &mut repo,
            "--drafts" => &mut drafts,
            _ => return Err(format!("知らない引数 {name}")),
        };
        if value.is_empty() {
            return Err(format!("{name} の値が空"));
        }
        if slot.replace(value).is_some() {
            return Err(format!("{name} が 2 度ある"));
        }
    }
    let Some(repo) = repo else {
        return Err("--repo が要る".into());
    };
    Ok(Args {
        repo: PathBuf::from(repo),
        drafts: drafts.map(PathBuf::from),
    })
}

/// 起草の置き場（--drafts か repo の git config の鍵・dir でなければ None）。
pub fn drafts(args: &Args) -> Option<PathBuf> {
    let dir = match &args.drafts {
        Some(d) => d.clone(),
        None => {
            let mut git_args = vec![OsStr::new("-C"), args.repo.as_os_str()];
            git_args.extend(DRAFTS_ARGS.iter().map(OsStr::new));
            let out = proc::capture(OsStr::new(GIT), git_args, &args.repo, GIT_TIMEOUT)?;
            PathBuf::from(String::from_utf8(out).ok()?.trim())
        }
    };
    dir.is_dir().then_some(dir)
}

/// USAGE の字 `usage` の口の名（`usage: ` の後の、最初の ` --` の前の字）。
fn mouth(usage: &str) -> &str {
    let rest = usage.strip_prefix("usage: ").unwrap_or(usage);
    rest.split(" --").next().unwrap_or(rest)
}

/// 使い方の誤りを、口の名と訳 `why` の 1 行と USAGE の字の 1 行で標準エラーに書き、rc 1 を返す。
pub fn misuse(usage: &str, why: &str) -> u8 {
    emit_err(&format!("{}: {why}\n{usage}", mouth(usage)));
    FAIL
}

/// 口の頭の段: 残りの引数 `rest` を読み、起草の置き場を解く。引数の誤りは `misuse` の rc 1 を、置き場を解けなければ
/// 口の名と字「起草の置き場を解けない（<miss>）」を標準エラーに書いて rc 0 を Err で返す。
pub fn begin(usage: &str, miss: &str, rest: &[&str]) -> Result<(Args, PathBuf), u8> {
    let args = parse(rest).map_err(|e| misuse(usage, &e))?;
    let Some(dir) = drafts(&args) else {
        emit_err(&format!(
            "{}: 起草の置き場を解けない（{miss}）",
            mouth(usage)
        ));
        return Err(0);
    };
    Ok((args, dir))
}
