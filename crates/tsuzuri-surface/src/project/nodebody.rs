//! block「本文と記録」（判断の記録 ADR-30 決定 (4)・要件 FR5）: 節点の頁の 3 つ目の block（つながりの後・
//! run の時間軸の前）。中心の節点が台帳の bead（契約・epic・memo・問い）の時だけ、その 1 本の引きの口（`ITEM_PATH`）から
//! 本文と記録（notes）を読み、本文の書式の部品 md で描いて 2 つの畳める段に置く（どちらの表示の型でも、その browser の
//! 保存に前の開き閉じが無ければ閉じて始め、開き閉じは畳める段の記録とその browser の保存〔段の種類ごとの鍵 2 つ・
//! store の FOLD_KEYS〕に書き戻す）。設計の節点と走行と決定は段を出さない。読む bead の決め方と中身の 3 値は
//! 純な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。
//! 1 本の引きの読みは頁に 1 つ（`item_source`）で、block「節点」の概要の箱も同じ読みから本文の頭の 1 行を取る。

use tsuzuri_contract::graph::{AroundRow, NodeKind};
use tsuzuri_contract::ledger::{BeadId, ITEM_PATH, LedgerItem};
use tsuzuri_contract::wire;

use super::node::center;
use super::nodearound::PageState;
use super::{Body, NO_CONTENT, NOT_READ};
use crate::frame::Block;
use crate::view::Fetched;
use crate::widgets::md::{self, Block as MdBlock};

pub const BLOCK: Block = Block {
    id: "body",
    heading: "nb_body",
    class: "panel",
};

/// この file が字を持つ口の path（無い・1 本の引きの口の path は契約の型の ITEM_PATH・行 hs-derived）。
pub const PATHS: &[&str] = &[];

/// この file の畳める段の開き閉じの鍵の形（本文と記録・行 hs-derived・行 g-node-body）。
pub const FOLDS: &[&str] = &["node:body", "node:notes"];

/// 本文と記録の段の見出しの語の鍵（FOLDS と同じ順）。
pub const PART_KEYS: [&str; 2] = ["nb_text", "nb_notes"];

/// 畳める段の class。
pub const PART_CLASS: &str = "fold nbody";

/// 段を出す節点の種類（台帳の bead）。
pub const KINDS: [NodeKind; 4] = [
    NodeKind::Task,
    NodeKind::Epic,
    NodeKind::Memo,
    NodeKind::Question,
];

/// 1 本の引きの口が読めないときの理由。
pub const REASON: &str = "台帳の 1 本の引きの口が読めない（server にまだ無い・届かない）";

/// 本文か記録が空のときの 1 行。
pub const BLANK: &str = "書かれていない";

/// 読んだ本文と記録（本文の書式の段の列）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texts {
    pub body: Vec<MdBlock>,
    pub notes: Vec<MdBlock>,
}

/// 中心の行の bead（台帳の bead の種類で、id が bead の id の形の時だけ）。
pub fn bead_of(row: &AroundRow) -> Option<BeadId> {
    if !KINDS.contains(&row.node.kind) {
        return None;
    }
    BeadId::new(row.node.id.clone()).ok()
}

/// bead の 1 本の引きの口の path。
pub fn item_path(bead: &BeadId) -> String {
    format!("{ITEM_PATH}{bead}")
}

/// 読む bead（近傍を読めた時は中心の行から決め、読み直しの間は前の bead を保つ）。
pub fn kept_bead(before: Option<BeadId>, state: &PageState) -> Option<BeadId> {
    match state {
        PageState::Doc(doc) => center(doc).and_then(bead_of),
        PageState::NotFound => None,
        PageState::NotRead | PageState::Unread(_) => before,
    }
}

/// 中身の 3 値（読めない間は測れていないの 1 行・ほかの bead の電文の間はまだ読んでいない）。
pub fn texts(fetched: &Fetched, bead: &BeadId) -> Body<Texts> {
    let text = match fetched {
        Fetched::NotRead => return Body::Unmeasured(NOT_READ),
        Fetched::Failed => return Body::Unmeasured(REASON),
        Fetched::Body(text) => text,
    };
    let Ok(item) = wire::decode::<LedgerItem>(text) else {
        return Body::Unmeasured(NO_CONTENT);
    };
    if item.row.id != *bead {
        return Body::Unmeasured(NOT_READ);
    }
    Body::Filled(Texts {
        body: md::parse(&item.description),
        notes: md::parse(&item.notes),
    })
}

#[cfg(target_arch = "wasm32")]
pub use dom::{ItemSource, item_source, view};

/// 本文と記録の block の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use std::cell::Cell;

    use leptos::prelude::*;
    use tsuzuri_contract::ledger::BeadId;

    use super::{
        BLANK, BLOCK, Body, Fetched, MdBlock, PART_CLASS, PART_KEYS, item_path, kept_bead, texts,
    };
    use crate::project::nodearound::{source, state};
    use crate::project::{fold, section, unmeasured};
    use crate::store;
    use crate::widgets::help::h2;
    use crate::widgets::md;

    /// toggle の後に今の開き閉じをその browser の保存にも写す（段の種類ごとの鍵・保存は便利の写しで正本でない・行 g-fold-keep）。
    fn kept(
        key: &'static str,
        toggle: impl Fn(web_sys::Event) + 'static,
    ) -> impl Fn(web_sys::Event) + 'static {
        move |ev: web_sys::Event| {
            let now = event_target::<web_sys::Element>(&ev).has_attribute("open");
            store::keep_fold(key, now);
            toggle(ev);
        }
    }

    /// 畳める段の中（空なら 1 行・字は台帳の字の印の下に描く）。
    fn part(blocks: Vec<MdBlock>) -> AnyView {
        if blocks.is_empty() {
            return view! { <div class="empty"><span>{BLANK}</span></div> }.into_any();
        }
        view! { <div data-ledger-text="">{md::view(blocks)}</div> }.into_any()
    }

    /// 頁に 1 つの中心の bead の 1 本の引きの読み（この block と block「節点」の概要の箱が分ける・行 g-node-excerpt）。
    #[derive(Clone, Copy)]
    pub struct ItemSource {
        near: ReadSignal<(Fetched, Option<u16>)>,
        pub bead: Memo<Option<BeadId>>,
        pub item: ReadSignal<(Fetched, Option<u16>)>,
    }

    thread_local! {
        /// 頁の block が分ける 1 本の引きの読み（近傍の読みと同じ一生・近傍の読みが替われば作り直す）。
        static ITEM: Cell<Option<ItemSource>> = const { Cell::new(None) };
    }

    /// 頁に 1 つの 1 本の引きの読み（近傍の読みが無ければ None・初めて呼んだ block が作り、次の block は同じ読みを使う）。
    pub fn item_source() -> Option<ItemSource> {
        let near = source().read?;
        if let Some(s) = ITEM.get().filter(|s| s.near == near) {
            return Some(s);
        }
        let bead = Memo::new(move |before: Option<&Option<BeadId>>| {
            let st = near.with(|(f, s)| state(f, *s));
            kept_bead(before.cloned().flatten(), &st)
        });
        let path =
            Signal::derive(move || bead.with(|b| b.as_ref().map(item_path).unwrap_or_default()));
        let item = crate::net::read_path(path);
        let s = ItemSource { near, bead, item };
        ITEM.set(Some(s));
        Some(s)
    }

    /// 本文と記録の block（台帳の bead でない節点と、近傍をまだ読んでいない間は何も出さない）。
    pub fn view() -> AnyView {
        let Some(ItemSource { bead, item, .. }) = item_source() else {
            return ().into_any();
        };
        let content = move || {
            let Some(b) = bead.get() else {
                return ().into_any();
            };
            match item.with(|(f, _)| texts(f, &b)) {
                Body::Filled(t) => {
                    let [text_key, notes_key] = PART_KEYS;
                    let (open_text, toggle_text) =
                        fold("node:body".to_string(), || store::fold_open("node:body"));
                    let (open_notes, toggle_notes) =
                        fold("node:notes".to_string(), || store::fold_open("node:notes"));
                    let toggle_text = kept("node:body", toggle_text);
                    let toggle_notes = kept("node:notes", toggle_notes);
                    let body = view! {
                        <details class=PART_CLASS prop:open=open_text on:toggle=toggle_text>
                            <summary>{h2(text_key)}</summary>
                            {part(t.body)}
                        </details>
                        <details class=PART_CLASS prop:open=open_notes on:toggle=toggle_notes>
                            <summary>{h2(notes_key)}</summary>
                            {part(t.notes)}
                        </details>
                    };
                    section(BLOCK, ().into_any(), body.into_any())
                }
                Body::Unmeasured(reason) => section(BLOCK, ().into_any(), unmeasured(reason)),
                Body::Empty(_) => ().into_any(),
            }
        };
        view! { {content} }.into_any()
    }
}
