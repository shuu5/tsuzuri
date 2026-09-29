//! 一覧の面（見本の map.html の list）: 絞り（帯・種類・種類の組）と並べ替え（id・更新・状態）の選択と、行の一覧。
//! 行は印・id・題 36 字・帯の chip・種類の語・状態の語・更新の時刻（日本時間の月日と時分・無ければ ―）と、
//! 要約の欄（節点の概要を 80 字で切った字・無ければ「要約なし」）。絞りと並べ替えは URL の query（band・kind・sort・pair）に残す。

use std::collections::BTreeSet;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::graph::{GraphDoc, GraphNode, NodeKind, title36};

use super::band::{Band, band_of, kind_from_name, kind_name};
use super::graph::cut;
use super::{
    VIEW_PARAM, View, kinds_by_id, natural, open_question, param, set_param, shape_class, state,
    unread_reasons,
};
use crate::view::{JST, clock};
use crate::widgets::hover::Card;
use crate::widgets::nodecard::{gist, node_card};

/// 帯の絞りを残す URL の query の鍵。
pub const BAND_PARAM: &str = "band";
/// 種類の絞りを残す URL の query の鍵。
pub const KIND_PARAM: &str = "kind";
/// 並べ替えを残す URL の query の鍵。
pub const SORT_PARAM: &str = "sort";
/// 種類の組の絞りを残す URL の query の鍵。
pub const PAIR_PARAM: &str = "pair";

/// 状態の無い節点の状態の語。
pub const NO_STATE: &str = "状態なし";

/// 更新の時刻の無い節点の更新の欄の字。
pub const NO_UPDATED: &str = "―";

/// 要約の無い節点の要約の欄の字。
pub const NO_GIST: &str = "要約なし";

/// 一覧の行の要約の欄の字数（見本の cut(g, 80)）。
const GIST_CHARS: usize = 80;

/// 並べ替え（閉じた 3・選択の順は見本と同じ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sort {
    Id,
    Updated,
    State,
}

impl Sort {
    pub const ALL: [Sort; 3] = [Sort::Id, Sort::Updated, Sort::State];

    /// URL の query の sort の値。
    pub fn name(self) -> &'static str {
        match self {
            Sort::Id => "id",
            Sort::Updated => "updated",
            Sort::State => "state",
        }
    }

    /// 選択の語の鍵。
    pub fn key(self) -> &'static str {
        match self {
            Sort::Id => "col_id",
            Sort::Updated => "col_updated",
            Sort::State => "col_state",
        }
    }
}

/// 一覧の面の絞りと並べ替え（URL の query から読む・知らない値は読み捨てる）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Query {
    pub band: Option<Band>,
    pub kind: Option<NodeKind>,
    pub sort: Sort,
    pub pair: Option<(NodeKind, NodeKind)>,
}

impl Query {
    pub fn from_search(search: &str) -> Self {
        let get = |key: &str| param(search, key).unwrap_or_default();
        Self {
            band: Band::from_name(&get(BAND_PARAM)),
            kind: kind_from_name(&get(KIND_PARAM)),
            sort: Sort::ALL
                .into_iter()
                .find(|s| s.name() == get(SORT_PARAM))
                .unwrap_or(Sort::Id),
            pair: parse_pair(&get(PAIR_PARAM)),
        }
    }
}

/// 種類の組の query の値（`種類|種類`・辺の from の種類が先）。
pub fn pair_value(from: NodeKind, to: NodeKind) -> String {
    format!("{}|{}", kind_name(from), kind_name(to))
}

/// 種類の組の query の値を読む（形が違うか知らない種類なら None）。
pub fn parse_pair(value: &str) -> Option<(NodeKind, NodeKind)> {
    let (a, b) = value.split_once('|')?;
    Some((kind_from_name(a)?, kind_from_name(b)?))
}

/// 絞りか並べ替えを選んだ後の URL の query（空の値は消す）。
pub fn with_choice(search: &str, key: &str, value: &str) -> String {
    set_param(search, key, Some(value))
}

/// 種類の組で絞った後の URL の query（一覧の面へ移り、帯と種類の絞りを外す・None は組の絞りを外す）。
pub fn with_pair(search: &str, pair: Option<(NodeKind, NodeKind)>) -> String {
    match pair {
        Some((from, to)) => {
            let s = set_param(search, VIEW_PARAM, Some(View::List.name()));
            let s = set_param(&s, PAIR_PARAM, Some(&pair_value(from, to)));
            let s = set_param(&s, BAND_PARAM, None);
            set_param(&s, KIND_PARAM, None)
        }
        None => set_param(search, PAIR_PARAM, None),
    }
}

/// 一覧の 1 行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    /// 題（36 字）。
    pub title: String,
    pub shape: String,
    pub alert: bool,
    pub band: Band,
    pub kind: NodeKind,
    /// 状態の字（bead は属性の状態・走行は属性の段・無ければ None）。
    pub state: Option<String>,
    /// 行の題の hover の card（節点の card）。
    pub card: Card,
    /// 要約の欄の字（節点の概要を 80 字で切った字・無ければ None）。
    pub gist: Option<String>,
    /// 更新の時刻（節点の updated の写し・無ければ None）。
    pub updated: Option<EpochSecs>,
}

impl Row {
    /// 状態の語（無ければ「状態なし」）。
    pub fn state_word(&self) -> &str {
        self.state.as_deref().unwrap_or(NO_STATE)
    }

    /// 更新の時刻の語（日本時間の月日と時分・`09-27 16:39 JST` の形・無ければ ―）。
    pub fn updated_word(&self) -> String {
        match self.updated {
            Some(at) => format!("{} {JST}", &clock(at)[5..16]),
            None => NO_UPDATED.to_string(),
        }
    }
}

/// 種類の組の絞りの札（組の名と絞った行の数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairNote {
    pub from: NodeKind,
    pub to: NodeKind,
    pub count: usize,
}

/// 一覧の面の中身。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub rows: Vec<Row>,
    /// 種類の絞りの選択肢（電文に在る種類・契約の型の順）。
    pub kinds: Vec<NodeKind>,
    pub pair: Option<PairNote>,
    /// 読めなかった出所の理由（その出所の節点は一覧に無い）。
    pub unread: Vec<&'static str>,
}

/// 種類の組の辺（from がはじめの種類・to が後の種類）の両端の節点の id。
pub fn pair_ends(doc: &GraphDoc, from: NodeKind, to: NodeKind) -> BTreeSet<&str> {
    let kinds = kinds_by_id(doc);
    doc.edges
        .iter()
        .filter(|e| {
            kinds.get(e.from.as_str()) == Some(&from) && kinds.get(e.to.as_str()) == Some(&to)
        })
        .flat_map(|e| [e.from.as_str(), e.to.as_str()])
        .collect()
}

/// 状態の並べ替えの順位（open の問い・open と in_progress・状態なし・それ以外）。
pub fn state_rank(doc: &GraphDoc, node: &GraphNode) -> u8 {
    let s = state(doc, node);
    if open_question(doc, node) {
        0
    } else if super::is_open(s) {
        1
    } else if s.is_none() {
        2
    } else {
        3
    }
}

/// 一覧の面の中身を組む（組の絞り・帯・種類の順に絞り、並べ替える）。
pub fn listing(doc: &GraphDoc, q: &Query) -> Listing {
    let mut nodes: Vec<&GraphNode> = doc.nodes.iter().collect();
    let mut pair = None;
    if let Some((from, to)) = q.pair {
        let hit = pair_ends(doc, from, to);
        nodes.retain(|n| hit.contains(n.id.as_str()));
        pair = Some(PairNote {
            from,
            to,
            count: nodes.len(),
        });
    }
    if let Some(band) = q.band {
        nodes.retain(|n| band_of(n.kind) == band);
    }
    if let Some(kind) = q.kind {
        nodes.retain(|n| n.kind == kind);
    }
    match q.sort {
        // 帯の順、次に、どの帯も id の自然な順（電文の順は字の順なので頼らない・便 g-graph）。
        Sort::Id => nodes.sort_by(|a, b| {
            band_of(a.kind)
                .cmp(&band_of(b.kind))
                .then_with(|| natural(&a.id, &b.id))
        }),
        // 新しい順、時刻の無い節点は後ろ、同じなら id の自然な順（見本の並べと同じ）。
        Sort::Updated => nodes.sort_by(|a, b| {
            b.updated
                .cmp(&a.updated)
                .then_with(|| natural(&a.id, &b.id))
        }),
        Sort::State => nodes.sort_by(|a, b| {
            state_rank(doc, a)
                .cmp(&state_rank(doc, b))
                .then_with(|| natural(&a.id, &b.id))
        }),
    }
    Listing {
        rows: nodes.into_iter().map(|n| row(doc, n)).collect(),
        kinds: NodeKind::ALL
            .into_iter()
            .filter(|k| doc.nodes.iter().any(|n| n.kind == *k))
            .collect(),
        pair,
        unread: unread_reasons(doc),
    }
}

fn row(doc: &GraphDoc, node: &GraphNode) -> Row {
    Row {
        id: node.id.clone(),
        title: title36(&node.title),
        shape: shape_class(doc, node),
        alert: open_question(doc, node),
        band: band_of(node.kind),
        kind: node.kind,
        state: state(doc, node).map(str::to_string),
        card: node_card(doc, node),
        gist: gist(node).map(|g| cut(g, GIST_CHARS)),
        updated: node.updated,
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::view;

/// 一覧の面の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;
    use tsuzuri_contract::graph::GraphDoc;

    use super::{
        BAND_PARAM, KIND_PARAM, NO_GIST, PairNote, Query, Row, SORT_PARAM, Sort, listing,
        with_choice, with_pair,
    };
    use crate::frame::{self, Mode};
    use crate::mapview::band::{Band, kind_key, kind_name};
    use crate::mapview::{band_chip, navigate};
    use crate::project::nodearound::mode_of;
    use crate::project::{ALERT_STYLE, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::hs;
    use crate::widgets::hover::attach;

    /// 絞らない選択肢の字。
    const ANY: &str = "―";

    pub fn view(doc: &GraphDoc, search: RwSignal<String>) -> AnyView {
        let q = search.with(|s| Query::from_search(s));
        let l = listing(doc, &q);
        let bands: Vec<(String, String)> = Band::ALL
            .into_iter()
            .map(|b| (b.name().to_string(), b.name().to_string()))
            .collect();
        let kinds: Vec<(String, String)> = l
            .kinds
            .iter()
            .map(|k| (kind_name(*k).to_string(), label(kind_key(*k))))
            .collect();
        let sorts: Vec<(String, String)> = Sort::ALL
            .into_iter()
            .map(|s| (s.name().to_string(), label(s.key())))
            .collect();
        let current_band = q.band.map(|b| b.name().to_string()).unwrap_or_default();
        let current_kind = q.kind.map(|k| kind_name(k).to_string()).unwrap_or_default();
        let unread = l.unread.iter().copied().map(unmeasured).collect_view();
        let pair = l.pair.map(|p| pair_note(p, search));
        let count = l.rows.len();
        let mode = mode_of(search);
        let rows = l.rows.iter().map(|r| row_view(r, mode)).collect_view();
        view! {
            {unread}
            {pair}
            <div class="filters">
                {select("col_band", BAND_PARAM, with_any(bands), current_band, search)}
                {select("col_kind", KIND_PARAM, with_any(kinds), current_kind, search)}
                {select("sort", SORT_PARAM, sorts, q.sort.name().to_string(), search)}
                <span class="chip num">{count}</span>
            </div>
            <ul class="items rows">{rows}</ul>
        }
        .into_any()
    }

    /// 絞らない選択肢を頭に足す。
    fn with_any(mut options: Vec<(String, String)>) -> Vec<(String, String)> {
        options.insert(0, (String::new(), ANY.to_string()));
        options
    }

    /// 1 つの選択（選ぶと URL の query に残す）。
    fn select(
        key: &'static str,
        param: &'static str,
        options: Vec<(String, String)>,
        current: String,
        search: RwSignal<String>,
    ) -> AnyView {
        let options = options
            .into_iter()
            .map(|(value, text)| {
                let selected = value == current;
                view! { <option value=value selected=selected>{text}</option> }
            })
            .collect_view();
        let change = move |ev: leptos::ev::Event| {
            let value = event_target_value(&ev);
            navigate(search, |s| with_choice(s, param, &value), false);
        };
        view! { <label>{hs(key)}<select on:change=change>{options}</select></label> }.into_any()
    }

    /// 種類の組の絞りの札（組の名・数・絞りを外す button）。
    fn pair_note(p: PairNote, search: RwSignal<String>) -> AnyView {
        let clear = move |_| navigate(search, |s| with_pair(s, None), false);
        view! {
            <div class="pairnote">
                {label(kind_key(p.from))}" → "{label(kind_key(p.to))}
                <span class="chip num">{p.count}</span>
                <button type="button" class="btn ghost" on:click=clear>"×"</button>
            </div>
        }
        .into_any()
    }

    /// 1 行（題が節点の頁への link）。
    fn row_view(r: &Row, mode: impl Fn() -> Mode + Copy + Send + Sync + 'static) -> AnyView {
        let style = if r.alert { ALERT_STYLE } else { "" };
        let meta = format!("{} · {}", label(kind_key(r.kind)), r.state_word());
        let id = r.id.clone();
        let href = move || frame::node_href(&id, mode());
        let gist = match r.gist.clone() {
            Some(g) => view! { <span class="gist" data-t="">{g}</span> }.into_any(),
            None => view! { <span class="gist none">{NO_GIST}</span> }.into_any(),
        };
        view! {
            <li>
                <span class=r.shape.clone() style=style aria-hidden="true"></span>
                <div class="rowb">
                    <span class="nid">{r.id.clone()}</span>
                    <a class="ttl" href=href use:attach=r.card.clone()><span data-t="">{r.title.clone()}</span></a>
                    <span class="meta">{band_chip(r.band)}<span>{meta}</span><span class="num">{r.updated_word()}</span></span>
                    {gist}
                </div>
            </li>
        }
        .into_any()
    }
}
