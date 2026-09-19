//! `folio ceiling --check`（便 39・docs/design/delivery-39.md §1 (a)〜(c)・FR18 / AC16・ADR-8 決定 (3)(4)）。
//! 席か器が観点の束の dir へ書いた所見 file（`findings.yaml`）を天井の正本の欄の決まりで数え、観点ごとに 3 値を返す。
//! 数えるのは形・実在・一致だけ（所見の外形と欄の決まり・根拠の逐語の実在・束の要約値の一致 3 方向・止める の反証の有無）で、
//! 所見の中身が正しいかは判定しない（P-1）。所見 file の名と外形（最上位の欄 3 つ）は床の定数で持ち、欄ごとの値域は
//! 天井の正本（`bundle::load` が読む）から取る（P-5.1）。folio は所見 file を書かない。
//! 終了 = 4 観点が全部 合格 のときだけ 0、まだ分からない が 1 つでも在れば 2、無くて 不合格 が在れば 1（P-4）。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::bundle::{self, CONTENTS, Ceiling, DIGEST_FILE, Files, Rules, Viewpoint};
use crate::face::R;
use crate::verdict::Verdict;
use crate::yaml::{self, Node};

/// 所見 file の名（床の定数・置き場は `<out>/<観点の id>/`）。
pub const FINDINGS_FILE: &str = "findings.yaml";

/// 所見 file の最上位の欄（床の定数・順は問わない・他の欄は違反）。
pub const FINDINGS_TOP_LEVEL: [&str; 3] = ["verdict", "record", "findings"];

/// 1 回の実行の結果。`stdout` / `stderr` は 1 行ずつ。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

/// 観点 1 つの数えの結果。
struct Counted {
    verdict: Verdict,
    reasons: Vec<String>,
    /// 所見の数
    findings: usize,
    /// 残る 止める の数（反証で退けたものは数えない）
    stops: usize,
}

// ── 命令の口 ──

/// `--faces` と `--out` は相対なら `--dir` からの相対・絶対ならそのまま（`--write` と同じ読み）。
pub fn run(dir: &Path, faces: &Path, out: &Path) -> Outcome {
    let faces_dir = dir.join(faces);
    let out_dir = dir.join(out);
    let ceiling = match bundle::load(dir) {
        Ok(c) => c,
        Err(e) => return before_viewpoints(e),
    };
    // 観点に入る前の「まだ分からない」（--write と同じ文言）
    if !faces_dir.is_dir() {
        return before_viewpoints(format!("{}: 配信先が無い", faces_dir.display()));
    }
    let face_files = match bundle::read_dir_names(&faces_dir) {
        Ok(f) => f,
        Err(e) => return before_viewpoints(e),
    };
    if !out_dir.is_dir() && !out_dir.parent().is_some_and(Path::is_dir) {
        return before_viewpoints(format!("{}: 置き場の親 dir が無い", out_dir.display()));
    }

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let (mut pass, mut fail, mut unknown) = (0, 0, 0);
    for vp in &ceiling.viewpoints {
        let counted = count_viewpoint(dir, &faces_dir, &face_files, &out_dir, &ceiling, vp);
        match counted.verdict {
            Verdict::Pass => pass += 1,
            Verdict::Fail => fail += 1,
            Verdict::Unknown => unknown += 1,
        }
        stdout.push(format!(
            "{}: {}（所見 {}・止める {}）",
            vp.id, counted.verdict, counted.findings, counted.stops
        ));
        for reason in counted.reasons {
            stderr.push(format!("folio ceiling: {}: {reason}", vp.id));
        }
    }
    let verdict = if unknown > 0 {
        Verdict::Unknown
    } else if fail > 0 {
        Verdict::Fail
    } else {
        Verdict::Pass
    };
    stdout.push(format!(
        "folio ceiling: {verdict}（観点 {}・合格 {pass}・不合格 {fail}・まだ分からない {unknown}）",
        ceiling.viewpoints.len()
    ));
    Outcome {
        verdict,
        stdout,
        stderr,
    }
}

fn before_viewpoints(reason: String) -> Outcome {
    let write = bundle::Outcome::unknown(reason);
    Outcome {
        verdict: write.verdict,
        stdout: Vec::new(),
        stderr: write.stderr.into_iter().collect(),
    }
}

// ── 観点ごとの 3 値（§1 (c)）──

fn count_viewpoint(
    dir: &Path,
    faces_dir: &Path,
    face_files: &[(String, bool)],
    out_dir: &Path,
    ceiling: &Ceiling,
    vp: &Viewpoint,
) -> Counted {
    let vp_dir = out_dir.join(&vp.id);
    let mut reasons = Vec::new();

    // 1. 束が無い
    if !bundle_present(&vp_dir) {
        return Counted {
            verdict: Verdict::Unknown,
            reasons: vec!["束が無い".to_string()],
            findings: 0,
            stops: 0,
        };
    }
    let digest = fs::read(vp_dir.join(DIGEST_FILE)).unwrap_or_default();
    let digest = String::from_utf8_lossy(&digest).into_owned();

    // 2. 束が壊れている（置き場の file から測り直す）
    match measure(&vp_dir) {
        Ok(files) if bundle::digest_text(&files) == digest => {}
        Ok(_) => reasons.push("束が壊れている（要約値が digest.txt と違う）".to_string()),
        Err(e) => reasons.push(e),
    }

    // 3. 束が古い（現在の正本から memory の上に組み直す）
    match bundle::build_one(dir, faces_dir, face_files, ceiling, vp) {
        Ok(files)
            if files
                .get(DIGEST_FILE)
                .is_some_and(|d| *d == digest.as_bytes()) => {}
        Ok(_) => reasons.push("束が古い（現在の正本から組んだ要約値と違う）".to_string()),
        Err(e) => reasons.push(e),
    }

    // 4.〜6. 所見 file
    let (sheet, findings, stops) = match read_findings(&vp_dir) {
        Ok(Some(root)) => {
            let mut sheet = count_sheet(
                &vp_dir,
                &root,
                &ceiling.rules,
                vp,
                digest.trim_end_matches('\n'),
            );
            reasons.append(&mut sheet.reasons);
            (
                Some(sheet),
                sheet_findings(&root),
                sheet_stops(&root, &ceiling.rules),
            )
        }
        Ok(None) => {
            reasons.push("所見 file が無い".to_string());
            (None, 0, 0)
        }
        Err(e) => {
            reasons.push(format!("所見 file: {e}"));
            (None, 0, 0)
        }
    };
    if !reasons.is_empty() {
        return Counted {
            verdict: Verdict::Unknown,
            reasons,
            findings,
            stops,
        };
    }
    let sheet = sheet.expect("理由が無ければ所見 file は読めている");

    // 7. 止める の反証
    for id in &sheet.unrefuted {
        reasons.push(format!("反証が未（{id}）"));
    }
    if !reasons.is_empty() {
        return Counted {
            verdict: Verdict::Unknown,
            reasons,
            findings,
            stops,
        };
    }
    // 8.〜10. file の verdict と残る所見
    let verdict = match sheet.verdict.as_str() {
        "不合格" if findings == 0 => {
            reasons.push("不合格なのに所見が無い".to_string());
            Verdict::Unknown
        }
        "不合格" => Verdict::Fail,
        "合格" if sheet.remaining_stops.is_empty() => Verdict::Pass,
        "合格" => {
            for id in &sheet.remaining_stops {
                reasons.push(format!("止める所見が残っている（{id}）"));
            }
            Verdict::Fail
        }
        _ => {
            reasons.push("AI が判定できなかった".to_string());
            Verdict::Unknown
        }
    };
    Counted {
        verdict,
        reasons,
        findings,
        stops,
    }
}

/// 観点の dir と 5 つの中身と digest.txt が全部在るか。
fn bundle_present(vp_dir: &Path) -> bool {
    vp_dir.is_dir()
        && vp_dir.join("sources").is_dir()
        && vp_dir.join("faces").is_dir()
        && ["question.yaml", "finding.yaml", "reads.yaml", DIGEST_FILE]
            .iter()
            .all(|f| vp_dir.join(f).is_file())
}

/// 置き場の束を --write と同じ規則で読み直す（sources/ と faces/ の下の全 file と 3 つの yaml・digest.txt と所見 file は数えない）。
fn measure(vp_dir: &Path) -> R<Files> {
    let mut files = Files::new();
    for name in CONTENTS {
        let path = vp_dir.join(name);
        if path.is_dir() {
            walk(&path, name, &mut files)?;
        } else {
            let rel = format!("{name}.yaml");
            let bytes = fs::read(vp_dir.join(&rel)).map_err(|e| format!("{rel}: 読めない: {e}"))?;
            files.insert(rel, bytes);
        }
    }
    Ok(files)
}

fn walk(dir: &Path, rel: &str, files: &mut Files) -> R<()> {
    for (name, is_file) in bundle::read_dir_names(dir)? {
        let here = format!("{rel}/{name}");
        let path = dir.join(&name);
        if is_file {
            let bytes = fs::read(&path).map_err(|e| format!("{here}: 読めない: {e}"))?;
            files.insert(here, bytes);
        } else if path.is_dir() {
            walk(&path, &here, files)?;
        }
    }
    Ok(())
}

// ── 所見 file を読む（§1 (b)）──

/// `<vp_dir>/findings.yaml`。無い = Ok(None)・読めない / parse できない / 重複キー / 最上位が表でない = Err（file 名: 理由）。
fn read_findings(vp_dir: &Path) -> R<Option<Node>> {
    let path = vp_dir.join(FINDINGS_FILE);
    if path.is_symlink() {
        return Err(format!("{FINDINGS_FILE}: symlink は認めない"));
    }
    if !path.exists() {
        return Ok(None);
    }
    if !path.is_file() {
        return Err(format!("{FINDINGS_FILE}: file でない"));
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("{FINDINGS_FILE}: 読めない: {e}"))?;
    let doc = yaml::parse(&text).map_err(|e| format!("{FINDINGS_FILE}: parse できない: {e}"))?;
    if let Some(d) = doc.duplicates.first() {
        return Err(format!(
            "{FINDINGS_FILE}: {} 行: 同じ表に同じキー「{}」が 2 度ある",
            d.line, d.key
        ));
    }
    if doc.root.as_map().is_none() {
        return Err(format!("{FINDINGS_FILE}: 最上位が欄の表でない"));
    }
    Ok(Some(doc.root))
}

/// 所見 file の欄の決まりを数えた結果。
struct Sheet {
    /// 欄の決まりの違反（5）と要約値の不一致（6）
    reasons: Vec<String>,
    /// file の verdict（値域の外は空）
    verdict: String,
    /// 止める なのに refute が無い所見の id
    unrefuted: Vec<String>,
    /// 残る 止める の所見の id（refute が 退けた でないもの）
    remaining_stops: Vec<String>,
}

fn count_sheet(vp_dir: &Path, root: &Node, rules: &Rules, vp: &Viewpoint, digest: &str) -> Sheet {
    let mut sheet = Sheet {
        reasons: Vec::new(),
        verdict: String::new(),
        unrefuted: Vec::new(),
        remaining_stops: Vec::new(),
    };
    let entries = root.as_map().expect("最上位は表と読んである");
    for (key, _) in entries {
        if !FINDINGS_TOP_LEVEL.contains(&key.as_str()) {
            sheet.reasons.push(format!("未知の欄「{key}」"));
        }
    }
    for key in FINDINGS_TOP_LEVEL {
        if root.get(key).is_none() {
            sheet.reasons.push(format!("{key} が無い"));
        }
    }

    // verdict = 3 値のどれか
    if let Some(node) = root.get("verdict") {
        match node.as_str() {
            Some(v) if rules.verdict_values.iter().any(|w| w == v) => sheet.verdict = v.to_string(),
            Some(v) => sheet.reasons.push(format!("verdict「{v}」が値域の外")),
            None => sheet.reasons.push("verdict が文でない".to_string()),
        }
    }

    // 観点の reads.yaml に在る doc
    let reads = match read_docs(vp_dir) {
        Ok(docs) => Some(docs),
        Err(e) => {
            sheet.reasons.push(e);
            None
        }
    };

    // record = 起動の記録
    if let Some(record) = root.get("record") {
        count_record(record, rules, reads.as_ref(), digest, &mut sheet.reasons);
    }

    // findings = 所見の一覧
    if let Some(findings) = root.get("findings") {
        count_findings(vp_dir, findings, rules, vp, reads.as_ref(), &mut sheet);
    }
    sheet
}

fn count_record(
    record: &Node,
    rules: &Rules,
    reads: Option<&BTreeSet<String>>,
    digest: &str,
    reasons: &mut Vec<String>,
) {
    let Some(entries) = record.as_map() else {
        reasons.push("起動の記録（record）が欄の表でない".to_string());
        return;
    };
    for (key, _) in entries {
        if !rules.record_required.iter().any(|k| k == key) {
            reasons.push(format!("起動の記録（record）に未知の欄「{key}」"));
        }
    }
    for key in &rules.record_required {
        let Some(node) = record.get(key) else {
            reasons.push(format!("起動の記録（record）に {key} が無い"));
            continue;
        };
        match key.as_str() {
            "read" => count_read(node, reads, reasons),
            "bundle" => match node.as_str() {
                Some(s) if s.trim().is_empty() => {
                    reasons.push("起動の記録（record）の bundle が空".to_string());
                }
                Some(s) if s != digest => {
                    reasons.push("要約値が束と合わない（読んだ束が違う）".to_string());
                }
                Some(_) => {}
                None => reasons.push("起動の記録（record）の bundle が文でない".to_string()),
            },
            _ => match node.as_str() {
                Some(s) if s.trim().is_empty() => {
                    reasons.push(format!("起動の記録（record）の {key} が空"));
                }
                Some(_) => {}
                None => reasons.push(format!("起動の記録（record）の {key} が文でない")),
            },
        }
    }
}

/// record.read = 読んだ文書の id の一覧（空でない・reads.yaml の doc と集合として一致）。
fn count_read(node: &Node, reads: Option<&BTreeSet<String>>, reasons: &mut Vec<String>) {
    let Some(items) = node.as_seq() else {
        reasons.push("起動の記録（record）の read が一覧でない".to_string());
        return;
    };
    if items.is_empty() {
        reasons.push("起動の記録（record）の read が空".to_string());
        return;
    }
    let mut read = BTreeSet::new();
    for item in items {
        match item.as_str() {
            Some(s) if !s.trim().is_empty() => {
                read.insert(s.to_string());
            }
            _ => reasons.push("起動の記録（record）の read に文でない要素がある".to_string()),
        }
    }
    let Some(reads) = reads else {
        return;
    };
    for doc in &read {
        if !reads.contains(doc) {
            reasons.push(format!(
                "起動の記録（record）の read「{doc}」が reads.yaml に無い"
            ));
        }
    }
    for doc in reads {
        if !read.contains(doc) {
            reasons.push(format!(
                "起動の記録（record）の read に {doc} が無い（reads.yaml の doc は全部読む）"
            ));
        }
    }
}

fn count_findings(
    vp_dir: &Path,
    findings: &Node,
    rules: &Rules,
    vp: &Viewpoint,
    reads: Option<&BTreeSet<String>>,
    sheet: &mut Sheet,
) {
    let Some(items) = findings.as_seq() else {
        sheet.reasons.push("findings が一覧でない".to_string());
        return;
    };
    let mut sources: Option<Vec<Vec<u8>>> = None;
    let mut ids: Vec<String> = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let Some(entries) = item.as_map() else {
            sheet
                .reasons
                .push(format!("findings[{i}]: 所見の行が欄の表でない"));
            continue;
        };
        let id = item
            .get("id")
            .and_then(Node::as_str)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("?")
            .to_string();
        let at = format!("所見 {id}");
        if id != "?" {
            if ids.contains(&id) {
                sheet.reasons.push(format!("{at}: id が重複"));
            }
            ids.push(id.clone());
        }
        for (key, _) in entries {
            if !rules.finding_required.iter().any(|k| k == key)
                && !rules.finding_optional.iter().any(|k| k == key)
            {
                sheet.reasons.push(format!("{at}: 未知の欄「{key}」"));
            }
        }
        for key in &rules.finding_required {
            let Some(node) = item.get(key) else {
                sheet.reasons.push(format!("{at}: {key} が無い"));
                continue;
            };
            if key == "place" {
                count_place(&at, node, rules, reads, &mut sheet.reasons);
                continue;
            }
            let Some(value) = node.as_str() else {
                sheet.reasons.push(format!("{at}: {key} が文でない"));
                continue;
            };
            if value.trim().is_empty() {
                sheet.reasons.push(format!("{at}: {key} が空"));
                continue;
            }
            match key.as_str() {
                "viewpoint" if value != vp.id => sheet
                    .reasons
                    .push(format!("{at}: viewpoint「{value}」が観点 {} と違う", vp.id)),
                "weight" if !rules.weight_values.iter().any(|w| w == value) => {
                    sheet
                        .reasons
                        .push(format!("{at}: weight「{value}」が値域の外"));
                }
                "evidence" => {
                    let files = sources.get_or_insert_with(|| read_sources(vp_dir));
                    if !verbatim(files, value.as_bytes()) {
                        sheet.reasons.push(format!(
                            "{at}: 根拠が正本に無い（sources/ の下に逐語が無い）"
                        ));
                    }
                }
                _ => {}
            }
        }
        // refute は在れば値域のどれか・止める の所見は反証を通す
        let refute = item.get("refute").map(|node| match node.as_str() {
            Some(v) if rules.refute_values.iter().any(|w| w == v) => Some(v.to_string()),
            Some(v) => {
                sheet.reasons.push(format!("{at}: refute「{v}」が値域の外"));
                None
            }
            None => {
                sheet.reasons.push(format!("{at}: refute が文でない"));
                None
            }
        });
        let weight = item
            .get("weight")
            .and_then(Node::as_str)
            .unwrap_or_default();
        if rules.weight_refute.iter().any(|w| w == weight) {
            match refute {
                None => sheet.unrefuted.push(id.clone()),
                Some(Some(v)) if v == "退けた" => {}
                Some(_) => sheet.remaining_stops.push(id.clone()),
            }
        }
    }
}

/// place = {doc, at}。doc はその観点の reads.yaml に在る doc・at は空でない文。
fn count_place(
    at: &str,
    place: &Node,
    rules: &Rules,
    reads: Option<&BTreeSet<String>>,
    reasons: &mut Vec<String>,
) {
    if place.as_map().is_none() {
        reasons.push(format!("{at}: place が欄の表でない"));
        return;
    }
    for key in &rules.place_required {
        let value = match place.get(key).map(Node::as_str) {
            None => {
                reasons.push(format!("{at}: place.{key} が無い"));
                continue;
            }
            Some(None) => {
                reasons.push(format!("{at}: place.{key} が文でない"));
                continue;
            }
            Some(Some(v)) if v.trim().is_empty() => {
                reasons.push(format!("{at}: place.{key} が空"));
                continue;
            }
            Some(Some(v)) => v,
        };
        if key == "doc" && reads.is_some_and(|r| !r.contains(value)) {
            reasons.push(format!(
                "{at}: place.doc「{value}」が reads.yaml に無い（読んでいない文書）"
            ));
        }
    }
}

/// その観点の reads.yaml の doc の集合。
fn read_docs(vp_dir: &Path) -> R<BTreeSet<String>> {
    let text = fs::read_to_string(vp_dir.join("reads.yaml"))
        .map_err(|e| format!("reads.yaml: 読めない: {e}"))?;
    let doc = yaml::parse(&text).map_err(|e| format!("reads.yaml: parse できない: {e}"))?;
    let rows = doc
        .root
        .as_seq()
        .ok_or_else(|| "reads.yaml: 行の一覧でない".to_string())?;
    let mut docs = BTreeSet::new();
    for row in rows {
        match row.get("doc").and_then(Node::as_str) {
            Some(d) => {
                docs.insert(d.to_string());
            }
            None => return Err("reads.yaml: doc が読めない".to_string()),
        }
    }
    Ok(docs)
}

/// 観点の束の sources/ の下の全 file の中身（読めない file は数えない）。
fn read_sources(vp_dir: &Path) -> Vec<Vec<u8>> {
    let mut files = Files::new();
    let _ = walk(&vp_dir.join("sources"), "sources", &mut files);
    files.into_values().collect()
}

/// `needle` がどれかの file の 1 行の中に byte 列として在るか（改行を跨がない）。
fn verbatim(files: &[Vec<u8>], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.contains(&b'\n') {
        return false;
    }
    files.iter().any(|body| {
        body.split(|b| *b == b'\n')
            .any(|line| line.windows(needle.len()).any(|w| w == needle))
    })
}

/// 標準出力に出す所見の数（一覧でなければ 0）。
fn sheet_findings(root: &Node) -> usize {
    root.get("findings")
        .and_then(Node::as_seq)
        .map_or(0, <[Node]>::len)
}

/// 標準出力に出す 残る 止める の数（weight が weights.refute のどれかで refute が 退けた でない所見）。
fn sheet_stops(root: &Node, rules: &Rules) -> usize {
    root.get("findings")
        .and_then(Node::as_seq)
        .map_or(0, |items| {
            items
                .iter()
                .filter(|item| {
                    let weight = item
                        .get("weight")
                        .and_then(Node::as_str)
                        .unwrap_or_default();
                    rules.weight_refute.iter().any(|w| w == weight)
                        && item.get("refute").and_then(Node::as_str) != Some("退けた")
                })
                .count()
        })
}

#[cfg(test)]
mod findings_tests {
    use super::*;

    #[test]
    fn findings_verbatim_matches_within_a_single_line() {
        let files = vec![b"a: 1\nplain: xyz\n".to_vec()];
        assert!(verbatim(&files, b"plain: xyz"));
        assert!(verbatim(&files, b"xyz"));
        assert!(!verbatim(&files, b"1\nplain"));
        assert!(!verbatim(&files, b""));
        assert!(!verbatim(&files, b"abc"));
    }
}
