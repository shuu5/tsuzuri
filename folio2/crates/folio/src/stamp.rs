//! `folio ceiling --stamp`（便 72・docs/design/delivery-72.md §1 (a)〜(c)・FR20 / AC18・設計ノート ceiling-gate.md §6 の 2）。
//! 周の結果（`--write` が組んだ観点ごとの束・席か器が書いた所見 file・止める の反証の結果 result.yaml）から天井の印を
//! 決定的に導出し `<dir>/preview/ceiling-stamp.yaml` へ書く。観点ごとの 3 値は `--check` と同じ口
//! （`findings::count_viewpoint`・配信先を取らないので規則 3〔束が古い〕は当てない＝名札と同じ）で数え、二重に実装しない。
//! 印の欄は閉じた一覧でこの順: round・at・verdict・sources・faces・viewpoints・refutes・reads（周の引き金の要約値 trigger は
//! 便 175、節点の表 rest・nodes は便 177〔docs/design/delivery-177.md §1 (b)〕で外した＝門も名札も読まない・ADR-30 決定 (5)(6)）。
//! 決定性: 時刻・絶対 path・環境の値を書かない（round は置き場の dir の名・at は所見 file の起動の記録から取る）。
//! 欄 sources は束の写しでなく `--dir` の正本から、名札と同じ関数（`gate.rs` の `sources_digest`）で測る（便 104・
//! docs/design/delivery-104.md §1 (b)・FR20 の 正本の要約値）。束の sources/ の写しは便 98 で観点ごとに絞られ、同じ文書でも
//! 観点で byte が違う。欄 faces は束の faces/ の和集合のまま（面は絞られていない・門は faces を突き合わせない）。
//! 全部か無しか: 観点のどれかの束・所見 file・起動の記録が読めない、または面の写しが観点で食い違うときは
//! まだ分からない（終了 2）で file を触らない。既に同じ byte なら書かない。判定の 3 値は印の中身で、命令は書けたら 0。
//! 印の file 名 `STAMP_FILE` は便 110 で `ceiling_src.rs` へ降ろした（ADR-15・層 1 読む・門も同じ名を読む）。
//! 便 169（docs/design/delivery-169.md §1 (b) の 2・ADR-30 決定 (6)）: refutes の行は 止める の所見の全件を
//! `{viewpoint, finding, refute, at, file}` で書く（反証の済んでいない 止める は欄 refute を書かない・file は `stop_file`）。
//! 観点の行は、まだ分からない の理由が反証の済んでいない 止める だけのとき末尾に欄 `wait: 反証` を足す。印は周の 3 値に依らず書く。
//! 便 176（docs/design/delivery-176.md §1 (b)・FR20）: dir 形の文書で at の頭が file 名に解けないときは、置き場の直下の .yaml の
//! 最上位の meta.id でも解く（ちょうど 1 file のときだけ・`meta_file`）。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::adr;
use crate::ceiling_src::{self, Ceiling, Files, STAMP_FILE};
use crate::cursor::{self, R};
use crate::findings::{self, Counted, Refute};
use crate::gate::{self, WAIT_REFUTE};
use crate::sha256;
use crate::verdict::Verdict;
use crate::yaml::{self, Node};

/// 印の先頭の注釈（1 行・頭に置き場の名〔導けなければ名なし〕を付ける・便 154）。
const HEADER: &str = "天井の印 — 生成物（folio ceiling --stamp が書く・手で直さない・P-6.2）";

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
    let ceiling = ceiling_src::load(dir)?;
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
    derive_text(dir, rows, &round, &sources, &faces)
}

/// 印の先頭の欄と観点の行を組み、反証の行と reads の行へ進む。
fn derive_text(dir: &Path, rows: Vec<Row>, round: &str, sources: &str, faces: &str) -> R<String> {
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
        "# {}\nround: {}\nat: {}\nverdict: {verdict}\nsources: {sources}\nfaces: {faces}\nviewpoints:\n",
        adr::named(adr::name_of(dir).as_deref(), " ", HEADER),
        plain(round),
        plain(&at)
    );
    for r in &rows {
        // 反証待ちだけの まだ分からない は門が file ごとの判定に任せる（便 169 §1 (b) の 2）
        let wait = if r.counted.verdict == Verdict::Unknown && r.counted.waiting {
            format!(", wait: {WAIT_REFUTE}")
        } else {
            String::new()
        };
        text.push_str(&format!(
            "  - {{id: {}, verdict: {}, findings: {}, stops: {}, bundle: {}, model: {}, effort: {}, at: {}{wait}}}\n",
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
    derive_refutes(dir, &rows, text, reads)
}

/// 止める の所見の全件の行（refutes）と reads の行を足して印の全文にする。
fn derive_refutes(dir: &Path, rows: &[Row], mut text: String, reads: BTreeSet<&str>) -> R<String> {
    let documents = gate::documents(dir)?;
    let mut refutes = Vec::new();
    for r in rows {
        let mut own: Vec<&Refute> = r.counted.refutes.iter().collect();
        own.sort_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()));
        for stop in own {
            let file = stop_file(dir, &documents, &stop.doc, &stop.at)
                .map_err(|e| format!("{}: {}: {e}", r.id, stop.id))?;
            let refute = stop
                .value
                .as_deref()
                .map(|v| format!(", refute: {}", plain(v)))
                .unwrap_or_default();
            refutes.push(format!(
                "  - {{viewpoint: {}, finding: {}{refute}, at: {}, file: {}}}\n",
                plain(&r.id),
                plain(&stop.id),
                plain(&stop.at),
                plain(&file)
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

/// 止める の場所の file（便 169 §1 (b) の 2・便 176）: doc を天井の正本の documents で解き、file 形はその file、dir 形は at の頭
/// （最初の `.` の前）の `<dir><頭>.yaml` が `--dir` の下に file として在ればそれ、無ければ `meta_file` がちょうど 1 つに
/// 解いた file、頭が空・`.` 始まり・区切りを含むか `<dir><頭>.yaml` が symlink か `meta_file` が解けなければ dir そのもの
/// （広い側）。doc が文書の一覧に無ければ Err（印を組まない・全部か無しか）。
fn stop_file(dir: &Path, documents: &[(String, String)], doc: &str, at: &str) -> R<String> {
    let file = documents
        .iter()
        .find(|(id, _)| id == doc)
        .map(|(_, f)| f.as_str())
        .ok_or_else(|| format!("止める の場所の doc「{doc}」が文書の一覧に無い"))?;
    if !file.ends_with('/') {
        return Ok(file.to_string());
    }
    let head = at.split('.').next().unwrap_or_default();
    let unsafe_head = head.is_empty() || head.starts_with('.') || head.contains(['/', '\\', '\0']);
    let named = format!("{file}{head}.yaml");
    if unsafe_head || dir.join(&named).is_symlink() {
        Ok(file.to_string())
    } else if dir.join(&named).is_file() {
        Ok(named)
    } else {
        Ok(meta_file(dir, file, head).unwrap_or_else(|| file.to_string()))
    }
}

/// 便 176（docs/design/delivery-176.md §1 (b) の 1）: dir 形の置き場 `file` の直下の .yaml（symlink と下の dir は見ない）のうち、
/// 最上位の meta.id が `head` と同じ file がちょうど 1 つならその file。0 か 2 つ以上か、置き場か .yaml のどれかが読めない
/// （UTF-8 でない・parse できない・重複キー）なら None（広い側・読み違いで狭めない）。数えるだけなので並びに依らない。
fn meta_file(dir: &Path, file: &str, head: &str) -> Option<String> {
    let path = dir.join(file.trim_end_matches('/'));
    if path.is_symlink() {
        return None;
    }
    let mut hits = Vec::new();
    for (name, is_file) in ceiling_src::read_dir_names(&path).ok()? {
        if !is_file || !name.ends_with(".yaml") || path.join(&name).is_symlink() {
            continue;
        }
        let doc = yaml::parse(&fs::read_to_string(path.join(&name)).ok()?).ok()?;
        if !doc.duplicates.is_empty() {
            return None;
        }
        if doc.root.get("meta").and_then(|m| m.get("id")).and_then(Node::as_str) == Some(head) {
            hits.push(format!("{file}{name}"));
        }
    }
    match hits.as_slice() {
        [one] => Some(one.clone()),
        _ => None,
    }
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
    #[expect(dead_code, reason = "読めるかだけ見て値は使わない")]
    pub at: String,
    pub bundle: String,
}

/// 面の天井の名札のために印から読む字（top-level の at と sources・観点の行を印の順に）。
pub struct Marks {
    pub at: String,
    /// 印の正本の要約値（名札が今の正本の要約値と比べる・便 127・delivery-127.md §1 (b) の 1）
    pub sources: String,
    pub rows: Vec<Mark>,
}

/// 面の天井の名札のために印を読む。印の file が無ければ None。symlink・読めない・parse できない・欄 at か sources が
/// 読めない・viewpoints が一覧でない・行の欄が読めないは Err（P-4.1）。
pub fn marks(dir: &Path) -> R<Option<Marks>> {
    let path = dir.join(STAMP_FILE);
    if path.is_symlink() {
        return Err(format!("{STAMP_FILE}: symlink は認めない"));
    }
    if !path.exists() {
        return Ok(None);
    }
    // 読めない・parse できないは床の面の段で まだ分からない に数える字（便 187）
    let text = fs::read_to_string(&path)
        .map_err(|e| cursor::unreadable(format!("{STAMP_FILE}: 読めない: {e}")))?;
    let root = yaml::parse(&text)
        .map_err(|e| cursor::unreadable(format!("{STAMP_FILE}: parse できない: {e}")))?
        .root;
    let field = |node: &Node, key: &str, at: &str| {
        node.get(key)
            .and_then(Node::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
            .ok_or_else(|| format!("{STAMP_FILE}: {at}{key} が読めない"))
    };
    let at = field(&root, "at", "")?;
    let sources = field(&root, "sources", "")?;
    let rows = root
        .get("viewpoints")
        .and_then(Node::as_seq)
        .ok_or_else(|| format!("{STAMP_FILE}: viewpoints が一覧でない"))?;
    let rows = rows
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
    Ok(Some(Marks { at, sources, rows }))
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

    /// 便 176（docs/design/delivery-176.md §1 (c) の 3）: meta.id で解くのは、読める .yaml のちょうど 1 つが持つときだけ。
    #[test]
    fn f176_the_meta_id_skips_links_and_bails_on_unreadable_files() {
        let td = std::env::temp_dir().join(format!("folio-f176-meta-{}", std::process::id()));
        let _ = fs::remove_dir_all(&td);
        let notes = td.join("design-note");
        fs::create_dir_all(&notes).unwrap();
        let put = |name: &str, bytes: &[u8]| fs::write(notes.join(name), bytes).unwrap();
        put("full.yaml", b"meta: {id: full}\n");
        put("schema.yaml", b"meta: {id: design-note-schema}\n");
        let documents = vec![("design-note".to_string(), "design-note/".to_string())];
        let at = |at: &str| stop_file(&td, &documents, "design-note", at).unwrap();
        let mut answers = vec![at("design-note-schema.schema.x"), at("full.sections.1"), at("nothing.x")];
        std::os::unix::fs::symlink(notes.join("schema.yaml"), notes.join("link.yaml")).unwrap();
        answers.push(at("design-note-schema.schema.x"));
        put("latin1.yaml", b"meta: {id: caf\xe9}\n");
        answers.push(at("design-note-schema.schema.x"));
        fs::remove_file(notes.join("latin1.yaml")).unwrap();
        put("dup.yaml", b"meta: {id: a}\nmeta: {id: b}\n");
        answers.push(at("design-note-schema.schema.x"));
        let _ = fs::remove_dir_all(&td);
        assert_eq!(
            answers,
            [
                "design-note/schema.yaml",
                "design-note/full.yaml",
                "design-note/",
                "design-note/schema.yaml",
                "design-note/",
                "design-note/"
            ]
        );
    }

    /// 便 176（docs/design/delivery-176.md §1 (c) の 3・検証役の提案）: meta.id は字のまま同じ file だけを数え（前方一致・大小文字の
    /// 違い・最上位の id は当たらない）、読めない .yaml は並びのどこに在っても広い側。名の規則が先で、頭が区切りを含むとき・
    /// `<頭>.yaml` が symlink のとき・置き場の dir が symlink のとき・file 形の文書では meta.id を見ない。
    #[test]
    fn f176_the_meta_id_matches_exactly_whatever_the_order() {
        let td = std::env::temp_dir().join(format!("folio-f176-exact-{}", std::process::id()));
        let _ = fs::remove_dir_all(&td);
        let notes = td.join("design-note");
        fs::create_dir_all(&notes).unwrap();
        let put = |name: &str, text: &str| fs::write(notes.join(name), text).unwrap();
        put("schema.yaml", "meta: {id: design-note-schema}\n");
        put("longer.yaml", "meta: {id: design-note-schema-v2}\n");
        put("design.yaml", "meta: {id: design-note}\n");
        put("upper.yaml", "meta: {id: DESIGN-NOTE-SCHEMA}\n");
        put("top.yaml", "id: design-note-schema\n");
        put("alias.yaml", "meta: {id: schema}\n");
        put("slash.yaml", "meta: {id: a/b}\n");
        fs::write(td.join("root.yaml"), "meta: {id: FR2}\n").unwrap();
        let documents = vec![
            ("design-note".to_string(), "design-note/".to_string()),
            ("srs".to_string(), "srs.yaml".to_string()),
            ("linked".to_string(), "linked/".to_string()),
        ];
        let at = |doc: &str, at: &str| stop_file(&td, &documents, doc, at).unwrap();
        let mut answers = vec![
            at("design-note", "design-note-schema.x"),
            at("design-note", "schema.x"),
            at("design-note", "a/b.x"),
            at("srs", "FR2.shall"),
        ];
        put("zz-broken.yaml", "meta: [\n");
        answers.push(at("design-note", "design-note-schema.x"));
        fs::remove_file(notes.join("zz-broken.yaml")).unwrap();
        std::os::unix::fs::symlink(&notes, td.join("linked")).unwrap();
        answers.push(at("linked", "design-note-schema.x"));
        std::os::unix::fs::symlink(notes.join("schema.yaml"), notes.join("via.yaml")).unwrap();
        put("target.yaml", "meta: {id: via}\n");
        answers.push(at("design-note", "via.x"));
        let _ = fs::remove_dir_all(&td);
        assert_eq!(
            answers,
            [
                "design-note/schema.yaml",
                "design-note/schema.yaml",
                "design-note/",
                "srs.yaml",
                "design-note/",
                "linked/",
                "design-note/"
            ]
        );
    }
}
