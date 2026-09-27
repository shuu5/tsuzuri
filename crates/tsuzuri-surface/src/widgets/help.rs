//! 「?」の注釈（見本の ui.js の qmark・tipContent・noteParts と同じ形）: 見出しの横の「?」を押すと語の説明を留める。
//! 本文は語彙の注釈の字を行に分ける: 1 行目 = 要点・2 行目から = 項（記号 + 本文）・行「▸」より後 = 詳しく。
//! 行の中の `{…}` は記号の見本、`` `…` `` は code にする。経験者には内部の名も出す。

use crate::frame::Mode;
use crate::vocab::{Term, vocab};

/// 行の中の 1 片。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    /// `` `…` ``。
    Code(String),
    /// `{…}`（記号の見本の名・例 `st:run`）。
    Sym(String),
}

/// 項の 1 行（先頭の記号と本文）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub sym: Vec<Inline>,
    pub text: Vec<Inline>,
}

/// 1 つの語の注釈。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub label: String,
    pub head: Vec<Inline>,
    pub items: Vec<Line>,
    pub more: Vec<Line>,
    /// 経験者だけに出す内部の名（行ごと）。
    pub internal: Vec<Vec<Inline>>,
}

/// 鍵の注釈（語彙に無ければ None）。
pub fn note(key: &str) -> Option<Note> {
    vocab().term(key).map(note_of)
}

/// 語から注釈を組む（見本の noteParts と同じ分け方）。
pub fn note_of(term: &Term) -> Note {
    let lines: Vec<&str> = term.note.split('\n').collect();
    let (main, more) = match lines.iter().position(|l| *l == "▸") {
        Some(i) => (&lines[..i], &lines[i + 1..]),
        None => (&lines[..], &[][..]),
    };
    let item_lines = |ls: &[&str]| -> Vec<Line> {
        ls.iter()
            .filter(|l| !l.is_empty())
            .map(|l| line(l))
            .collect()
    };
    Note {
        label: term.label.clone(),
        head: inline(main.first().copied().unwrap_or("")),
        items: item_lines(main.get(1..).unwrap_or(&[])),
        more: item_lines(more),
        internal: term
            .internal
            .split('\n')
            .filter(|l| !l.is_empty())
            .map(inline)
            .collect(),
    }
}

/// 項の 1 行: 先頭の `{…}` か 1〜3 字の語を記号に、残りを本文に（見本の itemHTML と同じ）。
pub fn line(s: &str) -> Line {
    let sym_end = if s.starts_with('{') {
        s.find('}').map(|i| i + 1)
    } else {
        s.find(char::is_whitespace)
            .filter(|&i| (1..=3).contains(&s[..i].chars().count()))
    };
    match sym_end {
        Some(end) if s[end..].starts_with(char::is_whitespace) && !s[end..].trim().is_empty() => {
            Line {
                sym: inline(&s[..end]),
                text: inline(s[end..].trim_start()),
            }
        }
        _ => Line {
            sym: Vec::new(),
            text: inline(s),
        },
    }
}

/// 行の中を片に分ける（`{…}` と `` `…` `` を見分ける）。
pub fn inline(s: &str) -> Vec<Inline> {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut rest = s;
    while let Some(c) = rest.chars().next() {
        let close = match c {
            '{' => rest[1..]
                .find(['}', '{'])
                .filter(|&i| rest[1 + i..].starts_with('}')),
            '`' => rest[1..].find('`'),
            _ => None,
        };
        match close {
            Some(i) if i > 0 => {
                if !text.is_empty() {
                    out.push(Inline::Text(std::mem::take(&mut text)));
                }
                let inner = rest[1..1 + i].to_string();
                out.push(if c == '{' {
                    Inline::Sym(inner)
                } else {
                    Inline::Code(inner)
                });
                rest = &rest[2 + i..];
            }
            _ => {
                text.push(c);
                rest = &rest[c.len_utf8()..];
            }
        }
    }
    if !text.is_empty() {
        out.push(Inline::Text(text));
    }
    out
}

/// 注釈の箱の class（出ているか・留めたか）。
pub fn tip_class(open: bool, pinned: bool) -> &'static str {
    match (open, pinned) {
        (false, _) => "tip",
        (true, false) => "tip on",
        (true, true) => "tip on pin",
    }
}

/// 注釈の箱の置き場（指の右 16 px・画面の右端からはみ出さない）。
pub fn place(x: f64, y: f64, viewport_width: f64) -> (f64, f64) {
    const WIDTH: f64 = 380.0;
    let left = (x + 16.0).min(viewport_width - WIDTH - 8.0).max(8.0);
    (left, y + 12.0)
}

/// 経験者は詳しくを開き、内部の名も出す。
pub fn shows_internal(mode: Mode) -> bool {
    mode == Mode::Expert
}

#[cfg(target_arch = "wasm32")]
pub use dom::{HelpCtx, TipLayer, h1, h2, hs, qmark};

#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;

    use super::{Inline, Line, note, place, shows_internal, tip_class};
    use crate::frame::Mode;
    use crate::project::{STATES, state_icon};
    use crate::vocab::label;

    /// 開いている注釈（語の鍵・置き場・留めたか）。
    #[derive(Debug, Clone, PartialEq)]
    pub struct Open {
        pub key: &'static str,
        pub x: f64,
        pub y: f64,
        pub pinned: bool,
    }

    /// 注釈と mode の状態（App が context に置く）。
    #[derive(Debug, Clone, Copy)]
    pub struct HelpCtx {
        pub open: RwSignal<Option<Open>>,
        pub mode: RwSignal<Mode>,
    }

    fn ctx() -> Option<HelpCtx> {
        use_context::<HelpCtx>()
    }

    fn at(ev: &ev::MouseEvent, key: &'static str, pinned: bool) -> Open {
        let width = window()
            .inner_width()
            .ok()
            .and_then(|w| w.as_f64())
            .unwrap_or(1280.0);
        let (x, y) = place(f64::from(ev.client_x()), f64::from(ev.client_y()), width);
        Open { key, x, y, pinned }
    }

    /// 「?」の印（初心者の mode だけ見える・押すと留める・指を置くと出す）。
    pub fn qmark(key: &'static str) -> AnyView {
        let aria = format!("{} の説明", label(key));
        let click = move |ev: ev::MouseEvent| {
            ev.stop_propagation();
            if let Some(c) = ctx() {
                let again = c
                    .open
                    .with_untracked(|o| o.as_ref().is_some_and(|o| o.key == key && o.pinned));
                c.open.set(if again {
                    None
                } else {
                    Some(at(&ev, key, true))
                });
            }
        };
        let enter = move |ev: ev::MouseEvent| {
            if let Some(c) = ctx()
                && !c
                    .open
                    .with_untracked(|o| o.as_ref().is_some_and(|o| o.pinned))
            {
                c.open.set(Some(at(&ev, key, false)));
            }
        };
        let leave = move |_: ev::MouseEvent| {
            if let Some(c) = ctx()
                && c.open
                    .with_untracked(|o| o.as_ref().is_some_and(|o| !o.pinned))
            {
                c.open.set(None);
            }
        };
        view! {
            <button type="button" class="q" data-term=key aria-label=aria on:click=click on:mouseenter=enter on:mouseleave=leave>"?"</button>
        }
        .into_any()
    }

    /// 見出しの語と「?」（見本の H / HS）。
    fn label_with_q(key: &'static str) -> impl IntoView {
        view! { <span class="hd-t" data-term=key>{label(key)}</span>{qmark(key)} }
    }

    pub fn h1(key: &'static str) -> AnyView {
        view! { <h1 class="hd" data-v=key>{label_with_q(key)}</h1> }.into_any()
    }

    pub fn h2(key: &'static str) -> AnyView {
        view! { <h2 class="hd" data-v=key>{label_with_q(key)}</h2> }.into_any()
    }

    /// 見出しの形をした短い札（列の名・数の名）。
    pub fn hs(key: &'static str) -> AnyView {
        view! { <span class="hd" data-v=key>{label_with_q(key)}</span> }.into_any()
    }

    fn inline_view(parts: &[Inline]) -> AnyView {
        parts
            .iter()
            .map(|p| match p {
                Inline::Text(t) => view! { <span>{t.clone()}</span> }.into_any(),
                Inline::Code(t) => view! { <code>{t.clone()}</code> }.into_any(),
                Inline::Sym(s) => sym_view(s),
            })
            .collect_view()
            .into_any()
    }

    /// 記号の見本（状態の記号は実物と同じ部品・ほかは名の字のまま）。
    fn sym_view(sym: &str) -> AnyView {
        if let Some(v) = sym.strip_prefix("st:")
            && let Some((state, _)) = STATES.iter().find(|(s, _)| *s == v)
        {
            return state_icon(state);
        }
        let shown = sym.rsplit(':').next().unwrap_or(sym).to_string();
        view! { <span>{shown}</span> }.into_any()
    }

    fn lines_view(lines: &[Line]) -> AnyView {
        let items = lines
            .iter()
            .map(|l| {
                let sym = (!l.sym.is_empty())
                    .then(|| view! { <span class="sy">{inline_view(&l.sym)}</span> });
                view! { <li>{sym}<span class="it">{inline_view(&l.text)}</span></li> }
            })
            .collect_view();
        view! { <ul class="nl">{items}</ul> }.into_any()
    }

    /// 注釈の箱（body に 1 つ・Esc と外を押すと閉じる）。
    #[component]
    pub fn TipLayer() -> impl IntoView {
        let Some(c) = ctx() else {
            return ().into_any();
        };
        let close = window_event_listener(ev::keydown, move |ev| {
            if ev.key() == "Escape" {
                c.open.set(None);
            }
        });
        let outside = window_event_listener(ev::click, move |_| {
            if c.open.with_untracked(Option::is_some) {
                c.open.set(None);
            }
        });
        on_cleanup(move || {
            close.remove();
            outside.remove();
        });
        let class = move || {
            c.open
                .with(|o| tip_class(o.is_some(), o.as_ref().is_some_and(|o| o.pinned)))
        };
        let style = move || {
            c.open.with(|o| {
                o.as_ref()
                    .map(|o| format!("left:{}px;top:{}px", o.x, o.y))
                    .unwrap_or_default()
            })
        };
        let content = move || {
            let open = c.open.get()?;
            let n = note(open.key)?;
            let expert = shows_internal(c.mode.get());
            let head = (!n.head.is_empty())
                .then(|| view! { <div class="n1">{inline_view(&n.head)}</div> });
            let items = (!n.items.is_empty()).then(|| lines_view(&n.items));
            let more = (!n.more.is_empty()).then(|| {
                view! {
                    <details class="more" open=expert>
                        <summary>{format!("{} ▸", label("p_more"))}</summary>
                        {lines_view(&n.more)}
                    </details>
                }
            });
            let internal = (expert && !n.internal.is_empty()).then(|| {
                let rows = n
                    .internal
                    .iter()
                    .map(|l| view! { <div>{inline_view(l)}</div> })
                    .collect_view();
                view! { <div class="int">{rows}</div> }
            });
            Some(view! {
                <div class="tl"><b>{n.label.clone()}</b></div>
                {head}
                {items}
                {more}
                {internal}
            })
        };
        view! {
            <div class=class role="tooltip" id="tip" style=style on:click=|ev: ev::MouseEvent| ev.stop_propagation()>
                {content}
            </div>
        }
        .into_any()
    }
}
