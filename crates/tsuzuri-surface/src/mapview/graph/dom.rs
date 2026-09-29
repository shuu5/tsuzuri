use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Duration;

use leptos::ev;
use leptos::html::Div;
use leptos::prelude::*;
use tsuzuri_contract::graph::{BoxFold, GraphView, ViewNode};
use web_sys::wasm_bindgen::JsCast;

use super::fold::{LEGEND_GROUP, OpenList, fold_key, refused_line};
use super::{
    ChainBand, Highlight, LEGEND_BORDERS, LEGEND_HOVER, LEGEND_SHAPES, Layout, Legend, PinAction,
    Side, Zoom, band_labels, border, box_height, chain, count_line, degrees, doc, dragged,
    edge_term, expert_line, highlight, initial_scale, layout, legend, legend_line, open_question,
    opens_node, pin_next, svg,
};
use crate::view::Fetched;
use crate::frame::{self, Mode};
use crate::mapview::band::Band;
use crate::mapview::band_chip;
use crate::mapview::table::edge_name;
use crate::project::{ALERT_STYLE, unmeasured};
use crate::vocab::label;
use crate::widgets::help::{HelpCtx, expert_tip, h2};
use crate::widgets::hover::{Card, attach, delegate, leaves};
use crate::widgets::nodecard::view_cards;

/// 1 つの眺めの値（事件の受け取りが引く）。
struct Model {
    view: GraphView,
    layout: Layout,
    degree: BTreeMap<String, u32>,
    /// 節点の id ごとの card（図の委ねが引く）。
    cards: BTreeMap<String, Card>,
}

impl Model {
    fn highlight(&self, center: &str) -> Highlight {
        highlight(&self.view.edges, &self.degree, center)
    }

    fn open_q(&self, id: &str) -> bool {
        self.view
            .nodes
            .iter()
            .any(|n| n.node.id == id && open_question(n))
    }

    fn has(&self, id: &str) -> bool {
        self.view.nodes.iter().any(|n| n.node.id == id)
    }

    fn node(&self, id: &str) -> Option<&ViewNode> {
        self.view.nodes.iter().find(|n| n.node.id == id)
    }
}

thread_local! {
    /// 開いた箱の id の列（頁の間だけ・URL にも画面の外の保存にも書かない・地図の中身の読み直しで消えない）。
    static OPEN: RefCell<OpenList> = RefCell::new(OpenList::default());
    /// 最後に読めた眺め（開き閉じの後の読み直しの間に出し続ける）。
    static LAST: RefCell<Option<GraphView>> = const { RefCell::new(None) };
}

/// 押した点（drag の始まり）。
#[derive(Debug, Clone, Copy)]
struct Press {
    x: f64,
    y: f64,
    base: Zoom,
    moved: bool,
}

/// グラフの面（口の読みの 3 値・読めた眺めの図）。固定と拡大は読み直しの後も保つ。
/// 口の path は開いた箱の列から組み、箱を押すと path が変わって読み直す（その間は最後に読めた眺めを出す）。
pub fn view(search: RwSignal<String>) -> AnyView {
    let path = RwSignal::new(OPEN.with_borrow(OpenList::path));
    let fetched = crate::net::read_path(Signal::derive(move || path.get()));
    let ctx = use_context::<HelpCtx>();
    // 今の mode（context が無ければ URL から・節点の頁の link に mode を残す）。
    let mode = move || match ctx {
        Some(c) => c.mode.get(),
        None => search.with(|s| Mode::from_query(s)),
    };
    let pin = RwSignal::new(None::<String>);
    let zoom = StoredValue::new(None::<Zoom>);
    let content = move || {
        let got = fetched.with(|(f, _)| match doc(f) {
            Ok(v) => {
                OPEN.with_borrow_mut(|o| o.adopt(&v.open));
                LAST.set(Some(v.clone()));
                Ok(v)
            }
            Err(reason) if matches!(f, Fetched::NotRead) => {
                LAST.with_borrow(Clone::clone).ok_or(reason)
            }
            Err(reason) => Err(reason),
        });
        match got {
            Err(reason) => unmeasured(reason),
            Ok(v) => panel(v, pin, zoom, mode, path),
        }
    };
    view! { {content} }.into_any()
}

/// event の的の節点の id（節点の箱の中でなければ None）。
fn node_key(target: Option<web_sys::EventTarget>) -> Option<String> {
    let el: web_sys::Element = target?.dyn_into().ok()?;
    el.closest("g.node")
        .ok()
        .flatten()?
        .get_attribute("data-key")
}

/// event の的の開き閉じの印の箱の id（印の中でなければ None）。
fn fold_target(target: Option<web_sys::EventTarget>) -> Option<String> {
    let el: web_sys::Element = target?.dyn_into().ok()?;
    el.closest("g.fold")
        .ok()
        .flatten()?
        .get_attribute("data-fold")
}

/// 図に拡大と移動を置き、帯の名を描き直す。
fn place(gz: NodeRef<Div>, model: StoredValue<Model>, z: Zoom) {
    let Some(el) = gz.get_untracked() else {
        return;
    };
    if let Ok(Some(vp)) = el.query_selector("#vp") {
        let _ = vp.set_attribute("transform", &z.transform());
    }
    let box_h = f64::from(el.client_height());
    if let Ok(Some(labels)) = el.query_selector("#blabs") {
        labels.set_inner_html(&model.with_value(|m| band_labels(&m.layout, z, box_h)));
    }
}

/// 光らせる（`center` が None なら消す）・固定した節点は太い縁。
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
            let open_q = model.with_value(|m| m.open_q(&key));
            let (stroke, width) = border(open_q, pinned == Some(key.as_str()));
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
    v: GraphView,
    pin: RwSignal<Option<String>>,
    zoom: StoredValue<Option<Zoom>>,
    mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
    path: RwSignal<String>,
) -> AnyView {
    let lay = layout(&v);
    let picture = svg(&v, &lay);
    let lg = legend(&v);
    let bands = chain(&v);
    let count = count_line(&v);
    let refused = refused_line(&v)
        .map(|r| view! { <div class="small muted" role="status">{r}</div> });
    let line = expert_line(&v);
    let degree = degrees(&v);
    let cards = view_cards(&v.nodes);
    let model = StoredValue::new(Model {
        view: v,
        layout: lay,
        degree,
        cards: cards.clone(),
    });
    let hc = delegate();
    // 読み直しで固定した節点が図から消えたら固定を外す。
    if pin.with_untracked(|p| {
        p.as_ref()
            .is_some_and(|id| !model.with_value(|m| m.has(id)))
    }) {
        pin.set(None);
    }
    let gz = NodeRef::<Div>::new();
    let s0 = StoredValue::new(1.0_f64);
    let press = StoredValue::new(None::<Press>);
    let drag_done = StoredValue::new(false);
    let current = move || {
        zoom.get_value()
            .unwrap_or_else(|| Zoom::initial(s0.get_value()))
    };
    let apply = move |z: Zoom| {
        zoom.set_value(Some(z));
        place(gz, model, z);
    };

    Effect::new(move |_| {
        let Some(el) = gz.get() else {
            return;
        };
        let (width, height) = model.with_value(|m| (m.layout.width, m.layout.height));
        let s = initial_scale(f64::from(el.client_width()), width);
        s0.set_value(s);
        let _ = el.set_attribute("style", &format!("height:{}px", box_height(height, s)));
        apply(current());
        let p = pin.get_untracked();
        paint(gz, model, p.as_deref(), p.as_deref());
    });
    Effect::new(move |_| {
        let p = pin.get();
        paint(gz, model, p.as_deref(), p.as_deref());
    });
    let escape = window_event_listener(ev::keydown, move |ev| {
        if ev.key() == "Escape" && pin.with_untracked(Option::is_some) {
            pin.set(pin_next(None, &PinAction::Escape));
        }
    });
    on_cleanup(move || escape.remove());

    // 箱の開き閉じ（頁の列を変え、変われば口の path を置き直して読み直す・固定は動かさない）。
    let toggle = move |id: String, fold: BoxFold| {
        if let Some(p) = OPEN.with_borrow_mut(|o| o.toggle(&id, fold).then(|| o.path())) {
            path.set(p);
        }
    };
    let fold_of = move |id: &str| model.with_value(|m| m.node(id).map(|n| n.fold));
    let press_fold = move |target: Option<web_sys::EventTarget>| {
        let Some(id) = fold_target(target) else {
            return false;
        };
        if let Some(fold) = fold_of(&id) {
            toggle(id, fold);
        }
        true
    };
    let group = move |id: &str| model.with_value(|m| m.node(id).is_some_and(|n| n.group));

    let hover =move |target: Option<web_sys::EventTarget>| {
        if pin.with_untracked(Option::is_some) {
            return;
        }
        if let Some(k) = node_key(target) {
            paint(gz, model, Some(&k), None);
        }
    };
    // 図の節点の card（固定の間も出す・同じ節点の中の移りでは出し直さない）。
    let over = move |ev: ev::MouseEvent| {
        let card = node_key(ev.target()).and_then(|k| model.with_value(|m| m.cards.get(&k).cloned()));
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
        let Some(k) = node_key(ev.target()) else {
            return;
        };
        if drag_done.get_value() || press_fold(ev.target()) {
            return;
        }
        pin.set(pin.with_untracked(|p| pin_next(p.as_deref(), &PinAction::Press(k))));
    };
    // 2 回押すとその節点の頁へ（見本の dblclick・開き閉じの印と組の箱には頁が無い）。
    let open = move |ev: ev::MouseEvent| {
        if fold_target(ev.target()).is_some() {
            return;
        }
        if let Some(k) = node_key(ev.target()).filter(|k| !group(k)) {
            let _ = window().location().set_href(&frame::node_href(&k, mode()));
        }
    };
    // focus の在る節点で Enter と Space を押すとその節点の頁へ（Space の scroll を止める）。
    // 開き閉じの印の上では箱を開き閉じし、組の箱では頁へ移らない。
    let key = move |ev: ev::KeyboardEvent| {
        if !opens_node(&ev.key()) {
            return;
        }
        let Some(k) = node_key(ev.target()) else {
            return;
        };
        ev.prevent_default();
        if press_fold(ev.target()) || group(&k) {
            return;
        }
        let _ = window().location().set_href(&frame::node_href(&k, mode()));
    };
    let wheel = move |ev: ev::WheelEvent| {
        ev.prevent_default();
        let Some(el) = gz.get_untracked() else {
            return;
        };
        let r = el.get_bounding_client_rect();
        let (cx, cy) = (
            f64::from(ev.client_x()) - r.left(),
            f64::from(ev.client_y()) - r.top(),
        );
        apply(current().wheel(cx, cy, ev.delta_y()));
    };
    let down = move |ev: ev::PointerEvent| {
        if ev.button() != 0 || !ev.is_primary() {
            return;
        }
        press.set_value(Some(Press {
            x: f64::from(ev.client_x()),
            y: f64::from(ev.client_y()),
            base: current(),
            moved: false,
        }));
    };
    let moved = move |ev: ev::PointerEvent| {
        let Some(mut p) = press.get_value() else {
            return;
        };
        let (dx, dy) = (
            f64::from(ev.client_x()) - p.x,
            f64::from(ev.client_y()) - p.y,
        );
        if !p.moved && dragged(dx, dy) {
            p.moved = true;
            press.set_value(Some(p));
            drag_done.set_value(true);
            if let Some(el) = gz.get_untracked() {
                let _ = el.set_pointer_capture(ev.pointer_id());
            }
        }
        if p.moved {
            apply(p.base.drag(dx, dy));
        }
    };
    // drag の後の離しは押したと数えない（click の後に印を戻す）。
    let up = move |_: ev::PointerEvent| {
        press.set_value(None);
        if drag_done.get_value() {
            set_timeout(move || drag_done.set_value(false), Duration::ZERO);
        }
    };
    let reset = move |_| apply(Zoom::initial(s0.get_value()));
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
    view! {
        <div class="gpanel">
            <header>{h2("lines")}</header>
            {legend_view(lg)}
            <div class="gtools">
                <button type="button" class="btn sm" on:click=reset>{label("zoom_reset")}</button>
                <span class="small muted" data-term="zoom_hint" tabindex="0">{label("zoom_hint")}</span>
            </div>
            <div class="gwrap">
                <div class="gzoom" node_ref=gz inner_html=picture
                    on:wheel=wheel on:pointerdown=down on:pointermove=moved on:pointerup=up on:pointercancel=up
                    on:mouseover=over on:mouseout=out
                    on:focusin=move |ev| hover(ev.target()) on:click=click on:dblclick=open
                    on:keydown=key></div>
            </div>
            <div class="pinbar" aria-live="polite">{bar}</div>
            {chain_view(bands, &cards, mode, toggle)}
            <div class="cutline num" tabindex="0" data-term="cut" use:expert_tip=line>{count}</div>
            {refused}
        </div>
    }
    .into_any()
}

/// 凡例（形と色と縁と hover の見方・図に出ている帯・図に出ている辺の型）。
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
            <span data-term="gf_group" tabindex="0"><span inner_html=LEGEND_GROUP></span>{label("gf_group")}</span>
            <span data-term="lg_shape" tabindex="0"><span inner_html=LEGEND_SHAPES></span>{label("lg_shape")}</span>
            <span data-term="lg_color" tabindex="0"><span class="sw7">{swatches}</span>{label("lg_color")}</span>
            <span data-term="lg_border" tabindex="0"><span inner_html=LEGEND_BORDERS></span>{label("lg_border")}</span>
            <span data-term="lg_hover" tabindex="0"><span inner_html=LEGEND_HOVER></span>{label("lg_hover")}</span>
        </div>
        <div class="legend">{bands}</div>
        {types}
    }
    .into_any()
}

/// 狭い幅の一覧（帯の chip の小見出しと行の一覧・行の id と題は節点の頁への link で節点の card を持つ）。
/// 組の箱の行は題だけ（頁が無いので link にしない）で、開き閉じできる行は図と同じ開き閉じの button を持つ。
fn chain_view(
    bands: Vec<ChainBand>,
    cards: &BTreeMap<String, Card>,
    mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
    toggle: impl Fn(String, BoxFold) + Copy + Send + Sync + 'static,
) -> AnyView {
    let parts = bands
        .into_iter()
        .map(|b| {
            let rows = b
                .rows
                .into_iter()
                .map(|r| {
                    let style = if r.alert { ALERT_STYLE } else { "" };
                    let kids = r.kids.map(|k| {
                        view! { <span class="aside">{format!("{} {k}", label("children"))}</span> }
                    });
                    let card = cards.get(&r.id).cloned().unwrap_or_default();
                    let id = r.id.clone();
                    let href = move || frame::node_href(&id, mode());
                    let (fid, fold) = (r.id.clone(), r.fold);
                    let button = fold_key(fold).map(|k| {
                        view! {
                            <button type="button" class="btn sm" on:click=move |_| toggle(fid.clone(), fold)>{label(k)}</button>
                        }
                    });
                    let head = if r.group {
                        view! { <b class="ttl" use:attach=card><span data-t="">{r.title}</span></b> }.into_any()
                    } else {
                        view! {
                            <a class="ttl" href=href use:attach=card><span class="nid">{r.id}</span>" "<span data-t="">{r.title}</span></a>
                        }
                        .into_any()
                    };
                    view! {
                        <li>
                            <span class=r.shape style=style aria-hidden="true"></span>
                            {head}
                            {kids}
                            {button}
                        </li>
                    }
                })
                .collect_view();
            view! { <div class="subh">{band_chip(b.band)}</div><ul class="items">{rows}</ul> }
        })
        .collect_view();
    view! { <div class="nb-chain">{parts}</div> }.into_any()
}
