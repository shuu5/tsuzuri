//! folio v2 の命令の入口（lib の入口・行 k-tz-entry）。便 0・便 1 の `folio check` と便 45 の
//! `folio schema` ほかを持つ。口 inject と serve は行 k-tz-drop で退役させた（`folio2/retired/` の下）。
//! 引数の列（頭は命令の名）と標準出力・標準エラーの書き先を受け、終了 code を返す。
//! binary の `main.rs` も tz の口も同じ入口を撃つ。

use std::io::Write;
use std::path::PathBuf;

use clap::{ArgGroup, Parser, Subcommand};

use crate::{
    bundle, ceiling_src, derive, face, figure, findings, floor_note, freeze, gate, graph, hello,
    init, mentions, parts,
    phase::{After, Flag},
    polarity, proposed, rules, schema, sheet, site, stamp,
    verdict::Verdict,
};

/// 書き先へ 1 行を書く。書けなければ黙る（閉じた pipe で落ちない）。
macro_rules! say {
    ($to:expr, $($arg:tt)*) => {{
        let _ = writeln!($to, $($arg)*);
    }};
}

#[derive(Parser)]
#[command(name = "folio", version, about = "folio v2 — 設計文書の生成と検査")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 設計文書の置き場の床（正本 7 file〔憲法・規則の表・語彙・要件書・入口・相談窓口・天井〕の形と、判断の記録・設計ノート・凍結 anchor・索引の欄の決まりと、参照 id と、索引と面が組めるか）を検査し、合格 0 / 不合格 1 / まだ分からない 2 で終わる
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
        /// 全検査が 0 違反で測れないも無いときだけ、発効した判断の記録の封の欠けた行を anchors/adr-seals.yaml の末尾に足す（在る行と本文が違えば断る・書いたら commit する）
        #[arg(long, conflicts_with_all = ["emit_amends", "freeze_anchor", "freeze_ids", "freeze_start"])]
        freeze_adrs: bool,
        /// 決定の欄から切り出した裁定 id を全部、1 件 1 行の JSON（ruling・form・bead・node・file・line・field）で標準出力へ書く（違反と要約は標準エラーへ・終了コードは素の床と同じ・一覧が全数なのは 0 のときだけ）
        #[arg(long, conflicts_with_all = ["emit_amends", "freeze_anchor", "freeze_ids", "freeze_start", "freeze_adrs"])]
        emit_rulings: bool,
        /// 置き場の中の 1 file（置き場からの相対）に標準入力の中身を書いた後の床を、書く前の床と比べ、後にだけ在る違反を返す（編集時の口・通す 0 / 止める 1 / まだ分からない 2・つながりの違反は止めない・file は書かない）
        #[arg(long, value_name = "PATH", conflicts_with_all = ["emit_amends", "freeze_anchor", "freeze_ids", "freeze_start", "freeze_adrs", "emit_rulings"])]
        proposed: Option<PathBuf>,
        /// 止める仕掛けの一覧（極性一覧）を 1 仕掛け 1 行（名 · 段 · 極性 · 出所）と集計の 1 行で標準出力へ書く（正本が読めなければ まだ分からない 2）
        #[arg(long, conflicts_with_all = ["emit_amends", "freeze_anchor", "freeze_ids", "freeze_start", "freeze_adrs", "emit_rulings", "proposed"])]
        polarity: bool,
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
    /// 正本から 5 面（入口・憲法・要件書・判断の記録・設計ノート）の 1 面を導出して書く（--write）・検査する（--check）
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
        /// 導出物の置き場（既定なし・消費側が宣言する・相対なら --dir からの相対〔--from-root なら根からの相対〕・絶対ならそのまま）
        #[arg(long)]
        out: PathBuf,
        /// --out の相対を、置き場を含む版管理の根（無ければ置き場の親 dir・器の導出 file を探す根と同じ）からの相対に解く
        #[arg(long)]
        from_root: bool,
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
    /// 設計文書の正本から節点と辺の索引を組み、2 つの表と要約の 1 行（--print）か、節点ごとの 1 行の JSON（--print --summary）か、
    /// 3 つの表と要約の 2 行の短い出力（--digest）を標準出力へ出す（repo へは書かない・組めなければ まだ分からない）
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
        /// --print の表の代わりに、節点ごとに id の行の番号・平易文・技術の要約を添えた 1 行の JSON（JSON Lines）を書く
        #[arg(long, conflicts_with = "digest")]
        summary: bool,
    },
    /// 新しい置き場に最初の文書一式（骨格・11 file）を書く。正本が 1 つでも在れば何も書かずに断る（書いた 0・断り 1・書けない 2）
    Init {
        /// 骨格を書く置き場（既定なし・相対なら撃った場所からの相対）
        #[arg(long)]
        dir: PathBuf,
    },
}

/// 命令の入口。`args` の頭は命令の名（clap の usage の行に出る字）・以降は口の名と旗。`out` と `err` へ書き、終了 code
/// （合格 0・不合格 1・まだ分からない 2・clap の読みの誤り 2・`--help` 0）を返す。読みの誤りと `--help` は clap の字（色なし）を
/// `use_stderr` が真なら `err`・偽なら `out` へ書く。
pub fn run<I, T>(args: I, out: &mut dyn Write, err: &mut dyn Write) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let code = match Cli::try_parse_from(args) {
        Ok(cli) => dispatch(cli, out, err),
        Err(e) => {
            let text = e.render().to_string();
            let _ = if e.use_stderr() {
                err.write_all(text.as_bytes())
            } else {
                out.write_all(text.as_bytes())
            };
            u8::try_from(e.exit_code()).unwrap_or(2)
        }
    };
    // 焼いた図の道具を書き出した一時の置き場は命令の終わりに消す（便 168・書き出していなければ何もしない）
    figure::sweep();
    code
}

/// 編集時の口の まだ分からない の行の接頭辞（素の床の まだ分からない の行と同じ字・器は標準出力を逐語で断りの字に写す）。
const UNKNOWN_HEAD: &str = "# まだ分からない: ";

/// 終了 code（folio の 3 値）。
fn code(verdict: Verdict) -> u8 {
    verdict.exit_code() as u8
}

/// 口が数えられなかった周（理由の 1 行と要約の 1 行を標準出力・終了 2）。
fn proposed_refused(why: &str, out: &mut dyn Write) -> u8 {
    say!(out, "{UNKNOWN_HEAD}{why}");
    say!(out, "folio check --proposed: まだ分からない（口は数えていない）");
    code(Verdict::Unknown)
}

/// 編集時の口（便 198・ADR-33 決定 (1)(3)）。止める行・まだ分からない の行・要約・つながりの行の順に標準出力（器が断りの字に写し、
/// 行の数で切るときはつながりから落ちる）。標準エラーは診断だけ。
fn proposed_check(dir: &std::path::Path, rel: &std::path::Path, out: &mut dyn Write) -> u8 {
    let mut content = String::new();
    if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut content) {
        return proposed_refused(&format!("標準入力を字（UTF-8）として読めない: {e}"), out);
    }
    let judged = match proposed::judge(dir, rel, &content) {
        Ok(j) => j,
        Err(why) => return proposed_refused(&why, out),
    };
    // 名札は置き場の今の憲法と規則の表で引く（便 203・書く前と後の突き合わせは名札の元の字で数える）
    let labels = rules::Labels::of(dir);
    for (kind, msg) in &judged.stop {
        say!(out, "[{}] {msg}", labels.shown(kind));
    }
    for msg in &judged.unknowns {
        say!(out, "{UNKNOWN_HEAD}{msg}");
    }
    say!(
        out,
        "folio check --proposed: {}（新しい違反 {}・つながり {}・まだ分からない {}・書く前から在る まだ分からない {}・面の段は数えない）",
        judged.word(),
        judged.stop.len(),
        judged.links.len(),
        judged.unknowns.len(),
        judged.before_unknowns
    );
    for (kind, msg) in &judged.links {
        say!(out, "# つながり（編集は止めない・事後の床が数える）: [{}] {msg}", labels.shown(kind));
    }
    code(judged.verdict())
}

/// 床の口の 1 行の書き先。--emit-amends では標準出力を貼れる差分だけに、--emit-rulings では JSON の行だけにするので
/// 違反と要約の行は標準エラーへ、ほかは標準出力へ。
fn emit(flag: Flag, out: &mut dyn Write, err: &mut dyn Write, line: &str) {
    if matches!(flag, Flag::EmitAmends | Flag::EmitRulings) {
        say!(err, "{line}");
    } else {
        say!(out, "{line}");
    }
}

fn dispatch(cli: Cli, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match cli.command {
        Command::Check {
            dir,
            proposed: Some(rel),
            ..
        } => proposed_check(&dir, &rel, out),
        Command::Check { dir, polarity: true, .. } => match polarity::render(&dir) {
            Ok(lines) => {
                lines.iter().for_each(|l| say!(out, "{l}"));
                0
            }
            Err(why) => {
                why.iter().for_each(|w| say!(out, "{UNKNOWN_HEAD}{w}"));
                say!(out, "folio check --polarity: まだ分からない（一覧を組めない）");
                code(Verdict::Unknown)
            }
        },
        Command::Check {
            dir,
            emit_amends,
            freeze_anchor,
            freeze_ids,
            freeze_start,
            freeze_adrs,
            emit_rulings,
            proposed: None,
            polarity: false,
        } => {
            let flag = if emit_amends {
                Flag::EmitAmends
            } else if freeze_anchor {
                Flag::FreezeAnchor
            } else if freeze_ids {
                Flag::FreezeIds
            } else if freeze_start {
                Flag::FreezeStart
            } else if freeze_adrs {
                Flag::FreezeAdrs
            } else if emit_rulings {
                Flag::EmitRulings
            } else {
                Flag::None
            };
            // 索引が組めない置き場を合格と言わない（便 136・層 2 の check_dir からは呼ばない）。編集時の口と同じ 1 本（便 198）
            let (mut report, materials) = proposed::floor(&dir, flag);
            // 面が組めない置き場を合格と言わない（便 187・編集時の口は面の段を撃たない＝ADR-33 決定 (6)）
            site::check_faces(&dir, &mut report);
            // 凍結の後始末は口を出た直後に 1 度だけ（判定の印字より前・後始末が足す違反も判定に入る）
            let after = freeze::after(
                &dir,
                flag,
                materials.state.as_ref(),
                materials.adr.as_ref(),
                materials.ids.as_ref(),
                materials.seals.as_ref(),
                &mut report,
            );
            if let After::Refused(msg) = &after {
                say!(err, "folio check: {msg}");
                return 1;
            }
            let labels = rules::Labels::of(&dir);
            for (kind, msg) in &report.violations {
                emit(flag, out, err, &format!("[{}] {msg}", labels.shown(kind)));
            }
            for msg in report.unknowns.iter().chain(&report.pendings) {
                say!(err, "# まだ分からない: {msg}");
            }
            // 便 207（ADR-35 決定 (1)(オ)）: 数の上限の違反の字は今の数を持たないので、今の数を違反ごとに 1 行
            for msg in &report.counts {
                say!(err, "# 今の数: {msg}");
            }
            // 便 156（FR5）: 行 R-17 が無くて散文の言及の歯が数えなかったら、判定を変えずに 1 行（機構の行より前）
            if materials.mentions_off {
                say!(err, "{}", mentions::OFF);
            }
            if materials.in_loop_min_off {
                say!(err, "{}", polarity::off(&dir));
            }
            // 便 131（ADR-23 決定 (3)）: 機構がまだ無い条は判定に数えず、要約の行の直前に 1 行（0 本なら出さない）
            if !materials.not_yet_live.is_empty() {
                say!(
                    err,
                    "# 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: {}",
                    materials.not_yet_live.join("・")
                );
            }
            let verdict = report.verdict();
            emit(
                flag,
                out,
                err,
                &format!(
                    "folio check: {verdict}（違反 {}・まだ分からない {}）",
                    report.violations.len(),
                    report.unknowns.len() + report.pendings.len()
                ),
            );
            match after {
                After::Emit(lines) => {
                    for line in lines {
                        say!(out, "{line}");
                    }
                }
                After::Freeze(msg) => say!(err, "folio check: {msg}"),
                After::Nothing | After::Refused(_) => {}
            }
            for line in &materials.rulings {
                say!(out, "{line}");
            }
            code(verdict)
        }
        Command::Parts {
            dir,
            css,
            pages,
            check: _,
            print,
        } => {
            if print {
                let _ = out.write_all(parts::print_catalog().as_bytes());
                return 0;
            }
            let report = parts::check(&dir, css.as_deref(), &pages);
            let labels = rules::Labels::of(&dir);
            for (kind, msg) in &report.violations {
                say!(out, "[{}] {msg}", labels.shown(kind));
            }
            for msg in report.unknowns.iter().chain(&report.pendings) {
                say!(err, "# まだ分からない: {msg}");
            }
            let verdict = report.verdict();
            say!(
                out,
                "folio parts: {verdict}（違反 {}・まだ分からない {}）",
                report.violations.len(),
                report.unknowns.len() + report.pendings.len()
            );
            code(verdict)
        }
        Command::Face {
            face,
            id,
            dir,
            out: path,
            write,
            check: _,
        } => {
            let mode = if write {
                face::Mode::Write
            } else {
                face::Mode::Check
            };
            let outcome = face::run(&face, id.as_deref(), &dir, &path, mode);
            if let Some(line) = &outcome.stdout {
                say!(out, "{line}");
            }
            if let Some(line) = &outcome.stderr {
                say!(err, "{line}");
            }
            code(outcome.verdict)
        }
        Command::Figure {
            doc,
            id,
            dir,
            out: path,
            write,
            check: _,
        } => {
            let mode = if write {
                figure::Mode::Write
            } else {
                figure::Mode::Check
            };
            let outcome = figure::run(&doc, &id, &dir, &path, mode);
            if let Some(line) = &outcome.stdout {
                say!(out, "{line}");
            }
            if let Some(line) = &outcome.stderr {
                say!(err, "{line}");
            }
            code(outcome.verdict)
        }
        Command::Build {
            dir,
            out: path,
            write,
            check: _,
        } => {
            let mode = if write {
                site::Mode::Write
            } else {
                site::Mode::Check
            };
            let outcome = site::run(&dir, &path, mode);
            if let Some(line) = &outcome.stdout {
                say!(out, "{line}");
            }
            if let Some(line) = &outcome.stderr {
                say!(err, "{line}");
            }
            code(outcome.verdict)
        }
        Command::Derive {
            dir,
            out: path,
            from_root,
            write,
            check: _,
        } => {
            let mode = if write {
                derive::Mode::Write
            } else {
                derive::Mode::Check
            };
            let outcome = derive::run(&dir, &path, from_root, mode);
            for line in &outcome.stdout {
                say!(out, "{line}");
            }
            if let Some(line) = &outcome.stderr {
                say!(err, "{line}");
            }
            code(outcome.verdict)
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
                say!(out, "{line}");
            }
            if let Some(line) = &outcome.stderr {
                say!(err, "{line}");
            }
            code(outcome.verdict)
        }
        Command::Hello { dir, state } => {
            let outcome = hello::run(&dir, state.as_deref());
            if let Some(line) = &outcome.stdout {
                say!(out, "{line}");
            }
            if let Some(line) = &outcome.stderr {
                say!(err, "{line}");
            }
            code(outcome.verdict)
        }
        Command::Ceiling {
            dir,
            faces,
            out: path,
            write,
            check: _,
            refute,
            stamp,
            gate,
            write_set,
        } => {
            if gate {
                let outcome = gate::run(&dir, &write_set);
                say!(out, "{}", outcome.stdout);
                return code(outcome.verdict);
            }
            let Some(path) = path else {
                say!(err, "folio ceiling: --gate の外では --out が要る");
                return 2;
            };
            if stamp {
                let outcome = stamp::run(&dir, &path);
                say!(out, "{}", outcome.stdout);
                return code(outcome.verdict);
            }
            if refute {
                let outcome = findings::refute(&dir, &path);
                for line in &outcome.stdout {
                    say!(out, "{line}");
                }
                for line in &outcome.stderr {
                    say!(err, "{line}");
                }
                return code(outcome.verdict);
            }
            // --write と --check は配信先が要る（束が古くないかを測る）
            let Some(faces) = faces else {
                let outcome = ceiling_src::Outcome::unknown("--faces が要る");
                if let Some(line) = &outcome.stderr {
                    say!(err, "{line}");
                }
                return code(outcome.verdict);
            };
            if write {
                let outcome = bundle::run(&dir, &faces, &path);
                if let Some(line) = &outcome.stdout {
                    say!(out, "{line}");
                }
                if let Some(line) = &outcome.stderr {
                    say!(err, "{line}");
                }
                return code(outcome.verdict);
            }
            let outcome = findings::run(&dir, &faces, &path);
            for line in &outcome.stdout {
                say!(out, "{line}");
            }
            for line in &outcome.stderr {
                say!(err, "{line}");
            }
            code(outcome.verdict)
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
                say!(out, "{line}");
            }
            if let Some(line) = &outcome.stderr {
                say!(err, "folio schema: {line}");
            }
            code(outcome.verdict)
        }
        Command::Graph {
            dir,
            print: _,
            digest,
            summary,
        } => {
            let outcome = graph::run(&dir, digest, summary);
            if let Some(body) = &outcome.stdout {
                let _ = out.write_all(body.as_bytes());
            }
            if let Some(line) = &outcome.stderr {
                say!(err, "folio graph: {line}");
            }
            code(outcome.verdict)
        }
        Command::Init { dir } => {
            let outcome = init::run(&dir);
            for line in &outcome.stdout {
                say!(out, "{line}");
            }
            for line in &outcome.stderr {
                say!(err, "folio init: {line}");
            }
            code(outcome.verdict)
        }
    }
}
