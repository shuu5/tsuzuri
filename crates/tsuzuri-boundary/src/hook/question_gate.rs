//! tz hook question-gate（要件 FR4・受入 AC6）: 問いの起票の門と memo の起票の門（判断の記録 ADR-72 の決定 (5)）。
//! 撃つのは Claude Code（PreToolUse の hook・matcher Bash）。標準入力の hook の入力から問いの起票の下書きと memo の起票の下書きを拾い、
//! 導出グラフで判じて、通さないときだけ deny の答えを標準出力に 1 行で書く（通すときは何も出さない）。順:
//! 1. 標準入力を全部読む（読めなければ空の字）。
//! 2. 使い方の誤りか repo が dir でなければ、下書き（問いか memo）が無ければ rc 1（止めない誤り）、在れば deny の args を書いて 0。
//! 3. bd か bdw の create の全部の metadata の短い題を先に見て、断れば子 process を撃たずに deny を書いて 0（規則の行 R-39）。
//!    続けて契約の書き（bd か bdw の create か update で、欄 acceptance が [[contract]] の行を持つか、契約の bead の本文だけの直し）を判じ、
//!    本文の file の散文の門（folio の入口の check --prose）で止める（判断の記録 ADR-72 の決定 (2)）。台帳は本文だけの直しが在る時だけ 1 度読む。
//! 4. 下書きも計画の memo を増やす書き（bd か bdw の create・reopen・update ほか・規則の行 R-45 の上限）も無ければ子 process を撃たずに 0。
//!    在れば台帳を bd で、設計の索引を設計の道具で並べて読み、グラフを 1 度だけ組んで判じて 0
//!    （計画の memo の上限の門の答えが先で、次に問いの門の答え、問いの門が通す時に memo の門の答えを書く。
//!    上限は --repo の下の設計文書の dir の rules.yaml の字から読み、読めなければ上限の門は まだ分からない）。
//!    memo の下書きは --body-file の file を payload の cwd（無ければ --repo）から読んで本文を足し、本文が code の語を持つ時だけ
//!    code の層（git ls-files と ast-grep の scan）を 1 度組んで、台帳の memo の字と共に判じの材料にする。
//!
//! rc 2 は使わない。停止の hook と違い、repo が git の worktree でも黙らない。hook は file を書かない。

use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use tsuzuri_core::contract_gate::{self, ContractWrite};
use tsuzuri_core::gate::{self, Gate, Why};
use tsuzuri_core::graph::code::{self as layer, CodeGraph};
use tsuzuri_core::graph::{Graph, Inputs, build};
use tsuzuri_core::memo_gate::{self, MemoDraft, MemoGate, MemoWhy, Seen};
use tsuzuri_core::plan_cap;

use crate::cli::code;
use crate::out::emit_err;
use crate::server::design::{DESIGN_DIR, Design};
use crate::server::ledger::{BD, Source};

pub const USAGE: &str =
    "usage: tz hook question-gate --repo <dir> [--bd <program>] [--folio <program>]";

/// 使い方の誤り（下書きの無い呼び出しだけ）。
const FAIL: u8 = 1;

/// 計画の memo の上限の行を読む規則の表の file 名（設計文書の dir の下）。
const RULES_FILE: &str = "rules.yaml";

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

/// 台帳と設計の索引を並べて読む（待ちは 1 本分の上限まで・読めない方は None・順は台帳・設計の索引）。
fn sources(args: &Args) -> (Option<String>, Option<String>) {
    std::thread::scope(|s| {
        let ledger = s.spawn(|| Source::new(&args.repo, &args.bd).text_alone());
        let design = Design::new(&args.repo, &args.folio).text();
        (ledger.join().ok().flatten(), design)
    })
}

/// 読んだ 2 つの字からグラフを組む（読めない方は空の字・event log は空の字）。
fn graph_from(ledger: Option<&str>, design: Option<&str>) -> Graph {
    build(&Inputs {
        design_index: design.unwrap_or_default(),
        ledger: ledger.unwrap_or_default(),
        events: "",
    })
}

/// 台帳と設計の索引を並べて読み、読めない方を空の字にしてグラフを組む。
pub fn graph(args: &Args) -> Graph {
    let (ledger, design) = sources(args);
    graph_from(ledger.as_deref(), design.as_deref())
}

/// memo の下書きの本文の file を読んで欄 text の末に改行と本文を足す（読めなければ欄 text を None にする・
/// 根は payload の cwd か --repo・絶対 path はそのまま）。
fn read_bodies(args: &Args, payload: &str, drafts: Vec<MemoDraft>) -> Vec<MemoDraft> {
    let root = memo_gate::payload_cwd(payload).map_or_else(|| args.repo.clone(), PathBuf::from);
    drafts
        .into_iter()
        .map(|mut d| {
            if let Some(file) = &d.body_file {
                d.text = d
                    .text
                    .zip(std::fs::read_to_string(root.join(file)).ok())
                    .map(|(text, body)| format!("{text}\n{body}"));
            }
            d
        })
        .collect()
}

/// code の層を組む（file の一覧と定義だけ・失敗は None）。
fn code_layer(args: &Args) -> Option<CodeGraph> {
    let (files, defs) = code::read_layer(&args.repo, code::SG).ok()?;
    Some(layer::build(&files, defs, &[]))
}

/// 散文の門を file `file` に撃つ（folio の入口を命令の名 tz と引数 check --dir <repo の設計文書の dir> --prose <file> で撃ち、
/// 標準出力と標準エラーを buffer に受ける・子 process は撃たない）。戻りは rc と標準出力の字。
pub fn shoot(repo: &Path, file: &OsStr) -> (u8, String) {
    let design = repo.join(DESIGN_DIR);
    let args: [&OsStr; 6] = [
        OsStr::new("tz"),
        OsStr::new("check"),
        OsStr::new("--dir"),
        design.as_os_str(),
        OsStr::new("--prose"),
        file,
    ];
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let rc = ::folio::entry::run(args, &mut out, &mut err);
    (rc, String::from_utf8_lossy(&out).into_owned())
}

/// 契約の書きの答えの字（契約の書きが無いか通すときは None・台帳は本文だけの直しが在る時だけ 1 度読む）。
fn contract_answer(args: &Args, payload: &str) -> Option<String> {
    let writes = contract_gate::writes(payload);
    let body_only = writes
        .iter()
        .any(|w| matches!(w, ContractWrite::BodyOnly { .. }));
    let ledger = if body_only {
        Source::new(&args.repo, &args.bd)
            .text_alone()
            .unwrap_or_default()
    } else {
        String::new()
    };
    let fire = |file: &str| shoot(&args.repo, OsStr::new(file));
    contract_gate::output(&contract_gate::judge(&writes, fire, &ledger))
}

/// 答えの字（短い題の断りが先・次に契約の書きの断り・下書きも計画の memo を増やす書きも無ければ子 process を撃たずに None・
/// 通すときも None・在れば台帳を 1 度だけ読んでグラフと memo の写しを組み、計画の memo の上限の門の答えを先に、
/// 次に問いの門の答えを返す）。
pub fn answer(args: &Args, payload: &str) -> Option<String> {
    if let Some(why) = gate::short_gate(payload) {
        return Some(gate::short_output(why));
    }
    if let Some(text) = contract_answer(args, payload) {
        return Some(text);
    }
    let planned = plan_cap::opens(payload);
    let questions = gate::drafts(payload);
    let memos = memo_gate::drafts(payload);
    if planned.is_empty() && questions.is_empty() && memos.is_empty() {
        return None;
    }
    let (ledger, design) = sources(args);
    let graph = graph_from(ledger.as_deref(), design.as_deref());
    if !planned.is_empty() {
        let rules = std::fs::read_to_string(args.repo.join(DESIGN_DIR).join(RULES_FILE));
        let cap = rules.map_err(|e| e.to_string()).and_then(|t| plan_cap::cap(&t));
        if let Some(text) = plan_cap::output(&plan_cap::judge(&planned, &graph, cap.ok())) {
            return Some(text);
        }
    }
    if let Some(text) = gate::output(&gate::judge(&questions, &graph)) {
        return Some(text);
    }
    let memos = read_bodies(args, payload, memos);
    let named = memos
        .iter()
        .any(|d| d.text.as_deref().is_some_and(|t| !memo_gate::code_words(t).is_empty()));
    let seen = Seen {
        graph: &graph,
        memos: memo_gate::memo_texts(ledger.as_deref().unwrap_or_default()),
        code: if named { code_layer(args) } else { None },
    };
    memo_gate::output(&memo_gate::judge(&memos, &seen))
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

/// 引数の誤り: 問いの起票と memo の起票だけを deny の args で断り（rc 0・問いの下書きが在れば問いの門の答え）、
/// ほかの呼び出しは止めない誤りの rc 1。子 process は撃たない。
fn refuse(payload: &str, what: &str) -> u8 {
    emit_err(&format!("tz hook question-gate: {what}\n{USAGE}"));
    let text = if !gate::drafts(payload).is_empty() {
        gate::output(&Gate::Deny {
            why: Why::Args,
            ids: Vec::new(),
            digest: None,
        })
    } else if !memo_gate::drafts(payload).is_empty() {
        memo_gate::output(&MemoGate::Deny {
            why: MemoWhy::Args,
            ids: Vec::new(),
            digest: None,
        })
    } else {
        return FAIL;
    };
    if let Some(text) = text {
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
