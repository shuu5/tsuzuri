//! folio v2 の命令の入口。便 0 は `folio check` だけを持つ。

mod check;
mod refs;
mod verdict;
mod yaml;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

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
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { dir } => {
            let report = check::check_dir(&dir);
            for (kind, msg) in &report.violations {
                println!("[{kind}] {msg}");
            }
            for msg in &report.unknowns {
                eprintln!("# まだ分からない: {msg}");
            }
            let verdict = report.verdict();
            println!(
                "folio check: {verdict}（違反 {}・まだ分からない {}）",
                report.violations.len(),
                report.unknowns.len()
            );
            ExitCode::from(verdict.exit_code() as u8)
        }
    }
}
