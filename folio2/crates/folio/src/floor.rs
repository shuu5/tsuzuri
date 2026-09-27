//! 欄の決まりの床の機械（便 106・docs/design/delivery-106.md §1 (b)・ADR-15 決定 (2)(3)・責務の層 1 読む）。
//! 床の定数の木の型 `Floor`・欄の集合を木に写すマクロ `keys_floor`・注釈の欄を落とす `strip_notes`・正本の木と
//! 突き合わせる `floor_diff`・木から生成区間の本文を組む `derive` を持つ。`schema.rs`（便 45）から字を変えずに降ろした。
//! 正本の形だけを知り、何も書かない。床の検査の側（`adr.rs`・`note.rs` ほか）と `folio schema` の口が使う。
//! 生成区間の印と区間を取る口は `schema.rs` に残る。
//!
//! 導出の体裁（§1 (c)・W = 100 字・字数は Unicode の字の数・行は字下げ込み）:
//! 1. 表の block の子は「<字下げ><キー>: <値>」・字下げは深さ × 2・キーの順は FLOOR の順。
//! 2. 文字列と数の一覧: flow の 1 行が W 以内なら flow、超えれば block（各項「<字下げ + 2>- <値>」）。空は []。
//! 3. 表: 子が全部 文字列・数・真偽 か その一覧 で flow の 1 行が W 以内なら flow、さもなくば block。空は {}。
//! 4. 表の一覧: 各項を 3 の flow で「<字下げ + 2>- {…}」、W を超える項は block（1 つ目のキーを「- 」の後ろに）。
//! 5. 値の字面: 数と真偽は裸。文字列は yaml で裸にできない形のときだけ単引用符（中の単引用符は 2 つ重ねる）。
//! 6. 置き場の名で行を選ぶ表（`Floor::Pick`・便 121・ADR-16 決定 (2)(ア)）: 写すのは鍵が置き場の名と等しい行だけ。
//!    行が無ければ {}、在れば flow の 1 行が W 以内なら flow、超えれば block（欄名の行と「<字下げ + 2><名>: <値>」の行）。
//! 7. 外の置き場（名が folio2 の置き場の名 `HOME` でない・便 174・ADR-16 決定 (2)(オ)）: 文字列の値は `text_for` を通し
//!    （folio2 の番号の印を持つ全角の括弧の項と文を落とす・注の欄で何も残らなければ欄ごと書かない）、`Floor::Home` の欄は書かない。
//!    folio2 の置き場と名の無い口は定数の字のまま（folio2 の生成区間は変わらない）。突き合わせ（`floor_diff_for`）も同じ規則で比べる。

use std::borrow::Cow;

use crate::yaml::Node;

/// 1 行の幅の上限（Unicode の字の数）。
const WIDTH: usize = 100;

/// folio2 自身の置き場の名（憲法 meta.id・列の根の表の folio2 の行の鍵と同じ字）。この名の置き場だけが定数の字のまま写す。
pub(crate) const HOME: &str = "folio2-constitution";

/// 外の置き場でも床が id で名指す行（ADR-16 決定 (2)(オ)＝対話面の行と散文の門の行）。印に数えない。
const RESERVED: [&str; 2] = ["R-8", "R-16"];

// ── 床の機械 ──

/// 床の定数の木（欄の決まりの file の schema 節と同じ形）。
#[derive(Debug)]
pub(crate) enum Floor {
    /// 値（yaml の値の字面・引用符を除く）
    Val(&'static str),
    /// 数（字面で比べる＝2.0 と 2 は違う）
    Num(usize),
    /// 値の一覧
    Strs(&'static [&'static str]),
    /// 木の一覧（順も比べる）。設計ノートの側（landing.verdict_cases）が使う
    Seq(&'static [Floor]),
    /// 表（欄の順は schema 節の順）
    Map(&'static [(&'static str, Floor)]),
    /// 置き場の名で行を選ぶ表（鍵 = 置き場の名・値 = 字）。写しと突き合わせは鍵が置き場の名と等しい行だけ
    /// （0 行か 1 行・名が無いか表に無ければ空の表）。判断の記録の列の根の表（便 121）が使う
    Pick(&'static [(&'static str, &'static str)]),
    /// folio2 の置き場にだけ在る欄（便 174）。値が folio2 の規則の表の行を名指す欄で、外の置き場では書かず、在れば未知の欄
    Home(&'static Floor),
}

/// 外の置き場か（名が在って `HOME` でない）。名の無い口（床の単体の歯と名なしの突き合わせ）は定数の字のまま。
fn abroad(name: Option<&str>) -> bool {
    name.is_some_and(|n| n != HOME)
}

/// 字の中の id の形（前の字が英字でないもの）を全部取る（便 125 の骨格の口から便 174 で降ろした・骨格と外の置き場の字が共有する）。形 = ADR- と数・便 と数（間の空白は任意）・
/// FR / NFR / AC / CON に続く数・P- / N- / A- に数（. と数が続いてもよい）・R- / D- に数。
pub(crate) fn ids_in(text: &str) -> Vec<String> {
    let c: Vec<char> = text.chars().collect();
    let digits = |from: usize| c[from..].iter().take_while(|x| x.is_ascii_digit()).count();
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if i > 0 && c[i - 1].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let rest: String = c[i..c.len().min(i + 4)].iter().collect();
        let mut found: Option<(usize, String)> = None;
        if rest.starts_with("ADR-") && digits(i + 4) > 0 {
            let n = digits(i + 4);
            found = Some((4 + n, c[i..i + 4 + n].iter().collect()));
        } else if c[i] == '便' {
            let sp = c[i + 1..].iter().take_while(|x| **x == ' ').count();
            let n = digits(i + 1 + sp);
            if n > 0 {
                let num: String = c[i + 1 + sp..i + 1 + sp + n].iter().collect();
                found = Some((1 + sp + n, format!("便{num}")));
            }
        } else if let Some(p) = ["NFR", "CON", "FR", "AC"]
            .iter()
            .find(|p| rest.starts_with(**p) && digits(i + p.chars().count()) > 0)
        {
            let len = p.chars().count();
            let n = digits(i + len);
            found = Some((len + n, c[i..i + len + n].iter().collect()));
        } else if matches!(c[i], 'P' | 'N' | 'A' | 'R' | 'D')
            && c.get(i + 1) == Some(&'-')
            && digits(i + 2) > 0
        {
            let mut len = 2 + digits(i + 2);
            if matches!(c[i], 'P' | 'N' | 'A') && c.get(i + len) == Some(&'.') && digits(i + len + 1) > 0
            {
                len += 1 + digits(i + len + 1);
            }
            found = Some((len, c[i..i + len].iter().collect()));
        }
        match found {
            Some((len, id)) => {
                out.push(id);
                i += len;
            }
            None => i += 1,
        }
    }
    out
}

/// 台帳の id の形の語（英小字 1 字・数字 1 字・「-」・英数字。前は頭か空白か全角の字＝正規表現の字面の中は数えない）を持つか。
pub(crate) fn has_ledger_id(t: &str) -> bool {
    let c: Vec<char> = t.chars().collect();
    (0..c.len()).any(|i| {
        (i == 0 || c[i - 1].is_ascii_whitespace() || !c[i - 1].is_ascii())
            && c[i].is_ascii_lowercase()
            && c.get(i + 1).is_some_and(char::is_ascii_digit)
            && c.get(i + 2) == Some(&'-')
            && c.get(i + 3).is_some_and(char::is_ascii_alphanumeric)
    })
}

/// folio2 の番号の印を持つか。印 = 骨格と同じ id の形（`ids_in`・条・要件・規則の表の行・判断の記録・便）のうち
/// `RESERVED` でないもの・判断の記録の決定の番号（「決定 (」）・台帳の id の形。
fn marked(t: &str) -> bool {
    ids_in(t).iter().any(|id| !RESERVED.contains(&id.as_str()))
        || t.contains("決定 (")
        || has_ledger_id(t)
}

/// 括弧の深さ（全角の丸括弧・亀甲括弧・鉤括弧）の増減。
fn depth_step(ch: char) -> i32 {
    match ch {
        '（' | '〔' | '「' => 1,
        '）' | '〕' | '」' => -1,
        _ => 0,
    }
}

/// 深さ 0 の `sep` の後ろで切る（`sep` は前の片に残す）。
fn split_after(c: &[char], sep: char) -> Vec<String> {
    let mut out = Vec::new();
    let (mut depth, mut cur) = (0, String::new());
    for &ch in c {
        depth += depth_step(ch);
        cur.push(ch);
        if depth == 0 && ch == sep {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// 全角の丸括弧ごとに、中の片のうち印を持つ片を落とす（入れ子は内から）。片は、中に深さ 0 の「。」が在れば文（「。」まで）、
/// 無ければ「・」で切った項。片が 1 つも残らなければ括弧ごと落とす。閉じない括弧から後ろはそのまま。
fn drop_marked_items(c: &[char]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < c.len() {
        if c[i] == '（' {
            let mut depth = 0;
            let close = (i..c.len()).find(|&j| {
                depth += i32::from(c[j] == '（') - i32::from(c[j] == '）');
                depth == 0
            });
            if let Some(j) = close {
                let inner: Vec<char> = drop_marked_items(&c[i + 1..j]).chars().collect();
                let sentences = split_after(&inner, '。');
                let kept: String = if sentences.len() > 1 {
                    sentences.into_iter().filter(|s| !marked(s)).collect()
                } else {
                    split_after(&inner, '・')
                        .into_iter()
                        .map(|item| item.strip_suffix('・').unwrap_or(&item).to_string())
                        .filter(|item| !item.is_empty() && !marked(item))
                        .collect::<Vec<_>>()
                        .join("・")
                };
                if !kept.is_empty() {
                    out.push_str(&format!("（{kept}）"));
                }
                i = j + 1;
                continue;
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// 外の置き場へ写す字（便 174）: 括弧の項を落とした後、なお印を持つ文（深さ 0 の「。」まで）を落とす。何も残らなければ None。
pub(crate) fn text_for(v: &str) -> Option<String> {
    let c: Vec<char> = v.chars().collect();
    let kept: Vec<char> = drop_marked_items(&c).chars().collect();
    let out: String = split_after(&kept, '。')
        .into_iter()
        .filter(|s| !marked(s))
        .collect();
    let out = out.trim();
    (!out.is_empty()).then(|| out.to_string())
}

/// 置き場へ写す値の字。folio2 の置き場と名の無い口は定数のまま。外の置き場は `text_for` で、何も残らないとき注の中（`note`）は
/// None（書かない）、注の外は空の字（床の単体の歯が型付きの欄で起きないことを見る）。
fn val_for(v: &'static str, name: Option<&str>, note: bool) -> Option<Cow<'static, str>> {
    if !abroad(name) {
        return Some(Cow::Borrowed(v));
    }
    match text_for(v) {
        Some(t) => Some(Cow::Owned(t)),
        None if note => None,
        None => Some(Cow::Borrowed("")),
    }
}

/// 一覧の各項を置き場へ写した字（注の中で何も残らない項は落とす）。
fn strs_for(
    items: &'static [&'static str],
    name: Option<&str>,
    note: bool,
) -> Vec<Cow<'static, str>> {
    items
        .iter()
        .filter_map(|v| val_for(v, name, note))
        .collect()
}

/// 欄を書くか（`note` は注の欄の中か）。外の置き場の `Home` と、注の中で何も残らない値・一覧・表は書かない。
fn emits(node: &Floor, name: Option<&str>, note: bool) -> bool {
    match node {
        Floor::Home(inner) => !abroad(name) && emits(inner, name, note),
        Floor::Val(v) => val_for(v, name, note).is_some(),
        Floor::Strs(items) if note && !items.is_empty() => !strs_for(items, name, note).is_empty(),
        Floor::Map(fields) if note && !fields.is_empty() => fields
            .iter()
            .any(|(k, v)| emits(v, name, note || k.ends_with("_note"))),
        _ => true,
    }
}

/// `Home` を剥いだ木（folio2 の置き場で書くときの形）。
fn bare(node: &Floor) -> &Floor {
    match node {
        Floor::Home(inner) => bare(inner),
        other => other,
    }
}

/// 置き場の名で選んだ行（0 行か 1 行）。
fn picked(
    rows: &'static [(&'static str, &'static str)],
    name: Option<&str>,
) -> Vec<(&'static str, &'static str)> {
    rows.iter()
        .filter(|(k, _)| Some(*k) == name)
        .copied()
        .collect()
}

/// 欄の集合（required / optional）を床の木の表に写す。
macro_rules! keys_floor {
    ($keys:expr) => {
        $crate::floor::Floor::Map(&[
            ("required", $crate::floor::Floor::Strs($keys.required)),
            ("optional", $crate::floor::Floor::Strs($keys.optional)),
        ])
    };
}
pub(crate) use keys_floor;

/// 名前が `_note` で終わる欄を（入れ子の表の中も含めて）落とす。
pub(crate) fn strip_notes(node: &Node) -> Node {
    match node {
        Node::Map(entries) => Node::Map(
            entries
                .iter()
                .filter(|(k, _)| !k.ends_with("_note"))
                .map(|(k, v)| (k.clone(), strip_notes(v)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// 写しと床の定数の違いを欄の道で並べる。床の側の `_note` の欄（説明の注）は突き合わせの外（data 側は strip_notes 済み）。
/// 置き場の名を持たない口＝名で行を選ぶ表は空の表と突き合わせる（`floor_diff_for` の名なし）。
pub(crate) fn floor_diff(data: &Node, floor: &Floor, path: &str, out: &mut Vec<String>) {
    floor_diff_for(data, floor, None, path, out);
}

/// 名つきの突き合わせ（便 121）。`name` は検査される置き場の憲法の名（meta.id）で、名で行を選ぶ表はその行だけと比べる。
/// 余分な行・欠けた行・値の違い・表でない値はどれも表の道 1 つ。
pub(crate) fn floor_diff_for(
    data: &Node,
    floor: &Floor,
    name: Option<&str>,
    path: &str,
    out: &mut Vec<String>,
) {
    match floor {
        Floor::Map(fields) => {
            let Some(entries) = data.as_map() else {
                out.push(format!(
                    "{}（欄の表でない）",
                    if path.is_empty() { "schema" } else { path }
                ));
                return;
            };
            let mut keys: Vec<&str> = entries
                .iter()
                .map(|(k, _)| k.as_str())
                .chain(
                    fields
                        .iter()
                        .map(|(k, _)| *k)
                        .filter(|k| !k.ends_with("_note")),
                )
                .collect();
            keys.sort_unstable();
            keys.dedup();
            for key in keys {
                let p = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };
                // 外の置き場の `Home` の欄は無いのが正しく、在れば未知の欄
                let field = fields
                    .iter()
                    .find(|(k, _)| *k == key)
                    .filter(|(_, f)| !matches!(f, Floor::Home(_)) || !abroad(name));
                match (field, data.get(key)) {
                    (None, None) => {}
                    (None, Some(_)) => out.push(format!(
                        "{p}（未知の欄＝機械が読まない欄は *_note で終える）"
                    )),
                    (Some(_), None) => out.push(format!("{p}（欠落）")),
                    (Some((_, f)), Some(d)) => floor_diff_for(d, bare(f), name, &p, out),
                }
            }
        }
        Floor::Seq(items) => match data.as_seq() {
            Some(seq) if seq.len() == items.len() => {
                for (i, (d, f)) in seq.iter().zip(items.iter()).enumerate() {
                    floor_diff_for(d, f, name, &format!("{path}[{i}]"), out);
                }
            }
            _ => out.push(path.to_string()),
        },
        Floor::Pick(rows) => {
            let want = picked(rows, name);
            let same = data.as_map().is_some_and(|entries| {
                entries.len() == want.len()
                    && want
                        .iter()
                        .all(|(k, v)| data.get(k).and_then(Node::as_str) == Some(*v))
            });
            if !same {
                out.push(path.to_string());
            }
        }
        Floor::Strs(items) => {
            let items = strs_for(items, name, false);
            match data.as_seq() {
                Some(seq) if seq.len() == items.len() => {
                    for (i, (d, v)) in seq.iter().zip(items.iter()).enumerate() {
                        if d.as_str() != Some(v.as_ref()) {
                            out.push(format!("{path}[{i}]"));
                        }
                    }
                }
                _ => out.push(path.to_string()),
            }
        }
        Floor::Val(v) => {
            if data.as_str() != val_for(v, name, false).as_deref() {
                out.push(path.to_string());
            }
        }
        Floor::Home(inner) => {
            if !abroad(name) {
                floor_diff_for(data, inner, name, path, out);
            }
        }
        Floor::Num(n) => {
            if data.as_str() != Some(n.to_string().as_str()) {
                out.push(path.to_string());
            }
        }
    }
}

// ── 導出（§1 (c)） ──

/// 規則 5: 文字列の字面。裸にできない形のときだけ単引用符で囲む（`flow` = flow の中では読点・括弧も囲む理由）。
fn quoted(s: &str, flow: bool) -> String {
    let quote = s.is_empty()
        || s.starts_with(char::is_whitespace)
        || s.ends_with(char::is_whitespace)
        || s.starts_with([
            '[', '{', '#', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`',
        ])
        || s.starts_with("- ")
        || s.starts_with("? ")
        || s.starts_with(": ")
        || s.contains(": ")
        || s.contains(" #")
        || s.ends_with(':')
        || (flow && s.contains([',', '[', ']', '{', '}']));
    if quote {
        format!("'{}'", s.replace('\'', "''"))
    } else {
        s.to_string()
    }
}

/// 表の子のうち書くもの（`Home` は剥いで・`note` は子が注の欄の中か）。
fn shown<'a>(
    fields: &'a [(&'a str, Floor)],
    name: Option<&str>,
    note: bool,
) -> Vec<(&'a str, &'a Floor, bool)> {
    fields
        .iter()
        .map(|(k, v)| (*k, v, note || k.ends_with("_note")))
        .filter(|(_, v, note)| emits(v, name, *note))
        .map(|(k, v, note)| (k, bare(v), note))
        .collect()
}

/// 木を flow の 1 行に（`name` は置き場の名＝名で行を選ぶ表の行と外の置き場の字を選ぶ・`note` は注の欄の中か）。
fn flow(node: &Floor, name: Option<&str>, note: bool) -> String {
    match node {
        Floor::Val(v) => quoted(&val_for(v, name, note).unwrap_or_default(), true),
        Floor::Num(n) => n.to_string(),
        Floor::Strs(items) => {
            let items: Vec<String> = strs_for(items, name, note)
                .iter()
                .map(|v| quoted(v, true))
                .collect();
            format!("[{}]", items.join(", "))
        }
        Floor::Seq(items) => {
            let items: Vec<String> = items.iter().map(|x| flow(x, name, note)).collect();
            format!("[{}]", items.join(", "))
        }
        Floor::Map(fields) => {
            let fields: Vec<String> = shown(fields, name, note)
                .into_iter()
                .map(|(k, v, note)| format!("{k}: {}", flow(v, name, note)))
                .collect();
            format!("{{{}}}", fields.join(", "))
        }
        Floor::Pick(rows) => {
            let rows: Vec<String> = picked(rows, name)
                .iter()
                .map(|(k, v)| format!("{k}: {}", quoted(v, true)))
                .collect();
            format!("{{{}}}", rows.join(", "))
        }
        Floor::Home(inner) => flow(inner, name, note),
    }
}

/// 規則 3 の flow の条件: 子が全部 文字列・数・真偽 か その一覧。
fn flat(fields: &[(&str, &Floor, bool)]) -> bool {
    fields
        .iter()
        .all(|(_, v, _)| matches!(v, Floor::Val(_) | Floor::Num(_) | Floor::Strs(_)))
}

fn fits(line: &str) -> bool {
    line.chars().count() <= WIDTH
}

/// 表を block で書く（各子を 1 行以上・字下げ `indent`）。
fn block_map(
    fields: &[(&str, Floor)],
    indent: usize,
    name: Option<&str>,
    note: bool,
    out: &mut Vec<String>,
) {
    let pad = " ".repeat(indent);
    for (key, value, note) in shown(fields, name, note) {
        match value {
            Floor::Val(v) => out.push(format!(
                "{pad}{key}: {}",
                quoted(&val_for(v, name, note).unwrap_or_default(), false)
            )),
            Floor::Num(n) => out.push(format!("{pad}{key}: {n}")),
            Floor::Strs(items) => {
                let items = strs_for(items, name, note);
                let line = format!("{pad}{key}: {}", flow(value, name, note));
                if items.is_empty() || fits(&line) {
                    out.push(line);
                } else {
                    out.push(format!("{pad}{key}:"));
                    for item in &items {
                        out.push(format!("{pad}  - {}", quoted(item, false)));
                    }
                }
            }
            Floor::Map(sub) => {
                let line = format!("{pad}{key}: {}", flow(value, name, note));
                let kids = shown(sub, name, note);
                if kids.is_empty() || (flat(&kids) && fits(&line)) {
                    out.push(line);
                } else {
                    out.push(format!("{pad}{key}:"));
                    block_map(sub, indent + 2, name, note, out);
                }
            }
            Floor::Seq(items) => {
                if items.is_empty() {
                    out.push(format!("{pad}{key}: []"));
                } else {
                    out.push(format!("{pad}{key}:"));
                    for item in items.iter() {
                        block_item(item, indent + 2, name, note, out);
                    }
                }
            }
            Floor::Pick(rows) => {
                let rows = picked(rows, name);
                let line = format!("{pad}{key}: {}", flow(value, name, note));
                if rows.is_empty() || fits(&line) {
                    out.push(line);
                } else {
                    out.push(format!("{pad}{key}:"));
                    for (k, v) in rows {
                        out.push(format!("{pad}  {k}: {}", quoted(v, false)));
                    }
                }
            }
            // `shown` が剥いだ後なので来ない
            Floor::Home(_) => {}
        }
    }
}

/// 規則 4: 一覧の項 1 つ（「- 」の行から）。表の項は flow が W に収まればその 1 行、さもなくば block
/// （1 つ目のキーを「- 」の後ろに、残りのキーを同じ列に）。
fn block_item(item: &Floor, indent: usize, name: Option<&str>, note: bool, out: &mut Vec<String>) {
    let pad = " ".repeat(indent);
    match item {
        Floor::Map(sub) if !sub.is_empty() => {
            let line = format!("{pad}- {}", flow(item, name, note));
            if flat(&shown(sub, name, note)) && fits(&line) {
                out.push(line);
                return;
            }
            let first = out.len();
            block_map(sub, indent + 2, name, note, out);
            out[first].replace_range(indent..indent + 2, "- ");
        }
        Floor::Val(v) => out.push(format!(
            "{pad}- {}",
            quoted(&val_for(v, name, note).unwrap_or_default(), false)
        )),
        other => out.push(format!("{pad}- {}", flow(other, name, note))),
    }
}

/// 床の定数 → 生成区間の本文（決定的）。「schema:」の行 + 規則で組んだ本体・各行の末尾は改行 1 つ。
/// 置き場の名を持たない口＝名で行を選ぶ表は空の表（`derive_for` の名なしと同じ字）。
/// 命令の口（`schema.rs`）は名つきの導出を呼ぶので、この口を読むのは各床の単体の歯だけ。
#[cfg_attr(not(test), allow(dead_code))]
pub fn derive(floor: &Floor) -> String {
    derive_for(floor, None)
}

/// 名つきの導出（便 121・ADR-16 決定 (2)(ア)・便 174）。`name` は置き場の憲法の名（meta.id）で、名で行を選ぶ表はその行だけを
/// 写し、外の置き場（`HOME` でない名）は規則 7 の字で書く。
pub fn derive_for(floor: &Floor, name: Option<&str>) -> String {
    let mut lines = vec!["schema:".to_string()];
    match floor {
        Floor::Map(fields) => block_map(fields, 2, name, false, &mut lines),
        other => lines[0] = format!("schema: {}", flow(other, name, false)),
    }
    lines.iter().map(|l| format!("{l}\n")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml;

    #[test]
    fn schema_floor_diff_compares_literally_and_skips_floor_notes() {
        let doc = yaml::parse("min: '2'\nadopted: 2.0\nextra: x\n").unwrap();
        let mut out = Vec::new();
        floor_diff(
            &doc.root,
            &Floor::Map(&[
                ("min", Floor::Num(2)),
                ("min_note", Floor::Val("説明")),
                ("adopted", Floor::Num(1)),
                ("x", Floor::Strs(&[])),
            ]),
            "options_rule",
            &mut out,
        );
        assert_eq!(
            out,
            [
                "options_rule.adopted",
                "options_rule.extra（未知の欄＝機械が読まない欄は *_note で終える）",
                "options_rule.x（欠落）"
            ]
        );
    }

    #[test]
    fn schema_quoting_follows_rule_5() {
        assert_eq!(quoted("持ち主", false), "持ち主");
        assert_eq!(quoted("[a-z]", false), "'[a-z]'");
        assert_eq!(quoted("", false), "''");
        assert_eq!(quoted("a: b", false), "'a: b'");
        assert_eq!(quoted("a #b", false), "'a #b'");
        assert_eq!(quoted("a:", false), "'a:'");
        assert_eq!(quoted("- a", false), "'- a'");
        assert_eq!(quoted("it's", true), "it's");
        assert_eq!(quoted("'it's", true), "'''it''s'");
        assert_eq!(quoted("a, b", false), "a, b");
        assert_eq!(quoted("a, b", true), "'a, b'");
        assert_eq!(quoted("x {y}", false), "x {y}");
        assert_eq!(quoted("x {y}", true), "'x {y}'");
    }

    #[test]
    fn schema_layout_switches_flow_and_block_at_the_width() {
        const LONG: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz";
        const F: Floor = Floor::Map(&[
            ("n", Floor::Num(2)),
            ("s", Floor::Strs(&["a", "b"])),
            ("e", Floor::Strs(&[])),
            ("m", Floor::Map(&[])),
            ("wide", Floor::Strs(&[LONG, LONG])),
            (
                "nest",
                Floor::Map(&[("k", Floor::Val("v")), ("t", Floor::Map(&[]))]),
            ),
            (
                "rows",
                Floor::Seq(&[
                    Floor::Map(&[("a", Floor::Val("1")), ("b", Floor::Num(2))]),
                    Floor::Map(&[("a", Floor::Val(LONG)), ("b", Floor::Val(LONG))]),
                ]),
            ),
        ]);
        let want = format!(
            "schema:\n  n: 2\n  s: [a, b]\n  e: []\n  m: {{}}\n  wide:\n    - {LONG}\n    - {LONG}\n  nest:\n    k: v\n    t: {{}}\n  rows:\n    - {{a: 1, b: 2}}\n    - a: {LONG}\n      b: {LONG}\n"
        );
        assert_eq!(derive(&F), want);
    }

    /// 便 121 の歯 8: 置き場の名で行を選ぶ表の導出と突き合わせ。行 a は flow の 1 行・行 b は幅を超えて block の 2 行・
    /// 表に無い名と名なしは空の表の 1 行（名なしは `derive` と同じ字）。突き合わせは名の行だけと比べる。
    #[test]
    fn f121_pick_table_derives_and_diffs_only_the_named_row() {
        const WIDE: &str =
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        const F: Floor = Floor::Map(&[
            ("n", Floor::Num(1)),
            ("roots", Floor::Pick(&[("a", "short"), ("b", WIDE)])),
        ]);
        assert_eq!(
            derive_for(&F, Some("a")),
            "schema:\n  n: 1\n  roots: {a: short}\n"
        );
        assert_eq!(
            derive_for(&F, Some("b")),
            format!("schema:\n  n: 1\n  roots:\n    b: {WIDE}\n")
        );
        let empty = "schema:\n  n: 1\n  roots: {}\n";
        assert_eq!(derive_for(&F, Some("c")), empty);
        assert_eq!(derive_for(&F, None), empty);
        assert_eq!(derive(&F), empty);

        let diff = |text: &str, name: Option<&str>| {
            let doc = yaml::parse(text).unwrap();
            let mut out = Vec::new();
            floor_diff_for(&doc.root, &F, name, "", &mut out);
            out
        };
        assert!(diff("n: 1\nroots: {a: short}\n", Some("a")).is_empty());
        assert!(diff(&format!("n: 1\nroots:\n  b: {WIDE}\n"), Some("b")).is_empty());
        assert!(diff("n: 1\nroots: {}\n", Some("c")).is_empty());
        assert!(diff("n: 1\nroots: {}\n", None).is_empty());
        for (text, name) in [
            ("n: 1\nroots: {a: short}\n", Some("c")),
            ("n: 1\nroots: {a: short}\n", None),
            ("n: 1\nroots: {a: long}\n", Some("a")),
            ("n: 1\nroots: {a: short, b: x}\n", Some("a")),
            ("n: 1\nroots: {}\n", Some("a")),
            ("n: 1\nroots: short\n", Some("a")),
        ] {
            assert_eq!(diff(text, name), ["roots"], "{text} {name:?}");
        }
    }

    /// 便 174 の歯 1: 外の置き場の字。括弧の項・括弧の中の文・括弧の外の文の 3 段で印を落とし、予約の行と正規表現の字面は残す。
    #[test]
    fn f174_text_for_drops_folio2_numbers_and_keeps_the_rest() {
        for (from, to) in [
            ("廃止（superseded_by 必須・P-7.2・承認欄を持つ）", Some("廃止（superseded_by 必須・承認欄を持つ）")),
            ("P-8.1。床は非空と kind の値域を見る", Some("床は非空と kind の値域を見る")),
            ("対話面（R-8）を通っていない記録（P-12.3）", Some("対話面（R-8）を通っていない記録")),
            ("導出物である（判断の記録 ADR-13 決定 (4)・P-6.3 / P-6.4）", Some("導出物である")),
            ("捨てず（semantic_attrs = keep・決定 (3)）", Some("捨てず（semantic_attrs = keep）")),
            ("出す（実装 x.rs・台帳 f2-648.132）。便 119 で入った。", Some("出す（実装 x.rs）。")),
            ("同じ（違えば → A-2。測らない。ただし条の消失・改番は測る）", Some("同じ（測らない。ただし条の消失・改番は測る）")),
            ("^[a-z][a-z0-9-]*$", Some("^[a-z][a-z0-9-]*$")),
            ("A-2.3 の記録（条文を改訂する発効した判断に必須）", None),
        ] {
            assert_eq!(text_for(from).as_deref(), to, "{from}");
        }
        assert!(!abroad(None) && !abroad(Some(HOME)) && abroad(Some("x-constitution")));
    }
}
