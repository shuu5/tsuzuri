//! `folio ceiling --stamp`（便 72・docs/design/delivery-72.md §1 (a)〜(c)・FR20 / AC18・設計ノート ceiling-gate.md §6 の 2）。
//! 周の結果（`--write` が組んだ観点ごとの束・席か器が書いた所見 file・止める の反証の結果 result.yaml）から天井の印を
//! 決定的に導出し `<dir>/preview/ceiling-stamp.yaml` へ書く。観点ごとの 3 値は `--check` と同じ口
//! （`findings::count_viewpoint`・配信先を取らないので規則 3〔束が古い〕は当てない＝名札と同じ）で数え、二重に実装しない。
//! 印の欄は閉じた一覧でこの順: round・at・verdict・sources・faces・viewpoints・refutes・reads・rest・nodes
//! （rest と nodes は便 99: 残差の要約値と節点ごとの要約値の表・組み方は `graph::stamp_table`）。
//! 決定性: 時刻・絶対 path・環境の値を書かない（round は置き場の dir の名・at は所見 file の起動の記録から取る）。
//! 欄 sources は束の写しでなく `--dir` の正本から、門と同じ関数（`gate.rs` の `sources_digest`）で測る（便 104・
//! docs/design/delivery-104.md §1 (b)・FR20 の 正本の要約値）。束の sources/ の写しは便 98 で観点ごとに絞られ、同じ文書でも
//! 観点で byte が違う。欄 faces は束の faces/ の和集合のまま（面は絞られていない・門は faces を突き合わせない）。
//! 全部か無しか: 観点のどれかの束・所見 file・起動の記録が読めない、または面の写しが観点で食い違うときは
//! まだ分からない（終了 2）で file を触らない。既に同じ byte なら書かない。判定の 3 値は印の中身で、命令は書けたら 0。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::bundle::{self, Ceiling, Files};
use crate::cursor::R;
use crate::findings::{self, Counted};
use crate::gate;
use crate::graph;
use crate::sha256;
use crate::verdict::Verdict;
use crate::yaml::{self, Node};

/// 印の置き場（`--dir` からの相対・床の定数）。
pub const STAMP_FILE: &str = "preview/ceiling-stamp.yaml";

/// 印の先頭の注釈（1 行）。
const HEADER: &str =
    "# folio2 天井の印 — 生成物（folio ceiling --stamp が書く・手で直さない・P-6.2）";

/// 1 回の実行の結果。`stdout` は 1 行。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: String,
}

impl Outcome {
    fn unknown(reason: impl Into<String>) -> Self {
        Outcome {
            verdict: Verdict::Unknown,
            stdout: format!("folio ceiling: まだ分からない（{}）", reason.into()),
        }
    }
}

/// 観点 1 つの行。
struct Row {
    id: String,
    counted: Counted,
    bundle: String,
    model: String,
    effort: String,
    at: String,
    read: Vec<String>,
}

// ── 命令の口 ──

/// `--out` は相対なら `--dir` からの相対・絶対ならそのまま（`--write` と同じ読み）。
pub fn run(dir: &Path, out: &Path) -> Outcome {
    let out_dir = dir.join(out);
    let text = match derive(dir, &out_dir) {
        Ok(t) => t,
        Err(e) => return Outcome::unknown(e),
    };
    let path = dir.join(STAMP_FILE);
    if path.is_symlink() {
        return Outcome::unknown(format!("{}: symlink は認めない", path.display()));
    }
    if fs::read(&path).is_ok_and(|old| old == text.as_bytes()) {
        return Outcome {
            verdict: Verdict::Pass,
            stdout: format!("folio ceiling: 印は同じ（{}）", path.display()),
        };
    }
    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        return Outcome::unknown(format!("{}: 作れない: {e}", parent.display()));
    }
    if let Err(e) = fs::write(&path, &text) {
        return Outcome::unknown(format!("{}: 書けない: {e}", path.display()));
    }
    Outcome {
        verdict: Verdict::Pass,
        stdout: format!(
            "folio ceiling: 印を書いた（{}・{} byte）",
            path.display(),
            text.len()
        ),
    }
}

// ── 印の導出（§1 (b)）──

/// 印の全文。組めなければ Err（理由 1 つ）。
fn derive(dir: &Path, out_dir: &Path) -> R<String> {
    let ceiling = bundle::load(dir)?;
    if !out_dir.is_dir() {
        return Err(format!("{}: 置き場が無い", out_dir.display()));
    }
    let round = out_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("{}: 置き場の名が読めない", out_dir.display()))?
        .to_string();
    let mut rows = Vec::with_capacity(ceiling.viewpoints.len());
    for vp in &ceiling.viewpoints {
        let counted = findings::count_viewpoint(dir, None, out_dir, &ceiling, vp);
        rows.push(
            row(&out_dir.join(&vp.id), &vp.id, counted).map_err(|e| format!("{}: {e}", vp.id))?,
        );
    }
    let sources = gate::sources_digest(dir, &ceiling)?;
    let faces = faces_digest(out_dir, &ceiling)?;

    let verdicts: Vec<Verdict> = rows.iter().map(|r| r.counted.verdict).collect();
    let verdict = if verdicts.contains(&Verdict::Unknown) {
        Verdict::Unknown
    } else if verdicts.contains(&Verdict::Fail) {
        Verdict::Fail
    } else {
        Verdict::Pass
    };
    let at = rows
        .iter()
        .map(|r| r.at.as_str())
        .max()
        .unwrap_or_default()
        .to_string();
    let reads: BTreeSet<&str> = rows
        .iter()
        .flat_map(|r| r.read.iter().map(String::as_str))
        .collect();

    let mut text = format!(
        "{HEADER}\nround: {}\nat: {}\nverdict: {verdict}\nsources: {sources}\nfaces: {faces}\nviewpoints:\n",
        plain(&round),
        plain(&at)
    );
    for r in &rows {
        text.push_str(&format!(
            "  - {{id: {}, verdict: {}, findings: {}, stops: {}, bundle: {}, model: {}, effort: {}, at: {}}}\n",
            plain(&r.id),
            r.counted.verdict,
            r.counted.findings,
            r.counted.stops,
            plain(&r.bundle),
            plain(&r.model),
            plain(&r.effort),
            plain(&r.at)
        ));
    }
    let mut refutes = Vec::new();
    for r in &rows {
        let mut own: Vec<&(String, String)> = r.counted.refutes.iter().collect();
        own.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        for (finding, refute) in own {
            refutes.push(format!(
                "  - {{viewpoint: {}, finding: {}, refute: {}}}\n",
                plain(&r.id),
                plain(finding),
                plain(refute)
            ));
        }
    }
    if refutes.is_empty() {
        text.push_str("refutes: []\n");
    } else {
        text.push_str("refutes:\n");
        text.push_str(&refutes.concat());
    }
    let reads: Vec<String> = reads.into_iter().map(plain).collect();
    text.push_str(&format!("reads: [{}]\n", reads.join(", ")));
    let (rest, nodes) = graph::stamp_table(dir, &ceiling)?;
    text.push_str(&format!("rest: {rest}\nnodes:\n"));
    for (id, digest) in nodes {
        text.push_str(&format!("  - {{id: {}, digest: {digest}}}\n", plain(&id)));
    }
    Ok(text)
}

/// 観点 1 つの行。束（要約値）・所見 file・起動の記録の model / effort / at / read が読めなければ Err。
fn row(vp_dir: &Path, id: &str, counted: Counted) -> R<Row> {
    let Some(bundle) = counted.digest.clone() else {
        return Err(counted
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "束の要約値が読めない".to_string()));
    };
    let root = match findings::read_findings(vp_dir) {
        Ok(Some(root)) => root,
        Ok(None) => return Err("所見 file が無い".to_string()),
        Err(e) => return Err(format!("所見 file: {e}")),
    };
    let record = root
        .get("record")
        .ok_or_else(|| "起動の記録（record）が無い".to_string())?;
    let text = |key: &str| {
        record
            .get(key)
            .and_then(Node::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
            .ok_or_else(|| format!("起動の記録（record）の {key} が読めない"))
    };
    let read = record
        .get("read")
        .and_then(Node::as_seq)
        .and_then(|items| {
            items
                .iter()
                .map(|n| n.as_str().map(str::to_string))
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| "起動の記録（record）の read が読めない".to_string())?;
    Ok(Row {
        id: id.to_string(),
        bundle,
        model: text("model")?,
        effort: text("effort")?,
        at: text("at")?,
        read,
        counted,
    })
}

/// 面の要約値 = `<out>/<観点>/faces/` の下の file を `faces/` からの相対 path で和集合にし（同じ相対 path が観点で違う
/// byte なら Err）、相対 path の byte 順に中身を区切りなしに連結した byte 列の sha256（「sha256 <16 進>」）。
fn faces_digest(out_dir: &Path, ceiling: &Ceiling) -> R<String> {
    let sub = "faces";
    let mut union = Files::new();
    for vp in &ceiling.viewpoints {
        let mut files = Files::new();
        findings::walk(&out_dir.join(&vp.id).join(sub), "", &mut files)
            .map_err(|e| format!("{}: {sub}: {e}", vp.id))?;
        for (rel, bytes) in files {
            let rel = rel.trim_start_matches('/').to_string();
            match union.get(&rel) {
                Some(have) if *have != bytes => {
                    return Err(format!("{sub}/{rel}: 観点で中身が違う（{}）", vp.id));
                }
                Some(_) => {}
                None => {
                    union.insert(rel, bytes);
                }
            }
        }
    }
    let bytes: Vec<u8> = union.into_values().flatten().collect();
    Ok(format!("sha256 {}", sha256::hex(&bytes)))
}

// ── 印の読み手（便 83・delivery-83.md §1 (a)）──

/// 名札のために印から読む観点 1 つの行（8 欄のうち 4 つ・どれも印の字のまま）。
pub struct Mark {
    pub id: String,
    pub verdict: String,
    /// 観点の at（名札の日付は top-level の at を使うので読むだけ・読めなければ Err）
    #[allow(dead_code)]
    pub at: String,
    pub bundle: String,
}

/// 面の天井の名札のために印を読む。返りは（top-level の at・観点の行を印の順に）。印の file が無ければ None。
/// symlink・読めない・parse できない・欄 at が読めない・viewpoints が一覧でない・行の欄が読めないは Err（P-4.1）。
pub fn marks(dir: &Path) -> R<Option<(String, Vec<Mark>)>> {
    let path = dir.join(STAMP_FILE);
    if path.is_symlink() {
        return Err(format!("{STAMP_FILE}: symlink は認めない"));
    }
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("{STAMP_FILE}: 読めない: {e}"))?;
    let root = yaml::parse(&text)
        .map_err(|e| format!("{STAMP_FILE}: parse できない: {e}"))?
        .root;
    let field = |node: &Node, key: &str, at: &str| {
        node.get(key)
            .and_then(Node::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
            .ok_or_else(|| format!("{STAMP_FILE}: {at}{key} が読めない"))
    };
    let at = field(&root, "at", "")?;
    let rows = root
        .get("viewpoints")
        .and_then(Node::as_seq)
        .ok_or_else(|| format!("{STAMP_FILE}: viewpoints が一覧でない"))?;
    let marks = rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let at = format!("viewpoints[{i}].");
            Ok(Mark {
                id: field(row, "id", &at)?,
                verdict: field(row, "verdict", &at)?,
                at: field(row, "at", &at)?,
                bundle: field(row, "bundle", &at)?,
            })
        })
        .collect::<R<Vec<_>>>()?;
    Ok(Some((at, marks)))
}

/// 流れの形（`{…}`・`[…]`）の中に素のまま置ける値はそのまま、置けない値は `"` で囲む。
fn plain(s: &str) -> String {
    let unsafe_char = |c: char| ",[]{}#\"'\n\t\\".contains(c);
    let unsafe_head = |c: char| "-?:&*!|>%@`".contains(c);
    let bare = !s.is_empty()
        && s.trim() == s
        && !s.contains(unsafe_char)
        && !s.contains(": ")
        && !s.ends_with(':')
        && !s.starts_with(unsafe_head);
    if bare {
        s.to_string()
    } else {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

#[cfg(test)]
mod stamp_tests {
    use super::*;

    #[test]
    fn stamp_plain_quotes_only_what_breaks_a_flow() {
        assert_eq!(plain("opus"), "opus");
        assert_eq!(plain("2026-09-19T05:00:00Z"), "2026-09-19T05:00:00Z");
        assert_eq!(plain("fable 5.1"), "fable 5.1");
        assert_eq!(plain("a, b"), "\"a, b\"");
        assert_eq!(plain("a: b"), "\"a: b\"");
        assert_eq!(plain(""), "\"\"");
        assert_eq!(plain("x\"y"), "\"x\\\"y\"");
    }
}
