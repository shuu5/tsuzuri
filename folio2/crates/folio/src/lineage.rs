//! `folio check` の凍結 anchor の列の区間の消し込み（便 8 (b)・docs/design/delivery-8.md §1）。
//! day-1 の床 `scripts/check_draft.py` の diff_targets / structural_diff / verify_pair を同じ式で写す:
//! 隣り合う anchor の content を欄単位で比べ、その版を名指す発効した判断の amends と 1 対 1 に消し込む。
//! 値は型付きの木（`yaml::Value`）で読み、欄の値の表現は便 7 の正規化（`yaml::canonical`）の字面。正規表現は使わない。

use std::collections::{BTreeMap, BTreeSet};

use crate::adr::{self, Adr};
use crate::anchor;
use crate::verdict::Report;
use crate::yaml::{self, Node, Value};

/// 対象 → 欄の道 → (前, 今)。
pub(crate) type Changed = BTreeMap<String, BTreeMap<String, (String, String)>>;

/// 比べられない理由。床が例外で落ちる形は読めない、正規化できない値は測れない。
#[derive(Debug)]
enum Fail {
    Unknown(String),
    Pending(String),
}

fn marker(name: &str) -> &'static str {
    adr::floor_val(&["amends_entry", name]).unwrap_or_default()
}

/// json の受理器（床の json.loads が読めるか）。全体が 1 値（前後の空白は許す）。
fn json_text(s: &str) -> bool {
    let mut p = Json {
        b: s.as_bytes(),
        i: 0,
    };
    p.ws();
    if !p.value(0) {
        return false;
    }
    p.ws();
    p.i == p.b.len()
}

struct Json<'a> {
    b: &'a [u8],
    i: usize,
}

impl Json<'_> {
    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn eat(&mut self, c: u8) -> bool {
        if self.peek() == Some(c) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn lit(&mut self, word: &str) -> bool {
        if self.b[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            true
        } else {
            false
        }
    }

    fn ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.i += 1;
        }
    }

    fn digits(&mut self) -> bool {
        let start = self.i;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.i += 1;
        }
        self.i > start
    }

    /// 値 1 つ（入れ子の深さは床の読み手の再帰の上限の手前で打ち切る）。
    fn value(&mut self, depth: usize) -> bool {
        if depth > 900 {
            return false;
        }
        match self.peek() {
            Some(b'n') => self.lit("null"),
            Some(b't') => self.lit("true"),
            Some(b'f') => self.lit("false"),
            Some(b'N') => self.lit("NaN"),
            Some(b'I') => self.lit("Infinity"),
            Some(b'"') => self.string(),
            Some(b'[') => self.array(depth),
            Some(b'{') => self.object(depth),
            Some(b'-') if self.b[self.i..].starts_with(b"-Infinity") => self.lit("-Infinity"),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => false,
        }
    }

    fn number(&mut self) -> bool {
        self.eat(b'-');
        match self.peek() {
            Some(b'0') => self.i += 1,
            Some(b'1'..=b'9') => {
                self.digits();
            }
            _ => return false,
        }
        if self.eat(b'.') && !self.digits() {
            return false;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.i += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.i += 1;
            }
            if !self.digits() {
                return false;
            }
        }
        true
    }

    /// 二重引用符で囲み、中は正しい escape だけ（生の制御文字は認めない）。
    fn string(&mut self) -> bool {
        if !self.eat(b'"') {
            return false;
        }
        while let Some(c) = self.peek() {
            self.i += 1;
            match c {
                b'"' => return true,
                b'\\' => match self.peek() {
                    Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => self.i += 1,
                    Some(b'u') => {
                        self.i += 1;
                        for _ in 0..4 {
                            if !self.peek().is_some_and(|h| h.is_ascii_hexdigit()) {
                                return false;
                            }
                            self.i += 1;
                        }
                    }
                    _ => return false,
                },
                c if c < 0x20 => return false,
                _ => {}
            }
        }
        false
    }

    fn array(&mut self, depth: usize) -> bool {
        self.i += 1;
        self.ws();
        if self.eat(b']') {
            return true;
        }
        loop {
            self.ws();
            if !self.value(depth + 1) {
                return false;
            }
            self.ws();
            if self.eat(b']') {
                return true;
            }
            if !self.eat(b',') {
                return false;
            }
        }
    }

    fn object(&mut self, depth: usize) -> bool {
        self.i += 1;
        self.ws();
        if self.eat(b'}') {
            return true;
        }
        loop {
            self.ws();
            if !self.string() {
                return false;
            }
            self.ws();
            if !self.eat(b':') {
                return false;
            }
            self.ws();
            if !self.value(depth + 1) {
                return false;
            }
            self.ws();
            if self.eat(b'}') {
                return true;
            }
            if !self.eat(b',') {
                return false;
            }
        }
    }
}

/// 欄の値の型付きの表現（床の render_val）。文字列はそのまま（json として読める字面は引用符付き）・空文字は印・
/// 文字列以外は正規化の字面。json として読めるかは床の json.loads と同じ受理器で決める。
fn render_val(x: &Value, at: &str) -> Result<String, Fail> {
    match x {
        Value::Str(s) if s.is_empty() => Ok(marker("empty_marker").to_string()),
        Value::Str(s) => {
            if json_text(s) {
                yaml::canonical(x).map_err(|e| Fail::Pending(format!("正規化できない値（{e}）")))
            } else {
                Ok(s.clone())
            }
        }
        _ => yaml::canonical(x)
            .map_err(|e| Fail::Pending(format!("正規化できない値（欄の道 {at}・{e}）"))),
    }
}

/// 節を「欄の道 → 値」に平らにする。表は道を伸ばし、一覧は丸ごと 1 値。節が無い側は空の表。
fn flatten(
    obj: Option<&Value>,
    target: &str,
    prefix: &str,
    out: &mut BTreeMap<String, String>,
) -> Result<(), Fail> {
    match obj {
        None | Some(Value::Null) if prefix.is_empty() => Ok(()),
        Some(Value::Map(entries)) => {
            for (k, v) in entries {
                let key = k.py_str();
                let path = if prefix.is_empty() {
                    key
                } else {
                    format!("{prefix}.{key}")
                };
                flatten(Some(v), target, &path, out)?;
            }
            Ok(())
        }
        other => {
            let path = if prefix.is_empty() { "value" } else { prefix };
            let v = render_val(other.unwrap_or(&Value::Null), &format!("{target}.{path}"))?;
            out.insert(path.to_string(), v);
            Ok(())
        }
    }
}

/// 行の一覧（無い・null は 0 行）。一覧でない・表でない行は床が例外で落ちる＝読めない。
fn rows<'a>(value: Option<&'a Value>, what: &str) -> Result<&'a [Value], Fail> {
    let items: &[Value] = match value {
        None | Some(Value::Null) => &[],
        Some(Value::Seq(items)) => items,
        Some(_) => return Err(Fail::Unknown(format!("{what} が一覧でない"))),
    };
    if items.iter().any(|r| r.as_map().is_none()) {
        return Err(Fail::Unknown(format!("{what} に表でない行がある")));
    }
    Ok(items)
}

fn py_id(row: &Value) -> String {
    row.get("id")
        .map_or_else(|| "None".to_string(), Value::py_str)
}

fn flat_article(a: &Value, target: &str) -> Result<BTreeMap<String, String>, Fail> {
    let mut out = BTreeMap::new();
    for k in adr::floor_strs(&["anchor", "projection_article_fields"])
        .iter()
        .filter(|k| !matches!(**k, "id" | "statements"))
    {
        out.insert(
            k.to_string(),
            render_val(a.get(k).unwrap_or(&Value::Null), &format!("{target}.{k}"))?,
        );
    }
    for st in rows(a.get("statements"), &format!("条 {target} の statements"))? {
        let sid = py_id(st);
        for f in adr::floor_strs(&["anchor", "statement_fields"])
            .iter()
            .filter(|f| **f != "id")
        {
            let path = format!("statements.{sid}.{f}");
            let v = render_val(
                st.get(f).unwrap_or(&Value::Null),
                &format!("{target}.{path}"),
            )?;
            out.insert(path, v);
        }
    }
    Ok(out)
}

/// 対象 → 欄の道 → 値。対象 = 範囲の各節（articles 以外）・各条 id。
fn flat_targets(
    content: &Value,
    scope: &[String],
) -> Result<BTreeMap<String, BTreeMap<String, String>>, Fail> {
    if content.as_map().is_none() {
        return Err(Fail::Unknown("content が表でない".to_string()));
    }
    let mut out = BTreeMap::new();
    for t in scope.iter().filter(|t| *t != "articles") {
        let mut flat = BTreeMap::new();
        flatten(content.get(t), t, "", &mut flat)?;
        out.insert(t.clone(), flat);
    }
    if scope.iter().any(|t| t == "articles") {
        for a in rows(content.get("articles"), "articles")? {
            let id = py_id(a);
            let flat = flat_article(a, &id)?;
            out.insert(id, flat);
        }
    }
    Ok(out)
}

/// 並びの差分は両側に在る id だけで見る。
fn order_diff(prev: &[String], cur: &[String]) -> Option<(String, String)> {
    let common: BTreeSet<&String> = prev.iter().filter(|x| cur.contains(x)).collect();
    let join = |ids: &[String]| {
        ids.iter()
            .filter(|x| common.contains(x))
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("・")
    };
    let (a, b) = (join(prev), join(cur));
    (a != b).then_some((a, b))
}

fn article_ids(content: &Value) -> Result<Vec<String>, Fail> {
    Ok(rows(content.get("articles"), "articles")?
        .iter()
        .map(py_id)
        .collect())
}

/// 欄単位の差分（床の diff_targets）。片側に無い欄は印。
fn diff_targets(
    prev: &Value,
    prev_scope: &[String],
    cur: &Value,
    cur_scope: &[String],
) -> Result<Changed, Fail> {
    let (new_mark, del_mark) = (marker("new_article_marker"), marker("deleted_marker"));
    let pf = flat_targets(prev, prev_scope)?;
    let cf = flat_targets(cur, cur_scope)?;
    let empty = BTreeMap::new();
    let mut out: Changed = BTreeMap::new();
    for t in pf.keys().chain(cf.keys()).collect::<BTreeSet<_>>() {
        let (pt, ct) = (pf.get(t).unwrap_or(&empty), cf.get(t).unwrap_or(&empty));
        let mut ch = BTreeMap::new();
        for k in pt.keys().chain(ct.keys()).collect::<BTreeSet<_>>() {
            let (p, c) = (pt.get(k), ct.get(k));
            if p != c {
                ch.insert(
                    k.clone(),
                    (
                        p.map_or(new_mark, String::as_str).to_string(),
                        c.map_or(del_mark, String::as_str).to_string(),
                    ),
                );
            }
        }
        if !ch.is_empty() {
            out.insert(t.clone(), ch);
        }
    }
    let has = |s: &[String]| s.iter().any(|t| t == "articles");
    if has(prev_scope) && has(cur_scope) {
        if let Some(od) = order_diff(&article_ids(prev)?, &article_ids(cur)?) {
            out.entry("articles".to_string())
                .or_default()
                .insert("order".to_string(), od);
        }
        let by_id = |content: &Value| -> Result<BTreeMap<String, Vec<String>>, Fail> {
            let mut m = BTreeMap::new();
            for a in rows(content.get("articles"), "articles")? {
                let sts = rows(a.get("statements"), "statements")?
                    .iter()
                    .map(py_id)
                    .collect();
                m.insert(py_id(a), sts);
            }
            Ok(m)
        };
        let (pm, cm) = (by_id(prev)?, by_id(cur)?);
        for (id, ps) in &pm {
            if let Some(cs) = cm.get(id)
                && let Some(od) = order_diff(ps, cs)
            {
                out.entry(id.clone())
                    .or_default()
                    .insert("statements.order".to_string(), od);
            }
        }
    }
    Ok(out)
}

/// 先頭 40 字。
fn head(s: &str) -> String {
    s.chars().take(40).collect()
}

fn node_str(node: Option<&Node>) -> String {
    anchor::node_str(node)
}

/// 直前 anchor との差分と、記録に依らない検査（条の消失 P-7・改番 P-7.1）。差分と消えた条を返す。
fn structural_diff(
    prev_doc: &Value,
    cur_content: &Value,
    cur_scope: &[String],
    label: &str,
    report: &mut Report,
) -> Result<(Changed, BTreeSet<String>), Fail> {
    let prev = prev_doc.get("content").unwrap_or(&Value::Null);
    let prev_scope = anchor::str_items(prev_doc.get("projection").and_then(|p| p.get("scope")));
    let pv = prev_doc
        .get("version")
        .map_or_else(|| "None".to_string(), Value::py_str);
    let changed = diff_targets(prev, &prev_scope, cur_content, cur_scope)?;
    let prev_ids: BTreeSet<String> = article_ids(prev)?.into_iter().collect();
    let cur_ids: BTreeSet<String> = article_ids(cur_content)?.into_iter().collect();
    let lost: BTreeSet<String> = prev_ids.difference(&cur_ids).cloned().collect();
    for i in &lost {
        report.violation(
            "P-7",
            format!(
                "条 {i} が anchor {pv} に在って現行に無い（{label}・番号は消さない。条の廃止の機構は day-1 に無い＝M0 で決める）"
            ),
        );
    }
    let (new_mark, del_mark) = (marker("new_article_marker"), marker("deleted_marker"));
    let mut del_texts: BTreeMap<&str, &str> = BTreeMap::new();
    let mut add_texts: BTreeMap<&str, &str> = BTreeMap::new();
    for ch in changed.values() {
        for (k, (p, c)) in ch {
            let Some(sid) = k
                .strip_prefix("statements.")
                .and_then(|r| r.strip_suffix(".text"))
                .filter(|s| !s.is_empty())
            else {
                continue;
            };
            if c == del_mark {
                del_texts.insert(p, sid);
            } else if p == new_mark {
                add_texts.insert(c, sid);
            }
        }
    }
    for (txt, del) in &del_texts {
        if let Some(add) = add_texts.get(txt) {
            report.violation(
                "P-7",
                format!(
                    "規範文の改番: {del} を消して同じ本文を {add} として足している（番号は付け替えない・P-7.1）"
                ),
            );
        }
    }
    Ok((changed, lost))
}

/// 列の区間 1 つ（前の anchor → 版 `ver` の写し）の差分を、その版を名指す発効した判断の amends と 1:1 に消し込む。
/// 余りも不足も落とす。`c` は現行の憲法（amended_by を引く）。比べられたら差分を返す（便 9 の凍結が使う）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn verify_pair(
    prev_doc: &Value,
    cur_content: &Value,
    cur_scope: &[String],
    ver: &str,
    label: &str,
    c: &Value,
    adr: &Adr,
    report: &mut Report,
) -> Option<Changed> {
    let pv = prev_doc
        .get("version")
        .map_or_else(|| "None".to_string(), Value::py_str);
    let (changed, lost) = match structural_diff(prev_doc, cur_content, cur_scope, label, report) {
        Ok(x) => x,
        Err(e) => {
            fail_report(e, &pv, label, report);
            return None;
        }
    };
    let names_ver = |d: &Node| anchor::amends_list(d).any(|e| node_str(e.get("version")) == ver);
    let ver_adrs: Vec<&(String, Node)> = adr
        .records
        .iter()
        .filter(|(_, d)| anchor::is_effective(d) && names_ver(d))
        .collect();
    let mut recorded: BTreeMap<(String, String), (&str, &Node)> = BTreeMap::new();
    for (aid, d) in &ver_adrs {
        for e in anchor::amends_list(d).filter(|e| node_str(e.get("version")) == ver) {
            let key = (node_str(e.get("target")), node_str(e.get("field")));
            if let Some((first, _)) = recorded.get(&key) {
                report.violation(
                    "A-2",
                    format!(
                        "{aid}: {}.{}（版 {ver}）の改訂が {first} と重複して記録されている",
                        key.0, key.1
                    ),
                );
            }
            recorded.insert(key, (aid.as_str(), e));
        }
    }
    for (t, ch) in changed.iter().filter(|(t, _)| !lost.contains(*t)) {
        for (k, (p, cv)) in ch {
            match recorded.remove(&(t.clone(), k.clone())) {
                None => report.violation(
                    "N-4",
                    format!(
                        "{t}.{k} が anchor {pv} から変わったが（{label}）、それを amends（version {ver}・field）に持つ発効した判断の記録が無い（前「{}」→ 今「{}」）",
                        head(p),
                        head(cv)
                    ),
                ),
                Some((aid, e)) => {
                    if node_str(e.get("previous_text")) != *p || node_str(e.get("new_text")) != *cv {
                        report.violation(
                            "A-2",
                            format!(
                                "{aid}: {t}.{k} の previous_text / new_text が anchor {pv} と {label} の値に完全一致しない（前「{}」今「{}」）",
                                head(p),
                                head(cv)
                            ),
                        );
                    }
                }
            }
        }
    }
    for ((t, k), (aid, _)) in &recorded {
        if lost.contains(t) {
            report.violation(
                "A-2",
                format!("{aid}: amends が {t}.{k} の改訂を記録しているが、条 {t} は消されている（条は消さない・P-7）"),
            );
        } else {
            report.violation(
                "A-2",
                format!(
                    "{aid}: amends が {t}.{k}（版 {ver}）の改訂を記録しているが、anchor {pv} と {label} に差分が無い（記録と差分が食い違う）"
                ),
            );
        }
    }
    let mut art_by_id: BTreeMap<String, &Value> = BTreeMap::new();
    for a in anchor::value_rows(Some(c), "articles") {
        art_by_id.insert(py_id(a), a);
    }
    for t in changed.keys() {
        let Some(a) = art_by_id.get(t) else {
            continue;
        };
        let pointed = a
            .get("amended_by")
            .and_then(Value::as_seq)
            .unwrap_or_default()
            .iter()
            .filter(|am| am.as_map().is_some())
            .any(|am| {
                let id = am
                    .get("adr")
                    .map_or_else(|| "None".to_string(), Value::py_str);
                ver_adrs.iter().any(|(aid, _)| *aid == id)
            });
        if !pointed {
            report.violation(
                "N-4",
                format!(
                    "{t} が anchor {pv} から変わったが（{label}）、その版（{ver}）の判断の記録を指す amended_by が無い"
                ),
            );
        }
    }
    Some(changed)
}

fn fail_report(e: Fail, pv: &str, label: &str, report: &mut Report) {
    match e {
        Fail::Unknown(m) => report.unknown(format!("anchor {pv} と {label} を比べられない: {m}")),
        Fail::Pending(m) => report.pending(format!(
            "anchor {pv} と {label} の区間を消し込めない（まだ分からない）: {m}"
        )),
    }
}

/// 最新 anchor の content と現行の写しの欄単位の差分（便 9 の `--emit-amends`・検査は足さない）。
/// 比べられなければ「読めない」か「測れない」を立てて None。
pub(crate) fn diff_with(
    prev_doc: &Value,
    cur_content: &Value,
    cur_scope: &[String],
    report: &mut Report,
) -> Option<Changed> {
    let prev_scope = anchor::str_items(prev_doc.get("projection").and_then(|p| p.get("scope")));
    match diff_targets(
        prev_doc.get("content").unwrap_or(&Value::Null),
        &prev_scope,
        cur_content,
        cur_scope,
    ) {
        Ok(ch) => Some(ch),
        Err(e) => {
            let pv = prev_doc
                .get("version")
                .map_or_else(|| "None".to_string(), Value::py_str);
            fail_report(e, &pv, "現行", report);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(text: &str) -> Value {
        yaml::parse_typed(text).unwrap()
    }

    const PREV: &str = "version: v1.0\nprojection: {scope: [schema, precedence, articles]}\ncontent:\n  schema: {v: 1}\n  precedence: {text: 前文}\n  articles:\n    - {id: P-1, title: 旧, tier: always, binds: null, statements: [{id: P-1.1, text: 本文, pattern: u, strength: must}]}\n    - {id: P-2, title: 二, tier: always, binds: null, statements: [{id: P-2.1, text: 二の文, pattern: u, strength: must}]}\n";

    fn scope() -> Vec<String> {
        ["schema", "precedence", "articles"]
            .map(String::from)
            .to_vec()
    }

    fn adr_with(amends: &str) -> Adr {
        let schema = yaml::parse("{}").unwrap().root;
        let record = yaml::parse(&format!(
            "status: accepted\napproval: {{who: 持ち主}}\namends:\n{amends}"
        ))
        .unwrap()
        .root;
        Adr {
            schema,
            records: vec![("ADR-2".to_string(), record)],
        }
    }

    fn run(cur: &str, amends: &str, constitution: &str) -> Report {
        let mut report = Report::default();
        let cur = typed(cur);
        verify_pair(
            &typed(PREV),
            &cur,
            &scope(),
            "v1.1",
            "anchor v1.1",
            &typed(constitution),
            &adr_with(amends),
            &mut report,
        );
        report
    }

    const CUR_TITLE: &str = "schema: {v: 1}\nprecedence: {text: 前文}\narticles:\n  - {id: P-1, title: 新, tier: always, binds: null, statements: [{id: P-1.1, text: 本文, pattern: u, strength: must}]}\n  - {id: P-2, title: 二, tier: always, binds: null, statements: [{id: P-2.1, text: 二の文, pattern: u, strength: must}]}\n";
    const C_POINTED: &str = "articles:\n  - {id: P-1, amended_by: [{adr: ADR-2}]}\n";

    fn kinds(r: &Report) -> Vec<(String, String)> {
        r.violations.clone()
    }

    #[test]
    fn lineage_diff_is_field_by_field_with_marks() {
        let prev = typed(PREV);
        let cur = typed(
            "schema: {v: 2, w: \"\"}\nprecedence: {text: 前文}\narticles:\n  - {id: P-2, title: 二, tier: always, binds: [R-1], statements: [{id: P-2.1, text: 'true', pattern: u, strength: must}]}\n  - {id: P-1, title: 旧, tier: always, binds: null, statements: [{id: P-1.1, text: 本文, pattern: u, strength: must}]}\n",
        );
        let d = diff_targets(prev.get("content").unwrap(), &scope(), &cur, &scope()).unwrap();
        let got = |t: &str, k: &str| d.get(t).and_then(|c| c.get(k)).cloned();
        assert_eq!(got("schema", "v"), Some(("1".into(), "2".into())));
        assert_eq!(
            got("schema", "w"),
            Some(("（新設）".into(), "（空）".into()))
        );
        assert_eq!(
            got("P-2", "binds"),
            Some(("null".into(), "[\"R-1\"]".into()))
        );
        assert_eq!(
            got("P-2", "statements.P-2.1.text"),
            Some(("二の文".into(), "\"true\"".into()))
        );
        assert_eq!(
            got("articles", "order"),
            Some(("P-1・P-2".into(), "P-2・P-1".into()))
        );
        assert_eq!(d.get("precedence"), None);
        assert_eq!(d.get("P-1"), None);
    }

    #[test]
    fn lineage_matching_record_clears_the_diff() {
        let r = run(
            CUR_TITLE,
            "  - {target: P-1, field: title, version: v1.1, previous_text: 旧, new_text: 新}\n",
            C_POINTED,
        );
        assert!(r.violations.is_empty() && r.pendings.is_empty(), "{r:?}");
    }

    #[test]
    fn lineage_missing_record_is_n4() {
        let r = run(
            CUR_TITLE,
            "  - {target: P-2, field: title, version: v1.0, previous_text: a, new_text: b}\n",
            C_POINTED,
        );
        let v = kinds(&r);
        assert!(
            v.iter().any(|(k, m)| k == "N-4"
                && m.starts_with("P-1.title が anchor v1.0 から変わった")
                && m.contains("記録が無い")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|(k, m)| k == "N-4" && m.contains("amended_by が無い")),
            "{v:?}"
        );
    }

    #[test]
    fn lineage_mismatched_text_is_a2() {
        let r = run(
            CUR_TITLE,
            "  - {target: P-1, field: title, version: v1.1, previous_text: 旧, new_text: 違う}\n",
            C_POINTED,
        );
        assert_eq!(r.violations.len(), 1, "{r:?}");
        assert!(r.violations[0].1.contains("完全一致しない"));
    }

    #[test]
    fn lineage_duplicate_record_is_a2() {
        let r = run(
            CUR_TITLE,
            "  - {target: P-1, field: title, version: v1.1, previous_text: 旧, new_text: 新}\n  - {target: P-1, field: title, version: v1.1, previous_text: 旧, new_text: 新}\n",
            C_POINTED,
        );
        assert_eq!(r.violations.len(), 1, "{r:?}");
        assert!(r.violations[0].0 == "A-2" && r.violations[0].1.contains("重複して記録"));
    }

    #[test]
    fn lineage_leftover_record_is_a2() {
        let r = run(
            CUR_TITLE,
            "  - {target: P-1, field: title, version: v1.1, previous_text: 旧, new_text: 新}\n  - {target: P-2, field: tier, version: v1.1, previous_text: a, new_text: b}\n",
            C_POINTED,
        );
        assert_eq!(r.violations.len(), 1, "{r:?}");
        assert!(r.violations[0].0 == "A-2" && r.violations[0].1.contains("差分が無い"));
    }

    #[test]
    fn lineage_lost_article_is_p7() {
        let cur = "schema: {v: 1}\nprecedence: {text: 前文}\narticles:\n  - {id: P-1, title: 旧, tier: always, binds: null, statements: [{id: P-1.1, text: 本文, pattern: u, strength: must}]}\n";
        let r = run(
            cur,
            "  - {target: P-2, field: title, version: v1.1, previous_text: 二, new_text: （削除）}\n",
            C_POINTED,
        );
        let v = kinds(&r);
        assert!(
            v.iter().any(|(k, m)| k == "P-7"
                && m.starts_with("条 P-2 が anchor v1.0 に在って現行に無い（anchor v1.1")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|(k, m)| k == "A-2" && m.contains("条 P-2 は消されている")),
            "{v:?}"
        );
        assert_eq!(v.len(), 2, "{v:?}");
    }

    #[test]
    fn lineage_renumbered_statement_is_p7() {
        let cur = "schema: {v: 1}\nprecedence: {text: 前文}\narticles:\n  - {id: P-1, title: 旧, tier: always, binds: null, statements: [{id: P-1.2, text: 本文, pattern: u, strength: must}]}\n  - {id: P-2, title: 二, tier: always, binds: null, statements: [{id: P-2.1, text: 二の文, pattern: u, strength: must}]}\n";
        let r = run(cur, "  []\n", C_POINTED);
        let v = kinds(&r);
        assert!(
            v.iter()
                .any(|(k, m)| k == "P-7" && m.contains("P-1.1 を消して同じ本文を P-1.2")),
            "{v:?}"
        );
    }

    #[test]
    fn lineage_bracket_strings_follow_json_loads() {
        for s in [
            "[1]",
            " [1, \"a\", {\"k\": [null, true]}] ",
            "{}",
            "{\"a\": -0.5e+2}",
            "[\"\\u00e9\"]",
        ] {
            assert!(
                render_val(&Value::Str(s.into()), "x")
                    .unwrap()
                    .starts_with('"'),
                "{s}"
            );
        }
        for s in [
            "[reject, build-check, human-review, none]",
            "[1,]",
            "{a: 1}",
            "{\"a\" 1}",
            "[1] x",
            "[\"\\x\"]",
            "[01]",
        ] {
            assert_eq!(render_val(&Value::Str(s.into()), "x").unwrap(), s);
        }
    }

    #[test]
    fn lineage_json_like_strings_are_quoted() {
        for s in ["null", "true", "-1.5e3", "0", "\"a\\n\"", "NaN"] {
            assert!(
                render_val(&Value::Str(s.into()), "x")
                    .unwrap()
                    .starts_with('"'),
                "{s}"
            );
        }
        for s in ["01", "1.", "v1.0", "\"a", "abc"] {
            assert_eq!(render_val(&Value::Str(s.into()), "x").unwrap(), s);
        }
    }
}
