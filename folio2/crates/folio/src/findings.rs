//! `folio ceiling --check`（便 39・docs/design/delivery-39.md §1 (a)〜(c)・FR18 / AC16・ADR-8 決定 (3)(4)）。
//! 席か器が観点の束の dir へ書いた所見 file（`findings.yaml`）を天井の正本の欄の決まりで数え、観点ごとに 3 値を返す。
//! 数えるのは形・実在・一致だけ（所見の外形と欄の決まり・根拠の逐語の実在・束の要約値の一致 3 方向・止める の反証の有無）で、
//! 所見の中身が正しいかは判定しない（P-1）。所見 file の名と外形（最上位の欄 3 つ）は床の定数で持ち、欄ごとの値域は
//! `bundle::load` の型 `Rules`（weights は天井の正本・残りは床の定数 `ceiling.rs`・便 47）から取る（P-5.1）。folio は所見 file を書かない。
//! 終了 = 4 観点が全部 合格 のときだけ 0、まだ分からない が 1 つでも在れば 2、無くて 不合格 が在れば 1（P-4）。
//! 名札（便 40・delivery-40.md §1 (b)・ADR-8 決定 (4)）: 面の生成器は `stamps` で観点ごとの 3 値を取る。同じ規則のうち
//! 3（束が古い）だけを当てない——名札を載せた面そのものが次の束の入力（faces/）になるので、面の生成の中で「現在の面から
//! 組んだ要約値」を求めると固定点が無い。束が古いかは `--check` が数える側の領分で、名札は代わりに要約値の先頭 8 字を出す。
//! 反証の束（便 42・delivery-42.md §1 (a)〜(d)・ADR-8 決定 (3)）: `--refute` は 止める の所見のうち反証が未のものごとに
//! 反証の材料の束（finding.yaml・question.yaml・reads.yaml・schema.yaml・sources.txt + digest.txt）を
//! `<out>/<観点>/refute/<所見の id>/` へ組む（所見 file は触らない・正本は写さず親の要約値 sources.txt で縛る・全部か無しか）。
//! `--check` は同じ dir の result.yaml（反証役が書く）を欄の決まりで読み、その refute の値を所見の反証の結果として規則 7〜10 に渡す。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::bundle::{self, Ceiling, DIGEST_FILE, Files, Rules, Viewpoint};
use crate::ceiling::{BUNDLE_CONTENTS, REFUTE_CONTENTS, REFUTE_RULE, RESULT_REQUIRED};
use crate::face::R;
use crate::verdict::Verdict;
use crate::yaml::{self, Node};

/// 所見 file の名（床の定数・置き場は `<out>/<観点の id>/`）。
pub const FINDINGS_FILE: &str = "findings.yaml";

/// 所見 file の最上位の欄（床の定数・順は問わない・他の欄は違反）。
pub const FINDINGS_TOP_LEVEL: [&str; 3] = ["verdict", "record", "findings"];

/// 反証の束の置き場（床の定数・`<out>/<観点の id>/refute/<所見の id>/`）。
pub const REFUTE_DIR: &str = "refute";

/// 反証役が書く結果の file（床の定数・反証の束と同じ dir）。
pub const RESULT_FILE: &str = "result.yaml";

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
    /// 起動の記録の at（読めたときだけ・名札の日付）
    at: Option<String>,
    /// digest.txt の 16 進の先頭 8 字（読めたときだけ・名札の要約値）
    digest: Option<String>,
}

impl Counted {
    /// 束が無い（観点の dir が無い・置き場が無い）。
    fn absent() -> Self {
        Counted {
            verdict: Verdict::Unknown,
            reasons: vec!["束が無い".to_string()],
            findings: 0,
            stops: 0,
            at: None,
            digest: None,
        }
    }
}

/// 名札（便 40）のための観点 1 つの結果。理由は持たない（面は数えた結果だけ・理由は `--check` の標準エラー）。
pub struct Stamp {
    pub id: String,
    pub verdict: Verdict,
    /// 起動の記録の at（読めたときだけ）
    pub at: Option<String>,
    /// digest.txt の 16 進の先頭 8 字（読めたときだけ）
    pub digest: Option<String>,
}

/// 観点ごとの 3 値を名札のために返す（便 40・§1 (b)）。`out` = 束の置き場（解決済み・None = 置き場なし）。
/// 便 39 の規則 1・2・4〜10 をそのまま当て、3（束が古い）だけを当てない。置き場が無い・観点の dir が無い観点は
/// まだ分からない（at と要約値は無し）。天井の正本の viewpoints の順に返す。天井の正本が読めなければ Err（面は導出できない）。
pub fn stamps(dir: &Path, out: Option<&Path>) -> R<Vec<Stamp>> {
    let ceiling = bundle::load(dir)?;
    Ok(ceiling
        .viewpoints
        .iter()
        .map(|vp| {
            let counted = match out {
                Some(out_dir) => count_viewpoint(dir, None, out_dir, &ceiling, vp),
                None => Counted::absent(),
            };
            Stamp {
                id: vp.id.clone(),
                verdict: counted.verdict,
                at: counted.at,
                digest: counted.digest,
            }
        })
        .collect())
}

/// 天井の正本 `<dir>/ceiling.yaml` の viewpoints の（id・name）を正本の順に（名札の観点の名は正本の逐語・P-6.3・
/// `bundle::load` は name を外に出さない）。
pub fn viewpoint_names(dir: &Path) -> R<Vec<(String, String)>> {
    let text = fs::read_to_string(dir.join("ceiling.yaml"))
        .map_err(|e| format!("ceiling.yaml: 読めない: {e}"))?;
    let doc = yaml::parse(&text).map_err(|e| format!("ceiling.yaml: parse できない: {e}"))?;
    let rows = doc
        .root
        .get("viewpoints")
        .and_then(Node::as_seq)
        .ok_or_else(|| "ceiling.yaml: viewpoints: 読めない".to_string())?;
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let field = |key: &str| {
                row.get(key)
                    .and_then(Node::as_str)
                    .filter(|s| !s.trim().is_empty())
                    .map(str::to_string)
                    .ok_or_else(|| format!("ceiling.yaml: viewpoints[{i}].{key}: 読めない"))
            };
            Ok((field("id")?, field("name")?))
        })
        .collect()
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
        let counted = count_viewpoint(dir, Some((&faces_dir, &face_files)), &out_dir, &ceiling, vp);
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

/// `faces` = `--check` の配信先（直下の名つき）。None なら 3（束が古い）を当てない（名札・便 40）。
fn count_viewpoint(
    dir: &Path,
    faces: Option<(&Path, &[(String, bool)])>,
    out_dir: &Path,
    ceiling: &Ceiling,
    vp: &Viewpoint,
) -> Counted {
    let vp_dir = out_dir.join(&vp.id);
    let mut reasons = Vec::new();

    // 1. 束が無い
    if !bundle_present(&vp_dir) {
        return Counted::absent();
    }
    let digest = fs::read(vp_dir.join(DIGEST_FILE)).unwrap_or_default();
    let digest = String::from_utf8_lossy(&digest).into_owned();
    let digest8 = digest_head(&digest);

    // 2. 束が壊れている（置き場の file から測り直す）
    match measure(&vp_dir) {
        Ok(files) if bundle::digest_text(&files) == digest => {}
        Ok(_) => reasons.push("束が壊れている（要約値が digest.txt と違う）".to_string()),
        Err(e) => reasons.push(e),
    }

    // 3. 束が古い（現在の正本から memory の上に組み直す）
    if let Some((faces_dir, face_files)) = faces {
        match bundle::build_one(dir, faces_dir, face_files, ceiling, vp) {
            Ok(files)
                if files
                    .get(DIGEST_FILE)
                    .is_some_and(|d| *d == digest.as_bytes()) => {}
            Ok(_) => reasons.push("束が古い（現在の正本から組んだ要約値と違う）".to_string()),
            Err(e) => reasons.push(e),
        }
    }

    // 4.〜6. 所見 file（止める の所見は反証役の result.yaml も読む・便 42）
    let (sheet, findings, stops, at) = match read_findings(&vp_dir) {
        Ok(Some(root)) => {
            let mut sheet = count_sheet(
                &vp_dir,
                &root,
                &ceiling.rules,
                vp,
                digest.trim_end_matches('\n'),
                true,
            );
            reasons.append(&mut sheet.reasons);
            // 残る 止める の数 = 反証が未 + 反証が 退けた でないもの
            let stops = sheet.unrefuted.len() + sheet.remaining_stops.len();
            (Some(sheet), sheet_findings(&root), stops, record_at(&root))
        }
        Ok(None) => {
            reasons.push("所見 file が無い".to_string());
            (None, 0, 0, None)
        }
        Err(e) => {
            reasons.push(format!("所見 file: {e}"));
            (None, 0, 0, None)
        }
    };
    if !reasons.is_empty() {
        return Counted {
            verdict: Verdict::Unknown,
            reasons,
            findings,
            stops,
            at,
            digest: digest8,
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
            at,
            digest: digest8,
        };
    }
    // 8.〜10. file の verdict と残る所見
    // 9（便 41・delivery-41.md §1 (a)）: 止める が 1 件以上あって全部 退けた なら、verdict の 不合格 は反証の前の判断
    // なので 合格 に読み替えず（P-1）、再判定待ち = まだ分からない。止める が元々 0 件の 不合格 は審査役の判定のまま。
    let verdict = match sheet.verdict.as_str() {
        "不合格" if findings == 0 => {
            reasons.push("不合格なのに所見が無い".to_string());
            Verdict::Unknown
        }
        "不合格" if !sheet.refuted_stops.is_empty() && sheet.remaining_stops.is_empty() => {
            reasons.push(format!(
                "止める所見が全部退けられた（再判定待ち・{}）",
                sheet.refuted_stops.join(", ")
            ));
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
        at,
        digest: digest8,
    }
}

/// digest.txt の 1 行「<規則の名> <16 進>」の 16 進の先頭 8 字（形が違えば None）。
fn digest_head(digest: &str) -> Option<String> {
    let mut words = digest.split_whitespace();
    let (_, hex, None) = (words.next()?, words.next()?, words.next()) else {
        return None;
    };
    let head = hex.get(..8)?;
    head.bytes()
        .all(|b| b.is_ascii_hexdigit())
        .then(|| head.to_string())
}

/// 所見 file の起動の記録の at（空でない文のときだけ）。
fn record_at(root: &Node) -> Option<String> {
    root.get("record")
        .and_then(|r| r.get("at"))
        .and_then(Node::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
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
    for name in BUNDLE_CONTENTS {
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
    read_table(&vp_dir.join(FINDINGS_FILE), FINDINGS_FILE)
}

/// 最上位が欄の表の yaml（所見 file・反証の結果 file）。無い = Ok(None)・読めない / parse できない / 重複キー /
/// 最上位が表でない = Err（file 名: 理由）。
fn read_table(path: &Path, name: &str) -> R<Option<Node>> {
    if path.is_symlink() {
        return Err(format!("{name}: symlink は認めない"));
    }
    if !path.exists() {
        return Ok(None);
    }
    if !path.is_file() {
        return Err(format!("{name}: file でない"));
    }
    let text = fs::read_to_string(path).map_err(|e| format!("{name}: 読めない: {e}"))?;
    let doc = yaml::parse(&text).map_err(|e| format!("{name}: parse できない: {e}"))?;
    if let Some(d) = doc.duplicates.first() {
        return Err(format!(
            "{name}: {} 行: 同じ表に同じキー「{}」が 2 度ある",
            d.line, d.key
        ));
    }
    if doc.root.as_map().is_none() {
        return Err(format!("{name}: 最上位が欄の表でない"));
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
    /// 反証で退けた 止める の所見の id（規則 9 の再判定待ち・便 41）
    refuted_stops: Vec<String>,
}

/// `results` = 止める の所見ごとに反証役の result.yaml も読む（`--check`・便 42 §1 (d)）。`--refute` は所見 file の欄の
/// 決まりだけを数える（result.yaml は在るかどうかだけ見る）。
fn count_sheet(
    vp_dir: &Path,
    root: &Node,
    rules: &Rules,
    vp: &Viewpoint,
    digest: &str,
    results: bool,
) -> Sheet {
    let mut sheet = Sheet {
        reasons: Vec::new(),
        verdict: String::new(),
        unrefuted: Vec::new(),
        remaining_stops: Vec::new(),
        refuted_stops: Vec::new(),
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
        count_findings(
            vp_dir,
            findings,
            rules,
            vp,
            reads.as_ref(),
            results,
            &mut sheet,
        );
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
    results: bool,
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
            // 反証役の result.yaml（便 42 §1 (d)）: 正しく読めればその値を所見の反証の結果にする。所見 file の refute の欄と
            // 両方在れば同じ値だけ可
            let result = if results && id != "?" {
                read_result(vp_dir, &id, rules, &mut sheet.reasons)
            } else {
                None
            };
            if let (Some(r), Some(Some(f))) = (&result, &refute)
                && r != f
            {
                sheet.reasons.push(format!("反証の結果が 2 つ（{id}）"));
            }
            let refute = match result {
                Some(r) => Some(Some(r)),
                None => refute,
            };
            match refute {
                None => sheet.unrefuted.push(id.clone()),
                Some(Some(v)) if v == "退けた" => sheet.refuted_stops.push(id.clone()),
                Some(_) => sheet.remaining_stops.push(id.clone()),
            }
        }
    }
}

// ── 反証の結果 file を読む（便 42・§1 (d)）──

/// 反証の束の dir `<vp_dir>/refute/<所見の id>/`。id が dir の名にできない形（空・`.`・`..`・区切りを含む）なら None。
fn refute_dir(vp_dir: &Path, id: &str) -> Option<PathBuf> {
    let unsafe_name = id.is_empty()
        || id == "."
        || id == ".."
        || id.contains('/')
        || id.contains('\\')
        || id.contains('\0');
    (!unsafe_name).then(|| vp_dir.join(REFUTE_DIR).join(id))
}

/// `<vp_dir>/refute/<id>/result.yaml`。無い = None（反証が未）。在れば欄の決まりで読み、違反は全部
/// 「反証の結果 file: <id>: <理由>」で `reasons` へ（そのときも None）。正しく読めたときだけ refute の値。
fn read_result(
    vp_dir: &Path,
    id: &str,
    rules: &Rules,
    reasons: &mut Vec<String>,
) -> Option<String> {
    let dir = refute_dir(vp_dir, id)?;
    let path = dir.join(RESULT_FILE);
    if !path.exists() && !path.is_symlink() {
        return None;
    }
    let mut why = Vec::new();
    let value = count_result(&dir, id, rules, &mut why);
    if why.is_empty() {
        return value;
    }
    for w in why {
        reasons.push(format!("反証の結果 file: {id}: {w}"));
    }
    None
}

/// result.yaml の欄の決まり: 最上位は表・欄は `RESULT_REQUIRED` の 6 つだけで全部空でない文・id は所見の id・refute は
/// 床の値域・bundle は同じ dir の digest.txt（末尾の改行を除く）と同じ・digest.txt は 5 つの file から測り直した
/// 要約値と同じ。理由は全部 `why` へ。
fn count_result(dir: &Path, id: &str, rules: &Rules, why: &mut Vec<String>) -> Option<String> {
    let root = match read_table(&dir.join(RESULT_FILE), RESULT_FILE) {
        Ok(Some(root)) => root,
        Ok(None) => return None,
        Err(e) => {
            why.push(e);
            return None;
        }
    };
    for (key, _) in root.as_map().expect("最上位は表と読んである") {
        if !RESULT_REQUIRED.contains(&key.as_str()) {
            why.push(format!("未知の欄「{key}」"));
        }
    }
    let digest = fs::read_to_string(dir.join(DIGEST_FILE)).ok();
    let mut refute = None;
    for key in RESULT_REQUIRED {
        let Some(node) = root.get(key) else {
            why.push(format!("{key} が無い"));
            continue;
        };
        let Some(value) = node.as_str() else {
            why.push(format!("{key} が文でない"));
            continue;
        };
        if value.trim().is_empty() {
            why.push(format!("{key} が空"));
            continue;
        }
        match key {
            "id" if value != id => why.push(format!("id「{value}」が所見の id と違う")),
            "refute" if !rules.refute_values.iter().any(|w| w == value) => {
                why.push(format!("refute「{value}」が値域の外"));
            }
            "refute" => refute = Some(value.to_string()),
            "bundle" => match &digest {
                Some(d) if d.trim_end_matches('\n') == value => {}
                Some(_) => why.push("bundle が digest.txt と違う（読んだ束が違う）".to_string()),
                None => why.push(format!("{DIGEST_FILE}: 読めない")),
            },
            _ => {}
        }
    }
    // 反証の束そのものが digest.txt のとおりか（5 つの file から測り直す）
    if let Some(d) = &digest {
        match measure_refute(dir) {
            Ok(files) if bundle::digest_text(&files) == *d => {}
            Ok(_) => why.push("反証の束が壊れている（要約値が digest.txt と違う）".to_string()),
            Err(e) => why.push(format!("反証の束が壊れている（{e}）")),
        }
    }
    refute
}

/// 反証の束の 5 つの file（digest.txt と result.yaml は数えない）。
fn measure_refute(dir: &Path) -> R<Files> {
    let mut files = Files::new();
    for name in REFUTE_CONTENTS {
        let bytes = fs::read(dir.join(name)).map_err(|e| format!("{name}: 読めない: {e}"))?;
        files.insert(name.to_string(), bytes);
    }
    Ok(files)
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

// ── 反証の束を組む（便 42・§1 (a)〜(c)）──

/// `folio ceiling --refute`。`--out` は相対なら `--dir` からの相対・絶対ならそのまま。天井の正本の viewpoints の順に
/// 所見 file を読み、止める で refute の欄が無く result.yaml も無い所見ごとに反証の束を memory の上で用意し、
/// 1 件でも用意できなければ何も書かず 2（全部か無しか）。対象が 0 件でも 0。判定を持たないので 1 は返さない。
pub fn refute(dir: &Path, out: &Path) -> Outcome {
    let out_dir = dir.join(out);
    let ceiling = match bundle::load(dir) {
        Ok(c) => c,
        Err(e) => return before_viewpoints(e),
    };
    if !out_dir.is_dir() {
        return before_viewpoints(format!("{}: 置き場が無い", out_dir.display()));
    }
    let mut stderr = Vec::new();
    let mut bundles: Vec<(PathBuf, Files)> = Vec::new();
    for vp in &ceiling.viewpoints {
        match plan_refutes(&out_dir.join(&vp.id), &ceiling, vp) {
            Ok(planned) => bundles.extend(planned),
            Err(reasons) => stderr.extend(
                reasons
                    .into_iter()
                    .map(|r| format!("folio ceiling: {}: {r}", vp.id)),
            ),
        }
    }
    if !stderr.is_empty() {
        return Outcome {
            verdict: Verdict::Unknown,
            stdout: Vec::new(),
            stderr,
        };
    }
    let (mut count, mut total) = (0, 0);
    for (target, files) in &bundles {
        if let Err(e) = fs::create_dir_all(target) {
            return before_viewpoints(format!("{}: 作れない: {e}", target.display()));
        }
        for (rel, bytes) in files {
            let path = target.join(rel);
            if let Err(e) = fs::write(&path, bytes) {
                return before_viewpoints(format!("{}: 書けない: {e}", path.display()));
            }
            count += 1;
            total += bytes.len();
        }
    }
    Outcome {
        verdict: Verdict::Pass,
        stdout: vec![format!(
            "folio ceiling: 反証の束を組んだ（所見 {}・file {count}・{total} byte）",
            bundles.len()
        )],
        stderr: Vec::new(),
    }
}

/// 反証の対象の所見 1 件（所見 file の欄の決まりを通った所見から取る）。
struct Target {
    id: String,
    doc: String,
    at: String,
    weight: String,
    evidence: String,
    note: Option<String>,
}

impl Target {
    /// 止める で refute の欄が無い所見だけ Some（欄は `count_sheet` を通っているので文で在る）。
    fn from(item: &Node, rules: &Rules) -> Option<Target> {
        let text = |node: Option<&Node>| node.and_then(Node::as_str).map(str::to_string);
        let weight = text(item.get("weight"))?;
        if !rules.weight_refute.contains(&weight) || item.get("refute").is_some() {
            return None;
        }
        let place = item.get("place")?;
        Some(Target {
            id: text(item.get("id"))?,
            doc: text(place.get("doc"))?,
            at: text(place.get("at"))?,
            weight,
            evidence: text(item.get("evidence"))?,
            note: text(item.get("note")),
        })
    }
}

/// 観点 1 つの反証の束を memory の上で用意する。所見 file が無い観点は空。読めない・欄の決まりの違反・親の束が無い・
/// id が dir の名にできないときは理由の列（Err）。戻り値 = (反証の束の dir, file) の列（所見の順）。
fn plan_refutes(
    vp_dir: &Path,
    ceiling: &Ceiling,
    vp: &Viewpoint,
) -> Result<Vec<(PathBuf, Files)>, Vec<String>> {
    let root = match read_findings(vp_dir) {
        Ok(Some(root)) => root,
        Ok(None) => return Ok(Vec::new()),
        Err(e) => return Err(vec![format!("所見 file: {e}")]),
    };
    if !bundle_present(vp_dir) {
        return Err(vec!["束が無い".to_string()]);
    }
    let parent_digest = fs::read_to_string(vp_dir.join(DIGEST_FILE))
        .map_err(|e| vec![format!("{DIGEST_FILE}: 読めない: {e}")])?;
    let sheet = count_sheet(
        vp_dir,
        &root,
        &ceiling.rules,
        vp,
        parent_digest.trim_end_matches('\n'),
        false,
    );
    if !sheet.reasons.is_empty() {
        return Err(sheet
            .reasons
            .into_iter()
            .map(|r| format!("所見 file: {r}"))
            .collect());
    }
    let reads = fs::read(vp_dir.join("reads.yaml"))
        .map_err(|e| vec![format!("reads.yaml: 読めない: {e}")])?;
    let question = bundle::question_text(vp);
    let mut planned = Vec::new();
    for item in root.get("findings").and_then(Node::as_seq).unwrap_or(&[]) {
        let Some(target) = Target::from(item, &ceiling.rules) else {
            continue;
        };
        let Some(dir) = refute_dir(vp_dir, &target.id) else {
            return Err(vec![format!(
                "所見 {}: id を dir の名にできない",
                target.id
            )]);
        };
        // 反証役が結果を書いた dir は触らない
        if dir.join(RESULT_FILE).exists() || dir.join(RESULT_FILE).is_symlink() {
            continue;
        }
        let files = refute_files(
            &target,
            vp,
            &question,
            &reads,
            &parent_digest,
            &ceiling.rules,
        );
        planned.push((dir, files));
    }
    Ok(planned)
}

/// 反証の束の 5 つ + digest.txt（§1 (c)）。`question` = 観点の束の question.yaml（6 行）・`reads` = 観点の束の
/// reads.yaml の byte・`parent_digest` = 観点の束の digest.txt の byte（sources.txt）。
fn refute_files(
    t: &Target,
    vp: &Viewpoint,
    question: &str,
    reads: &[u8],
    parent_digest: &str,
    rules: &Rules,
) -> Files {
    let mut files = Files::new();
    let mut finding = format!(
        "id: {}\nviewpoint: {}\nplace: {{doc: {}, at: {}}}\nweight: {}\nevidence: |\n{}",
        t.id,
        vp.id,
        t.doc,
        t.at,
        t.weight,
        block(&t.evidence, "  ")
    );
    if let Some(note) = &t.note {
        finding.push_str("note: |\n");
        finding.push_str(&block(note, "  "));
    }
    files.insert("finding.yaml".to_string(), finding.into_bytes());
    let values = rules.refute_values.join(", ");
    files.insert(
        "question.yaml".to_string(),
        format!(
            "{question}refute:\n  values: [{values}]\n  rule: |\n{}",
            block(REFUTE_RULE, "    ")
        )
        .into_bytes(),
    );
    files.insert("reads.yaml".to_string(), reads.to_vec());
    files.insert(
        "schema.yaml".to_string(),
        format!(
            "# 反証の結果の欄の決まり（folio ceiling --refute が組んだ・結果は同じ dir の result.yaml に書く）\n\
             result:\n  required: [{}]\n  values: [{values}]\n",
            RESULT_REQUIRED.join(", ")
        )
        .into_bytes(),
    );
    files.insert("sources.txt".to_string(), parent_digest.as_bytes().to_vec());
    let digest = bundle::digest_text(&files);
    files.insert(DIGEST_FILE.to_string(), digest.into_bytes());
    files
}

/// 区間の文（`|`）の本文: 末尾の改行を落として行ごとに `indent` を付け、各行を改行で閉じる。
fn block(text: &str, indent: &str) -> String {
    text.trim_end_matches('\n')
        .split('\n')
        .map(|line| format!("{indent}{line}\n"))
        .collect()
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

    #[test]
    fn findings_refute_dir_rejects_ids_that_cannot_name_a_dir() {
        let vp = Path::new("out/fidelity");
        assert_eq!(
            refute_dir(vp, "F-1"),
            Some(PathBuf::from("out/fidelity/refute/F-1"))
        );
        for id in ["", ".", "..", "a/b", "a\\b"] {
            assert!(refute_dir(vp, id).is_none(), "{id:?}");
        }
    }

    #[test]
    fn findings_block_indents_every_line_and_ends_with_one_newline() {
        assert_eq!(block("a", "  "), "  a\n");
        assert_eq!(block("a\nb\n", "    "), "    a\n    b\n");
    }
}
