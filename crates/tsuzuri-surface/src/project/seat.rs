//! block「orchestrator と口座」（見本の `#orch` と index.html の renderSeat・lowHTML・bandHTML・histHTML・
//! ui.js の stripSVG・stIcon・tkhbHTML・acct.js の meter・便 g-seat）。
//! 中身は口 /api/seat（契約の型の SeatCard）から読む。状態の判定は server が済ませていて、ここは写すだけ。
//! 電文の中の「まだ分からない」の欄はその欄だけ測れていないの記号にし、読めた欄は出す（要件 NFR2）。
//! 字と座標は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{AccountMove, QuotaUsed, SeatCard, SeatSpan, SeatState};
use tsuzuri_contract::wire;

use super::{Body, NO_CONTENT, NOT_READ, state_class, state_key};
use crate::frame::{self, Block};
use crate::view::{Fetched, clock};

pub const BLOCK: Block = Block {
    id: "orch",
    heading: "orch_acct",
    class: "panel seatcard",
};

/// 読みの口（便 g-parts）。
pub const PATH: &str = "/api/seat";

/// この file が字を持つ口の path（ほかの module の口を読む所は数えない・行 hs-derived）。
pub const PATHS: &[&str] = &[PATH];

/// この file の畳める段の開き閉じの鍵の形（行 hs-derived）。
pub const FOLDS: &[&str] = &["seat:hist", "seat:more"];

/// 口が読めないときの理由。
pub const REASON: &str =
    "orchestrator の状態と口座を読む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 状態の帯の字: 限度（見出しでない本文の字・語の辞書の鍵を使わない）。
/// 語の辞書の見出しの語と同じ字の定数は枠の歯が断るので、帯の字は続きの字まで含めて置く。
pub const LIMIT_LINE: &str = "限度で止まっている（↻ の時刻に再開する）";

/// 状態の帯の字: 登録の口座が群の今の口座と違う（後に 2 つの口座の名が続く）。
pub const MOVE_WAIT: &str = "移動待ち:";

/// 状態の帯の字: 群の次の移り先（後に口座の名が続く）。
pub const NEXT_TARGET: &str = "次の移り先:";

/// 「詳しく」の段の字。
pub const MORE: &str = "詳しく ▸";

/// 上段の class（見本の `.orow`・見本の ui.css にも stylesheet にも規則は無く、上段を見分ける名だけ）。
pub const OROW: &str = "orow";

/// 戻る時刻が無い窓の字。
pub const NO_RESET: &str = "―";

/// 稼働の記録の幅を残す URL の query の鍵。
pub const SPAN_KEY: &str = "span";

/// 稼働の記録の横の幅（SVG の座標）。
pub const WIDTH: f64 = 288.0;

/// 稼働の記録の高さ（SVG の座標・見本の home の 28）。
pub const HEIGHT: f64 = 28.0;

/// 待っている・測れていないの区間の高さの比（見本の STRIP_LOW）。
pub const LOW: f64 = 0.45;

/// 窓の名（語の鍵は window の字と同じ）。
pub const WINDOWS: [&str; 3] = ["five_hour", "seven_day", "seven_day_model"];

/// 窓の字と短い字の組（見本の acct.js の WSHORT・窓の名の欄に出す字）。
pub const SHORT: [(&str, &str); 3] = [
    ("five_hour", "5h"),
    ("seven_day", "7d"),
    ("seven_day_model", "model"),
];

/// 窓の短い字（知らない窓は電文の字のまま）。
pub fn short(window: &str) -> String {
    SHORT
        .into_iter()
        .find(|(w, _)| *w == window)
        .map_or_else(|| window.to_string(), |(_, s)| s.to_string())
}

/// 稼働の記録の幅（24h・6h・3h）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
    H24,
    H6,
    H3,
}

impl Span {
    pub const ALL: [Span; 3] = [Span::H24, Span::H6, Span::H3];

    /// URL と button の字。
    pub fn key(self) -> &'static str {
        match self {
            Span::H24 => "24h",
            Span::H6 => "6h",
            Span::H3 => "3h",
        }
    }

    /// 窓の長さ（秒）。
    pub fn secs(self) -> u64 {
        match self {
            Span::H24 => 86_400,
            Span::H6 => 21_600,
            Span::H3 => 10_800,
        }
    }
}

/// URL の query から幅（`span=6h` の形・無いか知らない値は 24h）。
pub fn span_of(search: &str) -> Span {
    frame::param(search, SPAN_KEY)
        .and_then(|v| Span::ALL.into_iter().find(|s| s.key() == v))
        .unwrap_or(Span::H24)
}

/// 幅を選んだ後の URL の query（`span=` に残す・ほかの値と順はそのまま）。
pub fn with_span(search: &str, span: Span) -> String {
    frame::with_param(search, SPAN_KEY, span.key())
}

/// 状態の値（見本の ST_V の字）。
pub fn state_value(state: SeatState) -> &'static str {
    match state {
        SeatState::Run => "run",
        SeatState::Wait => "wait",
        SeatState::Limit => "limit",
        SeatState::Silent => "silent",
        SeatState::Unknown => "unknown",
    }
}

/// 時と分と Z（`09:50Z`）。
pub fn hm(at: EpochSecs) -> String {
    format!("{}Z", &clock(at)[11..16])
}

/// 履歴の時刻（`now` と同じ日なら時と分と Z・違う日は月日を前に付ける）。
pub fn hmd(at: EpochSecs, now: EpochSecs) -> String {
    if at / 86_400 == now / 86_400 {
        hm(at)
    } else {
        format!("{} {}", &clock(at)[5..10], hm(at))
    }
}

/// 戻るまでの字（60 分未満は m・24 時間未満は h・それ以上は d・端数は切り捨て・戻る時刻が無ければ「―」）。
pub fn until(at: EpochSecs, resets_at: Option<EpochSecs>) -> String {
    match resets_at.map(|r| r.saturating_sub(at)) {
        None => NO_RESET.to_string(),
        Some(s) if s < 3_600 => format!("{}m", s / 60),
        Some(s) if s < 86_400 => format!("{}h", s / 3_600),
        Some(s) => format!("{}d", s / 86_400),
    }
}

/// 棒の class（100 以上は w100・80 を超えれば w80・ほかは無し）。
pub fn bar_class(used_pct: u8) -> &'static str {
    match used_pct {
        p if p >= 100 => "w100",
        p if p > 80 => "w80",
        _ => "",
    }
}

/// 印（字と class・見本の `.gi`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sign {
    pub glyph: &'static str,
    pub class: &'static str,
}

/// 良い（健康・一致）の印。
pub const OK: Sign = Sign {
    glyph: "✓",
    class: "gi ok",
};

/// 悪い（健康でない・不一致）の印。
pub const NG: Sign = Sign {
    glyph: "!",
    class: "gi ng",
};

/// 真偽を印にする。
pub fn sign(good: bool) -> Sign {
    if good { OK } else { NG }
}

/// 上段の左（状態の記号と語・から・合図の行）。
#[derive(Debug, Clone, PartialEq)]
pub struct Top {
    /// 状態の値（run・wait・limit・silent・unknown）。
    pub state: &'static str,
    /// 大きい記号の class（`st st-<値> lg`）。
    pub icon_class: String,
    /// 状態の語の鍵。
    pub key: &'static str,
    /// 状態の語の class（`state stlabel st-<値>`）。
    pub label_class: String,
    /// 「から」の時刻（時と分と Z・電文に無ければ None）。
    pub since: Option<String>,
    /// tick の健康の印。
    pub tick: Reading<Sign>,
    /// heartbeat（on か off）。
    pub heartbeat: Reading<&'static str>,
}

/// 稼働の記録の 1 つの矩形。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub class: &'static str,
}

/// 1 つの幅の稼働の記録（区間の矩形と口座の移動の縦線の x）。
#[derive(Debug, Clone, PartialEq)]
pub struct Strip {
    pub span: Span,
    pub rects: Reading<Vec<Rect>>,
    pub marks: Reading<Vec<f64>>,
}

/// 窓ごとの割合の 1 行（見本の `.wrow`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowRow {
    /// 窓の字（電文のまま）。
    pub window: String,
    /// 窓の名の欄に出す短い字（`5h`・知らない窓は電文の字のまま）。
    pub short: String,
    /// 窓の名の語の鍵（知らない窓は None で、字をそのまま出す）。
    pub key: Option<&'static str>,
    /// 行の class（数えない窓は薄く）。
    pub class: &'static str,
    /// 使った割合の字（`42%`）。
    pub used: String,
    /// 棒の長さ（百分率・100 で止める）。
    pub width: u8,
    pub bar_class: &'static str,
    /// 戻るまでの字。
    pub reset: String,
}

/// 下段（登録の口座・群の名・窓ごとの割合）。
#[derive(Debug, Clone, PartialEq)]
pub struct Low {
    pub account: Option<String>,
    pub group: Reading<String>,
    pub usage: Reading<Vec<WindowRow>>,
}

/// 状態の帯（1 行目の字の並びと、在れば次の移り先の 2 行目）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Band {
    pub l1: Vec<String>,
    pub l2: Option<String>,
}

/// 口座の履歴の 1 行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistRow {
    pub at: String,
    pub from: Option<String>,
    pub to: String,
}

/// 「詳しく」の中身（席の名・群の今の口座・登録の口座と同じかの印）。
#[derive(Debug, Clone, PartialEq)]
pub struct More {
    pub target: String,
    pub current: Reading<String>,
    pub same: Reading<Sign>,
}

/// block の中身。
#[derive(Debug, Clone, PartialEq)]
pub struct Seat {
    pub top: Top,
    /// 3 つの幅の稼働の記録（`Span::ALL` の順）。
    pub strips: Vec<Strip>,
    pub low: Low,
    pub band: Option<Band>,
    /// 口座の履歴（新しい順）。
    pub hist: Reading<Vec<HistRow>>,
    pub more: More,
}

impl Seat {
    /// 選んだ幅の稼働の記録。
    pub fn strip(&self, span: Span) -> &Strip {
        self.strips
            .iter()
            .find(|s| s.span == span)
            .expect("3 つの幅の記録を持つ")
    }
}

/// 口の本文を電文に読む（まだ読んでいない・読めない・電文が読めないは理由）。
/// 電文が読めない本文の理由は、中身の無い block と同じ（便 g-parts の歯が 3 値の理由を pin する）。
pub fn card(fetched: &Fetched) -> Result<SeatCard, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(REASON),
        Fetched::Body(text) => wire::decode::<SeatCard>(text).map_err(|_| NO_CONTENT),
    }
}

/// 中身の有無（測れていない・中身あり）。中身は `content` が組む。
pub fn body(fetched: &Fetched) -> Body<()> {
    match card(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(_) => Body::Filled(()),
    }
}

/// block の中身（口が読めなければ測れていない）。
pub fn content(fetched: &Fetched) -> Body<Seat> {
    match card(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(c) => Body::Filled(seat(&c)),
    }
}

/// 電文を中身に組む。
pub fn seat(card: &SeatCard) -> Seat {
    Seat {
        top: top(card),
        strips: Span::ALL.into_iter().map(|s| strip(card, s)).collect(),
        low: low(card),
        band: band(card),
        hist: hist(card),
        more: more(card),
    }
}

/// 上段の左。
pub fn top(card: &SeatCard) -> Top {
    let v = state_value(card.state);
    Top {
        state: v,
        icon_class: format!("{} lg", state_class(v)),
        key: state_key(v),
        label_class: format!("state stlabel st-{v}"),
        since: card.since.map(hm),
        tick: map(&card.tick_healthy, |h| sign(*h)),
        heartbeat: map(&card.heartbeat, |on| if *on { "on" } else { "off" }),
    }
}

fn map<T, U>(r: &Reading<T>, f: impl FnOnce(&T) -> U) -> Reading<U> {
    match r {
        Reading::Known(v) => Reading::Known(f(v)),
        Reading::Unknown => Reading::Unknown,
    }
}

/// 1 つの幅の稼働の記録（窓の右端は電文の at）。
pub fn strip(card: &SeatCard, span: Span) -> Strip {
    Strip {
        span,
        rects: map(&card.spans, |s| rects(s, card.at, span)),
        marks: map(&card.moves, |m| marks(m, card.at, span)),
    }
}

/// 窓の始まりからの秒を横の座標にする（秒 × 288 ÷ 窓の長さ）。
fn px(secs: u64, span: Span) -> f64 {
    secs as f64 * WIDTH / span.secs() as f64
}

/// 区間の矩形（窓の外にはみ出す区間は窓の端で切る・窓の外の区間は置かない・幅は 1 未満なら 1）。
pub fn rects(spans: &[SeatSpan], at: EpochSecs, span: Span) -> Vec<Rect> {
    let start = at.saturating_sub(span.secs());
    spans
        .iter()
        .filter_map(|s| {
            let from = s.from.max(start);
            let to = s.to.min(at);
            (to > from).then(|| {
                let v = state_value(s.state);
                let height = if matches!(s.state, SeatState::Wait | SeatState::Unknown) {
                    HEIGHT * LOW
                } else {
                    HEIGHT
                };
                Rect {
                    x: px(from - start, span),
                    y: HEIGHT - height,
                    width: px(to - from, span).max(1.0),
                    height,
                    class: strip_class(v),
                }
            })
        })
        .collect()
}

/// 区間の矩形の class（見本の STRIP_CLS の色の class・stylesheet に規則の無い `sg` は付けない）。
pub fn strip_class(v: &str) -> &'static str {
    match v {
        "run" => "sg-run",
        "wait" => "sg-wait",
        "limit" => "sg-limit",
        "silent" => "sg-silent",
        _ => "sg-unknown",
    }
}

/// 口座の移動の縦線の x（窓の中の移動だけ）。
pub fn marks(moves: &[AccountMove], at: EpochSecs, span: Span) -> Vec<f64> {
    let start = at.saturating_sub(span.secs());
    moves
        .iter()
        .filter(|m| m.at >= start && m.at <= at)
        .map(|m| px(m.at - start, span))
        .collect()
}

/// 稼働の記録の SVG の字（矩形・口座の移動の縦線・右端の今の線）。
pub fn strip_svg(rects: &[Rect], marks: &[f64]) -> String {
    let mut s = format!(
        "<svg viewBox=\"0 0 {WIDTH} {HEIGHT}\" preserveAspectRatio=\"none\" aria-hidden=\"true\">"
    );
    for r in rects {
        s.push_str(&format!(
            "<rect class=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
            r.class, r.x, r.y, r.width, r.height
        ));
    }
    for x in marks {
        s.push_str(&line("mk mk-acct", *x));
    }
    s.push_str(&line("mk mk-now", WIDTH - 1.0));
    s.push_str("</svg>");
    s
}

fn line(class: &str, x: f64) -> String {
    format!("<line class=\"{class}\" x1=\"{x}\" x2=\"{x}\" y1=\"0\" y2=\"{HEIGHT}\"/>")
}

/// 下段。
pub fn low(card: &SeatCard) -> Low {
    Low {
        account: card.account.clone(),
        group: map(&card.group, |g| g.group.clone()),
        usage: map(&card.usage, |u| {
            u.iter().map(|q| window_row(q, card.at)).collect()
        }),
    }
}

/// 窓ごとの割合の 1 行（数えない窓は薄く出す）。
pub fn window_row(q: &QuotaUsed, at: EpochSecs) -> WindowRow {
    WindowRow {
        window: q.window.clone(),
        short: short(&q.window),
        key: WINDOWS.into_iter().find(|w| *w == q.window),
        class: if q.counted { "wrow" } else { "wrow muted" },
        used: format!("{}%", q.used_pct),
        width: q.used_pct.min(100),
        bar_class: bar_class(q.used_pct),
        reset: until(at, q.resets_at),
    }
}

/// 状態の帯（限度のときと、登録の口座が群の今の口座と違うときだけ・平時は None）。
pub fn band(card: &SeatCard) -> Option<Band> {
    let mut l1 = Vec::new();
    if card.state == SeatState::Limit {
        l1.push(LIMIT_LINE.to_string());
    }
    let group = match &card.group {
        Reading::Known(g) => Some(g),
        Reading::Unknown => None,
    };
    if let (Some(reg), Some(g)) = (&card.account, group)
        && *reg != g.account
    {
        l1.push(format!("{MOVE_WAIT} {reg} → {}", g.account));
    }
    (!l1.is_empty()).then(|| Band {
        l1,
        l2: group
            .and_then(|g| g.next_account.as_ref())
            .map(|n| format!("{NEXT_TARGET} {n}")),
    })
}

/// 口座の履歴（新しい順・同じ時刻は電文の順）。
pub fn hist(card: &SeatCard) -> Reading<Vec<HistRow>> {
    map(&card.moves, |moves| {
        let mut sorted: Vec<&AccountMove> = moves.iter().collect();
        sorted.sort_by_key(|m| std::cmp::Reverse(m.at));
        sorted
            .into_iter()
            .map(|m| HistRow {
                at: hmd(m.at, card.at),
                from: m.from.clone(),
                to: m.to.clone(),
            })
            .collect()
    })
}

/// 「詳しく」の中身（登録の口座か群が分からなければ印は測れていない）。
pub fn more(card: &SeatCard) -> More {
    let same = match (&card.account, &card.group) {
        (Some(reg), Reading::Known(g)) => Reading::Known(sign(*reg == g.account)),
        _ => Reading::Unknown,
    };
    More {
        target: card.target.clone(),
        current: map(&card.group, |g| g.account.clone()),
        same,
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// block の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;
    use tsuzuri_contract::board::Reading;

    use super::{
        BLOCK, Band, HistRow, Low, MORE, More, OK, OROW, PATH, Seat, Sign, Span, Strip, Top,
        WindowRow, content, span_of, strip_svg, with_span,
    };
    use crate::project::{Body, UNKNOWN, body_view, fold, section, state_icon, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{hs, term};

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
        let (open, toggle) = fold("seat:more".to_string(), || false);
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
                </div>
            </details>
        }
        .into_any()
    }
}
