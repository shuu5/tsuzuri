//! block「台帳」（見本の `#ledger` と index.html の ledgerBlock・ledger.js の描き方の関数・便 g-ledger）:
//! 指標の段（上段の 4 数・主な指標の行・burndown・memo の段・「詳しく」・未反映の数）と台帳の一覧（便 g-min の中身を見本の class で描き直す）。
//! 指標は口 /api/metrics（本文は契約の型の Reading で包んだ LedgerStats）から読む。数え方と判定は中核の crate が済ませていて、ここは写すだけ
//! （数え直しと判定の分岐を持たない）。段の並びと段ごとの項は配置の表（`LAYOUT`）の値で持ち、DOM は表を上から順にたどる。
//! 一覧は epic の下に task・memo の順。一覧の口（/api/ledger）は問いの一覧（ask）と同じ口で、定数はこの module に 1 本だけ置く。
//! 字と座標は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use tsuzuri_contract::board::{LedgerJudge, Reading};
use tsuzuri_contract::stats::{DayCount, LedgerStats, OpenCounts, UnreflectedKind};
use tsuzuri_contract::wire;

use super::{Body, Item, LEDGER_UNREAD, NO_CONTENT, NOT_READ, item};
use crate::frame::Block;
use crate::view::{Fetched, Screen};

pub const BLOCK: Block = Block {
    id: "ledger",
    heading: "ledger_block",
    class: "panel",
};

/// 台帳の一覧の口（server の便 e-min・問いの一覧も読む）。
pub const PATH: &str = "/api/ledger";

/// 指標の段の口（便 g-parts）。
pub const METRICS_PATH: &str = "/api/metrics";

/// 指標の段の上段の語の鍵（見本の 4 数 = open task・memo・未反映・純減 24h）。
pub const METRICS: [&str; 4] = ["l_task", "l_memo", "l_unref", "l_net24"];

/// 指標の段の口が読めないときの理由。
pub const METRICS_REASON: &str =
    "台帳の指標（積みと速度）を組む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 口は読めたが、server が台帳を読めず指標が「まだ分からない」ときの理由。
pub const METRICS_UNKNOWN: &str = "server が台帳を読めないので、指標（積みと速度）はまだ分からない";

/// 測れて 0 件のときの 1 行。
pub const EMPTY: &str = "台帳に bead は無い";

/// epic の外の組の見出しの字（epic の項の代わり）。
pub const OUTSIDE: &str = "epic の外";

/// 値なしの字（年齢・lead）。
pub const NONE: &str = "―";

/// burndown の図の説明の 1 行（見本の lcap）。
pub const BURN_CAPTION: &str = "線 = open の task・棒 = 閉じた数 / 日";

/// burndown と sparkline の日の数。
pub const DAYS: usize = 14;

/// burndown の図の幅と高さ（見本の burndown(LS, 320, 72)）。
pub const BURN_W: f64 = 320.0;
pub const BURN_H: f64 = 72.0;

/// sparkline の幅と高さ（見本の spark14 の既定）。
pub const SPARK_W: f64 = 120.0;
pub const SPARK_H: f64 = 24.0;

/// 一覧の 1 組（epic の項・その下の項）。epic の外の組は `head` が None。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub head: Option<Item>,
    pub children: Vec<Item>,
}

/// 指標の段の 1 項（語の鍵を持つ）。作業中の数と最古の task の項は持たない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Task,
    Memo,
    Unref,
    Net24,
    Rate,
    Net7,
    Burn,
    MemoOpen,
    MemoWait,
    MemoPromo7,
    MemoAge,
    Ready,
    Blocked,
    Lead,
    Stale,
    Spark,
    Epics,
    Question,
    Epic,
    UnrefCount,
    List,
}

impl Part {
    /// 項の名（語の辞書の鍵）。
    pub const fn key(self) -> &'static str {
        match self {
            Part::Task => "l_task",
            Part::Memo => "l_memo",
            Part::Unref => "l_unref",
            Part::Net24 => "l_net24",
            Part::Rate => "l_rate",
            Part::Net7 => "l_net7",
            Part::Burn => "l_burn",
            Part::MemoOpen => "m_open",
            Part::MemoWait => "m_wait",
            Part::MemoPromo7 => "m_promo7",
            Part::MemoAge => "m_age",
            Part::Ready => "l_ready",
            Part::Blocked => "l_blocked",
            Part::Lead => "l_lead",
            Part::Stale => "l_stale",
            Part::Spark => "l_spark",
            Part::Epics => "l_epics",
            Part::Question => "l_question",
            Part::Epic => "l_epic",
            Part::UnrefCount => "unref",
            Part::List => "ledger_block",
        }
    }
}

/// block の中の段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// 上段（class l4）。
    Top,
    /// 主な指標の行（要件 FR13）。
    Main,
    /// burndown（class lmid）。
    Burn,
    /// memo の段（class mpro）。
    Memo,
    /// 「詳しく」（class gmore）。
    More,
    /// 未反映の数。
    Unref,
    /// 台帳の一覧。
    List,
}

impl Tier {
    pub const fn name(self) -> &'static str {
        match self {
            Tier::Top => "top",
            Tier::Main => "main",
            Tier::Burn => "burn",
            Tier::Memo => "memo",
            Tier::More => "more",
            Tier::Unref => "unref",
            Tier::List => "list",
        }
    }
}

/// 配置の表（段を上から順に・段ごとの項）。
pub const LAYOUT: [(Tier, &[Part]); 7] = [
    (
        Tier::Top,
        &[Part::Task, Part::Memo, Part::Unref, Part::Net24],
    ),
    (Tier::Main, &[Part::Rate, Part::Net7]),
    (Tier::Burn, &[Part::Burn]),
    (
        Tier::Memo,
        &[
            Part::MemoOpen,
            Part::MemoWait,
            Part::MemoPromo7,
            Part::MemoAge,
        ],
    ),
    (
        Tier::More,
        &[
            Part::Ready,
            Part::Blocked,
            Part::Lead,
            Part::Stale,
            Part::Spark,
            Part::Epics,
            Part::Question,
            Part::Epic,
        ],
    ),
    (Tier::Unref, &[Part::UnrefCount]),
    (Tier::List, &[Part::List]),
];

/// 配置の表の値。
pub fn layout() -> [(Tier, &'static [Part]); 7] {
    LAYOUT
}

/// 判定の 1 語の写し（記号・class・語の鍵）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Judge {
    pub judge: LedgerJudge,
    pub symbol: &'static str,
    pub class: &'static str,
    pub key: &'static str,
}

/// 判定の 5 値の表（1 か所・悪い順）。
pub const JUDGES: [Judge; 5] = [
    Judge {
        judge: LedgerJudge::Clogged,
        symbol: "!",
        class: "jdg j-bad",
        key: "j_bad",
    },
    Judge {
        judge: LedgerJudge::PilingUp,
        symbol: "↑",
        class: "jdg j-up",
        key: "j_up",
    },
    Judge {
        judge: LedgerJudge::Stalled,
        symbol: "→",
        class: "jdg j-stall",
        key: "j_stall",
    },
    Judge {
        judge: LedgerJudge::OnTrack,
        symbol: "✓",
        class: "jdg j-ok",
        key: "j_ok",
    },
    Judge {
        judge: LedgerJudge::NoLedger,
        symbol: "―",
        class: "jdg j-none",
        key: "j_none",
    },
];

/// 判定の写し（表から引く）。
pub fn judge(value: LedgerJudge) -> Judge {
    *JUDGES
        .iter()
        .find(|j| j.judge == value)
        .expect("判定の表は 5 値の全部を持つ")
}

/// 純減の矢印と数（見本の netHTML）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Net {
    pub arrow: &'static str,
    pub class: &'static str,
    /// 読み上げの語（純減・純増・横ばい）。
    pub word: &'static str,
    pub text: String,
}

/// 純減の値（今 − 過去・負が純減）を矢印と数にする。負は「−」と絶対値・正は「+」と値・0 は「0」。
pub fn net(value: i64) -> Net {
    let (arrow, class, word, text) = match value.signum() {
        -1 => (
            "↓",
            "net net-down",
            "純減",
            format!("−{}", value.unsigned_abs()),
        ),
        1 => ("↑", "net net-up", "純増", format!("+{value}")),
        _ => ("→", "net net-flat", "横ばい", "0".to_string()),
    };
    Net {
        arrow,
        class,
        word,
        text,
    }
}

/// 小数 1 桁の字（見本の toFixed(1)・ちょうど半分は大きい方へ）。
pub fn fixed1(value: f64) -> String {
    format!("{:.1}", (value * 10.0).round() / 10.0)
}

/// 日数の字（見本の days）: 値なしは「―」・1 日未満は時間の h・10 日未満は小数 1 桁の d・それ以上は整数の d。
pub fn age(days: Option<f64>) -> String {
    match days {
        None => NONE.to_string(),
        Some(d) if d < 1.0 => format!("{:.0}h", (d * 24.0).round()),
        Some(d) if d < 10.0 => format!("{}d", fixed1(d)),
        Some(d) => format!("{:.0}d", d.round()),
    }
}

/// 未反映の種類の名（1 か所の表）。
pub const UNREF_KINDS: [(UnreflectedKind, &str); 3] = [
    (UnreflectedKind::Memo, "memo"),
    (UnreflectedKind::Ruling, "ruling"),
    (UnreflectedKind::Request, "request"),
];

/// 未反映の種類の名（表から引く）。
pub fn kind_name(kind: UnreflectedKind) -> &'static str {
    UNREF_KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, n)| *n)
        .expect("種類の表は 3 つの全部を持つ")
}

/// 未反映の数（電文の数そのまま）と、分からない種類の名（測れていないの記号を添えて出す）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unref {
    pub count: u32,
    pub unknown: Vec<&'static str>,
}

/// burndown の棒の 1 本。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bar {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// burndown の図（棒 = 閉じた数 / 日・線 = その日の終わりの open の task）。
#[derive(Debug, Clone, PartialEq)]
pub struct Burn {
    pub bars: Vec<Bar>,
    pub line: Vec<(f64, f64)>,
}

/// burndown の座標（見本の burndown と同じ式）。
pub fn burndown(days: &[DayCount], w: f64, h: f64) -> Burn {
    let most_open = days.iter().map(|d| d.open).fold(1, u32::max);
    let most_closed = days.iter().map(|d| d.closed).fold(1, u32::max);
    let bw = (w - 8.0) / DAYS as f64;
    let bars = days
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let height = f64::from(d.closed) / f64::from(most_closed) * (h * 0.45);
            Bar {
                x: 4.0 + i as f64 * bw + 1.0,
                y: h - height,
                width: (bw - 2.0).max(1.0),
                height,
            }
        })
        .collect();
    let line = days
        .iter()
        .enumerate()
        .map(|(i, d)| {
            (
                4.0 + i as f64 * bw + bw / 2.0,
                4.0 + (1.0 - f64::from(d.open) / f64::from(most_open)) * (h * 0.5),
            )
        })
        .collect();
    Burn { bars, line }
}

/// sparkline の 2 本（created の点線・closed の実線）。
#[derive(Debug, Clone, PartialEq)]
pub struct Spark {
    pub created: Vec<(f64, f64)>,
    pub closed: Vec<(f64, f64)>,
}

/// sparkline の座標（見本の spark14 と同じ式）。
pub fn spark(days: &[DayCount], w: f64, h: f64) -> Spark {
    let most = days
        .iter()
        .map(|d| d.created.max(d.closed))
        .fold(1, u32::max);
    let at = |i: usize, v: u32| {
        (
            i as f64 / (DAYS - 1) as f64 * (w - 2.0) + 1.0,
            h - 2.0 - f64::from(v) / f64::from(most) * (h - 4.0),
        )
    };
    Spark {
        created: days
            .iter()
            .enumerate()
            .map(|(i, d)| at(i, d.created))
            .collect(),
        closed: days
            .iter()
            .enumerate()
            .map(|(i, d)| at(i, d.closed))
            .collect(),
    }
}

/// 線の点の字（`x,y x,y …`・小数 1 桁）。
pub fn points(line: &[(f64, f64)]) -> String {
    line.iter()
        .map(|(x, y)| format!("{},{}", fixed1(*x), fixed1(*y)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// burndown の svg の字（見本の burndown と同じ形）。
pub fn burn_svg(burn: &Burn) -> String {
    let bars: String = burn
        .bars
        .iter()
        .map(|b| {
            format!(
                r#"<rect class="lb-closed" x="{}" y="{}" width="{}" height="{}"/>"#,
                fixed1(b.x),
                fixed1(b.y),
                fixed1(b.width),
                fixed1(b.height)
            )
        })
        .collect();
    format!(
        r#"<svg class="lburn" viewBox="0 0 {BURN_W} {BURN_H}" preserveAspectRatio="none" role="img" aria-label="burndown 14 日">{bars}<polyline class="lb-open" fill="none" points="{}"/></svg>"#,
        points(&burn.line)
    )
}

/// sparkline の svg の字（見本の spark14 と同じ形）。
pub fn spark_svg(spark: &Spark) -> String {
    format!(
        r#"<svg class="lspark" viewBox="0 0 {SPARK_W} {SPARK_H}" width="{SPARK_W}" height="{SPARK_H}" role="img" aria-label="14 日の created と closed"><polyline class="ls-created" fill="none" points="{}"/><polyline class="ls-closed" fill="none" points="{}"/></svg>"#,
        points(&spark.created),
        points(&spark.closed)
    )
}

/// epic の進みの 1 行（題は台帳の一覧から id で引く・引けなければ None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpicBar {
    pub id: String,
    pub title: Option<String>,
    pub closed: u32,
    pub total: u32,
    /// 棒の幅の百分率（見本の Math.round(closed / total × 100)）。
    pub pct: u32,
}

/// memo の段の 4 数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoRow {
    pub open: u32,
    pub wait: u32,
    pub promo7: u32,
    pub age: String,
}

/// 指標の段の中身（電文の写し）。
#[derive(Debug, Clone, PartialEq)]
pub struct Metrics {
    pub judge: Judge,
    pub open: OpenCounts,
    pub unref: Unref,
    pub net24: Net,
    pub net7: Net,
    /// closed/日（小数 1 桁）。
    pub rate: String,
    pub burn: Burn,
    pub memo: MemoRow,
    pub ready: u32,
    pub blocked: u32,
    pub stale: u32,
    /// lead の中央値の字。
    pub lead: String,
    pub spark: Spark,
    pub epics: Vec<EpicBar>,
}

impl Metrics {
    /// 数の項の字（図・一覧の項は None）。
    pub fn text(&self, part: Part) -> Option<String> {
        let n = |v: u32| Some(v.to_string());
        match part {
            Part::Task => n(self.open.task),
            Part::Memo => n(self.open.memo),
            Part::Unref | Part::UnrefCount => n(self.unref.count),
            Part::Net24 => Some(self.net24.text.clone()),
            Part::Net7 => Some(self.net7.text.clone()),
            Part::Rate => Some(self.rate.clone()),
            Part::MemoOpen => n(self.memo.open),
            Part::MemoWait => n(self.memo.wait),
            Part::MemoPromo7 => n(self.memo.promo7),
            Part::MemoAge => Some(self.memo.age.clone()),
            Part::Ready => n(self.ready),
            Part::Blocked => n(self.blocked),
            Part::Lead => Some(self.lead.clone()),
            Part::Stale => n(self.stale),
            Part::Question => n(self.open.question),
            Part::Epic => n(self.open.epic),
            Part::Burn | Part::Spark | Part::Epics | Part::List => None,
        }
    }
}

/// 指標の口の本文を電文に読む（本文は読めた指標か「まだ分からない」・まだ読んでいない・読めない・
/// 台帳が読めない・電文が読めないは理由）。
pub fn stats(fetched: &Fetched) -> Result<LedgerStats, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(METRICS_REASON),
        Fetched::Body(text) => match wire::decode::<Reading<LedgerStats>>(text) {
            Ok(Reading::Known(s)) => Ok(s),
            Ok(Reading::Unknown) => Err(METRICS_UNKNOWN),
            Err(_) => Err(NO_CONTENT),
        },
    }
}

/// 指標の段の有無（測れていない・中身あり）。中身は `content` が組む。
pub fn metrics(fetched: &Fetched) -> Body<()> {
    match stats(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(_) => Body::Filled(()),
    }
}

/// 指標の段の中身（口が読めなければ測れていない・epic の題は画面の台帳の一覧から引く）。
pub fn content(fetched: &Fetched, screen: &Screen) -> Body<Metrics> {
    match stats(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(s) => Body::Filled(panel(&s, screen)),
    }
}

/// epic の題（台帳の一覧に在れば）。
fn epic_title(screen: &Screen, id: &str) -> Option<String> {
    match &screen.board {
        Reading::Known(b) => b
            .groups
            .iter()
            .filter_map(|g| g.epic.as_ref())
            .find(|e| e.id.as_str() == id)
            .map(|e| e.title.clone()),
        Reading::Unknown => None,
    }
}

/// 電文を中身に写す。
pub fn panel(s: &LedgerStats, screen: &Screen) -> Metrics {
    let epics = s
        .epics
        .iter()
        .map(|e| {
            let id = e.epic.to_string();
            let pct = if e.total == 0 {
                0.0
            } else {
                (f64::from(e.closed) / f64::from(e.total) * 100.0).round()
            };
            EpicBar {
                title: epic_title(screen, &id),
                id,
                closed: e.closed,
                total: e.total,
                pct: pct as u32,
            }
        })
        .collect();
    Metrics {
        judge: judge(s.judge),
        open: s.open,
        unref: Unref {
            count: s.unreflected,
            unknown: s
                .unreflected_unknown
                .iter()
                .map(|k| kind_name(*k))
                .collect(),
        },
        net24: net(s.net_drop_24h),
        net7: net(s.net_drop_7d),
        rate: fixed1(s.closed_per_day),
        burn: burndown(&s.days, BURN_W, BURN_H),
        memo: MemoRow {
            open: s.memo.open,
            wait: s.memo.awaiting_promotion,
            promo7: s.memo.closed_7d,
            age: age(s.memo.age_p50_days),
        },
        ready: s.ready,
        blocked: s.blocked,
        stale: s.stale,
        lead: age(s.lead.map(|l| l.p50)),
        spark: spark(&s.days, SPARK_W, SPARK_H),
        epics,
    }
}

/// 台帳の件数（epic も数える・測れていなければ Unknown）。
pub fn count(screen: &Screen) -> Reading<usize> {
    match &screen.board {
        Reading::Known(b) => Reading::Known(
            b.groups
                .iter()
                .map(|g| g.children.len() + usize::from(g.epic.is_some()))
                .sum(),
        ),
        Reading::Unknown => Reading::Unknown,
    }
}

/// 一覧の中身。
pub fn body(screen: &Screen) -> Body<Vec<Group>> {
    match &screen.board {
        Reading::Unknown => Body::Unmeasured(LEDGER_UNREAD),
        Reading::Known(b) if b.groups.is_empty() => Body::Empty(EMPTY),
        Reading::Known(b) => Body::Filled(
            b.groups
                .iter()
                .map(|g| Group {
                    head: g.epic.as_ref().map(item),
                    children: g.children.iter().map(item).collect(),
                })
                .collect(),
        ),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view(screen: leptos::prelude::RwSignal<Screen>) -> leptos::prelude::AnyView {
    dom::view(screen)
}

/// 台帳の block の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use tsuzuri_contract::board::Reading;

    use super::{
        BLOCK, BURN_CAPTION, EpicBar, Group, Judge, LAYOUT, METRICS_PATH, Metrics, NONE, Net,
        OUTSIDE, Part, Tier, body, burn_svg, content, count, spark_svg,
    };
    use crate::project::{Body, UNKNOWN, item_view, section, state_icon, unmeasured};
    use crate::view::Screen;
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, hs, shows_internal};

    pub fn view(screen: RwSignal<Screen>) -> AnyView {
        let fetched = crate::net::read(METRICS_PATH);
        let got = move || fetched.with(|f| screen.with(|s| content(f, s)));
        let extra = move || {
            let judge = match got() {
                Body::Filled(m) => judge_view(m.judge),
                Body::Unmeasured(_) | Body::Empty(_) => ().into_any(),
            };
            let chip = match screen.with(count) {
                Reading::Known(n) => view! { <span class="chip num">{n}</span> }.into_any(),
                Reading::Unknown => ().into_any(),
            };
            view! { {judge}{chip} }.into_any()
        };
        let body = move || match got() {
            Body::Filled(m) => LAYOUT
                .iter()
                .map(|(tier, parts)| tier_view(*tier, parts, &m, screen))
                .collect_view()
                .into_any(),
            Body::Unmeasured(reason) | Body::Empty(reason) => LAYOUT
                .iter()
                .map(|(tier, parts)| match tier {
                    Tier::Top => {
                        let boxes = parts
                            .iter()
                            .map(|p| {
                                view! {
                                    <div>
                                        <span class="small muted">{hs(p.key())}</span>
                                        <span class="big">{state_icon(UNKNOWN)}</span>
                                    </div>
                                }
                            })
                            .collect_view();
                        view! { <div class="l4">{boxes}</div>{unmeasured(reason)} }.into_any()
                    }
                    Tier::List => list_view(screen),
                    Tier::Main | Tier::Burn | Tier::Memo | Tier::More | Tier::Unref => {
                        ().into_any()
                    }
                })
                .collect_view()
                .into_any(),
        };
        section(BLOCK, extra.into_any(), body.into_any())
    }

    /// 1 つの段（表の項を順に）。
    fn tier_view(tier: Tier, parts: &[Part], m: &Metrics, screen: RwSignal<Screen>) -> AnyView {
        match tier {
            Tier::Top => {
                let boxes = parts.iter().map(|p| box_view(*p, m)).collect_view();
                view! { <div class="l4">{boxes}</div> }.into_any()
            }
            Tier::Main => {
                let items = parts.iter().map(|p| line_view(*p, m)).collect_view();
                view! { <div class="lcap num">{items}</div> }.into_any()
            }
            Tier::Burn => {
                let items = parts.iter().map(|p| line_view(*p, m)).collect_view();
                view! { <div class="lmid">{items}</div> }.into_any()
            }
            Tier::Memo => {
                let boxes = parts.iter().map(|p| box_view(*p, m)).collect_view();
                view! { <div class="mpro" aria-label=label("memo_promo")>{boxes}</div> }.into_any()
            }
            Tier::More => {
                let rows = parts
                    .iter()
                    .map(|p| view! { <div class="gm1 num">{line_view(*p, m)}</div> })
                    .collect_view();
                let expert =
                    move || use_context::<HelpCtx>().is_some_and(|c| shows_internal(c.mode.get()));
                view! {
                    <details class="gmore" open=expert>
                        <summary><span class="rm-t">{label("p_more")}</span>" "<span class="rm-a" aria-hidden="true">"▸"</span></summary>
                        <div class="gm">{rows}</div>
                    </details>
                }
                .into_any()
            }
            Tier::Unref => parts
                .iter()
                .map(|p| line_view(*p, m))
                .collect_view()
                .into_any(),
            Tier::List => parts
                .iter()
                .map(|_| list_view(screen))
                .collect_view()
                .into_any(),
        }
    }

    /// 上段と memo の段の箱（札と大きい数）。
    fn box_view(part: Part, m: &Metrics) -> AnyView {
        let value = match part {
            Part::Net24 => net_view(&m.net24),
            _ => {
                let text = m.text(part).unwrap_or_else(|| NONE.to_string());
                view! { <span class="num">{text}</span> }.into_any()
            }
        };
        let unknown =
            (part == Part::Unref && !m.unref.unknown.is_empty()).then(|| state_icon(UNKNOWN));
        let class = match part {
            Part::Unref if m.unref.count > 0 => "l4un on",
            Part::Unref => "l4un",
            Part::Net24 => "l4net",
            _ => "",
        };
        view! {
            <div class=class>
                <span class="small muted">{hs(part.key())}</span>
                <span class="big">{value}{unknown}</span>
            </div>
        }
        .into_any()
    }

    /// 札と字の 1 項（主な指標の行・「詳しく」・burndown・未反映の数）。
    fn line_view(part: Part, m: &Metrics) -> AnyView {
        match part {
            Part::Net7 => view! { <span>{hs(part.key())}" "{net_view(&m.net7)}</span> }.into_any(),
            Part::Burn => view! {
                <div class="lcap">{hs(part.key())}<span>{BURN_CAPTION}</span></div>
                <div inner_html=burn_svg(&m.burn)></div>
            }
            .into_any(),
            Part::Spark => view! {
                <span>{hs(part.key())}</span>
                <span inner_html=spark_svg(&m.spark)></span>
            }
            .into_any(),
            Part::Epics => {
                let rows = m.epics.iter().map(epic_view).collect_view();
                view! { <span>{hs(part.key())}</span><div class="lep">{rows}</div> }.into_any()
            }
            Part::UnrefCount => {
                let kinds = m
                    .unref
                    .unknown
                    .iter()
                    .map(|k| view! { <span class="chip num">{*k}" "{state_icon(UNKNOWN)}</span> })
                    .collect_view();
                view! {
                    <div class="lcap">
                        {hs(part.key())}
                        <span class="chip num">{m.unref.count}</span>
                        <span class="lchips">{kinds}</span>
                    </div>
                }
                .into_any()
            }
            _ => {
                let text = m.text(part).unwrap_or_else(|| NONE.to_string());
                view! { <span>{hs(part.key())}" "<span class="num">{text}</span></span> }.into_any()
            }
        }
    }

    /// 純減の矢印と数（見本の netHTML）。
    fn net_view(n: &Net) -> AnyView {
        let aria = format!("{} {}", n.word, n.text);
        view! {
            <span class=n.class aria-label=aria>
                <b>{n.arrow}</b>
                <span class="num">{n.text.clone()}</span>
            </span>
        }
        .into_any()
    }

    /// 判定の 1 語（見本の judgeHTML）。
    fn judge_view(j: Judge) -> AnyView {
        view! {
            <span class=j.class data-term=j.key tabindex="0">
                <b aria-hidden="true">{j.symbol}</b>
                <span>{label(j.key)}</span>
            </span>
        }
        .into_any()
    }

    /// epic の進みの 1 行（題が引けなければ id だけ）。
    fn epic_view(e: &EpicBar) -> AnyView {
        let name = e.title.clone().unwrap_or_else(|| e.id.clone());
        let ratio = format!("{} / {}", e.closed, e.total);
        view! {
            <div class="ep">
                <span><span class="nid">{e.id.clone()}</span>" "<span data-t="">{name}</span></span>
                <span class="bar" role="img" aria-label=ratio.clone()><i style=format!("width:{}%", e.pct)></i></span>
                <span class="meta num">{ratio}</span>
            </div>
        }
        .into_any()
    }

    /// 台帳の一覧（便 g-frame の中身）。
    fn list_view(screen: RwSignal<Screen>) -> AnyView {
        let list = move || match screen.with(body) {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => view! { <div class="empty"><span>{line}</span></div> }.into_any(),
            Body::Filled(groups) => {
                let rows = groups.iter().map(group_view).collect_view();
                view! { <ul class="items">{rows}</ul> }.into_any()
            }
        };
        view! { <div>{list}</div> }.into_any()
    }

    /// 1 組（epic の項と、その下の項を入れ子の一覧に）。
    fn group_view(group: &Group) -> AnyView {
        let head = match &group.head {
            Some(epic) => item_view(epic, None),
            None => view! {
                <li>
                    <span class="shape band-beads" aria-hidden="true"></span>
                    <span class="ttl muted">{OUTSIDE}</span>
                </li>
            }
            .into_any(),
        };
        let children = group
            .children
            .iter()
            .map(|c| item_view(c, None))
            .collect_view();
        view! {
            {head}
            <li class="nest"><ul class="items">{children}</ul></li>
        }
        .into_any()
    }
}
