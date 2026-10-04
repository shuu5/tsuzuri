//! グラフの面の部品（見本の ui.js の nodeBox・edgeSVG・legendHTML・hlCompute・hlPin・便 g-graph）のうち、
//! 近傍の図（around）と節点の頁の近傍（project の nodearound）と節点の card が引く物を置く。
//! 選ぶ・畳む・数える・段を決めるのは中核の crate が済ませた電文（契約の型の GraphView）で、
//! ここは電文の値を写すだけ（節点を選ぶ分岐と数え直しを書かない）。
//! 線の道と形・箱の縁・光らせ方・固定の移り方・凡例は純粋な関数にして host で試す。
//! グラフの面の DOM と、面だけの配置・拡大・狭い幅の一覧・数の行・眺めの口の読みは行 m-map-graph で消した。

use std::collections::BTreeMap;
use std::fmt::Write;

use tsuzuri_contract::graph::{EdgeType, GraphView, NodeKind, ViewEdge, ViewNode};

use super::around::edge_name;
use super::band::{Band, band_of};
use crate::vocab::vocab;

/// 節点の箱の高さ。
pub const NODE_H: u32 = 40;

/// 箱の左上。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}

/// beads の帯の種類の 1 行（上端・高さ・升のうち最も多い節点の数（8 まで））。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lane {
    pub kind: NodeKind,
    pub top: u32,
    pub height: u32,
    pub count: u32,
}

/// 1 つの帯の置き場（beads の帯は節点が在れば種類の 7 行を持つ）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandRow {
    pub band: Band,
    pub top: u32,
    pub height: u32,
    pub lanes: Vec<Lane>,
}

/// 線の形（閉じた 4・見本の STYLE_OF）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineStyle {
    Solid,
    Bold,
    Dotted,
    Arrow,
}

impl LineStyle {
    pub const ALL: [LineStyle; 4] = [
        LineStyle::Solid,
        LineStyle::Bold,
        LineStyle::Dotted,
        LineStyle::Arrow,
    ];

    /// 線の色と太さ（見本の edgeSVG と同じ値）。
    pub fn stroke(self) -> &'static str {
        match self {
            LineStyle::Solid => r#"stroke="var(--ink-3)" stroke-width="1.2""#,
            LineStyle::Bold => r#"stroke="var(--s-stop)" stroke-width="3""#,
            LineStyle::Dotted => {
                r#"stroke="var(--ink-3)" stroke-width="1.4" stroke-dasharray="2 3""#
            }
            LineStyle::Arrow => r#"stroke="var(--st-limit)" stroke-width="2""#,
        }
    }
}

/// 辺の型から線の形（閉じた表・型が増えれば組み立てで落ちる）。
pub fn line_style(t: EdgeType) -> LineStyle {
    use EdgeType as E;
    match t {
        E::Blocks => LineStyle::Bold,
        E::InArticle | E::ArticleRef | E::ParentChild | E::RunOf | E::RanBy | E::Landed => {
            LineStyle::Dotted
        }
        E::Amends => LineStyle::Arrow,
        E::RelationsArticles
        | E::RelationsReqs
        | E::RelationsRules
        | E::RelationsSections
        | E::AmendedBy
        | E::Refs
        | E::Basis
        | E::Goals
        | E::Rules
        | E::Adrs
        | E::VerifyAc
        | E::Verifies
        | E::Figures
        | E::Produced
        | E::Req
        | E::Depends
        | E::RelatesTo
        | E::DiscoveredFrom
        | E::Design
        | E::RuledBy
        | E::Answers
        | E::Touches
        | E::Premises
        | E::Source
        | E::Raised => LineStyle::Solid,
    }
}

/// 線の道（3 次の曲線・制御点の x は両端の x の中点・y はそれぞれの端の y）。
pub fn curve(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let mx = (x1 + x2) / 2.0;
    format!("M{x1} {y1} C {mx} {y1}, {mx} {y2}, {x2} {y2}")
}

/// 真っすぐな線の道（凡例の見本）。
pub fn straight(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    format!("M{x1} {y1} L {x2} {y2}")
}

/// 線の SVG（道と、矢印の形なら to の端の三角）。
pub fn line_svg(style: LineStyle, d: &str, (x2, y2): (f64, f64)) -> String {
    let mut s = format!(r#"<path d="{d}" fill="none" {}/>"#, style.stroke());
    if style == LineStyle::Arrow {
        let _ = write!(
            s,
            r#"<polygon points="{},{} {x2},{y2} {},{}" fill="var(--st-limit)"/>"#,
            x2 - 6.0,
            y2 - 3.5,
            x2 - 6.0,
            y2 + 3.5
        );
    }
    s
}

/// 凡例の線の見本（型の線の形で真っすぐ）。
pub fn legend_line(t: EdgeType) -> String {
    format!(
        r#"<svg viewBox="0 0 28 10" aria-hidden="true">{}</svg>"#,
        line_svg(line_style(t), &straight(1.0, 5.0, 27.0, 5.0), (27.0, 5.0))
    )
}

/// SVG と HTML の字の escape。
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// 字を n 字に切る（越えれば n − 1 字と「…」・見本の cut）。
pub fn cut(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > n {
        let mut out: String = chars.iter().take(n.saturating_sub(1)).collect();
        out.push('…');
        out
    } else {
        s.to_string()
    }
}

/// 頭の記号が丸か（台帳と走行の帯・file に書かれた行は四角）。
pub fn round_mark(band: Band) -> bool {
    matches!(band, Band::Beads | Band::Pipeline)
}

/// open の問い（止まりの色の縁と塗らない記号）。
pub fn open_question(n: &ViewNode) -> bool {
    n.node.kind == NodeKind::Question && n.status.as_deref() == Some("open")
}

/// 閉じた bead と終わった走行（id と題を薄い字にする）。
pub fn closed(n: &ViewNode) -> bool {
    round_mark(band_of(n.node.kind)) && n.status.as_deref().is_some_and(|s| s.starts_with("closed"))
}

/// 箱の縁の色と太さ（open の問いは止まりの色・固定した節点は太い縁・見本の nodeBox）。
pub fn border(open_q: bool, pinned: bool) -> (&'static str, u32) {
    let stroke = if open_q {
        "var(--s-stop)"
    } else if pinned {
        "var(--ink)"
    } else {
        "var(--line-2)"
    };
    let width = if pinned {
        3
    } else if open_q {
        2
    } else {
        1
    };
    (stroke, width)
}

/// 凡例の 1 行目の形の見本（四角 = file に書かれた行・丸 = 台帳と走行）。
pub const LEGEND_SHAPES: &str = r#"<svg viewBox="0 0 14 14" class="lk" aria-hidden="true"><rect x="2" y="2" width="10" height="10" rx="1" fill="var(--ink-2)"/></svg><svg viewBox="0 0 14 14" class="lk" aria-hidden="true"><circle cx="7" cy="7" r="5" fill="var(--ink-2)"/></svg>"#;

/// 凡例の 1 行目の縁の見本（open の問い・固定・閉じた）。
pub const LEGEND_BORDERS: &str = r#"<svg viewBox="0 0 30 16" class="lk wide" aria-hidden="true"><rect x="1.5" y="1.5" width="27" height="13" rx="4" fill="var(--panel)" stroke="var(--s-stop)" stroke-width="2"/></svg><svg viewBox="0 0 30 16" class="lk wide" aria-hidden="true"><rect x="1.5" y="1.5" width="27" height="13" rx="4" fill="var(--panel)" stroke="var(--ink)" stroke-width="3"/></svg><svg viewBox="0 0 30 16" class="lk wide" aria-hidden="true"><rect x="1.5" y="1.5" width="27" height="13" rx="4" fill="var(--panel)" stroke="var(--line-2)"/><rect x="7" y="7" width="16" height="2" fill="var(--ink-3)"/></svg>"#;

/// 凡例の 1 行目の hover の見本。
pub const LEGEND_HOVER: &str = r#"<svg viewBox="0 0 30 16" class="lk wide" aria-hidden="true"><rect x="1.5" y="1.5" width="27" height="13" rx="4" fill="var(--panel)" stroke="var(--accent)" stroke-width="2.5"/></svg>"#;

/// 凡例に出す帯と辺の型（図に出ている帯 = 節点を持つ帯・帯の順／図に出ている辺の型・型の順）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Legend {
    pub bands: Vec<Band>,
    pub types: Vec<EdgeType>,
}

pub fn legend(view: &GraphView) -> Legend {
    Legend {
        bands: Band::ALL
            .into_iter()
            .filter(|b| view.nodes.iter().any(|n| band_of(n.node.kind) == *b))
            .collect(),
        types: EdgeType::ALL
            .into_iter()
            .filter(|t| view.edges.iter().any(|e| e.edge_type == *t))
            .collect(),
    }
}

/// 辺の型の注釈の鍵（「e:」に型の字・語の辞書に在る型だけ）。
pub fn edge_term(t: EdgeType) -> Option<String> {
    let key = format!("e:{}", edge_name(t));
    vocab().term(&key).map(|_| key)
}

/// 次数がこれを超える節点は光らせるが、そこから先へは広げない（見本の HL_HUB）。
pub const HUB_DEGREE: u32 = 8;
/// 光る節点（中心を除く）の上限（規則の行 R-20 の値）。
pub const LIT_MAX: usize = 20;
/// たどる段（根拠の側と影響の側の各）。
pub const DEPTH: u8 = 2;

/// 光る節点の側（閉じた 3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Center,
    Basis,
    Impact,
}

impl Side {
    pub const ALL: [Side; 3] = [Side::Center, Side::Basis, Side::Impact];

    /// 節点の class（見本の lit-self・lit-up・lit-down）。
    pub fn class(self) -> &'static str {
        match self {
            Side::Center => "lit-self",
            Side::Basis => "lit-up",
            Side::Impact => "lit-down",
        }
    }
}

/// 光らせ方（節点ごとの側・着いた順・光る線の番号・段）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Highlight {
    pub center: String,
    pub side: BTreeMap<String, Side>,
    /// 着いた順（中心を除く）。
    pub order: Vec<String>,
    /// 光る線の電文の辺の番号（辺の順）。
    pub lit: Vec<usize>,
    pub depth: u8,
}

impl Highlight {
    fn on(&self, side: Side) -> Vec<&str> {
        self.order
            .iter()
            .filter(|k| self.side.get(*k) == Some(&side))
            .map(String::as_str)
            .collect()
    }

    /// 根拠の側の節点（着いた順）。
    pub fn basis(&self) -> Vec<&str> {
        self.on(Side::Basis)
    }

    /// 影響の側の節点（着いた順）。
    pub fn impact(&self) -> Vec<&str> {
        self.on(Side::Impact)
    }

    /// 固定の帯の値。
    pub fn bar(&self) -> PinBar {
        PinBar {
            id: self.center.clone(),
            basis: self.basis().len(),
            impact: self.impact().len(),
            depth: self.depth,
        }
    }
}

/// 節点の id から次数（電文の degree）。
pub fn degrees(view: &GraphView) -> BTreeMap<String, u32> {
    view.nodes
        .iter()
        .map(|n| (n.node.id.clone(), n.degree))
        .collect()
}

/// 中心から根拠の側と影響の側を `depth` 段たどる（図に出ている線だけ・hub から先へは広げない）。
fn walk(
    edges: &[ViewEdge],
    degree: &BTreeMap<String, u32>,
    center: &str,
    depth: u8,
) -> (BTreeMap<String, Side>, Vec<String>) {
    let mut side = BTreeMap::from([(center.to_string(), Side::Center)]);
    let mut order = Vec::new();
    for (to_side, basis) in [(Side::Basis, true), (Side::Impact, false)] {
        let mut front = vec![center.to_string()];
        for _ in 0..depth {
            let mut next = Vec::new();
            for k in &front {
                if k != center && degree.get(k).copied().unwrap_or(0) > HUB_DEGREE {
                    continue;
                }
                for e in edges {
                    // 電文の辺は from が影響の側の端・to が根拠の側の端。
                    let (here, there) = if basis {
                        (&e.from, &e.to)
                    } else {
                        (&e.to, &e.from)
                    };
                    if here == k && !side.contains_key(there) {
                        side.insert(there.clone(), to_side);
                        order.push(there.clone());
                        next.push(there.clone());
                    }
                }
            }
            front = next;
        }
    }
    (side, order)
}

/// 光らせ方（見本の hlCompute）: 各 2 段・20 を超えたら段を 1 に落とし、それでも超えたら着いた順の先頭の 20 個で切る。
/// 光る線はたどった向きに沿う線だけ（to の端が根拠の側で from の端が根拠の側か中心・from の端が影響の側で to の端が影響の側か中心）。
pub fn highlight(edges: &[ViewEdge], degree: &BTreeMap<String, u32>, center: &str) -> Highlight {
    let mut depth = DEPTH;
    let (mut side, mut order) = walk(edges, degree, center, depth);
    if order.len() > LIT_MAX {
        depth = 1;
        (side, order) = walk(edges, degree, center, depth);
    }
    if order.len() > LIT_MAX {
        for k in order.drain(LIT_MAX..) {
            side.remove(&k);
        }
    }
    let lit = edges
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            let to = side.get(&e.to).copied();
            let from = side.get(&e.from).copied();
            (to == Some(Side::Basis) && matches!(from, Some(Side::Basis | Side::Center)))
                || (from == Some(Side::Impact) && matches!(to, Some(Side::Impact | Side::Center)))
        })
        .map(|(i, _)| i)
        .collect();
    Highlight {
        center: center.to_string(),
        side,
        order,
        lit,
        depth,
    }
}

/// 固定の帯の値（固定した節点の id・根拠の側の数・影響の側の数・段）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinBar {
    pub id: String,
    pub basis: usize,
    pub impact: usize,
    pub depth: u8,
}

/// 固定を動かすもの（節点を押した・Escape・外す button）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinAction {
    Press(String),
    Escape,
    Unpin,
}

/// 次の固定（同じ節点をもう一度押すと外れる・別の節点を押すと移る・Escape と外す button で外れる・見本の hlPin）。
pub fn pin_next(now: Option<&str>, action: &PinAction) -> Option<String> {
    match action {
        PinAction::Press(id) if now == Some(id.as_str()) => None,
        PinAction::Press(id) => Some(id.clone()),
        PinAction::Escape | PinAction::Unpin => None,
    }
}

/// 組の箱の題と開き閉じの語の鍵（節点の card が引く・src/mapview/graph/fold.rs）。
pub mod fold;
