//! 上の固定の帯（行 g-topbar・判断の記録 ADR-27 決定 (3)・見本 board-v2 の `#bar` と renderTodo と renderClock）:
//! 左に account board へ戻る口と project の名、真ん中に次の一手の pill と、やる事の 3 つの数（質問・止まった run・
//! 抜けの検査・1 以上の物だけ色）と、席からの最新の知らせの 1 行と件数、右に席と口座（状態・直近 3 時間の細い帯・
//! 口座と 5 時間と 7 日の窓の使った割合）と、抜けの検査の印と、時計と、設定（⚙ の中に表示の型と記号の見方と表示先）。
//! 印を押すと窓（widgets の modal）を開く。窓の名は `Win` で、中身は行 g-win-parts と行 g-ask-win が描く。
//! 読む口は今のまま（/api/next・/api/notices・/api/seat・/api/graph）。字と並びは純粋な関数にして host で試し、
//! DOM は wasm の target のときだけ組む。帯は 1 枚の画面への切り替えの行が頁の上に置き、今の header と tab を外した。
//! 窓は名の字（`Win::key`）を home の頁の URL の query の win に置いて開ける（`win_href`・消した頁への link の替わり）。
//! URL で窓を開いた後は、頁の URL から win と問いの id を外す（`settled_query`・読み直しても同じ窓は開かない・行 g-win-url）。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::graph::{InvariantCheck, Verdict};
use tsuzuri_contract::stats::{CheckResult, NextStep};

use crate::frame::{self, Mode, PageId};
use crate::project::seat::{Span, bar_class, drawn, rects, short, state_value, strip_svg};
use crate::project::{Body, UNKNOWN, next, notice, state_key};
use crate::view::{Fetched, hhmm};
use crate::vocab::label;

/// 帯の印が開く窓の名（質問・止まった run・知らせ・席と口座・抜けの検査・記号の見方・表示先・相談〔行 cs-bar〕）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Win {
    Ask,
    Stalled,
    Notices,
    Seat,
    Gaps,
    Legend,
    Dest,
    Consult,
}

impl Win {
    /// 全部の窓（宣言の順）。
    pub const ALL: [Win; 8] = [
        Win::Ask,
        Win::Stalled,
        Win::Notices,
        Win::Seat,
        Win::Gaps,
        Win::Legend,
        Win::Dest,
        Win::Consult,
    ];

    /// URL の query の win の値の名。
    pub fn key(self) -> &'static str {
        match self {
            Win::Ask => "ask",
            Win::Stalled => "stalled",
            Win::Notices => "notices",
            Win::Seat => "seat",
            Win::Gaps => "gaps",
            Win::Legend => "legend",
            Win::Dest => "dest",
            Win::Consult => "consult",
        }
    }

    /// URL の query の win の値の窓（無い値と知らない値は None）。
    pub fn from_query(search: &str) -> Option<Win> {
        let want = frame::param(search, WIN_PARAM)?;
        Win::ALL.into_iter().find(|w| w.key() == want)
    }
}

/// 窓を開く URL の query の名（home の頁を開いた時にその窓を開く・行 g-one-screen-a）。
pub const WIN_PARAM: &str = "win";

/// 窓を開いた home の頁の URL（mode を残す・消した質問の頁と抜けの検査の頁への link の替わり）。
pub fn win_href(win: Win, mode: Mode) -> String {
    format!(
        "{}&{WIN_PARAM}={}",
        frame::href(PageId::Home, mode),
        win.key()
    )
}

/// link の href が窓を開く link なら、その窓と問いの id（query の id・空と無い時は None）。窓を開く link でなければ None。
/// href は `?` から始まる query か、`?` を含む URL（`?` より前は見ない・井桁から後は見ない・行 g-one-screen-b）。
pub fn win_of_href(href: &str) -> Option<(Win, Option<String>)> {
    let (_, query) = href.split_once('?')?;
    let query = query.split('#').next().unwrap_or_default();
    let search = format!("?{query}");
    let win = Win::from_query(&search)?;
    Some((win, crate::project::ask::focus(&search)))
}

/// 窓を開いた後の頁の URL の query（行 g-win-url）: 窓を開く値 win を外し、home の頁なら問いの id（質問の窓の名指し）も
/// 外す（節点の頁の id は頁の中心なので残す・ほかの値と順はそのまま・残りが無ければ空の字）。
pub fn settled_query(search: &str, page: PageId) -> String {
    let drop =
        |k: &str| k == WIN_PARAM || (page == PageId::Home && k == crate::project::ask::FOCUS_KEY);
    let kept: Vec<&str> = search
        .trim_start_matches('?')
        .split('&')
        .filter(|kv| !kv.is_empty() && !drop(kv.split_once('=').map_or(kv, |(k, _)| k)))
        .collect();
    if kept.is_empty() {
        String::new()
    } else {
        format!("?{}", kept.join("&"))
    }
}

/// 帯の id（見本の `#bar`・頁に 1 つ）。
pub const BAR: &str = "bar";

/// 次の一手の pill の頭の印（見本の `▶`）。
pub const GO: &str = "▶";

/// 次の一手の pill の class（当たった一手・無い一手か測れていない）。
pub const PILL_GO: &str = "pill go";
pub const PILL_NONE: &str = "pill none";

/// 止まった run の数の語の鍵。
pub const STALL_KEY: &str = "bar_stall";

/// 抜けの検査の数と印の語の鍵。
pub const GAPS_KEY: &str = "bar_gaps";

/// 直近 3 時間の細い帯の語の鍵。
pub const RECENT_KEY: &str = "bar_recent";

/// 設定（⚙）の button の語の鍵（aria-label）。
pub const GEAR_KEY: &str = "bar_gear";

/// 席と口座の印の語の鍵（aria-label・窓の題）。
pub const SEAT_KEY: &str = "seat_acct";

/// 次の一手の種類が開く窓（質問と束の承認は質問の窓・止まっている走行は止まった run の窓・
/// 限度と移動と応答なしは席と口座の窓・発効待ちは抜けの検査の窓・なしは開かない）。
pub fn lead_win(kind: NextMove) -> Option<Win> {
    match kind {
        NextMove::Question | NextMove::BatchApproval => Some(Win::Ask),
        NextMove::StalledRun => Some(Win::Stalled),
        NextMove::LimitOrMove | NextMove::Unresponsive => Some(Win::Seat),
        NextMove::AwaitingEffect => Some(Win::Gaps),
        NextMove::Nothing => None,
    }
}

/// 次の一手の pill（class・字・押すと開く窓）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lead {
    pub class: &'static str,
    pub text: String,
    pub win: Option<Win>,
}

/// 種類の件数（当たれば件数・当たらなければ 0・判じなかったか電文に無ければ None）。
fn count_of(step: &NextStep, kind: NextMove) -> Option<u32> {
    step.checks
        .iter()
        .find(|c| c.kind == kind)
        .and_then(|c| match c.result {
            CheckResult::Hit => Some(c.count),
            CheckResult::Miss => Some(0),
            CheckResult::NotJudged => None,
        })
}

/// 次の一手の pill: 電文の lead の種類と件数（質問は「答える」・止まっている走行は「確かめる」の字）。
/// lead がなしなら次の一手なし、口が読めないかなしを判じなかったなら測れていない（どちらも窓を開かない）。
pub fn lead(fetched: &Fetched) -> Lead {
    let none = |key: &str| Lead {
        class: PILL_NONE,
        text: label(key),
        win: None,
    };
    let Ok(step) = next::step(fetched) else {
        return none(next::UNJUDGED_KEY);
    };
    if next::unjudged(&step) {
        return none(next::UNJUDGED_KEY);
    }
    if step.lead == NextMove::Nothing {
        return none(next::key(NextMove::Nothing));
    }
    let n = count_of(&step, step.lead).unwrap_or(0);
    let text = match step.lead {
        NextMove::Question => format!("{GO} {} {n} 件に答える", label("questions")),
        NextMove::StalledRun => format!("{GO} {} {n} 件を確かめる", label(STALL_KEY)),
        kind => format!("{GO} {} {n} 件", label(next::key(kind))),
    };
    Lead {
        class: PILL_GO,
        text,
        win: lead_win(step.lead),
    }
}

/// やる事の 3 つ（語の鍵・1 以上の時の色の class・開く窓・帯の並びの順）。
pub const TODOS: [(&str, &str, Win); 3] = [
    ("questions", "org", Win::Ask),
    (STALL_KEY, "red", Win::Stalled),
    (GAPS_KEY, "amb", Win::Gaps),
];

/// やる事の数の札（数・class・字・押すと開く窓）。数が分からなければ None で字は「?」。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Count {
    pub key: &'static str,
    pub n: Option<u32>,
    pub class: String,
    pub text: String,
    pub win: Win,
}

/// 抜けありの判定の数。
fn violated(checks: &[InvariantCheck]) -> u32 {
    let n = checks
        .iter()
        .filter(|c| c.verdict == Verdict::Violation)
        .count();
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// やる事の 3 つの数: 質問と止まった run は次の一手の電文の件数、抜けの検査は導出グラフの電文の抜けありの判定の数。
/// 1 以上の物だけ色の class（`chip org` など）で、0 と分からない物は `chip zero`。
pub fn counts(next_doc: &Fetched, graph: &Fetched) -> Vec<Count> {
    let step = next::step(next_doc).ok();
    let of = |kind| step.as_ref().and_then(|s| count_of(s, kind));
    let gaps = crate::project::gaps::checks(graph)
        .ok()
        .map(|c| violated(&c));
    let ns = [of(NextMove::Question), of(NextMove::StalledRun), gaps];
    TODOS
        .into_iter()
        .zip(ns)
        .map(|((key, hot, win), n)| Count {
            key,
            n,
            class: match n {
                Some(n) if n > 0 => format!("chip {hot}"),
                _ => "chip zero".to_string(),
            },
            text: format!(
                "{} {}",
                label(key),
                n.map_or_else(|| "?".to_string(), |n| n.to_string())
            ),
            win,
        })
        .collect()
}

/// 知らせの 1 行（時刻の字・題・この project の知らせの件数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoticeLine {
    pub when: String,
    pub title: String,
    pub count: usize,
}

/// 席からの最新の知らせ: この project の知らせのうち最新の 1 つと件数（電文はほかの project の分も持つが数えない）。
/// 口が読めないか自分の記録が読めなければ測れていない、自分の知らせが無ければ空。
pub fn notice_line(fetched: &Fetched) -> Body<NoticeLine> {
    let doc = match notice::doc(fetched) {
        Ok(doc) => doc,
        Err(reason) => return Body::Unmeasured(reason),
    };
    if doc.unread.contains(&doc.project) {
        return Body::Unmeasured(notice::OWN_UNREAD);
    }
    let own: Vec<_> = doc
        .latest
        .iter()
        .filter(|n| n.project == doc.project)
        .collect();
    match own.iter().max_by_key(|n| n.at) {
        Some(n) => Body::Filled(NoticeLine {
            when: hhmm(n.at),
            title: n.title.clone(),
            count: own.len(),
        }),
        None => Body::Empty(notice::NONE_LINE),
    }
}

/// 口座の窓の 1 つ（短い字・使った割合・棒の長さ・棒の class）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Use {
    pub short: String,
    pub used: Option<u8>,
    pub width: u8,
    pub class: &'static str,
}

/// 帯に出す口座の窓（5 時間と 7 日・この順）。
pub const USE_WINDOWS: [&str; 2] = ["five_hour", "seven_day"];

/// 細い帯の幅（直近 3 時間）。
pub const STRIP_SPAN: Span = Span::H3;

/// 席と口座の印の中身（状態・語の鍵・から の時刻・tick と heartbeat の字・細い帯の SVG・口座の名・2 つの窓）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatBar {
    pub state: &'static str,
    pub key: &'static str,
    pub since: Option<String>,
    pub tick: String,
    pub strip: String,
    pub account: Option<String>,
    pub usage: Vec<Use>,
}

fn yes_no(r: &Reading<bool>, yes: &'static str, no: &'static str) -> &'static str {
    match r {
        Reading::Known(true) => yes,
        Reading::Known(false) => no,
        Reading::Unknown => "?",
    }
}

/// 席と口座の印: 口 /api/seat の電文を描く今で組む（細い帯は直近 3 時間の区間・窓は 5 時間と 7 日）。
/// 口が読めなければ状態は測れていないで、時刻と口座と割合は無い。
pub fn seat_bar(fetched: &Fetched, now: EpochSecs) -> SeatBar {
    let Ok(card) = crate::project::seat::card(fetched) else {
        return SeatBar {
            state: UNKNOWN,
            key: state_key(UNKNOWN),
            since: None,
            tick: "tick ? · hb ?".to_string(),
            strip: strip_svg(&[], &[]),
            account: None,
            usage: USE_WINDOWS.iter().map(|w| use_of(w, None)).collect(),
        };
    };
    let c = drawn(card, now);
    let state = state_value(c.state);
    let strip = match &c.spans {
        Reading::Known(s) => strip_svg(&rects(s, c.at, STRIP_SPAN), &[]),
        Reading::Unknown => strip_svg(&[], &[]),
    };
    let used = |w: &str| match &c.usage {
        Reading::Known(u) => u.iter().find(|q| q.window == w).map(|q| q.used_pct),
        Reading::Unknown => None,
    };
    SeatBar {
        state,
        key: state_key(state),
        since: c.since.map(|t| format!("{}〜", hhmm(t))),
        tick: format!(
            "tick {} · hb {}",
            yes_no(&c.tick_healthy, "✓", "✗"),
            yes_no(&c.heartbeat, "on", "off")
        ),
        strip,
        account: c.account.clone(),
        usage: USE_WINDOWS.iter().map(|w| use_of(w, used(w))).collect(),
    }
}

/// 窓の 1 つ（棒の長さは 100 で止める・class は席の block の棒と同じ決め）。
fn use_of(window: &str, used: Option<u8>) -> Use {
    Use {
        short: short(window),
        used,
        width: used.map_or(0, |u| u.min(100)),
        class: used.map_or("", bar_class),
    }
}

/// 抜けの検査の印（class と字・見本の `.gp`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapsMark {
    pub class: &'static str,
    pub text: String,
}

/// 抜けの検査の印: 抜けありが在れば `gp f`、無くてまだ分からないが在れば `gp u`、どちらも無ければ `gp p`。
/// 字は「抜け <抜けありの数> · ?<まだ分からないの数>」で、口が読めなければ `gp u` の「抜け ?」。
pub fn gaps_mark(graph: &Fetched) -> GapsMark {
    let Ok(checks) = crate::project::gaps::checks(graph) else {
        return GapsMark {
            class: "gp u",
            text: format!("{} ?", label(GAPS_KEY)),
        };
    };
    let fail = violated(&checks);
    let unknown = checks
        .iter()
        .filter(|c| c.verdict == Verdict::Unknown)
        .count();
    let class = if fail > 0 {
        "gp f"
    } else if unknown > 0 {
        "gp u"
    } else {
        "gp p"
    };
    GapsMark {
        class,
        text: format!("{} {fail} · ?{unknown}", label(GAPS_KEY)),
    }
}

/// 設定（⚙）の中の口（語の鍵と開く窓・見本の `.gm-i` の順）。表示の型の切り替えは口の前に置く。
pub const GEAR: [(&str, Win); 2] = [("status", Win::Legend), ("stage_target", Win::Dest)];

#[cfg(target_arch = "wasm32")]
pub use dom::{Bar, view};

#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;
    use web_sys::wasm_bindgen::JsCast;

    use super::{
        BAR, Count, GEAR, GEAR_KEY, NoticeLine, RECENT_KEY, SEAT_KEY, SeatBar, Use, Win, counts,
        gaps_mark, lead, notice_line, seat_bar,
    };
    use crate::consultwin::CONSULT_KEY;
    use crate::frame::Mode;
    use crate::project::{Body, map, next, notice, seat, state_icon};
    use crate::view::hhmm;
    use crate::vocab::label;
    use crate::widgets::modal::{Hit, WinCtx, closes};

    /// 知らせの印（見本の IC_MEGA）。
    const MEGA: &str = r##"<svg class="ic" viewBox="0 0 16 16" aria-hidden="true"><path d="M2.5 6.3v3.4h2.2l5.3 3.1V3.2L4.7 6.3zM12 5.8a2.6 2.6 0 010 4.4M4.8 9.8l.9 3.2" fill="none" stroke="#ea580c" stroke-width="1.6" stroke-linejoin="round" stroke-linecap="round"/></svg>"##;

    /// 帯が受ける物（窓の積み・表示の型・project の名・account board へ戻る手・表示の型を選んだ時の手）。
    #[derive(Clone, Copy)]
    pub struct Bar {
        pub wins: WinCtx<Win>,
        pub mode: RwSignal<Mode>,
        pub name: Signal<String>,
        pub back: Callback<()>,
        pub pick: Callback<Mode>,
    }

    fn count_view(c: Count, wins: WinCtx<Win>) -> AnyView {
        let win = c.win;
        view! { <button type="button" class=c.class on:click=move |_| wins.open(win, false)>{c.text}</button> }
            .into_any()
    }

    fn notice_view(line: NoticeLine, wins: WinCtx<Win>) -> AnyView {
        view! {
            <button type="button" class="notice" aria-label=label("notice") on:click=move |_| wins.open(Win::Notices, false)>
                <span inner_html=MEGA></span>
                <time>{line.when}</time>
                <span class="nt">{line.title}</span>
                <span class="nc">{line.count}</span>
            </button>
        }
        .into_any()
    }

    fn use_view(u: Use) -> AnyView {
        let pct = u.used.map_or_else(|| "?".to_string(), |p| format!("{p}%"));
        view! {
            <span class="ub">
                <em>{u.short}</em>
                <i><s class=u.class style=format!("width:{}%", u.width)></s></i>
                <em class="up">{pct}</em>
            </span>
        }
        .into_any()
    }

    fn seat_view(s: SeatBar, wins: WinCtx<Win>) -> AnyView {
        let account = s.account.clone().unwrap_or_else(|| "?".to_string());
        let pct = |i: usize| {
            s.usage
                .get(i)
                .and_then(|u| u.used)
                .map_or_else(|| "?".to_string(), |p| format!("{p}%"))
        };
        let ashort = format!("{account} {}", pct(1));
        let mshort = format!("5h {} · 7d {}", pct(0), pct(1));
        let uses = s.usage.into_iter().map(use_view).collect_view();
        view! {
            <button type="button" class="seatb" aria-label=label(SEAT_KEY) on:click=move |_| wins.open(Win::Seat, false)>
                {state_icon(s.state)}
                <span class="sd">
                    <span class="s1"><b>{label(s.key)}</b>{s.since}</span>
                    <span class="s2">{s.tick}</span>
                </span>
                <span class="msp"><span class="s2">{label(RECENT_KEY)}</span><span class="spans" inner_html=s.strip></span></span>
                <span class="acct"><b>{account}</b><span class="ubs">{uses}</span></span>
                <span class="ashort">{ashort}</span>
                <span class="mshort">{mshort}</span>
            </button>
        }
        .into_any()
    }

    /// 設定（⚙）の button と中身（表示の型の 2 つと、記号の見方と表示先の窓の口）。外の click と取り消しの鍵で畳む。
    fn gear_view(bar: Bar) -> AnyView {
        let open = RwSignal::new(false);
        let outside = window_event_listener(ev::click, move |e| {
            let inside = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .and_then(|el| el.closest(".gearw").ok().flatten())
                .is_some();
            if !inside && closes(Hit::Outside) {
                open.set(false);
            }
        });
        let keys = window_event_listener(ev::keydown, move |e| {
            let key = e.key();
            let hit = Hit::Key {
                key: &key,
                composing: e.is_composing(),
            };
            if closes(hit) {
                open.set(false);
            }
        });
        on_cleanup(move || {
            outside.remove();
            keys.remove();
        });
        let modes = Mode::ALL
            .into_iter()
            .map(|m| {
                let checked = move || bar.mode.get() == m;
                view! {
                    <label class="gm-r">
                        <input type="radio" name="mode" prop:checked=checked on:change=move |_| bar.pick.run(m)/>
                        {label(m.key())}
                    </label>
                }
            })
            .collect_view();
        let items = GEAR
            .into_iter()
            .map(|(key, win)| {
                view! {
                    <button type="button" class="gm-i" on:click=move |_| { open.set(false); bar.wins.open(win, false); }>
                        {label(key)}
                    </button>
                }
            })
            .collect_view();
        view! {
            <div class="gearw">
                <button type="button" class="gearb" aria-label=label(GEAR_KEY) on:click=move |_| open.update(|o| *o = !*o)>"⚙"</button>
                <div class="gmenu" hidden=move || !open.get()>
                    <div class="gm-h">{label("mode")}</div>
                    {modes}
                    <div class="gm-sep"></div>
                    {items}
                </div>
            </div>
        }
        .into_any()
    }

    /// 帯（頁に 1 つ・見本の `<header id="bar">`）。
    pub fn view(bar: Bar) -> AnyView {
        let next_doc = crate::net::read(next::PATH);
        let notices = crate::net::read(notice::PATH);
        let seat_doc = crate::net::read(seat::PATH);
        let graph = crate::net::read(map::PATH);
        let now = crate::net::ticker();
        let wins = bar.wins;
        let pill = move || {
            let l = next_doc.with(lead);
            let win = l.win;
            view! { <button type="button" class=l.class on:click=move |_| if let Some(w) = win { wins.open(w, false) }>{l.text}</button> }
        };
        let chips = move || {
            let cs = next_doc.with(|n| graph.with(|g| counts(n, g)));
            cs.into_iter().map(|c| count_view(c, wins)).collect_view()
        };
        let line = move || match notices.with(notice_line) {
            Body::Filled(l) => Some(notice_view(l, wins)),
            Body::Empty(_) | Body::Unmeasured(_) => None,
        };
        let seat_part = move || seat_doc.with(|s| seat_view(seat_bar(s, now.get()), wins));
        let gp = move || {
            let g = graph.with(gaps_mark);
            view! { <button type="button" class=g.class on:click=move |_| wins.open(Win::Gaps, false)>{g.text}</button> }
        };
        view! {
            <header id=BAR>
                <div class="b-left">
                    <button type="button" class="ab" aria-label=label("acct_back") on:click=move |_| bar.back.run(())>
                        <span>"↑"</span><span class="abt">" "{label("acct_board")}</span>
                    </button>
                    <b class="pname">{move || bar.name.get()}</b>
                </div>
                <div class="todo">{pill}{chips}{line}</div>
                <div class="clock">
                    {seat_part}
                    {gp}
                    <button type="button" class="cslt" aria-label=label(CONSULT_KEY) on:click=move |_| wins.open(Win::Consult, false)>{label(CONSULT_KEY)}</button>
                    <span class="tm">{move || hhmm(now.get())}" "<small>"JST"</small></span>
                    {gear_view(bar)}
                </div>
            </header>
        }
        .into_any()
    }
}
