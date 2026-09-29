//! 席の block の DOM（wasm の target のときだけ）。src/project/seat.rs が path の属性で module dom として読む。
use leptos::ev;
use leptos::prelude::*;
use tsuzuri_contract::board::Reading;

use super::{
    BLOCK, Band, GROUP_CARD, HistRow, LEGEND, Low, MORE, MORE_SRC, More, OROW, PATH, Seat, Sign,
    Span, Strip, Top, WindowRow, content, group_card, sample_svg, span_of, strip_svg, tick_age,
    tick_class, tick_mark, with_span,
};
use crate::view::Fetched;
use crate::widgets::hover::delegate;
use crate::account::heartbeat::{self, Dest, States};
use crate::project::{Body, UNKNOWN, body_view, fold, section, state_icon, unmeasured};
use crate::vocab::label;
use crate::widgets::help::{HelpCtx, hs, shows_internal, term};

/// 砂時計（限度の記号・見本の IC.hourglass）。
const HOURGLASS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linejoin="round" aria-hidden="true"><path d="M6 3h12M6 21h12"/><path d="M7 3c0 5 5 6 5 9s-5 4-5 9h10c0-5-5-6-5-9s5-4 5-9" /><path d="M9.5 19h5l-2.5-2.5z" fill="currentColor" stroke="none"/></svg>"#;

/// 今の URL の query（読めなければ空）。
fn search() -> String {
    window().location().search().unwrap_or_default()
}

/// 幅を選び、URL の query に残す（頁は読み直さない）。
fn pick(span: RwSignal<Span>, to: Span) {
    let url = with_span(&search(), to);
    if let Ok(history) = window().history() {
        let _ = history.replace_state_with_url(
            &web_sys::wasm_bindgen::JsValue::NULL,
            "",
            Some(&url),
        );
    }
    span.set(to);
}

pub fn view() -> AnyView {
    let fetched = crate::net::read(PATH);
    // 群の chip の card の電文（account board と同じ口・読み直しは変化の種類の表のまま）。
    let group_doc = crate::net::read(crate::account::PATH);
    let span = RwSignal::new(span_of(&search()));
    // 停止の切り替えの状態（読みの閉包の外・読み直しで組み直しても応答の字が残る）。
    let states: States = RwSignal::new(Default::default());
    let body = move || match fetched.with(content) {
        Body::Unmeasured(reason) => unmeasured(reason),
        Body::Empty(line) => body_view(Body::Empty(line)),
        Body::Filled(seat) => seat_view(seat, span, states, group_doc),
    };
    section(BLOCK, ().into_any(), body.into_any())
}

/// 測れていないの記号（その欄だけ）。
fn unknown() -> AnyView {
    state_icon(UNKNOWN)
}

fn text_or_unknown(r: Reading<String>) -> AnyView {
    match r {
        Reading::Known(t) => t.into_any(),
        Reading::Unknown => unknown(),
    }
}

fn sign_view(r: Reading<Sign>) -> AnyView {
    match r {
        Reading::Known(s) => {
            view! { <span class=s.class aria-hidden="true">{s.glyph}</span> }.into_any()
        }
        Reading::Unknown => unknown(),
    }
}

fn seat_view(
    seat: Seat,
    span: RwSignal<Span>,
    states: States,
    group_doc: ReadSignal<Fetched>,
) -> AnyView {
    let Seat {
        top,
        strips,
        low,
        band,
        hist,
        more,
    } = seat;
    view! {
        {band.map(band_view)}
        <div class=OROW>
            {top_view(top, states)}
            {strip_view(strips, span)}
        </div>
        {low_view(low, group_doc)}
        {hist_view(hist)}
        {more_view(more)}
    }
    .into_any()
}

fn band_view(band: Band) -> AnyView {
    let l2 = band
        .l2
        .map(|t| view! { <div class="l2"><span>{t}</span></div> });
    view! {
        <div class="sband" role="status">
            <div class="l1">{band.l1.join("・")}</div>
            {l2}
        </div>
    }
    .into_any()
}

fn big_icon(top: &Top) -> AnyView {
    let aria = label(top.key);
    let class = top.icon_class.clone();
    let v = top.state;
    if v == "limit" {
        view! { <span class=class role="img" aria-label=aria data-stv=v inner_html=HOURGLASS></span> }
            .into_any()
    } else {
        view! { <span class=class role="img" aria-label=aria data-stv=v><i></i></span> }
            .into_any()
    }
}

fn top_view(top: Top, states: States) -> AnyView {
    let icon = big_icon(&top);
    // 停止の切り替え（切り替えが無ければ button も段も出さない・送り先は席の口）。
    let hb_button = top.toggle.clone().map(|t| heartbeat::button(t, states));
    let hb_below = top
        .toggle
        .clone()
        .map(|t| heartbeat::below_to(Dest::Seat, t, states));
    let since = top.since.clone().map(|s| {
        view! { <span class="since num"><span class="small muted">{hs("since")}</span><b>{s}</b></span> }
    });
    // tick の欄（印・語・最後の tick からの経過・見本の tkhbHTML の順・経過は 1 秒の時計で書き直す）。
    let tk_class = tick_class(&top);
    let mark = tick_mark(&top);
    let word = match top.tick_word {
        Reading::Known(w) => Some(view! { <b>{w.as_str()}</b> }),
        Reading::Unknown => None,
    };
    let clock = crate::net::ticker();
    let age_top = top.clone();
    let age = move || tick_age(&age_top, clock.get()).map(|a| view! { <span class="num">{a}</span> });
    let hb_class = match top.heartbeat {
        Reading::Known("off") => "hb hb-off",
        _ => "hb",
    };
    let hb = match top.heartbeat {
        Reading::Known(w) => view! { <b>{w}</b> }.into_any(),
        Reading::Unknown => unknown(),
    };
    view! {
        <div>
            <div class="big">
                {icon}
                <span class=top.label_class.clone()>{label(top.key)}</span>
                {since}
            </div>
            <div class="tkrow">
                <span class="tkhb">
                    <span class=tk_class>{sign_view(mark)}{hs("tick_health")}{word}{age}</span>
                    <span class=hb_class>{hs("heartbeat")}{hb}</span>
                    {hb_button}
                </span>
            </div>
            {hb_below}
        </div>
    }
    .into_any()
}

fn strip_view(strips: Vec<Strip>, span: RwSignal<Span>) -> AnyView {
    let buttons = Span::ALL
        .into_iter()
        .map(|s| {
            let pressed = move || (span.get() == s).to_string();
            view! {
                <button type="button" data-span=s.key() aria-pressed=pressed on:click=move |_| pick(span, s)>
                    {s.key()}
                </button>
            }
        })
        .collect_view();
    let axis_strips = strips.clone();
    let axis = move || {
        let now = span.get();
        let ticks = axis_strips
            .iter()
            .find(|s| s.span == now)
            .map(|s| s.ticks.clone())
            .unwrap_or_default();
        ticks
            .into_iter()
            .map(|t| {
                let style = format!("left:{}%", t.left);
                view! { <span style=style>{t.label}</span> }
            })
            .collect_view()
    };
    let legend = LEGEND
        .into_iter()
        .map(|(name, key)| {
            view! {
                <span class="sleg-i"><span inner_html=sample_svg(name)></span><span>{label(key)}</span></span>
            }
        })
        .collect_view();
    let svg = move || {
        let now = span.get();
        let Some(strip) = strips.iter().find(|s| s.span == now) else {
            return unknown();
        };
        match (&strip.rects, &strip.marks) {
            (Reading::Known(r), Reading::Known(m)) => {
                view! { <div inner_html=strip_svg(r, m)></div> }.into_any()
            }
            (Reading::Known(r), Reading::Unknown) => view! {
                <div inner_html=strip_svg(r, &[])></div>
                <div class="small muted">{unknown()}</div>
            }
            .into_any(),
            (Reading::Unknown, _) => view! {
                <div class="small muted">{unknown()}<span>{label("st_unknown")}</span></div>
            }
            .into_any(),
        }
    };
    view! {
        <div class="strip">
            <div class="row small muted">
                {hs("history")}
                <span class="spanbar">{hs("span")}<span class="seg" role="group">{buttons}</span></span>
            </div>
            {svg}
            <div class="saxis" aria-hidden="true">{axis}</div>
            <div class="sleg" aria-label=label("history")>{legend}</div>
        </div>
    }
    .into_any()
}

fn window_view(row: WindowRow) -> AnyView {
    let name = match row.key {
        Some(k) => term(k, row.short.clone()),
        None => row.short.clone().into_any(),
    };
    let style = format!("width:{}%", row.width);
    view! {
        <div class=row.class>
            <span class="wl">{name}</span>
            <div class="meter">
                <div class="bar"><i class=row.bar_class style=style></i></div>
                <div class="v"><b class="num">{row.used}</b><span class="num">{format!("↻ {}", row.reset)}</span></div>
            </div>
        </div>
    }
    .into_any()
}

/// 群の chip（名が在れば指を置くと口 /api/account の群の枠の card を出す・名が分からなければ測れていないの記号だけ）。
fn group_chip(group: Reading<String>, account: ReadSignal<Fetched>) -> AnyView {
    let Reading::Known(name) = group else {
        return view! { <span class="pchip grp" data-t="">{unknown()}</span> }.into_any();
    };
    let hc = delegate();
    let key = name.clone();
    let over = move |ev: ev::MouseEvent| {
        hc.show(&ev, account.with_untracked(|a| group_card(a, &key)));
    };
    let out = move |ev: ev::MouseEvent| hc.leave(&ev);
    view! {
        <span class="pchip grp" data-t="" data-card=GROUP_CARD tabindex="0" on:mouseenter=over on:mouseleave=out>
            {name}
        </span>
    }
    .into_any()
}

fn low_view(low: Low, group_doc: ReadSignal<Fetched>) -> AnyView {
    let account = match low.account {
        Some(a) => a.into_any(),
        None => unknown(),
    };
    let usage = match low.usage {
        Reading::Known(rows) => rows.into_iter().map(window_view).collect_view().into_any(),
        Reading::Unknown => unknown(),
    };
    view! {
        <div class="olow">
            <div class="cur">
                <div>
                    <div class="small muted">{hs("reg_account")}</div>
                    <div class="acc mono">{account}</div>
                </div>
                <div class="gname">
                    <div class="small muted">{hs("group")}</div>
                    {group_chip(low.group, group_doc)}
                </div>
            </div>
            <div class="usage" aria-label=label("allowance")>
                <div class="small muted">{hs("allowance")}</div>
                {usage}
            </div>
        </div>
    }
    .into_any()
}

fn hist_row(row: HistRow) -> AnyView {
    let from = match row.from {
        Some(f) => f.into_any(),
        None => unknown(),
    };
    view! {
        <li>
            <span class="num">{row.at}</span>
            <span class="w"><span class="mono">{from}" → "{row.to}</span></span>
        </li>
    }
    .into_any()
}

fn hist_view(hist: Reading<Vec<HistRow>>) -> AnyView {
    let (count, rows) = match hist {
        Reading::Known(rows) => (
            view! { <span class="chip num">{rows.len()}</span> }.into_any(),
            rows.into_iter().map(hist_row).collect_view().into_any(),
        ),
        Reading::Unknown => (unknown(), ().into_any()),
    };
    let (open, toggle) = fold("seat:hist".to_string(), || false);
    view! {
        <details class="fold ahistd" prop:open=open on:toggle=toggle>
            <summary><span class="hd-t" data-term="acct_hist">{label("acct_hist")}</span>{count}</summary>
            <ul class="ahist">{rows}</ul>
        </details>
    }
    .into_any()
}

fn more_view(more: More) -> AnyView {
    let ctx = use_context::<HelpCtx>();
    let expert = move || ctx.is_some_and(|c| shows_internal(c.mode.get()));
    let (open, toggle) = fold("seat:more".to_string(), expert);
    // doctor の席の行の後に、器の移動の 4 つの欄の写しの行を同じ code の字で並べる。
    let doctor = more
        .doctor
        .into_iter()
        .chain(more.copied)
        .map(|l| view! { <div><code>{l}</code></div> })
        .collect_view();
    view! {
        <details class="gmore" prop:open=open on:toggle=toggle>
            <summary><span class="rm-t">{MORE}</span></summary>
            <div class="gm">
                <div class="gm1">
                    <span class="mono">{more.target}</span>
                    <span>
                        {label("current_account")}" "
                        <span class="mono">{text_or_unknown(more.current)}</span>" "
                        {sign_view(more.same)}
                    </span>
                </div>
                <div class="gm1 src">{MORE_SRC}</div>
                <div class="gm1 int xo">{doctor}</div>
            </div>
        </details>
    }
    .into_any()
}
