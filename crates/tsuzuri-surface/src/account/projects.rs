//! account board の各 project の表の block（便 h-frame の枠・便 h-proj の中身）: 見本の projects の tab の `#ptab`
//! （account/index.html の projTable・PSORTS）。
//! 行は電文の projects の 1 行ずつで 9 列。並べ方は 4 つ（need・group・judge・unref）で URL の query の `psort=` に残す。
//! 決定待ちは電文に値が無く、未反映は読めない種類を 0 と数えた和で確かな値と言えないので、どちらも「―」を出す（未決）。
//! 並べ・行の値・class は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use tsuzuri_contract::account::{AccountDoc, ProjectRow, RunCounts};
use tsuzuri_contract::board::{LedgerJudge, NextMove, Reading};

use super::windows::{NOT_YET_KEY, OPEN_NEW_KEY, open_url};
use crate::frame::{self, Block, Mode};
use crate::project::Body;
use crate::project::ledger::{JUDGES, Judge, Net, judge, net};
use crate::project::next::{big, key as next_key};
use crate::project::seat::{NG, OK, Sign, top};
use crate::project::{UNKNOWN, state_key};
use crate::view::Fetched;

pub const BLOCK: Block = Block {
    id: "ptab",
    heading: "dashboards",
    class: "panel",
};

/// block の中身（中身の関数がまだ無い: 測れていない）。
/// 便 h-frame の歯が pin するので変えない。中身は `content` が組む。
pub fn body(fetched: &Fetched) -> Body<()> {
    super::pending(fetched)
}

/// 並べ方を残す URL の query の鍵。
pub const PSORT_KEY: &str = "psort";

/// 表の class（見本の `.ptab`）。
pub const PTAB: &str = "ptab";

/// 行の class（見本の `.prow`）。
pub const PROW: &str = "prow";

/// 見出しの行の class（見本の `.prow.hrow`）。
pub const HROW: &str = "prow hrow";

/// 群の見出しの行の class（見本の `.pgh`）。
pub const PGH: &str = "pgh";

/// 見出しの行の列の語の鍵（この順・列の見出しの class は `h-<鍵>`）。
pub const COLUMNS: [&str; 9] = [
    "project",
    "p_need",
    "p_wait",
    "l_unref",
    "ledger_block",
    "runs4",
    "seat",
    "accounts",
    "p_open",
];

/// 列の class（見本の `.c-pn`・`.c-need`・`.c-wait`・`.c-un2`・`.c-led`・`.c-run`・`.c-orch`・`.c-acc`・`.c-open`）。
pub const C_PN: &str = "c-pn";
pub const C_NEED: &str = "c-need";
pub const C_WAIT: &str = "c-wait";
pub const C_UN2: &str = "c-un2";
pub const C_LED: &str = "c-led";
pub const C_RUN: &str = "c-run";
pub const C_ORCH: &str = "c-orch";
pub const C_ACC: &str = "c-acc";
pub const C_OPEN: &str = "c-open";

/// run の 4 列の箱の class（見本の `.rc4`）。
pub const RC4: &str = "rc4";

/// 0 の数の class（見本の `.rq.z`）。
pub const ZERO: &str = "z";

/// run の 4 列（電文の数の名・class・この順）。
pub const RUNS: [(&str, &str); 4] = [
    ("wait", "rq q-wait"),
    ("run", "rq q-run"),
    ("stop", "rq q-stop"),
    ("land", "rq q-land"),
];

/// tick の印の箱と heartbeat の字の class（見本の `.tkm`・`.hbm`・off は `.hb-off`）。
pub const TKM: &str = "tkm";
pub const HBM: &str = "hbm small";
pub const HBM_OFF: &str = "hbm small hb-off";

/// 押せない字の class（見本の `.btn.static.dis`）。
pub const NOT_YET_CLASS: &str = "btn static dis";

/// 値の無い欄の字。
pub const NONE_MARK: &str = "―";

/// 電文の projects が 0 行のときの 1 行（測れて 0 件・測れていないと分ける）。
pub const NO_ROWS: &str = "project の行が 0 件（電文の projects が空）";

/// 各 project の表の並べ方（見本の PSORTS の順）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PSort {
    Need,
    Group,
    Judge,
    Unref,
}

impl PSort {
    pub const ALL: [PSort; 4] = [PSort::Need, PSort::Group, PSort::Judge, PSort::Unref];

    /// URL と button の字。
    pub fn key(self) -> &'static str {
        match self {
            PSort::Need => "need",
            PSort::Group => "group",
            PSort::Judge => "judge",
            PSort::Unref => "unref",
        }
    }

    /// button の語の鍵。
    pub fn label_key(self) -> &'static str {
        match self {
            PSort::Need => "sort_need",
            PSort::Group => "sort_group",
            PSort::Judge => "sort_judge",
            PSort::Unref => "sort_unref",
        }
    }
}

/// URL の query から並べ方（`psort=group` の形・無いか知らない値は need）。
pub fn psort_of(search: &str) -> PSort {
    frame::param(search, PSORT_KEY)
        .and_then(|v| PSort::ALL.into_iter().find(|s| s.key() == v))
        .unwrap_or(PSort::Need)
}

/// 並べ方を選んだ後の URL の query（既定の need は `psort=` を消す・ほかは `psort=` に残す・ほかの値と順はそのまま）。
pub fn with_psort(search: &str, sort: PSort) -> String {
    match sort {
        PSort::Need => without_param(search, PSORT_KEY),
        _ => frame::with_param(search, PSORT_KEY, sort.key()),
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

/// 要対応の欄（lead の種類・語の鍵・重さ・1 行）。next が Unknown なら種類と 1 行は None で語の鍵は st_unknown。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Need {
    pub lead: Option<NextMove>,
    pub key: &'static str,
    /// 重さ（hi・mid・lo・行の class の `sev-<重さ>`）。
    pub sev: &'static str,
    pub line: Option<String>,
}

/// 要対応の欄（1 行は着地済みの次の一手の大きく出す箱の中身の字）。
pub fn need(project: &ProjectRow) -> Need {
    match &project.next {
        Reading::Known(step) => {
            let check = step.checks.iter().find(|c| c.kind == step.lead);
            Need {
                lead: Some(step.lead),
                key: next_key(step.lead),
                sev: sev(Some(step.lead)),
                line: Some(big(step.lead, check).what),
            }
        }
        Reading::Unknown => Need {
            lead: None,
            key: state_key(UNKNOWN),
            sev: sev(None),
            line: None,
        },
    }
}

/// 重さ（見本の SEV: 限度と移動・応答なし・止まっている走行は hi、束の承認・質問・発効待ちは mid、なしと測れていないは lo）。
pub fn sev(lead: Option<NextMove>) -> &'static str {
    match lead {
        Some(NextMove::LimitOrMove | NextMove::Unresponsive | NextMove::StalledRun) => "hi",
        Some(NextMove::BatchApproval | NextMove::Question | NextMove::AwaitingEffect) => "mid",
        Some(NextMove::Nothing) | None => "lo",
    }
}

/// need の並べの位（lead の NextMove の ALL の中の位置・next が Unknown はその後ろ）。
pub fn need_rank(project: &ProjectRow) -> usize {
    match &project.next {
        Reading::Known(step) => NextMove::ALL
            .iter()
            .position(|k| *k == step.lead)
            .unwrap_or(NextMove::ALL.len()),
        Reading::Unknown => NextMove::ALL.len(),
    }
}

/// 台帳の判定（台帳が Unknown は台帳なし）。
pub fn judge_of(project: &ProjectRow) -> LedgerJudge {
    match &project.ledger {
        Reading::Known(s) => s.judge,
        Reading::Unknown => LedgerJudge::NoLedger,
    }
}

/// judge の並べの位（着地済みの JUDGES の表の位置・悪い順）。
pub fn judge_rank(project: &ProjectRow) -> usize {
    let j = judge_of(project);
    JUDGES
        .iter()
        .position(|x| x.judge == j)
        .unwrap_or(JUDGES.len())
}

/// 台帳の欄（判定の 1 語と、台帳が読めれば open の task の数と純減 24h）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Led {
    pub judge: Judge,
    pub task: Option<u32>,
    pub net24: Option<Net>,
}

pub fn led(project: &ProjectRow) -> Led {
    let known = match &project.ledger {
        Reading::Known(s) => Some(s),
        Reading::Unknown => None,
    };
    Led {
        judge: judge(judge_of(project)),
        task: known.map(|s| s.open.task),
        net24: known.map(|s| net(s.net_drop_24h)),
    }
}

/// run の 4 列の 1 つ（class と字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub class: String,
    pub text: String,
}

/// run の 4 列（wait・run・stop・land の順・0 は class z・Unknown は 4 つとも「―」と class z）。
pub fn runs(counts: &Reading<RunCounts>) -> [Run; 4] {
    let value = |name: &str, c: &RunCounts| match name {
        "wait" => c.wait,
        "run" => c.run,
        "stop" => c.stop,
        _ => c.land,
    };
    RUNS.map(|(name, class)| {
        let n = match counts {
            Reading::Known(c) => Some(value(name, c)),
            Reading::Unknown => None,
        };
        Run {
            class: match n {
                Some(v) if v > 0 => class.to_string(),
                _ => format!("{class} {ZERO}"),
            },
            text: n.map_or_else(|| NONE_MARK.to_string(), |v| v.to_string()),
        }
    })
}

/// orchestrator の欄（状態の値・tick の健康の印・heartbeat）。席が Unknown なら 3 つとも測れていない。
#[derive(Debug, Clone, PartialEq)]
pub struct Orch {
    /// 状態の値（状態の記号に渡す・席が Unknown は unknown）。
    pub state: &'static str,
    pub tick: Reading<Sign>,
    pub heartbeat: Reading<&'static str>,
}

pub fn orch(project: &ProjectRow) -> Orch {
    match &project.seat {
        Reading::Known(card) => {
            let t = top(card);
            Orch {
                state: t.state,
                tick: t.tick,
                heartbeat: t.heartbeat,
            }
        }
        Reading::Unknown => Orch {
            state: UNKNOWN,
            tick: Reading::Unknown,
            heartbeat: Reading::Unknown,
        },
    }
}

/// heartbeat の字の class（off だけ `hb-off`）。
pub fn hbm_class(heartbeat: &Reading<&'static str>) -> &'static str {
    match heartbeat {
        Reading::Known("off") => HBM_OFF,
        _ => HBM,
    }
}

/// 群の今の口座（電文の groups の中で group が同じ GroupCard の row の account・groups が Unknown か群が無ければ None）。
pub fn current<'a>(doc: &'a AccountDoc, group: &str) -> Option<&'a str> {
    match &doc.groups {
        Reading::Known(cards) => cards
            .iter()
            .find(|g| g.row.group == group)
            .map(|g| g.row.account.as_str()),
        Reading::Unknown => None,
    }
}

/// accounts の欄（席の口座と、群の今の口座と同じかの印）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acc {
    /// 席の口座（席が Unknown か card の account が無ければ None で「―」を出す）。
    pub account: Option<String>,
    /// 同じなら ✓・違えば !・どちらかが分からなければ None（読めない一致を不一致と出さない・要件 NFR2）。
    pub mark: Option<Sign>,
}

pub fn acc(doc: &AccountDoc, project: &ProjectRow) -> Acc {
    let account = match &project.seat {
        Reading::Known(card) => card.account.clone(),
        Reading::Unknown => None,
    };
    let now = project.group.as_deref().and_then(|g| current(doc, g));
    let mark = match (&account, now) {
        (Some(a), Some(c)) => Some(if a == c { OK } else { NG }),
        _ => None,
    };
    Acc { account, mark }
}

/// 開くの欄（board を持つ project は開く button と URL・持たない project は押せない字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Open {
    New(String),
    NotYet,
}

impl Open {
    /// 語の鍵。
    pub fn key(&self) -> &'static str {
        match self {
            Open::New(_) => OPEN_NEW_KEY,
            Open::NotYet => NOT_YET_KEY,
        }
    }
}

pub fn open(project: &ProjectRow, mode: Mode) -> Open {
    open_url(project, mode).map_or(Open::NotYet, Open::New)
}

/// 表の 1 行。
#[derive(Debug, Clone, PartialEq)]
pub struct ProjLine {
    /// 電文の projects の中の位置（0 から）。
    pub index: usize,
    /// 行の class（`prow sev-<重さ>`）。
    pub class: String,
    pub name: String,
    pub group: Option<String>,
    pub need: Need,
    /// 決定待ち（この便は「―」）。
    pub wait: &'static str,
    /// 未反映（この便は「―」）。
    pub unref: &'static str,
    pub led: Led,
    pub runs: [Run; 4],
    pub orch: Orch,
    pub acc: Acc,
    pub open: Open,
}

/// 群の見出しの行（群の名と今の口座・群の無い project の見出しは名が None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupHead {
    pub group: Option<String>,
    pub current: Option<String>,
}

/// 見出しと行（group のほかの並べは見出しが無い 1 つ）。
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub head: Option<GroupHead>,
    pub rows: Vec<ProjLine>,
}

/// 表（並べ方と束の並び）。
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub sort: PSort,
    pub groups: Vec<Group>,
}

impl Table {
    /// 行の電文の中の位置（並びの順）。
    pub fn order(&self) -> Vec<usize> {
        self.groups
            .iter()
            .flat_map(|g| g.rows.iter().map(|r| r.index))
            .collect()
    }
}

/// block の中身（口が読めない・まだ読んでいない・本文が電文として読めないときは測れていないと理由の 1 行）。
pub fn content(fetched: &Fetched, sort: PSort, mode: Mode) -> Body<Table> {
    match super::doc(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(doc) if doc.projects.is_empty() => Body::Empty(NO_ROWS),
        Ok(doc) => Body::Filled(table(&doc, sort, mode)),
    }
}

/// 群の並び（電文の groups の順・groups に無い群はその後に projects の順で）。
pub fn group_order(doc: &AccountDoc) -> Vec<String> {
    let mut out: Vec<String> = match &doc.groups {
        Reading::Known(cards) => cards.iter().map(|g| g.row.group.clone()).collect(),
        Reading::Unknown => Vec::new(),
    };
    for g in doc.projects.iter().filter_map(|p| p.group.as_ref()) {
        if !out.contains(g) {
            out.push(g.clone());
        }
    }
    out
}

/// 電文を表に組む（同じ値は電文の projects の順・group の並べは行の無い見出しを出さない）。
pub fn table(doc: &AccountDoc, sort: PSort, mode: Mode) -> Table {
    let ps = &doc.projects;
    let all = || 0..ps.len();
    let heads: Vec<(Option<GroupHead>, Vec<usize>)> = match sort {
        PSort::Need => {
            let mut idx: Vec<usize> = all().collect();
            idx.sort_by_key(|&i| need_rank(&ps[i]));
            vec![(None, idx)]
        }
        PSort::Judge => {
            let mut idx: Vec<usize> = all().collect();
            idx.sort_by_key(|&i| judge_rank(&ps[i]));
            vec![(None, idx)]
        }
        PSort::Unref => vec![(None, all().collect())],
        PSort::Group => {
            let mut out: Vec<(Option<GroupHead>, Vec<usize>)> = group_order(doc)
                .into_iter()
                .map(|name| {
                    let idx = all()
                        .filter(|&i| ps[i].group.as_ref() == Some(&name))
                        .collect();
                    let head = GroupHead {
                        current: current(doc, &name).map(str::to_string),
                        group: Some(name),
                    };
                    (Some(head), idx)
                })
                .collect();
            let none = all().filter(|&i| ps[i].group.is_none()).collect();
            out.push((
                Some(GroupHead {
                    group: None,
                    current: None,
                }),
                none,
            ));
            out
        }
    };
    let groups = heads
        .into_iter()
        .filter(|(_, idx)| !idx.is_empty())
        .map(|(head, idx)| Group {
            head,
            rows: idx.into_iter().map(|i| row(doc, i, mode)).collect(),
        })
        .collect();
    Table { sort, groups }
}

/// 電文の projects の i 行目を表の行にする。
pub fn row(doc: &AccountDoc, index: usize, mode: Mode) -> ProjLine {
    let p = &doc.projects[index];
    let need = need(p);
    ProjLine {
        index,
        class: format!("{PROW} sev-{}", need.sev),
        name: p.name.clone(),
        group: p.group.clone(),
        need,
        wait: NONE_MARK,
        unref: NONE_MARK,
        led: led(p),
        runs: runs(&p.runs),
        orch: orch(p),
        acc: acc(doc, p),
        open: open(p, mode),
    }
}

/// 表を組む関数を view が使う（口は account の mod.rs の PATH・本文は doc で読む）。
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
        BLOCK, C_ACC, C_LED, C_NEED, C_OPEN, C_ORCH, C_PN, C_RUN, C_UN2, C_WAIT, COLUMNS, Group,
        GroupHead, HROW, Led, NONE_MARK, NOT_YET_CLASS, Open, Orch, PGH, PSort, PTAB, ProjLine,
        RC4, TKM, Table, content, hbm_class, psort_of, with_psort,
    };
    use crate::account::PATH;
    use crate::account::windows;
    use crate::frame::Mode;
    use crate::project::{Body, UNKNOWN, body_view, section, state_icon, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, hs, term};

    /// 今の URL の query（読めなければ空）。
    fn search() -> String {
        window().location().search().unwrap_or_default()
    }

    /// URL の query を置き換える（頁は読み直さない）。
    fn replace(url: &str) {
        if let Ok(history) = window().history() {
            let _ = history.replace_state_with_url(
                &web_sys::wasm_bindgen::JsValue::NULL,
                "",
                Some(url),
            );
        }
    }

    pub fn view() -> AnyView {
        let fetched = crate::net::read(PATH);
        let sort = RwSignal::new(psort_of(&search()));
        let mode = use_context::<HelpCtx>().map(|c| c.mode);
        let body = move || {
            let m = mode.map_or(Mode::Beginner, |m| m.get());
            match fetched.with(|f| content(f, sort.get(), m)) {
                Body::Unmeasured(reason) => unmeasured(reason),
                Body::Empty(line) => body_view(Body::Empty(line)),
                Body::Filled(table) => table_view(table),
            }
        };
        section(BLOCK, sort_bar(sort), body.into_any())
    }

    fn sort_bar(sort: RwSignal<PSort>) -> AnyView {
        let buttons = PSort::ALL
            .into_iter()
            .map(|s| {
                let pressed = move || (sort.get() == s).to_string();
                let pick = move |_| {
                    replace(&with_psort(&search(), s));
                    sort.set(s);
                };
                view! {
                    <button type="button" data-psort=s.key() aria-pressed=pressed data-term=s.label_key() on:click=pick>
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

    fn table_view(table: Table) -> AnyView {
        let head = COLUMNS
            .into_iter()
            .map(|k| view! { <div class=format!("h-{k}")>{hs(k)}</div> })
            .collect_view();
        let groups = table.groups.into_iter().map(group_view).collect_view();
        view! {
            <div class=PTAB>
                <div class=HROW>{head}</div>
                {groups}
            </div>
        }
        .into_any()
    }

    fn group_view(group: Group) -> AnyView {
        let head = group.head.map(head_view);
        let rows = group.rows.into_iter().map(row_view).collect_view();
        view! { {head}{rows} }.into_any()
    }

    fn head_view(head: GroupHead) -> AnyView {
        let name = match head.group {
            Some(g) => {
                view! { <span class="pchip grp"><span data-t="">{g}</span></span> }.into_any()
            }
            None => view! { <span class="pchip grp">{NONE_MARK}</span> }.into_any(),
        };
        let now = match head.current {
            Some(a) => view! { <b class="mono">{a}</b> }.into_any(),
            None => state_icon(UNKNOWN),
        };
        view! {
            <div class=PGH>
                {name}
                <span class="small muted">{label("current_account")}" "{now}</span>
            </div>
        }
        .into_any()
    }

    fn row_view(row: ProjLine) -> AnyView {
        let need_word = match row.need.lead {
            Some(_) => format!("({}) {}", &row.need.key[3..], label(row.need.key)),
            None => label(row.need.key),
        };
        let need_icon = row.need.lead.is_none().then(|| state_icon(UNKNOWN));
        let runs = row
            .runs
            .iter()
            .map(|r| {
                view! {
                    <span class=r.class.clone()>
                        <span class="dot" aria-hidden="true"></span>
                        <span class="n num">{r.text.clone()}</span>
                    </span>
                }
            })
            .collect_view();
        let acc_mark = row
            .acc
            .mark
            .map(|s| view! { <span class=s.class aria-hidden="true">{s.glyph}</span> });
        let key = row.open.key();
        let open = match row.open {
            Open::New(url) => {
                let project = row.name.clone();
                let click = move |_| windows::open(&project, &url);
                view! {
                    <button type="button" class="btn" on:click=click>
                        <span class="lg">{label(key)}</span>
                        <span class="sh">"↗"</span>
                    </button>
                }
                .into_any()
            }
            Open::NotYet => view! {
                <span class=NOT_YET_CLASS title=label(key)>
                    <span class="lg">{label(key)}</span>
                    <span class="sh">{NONE_MARK}</span>
                </span>
            }
            .into_any(),
        };
        view! {
            <div class=row.class.clone()>
                <div class=C_PN>
                    <b data-t="">{row.name.clone()}</b>
                    <span class="small muted" data-t="">{row.group.clone().unwrap_or_default()}</span>
                </div>
                <div class=C_NEED>
                    <span class="l1">{need_icon}<b>{need_word}</b></span>
                    <span class="l2">{row.need.line.clone().unwrap_or_default()}</span>
                </div>
                <div class=C_WAIT><span class="l1"><b class="num">{row.wait}</b></span></div>
                <div class=C_UN2 data-term="l_unref"><span class="l1"><b class="num">{row.unref}</b></span></div>
                <div class=C_LED>{led_view(row.led)}</div>
                <div class=C_RUN><span class=RC4 data-term="runs4">{runs}</span></div>
                <div class=C_ORCH>{orch_view(row.orch)}</div>
                <div class=C_ACC>
                    <span class="mono">{row.acc.account.clone().unwrap_or_else(|| NONE_MARK.to_string())}</span>
                    {acc_mark}
                </div>
                <div class=C_OPEN>{open}</div>
            </div>
        }
        .into_any()
    }

    fn led_view(led: Led) -> AnyView {
        let j = led.judge;
        let task = led.task.map(|n| view! { <b class="num">{n}</b> });
        let net = led.net24.map(|n| {
            let aria = format!("{} {}", n.word, n.text);
            view! {
                <span class=n.class aria-label=aria>
                    <b>{n.arrow}</b>
                    <span class="num">{n.text}</span>
                </span>
            }
        });
        view! {
            <span class="l1">
                <span class=j.class data-term=j.key tabindex="0">
                    <b aria-hidden="true">{j.symbol}</b>
                    <span>{label(j.key)}</span>
                </span>
                {task}
                {net}
            </span>
        }
        .into_any()
    }

    fn orch_view(orch: Orch) -> AnyView {
        let hb = hbm_class(&orch.heartbeat);
        let tick = match orch.tick {
            Reading::Known(s) => {
                view! { <span class=s.class aria-hidden="true">{s.glyph}</span> }.into_any()
            }
            Reading::Unknown => state_icon(UNKNOWN),
        };
        let hb_word = match orch.heartbeat {
            Reading::Known(w) => w.into_any(),
            Reading::Unknown => state_icon(UNKNOWN),
        };
        view! {
            {state_icon(orch.state)}
            <span class=TKM>{tick}{term("tick_health", "tick".to_string())}</span>
            <span class=hb>"hb "{hb_word}</span>
        }
        .into_any()
    }
}
