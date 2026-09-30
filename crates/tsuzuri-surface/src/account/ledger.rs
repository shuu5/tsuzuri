//! account board の台帳の処理状況の block（便 h-frame の枠・便 h-led の中身）: 見本の session の tab の `#ledger`
//! （account/index.html の ledTable・ledOrder・ledMore・extremes と ledger.js の judge）。
//! 行は電文の projects の 1 行ずつ（台帳は ProjectRow の ledger）。並べ方は 4 つ（judge・project・net・backlog）で URL の query の `lsort=` に残し、
//! 並べ方の押しは履歴の 1 歩にする（見本の pushState・行 h-sort-hist）。
//! 判定の写し・純減の矢印・小数 1 桁・日数の字は着地済みの project の ledger の module の値と関数を使う（その block の DOM は呼ばない）。
//! 詳しくの段の 14 日の sparkline（見本の ledMore の spark14）も同じ module の spark と spark_svg で組む。
//! 未反映は電文の台帳の未反映の数で、project board の指標の段と同じ Unref の形にする。1 種か 2 種が読めなければ数（部分の和）に
//! 測れていないの印を添え、3 種とも分からなければ数えない字「―」で印は添えない（行 g-unref-dash-acct）。台帳が Unknown の行も「―」。
//! 列の最大と最小の印は open の task・closed/日・未反映の 3 列に付ける（見本の ledTable の eT・eR・eU・行 h-acct-rest）。
//! 未反映の列は unref_of が数を返す行（3 種とも分からない行と台帳が Unknown の行を除く）だけで数え、印もその行だけに付ける。
//! 行の project の欄は hover の card（便 h-cards-led・見本の ledCard）を持ち、中身は account の cards の led_card で組む。
//! 行ごとの「詳しく」の開き閉じは頁の一生の間だけ signal に持ち、URL にも画面の外にも書かない。
//! 並べ・行の値・列の最大と最小は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use std::cmp::Reverse;

use tsuzuri_contract::account::{AccountDoc, ProjectRow};
use tsuzuri_contract::board::{LedgerJudge, Reading};
use tsuzuri_contract::stats::LedgerStats;

use super::cards::led_card;
use super::projects::{unref_count, unref_of};
use crate::frame::{self, Block};
use crate::project::Body;
use crate::project::ledger::{
    JUDGES, Judge, NONE, Net, SPARK_H, SPARK_W, Unref, age, fixed1, judge, net, spark, spark_svg,
};
use crate::view::Fetched;
use crate::widgets::hover::Card;

pub const BLOCK: Block = Block {
    id: "ledger",
    heading: "ledger_state",
    class: "panel",
};

/// block の中身（中身の関数がまだ無い: 測れていない）。
/// 便 h-frame の歯が pin するので変えない。中身は `content` が組む。
pub fn body(fetched: &Fetched) -> Body<()> {
    super::pending(fetched)
}

/// 並べ方を残す URL の query の鍵。
pub const SORT_KEY: &str = "lsort";

/// 表の class（見本の `.jtab`）。
pub const JTAB: &str = "jtab";

/// 見出しの行の class（見本の `.jrow.hrow`）。
pub const HROW: &str = "jrow hrow";

/// 見出しの行の列（class と語の鍵・この順・最後に「詳しく」の空の列）。
pub const COLUMNS: [(&str, &str); 5] = [
    ("c-j", "l_judge"),
    ("c-p", "project"),
    ("c-task", "l_task"),
    ("c-rate", "l_rate"),
    ("c-un", "l_unref"),
];

/// 詳しくの段の項の語の鍵（この順）。
pub const MORE: [&str; 5] = ["l_ready", "l_blocked", "l_stale", "l_net7", "l_lead"];

/// 台帳が Unknown の行の判定の欄の語の鍵。
pub const UNKNOWN_KEY: &str = "st_unknown";

/// 列の最大と最小の印の class（見本の xm）。
pub const HI: &str = "hi";
pub const LO: &str = "lo";

/// 表の下の凡例（印の class と字）。
pub const LEGEND: [(&str, &str); 2] = [("xm hi", "列の最大"), ("xm lo", "列の最小")];

/// 電文の projects が 0 行のときの 1 行（測れて 0 件・測れていないと分ける）。
pub const NO_ROWS: &str = "project の行が 0 件（電文の projects が空）";

/// 台帳の表の並べ方（見本の LSORTS の順）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Judge,
    Project,
    Net,
    Backlog,
}

impl Sort {
    pub const ALL: [Sort; 4] = [Sort::Judge, Sort::Project, Sort::Net, Sort::Backlog];

    /// URL と button の字。
    pub fn key(self) -> &'static str {
        match self {
            Sort::Judge => "judge",
            Sort::Project => "project",
            Sort::Net => "net",
            Sort::Backlog => "backlog",
        }
    }

    /// button の語の鍵。
    pub fn label_key(self) -> &'static str {
        match self {
            Sort::Judge => "sort_judge",
            Sort::Project => "sort_project",
            Sort::Net => "sort_net",
            Sort::Backlog => "sort_backlog",
        }
    }
}

/// URL の query から並べ方（`lsort=net` の形・無いか知らない値は judge）。
pub fn sort_of(search: &str) -> Sort {
    frame::param(search, SORT_KEY)
        .and_then(|v| Sort::ALL.into_iter().find(|s| s.key() == v))
        .unwrap_or(Sort::Judge)
}

/// 並べ方を選んだ後の URL の query（既定の judge は `lsort=` を消す・ほかは `lsort=` に残す・ほかの値と順はそのまま）。
pub fn with_sort(search: &str, sort: Sort) -> String {
    match sort {
        Sort::Judge => without_param(search, SORT_KEY),
        _ => frame::with_param(search, SORT_KEY, sort.key()),
    }
}

/// query の 1 つの鍵を消す（同じ鍵が幾つあっても全部・ほかの値と順はそのまま・`?` で始める）。
pub fn without_param(search: &str, key: &str) -> String {
    let parts: Vec<&str> = search
        .trim_start_matches('?')
        .split('&')
        .filter(|kv| !kv.is_empty())
        .filter(|kv| kv.split_once('=').map_or(*kv, |(k, _)| k) != key)
        .collect();
    format!("?{}", parts.join("&"))
}

/// 判定の位（JUDGES の表の順 = 悪い順・台帳が Unknown は台帳なしと同じ位・表に無い判定は後ろ）。
pub fn rank(ledger: &Reading<LedgerStats>) -> usize {
    let value = match ledger {
        Reading::Known(s) => s.judge,
        Reading::Unknown => LedgerJudge::NoLedger,
    };
    JUDGES
        .iter()
        .position(|j| j.judge == value)
        .unwrap_or(JUDGES.len())
}

/// 行の電文の projects の中の位置の並び（どの並べも同じ値は電文の projects の順）。
/// judge は悪い順・project は電文の順・net は 7 日の純減が小さい順で同じなら 24 時間の純減が小さい順・
/// backlog は open の task が多い順。net と backlog は台帳が Unknown の project を末尾に置く。
pub fn order(projects: &[ProjectRow], sort: Sort) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..projects.len()).collect();
    let known = |i: usize| match projects.get(i).map(|p| &p.ledger) {
        Some(Reading::Known(s)) => Some(s),
        _ => None,
    };
    match sort {
        Sort::Judge => idx.sort_by_key(|&i| projects.get(i).map(|p| rank(&p.ledger))),
        Sort::Project => {}
        Sort::Net => idx.sort_by_key(|&i| match known(i) {
            Some(s) => (false, s.net_drop_7d, s.net_drop_24h),
            None => (true, 0, 0),
        }),
        Sort::Backlog => idx.sort_by_key(|&i| match known(i) {
            Some(s) => (false, Reverse(s.open.task)),
            None => (true, Reverse(0)),
        }),
    }
    idx
}

/// 列の最大と最小。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extremes {
    pub max: f64,
    pub min: f64,
}

/// 列の最大と最小（値が 2 つ以上在り、最大と最小が違うときだけ）。
pub fn extremes(values: &[f64]) -> Option<Extremes> {
    if values.len() < 2 {
        return None;
    }
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    (max > min).then_some(Extremes { max, min })
}

/// cell の印の class（最大は hi・最小は lo・ほかと印の無い列は None）。
pub fn mark(ext: Option<Extremes>, value: f64) -> Option<&'static str> {
    let e = ext?;
    if value >= e.max {
        Some(HI)
    } else if value <= e.min {
        Some(LO)
    } else {
        None
    }
}

/// 数の cell の class（`c-n c-<列>` と印）。
pub fn cell_class(column: &str, mark: Option<&str>) -> String {
    match mark {
        Some(m) => format!("c-n {column} {m}"),
        None => format!("c-n {column}"),
    }
}

/// 未反映の cell の class（`c-n c-un` と印・数が 1 以上なら on を足す）。
pub fn un_class(mark: Option<&str>, count: u32) -> String {
    let class = cell_class("c-un", mark);
    if count > 0 {
        format!("{class} on")
    } else {
        class
    }
}

/// closed/日の棒の幅の百分率（見本の rateBar・列の最大を 100・値が在れば 2 以上）。
pub fn bar_pct(value: f64, max: f64) -> u32 {
    if max > 0.0 {
        (value / max * 100.0).round().clamp(2.0, 100.0) as u32
    } else {
        0
    }
}

/// 台帳が Known の行の欄。
#[derive(Debug, Clone, PartialEq)]
pub struct Cells {
    pub judge: Judge,
    /// open の task の数と cell の class。
    pub task: u32,
    pub task_class: String,
    /// 24 時間の純減。
    pub net24: Net,
    /// closed/日（小数 1 桁）と棒の幅と cell の class。
    pub rate: String,
    pub rate_pct: u32,
    pub rate_class: String,
    pub ready: u32,
    pub blocked: u32,
    pub stale: u32,
    /// 7 日の純減。
    pub net7: Net,
    /// lead の p50 の字（無ければ「―」）。
    pub lead: String,
    /// 14 日の created と closed の sparkline の svg の字（project board の台帳の block と同じ関数で組む）。
    pub spark: String,
    /// 未反映の数と読めない種類（各 project の表と同じ unref_count で組む）。
    pub unref: Unref,
    /// 未反映の cell の class（列の最大と最小の印・1 以上は on）。
    pub un_class: String,
}

/// 表の 1 行。
#[derive(Debug, Clone, PartialEq)]
pub struct LedRow {
    /// 電文の projects の中の位置（0 から）。
    pub index: usize,
    pub name: String,
    /// 行の class（`jrow j-<判定>`・台帳が Unknown は j-none）。
    pub class: String,
    pub cells: Reading<Cells>,
    /// project の欄の hover の card（見本の ledCard・各 project の表の台帳の欄と同じ関数）。
    pub card: Card,
}

impl LedRow {
    /// 判定の欄の語の鍵（台帳が Unknown は st_unknown）。
    pub fn judge_key(&self) -> &'static str {
        match &self.cells {
            Reading::Known(c) => c.judge.key,
            Reading::Unknown => UNKNOWN_KEY,
        }
    }

    /// 行の数の欄の字（open の task・24 時間の純減・closed/日・未反映の順・台帳が Unknown は全部「―」）。
    /// 未反映は Unref の text（3 種とも分からなければ「―」・ほかは数で、部分の和の測れていないの印は DOM が添える）。
    pub fn numbers(&self) -> [String; 4] {
        match &self.cells {
            Reading::Known(c) => [
                c.task.to_string(),
                c.net24.text.clone(),
                c.rate.clone(),
                c.unref.text(),
            ],
            Reading::Unknown => std::array::from_fn(|_| NONE.to_string()),
        }
    }

    /// 詳しくの段の項（MORE の語の鍵と字・この順・台帳が Unknown は全部「―」）。
    pub fn more(&self) -> [(&'static str, String); 5] {
        let texts: [String; 5] = match &self.cells {
            Reading::Known(c) => [
                c.ready.to_string(),
                c.blocked.to_string(),
                c.stale.to_string(),
                c.net7.text.clone(),
                c.lead.clone(),
            ],
            Reading::Unknown => std::array::from_fn(|_| NONE.to_string()),
        };
        let mut texts = texts.into_iter();
        MORE.map(|k| (k, texts.next().unwrap_or_default()))
    }

    /// 印の付いた列の class（open の task・closed/日・未反映の順）。
    pub fn marks(&self) -> [Option<&'static str>; 3] {
        let of = |class: &str| {
            [HI, LO]
                .into_iter()
                .find(|m| class.split_whitespace().any(|w| w == *m))
        };
        match &self.cells {
            Reading::Known(c) => [of(&c.task_class), of(&c.rate_class), of(&c.un_class)],
            Reading::Unknown => [None, None, None],
        }
    }
}

/// 表（並べ方と行の並び）。
#[derive(Debug, Clone, PartialEq)]
pub struct LedTable {
    pub sort: Sort,
    pub rows: Vec<LedRow>,
}

impl LedTable {
    /// 行の電文の中の位置（並びの順）。
    pub fn order(&self) -> Vec<usize> {
        self.rows.iter().map(|r| r.index).collect()
    }

    /// 行の project の名（並びの順）。
    pub fn names(&self) -> Vec<&str> {
        self.rows.iter().map(|r| r.name.as_str()).collect()
    }
}

/// block の中身（口が読めない・まだ読んでいない・本文が電文として読めないときは測れていないと理由の 1 行）。
pub fn content(fetched: &Fetched, sort: Sort) -> Body<LedTable> {
    match super::doc(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(doc) if doc.projects.is_empty() => Body::Empty(NO_ROWS),
        Ok(doc) => Body::Filled(table(&doc, sort)),
    }
}

/// 電文を表に組む（列の最大と最小は台帳が Known の行だけで数え、未反映の列は unref_of が数を返す行だけで数える）。
pub fn table(doc: &AccountDoc, sort: Sort) -> LedTable {
    let known: Vec<&LedgerStats> = doc
        .projects
        .iter()
        .filter_map(|p| match &p.ledger {
            Reading::Known(s) => Some(s),
            Reading::Unknown => None,
        })
        .collect();
    let tasks: Vec<f64> = known.iter().map(|s| f64::from(s.open.task)).collect();
    let rates: Vec<f64> = known.iter().map(|s| s.closed_per_day).collect();
    let unrefs: Vec<f64> = doc
        .projects
        .iter()
        .filter_map(unref_of)
        .map(|u| f64::from(u.count))
        .collect();
    let (ext_task, ext_rate, ext_un) = (extremes(&tasks), extremes(&rates), extremes(&unrefs));
    let rate_max = rates.iter().copied().fold(0.0, f64::max);
    let rows = order(&doc.projects, sort)
        .into_iter()
        .filter_map(|i| doc.projects.get(i).map(|p| (i, p)))
        .map(|(i, p)| {
            let cells = match &p.ledger {
                Reading::Known(s) => Reading::Known(Cells {
                    judge: judge(s.judge),
                    task: s.open.task,
                    task_class: cell_class("c-task", mark(ext_task, f64::from(s.open.task))),
                    net24: net(s.net_drop_24h),
                    rate: fixed1(s.closed_per_day),
                    rate_pct: bar_pct(s.closed_per_day, rate_max),
                    rate_class: cell_class("c-rate", mark(ext_rate, s.closed_per_day)),
                    ready: s.ready,
                    blocked: s.blocked,
                    stale: s.stale,
                    net7: net(s.net_drop_7d),
                    lead: age(s.lead.map(|l| l.p50)),
                    spark: spark_svg(&spark(&s.days, SPARK_W, SPARK_H)),
                    unref: unref_count(s),
                    un_class: un_class(
                        unref_of(p).and_then(|u| mark(ext_un, f64::from(u.count))),
                        s.unreflected,
                    ),
                }),
                Reading::Unknown => Reading::Unknown,
            };
            LedRow {
                index: i,
                name: p.name.clone(),
                class: row_class(&cells),
                cells,
                card: led_card(p),
            }
        })
        .collect();
    LedTable { sort, rows }
}

/// 行の class（判定の class の 2 語目・台帳が Unknown は j-none）。
fn row_class(cells: &Reading<Cells>) -> String {
    let judge_class = match cells {
        Reading::Known(c) => c.judge.class,
        Reading::Unknown => judge(LedgerJudge::NoLedger).class,
    };
    let word = judge_class.split_whitespace().nth(1).unwrap_or_default();
    format!("jrow {word}")
}

/// 表を組む関数を view が使う（口は account の mod.rs の PATH・本文は doc で読む）。
#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// block の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use std::collections::BTreeMap;

    use leptos::prelude::*;
    use tsuzuri_contract::board::Reading;

    use super::{
        BLOCK, COLUMNS, Cells, HROW, JTAB, LEGEND, LedRow, LedTable, MORE, Sort, UNKNOWN_KEY,
        content, sort_of, with_sort,
    };
    use crate::account::PATH;
    use crate::project::ledger::{Judge, NONE, Net};
    use crate::project::{Body, UNKNOWN, body_view, section, state_icon, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, hs, shows_internal};
    use crate::widgets::hover::attach;

    /// 行ごとの「詳しく」の開き閉じ（project の名ごと・頁の一生の間だけ）。
    type Opened = RwSignal<BTreeMap<String, bool>>;

    /// 今の URL の query（読めなければ空）。
    fn search() -> String {
        window().location().search().unwrap_or_default()
    }

    /// URL の query を履歴に積む（頁は読み直さない・見本の pushState）。
    /// 戻ると進むは board が popstate で block を組み直し、block は組むときに URL の並べ方を読む。
    fn push(url: &str) {
        if let Ok(history) = window().history() {
            let _ = history.push_state_with_url(
                &web_sys::wasm_bindgen::JsValue::NULL,
                "",
                Some(url),
            );
        }
    }

    pub fn view() -> AnyView {
        let fetched = crate::net::read(PATH);
        let sort = RwSignal::new(sort_of(&search()));
        let opened: Opened = RwSignal::new(BTreeMap::new());
        // 記録に値が無い行は mode から初めの値を取る（見本の isOpen・経験者は開く）。
        let ctx = use_context::<HelpCtx>();
        let expert = Signal::derive(move || ctx.is_some_and(|c| shows_internal(c.mode.get())));
        let body = move || match fetched.with(|f| content(f, sort.get())) {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Filled(table) => table_view(table, opened, expert),
        };
        section(BLOCK, sort_bar(sort), body.into_any())
    }

    fn sort_bar(sort: RwSignal<Sort>) -> AnyView {
        let buttons = Sort::ALL
            .into_iter()
            .map(|s| {
                let pressed = move || (sort.get() == s).to_string();
                // 今の並べの押しは履歴に積まない。
                let pick = move |_| {
                    if sort.get_untracked() == s {
                        return;
                    }
                    push(&with_sort(&search(), s));
                    sort.set(s);
                };
                view! {
                    <button type="button" data-lsort=s.key() aria-pressed=pressed data-term=s.label_key() on:click=pick>
                        {label(s.label_key())}
                    </button>
                }
            })
            .collect_view();
        view! {
            <div class="sortbar">
                {hs("sort_by")}
                <div class="seg wrap" role="group" aria-label=label("sort_by")>{buttons}</div>
            </div>
        }
        .into_any()
    }

    fn table_view(table: LedTable, opened: Opened, expert: Signal<bool>) -> AnyView {
        let head = COLUMNS
            .into_iter()
            .map(|(class, key)| {
                let sub = (class == "c-task").then(|| view! { <span class="hsub">"↕ 24h"</span> });
                view! { <div class=class>{hs(key)}{sub}</div> }
            })
            .collect_view();
        let rows = table
            .rows
            .into_iter()
            .map(|r| row_view(r, opened, expert))
            .collect_view();
        let legend = LEGEND
            .into_iter()
            .map(|(class, text)| view! { <span><i class=class></i>{text}</span> })
            .collect_view();
        view! {
            <div class=JTAB>
                <div class=HROW>{head}<div class="rowmore"></div></div>
                {rows}
            </div>
            <div class="lgline xmleg">{legend}</div>
        }
        .into_any()
    }

    fn row_view(row: LedRow, opened: Opened, expert: Signal<bool>) -> AnyView {
        let name = row.name.clone();
        let card = row.card;
        let project = view! { <div class="c-p" tabindex="0" use:attach=card><span data-t="">{name.clone()}</span></div> };
        let cells = match row.cells {
            Reading::Known(c) => c,
            Reading::Unknown => {
                return view! {
                    <div class=row.class>
                        <div class="c-j">{state_icon(UNKNOWN)}<span>{label(UNKNOWN_KEY)}</span></div>
                        {project}
                        <div class="c-n c-task muted">{NONE}</div>
                        <div class="c-n c-rate muted">{NONE}</div>
                        <div class="c-n c-un muted">{NONE}</div>
                        <div class="rowmore"></div>
                    </div>
                }
                .into_any();
            }
        };
        let is_open = {
            let key = name.clone();
            move || {
                opened
                    .with(|m| m.get(&key).copied())
                    .unwrap_or_else(|| expert.get())
            }
        };
        let class = {
            let (base, is_open) = (row.class.clone(), is_open.clone());
            move || {
                if is_open() {
                    format!("{base} open")
                } else {
                    base.clone()
                }
            }
        };
        let expanded = {
            let is_open = is_open.clone();
            move || is_open().to_string()
        };
        let arrow = {
            let is_open = is_open.clone();
            move || if is_open() { "▾" } else { "▸" }
        };
        let toggle = move |_| {
            let now = untrack(&is_open);
            opened.update(|m| {
                m.insert(name.clone(), !now);
            });
        };
        view! {
            <div class=class>
                <div class="c-j">{judge_view(cells.judge)}</div>
                {project}
                <div class=cells.task_class.clone()>
                    <b class="num">{cells.task}</b>
                    {net_view(&cells.net24)}
                </div>
                <div class=cells.rate_class.clone()>
                    <span class="rbar" aria-hidden="true"><i style=format!("width:{}%", cells.rate_pct)></i></span>
                    <span class="num">{cells.rate.clone()}</span>
                </div>
                <div class=cells.un_class.clone()>
                    <b class="num">{cells.unref.text()}</b>
                    {cells.unref.partial().then(|| state_icon(UNKNOWN))}
                </div>
                <button type="button" class="rowmore" aria-expanded=expanded aria-label=label("p_more") on:click=toggle>
                    <span class="rm-t">{label("p_more")}</span>" "<span class="rm-a">{arrow}</span>
                </button>
                <div class="c-more">{more_view(&cells)}</div>
            </div>
        }
        .into_any()
    }

    /// 詳しくの段（ready・blocked・stale・7 日の純減・lead の p50・最後に 14 日の sparkline）。
    fn more_view(c: &Cells) -> AnyView {
        let values = [
            c.ready.to_string().into_any(),
            c.blocked.to_string().into_any(),
            c.stale.to_string().into_any(),
            net_view(&c.net7),
            c.lead.clone().into_any(),
        ];
        let items = MORE
            .into_iter()
            .zip(values)
            .map(|(key, value)| {
                view! { <span class="mi"><span class="lk">{hs(key)}</span><b class="num">{value}</b></span> }
            })
            .collect_view();
        view! {
            {items}
            <span class="mi sp" inner_html=c.spark.clone()></span>
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

    /// 判定の 1 語（見本の judgeHTML の big）。
    fn judge_view(j: Judge) -> AnyView {
        view! {
            <span class=format!("{} big", j.class) data-term=j.key tabindex="0">
                <b aria-hidden="true">{j.symbol}</b>
                <span>{label(j.key)}</span>
            </span>
        }
        .into_any()
    }
}
