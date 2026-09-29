//! block「台帳」（見本の `#ledger` と index.html の ledgerBlock・ledger.js の描き方の関数・便 g-ledger）:
//! 指標の段（上段の 4 数・主な指標の行・burndown・memo の段・「詳しく」・未反映の数と一覧）と台帳の一覧（便 g-min の中身を見本の class で描き直す）。
//! 指標は口 /api/metrics（本文は契約の型の Reading で包んだ LedgerStats）から読む。数え方と判定は中核の crate が済ませていて、ここは写すだけ
//! （数え直しと判定の分岐を持たない）。段の並びと段ごとの項は配置の表（`LAYOUT`）の値で持ち、DOM は表を上から順にたどる。
//! 未反映の一覧は口 /api/unreflected（本文は契約の型の UnreflectedList）から読み、電文の行の順と数をそのまま写す（行 g-unref-panel）。
//! 一覧は epic の下に task・memo の順で、閉じた bead は出さない（全件は地図の方・行 g-ledger-home）。一覧の口（/api/ledger）は問いの一覧（ask）と同じ口で、定数はこの module に 1 本だけ置く。
//! 一覧の下の項は、板に札が在る bead なら板と同じ読み（block pipeline の `stages`）の段の記号と字を出す（行 c-ledger-stage）。
//! epic の進みの行の題と一覧の項の題に、グラフの口の電文から引いた節点の hover の card を付ける（行 g-card-adopt-c）。
//! 未反映の種類の見出しは語の辞書の鍵 `unref:` と種類の名の label で、account board もここの関数で引く（行 g-kind-label）。
//! memo と未反映の年齢は、電文の作った時刻から block を組む時の今までの日数（`days_since`・行 c-abs-time）。
//! 未反映の数は 3 種とも分からなければ数えない字 ― にし、1 種でも分かれば数に測れていないの印を添える（行 g-unref-dash）。
//! 字と座標は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use std::collections::BTreeMap;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{LedgerJudge, Reading};
use tsuzuri_contract::ledger::LedgerRow;
use tsuzuri_contract::stats::{
    DayCount, LedgerStats, OpenCounts, UnreflectedKind, UnreflectedList, UnreflectedRow,
};
use tsuzuri_contract::wire;

use super::seat::hm;
use super::{Body, Item, LEDGER_UNREAD, NO_CONTENT, NOT_READ, Staged, item, staged_item};
use crate::frame::Block;
use crate::view::{Fetched, Screen, clock};
use crate::vocab::label;
use crate::widgets::hover::Card;
use crate::widgets::nodecard::card_of;

pub const BLOCK: Block = Block {
    id: "ledger",
    heading: "ledger_block",
    class: "panel",
};

/// 台帳の一覧の口（server の便 e-min・問いの一覧も読む）。
pub const PATH: &str = "/api/ledger";

/// 指標の段の口（便 g-parts）。
pub const METRICS_PATH: &str = "/api/metrics";

/// 未反映の一覧の口（本文は契約の型の UnreflectedList・便 e-view）。
pub const UNREF_PATH: &str = "/api/unreflected";

/// この file が字を持つ口の path（ほかの module の口を読む所は数えない・行 hs-derived）。
pub const PATHS: &[&str] = &[PATH, METRICS_PATH, UNREF_PATH];

/// この file の畳める段の開き閉じの鍵の形（行 hs-derived）。
pub const FOLDS: &[&str] = &["ledger:more", "ledger:unref"];

/// 指標の段の上段の語の鍵（見本の 4 数 = open task・memo・未反映・純減 24h）。
pub const METRICS: [&str; 4] = ["l_task", "l_memo", "l_unref", "l_net24"];

/// 指標の段の口が読めないときの理由。
pub const METRICS_REASON: &str =
    "台帳の指標（積みと速度）を組む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 口は読めたが、server が台帳を読めず指標が「まだ分からない」ときの理由。
pub const METRICS_UNKNOWN: &str = "server が台帳を読めないので、指標（積みと速度）はまだ分からない";

/// 測れて 0 件のときの 1 行。
pub const EMPTY: &str = "台帳に bead は無い";

/// 一覧に出さない状態の字（bd の語・行 g-ledger-home）。
pub const CLOSED: &str = "closed";

/// 台帳に行は在るが、閉じていない行が 1 つも無いときの 1 行（EMPTY と分ける）。
pub const NO_OPEN: &str = "閉じていない bead は無い";

/// 一覧に出す行か（状態が CLOSED の行は出さない・全件は地図の方で見る）。
pub fn listed(row: &LedgerRow) -> bool {
    row.status != CLOSED
}

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

/// 時刻 at から今 now までの日数（今より後の時刻は 0・memo と未反映の年齢は面の今から引く・行 c-abs-time）。
pub fn days_since(at: EpochSecs, now: EpochSecs) -> f64 {
    now.saturating_sub(at) as f64 / 86_400.0
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

/// 未反映の種類の見出しの語の鍵の接頭（鍵は接頭と kind_name の字）。
pub const UNREF_KIND_KEY: &str = "unref:";

/// 未反映の種類の見出し（語の辞書の label・見本の unrefBreak と unrefHTML の字）。
pub fn kind_label(kind: UnreflectedKind) -> String {
    name_label(kind_name(kind))
}

/// 未反映の種類の名（kind_name の字）の見出し（語の辞書の label）。
pub fn name_label(name: &str) -> String {
    label(&format!("{UNREF_KIND_KEY}{name}"))
}

/// 未反映の数（電文の数そのまま）と、分からない種類の名（測れていないの記号を添えて出す）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unref {
    pub count: u32,
    pub unknown: Vec<&'static str>,
}

impl Unref {
    /// 数の字（3 種とも分からなければ数えない字 NONE・ほかは数・行 g-unref-dash）。
    pub fn text(&self) -> String {
        if self.unknown.len() < UnreflectedKind::ALL.len() {
            self.count.to_string()
        } else {
            NONE.to_string()
        }
    }

    /// 数に測れていないの印を添えるか（分からない種類が 1 つ以上で、3 種の全部ではない時だけ）。
    pub fn partial(&self) -> bool {
        !self.unknown.is_empty() && self.unknown.len() < UnreflectedKind::ALL.len()
    }
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

/// epic の進みの行の節点の card（電文に在る epic の id だけ・グラフの口が読めなければ空・行 g-card-adopt-c）。
pub fn epic_cards(epics: &[EpicBar], graph: &Fetched) -> BTreeMap<String, Card> {
    cards_of(epics.iter().map(|e| e.id.as_str()), graph)
}

/// 一覧の組の項の節点の card（組ごとに epic の項と下の項・電文に在る id だけ・行 g-card-adopt-c）。
pub fn group_cards(groups: &[Group], graph: &Fetched) -> BTreeMap<String, Card> {
    cards_of(
        groups
            .iter()
            .flat_map(|g| g.head.iter().chain(&g.children).map(|i| i.id.as_str())),
        graph,
    )
}

/// id の列のうち電文の節点に在るものの card（id の字の鍵）。
fn cards_of<'a>(ids: impl Iterator<Item = &'a str>, graph: &Fetched) -> BTreeMap<String, Card> {
    let Ok(doc) = super::map::doc(graph) else {
        return BTreeMap::new();
    };
    ids.filter_map(|id| card_of(&doc, id).map(|c| (id.to_string(), c)))
        .collect()
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
    /// burndown の図の hover の card（行 g-ledger-card）。
    pub burn_card: Card,
}

impl Metrics {
    /// 数の項の字（図・一覧の項は None）。
    pub fn text(&self, part: Part) -> Option<String> {
        let n = |v: u32| Some(v.to_string());
        match part {
            Part::Task => n(self.open.task),
            Part::Memo => n(self.open.memo),
            Part::Unref | Part::UnrefCount => Some(self.unref.text()),
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

/// 指標の段の中身（口が読めなければ測れていない・epic の題は画面の台帳の一覧から引く・memo の年齢は今 now から数える）。
pub fn content(fetched: &Fetched, screen: &Screen, now: EpochSecs) -> Body<Metrics> {
    match stats(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(s) => Body::Filled(panel(&s, screen, now)),
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

/// 電文を中身に写す（memo の年齢は作った時刻の中央値から今 now までの日数）。
pub fn panel(s: &LedgerStats, screen: &Screen, now: EpochSecs) -> Metrics {
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
            age: age(s.memo.created_p50.map(|c| days_since(c, now))),
        },
        ready: s.ready,
        blocked: s.blocked,
        stale: s.stale,
        lead: age(s.lead.map(|l| l.p50)),
        spark: spark(&s.days, SPARK_W, SPARK_H),
        epics,
        burn_card: burn_card(s),
    }
}

/// burndown の数の出所（台帳の読み）。
pub const BURN_SRC: &str = "bd list --all の task";

/// burndown の図の card（見本の ledgerCard の burn の枝）: 題・純減 24h と 7d と closed/日・
/// 14 日の始めと終わりの open・出所と時点。詳しくは最後の日から 1 日おきに選んだ日を古い順に。
pub fn burn_card(s: &LedgerStats) -> Card {
    let n24 = net(s.net_drop_24h);
    let n7 = net(s.net_drop_7d);
    let kind = format!(
        "{}{} 24h · {}{} 7d · {} {}",
        n24.arrow,
        n24.text,
        n7.arrow,
        n7.text,
        label("l_rate"),
        fixed1(s.closed_per_day)
    );
    let value = match (s.days.first(), s.days.last()) {
        (Some(first), Some(last)) => {
            format!("open {} → {}（{} 日）", first.open, last.open, s.days.len())
        }
        _ => format!("open {NONE}"),
    };
    let n = s.days.len();
    let more = s
        .days
        .iter()
        .enumerate()
        .filter(|(i, _)| (n - 1 - i).is_multiple_of(2))
        .map(|(_, d)| {
            format!(
                "{} open {} · +{} / −{}",
                &clock(d.end)[5..10],
                d.open,
                d.created,
                d.closed
            )
        })
        .collect();
    Card {
        title: label("l_burn"),
        kind,
        value,
        src: format!("{BURN_SRC} · 時点 {}", hm(s.at)),
        more,
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

/// 一覧の中身（閉じた行は出さない・閉じた epic の頭は、閉じていない下の項が在るときだけ残す・行 g-ledger-home）。
/// 板の段は持たない（`staged_body` に空の段を渡した値）。
pub fn body(screen: &Screen) -> Body<Vec<Group>> {
    staged_body(screen, &BTreeMap::new())
}

/// 板の段つきの一覧の中身（`body` と同じ組で、下の項は bead の id の段が在れば段の字と記号を出す・epic の頭は段を出さない）。
pub fn staged_body(screen: &Screen, stages: &BTreeMap<String, Staged>) -> Body<Vec<Group>> {
    match &screen.board {
        Reading::Unknown => Body::Unmeasured(LEDGER_UNREAD),
        Reading::Known(b) if b.groups.is_empty() => Body::Empty(EMPTY),
        Reading::Known(b) => {
            let groups: Vec<Group> = b
                .groups
                .iter()
                .filter_map(|g| {
                    let children: Vec<Item> = g
                        .children
                        .iter()
                        .filter(|r| listed(r))
                        .map(|r| staged_item(r, stages.get(r.id.as_str())))
                        .collect();
                    let keep = match &g.epic {
                        Some(epic) => listed(epic) || !children.is_empty(),
                        None => !children.is_empty(),
                    };
                    keep.then(|| Group {
                        head: g.epic.as_ref().map(item),
                        children,
                    })
                })
                .collect();
            if groups.is_empty() {
                Body::Empty(NO_OPEN)
            } else {
                Body::Filled(groups)
            }
        }
    }
}

/// 未反映の段は最初は開く（見本の id unref の details）。
pub const UNREF_OPEN: bool = true;

/// 未反映の一覧に出す行の数の上限（超える分は残りの数の 1 行）。
pub const UNREF_MAX: usize = 20;

/// 未反映の一覧の口が読めないときの理由。
pub const UNREF_REASON: &str =
    "未反映の一覧の口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 口は読めたが、3 種の全部が「まだ分からない」ときの理由。
pub const UNREF_UNKNOWN: &str =
    "未反映は器（scribe2）の局面の出力から読むが、その出力がまだ無いので、未反映の一覧はまだ分からない";

/// 測れて 0 件のときの 1 行。
pub const UNREF_EMPTY: &str = "未反映のものは無い";

/// 未反映の種類ごとの次の 1 手の語の鍵（1 か所の表）。
pub const UNREF_NEXT: [(UnreflectedKind, &str); 3] = [
    (UnreflectedKind::Memo, "nx_promote"),
    (UnreflectedKind::Ruling, "nx_declare"),
    (UnreflectedKind::Request, "nx_reflect"),
];

/// 未反映の一覧の 1 行（id・題・種類の名・年齢の字・次の 1 手の語の鍵）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrefRow {
    pub id: String,
    pub title: String,
    pub kind: &'static str,
    pub age: String,
    pub next: &'static str,
}

/// 未反映の一覧（先頭から UNREF_MAX までの行・超えた数・分からない種類の名）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrefList {
    pub rows: Vec<UnrefRow>,
    pub more: Option<usize>,
    pub unknown: Vec<&'static str>,
}

/// 次の 1 手の語の鍵（表から引く）。
fn next_key(kind: UnreflectedKind) -> &'static str {
    UNREF_NEXT
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, n)| *n)
        .expect("次の 1 手の表は 3 つの全部を持つ")
}

/// 電文の 1 行を一覧の 1 行にする（作った時刻から今 now までの日数の字に）。
fn unref_row(kind: UnreflectedKind, row: &UnreflectedRow, now: EpochSecs) -> UnrefRow {
    UnrefRow {
        id: row.id.clone(),
        title: row.title.clone(),
        kind: kind_name(kind),
        age: age(row.created.map(|c| days_since(c, now))),
        next: next_key(kind),
    }
}

/// 未反映の一覧の口の本文を一覧にする（種類は UnreflectedKind の ALL の順・行は電文の順のまま・
/// 並べ直しと数え直しをしない・年齢は今 now から数える）。
pub fn unref_list(fetched: &Fetched, now: EpochSecs) -> Body<UnrefList> {
    let list = match fetched {
        Fetched::NotRead => return Body::Unmeasured(NOT_READ),
        Fetched::Failed => return Body::Unmeasured(UNREF_REASON),
        Fetched::Body(text) => match wire::decode::<UnreflectedList>(text) {
            Ok(list) => list,
            Err(_) => return Body::Unmeasured(NO_CONTENT),
        },
    };
    let mut rows = Vec::new();
    let mut unknown = Vec::new();
    for kind in UnreflectedKind::ALL {
        let reading = match kind {
            UnreflectedKind::Memo => &list.memos,
            UnreflectedKind::Ruling => &list.rulings,
            UnreflectedKind::Request => &list.requests,
        };
        match reading {
            Reading::Known(items) => rows.extend(items.iter().map(|r| unref_row(kind, r, now))),
            Reading::Unknown => unknown.push(kind_name(kind)),
        }
    }
    if unknown.len() == UnreflectedKind::ALL.len() {
        return Body::Unmeasured(UNREF_UNKNOWN);
    }
    if rows.is_empty() && unknown.is_empty() {
        return Body::Empty(UNREF_EMPTY);
    }
    let more = rows.len().checked_sub(UNREF_MAX).filter(|n| *n > 0);
    rows.truncate(UNREF_MAX);
    Body::Filled(UnrefList {
        rows,
        more,
        unknown,
    })
}

/// 上限を超えた残りの数の 1 行。
pub fn more_line(n: usize) -> String {
    format!("ほか {n} 件")
}

/// 未反映の数の chip の class（0 は class z を足す）。
pub fn unref_chip(count: u32) -> &'static str {
    if count == 0 {
        "chip num unref-n z"
    } else {
        "chip num unref-n"
    }
}

/// 画面の状態の signal は context から受ける（無いときは Screen の initial で作る）。
#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::{RwSignal, use_context};
    let screen = use_context::<RwSignal<Screen>>().unwrap_or_else(|| RwSignal::new(Screen::initial()));
    dom::view(screen)
}

/// 台帳の block の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use std::collections::BTreeMap;

    use leptos::prelude::*;

    use tsuzuri_contract::board::Reading;

    use super::{
        BLOCK, BURN_CAPTION, Card, EpicBar, Group, Judge, LAYOUT, METRICS_PATH, Metrics, NONE,
        Net, OUTSIDE, Part, Tier, UNREF_OPEN, UNREF_PATH, UnrefList, UnrefRow, burn_svg, content,
        count, epic_cards, group_cards, more_line, name_label, spark_svg, staged_body, unref_chip,
        unref_list,
    };
    use crate::frame::{self, Mode};
    use crate::project::pipeline::{self, stages};
    use crate::project::{Body, UNKNOWN, fold, item_view, map, section, state_icon, unmeasured};
    use crate::view::{Fetched, Screen};
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, hs, shows_internal};
    use crate::widgets::hover::attach;
    use crate::widgets::hover::attach_some;

    pub fn view(screen: RwSignal<Screen>) -> AnyView {
        let fetched = crate::net::read(METRICS_PATH);
        let unref = crate::net::read(UNREF_PATH);
        let graph = crate::net::read(map::PATH);
        let got = move || fetched.with(|f| screen.with(|s| content(f, s, crate::net::now())));
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
                .map(|(tier, parts)| tier_view(*tier, parts, &m, screen, unref, graph))
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
                    Tier::List => list_view(screen, graph),
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
    fn tier_view(
        tier: Tier,
        parts: &[Part],
        m: &Metrics,
        screen: RwSignal<Screen>,
        unref: ReadSignal<Fetched>,
        graph: ReadSignal<Fetched>,
    ) -> AnyView {
        match tier {
            Tier::Top => {
                let boxes = parts.iter().map(|p| box_view(*p, m)).collect_view();
                view! { <div class="l4">{boxes}</div> }.into_any()
            }
            Tier::Main => {
                let items = parts.iter().map(|p| line_view(*p, m, graph)).collect_view();
                view! { <div class="lcap num">{items}</div> }.into_any()
            }
            Tier::Burn => {
                let items = parts.iter().map(|p| line_view(*p, m, graph)).collect_view();
                view! { <div class="lmid" tabindex="0" use:attach=m.burn_card.clone()>{items}</div> }
                    .into_any()
            }
            Tier::Memo => {
                let boxes = parts.iter().map(|p| box_view(*p, m)).collect_view();
                view! { <div class="mpro" aria-label=label("memo_promo")>{boxes}</div> }.into_any()
            }
            Tier::More => {
                let rows = parts
                    .iter()
                    .map(|p| view! { <div class="gm1 num">{line_view(*p, m, graph)}</div> })
                    .collect_view();
                // 記録に値が無いときだけ mode から初めの値を取る（持ち主が開き閉じを変えた後は記録の値）。
                let ctx = use_context::<HelpCtx>();
                let expert = move || ctx.is_some_and(|c| shows_internal(c.mode.get()));
                let (open, toggle) = fold("ledger:more".to_string(), expert);
                view! {
                    <details class="gmore" prop:open=open on:toggle=toggle>
                        <summary><span class="rm-t">{label("p_more")}</span>" "<span class="rm-a" aria-hidden="true">"▸"</span></summary>
                        <div class="gm">{rows}</div>
                    </details>
                }
                .into_any()
            }
            Tier::Unref => parts
                .iter()
                .map(|p| match p {
                    Part::UnrefCount => unref_view(*p, m, unref),
                    _ => line_view(*p, m, graph),
                })
                .collect_view()
                .into_any(),
            Tier::List => parts
                .iter()
                .map(|_| list_view(screen, graph))
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
        let unknown = (part == Part::Unref && m.unref.partial()).then(|| state_icon(UNKNOWN));
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

    /// 札と字の 1 項（主な指標の行・「詳しく」・burndown）。
    fn line_view(part: Part, m: &Metrics, graph: ReadSignal<Fetched>) -> AnyView {
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
                // グラフの読みが替わっても epic の card が同じなら行を組み直さない。
                let epics = m.epics.clone();
                let cards = Memo::new(move |_| graph.with(|g| epic_cards(&epics, g)));
                let bars = m.epics.clone();
                let rows = move || {
                    cards.with(|c| {
                        bars.iter()
                            .map(|e| epic_view(e, c.get(&e.id).cloned()))
                            .collect_view()
                    })
                };
                view! { <span>{hs(part.key())}</span><div class="lep">{rows}</div> }.into_any()
            }
            _ => {
                let text = m.text(part).unwrap_or_else(|| NONE.to_string());
                view! { <span>{hs(part.key())}" "<span class="num">{text}</span></span> }.into_any()
            }
        }
    }

    /// 未反映の段（見出しに数と分からない種類の chip・中に未反映の一覧・記録の鍵 ledger:unref）。
    fn unref_view(part: Part, m: &Metrics, unref: ReadSignal<Fetched>) -> AnyView {
        let kinds = m
            .unref
            .unknown
            .iter()
            .map(|k| {
                view! { <span class="chip num">{name_label(k)}" "{state_icon(UNKNOWN)}</span> }
            })
            .collect_view();
        let ctx = use_context::<HelpCtx>();
        let mode = move || match ctx {
            Some(c) => c.mode.get(),
            None => Mode::from_query(&window().location().search().unwrap_or_default()),
        };
        let list = move || match unref_list(&unref.get(), crate::net::now()) {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => view! { <div class="small muted">{line}</div> }.into_any(),
            Body::Filled(l) => unref_rows(&l, mode()),
        };
        let (open, toggle) = fold("ledger:unref".to_string(), || UNREF_OPEN);
        view! {
            <details class="fold lep" id="unref" prop:open=open on:toggle=toggle>
                <summary>
                    {hs(part.key())}
                    <span class=unref_chip(m.unref.count)>{m.unref.text()}</span>
                    <span class="lchips">{kinds}</span>
                </summary>
                {list}
            </details>
        }
        .into_any()
    }

    /// 未反映の一覧の行（見本の unrefHTML・id・題・種類・年齢・次の 1 手の 5 列）と、残りの数と分からない種類の行。
    fn unref_rows(l: &UnrefList, mode: Mode) -> AnyView {
        let table = (!l.rows.is_empty()).then(|| {
            let rows = l.rows.iter().map(|r| unref_row_view(r, mode)).collect_view();
            view! {
                <div class="ulist">
                    <div class="urow hrow">
                        <span>"id"</span>
                        <span>{label("col_title")}</span>
                        {hs("u_kind")}
                        {hs("u_age")}
                        {hs("u_next")}
                    </div>
                    {rows}
                </div>
            }
        });
        let more = l
            .more
            .map(|n| view! { <div class="small muted">{more_line(n)}</div> });
        let unknown = l
            .unknown
            .iter()
            .map(|k| {
                view! {
                    <div class="small muted">{name_label(k)}" "{state_icon(UNKNOWN)}" "{label("gap_unknown")}</div>
                }
            })
            .collect_view();
        view! { {table}{more}{unknown} }.into_any()
    }

    /// 未反映の 1 行（id は節点の頁への link）。
    fn unref_row_view(r: &UnrefRow, mode: Mode) -> AnyView {
        view! {
            <div class="urow">
                <a class="u-id mono" href=frame::node_href(&r.id, mode)>{r.id.clone()}</a>
                <span class="u-t">{r.title.clone()}</span>
                <span>{name_label(r.kind)}</span>
                <span class="num">{r.age.clone()}</span>
                <span class="u-n">{label(r.next)}</span>
            </div>
        }
        .into_any()
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

    /// epic の進みの 1 行（題が引けなければ id だけ・題は節点の頁への link・節点の card が在れば付ける）。
    fn epic_view(e: &EpicBar, card: Option<Card>) -> AnyView {
        let name = e.title.clone().unwrap_or_else(|| e.id.clone());
        let ratio = format!("{} / {}", e.closed, e.total);
        let ctx = use_context::<HelpCtx>();
        let id = e.id.clone();
        let href = move || {
            let mode = match ctx {
                Some(c) => c.mode.get(),
                None => Mode::from_query(&window().location().search().unwrap_or_default()),
            };
            frame::node_href(&id, mode)
        };
        view! {
            <div class="ep">
                <a href=href use:attach_some=card><span class="nid">{e.id.clone()}</span>" "<span data-t="">{name}</span></a>
                <span class="bar" role="img" aria-label=ratio.clone()><i style=format!("width:{}%", e.pct)></i></span>
                <span class="meta num">{ratio}</span>
            </div>
        }
        .into_any()
    }

    /// 台帳の一覧（便 g-frame の中身・項の節点の card はグラフの読みから引く）。
    /// 項の段は block pipeline と同じ口の読みから組む（板の口を先に読み、その中で台帳の画面を読む）。
    fn list_view(screen: RwSignal<Screen>, graph: ReadSignal<Fetched>) -> AnyView {
        let pipe = crate::net::read(pipeline::PATH);
        let list = move || match pipe.with(|p| screen.with(|s| staged_body(s, &stages(p, crate::net::now())))) {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => view! { <div class="empty"><span>{line}</span></div> }.into_any(),
            Body::Filled(groups) => {
                let cards = graph.with(|g| group_cards(&groups, g));
                let rows = groups.iter().map(|g| group_view(g, &cards)).collect_view();
                view! { <ul class="items">{rows}</ul> }.into_any()
            }
        };
        view! { <div>{list}</div> }.into_any()
    }

    /// 1 組（epic の項と、その下の項を入れ子の一覧に）。
    fn group_view(group: &Group, cards: &BTreeMap<String, Card>) -> AnyView {
        let head = match &group.head {
            Some(epic) => item_view(epic, None, cards.get(&epic.id).cloned()),
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
            .map(|c| item_view(c, None, cards.get(&c.id).cloned()))
            .collect_view();
        view! {
            {head}
            <li class="nest"><ul class="items">{children}</ul></li>
        }
        .into_any()
    }
}
