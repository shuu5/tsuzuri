//! 圧縮の面（見本の map.html の compact）: 帯ごとに 1 つの箱を帯の順に置き、箱の中に札（印・id・題 30 字）を並べる。
//! constitution は条の札（id の自然な順）の下に規範文の id・ほかの帯は id の自然な順・beads は種類の 7 行。
//! 電文の順は字の順（P-1 の次が P-10）なので、条も規則行も電文の順に頼らない（便 g-graph）。
//! 読めなかった出所の帯は測れていない（理由の 1 行）・読めて 0 件の帯は 0 件の帯（要件 NFR2）。

use tsuzuri_contract::graph::{GraphDoc, GraphNode, NodeKind, title36};

use super::band::{BEADS_LANES, Band, band_of, kind_key, unread_reason};
use super::{natural, open_question, shape_class};

/// 札の題の字数の上限。
pub const TAG_TITLE_MAX: usize = 30;

/// 札の題（空白を 1 つに畳んで 30 字に切る）。
pub fn cut30(s: &str) -> String {
    title36(s).chars().take(TAG_TITLE_MAX).collect()
}

/// 1 枚の札（見本の `.tag`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: String,
    pub title: String,
    /// 札の class（`tag` と帯の class・open の問いは `open-q`）。
    pub class: String,
    /// 印の class。
    pub shape: String,
    /// open の問い（印は赤）。
    pub alert: bool,
}

/// 条の札と、その条の規範文の id（見本の `.art7`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    pub tag: Tag,
    pub norms: Vec<String>,
}

/// beads の帯の種類の 1 行（小見出しと数と札）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lane {
    pub kind: NodeKind,
    pub key: &'static str,
    pub tags: Vec<Tag>,
}

/// 帯の箱の中身。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cards {
    /// 出所が読めなかった（理由の 1 行）。
    Unmeasured(&'static str),
    /// 読めて 0 件。
    Empty,
    Tags(Vec<Tag>),
    /// 条の札（id の自然な順）と、条の見つからない規範文の札（id の自然な順）。
    Articles {
        articles: Vec<Article>,
        loose: Vec<Tag>,
    },
    Lanes(Vec<Lane>),
}

/// 1 つの帯の箱（見本の `section.band`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandBox {
    pub band: Band,
    /// 節点の数（測れていなければ None）。
    pub count: Option<usize>,
    pub cards: Cards,
}

impl BandBox {
    /// 箱の class（`band` と帯の class）。
    pub fn class(&self) -> String {
        format!("band {}", self.band.class_name())
    }

    /// 箱に出す id の全部（出す順）。
    pub fn ids(&self) -> Vec<&str> {
        fn tag_ids(tags: &[Tag]) -> impl Iterator<Item = &str> {
            tags.iter().map(|t| t.id.as_str())
        }
        match &self.cards {
            Cards::Unmeasured(_) | Cards::Empty => Vec::new(),
            Cards::Tags(tags) => tag_ids(tags).collect(),
            Cards::Articles { articles, loose } => articles
                .iter()
                .flat_map(|a| {
                    std::iter::once(a.tag.id.as_str()).chain(a.norms.iter().map(String::as_str))
                })
                .chain(tag_ids(loose))
                .collect(),
            Cards::Lanes(lanes) => lanes.iter().flat_map(|l| tag_ids(&l.tags)).collect(),
        }
    }
}

/// 札を組む。
pub fn tag(doc: &GraphDoc, node: &GraphNode) -> Tag {
    let alert = open_question(doc, node);
    Tag {
        id: node.id.clone(),
        title: cut30(&node.title),
        class: format!(
            "tag {}{}",
            band_of(node.kind).class_name(),
            if alert { " open-q" } else { "" }
        ),
        shape: shape_class(doc, node),
        alert,
    }
}

/// 圧縮の面の中身（7 つの帯の箱・帯の順）。
pub fn compact(doc: &GraphDoc) -> Vec<BandBox> {
    Band::ALL
        .into_iter()
        .map(|band| band_box(doc, band))
        .collect()
}

fn band_box(doc: &GraphDoc, band: Band) -> BandBox {
    if doc.unread.contains(&band.source()) {
        return BandBox {
            band,
            count: None,
            cards: Cards::Unmeasured(unread_reason(band.source())),
        };
    }
    let mine: Vec<&GraphNode> = doc
        .nodes
        .iter()
        .filter(|n| band_of(n.kind) == band)
        .collect();
    let count = Some(mine.len());
    let cards = match band {
        _ if mine.is_empty() => Cards::Empty,
        Band::Constitution => articles(doc, &mine),
        Band::Beads => Cards::Lanes(lanes(doc, &mine)),
        _ => Cards::Tags(natural_tags(doc, mine)),
    };
    BandBox { band, count, cards }
}

/// 節点を id の自然な順の札にする。
fn natural_tags(doc: &GraphDoc, mut nodes: Vec<&GraphNode>) -> Vec<Tag> {
    nodes.sort_by(|a, b| natural(&a.id, &b.id));
    nodes.into_iter().map(|n| tag(doc, n)).collect()
}

/// 規範文の条の id（`P-1.2` なら `P-1`・末尾が「.」と数でなければ None）。
pub fn article_of(norm: &str) -> Option<&str> {
    norm.rsplit_once('.')
        .filter(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .map(|(article, _)| article)
}

/// constitution の帯: 条の札（id の自然な順）の下に規範文の id（自然な順）。
fn articles(doc: &GraphDoc, nodes: &[&GraphNode]) -> Cards {
    let mut arts: Vec<&GraphNode> = nodes
        .iter()
        .copied()
        .filter(|n| n.kind == NodeKind::Article)
        .collect();
    arts.sort_by(|a, b| natural(&a.id, &b.id));
    let mut norms: Vec<&GraphNode> = nodes
        .iter()
        .copied()
        .filter(|n| n.kind == NodeKind::Norm)
        .collect();
    norms.sort_by(|a, b| natural(&a.id, &b.id));
    let articles = arts
        .iter()
        .map(|a| Article {
            tag: tag(doc, a),
            norms: norms
                .iter()
                .filter(|s| article_of(&s.id) == Some(a.id.as_str()))
                .map(|s| s.id.clone())
                .collect(),
        })
        .collect();
    let loose = norms
        .into_iter()
        .filter(|s| {
            !arts
                .iter()
                .any(|a| article_of(&s.id) == Some(a.id.as_str()))
        })
        .map(|s| tag(doc, s))
        .collect();
    Cards::Articles { articles, loose }
}

/// beads の帯: 種類の 7 行（行の中は id の自然な順・0 件の行も出す）。
fn lanes(doc: &GraphDoc, nodes: &[&GraphNode]) -> Vec<Lane> {
    BEADS_LANES
        .into_iter()
        .map(|kind| Lane {
            kind,
            key: kind_key(kind),
            tags: natural_tags(
                doc,
                nodes.iter().copied().filter(|n| n.kind == kind).collect(),
            ),
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
pub use dom::view;

/// 圧縮の面の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;
    use tsuzuri_contract::graph::GraphDoc;

    use super::{BandBox, Cards, Tag, compact};
    use crate::frame::{self, Mode};
    use crate::mapview::band::Band;
    use crate::mapview::current;
    use crate::project::nodearound::mode_of;
    use crate::project::{ALERT_STYLE, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{h2, hs};

    /// 測れていない帯の数の字。
    const NO_COUNT: &str = "―";

    pub fn view(doc: &GraphDoc) -> AnyView {
        let mode = mode_of(RwSignal::new(current()));
        let boxes = compact(doc)
            .into_iter()
            .map(|b| box_view(b, mode))
            .collect_view();
        view! {
            <header>{h2("bands")}</header>
            {boxes}
        }
        .into_any()
    }

    fn box_view(b: BandBox, mode: impl Fn() -> Mode + Copy + Send + Sync + 'static) -> AnyView {
        let count = b
            .count
            .map_or_else(|| NO_COUNT.to_string(), |n| n.to_string());
        let class = b.class();
        let band = b.band;
        view! {
            <section class=class>
                <header>
                    <span class="bn">{hs(band.key())}</span>
                    <code class="path">{band.path()}</code>
                    <span class="n num">{count}</span>
                </header>
                {cards_view(b.cards, band, mode)}
            </section>
        }
        .into_any()
    }

    fn cards_view(
        cards: Cards,
        band: Band,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
    ) -> AnyView {
        match cards {
            Cards::Unmeasured(reason) => unmeasured(reason),
            Cards::Empty => view! {
                <div class="empty"><span>{label(band.key())}</span><b class="num">"0"</b></div>
            }
            .into_any(),
            Cards::Tags(tags) => {
                let tags = tags.iter().map(|t| tag_view(t, mode)).collect_view();
                view! { <div class="cards7">{tags}</div> }.into_any()
            }
            Cards::Articles { articles, loose } => {
                let arts = articles
                    .into_iter()
                    .map(|a| {
                        let class = format!("art7 {}", band.class_name());
                        let kids = a
                            .norms
                            .into_iter()
                            .map(|id| {
                                let href = {
                                    let id = id.clone();
                                    move || frame::node_href(&id, mode())
                                };
                                view! { <a href=href>{id}</a> }
                            })
                            .collect_view();
                        let tag_id = a.tag.id.clone();
                        let href = move || frame::node_href(&tag_id, mode());
                        view! {
                            <div class=class>
                                <a href=href>{tag_head(&a.tag)}</a>
                                <div class="kids">{kids}</div>
                            </div>
                        }
                    })
                    .collect_view();
                let loose = loose.iter().map(|t| tag_view(t, mode)).collect_view();
                view! { <div class="cards7">{arts}{loose}</div> }.into_any()
            }
            Cards::Lanes(lanes) => {
                let shape = format!("shape {} fill", band.class_name());
                let lanes = lanes
                    .into_iter()
                    .map(|l| {
                        let tags = l.tags.iter().map(|t| tag_view(t, mode)).collect_view();
                        view! {
                            <div class="subh">
                                <span class=shape.clone() aria-hidden="true"></span>
                                {hs(l.key)}
                                <span class="num muted">{l.tags.len()}</span>
                            </div>
                            {tags}
                        }
                    })
                    .collect_view();
                view! { <div class="cards7">{lanes}</div> }.into_any()
            }
        }
    }

    /// 札の印と id と題（包む a が節点の頁への link）。
    fn tag_head(t: &Tag) -> AnyView {
        let style = if t.alert { ALERT_STYLE } else { "" };
        view! {
            <span class="tid"><span class=t.shape.clone() style=style aria-hidden="true"></span>" "{t.id.clone()}</span>
            <span class="tt" data-t="">{t.title.clone()}</span>
        }
        .into_any()
    }

    /// 1 つの札（節点の頁への link）。
    fn tag_view(t: &Tag, mode: impl Fn() -> Mode + Copy + Send + Sync + 'static) -> AnyView {
        let id = t.id.clone();
        let href = move || frame::node_href(&id, mode());
        view! { <a class=t.class.clone() href=href>{tag_head(t)}</a> }.into_any()
    }
}
