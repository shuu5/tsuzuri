//! `folio render`（便 11・docs/design/delivery-11.md §1）。正本 4 file と判断の記録から day-1 の読み物
//! `preview/readable.html` を導出して書く（--write）・検査する（--check）。
//! day-1 の script `scripts/render_preview.py` の 8 行目〜136 行目の写し（同じ入力から 1 byte も違わない出力）。
//! script が traceback で落ちる入力（読めない・重複キー・欄が無い・型が違う・表引きに無い値・repr の狭い写しが断る形）は
//! 2「まだ分からない」に倒し、そのときは出力先に 1 byte も書かない（P-4.1）。

use std::fs;
use std::path::Path;

use crate::verdict::Verdict;
use crate::yaml::{self, Value};

/// design token（css 1 本）。binary に埋め込む（実行時に file を探さない・本文は変えない）。
const CSS: &str = include_str!("../../../scripts/render-preview.css");

type R<T> = Result<T, String>;

pub enum Mode {
    Write,
    Check,
}

/// 1 回の実行の結果。`stdout` / `stderr` は 1 行ずつ（字面は script と同じ）。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl Outcome {
    fn unknown(reason: impl Into<String>) -> Self {
        Outcome {
            verdict: Verdict::Unknown,
            stdout: None,
            stderr: Some(format!("folio render: まだ分からない: {}", reason.into())),
        }
    }
}

// ── script の表引き（KeyError = 2 の表と、無ければ文字列化を出す表）──

const AST: &[(&str, (&str, &str))] = &[
    ("proposed", ("提案中・拘束力なし", "#916626")),
    ("accepted", ("発効", "#1e7b65")),
    ("retired", ("廃止", "#666666")),
];
const VERD: &[(&str, &str)] = &[("adopted", "採用"), ("rejected", "退けた")];
const DOCST: &[(&str, &str)] = &[
    ("effective", "発効・拘束力あり"),
    ("draft", "未承認・拘束力なし"),
];
/// 段: 名・色・見出しの添え書き（script の TIER と 53 行目の表を 1 つに）
const TIER: &[(&str, (&str, &str, &str))] = &[
    ("always", ("いつも守る", "#1e7b65", "道具も AI も守る")),
    (
        "ask-first",
        ("確認してから変える", "#916626", "実行前に持ち主へ聞く"),
    ),
    (
        "never",
        ("絶対にやらない", "#b22323", "やろうとしたら機械が拒む"),
    ),
];
const STR: &[(&str, &str)] = &[
    ("must", "必ず守る"),
    ("must-not", "決してしない"),
    ("should", "既定（外すなら理由）"),
];
const MECH: &[(&str, &str)] = &[
    ("reject", "機械が落とす"),
    ("build-check", "生成時の検査"),
    ("human-review", "人が目で確かめる"),
    ("none", "なし"),
];
const LIVE: &[(&str, &str)] = &[
    ("now", "いま動く"),
    ("M0", "M0 で動く"),
    ("delivery-0", "便 0 で動く"),
    ("M1", "M1 で動く"),
    ("adr", "ADR 正本の schema 後"),
];
const BIND: &[(&str, &str)] = &[("tool", "道具"), ("practice", "作法"), ("both", "両方")];
const KIND: &[(&str, &str)] = &[
    ("v1-incident", "v1 の実害"),
    ("scribe2-article", "scribe2 の条"),
    ("folio2-ruling", "持ち主の裁定"),
];
const PAT: &[(&str, &str)] = &[
    ("ubiquitous", "つねに"),
    ("event", "〜のとき"),
    ("state", "〜のあいだ"),
    ("unwanted", "〜になったら"),
    ("optional", "〜ならば"),
];

// ── Python の字面の写し（pyfmt）──

/// script の `html.escape`（quote 込み）と同じ 5 字。
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            c => out.push(c),
        }
    }
    out
}

/// script の `if 値:` と `値 or 既定` の判定（Python の偽 = null・false・整数 0・小数 0.0・空の文字列・空の一覧・空の表）。
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null | Value::Bool(false) => false,
        Value::Bool(true) | Value::Date(_) => true,
        Value::Int(s) => !s.trim_start_matches(['-', '+']).bytes().all(|b| b == b'0'),
        Value::Float(f) => *f != 0.0,
        Value::Str(s) => !s.is_empty(),
        Value::Seq(s) => !s.is_empty(),
        Value::Map(m) => !m.is_empty(),
    }
}

/// script の `str(x)`。scalar は `Value::py_str`・一覧と表は repr の狭い写し。
fn py_str(v: &Value) -> R<String> {
    match v {
        Value::Seq(_) | Value::Map(_) => repr(v),
        other => Ok(other.py_str()),
    }
}

/// repr の文字列の中身に受ける字（それ以外は Python の repr が escape や別の字面を出しうるので断る・fail-closed）。
fn repr_char_ok(c: char) -> bool {
    let ascii_printable = (' '..='~').contains(&c) && !matches!(c, '\'' | '"' | '\\');
    ascii_printable || c.is_alphanumeric() || "、。・「」（）〔〕→：／—＝＋〜".contains(c)
}

/// Python の repr の狭い写し（一覧と表が `str` に渡る所・正本に 1 か所在る）。
fn repr(v: &Value) -> R<String> {
    match v {
        Value::Null | Value::Bool(_) | Value::Int(_) => Ok(v.py_str()),
        Value::Float(f) => Err(format!("repr の狭い写しが断る形（小数 {f}）")),
        Value::Date(d) => Err(format!("repr の狭い写しが断る形（日付 {d}）")),
        Value::Str(s) => {
            if s.chars().all(repr_char_ok) {
                Ok(format!("'{s}'"))
            } else {
                Err(format!("repr の狭い写しが断る形（文字列「{s}」）"))
            }
        }
        Value::Seq(items) => {
            let parts = items.iter().map(repr).collect::<R<Vec<_>>>()?;
            Ok(format!("[{}]", parts.join(", ")))
        }
        Value::Map(entries) => {
            let parts = entries
                .iter()
                .map(|(k, v)| Ok(format!("{}: {}", repr(k)?, repr(v)?)))
                .collect::<R<Vec<_>>>()?;
            Ok(format!("{{{}}}", parts.join(", ")))
        }
    }
}

/// 型付きの木の 1 点（欄の道つき）。script の `x['k']`・`x.get('k')`・`E(x)`・`str(x)`・`%d` を写す口。
struct X<'a> {
    v: &'a Value,
    at: String,
}

impl<'a> X<'a> {
    fn root(v: &'a Value, at: &str) -> Self {
        X {
            v,
            at: at.to_string(),
        }
    }

    fn child(&self, v: &'a Value, seg: &str) -> Self {
        X {
            v,
            at: format!("{}{seg}", self.at),
        }
    }

    fn map(&self) -> R<()> {
        self.v
            .as_map()
            .map(|_| ())
            .ok_or_else(|| format!("{}: 表でない", self.at))
    }

    /// `x['k']`（無ければ KeyError）。
    fn f(&self, key: &str) -> R<X<'a>> {
        self.map()?;
        self.v
            .get(key)
            .map(|v| self.child(v, &format!(".{key}")))
            .ok_or_else(|| format!("{}: 欄 {key} が無い", self.at))
    }

    /// `x.get('k')`（x が表でなければ AttributeError）。
    fn g(&self, key: &str) -> R<Option<X<'a>>> {
        self.map()?;
        Ok(self.v.get(key).map(|v| self.child(v, &format!(".{key}"))))
    }

    /// `if x.get('k'):`
    fn t(&self, key: &str) -> R<bool> {
        Ok(self.g(key)?.is_some_and(|x| truthy(x.v)))
    }

    /// `for i in x`（一覧だけを受ける）。
    fn seq(&self) -> R<Vec<X<'a>>> {
        let items = self
            .v
            .as_seq()
            .ok_or_else(|| format!("{}: 一覧でない", self.at))?;
        Ok(items
            .iter()
            .enumerate()
            .map(|(i, v)| self.child(v, &format!("[{i}]")))
            .collect())
    }

    /// `E(x)` や `'・'.join` に渡る所（文字列だけを受ける）。
    fn text(&self) -> R<&'a str> {
        self.v
            .as_str()
            .ok_or_else(|| format!("{}: 文字列でない", self.at))
    }

    /// `E(x)`
    fn e(&self) -> R<String> {
        Ok(esc(self.text()?))
    }

    /// `str(x)`
    fn s(&self) -> R<String> {
        py_str(self.v).map_err(|e| format!("{}: {e}", self.at))
    }

    /// `E(str(x))`
    fn es(&self) -> R<String> {
        Ok(esc(&self.s()?))
    }

    /// `%d`（整数だけを受ける）。
    fn int(&self) -> R<&'a str> {
        match self.v {
            Value::Int(n) => Ok(n),
            _ => Err(format!("{}: 整数でない", self.at)),
        }
    }

    /// `E(x['k'])`
    fn ef(&self, key: &str) -> R<String> {
        self.f(key)?.e()
    }

    /// `E(str(x['k']))`
    fn esf(&self, key: &str) -> R<String> {
        self.f(key)?.es()
    }

    /// `sep.join(x)`（要素は文字列だけ）。
    fn join_e(&self, sep: &str) -> R<String> {
        let parts = self.seq()?.iter().map(X::text).collect::<R<Vec<_>>>()?;
        Ok(parts.join(sep))
    }

    /// `sep.join(map(str, x))`
    fn join_s(&self, sep: &str) -> R<String> {
        let parts = self.seq()?.iter().map(X::s).collect::<R<Vec<_>>>()?;
        Ok(parts.join(sep))
    }

    /// `TABLE[x]`（表に無い値は KeyError）。
    fn lookup<T: Copy>(&self, table: &[(&str, T)], what: &str) -> R<T> {
        if let Value::Str(s) = self.v
            && let Some((_, t)) = table.iter().find(|(k, _)| k == s)
        {
            return Ok(*t);
        }
        Err(format!(
            "{}: {what} の表に無い値「{}」",
            self.at,
            self.v.py_str()
        ))
    }
}

/// `E(TABLE.get(x, str(x)))`（LIVE・DOCST）: 表に無ければ値の文字列化を escape して出す。無い欄は `None`。
fn get_or_str(table: &[(&str, &str)], x: Option<&X<'_>>) -> R<String> {
    match x.map(|x| x.v) {
        None => Ok("None".to_string()),
        Some(Value::Str(s)) => Ok(table
            .iter()
            .find(|(k, _)| k == s)
            .map_or_else(|| esc(s), |(_, t)| t.to_string())),
        Some(Value::Seq(_) | Value::Map(_)) => Err(format!(
            "{}: 表引きの鍵が一覧か表",
            x.map_or("", |x| x.at.as_str())
        )),
        Some(other) => Ok(esc(&other.py_str())),
    }
}

/// `str(x)`（無い欄は `None`）。
fn s_opt(x: Option<&X<'_>>) -> R<String> {
    x.map_or_else(|| Ok("None".to_string()), X::s)
}

/// 閾値の値の読める形（script の関数 val）。
fn val(x: &X<'_>, d: usize) -> R<String> {
    match x.v {
        Value::Map(entries) => {
            let mut parts = Vec::with_capacity(entries.len());
            for (k, vv) in entries {
                let vx = x.child(vv, &format!(".{}", k.py_str()));
                let body = if matches!(vv, Value::Map(_)) {
                    format!("<br>{}", val(&vx, d + 1)?)
                } else {
                    val(&vx, d + 1)?
                };
                parts.push(format!(
                    "{}<b>{}</b>: {body}",
                    "　".repeat(d),
                    esc(&py_str(k)?)
                ));
            }
            Ok(parts.join("<br>"))
        }
        Value::Seq(_) => {
            let parts = x
                .seq()?
                .iter()
                .map(|i| match i.v {
                    Value::Str(s) => Ok(format!("「{}」", esc(s))),
                    _ => val(i, d),
                })
                .collect::<R<Vec<_>>>()?;
            Ok(parts.join("・"))
        }
        Value::Null => Ok("<code>null</code>".to_string()),
        _ => x.es(),
    }
}

/// 関係の欄の読める形（script の関数 rel）。
fn rel(x: &X<'_>) -> R<String> {
    let Value::Map(entries) = x.v else {
        return x.join_s("・");
    };
    const NAMES: &[(&str, &str)] = &[
        ("reqs", "要件"),
        ("rules", "rules 行"),
        ("articles", "条"),
        ("sections", "節"),
    ];
    let mut parts = Vec::new();
    for (k, vv) in entries {
        if !truthy(vv) {
            continue;
        }
        let name = match k {
            Value::Str(s) => NAMES
                .iter()
                .find(|(n, _)| n == s)
                .map_or_else(|| s.clone(), |(_, t)| t.to_string()),
            Value::Seq(_) | Value::Map(_) => {
                return Err(format!("{}: 表のキーが一覧か表", x.at));
            }
            other => other.py_str(),
        };
        let vx = x.child(vv, &format!(".{name}"));
        parts.push(format!("{name}: {}", vx.join_s("・")?));
    }
    Ok(parts.join(" ／ "))
}

// ── 読み（load）──

/// 正本 1 file を型付きで読む。読めない・重複キー・空の文書は Err（まだ分からない）。
fn load(dir: &Path, name: &str) -> R<Value> {
    let path = dir.join(name);
    let text = fs::read_to_string(&path).map_err(|e| format!("{name}: 読めない: {e}"))?;
    let doc = yaml::parse(&text).map_err(|e| format!("{name}: 読めない: {e}"))?;
    if let Some(d) = doc.duplicates.first() {
        return Err(format!("{name}: 重複キー「{}」（{} 行）", d.key, d.line));
    }
    yaml::parse_typed(&text).map_err(|e| format!("{name}: {e}"))
}

/// 判断の記録の id（ADR-n）の n（「-」で割った 2 番目・ASCII の数字列だけ）。
fn adr_number(id: &str) -> Option<u64> {
    let second = id.split('-').nth(1)?;
    if second.is_empty() || !second.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    second.parse().ok()
}

/// 判断の記録を id の数の昇順に並べる（数字列でない・同じ数が 2 本以上は Err）。
fn order_adrs(docs: Vec<(String, Value)>) -> R<Vec<Value>> {
    let mut keyed = Vec::with_capacity(docs.len());
    for (name, doc) in docs {
        let id = X::root(&doc, &name).f("id")?.s()?;
        let n = adr_number(&id)
            .ok_or_else(|| format!("{name}: id「{id}」の番号が ASCII の数字列でない"))?;
        keyed.push((n, name, doc));
    }
    keyed.sort_by_key(|(n, _, _)| *n);
    if let Some(w) = keyed.windows(2).find(|w| w[0].0 == w[1].0) {
        return Err(format!(
            "判断の記録の番号 {} が 2 本以上（{}・{}）",
            w[0].0, w[0].1, w[1].1
        ));
    }
    Ok(keyed.into_iter().map(|(_, _, doc)| doc).collect())
}

/// `adr/` の下で名が ADR- で始まり .yaml で終わる file を読む（dir が無ければ script の glob と同じく空）。
fn load_adrs(dir: &Path) -> R<Vec<Value>> {
    let entries = match fs::read_dir(dir.join("adr")) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("adr: 読めない: {e}")),
    };
    let mut docs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("adr: 読めない: {e}"))?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| format!("adr: file 名が UTF-8 でない: {}", name.to_string_lossy()))?;
        if name.starts_with("ADR-") && name.ends_with(".yaml") {
            let rel = format!("adr/{name}");
            let doc = load(dir, &rel)?;
            docs.push((rel, doc));
        }
    }
    order_adrs(docs)
}

/// 凍結 anchor の列（`anchors/index.yaml` が無ければ空・在れば entries の各要素の version の文字列化）。
fn load_anchors(dir: &Path) -> R<Vec<String>> {
    if !dir.join("anchors/index.yaml").exists() {
        return Ok(Vec::new());
    }
    let idx = load(dir, "anchors/index.yaml")?;
    let idx = X::root(&idx, "anchors/index.yaml");
    match idx.g("entries")? {
        None => Ok(Vec::new()),
        Some(entries) => entries
            .seq()?
            .iter()
            .map(|e| s_opt(e.g("version")?.as_ref()))
            .collect(),
    }
}

// ── 導出（page）──

/// 正本 → 読み物の HTML（決定的）。script の 8 行目〜136 行目の順にそのまま写す。
pub fn derive(dir: &Path) -> R<String> {
    let c_doc = load(dir, "constitution.yaml")?;
    let r_doc = load(dir, "rules.yaml")?;
    let v_doc = load(dir, "vocabulary.yaml")?;
    let s_doc = load(dir, "srs.yaml")?;
    let c = X::root(&c_doc, "constitution.yaml");
    let r = X::root(&r_doc, "rules.yaml");
    let v = X::root(&v_doc, "vocabulary.yaml");
    let s = X::root(&s_doc, "srs.yaml");
    let m = c.f("meta")?;
    let sm = s.f("meta")?;
    let ver = m.ef("version")?;
    let sver = sm.ef("version")?;
    let adr_docs = load_adrs(dir)?;
    let adrs: Vec<X<'_>> = adr_docs
        .iter()
        .enumerate()
        .map(|(i, d)| X::root(d, &format!("adr[{i}]")))
        .collect();
    let anch = load_anchors(dir)?;
    let mut adrterm = None;
    for t in v.f("terms")?.seq()? {
        if t.g("id")?.is_some_and(|x| x.v.as_str() == Some("adr")) {
            adrterm = Some(t);
            break;
        }
    }

    let mut o: Vec<String> = Vec::new();
    o.push(format!(
        "<!doctype html><html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>folio2 — 設計文書の読み物（憲法 {ver}・要件書 {sver}・判断の記録 {} 件）</title><style>{CSS}</style></head><body><div class=\"page\">",
        adrs.len()
    ));
    let cnt = m.f("counts")?;
    let status_note = if sm.t("status_note")? {
        format!("（{}）", sm.esf("status_note")?)
    } else {
        String::new()
    };
    o.push(format!(
        "<h1>folio2 — 設計文書の読み物（憲法 {ver}・要件書 {sver}）</h1><p class=\"sub\">正本 4 file（<code>constitution.yaml</code> / <code>rules.yaml</code> / <code>vocabulary.yaml</code> / <code>srs.yaml</code>）と判断の記録（<code>adr/ADR-n.yaml</code>）を <code>render_preview.py</code> で読み物にしたもの。この HTML は手で直さない。憲法 {ver}: <b>{}</b>（binding: {}）／ 要件書 {sver}: <b>{}</b>{status_note}</p>",
        get_or_str(DOCST, m.g("status")?.as_ref())?,
        esc(&s_opt(m.g("binding")?.as_ref())?),
        get_or_str(DOCST, sm.g("status")?.as_ref())?,
    ));
    let articles = c.f("articles")?.seq()?;
    let field_terms_len = match v.g("field_terms")? {
        None => 0,
        Some(x) => x.seq()?.len(),
    };
    o.push(format!(
        "<nav class=\"toc\"><a href=\"#const\">憲法（{} 条）</a><a href=\"#rules\">rules（閾値 {}・開発規律 {}）</a><a href=\"#vocab\">語彙（{} 語 + 欄 {field_terms_len}）</a><a href=\"#srs\">要件書 {sver}（FR {}・NFR {}・AC {}・CON {}）</a><a href=\"#adr\">判断の記録（{}）</a></nav>",
        articles.len(),
        r.f("thresholds")?.seq()?.len(),
        r.f("discipline")?.seq()?.len(),
        v.f("terms")?.seq()?.len(),
        s.f("requirements")?.seq()?.len(),
        s.f("nonfunctional")?.seq()?.len(),
        s.f("acceptance")?.seq()?.len(),
        s.f("constraints")?.seq()?.len(),
        adrs.len()
    ));
    for (key, open, label) in [
        ("changes_from_v0_2", " open", "v0.2"),
        ("changes_from_v0_1", "", "v0.1"),
    ] {
        if m.t(key)? {
            let items = m.f(key)?.seq()?;
            let lis = items
                .iter()
                .map(|x| Ok(format!("<li>{}</li>", x.es()?)))
                .collect::<R<Vec<_>>>()?
                .concat();
            o.push(format!(
                "<details{open}><summary>{label} からの変更（{} 件）</summary><ul>{lis}</ul></details>",
                items.len()
            ));
        }
    }
    // ── 憲法
    o.push(format!(
        "<h2 id=\"const\">憲法（{} 条 = いつも守る {} ／ 確認してから変える {} ／ 絶対にやらない {}）</h2>",
        articles.len(),
        cnt.f("always")?.int()?,
        cnt.f("ask-first")?.int()?,
        cnt.f("never")?.int()?
    ));
    let ns = c.f("north_star")?;
    o.push(format!(
        "<h3>00 北極星</h3><p><b>{}</b></p><p>誰のため: {}<br>達成の判定: {}</p>",
        ns.ef("statement")?,
        ns.ef("for_whom")?,
        ns.ef("judged_by")?
    ));
    let pr = c.f("precedence")?;
    let pr_binds = match pr.g("binds")? {
        None => "両方",
        Some(x) => x.lookup(BIND, "縛る相手")?,
    };
    let pr_mech = pr.f("mechanism")?;
    let pr_note = match pr_mech.g("note")? {
        None => String::new(),
        Some(x) => x.e()?,
    };
    o.push(format!(
        "<h3>前文 — 順位（条ではない・全条に先立つ）</h3><div class=\"st\">{}</div><p class=\"plain\"><b>やさしく言うと</b><br>{}</p><p class=\"sub\">縛る相手: {pr_binds} ／ 機構: {} — {pr_note}</p>",
        pr.ef("text")?,
        pr.ef("plain")?,
        pr_mech.f("kind")?.lookup(MECH, "機構")?
    ));
    let toc = articles
        .iter()
        .map(|a| {
            let id = a.f("id")?.s()?;
            Ok(format!("<a href=\"#{id}\">{id} {}</a>", a.ef("title")?))
        })
        .collect::<R<Vec<_>>>()?
        .concat();
    o.push(format!("<nav class=\"toc\">{toc}</nav>"));
    let mut cur: Option<&str> = None;
    for a in &articles {
        let tier = a.f("tier")?;
        let (tn, tc, sub) = tier.lookup(TIER, "段")?;
        if cur != Some(tn) {
            cur = Some(tn);
            o.push(format!("<h3>{tn}（{sub}）</h3>"));
        }
        let id = a.f("id")?.s()?;
        o.push(format!(
            "<section class=\"art\" id=\"{id}\"><h4><span class=\"gid\">{id}</span>{}<span class=\"tier\" style=\"background:{tc}\">{tn}</span><span class=\"bind\">縛る相手: {}</span></h4>",
            a.ef("title")?,
            a.f("binds")?.lookup(BIND, "縛る相手")?
        ));
        for st in a.f("statements")?.seq()? {
            o.push(format!(
                "<div class=\"st\"><span class=\"sid\">{}</span>{}<span class=\"str\">{}・{}</span></div>",
                st.f("id")?.s()?,
                st.ef("text")?,
                st.f("pattern")?.lookup(PAT, "型")?,
                st.f("strength")?.lookup(STR, "強さ")?
            ));
        }
        o.push(format!(
            "<p class=\"plain\"><b>やさしく言うと</b><br>{}</p><dl>",
            a.ef("plain")?
        ));
        let rationale = a
            .f("rationale")?
            .seq()?
            .iter()
            .map(|x| {
                Ok(format!(
                    "<b>{}</b>: {}",
                    x.f("kind")?.lookup(KIND, "出所の種別")?,
                    x.esf("ref")?
                ))
            })
            .collect::<R<Vec<_>>>()?
            .join("<br>");
        o.push(format!("<dt>出所</dt><dd>{rationale}</dd>"));
        let mch = a.f("mechanism")?;
        let mch_note = match mch.g("note")? {
            None => String::new(),
            Some(x) => x.e()?,
        };
        o.push(format!(
            "<dt>どう守らせるか</dt><dd><b>{}</b>（{}）— {mch_note}</dd>",
            mch.f("kind")?.lookup(MECH, "機構")?,
            get_or_str(LIVE, mch.g("live")?.as_ref())?
        ));
        if a.t("retreat")? {
            let rt = a.f("retreat")?;
            o.push(format!(
                "<dt>撤退条件</dt><dd>[{}] {}</dd>",
                rt.ef("kind")?,
                rt.ef("condition")?
            ));
        }
        if a.t("relations")? {
            o.push(format!(
                "<dt>関係</dt><dd>{}</dd>",
                esc(&rel(&a.f("relations")?)?)
            ));
        }
        if a.t("supersedes_v1")? {
            let sv = a.f("supersedes_v1")?;
            o.push(format!(
                "<dt>v1 から置き換えた条</dt><dd>{}「{}」（{}）<br>理由: {}</dd>",
                sv.ef("doc")?,
                sv.ef("article")?,
                sv.ef("ruling")?,
                sv.ef("rationale")?
            ));
        }
        if a.t("amended_by")? {
            for am in a.f("amended_by")?.seq()? {
                o.push(format!(
                    "<dt>改訂来歴</dt><dd>{}（{}・{}・{}）<br>消す文: {}<br>理由: {}</dd>",
                    am.esf("adr")?,
                    am.esf("date")?,
                    am.ef("approved_by")?,
                    am.ef("ruling")?,
                    am.ef("previous_text")?,
                    am.ef("rationale")?
                ));
            }
        }
        if a.t("note")? {
            o.push(format!("<dt>planner 注</dt><dd>{}</dd>", a.ef("note")?));
        }
        o.push("</dl></section>".to_string());
    }
    let am = c.f("amendment")?;
    let steps = am
        .f("steps")?
        .seq()?
        .iter()
        .map(|st| {
            let article = st.f("article")?;
            let article = if matches!(article.v, Value::Seq(_)) {
                rel(&article)?
            } else {
                article.s()?
            };
            Ok(format!(
                "<li><b>{}</b>（{}）— {}　<span class=\"sub\">条: {}</span></li>",
                st.esf("n")?,
                st.ef("who")?,
                st.ef("what")?,
                esc(&article)
            ))
        })
        .collect::<R<Vec<_>>>()?
        .concat();
    o.push(format!(
        "<h3>06 改訂</h3><p class=\"note\">{}</p><ol>{steps}</ol><p>発効: {}</p>",
        am.ef("declaration")?,
        am.f("effective_step")?.ef("what")?
    ));
    let sources = c
        .f("sources")?
        .seq()?
        .iter()
        .map(|x| {
            Ok(format!(
                "<li><b>{}</b> — {}</li>",
                x.ef("name")?,
                x.esf("ref")?
            ))
        })
        .collect::<R<Vec<_>>>()?
        .concat();
    o.push(format!("<h3>08 出所</h3><ol>{sources}</ol>"));
    // ── rules
    o.push("<h2 id=\"rules\">rules — 閾値の行（数値は条文に書かない）</h2><div class=\"wrap\"><table><tr><th>id</th><th>条</th><th>何の数値か</th><th>値</th><th>種別</th><th>状態</th><th>裁定</th><th>時刻</th><th>注</th></tr>".to_string());
    for x in r.f("thresholds")?.seq()? {
        let mut extra = Vec::new();
        for k in ["projection", "basis", "same_failure", "population", "note"] {
            if x.t(k)? {
                extra.push(format!("<b>{k}</b>: {}", x.esf(k)?));
            }
        }
        let mut row = String::from("<tr>");
        for k in [
            "id", "article", "what", "value", "kind", "status", "ruling", "ruled_at",
        ] {
            let cell = match x.g(k)? {
                Some(cv) if k == "value" && matches!(cv.v, Value::Map(_) | Value::Seq(_)) => {
                    val(&cv, 0)?
                }
                Some(cv) if !matches!(cv.v, Value::Null) => cv.es()?,
                _ => "—".to_string(),
            };
            row.push_str(&format!("<td>{cell}</td>"));
        }
        row.push_str(&format!("<td>{}</td></tr>", extra.join("<br>")));
        o.push(row);
    }
    let excluded = r.f("schema")?.f("excluded")?;
    o.push(format!(
        "</table></div><p class=\"sub\">置かない数値: {}（{}）</p>",
        esc(&excluded.f("what")?.join_e(" ／ ")?),
        excluded.ef("why")?
    ));
    o.push("<h3>開発規律の行（人が守る作法・条から id で参照）</h3><div class=\"wrap\"><table><tr><th>id</th><th>条</th><th>内容</th><th>状態</th><th>裁定</th><th>時刻</th><th>注</th></tr>".to_string());
    for x in r.f("discipline")?.seq()? {
        let mut row = String::from("<tr>");
        for k in [
            "id", "article", "what", "status", "ruling", "ruled_at", "note",
        ] {
            let cell = match x.g(k)? {
                Some(cv) if !matches!(cv.v, Value::Null) => cv.es()?,
                _ => "—".to_string(),
            };
            row.push_str(&format!("<td>{cell}</td>"));
        }
        row.push_str("</tr>");
        o.push(row);
    }
    o.push("</table></div>".to_string());
    // ── 語彙
    let terms = v.f("terms")?.seq()?;
    o.push(format!(
        "<h2 id=\"vocab\">語彙（{} 語・正本 vocabulary.yaml・読者が引く場所は憲法 §7）</h2><div class=\"wrap\"><table><tr><th>語</th><th>原語</th><th>ひとことで</th><th>定義</th></tr>",
        terms.len()
    ));
    let en_of = |t: &X<'_>| -> R<String> {
        match t.g("en")? {
            Some(x) if truthy(x.v) => x.es(),
            _ => Ok(String::new()),
        }
    };
    for t in &terms {
        o.push(format!(
            "<tr><td id=\"v-{}\"><b>{}</b></td><td>{}</td><td>{}</td><td>{}</td></tr>",
            t.ef("id")?,
            t.ef("term")?,
            en_of(t)?,
            t.ef("short")?,
            t.ef("def")?
        ));
    }
    o.push("</table></div>".to_string());
    if v.t("field_terms")? {
        let rows = v
            .f("field_terms")?
            .seq()?
            .iter()
            .map(|t| {
                Ok(format!(
                    "<tr><td><b>{}</b></td><td>{}</td><td>{}</td></tr>",
                    t.ef("term")?,
                    en_of(t)?,
                    t.ef("def")?
                ))
            })
            .collect::<R<Vec<_>>>()?
            .concat();
        o.push(format!(
            "<h3>欄の名前（機械層・読者は引かない）</h3><div class=\"wrap\"><table><tr><th>欄</th><th>原語</th><th>定義</th></tr>{rows}</table></div>"
        ));
    }
    // ── 要件書
    o.push(format!(
        "<h2 id=\"srs\">{} — {sver}</h2><p class=\"note\">{}</p>",
        sm.ef("title")?,
        sm.ef("promise")?
    ));
    // 版ごとの変更点（meta のキーのうち changes_from_ で始まるものを文字列の昇順に）
    let mut change_keys: Vec<String> = Vec::new();
    for (k, _) in sm.v.as_map().unwrap_or_default() {
        let key = match k {
            Value::Seq(_) | Value::Map(_) => {
                return Err(format!("{}: 表のキーが一覧か表", sm.at));
            }
            other => other.py_str(),
        };
        if key.starts_with("changes_from_") {
            change_keys.push(key);
        }
    }
    change_keys.sort();
    for k in &change_keys {
        let items = sm.f(k)?.seq()?;
        let lis = items
            .iter()
            .map(|x| Ok(format!("<li>{}</li>", x.es()?)))
            .collect::<R<Vec<_>>>()?
            .concat();
        o.push(format!(
            "<details><summary>{} からの変更（{} 件）</summary><ul>{lis}</ul></details>",
            esc(&k["changes_from_".len()..].replace('_', ".")),
            items.len()
        ));
    }
    let goals = s.f("goals")?.seq()?;
    let goal_lis = goals
        .iter()
        .map(|g| {
            Ok(format!(
                "<li><b>{} {}</b> — {}</li>",
                g.f("id")?.s()?,
                g.ef("title")?,
                g.ef("text")?
            ))
        })
        .collect::<R<Vec<_>>>()?
        .concat();
    o.push(format!("<h3>01 ゴール</h3><ul>{goal_lis}</ul>"));
    let scope = s.f("scope")?;
    o.push(format!(
        "<h3>02 範囲</h3><p><b>M0 で作る</b>: {}</p><p><b>M0 では作らない</b>: {}</p>",
        esc(&scope.f("build")?.join_e(" ／ ")?),
        esc(&scope.f("not_build")?.join_e(" ／ ")?)
    ));
    if s.t("scope_m1")? {
        let sm1 = s.f("scope_m1")?;
        let join_opt = |key: &str| -> R<String> {
            match sm1.g(key)? {
                None => Ok(String::new()),
                Some(x) => x.join_e(" ／ "),
            }
        };
        let note = if sm1.t("note")? {
            format!("<p class=\"sub\">{}</p>", sm1.esf("note")?)
        } else {
            String::new()
        };
        o.push(format!(
            "<p><b>M1 で作る</b>: {}</p><p><b>M1 では作らない</b>: {}</p>{note}",
            esc(&join_opt("build")?),
            esc(&join_opt("not_build")?)
        ));
    }
    let rail = s
        .f("rail")?
        .seq()?
        .iter()
        .map(|st| {
            let reqs = st.f("reqs")?.join_e("・")?;
            let reqs: &str = if reqs.is_empty() {
                "（この段を定める要件は無い）"
            } else {
                reqs.as_str()
            };
            let note = if st.t("note")? {
                format!("　<span class=\"sub\">{}</span>", st.ef("note")?)
            } else {
                String::new()
            };
            Ok(format!(
                "<li><b>{}</b>（{}）— 要件: {}{note}</li>",
                st.ef("what")?,
                st.ef("who")?,
                esc(reqs)
            ))
        })
        .collect::<R<Vec<_>>>()?
        .concat();
    o.push(format!(
        "<h3>図 2 folio が 1 回で通す 7 段</h3><ol>{rail}</ol>"
    ));
    let req = |o: &mut Vec<String>, x: &X<'_>| -> R<()> {
        // 要件の見出しと規範文（when → shall）は同じ 1 行に出す（器の審査は id="FRn" を含む行 1 本を材料に読む）
        let id = x.f("id")?.s()?;
        let milestone = if x.t("milestone")? {
            format!("<span class=\"bind\">段: {}</span>", x.esf("milestone")?)
        } else {
            String::new()
        };
        o.push(format!(
            "<section class=\"art\" id=\"{id}\"><h4><span class=\"gid\">{id}</span>{}<span class=\"tier\" style=\"background:#2a4d6e\">{}</span>{milestone}</h4><div class=\"st\"><span class=\"sid\">{}</span>{} → {}</div>",
            x.ef("title")?,
            x.f("strength")?.lookup(STR, "強さ")?,
            x.f("pattern")?.lookup(PAT, "型")?,
            x.ef("when")?,
            x.ef("shall")?
        ));
        o.push(format!(
            "<p class=\"plain\"><b>やさしく言うと</b><br>{}</p><dl>",
            x.ef("plain")?
        ));
        let verify = x.f("verify")?;
        let basis = x.f("basis")?.join_e("・")?;
        let ac = match verify.g("ac")? {
            None => String::new(),
            Some(a) => a.join_e("・")?,
        };
        o.push(format!(
            "<dt>根拠（条）</dt><dd>{}</dd><dt>確かめ方</dt><dd>{}: {}（受入: {}）</dd><dt>ゴール</dt><dd>{}</dd>",
            esc(or_dash(&basis)),
            verify.ef("method")?,
            verify.ef("how")?,
            esc(or_dash(&ac)),
            esc(&x.f("goals")?.join_e("・")?)
        ));
        if x.t("rules")? {
            o.push(format!(
                "<dt>rules 行</dt><dd>{}</dd>",
                esc(&x.f("rules")?.join_e("・")?)
            ));
        }
        if x.t("note")? {
            o.push(format!("<dt>planner 注</dt><dd>{}</dd>", x.ef("note")?));
        }
        o.push("</dl></section>".to_string());
        Ok(())
    };
    let requirements = s.f("requirements")?.seq()?;
    o.push(format!("<h3>03 機能要件（{}）</h3>", requirements.len()));
    for x in &requirements {
        req(&mut o, x)?;
    }
    let nonfunctional = s.f("nonfunctional")?.seq()?;
    o.push(format!("<h3>04 非機能要件（{}）</h3>", nonfunctional.len()));
    for x in &nonfunctional {
        req(&mut o, x)?;
    }
    let acceptance = s.f("acceptance")?.seq()?;
    o.push(format!("<h3>05 受入基準（{}）</h3>", acceptance.len()));
    for a in &acceptance {
        let id = a.f("id")?.s()?;
        let red = a.f("red_test")?;
        let note = if a.t("note")? {
            format!("<dt>注</dt><dd>{}</dd>", a.ef("note")?)
        } else {
            String::new()
        };
        o.push(format!(
            "<section class=\"art\" id=\"{id}\"><h4><span class=\"gid\">{id}</span>{}</h4><p class=\"plain\"><b>やさしく言うと</b><br>{}</p><dl><dt>確かめる要件</dt><dd>{}</dd><dt>RED になる test</dt><dd>{}<br><code>{}</code></dd>{note}</dl></section>",
            a.ef("title")?,
            a.ef("plain")?,
            esc(&a.f("verifies")?.join_e("・")?),
            red.ef("sentence")?,
            red.ef("fixture")?
        ));
    }
    o.push(format!("<p class=\"sub\">{}</p>", s.ef("not_frozen")?));
    let constraints = s.f("constraints")?.seq()?;
    o.push(format!(
        "<h3>06 制約（{}）</h3><div class=\"wrap\"><table><tr><th>id</th><th>制約</th><th>中身</th><th>根拠（条）</th><th>出所</th></tr>",
        constraints.len()
    ));
    for cn in &constraints {
        let basis = match cn.g("basis")? {
            None => String::new(),
            Some(b) => b.join_e("・")?,
        };
        o.push(format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            cn.f("id")?.s()?,
            cn.ef("title")?,
            cn.ef("text")?,
            esc(or_dash(&basis)),
            cn.ef("source")?
        ));
    }
    o.push("</table></div>".to_string());
    let goal_ths = goals
        .iter()
        .map(|g| Ok(format!("<th>{}</th>", g.f("id")?.s()?)))
        .collect::<R<Vec<_>>>()?
        .concat();
    o.push(format!(
        "<h3>07 対応表（要件 × ゴール × 受入 × 図・正本から導出）</h3><div class=\"wrap\"><table><tr><th>要件</th>{goal_ths}<th>受入</th><th>図</th></tr>"
    ));
    // acmap: 要件 id → それを確かめる受入基準の id（書かれた順）
    let mut acmap: Vec<(String, Vec<X<'_>>)> = Vec::new();
    for a in &acceptance {
        for q in a.f("verifies")?.seq()? {
            let q = q.text()?;
            let pos = match acmap.iter().position(|(k, _)| k == q) {
                Some(p) => p,
                None => {
                    acmap.push((q.to_string(), Vec::new()));
                    acmap.len() - 1
                }
            };
            acmap[pos].1.push(a.f("id")?);
        }
    }
    for x in requirements.iter().chain(&nonfunctional) {
        let xid = x.f("id")?;
        let goal_ids = x.f("goals")?;
        let goal_seq = goal_ids.seq()?;
        let cells = goals
            .iter()
            .map(|g| {
                let gid = g.f("id")?;
                let hit = match gid.v {
                    Value::Str(s) => goal_seq.iter().any(|q| q.v.as_str() == Some(s)),
                    _ => false,
                };
                Ok(format!("<td>{}</td>", if hit { "●" } else { "" }))
            })
            .collect::<R<Vec<_>>>()?
            .concat();
        let acs = match xid.v {
            Value::Str(s) => match acmap.iter().find(|(k, _)| k == s) {
                Some((_, ids)) => ids.iter().map(X::text).collect::<R<Vec<_>>>()?.join("・"),
                None => String::new(),
            },
            Value::Seq(_) | Value::Map(_) => {
                return Err(format!("{}: 一覧か表は表引きの鍵にならない", xid.at));
            }
            _ => String::new(),
        };
        o.push(format!(
            "<tr><td>{} {}</td>{cells}<td>{}</td><td>{}</td></tr>",
            xid.s()?,
            x.ef("title")?,
            esc(or_dash(&acs)),
            esc(&x.f("figures")?.join_e("・")?)
        ));
    }
    o.push("</table></div>".to_string());
    let approvals = sm
        .f("approval")?
        .seq()?
        .iter()
        .map(|ap| {
            let version = if ap.t("version")? {
                format!("<b>{}</b> ", ap.esf("version")?)
            } else {
                String::new()
            };
            let when = match ap.g("when")? {
                Some(x) if truthy(x.v) => x.es()?,
                _ => "未".to_string(),
            };
            let verbatim = if ap.t("verbatim")? {
                format!("・逐語「{}」", ap.esf("verbatim")?)
            } else {
                String::new()
            };
            let stamp = if ap.t("stamp")? {
                format!("（{}）", ap.esf("stamp")?)
            } else {
                String::new()
            };
            Ok(format!(
                "<li>{version}{}: {} — {when}{verbatim}{stamp}</li>",
                ap.ef("role")?,
                ap.esf("who")?
            ))
        })
        .collect::<R<Vec<_>>>()?
        .concat();
    o.push(format!(
        "<h3>承認欄</h3><ul>{approvals}</ul><p class=\"sub\">{}</p>",
        sm.ef("effective")?
    ));
    // ── 判断の記録（ADR・正本 adr/ADR-n.yaml・schema は adr/schema.yaml・M0 の生成器は無いので day-1 の暫定）
    let adr_def = match &adrterm {
        Some(t) if truthy(t.v) => match t.g("def")? {
            None => String::new(),
            Some(x) => x.e()?,
        },
        _ => String::new(),
    };
    let anch_text = if anch.is_empty() {
        "<b>なし（または索引が空）＝差分検査は「まだ分からない」</b>".to_string()
    } else {
        esc(&anch.join(" → "))
    };
    o.push(format!(
        "<h2 id=\"adr\">判断の記録（ADR・{} 件・正本 <code>adr/ADR-n.yaml</code>・欄の決まりは <code>adr/schema.yaml</code>）</h2><p class=\"note\">{adr_def} 凍結 anchor（差分検査の比較元・索引 <code>anchors/index.yaml</code> の列・古い順）: {anch_text}</p>",
        adrs.len()
    ));
    for d in &adrs {
        let (sn, scol) = d.f("status")?.lookup(AST, "判断の記録の状態")?;
        o.push(format!(
            "<section class=\"art\" id=\"{}\"><h4><span class=\"gid\">{}</span>{}<span class=\"tier\" style=\"background:{scol}\">{sn}</span><span class=\"bind\">{}</span></h4>",
            d.ef("id")?,
            d.ef("id")?,
            d.ef("title")?,
            d.esf("date")?
        ));
        o.push(format!(
            "<p class=\"plain\"><b>やさしく言うと</b><br>{}</p><dl>",
            d.ef("plain")?
        ));
        o.push(format!(
            "<dt>何が問題か</dt><dd>{}</dd><dt>何を決めたか</dt><dd>{}</dd>",
            d.ef("context")?,
            d.ef("decision")?
        ));
        let options = d
            .f("options")?
            .seq()?
            .iter()
            .map(|x| {
                Ok(format!(
                    "<b>{}</b>〔{}〕 {} — {}",
                    x.ef("name")?,
                    x.f("verdict")?.lookup(VERD, "案の判定")?,
                    x.ef("text")?,
                    x.ef("reason")?
                ))
            })
            .collect::<R<Vec<_>>>()?
            .join("<br>");
        o.push(format!("<dt>案（採用は 1 つ）</dt><dd>{options}</dd>"));
        let rt = d.f("retreat")?;
        o.push(format!(
            "<dt>根拠</dt><dd>{}</dd><dt>撤退条件</dt><dd>[{}] {}</dd>",
            esc(&d.f("basis")?.join_s("・")?),
            rt.ef("kind")?,
            rt.ef("condition")?
        ));
        if d.t("amends")? {
            let amends = d
                .f("amends")?
                .seq()?
                .iter()
                .map(|e| {
                    Ok(format!(
                        "<b>{}</b> {}.{}: 「{}」→「{}」",
                        esc(&s_opt(e.g("version")?.as_ref())?),
                        e.esf("target")?,
                        esc(&s_opt(e.g("field")?.as_ref())?),
                        e.esf("previous_text")?,
                        e.esf("new_text")?
                    ))
                })
                .collect::<R<Vec<_>>>()?
                .join("<br>");
            o.push(format!("<dt>改訂する条文（版・欄）</dt><dd>{amends}</dd>"));
        }
        if d.t("grill")? {
            let g = d.f("grill")?;
            o.push(format!(
                "<dt>反対側からの確認（grill）</dt><dd>{}・{}・{}<br>{}</dd>",
                g.esf("when")?,
                g.esf("who")?,
                g.esf("where")?,
                g.esf("summary")?
            ));
        }
        if d.t("consequences")? {
            let cs = d
                .f("consequences")?
                .seq()?
                .iter()
                .map(X::es)
                .collect::<R<Vec<_>>>()?
                .join("<br>");
            o.push(format!("<dt>この判断で変わること</dt><dd>{cs}</dd>"));
        }
        let approval = match d.g("approval")? {
            Some(ap) if truthy(ap.v) => esc(&format!(
                "{}・{}・裁定 {}・「{}」（対話面 = rules 行 {}）",
                ap.f("who")?.s()?,
                ap.f("date")?.s()?,
                ap.f("ruling")?.s()?,
                ap.f("verbatim")?.s()?,
                ap.f("surface")?.s()?
            )),
            _ => "未（提案中・持ち主の逐語と日付が入ると発効）".to_string(),
        };
        o.push(format!("<dt>承認</dt><dd>{approval}</dd>"));
        for (k, lab) in [
            ("supersedes", "置き換えた判断"),
            ("superseded_by", "後継の判断"),
        ] {
            if d.t(k)? {
                o.push(format!("<dt>{lab}</dt><dd>{}</dd>", d.esf(k)?));
            }
        }
        if d.t("note")? {
            o.push(format!("<dt>planner 注</dt><dd>{}</dd>", d.esf("note")?));
        }
        o.push("</dl></section>".to_string());
    }
    o.push(format!(
        "<p class=\"sub\">この文書の所属: folio2 / 設計文書（design-intent）の読み物 — 憲法 {ver}・要件書 {sver}。<a href=\"index.html\">入口へ戻る</a></p></div></body></html>"
    ));
    Ok(o.join("\n"))
}

/// `'・'.join(x) or '—'`
fn or_dash(s: &str) -> &str {
    if s.is_empty() { "—" } else { s }
}

// ── write / check（mode）──

pub fn run(dir: &Path, out: &Path, mode: Mode) -> Outcome {
    // --out が相対なら --dir からの相対（script が --dir へ移ってから開くのと同じ）・絶対ならそのまま
    let out_path = dir.join(out);
    let html = match derive(dir) {
        Ok(h) => h,
        Err(e) => return Outcome::unknown(e),
    };
    let size = html.len();
    match mode {
        Mode::Check => {
            if !out_path.exists() {
                return Outcome {
                    verdict: Verdict::Unknown,
                    stdout: None,
                    stderr: Some("render: 読み物が無い（未生成）".to_string()),
                };
            }
            let cur = match fs::read(&out_path) {
                Ok(b) => b,
                Err(e) => {
                    return Outcome::unknown(format!("{}: 読めない: {e}", out_path.display()));
                }
            };
            if cur != html.as_bytes() {
                return Outcome {
                    verdict: Verdict::Fail,
                    stdout: None,
                    stderr: Some(format!(
                        "render: DRIFT — 読み物 {} byte ≠ 導出 {size} byte（手で直したか正本が変わった）",
                        cur.len()
                    )),
                };
            }
            Outcome {
                verdict: Verdict::Pass,
                stdout: Some(format!("render: OK — 読み物は正本と一致（{size} byte）")),
                stderr: None,
            }
        }
        Mode::Write => {
            if let Err(e) = fs::write(&out_path, &html) {
                return Outcome::unknown(format!("{}: 書けない: {e}", out_path.display()));
            }
            Outcome {
                verdict: Verdict::Pass,
                stdout: Some(format!("rendered bytes {size}")),
                stderr: None,
            }
        }
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    #[test]
    fn render_escape_is_the_five_chars_of_html_escape() {
        assert_eq!(esc("a&b<c>d\"e'f あ"), "a&amp;b&lt;c&gt;d&quot;e&#x27;f あ");
        assert_eq!(esc(""), "");
    }

    #[test]
    fn render_truthy_follows_python() {
        let s = |x: &str| Value::Str(x.to_string());
        for falsy in [
            Value::Null,
            Value::Bool(false),
            Value::Int("0".into()),
            Value::Int("-0".into()),
            Value::Float(0.0),
            s(""),
            Value::Seq(vec![]),
            Value::Map(vec![]),
        ] {
            assert!(!truthy(&falsy), "{falsy:?}");
        }
        for truthy_v in [
            Value::Bool(true),
            Value::Int("1".into()),
            Value::Int("-2".into()),
            Value::Float(0.5),
            Value::Date("2026-09-17".into()),
            s(" "),
            s("0"),
            Value::Seq(vec![Value::Null]),
            Value::Map(vec![(s("a"), Value::Null)]),
        ] {
            assert!(truthy(&truthy_v), "{truthy_v:?}");
        }
    }

    #[test]
    fn render_repr_is_a_narrow_copy_that_fails_closed() {
        let s = |x: &str| Value::Str(x.to_string());
        let ok = Value::Seq(vec![
            Value::Map(vec![
                (
                    s("CON1（順序）を改訂した"),
                    s("「a」→「b」。P-7.2 ／ x ＝ y"),
                ),
                (s("n"), Value::Int("3".into())),
            ]),
            Value::Null,
            Value::Bool(true),
        ]);
        assert_eq!(
            py_str(&ok).unwrap(),
            "[{'CON1（順序）を改訂した': '「a」→「b」。P-7.2 ／ x ＝ y', 'n': 3}, None, True]"
        );
        for ng in [
            Value::Seq(vec![s("it's")]),
            Value::Seq(vec![Value::Date("2026-09-17".into())]),
            Value::Seq(vec![s("全角　空白")]),
            Value::Seq(vec![s("back\\slash")]),
            Value::Seq(vec![Value::Float(1.5)]),
        ] {
            assert!(py_str(&ng).is_err(), "{ng:?}");
        }
        // scalar は py_str のまま
        assert_eq!(py_str(&Value::Float(1.5)).unwrap(), "1.5");
        assert_eq!(py_str(&s("it's")).unwrap(), "it's");
    }

    #[test]
    fn render_adr_order_is_numeric_not_lexical() {
        let doc = |id: &str| {
            (
                format!("adr/{id}.yaml"),
                yaml::parse_typed(&format!("id: {id}\n")).unwrap(),
            )
        };
        let ordered = order_adrs(vec![doc("ADR-10"), doc("ADR-2"), doc("ADR-1")]).unwrap();
        let ids: Vec<_> = ordered
            .iter()
            .map(|d| d.get("id").unwrap().as_str().unwrap().to_string())
            .collect();
        assert_eq!(ids, ["ADR-1", "ADR-2", "ADR-10"]);
        assert!(order_adrs(vec![doc("ADR-2"), doc("ADR-02")]).is_err());
        assert!(order_adrs(vec![doc("ADR-x")]).is_err());
        assert!(order_adrs(vec![doc("ADR")]).is_err());
        assert_eq!(adr_number("ADR-7"), Some(7));
        assert_eq!(adr_number("ADR- 7"), None);
    }
}
