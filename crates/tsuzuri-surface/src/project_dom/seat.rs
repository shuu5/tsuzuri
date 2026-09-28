//! 席の block の DOM（wasm の target のときだけ）。src/project/seat.rs が path の属性で module dom として読む。
use leptos::prelude::*;
use tsuzuri_contract::board::Reading;

use super::{
    BLOCK, Band, HistRow, LEGEND, Low, MORE, MORE_SRC, More, OK, OROW, PATH, Seat, Sign, Span,
    Strip, Top, WindowRow, content, sample_svg, span_of, strip_svg, with_span,
};
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
    let span = RwSignal::new(span_of(&search()));
    let body = move || match fetched.with(content) {
        Body::Unmeasured(reason) => unmeasured(reason),
        Body::Empty(line) => body_view(Body::Empty(line)),
        Body::Filled(seat) => seat_view(seat, span),
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

fn seat_view(seat: Seat, span: RwSignal<Span>) -> AnyView {
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
            {top_view(top)}
            {strip_view(strips, span)}
        </div>
        {low_view(low)}
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

fn top_view(top: Top) -> AnyView {
    let icon = big_icon(&top);
    let since = top.since.clone().map(|s| {
        view! { <span class="since num"><span class="small muted">{hs("since")}</span><b>{s}</b></span> }
    });
    let tick_class = match top.tick {
        Reading::Known(s) if s == OK => "tk tk-healthy",
        Reading::Known(_) => "tk tk-stale",
        Reading::Unknown => "tk",
    };
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
                    <span class=tick_class>{sign_view(top.tick)}{hs("tick_health")}</span>
                    <span class=hb_class>{hs("heartbeat")}{hb}</span>
                </span>
            </div>
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

fn low_view(low: Low) -> AnyView {
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
                    <span class="pchip grp" data-t="">{text_or_unknown(low.group)}</span>
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
    let doctor = more
        .doctor
        .into_iter()
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
