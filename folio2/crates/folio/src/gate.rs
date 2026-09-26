//! `folio ceiling --gate`（便 73・docs/design/delivery-73.md §1 (a)〜(c)・FR20 / AC18・設計ノート ceiling-gate.md §2）。
//! 天井の印 `<dir>/preview/ceiling-stamp.yaml`（便 72 の欄の決まり）と便の書き換える file の一覧（`--write-set`）から
//! 3 値を返す: 設計文書の正本を書き換えない便は 通す（0）・印が 4 観点とも合格で正本の要約値が同じなら 通す（0）・
//! 不合格 が在れば 止める（1）・印が無い / 古い / まだ分からない が在れば まだ分からない（2）。
//! 設計文書の正本 = `<dir>` の下に在り、`<dir>/preview/` の下でなく、path のどの要素も retired でないもの。
//! 正本の要約値は観点の reads が指す文書の file（file 形はその file・dir 形は直下の .yaml）の全文を `<dir>` からの
//! 相対 path の byte 順に連結した sha256。印（`stamp.rs`）も欄 sources をこの関数で測る（便 104・2 面に実装しない）。
//! 何も書かない。標準出力は 1 行「folio ceiling: <3 値>（<理由>）」。
//!
//! 便 126（docs/design/delivery-126.md §1 (b)(c)・ADR-18 決定 (1)(4)(5)）: 古さは印の欄 trigger（引き金の要約値）で判定する。
//! 引き金の要約値は天井の床の定数の規範の欄の一覧（`ceiling.rs` の TRIGGER_*）だけを写した木の正規化の sha256 で、印も
//! この `trigger_digest` で測る。合格で引き金が同じなら通し、正本の要約値だけが違えば印の nodes と rest を今の表と突き合わせて
//! 変わった節点の数を理由の行に添える（今の表は命令の入口が渡す＝層 3 の `graph.rs` を名指さない）。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::anchor;
use crate::ceiling::{
    TRIGGER_ADR_FIELDS, TRIGGER_CEILING_ROWS, TRIGGER_CEILING_WHOLE,
    TRIGGER_CONSTITUTION_SCOPE, TRIGGER_RULES_FIELDS, TRIGGER_RULES_SECTIONS, TRIGGER_SRS_ROWS, TRIGGER_SRS_WHOLE, TriggerRows,
};
use crate::ceiling_src::{self, STAMP_FILE};
use crate::cursor::{self, R};
use crate::sha256;
use crate::verdict::Verdict;
use crate::yaml::{self, Node, Value};

/// 置き場と天井の正本から（残差の要約値・節点ごとの要約値の表）を組む関数（命令の入口が `graph::stamp_table` を渡す）。
pub(crate) type NodeTable = fn(&Path, &ceiling_src::Ceiling) -> R<(String, Vec<(String, String)>)>;

/// 1 回の実行の結果。`stdout` は 1 行。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: String,
}

impl Outcome {
    fn new(verdict: Verdict, reason: impl Into<String>) -> Self {
        let word = match verdict {
            Verdict::Pass => "通す",
            Verdict::Fail => "止める",
            Verdict::Unknown => "まだ分からない",
        };
        Outcome {
            verdict,
            stdout: format!("folio ceiling: {word}（{}）", reason.into()),
        }
    }
}

// ── 命令の口 ──

/// 理由の字（便 142 §1 (b) の 3）: `--dir` が今の dir の下に無い。
const UNKNOWN_DIR_OUTSIDE: &str =
    "--dir が今の dir の下に無い・write-set の根と照らせない・作業ツリーの一番上から撃つ";
/// 理由の字の頭: write-set の path が `--dir` と同じ根からの相対で読めない。
const UNKNOWN_OTHER_ROOT: &str = "--dir と write-set の根が違う";
/// 理由の字の末尾（撃ち直し方）。
const FROM_THE_TOP: &str = "作業ツリーの一番上から撃つ";
/// 理由の字の頭（便 150 §1 (b) の 2）: `--dir` が設計文書の置き場でない。
const UNKNOWN_NOT_A_PLACE: &str = "--dir が設計文書の置き場でない";
/// 置き場の印の file（便 150 §1 (b) の 1）: `folio check` が最初に読む正本（`check.rs` の FILES の先頭）。
const PLACE_MARK: &str = "constitution.yaml";

/// `write_set` の各 path は repo の根からの相対（接頭辞 + / - / ~ は剥がす）。`dir` は同じ根からの `--dir`。
/// `table` は今の節点の表を組む関数（§1 (c) の 3）。判定の順は §1 (c) の 2 のとおりで、最初に当たったもので決まる。
/// その前に、`--dir` と write-set を同じ根（今の dir）で照らせるかを確かめる（便 142・照らせなければ まだ分からない）。
pub(crate) fn run(dir: &Path, write_set: &[String], table: NodeTable) -> Outcome {
    // 根の突き合わせ（便 142）: 照らせなければ設計文書の判定より前に まだ分からない（P-4.1 / P-4.2）
    let Some(root) = dir_parts(dir) else {
        return Outcome::new(Verdict::Unknown, UNKNOWN_DIR_OUTSIDE);
    };
    if let Some(p) = write_set.iter().find(|p| other_root(&root, p)) {
        let p = p.trim_start_matches(['+', '-', '~']);
        return Outcome::new(
            Verdict::Unknown,
            format!("{UNKNOWN_OTHER_ROOT}＝{p}・{FROM_THE_TOP}"),
        );
    }
    // 置き場の確かめ（便 150）: 置き場でない `--dir` では write-set のどれが置き場の file かを照らせない（P-4.1 / P-4.2）
    if !is_place(dir) {
        return Outcome::new(
            Verdict::Unknown,
            format!(
                "{UNKNOWN_NOT_A_PLACE}（{}・{PLACE_MARK} が無い）・{FROM_THE_TOP}",
                dir.display()
            ),
        );
    }
    if !write_set.iter().any(|p| is_design_source(&root, p)) {
        return Outcome::new(Verdict::Pass, "設計文書の正本を書き換えない便");
    }
    let stamp = match read_stamp(dir) {
        Ok(Some(s)) => s,
        Ok(None) => return Outcome::new(Verdict::Unknown, "印が無い"),
        Err(e) => return Outcome::new(Verdict::Unknown, format!("印が読めない: {e}")),
    };
    let ceiling = match ceiling_src::load(dir) {
        Ok(c) => c,
        Err(e) => return Outcome::new(Verdict::Unknown, e),
    };
    let Some(trigger) = &stamp.trigger else {
        return Outcome::new(Verdict::Unknown, "印が古い（引き金の要約値の欄が無い）");
    };
    match trigger_digest(dir) {
        Ok(now) if now == *trigger => {}
        Ok(_) => return Outcome::new(Verdict::Unknown, "印が古い（引き金の要約値が違う）"),
        Err(e) => {
            return Outcome::new(Verdict::Unknown, format!("引き金の要約値が測れない: {e}"));
        }
    }
    let mut unknown = Vec::new();
    let mut failed = Vec::new();
    for vp in &ceiling.viewpoints {
        match stamp.viewpoints.iter().find(|(id, _)| *id == vp.id) {
            Some((_, v)) if v == "合格" => {}
            Some((_, v)) if v == "不合格" => failed.push(vp.id.as_str()),
            _ => unknown.push(vp.id.as_str()),
        }
    }
    if !unknown.is_empty() {
        return Outcome::new(
            Verdict::Unknown,
            format!("まだ分からない観点: {}", unknown.join("・")),
        );
    }
    if !failed.is_empty() {
        return Outcome::new(Verdict::Fail, format!("不合格の観点: {}", failed.join("・")));
    }
    match sources_digest(dir, &ceiling) {
        Ok(now) if now == stamp.sources => {
            return Outcome::new(
                Verdict::Pass,
                "印が 4 観点とも合格・引き金の要約値が同じ・正本の要約値が同じ",
            );
        }
        Ok(_) => {}
        Err(e) => return Outcome::new(Verdict::Unknown, e),
    }
    // 正本の要約値だけが違う: 印の後に変わった節点を数えて添える（数えられなければ通さない・P-4.1 / P-4.2）
    let Some((rest, nodes)) = &stamp.nodes else {
        return Outcome::new(Verdict::Unknown, "印の節点の表が読めない");
    };
    let (now_rest, now_nodes) = match table(dir, &ceiling) {
        Ok(t) => t,
        Err(e) => {
            return Outcome::new(
                Verdict::Unknown,
                format!("印の後に変わった節点が数えられない: {e}"),
            );
        }
    };
    let then: BTreeMap<&str, &str> = nodes.iter().map(|(i, d)| (i.as_str(), d.as_str())).collect();
    let now: BTreeMap<&str, &str> = now_nodes.iter().map(|(i, d)| (i.as_str(), d.as_str())).collect();
    let changed = then.iter().filter(|(id, d)| now.get(*id) != Some(*d)).count()
        + now.keys().filter(|id| !then.contains_key(*id)).count();
    let outside = if *rest == now_rest { "" } else { "と節点の外の字" };
    Outcome::new(
        Verdict::Pass,
        format!(
            "印が 4 観点とも合格・引き金の要約値が同じ・印の後に引き金の外の変更が在る（節点 {changed} 個{outside}・次の引き金の周が読む）"
        ),
    )
}

// ── 設計文書の判定（§1 (b)）──

/// path を要素に分ける（`.` と空の要素は落とす）。
fn parts(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty() && *s != ".").collect()
}

/// `--dir` の今の dir からの要素（字面で解く・symlink は解かない・便 142 §1 (b) の 1）。絶対 path は今の dir の下なら
/// 今の dir からの相対に直す。`.` は落とし、`..` は 1 つ前の要素を外す。今の dir の下に無い（外へ出る・下に無い絶対
/// path・今の dir が取れない・UTF-8 でない要素）なら None。
fn dir_parts(dir: &Path) -> Option<Vec<String>> {
    let rel: PathBuf = if dir.is_absolute() {
        let cwd = std::env::current_dir().ok()?;
        dir.strip_prefix(cwd).ok()?.to_path_buf()
    } else {
        dir.to_path_buf()
    };
    let mut out: Vec<String> = Vec::new();
    for c in rel.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop()?;
            }
            Component::Normal(s) => out.push(s.to_str()?.to_string()),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(out)
}

/// write-set の path が `--dir` と同じ根（作業ツリーの一番上）からの相対で読めないか（便 142 §1 (b) の 2）: 絶対 path・
/// `..` の要素を持つ・`--dir` の要素の列の途中から後ろの部分の下に在り、列の全部では始まらない。
fn other_root(root: &[String], path: &str) -> bool {
    let path = path.trim_start_matches(['+', '-', '~']);
    if path.starts_with('/') {
        return true;
    }
    let parts = parts(path);
    if parts.contains(&"..") {
        return true;
    }
    let under = |base: &[String]| {
        parts.len() > base.len() && parts.iter().zip(base).all(|(a, b)| *a == b)
    };
    !under(root) && (1..root.len()).any(|k| under(&root[k..]))
}

/// `--dir` が設計文書の置き場か（便 150 §1 (b) の 1）: 直下に印 PLACE_MARK が file として在る（中身は読まない）。
fn is_place(dir: &Path) -> bool {
    dir.join(PLACE_MARK).is_file()
}

/// 設計文書の正本か: `<dir>` の下・`<dir>/preview/` の下でない・どの要素も retired でない。
fn is_design_source(root: &[String], path: &str) -> bool {
    let path = path.trim_start_matches(['+', '-', '~']);
    let parts = parts(path);
    if parts.len() <= root.len() || parts.iter().zip(root).any(|(a, b)| *a != b) {
        return false;
    }
    let rest = &parts[root.len()..];
    rest[0] != "preview" && !parts.contains(&"retired")
}

// ── 印の判定（§1 (c)）──

/// 印から読む欄（sources・trigger・観点ごとの 3 値・rest と nodes）。trigger は欄が無ければ None、rest と nodes は
/// どちらかが無いか読めなければ None（門は通す前に読めないと言う）。
struct Stamp {
    sources: String,
    trigger: Option<String>,
    viewpoints: Vec<(String, String)>,
    nodes: Option<(String, Vec<(String, String)>)>,
}

/// 印を読む。無ければ None。
fn read_stamp(dir: &Path) -> R<Option<Stamp>> {
    let path = dir.join(STAMP_FILE);
    if path.is_symlink() {
        return Err("symlink は認めない".to_string());
    }
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("読めない: {e}"))?;
    let root = yaml::parse(&text)
        .map_err(|e| format!("parse できない: {e}"))?
        .root;
    let sources = root
        .get("sources")
        .and_then(Node::as_str)
        .ok_or_else(|| "sources が読めない".to_string())?
        .to_string();
    let viewpoints = root
        .get("viewpoints")
        .and_then(Node::as_seq)
        .and_then(|rows| {
            rows.iter()
                .map(|row| {
                    let id = row.get("id").and_then(Node::as_str)?;
                    let verdict = row.get("verdict").and_then(Node::as_str)?;
                    Some((id.to_string(), verdict.to_string()))
                })
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| "viewpoints が読めない".to_string())?;
    let trigger = root.get("trigger").and_then(Node::as_str).map(str::to_string);
    let rows = match root.get("nodes") {
        Some(Node::Null) => Some(Vec::new()),
        Some(Node::Seq(rows)) => rows
            .iter()
            .map(|row| {
                let id = row.get("id").and_then(Node::as_str)?;
                let digest = row.get("digest").and_then(Node::as_str)?;
                Some((id.to_string(), digest.to_string()))
            })
            .collect::<Option<Vec<_>>>(),
        _ => None,
    };
    let rest = root.get("rest").and_then(Node::as_str).map(str::to_string);
    Ok(Some(Stamp {
        sources,
        trigger,
        viewpoints,
        nodes: rest.zip(rows),
    }))
}

// ── 引き金の要約値（§1 (b) の 3）──

/// 今の引き金の要約値（「sha256 <16 進>」）。文書を型付きで読み（`cursor::load`）、天井の床の定数の規範の欄の一覧だけを
/// 写した木（最上位は文書の id を鍵にした表）を正規化（`yaml::canonical`）して測る。印（`stamp.rs`）もこの関数で測る
/// （P-15.2）。文書の file は天井の正本の documents で解く。欄や節が無ければ null、行の一覧の節が一覧でない・行が表で
/// ないなど、どの手順で失敗しても Err（理由 1 つ）。
pub(crate) fn trigger_digest(dir: &Path) -> R<String> {
    let documents = documents(dir)?;
    let file = |id: &str| {
        documents
            .iter()
            .find(|(d, _)| d == id)
            .map(|(_, f)| f.clone())
            .ok_or_else(|| format!("{id}: 文書の一覧に無い"))
    };
    let key = |k: &str| Value::Str(k.to_string());
    let mut tree: Vec<(Value, Value)> = Vec::new();

    // 憲法: 凍結 anchor の写しの式（範囲 = 改訂の範囲の定数・schema 節と前文は丸ごと・便 129）
    let name = file("constitution")?;
    let constitution = cursor::load(dir, &name)?;
    let scope: Vec<String> = TRIGGER_CONSTITUTION_SCOPE.iter().map(|s| s.to_string()).collect();
    let projected = anchor::project(&constitution, &scope).map_err(|e| format!("{name}: {e}"))?;
    tree.push((key("constitution"), projected));

    // 判断の記録: 状態を問わず記録ごとに fields（便 151・ADR-26 決定 (3)）
    tree.push((key("adr"), adr_records(dir, &file("adr")?)?));

    // 要件書: 行の一覧の節と丸ごとの節
    let name = file("srs")?;
    let srs = cursor::load(dir, &name)?;
    tree.push((key("srs"), sections(&name, &srs, &TRIGGER_SRS_ROWS, &TRIGGER_SRS_WHOLE)?));

    // 規則の表: sections の各行の fields
    let name = file("rules")?;
    let rules = cursor::load(dir, &name)?;
    let fields: &'static [&'static str] = &TRIGGER_RULES_FIELDS;
    let rows: Vec<(&'static str, &'static [&'static str])> =
        TRIGGER_RULES_SECTIONS.iter().map(|s| (*s, fields)).collect();
    tree.push((key("rules"), sections(&name, &rules, &rows, &[])?));

    // 天井の正本: 丸ごとの節と行の一覧の節
    let name = file("ceiling")?;
    let ceiling = cursor::load(dir, &name)?;
    tree.push((
        key("ceiling"),
        sections(&name, &ceiling, &TRIGGER_CEILING_ROWS, &TRIGGER_CEILING_WHOLE)?,
    ));

    let text = yaml::canonical(&Value::Map(tree))?;
    Ok(format!("sha256 {}", sha256::hex(text.as_bytes())))
}

/// 行の一覧の節（節の名を鍵に、各行を欄の表にした一覧）と丸ごとの節（節の名を鍵に、木を丸ごと）の表。
fn sections(name: &str, doc: &Value, rows: &TriggerRows, whole: &[&str]) -> R<Value> {
    let mut out: Vec<(Value, Value)> = Vec::new();
    for (section, fields) in rows {
        let value = match doc.get(section) {
            None | Some(Value::Null) => Value::Null,
            Some(Value::Seq(items)) => Value::Seq(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, row)| {
                        row.as_map()
                            .map(|_| pick(row, fields))
                            .ok_or_else(|| format!("{name}: {section}[{i}] が表でない"))
                    })
                    .collect::<R<Vec<_>>>()?,
            ),
            Some(_) => return Err(format!("{name}: {section} が一覧でない")),
        };
        out.push((Value::Str(section.to_string()), value));
    }
    for section in whole {
        let value = doc.get(section).cloned().unwrap_or(Value::Null);
        out.push((Value::Str(section.to_string()), value));
    }
    Ok(Value::Map(out))
}

/// 行 1 つを欄の表に写す（点を含む欄は入れ子を辿った値・鍵は点を含む字のまま・無ければ null）。
fn pick(row: &Value, fields: &[&str]) -> Value {
    Value::Map(
        fields
            .iter()
            .map(|f| {
                let value = f
                    .split('.')
                    .try_fold(row, |node, k| node.get(k))
                    .cloned()
                    .unwrap_or(Value::Null);
                (Value::Str(f.to_string()), value)
            })
            .collect(),
    )
}

/// 判断の記録の dir の直下の .yaml（欄の決まり schema.yaml を除く・名の byte 順）を状態を問わず全部、fields の欄の表に
/// して並べた一覧（下の dir は読まない・便 151・ADR-26 決定 (3)）。
fn adr_records(dir: &Path, file: &str) -> R<Value> {
    if !file.ends_with('/') {
        return Err(format!("{file}: 判断の記録の置き場が dir 形でない"));
    }
    let path = dir.join(file);
    if path.is_symlink() {
        return Err(format!("{file}: symlink は認めない"));
    }
    let mut out = Vec::new();
    for (name, is_file) in ceiling_src::read_dir_names(&path)? {
        if !is_file || !name.ends_with(".yaml") || name == "schema.yaml" {
            continue;
        }
        if path.join(&name).is_symlink() {
            return Err(format!("{file}{name}: symlink は認めない"));
        }
        let record = cursor::load(dir, &format!("{file}{name}"))?;
        out.push(pick(&record, &TRIGGER_ADR_FIELDS));
    }
    Ok(Value::Seq(out))
}

/// 今の正本の要約値（「sha256 <16 進>」）。観点の reads が指す文書だけを全文で集める（印の sources も同じ関数・便 104）。
pub(crate) fn sources_digest(dir: &Path, ceiling: &ceiling_src::Ceiling) -> R<String> {
    let documents = documents(dir)?;
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut seen: Vec<&str> = Vec::new();
    for vp in &ceiling.viewpoints {
        for (doc, _) in &vp.reads {
            if seen.contains(&doc.as_str()) {
                continue;
            }
            seen.push(doc);
            let file = documents
                .iter()
                .find(|(id, _)| id == doc)
                .map(|(_, f)| f.as_str())
                .ok_or_else(|| format!("{doc}: 文書の一覧に無い"))?;
            collect(dir, file, &mut files)?;
        }
    }
    let bytes: Vec<u8> = files.into_values().flatten().collect();
    Ok(format!("sha256 {}", sha256::hex(&bytes)))
}

/// 天井の正本の documents（id・file）。
pub(crate) fn documents(dir: &Path) -> R<Vec<(String, String)>> {
    let text = fs::read_to_string(dir.join("ceiling.yaml"))
        .map_err(|e| format!("ceiling.yaml: 読めない: {e}"))?;
    let root = yaml::parse(&text)
        .map_err(|e| format!("ceiling.yaml: parse できない: {e}"))?
        .root;
    root.get("documents")
        .and_then(Node::as_seq)
        .and_then(|rows| {
            rows.iter()
                .map(|row| {
                    let id = row.get("id").and_then(Node::as_str)?;
                    let file = row.get("file").and_then(Node::as_str)?;
                    Some((id.to_string(), file.to_string()))
                })
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| "ceiling.yaml: documents: 読めない".to_string())
}

/// 文書 1 つの file。file 形はその file、dir 形（末尾が /）は直下の .yaml（`<dir>` からの相対 path で持つ）。
pub(crate) fn collect(dir: &Path, file: &str, files: &mut BTreeMap<String, Vec<u8>>) -> R<()> {
    let path = dir.join(file);
    if path.is_symlink() {
        return Err(format!("{file}: symlink は認めない"));
    }
    if !file.ends_with('/') {
        let bytes = fs::read(&path).map_err(|e| format!("{file}: 読めない: {e}"))?;
        files.insert(file.to_string(), bytes);
        return Ok(());
    }
    for (name, is_file) in ceiling_src::read_dir_names(&path)? {
        if !is_file || !name.ends_with(".yaml") {
            continue;
        }
        let entry = path.join(&name);
        if entry.is_symlink() {
            return Err(format!("{file}{name}: symlink は認めない"));
        }
        let bytes = fs::read(&entry).map_err(|e| format!("{file}{name}: 読めない: {e}"))?;
        files.insert(format!("{file}{name}"), bytes);
    }
    Ok(())
}

#[cfg(test)]
mod gate_tests {
    use super::*;

    #[test]
    fn gate_classifies_design_sources() {
        let root = vec!["design-intent".to_string()];
        let yes = |p: &str| is_design_source(&root, p);
        assert!(yes("design-intent/srs.yaml"));
        assert!(yes("+design-intent/adr/ADR-9.yaml"));
        assert!(yes("~./design-intent/design-note/x.yaml"));
        assert!(!yes("design-intent"));
        assert!(!yes("design-intent/preview/ceiling-stamp.yaml"));
        assert!(!yes("design-intent/adr/retired/ADR-0.yaml"));
        assert!(!yes("crates/folio/src/gate.rs"));
        assert!(!yes("docs/design/delivery-73.md"));
        assert!(!yes("design-intent-x/srs.yaml"));
    }

    #[test]
    fn f142_the_dir_and_the_write_set_share_one_root() {
        let one = |s: &[&str]| Some(s.iter().map(|p| p.to_string()).collect::<Vec<_>>());
        assert_eq!(dir_parts(Path::new("design-intent")), one(&["design-intent"]));
        assert_eq!(dir_parts(Path::new("./design-intent")), one(&["design-intent"]));
        assert_eq!(dir_parts(Path::new("a/../design-intent")), one(&["design-intent"]));
        let below = std::env::current_dir().unwrap().join("x/design-intent");
        assert_eq!(dir_parts(&below), one(&["x", "design-intent"]));
        assert_eq!(dir_parts(Path::new("../design-intent")), None);
        assert_eq!(dir_parts(Path::new("/nonexistent-f142/design-intent")), None);

        let deep: Vec<String> = [".worktrees", "x", "design-intent"].map(String::from).to_vec();
        let other = |p: &str| other_root(&deep, p);
        assert!(other("design-intent/srs.yaml"));
        assert!(other("+design-intent/adr/ADR-9.yaml"));
        assert!(other("x/design-intent/srs.yaml"));
        assert!(!other(".worktrees/x/design-intent/srs.yaml"));
        assert!(!other("crates/folio/src/gate.rs"));
        assert!(!other("design-intent"));

        let top = vec!["design-intent".to_string()];
        let other = |p: &str| other_root(&top, p);
        assert!(!other("design-intent/srs.yaml"));
        assert!(!other("~./design-intent/srs.yaml"));
        assert!(!other("crates/folio/src/gate.rs"));
        assert!(other("/abs/design-intent/srs.yaml"));
        assert!(other("crates/../design-intent/srs.yaml"));
    }

    #[test]
    fn f150_the_place_is_a_dir_with_the_mark() {
        assert_eq!(PLACE_MARK, format!("{}.yaml", crate::check::FILES[0]));
        let td = std::env::temp_dir().join(format!("folio-f150-place-{}", std::process::id()));
        let _ = fs::remove_dir_all(&td);
        let dir = |name: &str| {
            let d = td.join(name);
            fs::create_dir_all(&d).unwrap();
            d
        };
        let place = dir("place");
        fs::write(place.join(PLACE_MARK), "").unwrap();
        let empty = dir("empty");
        let named = dir("named");
        fs::create_dir_all(named.join(PLACE_MARK)).unwrap();
        let others = dir("others");
        fs::write(others.join("ceiling.yaml"), "").unwrap();
        fs::write(others.join("index.yaml"), "").unwrap();
        let answers = [
            is_place(&place),
            is_place(&td.join("missing")),
            is_place(&empty),
            is_place(&named),
            is_place(&others),
            is_place(&place.join(PLACE_MARK)),
        ];
        let _ = fs::remove_dir_all(&td);
        assert_eq!(answers, [true, false, false, false, false, false]);
    }
}
