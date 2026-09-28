//! tz graph（行 k-graph・要件 FR2・FR3・FR17）: 導出グラフを組み直して出す。書かない。
//! - 旗なし — GraphDoc の電文を標準出力に 1 行
//! - --check — 不変条件の 12 本を 3 値で数え、違反の行と要約の行を標準出力、まだ分からないの行を標準エラー
//! - --design — 設計の索引だけを読み、設計の節点と辺に絞った GraphDoc の電文（folio の graph の吸収・ADR-8 決定 (3)）
//!
//! 終了 code は folio の床の check の口に揃える（合格 0・不合格 1・まだ分からない 2）。旗なしと --design は
//! 読めない出所が無ければ 0、在れば 2。使い方の誤りは 1。組みは口 /api/graph と同じ `Sources::gather` と
//! `board::graph` の 2 つの呼び。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tsuzuri_contract::graph::{
    EdgeType, GraphDoc, GraphSource, InvariantCheck, SkippedEdges, Verdict,
};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::{self, build::DESIGN_EDGE_TYPES, check::UNMEASURED};

use crate::server::board::{self, Sources, Texts};
use crate::server::design::{Design, FOLIO};
use crate::server::ledger::{BD, Source};
use crate::server::runs::Runs;

pub const USAGE: &str = "usage: tz graph [--check | --design] [--repo <dir>] [--bd <program>] [--folio <program>] [--state-dir <dir>]";

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
    folio: String,
    state_dir: Option<PathBuf>,
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
    let doc = match args.mode {
        Mode::Doc | Mode::Check => {
            let sources = Sources {
                ledger: Source::new(&args.repo, &args.bd),
                design: Design::new(&args.repo, &args.folio),
                runs: Runs::new(args.state_dir.as_deref()),
            };
            board::graph(&sources.gather(true, true))
        }
        Mode::Design => {
            let design = Design::new(&args.repo, &args.folio);
            let texts = Texts {
                design: design.text().unwrap_or_default(),
                summary: design.summary().unwrap_or_default(),
                ..Texts::default()
            };
            design_view(&board::graph(&texts))
        }
    };
    if !doc.unread.is_empty() {
        let names: Vec<&str> = doc.unread.iter().map(|s| source_name(*s)).collect();
        eprintln!("# 読めない出所: {}", names.join("・"));
    }
    match args.mode {
        Mode::Check => check(&doc.invariants),
        Mode::Doc | Mode::Design => match wire::encode(&doc) {
            Ok(text) => {
                println!("{text}");
                if doc.unread.is_empty() { 0 } else { UNKNOWN }
            }
            Err(e) => {
                eprintln!("tz graph: 電文にできない: {e}");
                UNKNOWN
            }
        },
    }
}

/// 違反の行と要約の行を標準出力、まだ分からないの行を標準エラーに出し、3 値の終了 code を返す。
fn check(invariants: &[InvariantCheck]) -> u8 {
    let (mut violations, mut unknowns) = (0, 0);
    for inv in invariants {
        match inv.verdict {
            Verdict::Violation => {
                violations += 1;
                let next = NEXT
                    .iter()
                    .find(|(id, _)| *id == inv.id)
                    .map_or("", |(_, next)| next);
                println!(
                    "[{}] 違反 {}（{}） next={next}",
                    inv.id,
                    inv.violations,
                    inv.ids.join("・")
                );
            }
            Verdict::Unknown => {
                unknowns += 1;
                let why = if UNMEASURED.contains(&inv.id.as_str()) {
                    "測る機構がまだ無い"
                } else {
                    "読めない出所が在る"
                };
                eprintln!("# まだ分からない: [{}] {why}", inv.id);
            }
            Verdict::Pass => {}
        }
    }
    let verdict = overall(invariants);
    let word = match verdict {
        Verdict::Pass => "合格",
        Verdict::Violation => "不合格",
        Verdict::Unknown => "まだ分からない",
    };
    println!("tz graph --check: {word}（違反 {violations}・まだ分からない {unknowns}）");
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
    eprintln!("tz graph: {what}\n{USAGE}");
    FAIL
}

/// 値を取らない --check・--design と、`--名 値` か `--名=値` の --repo・--bd・--folio・--state-dir を読む。
fn parse(rest: &[&str]) -> Result<Args, String> {
    let (mut check, mut design) = (false, false);
    let (mut repo, mut bd, mut folio, mut state_dir) = (None, None, None, None);
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
            "--repo" => &mut repo,
            "--bd" => &mut bd,
            "--folio" => &mut folio,
            "--state-dir" => &mut state_dir,
            _ => return Err(format!("知らない引数 {arg}")),
        };
        let value = match value {
            Some(v) => v,
            None => *it.next().ok_or_else(|| format!("{name} の値が無い"))?,
        };
        if value.is_empty() {
            return Err(format!("{name} の値が空"));
        }
        if slot.replace(value).is_some() {
            return Err(format!("{name} が 2 度ある"));
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
        folio: folio.unwrap_or(FOLIO).to_string(),
        state_dir: state_dir.map(|d| Path::new(d).to_path_buf()),
    })
}
