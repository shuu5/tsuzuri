//! project board の block（便 g-frame）: 1 つの block（と地図の頁）に 1 つの module。
//! 各 module は枠の値（`BLOCK`）と中身の純粋な関数を持ち、DOM は wasm の target のときだけ組み立てる。
//! 中身を持つのは ask（問いの一覧）・ledger（台帳の一覧）・legend（凡例）の 3 つ。
//! data の口がまだ無い next・pipeline・seat・map は測れていないの印と理由の 1 行を出す（後の便が module ごとに足す）。

use tsuzuri_contract::ledger::LedgerRow;

use crate::view::{self, QUESTION_KIND};

pub mod ask;
pub mod ledger;
pub mod legend;
pub mod map;
pub mod next;
pub mod pipeline;
pub mod seat;

/// block の中身（測れていない・0 件・中身あり）。0 件と測れていないを分ける（要件 NFR2）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body<T> {
    /// 測れていない（理由の 1 行）。
    Unmeasured(&'static str),
    /// 測れて 0 件（その 1 行）。
    Empty(&'static str),
    Filled(T),
}

/// 状態の記号の 5 値（見本の ui.js の ST_V と同じ順・語の鍵）。
pub const STATES: [(&str, &str); 5] = [
    ("run", "st_run"),
    ("wait", "st_wait"),
    ("limit", "st_limit"),
    ("silent", "st_silent"),
    ("unknown", "st_unknown"),
];

/// 測れていないの記号の値。
pub const UNKNOWN: &str = "unknown";

/// 状態の記号の語の鍵（知らない値は測れていない）。
pub fn state_key(v: &str) -> &'static str {
    STATES
        .iter()
        .find(|(s, _)| *s == v)
        .map_or("st_unknown", |(_, k)| k)
}

/// 状態の記号の class（見本の `.st.st-<値>`）。
pub fn state_class(v: &str) -> String {
    format!("st st-{v}")
}

/// 一覧の 1 項（見本の `ul.items > li` = 印・番号・題・右に小さい字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// 印の class（出所の帯 beads の丸・閉じていれば塗る）。
    pub shape: String,
    /// open の問いの印は赤（見本の nodeShape と同じ）。
    pub alert: bool,
    pub id: String,
    pub title: String,
    /// 状態の印と語と種類（`○ 未着手 · task`）。
    pub aside: String,
}

/// 印の色の上書き（open の問い）。
pub const ALERT_STYLE: &str = "color:var(--s-stop)";

/// 台帳の行を一覧の 1 項にする。
pub fn item(row: &LedgerRow) -> Item {
    let open = matches!(row.status.as_str(), "open" | "in_progress");
    let alert = row.kind == QUESTION_KIND && row.status == "open";
    let mark = view::mark(&row.status);
    Item {
        shape: format!("shape band-beads{}", if open { "" } else { " fill" }),
        alert,
        id: row.id.to_string(),
        title: row.title.clone(),
        aside: format!("{} {} · {}", mark.glyph, mark.word, row.kind),
    }
}

/// 台帳の一覧の口が読めないときの理由。
pub const LEDGER_UNREAD: &str =
    "台帳の一覧の口が読めない（届かない・知らせが切れた）ので今の一覧が正しいと言えない";

#[cfg(target_arch = "wasm32")]
pub use dom::{item_view, section, state_icon, unmeasured};

/// block に共通の DOM の部品（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::{ALERT_STYLE, Item, UNKNOWN, state_class, state_key};
    use crate::frame::Block;
    use crate::vocab::label;
    use crate::widgets::help::h2;

    /// 砂時計（限度の記号・見本の IC.hourglass）。
    const HOURGLASS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linejoin="round" aria-hidden="true"><path d="M6 3h12M6 21h12"/><path d="M7 3c0 5 5 6 5 9s-5 4-5 9h10c0-5-5-6-5-9s5-4 5-9" /><path d="M9.5 19h5l-2.5-2.5z" fill="currentColor" stroke="none"/></svg>"#;

    /// block の section（見出しの横に `extra`・下に `body`）。
    pub fn section(block: Block, extra: AnyView, body: AnyView) -> AnyView {
        view! {
            <section class=block.class id=block.id>
                <header>{h2(block.heading)}{extra}</header>
                {body}
            </section>
        }
        .into_any()
    }

    /// 状態の記号（5 値・色 + 形 + 動き）。
    pub fn state_icon(v: &'static str) -> AnyView {
        let aria = label(state_key(v));
        if v == "limit" {
            view! { <span class=state_class(v) role="img" aria-label=aria data-stv=v inner_html=HOURGLASS></span> }
                .into_any()
        } else {
            view! { <span class=state_class(v) role="img" aria-label=aria data-stv=v><i></i></span> }.into_any()
        }
    }

    /// 測れていないの印と理由の 1 行（0 件と区別する）。
    pub fn unmeasured(reason: &'static str) -> AnyView {
        view! {
            <div class="empty">
                {state_icon(UNKNOWN)}
                <span>{label(state_key(UNKNOWN))}</span>
                <span class="small muted">{reason}</span>
            </div>
        }
        .into_any()
    }

    /// 一覧の 1 項（`lead` は印の代わりに置く番号など・None なら印）。
    pub fn item_view(item: &Item, number: Option<usize>) -> AnyView {
        let lead = match number {
            Some(n) => view! { <span class="nb">{n}</span> }.into_any(),
            None => {
                let style = if item.alert { ALERT_STYLE } else { "" };
                view! { <span class=item.shape.clone() style=style aria-hidden="true"></span> }
                    .into_any()
            }
        };
        view! {
            <li>
                {lead}
                <span class="ttl"><span class="nid">{item.id.clone()}</span>" "<span data-t="">{item.title.clone()}</span></span>
                <span class="aside">{item.aside.clone()}</span>
            </li>
        }
        .into_any()
    }
}
