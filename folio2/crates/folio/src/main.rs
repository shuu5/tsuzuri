//! folio v2 の命令の入口。便 0・便 1 の `folio check` と便 2 の `folio inject` を持つ。

mod adr;
mod anchor;
mod check;
mod gitcheck;
mod inject;
mod lineage;
mod link;
mod refs;
mod sha256;
mod verdict;
mod vocab;
mod yaml;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgGroup, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "folio", version, about = "folio v2 — 設計文書の生成と検査")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// design-intent の正本 4 file の形を検査し、合格 0 / 不合格 1 / まだ分からない 2 で終わる
    Check {
        /// 正本の置き場
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
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
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { dir } => {
            let report = check::check_dir(&dir);
            for (kind, msg) in &report.violations {
                println!("[{kind}] {msg}");
            }
            for msg in report.unknowns.iter().chain(&report.pendings) {
                eprintln!("# まだ分からない: {msg}");
            }
            let verdict = report.verdict();
            println!(
                "folio check: {verdict}（違反 {}・まだ分からない {}）",
                report.violations.len(),
                report.unknowns.len() + report.pendings.len()
            );
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
    }
}
