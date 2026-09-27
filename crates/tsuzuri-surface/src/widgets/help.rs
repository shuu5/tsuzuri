//! 「?」の注釈（見本の ui.js の qmark・tipContent・noteParts と同じ形）: 見出しの横の「?」を押すと語の説明を留める。
//! 本文は語彙の注釈の字を行に分ける: 1 行目 = 要点・2 行目から = 項（記号 + 本文）・行「▸」より後 = 詳しく。
//! 行の中の `{…}` は記号の見本、`` `…` `` は code にする。経験者には内部の名も出す。
//! 行の全部が `{fig:名}` の 1 つなら、その行は手順と流れの図（widgets の fig）を描く。

use crate::frame::Mode;
use crate::mapview::band::Band;
use crate::project::seat::{Span, span_of};
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

/// 図の記号だけの行（`{fig:名}` の 1 片）の図の名（見本の itemHTML の figli と同じ見分け方）。
pub fn fig_of(l: &Line) -> Option<&str> {
    match (l.sym.as_slice(), l.text.as_slice()) {
        ([], [Inline::Sym(s)]) => s.strip_prefix("fig:").filter(|n| !n.is_empty()),
        _ => None,
    }
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

/// 幅の字（`24h` → `24 時間`・見本の spanKey の replace と同じ）。
pub fn span_words(span: Span) -> String {
    span.key().replace('h', " 時間")
}

/// 目盛の間の字（見本の SPANS の tickL と同じ）。
pub fn tick_words(span: Span) -> String {
    let secs = span.tick();
    if secs.is_multiple_of(3600) {
        format!("{} 時間", secs / 3600)
    } else {
        format!("{} 分", secs / 60)
    }
}

/// 注釈の置き換えの字（`{SPAN}` と `{TICK}`）を今の幅の字に替える（見本の tipContent と同じ）。
pub fn fill(s: &str, span: Span) -> String {
    s.replace("{SPAN}", &span_words(span))
        .replace("{TICK}", &tick_words(span))
}

/// 鍵の注釈を URL の query の幅で置き換えて組む（語彙に無ければ None）。
pub fn note_in(key: &str, search: &str) -> Option<Note> {
    let term = vocab().term(key)?;
    let span = span_of(search);
    Some(note_of(&Term {
        label: term.label.clone(),
        note: fill(&term.note, span),
        internal: fill(&term.internal, span),
    }))
}

/// 記号の見本の HTML の字（見本の symHTML と同じ並び・この面で描く名だけ Some・ほかは None）。
/// 値は閉じた集合と数字と英数字 1 字に限るので escape の要る字は入らない。
pub fn sym_html(sym: &str) -> Option<String> {
    let (head, rest) = sym.split_once(':')?;
    match head {
        "gi" => {
            let mark = match rest {
                "ok" => "✓",
                "ng" => "!",
                "unknown" => "",
                _ => return None,
            };
            Some(format!(
                "<span class=\"gi {rest} sm\" aria-hidden=\"true\">{mark}</span>"
            ))
        }
        "nxm" => {
            let (value, tag) = rest.split_once(':').unwrap_or((rest, ""));
            let tag_ok = tag.is_empty()
                || (tag.chars().count() == 1 && tag.chars().all(|c| c.is_ascii_alphanumeric()));
            if !matches!(value, "on" | "off" | "na") || !tag_ok {
                return None;
            }
            let tag = if tag.is_empty() { "c" } else { tag };
            Some(format!(
                "<span class=\"nxm {value}\" aria-hidden=\"true\">{tag}</span>"
            ))
        }
        "thr" => (!rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())).then(|| {
            format!("<span class=\"thrs\" aria-hidden=\"true\"><i></i>{rest}</span>")
        }),
        "tk" => {
            let (class, mark) = match rest {
                "healthy" => ("tk tk-healthy", "gi:ok"),
                "stale" => ("tk tk-stale", "gi:ng"),
                "absent" => ("tk", "gi:unknown"),
                "unreadable" => ("tk", "gi:ng"),
                _ => return None,
            };
            let mark = sym_html(mark)?;
            Some(format!(
                "<span class=\"tkhb smpl\"><span class=\"{class}\">{mark}<b>{rest}</b></span></span>"
            ))
        }
        "band" => Band::from_name(rest).map(|b| {
            format!(
                "<span class=\"shape {} fill\" aria-hidden=\"true\"></span>",
                b.class_name()
            )
        }),
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::{HelpCtx, TipLayer, h1, h2, hs, qmark, term};

#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;

    use super::{Inline, Line, fig_of, note_in, place, shows_internal, sym_html, tip_class};
    use crate::frame::Mode;
    use crate::project::{STATES, state_icon};
    use crate::vocab::label;
    use crate::widgets::fig;

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

    /// 押すと留める・同じ語をもう一度押すと外す。
    fn click(ev: &ev::MouseEvent, key: &'static str) {
        ev.stop_propagation();
        if let Some(c) = ctx() {
            let again = c
                .open
                .with_untracked(|o| o.as_ref().is_some_and(|o| o.key == key && o.pinned));
            c.open.set(if again {
                None
            } else {
                Some(at(ev, key, true))
            });
        }
    }

    /// 指を置くと出す（留めていなければ）。
    fn enter(ev: &ev::MouseEvent, key: &'static str) {
        if let Some(c) = ctx()
            && !c
                .open
                .with_untracked(|o| o.as_ref().is_some_and(|o| o.pinned))
        {
            c.open.set(Some(at(ev, key, false)));
        }
    }

    /// 指が離れると閉じる（留めていなければ）。
    fn leave() {
        if let Some(c) = ctx()
            && c.open
                .with_untracked(|o| o.as_ref().is_some_and(|o| !o.pinned))
        {
            c.open.set(None);
        }
    }

    /// 「?」の印（初心者の mode だけ見える・押すと留める・指を置くと出す）。
    pub fn qmark(key: &'static str) -> AnyView {
        let aria = format!("{} の説明", label(key));
        view! {
            <button
                type="button"
                class="q"
                data-term=key
                aria-label=aria
                on:click=move |ev: ev::MouseEvent| click(&ev, key)
                on:mouseenter=move |ev: ev::MouseEvent| enter(&ev, key)
                on:mouseleave=move |_: ev::MouseEvent| leave()
            >
                "?"
            </button>
        }
        .into_any()
    }

    /// 語の説明を持つ字（見本の `data-term` と `tabindex="0"` の字・開き方は「?」と同じ）。
    pub fn term(key: &'static str, text: String) -> AnyView {
        view! {
            <span
                data-term=key
                tabindex="0"
                on:click=move |ev: ev::MouseEvent| click(&ev, key)
                on:mouseenter=move |ev: ev::MouseEvent| enter(&ev, key)
                on:mouseleave=move |_: ev::MouseEvent| leave()
            >
                {text}
            </span>
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

    /// 記号の見本（状態の記号は実物と同じ部品・sym_html が描く名は実物と同じ class の字・ほかは名の字のまま）。
    fn sym_view(sym: &str) -> AnyView {
        if let Some(v) = sym.strip_prefix("st:")
            && let Some((state, _)) = STATES.iter().find(|(s, _)| *s == v)
        {
            return state_icon(state);
        }
        if let Some(html) = sym_html(sym) {
            return view! { <span inner_html=html></span> }.into_any();
        }
        let shown = sym.rsplit(':').next().unwrap_or(sym).to_string();
        view! { <span>{shown}</span> }.into_any()
    }

    fn lines_view(lines: &[Line]) -> AnyView {
        let items = lines
            .iter()
            .map(|l| {
                // 図の記号だけの行は図を描く（見本の itemHTML の figli・記号と本文の span は置かない）。
                if let Some(svg) = fig_of(l).and_then(fig::svg) {
                    return view! { <li class="figli" inner_html=svg></li> }.into_any();
                }
                let sym = (!l.sym.is_empty())
                    .then(|| view! { <span class="sy">{inline_view(&l.sym)}</span> });
                view! { <li>{sym}<span class="it">{inline_view(&l.text)}</span></li> }.into_any()
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
            // 出すときの幅で置き換える（出た後に幅を替えても出ている注釈は替えない・見本と同じ）。
            let n = note_in(open.key, &window().location().search().unwrap_or_default())?;
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
