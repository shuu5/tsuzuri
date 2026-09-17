//! folio v2 の命令の入口。便 0・便 1 の `folio check` と便 2 の `folio inject` を持つ。

mod adr;
mod anchor;
mod check;
mod entrance;
mod freeze;
mod gitcheck;
mod inject;
mod lineage;
mod link;
mod refs;
mod render;
mod sha256;
mod verdict;
mod vocab;
mod yaml;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgGroup, Parser, Subcommand};

use crate::freeze::{After, Flag};

#[derive(Parser)]
#[command(name = "folio", version, about = "folio v2 — 設計文書の生成と検査")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// design-intent の正本 5 file の形を検査し、合格 0 / 不合格 1 / まだ分からない 2 で終わる
    Check {
        /// 正本の置き場
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 最新 anchor と現行の欄単位の差分を amends にそのまま貼れる形で標準出力へ書く（違反は標準エラーへ）
        #[arg(long, conflicts_with = "freeze_anchor")]
        emit_amends: bool,
        /// 全検査が 0 違反で測れないも無いときだけ、現行の写しを新しい版の anchor として書き索引に追記する
        #[arg(long)]
        freeze_anchor: bool,
    },
    /// 憲法の前文と規範文を CLAUDE.md の生成区間へ書く（--write）・検査する（--check）・出す（--print）
    #[command(group(ArgGroup::new("mode").required(true).args(["write", "check", "print"])))]
    Inject {
        /// 正本の置き場（constitution.yaml と rules.yaml だけを読む）
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// CLAUDE.md の path
        #[arg(long, default_value = "CLAUDE.md")]
        claude_md: PathBuf,
        /// 区間の中身を導出で置き換えて書く（差が無ければ書かない）
        #[arg(long)]
        write: bool,
        /// 区間と導出の byte 一致・区間の外の規範語の行を検査する
        #[arg(long)]
        check: bool,
        /// 導出した本文を標準出力へ書く
        #[arg(long)]
        print: bool,
    },
    /// 正本 4 file と判断の記録から day-1 の読み物（HTML 1 面）を導出して書く（--write）・検査する（--check）
    #[command(group(ArgGroup::new("mode").required(true).args(["write", "check"])))]
    Render {
        /// 正本の置き場
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 出力先（相対なら --dir からの相対・絶対ならそのまま）
        #[arg(long, default_value = "preview/readable.html")]
        out: PathBuf,
        /// 導出した読み物を出力先へ書く
        #[arg(long)]
        write: bool,
        /// 出力先と導出の byte 一致を検査する（無い 2・不一致 1・一致 0）
        #[arg(long)]
        check: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Check {
            dir,
            emit_amends,
            freeze_anchor,
        } => {
            let flag = if emit_amends {
                Flag::EmitAmends
            } else if freeze_anchor {
                Flag::FreezeAnchor
            } else {
                Flag::None
            };
            let (report, after) = check::check_dir(&dir, flag);
            if let After::Refused(msg) = &after {
                eprintln!("folio check: {msg}");
                return ExitCode::from(1);
            }
            // --emit-amends では標準出力を貼れる差分だけにし、違反は標準エラーへ
            let out = |line: String| {
                if flag == Flag::EmitAmends {
                    eprintln!("{line}");
                } else {
                    println!("{line}");
                }
            };
            for (kind, msg) in &report.violations {
                out(format!("[{kind}] {msg}"));
            }
            for msg in report.unknowns.iter().chain(&report.pendings) {
                eprintln!("# まだ分からない: {msg}");
            }
            let verdict = report.verdict();
            out(format!(
                "folio check: {verdict}（違反 {}・まだ分からない {}）",
                report.violations.len(),
                report.unknowns.len() + report.pendings.len()
            ));
            match after {
                After::Emit(lines) => {
                    for line in lines {
                        println!("{line}");
                    }
                }
                After::Freeze(msg) => eprintln!("folio check: {msg}"),
                After::Nothing | After::Refused(_) => {}
            }
            ExitCode::from(verdict.exit_code() as u8)
        }
        Command::Inject {
            dir,
            claude_md,
            write,
            check,
            print: _,
        } => {
            let mode = if write {
                inject::Mode::Write
            } else if check {
                inject::Mode::Check
            } else {
                inject::Mode::Print
            };
            let outcome = inject::run(&dir, &claude_md, mode);
            if let Some(body) = &outcome.stdout {
                print!("{body}");
            }
            for msg in &outcome.messages {
                eprintln!("folio inject: {msg}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
        Command::Render {
            dir,
            out,
            write,
            check: _,
        } => {
            let mode = if write {
                render::Mode::Write
            } else {
                render::Mode::Check
            };
            let outcome = render::run(&dir, &out, mode);
            if let Some(line) = &outcome.stdout {
                println!("{line}");
            }
            if let Some(line) = &outcome.stderr {
                eprintln!("{line}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
    }
}
