//! hover の card（見本の ui.js の card4 と同じ 4 行 + 「詳しく ▸」）。節点と札に指を置くと出す。
//! この便は部品の形だけを置き、動かすのは後の便（札と節点を持つ block を足す便）が決める。

/// card の中身（題・種類の行・値の行・出所・詳しく）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Card {
    pub title: String,
    pub kind: String,
    pub value: String,
    /// 出所の file と行（見本は card に出所を必ず持たせる）。
    pub src: String,
    pub more: Vec<String>,
}

impl Card {
    /// 4 行の class（見本の `.c4 .r.ti` などの行の順）。
    pub fn rows(&self) -> [(&'static str, &str); 4] {
        [
            ("r ti", &self.title),
            ("r k", &self.kind),
            ("r v", &self.value),
            ("r src", &self.src),
        ]
    }
}

/// card の箱の class（出ているか）。
pub fn card_class(open: bool) -> &'static str {
    if open { "hcard c4 on" } else { "hcard c4" }
}

#[cfg(target_arch = "wasm32")]
pub fn view(card: &Card, open: bool) -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    let rows = card
        .rows()
        .into_iter()
        .map(|(class, text)| view! { <div class=class>{text.to_string()}</div> })
        .collect_view();
    let more = (!card.more.is_empty()).then(|| {
        let lines = card
            .more
            .iter()
            .map(|m| view! { <div class="r2">{m.clone()}</div> })
            .collect_view();
        view! { <details class="more"><summary>"▸"</summary>{lines}</details> }
    });
    view! { <div class=card_class(open) role="tooltip">{rows}{more}</div> }.into_any()
}
