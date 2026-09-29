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
//!    落とした跡の字の壊れは残さない（便 194）: 括弧ごと落として英数字と和字が接したら空白を 1 つ置き、落とした文の直後の文が
//!    前の文を指す語で始まれば、語が文の主なら（`POINTERS`）文も続けて落とし、抜いても文が立つ語なら（`POINTER_ADVERBS`）語だけを
//!    落とし、同じ括弧の中で台帳の id の項を落としたら「持ち主の裁定」の項も落とす。
//!    folio2 の置き場と名の無い口は定数の字のまま（folio2 の生成区間は変わらない）。突き合わせ（`floor_diff_for`）も同じ規則で比べる。

use std::borrow::Cow;

use crate::ruling::has_ruling;
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
/// 床の知らせ・違反の字と名札・まだ分からない の行（`said` と `rules::Labels`・便 202・便 203）も同じ関数で決める。
pub(crate) fn abroad(name: Option<&str>) -> bool {
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

/// folio2 の番号の印を持つか。印 = 骨格と同じ id の形（`ids_in`・条・要件・規則の表の行・判断の記録・便）のうち
/// `RESERVED` でないもの・判断の記録の決定の番号（「決定 (」）・台帳の id の形（決定の欄の床と同じ文法・便 181）。
fn marked(t: &str) -> bool {
    ids_in(t).iter().any(|id| !RESERVED.contains(&id.as_str()))
        || t.contains("決定 (")
        || has_ruling(t)
}

/// 落とした文の直後で、続けて落とす文の頭の語（前の文を指す語が文の主で、語だけは抜けない・便 194）。
const POINTERS: [&str; 2] = ["これ", "その"];

/// 落とした文の直後の文の頭で、語だけを落とす指す語（抜いても文が立つ＝文の中身は残す・便 194）。
const POINTER_ADVERBS: [&str; 1] = ["どちらも"];

/// 出所の台帳の id を落とした括弧で、一緒に落とす項の字（出所の無い裁定の名指しを残さない・便 194）。
const OWNER_RULING: &str = "持ち主の裁定";

/// 和字（平仮名・片仮名の字と長音・漢字）か。英数字と接したら間に空白を置く相手（便 194）。
fn wa(ch: char) -> bool {
    matches!(ch, '\u{3041}'..='\u{3096}' | '\u{30a1}'..='\u{30fa}' | 'ー' | '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}')
}

/// 括弧ごと落とした跡の前後の字が、英数字と和字（か英数字どうし）で空白なしに接するか。
fn joins(before: Option<char>, after: Option<char>) -> bool {
    match (before, after) {
        (Some(a), Some(b)) => {
            (a.is_ascii_alphanumeric() && (wa(b) || b.is_ascii_alphanumeric())) || (wa(a) && b.is_ascii_alphanumeric())
        }
        _ => false,
    }
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
/// 無ければ「・」で切った項（台帳の id の項が在れば `OWNER_RULING` を含む項も落とす）。片が 1 つも残らなければ括弧ごと落とし、
/// 前後の字が `joins` なら空白を 1 つ置く。閉じない括弧から後ろはそのまま。
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
                    let items: Vec<String> = split_after(&inner, '・')
                        .into_iter()
                        .map(|item| item.strip_suffix('・').unwrap_or(&item).to_string())
                        .filter(|item| !item.is_empty())
                        .collect();
                    let sourced = items.iter().any(|item| has_ruling(item));
                    items
                        .into_iter()
                        .filter(|item| !marked(item) && !(sourced && item.contains(OWNER_RULING)))
                        .collect::<Vec<_>>()
                        .join("・")
                };
                if !kept.is_empty() {
                    out.push_str(&format!("（{kept}）"));
                } else if joins(out.chars().last(), c.get(j + 1).copied()) {
                    out.push(' ');
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

/// 外の置き場へ写す字（便 174）: 括弧の項を落とした後、なお印を持つ文（深さ 0 の「。」まで）を落とす。落とした文の直後の文が
/// `POINTERS` で始まれば、それも続けて落とし、`POINTER_ADVERBS` で始まれば、その語だけを落とす（便 194）。何も残らなければ None。
pub(crate) fn text_for(v: &str) -> Option<String> {
    let c: Vec<char> = v.chars().collect();
    let kept: Vec<char> = drop_marked_items(&c).chars().collect();
    let mut dropped = false;
    let out: String = split_after(&kept, '。')
        .into_iter()
        .filter_map(|s| {
            let (after, head) = (dropped, s.trim_start());
            dropped = marked(&s) || (after && POINTERS.iter().any(|p| head.starts_with(p)));
            match POINTER_ADVERBS.iter().find_map(|p| head.strip_prefix(p)) {
                _ if dropped => None,
                Some(rest) if after => Some(rest.to_string()),
                _ => Some(s),
            }
        })
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

/// 置き場へ出す床の字の番号の片（便 202・便 203）: 注の外の `val_for`（folio2 の置き場と名の無い口は定数のまま・外の置き場は
/// folio2 の番号の印を持つ括弧の項と文を落とし、何も残らなければ空の字）。知らせ・違反の字・まだ分からない の行が、字の中の
/// folio2 の番号の片だけをこれに通す（置き場の自分の id を持つ片は通さない＝印に数えて文ごと落とさない）。
pub(crate) fn said(v: &'static str, name: Option<&str>) -> Cow<'static, str> {
    val_for(v, name, false).unwrap_or_default()
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

    /// 便 194（delivery-194.md §1 (c)・台帳 f2-648.261）: 落とした跡に字の壊れ（英数字と和字の詰まり・宙に浮く指す語・
    /// 出所の無い持ち主の裁定）を残さない。folio2 の置き場の名だけが定数のまま（名の包含で判定しない）。
    #[test]
    fn f194_text_for_leaves_no_broken_joins() {
        for (from, to) in [
            ("folio2 の他の id（P-1・R-7・FR1）と同じく", Some("folio2 の他の id と同じく")),
            ("読む（ADR-1）file", Some("読む file")),
            ("file（ADR-1）2 本", Some("file 2 本")),
            ("4 桁（ADR-0047）は前の版", Some("4 桁は前の版")),
            ("id（ADR-1）、次", Some("id、次")),
            ("id（ADR-1）・次", Some("id・次")),
            ("口は便 119 で入った。どちらも面を呼ばない。値は字。", Some("面を呼ばない。値は字。")),
            ("口は便 119 で入った。 どちらも面を呼ばない。 これも同じ。", Some("面を呼ばない。 これも同じ。")),
            ("口は便 119 で入った。これも同じ。どちらも面を呼ばない。", Some("面を呼ばない。")),
            ("口は便 119 で入った。そのため足す。これも同じ。値は字。", Some("値は字。")),
            ("口は便 119 で入った。 その値は字。値は字。", Some("値は字。")),
            ("値は字。どちらも面を呼ばない。", Some("値は字。どちらも面を呼ばない。")),
            ("口は便 119 で入った。値は字。どちらも同じ。", Some("値は字。どちらも同じ。")),
            ("図の対（持ち主の裁定 2026-09-19・f2-648 notes）＝図", Some("図の対＝図")),
            ("対（持ち主の裁定 2026-09-19・版 v1）", Some("対（持ち主の裁定 2026-09-19・版 v1）")),
            ("対（持ち主の裁定・f2-648 notes・字）", Some("対（字）")),
        ] {
            assert_eq!(text_for(from).as_deref(), to, "{from}");
        }
        assert_eq!(POINTERS, ["これ", "その"]);
        assert_eq!(POINTER_ADVERBS, ["どちらも"]);
        assert_eq!(OWNER_RULING, "持ち主の裁定");
        assert!(abroad(Some("folio2x-constitution")) && abroad(Some("folio2")) && !abroad(Some(HOME)));
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

    /// 置き場へ出す床の字の番号の片（便 203）: 名の無い口と folio2 の置き場は片のまま、外の置き場は folio2 の番号の項と文を
    /// 落とし、印の無い項は残す（期待の字は手書き）。
    #[test]
    fn f203_said_drops_only_the_folio2_number_pieces_abroad() {
        let pieces: [(&str, &str); 6] = [
            ("（骨格の印・裁定の前＝条 P-17.3）", "（骨格の印）"),
            ("（まだ分からない・P-10.3）", "（まだ分からない）"),
            ("（folio check --freeze-adrs で封を書き、commit する・P-10.3）", "（folio check --freeze-adrs で封を書き、commit する）"),
            ("A-2 / N-4 の", ""),
            ("（FR25）", ""),
            ("・FR25", ""),
        ];
        for (piece, abroad_text) in pieces {
            for name in [None, Some(HOME)] {
                assert_eq!(said(piece, name), piece, "{name:?}");
            }
            for name in ["tsuzuri-constitution", "未記入", "folio2"] {
                assert_eq!(said(piece, Some(name)), abroad_text, "{name}");
            }
        }
    }
}
