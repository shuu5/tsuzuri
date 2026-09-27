use std::collections::BTreeMap;
use std::time::Duration;

use leptos::ev;
use leptos::html::Div;
use leptos::prelude::*;
use tsuzuri_contract::graph::GraphView;
use web_sys::wasm_bindgen::JsCast;

use super::{
    ChainBand, Highlight, LEGEND_BORDERS, LEGEND_HOVER, LEGEND_SHAPES, Layout, Legend, PATH,
    PinAction, Side, Zoom, band_labels, border, box_height, chain, count_line, degrees, doc,
    dragged, edge_term, expert_line, highlight, initial_scale, layout, legend, legend_line,
    open_question, opens_node, pin_next, svg,
};
use crate::frame::{self, Mode};
use crate::mapview::band::Band;
use crate::mapview::band_chip;
use crate::mapview::table::edge_name;
use crate::project::{ALERT_STYLE, unmeasured};
use crate::vocab::label;
use crate::widgets::help::{HelpCtx, h2, shows_internal};

/// 1 つの眺めの値（事件の受け取りが引く）。
struct Model {
    view: GraphView,
    layout: Layout,
    degree: BTreeMap<String, u32>,
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
pub fn view(search: RwSignal<String>) -> AnyView {
    let fetched = crate::net::read(PATH);
    let ctx = use_context::<HelpCtx>();
    // 今の mode（context が無ければ URL から・節点の頁の link に mode を残す）。
    let mode = move || match ctx {
        Some(c) => c.mode.get(),
        None => search.with(|s| Mode::from_query(s)),
    };
    let pin = RwSignal::new(None::<String>);
    let zoom = StoredValue::new(None::<Zoom>);
    let content = move || {
        fetched.with(|f| match doc(f) {
            Err(reason) => unmeasured(reason),
            Ok(v) => panel(v, pin, zoom, mode),
        })
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
) -> AnyView {
    let lay = layout(&v);
    let picture = svg(&v, &lay);
    let lg = legend(&v);
    let bands = chain(&v);
    let count = count_line(&v);
    let line = expert_line(&v);
    let degree = degrees(&v);
    let model = StoredValue::new(Model {
        view: v,
        layout: lay,
        degree,
    });
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

    let hover = move |target: Option<web_sys::EventTarget>| {
        if pin.with_untracked(Option::is_some) {
            return;
        }
        if let Some(k) = node_key(target) {
            paint(gz, model, Some(&k), None);
        }
    };
    let out = move |ev: ev::MouseEvent| {
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
        if drag_done.get_value() {
            return;
        }
        pin.set(pin.with_untracked(|p| pin_next(p.as_deref(), &PinAction::Press(k))));
    };
    // 2 回押すとその節点の頁へ（見本の dblclick）。
    let open = move |ev: ev::MouseEvent| {
        if let Some(k) = node_key(ev.target()) {
            let _ = window().location().set_href(&frame::node_href(&k, mode()));
        }
    };
    // focus の在る節点で Enter と Space を押すとその節点の頁へ（Space の scroll を止める）。
    let key = move |ev: ev::KeyboardEvent| {
        if !opens_node(&ev.key()) {
            return;
        }
        let Some(k) = node_key(ev.target()) else {
            return;
        };
        ev.prevent_default();
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
    let expert_row = move || {
        shows_internal(mode()).then(|| view! { <div class="small muted mono">{line.clone()}</div> })
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
                    on:mouseover=move |ev| hover(ev.target()) on:mouseout=out
                    on:focusin=move |ev| hover(ev.target()) on:click=click on:dblclick=open
                    on:keydown=key></div>
            </div>
            <div class="pinbar" aria-live="polite">{bar}</div>
            {chain_view(bands, mode)}
            <div class="cutline num" tabindex="0" data-term="cut">{count}</div>
            {expert_row}
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

/// 狭い幅の一覧（帯の chip の小見出しと行の一覧・行の id と題は節点の頁への link）。
fn chain_view(
    bands: Vec<ChainBand>,
    mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
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
                    let id = r.id.clone();
                    let href = move || frame::node_href(&id, mode());
                    view! {
                        <li>
                            <span class=r.shape style=style aria-hidden="true"></span>
                            <a class="ttl" href=href><span class="nid">{r.id}</span>" "<span data-t="">{r.title}</span></a>
                            {kids}
                        </li>
                    }
                })
                .collect_view();
            view! { <div class="subh">{band_chip(b.band)}</div><ul class="items">{rows}</ul> }
        })
        .collect_view();
    view! { <div class="nb-chain">{parts}</div> }.into_any()
}
