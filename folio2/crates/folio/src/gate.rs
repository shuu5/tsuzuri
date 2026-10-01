//! `folio ceiling --gate`（便 73・docs/design/delivery-73.md §1 (a)〜(c)・FR20 / AC18・設計ノート ceiling-gate.md §2）。
//! 天井の印 `<dir>/preview/ceiling-stamp.yaml`（便 72 の欄の決まり）と便の書き換える file の一覧（`--write-set`）から
//! 3 値を返す（式は下の便 169 の段）。
//! 設計文書の正本 = `<dir>` の下に在り、`<dir>/preview/` の下でなく、path のどの要素も retired でないもの。
//! 正本の要約値は観点の reads が指す文書の file（file 形はその file・dir 形は直下の .yaml）の全文を `<dir>` からの
//! 相対 path の byte 順に連結した sha256。印（`stamp.rs`）が欄 sources をこの関数で測り、面の名札が比べる（便 104・門は読まない）。
//! 何も書かない。標準出力は 1 行「folio ceiling: <3 値>（<理由>）」。
//!
//! 便 169（docs/design/delivery-169.md §1 (b) の 3・ADR-30 決定 (6)・FR20）: 門は印の古さで止めない。印が無い・読めない・
//! 観点の結果が欠けている・反証の済んでいない 止める の場所の file を書き換えるなら まだ分からない（2）、反証で支持された
//! 止める の場所の file を書き換えるなら 止める（1）、ほかは 通す（0・印の後の変更は審査していない）。印からは round・
//! verdict・viewpoints・refutes だけを読む（sources は読まない）。
//! 便 175（docs/design/delivery-175.md §1 (b)・ADR-30 決定 (5)(6)）: 周の引き金の要約値（`trigger_digest`）を外した。
//! 便 178（docs/design/delivery-178.md §1 (b)・FR20）: 置き場そのものか置き場を下に持つ dir（作業ツリーの一番上を含む）の
//! write-set の項目は、置き場の file（preview/ と retired を除く）を全部書き換える項目として読む。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::ceiling_src::{self, STAMP_FILE};
use crate::cursor::R;
use crate::sha256;
use crate::verdict::Verdict;
use crate::yaml::{self, Node};

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
/// 印の観点の行の欄 wait の値（便 169）: まだ分からない の理由が反証の済んでいない 止める だけ（印が書き、門が読む）。
pub(crate) const WAIT_REFUTE: &str = "反証";
/// 理由の字の頭（便 169 §1 (b) の 3）: 天井の正本の観点の行が印に無いか 3 値でない、または反証待ちでない まだ分からない。
const UNKNOWN_VIEWPOINTS: &str = "印の観点の結果が欠けている";
/// 理由の字の頭: 反証の済んでいない 止める の場所の file を書き換える。
const UNKNOWN_UNREFUTED: &str = "反証の済んでいない 止める の場所の file を書き換える";
/// 理由の字の頭: 反証で支持された 止める の場所の file を書き換える。
const FAIL_UPHELD: &str = "反証で支持された 止める の場所の file を書き換える";
/// 通す理由の字の末尾（P-3.3・通す は天井の合格ではない）。
const PASS_UNREVIEWED: &str = "書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない";

/// `write_set` の各 path は repo の根からの相対（接頭辞 + / - / ~ は剥がす）。`dir` は同じ根からの `--dir`。
/// 判定の順は便 169 §1 (b) の 3 のとおりで、最初に当たったもので決まる。
/// その前に、`--dir` と write-set を同じ根（今の dir）で照らせるかを確かめる（便 142・照らせなければ まだ分からない）。
pub(crate) fn run(dir: &Path, write_set: &[String]) -> Outcome {
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
    // 置き場を名指す項目（便 178）は置き場の file を全部指す: 置き場には印 PLACE_MARK が在るので正本に当たる
    let touches = |p: &str| is_design_source(&root, p) || names_the_place(&root, p);
    if !write_set.iter().any(|p| touches(p)) {
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
    // 観点の結果が欠けている（P-4.1 / P-4.2）: 反証待ちだけの まだ分からない は file ごとの判定に任せる
    let mut missing = Vec::new();
    for vp in &ceiling.viewpoints {
        match stamp.viewpoints.iter().find(|v| v.id == vp.id) {
            Some(v) if v.verdict == "合格" || v.verdict == "不合格" => {}
            Some(v) if v.verdict == "まだ分からない" => {
                if v.wait.as_deref() != Some(WAIT_REFUTE) {
                    missing.push(format!("{}（まだ分からない）", vp.id));
                }
            }
            _ => missing.push(format!("{}（無い）", vp.id)),
        }
    }
    if !missing.is_empty() {
        return Outcome::new(
            Verdict::Unknown,
            format!("{UNKNOWN_VIEWPOINTS}: {}", missing.join("・")),
        );
    }
    stop_verdict(dir, write_set, &root, &stamp, touches)
}

/// 書き換える file を場所とする 止める が印に在るかで決める（反証の済んでいないものが先・支持されたものが次・無ければ通す）。
fn stop_verdict(
    dir: &Path,
    write_set: &[String],
    root: &[String],
    stamp: &Stamp,
    touches: impl Fn(&str) -> bool,
) -> Outcome {
    // 書き換える file を場所とする 止める（2 が 1 より先・退けた は見ない）。置き場を名指す項目は空の列の dir（全部の下）
    let items: Vec<(Vec<&str>, bool)> = write_set
        .iter()
        .filter(|p| touches(p))
        .map(|p| {
            let p = p.trim_start_matches(['+', '-', '~']);
            if names_the_place(root, p) {
                return (Vec::new(), true);
            }
            (
                parts(p).into_iter().skip(root.len()).collect(),
                p.ends_with('/'),
            )
        })
        .collect();
    let hit = |s: &&StampStop| items.iter().any(|(rel, slash)| covers(dir, rel, *slash, &s.file));
    // 反証の済んでいない 止める = refute が無いか まだ分からない（値域の外も済んでいないと読む）
    let unrefuted = |s: &&StampStop| !matches!(s.refute.as_deref(), Some("支持" | "退けた"));
    if let Some(s) = stamp.stops.iter().filter(unrefuted).find(hit) {
        return Outcome::new(
            Verdict::Unknown,
            format!("{UNKNOWN_UNREFUTED}: {}（{} {}）", s.file, s.viewpoint, s.finding),
        );
    }
    let upheld = |s: &&StampStop| s.refute.as_deref() == Some("支持");
    if let Some(s) = stamp.stops.iter().filter(upheld).find(hit) {
        return Outcome::new(
            Verdict::Fail,
            format!("{FAIL_UPHELD}: {}（{} {}）", s.file, s.viewpoint, s.finding),
        );
    }
    Outcome::new(
        Verdict::Pass,
        format!("印の周 {}（判定 {}）に、{PASS_UNREVIEWED}", stamp.round, stamp.verdict),
    )
}

/// write-set の設計文書の path（`--dir` の要素を外した要素の列）が 止める の場所の file に当たるか（便 169 §1 (b) の 3）:
/// 同じ file・場所が dir 形でその下に在る・write-set の項目が dir（末尾が / か `--dir` の下の dir）で場所がその下に在る。
fn covers(dir: &Path, rel: &[&str], slash: bool, file: &str) -> bool {
    let place = parts(file);
    let under = |long: &[&str], short: &[&str]| long.starts_with(short);
    if rel == place.as_slice() || (file.ends_with('/') && under(rel, &place)) {
        return true;
    }
    (slash || dir.join(rel.join("/")).is_dir()) && under(&place, rel)
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
    !under(root)
        && (1..root.len())
            .filter_map(|k| root.get(k..))
            .any(under)
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
    parts.get(root.len()).is_some_and(|first| *first != "preview") && !parts.contains(&"retired")
}

/// 置き場そのものか置き場を下に持つ dir を名指すか（便 178 §1 (b) の 1）: 要素の列（頭の `./`・末尾の `/` は落ちる）が
/// `--dir` の要素の列そのものか、その前方の部分列（作業ツリーの一番上 `.` は空の列）。
fn names_the_place(root: &[String], path: &str) -> bool {
    let parts = parts(path.trim_start_matches(['+', '-', '~']));
    parts.len() <= root.len() && parts.iter().zip(root).all(|(a, b)| *a == b)
}

// ── 印の判定（§1 (c)）──

/// 印から読む欄（便 169: round・verdict・観点の行・止める の行だけ）。
struct Stamp {
    round: String,
    verdict: String,
    viewpoints: Vec<StampViewpoint>,
    stops: Vec<StampStop>,
}

/// 印の観点の行（id・3 値の字・欄 wait〔無ければ None〕）。
struct StampViewpoint {
    id: String,
    verdict: String,
    wait: Option<String>,
}

/// 印の refutes の行（観点・所見・反証の結果〔済んでいなければ None〕・場所の file）。
struct StampStop {
    viewpoint: String,
    finding: String,
    refute: Option<String>,
    file: String,
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
    let text = |node: &Node, key: &str| {
        node.get(key)
            .and_then(Node::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
    };
    let round = text(&root, "round").ok_or_else(|| "round が読めない".to_string())?;
    let verdict = text(&root, "verdict").ok_or_else(|| "verdict が読めない".to_string())?;
    let viewpoints = root
        .get("viewpoints")
        .and_then(Node::as_seq)
        .and_then(|rows| {
            rows.iter()
                .map(|row| {
                    Some(StampViewpoint {
                        id: text(row, "id")?,
                        verdict: text(row, "verdict")?,
                        wait: row.get("wait").map(|w| w.as_str().unwrap_or_default().to_string()),
                    })
                })
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| "viewpoints が読めない".to_string())?;
    let stops = root
        .get("refutes")
        .and_then(Node::as_seq)
        .and_then(|rows| {
            rows.iter()
                .map(|row| {
                    let refute = match row.get("refute") {
                        None => None,
                        Some(r) => Some(r.as_str()?.to_string()),
                    };
                    Some(StampStop {
                        viewpoint: text(row, "viewpoint")?,
                        finding: text(row, "finding")?,
                        refute,
                        file: text(row, "file")?,
                    })
                })
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| "refutes が読めない".to_string())?;
    Ok(Some(Stamp {
        round,
        verdict,
        viewpoints,
        stops,
    }))
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
        f142_a_top_root_is_not_another_root();
    }

    /// 根が 1 要素 `design-intent` のときの `other_root`（上の歯の続き）。
    fn f142_a_top_root_is_not_another_root() {
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

    #[test]
    fn f178_an_item_names_the_place_or_a_dir_above_it() {
        let top = vec!["design-intent".to_string()];
        let names = |p: &str| names_the_place(&top, p);
        let place = [".", "./", "", "design-intent", "design-intent/", "./design-intent"];
        for p in place.into_iter().chain(["+design-intent", "~./design-intent/", "-."]) {
            assert!(names(p), "{p}");
        }
        for p in ["design-intent/srs.yaml", "design-intent/adr/", "design", "design-intent-x", "crates", "docs/"] {
            assert!(!names(p), "{p}");
        }
        let deep: Vec<String> = [".worktrees", "x", "design-intent"].map(String::from).to_vec();
        let names = |p: &str| names_the_place(&deep, p);
        for p in [".", ".worktrees", ".worktrees/x/", "./.worktrees/x/design-intent"] {
            assert!(names(p), "{p}");
        }
        for p in ["x", "design-intent", "x/design-intent", ".worktrees/y", ".worktrees/x/design-intent/srs.yaml"] {
            assert!(!names(p), "{p}");
        }
        assert!(names_the_place(&[], "."));
        assert!(!names_the_place(&[], "srs.yaml"));
        // 置き場を名指す項目（空の列の dir）は、場所がどこに在っても覆う
        let dir = Path::new("design-intent");
        for file in ["srs.yaml", "adr/ADR-1.yaml", "design-note/"] {
            assert!(covers(dir, &[], true, file), "{file}");
        }
    }
}
