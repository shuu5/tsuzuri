//! 不変条件の 12 本を 3 値（合格・違反・まだ分からない）で数える。
//! 数えるのは 11 本。g-3・g-7 は裁定の書き出しを結んだ表（`Graph::rulings`・行 c-g3g7）から数え、表が無ければ
//! 「まだ分からない」。g-9 は detect の間（2 周の実測の前）は判定をつねに「まだ分からない」とし、
//! 本文で名指した id のうち欄にも辺にも無い対は床の値 `unfielded_mentions` が数えて名指す（行 c-g9）。
//! 要る出所が読めなければ「まだ分からない」で、合格にしない。違反は名指す id を持つ。
//! g-7 は宙に浮いた裁定のうち id が folio の裁定 id の文法の外の裁定を数えず、それだけが在れば「まだ分からない」。
//! 要約の無い節点は不変条件でなく床の値で、`unsummarized` が数えて名指す（要件 FR15）。

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{EdgeType, NodeKind};
use tsuzuri_contract::ledger::MEMO_LABEL;

use super::{Graph, Source};

/// 不変条件の判定（3 値）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "verdict", content = "ids", rename_all = "kebab-case")]
pub enum Verdict {
    Pass,
    /// 違反（名指す id・名の順・重複なし）。
    Violation(Vec<String>),
    Unknown,
}

/// 1 本の不変条件の判定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Invariant {
    pub id: &'static str,
    pub verdict: Verdict,
}

type Rule = fn(&Graph) -> Verdict;

/// 不変条件の 12 本（id の順）。
const RULES: [(&str, Rule); 12] = [
    ("g-1", g1_edge_ends),
    ("g-2", g2_one_pointer),
    ("g-3", g3_ruled),
    ("g-4", g4_touches),
    ("g-5", g5_reaches_root),
    ("g-6", g6_no_loop),
    ("g-7", g7_bound),
    ("g-8", g8_memo_or_contract),
    ("g-9", unmeasured),
    ("g-10", g10_disjoint_ids),
    ("g-11", g11_run_of),
    ("g-12", g12_raised),
];

/// 不変条件の id の一覧（順も固定）。
pub const INVARIANTS: [&str; 12] = [
    "g-1", "g-2", "g-3", "g-4", "g-5", "g-6", "g-7", "g-8", "g-9", "g-10", "g-11", "g-12",
];

/// つねに「まだ分からない」を返す 1 本（g-9 は detect の間は判定に出さず、数は `unfielded_mentions` が名指す）。
pub const UNMEASURED: [&str; 1] = ["g-9"];

/// 裁定の書き出しを結んだ表から数える 2 本（表が無ければ「まだ分からない」・行 c-g3g7）。
pub const RULED: [&str; 2] = ["g-3", "g-7"];

/// 12 本を数える（id の順）。
pub fn check(g: &Graph) -> Vec<Invariant> {
    RULES
        .iter()
        .map(|(id, rule)| Invariant {
            id,
            verdict: rule(g),
        })
        .collect()
}

/// 要る出所が全部読めていれば、名指す id の有無で合格か違反。読めない出所が在れば「まだ分からない」。
fn judge(g: &Graph, needs: &[Source], bad: BTreeSet<String>) -> Verdict {
    if needs.iter().any(|s| !g.is_read(*s)) {
        Verdict::Unknown
    } else if bad.is_empty() {
        Verdict::Pass
    } else {
        Verdict::Violation(bad.into_iter().collect())
    }
}

fn unmeasured(_: &Graph) -> Verdict {
    Verdict::Unknown
}

/// bead の id の族（最初の「.」の前の字）。
fn family(bead: &str) -> &str {
    bead.split('.').next().unwrap_or(bead)
}

/// g-3 発効した記録（判断の記録・規則行ほか結んだ節点）の裁定の字の先が台帳に在る。
/// 先の無い行のうち族が台帳に在る行の節点を名指す。名指す節点が無く、族が台帳のどの bead とも違う行
/// （外の台帳）が在れば「まだ分からない」。
fn g3_ruled(g: &Graph) -> Verdict {
    let Some(rulings) = &g.rulings else {
        return Verdict::Unknown;
    };
    if !g.is_read(Source::Design) || !g.is_read(Source::Ledger) {
        return Verdict::Unknown;
    }
    let families: BTreeSet<&str> = g.beads.keys().map(|id| family(id)).collect();
    let mut bad: BTreeSet<String> = BTreeSet::new();
    let mut foreign = false;
    for (id, rows) in rulings {
        for row in rows.iter().filter(|r| !g.holds(r)) {
            if families.contains(family(&row.bead)) {
                bad.insert(id.clone());
            } else {
                foreign = true;
            }
        }
    }
    if !bad.is_empty() {
        Verdict::Violation(bad.into_iter().collect())
    } else if foreign {
        Verdict::Unknown
    } else {
        Verdict::Pass
    }
}

/// 宙に浮いた裁定（answers の辺の元でも ruled_by の辺の先でもない裁定の節点）の id を、
/// folio の裁定 id の文法の内の組と外の組に分ける。
fn unbound(g: &Graph) -> (BTreeSet<String>, BTreeSet<String>) {
    let answers: BTreeSet<&str> = pairs(g, EdgeType::Answers)
        .into_iter()
        .map(|(from, _)| from)
        .collect();
    let ruled: BTreeSet<&str> = pairs(g, EdgeType::RuledBy)
        .into_iter()
        .map(|(_, to)| to)
        .collect();
    g.nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Ruling)
        .filter(|n| !answers.contains(n.id.as_str()) && !ruled.contains(n.id.as_str()))
        .map(|n| n.id.clone())
        .partition(|id| in_ruling_grammar(id))
}

/// g-7 裁定は問い（answers）か発効した記録（ruled_by）のどちらかに結ばれる。
/// 文法の外の id の宙に浮いた裁定は数えず、それが在れば合格にしない。
fn g7_bound(g: &Graph) -> Verdict {
    if g.rulings.is_none() {
        return Verdict::Unknown;
    }
    let (inside, outside) = unbound(g);
    match judge(g, &[Source::Design, Source::Ledger], inside) {
        Verdict::Pass if !outside.is_empty() => Verdict::Unknown,
        v => v,
    }
}

/// g-7 が文法の外の id の宙に浮いた裁定だけのために「まだ分からない」のとき、その数（ほかは None）。
pub fn outside_rulings(g: &Graph) -> Option<usize> {
    if g.rulings.is_none() || !g.is_read(Source::Design) || !g.is_read(Source::Ledger) {
        return None;
    }
    let (inside, outside) = unbound(g);
    (inside.is_empty() && !outside.is_empty()).then_some(outside.len())
}

/// 字の走査の位置（`in_ruling_grammar` の道具）。
#[derive(Clone, Copy)]
struct Scan<'a> {
    b: &'a [u8],
    i: usize,
}

impl Scan<'_> {
    /// 字 s が続けば進んで真。
    fn lit(&mut self, s: &str) -> bool {
        let ok = self.b[self.i..].starts_with(s.as_bytes());
        if ok {
            self.i += s.len();
        }
        ok
    }

    /// f に合う字がちょうど n 続けば進んで真。
    fn take(&mut self, n: usize, f: fn(u8) -> bool) -> bool {
        let ok = self.b.len() >= self.i + n && self.b[self.i..self.i + n].iter().all(|c| f(*c));
        if ok {
            self.i += n;
        }
        ok
    }

    /// f に合う字を続く限り進み、その数を返す。
    fn many(&mut self, f: fn(u8) -> bool) -> usize {
        let n = self.b[self.i..].iter().take_while(|c| f(**c)).count();
        self.i += n;
        n
    }

    /// 試しに進め、合えば進んだ位置を残す（合わなければ戻す）。
    fn attempt(&mut self, f: impl FnOnce(&mut Self) -> bool) -> bool {
        let mut t = *self;
        let ok = f(&mut t);
        if ok {
            *self = t;
        }
        ok
    }

    fn end(&self) -> bool {
        self.i == self.b.len()
    }
}

fn digit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn lower(c: u8) -> bool {
    c.is_ascii_lowercase()
}

fn word(c: u8) -> bool {
    lower(c) || digit(c)
}

/// 分の 1 の位（数字か x）。
fn minute_one(c: u8) -> bool {
    digit(c) || c == b'x'
}

/// 字の全部が folio の裁定 id の文法（folio の ruling.rs の写し・書き出しに出る形）に合うか。
/// 台帳の id（英小字 1・数字 1・「-」・英小字か数字の並び・「.数字」の段を何段でも）に、器の問いの印
/// （「:」年月日 8 桁「T」時分 4 桁「Z-」連番）か notes の日時（「 notes」に「 年-月-日」か「 時:分」か両方・
/// 分の 1 の位は x でもよい・任意の「 JST」）が続くか、どちらも続かない形。
pub fn in_ruling_grammar(id: &str) -> bool {
    let mut s = Scan {
        b: id.as_bytes(),
        i: 0,
    };
    if !(s.take(1, lower) && s.take(1, digit) && s.lit("-") && s.many(word) > 0) {
        return false;
    }
    while s.attempt(|s| s.lit(".") && s.many(digit) > 0) {}
    if s.end() {
        return true;
    }
    let question = s.attempt(|s| {
        s.lit(":")
            && s.take(8, digit)
            && s.lit("T")
            && s.take(4, digit)
            && s.lit("Z-")
            && s.many(digit) > 0
    });
    if question {
        return s.end();
    }
    if !s.lit(" notes") {
        return false;
    }
    let date = s.attempt(|s| {
        s.lit(" ")
            && s.take(4, digit)
            && s.lit("-")
            && s.take(2, digit)
            && s.lit("-")
            && s.take(2, digit)
    });
    let time = s.attempt(|s| {
        s.lit(" ") && s.take(2, digit) && s.lit(":") && s.take(1, digit) && s.take(1, minute_one)
    });
    s.attempt(|s| s.lit(" JST"));
    (date || time) && s.end()
}

/// g-1 組んだ辺の両端の節点が実在する。片端の無い辺が在り、読めない出所が在れば「まだ分からない」。
fn g1_edge_ends(g: &Graph) -> Verdict {
    let ids: BTreeSet<&str> = g.nodes.iter().map(|n| n.id.as_str()).collect();
    let bad: BTreeSet<String> = g
        .edges
        .iter()
        .flat_map(|e| [&e.from, &e.to])
        .filter(|id| !ids.contains(id.as_str()))
        .cloned()
        .collect();
    if bad.is_empty() {
        Verdict::Pass
    } else {
        judge(g, &Source::ALL, bad)
    }
}

/// g-2 open の契約は pointer の行をちょうど 1 つ持つ（closed の契約は数えない）。
fn g2_one_pointer(g: &Graph) -> Verdict {
    let bad = g
        .beads
        .iter()
        .filter(|(_, b)| b.kind == NodeKind::Task && b.is_open() && b.pointers.len() != 1)
        .map(|(id, _)| id.clone())
        .collect();
    judge(g, &[Source::Ledger], bad)
}

/// g-4 open の問いは metadata の touches の欄に id を 1 つ以上持つ。
fn g4_touches(g: &Graph) -> Verdict {
    let bad = g
        .beads
        .iter()
        .filter(|(_, b)| b.kind == NodeKind::Question && b.is_open() && b.touches.is_empty())
        .map(|(id, _)| id.clone())
        .collect();
    judge(g, &[Source::Ledger], bad)
}

/// 辺の型ごとの (from, to) の一覧。
fn pairs(g: &Graph, edge_type: EdgeType) -> Vec<(&str, &str)> {
    g.edges
        .iter()
        .filter(|e| e.edge_type == edge_type)
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect()
}

/// bead ごとの親（parent-child の先・辺の順）。
fn parents(g: &Graph) -> BTreeMap<&str, Vec<&str>> {
    let mut map: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (child, parent) in pairs(g, EdgeType::ParentChild) {
        map.entry(child).or_default().push(parent);
    }
    map
}

/// g-5 根（親を持たない epic）でない bead は、parent-child をたどって根に着く。
fn g5_reaches_root(g: &Graph) -> Verdict {
    let parents = parents(g);
    let is_root = |id: &str| {
        g.beads.get(id).is_some_and(|b| b.kind == NodeKind::Epic) && !parents.contains_key(id)
    };
    let reaches_root = |start: &str| {
        let mut seen = BTreeSet::from([start]);
        let mut cur = start;
        loop {
            if is_root(cur) {
                return true;
            }
            match parents.get(cur).and_then(|p| p.first()) {
                Some(p) if g.beads.contains_key(*p) && seen.insert(*p) => cur = p,
                _ => return false,
            }
        }
    };
    let bad = g
        .beads
        .keys()
        .filter(|id| !reaches_root(id))
        .cloned()
        .collect();
    judge(g, &[Source::Ledger], bad)
}

/// 輪の上か、輪と輪の間に在る節点（入りの無い節点と出の無い節点を繰り返し除いた残り）。
/// 残りが空なら輪は無い。
fn on_loop(pairs: &[(&str, &str)]) -> BTreeSet<String> {
    let mut live: Vec<(&str, &str)> = pairs.to_vec();
    loop {
        let froms: BTreeSet<&str> = live.iter().map(|(f, _)| *f).collect();
        let tos: BTreeSet<&str> = live.iter().map(|(_, t)| *t).collect();
        let before = live.len();
        live.retain(|(f, t)| tos.contains(f) && froms.contains(t));
        if live.len() == before {
            break;
        }
    }
    live.iter()
        .flat_map(|(f, t)| [*f, *t])
        .map(str::to_string)
        .collect()
}

/// g-6 blocks に輪が無く、parent-child は木（親は 1 つまで・輪が無い）。
fn g6_no_loop(g: &Graph) -> Verdict {
    let mut bad = on_loop(&pairs(g, EdgeType::Blocks));
    bad.extend(on_loop(&pairs(g, EdgeType::ParentChild)));
    bad.extend(
        parents(g)
            .into_iter()
            .filter(|(_, ps)| ps.iter().collect::<BTreeSet<_>>().len() > 1)
            .map(|(child, _)| child.to_string()),
    );
    judge(g, &[Source::Ledger], bad)
}

/// g-8 memo の label と pointer の行の両方を持つ bead が無い。
fn g8_memo_or_contract(g: &Graph) -> Verdict {
    let bad = g
        .beads
        .iter()
        .filter(|(_, b)| b.labels.iter().any(|l| l == MEMO_LABEL) && !b.pointers.is_empty())
        .map(|(id, _)| id.clone())
        .collect();
    judge(g, &[Source::Ledger], bad)
}

/// g-10 設計の索引と台帳と走行の id が互いに重ならない。重なりが在れば、読めない出所が在っても違反。
fn g10_disjoint_ids(g: &Graph) -> Verdict {
    let mut families: BTreeMap<&str, BTreeSet<Source>> = BTreeMap::new();
    for n in &g.nodes {
        if let Some(s) = Source::of(n.kind) {
            families.entry(n.id.as_str()).or_default().insert(s);
        }
    }
    let bad: BTreeSet<String> = families
        .into_iter()
        .filter(|(_, fs)| fs.len() > 1)
        .map(|(id, _)| id.to_string())
        .collect();
    if bad.is_empty() {
        judge(g, &Source::ALL, bad)
    } else {
        Verdict::Violation(bad.into_iter().collect())
    }
}

/// g-11 走行は run_of の先の bead を持つ。
fn g11_run_of(g: &Graph) -> Verdict {
    let run_of = pairs(g, EdgeType::RunOf);
    let bad = g
        .runs
        .keys()
        .filter(|run| {
            !run_of
                .iter()
                .any(|(from, to)| from == run && g.beads.contains_key(*to))
        })
        .cloned()
        .collect();
    judge(g, &[Source::Runs, Source::Ledger], bad)
}

/// 走行の段の Questioned（器の語）。
pub const QUESTIONED: &str = "Questioned";

/// g-12 段が Questioned の走行は答えの無い問いを持つ（器の問いは event log の中だけに在り、raised の辺は見ない）。
fn g12_raised(g: &Graph) -> Verdict {
    let bad = g
        .runs
        .iter()
        .filter(|(_, r)| r.stage.as_deref() == Some(QUESTIONED) && r.unanswered == 0)
        .map(|(run, _)| run.clone())
        .collect();
    judge(g, &[Source::Runs], bad)
}

/// 要約を数える種類（設計文書の 11 と、台帳の epic・task・memo・問い）。
/// 裁定・受け・方針は notes の 1 行から、走行は event log から導き、要約の欄を持たないので数えない。
pub const SUMMARY_KINDS: [NodeKind; 15] = [
    NodeKind::ALL[0],
    NodeKind::ALL[1],
    NodeKind::ALL[2],
    NodeKind::ALL[3],
    NodeKind::ALL[4],
    NodeKind::ALL[5],
    NodeKind::ALL[6],
    NodeKind::ALL[7],
    NodeKind::ALL[8],
    NodeKind::ALL[9],
    NodeKind::ALL[10],
    NodeKind::Epic,
    NodeKind::Task,
    NodeKind::Memo,
    NodeKind::Question,
];

/// 要約の無い節点（種類が `SUMMARY_KINDS` に在り、plain と eng のどちらも空でない字を持たない節点の id・字の順・重複なし）。
/// `summary` は設計の索引の要約の字を読めたか（`add_summary` の返り）。
/// 設計の索引か台帳が読めないか、要約の字が読めなければ「まだ分からない」（0 件とも名指しとも言わない）。
pub fn unsummarized(g: &Graph, summary: bool) -> Reading<Vec<String>> {
    if !summary || !g.is_read(Source::Design) || !g.is_read(Source::Ledger) {
        return Reading::Unknown;
    }
    let has = |s: &Option<String>| s.as_deref().is_some_and(|s| !s.is_empty());
    let ids: BTreeSet<String> = g
        .nodes
        .iter()
        .filter(|n| SUMMARY_KINDS.contains(&n.kind) && !has(&n.plain) && !has(&n.eng))
        .map(|n| n.id.clone())
        .collect();
    Reading::Known(ids.into_iter().collect())
}

/// 要件書の id の頭（頭と数字の並び）。
const SRS_PREFIXES: [&str; 5] = ["FR", "NFR", "AC", "CON", "GOAL"];

/// 字の列 b の start から始まる id の終わりの位置（folio の refs.rs の scan_ids と link.rs の scan_adr_ids の写し）。
/// 前の字が ASCII の英数字か「-」なら始まらず、後ろの字が ASCII の英数字なら終わらない。
/// 判断の記録（「ADR-」と 1 から 9 の数字 1 つと数字の並び）・条（P・A・N の 1 字と「-」と数字の並び・その後の
/// 「.」と数字の並びは後ろが空けば枝番まで、空かなければ枝番を外した形）・要件書の id（`SRS_PREFIXES` の頭と
/// 数字の並び）・規則行（R か D の 1 字と「-」と数字の並び）。
fn id_end(b: &[u8], start: usize) -> Option<usize> {
    if start > 0 && (b[start - 1].is_ascii_alphanumeric() || b[start - 1] == b'-') {
        return None;
    }
    let open = |i: usize| b.get(i).is_none_or(|c| !c.is_ascii_alphanumeric());
    let at = || Scan { b, i: start };
    let mut s = at();
    if s.lit("ADR-") && s.take(1, |c| (b'1'..=b'9').contains(&c)) {
        s.many(digit);
        return open(s.i).then_some(s.i);
    }
    let mut s = at();
    if s.take(1, |c| matches!(c, b'P' | b'A' | b'N')) && s.lit("-") && s.many(digit) > 0 {
        let head = s.i;
        if s.attempt(|s| s.lit(".") && s.many(digit) > 0) && open(s.i) {
            return Some(s.i);
        }
        return open(head).then_some(head);
    }
    for prefix in SRS_PREFIXES {
        let mut s = at();
        if s.lit(prefix) && s.many(digit) > 0 {
            return open(s.i).then_some(s.i);
        }
    }
    let mut s = at();
    let rule = s.take(1, |c| matches!(c, b'R' | b'D')) && s.lit("-") && s.many(digit) > 0;
    (rule && open(s.i)).then_some(s.i)
}

/// 字の中で名指した id（`id_end` の文法・拾った id の後ろから続けて探す・現れた順・同じ id も現れた数だけ）。
pub fn mentioned_ids(text: &str) -> Vec<&str> {
    let b = text.as_bytes();
    let mut ids = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match id_end(b, i) {
            Some(end) => {
                ids.push(&text[i..end]);
                i = end;
            }
            None => i += 1,
        }
    }
    ids
}

/// 規範文の id は親の条の id（最初の「.」の前の字）へ丸める（ほかの種類はそのまま）。
fn rounded(id: &str, kind: NodeKind) -> &str {
    if kind == NodeKind::Norm {
        id.split('.').next().unwrap_or(id)
    } else {
        id
    }
}

/// 本文で名指した id のうち欄にも辺にも無い対（g-9 の detect の数・規則行 R-17 と同じ見方・行 c-g9）。
/// `summary` は設計の索引の要約の字を読めたか（`add_summary` の返り）。偽か、設計の索引が読めなければ「まだ分からない」。
/// 設計の節点ごとに plain と eng の字（在る方）で名指した id のうち、設計の節点に引ける id を見る。規範文の id は
/// 親の条の id へ丸め（本文を持つ節点も辺の両端も）、丸めた 2 つが同じか、どれかの辺（型も向きも問わない）の
/// 丸めた両端なら数えない。対は本文を持つ節点の id と拾った字のままの id（字の順・重複なし）。
pub fn unfielded_mentions(g: &Graph, summary: bool) -> Reading<Vec<(String, String)>> {
    if !summary || !g.is_read(Source::Design) {
        return Reading::Unknown;
    }
    let kinds = Source::Design.kinds();
    let index = g.index();
    let mut linked: BTreeSet<(&str, &str)> = BTreeSet::new();
    for e in &g.edges {
        let [from, to] = [e.from.as_str(), e.to.as_str()]
            .map(|id| index.get(id).map_or(id, |n| rounded(id, n.kind)));
        linked.insert((from, to));
        linked.insert((to, from));
    }
    let mut pairs: BTreeSet<(String, String)> = BTreeSet::new();
    for n in g.nodes.iter().filter(|n| kinds.contains(&n.kind)) {
        let own = rounded(&n.id, n.kind);
        for text in [&n.plain, &n.eng].into_iter().flatten() {
            for id in mentioned_ids(text) {
                let Some(m) = index.get(id).filter(|m| kinds.contains(&m.kind)) else {
                    continue;
                };
                let other = rounded(id, m.kind);
                if own != other && !linked.contains(&(own, other)) {
                    pairs.insert((n.id.clone(), id.to_string()));
                }
            }
        }
    }
    Reading::Known(pairs.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::{INVARIANTS, RULES, on_loop};

    #[test]
    fn graph_rules_follow_the_id_list() {
        let ids: Vec<&str> = RULES.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, INVARIANTS.to_vec());
    }

    #[test]
    fn graph_on_loop_finds_only_loops() {
        assert!(on_loop(&[("a", "b"), ("b", "c")]).is_empty());
        let got: Vec<String> = on_loop(&[("x", "a"), ("a", "b"), ("b", "a"), ("b", "y")])
            .into_iter()
            .collect();
        assert_eq!(got, vec!["a", "b"]);
        assert_eq!(on_loop(&[("s", "s")]).len(), 1);
    }
}
