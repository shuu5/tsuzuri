//! 頁の描画（wasm の target のときだけ組み立てる・Leptos の csr）: 上の固定の帯と頁の枠を frame の値のとおりに並べる。
//! block の中身は project の下の module が描く。ここは枠を描き、mode と頁を URL から読んで URL に残すだけ。
//! 1 枚の画面（行 g-one-screen-a・判断の記録 ADR-27 決定 (3)(4)(5)）: 頁の上は帯（topbar の view）だけで、tab と頁の切り替えは
//! 持たない。home の頁は台帳 open の一覧と pipeline の 2 つの面で、帯の印が開く窓の層（widgets の modal の layer・中身は wins の
//! draw）を頁に 1 つ置き、窓を開くと吹き出しを閉じる。節点の頁（便 g-node）は query の page=node で開く（文書を読み直す）。
//! 窓を開く link（query の win・topbar の `win_of_href`）は、home の頁では普通の押しで頁を読み直さずに窓を開き、問いの id を
//! 持てば質問の窓をその問いから出す（行 g-one-screen-b）。URL の win で窓を開いた後は history を置き替えて URL から win と
//! 問いの id を外す（読み直しても同じ窓は開かない・窓の開け閉めは history を足さないので、戻るは前の頁へ・行 g-win-url）。
//! 「?」の注釈の層と hover の card の層は頁に 1 つずつ置く（便 g-parts）。札と一覧の行の吹き出しの層も頁に 1 つ置く（行 g-pop）。
//! 帯の戻る口（行 h-wire）は account board の窓 tz-account へ戻り、自分の窓を閉じる。
//! 在った窓へは前面へ出す前に閉じの知らせ（自分の窓の名）を送る（行 h-win-store）。
//! 最終の記録と読みの脈と読み込み不良の印（行 g-fresh・g-pulse）は帯の下の右端の小さな札に置く。
//! 頁の題は project の名と頁の見出しの語で、節点の頁では読めた節点の題（行 g-title）。
//! 最初の案内（coach mark）の層は home の頁だけに置く（行 g-coach）。

use std::time::Duration;

use leptos::ev;
use leptos::prelude::*;
use tsuzuri_contract::project::PATH as PROJECT_PATH;
use web_sys::wasm_bindgen::JsCast;

use crate::account::windows::{ACCOUNT_WIN, closed_message};
use crate::askwin::AskFocus;
use crate::frame::{self, BackHow, BackStep, Block, Mode, PageId, Press};
use crate::fresh::{self, Fresh};
use crate::ledgerlist::SelCtx;
use crate::net;
use crate::project::{self, Module};
use crate::seatpill;
use crate::store;
use crate::topbar::{self, Bar, Win, win_of_href};
use crate::view::{PageSubject, brand, clock, clock_short, doc_title, kept_name};
use crate::vocab::label;
use crate::widgets::coach::CoachLayer;
use crate::widgets::help::{HelpCtx, TipLayer, term};
use crate::widgets::hover::{CardLayer, HoverCtx};
use crate::widgets::modal::{SCRIM, WinCtx, layer};
use crate::widgets::pop::{PopCtx, PopLayer};
use crate::wins;

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

/// URL の win で窓を開いた後に、頁の URL から窓を開く値を外す（history の置き替え・行 g-win-url）。
fn settle_url(page: PageId, query: &str) {
    let loc = window().location();
    let url = format!(
        "{}{}{}",
        loc.pathname().unwrap_or_default(),
        topbar::settled_query(query, page),
        loc.hash().unwrap_or_default()
    );
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

#[component]
fn App() -> impl IntoView {
    let query = search();
    // 頁は文書の一生の間替わらない（tab と頁の切り替えを持たない・行 g-one-screen-a）。
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
    provide_context(PopCtx::default());
    let pop = use_context::<PopCtx>();
    // 選んだ epic の組（一覧と板が読む・行 g-select）。
    provide_context(SelCtx::default());
    // 帯の印が開く窓の積み（窓を開くと吹き出しを閉じる・見本の openModal の closePop）。
    let win = WinCtx::<Win>::default();
    provide_context(win);
    // 質問の窓を開いた link の問いの id（行 g-one-screen-b）。
    let focus = AskFocus(RwSignal::new(None));
    provide_context(focus);
    // URL の query の win が名指す窓は頁を開いた時に開く（消した質問の頁と抜けの検査の頁への link の替わり）。
    if let Some((w, id)) = win_of_href(&query) {
        focus.0.set(id);
        win.open(w, false);
        settle_url(page, &query);
    }
    // home の頁では、窓を開く link の普通の押しは頁を読み直さずにその窓を開く（窓の中の link は前の窓に戻る口を持つ）。
    if page == PageId::Home {
        let links = window_event_listener(ev::click, move |e| open_link(&e, win, focus));
        on_cleanup(move || links.remove());
    }
    Effect::new(move |_| {
        if win.top().is_some()
            && let Some(p) = pop
        {
            p.close();
        }
    });
    // 頁の題の語に替える字（節点の頁の block が節点の題を置く・行 g-title）。
    let subject = RwSignal::new(PageSubject::default());
    provide_context(subject);
    view! {
        {top(page, subject, mode, win)}
        <main class="page">{page_view(page)}</main>
        {layer(win, move |w| wins::draw(w, win))}
        <TipLayer/>
        <CardLayer/>
        <PopLayer/>
        {(page == PageId::Home).then(|| view! { <CoachLayer/> })}
    }
}

/// 押した要素の在る link が窓を開く link で、普通の押しなら、既定の遷移を止めて窓を開く（行 g-one-screen-b）。
fn open_link(e: &ev::MouseEvent, win: WinCtx<Win>, focus: AskFocus) {
    let Some(el) = e
        .target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let Some(link) = el.closest("a[href]").ok().flatten() else {
        return;
    };
    let href = link.get_attribute("href").unwrap_or_default();
    let Some((w, id)) = win_of_href(&href) else {
        return;
    };
    if !plain_click(e) {
        return;
    }
    e.prevent_default();
    let from_win = el.closest(&format!("#{}", SCRIM)).ok().flatten().is_some();
    focus.0.set(id);
    win.open(w, from_win);
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
/// 題の語は頁の見出しの語で、PageSubject が在ればその字（行 g-title）。
fn project_name(page: PageId, subject: RwSignal<PageSubject>) -> RwSignal<Option<String>> {
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
        let heading = page.def().heading;
        let title = name.with(|n| {
            subject.with(|PageSubject(s)| doc_title(n.as_deref(), heading, s.as_deref()))
        });
        document().set_title(&title);
    });
    name
}

/// 上の固定の帯（topbar の Bar・頁の一生に 1 度だけ組む・行 g-one-screen-a）: 戻る口は account board の窓へ戻り、
/// 表示の型の選びは mode を替えて URL と保存に残す。閉じられない窓の注記は帯の直後（行 g-back-note）。
/// 最終の記録と読みの脈と読み込み不良の印と席の pill（home のほかの頁だけ・行 g-seatpill）は帯の下の右端の小さな札。
fn top(
    page: PageId,
    subject: RwSignal<PageSubject>,
    mode: RwSignal<Mode>,
    wins: WinCtx<Win>,
) -> impl IntoView {
    let name = project_name(page, subject);
    let note = RwSignal::new(None);
    let bar = Bar {
        wins,
        mode,
        name: Signal::derive(move || name.with(|n| brand(n.as_deref()).to_string())),
        back: Callback::new(move |()| back_to_board(note)),
        pick: Callback::new(move |m: Mode| {
            mode.set(m);
            keep_mode_in_url(m);
        }),
    };
    // 字は短い時刻で、pointer を乗せると語の説明を出す（長い時刻は title に残す）。
    // 時刻は最後に読めた時刻（読みの落ちた口が在ればその最も古い値・行 g-fresh）。
    let fresh = net::fresh();
    let updated = move || fresh.with(Fresh::record);
    let title = move || updated().map(clock);
    let key = frame::UPDATED.key;
    let at = move || match updated() {
        Some(t) => term(key, clock_short(t, net::now())),
        None => view! { {project::state_icon(project::UNKNOWN)}{label("not_yet")} }.into_any(),
    };
    let pill = frame::seat_shown(page).then(seatpill::view);
    view! {
        {topbar::view(bar)}
        {move || note.get().map(back_note)}
        <div class=FRESH_CLASS><span class="chip num" title=title>{at}</span>{fresh::pulse()}{fresh::mark()}{pill}</div>
    }
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

/// 帯の下の右端の小さな札の class（最終の記録と読みの脈と読み込み不良の印）。
const FRESH_CLASS: &str = "freshw";

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
