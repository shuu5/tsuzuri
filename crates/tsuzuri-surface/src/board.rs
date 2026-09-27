//! 最小の project board の描画（wasm の target のときだけ組み立てる・Leptos の csr）。
//! 出すのは問いの一覧・台帳の一覧・最終更新の 3 つ。何を出すか・どの順かは view が決め、ここは描くだけ。

use leptos::prelude::*;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::LedgerRow;

use crate::net;
use crate::view::{self, Board, EpicGroup, Screen, UNMEASURED};

/// body に画面を載せる。
pub fn mount() {
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let screen = RwSignal::new(Screen::initial());
    net::follow(screen);
    view! {
        <main class="board">
            <header class="top">
                <h1>"project board"</h1>
                <span class="updated">{move || screen.with(view::updated_label)}</span>
            </header>
            <section class="questions">
                <h2>{move || screen.with(view::questions_label)}</h2>
                {move || screen.with(|s| match &s.board {
                    Reading::Known(board) => questions(board),
                    Reading::Unknown => unmeasured(),
                })}
            </section>
            <section class="ledger">
                <h2>{move || screen.with(view::ledger_label)}</h2>
                {move || screen.with(|s| match &s.board {
                    Reading::Known(board) => ledger(board),
                    Reading::Unknown => unmeasured(),
                })}
            </section>
        </main>
    }
}

/// 測れていないの表示（0 件の空の一覧と区別する）。
fn unmeasured() -> AnyView {
    view! { <p class="unmeasured">{UNMEASURED}</p> }.into_any()
}

fn questions(board: &Board) -> AnyView {
    if board.questions.is_empty() {
        return view! { <p class="empty">"open の問いは無い"</p> }.into_any();
    }
    let items = board
        .questions
        .iter()
        .map(|q| view! { <li class="row">{row(q)}</li> })
        .collect_view();
    view! { <ol class="rows">{items}</ol> }.into_any()
}

fn ledger(board: &Board) -> AnyView {
    if board.groups.is_empty() {
        return view! { <p class="empty">"台帳に bead は無い"</p> }.into_any();
    }
    board.groups.iter().map(group).collect_view().into_any()
}

fn group(group: &EpicGroup) -> AnyView {
    let head = match &group.epic {
        Some(epic) => row(epic),
        None => view! { <span class="title">"epic の外"</span> }.into_any(),
    };
    let children = group
        .children
        .iter()
        .map(|c| view! { <li class="row">{row(c)}</li> })
        .collect_view();
    view! {
        <div class="epic">
            <div class="row epic-head">{head}</div>
            <ul class="rows children">{children}</ul>
        </div>
    }
    .into_any()
}

/// 1 行（状態の印・id・題・種類）。状態の bd の語は印の title に残す。
fn row(r: &LedgerRow) -> AnyView {
    let mark = view::mark(&r.status);
    let tip = format!("{}（{}）", mark.word, r.status);
    view! {
        <span class=format!("mark {}", mark.class) title=tip>{mark.glyph}</span>
        <span class="id">{r.id.to_string()}</span>
        <span class="title">{r.title.clone()}</span>
        <span class="kind">{r.kind.clone()}</span>
    }
    .into_any()
}
