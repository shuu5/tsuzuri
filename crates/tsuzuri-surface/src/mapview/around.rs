//! 節点の近傍（見本の ui.js の svgAround・chainAround と aroundBlock の数の行・便 g-node）。
//! たどる・選ぶ・数えるのは中核の crate が済ませた近傍の電文（契約の型の AroundDoc）で、
//! ここは電文の値を写して座標を決めるだけ（節点を選ぶ分岐と数え直しを書かない）。
//! 図の配置・線の端と道・図の字・狭い幅の一覧の並び・数の行の字は純粋な関数にして host で試す。
//! 箱の縁と記号・線の形と道・光らせ方・凡例はグラフの module の関数を使う（同じ決め方）。

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use tsuzuri_contract::graph::{
    AroundDoc, AroundRow, EdgeType, GraphView, NodeKind, ViewEdge, ViewNode, title36,
};

use super::band::{BEADS_LANES, Band, band_of, kind_key};
use super::graph::{
    BandRow, Lane, NODE_H, Pos, border, closed, curve, cut, esc, line_style, line_svg,
    open_question, round_mark,
};
use super::is_open;
use super::table::edge_name;
use crate::vocab::label;

/// 列の全部（根拠の側は負・中心は 0・影響の側は正）。
pub const COLS: [i8; 7] = [-3, -2, -1, 0, 1, 2, 3];
/// 列の見出しの語の鍵（`COLS` と同じ順）。
pub const COL_KEYS: [&str; 7] = [
    "nb_up3", "nb_up2", "nb_up", "nb_self", "nb_down", "nb_down2", "nb_down3",
];
/// 列の幅を決める全体の幅（列の数で割る）。
pub const SPAN: u32 = 1100;
/// 列の幅の下限と上限。
pub const COL_MIN: u32 = 168;
pub const COL_MAX: u32 = 300;
/// 帯の名の幅（図の左端）。
pub const LABEL_W: u32 = 128;
/// 上の余白（列の見出しの行）。
pub const HEAD: u32 = 28;
/// 箱の縦の送り（箱の高さ 40 と縦の間 8）。
pub const ROW_H: u32 = 48;
/// 行の余白（上下の和）。
pub const PAD: u32 = 12;
/// beads でない帯の高さの下限。
pub const MIN_BAND_H: u32 = 44;
/// 節点の無い beads の行の高さ。
pub const EMPTY_LANE_H: u32 = 18;
/// 列の左端から箱の左端まで（箱の幅は列の幅からこの 2 倍を引いた値）。
pub const INSET: u32 = 10;
/// 図の下の余白。
pub const BOTTOM: u32 = 4;

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// 列の見出しの語の鍵（-3 から 3 の外は None）。
pub fn col_key(col: i8) -> Option<&'static str> {
    COLS.iter().position(|c| *c == col).map(|i| COL_KEYS[i])
}

/// 近傍の図の配置（列・列の幅・箱の幅・題の字数・図の幅と高さ・帯・箱の左上）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub cols: Vec<i8>,
    pub col_w: u32,
    pub box_w: u32,
    pub chars: usize,
    pub width: u32,
    pub height: u32,
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

    pub fn pos(&self, id: &str) -> Option<Pos> {
        self.boxes.get(id).copied()
    }
}

/// 節点の帯と行（beads の帯は種類の行・ほかの帯は 1 行）。
fn lane_key(kind: NodeKind) -> (Band, Option<NodeKind>) {
    let band = band_of(kind);
    (band, (band == Band::Beads).then_some(kind))
}

/// 近傍の配置（升 = 帯と行と列・升の中は電文の rows の順・見本の svgAround と同じ決め方）。
pub fn layout(doc: &AroundDoc) -> Layout {
    let cols: Vec<i8> = COLS
        .into_iter()
        .filter(|c| *c == 0 || doc.rows.iter().any(|r| r.col == *c))
        .collect();
    let n = count(cols.len()).max(1);
    let col_w = (SPAN / n).clamp(COL_MIN, COL_MAX);
    let box_w = col_w - 2 * INSET;
    let chars = usize::try_from(box_w.saturating_sub(16) * 2 / 25).unwrap_or(0);
    let width = LABEL_W + n * col_w;

    let mut cells: BTreeMap<(Band, Option<NodeKind>, i8), Vec<&str>> = BTreeMap::new();
    for r in &doc.rows {
        let (band, lane) = lane_key(r.node.kind);
        cells
            .entry((band, lane, r.col))
            .or_default()
            .push(r.node.id.as_str());
    }
    let most = |band: Band, lane: Option<NodeKind>| {
        cells
            .iter()
            .filter(|((b, l, _), _)| *b == band && *l == lane)
            .map(|(_, ids)| count(ids.len()))
            .max()
            .unwrap_or(0)
    };

    let mut bands = Vec::new();
    let mut lane_tops: BTreeMap<(Band, Option<NodeKind>), u32> = BTreeMap::new();
    let mut y = HEAD;
    for band in Band::ALL {
        if !doc.rows.iter().any(|r| band_of(r.node.kind) == band) {
            continue;
        }
        let top = y;
        let mut lanes = Vec::new();
        if band == Band::Beads {
            for kind in BEADS_LANES {
                let m = most(band, Some(kind));
                let height = if m > 0 { m * ROW_H + PAD } else { EMPTY_LANE_H };
                lane_tops.insert((band, Some(kind)), y);
                lanes.push(Lane {
                    kind,
                    top: y,
                    height,
                    count: m,
                });
                y += height;
            }
        } else {
            lane_tops.insert((band, None), y);
            y += MIN_BAND_H.max(most(band, None).max(1) * ROW_H + PAD);
        }
        bands.push(BandRow {
            band,
            top,
            height: y - top,
            lanes,
        });
    }

    let mut boxes = BTreeMap::new();
    for ((band, lane, col), ids) in &cells {
        let (Some(i), Some(top)) = (
            cols.iter().position(|c| c == col),
            lane_tops.get(&(*band, *lane)),
        ) else {
            continue;
        };
        for (j, id) in ids.iter().enumerate() {
            boxes.insert(
                (*id).to_string(),
                Pos {
                    x: LABEL_W + count(i) * col_w + INSET,
                    y: top + PAD / 2 + count(j) * ROW_H,
                },
            );
        }
    }

    Layout {
        cols,
        col_w,
        box_w,
        chars,
        width,
        height: y + BOTTOM,
        bands,
        boxes,
    }
}

/// 線の両端（1 つ前の節点の箱から着いた節点の箱へ・右に在れば右の辺から左の辺へ・左に在れば左の辺から右の辺へ・
/// 同じ列なら横の中点どうし・どちらかの箱が無ければ None）。
pub fn line_ends(layout: &Layout, via: &str, to: &str) -> Option<((f64, f64), (f64, f64))> {
    let a = layout.pos(via)?;
    let b = layout.pos(to)?;
    let w = f64::from(layout.box_w);
    let (ax, bx) = (f64::from(a.x), f64::from(b.x));
    let (x1, x2) = match b.x.cmp(&a.x) {
        Ordering::Greater => (ax + w, bx),
        Ordering::Less => (ax, bx + w),
        Ordering::Equal => (ax + w / 2.0, bx + w / 2.0),
    };
    let half = f64::from(NODE_H) / 2.0;
    Some(((x1, f64::from(a.y) + half), (x2, f64::from(b.y) + half)))
}

/// 1 つの行の線の道の字（1 つ前の節点の無い行と箱の無い行は None）。
pub fn edge_path(layout: &Layout, row: &AroundRow) -> Option<String> {
    let via = row.via.as_deref()?;
    let ((x1, y1), (x2, y2)) = line_ends(layout, via, &row.node.id)?;
    Some(curve(x1, y1, x2, y2))
}

/// 線を持つ行（1 つ前の節点と着いた辺の型を持つ行・rows の順）。
fn linked(doc: &AroundDoc) -> impl Iterator<Item = (&AroundRow, &str, EdgeType)> {
    doc.rows
        .iter()
        .filter_map(|r| Some((r, r.via.as_deref()?, r.edge_type?)))
}

/// 行の線を眺めの辺にする（to が根拠の側の端: 根拠の側の行では着いた節点、影響の側の行では 1 つ前の節点）。
fn edge_of(row: &AroundRow, via: &str, t: EdgeType) -> ViewEdge {
    let (from, to) = if row.col < 0 {
        (via.to_string(), row.node.id.clone())
    } else {
        (row.node.id.clone(), via.to_string())
    };
    ViewEdge {
        from,
        to,
        edge_type: t,
        count: 1,
    }
}

/// 線の全部（rows の順・図の線の data-i はこの番号）。
pub fn edges(doc: &AroundDoc) -> Vec<ViewEdge> {
    linked(doc).map(|(r, via, t)| edge_of(r, via, t)).collect()
}

/// 行を眺めの節点にする（段と子の数は近傍に無いので 0）。
pub fn view_node(row: &AroundRow) -> ViewNode {
    ViewNode {
        node: row.node.clone(),
        status: row.status.clone(),
        rank: 0,
        kids: 0,
        degree: row.degree,
    }
}

/// 近傍を眺めの電文の形に写す（グラフの module の凡例と光らせ方に渡す）。
pub fn as_view(doc: &AroundDoc) -> GraphView {
    GraphView {
        nodes: doc.rows.iter().map(view_node).collect(),
        edges: edges(doc),
        shown: doc.shown,
        folded: 0,
        cut: doc.cut_hub + doc.cut_cap,
        total: doc.total,
        unread: doc.unread.clone(),
    }
}

/// 帯の色（CSS の変数）。
fn band_var(band: Band) -> String {
    format!("var(--{})", band.class_name())
}

/// 1 つの節点の箱（グラフの module の縁と記号の決め方・幅と題の字数は近傍の列から・中心は太い縁と太い題）。
pub fn node_box(n: &ViewNode, p: Pos, w: u32, chars: usize, center: bool) -> String {
    let band = band_of(n.node.kind);
    let lc = band_var(band);
    let open_q = open_question(n);
    let faded = closed(n);
    let (x, y) = (f64::from(p.x), f64::from(p.y));
    let (w, h) = (f64::from(w), f64::from(NODE_H));
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
    let (edge, ew) = border(open_q, center);
    let id_len = (w - 26.0).min(f64::from(count(n.node.id.chars().count())) * 6.4);
    let id_fill = if faded {
        "var(--ink-3)".to_string()
    } else {
        lc
    };
    let title_fill = if faded { "var(--ink-3)" } else { "var(--ink)" };
    let weight = if center { r#" font-weight="700""# } else { "" };
    let aria = format!(
        "{} {} {}",
        n.node.id,
        label(kind_key(n.node.kind)),
        n.node.title
    );
    format!(
        r#"<g class="node" data-key="{key}" tabindex="0" aria-label="{aria}"><rect class="hit" x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="var(--panel)" stroke="{edge}" stroke-width="{ew}"/>{mark}<text x="{}" y="{}" font-size="10.5" font-weight="700" font-family="ui-monospace,SFMono-Regular,Menlo,monospace" fill="{id_fill}" textLength="{id_len:.1}" lengthAdjust="spacingAndGlyphs">{key}</text><text x="{}" y="{}" font-size="12" fill="{title_fill}"{weight} data-t="">{title}</text></g>"#,
        x + 21.0,
        y + 16.5,
        x + 8.0,
        y + 32.0,
        key = esc(&n.node.id),
        aria = esc(&aria),
        title = esc(&cut(&title36(&n.node.title), chars)),
    )
}

/// beads の行の名（種類の語・節点の無い行は「 0」を足す）。
pub fn lane_label(lane: &Lane) -> String {
    let word = label(kind_key(lane.kind));
    if lane.count == 0 {
        format!("{word} 0")
    } else {
        word
    }
}

/// 図の全部（列の見出し・帯の地と名・beads の行・線・箱）。
pub fn svg(doc: &AroundDoc, layout: &Layout) -> String {
    let (w, h) = (layout.width, layout.height);
    let mut s = format!(
        r#"<svg class="hlsvg" viewBox="0 0 {w} {h}" style="max-width:{w}px" role="group" aria-label="{}">"#,
        esc(&label("around"))
    );
    for (i, col) in layout.cols.iter().enumerate() {
        let key = col_key(*col).unwrap_or("nb_self");
        let _ = write!(
            s,
            r#"<text x="{}" y="18" font-size="12" font-weight="600" text-anchor="middle" fill="var(--ink-3)">{}</text>"#,
            LABEL_W + count(i) * layout.col_w + layout.col_w / 2,
            esc(&label(key))
        );
    }
    for (i, row) in layout.bands.iter().enumerate() {
        let color = band_var(row.band);
        let _ = write!(
            s,
            r#"<rect x="0" y="{}" width="{w}" height="{}" fill="{color}" fill-opacity="{}"/><text x="8" y="{}" font-size="12" font-weight="700" fill="{color}">{}</text>"#,
            row.top,
            row.height,
            if i % 2 == 1 { ".05" } else { ".09" },
            row.top + 14,
            esc(&label(row.band.key()))
        );
        if row.band != Band::Beads {
            let _ = write!(
                s,
                r#"<text x="8" y="{}" font-size="9" fill="var(--ink-3)" font-family="ui-monospace,monospace">{}</text>"#,
                row.top + 30,
                esc(&cut(&row.band.path().replace("design-intent/", "di/"), 15))
            );
        }
        for (j, lane) in row.lanes.iter().enumerate() {
            if j > 0 {
                let _ = write!(
                    s,
                    r#"<line class="lane-sep" x1="0" x2="{w}" y1="{}" y2="{}"/>"#,
                    lane.top, lane.top
                );
            }
            let (x, word) = (LABEL_W - 6, esc(&lane_label(lane)));
            let _ = if lane.count == 0 {
                write!(
                    s,
                    r#"<text class="lane-lab zero" x="{x}" y="{}" text-anchor="end">{word}</text>"#,
                    lane.top + 13
                )
            } else {
                write!(
                    s,
                    r#"<text class="lane-lab" x="{x}" y="{}" text-anchor="end">{word}</text>"#,
                    lane.top + 16
                )
            };
        }
    }
    for (i, (row, via, t)) in linked(doc).enumerate() {
        let Some(((x1, y1), (x2, y2))) = line_ends(layout, via, &row.node.id) else {
            continue;
        };
        let e = edge_of(row, via, t);
        let _ = write!(
            s,
            r#"<g class="e" data-i="{i}" data-u="{}" data-d="{}">{}</g>"#,
            esc(&e.to),
            esc(&e.from),
            line_svg(line_style(t), &curve(x1, y1, x2, y2), (x2, y2))
        );
    }
    for r in &doc.rows {
        if let Some(p) = layout.pos(&r.node.id) {
            s.push_str(&node_box(
                &view_node(r),
                p,
                layout.box_w,
                layout.chars,
                r.col == 0,
            ));
        }
    }
    s.push_str("</svg>");
    s
}

/// 狭い幅の一覧の 1 行（印・id・題 36 字・右に辺の型の字・入れ子の行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainItem {
    pub id: String,
    pub title: String,
    pub shape: String,
    pub alert: bool,
    /// 着いた辺の型の字。
    pub edge: Option<String>,
    /// その先の行（1 段目の行だけが持つ・rows の順に深さ優先）。
    pub nest: Vec<ChainItem>,
}

/// 狭い幅の一覧の 1 段（根拠の側か影響の側・語の鍵と 1 段目の行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainSide {
    pub key: &'static str,
    pub items: Vec<ChainItem>,
}

fn chain_item(row: &AroundRow) -> ChainItem {
    let n = view_node(row);
    let alert = open_question(&n);
    let open = alert || is_open(row.status.as_deref());
    ChainItem {
        id: row.node.id.clone(),
        title: title36(&row.node.title),
        shape: format!(
            "shape {}{}",
            band_of(row.node.kind).class_name(),
            if open { "" } else { " fill" }
        ),
        alert,
        edge: row.edge_type.map(edge_name),
        nest: Vec::new(),
    }
}

/// 狭い幅の一覧（見本の chainAround: 根拠の側と影響の側の 2 段・1 段目は中心の隣・その先は入れ子・行の無い側は出さない）。
pub fn chain(doc: &AroundDoc) -> Vec<ChainSide> {
    let mut kids: BTreeMap<&str, Vec<&AroundRow>> = BTreeMap::new();
    for r in &doc.rows {
        if let Some(via) = r.via.as_deref()
            && r.col != 0
        {
            kids.entry(via).or_default().push(r);
        }
    }
    fn desc<'a>(
        kids: &BTreeMap<&str, Vec<&'a AroundRow>>,
        id: &str,
        seen: &mut BTreeSet<String>,
        out: &mut Vec<&'a AroundRow>,
    ) {
        for r in kids.get(id).into_iter().flatten() {
            if seen.insert(r.node.id.clone()) {
                out.push(r);
                desc(kids, &r.node.id, seen, out);
            }
        }
    }
    [(-1_i8, "nb_up"), (1, "nb_down")]
        .into_iter()
        .filter_map(|(sign, key)| {
            let items: Vec<ChainItem> = kids
                .get(doc.center.as_str())
                .into_iter()
                .flatten()
                .filter(|r| r.col.signum() == sign)
                .map(|r| {
                    let mut seen = BTreeSet::from([doc.center.clone(), r.node.id.clone()]);
                    let mut below = Vec::new();
                    desc(&kids, &r.node.id, &mut seen, &mut below);
                    ChainItem {
                        nest: below.into_iter().map(chain_item).collect(),
                        ..chain_item(r)
                    }
                })
                .collect();
            (!items.is_empty()).then_some(ChainSide { key, items })
        })
        .collect()
}

/// 切った数（hub で隠れた数と上限で切った数の和）。
pub fn cut_count(doc: &AroundDoc) -> u32 {
    doc.cut_hub + doc.cut_cap
}

/// 数の行（「shown / total」・切った数が 1 以上なら「 ・ ✂ 」と数を足す）。
pub fn count_line(doc: &AroundDoc) -> String {
    let mut s = format!("{} / {}", doc.shown, doc.total);
    let c = cut_count(doc);
    if c > 0 {
        let _ = write!(s, " ・ ✂ {c}");
    }
    s
}

/// 経験者向けの 1 行（up と down は、畳んだ側が 0・畳んでいない側が電文の steps）。
pub fn expert_line(doc: &AroundDoc) -> String {
    let up = if doc.fold.folds_basis() { 0 } else { doc.steps };
    let down = if doc.fold.folds_impact() {
        0
    } else {
        doc.steps
    };
    format!(
        "shown={} total={} up={up} down={down} cut.hub={} cut.cap={}",
        doc.shown, doc.total, doc.cut_hub, doc.cut_cap
    )
}
