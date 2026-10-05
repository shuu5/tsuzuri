//! tz hook agent-spawn（行 ag-spec・判断の記録 ADR-59 決定 (2)(4)・要件 FR21）: 係の起こしの門。
//! 撃つのは Claude Code（PreToolUse の hook・matcher Agent）。標準入力の hook の入力が席の Agent の呼び（係の id の無い呼び）の時だけ判じる。順:
//! 1. 標準入力を全部読む。席の Agent の呼びでなければ（係の呼び・ほかの道具）何も読まず何も出さずに 0。
//! 2. 起草の置き場を --drafts か、repo の git config の鍵 `tsuzuri.draftsdir` から解く。解けなければ標準エラーに書いて通す（fail-open）。
//! 3. 置き場の直下の dir の係の札を全部読み、中核の `judge` で判じる。断る時は deny の答えを標準出力に 1 行で書いて 0。
//! 4. 通す時は `<名>/brief.md` に prompt の字を、`<名>/spec.json` に係の札を書いて 0（書けなければ標準エラーに書いて通す）。
//!
//! rc は 0 か 1（使い方の誤り）だけ。置き場の解き方と引数の読みは結びの口（`agent_bind`）も使う。

use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use tsuzuri_core::agent::spec::{BRIEF, SPEC, Spec, deny, judge, reason, spawn_call};

use crate::acct::GIT;
use crate::consult::{DRAFTS_ARGS, GIT_TIMEOUT};
use crate::out::{emit, emit_err};
use crate::server::events::now;
use crate::server::proc;

pub const USAGE: &str = "usage: tz hook agent-spawn --repo <dir> [--drafts <dir>]";

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

/// 置き場の直下の dir の読める係の札の全部（読めない札は飛ばす）。
pub fn specs(drafts: &Path) -> Vec<Spec> {
    let Ok(dir) = fs::read_dir(drafts) else {
        return Vec::new();
    };
    dir.flatten()
        .filter_map(|e| fs::read_to_string(e.path().join(SPEC)).ok())
        .filter_map(|t| Spec::parse(&t))
        .collect()
}

/// 通した頼みと札を係の dir に書く。
fn place(drafts: &Path, prompt: &str, spec: &Spec) -> std::io::Result<()> {
    let dir = drafts.join(&spec.name);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join(BRIEF), prompt)?;
    fs::write(dir.join(SPEC), spec.render())
}

/// tz hook agent-spawn の残りの引数を受けて終了 code を返す（0 か 1 だけ）。
pub fn run(rest: &[&str]) -> u8 {
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    let Some(call) = spawn_call(&payload) else {
        return 0;
    };
    let args = match parse(rest) {
        Ok(args) => args,
        Err(e) => {
            emit_err(&format!("tz hook agent-spawn: {e}\n{USAGE}"));
            return FAIL;
        }
    };
    let Some(dir) = drafts(&args) else {
        emit_err("tz hook agent-spawn: 起草の置き場を解けない（通す）");
        return 0;
    };
    match judge(&call, &specs(&dir), now()) {
        Ok(spec) => {
            if let Err(e) = place(&dir, &call.prompt, &spec) {
                emit_err(&format!(
                    "tz hook agent-spawn: 頼みと札を書けない（通す）: {e}"
                ));
            }
        }
        Err(r) => emit(&deny(&reason(&r, &call.prompt))),
    }
    0
}
