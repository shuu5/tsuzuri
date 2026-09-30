//! 地図の頁（見本の map.html・便 g-map）: 見出しと節点の数・面の切り替えの tab（6 つ）・今の面の中身。
//! 口（/api/graph）の本文を契約の型の GraphDoc に読み、面の中身は mapview の下の module が組む。
//! グラフの面だけは自分の口（/api/graph/view）を graph の module の中で読む（便 g-graph）。
//! 今の面は URL の query の view に残す（無い・知らない値は圧縮の面）。

use tsuzuri_contract::graph::GraphDoc;
use tsuzuri_contract::wire;

use super::{Body, NO_CONTENT, NOT_READ};
use crate::frame::Block;
use crate::mapview::View;
use crate::view::Fetched;

pub const BLOCK: Block = Block {
    id: "map",
    heading: "map",
    class: "panel",
};

/// 読みの口（便 g-parts）。
pub const PATH: &str = "/api/graph";

/// この file が字を持つ口の path（ほかの module の口を読む所は数えない・行 hs-derived）。
pub const PATHS: &[&str] = &[PATH];

/// この file の畳める段の開き閉じの鍵の形（無い・行 hs-derived）。
pub const FOLDS: &[&str] = &[];

/// 口が読めないときの理由。
pub const REASON: &str =
    "地図の 7 つの出所を組む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 口の本文を電文に読む（まだ読んでいない・読めない・電文の型として読めないは理由）。
/// 電文が読めない本文の理由は、中身の無い block と同じ（便 g-parts の歯が 3 値の理由を pin する）。
pub fn doc(fetched: &Fetched) -> Result<GraphDoc, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(REASON),
        Fetched::Body(text) => wire::decode::<GraphDoc>(text).map_err(|_| NO_CONTENT),
    }
}

/// 中身の有無（測れていない・電文あり）。0 件と測れていないは帯ごとに面の中身が分ける。
pub fn body(fetched: &Fetched) -> Body<()> {
    match doc(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(_) => Body::Filled(()),
    }
}

/// tab の上の key で移る面（右の矢印は次・左の矢印は前・端は反対の端へ回る・ほかの key は None・見本の map.html）。
pub fn tab_step(now: View, key: &str) -> Option<View> {
    let n = View::ALL.len();
    let i = View::ALL.iter().position(|v| *v == now)?;
    let j = match key {
        "ArrowRight" => (i + 1) % n,
        "ArrowLeft" => (i + n - 1) % n,
        _ => return None,
    };
    View::ALL.get(j).copied()
}

/// 見出しは頁の題（h1）にする（見本の map.html と同じ）。
#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 地図の頁の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;
    use web_sys::wasm_bindgen::JsCast;

    use super::{BLOCK, PATH, doc, tab_step};
    use crate::mapview::tree::{self, Tree};
    use crate::mapview::{View, compact, current, graph, list, navigate, table, with_view};
    use crate::project::unmeasured;
    use crate::vocab::label;
    use crate::widgets::help::h1;

    /// 札の上の key（左右の矢印で次か前の札へ移り、その札に focus を移す）。
    fn tab_key(search: RwSignal<String>, e: &ev::KeyboardEvent) {
        let now = search.with_untracked(|s| View::from_query(s));
        let Some(next) = tab_step(now, &e.key()) else {
            return;
        };
        e.prevent_default();
        navigate(search, |s| with_view(s, next), true);
        let el = document()
            .get_element_by_id(&format!("tab-{}", next.name()))
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(el) = el {
            let _ = el.focus();
        }
    }

    pub fn view() -> AnyView {
        let fetched = crate::net::read(PATH);
        let parsed = Memo::new(move |_| fetched.with(doc));
        let search = RwSignal::new(current());
        let back = window_event_listener(ev::popstate, move |_| search.set(current()));
        on_cleanup(move || back.remove());
        let count = move || {
            parsed.with(|d| {
                d.as_ref()
                    .ok()
                    .map(|d| view! { <span class="chip num">{d.nodes.len()}</span> })
            })
        };
        let step = move |e: ev::KeyboardEvent| tab_key(search, &e);
        let tabs = View::ALL
            .into_iter()
            .map(|v| {
                let on = move || search.with(|s| View::from_query(s) == v);
                let pick = move |_| navigate(search, |s| with_view(s, v), true);
                view! {
                    <button type="button" role="tab" id=format!("tab-{}", v.name()) aria-controls="view" data-view=v.name()
                        aria-selected=move || on().to_string() tabindex=move || if on() { "0" } else { "-1" } on:click=pick on:keydown=step>
                        <span class="hd" data-v=v.key()><span class="hd-t" data-term=v.key()>{label(v.key())}</span></span>
                    </button>
                }
            })
            .collect_view();
        let content = move || {
            let v = search.with(|s| View::from_query(s));
            parsed.with(|d| match d {
                Err(reason) => unmeasured(reason),
                Ok(d) => match v {
                    View::Compact => compact::view(d),
                    View::List => list::view(d, search),
                    View::Graph => graph::view(search),
                    View::Table => table::view(d, search),
                    View::Design => tree::view(d, Tree::Design, search),
                    View::Ledger => tree::view(d, Tree::Ledger, search),
                },
            })
        };
        let labelled = move || format!("tab-{}", search.with(|s| View::from_query(s)).name());
        view! {
            <section class=BLOCK.class id=BLOCK.id>
                <header class="row">{h1(BLOCK.heading)}{count}</header>
                <div class="tabs" role="tablist" aria-label=label("views") data-term="views">{tabs}</div>
                <div id="view" role="tabpanel" aria-labelledby=labelled>{content}</div>
            </section>
        }
        .into_any()
    }
}
