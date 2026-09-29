//! block「つながり」（見本の bead.html の近傍と ui.js の aroundBlock・nbK・nbFold・便 g-node）: 節点の頁の 2 つ目の block。
//! 口の本文を契約の型の AroundDoc に読み、図と一覧と数の行は mapview の around が組む（ここは写すだけ）。
//! 口の path は URL の query（id・k・fold）で変わるので、path の字の signal を通信の module の読みの関数に渡す。
//! 段数と畳みは URL の query の k と fold に残し（履歴に積まない）、変われば新しい query で口を読み直す。
//! 口の path・query の読み書き・畳みの移り方・畳みの button・頁の 4 つの状態は純粋な関数にして host で試す。

use std::fmt::Write;

use tsuzuri_contract::graph::{AroundDoc, Fold};
use tsuzuri_contract::wire;

use super::{NO_CONTENT, NOT_READ};
use crate::frame::Block;
use crate::mapview::{encode, param, set_param};
use crate::view::Fetched;

pub const BLOCK: Block = Block {
    id: "around",
    heading: "around",
    class: "panel",
};

/// 読みの口の path の頭（query の前・server の便 e-view が足す）。
pub const PATH: &str = "/api/around";

/// この file が字を持つ口の path（ほかの module の口を読む所は数えない・行 hs-derived）。
pub const PATHS: &[&str] = &[PATH];

/// この file の畳める段の開き閉じの鍵の形（無い・行 hs-derived）。
pub const FOLDS: &[&str] = &[];

/// 口が読めないときの理由。
pub const REASON: &str =
    "節点の近傍を組む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 節点が無い id に口が返す状態の数。
pub const NOT_FOUND: u16 = 404;

/// 中心の節点の id を残す URL の query の鍵。
pub const ID_PARAM: &str = "id";
/// 段数を残す URL の query の鍵。
pub const K_PARAM: &str = "k";
/// 畳みを残す URL の query の鍵。
pub const FOLD_PARAM: &str = "fold";

/// 段数の選択肢（段数の切り替えの button の順）。
pub const K_CHOICES: [u8; 3] = [1, 2, 3];
/// 段数の既定（URL と口の path に付けない）。
pub const K_DEFAULT: u8 = 2;

/// 畳みの query の字（畳みなしは None）。
pub fn fold_name(fold: Fold) -> Option<&'static str> {
    match fold {
        Fold::None => None,
        Fold::Up => Some("up"),
        Fold::Down => Some("down"),
        Fold::Both => Some("both"),
    }
}

/// 口の path（id は `%XX` にする・段数 2 と畳みなしは付けない）。
pub fn path(id: &str, k: u8, fold: Fold) -> String {
    let mut s = format!("{PATH}?{ID_PARAM}={}", encode(id));
    if k != K_DEFAULT {
        let _ = write!(s, "&{K_PARAM}={k}");
    }
    if let Some(f) = fold_name(fold) {
        let _ = write!(s, "&{FOLD_PARAM}={f}");
    }
    s
}

/// URL の query の中心の節点の id（無いか空なら None）。
pub fn id_of(search: &str) -> Option<String> {
    param(search, ID_PARAM).filter(|id| !id.is_empty())
}

/// URL の query の段数（1・2・3 のほかと無いときは 2）。
pub fn k_of(search: &str) -> u8 {
    param(search, K_PARAM)
        .and_then(|k| k.parse::<u8>().ok())
        .filter(|k| K_CHOICES.contains(k))
        .unwrap_or(K_DEFAULT)
}

/// URL の query の畳み（up・down・both のほかと無いときは畳みなし）。
pub fn fold_of(search: &str) -> Fold {
    let value = param(search, FOLD_PARAM);
    Fold::ALL
        .into_iter()
        .find(|f| fold_name(*f).is_some() && fold_name(*f) == value.as_deref())
        .unwrap_or(Fold::None)
}

/// 段数を変えた後の URL の query（2 は消す・ほかの値と順はそのまま）。
pub fn with_k(search: &str, k: u8) -> String {
    let value = (k != K_DEFAULT).then(|| k.to_string());
    set_param(search, K_PARAM, value.as_deref())
}

/// 畳みを変えた後の URL の query（畳みなしは消す・ほかの値と順はそのまま）。
pub fn with_fold(search: &str, fold: Fold) -> String {
    set_param(search, FOLD_PARAM, fold_name(fold))
}

/// 中心の id を引数で受けた口の path（段数と畳みは URL の query から・query の id は見ない・問いの card の図）。
pub fn center_path(center: &str, search: &str) -> String {
    path(center, k_of(search), fold_of(search))
}

/// URL の query から口の path（id が無いか空なら None で、口を読まない）。
pub fn request(search: &str) -> Option<String> {
    id_of(search).map(|id| center_path(&id, search))
}

/// 埋め込みの図の口の path（段を開くまでは空の字＝通信の module は空の path を読まない）。
pub fn embed_path(center: &str, search: &str, opened: bool) -> String {
    if opened {
        center_path(center, search)
    } else {
        String::new()
    }
}

/// 埋め込みの図で、口が中心の節点を見つけないときの理由。
pub const NO_NODE: &str =
    "近傍の口がこの id の節点を見つけない（台帳か設計の索引が読めないか、節点が無い）";

/// 畳みの button の側（閉じた 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldSide {
    Basis,
    Impact,
}

impl FoldSide {
    pub const ALL: [FoldSide; 2] = [FoldSide::Basis, FoldSide::Impact];

    /// 語の鍵。
    pub fn key(self) -> &'static str {
        match self {
            FoldSide::Basis => "nb_up",
            FoldSide::Impact => "nb_down",
        }
    }

    /// その側を畳んでいるか。
    pub fn folded(self, fold: Fold) -> bool {
        match self {
            FoldSide::Basis => fold.folds_basis(),
            FoldSide::Impact => fold.folds_impact(),
        }
    }
}

/// 畳みの button を押した後の畳み（押した側だけを開き閉じする）。
pub fn toggle(fold: Fold, side: FoldSide) -> Fold {
    let mut up = fold.folds_basis();
    let mut down = fold.folds_impact();
    match side {
        FoldSide::Basis => up = !up,
        FoldSide::Impact => down = !down,
    }
    match (up, down) {
        (false, false) => Fold::None,
        (true, false) => Fold::Up,
        (false, true) => Fold::Down,
        (true, true) => Fold::Both,
    }
}

/// 畳みの button の値（側・語の鍵・畳んだか・印の字・数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SideButton {
    pub side: FoldSide,
    pub key: &'static str,
    pub folded: bool,
    /// 開いていれば「▾」・畳んでいれば「▸」。
    pub glyph: &'static str,
    /// 畳みなしでたどったときの数（電文の basis か impact・畳んでも消えない）。
    pub count: u32,
}

/// 根拠の側と影響の側の畳みの button（この順）。
pub fn buttons(doc: &AroundDoc) -> [SideButton; 2] {
    FoldSide::ALL.map(|side| {
        let folded = side.folded(doc.fold);
        SideButton {
            side,
            key: side.key(),
            folded,
            glyph: if folded { "▸" } else { "▾" },
            count: match side {
                FoldSide::Basis => doc.basis,
                FoldSide::Impact => doc.impact,
            },
        }
    })
}

/// 節点の頁の 4 つの状態。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageState {
    /// まだ読んでいない（頁を開いた直後・query を変えた直後）。
    NotRead,
    /// 口が節点を見つけない（状態の数 404）。
    NotFound,
    /// 読めない（理由の 1 行・口が読めない・本文が電文の型として読めない）。
    Unread(&'static str),
    /// 電文あり。
    Doc(AroundDoc),
}

/// 読みの結果と応答の状態の数（応答が無ければ None）の組から頁の状態を決める（状態の数が 404 なら見つからない）。
pub fn state(fetched: &Fetched, status: Option<u16>) -> PageState {
    if status == Some(NOT_FOUND) {
        return PageState::NotFound;
    }
    match fetched {
        Fetched::NotRead => PageState::NotRead,
        Fetched::Failed => PageState::Unread(REASON),
        Fetched::Body(text) => match wire::decode::<AroundDoc>(text) {
            Ok(doc) => PageState::Doc(doc),
            Err(_) => PageState::Unread(NO_CONTENT),
        },
    }
}

/// 測れていないときの理由（まだ読んでいない・読めない・ほかは None）。
pub fn unmeasured_reason(state: &PageState) -> Option<&'static str> {
    match state {
        PageState::NotRead => Some(NOT_READ),
        PageState::Unread(reason) => Some(reason),
        PageState::NotFound | PageState::Doc(_) => None,
    }
}

/// 埋め込みの図の理由の 1 行（見つからないも理由で出す・ほかは測れていないときの理由と同じ）。
pub fn embed_reason(state: &PageState) -> Option<&'static str> {
    match state {
        PageState::NotFound => Some(NO_NODE),
        other => unmeasured_reason(other),
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::{Embeds, Source, embeds, forget, mode_of, source, view};

/// つながりの block の DOM と事件の受け取り（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use std::cell::Cell;
    use std::collections::{BTreeMap, BTreeSet};

    use leptos::ev;
    use leptos::html::Div;
    use leptos::prelude::*;
    use tsuzuri_contract::graph::{AroundDoc, ViewEdge};
    use web_sys::wasm_bindgen::JsCast;

    use super::{
        BLOCK, K_CHOICES, PageState, SideButton, buttons, embed_path, embed_reason, fold_of, k_of,
        request, state, toggle, unmeasured_reason, with_fold, with_k,
    };
    use crate::frame::{self, Mode};
    use crate::mapview::around::{
        ChainItem, ChainSide, as_view, chain, count_line, expert_line, layout, svg,
    };
    use crate::mapview::band::Band;
    use crate::mapview::graph::{
        Highlight, LEGEND_BORDERS, LEGEND_HOVER, LEGEND_SHAPES, Legend, PinAction, Side, border,
        degrees, edge_term, highlight, legend, legend_line, open_question, pin_next,
    };
    use crate::mapview::table::edge_name;
    use crate::mapview::{current, navigate};
    use crate::project::{ALERT_STYLE, section, unmeasured};
    use crate::view::Fetched;
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, hs, shows_internal};
    use crate::widgets::hover::{Card, attach, delegate, leaves};
    use crate::widgets::nodecard::view_cards;

    /// 頁に 1 つの読み（URL の query の signal と口の読みの結果・id が無ければ読まない）。
    #[derive(Debug, Clone, Copy)]
    pub struct Source {
        pub search: RwSignal<String>,
        pub read: Option<ReadSignal<(Fetched, Option<u16>)>>,
    }

    thread_local! {
        /// 節点の頁の block が分ける読み（頁の枠を組んでから、頁を切り替えて次に組み直すまで・行 g-nav）。
        static SOURCE: Cell<Option<Source>> = const { Cell::new(None) };
    }

    /// 頁を切り替える前に読みを捨てる（前の枠の signal は片付くので、次に組む block が読みを作り直す）。
    pub fn forget() {
        SOURCE.set(None);
    }

    /// 頁に 1 つの読み（初めて呼んだ block が作り、次の block は同じ読みを使う）。
    pub fn source() -> Source {
        if let Some(s) = SOURCE.get() {
            return s;
        }
        let search = RwSignal::new(current());
        let read = search.with_untracked(|s| request(s)).map(|_| {
            let path = Signal::derive(move || search.with(|s| request(s).unwrap_or_default()));
            crate::net::read_path(path)
        });
        let s = Source { search, read };
        SOURCE.set(Some(s));
        s
    }

    /// 今の mode を返す関数（context が無ければ URL から・頁の link に mode を残す）。
    pub fn mode_of(search: RwSignal<String>) -> impl Fn() -> Mode + Copy + Send + Sync + 'static {
        let ctx = use_context::<HelpCtx>();
        move || match ctx {
            Some(c) => c.mode.get(),
            None => search.with(|s| Mode::from_query(s)),
        }
    }

    /// 1 つの近傍の値（事件の受け取りが引く）。
    struct Model {
        center: String,
        edges: Vec<ViewEdge>,
        degree: BTreeMap<String, u32>,
        open_q: BTreeSet<String>,
        ids: BTreeSet<String>,
        /// 近傍の節点の id ごとの card（図の委ねが引く）。
        cards: BTreeMap<String, Card>,
    }

    impl Model {
        fn highlight(&self, center: &str) -> Highlight {
            highlight(&self.edges, &self.degree, center)
        }
    }

    /// つながりの block（見つからないときは何も出さない・見出しと id は節点の block が出す）。
    pub fn view() -> AnyView {
        let src = source();
        let Some(read) = src.read else {
            return ().into_any();
        };
        let pin = RwSignal::new(None::<String>);
        let search = src.search;
        let mode = mode_of(search);
        let content = move || {
            let st = read.with(|(f, s)| state(f, *s));
            match (unmeasured_reason(&st), st) {
                (Some(reason), _) => section(BLOCK, ().into_any(), unmeasured(reason)),
                (None, PageState::Doc(doc)) => {
                    section(BLOCK, ().into_any(), panel(doc, search, pin, mode))
                }
                (None, _) => ().into_any(),
            }
        };
        view! { {content} }.into_any()
    }

    /// 埋め込みの図の 1 つの口の読み（読みの結果と応答の状態の数）。
    type Read = ReadSignal<(Fetched, Option<u16>)>;

    /// 埋め込みの図の置き場の中身（読みを作る owner と、中心の id ごとの印と読みの列）。
    struct Places {
        owner: Option<Owner>,
        rows: Vec<(String, ArcRwSignal<bool>, Read)>,
    }

    /// 問いの頁に 1 つの埋め込みの図の置き場（段数と畳みは URL の query の k と fold・全部の図が分ける）。
    #[derive(Clone, Copy)]
    pub struct Embeds {
        search: RwSignal<String>,
        places: StoredValue<Places>,
    }

    /// 埋め込みの図の置き場を作る（呼んだ block の owner の下で読みを作る・card の組み直しで捨てられない）。
    pub fn embeds() -> Embeds {
        // 読みは呼んだ block の owner に持たせ、card を組み直す closure の owner に持たせない。
        let owner = Owner::current();
        Embeds {
            search: RwSignal::new(current()),
            places: StoredValue::new(Places {
                owner,
                rows: Vec::new(),
            }),
        }
    }

    impl Embeds {
        /// 中心の id の段が開いた（初めて開いたときから口を読む・2 度目からは何もしない）。
        pub fn open(self, center: &str) {
            let (mark, _) = self.place(center);
            if !mark.get_untracked() {
                mark.set(true);
            }
        }

        /// 中心の id の近傍の図（section と見出しで包まない・測れていないと見つからないは理由の 1 行）。
        pub fn view(self, center: String) -> AnyView {
            let (_, read) = self.place(&center);
            let search = self.search;
            let pin = RwSignal::new(None::<String>);
            let mode = mode_of(search);
            let content = move || {
                let st = read.with(|(f, s)| state(f, *s));
                match (embed_reason(&st), st) {
                    (Some(reason), _) => unmeasured(reason),
                    (None, PageState::Doc(doc)) => panel(doc, search, pin, mode),
                    (None, _) => ().into_any(),
                }
            };
            view! { {content} }.into_any()
        }

        /// 中心の id の印と読み（無ければ作る・読みは持った owner の下で頁の一生の間 1 つ）。
        fn place(self, center: &str) -> (ArcRwSignal<bool>, Read) {
            let found = self.places.with_value(|p| {
                p.rows
                    .iter()
                    .find(|(id, _, _)| id == center)
                    .map(|(_, mark, read)| (mark.clone(), *read))
            });
            if let Some(found) = found {
                return found;
            }
            let mark = ArcRwSignal::new(false);
            let search = self.search;
            let make = {
                let (id, mark) = (center.to_string(), mark.clone());
                move || {
                    let path = Signal::derive(move || search.with(|q| embed_path(&id, q, mark.get())));
                    crate::net::read_path(path)
                }
            };
            let owner = self.places.with_value(|p| p.owner.clone());
            let read = match owner {
                Some(o) => o.with(make),
                None => make(),
            };
            self.places
                .update_value(|p| p.rows.push((center.to_string(), mark.clone(), read)));
            (mark, read)
        }
    }

    /// event の的の節点の id（節点の箱の中でなければ None）。
    fn node_key(target: Option<web_sys::EventTarget>) -> Option<String> {
        let el: web_sys::Element = target?.dyn_into().ok()?;
        el.closest("g.node")
            .ok()
            .flatten()?
            .get_attribute("data-key")
    }

    /// 光らせる（`center` が None なら消す）・固定した節点と中心の節点は太い縁。
    fn paint(
        gz: NodeRef<Div>,
        model: StoredValue<Model>,
        center: Option<&str>,
        pinned: Option<&str>,
    ) {
        let Some(el) = gz.get_untracked() else {
            return;
        };
        let Some(picture) = el.first_element_child() else {
            return;
        };
        let lit = center.map(|c| model.with_value(|m| m.highlight(c)));
        let _ = picture
            .class_list()
            .toggle_with_force("hl-on", lit.is_some());
        let nodes = picture.get_elements_by_class_name("node");
        for i in 0..nodes.length() {
            let Some(n) = nodes.item(i) else {
                continue;
            };
            let key = n.get_attribute("data-key").unwrap_or_default();
            let side = lit.as_ref().and_then(|h| h.side.get(&key).copied());
            let list = n.class_list();
            let _ = list.toggle_with_force("lit", side.is_some());
            for s in Side::ALL {
                let _ = list.toggle_with_force(s.class(), side == Some(s));
            }
            if let Ok(Some(hit)) = n.query_selector("rect.hit") {
                let (open_q, thick) = model.with_value(|m| {
                    (
                        m.open_q.contains(&key),
                        pinned == Some(key.as_str()) || m.center == key,
                    )
                });
                let (stroke, width) = border(open_q, thick);
                let _ = hit.set_attribute("stroke", stroke);
                let _ = hit.set_attribute("stroke-width", &width.to_string());
            }
        }
        let edges = picture.get_elements_by_class_name("e");
        for i in 0..edges.length() {
            let Some(e) = edges.item(i) else {
                continue;
            };
            let index = e
                .get_attribute("data-i")
                .and_then(|s| s.parse::<usize>().ok());
            let on = lit
                .as_ref()
                .zip(index)
                .is_some_and(|(h, i)| h.lit.contains(&i));
            let _ = e.class_list().toggle_with_force("lit", on);
        }
    }

    fn panel(
        doc: AroundDoc,
        search: RwSignal<String>,
        pin: RwSignal<Option<String>>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let lay = layout(&doc);
        let picture = svg(&doc, &lay);
        let gv = as_view(&doc);
        let cards = view_cards(&gv.nodes);
        let hc = delegate();
        let lg = legend(&gv);
        let sides = chain(&doc);
        let count = count_line(&doc);
        let line = expert_line(&doc);
        let [up, down] = buttons(&doc);
        let model = StoredValue::new(Model {
            center: doc.center.clone(),
            degree: degrees(&gv),
            open_q: gv
                .nodes
                .iter()
                .filter(|n| open_question(n))
                .map(|n| n.node.id.clone())
                .collect(),
            ids: gv.nodes.iter().map(|n| n.node.id.clone()).collect(),
            cards: cards.clone(),
            edges: gv.edges,
        });
        // 読み直しで固定した節点が図から消えたら固定を外す。
        if pin.with_untracked(|p| {
            p.as_ref()
                .is_some_and(|id| !model.with_value(|m| m.ids.contains(id)))
        }) {
            pin.set(None);
        }
        let gz = NodeRef::<Div>::new();
        Effect::new(move |_| {
            let p = pin.get();
            if gz.get().is_some() {
                paint(gz, model, p.as_deref(), p.as_deref());
            }
        });
        let escape = window_event_listener(ev::keydown, move |ev| {
            if ev.key() == "Escape" && pin.with_untracked(Option::is_some) {
                pin.set(pin_next(None, &PinAction::Escape));
            }
        });
        on_cleanup(move || escape.remove());

        let hover = move |target: Option<web_sys::EventTarget>| {
            if pin.with_untracked(Option::is_some) {
                return;
            }
            if let Some(k) = node_key(target) {
                paint(gz, model, Some(&k), None);
            }
        };
        // 指の下の節点の card を出す（固定の有無によらない・光らせは hover が固定を見る）。
        let over = move |ev: ev::MouseEvent| {
            let card = node_key(ev.target())
                .and_then(|k| model.with_value(|m| m.cards.get(&k).cloned()));
            if let Some(card) = card {
                hc.show(&ev, card);
            }
            hover(ev.target());
        };
        let out = move |ev: ev::MouseEvent| {
            if leaves(
                node_key(ev.target()).as_deref(),
                node_key(ev.related_target()).as_deref(),
            ) {
                hc.leave(&ev);
            }
            if pin.with_untracked(Option::is_some) || node_key(ev.target()).is_none() {
                return;
            }
            if node_key(ev.related_target()).is_none() {
                paint(gz, model, None, None);
            }
        };
        let click = move |ev: ev::MouseEvent| {
            if let Some(k) = node_key(ev.target()) {
                pin.set(pin.with_untracked(|p| pin_next(p.as_deref(), &PinAction::Press(k))));
            }
        };
        // 2 回押すとその節点の頁へ（見本の dblclick）。
        let open = move |ev: ev::MouseEvent| {
            if let Some(k) = node_key(ev.target()) {
                let _ = window().location().set_href(&frame::node_href(&k, mode()));
            }
        };
        let unpin =
            move |_| pin.set(pin.with_untracked(|p| pin_next(p.as_deref(), &PinAction::Unpin)));
        let bar = move || {
            pin.get().map(|id| {
                let b = model.with_value(|m| m.highlight(&id).bar());
                let counts = format!(
                    " · {} {} · {} {} · {}",
                    label("nb_up"),
                    b.basis,
                    label("nb_down"),
                    b.impact,
                    b.depth
                );
                let href = frame::node_href(&b.id, mode());
                view! {
                    <span class="pinned">{label("pinned")}" "<b class="mono">{b.id}</b>{counts}</span>
                    <a class="btn sm" href=href>{label("open_node")}" ›"</a>
                    <button type="button" class="btn sm" on:click=unpin>{label("unpin")}</button>
                }
            })
        };
        let side_button = move |b: SideButton| {
            let pick =
                move |_| navigate(search, |s| with_fold(s, toggle(fold_of(s), b.side)), false);
            view! {
                <button type="button" class="btn sm nbside" aria-expanded=(!b.folded).to_string()
                    data-term=b.key on:click=pick>
                    {format!("{} {} ", b.glyph, label(b.key))}<span class="num">{b.count}</span>
                </button>
            }
        };
        let ks = K_CHOICES
            .into_iter()
            .map(|n| {
                let pressed = move || search.with(|s| k_of(s) == n).to_string();
                let pick = move |_| navigate(search, |s| with_k(s, n), false);
                view! { <button type="button" aria-pressed=pressed on:click=pick>{n}</button> }
            })
            .collect_view();
        let expert_row = move || {
            shows_internal(mode())
                .then(|| view! { <div class="small muted mono">{line.clone()}</div> })
        };
        view! {
            <div class="nb-wrap" data-origin=doc.center.clone()>
                <div class="nbctl">
                    <span class="sides">
                        {side_button(up)}
                        <span class="arr" aria-hidden="true">"←"</span>
                        <b class="self">{label("nb_self")}</b>
                        <span class="arr" aria-hidden="true">"→"</span>
                        {side_button(down)}
                    </span>
                    <span class="kseg">
                        {hs("nb_k")}
                        <span class="seg" role="group" aria-label=label("nb_k")>{ks}</span>
                    </span>
                </div>
                {legend_view(lg)}
                <div class="nb-graph" node_ref=gz inner_html=picture
                    on:mouseover=over on:mouseout=out
                    on:focusin=move |ev| hover(ev.target()) on:click=click on:dblclick=open></div>
                <div class="pinbar" aria-live="polite">{bar}</div>
                {chain_view(sides, &cards, mode)}
                <div class="cutline num" tabindex="0" data-term="cut">{count}</div>
                {expert_row}
            </div>
        }
        .into_any()
    }

    /// 凡例（形と色と縁と hover の見方・図に出ている帯・図に出ている辺の型・グラフの面と同じ部品）。
    fn legend_view(lg: Legend) -> AnyView {
        let swatches = Band::ALL
            .into_iter()
            .map(|b| view! { <i style=format!("background:var(--{})", b.class_name())></i> })
            .collect_view();
        let bands = lg
            .bands
            .into_iter()
            .map(|b| {
                view! {
                    <span data-term=b.key() tabindex="0">
                        <span class=format!("shape {} fill", b.class_name()) aria-hidden="true"></span>
                        <b>{label(b.key())}</b><code class="path">{b.path()}</code>
                    </span>
                }
            })
            .collect_view();
        let types = (!lg.types.is_empty()).then(|| {
            let items = lg
                .types
                .into_iter()
                .map(|t| {
                    view! {
                        <span data-term=edge_term(t) tabindex="0">
                            <span inner_html=legend_line(t)></span><code>{edge_name(t)}</code>
                        </span>
                    }
                })
                .collect_view();
            view! { <div class="legend">{items}</div> }
        });
        view! {
            <div class="legend lkey">
                <span data-term="lg_shape" tabindex="0"><span inner_html=LEGEND_SHAPES></span>{label("lg_shape")}</span>
                <span data-term="lg_color" tabindex="0"><span class="sw7">{swatches}</span>{label("lg_color")}</span>
                <span data-term="lg_border" tabindex="0"><span inner_html=LEGEND_BORDERS></span>{label("lg_border")}</span>
            </div>
            <div class="legend lkey">
                <span data-term="lg_hover" tabindex="0"><span inner_html=LEGEND_HOVER></span>{label("lg_hover")}</span>
            </div>
            <div class="legend">{bands}</div>
            {types}
        }
        .into_any()
    }

    /// 一覧の 1 行（印・id と題の節点の頁への link と hover の card・右に辺の型）。
    fn item_view(
        item: &ChainItem,
        card: Card,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let style = if item.alert { ALERT_STYLE } else { "" };
        let id = item.id.clone();
        let href = move || frame::node_href(&id, mode());
        let edge = item
            .edge
            .clone()
            .map(|e| view! { <span class="aside"><code>{e}</code></span> });
        view! {
            <li>
                <span class=item.shape.clone() style=style aria-hidden="true"></span>
                <a class="ttl" href=href use:attach=card><span class="nid">{item.id.clone()}</span>" "<span data-t="">{item.title.clone()}</span></a>
                {edge}
            </li>
        }
        .into_any()
    }

    /// 狭い幅の一覧（根拠の側と影響の側の 2 段・1 段目の行の下にその先の行を入れ子で並べる）。
    fn chain_view(
        sides: Vec<ChainSide>,
        cards: &BTreeMap<String, Card>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let row = |item: &ChainItem| {
            let card = cards.get(&item.id).cloned().unwrap_or_default();
            item_view(item, card, mode)
        };
        let parts = sides
            .into_iter()
            .map(|side| {
                let rows = side
                    .items
                    .iter()
                    .map(|item| {
                        let nest = (!item.nest.is_empty()).then(|| {
                            let inner = item.nest.iter().map(row).collect_view();
                            view! { <li class="nest"><ul class="items">{inner}</ul></li> }
                        });
                        view! { {row(item)}{nest} }
                    })
                    .collect_view();
                view! { <div class="subh">{label(side.key)}</div><ul class="items">{rows}</ul> }
            })
            .collect_view();
        view! { <div class="nb-chain">{parts}</div> }.into_any()
    }
}
