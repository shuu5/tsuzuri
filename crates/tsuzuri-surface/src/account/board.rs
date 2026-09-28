//! account board の頁の描画（wasm の target のときだけ組み立てる・Leptos の csr）: header と tab の枠を account の値のとおりに並べる。
//! block の中身は account の下の module が描く。ここは枠を描き、tab と mode を URL から読んで URL に残すだけ。
//! 関係の頁への link・時点のつまみは出さない（未決）。最終の記録の chip は tab の link の後に、休止中の chip はその直後に置く。

use leptos::ev;
use leptos::prelude::*;

use super::{
    BRAND, DORMANT_CARD, DORMANT_CLASS, DORMANT_STATE, HEADER, PATH, TOP, Tab, UPDATED_CLASS,
    UPDATED_KEY, badge, doc, dormant_card, dormant_chip, home, ledger, page, page_title, projects,
    session, tab_href, tab_links, tab_url, windows,
};
use crate::frame::{self, Block, Mode};
use crate::fresh::{self, Fresh};
use crate::net;
use crate::project;
use crate::store;
use crate::view::{clock, clock_short};
use crate::vocab::label;
use crate::widgets::help::{HelpCtx, TipLayer, term};
use crate::widgets::hover::{CardLayer, HoverCtx, attach};

/// 題の印（見本の IC.vessel）。
const VESSEL: &str = r#"<svg class="logo" viewBox="0 0 24 24" aria-hidden="true"><rect x="2" y="2" width="20" height="20" rx="6" fill="var(--ink)"/><path d="M6 9h12M6 13h12M6 17h12" stroke="var(--bg)" stroke-width="2" stroke-linecap="round"/><circle cx="17" cy="6" r="2" fill="var(--st-run)"/></svg>"#;

/// tab の印（見本の ACCT_TAB_IC = IC.home・IC.clock・IC.folder）。
fn tab_icon(tab: Tab) -> &'static str {
    match tab {
        Tab::Home => {
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M3 11l9-7 9 7v9a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z"/></svg>"#
        }
        Tab::Session => {
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg>"#
        }
        Tab::Projects => {
            r#"<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M3 6h6l2 2h10v11H3z"/></svg>"#
        }
    }
}

/// body に account board を載せる。
pub fn mount() {
    // 自分の窓の名（project board の「戻る」がこの名でこの窓を見つける）。
    let _ = window().set_name(windows::ACCOUNT_WIN);
    // 頁の題（index.html は project board と同じ file なので、ここで account board の題に替える）。
    document().set_title(&page_title());
    // 保存の mode を URL に揃えてから App が URL を読む（行 g-mode-store）。
    store::settle_mode();
    leptos::mount::mount_to_body(App);
}

/// 今の URL の query（読めなければ空）。
fn search() -> String {
    window().location().search().unwrap_or_default()
}

/// mode を URL の query に残し、保存にも写す（頁は読み直さない）。
fn keep_mode_in_url(mode: Mode) {
    store::keep_mode(mode);
    let url = frame::with_param(&search(), "mode", mode.key());
    if let Ok(history) = window().history() {
        let _ =
            history.replace_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, "", Some(&url));
    }
}

#[component]
fn App() -> impl IntoView {
    // 頁の今の URL の query（tab の押しと戻ると進むで替わる・今の tab はここから導く）。
    let query = RwSignal::new(search());
    let mode = RwSignal::new(Mode::from_query(&query.get_untracked()));
    // 戻ると進むは履歴の URL から tab と mode を戻す（URL が状態の正）。
    let back = window_event_listener(ev::popstate, move |_| {
        let now = search();
        mode.set(Mode::from_query(&now));
        query.set(now);
    });
    on_cleanup(move || back.remove());
    provide_context(HelpCtx {
        open: RwSignal::new(None),
        mode,
    });
    // body の class で「?」の出し分けを決める（見本の ui.css の body.mode-beginner）。
    Effect::new(move |_| {
        if let Some(body) = document().body() {
            body.set_class_name(mode.get().body_class());
        }
    });
    provide_context(HoverCtx::default());
    // 口は頁に 1 本（block も同じ path を read に渡し、同じ signal を分け合う）。
    let fetched = net::read(PATH);
    let read = Memo::new(move |_| fetched.with(doc).ok());
    view! {
        {top(query, mode, read)}
        // query が替わるたびに今の tab の block を組み直す（block は組むときに URL の並べ方と幅を読む）。
        <main class="page">{move || page_view(query.with(|q| Tab::from_query(q)))}</main>
        <TipLayer/>
        <CardLayer/>
    }
}

/// 上端の帯: 題・tab の link・最終の記録と読みの脈と読み込み不良の印・mode の切り替え（account の HEADER の順）。
fn top(
    query: RwSignal<String>,
    mode: RwSignal<Mode>,
    read: Memo<Option<tsuzuri_contract::account::AccountDoc>>,
) -> impl IntoView {
    let tab = Memo::new(move |_| query.with(|q| Tab::from_query(q)));
    let parts = HEADER
        .iter()
        .map(|part| match part.part {
            "brand" => view! {
                <a class=part.class href=move || tab_href(Tab::Home, mode.get()) aria-label=label(part.key)>
                    <span inner_html=VESSEL></span>
                    <span class="name">{BRAND}</span>
                </a>
            }
            .into_any(),
            "nav" => {
                let links = tab_links(tab.get_untracked())
                    .into_iter()
                    .map(|l| {
                        let n = move || read.with(|d| d.as_ref().and_then(|d| badge(l.tab, d)));
                        let mark = move || {
                            n().map(|n| {
                                let aria = n.clone();
                                view! { <span class=l.badge aria-label=aria>{n}</span> }
                            })
                        };
                        // 今の tab だけ on（tab_links の class）と aria-current。
                        let class = move || {
                            tab_links(tab.get())
                                .into_iter()
                                .find(|c| c.tab == l.tab)
                                .map_or("", |c| c.class)
                        };
                        let current = move || (tab.get() == l.tab).then_some("page");
                        // 押しは頁を読み直さず履歴に積む（新しい窓や tab で開く押しは browser の既定のまま・見本の setTab）。
                        let press = move |e: ev::MouseEvent| {
                            if e.ctrl_key() || e.meta_key() || e.shift_key() {
                                return;
                            }
                            e.prevent_default();
                            let url = tab_url(&search(), l.tab);
                            if let Ok(history) = window().history() {
                                let _ = history.push_state_with_url(
                                    &web_sys::wasm_bindgen::JsValue::NULL,
                                    "",
                                    Some(&url),
                                );
                            }
                            query.set(url);
                            window().scroll_to_with_x_and_y(0.0, 0.0);
                        };
                        view! {
                            <a href=move || tab_href(l.tab, mode.get()) class=class aria-current=current on:click=press data-tab=l.tab.id() data-v=l.key data-term=l.key>
                                <span inner_html=tab_icon(l.tab)></span>
                                <span class="lbl hd-t">{label(l.key)}</span>
                                {mark}
                            </a>
                        }
                    })
                    .collect_view();
                view! {
                    <nav class=part.class aria-label=label(part.key)>{links}</nav>
                    <span class="grow"></span>
                    {updated_chip()}
                    {move || dormant_view(read)}
                    {fresh::pulse()}
                    {fresh::mark()}
                }
                .into_any()
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
    view! { <header class=TOP>{parts}</header> }
}

/// 最終の記録の chip（最後に読めた時刻・読みの落ちた口が在ればその最も古い値・行 g-fresh・
/// 短い時刻・長い時刻は title・まだ読めていなければ測れていないの印と語）。
fn updated_chip() -> impl IntoView {
    let fresh = net::fresh();
    let updated = move || fresh.with(Fresh::record);
    let title = move || updated().map(clock);
    let at = move || match updated() {
        Some(t) => term(UPDATED_KEY, clock_short(t, net::now())),
        None => view! { {project::state_icon(project::UNKNOWN)}{label("not_yet")} }.into_any(),
    };
    view! { <span class=UPDATED_CLASS title=title>{at}</span> }
}

/// 休止中の chip（行 h-dormant・電文が読めて休止中の席が 1 つ以上のときだけ・状態の記号と語と数・hover で card）。
fn dormant_view(read: Memo<Option<tsuzuri_contract::account::AccountDoc>>) -> Option<AnyView> {
    read.with(|d| {
        let d = d.as_ref()?;
        let chip = dormant_chip(d)?;
        let card = dormant_card(d)?;
        Some(
            view! {
                <span class=DORMANT_CLASS data-card=DORMANT_CARD tabindex="0" use:attach=card>
                    {project::state_icon(DORMANT_STATE)}{chip}
                </span>
            }
            .into_any(),
        )
    })
}

/// tab の枠（段の class と block の並びのとおり）。
fn page_view(tab: Tab) -> impl IntoView {
    page(tab)
        .rows
        .into_iter()
        .map(|r| {
            let blocks = r.blocks.into_iter().map(block_view).collect_view();
            view! { <div class=r.class>{blocks}</div> }
        })
        .collect_view()
}

/// block の id から中身を描く module を選ぶ。
fn block_view(block: Block) -> AnyView {
    match block.id {
        id if home::BLOCKS.iter().any(|b| b.id == id) => home::view(block),
        id if id == windows::BLOCK.id => windows::view(),
        id if id == session::BLOCK.id => session::view(),
        id if id == ledger::BLOCK.id => ledger::view(),
        id if id == projects::BLOCK.id => projects::view(),
        _ => ().into_any(),
    }
}
