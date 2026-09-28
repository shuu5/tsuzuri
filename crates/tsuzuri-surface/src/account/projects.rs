//! account board の各 project の表の block（便 h-frame の枠・便 h-proj の中身）: 見本の projects の tab の `#ptab`
//! （account/index.html の projTable・PSORTS）。
//! 行は電文の projects の 1 行ずつで 9 列。並べ方は 4 つ（need・group・judge・unref）で URL の query の `psort=` に残し、
//! 並べ方の押しは履歴の 1 歩にする（見本の pushState・行 h-sort-hist）。
//! 決定待ちは電文の台帳の open の問いの数、未反映は電文の台帳の未反映の数で、読めない種類が在れば
//! project board の指標の段と同じ Unref の形で測れていないの印を添える（部分の和）。台帳が Unknown の行はどちらも「―」。
//! 決定待ちの 2 段目は電文の次の一手の束の承認の件数、未反映の 2 段目は電文の台帳の読めた種類ごとの件数（見本の unrefBreak）。
//! 未反映の列の見出しは電文の projects の未反映の和（見本の Σ・行 h-acct-rest）。
//! 並べ・行の値・class は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。
//! 5 つの欄と群の見出しの chip の hover の card は account の cards の module が組む（行 h-acct-split で割った）。

use std::cmp::Reverse;

use tsuzuri_contract::account::{AccountDoc, ProjectRow, RunCounts};
use tsuzuri_contract::board::{LedgerJudge, NextMove, Reading};
use tsuzuri_contract::stats::{CheckResult, LedgerStats, UnreflectedKind};

use super::cards::{RowCards, grp_card, row_cards};
use super::windows::{NOT_YET_KEY, OPEN_NEW_KEY, open_url};
use crate::frame::{self, Block, Mode};
use crate::project::Body;
use crate::project::ledger::{
    JUDGES, Judge, Net, SPARK_H, SPARK_W, Unref, age, judge, kind_name, net, spark, spark_svg,
};
use crate::project::next::{UNJUDGED_LINE, big, key as next_key, unjudged};
use crate::project::seat::{NG, OK, Sign, state_value, top};
use crate::project::{UNKNOWN, state_key};
use crate::view::Fetched;
use crate::vocab::label;
use crate::widgets::hover::Card;

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
/// なしを判じなかった行は読めない行と同じ測れていないで、1 行は project board の測れていないの箱と同じ字。
pub fn need(project: &ProjectRow) -> Need {
    match &project.next {
        Reading::Known(step) if unjudged(step) => Need {
            lead: None,
            key: state_key(UNKNOWN),
            sev: sev(None),
            line: Some(UNJUDGED_LINE.to_string()),
        },
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

/// need の並べの位（lead の NextMove の ALL の中の位置・next が Unknown となしを判じなかった行はその後ろ）。
pub fn need_rank(project: &ProjectRow) -> usize {
    match &project.next {
        Reading::Known(step) if unjudged(step) => NextMove::ALL.len(),
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

/// 決定待ち（台帳が Known なら open の問いの数・Unknown は None）。
pub fn wait_of(project: &ProjectRow) -> Option<u32> {
    match &project.ledger {
        Reading::Known(s) => Some(s.open.question),
        Reading::Unknown => None,
    }
}

/// 未反映（数は電文の数そのまま・読めない種類の名は電文の順・project board の指標の段の panel の欄 unref と同じ組み方）。
pub fn unref_count(s: &LedgerStats) -> Unref {
    Unref {
        count: s.unreflected,
        unknown: s
            .unreflected_unknown
            .iter()
            .map(|k| kind_name(*k))
            .collect(),
    }
}

/// 未反映（台帳が Known なら unref_count・Unknown は None）。
pub fn unref_of(project: &ProjectRow) -> Option<Unref> {
    match &project.ledger {
        Reading::Known(s) => Some(unref_count(s)),
        Reading::Unknown => None,
    }
}

/// 数の欄の字（None は「―」）。
pub fn count_text(n: Option<u32>) -> String {
    n.map_or_else(|| NONE_MARK.to_string(), |v| v.to_string())
}

/// 決定待ちの数の class（1 以上は on）。
pub fn wait_class(n: Option<u32>) -> &'static str {
    match n {
        Some(v) if v > 0 => "num on",
        _ => "num",
    }
}

/// 未反映の列の class（count が 1 以上は on）。
pub fn unref_class(unref: Option<&Unref>) -> &'static str {
    match unref {
        Some(u) if u.count > 0 => "c-un2 on",
        _ => C_UN2,
    }
}

/// unref の並べの位（count の多い順・台帳が Unknown はその後ろ）。
pub fn unref_rank(project: &ProjectRow) -> Reverse<i64> {
    Reverse(unref_of(project).map_or(-1, |u| i64::from(u.count)))
}

/// 決定待ちの 2 段目の束の承認の件数（次の一手が Known で束の承認を判じたならその件数・判じなかったか Unknown は None）。
/// 件数は中核の束の承認の判じのまま（当たらないは 0・面で数え直さない）。
pub fn batch_of(project: &ProjectRow) -> Option<u32> {
    match &project.next {
        Reading::Known(step) => step
            .checks
            .iter()
            .find(|c| c.kind == NextMove::BatchApproval)
            .filter(|c| c.result != CheckResult::NotJudged)
            .map(|c| c.count),
        Reading::Unknown => None,
    }
}

/// 束の承認の件数の字（見本の `束 <件数>`・None は空の字）。
pub fn batch_text(batch: Option<u32>) -> String {
    batch.map_or_else(String::new, |n| format!("束 {n}"))
}

/// 未反映の種類の見出しの語の鍵の接頭（鍵は接頭と kind_name の字）。
pub const UNREF_KIND_KEY: &str = "unref:";

/// 未反映の種類の見出し（語の辞書の label・見本の unrefBreak の字）。
pub fn kind_label(kind: UnreflectedKind) -> String {
    label(&format!("{UNREF_KIND_KEY}{}", kind_name(kind)))
}

/// 未反映の 2 段目（読めた種類の見出しと件数・読めない種類の見出し・どちらも電文の順）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnrefKinds {
    pub known: Vec<(String, u32)>,
    pub unknown: Vec<String>,
}

/// 未反映の種類ごとの件数（読めた種類は 1 以上だけ・見本の unrefBreak）。
pub fn unref_break(s: &LedgerStats) -> UnrefKinds {
    UnrefKinds {
        known: s
            .unreflected_kinds
            .iter()
            .filter(|k| k.count > 0)
            .map(|k| (kind_label(k.kind), k.count))
            .collect(),
        unknown: s
            .unreflected_unknown
            .iter()
            .map(|k| kind_label(*k))
            .collect(),
    }
}

/// 未反映の 2 段目（台帳が Known なら unref_break・Unknown は空）。
pub fn kinds_of(project: &ProjectRow) -> UnrefKinds {
    match &project.ledger {
        Reading::Known(s) => unref_break(s),
        Reading::Unknown => UnrefKinds::default(),
    }
}

/// 未反映の列の見出しの和（見本の Σ・台帳が Known の行の和と、部分の和かと、電文の projects の行の数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnrefSum {
    pub total: u32,
    /// 台帳が Unknown の行か読めない種類の在る行が在れば真（測れていないの印を添える）。
    pub partial: bool,
    pub projects: usize,
}

impl UnrefSum {
    pub fn text(&self) -> String {
        format!("Σ {}", self.total)
    }

    pub fn title(&self) -> String {
        format!("{} project の合計", self.projects)
    }
}

/// 電文の projects の未反映の和。
pub fn unref_sum(doc: &AccountDoc) -> UnrefSum {
    let mut sum = UnrefSum {
        total: 0,
        partial: false,
        projects: doc.projects.len(),
    };
    for p in &doc.projects {
        match &p.ledger {
            Reading::Known(s) => {
                sum.total = sum.total.saturating_add(s.unreflected);
                sum.partial |= !s.unreflected_unknown.is_empty();
            }
            Reading::Unknown => sum.partial = true,
        }
    }
    sum
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
    /// 決定待ち（台帳の open の問いの数・台帳が Unknown は None）。
    pub wait: Option<u32>,
    /// 決定待ちの 2 段目の束の承認の件数（判じなかったか次の一手が Unknown は None）。
    pub batch: Option<u32>,
    /// 未反映（数と読めない種類・台帳が Unknown は None）。
    pub unref: Option<Unref>,
    /// 未反映の 2 段目の読めた種類ごとの件数と読めない種類の見出し。
    pub kinds: UnrefKinds,
    pub led: Led,
    pub runs: [Run; 4],
    pub orch: Orch,
    pub acc: Acc,
    pub open: Open,
    /// 詳しくの段（行を押すと開く）。
    pub more: More,
    /// 5 つの欄の hover の card。
    pub cards: RowCards,
}

/// 詳しくの台帳の項の語の鍵（見本の ledMore の順）。
pub const MORE_LED: [&str; 6] = ["l_ready", "l_blocked", "l_memo", "l_lead", "l_stale", "l_net7"];

/// 行を押しても開閉しない要素の selector（button・link・「?」）。
pub const NO_TOGGLE: &str = "button, a, .q";

/// 詳しくの台帳の項（見本の ledMore）。
#[derive(Debug, Clone, PartialEq)]
pub struct LedMore {
    /// MORE_LED の鍵と字の組（l_net7 の字は net7 の text）。
    pub items: [(&'static str, String); 6],
    /// 7 日の純減の矢印と数。
    pub net7: Net,
    /// 14 日の sparkline の svg の字。
    pub spark: String,
}

/// 詳しくの段の値。
#[derive(Debug, Clone, PartialEq)]
pub struct More {
    /// 台帳が Unknown なら None。
    pub ledger: Option<LedMore>,
    /// この project の session（状態の値と名・電文の順・名が空の行は除く）。
    pub sessions: Vec<(&'static str, String)>,
    /// 口座の移動の履歴の数（席と履歴が読めなければ None）。
    pub hist: Option<usize>,
}

/// 電文の projects の i 行目の詳しくの段。
pub fn more(doc: &AccountDoc, index: usize) -> More {
    let p = &doc.projects[index];
    let ledger = match &p.ledger {
        Reading::Known(s) => {
            let net7 = net(s.net_drop_7d);
            let values = [
                s.ready.to_string(),
                s.blocked.to_string(),
                s.open.memo.to_string(),
                age(s.lead.as_ref().map(|l| l.p50)),
                s.stale.to_string(),
                net7.text.clone(),
            ];
            let mut it = values.into_iter();
            Some(LedMore {
                items: MORE_LED.map(|k| (k, it.next().unwrap_or_default())),
                net7,
                spark: spark_svg(&spark(&s.days, SPARK_W, SPARK_H)),
            })
        }
        Reading::Unknown => None,
    };
    let sessions = doc
        .sessions
        .iter()
        .filter(|s| s.project == p.name && !s.name.is_empty())
        .map(|s| (state_value(s.state), s.name.clone()))
        .collect();
    let hist = match &p.seat {
        Reading::Known(card) => match &card.moves {
            Reading::Known(m) => Some(m.len()),
            Reading::Unknown => None,
        },
        Reading::Unknown => None,
    };
    More {
        ledger,
        sessions,
        hist,
    }
}

/// 字を字数に収める（見本の fitLines の 1 歩: 先頭から字数の字・末尾の空白と「・」「（」「(」を除き … を足す）。
pub fn fit_cut(text: &str, count: usize) -> String {
    if text.chars().count() <= count {
        return text.to_string();
    }
    let head: String = text.chars().take(count).collect();
    let trimmed = head.trim_end_matches(|c: char| c.is_whitespace() || matches!(c, '・' | '（' | '('));
    format!("{trimmed}…")
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
    /// 見出しの群の chip の card（群の名の見出しだけ・見本の group の枝）。
    pub card: Option<Card>,
}

/// 表（並べ方と束の並び）。
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub sort: PSort,
    /// 未反映の列の見出しの和。
    pub sum: UnrefSum,
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
        PSort::Unref => {
            let mut idx: Vec<usize> = all().collect();
            idx.sort_by_key(|&i| unref_rank(&ps[i]));
            vec![(None, idx)]
        }
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
            card: head
                .as_ref()
                .and_then(|h| h.group.as_deref())
                .and_then(|name| grp_card(doc, name)),
            head,
            rows: idx.into_iter().map(|i| row(doc, i, mode)).collect(),
        })
        .collect();
    Table {
        sort,
        sum: unref_sum(doc),
        groups,
    }
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
        wait: wait_of(p),
        batch: batch_of(p),
        unref: unref_of(p),
        kinds: kinds_of(p),
        led: led(p),
        runs: runs(&p.runs),
        orch: orch(p),
        acc: acc(doc, p),
        open: open(p, mode),
        more: more(doc, index),
        cards: row_cards(doc, p),
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
    use std::collections::BTreeMap;

    use leptos::ev;
    use leptos::html::Span;
    use leptos::prelude::*;
    use tsuzuri_contract::board::Reading;
    use web_sys::wasm_bindgen::JsCast;

    use super::{
        Acc, BLOCK, C_ACC, C_LED, C_NEED, C_OPEN, C_ORCH, C_PN, C_RUN, C_WAIT, COLUMNS, Group,
        GroupHead, HROW, Led, NO_TOGGLE, NONE_MARK, NOT_YET_CLASS, Open, Orch, PGH, PSort, PTAB,
        ProjLine, RC4, Run, TKM, Table, UnrefKinds, UnrefSum, batch_text, content, count_text,
        fit_cut, hbm_class, psort_of, unref_class, wait_class, with_psort,
    };
    use crate::account::PATH;
    use crate::account::windows;
    use crate::frame::Mode;
    use crate::project::ledger::{Net, Unref};
    use crate::project::{Body, UNKNOWN, body_view, section, state_icon, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, hs, shows_internal, term};
    use crate::widgets::hover::{Card, attach};

    /// 行ごとの詳しくの開き閉じ（project の名ごと・頁の一生の間だけ）。
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
        let sort = RwSignal::new(psort_of(&search()));
        let mode = use_context::<HelpCtx>().map(|c| c.mode);
        let opened: Opened = RwSignal::new(BTreeMap::new());
        // 記録に値が無い行は mode から初めの値を取る（見本の isOpen・経験者は開く）。
        let expert = Signal::derive(move || mode.is_some_and(|m| shows_internal(m.get())));
        let body = move || {
            let m = mode.map_or(Mode::Beginner, |m| m.get());
            match fetched.with(|f| content(f, sort.get(), m)) {
                Body::Unmeasured(reason) => unmeasured(reason),
                Body::Empty(line) => body_view(Body::Empty(line)),
                Body::Filled(table) => table_view(table, opened, expert),
            }
        };
        section(BLOCK, sort_bar(sort), body.into_any())
    }

    fn sort_bar(sort: RwSignal<PSort>) -> AnyView {
        let buttons = PSort::ALL
            .into_iter()
            .map(|s| {
                let pressed = move || (sort.get() == s).to_string();
                // 今の並べの押しは履歴に積まない。
                let pick = move |_| {
                    if sort.get_untracked() == s {
                        return;
                    }
                    push(&with_psort(&search(), s));
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

    fn table_view(table: Table, opened: Opened, expert: Signal<bool>) -> AnyView {
        let sum = table.sum;
        let head = COLUMNS
            .into_iter()
            .map(|k| {
                let total = (k == "l_unref").then(|| sum_view(sum));
                view! { <div class=format!("h-{k}")>{hs(k)}{total}</div> }
            })
            .collect_view();
        let groups = table
            .groups
            .into_iter()
            .map(|g| group_view(g, opened, expert))
            .collect_view();
        view! {
            <div class=PTAB>
                <div class=HROW>{head}</div>
                {groups}
            </div>
        }
        .into_any()
    }

    fn group_view(group: Group, opened: Opened, expert: Signal<bool>) -> AnyView {
        let head = group.head.map(|h| head_view(h, group.card));
        let rows = group
            .rows
            .into_iter()
            .map(|r| row_view(r, opened, expert))
            .collect_view();
        view! { {head}{rows} }.into_any()
    }

    /// card が在れば要素に付ける（席の無い行の欄と群の無い見出しは card を持たない・Option の card の directive の口）。
    fn attach_some(el: web_sys::Element, card: Option<Card>) {
        if let Some(card) = card {
            attach(el, card);
        }
    }

    fn head_view(head: GroupHead, card: Option<Card>) -> AnyView {
        let name = match head.group {
            Some(g) => view! {
                <span class="pchip grp" tabindex="0" use:attach_some=card><span data-t="">{g}</span></span>
            }
            .into_any(),
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

    fn row_view(row: ProjLine, opened: Opened, expert: Signal<bool>) -> AnyView {
        let need_word = match row.need.lead {
            Some(_) => format!("({}) {}", &row.need.key[3..], label(row.need.key)),
            None => label(row.need.key),
        };
        let need_icon = row.need.lead.is_none().then(|| state_icon(UNKNOWN));
        let more = more_view(&row);
        // 2 段目は描いた後と窓の幅が変わった後に幅へ収める（見本の fitLines）。
        let l2 = NodeRef::<Span>::new();
        let fit = {
            let line = row.need.line.clone().unwrap_or_default();
            move || {
                if let Some(el) = l2.get_untracked() {
                    fit_line(&el, &line);
                }
            }
        };
        request_animation_frame(fit.clone());
        let resize = window_event_listener(ev::resize, move |_| fit());
        on_cleanup(move || resize.remove());
        // 記録の無い行は mode の初めの値（見本の isOpen）。
        let name = row.name.clone();
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
        // 行のどこを押しても開閉する（button・link・「?」の中は除く）。
        let toggle = move |ev: web_sys::MouseEvent| {
            let inside = ev
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .and_then(|el| el.closest(NO_TOGGLE).ok().flatten());
            if inside.is_some() {
                return;
            }
            let now = untrack(&is_open);
            opened.update(|m| {
                m.insert(name.clone(), !now);
            });
        };
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
            <div class=class aria-expanded=expanded on:click=toggle>
                <div class=C_PN>
                    <b data-t="">{row.name.clone()}</b>
                    <span class="small muted" data-t="">{row.group.clone().unwrap_or_default()}</span>
                </div>
                <div class=C_NEED tabindex="0" use:attach=row.cards.need.clone()>
                    <span class="l1">{need_icon}<b>{need_word}</b></span>
                    <span class="l2" node_ref=l2>{row.need.line.clone().unwrap_or_default()}</span>
                </div>
                <div class=C_WAIT>
                    <span class="l1"><b class=wait_class(row.wait)>{count_text(row.wait)}</b></span>
                    <span class="l2 small muted">{batch_text(row.batch)}</span>
                </div>
                <div class=unref_class(row.unref.as_ref()) data-term="l_unref">{unref_view(row.unref.as_ref(), &row.kinds)}</div>
                <div class=C_LED tabindex="0" use:attach=row.cards.led.clone()>{led_view(row.led)}</div>
                <div class=C_RUN tabindex="0" use:attach=row.cards.runs.clone()>{runs_view(&row.runs)}</div>
                <div class=C_ORCH use:attach_some=row.cards.orch.clone()>{orch_view(row.orch)}</div>
                <div class=C_ACC use:attach_some=row.cards.acc.clone()>{acc_view(&row.acc)}</div>
                <div class=C_OPEN>{open}</div>
                {more}
            </div>
        }
        .into_any()
    }

    /// 詳しくの段（台帳の項と sparkline・session・口座の履歴・狭い幅で隠れる 4 列の値・見本の projTable の more）。
    fn more_view(row: &ProjLine) -> AnyView {
        let more = &row.more;
        let ledger = match &more.ledger {
            Some(l) => {
                let items = l
                    .items
                    .clone()
                    .into_iter()
                    .map(|(key, text)| {
                        let value = if key == "l_net7" {
                            net_view(&l.net7)
                        } else {
                            text.into_any()
                        };
                        view! { <span class="mi"><span class="lk">{hs(key)}</span><b class="num">{value}</b></span> }
                    })
                    .collect_view();
                view! {
                    {items}
                    <span class="mi sp" inner_html=l.spark.clone()></span>
                }
                .into_any()
            }
            None => view! { <span class="mi muted">{label("j_none")}</span> }.into_any(),
        };
        let list = (!more.sessions.is_empty()).then(|| {
            let names = more
                .sessions
                .iter()
                .map(|(state, name)| view! { {state_icon(state)}<span class="mono">{name.clone()}</span> })
                .collect_view();
            view! { <span class="mi slist">{names}</span> }
        });
        let hist = more
            .hist
            .map_or_else(|| NONE_MARK.to_string(), |n| n.to_string());
        view! {
            <div class="c-more">
                {ledger}
                <span class="mi"><span class="lk">{hs("session")}</span><b class="num">{more.sessions.len()}</b></span>
                {list}
                <span class="mi"><span class="lk">{hs("acct_hist")}</span><b class="num">{hist}</b></span>
                <span class="mi m-run" use:attach=row.cards.runs.clone()>{runs_view(&row.runs)}</span>
                <span class="mi m-orch" use:attach_some=row.cards.orch.clone()>{orch_view(row.orch.clone())}</span>
                <span class="mi m-acc" use:attach_some=row.cards.acc.clone()>{acc_view(&row.acc)}</span>
                <span class="mi m-led" use:attach=row.cards.led.clone()>{led_view(row.led.clone())}</span>
            </div>
        }
        .into_any()
    }

    /// 未反映の欄（数と、読めない種類が在れば測れていないの印・台帳が Unknown は「―」）。
    /// 2 段目は読めた種類の見出しと件数の後に読めない種類の見出しと印（見出しは語の辞書・span ごとに後へ空白）。
    fn unref_view(unref: Option<&Unref>, kinds: &UnrefKinds) -> AnyView {
        let Some(u) = unref else {
            return view! { <span class="l1"><b class="num">{NONE_MARK}</b></span> }.into_any();
        };
        let mark = (!u.unknown.is_empty()).then(|| state_icon(UNKNOWN));
        let counts = kinds
            .known
            .iter()
            .map(|(k, n)| view! { <span>{k.clone()}" "{*n}</span>" " })
            .collect_view();
        let unknown = kinds
            .unknown
            .iter()
            .map(|k| view! { <span>{k.clone()}" "{state_icon(UNKNOWN)}</span>" " })
            .collect_view();
        view! {
            <span class="l1"><b class="num">{u.count}{mark}</b></span>
            <span class="l2 small muted">{counts}{unknown}</span>
        }
        .into_any()
    }

    /// 未反映の列の見出しの和（見本の Σ・部分の和なら測れていないの印）。
    fn sum_view(sum: UnrefSum) -> AnyView {
        let mark = sum.partial.then(|| state_icon(UNKNOWN));
        view! { <span class="small muted num" title=sum.title()>{sum.text()}{mark}</span> }
            .into_any()
    }

    /// run の 4 列（wait・run・stop・land の順）。
    fn runs_view(runs: &[Run]) -> AnyView {
        let items = runs
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
        view! { <span class=RC4 data-term="runs4">{items}</span> }.into_any()
    }

    /// 席の口座と、群の今の口座と同じかの印。
    fn acc_view(acc: &Acc) -> AnyView {
        let mark = acc
            .mark
            .map(|s| view! { <span class=s.class aria-hidden="true">{s.glyph}</span> });
        view! {
            <span class="mono">{acc.account.clone().unwrap_or_else(|| NONE_MARK.to_string())}</span>
            {mark}
        }
        .into_any()
    }

    fn led_view(led: Led) -> AnyView {
        let j = led.judge;
        let task = led.task.map(|n| view! { <b class="num">{n}</b> });
        let net = led.net24.map(|n| net_view(&n));
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

    /// 2 段目を幅に収める（元の字に戻し、はみ出す間 1 字ずつ減らして … で切り、切ったら元の字を読み上げに置く）。
    fn fit_line(el: &web_sys::HtmlElement, line: &str) {
        el.set_inner_text(line);
        let _ = el.remove_attribute("aria-label");
        let over = |e: &web_sys::HtmlElement| e.scroll_width() > e.client_width() + 1;
        if !over(el) {
            return;
        }
        for count in (1..line.chars().count()).rev() {
            el.set_inner_text(&fit_cut(line, count));
            if !over(el) {
                break;
            }
        }
        let _ = el.set_attribute("aria-label", line);
    }
}
