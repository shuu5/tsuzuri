//! 設計ノートの行 1 本の近い仕様（設計の層の塊）を組む（行 f-ctx-build・判断の記録 ADR-51 の決定 (1) と (4)・係 w150a の 3.0 の順）。
//! 塊の順は 要件 → 判断の記録 → 規則行 → 条 → 受入基準 → 引けない id。塊の頭は `- <id> <種>`（種は索引の節点の種類の字・
//! 引けない id の塊は `- 引けない id`）、本文は頭に空白 2 つを置いた行。同じ種の塊は見つけた順（行の欄 req・欄 basis・題・
//! section の節の本文・done の順、その後に要件から辿った物）。
//! 名指しは行の欄 req と basis（在れば・行 f-row-basis）と、題・節の本文・done の字の id（refs.rs の `id_end` と link.rs の
//! `adr_end`）。深さは、欄 req の要件から、その basis の条と、その要件を verifies に持つ受入基準までの 2 段で、ほかの名指しは
//! 名指した id の本文だけの 1 段。判断の記録は、字「ADR-n 決定 (k)」の形（間の字「の」・項をつなぐ字「と」「・」「、」・範囲の
//! 字「〜」「から」を含む）の名指しなら決定の項 k だけ（項は行頭の `(k)` から次の行頭の `(数)` の前まで）、項の無い名指しと
//! 欄 basis の id と行頭の項を持たない決定は決定の全文。名前空間は渡された置き場の 1 つだけで、ほかの置き場の同じ id を混ぜない。
//! 引けない id（正本に無い id・条に無い規範文・行頭の項を持つ決定に無い項）は黙って落とさず末の 1 塊に並べる（条 P-7）。
//! 置き場を読むのは `build` だけで、塊の組み `blocks` は読んだ木だけの純関数。

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::floor_note::{CONTRACT_TABLE, PROSE};
use crate::graph::NODE_KINDS;
use crate::link::adr_end;
use crate::refs::{RULE_SECTIONS, SRS_ID_SECTIONS, id_end};
use crate::verdict::Verdict;
use crate::yaml::{self, Node};

/// 引けない id の塊の頭（`- ` の後の字）。
pub(crate) const UNRESOLVED_HEAD: &str = "引けない id";

/// 要件書の節と塊の種（索引の節点の種類の字）。受入基準の節だけは要件の塊でなく受入基準の塊に置く。
const SRS_KINDS: [(&str, &str); 7] = [
    ("goals", NODE_KINDS[3]),
    ("requirements", NODE_KINDS[4]),
    ("nonfunctional", NODE_KINDS[5]),
    ("acceptance", NODE_KINDS[6]),
    ("constraints", NODE_KINDS[7]),
    ("actors", NODE_KINDS[8]),
    ("outputs", NODE_KINDS[9]),
];

/// 項の番号の桁の上限（範囲の字で項を広げすぎない）。
const ITEM_DIGITS: usize = 3;

/// 組めない理由（終了 code を分ける）。
pub(crate) enum Refusal {
    /// 名指しの形が `<ノート>#<行>` でない（空の部分・ノートの名に字「/」か頭の字「.」）・ノートか行が無い（1）。
    Row(String),
    /// 正本かノートが読めない（2・まだ分からない）。
    Canon(String),
}

impl Refusal {
    /// 終了 code と理由の 1 行（名指しの誤りとノートか行が無い は不合格 1・読めない は まだ分からない 2）。
    pub(crate) fn split(self) -> (u8, String) {
        match self {
            Refusal::Row(why) => (Verdict::Fail.exit_code() as u8, why),
            Refusal::Canon(why) => (Verdict::Unknown.exit_code() as u8, why),
        }
    }
}

/// 正本 4 種の読んだ木（憲法・規則の表・要件書と、判断の記録の id と木の組）。
pub(crate) struct Canon {
    pub(crate) constitution: Node,
    pub(crate) rules: Node,
    pub(crate) srs: Node,
    pub(crate) adrs: Vec<(String, Node)>,
}

/// 行の字（欄 req と basis の id と、名指しを拾う字＝題・節の本文・done を改行でつないだ字）。
pub(crate) struct Row {
    pub(crate) req: Vec<String>,
    pub(crate) basis: Vec<String>,
    pub(crate) text: String,
}

/// 置き場 `dir` のノートの行 `named`（`<ノート>#<行>`）の塊の字（行ごとに末に改行）。
pub(crate) fn build(dir: &Path, named: &str) -> Result<String, Refusal> {
    let (note, id) = named
        .split_once('#')
        .filter(|(n, r)| !n.is_empty() && !r.is_empty() && !n.contains('/') && !n.starts_with('.'))
        .ok_or_else(|| Refusal::Row(format!("行の名指し {named} が <ノート>#<行> の形でない")))?;
    let path = dir.join("design-note").join(format!("{note}.yaml"));
    if !path.is_file() {
        return Err(Refusal::Row(format!(
            "ノート {note} が {} に無い",
            path.display()
        )));
    }
    let row = row_of(&load(&path).map_err(Refusal::Canon)?, id)
        .ok_or_else(|| Refusal::Row(format!("ノート {note} の契約表に行 {id} が無い")))?;
    Ok(blocks(&canon(dir).map_err(Refusal::Canon)?, &row))
}

/// 正本 1 file を読む。読めない・最上位が欄の表でないは Err。
fn load(path: &Path) -> Result<Node, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("{} を読めない: {e}", path.display()))?;
    let root = yaml::parse(&text)
        .map_err(|e| format!("{} を読めない: {e}", path.display()))?
        .root;
    root.as_map()
        .map(|_| root.clone())
        .ok_or_else(|| format!("{} の最上位が欄の表でない", path.display()))
}

/// 置き場の正本 4 種（判断の記録は adr/ の直下の ADR-*.yaml を名の byte 順・id は file の名の stem）。
fn canon(dir: &Path) -> Result<Canon, String> {
    let ad = dir.join("adr");
    let mut names: Vec<String> = fs::read_dir(&ad)
        .map_err(|e| format!("{} を読めない: {e}", ad.display()))?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("ADR-") && n.ends_with(".yaml"))
        .collect();
    names.sort();
    let mut adrs = Vec::new();
    for name in names {
        let root = load(&ad.join(&name))?;
        adrs.push((name.trim_end_matches(".yaml").to_string(), root));
    }
    Ok(Canon {
        constitution: load(&dir.join("constitution.yaml"))?,
        rules: load(&dir.join("rules.yaml"))?,
        srs: load(&dir.join("srs.yaml"))?,
        adrs,
    })
}

fn seq<'a>(node: &'a Node, key: &str) -> &'a [Node] {
    node.get(key).and_then(Node::as_seq).unwrap_or_default()
}

fn text<'a>(node: &'a Node, key: &str) -> &'a str {
    node.get(key).and_then(Node::as_str).unwrap_or_default()
}

fn ids(node: &Node, key: &str) -> Vec<String> {
    seq(node, key)
        .iter()
        .filter_map(Node::as_str)
        .map(str::to_string)
        .collect()
}

/// 空白の連なりを 1 つに畳み、前後を落とす。
fn fold(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// ノートの契約表の行 `id` の字（節の本文は section の値と n が等しい散文の節・無ければ空）。
fn row_of(note: &Node, id: &str) -> Option<Row> {
    let sections = seq(note, "sections");
    let row = sections
        .iter()
        .filter(|s| text(s, "type") == CONTRACT_TABLE)
        .flat_map(|s| seq(s, "rows"))
        .find(|r| text(r, "id") == id)?;
    let body = sections
        .iter()
        .find(|s| text(s, "type") == PROSE && text(s, "n") == text(row, "section"))
        .map_or("", |s| text(s, "body"));
    Some(Row {
        req: ids(row, "req"),
        basis: ids(row, "basis"),
        text: [text(row, "title"), body, text(row, "done")].join("\n"),
    })
}

/// 見つけた順の鍵と、鍵ごとの選び（None は全部・Some は名指した部分の列）。
#[derive(Default)]
struct Want {
    order: Vec<String>,
    picks: HashMap<String, Option<Vec<String>>>,
}

impl Want {
    fn add(&mut self, key: &str, part: Option<String>) {
        match self.picks.get_mut(key) {
            None => {
                self.order.push(key.to_string());
                self.picks.insert(key.to_string(), part.map(|p| vec![p]));
            }
            Some(slot) => match (slot.as_mut(), part) {
                (Some(parts), Some(p)) if !parts.contains(&p) => parts.push(p),
                (Some(_), None) => *slot = None,
                _ => {}
            },
        }
    }

    fn pick(&self, key: &str) -> Option<&Vec<String>> {
        self.picks.get(key).and_then(Option::as_ref)
    }
}

/// 名指しを種ごとに集めた物（要件書の id・判断の記録・規則行・条）。
#[derive(Default)]
struct Named {
    srs: Want,
    adr: Want,
    rule: Want,
    article: Want,
}

impl Named {
    /// id 1 つ（判断の記録は `items` の項・None は決定の全文・規範文は親の条の部分）。
    fn add(&mut self, id: &str, items: Option<Vec<u32>>) {
        if id.starts_with("ADR-") {
            match items {
                Some(items) => items
                    .iter()
                    .for_each(|k| self.adr.add(id, Some(k.to_string()))),
                None => self.adr.add(id, None),
            }
        } else if id.starts_with("R-") || id.starts_with("D-") {
            self.rule.add(id, None);
        } else if ["P-", "A-", "N-"].iter().any(|p| id.starts_with(p)) {
            match id.split_once('.') {
                Some((article, _)) => self.article.add(article, Some(id.to_string())),
                None => self.article.add(id, None),
            }
        } else {
            self.srs.add(id, None);
        }
    }

    /// 字の中の名指しを前から順に足す。
    fn scan(&mut self, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if let Some(end) = adr_end(&chars, i) {
                let id: String = chars.iter().skip(i).take(end - i).collect();
                self.add(&id, items_after(&chars, end));
                i = end;
            } else if let Some(end) = id_end(&chars, i) {
                let id: String = chars.iter().skip(i).take(end - i).collect();
                self.add(&id, None);
                i = end;
            } else {
                i += 1;
            }
        }
    }
}

fn skip_space(chars: &[char], mut i: usize) -> usize {
    while chars.get(i).is_some_and(|c| c.is_whitespace()) {
        i += 1;
    }
    i
}

/// `i` から字 `word` が在れば、その後ろの位置。
fn after_word(chars: &[char], i: usize, word: &str) -> Option<usize> {
    let w: Vec<char> = word.chars().collect();
    (chars.get(i..i + w.len()) == Some(&w[..])).then_some(i + w.len())
}

/// `i` から項の印 `(k)`（半角と全角の括弧）が在れば、k とその後ろの位置。
fn item_at(chars: &[char], i: usize) -> Option<(u32, usize)> {
    if !matches!(chars.get(i), Some('(' | '（')) {
        return None;
    }
    let n = chars
        .iter()
        .skip(i + 1)
        .take_while(|c| c.is_ascii_digit())
        .count();
    if n == 0 || n > ITEM_DIGITS || !matches!(chars.get(i + 1 + n), Some(')' | '）')) {
        return None;
    }
    let k: String = chars.iter().skip(i + 1).take(n).collect();
    k.parse().ok().map(|k| (k, i + 2 + n))
}

/// 判断の記録の id の後ろ `at` からの「決定」の項の列。字「決定」が無いか項が 1 つも無ければ None（決定の全文）。
fn items_after(chars: &[char], at: usize) -> Option<Vec<u32>> {
    let mut i = skip_space(chars, at);
    if chars.get(i) == Some(&'の') {
        i = skip_space(chars, i + 1);
    }
    i = after_word(chars, i, "決定")?;
    let mut items: Vec<u32> = Vec::new();
    let mut range = false;
    while let Some((k, next)) = item_at(chars, skip_space(chars, i)) {
        match items.last() {
            Some(&from) if range && from < k => items.extend(from + 1..=k),
            _ => items.push(k),
        }
        i = skip_space(chars, next);
        range = false;
        if let Some(n) = after_word(chars, i, "〜").or_else(|| after_word(chars, i, "から")) {
            range = true;
            i = n;
        } else if matches!(chars.get(i), Some('と' | '・' | '、')) {
            i += 1;
        }
    }
    (!items.is_empty()).then_some(items)
}

/// 決定の字を行頭の項 `(k)` ごとに割る（項の前の行は落とす）。行頭の項が無ければ空。
fn decision_items(decision: &str) -> Vec<(String, Vec<&str>)> {
    let mut items: Vec<(String, Vec<&str>)> = Vec::new();
    for line in decision.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let chars: Vec<char> = line.chars().collect();
        match item_at(&chars, 0).filter(|_| line.starts_with('(')) {
            Some((k, _)) => items.push((k.to_string(), vec![line])),
            None => {
                if let Some((_, lines)) = items.last_mut() {
                    lines.push(line);
                }
            }
        }
    }
    items
}

/// 要件書の id の行と節（`SRS_ID_SECTIONS` の節を順に引く）。
fn srs_row<'a>(srs: &'a Node, id: &str) -> Option<(&'a str, &'a Node)> {
    SRS_ID_SECTIONS.iter().find_map(|sec| {
        seq(srs, sec)
            .iter()
            .find(|r| text(r, "id") == id)
            .map(|r| (*sec, r))
    })
}

fn kind_of(section: &str) -> &'static str {
    SRS_KINDS
        .iter()
        .find(|(s, _)| *s == section)
        .map_or(NODE_KINDS[4], |(_, k)| k)
}

/// 要件の本文（器の requirements.txt と同じ字: text があればそれ、無ければ when と字「 — 」と shall、when も無ければ shall・空白を畳む）。
fn requirement_body(row: &Node) -> String {
    let (body, when, shall) = (text(row, "text"), text(row, "when"), text(row, "shall"));
    match (body.is_empty(), when.is_empty()) {
        (false, _) => fold(body),
        (true, false) => format!("{} — {}", fold(when), fold(shall)),
        (true, true) => fold(shall),
    }
}

/// 塊 1 つを書く（頭と、空でない本文の行を頭に空白 2 つを置いて）。
fn push_block(out: &mut String, head: &str, lines: &[String]) {
    out.push_str(&format!("- {head}\n"));
    for line in lines.iter().filter(|l| !l.is_empty()) {
        out.push_str(&format!("  {line}\n"));
    }
}

/// 行の塊（読んだ木だけの純関数）。
pub(crate) fn blocks(canon: &Canon, row: &Row) -> String {
    let mut named = Named::default();
    for id in row.req.iter().chain(&row.basis) {
        named.add(id, None);
    }
    named.scan(&row.text);
    for req in &row.req {
        if let Some((_, r)) = srs_row(&canon.srs, req) {
            ids(r, "basis")
                .iter()
                .filter(|b| ["P-", "A-", "N-"].iter().any(|p| b.starts_with(p)))
                .for_each(|b| named.add(b, None));
        }
    }
    for ac in seq(&canon.srs, "acceptance") {
        if ids(ac, "verifies").iter().any(|v| row.req.contains(v)) {
            named.add(text(ac, "id"), None);
        }
    }
    let (mut out, mut tail, mut unresolved) = (String::new(), String::new(), Vec::new());
    for id in &named.srs.order {
        match srs_row(&canon.srs, id) {
            Some(("acceptance", r)) => push_block(
                &mut tail,
                &format!("{id} {}", NODE_KINDS[6]),
                &[fold(text(r, "title"))],
            ),
            Some((sec, r)) => {
                let title = [text(r, "title"), text(r, "name")].concat();
                push_block(
                    &mut out,
                    &format!("{id} {}", kind_of(sec)),
                    &[fold(&title), requirement_body(r)],
                );
            }
            None => unresolved.push(id.clone()),
        }
    }
    adr_blocks(canon, &named.adr, &mut out, &mut unresolved);
    for id in &named.rule.order {
        match RULE_SECTIONS
            .iter()
            .find_map(|s| seq(&canon.rules, s).iter().find(|r| text(r, "id") == id))
        {
            Some(r) => push_block(
                &mut out,
                &format!("{id} {}", NODE_KINDS[2]),
                &[rule_line(r)],
            ),
            None => unresolved.push(id.clone()),
        }
    }
    article_blocks(canon, &named.article, &mut out, &mut unresolved);
    out.push_str(&tail);
    if !unresolved.is_empty() {
        push_block(&mut out, UNRESOLVED_HEAD, &unresolved);
    }
    out
}

/// 規則行の 1 行（what と字「 = 」と value と全角の括弧の中の字「条 」と article・value の無い行は what と括弧だけ）。
fn rule_line(r: &Node) -> String {
    let (what, value, article) = (
        fold(text(r, "what")),
        fold(text(r, "value")),
        text(r, "article"),
    );
    let head = if value.is_empty() {
        what
    } else {
        format!("{what} = {value}")
    };
    if article.is_empty() {
        head
    } else {
        format!("{head}（条 {article}）")
    }
}

/// 判断の記録の塊（題と、決定の全文か名指した項）。
fn adr_blocks(canon: &Canon, want: &Want, out: &mut String, unresolved: &mut Vec<String>) {
    for id in &want.order {
        let Some((_, record)) = canon.adrs.iter().find(|(name, _)| name == id) else {
            unresolved.push(id.clone());
            continue;
        };
        let decision = text(record, "decision");
        let items = decision_items(decision);
        let mut lines = vec![fold(text(record, "title"))];
        match want.pick(id).filter(|_| !items.is_empty()) {
            None => lines.extend(decision.lines().map(str::trim).map(str::to_string)),
            Some(picks) => {
                for (k, item) in &items {
                    if picks.contains(k) {
                        lines.extend(item.iter().map(|l| (*l).to_string()));
                    }
                }
                let missing = picks.iter().filter(|p| !items.iter().any(|(k, _)| k == *p));
                unresolved.extend(missing.map(|k| format!("{id} 決定 ({k})")));
                if lines.len() == 1 {
                    continue;
                }
            }
        }
        push_block(out, &format!("{id} {}", NODE_KINDS[10]), &lines);
    }
}

/// 条の塊（題と、規範文の全部か名指した規範文）。
fn article_blocks(canon: &Canon, want: &Want, out: &mut String, unresolved: &mut Vec<String>) {
    for id in &want.order {
        let Some(article) = seq(&canon.constitution, "articles")
            .iter()
            .find(|a| text(a, "id") == id)
        else {
            unresolved.push(id.clone());
            continue;
        };
        let statements = seq(article, "statements");
        let picks = want.pick(id);
        let mut lines = vec![fold(text(article, "title"))];
        for s in statements {
            let sid = text(s, "id");
            if picks.is_none_or(|p| p.iter().any(|x| x == sid)) {
                lines.push(format!("{sid} {}", fold(text(s, "text"))));
            }
        }
        if let Some(picks) = picks {
            unresolved.extend(
                picks
                    .iter()
                    .filter(|p| !statements.iter().any(|s| text(s, "id") == p.as_str()))
                    .cloned(),
            );
        }
        if picks.is_none() || lines.len() > 1 {
            push_block(out, &format!("{id} {}", NODE_KINDS[0]), &lines);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONSTITUTION: &str = "articles:
  - id: P-1
    title: 条一
    statements:
      - {id: P-1.1, text: 規範一の一}
      - {id: P-1.2, text: 規範一の二}
  - id: P-2
    title: 条二
    statements:
      - {id: P-2.1, text: 規範二の一}
      - {id: P-2.2, text: 規範二の二}
  - id: P-3
    title: 条三
    statements:
      - {id: P-3.1, text: 規範三の一}
";

    const RULES: &str = "thresholds:
  - {id: R-1, article: P-2, what: 上限の何, value: 3 本 以下}
discipline:
  - {id: D-1, article: P-1, what: 決まりの何}
";

    const SRS: &str = "goals:
  - {id: GOAL1, title: 目的一, text: 目的の  本文}
requirements:
  - id: FR1
    title: 要件一
    when: 何かの  とき
    shall: 何かを する
    basis: [P-1, ADR-2]
  - id: FR2
    title: 要件二
    shall: 二の本文
    basis: [P-3]
acceptance:
  - {id: AC1, title: 受入一, verifies: [FR1]}
  - {id: AC2, title: 受入二, verifies: [FR2]}
  - {id: AC3, title: 受入三, verifies: [FR2, FR1]}
  - {id: AC4, title: 受入四, verifies: [FR2]}
";

    const ADR1: &str = "title: 記録一
decision: |
  前置き
  (1) 項一
  (ア) 項一の続き
  (2) 項二
  (3) 項三
";

    const ADR2: &str = "title: 記録二
decision: 決めた字 (1) 行の中の項
";

    fn node(text: &str) -> Node {
        yaml::parse(text).unwrap().root
    }

    fn fixture() -> Canon {
        Canon {
            constitution: node(CONSTITUTION),
            rules: node(RULES),
            srs: node(SRS),
            adrs: vec![
                ("ADR-1".to_string(), node(ADR1)),
                ("ADR-2".to_string(), node(ADR2)),
            ],
        }
    }

    fn row(req: &[&str], basis: &[&str], text: &str) -> Row {
        Row {
            req: req.iter().map(|s| (*s).to_string()).collect(),
            basis: basis.iter().map(|s| (*s).to_string()).collect(),
            text: text.to_string(),
        }
    }

    /// 塊の頭の行（`- ` で始まる行）だけ。
    fn heads(out: &str) -> Vec<&str> {
        out.lines().filter(|l| l.starts_with("- ")).collect()
    }

    #[test]
    fn fctx_blocks_follow_the_order_and_the_depth() {
        let out = blocks(
            &fixture(),
            &row(
                &["FR1"],
                &[],
                "題 FR2 と R-1 と P-2.1\n本文 ADR-2 と AC2 と D-1\ndone GOAL1",
            ),
        );
        let want = "- FR1 要件
  要件一
  何かの とき — 何かを する
- FR2 要件
  要件二
  二の本文
- GOAL1 目的
  目的一
  目的の 本文
- ADR-2 判断の記録
  記録二
  決めた字 (1) 行の中の項
- R-1 規則行
  上限の何 = 3 本 以下（条 P-2）
- D-1 規則行
  決まりの何（条 P-1）
- P-2 条
  条二
  P-2.1 規範二の一
- P-1 条
  条一
  P-1.1 規範一の一
  P-1.2 規範一の二
- AC2 受入基準
  受入二
- AC1 受入基準
  受入一
- AC3 受入基準
  受入三
";
        assert_eq!(out, want);
    }

    #[test]
    fn fctx_decision_names_cut_only_their_items() {
        let canon = fixture();
        let cut = |text: &str| blocks(&canon, &row(&[], &[], text));
        let one_three = "- ADR-1 判断の記録\n  記録一\n  (1) 項一\n  (ア) 項一の続き\n  (3) 項三\n";
        assert_eq!(cut("ADR-1 決定 (1) と (3)・束 B05"), one_three);
        assert_eq!(cut("ADR-1 の決定 (3)、ADR-1 決定 (1)"), one_three);
        for joined in [
            "ADR-1 決定 (1)・(3)",
            "ADR-1 決定 (3)、(1)",
            "ADR-1 決定 (1)(3)",
        ] {
            assert_eq!(cut(joined), one_three, "{joined}");
        }
        assert_eq!(
            cut("ADR-1 決定（2）〜（3）"),
            "- ADR-1 判断の記録\n  記録一\n  (2) 項二\n  (3) 項三\n"
        );
        assert_eq!(
            cut("ADR-1 決定 (1) から (2)"),
            "- ADR-1 判断の記録\n  記録一\n  (1) 項一\n  (ア) 項一の続き\n  (2) 項二\n"
        );
        let whole = "- ADR-1 判断の記録\n  記録一\n  前置き\n  (1) 項一\n  (ア) 項一の続き\n  (2) 項二\n  (3) 項三\n";
        assert_eq!(cut("ADR-1 の段 2"), whole);
        assert_eq!(cut("ADR-1 決定 (2) と ADR-1 の段"), whole);
        assert_eq!(cut("ADR-1 決定 と (2)"), whole);
        assert_eq!(
            cut("ADR-2 決定 (1)"),
            "- ADR-2 判断の記録\n  記録二\n  決めた字 (1) 行の中の項\n"
        );
        assert_eq!(
            cut("ADR-1 決定 (2) と (9)"),
            "- ADR-1 判断の記録\n  記録一\n  (2) 項二\n- 引けない id\n  ADR-1 決定 (9)\n"
        );
    }

    #[test]
    fn fctx_basis_ids_are_named_whole() {
        let canon = fixture();
        let text = "ADR-1 決定 (2) と P-2.1";
        let with = blocks(&canon, &row(&[], &["FR2", "ADR-1", "P-2", "R-1"], text));
        assert_eq!(
            heads(&with),
            [
                "- FR2 要件",
                "- ADR-1 判断の記録",
                "- R-1 規則行",
                "- P-2 条"
            ]
        );
        assert!(
            with.contains("  前置き\n") && with.contains("  P-2.2 規範二の二\n"),
            "{with}"
        );
        let without = blocks(&canon, &row(&[], &[], text));
        assert_eq!(heads(&without), ["- ADR-1 判断の記録", "- P-2 条"]);
        assert!(
            !without.contains("前置き") && !without.contains("P-2.2"),
            "{without}"
        );
    }

    #[test]
    fn fctx_unresolved_ids_make_one_last_block() {
        let out = blocks(
            &fixture(),
            &row(
                &["FR9"],
                &["ADR-77"],
                "P-1.9 と GOAL7 と R-99 と P-9 と AC99 と D-1",
            ),
        );
        assert_eq!(heads(&out), ["- D-1 規則行", "- 引けない id"]);
        assert!(
            out.ends_with(
                "- 引けない id\n  FR9\n  GOAL7\n  AC99\n  ADR-77\n  R-99\n  P-1.9\n  P-9\n"
            ),
            "{out}"
        );
        assert_eq!(
            blocks(&fixture(), &row(&[], &[], "ADR-1 決定 (8) と (9)")),
            "- 引けない id\n  ADR-1 決定 (8)\n  ADR-1 決定 (9)\n"
        );
    }

    /// 置き場の写しを一時の dir に書く（正本 3 file と判断の記録 2 本とノート 1 本）。
    fn place(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        let dir =
            std::env::temp_dir().join(format!("folio-fctx-{tag}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(dir.join("adr")).unwrap();
        fs::create_dir_all(dir.join("design-note")).unwrap();
        fs::write(dir.join("constitution.yaml"), CONSTITUTION).unwrap();
        fs::write(dir.join("rules.yaml"), RULES).unwrap();
        fs::write(dir.join("srs.yaml"), SRS).unwrap();
        fs::write(dir.join("adr/ADR-1.yaml"), ADR1).unwrap();
        fs::write(dir.join("adr/ADR-2.yaml"), ADR2).unwrap();
        fs::write(
            dir.join("design-note/n1.yaml"),
            "sections:\n  - {n: 1, type: prose, title: 節, body: 本文の R-1}\n  - n: 2\n    type: contract-table\n    rows:\n      - {id: a, title: 題の P-2.1, req: [FR1], section: \"1\", done: 測る ADR-1 決定 (2)}\n",
        )
        .unwrap();
        dir
    }

    fn refused(dir: &Path, named: &str) -> Result<String, (u8, String)> {
        build(dir, named).map_err(Refusal::split)
    }

    #[test]
    fn fctx_build_reads_one_place_and_refuses_with_one_or_two() {
        let dir = place("build");
        let got = refused(&dir, "n1#a");
        let want = blocks(
            &fixture(),
            &row(&["FR1"], &[], "題の P-2.1\n本文の R-1\n測る ADR-1 決定 (2)"),
        );
        assert_eq!(got, Ok(want));
        let note = fs::read_to_string(dir.join("design-note/n1.yaml")).unwrap();
        fs::create_dir_all(dir.join("design-note/sub")).unwrap();
        fs::write(dir.join("design-note/sub/n1.yaml"), &note).unwrap();
        fs::write(dir.join("design-note/.n1.yaml"), &note).unwrap();
        for (bad, why) in [
            ("n1", "行の名指し n1 が <ノート>#<行> の形でない"),
            ("#a", "行の名指し #a が <ノート>#<行> の形でない"),
            ("n1#", "行の名指し n1# が <ノート>#<行> の形でない"),
            (
                "sub/n1#a",
                "行の名指し sub/n1#a が <ノート>#<行> の形でない",
            ),
            (".n1#a", "行の名指し .n1#a が <ノート>#<行> の形でない"),
            ("n1#b", "ノート n1 の契約表に行 b が無い"),
        ] {
            assert_eq!(refused(&dir, bad), Err((1, why.to_string())), "{bad}");
        }
        let (code, why) = refused(&dir, "n2#a").unwrap_err();
        assert!(code == 1 && why.starts_with("ノート n2 が "), "{why}");
        fs::write(dir.join("adr/ADR-3.yaml"), "title: [壊れた").unwrap();
        assert_eq!(refused(&dir, "n1#a").map_err(|e| e.0), Err(2));
        fs::remove_file(dir.join("adr/ADR-3.yaml")).unwrap();
        fs::remove_file(dir.join("srs.yaml")).unwrap();
        let (code, why) = refused(&dir, "n1#a").unwrap_err();
        assert_eq!(code, 2);
        assert!(why.contains("srs.yaml を読めない"), "{why}");
        fs::remove_dir_all(&dir).unwrap();
    }
}
