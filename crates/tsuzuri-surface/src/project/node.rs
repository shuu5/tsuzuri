//! block「節点」（見本の bead.html の頭と 2 面の概要・便 g-node）: 節点の頁の 1 つ目の block。
//! 近傍の口の電文（block「つながり」と同じ 1 つの読み・nodearound の module が持つ）の中心の行（列 0）から頭と概要を組む。
//! 頭・概要・質問の頁への link は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use tsuzuri_contract::graph::{AroundDoc, AroundRow, NodeKind, title36};

use crate::frame::{Block, Mode};
use crate::mapview::band::{Band, band_of, kind_key};
use crate::mapview::{encode, is_open};

pub const BLOCK: Block = Block {
    id: "node",
    heading: "summary_plain",
    class: "stack",
};

/// この file が字を持つ口の path（無い・近傍の口は nodearound の module が持つ・行 hs-derived）。
pub const PATHS: &[&str] = &[];

/// この file の畳める段の開き閉じの鍵の形（無い・行 hs-derived）。
pub const FOLDS: &[&str] = &[];

/// 電文に要約の欄がまだ無いときの概要の字。
pub const NO_SUMMARY: &str = "要約なし";

/// 出所の file を持たない節点（台帳と走行）の出所の字。
pub const NO_SRC: &str = "出所なし";

/// 概要の箱の見出しの語の鍵（非エンジニア向け・エンジニア向けの順）。
pub const SUMMARY_KEYS: [&str; 2] = ["summary_plain", "summary_eng"];

/// 要約の無い概要の箱の class。
pub const SUMMARY_NONE: &str = "sumbox none";

/// 節点の頁の頭（印・種類の見出し・帯・状態・id・題・出所・質問の頁への link）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
    /// 印の class（帯の色・動いている節点は塗らない・大きい印）。
    pub shape: String,
    /// open の問いの印は赤。
    pub alert: bool,
    /// 種類の見出しの語の鍵（「k:」に種類の名）。
    pub kind_key: &'static str,
    pub band: Band,
    /// bead は状態の字・走行は段の字・ほかは None。
    pub state: Option<String>,
    pub id: String,
    /// 題（36 字で切る）。
    pub title: String,
    /// 出所の file（無ければ「出所なし」）。
    pub src: String,
    /// 中心が open の問いなら質問の頁への link を出す。
    pub answer: bool,
}

/// 電文の中心の行（列 0）。
pub fn center(doc: &AroundDoc) -> Option<&AroundRow> {
    doc.rows.iter().find(|r| r.col == 0)
}

/// 頭（中心の行から組む・中心の行が無ければ None）。
pub fn head(doc: &AroundDoc) -> Option<Head> {
    let row = center(doc)?;
    let band = band_of(row.node.kind);
    let state = matches!(band, Band::Beads | Band::Pipeline)
        .then(|| row.status.clone())
        .flatten();
    let alert = row.node.kind == NodeKind::Question && state.as_deref() == Some("open");
    let open = alert || is_open(state.as_deref());
    Some(Head {
        shape: format!(
            "shape {} big{}",
            band.class_name(),
            if open { "" } else { " fill" }
        ),
        alert,
        kind_key: kind_key(row.node.kind),
        band,
        state,
        id: row.node.id.clone(),
        title: title36(&row.node.title),
        src: row.node.file.clone().unwrap_or_else(|| NO_SRC.to_string()),
        answer: alert,
    })
}

/// 概要の 1 つの箱（見出しの語の鍵・class・字）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SumBox {
    pub key: &'static str,
    pub class: &'static str,
    pub text: &'static str,
}

/// 概要の 2 つの箱（電文に要約の欄がまだ無いので、どちらも要約なし）。
pub fn summary(_center: &AroundRow) -> [SumBox; 2] {
    SUMMARY_KEYS.map(|key| SumBox {
        key,
        class: SUMMARY_NONE,
        text: NO_SUMMARY,
    })
}

/// 質問の頁への link（問いの id を `%XX` にして残す・mode を URL に残す）。
pub fn answer_href(id: &str, mode: Mode) -> String {
    format!("?page=ask&id={}&mode={}", encode(id), mode.key())
}

#[cfg(target_arch = "wasm32")]
pub use dom::view;

/// 節点の block の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::{BLOCK, Head, answer_href, center, head, summary};
    use crate::mapview::band_chip;
    use crate::project::nodearound::{PageState, id_of, mode_of, source, state, unmeasured_reason};
    use crate::project::{ALERT_STYLE, NO_CONTENT, UNKNOWN, state_icon, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{h1, h2};

    /// 出所の印（見本の IC.file）。
    const FILE_ICON: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M6 3h8l4 4v14H6z"/><path d="M14 3v4h4"/></svg>"#;

    /// 節点の block（見つからないときは見出しと id・読めないときは測れていないと理由の 1 行）。
    pub fn view() -> AnyView {
        let src = source();
        let search = src.search;
        let id = search.with_untracked(|s| id_of(s));
        let Some(read) = src.read else {
            return not_found(id);
        };
        let mode = mode_of(search);
        let content = move || {
            let st = read.with(|(f, s)| state(f, *s));
            if let Some(reason) = unmeasured_reason(&st) {
                return unmeasured(reason);
            }
            match st {
                PageState::Doc(doc) => match (head(&doc), center(&doc)) {
                    (Some(h), Some(c)) => {
                        let boxes = summary(c)
                            .map(|b| {
                                view! {
                                    <section class=b.class><header>{h2(b.key)}</header><p>{b.text}</p></section>
                                }
                            })
                            .into_iter()
                            .collect_view();
                        view! { {head_view(h, mode)}<div class="two">{boxes}</div> }.into_any()
                    }
                    _ => unmeasured(NO_CONTENT),
                },
                _ => not_found(id.clone()),
            }
        };
        view! { <section class=BLOCK.class id=BLOCK.id>{content}</section> }.into_any()
    }

    /// 見つからない（見出しと、id が在れば id の字）。
    fn not_found(id: Option<String>) -> AnyView {
        let line = id.map(|id| {
            view! { <div class="empty">{state_icon(UNKNOWN)}<span data-t="">{id}</span></div> }
        });
        view! { {h1("not_found")}{line} }.into_any()
    }

    /// 頭（印・種類の見出し・帯の chip・状態・id・題・出所・質問の頁への link）。
    fn head_view(
        h: Head,
        mode: impl Fn() -> crate::frame::Mode + Send + Sync + 'static,
    ) -> AnyView {
        let style = if h.alert { ALERT_STYLE } else { "" };
        let state = h.state.map(|s| view! { <span>{s}</span> });
        let answer = h.answer.then(|| {
            let id = h.id.clone();
            view! {
                <a class="btn sm primary" href=move || answer_href(&id, mode())>{label("answer_here")}" ›"</a>
            }
        });
        view! {
            <div class="nhead">
                <span class=h.shape style=style aria-hidden="true"></span>
                <div style="min-width:0">
                    <div class="kind">{h1(h.kind_key)}{band_chip(h.band)}{state}</div>
                    <div class="cid">{h.id}</div>
                    <div class="t" data-t="">{h.title}</div>
                    <div class="srcline" data-term="src" tabindex="0"><span inner_html=FILE_ICON></span><span>{h.src}</span></div>
                    {answer}
                </div>
            </div>
        }
        .into_any()
    }
}
