//! folio v2 の命令の入口。便 0・便 1 の `folio check` と便 2 の `folio inject`・便 45 の `folio schema` ほかを持つ。

mod adr;
mod anchor;
mod bundle;
mod catalog;
mod ceiling;
mod ceiling_src;
mod check;
mod constitution_enums;
mod cursor;
mod derive;
mod entrance;
mod face;
mod face_adr;
mod face_constitution;
mod face_constitution_read;
mod face_index;
mod face_index_read;
mod face_labels;
mod face_note;
mod face_srs;
mod face_srs_items;
mod face_srs_rtm;
mod figure;
mod findings;
mod floor;
mod floor_adr;
mod floor_note;
mod freeze;
mod gate;
mod gitcheck;
mod graph;
mod hello;
mod ids;
mod init;
mod inject;
mod intake;
mod lineage;
mod link;
mod mentions;
mod note;
mod parts;
mod phase;
mod prose;
mod refs;
mod rules;
mod schema;
mod serve;
mod sha256;
mod sheet;
mod shelf;
mod site;
mod stamp;
mod verdict;
mod vocab;
mod yaml;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgGroup, Parser, Subcommand};

use crate::phase::{After, Flag};

#[derive(Parser)]
#[command(name = "folio", version, about = "folio v2 — 設計文書の生成と検査")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// design-intent の正本 7 file の形を検査し、合格 0 / 不合格 1 / まだ分からない 2 で終わる
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
        /// 全検査が 0 違反で測れないも無いときだけ、要件・判断・受入基準の id の一覧を anchors/ids-<要件書の版>.yaml として書く（書いたら commit する）
        #[arg(long, conflicts_with_all = ["emit_amends", "freeze_anchor"])]
        freeze_ids: bool,
        /// 憲法の列と id の一覧がどちらも無い置き場でだけ、全検査が 0 違反で測れないも無いときに最初の版の anchor と索引と id の一覧を同時に書く（書いたら commit する）
        #[arg(long, conflicts_with_all = ["emit_amends", "freeze_anchor", "freeze_ids"])]
        freeze_start: bool,
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
    /// 部品目録から組み立て時に導出した一覧を出す（--print）・面の class と部品の名札と行内の様式を部品目録と突き合わせる（--check）
    #[command(group(ArgGroup::new("mode").required(true).args(["check", "print"])))]
    Parts {
        /// 正本の置き場（preview/parts.json を読む）
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 様式の定義（既定は --dir の下の preview/folio.css）
        #[arg(long)]
        css: Option<PathBuf>,
        /// 面の名と path（<面の名>=<path>・何度でも・1 つも無ければ --dir の下の preview/ の index / constitution / srs / adr / note）
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
    /// 契約表を持つ設計ノートから器の形の導出物（1 文書 1 file の <文書 id>.toml）を置き場へ書く（--write）・置き場の導出物との
    /// byte 一致を数える（--check）。配信の組み立てから切り離した独立の命令で、面の生成器も様式の file も呼ばない
    #[command(name = floor_note::DERIVED_SUBCOMMAND, group(ArgGroup::new("mode").required(true).args(["write", "check"])))]
    Derive {
        /// 正本の置き場（design-note/ と、版管理の根〔無ければ置き場の親 dir〕の contracts/schema.toml を読む）
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 導出物の置き場（既定なし・消費側が宣言する・相対なら --dir からの相対・絶対ならそのまま）
        #[arg(long)]
        out: PathBuf,
        /// 違う導出物だけを置き場へ書く（全部か無しか・導出元の無い .toml は消さずに名を出す）
        #[arg(long)]
        write: bool,
        /// 導出元を持つ導出物と置き場の file の byte 一致を数える（一致 0・違う・置き場に無い 1・導出できない・置き場が無い 2）
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
    /// 設計文書がまだ無いことを 1 行だけ知らせる（止める設定・出した印のどれかが在れば何も出さない）。整備済みなら索引の数と
    /// 全体像の口を 1 行で毎回知らせる
    Hello {
        /// 正本の置き場
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 印の置き場（既定は環境変数 XDG_STATE_HOME の下の folio・無ければ HOME の下の .local/state/folio）
        #[arg(long)]
        state: Option<PathBuf>,
    },
    /// 天井の材料の束を観点ごとに置き場へ組む（--write）・席か器が書いた所見 file を数えて観点ごとの 3 値を返す（--check）・
    /// 止める の所見ごとに反証の材料の束を組む（--refute）・周の結果から天井の印を書く（--stamp）・
    /// 印と便の書き換える file の一覧から門の 3 値を返す（--gate）。AI は起動しない
    #[command(group(ArgGroup::new("mode").required(true).args(["write", "check", "refute", "stamp", "gate"])))]
    Ceiling {
        /// 正本の置き場
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 配信先（folio build の --out・相対なら --dir からの相対・絶対ならそのまま・--write と --check に要る〔--check でも束が古くないかを測る〕・--refute は読まない）
        #[arg(long)]
        faces: Option<PathBuf>,
        /// 束の置き場（既定なし・相対なら --dir からの相対・絶対ならそのまま・--gate は読まない）
        #[arg(long, required_unless_present = "gate")]
        out: Option<PathBuf>,
        /// 観点ごとの束（sources/・faces/・question.yaml・finding.yaml・reads.yaml・digest.txt）を置き場へ書く（全部か無しか）
        #[arg(long)]
        write: bool,
        /// 観点ごとの所見 file（findings.yaml）を天井の正本の欄の決まりで数える（4 観点が全部合格 0・不合格 1・まだ分からない 2）
        #[arg(long)]
        check: bool,
        /// 反証が未の 止める の所見ごとに反証の材料の束（finding・question・reads・schema・sources.txt・digest.txt）を <out>/<観点>/refute/<所見の id>/ へ組む（全部か無しか）
        #[arg(long)]
        refute: bool,
        /// 周の結果（観点ごとの所見 file と止めるの反証の結果）から天井の印を導出し <dir>/preview/ceiling-stamp.yaml へ書く（同じなら書かない・組めなければ まだ分からない で何も書かない）
        #[arg(long)]
        stamp: bool,
        /// 印 <dir>/preview/ceiling-stamp.yaml と --write-set から門の 3 値を返す（通す 0・止める 1・まだ分からない 2・何も書かない）
        #[arg(long, requires = "write_set")]
        gate: bool,
        /// 便の書き換える file（repo の根からの相対・接頭辞 + / - / ~ は剥がす・1 つ以上・--gate に要る）
        #[arg(long = "write-set", value_name = "PATH", num_args = 1.., requires = "gate")]
        write_set: Vec<String>,
    },
    /// 欄の決まりの file の schema 節（生成区間）を床の定数から導出して書く（--write）・検査する（--check）
    #[command(group(ArgGroup::new("mode").required(true).args(["write", "check"])))]
    Schema {
        /// 設計文書の置き場（生成区間を持つ file 9 本 = adr/schema.yaml・design-note/schema.yaml・ceiling.yaml・rules.yaml・index.yaml・srs.yaml・vocabulary.yaml・intake.yaml・graph.yaml を読む）
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 生成区間を導出で置き換えて書く（既に同じなら書かない・区間の外の byte は変えない）
        #[arg(long)]
        write: bool,
        /// 生成区間と導出の byte 一致を検査する（読めない・印が 1 対でない 2・不一致 1・一致 0）
        #[arg(long)]
        check: bool,
    },
    /// 設計文書の正本から節点と辺の索引を組み、2 つの表と要約の 1 行（--print）か、3 つの表と要約の 2 行の短い出力（--digest）を
    /// 標準出力へ出す（repo へは書かない・組めなければ まだ分からない）
    #[command(group(ArgGroup::new("mode").required(true).args(["print", "digest"])))]
    Graph {
        /// 正本の置き場（constitution.yaml・rules.yaml・srs.yaml・adr/ADR-*.yaml を読む）
        #[arg(long, default_value = "design-intent")]
        dir: PathBuf,
        /// 索引を標準出力へ書く
        #[arg(long)]
        print: bool,
        /// 全体像の短い出力（種類ごとの節点・型ごとの辺・file ごとの節点の数）を標準出力へ書く
        #[arg(long)]
        digest: bool,
    },
    /// 新しい置き場に最初の文書一式（骨格・11 file）を書く。正本が 1 つでも在れば何も書かずに断る（書いた 0・断り 1・書けない 2）
    Init {
        /// 骨格を書く置き場（既定なし・相対なら撃った場所からの相対）
        #[arg(long)]
        dir: PathBuf,
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
            freeze_ids,
            freeze_start,
        } => {
            let flag = if emit_amends {
                Flag::EmitAmends
            } else if freeze_anchor {
                Flag::FreezeAnchor
            } else if freeze_ids {
                Flag::FreezeIds
            } else if freeze_start {
                Flag::FreezeStart
            } else {
                Flag::None
            };
            let (mut report, materials) = check::check_dir(&dir, flag);
            // 索引と天井の印が組めない置き場を合格と言わない（便 136・層 2 の check_dir からは呼ばない）
            graph::check_index(&dir, &mut report);
            // 凍結の後始末は口を出た直後に 1 度だけ（判定の印字より前・後始末が足す違反も判定に入る）
            let after = freeze::after(
                &dir,
                flag,
                materials.state.as_ref(),
                materials.adr.as_ref(),
                materials.ids.as_ref(),
                &mut report,
            );
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
            // 便 156（FR5）: 行 R-17 が無くて散文の言及の歯が数えなかったら、判定を変えずに 1 行（機構の行より前）
            if materials.mentions_off {
                eprintln!("{}", mentions::OFF);
            }
            // 便 131（ADR-23 決定 (3)）: 機構がまだ無い条は判定に数えず、要約の行の直前に 1 行（0 本なら出さない）
            if !materials.not_yet_live.is_empty() {
                eprintln!(
                    "# 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: {}",
                    materials.not_yet_live.join("・")
                );
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
        Command::Derive {
            dir,
            out,
            write,
            check: _,
        } => {
            let mode = if write {
                derive::Mode::Write
            } else {
                derive::Mode::Check
            };
            let outcome = derive::run(&dir, &out, mode);
            for line in &outcome.stdout {
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
        Command::Ceiling {
            dir,
            faces,
            out,
            write,
            check: _,
            refute,
            stamp,
            gate,
            write_set,
        } => {
            if gate {
                let outcome = gate::run(&dir, &write_set, graph::stamp_table);
                println!("{}", outcome.stdout);
                return ExitCode::from(outcome.verdict.exit_code() as u8);
            }
            let out = out.expect("--gate の外では clap が --out を要る");
            if stamp {
                let outcome = stamp::run(&dir, &out);
                println!("{}", outcome.stdout);
                return ExitCode::from(outcome.verdict.exit_code() as u8);
            }
            if refute {
                let outcome = findings::refute(&dir, &out);
                for line in &outcome.stdout {
                    println!("{line}");
                }
                for line in &outcome.stderr {
                    eprintln!("{line}");
                }
                return ExitCode::from(outcome.verdict.exit_code() as u8);
            }
            // --write と --check は配信先が要る（束が古くないかを測る）
            let Some(faces) = faces else {
                let outcome = ceiling_src::Outcome::unknown("--faces が要る");
                if let Some(line) = &outcome.stderr {
                    eprintln!("{line}");
                }
                return ExitCode::from(outcome.verdict.exit_code() as u8);
            };
            if write {
                let outcome = bundle::run(&dir, &faces, &out);
                if let Some(line) = &outcome.stdout {
                    println!("{line}");
                }
                if let Some(line) = &outcome.stderr {
                    eprintln!("{line}");
                }
                return ExitCode::from(outcome.verdict.exit_code() as u8);
            }
            let outcome = findings::run(&dir, &faces, &out);
            for line in &outcome.stdout {
                println!("{line}");
            }
            for line in &outcome.stderr {
                eprintln!("{line}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
        Command::Schema {
            dir,
            write,
            check: _,
        } => {
            let mode = if write {
                schema::Mode::Write
            } else {
                schema::Mode::Check
            };
            let outcome = schema::run(&dir, mode);
            for line in &outcome.stdout {
                println!("{line}");
            }
            if let Some(line) = &outcome.stderr {
                eprintln!("folio schema: {line}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
        Command::Graph {
            dir,
            print: _,
            digest,
        } => {
            let outcome = graph::run(&dir, digest);
            if let Some(body) = &outcome.stdout {
                print!("{body}");
            }
            if let Some(line) = &outcome.stderr {
                eprintln!("folio graph: {line}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
        Command::Init { dir } => {
            let outcome = init::run(&dir);
            for line in &outcome.stdout {
                println!("{line}");
            }
            for line in &outcome.stderr {
                eprintln!("folio init: {line}");
            }
            ExitCode::from(outcome.verdict.exit_code() as u8)
        }
        Command::Serve { dir, host, port } => {
            let verdict = serve::run(&dir, host.as_deref(), port);
            ExitCode::from(verdict.exit_code() as u8)
        }
    }
}
