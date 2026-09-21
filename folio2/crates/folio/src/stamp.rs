//! `folio ceiling --stamp`（便 72・docs/design/delivery-72.md §1 (a)〜(c)・FR20 / AC18・設計ノート ceiling-gate.md §6 の 2）。
//! 周の結果（`--write` が組んだ観点ごとの束・席か器が書いた所見 file・止める の反証の結果 result.yaml）から天井の印を
//! 決定的に導出し `<dir>/preview/ceiling-stamp.yaml` へ書く。観点ごとの 3 値は `--check` と同じ口
//! （`findings::count_viewpoint`・配信先を取らないので規則 3〔束が古い〕は当てない＝名札と同じ）で数え、二重に実装しない。
//! 印の欄は閉じた一覧でこの順: round・at・verdict・sources・faces・viewpoints・refutes・reads。
//! 決定性: 時刻・絶対 path・環境の値を書かない（round は置き場の dir の名・at は所見 file の起動の記録から取る）。
//! 全部か無しか: 観点のどれかの束・所見 file・起動の記録が読めない、または正本と面の写しが観点で食い違うときは
//! まだ分からない（終了 2）で file を触らない。既に同じ byte なら書かない。判定の 3 値は印の中身で、命令は書けたら 0。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::bundle::{self, Ceiling, Files};
use crate::face::R;
use crate::findings::{self, Counted};
use crate::sha256;
use crate::verdict::Verdict;
use crate::yaml::Node;

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
    let sources = union_digest(out_dir, &ceiling, "sources")?;
    let faces = union_digest(out_dir, &ceiling, "faces")?;

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

/// 要約値 = `<out>/<観点>/<sub>/` の下の file を `<sub>/` からの相対 path で和集合にし（同じ相対 path が観点で違う byte
/// なら Err）、相対 path の byte 順に中身を区切りなしに連結した byte 列の sha256（「sha256 <16 進>」）。
fn union_digest(out_dir: &Path, ceiling: &Ceiling, sub: &str) -> R<String> {
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
