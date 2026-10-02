//! 頁の描画（wasm の target のときだけ組み立てる・Leptos の csr）: header と頁の枠を frame の値のとおりに並べる。
//! block の中身は project の下の module が描く。ここは枠を描き、mode と頁を URL から読んで URL に残すだけ。
//! 「?」の注釈の層と hover の card の層は頁に 1 つずつ置く（便 g-parts）。札と一覧の行の吹き出しの層も頁に 1 つ置く（行 g-pop）。
//! 質問の link の数の印は問いの一覧の口（ask の module の口）の件数を出す（便 g-ask）。
//! 節点の頁（便 g-node）は query の page=node で開き、block は node と around の 2 つ（nav には出さない）。
//! 問いの頁の右の列（便 g-batch）は batch と policy の 2 つの block。
//! header の先頭の「戻る」（行 h-wire）は account board の窓 tz-account へ戻り、自分の窓を閉じる。
//! 在った窓へは前面へ出す前に閉じの知らせ（自分の窓の名）を送る（行 h-win-store）。
//! nav の印（見本の IC.home・IC.ask・IC.map・IC.gaps）は頁の定義の icon の字（行 hs-pages）。
//! 頁の題は project の名と頁の見出しの語で、節点の頁では読めた節点の題（行 g-title）。
//! 最初の案内（coach mark）の層は home の頁だけに置く（行 g-coach）。
//! 頁の link の押しは文書を読み直さずに URL を履歴に積んで頁の枠だけを組み直し、戻ると進むは URL から頁と mode を戻す（行 g-nav）。

use std::time::Duration;

use leptos::ev;
use leptos::prelude::*;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::project::PATH as PROJECT_PATH;

use crate::account::windows::{ACCOUNT_WIN, closed_message};
use crate::frame::{self, BACK, BACK_WRAP, BackHow, BackStep, Block, HEADER, Mode, PageId, Press};
use crate::fresh::{self, Fresh};
use crate::ledgerlist::SelCtx;
use crate::net;
use crate::project::{self, Module, ask, ledger};
use crate::seatpill;
use crate::store;
use crate::view::{PageSubject, Screen, brand, clock, clock_short, doc_title, kept_name};
use crate::vocab::label;
use crate::widgets::coach::CoachLayer;
use crate::widgets::help::{HelpCtx, TipLayer, qmark, term};
use crate::widgets::hover::{CardLayer, HoverCtx};
use crate::widgets::pop::{PopCtx, PopLayer};

/// 題の印（見本の IC.logo）。
const LOGO: &str = r##"<svg class="logo" viewBox="0 0 24 24" aria-hidden="true"><rect x="2" y="2" width="20" height="20" rx="6" fill="var(--accent)"/><path d="M7 8h10M7 12h10M7 16h6" stroke="#fff" stroke-width="2" stroke-linecap="round"/></svg>"##;

/// 戻るの印（見本の IC.up）。
const UP: &str = r#"<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M12 19V5M6 11l6-6 6 6"/></svg>"#;

/// body に画面を載せる。
pub fn mount() {
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

/// 頁の link の押し（mouse の button と修飾の鍵）。
fn press(e: &ev::MouseEvent) -> Press {
    Press {
        button: e.button(),
        ctrl: e.ctrl_key(),
        meta: e.meta_key(),
        shift: e.shift_key(),
        alt: e.alt_key(),
    }
}

/// 頁が app の窓の中か（display-mode の media query だけで見る・読めなければ普通の browser と見る）。
pub fn standalone() -> bool {
    window()
        .match_media(frame::STANDALONE_QUERY)
        .ok()
        .flatten()
        .is_some_and(|q| q.matches())
}

/// 名前つきの窓を開く（面の名前つきの窓はここだけが開く）: app の窓の中なら app の窓で、ほかは features なし。
/// 名で在る窓が返る時は features は効かない。
pub fn open_named(
    url: &str,
    name: &str,
) -> Result<Option<web_sys::Window>, web_sys::wasm_bindgen::JsValue> {
    window().open_with_url_and_target_and_features(url, name, frame::window_features(standalone()))
}

/// 頁の link の押しが、新しい窓や tab・保存でない左の押しか（frame の Press の plain）。
pub fn plain_click(e: &ev::MouseEvent) -> bool {
    press(e).plain()
}

/// 頁を替える（同じ頁なら何もしない）。前の枠の signal は片付くので、節点の頁の読みと頁の題の語を捨ててから替える。
fn go(page: RwSignal<PageId>, subject: RwSignal<PageSubject>, next: PageId) {
    if page.get_untracked() == next {
        return;
    }
    project::nodearound::forget();
    subject.set(PageSubject::default());
    page.set(next);
}

/// 頁の link の押し: 文書を読み直さずに URL を履歴に積んで頁を替える（新しい窓や tab で開く押しと
/// 今と同じ頁への押しは browser の既定のまま・見本の account board の tab の押しと同じ形）。
fn switch(
    e: ev::MouseEvent,
    to: PageId,
    page: RwSignal<PageId>,
    subject: RwSignal<PageSubject>,
    mode: RwSignal<Mode>,
) {
    let Some(url) = frame::switch_url(page.get_untracked(), to, mode.get_untracked(), press(&e))
    else {
        return;
    };
    e.prevent_default();
    if let Ok(history) = window().history() {
        let _ = history.push_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, "", Some(&url));
    }
    go(page, subject, to);
    window().scroll_to_with_x_and_y(0.0, 0.0);
}

#[component]
fn App() -> impl IntoView {
    let query = search();
    let page = RwSignal::new(PageId::from_query(&query));
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
    provide_context(PopCtx::default());
    // 選んだ epic の組（一覧と板が読む・行 g-select）。
    provide_context(SelCtx::default());
    // 台帳の一覧の口を読み、読みの結果が変わるたびに画面の状態を進める（最終更新は net の古さの最後に読めた時刻）。
    let fetched = net::read(ledger::PATH);
    let screen = RwSignal::new(Screen::initial());
    Effect::new(move |_| {
        let f = fetched.get();
        screen.update(|s| *s = s.after_read(&f, net::now()));
    });
    // 台帳の block は画面の状態を context から受ける（行 hs-blocks）。
    provide_context(screen);
    // 頁の題の語に替える字（節点の頁の block が節点の題を置く・行 g-title）。
    let subject = RwSignal::new(PageSubject::default());
    provide_context(subject);
    // 戻ると進むは履歴の URL から頁と mode を戻す（URL が状態の正・行 g-nav）。
    let back = window_event_listener(ev::popstate, move |_| {
        let now = search();
        let next = Mode::from_query(&now);
        if mode.get_untracked() != next {
            mode.set(next);
        }
        go(page, subject, PageId::from_query(&now));
    });
    on_cleanup(move || back.remove());
    view! {
        {top(page, subject, mode)}
        // 頁が替わるたびに頁の枠を組み直す（上端の帯と知らせの接続は組み直さない）。
        <main class="page">{move || page_view(page.get())}</main>
        <TipLayer/>
        <CardLayer/>
        <PopLayer/>
        {move || (page.get() == PageId::Home).then(|| view! { <CoachLayer/> })}
    }
}

/// account board へ戻る: 空の URL と窓の名 tz-account で開き、about:blank なら窓が無かった
/// （frame の `back_steps` の段のとおり、前面へか新しく開くかの後に自分の窓を閉じる）。
/// href が読めない窓（別の origin の account board）は在った窓と見る（行 g-back-note）。
/// close が効かない窓（script が開いた窓でない）は、待ちの後に探した結果を `note` に置く。
fn back_to_board(note: RwSignal<Option<BackHow>>) {
    let me = window();
    let win = open_named("", ACCOUNT_WIN).ok().flatten();
    let href = win.as_ref().and_then(|w| w.location().href().ok());
    let how = frame::back_how(win.is_some(), href.as_deref());
    for step in frame::back_steps(how == BackHow::Front) {
        match step {
            BackStep::Front => {
                if let Some(w) = &win {
                    // 在った窓へ自分の窓の名を知らせる（読み直した account board の一覧が閉じたにする・送り先の origin は限らない）。
                    let msg = closed_message(&me.name().unwrap_or_default());
                    let _ = w.post_message(&msg.into(), "*");
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
                // 閉じたかは読めなければ閉じたと見る（注記を出さない）。
                set_timeout(
                    move || {
                        if !window().closed().unwrap_or(true) {
                            note.set(Some(how));
                        }
                    },
                    Duration::from_millis(frame::BACK_NOTE_MS),
                );
            }
        }
    }
}

/// project の名の口を読み、読みの結果が変わるたびに名を進め、名が変わるたびに頁の題を置く（行 g-brand）。
/// 一度読めた名は読めない間も持ち続け、読めるまでは None（題の字は frame の BRAND）。
/// 題の語は今の頁の見出しの語で、PageSubject が在ればその字（行 g-title・頁が替われば置き直す・行 g-nav）。
fn project_name(page: RwSignal<PageId>, subject: RwSignal<PageSubject>) -> RwSignal<Option<String>> {
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
        let heading = page.get().def().heading;
        let title = name.with(|n| {
            subject.with(|PageSubject(s)| doc_title(n.as_deref(), heading, s.as_deref()))
        });
        document().set_title(&title);
    });
    name
}

/// 上端の帯: 戻る・題・頁の link・最終更新と読みの脈と読み込み不良の印と席の pill・mode の切り替え（frame の BACK と HEADER の順）。
/// 頁の一生に 1 度だけ組み、頁の link の押しは頁を切り替える（行 g-nav）。
fn top(
    page: RwSignal<PageId>,
    subject: RwSignal<PageSubject>,
    mode: RwSignal<Mode>,
) -> impl IntoView {
    let name = project_name(page, subject);
    let note = RwSignal::new(None);
    let back = view! {
        <span class=BACK_WRAP>
            <button type="button" class=BACK.class aria-label=label(BACK.key) on:click=move |_| back_to_board(note)>
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
                <a class=part.class href=move || frame::href(PageId::Home, mode.get()) on:click=move |e| switch(e, PageId::Home, page, subject, mode) aria-label=label(part.key)>
                    <span inner_html=LOGO></span>
                    <span class="name">{move || name.with(|n| brand(n.as_deref()).to_string())}</span>
                </a>
            }
            .into_any(),
            "nav" => {
                let questions = net::read(ask::PATH);
                let open = move || open_count(questions);
                let links = frame::nav_links(page.get_untracked())
                    .into_iter()
                    .map(|l| {
                        let badge = move || nav_badge(l.badge, open);
                        // 今の頁だけ on（nav_links の class）。
                        let class = move || nav_class(page.get(), l.page);
                        view! {
                            <a href=move || frame::href(l.page, mode.get()) class=class on:click=move |e| switch(e, l.page, page, subject, mode) data-v=l.key data-term=l.key>
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
                // 時刻は最後に読めた時刻（読みの落ちた口が在ればその最も古い値・行 g-fresh）。
                let fresh = net::fresh();
                let updated = move || fresh.with(Fresh::record);
                let title = move || updated().map(clock);
                let at = move || match updated() {
                    Some(t) => term(part.key, clock_short(t, net::now())),
                    None => view! { {project::state_icon(project::UNKNOWN)}{label("not_yet")} }.into_any(),
                };
                // 席の pill は mode の切り替えの前（home の頁は席の block が同じ状態を出すので出さない・行 g-seatpill）。
                let shown = Memo::new(move |_| frame::seat_shown(page.get()));
                let pill = move || shown.get().then(seatpill::view);
                view! { <span class=part.class title=title>{at}</span>{fresh::pulse()}{fresh::mark()}{pill} }.into_any()
            }
            _ => mode_seg(part, mode),
        })
        .collect_view();
    // 閉じられない窓の注記は header の直後（行 g-back-note）。
    view! { <header class="top">{back}{parts}</header>{move || note.get().map(back_note)} }
}

/// 開いた問いの数（読めていなければ None）。
fn open_count(questions: ReadSignal<crate::view::Fetched>) -> Option<usize> {
    match questions.with(ask::count) {
        Reading::Known(n) => Some(n),
        Reading::Unknown => None,
    }
}

/// nav の 1 項の数の札（札の class が空の項は数を読まない）。
fn nav_badge(badge: &'static str, open: impl Fn() -> Option<usize>) -> Option<impl IntoView> {
    let n = if badge.is_empty() { None } else { open() };
    frame::badge(n).map(|n| view! { <span class=badge>{n}</span> })
}

/// nav の 1 項の class（今の頁だけ on）。
fn nav_class(now: PageId, page: PageId) -> &'static str {
    frame::nav_links(now)
        .into_iter()
        .find(|c| c.page == page)
        .map_or("", |c| c.class)
}

/// mode の切り替えの段（押すと mode を替え、URL にも残す）。
fn mode_seg(part: &frame::HeaderPart, mode: RwSignal<Mode>) -> AnyView {
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

/// 閉じられない窓の注記（見本の ui.js の backToBoard の `#winnote`・本番は id で引かないので id を付けない）。
fn back_note(how: BackHow) -> impl IntoView {
    view! {
        <div class=frame::BACK_NOTE role="status">
            <b>{label(frame::BACK_NOTE_KEY)}</b>
            <span class="small muted">{frame::back_note(how)}</span>
        </div>
    }
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
