//! 頁の描画（wasm の target のときだけ組み立てる・Leptos の csr）: header と頁の枠を frame の値のとおりに並べる。
//! block の中身は project の下の module が描く。ここは枠を描き、mode と頁を URL から読んで URL に残すだけ。
//! 「?」の注釈の層と hover の card の層は頁に 1 つずつ置く（便 g-parts）。
//! 質問の link の数の印は問いの一覧の口（ask の module の口）の件数を出す（便 g-ask）。
//! 節点の頁（便 g-node）は query の page=node で開き、block は node と around の 2 つ（nav には出さない）。
//! 問いの頁の右の列（便 g-batch）は batch と policy の 2 つの block。
//! header の先頭の「戻る」（行 h-wire）は account board の窓 tz-account へ戻り、自分の窓を閉じる。
//! nav の印（見本の IC.home・IC.ask・IC.map・IC.gaps）は頁の定義の icon の字（行 hs-pages）。

use leptos::prelude::*;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::project::PATH as PROJECT_PATH;

use crate::account::windows::ACCOUNT_WIN;
use crate::frame::{self, BACK, BACK_WRAP, BackStep, Block, HEADER, Mode, PageId};
use crate::net;
use crate::project::{self, Module, ask, ledger};
use crate::view::{Screen, board_title, brand, clock, clock_short, kept_name};
use crate::vocab::label;
use crate::widgets::help::{HelpCtx, TipLayer, qmark, term};
use crate::widgets::hover::{CardLayer, HoverCtx};

/// 題の印（見本の IC.logo）。
const LOGO: &str = r##"<svg class="logo" viewBox="0 0 24 24" aria-hidden="true"><rect x="2" y="2" width="20" height="20" rx="6" fill="var(--accent)"/><path d="M7 8h10M7 12h10M7 16h6" stroke="#fff" stroke-width="2" stroke-linecap="round"/></svg>"##;

/// 戻るの印（見本の IC.up）。
const UP: &str = r#"<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M12 19V5M6 11l6-6 6 6"/></svg>"#;

/// 空の URL で開いた新しい窓の href。
const BLANK: &str = "about:blank";

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
    // 台帳の block は画面の状態を context から受ける（行 hs-blocks）。
    provide_context(screen);
    view! {
        {top(page, mode, screen)}
        <main class="page">{page_view(page)}</main>
        <TipLayer/>
        <CardLayer/>
    }
}

/// account board へ戻る: 空の URL と窓の名 tz-account で開き、about:blank なら窓が無かった
/// （frame の `back_steps` の段のとおり、前面へか新しく開くかの後に自分の窓を閉じる）。
fn back_to_board() {
    let me = window();
    let win = me
        .open_with_url_and_target("", ACCOUNT_WIN)
        .ok()
        .flatten();
    let account_open = win
        .as_ref()
        .is_some_and(|w| w.location().href().is_ok_and(|href| href != BLANK));
    for step in frame::back_steps(account_open) {
        match step {
            BackStep::Front => {
                if let Some(w) = &win {
                    let _ = w.focus();
                }
            }
            BackStep::OpenNew(url) => {
                if let Some(w) = &win {
                    // 空の窓の URL は、この頁の origin と path に query を足した字。
                    let loc = me.location();
                    let base = format!(
                        "{}{}",
                        loc.origin().unwrap_or_default(),
                        loc.pathname().unwrap_or_default()
                    );
                    let _ = w.location().set_href(&format!("{base}{url}"));
                    let _ = w.focus();
                }
            }
            BackStep::CloseSelf => {
                let _ = me.close();
            }
        }
    }
}

/// project の名の口を読み、読みの結果が変わるたびに名を進め、名が変わるたびに頁の題を置く（行 g-brand）。
/// 一度読めた名は読めない間も持ち続け、読めるまでは None（題の字は frame の BRAND）。
fn project_name() -> RwSignal<Option<String>> {
    let fetched = net::read(PROJECT_PATH);
    let name = RwSignal::new(None);
    Effect::new(move |_| {
        let f = fetched.get();
        let next = kept_name(name.get_untracked(), &f);
        if name.with_untracked(|n| *n != next) {
            name.set(next);
        }
    });
    Effect::new(move |_| {
        let title = name.with(|n| board_title(n.as_deref()));
        document().set_title(&title);
    });
    name
}

/// 上端の帯: 戻る・題・頁の link・最終更新・mode の切り替え（frame の BACK と HEADER の順）。
fn top(page: PageId, mode: RwSignal<Mode>, screen: RwSignal<Screen>) -> impl IntoView {
    let name = project_name();
    let back = view! {
        <span class=BACK_WRAP>
            <button type="button" class=BACK.class aria-label=label(BACK.key) on:click=move |_| back_to_board()>
                <span inner_html=UP></span>
                <span class="lbl">{label(BACK.key)}</span>
            </button>
            {qmark(BACK.key)}
        </span>
    };
    let parts = HEADER
        .iter()
        .map(|part| match part.part {
            "brand" => view! {
                <a class=part.class href=move || frame::href(PageId::Home, mode.get()) aria-label=label(part.key)>
                    <span inner_html=LOGO></span>
                    <span class="name">{move || name.with(|n| brand(n.as_deref()).to_string())}</span>
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
                                <span inner_html=l.page.def().icon></span>
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
                // 字は短い時刻で、pointer を乗せると語の説明を出す（長い時刻は title に残す）。
                let updated = move || screen.with(|s| s.updated_at);
                let title = move || updated().map(clock);
                let at = move || match updated() {
                    Some(t) => term(part.key, clock_short(t, net::now())),
                    None => view! { {project::state_icon(project::UNKNOWN)}{label("not_yet")} }.into_any(),
                };
                view! { <span class=part.class title=title>{at}</span> }.into_any()
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
    view! { <header class="top">{back}{parts}</header> }
}

/// 頁の枠（frame の列と block の並びのとおり）。
fn page_view(page: PageId) -> impl IntoView {
    let frame = frame::page(page);
    let columns = frame
        .columns
        .into_iter()
        .map(|c| {
            let blocks = c.blocks.into_iter().map(block_view).collect_view();
            view! { <div class=c.class>{blocks}</div> }
        })
        .collect_view();
    view! { <div class=frame.class>{columns}</div> }
}

/// block の id から中身を描く module を選ぶ（生成した列の中から枠の id の同じ module・無ければ空）。
fn block_view(block: Block) -> AnyView {
    Module::ALL
        .into_iter()
        .find(|m| m.block().id == block.id)
        .map_or_else(|| ().into_any(), Module::view)
}
