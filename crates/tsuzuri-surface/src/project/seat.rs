//! block「orchestrator と口座」（見本の `#orch` と index.html の renderSeat・lowHTML・bandHTML・
//! ui.js の stripSVG・stIcon・tkhbHTML・acct.js の meter・便 g-seat）。
//! 中身は口 /api/seat（契約の型の SeatCard）から読む。状態の判定は server が済ませていて、ここは写すだけ。
//! 電文の中の「まだ分からない」の欄はその欄だけ測れていないの記号にし、読めた欄は出す（要件 NFR2）。
//! 字と座標は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。
//! 群の chip の card（見本の pa:group）は、同じ server の口 /api/account の群の枠（GroupCard）から写す。
//! 状態の帯の猶予の内・移り先の無い断り・逼迫の行は、電文の器の移動の 4 つの欄
//! （move_to・grace_until・refused・pressure）を写すだけで判じない。猶予の残り秒だけは終わる時刻から
//! 描く card の at を引く。card は描く今で描き直す（`drawn`・at は今と電文の at の大きい方・行 c-abs-seat）。
//! 窓の行の閾値の線と印は、同じ口 /api/account の電文の caps（器の rules 行の写し）を窓の名で写すだけで判じない。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::{GroupMember, WindowCap};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{AccountMove, QuotaUsed, SeatCard, SeatSpan, SeatState, TickHealth};
use tsuzuri_contract::wire;

use super::{Body, NO_CONTENT, NOT_READ, UNKNOWN, state_class, state_key};
use crate::account::heartbeat::{Toggle, seat_toggle};
use crate::account::home::cap_of;
use crate::frame::{self, Block};
use crate::view::{Fetched, JST, clock, hhmm, jst};
use crate::vocab::label;
use crate::widgets::hover::Card;

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
pub const FOLDS: &[&str] = &[];

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

/// 状態の帯の字: 器の退避の合図の後の猶予の内（後に器の残り秒と字 秒 が続く・見本の bandHTML の移動中）。
pub const MOVING: &str = "移動中: 退避まで";

/// 状態の帯の 2 行目の字: 猶予の内（後に器が移す先の口座の名が続く）。
pub const GRACE_NEXT: &str = "猶予の後に器が /exit を送る →";

/// 状態の帯の 2 行目の字: 群の移り先の無い断り（猶予の外）。
pub const REFUSED_NEXT: &str = "器は移らない（移り先が出るまで今の口座のまま）";

/// 状態の帯の字: 群の今の口座の逼迫（後に口座と窓と使った割合と閾値が続く）。
pub const PRESSED: &str = "逼迫:";

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

    /// 目盛の間（秒・見本の SPANS の tick）。
    pub fn tick(self) -> u64 {
        match self {
            Span::H24 => 21_600,
            Span::H6 => 3_600,
            Span::H3 => 1_800,
        }
    }
}

/// 稼働の記録の下の目盛の 1 つ（見本の spanAxis の span）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tick {
    /// 左からの位置（百分率・小数 2 桁の字）。
    pub left: String,
    /// 時と分の字（`18:00`）。
    pub label: String,
}

/// 目盛の位置の上限（百分率・これを越える目盛は右端の字と重なるので落とす）。
const TICK_MAX: f64 = 92.0;

/// 目盛の数の上限（越えれば 1 つおきに残す・見本の spanTicks）。
const TICK_MANY: usize = 6;

/// 窓の目盛（窓の右端は `at`・日本時間で tick の倍数の時刻・字は日本時間の時と分・見本の spanTicks と spanAxis）。
/// tick はどれも 1 日の秒を割り切るので、日本の日の中の秒を切り上げれば日本時間の倍数の時刻になる。
pub fn span_ticks(at: EpochSecs, span: Span) -> Vec<Tick> {
    let start = at.saturating_sub(span.secs());
    let tick = span.tick();
    let mut times = Vec::new();
    let (_, secs) = jst(start);
    let mut t = start + (secs.div_ceil(tick) * tick - secs);
    while t <= at {
        times.push(t);
        t += tick;
    }
    if times.len() > TICK_MANY {
        times = times.into_iter().step_by(2).collect();
    }
    times
        .into_iter()
        .filter_map(|t| {
            let pct = (t - start) as f64 / span.secs() as f64 * 100.0;
            (pct <= TICK_MAX).then(|| Tick {
                left: format!("{pct:.2}"),
                label: hhmm(t),
            })
        })
        .collect()
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

/// 日本時間の時と分と空白と JST（`18:50 JST`）。
pub fn hm(at: EpochSecs) -> String {
    format!("{} {JST}", hhmm(at))
}

/// 履歴の時刻（`now` と日本の日が同じなら日本時間の時と分と JST・違う日は月日を前に付ける）。
pub fn hmd(at: EpochSecs, now: EpochSecs) -> String {
    if jst(at).0 == jst(now).0 {
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

/// 打刻の無い（absent）tick の印（字は空・見本の `.gi.unknown`）。
pub const BLANK: Sign = Sign {
    glyph: "",
    class: "gi unknown",
};

/// 真偽を印にする。
pub fn sign(good: bool) -> Sign {
    if good { OK } else { NG }
}

/// tick の語の印（healthy は良い・stale と unreadable は悪い・absent は空）。
pub fn tick_sign(word: TickHealth) -> Sign {
    match word {
        TickHealth::Healthy => OK,
        TickHealth::Stale | TickHealth::Unreadable => NG,
        TickHealth::Absent => BLANK,
    }
}

/// 上段の tick の欄の印（語が読めれば語の印・読めなければ健康の真偽の印）。
pub fn tick_mark(top: &Top) -> Reading<Sign> {
    match top.tick_word {
        Reading::Known(w) => Reading::Known(tick_sign(w)),
        Reading::Unknown => top.tick.clone(),
    }
}

/// 上段の tick の欄の class（語が読めれば `tk tk-<語>`・読めなければ健康の真偽から・どちらも無ければ `tk`）。
pub fn tick_class(top: &Top) -> &'static str {
    match (&top.tick_word, &top.tick) {
        (Reading::Known(TickHealth::Healthy), _) => "tk tk-healthy",
        (Reading::Known(TickHealth::Stale), _) => "tk tk-stale",
        (Reading::Known(TickHealth::Absent), _) => "tk tk-absent",
        (Reading::Known(TickHealth::Unreadable), _) => "tk tk-unreadable",
        (Reading::Unknown, Reading::Known(s)) if *s == OK => "tk tk-healthy",
        (Reading::Unknown, Reading::Known(_)) => "tk tk-stale",
        (Reading::Unknown, Reading::Unknown) => "tk",
    }
}

/// 経過の字（60 秒未満は s・60 分未満は m・48 時間未満は h と 2 桁の分・それ以上は d・端数は切り捨て）。
pub fn age_text(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else {
        super::ask::age(secs, 0)
    }
}

/// 最後の tick からの今の経過（今と電文の at の大きい方から tick_at を引く・語が absent か unreadable の時と
/// 時刻が無い時は None）。端末の時計が電文の at より遅れても、電文を読んだ時点の字より小さくしない。
pub fn tick_age(top: &Top, now: EpochSecs) -> Option<String> {
    if matches!(
        top.tick_word,
        Reading::Known(TickHealth::Absent | TickHealth::Unreadable)
    ) {
        return None;
    }
    top.tick_at
        .map(|t| age_text(now.max(top.at).saturating_sub(t)))
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
    /// tick の健康の印（器の seat tick status の真偽から・account board の印もこれを読む）。
    pub tick: Reading<Sign>,
    /// 器の doctor の席の行の tick の語。
    pub tick_word: Reading<TickHealth>,
    /// 合図の最後の判定の時刻。
    pub tick_at: Option<EpochSecs>,
    /// 電文の at（tick の経過の字の下限）。
    pub at: EpochSecs,
    /// heartbeat（on か off）。
    pub heartbeat: Reading<&'static str>,
    /// 停止の切り替え（席の名が在り heartbeat が読めるときだけ・行 g-seat-hb）。
    pub toggle: Option<Toggle>,
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

/// 1 つの幅の稼働の記録（区間の矩形と口座の移動の縦線の x と下の目盛）。
#[derive(Debug, Clone, PartialEq)]
pub struct Strip {
    pub span: Span,
    pub rects: Reading<Vec<Rect>>,
    pub marks: Reading<Vec<f64>>,
    /// 目盛（電文の at から出す・区間が読めなくても在る）。
    pub ticks: Vec<Tick>,
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
    /// 群の行が区画の行の写しか（行 g-park-view・区画は今の口座を 1 つに決めない）。
    pub park: bool,
    pub usage: Reading<Vec<WindowRow>>,
}

/// 状態の帯（1 行目の字の並びと、在れば次の移り先の 2 行目）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Band {
    pub l1: Vec<String>,
    pub l2: Option<String>,
}

/// block の中身。
#[derive(Debug, Clone, PartialEq)]
pub struct Seat {
    pub top: Top,
    /// 3 つの幅の稼働の記録（`Span::ALL` の順）。
    pub strips: Vec<Strip>,
    pub low: Low,
    pub band: Option<Band>,
}

impl Seat {
    /// 選んだ幅の稼働の記録。
    #[expect(clippy::expect_used, reason = "3 つの幅の記録を持つ")]
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

/// 最後の区間の to を at まで伸ばす（区間が at より後まで在れば変えない・「まだ分からない」と空の列はそのまま）。
pub fn reach(spans: &mut Reading<Vec<SeatSpan>>, at: EpochSecs) {
    if let Reading::Known(v) = spans
        && let Some(last) = v.last_mut()
    {
        last.to = last.to.max(at);
    }
}

/// 描く今の card（at は今と電文の at の大きい方・端末の時計が server より遅れても縮めない・最後の区間は at まで伸ばす）。
pub fn drawn(mut card: SeatCard, now: EpochSecs) -> SeatCard {
    card.at = card.at.max(now);
    reach(&mut card.spans, card.at);
    card
}

/// block の中身（口が読めなければ測れていない・描く今の card で組む）。
pub fn content(fetched: &Fetched, now: EpochSecs) -> Body<Seat> {
    match card(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(c) => Body::Filled(seat(&drawn(c, now))),
    }
}

/// 電文を中身に組む。
pub fn seat(card: &SeatCard) -> Seat {
    Seat {
        top: top(card),
        strips: Span::ALL.into_iter().map(|s| strip(card, s)).collect(),
        low: low(card),
        band: band(card),
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
        tick_word: card.tick.clone(),
        tick_at: card.tick_at,
        at: card.at,
        heartbeat: map(&card.heartbeat, |on| if *on { "on" } else { "off" }),
        toggle: seat_toggle(card),
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
        ticks: span_ticks(card.at, span),
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

/// 稼働の記録の凡例（記号の名と語の鍵・見本の stripLegend の順・起動と終了の縦線は描かないので置かない）。
pub const LEGEND: [(&str, &str); 7] = [
    ("run", "lg_run"),
    ("wait", "lg_wait"),
    ("silent", "lg_silent"),
    ("limit", "lg_limit"),
    ("unknown", "lg_unknown"),
    ("acct", "lg_acct"),
    ("now", "lg_now"),
];

/// 凡例の見本の横の幅（SVG の座標）。
const SAMPLE_W: f64 = 28.0;

/// 凡例の見本の高さ（SVG の座標）。
const SAMPLE_H: f64 = 12.0;

/// 凡例の 1 つの見本の SVG の字（状態は矩形・口座の移動と今は縦線・知らない名は中の無い svg）。
pub fn sample_svg(name: &str) -> String {
    let inner = match name {
        "run" | "wait" | "limit" | "silent" | "unknown" => {
            let height = if matches!(name, "wait" | "unknown") {
                SAMPLE_H * LOW
            } else {
                SAMPLE_H
            };
            format!(
                "<rect class=\"{}\" x=\"0\" y=\"{}\" width=\"{SAMPLE_W}\" height=\"{height}\"/>",
                strip_class(name),
                SAMPLE_H - height
            )
        }
        "acct" | "now" => {
            let x = SAMPLE_W / 2.0;
            format!(
                "<line class=\"mk mk-{name}\" x1=\"{x}\" x2=\"{x}\" y1=\"0\" y2=\"{SAMPLE_H}\"/>"
            )
        }
        _ => String::new(),
    };
    format!(
        "<svg class=\"ssample\" width=\"{SAMPLE_W}\" height=\"{SAMPLE_H}\" viewBox=\"0 0 {SAMPLE_W} {SAMPLE_H}\" aria-hidden=\"true\">{inner}</svg>"
    )
}

/// 群の行が区画の行の写しか（群の行が読めて欄 park が真・読めなければ偽）。
pub fn park_of(card: &SeatCard) -> bool {
    matches!(&card.group, Reading::Known(g) if g.park)
}

/// 下段の群の見出しの語の鍵（区画なら park・ほかは group）。
pub fn group_head(park: bool) -> &'static str {
    if park { "park" } else { "group" }
}

/// 下段。
pub fn low(card: &SeatCard) -> Low {
    Low {
        account: card.account.clone(),
        group: map(&card.group, |g| g.group.clone()),
        park: park_of(card),
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

/// 閾値の印の字（見本の lowHTML の th・後に値か `?` が続く）。
pub const THR_MARK: &str = "┆";

/// 窓の閾値の印（棒の中の線と行の右の字と読み上げの字・見本の lowHTML の wrow の th）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thr {
    /// 線の left の百分率（100 で止める・値が読めなければ None で線を引かない）。
    pub line: Option<u64>,
    /// 行の右の字（`┆85`・値が読めなければ `┆?`）。
    pub text: String,
    /// 読み上げの字（語の辞書の鍵 threshold の語と空白と値）。
    pub aria: String,
}

/// 口 /api/account の電文の窓ごとの閾値（器の rules 行の写し・電文が読めなければ None）。
pub fn seat_caps(account: &Fetched) -> Option<Vec<WindowCap>> {
    crate::account::doc(account).ok().map(|d| d.caps)
}

/// 窓の閾値の印（account board の cap_of と同じ読み・電文か窓の行か値が読めなければ線なしの `?`・
/// 値が 100 を越えれば線は 100 で止め、字は値のまま）。
pub fn thr(caps: Option<&[WindowCap]>, window: &str) -> Thr {
    let cap = caps.and_then(|c| cap_of(c, window));
    let value = cap.map_or_else(|| "?".to_string(), |c| c.to_string());
    Thr {
        line: cap.map(|c| c.min(100)),
        text: format!("{THR_MARK}{value}"),
        aria: format!("{} {value}", label("threshold")),
    }
}

/// 状態の帯（見本の bandHTML の順:限度・猶予の内か移動待ち・移り先の無い断り・逼迫・平時は None）。
/// 猶予の内と断りと逼迫は器の字の写しが読めた（Known(Some)）ときだけ出し、測れていない欄は行を出さない。
/// 群の行が区画の行の写しなら移動待ちの行と次の移り先の行は出さない（行 c-park-acct）。
pub fn band(card: &SeatCard) -> Option<Band> {
    let limit = card.state == SeatState::Limit;
    let mut l1 = Vec::new();
    if limit {
        l1.push(LIMIT_LINE.to_string());
    }
    let group = match &card.group {
        Reading::Known(g) => Some(g),
        Reading::Unknown => None,
    };
    let grace = match (&card.move_to, &card.grace_until) {
        (Reading::Known(Some(to)), Reading::Known(Some(until))) => {
            Some((to, until.saturating_sub(card.at)))
        }
        _ => None,
    };
    if let Some((_, left)) = grace {
        l1.push(format!("{MOVING} {left} 秒"));
    } else if let (Some(reg), Some(g)) = (&card.account, group)
        && !g.park
        && *reg != g.account
    {
        l1.push(format!("{MOVE_WAIT} {reg} → {}", g.account));
    }
    let refused = match card.refused {
        Reading::Known(Some(t)) => Some(t),
        _ => None,
    };
    if let Some(t) = refused {
        l1.push(format!("{} ◷ {}", label("no_target"), hmd(t, card.at)));
    }
    if !limit && let Reading::Known(Some(p)) = &card.pressure {
        let current = group.map_or(MEMBER_UNKNOWN, |g| g.account.as_str());
        l1.push(format!(
            "{PRESSED} {current} {} {}% ≥ {}",
            p.window, p.used, p.cap
        ));
    }
    let l2 = match (grace, refused) {
        (Some((to, _)), _) => Some(format!("{GRACE_NEXT} {to}")),
        (None, Some(_)) => Some(REFUSED_NEXT.to_string()),
        (None, None) => group
            .filter(|g| !g.park)
            .and_then(|g| g.next_account.as_ref())
            .map(|n| format!("{NEXT_TARGET} {n}")),
    };
    (!l1.is_empty()).then_some(Band { l1, l2 })
}

/// 区画の card の種の字。
pub const PARK_KIND: &str = "今の口座を持たない";

/// 区画の card の値の字。
pub const PARK_VALUE: &str = "口座は席ごと · 閾値に届いた席だけ器が移す";

/// 区画の card の出所の字。
pub const PARK_SRC: &str = "host.toml の区画の行・doctor";

/// 区画の chip の card（電文に依らない説明だけ・席ごとの口座は行の口座の列で見る）。
pub fn park_card(name: &str) -> Card {
    Card {
        title: format!("{} {name}", label("park")),
        kind: PARK_KIND.to_string(),
        value: PARK_VALUE.to_string(),
        src: PARK_SRC.to_string(),
        more: Vec::new(),
    }
}

/// 群の chip の card の鍵（見本の data-card の pa:group）。
pub const GROUP_CARD: &str = "pa:group";

/// 群の card の値の行の終わりの字（一致した席の数と群の席の数の後）。
pub const MATCHED: &str = "席が一致";

/// 席の行が無い project の印（hover の card は字だけなので見本の gi unknown の丸の代わり）。
pub const MEMBER_UNKNOWN: &str = "?";

/// account board の電文に群の枠が無いときの理由。
pub const GROUP_MISSING: &str =
    "account board の口の電文に、この名の群の枠が無い（群の枠の列が読めないか、名が合わない）";

/// 群の project の印（一致は ✓・移動待ちは !・席の行が無ければ ?）。
fn member_mark(m: &GroupMember) -> &'static str {
    match m.matches {
        Reading::Known(true) => OK.glyph,
        Reading::Known(false) => NG.glyph,
        Reading::Unknown => MEMBER_UNKNOWN,
    }
}

/// 群の測れていないの card（題は群の名・値は理由の字）。
fn group_unknown(name: &str, reason: &str) -> Card {
    Card {
        title: format!("{} {name}", label("group")),
        kind: label(state_key(UNKNOWN)),
        value: reason.to_string(),
        src: crate::account::PATH.to_string(),
        more: Vec::new(),
    }
}

/// 群の chip の card（見本の __tz_card の pa:group・口 /api/account の群の枠を写す・一致は電文の matches のまま・
/// 描く今の電文で組む）。
pub fn group_card(account: &Fetched, name: &str, now: EpochSecs) -> Card {
    let doc = match crate::account::doc(account) {
        Ok(d) => crate::account::drawn(d, now),
        Err(reason) => return group_unknown(name, reason),
    };
    let found = match &doc.groups {
        Reading::Known(gs) => gs.iter().find(|g| g.row.group == name),
        Reading::Unknown => None,
    };
    let Some(g) = found else {
        return group_unknown(name, GROUP_MISSING);
    };
    let mut kind = format!("{} {}", label("current_account"), g.row.account);
    if g.recorded
        && let Some(s) = g.since
    {
        kind.push_str(&format!(" ◷ {}", hmd(s, doc.at)));
    }
    let marks: String = g.members.iter().map(member_mark).collect();
    let matched = g
        .members
        .iter()
        .filter(|m| m.matches == Reading::Known(true))
        .count();
    let more = g
        .members
        .iter()
        .map(|m| {
            let seat = m.seat_account.clone().unwrap_or_else(|| label("seat_none"));
            let mut line = format!("{} {} {seat}", member_mark(m), m.project);
            if m.matches == Reading::Known(false) {
                line.push_str(&format!(" {}", label("seat_mismatch")));
            }
            line
        })
        .collect();
    Card {
        title: format!("{} {name}", label("group")),
        kind,
        value: format!("{marks} {matched} / {} {MATCHED}", g.members.len()),
        src: format!("groups/{name}.account ほか"),
        more,
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 席と口座の窓の稼働の記録の幅（24 時間だけ・判断の記録 ADR-27 決定 (1)(4)・行 g-win-parts）。
pub const WIN_SPAN: Span = Span::H24;

/// 席と口座の窓の中身（見出しの無い本文・稼働の記録は `WIN_SPAN` の幅だけ・行 g-win-parts）。
#[cfg(target_arch = "wasm32")]
pub fn inner() -> leptos::prelude::AnyView {
    dom::inner()
}

/// block の DOM（wasm の target のときだけ・中身は src/project_dom/seat.rs）。
#[cfg(target_arch = "wasm32")]
#[path = "../project_dom/seat.rs"]
mod dom;
