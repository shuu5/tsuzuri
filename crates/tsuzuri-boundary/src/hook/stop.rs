//! tz hook stop（行 f-stop・要件 FR9・器の要件 FR79）: 席の停止の hook。
//! 席が考え中から待ちに移る境で、台帳の問いの notes の裁定のうち配達済みの印の無いものを拾い、
//! 指し示し（裁定の id）を答え（decision block）として標準出力に返して席を続けさせ、返した後にだけ印を置く。
//! 撃つのは Claude Code（plugin の hooks.json の Stop）。順:
//! 1. 使い方の誤りか repo が dir でなければ rc 1（標準入力は読まない）。
//! 2. repo の下の .git が file（git の worktree・器の便の runner）なら何もせずに 0。
//! 3. 標準入力の stop_hook_active が false の object でなければ（続けた後の停止・JSON の object でない）何も出さずに 0。
//! 4. 台帳を bd で読み、未配達の裁定を取る。読めなければ標準エラーに 1 行を書く。
//! 5. 相談の拾い（行 cs-hooks・`consult::nudge`）: 見張りが居ない間、受けの無い所見と頼みの行か、見張りを置かせる 1 行を取る。
//! 6. 未配達も相談の行も無ければ何も出さずに 0。在れば答え（裁定の reason の後ろに相談の行）を標準出力に書いて flush し、
//!    その後に未配達の裁定にだけ停止の印を bdw で置く（上限 `MARK_BUDGET`）。
//!
//! どの形でも 2 は返さない（Claude Code は停止の hook の rc 2 を続けの指示として読む）。
//! hook は repo と state dir に file を書かない（台帳の書きは bdw だけ）。

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BDW, LedgerWrite};
use tsuzuri_core::consult::pickup::block_with;
use tsuzuri_core::delivery::{Pending, Route, block, mark_line, stop_active, undelivered};

use crate::out::emit_err;
use crate::server::events::now;
use crate::server::ledger::{BD, Source, capture};
use crate::server::ruling::{WRITE_TIMEOUT, minute};

use super::consult;

pub const USAGE: &str = "usage: tz hook stop --repo <dir> [--bd <program>] [--bdw <program>]";

/// 1 回の停止の印の書きの全部の上限（台帳の読みの `BD_TIMEOUT` と足して hooks.json の timeout より短く）。
pub const MARK_BUDGET: Duration = Duration::from_secs(20);

/// 使い方の誤り。
const FAIL: u8 = 1;

/// 読んだ引数（repo の置き場・台帳の読みと書きの program）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub repo: PathBuf,
    pub bd: OsString,
    pub bdw: OsString,
}

/// `--名 値` か `--名=値` の --repo と、省ける --bd・--bdw を読む（空の値は断る・--repo は省けない）。
pub fn parse(rest: &[&str]) -> Result<Args, String> {
    let (mut repo, mut bd, mut bdw) = (None, None, None);
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, v),
            None => (*arg, *it.next().ok_or_else(|| format!("{arg} の値が無い"))?),
        };
        let slot = match name {
            "--repo" => &mut repo,
            "--bd" => &mut bd,
            "--bdw" => &mut bdw,
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
        bdw: bdw.unwrap_or(BDW).into(),
    })
}

/// 未配達の裁定に停止の印を順に置き、書けた数を返す（`budget` は呼ばれた時から数える全部の上限）。
/// 落ちた裁定と上限で撃たなかった裁定は、その裁定の id を含む 1 行ずつを標準エラーに書いて次へ進む。
pub fn mark(args: &Args, pending: &[Pending], now: EpochSecs, budget: Duration) -> usize {
    let start = Instant::now();
    let minute = minute(now);
    let mut done = 0;
    for p in pending {
        let left = budget.saturating_sub(start.elapsed());
        if left.is_zero() {
            emit_err(&format!(
                "tz hook stop: 時間の上限で印を置かない: 裁定 {}",
                p.ruling
            ));
            continue;
        }
        let write = LedgerWrite::AppendNotes {
            id: p.question.clone(),
            line: mark_line(&p.ruling, Route::Stop, &minute),
        };
        if capture(&args.bdw, write.argv(), &args.repo, left.min(WRITE_TIMEOUT)).is_some() {
            done += 1;
        } else {
            emit_err(&format!("tz hook stop: 印の書きが落ちた: 裁定 {}", p.ruling));
        }
    }
    done
}

/// tz hook stop の残りの引数を受けて終了 code を返す（0 か 1 だけ）。
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
    // git の worktree（器の便の runner・人が worktree で起こした session）では黙る。
    if args.repo.join(".git").is_file() {
        return 0;
    }
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    // 続けた後の停止と、JSON の object でない入力は何も出さない（輪の守り）。
    if stop_active(&payload) != Some(false) {
        return 0;
    }
    let pending = match Source::new(&args.repo, &args.bd)
        .text_alone()
        .map(|text| undelivered(&text))
    {
        Some(Reading::Known(pending)) => pending,
        _ => {
            emit_err("tz hook stop: 台帳が読めない（未配達の裁定を拾えない）");
            Vec::new()
        }
    };
    let Some(answer) = block_with(block(&pending), &consult::nudge(&args.repo)) else {
        return 0;
    };
    let mut out = std::io::stdout().lock();
    if let Err(e) = writeln!(out, "{answer}").and_then(|()| out.flush()) {
        emit_err(&format!("tz hook stop: 答えを書けない（印を置かない）: {e}"));
        return 0;
    }
    drop(out);
    mark(&args, &pending, now(), MARK_BUDGET);
    0
}

fn usage(what: &str) -> u8 {
    emit_err(&format!("tz hook stop: {what}\n{USAGE}"));
    FAIL
}
