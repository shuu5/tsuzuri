//! グラフの面（見本の map.html の graph と initZoom・ui.js の nodeBox・edgeSVG・legendHTML・hlCompute・hlPin・便 g-graph）。
//! 選ぶ・畳む・数える・段を決めるのは中核の crate が済ませた眺めの電文（口 /api/graph/view の GraphView）で、
//! ここは電文の値を写して座標を決めるだけ（節点を選ぶ分岐と数え直しを書かない）。
//! 配置・線の道と形・拡大の値・光らせ方・固定の移り方・狭い幅の一覧・数の行・凡例は純粋な関数にして host で試し、
//! DOM と事件の受け取りは wasm の target のときだけ組み立てる。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use tsuzuri_contract::graph::{
    BoxFold, EdgeType, GraphView, NodeKind, ViewEdge, ViewNode, natural_cmp, title36,
};
use tsuzuri_contract::wire;

use super::band::{BEADS_LANES, Band, band_of, kind_key, unread_reason};
use super::is_open;
use super::table::edge_name;
use crate::project::{NO_CONTENT, NOT_READ};
use crate::view::Fetched;
use crate::vocab::{label, vocab};

/// 読みの口（server の便 e-view が足す）。
pub const PATH: &str = "/api/graph/view";

/// 口が読めないときの理由。
pub const REASON: &str =
    "グラフの眺めを組む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 口の本文を眺めの電文に読む（まだ読んでいない・読めない・電文の型として読めないは理由）。
pub fn doc(fetched: &Fetched) -> Result<GraphView, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(REASON),
        Fetched::Body(text) => wire::decode::<GraphView>(text).map_err(|_| NO_CONTENT),
    }
}

/// 節点の箱の幅。
pub const NODE_W: u32 = 176;
/// 節点の箱の高さ。
pub const NODE_H: u32 = 40;
/// 段の間。
pub const RANK_GAP: u32 = 36;
/// 箱の縦の間。
pub const BOX_GAP: u32 = 6;
/// 1 列の箱の数。
pub const PER_COL: u32 = 8;
/// 帯の名の幅（図の左端）。
pub const LABEL_W: u32 = 116;
/// 上の余白。
pub const TOP: u32 = 8;
/// 行の余白。
pub const PAD: u32 = 10;
/// 列の間。
pub const COL_GAP: u32 = 8;
/// 節点の無い帯の高さ。
pub const EMPTY_BAND_H: u32 = 44;
/// 節点の無い beads の行の高さ。
pub const EMPTY_LANE_H: u32 = 28;
/// 図の下の余白。
pub const BOTTOM: u32 = 4;

/// 1 列の幅（箱と列の間）。
const COL_W: u32 = NODE_W + COL_GAP;
/// 1 つの箱の縦の送り（箱と箱の縦の間）。
const ROW_H: u32 = NODE_H + BOX_GAP;

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// 箱の左上。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}

/// 1 つの段の列（段・左端の x・列の数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    pub rank: u32,
    pub x: u32,
    pub cols: u32,
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

/// 図の配置（幅・高さ・段の列・7 つの帯・箱の左上）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub width: u32,
    pub height: u32,
    pub columns: Vec<Column>,
    pub bands: Vec<BandRow>,
    pub boxes: BTreeMap<String, Pos>,
}

impl Layout {
    pub fn band(&self, band: Band) -> Option<&BandRow> {
        self.bands.iter().find(|b| b.band == band)
    }

    pub fn lane(&self, kind: NodeKind) -> Option<&Lane> {
        self.bands
            .iter()
            .flat_map(|b| &b.lanes)
            .find(|l| l.kind == kind)
    }

    pub fn column(&self, rank: u32) -> Option<&Column> {
        self.columns.iter().find(|c| c.rank == rank)
    }

    pub fn pos(&self, id: &str) -> Option<Pos> {
        self.boxes.get(id).copied()
    }
}

/// 節点の帯と行（beads の帯は種類の行・ほかの帯は 1 行）。
fn lane_key(kind: NodeKind) -> (Band, Option<NodeKind>) {
    let band = band_of(kind);
    (band, (band == Band::Beads).then_some(kind))
}

fn lanes_of(band: Band) -> Vec<Option<NodeKind>> {
    if band == Band::Beads {
        BEADS_LANES.into_iter().map(Some).collect()
    } else {
        vec![None]
    }
}

/// 眺めの配置（升 = 帯と行と段・升の中は id の自然な順・見本の graph と同じ決め方）。
pub fn layout(view: &GraphView) -> Layout {
    let mut cells: BTreeMap<(Band, Option<NodeKind>, u32), Vec<&str>> = BTreeMap::new();
    for n in &view.nodes {
        let (band, lane) = lane_key(n.node.kind);
        cells
            .entry((band, lane, n.rank))
            .or_default()
            .push(n.node.id.as_str());
    }
    for ids in cells.values_mut() {
        ids.sort_by(|a, b| natural_cmp(a, b));
    }

    let ranks: BTreeSet<u32> = view.nodes.iter().map(|n| n.rank).collect();
    let mut columns = Vec::new();
    let mut x = LABEL_W;
    for rank in ranks {
        let cols = cells
            .iter()
            .filter(|((_, _, r), _)| *r == rank)
            .map(|(_, ids)| count(ids.len()).div_ceil(PER_COL))
            .max()
            .unwrap_or(1)
            .max(1);
        columns.push(Column { rank, x, cols });
        x += cols * COL_W + RANK_GAP;
    }
    let width = columns.last().map_or(LABEL_W + NODE_W + COL_GAP, |c| {
        c.x + c.cols * COL_W + COL_GAP
    });

    let mut bands = Vec::new();
    let mut lane_tops: BTreeMap<(Band, Option<NodeKind>), u32> = BTreeMap::new();
    let mut y = TOP;
    for band in Band::ALL {
        let top = y;
        let mut lanes = Vec::new();
        if view.nodes.iter().any(|n| band_of(n.node.kind) == band) {
            for lane in lanes_of(band) {
                let most = cells
                    .iter()
                    .filter(|((b, l, _), _)| *b == band && *l == lane)
                    .map(|(_, ids)| count(ids.len()).min(PER_COL))
                    .max()
                    .unwrap_or(0);
                let height = if band == Band::Beads && most == 0 {
                    EMPTY_LANE_H
                } else {
                    most.max(1) * ROW_H + 2 * PAD
                };
                lane_tops.insert((band, lane), y);
                if let Some(kind) = lane {
                    lanes.push(Lane {
                        kind,
                        top: y,
                        height,
                        count: most,
                    });
                }
                y += height;
            }
        } else {
            y += EMPTY_BAND_H;
        }
        bands.push(BandRow {
            band,
            top,
            height: y - top,
            lanes,
        });
    }

    let mut boxes = BTreeMap::new();
    for ((band, lane, rank), ids) in &cells {
        let (Some(col), Some(top)) = (
            columns.iter().find(|c| c.rank == *rank),
            lane_tops.get(&(*band, *lane)),
        ) else {
            continue;
        };
        for (i, id) in ids.iter().enumerate() {
            let i = count(i);
            boxes.insert(
                (*id).to_string(),
                Pos {
                    x: col.x + i / PER_COL * COL_W,
                    y: top + PAD + i % PER_COL * ROW_H,
                },
            );
        }
    }

    Layout {
        width,
        height: y + BOTTOM,
        columns,
        bands,
        boxes,
    }
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
        E::InArticle | E::ArticleRef | E::ParentChild | E::RunOf | E::RanBy => LineStyle::Dotted,
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

/// 線の両端（from の箱の左の辺の中点・to の箱の右の辺の中点・どちらかの箱が無ければ None）。
pub fn line_ends(layout: &Layout, edge: &ViewEdge) -> Option<((f64, f64), (f64, f64))> {
    let a = layout.pos(&edge.from)?;
    let b = layout.pos(&edge.to)?;
    Some((
        (f64::from(a.x), f64::from(a.y + NODE_H / 2)),
        (f64::from(b.x + NODE_W), f64::from(b.y + NODE_H / 2)),
    ))
}

/// 辺の線の道の字。
pub fn edge_path(layout: &Layout, edge: &ViewEdge) -> Option<String> {
    let ((x1, y1), (x2, y2)) = line_ends(layout, edge)?;
    Some(curve(x1, y1, x2, y2))
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
        let mut out: String = chars[..n.saturating_sub(1)].iter().collect();
        out.push('…');
        out
    } else {
        s.to_string()
    }
}

/// 箱の 2 行目の題の字数。
pub const BOX_TITLE_CHARS: usize = 12;

/// 帯の色（CSS の変数）。
fn band_var(band: Band) -> String {
    format!("var(--{})", band.class_name())
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

/// 子の数の札の字（子を持つ節点だけ・語の鍵 children の字と数）。
pub fn kids_badge(n: &ViewNode) -> Option<String> {
    (n.kids > 0).then(|| format!("{} {}", label("children"), n.kids))
}

/// 1 つの節点の箱（囲みは角の丸い四角 1 種・頭の記号・1 行目は id の全部・2 行目は題 12 字・子の数の札）。
/// 箱の字は link にしない（2 回押すか、focus の在るときの Enter と Space で節点の頁へ移る）。
pub fn node_svg(n: &ViewNode, p: Pos) -> String {
    let band = band_of(n.node.kind);
    let lc = band_var(band);
    let open_q = open_question(n);
    let faded = closed(n);
    let (x, y) = (f64::from(p.x), f64::from(p.y));
    let (w, h) = (f64::from(NODE_W), f64::from(NODE_H));
    let (cx, cy) = (x + 11.0, y + 13.0);
    let (fill, stroke) = if open_q {
        ("none".to_string(), "var(--s-stop)".to_string())
    } else {
        (lc.clone(), lc.clone())
    };
    let mark = if round_mark(band) {
        format!(
            r#"<circle cx="{cx}" cy="{cy}" r="5" fill="{fill}" stroke="{stroke}" stroke-width="2"/>"#
        )
    } else {
        format!(
            r#"<rect x="{}" y="{}" width="10" height="10" rx="1" fill="{fill}" stroke="{stroke}" stroke-width="2"/>"#,
            cx - 5.0,
            cy - 5.0
        )
    };
    let (edge, ew) = border(open_q, false);
    let badge = kids_badge(n);
    let id_max = if badge.is_some() {
        w - 26.0 - 50.0
    } else {
        w - 26.0
    };
    let id_len = id_max.min(f64::from(count(n.node.id.chars().count())) * 6.4);
    let id_fill = if faded {
        "var(--ink-3)".to_string()
    } else {
        lc
    };
    let title_fill = if faded { "var(--ink-3)" } else { "var(--ink)" };
    // 組の箱は 1 行目に題・2 行目に種類の語と組の語・縁は点線（id を字にしない）。
    let (aria, head, second) = if n.group {
        fold::group_lines(n, x, y, id_max, &id_fill)
    } else {
        (
            format!("{} {} {}", n.node.id, label(kind_key(n.node.kind)), n.node.title),
            format!(
                r#"<text x="{}" y="{}" font-size="10.5" font-weight="700" font-family="ui-monospace,SFMono-Regular,Menlo,monospace" fill="{id_fill}" textLength="{id_len:.1}" lengthAdjust="spacingAndGlyphs">{}</text>"#,
                x + 21.0,
                y + 16.5,
                esc(&n.node.id)
            ),
            cut(&title36(&n.node.title), BOX_TITLE_CHARS),
        )
    };
    let dash = if n.group { r#" stroke-dasharray="4 2""# } else { "" };
    let toggle = fold::mark_svg(n, p).unwrap_or_default();
    let badge = badge.map_or_else(String::new, |b| {
        format!(
            r#"<rect x="{}" y="{}" width="42" height="16" rx="8" fill="var(--panel-2)" stroke="var(--line-2)"/><text x="{}" y="{}" font-size="10" font-weight="700" text-anchor="middle" fill="var(--ink-2)">{}</text>"#,
            x + w - 46.0,
            y + 4.0,
            x + w - 25.0,
            y + 15.5,
            esc(&b)
        )
    });
    format!(
        r#"<g class="node" data-key="{key}" tabindex="0" aria-label="{aria}"><rect class="hit" x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="var(--panel)" stroke="{edge}" stroke-width="{ew}"{dash}/>{mark}{head}<text x="{}" y="{}" font-size="12" fill="{title_fill}" data-t="">{title}</text>{badge}{toggle}</g>"#,
        x + 8.0,
        y + 32.0,
        key = esc(&n.node.id),
        aria = esc(&aria),
        title = esc(&second),
    )
}

/// 帯の中身（読めなかった出所・読めて節点が無い・節点の数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandFill {
    /// 電文の unread が名指す出所の帯（理由の 1 行）。
    Unread(&'static str),
    /// 読めて節点が無い。
    Zero,
    Nodes(usize),
}

/// 帯の中身（読めなかった出所の帯は節点の数より先に測れていない）。
pub fn band_fill(view: &GraphView, band: Band) -> BandFill {
    if view.unread.contains(&band.source()) {
        return BandFill::Unread(unread_reason(band.source()));
    }
    match view
        .nodes
        .iter()
        .filter(|n| band_of(n.node.kind) == band)
        .count()
    {
        0 => BandFill::Zero,
        n => BandFill::Nodes(n),
    }
}

/// 読めて節点の無い帯の字（帯の語と 0・見本の「run 0」の形）。
pub fn zero_text(band: Band) -> String {
    format!("{} 0", label(band.key()))
}

/// 図の全部（帯の地・行の区切り・測れていないと 0 の字・線・箱）。拡大と移動は `vp` の transform、帯の名は `blabs` に置く。
pub fn svg(view: &GraphView, layout: &Layout) -> String {
    let w = layout.width;
    let mut s = format!(
        r#"<svg class="hlsvg" role="group" aria-label="{}"><g id="vp">"#,
        esc(&label("view_graph"))
    );
    for (i, row) in layout.bands.iter().enumerate() {
        let _ = write!(
            s,
            r#"<rect x="-4000" y="{}" width="{}" height="{}" fill="{}" fill-opacity="{}"/>"#,
            row.top,
            w + 8000,
            row.height,
            band_var(row.band),
            if i % 2 == 1 { ".05" } else { ".09" }
        );
        for lane in row.lanes.iter().skip(1) {
            let _ = write!(
                s,
                r#"<line class="lane-sep" x1="-4000" x2="{}" y1="{}" y2="{}"/>"#,
                w + 4000,
                lane.top,
                lane.top
            );
        }
        match band_fill(view, row.band) {
            BandFill::Unread(reason) => {
                let _ = write!(
                    s,
                    r#"<circle cx="{}" cy="{}" r="5" fill="none" stroke="var(--st-unknown)" stroke-width="2" stroke-dasharray="2 2"/><text x="{}" y="{}" font-size="12" fill="var(--ink-3)">{} · {}</text>"#,
                    LABEL_W + 6,
                    row.top + 23,
                    LABEL_W + 16,
                    row.top + 27,
                    esc(&label("st_unknown")),
                    esc(reason)
                );
            }
            BandFill::Zero => {
                let _ = write!(
                    s,
                    r#"<text x="{LABEL_W}" y="{}" font-size="12" fill="var(--ink-3)">{}</text>"#,
                    row.top + 27,
                    esc(&zero_text(row.band))
                );
            }
            BandFill::Nodes(_) => {}
        }
    }
    for (i, e) in view.edges.iter().enumerate() {
        let Some(((x1, y1), (x2, y2))) = line_ends(layout, e) else {
            continue;
        };
        let _ = write!(
            s,
            r#"<g class="e" data-i="{i}" data-u="{}" data-d="{}">{}</g>"#,
            esc(&e.to),
            esc(&e.from),
            line_svg(line_style(e.edge_type), &curve(x1, y1, x2, y2), (x2, y2))
        );
    }
    for n in &view.nodes {
        if let Some(p) = layout.pos(&n.node.id) {
            s.push_str(&node_svg(n, p));
        }
    }
    s.push_str(r#"</g><g id="blabs"></g></svg>"#);
    s
}

/// 帯の名の欄（図の左端に固定・拡大と移動に合わせて縦の位置と高さだけを変える・見本の initZoom の labels）。
pub fn band_labels(layout: &Layout, z: Zoom, box_h: f64) -> String {
    let lw = f64::from(LABEL_W);
    let mut s =
        format!(r#"<rect x="0" y="0" width="{LABEL_W}" height="{box_h:.1}" fill="var(--panel)"/>"#);
    for row in &layout.bands {
        let b = row.band;
        let y0 = z.ty + f64::from(row.top) * z.s;
        let h = f64::from(row.height) * z.s;
        if y0 > box_h || y0 + h < 0.0 {
            continue;
        }
        let ty = (y0.max(0.0) + 17.0).min(y0 + h - 4.0);
        let color = band_var(b);
        let _ = write!(
            s,
            r#"<g class="blab" data-band="{}" data-term="{}"><rect x="0" y="{y0:.1}" width="{LABEL_W}" height="{h:.1}" fill="{color}" fill-opacity=".14"/><rect x="{}" y="{y0:.1}" width="3" height="{h:.1}" fill="{color}"/>"#,
            esc(b.name()),
            esc(b.key()),
            LABEL_W - 3
        );
        if h >= 18.0 {
            let _ = write!(
                s,
                r#"<text x="8" y="{ty:.1}" font-size="13" font-weight="750" fill="{color}">{}</text>"#,
                esc(b.name())
            );
        }
        if b != Band::Beads && h >= 44.0 && ty + 14.0 < y0 + h - 2.0 {
            let _ = write!(
                s,
                r#"<text x="8" y="{:.1}" font-size="9" fill="var(--ink-3)" font-family="ui-monospace,monospace">{}</text>"#,
                ty + 14.0,
                esc(&cut(&b.path().replace("design-intent/", "di/"), 14))
            );
        }
        s.push_str("</g>");
        for (i, lane) in row.lanes.iter().enumerate() {
            let y0 = z.ty + f64::from(lane.top) * z.s;
            let h = f64::from(lane.height) * z.s;
            if y0 > box_h || y0 + h < 0.0 || h < 11.0 {
                continue;
            }
            if i > 0 {
                let _ = write!(
                    s,
                    r#"<line class="lane-sep" x1="0" x2="{LABEL_W}" y1="{y0:.1}" y2="{y0:.1}"/>"#
                );
            }
            let word = label(kind_key(lane.kind));
            if lane.count == 0 {
                let _ = write!(
                    s,
                    r#"<text class="lane-lab zero" x="{}" y="{:.1}" text-anchor="end">{} 0</text>"#,
                    lw - 8.0,
                    y0 + h / 2.0 + 4.0,
                    esc(&word)
                );
            } else {
                let _ = write!(
                    s,
                    r#"<text class="lane-lab" x="{}" y="{:.1}" text-anchor="end">{}</text>"#,
                    lw - 8.0,
                    y0 + (h - 3.0).min(30.0),
                    esc(&word)
                );
            }
        }
    }
    s
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

/// 狭い幅の一覧の 1 行（印・id・題 36 字・子の数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainRow {
    pub id: String,
    pub title: String,
    pub shape: String,
    pub alert: bool,
    pub kids: Option<u32>,
    /// 組の箱の行か（題だけを出す）。
    pub group: bool,
    pub fold: BoxFold,
}

/// 狭い幅の一覧の 1 つの帯。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainBand {
    pub band: Band,
    pub rows: Vec<ChainRow>,
}

/// 狭い幅の一覧（節点の在る帯だけ・帯の順・帯の中は段の小さい順・同じなら id の自然な順）。
pub fn chain(view: &GraphView) -> Vec<ChainBand> {
    Band::ALL
        .into_iter()
        .filter_map(|band| {
            let mut nodes: Vec<&ViewNode> = view
                .nodes
                .iter()
                .filter(|n| band_of(n.node.kind) == band)
                .collect();
            if nodes.is_empty() {
                return None;
            }
            nodes.sort_by(|a, b| {
                a.rank
                    .cmp(&b.rank)
                    .then_with(|| natural_cmp(&a.node.id, &b.node.id))
            });
            let rows = nodes
                .into_iter()
                .map(|n| {
                    let alert = open_question(n);
                    let open = alert || is_open(n.status.as_deref());
                    ChainRow {
                        id: n.node.id.clone(),
                        title: title36(&fold::plain_title(n)),
                        shape: format!(
                            "shape {}{}",
                            band.class_name(),
                            if open { "" } else { " fill" }
                        ),
                        alert,
                        kids: (n.kids > 0).then_some(n.kids),
                        group: n.group,
                        fold: n.fold,
                    }
                })
                .collect();
            Some(ChainBand { band, rows })
        })
        .collect()
}

/// 数の行（「shown / total ・ 子 folded ・ ✂ cut」・電文の欄の値をそのまま）。
pub fn count_line(view: &GraphView) -> String {
    format!(
        "{} / {} ・ {} {} ・ ✂ {}",
        view.shown,
        view.total,
        label("children"),
        view.folded,
        view.cut
    )
}

/// 経験者向けの 1 行（数の行の下）。
pub fn expert_line(view: &GraphView) -> String {
    format!(
        "shown={} folded={} cut={} total={}",
        view.shown, view.folded, view.cut, view.total
    )
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

/// 倍率の下限。
pub const ZOOM_MIN: f64 = 0.2;
/// 倍率の上限。
pub const ZOOM_MAX: f64 = 4.0;
/// wheel の縦の量に掛ける値（倍率は e のこの積の乗を掛ける）。
pub const WHEEL_RATE: f64 = -0.0015;
/// 箱の高さの下限と上限。
pub const BOX_MIN_H: f64 = 360.0;
pub const BOX_MAX_H: f64 = 720.0;
/// drag と数える pointer の動いた量（これを超えたら drag・離しは押したと数えない）。
pub const DRAG_SLOP: f64 = 4.0;

/// 拡大と移動（倍率・横の移動・縦の移動）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zoom {
    pub s: f64,
    pub tx: f64,
    pub ty: f64,
}

/// 最初の倍率（箱の幅から帯の名の幅を引いた値を、図の幅から帯の名の幅を引いた値で割る・1 より大きくしない）。
/// 箱の幅が測れていない（帯の名の幅以下）ときは 1。
pub fn initial_scale(box_w: f64, width: u32) -> f64 {
    let lw = f64::from(LABEL_W);
    let s = ((box_w - lw) / (f64::from(width) - lw)).min(1.0);
    if s > 0.0 { s } else { 1.0 }
}

/// 箱の高さ（図の高さに最初の倍率を掛けて 8 を足し、360 から 720 に収めて整数に丸める）。
pub fn box_height(height: u32, s0: f64) -> f64 {
    (f64::from(height) * s0 + 8.0)
        .clamp(BOX_MIN_H, BOX_MAX_H)
        .round()
}

/// drag と数えるか。
pub fn dragged(dx: f64, dy: f64) -> bool {
    dx.hypot(dy) > DRAG_SLOP
}

impl Zoom {
    /// 最初（移動 0）。
    pub fn initial(s: f64) -> Self {
        Self {
            s,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// 箱の点 (cx, cy) の下の図の点。
    pub fn to_graph(self, cx: f64, cy: f64) -> (f64, f64) {
        let lw = f64::from(LABEL_W);
        ((cx - lw - self.tx) / self.s + lw, (cy - self.ty) / self.s)
    }

    /// 倍率を変える（0.2 から 4 に収め、箱の点 (cx, cy) の下の図の点を動かさない）。
    pub fn at(self, cx: f64, cy: f64, s: f64) -> Self {
        let lw = f64::from(LABEL_W);
        let ns = s.clamp(ZOOM_MIN, ZOOM_MAX);
        let x = (cx - lw - self.tx) / self.s;
        let y = (cy - self.ty) / self.s;
        Self {
            s: ns,
            tx: cx - lw - x * ns,
            ty: cy - y * ns,
        }
    }

    /// wheel（倍率に e の（縦の量 × −0.0015）乗を掛ける）。
    pub fn wheel(self, cx: f64, cy: f64, delta_y: f64) -> Self {
        self.at(cx, cy, self.s * (delta_y * WHEEL_RATE).exp())
    }

    /// drag（移動の量に pointer の動いた量を足す）。
    pub fn drag(self, dx: f64, dy: f64) -> Self {
        Self {
            s: self.s,
            tx: self.tx + dx,
            ty: self.ty + dy,
        }
    }

    /// 図の transform の字。
    pub fn transform(self) -> String {
        format!(
            "translate({:.2} {:.2}) scale({:.4}) translate(-{LABEL_W} 0)",
            f64::from(LABEL_W) + self.tx,
            self.ty,
            self.s
        )
    }
}

/// focus の在る節点でその節点の頁へ移る key か（見本の ui.js の Enter と Space・KeyboardEvent の key の値）。
pub fn opens_node(key: &str) -> bool {
    matches!(key, "Enter" | " ")
}

/// 組の箱の開き閉じ（開いた箱の列・口の path・右下の印・開けなかった行・src/mapview/graph/fold.rs）。
pub mod fold;

#[cfg(target_arch = "wasm32")]
pub use dom::view;

/// グラフの面の DOM と事件の受け取り（wasm の target のときだけ・src/mapview/graph/dom.rs）。
#[cfg(target_arch = "wasm32")]
mod dom;
