//! folio v2 の命令の入口。便 0・便 1 の `folio check` と便 2 の `folio inject` を持つ。

mod adr;
mod anchor;
mod check;
mod entrance;
mod face;
mod face_adr;
mod face_constitution;
mod face_index;
mod face_note;
mod face_srs;
mod figure;
mod freeze;
mod gitcheck;
mod hello;
mod inject;
mod intake;
mod lineage;
mod link;
mod note;
mod parts;
mod prose;
mod refs;
mod render;
mod serve;
mod sha256;
mod sheet;
mod site;
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
    /// 部品目録から組み立て時に導出した一覧を出す（--print）・面の class と部品の名札と行内の様式を部品目録と突き合わせる（--check）
    #[command(group(ArgGroup::new("mode").required(true).args(["check", "print"])))]
    Parts {
        /// 正本の置き場（preview/parts.json を読む）
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 様式の定義（既定は --dir の下の preview/folio.css）
        #[arg(long)]
        css: Option<PathBuf>,
        /// 面の名と path（<面の名>=<path>・何度でも・1 つも無ければ --dir の下の preview/ の index / constitution / srs）
        #[arg(long = "page", value_name = "FACE=PATH")]
        pages: Vec<String>,
        /// 部品目録との一致・class・部品の名札・行内の様式を検査する（合格 0・不合格 1・まだ分からない 2）
        #[arg(long)]
        check: bool,
        /// 導出した一覧を JSON の 1 行で標準出力へ書く
        #[arg(long)]
        print: bool,
    },
    /// 正本から見本 3 面の 1 面を導出して書く（--write）・検査する（--check）。本便で生成器を持つのは憲法の面だけ
    #[command(group(ArgGroup::new("mode").required(true).args(["write", "check"])))]
    Face {
        /// 面の名（index・constitution・srs・adr・note）
        #[arg(long)]
        face: String,
        /// 判断の記録の id・設計ノートの文書 id（面 adr と note に付く・その面には要る）
        #[arg(long)]
        id: Option<String>,
        /// 正本の置き場
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 出力先（既定なし・相対なら --dir からの相対・絶対ならそのまま）
        #[arg(long)]
        out: PathBuf,
        /// 導出した面を出力先へ書く
        #[arg(long)]
        write: bool,
        /// 出力先と導出の byte 一致を検査する（無い 2・不一致 1・一致 0）
        #[arg(long)]
        check: bool,
    },
    /// 設計ノートの図 1 枚を型付き記述から図の道具（rules 行 R-15）で検査と描画に掛け、図の本体（SVG）を出力先へ書く（--write）・検査する（--check）
    #[command(group(ArgGroup::new("mode").required(true).args(["write", "check"])))]
    Figure {
        /// 設計ノートの文書 id
        #[arg(long)]
        doc: String,
        /// 図の id（figures の行の id）
        #[arg(long)]
        id: String,
        /// 正本の置き場
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 出力先（既定なし・相対なら --dir からの相対・絶対ならそのまま）
        #[arg(long)]
        out: PathBuf,
        /// 導出した図の本体を出力先へ書く
        #[arg(long)]
        write: bool,
        /// 出力先と導出の byte 一致を検査する（無い 2・不一致 1・一致 0）
        #[arg(long)]
        check: bool,
    },
    /// 3 面 + 判断の記録の面 + 設計ノートの面 + 様式 2 本を 1 つの配信先へまとめて出す（--write）・配信先と正本の一致を検査する（--check）
    #[command(group(ArgGroup::new("mode").required(true).args(["write", "check"])))]
    Build {
        /// 正本の置き場
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 配信先（既定なし・相対なら --dir からの相対・絶対ならそのまま）
        #[arg(long)]
        out: PathBuf,
        /// 3 面 + 判断の記録の面 + 設計ノートの面 + 様式 2 本を配信先へ書く（全部か無しか）
        #[arg(long)]
        write: bool,
        /// 配信先の 3 面 + 判断の記録の面 + 設計ノートの面 + 様式 2 本と正本の byte 一致を検査する（無い 2・不一致 1・一致 0）
        #[arg(long)]
        check: bool,
    },
    /// 相談窓口の答えの無い質問と推奨回答を散文で出す（--print）・回答から支度表 1 枚を書く（--write）
    #[command(group(ArgGroup::new("mode").required(true).args(["print", "write"])))]
    Intake {
        /// 正本の置き場（intake.yaml と、在れば支度表を読む）
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 回答の file（相対なら --dir からの相対・絶対ならそのまま）
        #[arg(long)]
        answers: Option<PathBuf>,
        /// 答えの無い質問を標準出力へ書く
        #[arg(long)]
        print: bool,
        /// 支度表を <dir>/<sheet.file> へ書く（在れば上書き）
        #[arg(long)]
        write: bool,
    },
    /// 設計文書がまだ無いことを 1 行だけ知らせる（整備済み・止める設定・出した印のどれかが在れば何も出さない）
    Hello {
        /// 正本の置き場
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 印の置き場（既定は環境変数 XDG_STATE_HOME の下の folio・無ければ HOME の下の .local/state/folio）
        #[arg(long)]
        state: Option<PathBuf>,
    },
    /// 配信先を tailnet の内側だけで見せる（bind 先が tailnet の外なら起動を拒む）
    Serve {
        /// 配信先（folio build の --out）
        #[arg(long)]
        dir: PathBuf,
        /// bind 先の IPv4（既定なし = tailnet の住所を自分で解く）
        #[arg(long)]
        host: Option<String>,
        /// bind 先の port（既定 0 = 空きを OS が選ぶ）
        #[arg(long, default_value_t = 0)]
        port: u16,
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
        Command::Parts {
            dir,
            css,
            pages,
            check: _,
            print,
        } => {
            if print {
                print!("{}", parts::print_catalog());
                return ExitCode::from(0);
            }
            let report = parts::check(&dir, css.as_deref(), &pages);
            for (kind, msg) in &report.violations {
                println!("[{kind}] {msg}");
            }
            for msg in report.unknowns.iter().chain(&report.pendings) {
                eprintln!("# まだ分からない: {msg}");
            }
            let verdict = report.verdict();
            println!(
                "folio parts: {verdict}（違反 {}・まだ分からない {}）",
                report.violations.len(),
                report.unknowns.len() + report.pendings.len()
            );
            ExitCode::from(verdict.exit_code() as u8)
        }
        Command::Face {
            face,
            id,
            dir,
            out,
            write,
            check: _,
        } => {
            let mode = if write {
                face::Mode::Write
            } else {
                face::Mode::Check
            };
            let outcome = face::run(&face, id.as_deref(), &dir, &out, mode);
            if let Some(line) = &outcome.stdout {
                println!("{line}");
            }
            if let Some(line) = &outcome.stderr {
                eprintln!("{line}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
        Command::Figure {
            doc,
            id,
            dir,
            out,
            write,
            check: _,
        } => {
            let mode = if write {
                figure::Mode::Write
            } else {
                figure::Mode::Check
            };
            let outcome = figure::run(&doc, &id, &dir, &out, mode);
            if let Some(line) = &outcome.stdout {
                println!("{line}");
            }
            if let Some(line) = &outcome.stderr {
                eprintln!("{line}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
        Command::Build {
            dir,
            out,
            write,
            check: _,
        } => {
            let mode = if write {
                site::Mode::Write
            } else {
                site::Mode::Check
            };
            let outcome = site::run(&dir, &out, mode);
            if let Some(line) = &outcome.stdout {
                println!("{line}");
            }
            if let Some(line) = &outcome.stderr {
                eprintln!("{line}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
        Command::Intake {
            dir,
            answers,
            print: _,
            write,
        } => {
            let mode = if write {
                sheet::Mode::Write
            } else {
                sheet::Mode::Print
            };
            let outcome = sheet::run(&dir, answers.as_deref(), mode);
            for line in &outcome.stdout {
                println!("{line}");
            }
            if let Some(line) = &outcome.stderr {
                eprintln!("{line}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
        Command::Hello { dir, state } => {
            let outcome = hello::run(&dir, state.as_deref());
            if let Some(line) = &outcome.stdout {
                println!("{line}");
            }
            if let Some(line) = &outcome.stderr {
                eprintln!("{line}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
        Command::Serve { dir, host, port } => {
            let verdict = serve::run(&dir, host.as_deref(), port);
            ExitCode::from(verdict.exit_code() as u8)
        }
    }
}
