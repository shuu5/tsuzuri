//! tz hook question-gate（行 f-gate・要件 FR4・受入 AC6）: 問いの起票の門。
//! 撃つのは Claude Code（PreToolUse の hook・matcher Bash）。標準入力の hook の入力から問いの起票の下書きを拾い、
//! 導出グラフで判じて、通さないときだけ deny の答えを標準出力に 1 行で書く（通すときは何も出さない）。順:
//! 1. 標準入力を全部読む（読めなければ空の字）。
//! 2. 使い方の誤りか repo が dir でなければ、下書きが無ければ rc 1（止めない誤り）、在れば deny の args を書いて 0。
//! 3. bd か bdw の create の全部の metadata の短い題を先に見て、断れば子 process を撃たずに deny を書いて 0（規則の行 R-39・行 c-short-gate）。
//! 4. 下書きが無ければ子 process を撃たずに 0。在れば台帳を bd で、設計の索引を設計の道具で並べて読み、判じて 0。
//!
//! rc 2 は使わない。停止の hook と違い、repo が git の worktree でも黙らない。hook は file を書かない。

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::PathBuf;

use tsuzuri_core::gate::{self, Gate, Why};
use tsuzuri_core::graph::{Graph, Inputs, build};

use crate::out::emit_err;
use crate::server::design::Design;
use crate::server::ledger::{BD, Source};

pub const USAGE: &str =
    "usage: tz hook question-gate --repo <dir> [--bd <program>] [--folio <program>]";

/// 使い方の誤り（下書きの無い呼び出しだけ）。
const FAIL: u8 = 1;

/// 読んだ引数（repo の置き場・台帳の読みの program・設計の索引の読みの program）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub repo: PathBuf,
    pub bd: OsString,
    pub folio: OsString,
}

/// `--名 値` か `--名=値` の --repo と、省ける --bd・--folio を読む（空の値は断る・--repo は省けない）。
pub fn parse(rest: &[&str]) -> Result<Args, String> {
    let (mut repo, mut bd, mut folio) = (None, None, None);
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, v),
            None => (*arg, *it.next().ok_or_else(|| format!("{arg} の値が無い"))?),
        };
        let slot = match name {
            "--repo" => &mut repo,
            "--bd" => &mut bd,
            "--folio" => &mut folio,
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
        folio: crate::cli::folio::program(folio),
    })
}

/// 台帳と設計の索引を並べて読み（待ちは 1 本分の上限まで）、読めない方を空の字にしてグラフを組む（event log は空の字）。
pub fn graph(args: &Args) -> Graph {
    let (ledger, design) = std::thread::scope(|s| {
        let ledger = s.spawn(|| Source::new(&args.repo, &args.bd).text_alone());
        let design = Design::new(&args.repo, &args.folio).text();
        (ledger.join().ok().flatten(), design)
    });
    build(&Inputs {
        design_index: design.as_deref().unwrap_or_default(),
        ledger: ledger.as_deref().unwrap_or_default(),
        events: "",
    })
}

/// 答えの字（短い題の断りが先・下書きが無ければ子 process を撃たずに None・通すときも None）。
pub fn answer(args: &Args, payload: &str) -> Option<String> {
    if let Some(why) = gate::short_gate(payload) {
        return Some(gate::short_output(why));
    }
    let drafts = gate::drafts(payload);
    if drafts.is_empty() {
        return None;
    }
    gate::output(&gate::judge(&drafts, &graph(args)))
}

/// tz hook question-gate の残りの引数を受けて終了 code を返す（0 か 1 だけ）。
pub fn run(rest: &[&str]) -> u8 {
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    let args = match parse(rest) {
        Ok(args) if args.repo.is_dir() => args,
        Ok(args) => {
            return refuse(
                &payload,
                &format!("repo の置き場 {} が dir でない", args.repo.display()),
            );
        }
        Err(e) => return refuse(&payload, &e),
    };
    let Some(text) = answer(&args, &payload) else {
        return 0;
    };
    write_line(&text);
    0
}

/// 引数の誤り: 問いの起票だけを deny の args で断り（rc 0）、ほかの呼び出しは止めない誤りの rc 1。子 process は撃たない。
fn refuse(payload: &str, what: &str) -> u8 {
    emit_err(&format!("tz hook question-gate: {what}\n{USAGE}"));
    if gate::drafts(payload).is_empty() {
        return FAIL;
    }
    let deny = Gate::Deny {
        why: Why::Args,
        ids: Vec::new(),
        digest: None,
    };
    if let Some(text) = gate::output(&deny) {
        write_line(&text);
    }
    0
}

fn write_line(text: &str) {
    let mut out = std::io::stdout().lock();
    if let Err(e) = writeln!(out, "{text}").and_then(|()| out.flush()) {
        emit_err(&format!("tz hook question-gate: 答えを書けない: {e}"));
    }
}
