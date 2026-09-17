//! `folio check` の凍結 anchor の列（`anchors/`）の検査のうち版管理（git）を見ない部分（便 7・docs/design/delivery-7.md §1）。
//! day-1 の床 `scripts/check_draft.py` の anchor の節と同じ式で写す: 現行の写し（(e)）・anchor の file と索引（(f)）・
//! 列（(g)）・版と記録（(h)）・現行との一致（(i)）。版管理との照合・列の区間の amends の消し込み・`--freeze-anchor`・
//! `--emit-amends` は便 8。値は型付きの木（`yaml::Value`）で読み、digest は正規化（`yaml::canonical`）の sha256。
//! 床の定数は `adr.rs` の `FLOOR` を読み口（`adr::floor_strs` / `adr::floor_val`）で読み、値は持ち直さない。正規表現は使わない。

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::Path;

use crate::adr::{self, Adr};
use crate::sha256;
use crate::verdict::Report;
use crate::yaml::{self, Node, Value};

/// 凍結 anchor の file 名の頭と尻（`constitution-*.yaml`）。
const ANCHOR_PREFIX: &str = "constitution-";
const ANCHOR_SUFFIX: &str = ".yaml";
const ANCHOR_KIND: &str = "constitution-anchor";
const INDEX_KIND: &str = "constitution-anchor-index";

/// 床の定数の値（道は FLOOR に在る＝無ければ空の字面）。
fn floor(path: &[&str]) -> &'static str {
    adr::floor_val(path).unwrap_or_default()
}

/// 読めた anchor（版・木・file 名）。
struct Anchor {
    version: String,
    doc: Value,
    name: String,
}

/// 凍結 anchor の列に在った条・規範文の id（(j)・便 1 の解決先へ足す）。
/// `<dir>/anchors/` の直下の `constitution-*.yaml` を名前順に読み、読めない file は寄与しない。
pub fn history_ids(dir: &Path) -> HashSet<String> {
    let mut ids = HashSet::new();
    let anchors = dir.join(floor(&["anchor", "dir"]));
    if !anchors.is_dir() {
        return ids;
    }
    for name in anchor_names(&anchors).unwrap_or_default() {
        let Ok(text) = fs::read_to_string(anchors.join(&name)) else {
            continue;
        };
        // 床の読み手は重複キーを読めないとする
        let Ok(doc) = yaml::parse(&text) else {
            continue;
        };
        if !doc.duplicates.is_empty() {
            continue;
        }
        let Some(content @ Node::Map(_)) = doc.root.get("content") else {
            continue;
        };
        for a in node_rows(content, "articles") {
            ids.insert(node_str(a.get("id")));
            for st in node_rows(a, "statements") {
                ids.insert(node_str(st.get("id")));
            }
        }
    }
    ids
}

fn node_rows<'a>(root: &'a Node, section: &str) -> impl Iterator<Item = &'a Node> {
    root.get(section)
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .filter(|n| n.as_map().is_some())
}

/// 床の `str(x)` の写し（Node の側・無い・null は None）。
fn node_str(node: Option<&Node>) -> String {
    match node {
        Some(Node::Scalar(s)) => s.clone(),
        Some(Node::Null) | None => "None".to_string(),
        Some(Node::Seq(_)) => "（一覧）".to_string(),
        Some(Node::Map(_)) => "（表）".to_string(),
    }
}

fn py_str(value: Option<&Value>) -> String {
    value.map_or_else(|| "None".to_string(), Value::py_str)
}

/// 床の `x is None` の写し（無い・null）。
fn is_none(value: Option<&Value>) -> bool {
    matches!(value, None | Some(Value::Null))
}

/// `anchors/` の直下の `constitution-*.yaml` の名前（名前順）。
fn anchor_names(anchors: &Path) -> std::io::Result<Vec<String>> {
    let mut names: Vec<String> = fs::read_dir(anchors)?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| {
            n.len() >= ANCHOR_PREFIX.len() + ANCHOR_SUFFIX.len()
                && n.starts_with(ANCHOR_PREFIX)
                && n.ends_with(ANCHOR_SUFFIX)
        })
        .collect();
    names.sort();
    Ok(names)
}

/// file を型付きで読む。読めない・重複キー（床の読み手は読めない）・この便で解かない scalar は「まだ分からない」。
fn read_typed(path: &Path, file: &str, report: &mut Report) -> Option<Value> {
    if !path.is_file() {
        report.unknown(format!("{file}: file でない"));
        return None;
    }
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            report.unknown(format!("{file}: 読めない: {e}"));
            return None;
        }
    };
    match yaml::parse(&text) {
        Ok(doc) if !doc.duplicates.is_empty() => {
            report.unknown(format!(
                "{file}: parse できない: 同じ表にキー「{}」を 2 度書いている",
                doc.duplicates[0].key
            ));
            return None;
        }
        Ok(_) => {}
        Err(e) => {
            report.unknown(format!("{file}: parse できない: {e}"));
            return None;
        }
    }
    match yaml::parse_typed(&text) {
        Ok(v) => Some(v),
        Err(e) => {
            report.unknown(format!("{file}: まだ分からない（{e}）"));
            None
        }
    }
}

/// symlink と design-intent の外の実体を拒む（1 違反ずつ・adr）。
fn real_under(root: Option<&Path>, path: &Path, what: &str, report: &mut Report) -> bool {
    let mut ok = true;
    if path.is_symlink() {
        report.violation("adr", format!("{what}: symlink は認めない"));
        ok = false;
    }
    let inside = match (root, fs::canonicalize(path)) {
        (Some(r), Ok(p)) => p.starts_with(r),
        _ => false,
    };
    if !inside {
        report.violation("adr", format!("{what}: design-intent の外を指している"));
        ok = false;
    }
    ok
}

/// 正規化の字面。正規化できなければ「まだ分からない」を立てて None。
fn canon(value: &Value, what: &str, report: &mut Report) -> Option<String> {
    match yaml::canonical(value) {
        Ok(s) => Some(s),
        Err(e) => {
            report.unknown(format!("{what}: {e}"));
            None
        }
    }
}

/// digest 欄を除く全欄の正規化の sha256。
fn digest_of(doc: &Value) -> Result<String, String> {
    let body: Vec<(Value, Value)> = doc
        .as_map()
        .unwrap_or_default()
        .iter()
        .filter(|(k, _)| k.as_str() != Some("digest"))
        .cloned()
        .collect();
    yaml::canonical(&Value::Map(body)).map(|s| sha256::hex(s.as_bytes()))
}

/// v + 数字列 + 「.」+ 数字列（version_pattern）。
fn is_version(s: &str) -> bool {
    let digits = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
    s.strip_prefix('v')
        .and_then(|r| r.split_once('.'))
        .is_some_and(|(a, b)| digits(a) && digits(b))
}

/// 一覧の各項が文字列で、並びが `want` と同じか（床の `list(x or []) == want`）。
fn str_list_eq(value: Option<&Value>, want: &[&str]) -> bool {
    let items: &[Value] = match value {
        None | Some(Value::Null) => &[],
        Some(Value::Seq(items)) => items,
        Some(_) => return false,
    };
    items.len() == want.len() && items.iter().zip(want).all(|(v, w)| v.as_str() == Some(w))
}

/// 一覧の各項の `str(x)`（一覧でなければ空）。
fn str_items(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_seq)
        .unwrap_or_default()
        .iter()
        .map(Value::py_str)
        .collect()
}

/// 床の `str(x if x is not None else '').strip()` が空でない。
fn non_empty(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Str(s)) => !s.trim().is_empty(),
        Some(v) => !v.py_str().trim().is_empty(),
    }
}

fn value_rows<'a>(root: Option<&'a Value>, section: &str) -> impl Iterator<Item = &'a Value> {
    root.and_then(|r| r.get(section))
        .and_then(Value::as_seq)
        .unwrap_or_default()
        .iter()
        .filter(|n| n.as_map().is_some())
}

/// 条 id と規範文 id（表の行だけ）。
fn article_ids(root: Option<&Value>) -> (Vec<String>, Vec<(String, Option<&Value>)>) {
    let mut articles = Vec::new();
    let mut statements = Vec::new();
    for a in value_rows(root, "articles") {
        articles.push(py_str(a.get("id")));
        for st in value_rows(Some(a), "statements") {
            statements.push((py_str(st.get("id")), st.get("text")));
        }
    }
    (articles, statements)
}

/// (e) 現行の写し。範囲の各節（articles 以外はその木）と、条の 5 欄・規範文の 4 欄。
fn project(c: &Value, scope: &[String]) -> Result<Value, String> {
    let article_fields = adr::floor_strs(&["anchor", "projection_article_fields"]);
    let statement_fields = adr::floor_strs(&["anchor", "statement_fields"]);
    let get = |node: &Value, k: &str| node.get(k).cloned().unwrap_or(Value::Null);
    let mut out: Vec<(Value, Value)> = Vec::new();
    let push = |out: &mut Vec<(Value, Value)>, k: &str, v: Value| {
        if !out.iter().any(|(key, _)| key.as_str() == Some(k)) {
            out.push((Value::Str(k.to_string()), v));
        }
    };
    for k in scope.iter().filter(|k| *k != "articles") {
        push(&mut out, k, get(c, k));
    }
    if scope.iter().any(|k| k == "articles") {
        let articles: &[Value] = match c.get("articles") {
            None | Some(Value::Null) => &[],
            Some(Value::Seq(items)) => items,
            Some(_) => return Err("articles が一覧でない".to_string()),
        };
        let mut projected = Vec::new();
        for a in articles {
            if a.as_map().is_none() {
                return Err("articles に表でない行がある".to_string());
            }
            let mut fields = Vec::new();
            for f in article_fields {
                let v = if *f == "statements" {
                    let sts: &[Value] = match a.get(f) {
                        None | Some(Value::Null) => &[],
                        Some(Value::Seq(items)) => items,
                        Some(_) => {
                            return Err(format!(
                                "条 {} の statements が一覧でない",
                                py_str(a.get("id"))
                            ));
                        }
                    };
                    let mut rows = Vec::new();
                    for st in sts {
                        if st.as_map().is_none() {
                            return Err(format!(
                                "条 {} の statements に表でない行がある",
                                py_str(a.get("id"))
                            ));
                        }
                        rows.push(Value::Map(
                            statement_fields
                                .iter()
                                .map(|sf| (Value::Str(sf.to_string()), get(st, sf)))
                                .collect(),
                        ));
                    }
                    Value::Seq(rows)
                } else {
                    get(a, f)
                };
                fields.push((Value::Str(f.to_string()), v));
            }
            projected.push(Value::Map(fields));
        }
        push(&mut out, "articles", Value::Seq(projected));
    }
    Ok(Value::Map(out))
}

/// (e) 印を値として持つ欄（A-2）。
fn marks_in(value: &Value, at: &str, marks: &[&str], report: &mut Report) {
    match value {
        Value::Str(s) if marks.contains(&s.as_str()) => report.violation(
            "A-2",
            format!("{at}: 印「{s}」を値として持つ欄（記録の読みと実体が食い違う）"),
        ),
        Value::Seq(items) => {
            for (i, v) in items.iter().enumerate() {
                marks_in(v, &format!("{at}[{i}]"), marks, report);
            }
        }
        Value::Map(entries) => {
            for (k, v) in entries {
                marks_in(v, &format!("{at}.{}", k.py_str()), marks, report);
            }
        }
        _ => {}
    }
}

/// (e) 欄名が文字列でないか「.」を含む欄（A-2）。
fn keys_in(value: &Value, at: &str, report: &mut Report) {
    match value {
        Value::Seq(items) => {
            for (i, v) in items.iter().enumerate() {
                keys_in(v, &format!("{at}[{i}]"), report);
            }
        }
        Value::Map(entries) => {
            for (k, v) in entries {
                let name = k.py_str();
                if !matches!(k, Value::Str(s) if !s.contains('.')) {
                    report.violation(
                        "A-2",
                        format!(
                            "{at}: 欄名「{name}」が文字列でないか「.」を含む（欄の道が衝突する）"
                        ),
                    );
                }
                keys_in(v, &format!("{at}.{name}"), report);
            }
        }
        _ => {}
    }
}

/// 憲法の schema.amendment_scope の各項の `str(x)`。
fn amendment_scope(c: &Value) -> Vec<String> {
    str_items(c.get("schema").and_then(|s| s.get("amendment_scope")))
}

/// 発効した判断（status が effective_status で approval が表）。
fn is_effective(d: &Node) -> bool {
    adr::in_enum(d.get("status"), adr::floor_strs(&["effective_status"]))
        && matches!(d.get("approval"), Some(Node::Map(_)))
}

fn amends_list(d: &Node) -> impl Iterator<Item = &Node> {
    d.get("amends")
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .filter(|e| e.as_map().is_some())
}

/// (e)〜(i) を掛ける。`history` は (j) の列に在った id（`history_ids`）。
pub fn check_anchor(dir: &Path, adr: &Adr, history: &HashSet<String>, report: &mut Report) {
    let Some(c) = read_typed(&dir.join("constitution.yaml"), "constitution.yaml", report) else {
        return;
    };
    let scope = amendment_scope(&c);
    let cur_proj = match project(&c, &scope) {
        Ok(p) => p,
        Err(e) => {
            report.unknown(format!("constitution.yaml: 現行の写しを取れない: {e}"));
            return;
        }
    };
    let marks = [
        floor(&["amends_entry", "new_article_marker"]),
        floor(&["amends_entry", "deleted_marker"]),
        floor(&["amends_entry", "empty_marker"]),
    ];
    marks_in(&cur_proj, "憲法の写し", &marks, report);
    keys_in(&cur_proj, "憲法の写し", report);
    let meta = c.get("meta");
    let cur_ver = py_str(meta.and_then(|m| m.get("version")));
    if !is_version(&cur_ver) {
        report.violation(
            "A-2",
            format!("憲法 meta.version「{cur_ver}」の綴りが v<数>.<数> でない"),
        );
    }
    let meta_approval = meta
        .and_then(|m| m.get("approval"))
        .cloned()
        .unwrap_or(Value::Null);

    let first_ver = floor(&["anchor", "first_version"]);
    let root_digest = floor(&["anchor", "root_digest"]);
    let anch = dir.join(floor(&["anchor", "dir"]));
    let index_file = floor(&["anchor", "index_file"]);
    let root = fs::canonicalize(dir).ok();
    let mut index: Option<Value> = None;
    let mut anchors: Vec<Anchor> = Vec::new();
    if anch.exists() || anch.is_symlink() {
        if anch.is_symlink() || !anch.is_dir() {
            report.unknown(format!(
                "anchors/ が dir でない（symlink か file）: {}",
                anch.display()
            ));
            return;
        }
        let index_path = anch.join(index_file);
        if index_path.exists()
            && real_under(root.as_deref(), &index_path, index_file, report)
            && let Some(ix) = read_typed(&index_path, &format!("anchors/{index_file}"), report)
        {
            if ix.get("kind").and_then(Value::as_str) == Some(INDEX_KIND)
                && matches!(ix.get("entries"), Some(Value::Seq(_)))
            {
                index = Some(ix);
            } else {
                report.violation("anchor", format!("{index_file}: 索引の形が違う"));
            }
        }
        let names = match anchor_names(&anch) {
            Ok(n) => n,
            Err(e) => {
                report.unknown(format!("anchors/: 読めない: {e}"));
                return;
            }
        };
        for name in names {
            let path = anch.join(&name);
            if !real_under(root.as_deref(), &path, &name, report) {
                continue;
            }
            let Some(d) = read_typed(&path, &format!("anchors/{name}"), report) else {
                continue;
            };
            if let Some(a) = check_file(dir, &name, d, &meta_approval, adr, &anchors, report) {
                anchors.push(a);
            }
        }
    }
    let find = |v: &str| anchors.iter().find(|a| a.version == v);

    // (g) 列
    let records_exist = value_rows(Some(&c), "articles").any(|a| {
        a.get("amended_by")
            .and_then(Value::as_seq)
            .is_some_and(|s| !s.is_empty())
    }) || adr
        .records
        .iter()
        .any(|(_, d)| is_effective(d) && amends_list(d).next().is_some());
    let mut newest: Option<String> = None;
    let mut chain_versions: Vec<String> = Vec::new();
    if let Some(ix) = &index {
        let entries = ix
            .get("entries")
            .and_then(Value::as_seq)
            .unwrap_or_default();
        if entries.is_empty() {
            report.pending(format!(
                "{index_file}: 索引はあるが entries が空＝比較元が立たない（まだ分からない・P-10.3）。索引と anchor は消さない・空にしない"
            ));
        }
        for (n, e) in entries.iter().enumerate() {
            if e.as_map().is_none()
                || ["version", "previous", "digest"]
                    .iter()
                    .any(|k| e.get(k).is_none())
            {
                report.violation(
                    "anchor",
                    format!("{index_file}: entries[{n}] の欄が壊れている"),
                );
                continue;
            }
            let vs = py_str(e.get("version"));
            let expect_prev = (n > 0).then(|| py_str(entries[n - 1].get("version")));
            if n == 0 && vs != first_ver {
                report.violation(
                    "anchor",
                    format!("{index_file}: 列の根 {vs} が最初の版 {first_ver}（床の定数）でない"),
                );
            }
            if n == 0 && py_str(e.get("digest")) != root_digest {
                report.violation(
                    "anchor",
                    format!(
                        "{index_file}: 列の根 {vs} の digest が床の定数と違う（根は 1 度きり・別の中身で凍結し直せない。変えるのは移行＝床の外の手順）"
                    ),
                );
            }
            let prev = (!is_none(e.get("previous"))).then(|| py_str(e.get("previous")));
            if prev != expect_prev {
                report.violation(
                    "anchor",
                    format!(
                        "{index_file}: entries[{n}]（{vs}）の previous {} が直前の版 {} でない（列の付け替え）",
                        py_str(e.get("previous")),
                        expect_prev.as_deref().unwrap_or("None")
                    ),
                );
            }
            let Some(a) = find(&vs) else {
                report.pending(format!(
                    "anchor の列が切れている: 索引にある版 {vs} の anchor file が無いか読めない＝差分検査は「まだ分からない」（P-10.3）。anchors/ は消さない"
                ));
                continue;
            };
            if py_str(a.doc.get("digest")) != py_str(e.get("digest")) {
                report.violation(
                    "anchor",
                    format!("{vs}: anchor の digest が索引の記載と違う（差し替えられた）"),
                );
            }
            let a_prev = (!is_none(a.doc.get("previous"))).then(|| py_str(a.doc.get("previous")));
            if a_prev != expect_prev {
                report.violation(
                    "anchor",
                    format!(
                        "{vs}: anchor の previous {} が索引の列 {} と違う",
                        py_str(a.doc.get("previous")),
                        expect_prev.as_deref().unwrap_or("None")
                    ),
                );
            }
        }
        chain_versions = entries
            .iter()
            .filter(|e| e.as_map().is_some())
            .map(|e| py_str(e.get("version")))
            .collect();
        let mut extra: Vec<&str> = anchors
            .iter()
            .map(|a| a.version.as_str())
            .filter(|v| !chain_versions.iter().any(|c| c == v))
            .collect();
        extra.sort_unstable();
        for v in extra {
            report.violation(
                "anchor",
                format!("anchor {v} が索引に無い（列の外の anchor）"),
            );
        }
        if let Some(last) = entries.last()
            && last.as_map().is_some()
        {
            newest = Some(py_str(last.get("version")));
        }
    } else if !anchors.is_empty() {
        report.violation(
            "anchor",
            format!("anchor file はあるが索引（{index_file}）が無い（消された）"),
        );
    } else if records_exist {
        report.violation(
            "N-4",
            "改訂の記録（amended_by か発効した判断の amends）があるのに anchor が 1 本も無い＝anchor が消された（列の始め直しは認めない）",
        );
    }

    // (h) 版と記録
    let root_ver = chain_versions.first().map_or(first_ver, String::as_str);
    let mut nameable: HashSet<&str> = chain_versions.iter().skip(1).map(String::as_str).collect();
    if cur_ver != root_ver {
        nameable.insert(&cur_ver);
    }
    for (aid, d) in adr.records.iter().filter(|(_, d)| is_effective(d)) {
        let versions: BTreeSet<String> =
            amends_list(d).map(|e| node_str(e.get("version"))).collect();
        for v in versions.iter().filter(|v| !nameable.contains(v.as_str())) {
            if v == root_ver {
                report.violation(
                    "A-2",
                    format!(
                        "{aid}: amends が列の根の版 {root_ver} を名指している（根の版には改訂前が無い＝突き合わせる差分が存在しない架空の記録）"
                    ),
                );
            } else {
                report.violation(
                    "anchor",
                    format!(
                        "{aid}: 発効した判断の記録が名指す版 {v} の anchor が列に無い（最新版の anchor と索引の項を消して前の版へ戻した・列の頭は記録が名指す版まで固定）"
                    ),
                );
            }
        }
    }
    let newest_anchor = newest.as_deref().and_then(find);
    let (cur_articles, cur_statements) = article_ids(Some(&c));
    if let (Some(v), Some(na)) = (&newest, newest_anchor) {
        let (n_articles, n_statements) = article_ids(na.doc.get("content"));
        let newest_ids: HashSet<&str> = n_articles
            .iter()
            .map(String::as_str)
            .chain(n_statements.iter().map(|(s, _)| s.as_str()))
            .collect();
        let reused: BTreeSet<&str> = cur_articles
            .iter()
            .map(String::as_str)
            .chain(cur_statements.iter().map(|(s, _)| s.as_str()))
            .filter(|i| history.contains(*i) && !newest_ids.contains(i))
            .collect();
        for i in reused {
            report.violation(
                "P-7",
                format!(
                    "{i}: 過去の版の anchor に在って最新 anchor {v} に無い番号の再利用（廃止した番号は空けたまま・P-7.1）"
                ),
            );
        }
    }

    // (i) 現行との一致
    if index.is_none() && anchors.is_empty() && !records_exist {
        report.pending(format!(
            "凍結 anchor が 0 本（{}）＝A-2 / N-4 の差分検査は「まだ分からない」（P-10.3）。発効版で --freeze-anchor を実行する",
            anch.display()
        ));
    } else if let (Some(v), Some(na)) = (&newest, newest_anchor) {
        if *v != cur_ver {
            report.violation(
                "A-2",
                format!(
                    "憲法の版 {cur_ver} と最新 anchor の版 {v} が違う＝版を上げたのに凍結していない（--freeze-anchor）か、版を上げずに直した。凍結するまで記録の消し込みは測らない"
                ),
            );
            structural_diff(na, &cur_articles, &cur_statements, report);
        } else if let (Some(a), Some(b)) = (
            na.doc
                .get("content")
                .and_then(|x| canon(x, &na.name, report)),
            canon(&cur_proj, "憲法の写し", report),
        ) && a != b
        {
            report.violation(
                "N-4",
                format!(
                    "現行の条文（{} の写し）が凍結 anchor {} と一致しない＝判断の記録と承認を伴わない改憲（N-4.1）",
                    scope.join("・"),
                    na.name
                ),
            );
        }
    }
}

/// (i) 版を上げて未凍結の間も測る: 条の消失と規範文の改番（P-7）。
fn structural_diff(
    newest: &Anchor,
    cur_articles: &[String],
    cur_statements: &[(String, Option<&Value>)],
    report: &mut Report,
) {
    let (prev_articles, prev_statements) = article_ids(newest.doc.get("content"));
    let lost: BTreeSet<&String> = prev_articles
        .iter()
        .filter(|i| !cur_articles.contains(i))
        .collect();
    for i in lost {
        report.violation(
            "P-7",
            format!(
                "条 {i} が anchor {} に在って現行に無い（現行・番号は消さない。条の廃止の機構は day-1 に無い＝M0 で決める）",
                newest.version
            ),
        );
    }
    let text = |t: Option<&Value>| yaml::canonical(t.unwrap_or(&Value::Null)).unwrap_or_else(|e| e);
    let only = |a: &[(String, Option<&Value>)], b: &[(String, Option<&Value>)]| {
        a.iter()
            .filter(|(id, _)| !b.iter().any(|(x, _)| x == id))
            .map(|(id, t)| (text(*t), id.clone()))
            .collect::<BTreeMap<String, String>>()
    };
    let deleted = only(&prev_statements, cur_statements);
    let added = only(cur_statements, &prev_statements);
    for (t, del) in &deleted {
        if let Some(add) = added.get(t) {
            report.violation(
                "P-7",
                format!(
                    "規範文の改番: {del} を消して同じ本文を {add} として足している（番号は付け替えない・P-7.1）"
                ),
            );
        }
    }
}

/// (f) anchor の file 1 本。列に載せられる形なら返す。
fn check_file(
    dir: &Path,
    name: &str,
    d: Value,
    meta_approval: &Value,
    adr: &Adr,
    seen: &[Anchor],
    report: &mut Report,
) -> Option<Anchor> {
    let file_keys = adr::floor_strs(&["anchor", "file_keys"]);
    let shape_ok = d.as_map().is_some_and(|m| {
        m.len() == file_keys.len() && file_keys.iter().all(|k| d.get(k).is_some())
    }) && d.get("kind").and_then(Value::as_str) == Some(ANCHOR_KIND);
    if !shape_ok {
        let mut keys: Vec<String> = d
            .as_map()
            .unwrap_or_default()
            .iter()
            .map(|(k, _)| k.py_str())
            .collect();
        keys.sort();
        report.violation(
            "anchor",
            format!(
                "{name}: anchor の欄が壊れている（固定の欄 [{}]）: [{}]",
                file_keys.join(", "),
                keys.join(", ")
            ),
        );
        return None;
    }
    let vs = py_str(d.get("version"));
    let expected_name = floor(&["anchor", "file_name"]).replace("<version>", &vs);
    if name != expected_name {
        report.violation("anchor", format!("{name}: file 名が版 {vs} と違う"));
    }
    if seen.iter().any(|a| a.version == vs) {
        report.violation("anchor", format!("{name}: 版 {vs} の anchor が重複"));
        return None;
    }
    if !is_version(&vs) {
        report.violation(
            "anchor",
            format!("{name}: 版の綴り「{vs}」が v<数>.<数> でない"),
        );
    }
    let algo = floor(&["anchor", "digest_algo"]);
    if d.get("digest_algo").and_then(Value::as_str) != Some(algo) {
        report.pending(format!(
            "{name}: digest の方式 {} が床の {algo} と違う＝digest を照合できない（まだ分からない・写しの内容の照合は行う）",
            py_str(d.get("digest_algo"))
        ));
    } else {
        match digest_of(&d) {
            Ok(h) if h != py_str(d.get("digest")) => report.violation(
                "anchor",
                format!("{name}: digest が中身と一致しない（anchor のどこかが手で変えられた）"),
            ),
            Ok(_) => {}
            Err(e) => report.unknown(format!("anchors/{name}: {e}")),
        }
    }
    let empty = Value::Map(Vec::new());
    let pj = match d.get("projection") {
        Some(p @ Value::Map(_)) => p,
        _ => &empty,
    };
    let article_fields = adr::floor_strs(&["anchor", "projection_article_fields"]);
    let statement_fields = adr::floor_strs(&["anchor", "statement_fields"]);
    if !str_list_eq(pj.get("article_fields"), article_fields)
        || !str_list_eq(pj.get("statement_fields"), statement_fields)
    {
        report.pending(format!(
            "{name}: anchor の写しの取り方（条 {}・規範文 {}）が床の取り方（条 [{}]・規範文 [{}]）と違う＝比べられない（まだ分からない・床の取り方を変えたなら移行の手順が要る）",
            py_str(pj.get("article_fields")),
            py_str(pj.get("statement_fields")),
            article_fields.join(", "),
            statement_fields.join(", ")
        ));
        return None;
    }
    let a_scope = str_items(pj.get("scope"));
    let minimum = adr::floor_strs(&["anchor", "scope_minimum"]);
    if !minimum.iter().all(|m| a_scope.iter().any(|s| s == m)) {
        report.violation(
            "anchor",
            format!(
                "{name}: anchor の範囲 [{}] が A-2.2 の下限 [{}] を含まない",
                a_scope.join(", "),
                minimum.join(", ")
            ),
        );
    }
    let content_keys: Option<BTreeSet<String>> = d
        .get("content")
        .and_then(Value::as_map)
        .map(|m| m.iter().map(|(k, _)| k.py_str()).collect());
    let scope_set: BTreeSet<String> = a_scope.iter().cloned().collect();
    if content_keys.as_ref() != Some(&scope_set) {
        report.violation(
            "anchor",
            format!(
                "{name}: 写しの節 [{}] が範囲 [{}] と一致しない",
                content_keys.map_or_else(
                    || "?".to_string(),
                    |k| k.into_iter().collect::<Vec<_>>().join(", ")
                ),
                a_scope.join(", ")
            ),
        );
    }
    if let (Some(a), Some(b)) = (
        canon(
            d.get("meta_approval").unwrap_or(&Value::Null),
            &format!("anchors/{name}"),
            report,
        ),
        canon(meta_approval, "constitution.yaml meta.approval", report),
    ) && a != b
    {
        report.violation(
            "N-4",
            format!(
                "{name}: anchor の meta_approval（発効の承認の写し）が憲法 meta.approval と一致しない（承認の記録が書き換えられた・P-12.2）"
            ),
        );
    }
    check_approvals(dir, name, d.get("approvals"), adr, report);
    Some(Anchor {
        version: vs,
        doc: d,
        name: name.to_string(),
    })
}

/// (f) approvals の各項と判断の記録の承認欄。
fn check_approvals(
    dir: &Path,
    name: &str,
    approvals: Option<&Value>,
    adr: &Adr,
    report: &mut Report,
) {
    let items = match approvals {
        Some(Value::Seq(items)) if !items.is_empty() => items,
        _ => {
            report.violation("anchor", format!("{name}: 承認の一覧（approvals）が空"));
            return;
        }
    };
    let approval_fields = adr::floor_strs(&["approval", "required"]);
    for (n, ap) in items.iter().enumerate() {
        if ap.as_map().is_none()
            || !["who", "ruling", "verbatim"]
                .iter()
                .all(|k| non_empty(ap.get(k)))
        {
            report.violation(
                "anchor",
                format!("{name}: approvals[{n}] に who / ruling / verbatim が無い"),
            );
            continue;
        }
        if !adr::has_ledger_id(&py_str(ap.get("ruling"))) {
            report.violation(
                "anchor",
                format!("{name}: approvals[{n}].ruling に台帳 id が無い"),
            );
            continue;
        }
        if is_none(ap.get("adr")) {
            continue;
        }
        let aid = py_str(ap.get("adr"));
        if !adr.records.iter().any(|(id, _)| *id == aid) {
            report.violation(
                "N-4",
                format!(
                    "{name}: approvals[{n}].adr {aid} の判断の記録が実在しない（凍結時の承認の相手が消えた）"
                ),
            );
            continue;
        }
        let file = format!("adr/{aid}.yaml");
        let Some(record) = read_typed(&dir.join(&file), &file, report) else {
            continue;
        };
        let same = match record.get("approval") {
            Some(ra @ Value::Map(_)) => {
                let mut same = true;
                for f in approval_fields {
                    let (Some(x), Some(y)) = (
                        canon(ra.get(f).unwrap_or(&Value::Null), &file, report),
                        canon(
                            ap.get(f).unwrap_or(&Value::Null),
                            &format!("anchors/{name}"),
                            report,
                        ),
                    ) else {
                        return;
                    };
                    same &= x == y;
                }
                same
            }
            _ => false,
        };
        if !same {
            report.violation(
                "N-4",
                format!(
                    "{name}: approvals[{n}]（{aid}）の承認の写しが {aid} の承認欄と一致しない（凍結後に承認の逐語・日付・裁定 id・対話面が書き換えられた・P-12.2）"
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_spelling_is_scanned_without_regex() {
        for ok in ["v1.0", "v1.10", "v12.3"] {
            assert!(is_version(ok), "{ok}");
        }
        for ng in ["v1", "v1.0.0", "1.0", "v.1", "v1.", "V1.0"] {
            assert!(!is_version(ng), "{ng}");
        }
    }

    #[test]
    fn projection_keeps_floor_fields_only() {
        let c = yaml::parse_typed(
            "schema: {v: 1}\nmeta: {version: v1.0}\nprecedence: {text: x}\narticles:\n  - {id: P-1, title: t, tier: always, plain: p, statements: [{id: P-1.1, text: s, pattern: u, strength: must, extra: 1}]}\n",
        )
        .unwrap();
        let scope: Vec<String> = ["schema", "precedence", "articles"]
            .map(String::from)
            .to_vec();
        let p = project(&c, &scope).unwrap();
        assert_eq!(
            yaml::canonical(&p).unwrap(),
            "{\"articles\":[{\"binds\":null,\"id\":\"P-1\",\"statements\":[{\"id\":\"P-1.1\",\"pattern\":\"u\",\"strength\":\"must\",\"text\":\"s\"}],\"tier\":\"always\",\"title\":\"t\"}],\"precedence\":{\"text\":\"x\"},\"schema\":{\"v\":1}}"
        );
    }
}
