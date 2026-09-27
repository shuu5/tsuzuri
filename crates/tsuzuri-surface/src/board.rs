//! 頁の描画（wasm の target のときだけ組み立てる・Leptos の csr）: header と頁の枠を frame の値のとおりに並べる。
//! block の中身は project の下の module が描く。ここは枠を描き、mode と頁を URL から読んで URL に残すだけ。
//! 「?」の注釈の層と hover の card の層は頁に 1 つずつ置く（便 g-parts）。
//! 質問の link の数の印は問いの一覧の口（ask の module の口）の件数を出す（便 g-ask）。
//! 節点の頁（便 g-node）は query の page=node で開き、block は node と around の 2 つ（nav には出さない）。
//! 問いの頁の右の列（便 g-batch）は batch と policy の 2 つの block。

use leptos::prelude::*;
use tsuzuri_contract::board::Reading;

use crate::frame::{self, BRAND, Block, HEADER, Mode, PageId};
use crate::net;
use crate::project::{
    self, ask, askpage, batch, gaps, ledger, legend, map, next, node, nodearound, pipeline, policy,
    seat,
};
use crate::view::{Screen, clock};
use crate::vocab::label;
use crate::widgets::help::{HelpCtx, TipLayer, hs};
use crate::widgets::hover::{CardLayer, HoverCtx};

/// 題の印（見本の IC.logo）。
const LOGO: &str = r##"<svg class="logo" viewBox="0 0 24 24" aria-hidden="true"><rect x="2" y="2" width="20" height="20" rx="6" fill="var(--accent)"/><path d="M7 8h10M7 12h10M7 16h6" stroke="#fff" stroke-width="2" stroke-linecap="round"/></svg>"##;

/// nav の印（見本の IC.home・IC.ask・IC.map・IC.gaps）。
fn nav_icon(page: PageId) -> &'static str {
    match page {
        PageId::Home => {
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M3 11l9-7 9 7v9a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z"/></svg>"#
        }
        PageId::Ask => {
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M4 5h16v11H9l-5 4z"/><path d="M10 9.5a2 2 0 1 1 2.8 1.8c-.5.3-.8.7-.8 1.2"/><circle cx="12" cy="14.5" r=".6" fill="currentColor"/></svg>"#
        }
        PageId::Map => {
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="3" y="4" width="18" height="4" rx="1"/><rect x="3" y="10" width="18" height="4" rx="1"/><rect x="3" y="16" width="18" height="4" rx="1"/></svg>"#
        }
        PageId::Gaps => {
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="11" cy="11" r="6"/><path d="M20 20l-4.5-4.5"/><path d="M8.5 11h5"/></svg>"#
        }
        // 節点の頁は nav に出さない。
        PageId::Node => "",
    }
}

/// body に画面を載せる。
pub fn mount() {
    leptos::mount::mount_to_body(App);
}

/// 今の URL の query（読めなければ空）。
fn search() -> String {
    window().location().search().unwrap_or_default()
}

/// mode を URL の query に残す（頁は読み直さない）。
fn keep_mode_in_url(mode: Mode) {
    let url = frame::with_param(&search(), "mode", mode.key());
    if let Ok(history) = window().history() {
        let _ =
            history.replace_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, "", Some(&url));
    }
}

#[component]
fn App() -> impl IntoView {
    let query = search();
    let page = PageId::from_query(&query);
    let mode = RwSignal::new(Mode::from_query(&query));
    let help = HelpCtx {
        open: RwSignal::new(None),
        mode,
    };
    provide_context(help);
    // body の class で「?」の出し分けを決める（見本の ui.css の body.mode-beginner）。
    Effect::new(move |_| {
        if let Some(body) = document().body() {
            body.set_class_name(mode.get().body_class());
        }
    });
    provide_context(HoverCtx::default());
    // 台帳の一覧の口を読み、読みの結果が変わるたびに画面の状態を進める（最終更新は読めた時刻）。
    let fetched = net::read(ledger::PATH);
    let screen = RwSignal::new(Screen::initial());
    Effect::new(move |_| {
        let f = fetched.get();
        screen.update(|s| *s = s.after_read(&f, net::now()));
    });
    view! {
        {top(page, mode, screen)}
        <main class="page">{page_view(page, screen)}</main>
        <TipLayer/>
        <CardLayer/>
    }
}

/// 上端の帯: 題・頁の link・最終更新・mode の切り替え（frame の HEADER の順）。
fn top(page: PageId, mode: RwSignal<Mode>, screen: RwSignal<Screen>) -> impl IntoView {
    let parts = HEADER
        .iter()
        .map(|part| match part.part {
            "brand" => view! {
                <a class=part.class href=move || frame::href(PageId::Home, mode.get()) aria-label=label(part.key)>
                    <span inner_html=LOGO></span>
                    <span class="name">{BRAND}</span>
                </a>
            }
            .into_any(),
            "nav" => {
                let questions = net::read(ask::PATH);
                let open = move || match questions.with(ask::count) {
                    Reading::Known(n) => Some(n),
                    Reading::Unknown => None,
                };
                let links = frame::nav_links(page)
                    .into_iter()
                    .map(|l| {
                        let badge = move || {
                            let n = if l.badge.is_empty() { None } else { open() };
                            frame::badge(n)
                                .map(|n| view! { <span class=l.badge>{n}</span> })
                        };
                        view! {
                            <a href=move || frame::href(l.page, mode.get()) class=l.class data-v=l.key data-term=l.key>
                                <span inner_html=nav_icon(l.page)></span>
                                <span class="lbl hd-t">{label(l.key)}</span>
                                {badge}
                            </a>
                        }
                    })
                    .collect_view();
                view! {
                    <nav class=part.class aria-label=label(part.key)>{links}</nav>
                    <span class="grow"></span>
                }
                .into_any()
            }
            "updated" => {
                let at = move || match screen.with(|s| s.updated_at) {
                    Some(t) => clock(t).into_any(),
                    None => view! { {project::state_icon(project::UNKNOWN)}{label("not_yet")} }.into_any(),
                };
                view! { {hs(part.key)}<span class=part.class>{at}</span> }.into_any()
            }
            _ => {
                let choices = Mode::ALL
                    .into_iter()
                    .map(|m| {
                        let pressed = move || (mode.get() == m).to_string();
                        let pick = move |_| {
                            mode.set(m);
                            keep_mode_in_url(m);
                        };
                        view! { <button type="button" data-mode=m.key() aria-pressed=pressed on:click=pick>{label(m.key())}</button> }
                    })
                    .collect_view();
                view! { <div class=part.class role="group" aria-label=label(part.key) data-term=part.key>{choices}</div> }
                    .into_any()
            }
        })
        .collect_view();
    view! { <header class="top">{parts}</header> }
}

/// 頁の枠（frame の列と block の並びのとおり）。
fn page_view(page: PageId, screen: RwSignal<Screen>) -> impl IntoView {
    let frame = frame::page(page);
    let columns = frame
        .columns
        .into_iter()
        .map(|c| {
            let blocks = c
                .blocks
                .into_iter()
                .map(|b| block_view(b, screen))
                .collect_view();
            view! { <div class=c.class>{blocks}</div> }
        })
        .collect_view();
    view! { <div class=frame.class>{columns}</div> }
}

/// block の id から中身を描く module を選ぶ。
fn block_view(block: Block, screen: RwSignal<Screen>) -> AnyView {
    match block.id {
        id if id == next::BLOCK.id => next::view(),
        id if id == ask::BLOCK.id => ask::view(),
        id if id == askpage::BLOCK.id => askpage::view(),
        id if id == batch::BLOCK.id => batch::view(),
        id if id == policy::BLOCK.id => policy::view(),
        id if id == pipeline::BLOCK.id => pipeline::view(),
        id if id == seat::BLOCK.id => seat::view(),
        id if id == ledger::BLOCK.id => ledger::view(screen),
        id if id == legend::BLOCK.id => legend::view(),
        id if id == map::BLOCK.id => map::view(),
        id if id == gaps::BLOCK.id => gaps::view(),
        id if id == node::BLOCK.id => node::view(),
        id if id == nodearound::BLOCK.id => nodearound::view(),
        _ => ().into_any(),
    }
}
