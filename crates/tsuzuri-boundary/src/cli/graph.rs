//! tz graph（要件 FR2・FR3・FR17）: 導出グラフを組み直して出す。書かない。
//! - 旗なし — GraphDoc の電文を標準出力に 1 行
//! - --check — 不変条件の 12 本を 3 値で数え、違反の行と要約の行を標準出力、まだ分からないの行を標準エラー
//!   （g-7 が folio の裁定 id の文法の外の id の裁定だけのためにまだ分からないなら、理由にその数を書く）
//!   （g-3 が --project の組の台帳で確かめられない外の台帳の行だけのためにまだ分からないなら、理由に `OUTSIDE_HEAD` と
//!   その節点の数と族を書く）
//!   要約の行の前に要約の無い節点の数と id の行（読めなければまだ分からないの行を標準エラー・終了 code は変えない）
//!   その次に本文だけで名指した id の対の数と対の行（g-9 の detect の数・読めなければまだ分からないの行を標準エラー・
//!   終了 code は変えない）
//!   その次に全部の契約表の行が着地した設計ノート（廃止したものを除く）の数と文書 id の行（設計の索引か台帳か
//!   要約の状態の欄が読めなければまだ分からないの行を標準エラー・終了 code は変えない）
//!   その次に着地の commit の節点から landed の辺を受けない着地した契約の数と id の行（台帳か走行の記録が読めなければ
//!   まだ分からないの行を標準エラー・終了 code は変えない）
//! - --design — 設計の索引だけを読み、設計の節点と辺に絞った GraphDoc の電文（folio の graph の吸収・ADR-8 決定 (3)）
//!
//! 終了 code は folio の床の check の口に揃える（合格 0・不合格 1・まだ分からない 2）。旗なしと --design は
//! 読めない出所が無ければ 0、在れば 2。使い方の誤りは 1。組みは口 /api/graph と同じ `Sources::gather` と
//! `board::built_floors`（`board::built` と同じ組み）と `board::doc` の呼び。
//! --project（何度でも）は旗なしと --check が置き場ごとに読み取りの bd を 1 本ずつ、3 つの字の集めと並べて撃ち、
//! 外の台帳の読みを g-3 に渡す（--design は読まない）。

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::thread;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{
    EdgeType, GraphDoc, GraphSource, InvariantCheck, SkippedEdges, Verdict,
};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::{self, Outside, build, build::DESIGN_EDGE_TYPES, check::UNMEASURED};

use crate::out::{emit, emit_err};
use crate::server::board::{self, Floors, Sources, Texts};
use crate::server::design::Design;
use crate::server::ledger::{BD, Source};
use crate::server::runs::Runs;

pub const USAGE: &str = "usage: tz graph [--check | --design] [--repo <dir>] [--bd <program>] [--folio <program>] [--state-dir <dir>] [--project <dir>]...";

/// g-3 が --project の組の台帳で確かめられない外の台帳の行だけのためにまだ分からないときの理由の頭の字
/// （空白と節点の数と（族 と中黒つなぎの族と）が続く）。
pub const OUTSIDE_HEAD: &str = "--project の組の台帳で確かめられない外の台帳の行を持つ節点";

/// 不変条件の id と次の 1 手（中核の INVARIANTS の順）。
pub const NEXT: [(&str, &str); 12] = [
    (
        "g-1",
        "片端の無い辺の元の行（索引の辺・bead の依存・metadata）を直すか、消えた節点を戻す",
    ),
    (
        "g-2",
        "契約の bead の acceptance に design = で始まる行をちょうど 1 つ置く",
    ),
    (
        "g-3",
        "発効した記録の承認欄に台帳の id と裁定 id を書き、その bead の notes に裁定の行を置く",
    ),
    (
        "g-4",
        "問いの bead の metadata の touches に関わる節点の id を 1 つ以上書く",
    ),
    (
        "g-5",
        "bead に根の epic へ parent-child でたどれる親を付ける",
    ),
    ("g-6", "blocks の輪を切り、parent-child の親を 1 つにする"),
    (
        "g-7",
        "裁定を問い（answers）か発効した記録（ruled_by）に結ぶ",
    ),
    (
        "g-8",
        "memo の label か design = で始まる行のどちらかを外す",
    ),
    ("g-9", "本文で名指した id を欄か辺に書く"),
    ("g-10", "重なった id の片方を改める"),
    (
        "g-11",
        "走行の run_of の先の bead を台帳に置くか、宙に浮いた走行を器で片付ける",
    ),
    (
        "g-12",
        "Questioned の走行に問いを起こすか、走行の段を進める",
    ),
];

/// 要約の無い節点の行の頭の字（行 k-sum-count）。
pub const BARE_HEAD: &str = "要約の無い節点";

/// 要約の無い節点がまだ分からないときの字（標準エラーのまだ分からないの行に続ける）。
pub const BARE_UNKNOWN: &str = "要約の無い節点（要約か設計の索引か台帳が読めない）";

/// 要約の無い節点の行（空なら数 0 だけ、在れば数と中黒つなぎの id・違反の行と同じ形）。
pub fn bare_line(ids: &[String]) -> String {
    if ids.is_empty() {
        format!("{BARE_HEAD} 0")
    } else {
        format!("{BARE_HEAD} {}（{}）", ids.len(), ids.join("・"))
    }
}

/// 本文だけで名指した id の対の行の頭の字（行 k-g9-count）。
pub const UNFIELDED_HEAD: &str = "本文だけで名指した id の対";

/// 本文だけで名指した id の対がまだ分からないときの字（標準エラーのまだ分からないの行に続ける）。
pub const UNFIELDED_UNKNOWN: &str = "本文だけで名指した id の対（要約か設計の索引が読めない）";

/// 本文だけで名指した id の対の行（空なら数 0 だけ、在れば数と、本文を持つ節点の id と矢印と名指した id の
/// 中黒つなぎ・列の順のまま）。
pub fn unfielded_line(pairs: &[(String, String)]) -> String {
    if pairs.is_empty() {
        format!("{UNFIELDED_HEAD} 0")
    } else {
        let named: Vec<String> = pairs
            .iter()
            .map(|(from, to)| format!("{from}→{to}"))
            .collect();
        format!("{UNFIELDED_HEAD} {}（{}）", pairs.len(), named.join("・"))
    }
}

/// 全部の契約表の行が着地した設計ノートの行の頭の字（行 c-note-stale）。
pub const LANDED_HEAD: &str = "全部の行が着地した設計ノート";

/// 全部の行が着地した設計ノートがまだ分からないときの字（標準エラーのまだ分からないの行に続ける）。
pub const LANDED_UNKNOWN: &str = "全部の行が着地した設計ノート（設計の索引か台帳か状態の欄が読めない）";

/// 全部の行が着地した設計ノートの行（空なら数 0 だけ、在れば数と中黒つなぎの文書 id・違反の行と同じ形）。
pub fn landed_line(docs: &[String]) -> String {
    if docs.is_empty() {
        format!("{LANDED_HEAD} 0")
    } else {
        format!("{LANDED_HEAD} {}（{}）", docs.len(), docs.join("・"))
    }
}

/// 着地の commit の無い着地した契約の行の頭の字（行 c-commit-node）。
pub const UNLANDED_HEAD: &str = "着地の commit の無い着地した契約";

/// 着地の commit の無い着地した契約がまだ分からないときの字（標準エラーのまだ分からないの行に続ける）。
pub const UNLANDED_UNKNOWN: &str = "着地の commit の無い着地した契約（台帳か走行の記録が読めない）";

/// 着地の commit の無い着地した契約の行（空なら数 0 だけ、在れば数と中黒つなぎの bead の id・違反の行と同じ形）。
pub fn unlanded_line(ids: &[String]) -> String {
    if ids.is_empty() {
        format!("{UNLANDED_HEAD} 0")
    } else {
        format!("{UNLANDED_HEAD} {}（{}）", ids.len(), ids.join("・"))
    }
}

/// 使い方の誤り。
const FAIL: u8 = 1;

/// 読めない出所が在る（まだ分からない）。
const UNKNOWN: u8 = 2;

/// 出す形（旗なし・--check・--design）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Doc,
    Check,
    Design,
}

/// 読んだ引数。
struct Args {
    mode: Mode,
    repo: PathBuf,
    bd: String,
    folio: OsString,
    state_dir: Option<PathBuf>,
    /// ほかの project の置き場（引数の順・何度でも）。
    projects: Vec<PathBuf>,
}

/// 12 本の判定を 1 つの 3 値にまとめる（違反を先に立てる・空の列はまだ分からない）。
pub fn overall(checks: &[InvariantCheck]) -> Verdict {
    if checks.iter().any(|c| c.verdict == Verdict::Violation) {
        Verdict::Violation
    } else if checks.is_empty() || checks.iter().any(|c| c.verdict == Verdict::Unknown) {
        Verdict::Unknown
    } else {
        Verdict::Pass
    }
}

/// 3 値の終了 code（合格 0・不合格 1・まだ分からない 2）。
pub fn exit_code(verdict: Verdict) -> u8 {
    match verdict {
        Verdict::Pass => 0,
        Verdict::Violation => 1,
        Verdict::Unknown => 2,
    }
}

/// 設計の索引の眺め（設計の種類の節点と索引の型の辺だけ・台帳と走行の部分は空）。
pub fn design_view(doc: &GraphDoc) -> GraphDoc {
    let kinds = graph::Source::Design.kinds();
    let types = &EdgeType::ALL[..DESIGN_EDGE_TYPES];
    GraphDoc {
        nodes: doc
            .nodes
            .iter()
            .filter(|n| kinds.contains(&n.kind))
            .cloned()
            .collect(),
        edges: doc
            .edges
            .iter()
            .filter(|e| types.contains(&e.edge_type))
            .cloned()
            .collect(),
        unread: doc
            .unread
            .iter()
            .copied()
            .filter(|s| *s == GraphSource::Design)
            .collect(),
        beads: BTreeMap::new(),
        runs: BTreeMap::new(),
        invariants: Vec::new(),
        skipped: SkippedEdges {
            design: doc.skipped.design,
            ledger: 0,
            design_nodes: doc.skipped.design_nodes,
        },
        retired: None,
    }
}

/// tz graph の残りの引数を受けて終了 code を返す。
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
    let (doc, outside, heads, floors) = match args.mode {
        Mode::Doc | Mode::Check => {
            let sources = Sources {
                ledger: Source::new(&args.repo, &args.bd),
                design: Design::new(&args.repo, &args.folio),
                runs: Runs::new(args.state_dir.as_deref()),
            };
            let (texts, others) = thread::scope(|s| {
                let others = s.spawn(|| outside_of(&args.projects, &args.bd));
                let texts = sources.gather(true, true);
                (texts, others.join().unwrap_or_default())
            });
            let (mut g, floors) = board::built_floors(&texts);
            g.outside = others;
            (
                board::doc(&g, &graph::check(&g), &texts.summary),
                graph::check::outside_rulings(&g),
                graph::check::outside_heads(&g),
                floors,
            )
        }
        Mode::Design => {
            let (doc, floors) = design_doc(&args);
            (doc, None, None, floors)
        }
    };
    if !doc.unread.is_empty() {
        let names: Vec<&str> = doc.unread.iter().map(|s| source_name(*s)).collect();
        emit_err(&format!("# 読めない出所: {}", names.join("・")));
    }
    match args.mode {
        Mode::Check => check(&doc.invariants, outside, heads, &floors),
        Mode::Doc | Mode::Design => match wire::encode(&doc) {
            Ok(text) => {
                emit(&text);
                if doc.unread.is_empty() { 0 } else { UNKNOWN }
            }
            Err(e) => {
                emit_err(&format!("tz graph: 電文にできない: {e}"));
                UNKNOWN
            }
        },
    }
}

/// --design の読みの電文と床（設計の字だけを読み、床は全部まだ分からない）。
fn design_doc(args: &Args) -> (GraphDoc, Floors) {
    let design = Design::new(&args.repo, &args.folio);
    let texts = Texts {
        design: design.text().unwrap_or_default(),
        summary: design.summary().unwrap_or_default(),
        ..Texts::default()
    };
    let floors = Floors {
        bare: Reading::Unknown,
        unfielded: Reading::Unknown,
        landed: Reading::Unknown,
        unlanded: Reading::Unknown,
    };
    (design_view(&board::graph(&texts)), floors)
}

/// --project の置き場ごとに読み取りの bd を 1 本ずつ並べて撃ち、外の台帳の読みを置き場の順に返す
/// （読めない置き場は None・1 度だけ撃つ口なので持ち回さない）。
fn outside_of(projects: &[PathBuf], bd: &str) -> Vec<Option<Outside>> {
    thread::scope(|s| {
        let reads: Vec<_> = projects
            .iter()
            .map(|dir| s.spawn(move || Source::new(dir, bd).text()))
            .collect();
        reads
            .into_iter()
            .map(|h| h.join().ok().flatten().as_deref().and_then(build::outside))
            .collect()
    })
}

/// 違反の行と要約の行を標準出力、まだ分からないの行を標準エラーに出し、3 値の終了 code を返す。
/// `outside` は g-7 が文法の外の id の裁定だけのためにまだ分からないときのその数（`check::outside_rulings`）。
/// `heads` は g-3 が確かめられない外の台帳の行だけのためにまだ分からないときの節点の数と族（`check::outside_heads`）。
/// `floors` は要約の無い節点の列と本文だけで名指した id の対と全部の行が着地した設計ノートと着地の commit の無い
/// 着地した契約（`board::built_floors`・要約の行の前にこの順に出し、終了 code は変えない）。
fn check(
    invariants: &[InvariantCheck],
    outside: Option<usize>,
    heads: Option<(usize, Vec<String>)>,
    floors: &Floors,
) -> u8 {
    let (mut violations, mut unknowns) = (0, 0);
    for inv in invariants {
        match inv.verdict {
            Verdict::Violation => {
                violations += 1;
                let next = NEXT
                    .iter()
                    .find(|(id, _)| *id == inv.id)
                    .map_or("", |(_, next)| next);
                emit(&format!(
                    "[{}] 違反 {}（{}） next={next}",
                    inv.id,
                    inv.violations,
                    inv.ids.join("・")
                ));
            }
            Verdict::Unknown => {
                unknowns += 1;
                let why = match (&heads, outside) {
                    _ if UNMEASURED.contains(&inv.id.as_str()) => "測る機構がまだ無い".to_string(),
                    (Some((n, families)), _) if inv.id == "g-3" => {
                        format!("{OUTSIDE_HEAD} {n}（族 {}）", families.join("・"))
                    }
                    (_, Some(n)) if inv.id == "g-7" => {
                        format!("folio の裁定 id の文法の外の id の裁定 {n}")
                    }
                    _ => "読めない出所が在る".to_string(),
                };
                emit_err(&format!("# まだ分からない: [{}] {why}", inv.id));
            }
            Verdict::Pass => {}
        }
    }
    match &floors.bare {
        Reading::Known(ids) => emit(&bare_line(ids)),
        Reading::Unknown => emit_err(&format!("# まだ分からない: {BARE_UNKNOWN}")),
    }
    match &floors.unfielded {
        Reading::Known(pairs) => emit(&unfielded_line(pairs)),
        Reading::Unknown => emit_err(&format!("# まだ分からない: {UNFIELDED_UNKNOWN}")),
    }
    match &floors.landed {
        Reading::Known(docs) => emit(&landed_line(docs)),
        Reading::Unknown => emit_err(&format!("# まだ分からない: {LANDED_UNKNOWN}")),
    }
    match &floors.unlanded {
        Reading::Known(ids) => emit(&unlanded_line(ids)),
        Reading::Unknown => emit_err(&format!("# まだ分からない: {UNLANDED_UNKNOWN}")),
    }
    let verdict = overall(invariants);
    let word = match verdict {
        Verdict::Pass => "合格",
        Verdict::Violation => "不合格",
        Verdict::Unknown => "まだ分からない",
    };
    emit(&format!(
        "tz graph --check: {word}（違反 {violations}・まだ分からない {unknowns}）"
    ));
    exit_code(verdict)
}

fn source_name(s: GraphSource) -> &'static str {
    match s {
        GraphSource::Design => "設計の索引",
        GraphSource::Ledger => "台帳",
        GraphSource::Runs => "走行の記録",
    }
}

fn usage(what: &str) -> u8 {
    emit_err(&format!("tz graph: {what}\n{USAGE}"));
    FAIL
}

/// 値を取らない --check・--design と、`--名 値` か `--名=値` の --repo・--bd・--folio・--state-dir・
/// --project（--project だけは何度でも）を読む。
fn parse(rest: &[&str]) -> Result<Args, String> {
    let (mut check, mut design) = (false, false);
    let (mut repo, mut bd, mut folio, mut state_dir) = (None, None, None, None);
    let mut projects: Vec<PathBuf> = Vec::new();
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let flag = match *arg {
            "--check" => Some(&mut check),
            "--design" => Some(&mut design),
            _ => None,
        };
        if let Some(flag) = flag {
            if std::mem::replace(flag, true) {
                return Err(format!("{arg} が 2 度ある"));
            }
            continue;
        }
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (*arg, None),
        };
        let slot = match name {
            "--repo" => Some(&mut repo),
            "--bd" => Some(&mut bd),
            "--folio" => Some(&mut folio),
            "--state-dir" => Some(&mut state_dir),
            "--project" => None,
            _ => return Err(format!("知らない引数 {arg}")),
        };
        let value = match value {
            Some(v) => v,
            None => *it.next().ok_or_else(|| format!("{name} の値が無い"))?,
        };
        if value.is_empty() {
            return Err(format!("{name} の値が空"));
        }
        match slot {
            Some(slot) => {
                if slot.replace(value).is_some() {
                    return Err(format!("{name} が 2 度ある"));
                }
            }
            None => projects.push(PathBuf::from(value)),
        }
    }
    let mode = match (check, design) {
        (true, true) => return Err("--check と --design は 1 つだけ".into()),
        (true, false) => Mode::Check,
        (false, true) => Mode::Design,
        (false, false) => Mode::Doc,
    };
    Ok(Args {
        mode,
        repo: PathBuf::from(repo.unwrap_or(".")),
        bd: bd.unwrap_or(BD).to_string(),
        folio: super::folio::program(folio),
        state_dir: state_dir.map(|d| Path::new(d).to_path_buf()),
        projects,
    })
}
