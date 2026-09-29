//! tz hook deliver（行 f-deliver・要件 FR9・器の要件 FR79）: 席の UserPromptSubmit の hook。
//! 器の配達の口が待ちの席へ送る指し示しの 1 行（「裁定 <id> が届いた」）が prompt に在るとき、
//! 名指された裁定の逐語を台帳から写して席の文脈に足す（hookSpecificOutput の additionalContext）。
//! 撃つのは Claude Code（plugin の hooks.json の UserPromptSubmit）。順:
//! 1. 使い方の誤りか repo が dir でなければ rc 1（標準入力は読まない）。
//! 2. 標準入力の prompt に指し示しが無ければ、子 process を撃たずに何も出さずに 0。
//! 3. 在れば台帳を bd で 1 回読む。読めない・名指された行が無い時は標準エラーに 1 行を書いて 0
//!    （席には指し示しだけが残り、席が台帳を読む）。読めたら答えを標準出力に書いて 0。
//!
//! 配達済みの印は置かない（経路の印は server が置く）。hook は file を書かない。

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_core::delivery::{context, pointed, said};

use crate::server::ledger::{BD, Source};

pub const USAGE: &str = "usage: tz hook deliver --repo <dir> [--bd <program>]";

/// 使い方の誤り。
const FAIL: u8 = 1;

/// 読んだ引数（repo の置き場・台帳の読みに撃つ program）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub repo: PathBuf,
    pub bd: OsString,
}

/// `--名 値` か `--名=値` の --repo と、省ける --bd を読む（空の値は断る・--repo は省けない）。
pub fn parse(rest: &[&str]) -> Result<Args, String> {
    let (mut repo, mut bd) = (None, None);
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, v),
            None => (*arg, *it.next().ok_or_else(|| format!("{arg} の値が無い"))?),
        };
        let slot = match name {
            "--repo" => &mut repo,
            "--bd" => &mut bd,
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
        bd: bd.unwrap_or(BD).into(),
    })
}

/// tz hook deliver の残りの引数を受けて終了 code を返す（0 か 1 だけ）。
pub fn run(rest: &[&str]) -> u8 {
    let args = match parse(rest) {
        Ok(args) => args,
        Err(e) => return usage(&e),
    };
    if !args.repo.is_dir() {
        return usage(&format!(
            "repo の置き場 {} が dir でない",
            args.repo.display()
        ));
    }
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    let ids = pointed(&payload);
    if ids.is_empty() {
        return 0;
    }
    let named = ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ");
    let found = Source::new(&args.repo, &args.bd)
        .text_alone()
        .map(|text| said(&text, &ids));
    let answer = match found {
        Some(Reading::Known(found)) => context(&found),
        _ => {
            eprintln!("tz hook deliver: 台帳が読めない（逐語を写せない）: 裁定 {named}");
            return 0;
        }
    };
    let Some(answer) = answer else {
        eprintln!("tz hook deliver: 台帳に名指された裁定の行が無い: 裁定 {named}");
        return 0;
    };
    let mut out = std::io::stdout().lock();
    if let Err(e) = writeln!(out, "{answer}").and_then(|()| out.flush()) {
        eprintln!("tz hook deliver: 答えを書けない: {e}");
    }
    0
}

fn usage(what: &str) -> u8 {
    eprintln!("tz hook deliver: {what}\n{USAGE}");
    FAIL
}
