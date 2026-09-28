//! account board の session の表の block（便 h-frame の枠・便 h-sess の中身）: 見本の session の tab の 1 つ目の panel
//! （account/index.html の sessRow・sessTable・stageCell と ui.js の tkhbHTML）。
//! 行は電文の sessions の 1 行ずつ。並べ方は 4 つ（project・account・stage・elapsed）で URL の query の `sort=` に残し、
//! 並べ方の押しは履歴の 1 歩にする（見本の pushState・行 h-sort-hist・幅の押しは見本の setSpan と同じく置き換える）。
//! 稼働の記録は着地済みの seat の module の幅と矩形と SVG を使い、窓の右端は電文の at。
//! orchestrator の行の合図（tick の健康・heartbeat・退避までの残り秒・移動待ち）は電文の projects の同じ名の行から引く。
//! orchestrator の行の停止の切り替え（button と行の下の確かめの段）は heartbeat の module が決める（便 h-hb）。
//! 並べ・束・行の値・合図の class は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::{AccountDoc, ProjectRow, SessionLine};
use tsuzuri_contract::board::{Reading, Stage};
use tsuzuri_contract::seat::{SeatSpan, SeatState};
use tsuzuri_contract::surface::SeatRole;

use super::cards::orch_card;
use super::heartbeat::{Toggle, toggle};
use crate::frame::{self, Block};
use crate::project::pipeline::age;
use crate::project::seat::{OK, Sign, Span, hmd, rects, span_ticks, state_value, strip_svg, top};
use crate::project::{Body, state_key};
use crate::view::Fetched;
use crate::vocab::label;
use crate::widgets::hover::Card;

pub const BLOCK: Block = Block {
    id: "sessions",
    heading: "sessions",
    class: "panel",
};

/// block の中身（中身の関数がまだ無い: 測れていない）。
/// 便 h-frame の歯が pin するので変えない。中身は `content` が組む。
pub fn body(fetched: &Fetched) -> Body<()> {
    super::pending(fetched)
}

/// 並べ方を残す URL の query の鍵。
pub const SORT_KEY: &str = "sort";

/// 表の class（見本の `.sess`）。
pub const SESS: &str = "sess";

/// 見出しの行の class（見本の `.srow.hrow`）。
pub const HROW: &str = "srow hrow";

/// 見出しの行の列の語の鍵（この順・最後の列は稼働の記録）。
pub const COLUMNS: [&str; 7] = [
    "project", "role", "session", "accounts", "stage", "elapsed", "history",
];

/// 束の見出しの class（見本の `.ghead`）。
pub const GHEAD: &str = "ghead";

/// 列の class（見本の `.c-proj`・`.c-role`・`.c-stage`・`.c-strip.strip24`）。
pub const C_PROJ: &str = "c-proj";
pub const C_ROLE: &str = "c-role";
pub const C_STAGE: &str = "c-stage";
pub const C_STRIP: &str = "c-strip strip24";

/// 稼働の記録の見出しの欄の class（見本の `.c-hh`・語の下に目盛の段）。
pub const C_HH: &str = "c-hh";

/// 目盛の段の class（見本の `.saxis`）。
pub const SAXIS: &str = "saxis";

/// 狭い幅の目盛の段の class（見本の `.saxis-m`・見出しの行が隠れる幅でだけ出る）。
pub const SAXIS_M: &str = "saxis-m";

/// 合図の行の class（見本の `.tkhb`）。
pub const TKHB: &str = "tkhb";

/// 経過と稼働の記録が無い欄の字。
pub const NONE_MARK: &str = "―";

/// 電文の sessions が 0 行のときの 1 行（測れて 0 件・測れていないと分ける）。
pub const NO_ROWS: &str = "session の行が 0 件（電文の sessions が空）";

/// 止まった run の段（見本の acct.js の runStopped の段の写し・Questioned と Failed と Stopped）。
/// 中核の next_step の STALLED_STAGES（Failed と Stopped・行 c-next-stall）は次の一手の止まっている走行の段で、
/// Questioned を質問の側で数えるので値が違う（面の crate は中核の crate に依存しない）。
pub const STALLED_STAGES: [Stage; 3] = [Stage::Questioned, Stage::Failed, Stage::Stopped];

/// session の表の並べ方（見本の SORTS の順）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Project,
    Account,
    Stage,
    Elapsed,
}

impl Sort {
    pub const ALL: [Sort; 4] = [Sort::Project, Sort::Account, Sort::Stage, Sort::Elapsed];

    /// URL と button の字。
    pub fn key(self) -> &'static str {
        match self {
            Sort::Project => "project",
            Sort::Account => "account",
            Sort::Stage => "stage",
            Sort::Elapsed => "elapsed",
        }
    }

    /// button の語の鍵。
    pub fn label_key(self) -> &'static str {
        match self {
            Sort::Project => "sort_project",
            Sort::Account => "sort_account",
            Sort::Stage => "sort_stage",
            Sort::Elapsed => "sort_elapsed",
        }
    }
}

/// URL の query から並べ方（`sort=stage` の形・無いか知らない値は project）。
pub fn sort_of(search: &str) -> Sort {
    frame::param(search, SORT_KEY)
        .and_then(|v| Sort::ALL.into_iter().find(|s| s.key() == v))
        .unwrap_or(Sort::Project)
}

/// 並べ方を選んだ後の URL の query（既定の project は `sort=` を消す・ほかは `sort=` に残す・ほかの値と順はそのまま）。
pub fn with_sort(search: &str, sort: Sort) -> String {
    match sort {
        Sort::Project => without_param(search, SORT_KEY),
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

/// 行の位（limit 0・silent 1・止まった run 2・wait 3・run 4・unknown 5・見本の rank）。
pub fn rank(line: &SessionLine) -> u8 {
    match line.state {
        SeatState::Limit => 0,
        SeatState::Silent => 1,
        _ if stalled(line) => 2,
        SeatState::Wait => 3,
        SeatState::Run => 4,
        SeatState::Unknown => 5,
    }
}

/// 止まった run（role が pipeline で段が止まった run の段）。
pub fn stalled(line: &SessionLine) -> bool {
    line.role == SeatRole::Pipeline && line.stage.is_some_and(|s| STALLED_STAGES.contains(&s))
}

/// stage の並べの束の語の鍵（位 0〜2 は止まっている・3 と 4 は動いている・5 は記録なし）。
pub fn band_key(rank: u8) -> &'static str {
    match rank {
        0..=2 => "stopped_group",
        3 | 4 => "moving_group",
        _ => "no_record",
    }
}

/// stage の並べの束（この順）。
pub const BANDS: [&str; 3] = ["stopped_group", "moving_group", "no_record"];

/// 役の字（行の class の `r-<役>`）。
pub fn role_word(role: SeatRole) -> &'static str {
    match role {
        SeatRole::Orchestrator => "orchestrator",
        SeatRole::Pipeline => "pipeline",
    }
}

/// 役の語の鍵。
pub fn role_key(role: SeatRole) -> &'static str {
    match role {
        SeatRole::Orchestrator => "role:orchestrator",
        SeatRole::Pipeline => "role:pipeline",
    }
}

/// 段の字（段の名は英語のまま面に出す）。
pub fn stage_word(stage: Stage) -> &'static str {
    match stage {
        Stage::Queued => "Queued",
        Stage::Blocked => "Blocked",
        Stage::Running => "Running",
        Stage::Gated => "Gated",
        Stage::Questioned => "Questioned",
        Stage::Failed => "Failed",
        Stage::Stopped => "Stopped",
        Stage::Landed => "Landed",
    }
}

/// orchestrator の行の合図（tick の健康・heartbeat）。
#[derive(Debug, Clone, PartialEq)]
pub struct Signs {
    pub tick: Reading<Sign>,
    pub heartbeat: Reading<&'static str>,
}

/// tick の健康の class（見本の `.tk.tk-<値>`・測れていないは `tk` だけ）。
pub fn tick_class(tick: &Reading<Sign>) -> &'static str {
    match tick {
        Reading::Known(s) if *s == OK => "tk tk-healthy",
        Reading::Known(_) => "tk tk-stale",
        Reading::Unknown => "tk",
    }
}

/// heartbeat の class（見本の `.hb.hb-off`・on と測れていないは `hb` だけ）。
pub fn hb_class(heartbeat: &Reading<&'static str>) -> &'static str {
    match heartbeat {
        Reading::Known("off") => "hb hb-off",
        _ => "hb",
    }
}

/// orchestrator の行の移動の印（退避までの残り秒か、席の口座が群の今の口座と違う）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    /// 退避までの残り秒。
    Grace(u64),
    /// 席の口座が群の今の口座と違う。
    Wait,
}

impl Move {
    pub fn class(self) -> &'static str {
        match self {
            Move::Grace(_) => "mvgrace",
            Move::Wait => "mvwait",
        }
    }

    /// 語の鍵。
    pub fn key(self) -> &'static str {
        match self {
            Move::Grace(_) => "move_grace",
            Move::Wait => "seat_mismatch",
        }
    }
}

/// 退避までの残り秒の字（`1101 秒`）。
pub fn grace_text(secs: u64) -> String {
    format!("{secs} 秒")
}

/// 今の経過の字（since が無ければ「―」・今と電文の at の大きい方から since を引く・負は 0）。
/// 端末の時計が電文の at より遅れても、電文を読んだ時点の字より小さくしない。
pub fn elapsed_at(since: Option<EpochSecs>, at: EpochSecs, now: EpochSecs) -> String {
    since.map_or_else(
        || NONE_MARK.to_string(),
        |s| age(now.max(at).saturating_sub(s)),
    )
}

/// 今の退避までの残り秒（電文の at の時点の残り秒から今 − at を引く・今が at より前なら引かない・0 で止める）。
pub fn grace_left(left: u64, at: EpochSecs, now: EpochSecs) -> u64 {
    left.saturating_sub(now.saturating_sub(at))
}

/// 表の 1 行。
#[derive(Debug, Clone, PartialEq)]
pub struct SessRow {
    /// 電文の sessions の中の位置（0 から）。
    pub index: usize,
    /// 行の class（`srow r-<役>`）。
    pub class: String,
    pub project: String,
    pub role_key: &'static str,
    /// session の名（席が無い行は None で、語の鍵 seat_none を出す）。
    pub session: Option<String>,
    pub account: Option<String>,
    /// 状態の値（状態の記号に渡す）。
    pub state: &'static str,
    /// 段の字（pipeline で段が在る行だけ・ほかは状態の語を出す）。
    pub stage: Option<&'static str>,
    /// 今の状態になった時刻（電文の同じ行の since・DOM は 1 秒の時計で経過の字を組み直す）。
    pub since: Option<EpochSecs>,
    /// 経過の字（電文の at の時点・since が無ければ「―」）。
    pub elapsed: String,
    /// 稼働の記録の区間（幅を選ぶたびに `strip` で SVG にする）。
    pub spans: Reading<Vec<SeatSpan>>,
    /// orchestrator の行だけ。
    pub signs: Option<Signs>,
    pub moving: Option<Move>,
    /// 停止の切り替え（orchestrator の行で席の名が在り heartbeat が読めるときだけ・便 h-hb）。
    pub toggle: Option<Toggle>,
    /// session の欄の hover の card（`sess_card`）。
    pub card: Card,
}

/// 束の見出し。
#[derive(Debug, Clone, PartialEq)]
pub enum Head {
    /// project の名と群の名（群が無ければ None）。
    Project { name: String, group: Option<String> },
    /// 口座の名と占有の群（None は語の鍵 free_for_pipeline・電文の accounts に無い口座は測れていない）。
    Account {
        label: String,
        occupant: Reading<Option<String>>,
    },
    /// 口座が無い行（語の鍵 st_unknown）。
    NoAccount,
    /// stage の並べの束（語の鍵）。
    Band(&'static str),
}

/// 見出しと行（elapsed の並べは見出しが無い 1 つ）。
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub head: Option<Head>,
    pub rows: Vec<SessRow>,
}

/// 表（稼働の記録の窓の右端と束の並び）。
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub at: EpochSecs,
    pub sort: Sort,
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
pub fn content(fetched: &Fetched, sort: Sort) -> Body<Table> {
    match super::doc(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(doc) if doc.sessions.is_empty() => Body::Empty(NO_ROWS),
        Ok(doc) => Body::Filled(table(&doc, sort)),
    }
}

/// 電文を表に組む（行の無い見出しと束は出さない・同じ値は電文の sessions の順）。
pub fn table(doc: &AccountDoc, sort: Sort) -> Table {
    let lines = &doc.sessions;
    let all = || 0..lines.len();
    let heads: Vec<(Option<Head>, Vec<usize>)> = match sort {
        Sort::Project => project_heads(doc)
            .into_iter()
            .map(|(name, group)| {
                let mut idx: Vec<usize> = all().filter(|&i| lines[i].project == name).collect();
                idx.sort_by(|&a, &b| {
                    (lines[a].role, &lines[a].name).cmp(&(lines[b].role, &lines[b].name))
                });
                (Some(Head::Project { name, group }), idx)
            })
            .collect(),
        Sort::Account => {
            let by_project = |mut idx: Vec<usize>| {
                idx.sort_by(|&a, &b| {
                    (lines[a].role, &lines[a].project).cmp(&(lines[b].role, &lines[b].project))
                });
                idx
            };
            let mut out: Vec<(Option<Head>, Vec<usize>)> = account_heads(doc)
                .into_iter()
                .map(|(label, occupant)| {
                    let idx = by_project(
                        all()
                            .filter(|&i| lines[i].account.as_ref() == Some(&label))
                            .collect(),
                    );
                    (Some(Head::Account { label, occupant }), idx)
                })
                .collect();
            let none = by_project(all().filter(|&i| lines[i].account.is_none()).collect());
            out.push((Some(Head::NoAccount), none));
            out
        }
        Sort::Stage => BANDS
            .into_iter()
            .map(|key| {
                let mut idx: Vec<usize> = all()
                    .filter(|&i| band_key(rank(&lines[i])) == key)
                    .collect();
                idx.sort_by_key(|&i| (rank(&lines[i]), since_key(&lines[i])));
                (Some(Head::Band(key)), idx)
            })
            .collect(),
        Sort::Elapsed => {
            let mut idx: Vec<usize> = all().collect();
            idx.sort_by_key(|&i| since_key(&lines[i]));
            vec![(None, idx)]
        }
    };
    let groups = heads
        .into_iter()
        .filter(|(_, idx)| !idx.is_empty())
        .map(|(head, idx)| Group {
            head,
            rows: idx.into_iter().map(|i| row(doc, i)).collect(),
        })
        .collect();
    Table {
        at: doc.at,
        sort,
        groups,
    }
}

/// since の古い順の鍵（since が無い行は末尾）。
fn since_key(line: &SessionLine) -> (bool, EpochSecs) {
    (line.since.is_none(), line.since.unwrap_or_default())
}

/// project の見出し（電文の projects の順・projects に無い project の行はその後に、行の順で）。
fn project_heads(doc: &AccountDoc) -> Vec<(String, Option<String>)> {
    let mut out: Vec<(String, Option<String>)> = doc
        .projects
        .iter()
        .map(|p| (p.name.clone(), p.group.clone()))
        .collect();
    for line in &doc.sessions {
        if !out.iter().any(|(n, _)| *n == line.project) {
            out.push((line.project.clone(), None));
        }
    }
    out
}

/// 口座の見出し（電文の accounts の順・accounts に無い口座の行はその後に、行の順で占有は測れていない）。
fn account_heads(doc: &AccountDoc) -> Vec<(String, Reading<Option<String>>)> {
    let mut out: Vec<(String, Reading<Option<String>>)> = match &doc.accounts {
        Reading::Known(rows) => rows
            .iter()
            .map(|a| (a.label.clone(), Reading::Known(a.occupant.clone())))
            .collect(),
        Reading::Unknown => Vec::new(),
    };
    for account in doc.sessions.iter().filter_map(|l| l.account.as_ref()) {
        if !out.iter().any(|(a, _)| a == account) {
            out.push((account.clone(), Reading::Unknown));
        }
    }
    out
}

/// 行の project の ProjectRow（電文の projects のうち name が同じ行）。
fn project_of<'a>(doc: &'a AccountDoc, name: &str) -> Option<&'a ProjectRow> {
    doc.projects.iter().find(|p| p.name == name)
}

/// 電文の sessions の i 行目を表の行にする。
pub fn row(doc: &AccountDoc, index: usize) -> SessRow {
    let line = &doc.sessions[index];
    let orchestrator = line.role == SeatRole::Orchestrator;
    SessRow {
        index,
        class: format!("srow r-{}", role_word(line.role)),
        project: line.project.clone(),
        role_key: role_key(line.role),
        session: (!line.name.is_empty()).then(|| line.name.clone()),
        account: line.account.clone(),
        state: state_value(line.state),
        stage: line
            .stage
            .filter(|_| line.role == SeatRole::Pipeline)
            .map(stage_word),
        since: line.since,
        elapsed: elapsed_at(line.since, doc.at, doc.at),
        spans: line.spans.clone(),
        signs: orchestrator.then(|| signs(doc, line)),
        moving: if orchestrator {
            moving(doc, line)
        } else {
            None
        },
        toggle: toggle(doc, line),
        card: sess_card(doc, index),
    }
}

/// orchestrator の行の tick の健康と heartbeat（同じ project の席の card から・card が Unknown なら両方とも測れていない）。
pub fn signs(doc: &AccountDoc, line: &SessionLine) -> Signs {
    match project_of(doc, &line.project).map(|p| &p.seat) {
        Some(Reading::Known(card)) => {
            let t = top(card);
            Signs {
                tick: t.tick,
                heartbeat: t.heartbeat,
            }
        }
        _ => Signs {
            tick: Reading::Unknown,
            heartbeat: Reading::Unknown,
        },
    }
}

/// 群の今の口座（電文の groups の中で group が ProjectRow の group と同じ GroupCard の row の account）。
pub fn current_account<'a>(doc: &'a AccountDoc, project: &ProjectRow) -> Option<&'a str> {
    let group = project.group.as_ref()?;
    let Reading::Known(groups) = &doc.groups else {
        return None;
    };
    groups
        .iter()
        .find(|g| g.row.group == *group)
        .map(|g| g.row.account.as_str())
}

/// orchestrator の行の移動の印（残り秒が在れば残り秒・無ければ席の口座が群の今の口座と違うと分かるときだけ移動待ち）。
/// 行の口座か群の今の口座が分からない行は出さない（読めない一致を不一致と出さない・要件 NFR2）。
pub fn moving(doc: &AccountDoc, line: &SessionLine) -> Option<Move> {
    let project = project_of(doc, &line.project)?;
    if let Some(secs) = project.move_left_s {
        return Some(Move::Grace(secs));
    }
    let account = line.account.as_deref()?;
    let current = current_account(doc, project)?;
    (account != current).then_some(Move::Wait)
}

/// 稼働の記録の SVG の字（窓の右端は電文の at・session の行は口座の移動を持たないので縦線は無い・区間が読めなければ測れていない）。
pub fn strip(spans: &Reading<Vec<SeatSpan>>, at: EpochSecs, span: Span) -> Reading<String> {
    match spans {
        Reading::Known(s) => Reading::Known(strip_svg(&rects(s, at, span), &[])),
        Reading::Unknown => Reading::Unknown,
    }
}

/// 目盛の段の span の style と時刻の対（seat の span_ticks の順・窓の右端は at・seat の block の DOM と同じ `left:<値>%` の形）。
pub fn axis_labels(at: EpochSecs, span: Span) -> Vec<(String, String)> {
    span_ticks(at, span)
        .into_iter()
        .map(|t| (format!("left:{}%", t.left), t.label))
        .collect()
}

/// run の行の card の出所（見本の run の枝の出所の字）。
pub const RUN_SRC: &str = "fleet/events.jsonl · RunStage";

/// run の行の session の欄の card（見本の run の枝: 名・役と project・段と状態・口座といつから）。
pub fn run_card(line: &SessionLine, at: EpochSecs) -> Card {
    let stage = line.stage.map_or(NONE_MARK, stage_word);
    let account = line
        .account
        .clone()
        .unwrap_or_else(|| label("st_unknown"));
    let mut more = vec![format!("口座 {account}")];
    if let Some(s) = line.since {
        more.push(format!("◷ {} から", hmd(s, at)));
    }
    Card {
        title: line.name.clone(),
        kind: format!("{} · {}", label("role:pipeline"), line.project),
        value: format!("{stage} · {}", label(state_key(state_value(line.state)))),
        src: RUN_SRC.to_string(),
        more,
    }
}

/// project の card が詳しくに並べる session の数（見本の proj の枝）。
pub const PROJ_SHOWN: usize = 5;

/// 席の無い project の行の session の欄の card（見本の proj の枝: 群・session の数・名と口座・board）。
pub fn proj_card(doc: &AccountDoc, name: &str) -> Card {
    let named: Vec<&SessionLine> = doc
        .sessions
        .iter()
        .filter(|l| l.project == name && !l.name.is_empty())
        .collect();
    let project = project_of(doc, name);
    let group = project
        .and_then(|p| p.group.as_deref())
        .unwrap_or(NONE_MARK);
    let value = if named.is_empty() {
        label("seat_none")
    } else {
        let orch = named
            .iter()
            .filter(|l| l.role == SeatRole::Orchestrator)
            .count();
        format!("session {} · orchestrator {orch}", named.len())
    };
    let mut more: Vec<String> = named
        .iter()
        .take(PROJ_SHOWN)
        .map(|l| format!("{} {}", l.name, l.account.as_deref().unwrap_or("?")))
        .collect();
    more.push(match project.and_then(|p| p.board) {
        Some(port) => format!("board :{port}"),
        None => label("not_yet"),
    });
    Card {
        title: name.to_string(),
        kind: format!("project · {group}"),
        value,
        src: format!("{name} · state dir"),
        more,
    }
}

/// 電文の sessions の i 行目の session の欄の card（run の行は run・席の読める orchestrator の行は席・ほかは project）。
pub fn sess_card(doc: &AccountDoc, index: usize) -> Card {
    let line = &doc.sessions[index];
    if line.role == SeatRole::Pipeline {
        return run_card(line, doc.at);
    }
    (!line.name.is_empty())
        .then(|| project_of(doc, &line.project))
        .flatten()
        .and_then(|p| orch_card(doc, p))
        .unwrap_or_else(|| proj_card(doc, &line.project))
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
    use tsuzuri_contract::EpochSecs;
    use tsuzuri_contract::board::Reading;

    use super::{
        BLOCK, C_HH, C_PROJ, C_ROLE, C_STAGE, C_STRIP, COLUMNS, GHEAD, Group, HROW, Head, Move,
        NONE_MARK, SAXIS, SAXIS_M, SESS, SessRow, Signs, Sort, TKHB, Table, axis_labels, content,
        elapsed_at, grace_left, grace_text, hb_class, sort_of, strip, tick_class, with_sort,
    };
    use crate::account::PATH;
    use crate::account::heartbeat::{self, States, Toggle};
    use crate::project::seat::{Sign, Span, span_of, with_span};
    use crate::project::{Body, UNKNOWN, body_view, section, state_icon, state_key, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{hs, term};
    use crate::widgets::hover::attach;

    /// 今の URL の query（読めなければ空）。
    fn search() -> String {
        window().location().search().unwrap_or_default()
    }

    /// URL の query を置き換える（頁は読み直さない）。
    /// 幅の押しは見本の setSpan と同じく履歴に積まない。
    fn replace(url: &str) {
        if let Ok(history) = window().history() {
            let _ = history.replace_state_with_url(
                &web_sys::wasm_bindgen::JsValue::NULL,
                "",
                Some(url),
            );
        }
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
        let query = search();
        let sort = RwSignal::new(sort_of(&query));
        let span = RwSignal::new(span_of(&query));
        let hb: States = RwSignal::new(Default::default());
        let tick = crate::net::ticker();
        let body = move || match fetched.with(|f| content(f, sort.get())) {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Filled(table) => table_view(table, span, tick, hb),
        };
        let extra = view! { {sort_bar(sort)}{span_bar(span)} }.into_any();
        section(BLOCK, extra, body.into_any())
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
                    <button type="button" data-sort=s.key() aria-pressed=pressed data-term=s.label_key() on:click=pick>
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

    fn span_bar(span: RwSignal<Span>) -> AnyView {
        let buttons = Span::ALL
            .into_iter()
            .map(|s| {
                let pressed = move || (span.get() == s).to_string();
                let pick = move |_| {
                    replace(&with_span(&search(), s));
                    span.set(s);
                };
                view! {
                    <button type="button" data-span=s.key() aria-pressed=pressed on:click=pick>
                        {s.key()}
                    </button>
                }
            })
            .collect_view();
        view! {
            <span class="spanbar">{hs("span")}<span class="seg" role="group">{buttons}</span></span>
        }
        .into_any()
    }

    fn table_view(
        table: Table,
        span: RwSignal<Span>,
        tick: ReadSignal<EpochSecs>,
        hb: States,
    ) -> AnyView {
        let at = table.at;
        let head = COLUMNS[..6]
            .iter()
            .map(|&k| view! { <div>{hs(k)}</div> })
            .collect_view();
        let groups = table
            .groups
            .into_iter()
            .map(|g| group_view(g, at, span, tick, hb))
            .collect_view();
        view! {
            <div class=SESS>
                <div class=HROW>
                    {head}
                    <div class=C_HH>{hs("history")}{axis_view(at, span)}</div>
                </div>
                <div class=SAXIS_M>{hs("history")}{axis_view(at, span)}</div>
                {groups}
            </div>
        }
        .into_any()
    }

    /// 稼働の記録の目盛の段（窓の右端は電文の at・幅を切り替えると目盛も替わる）。
    fn axis_view(at: EpochSecs, span: RwSignal<Span>) -> AnyView {
        view! {
            <div class=SAXIS aria-hidden="true">
                {move || {
                    axis_labels(at, span.get())
                        .into_iter()
                        .map(|(style, label)| view! { <span style=style>{label}</span> })
                        .collect_view()
                }}
            </div>
        }
        .into_any()
    }

    fn head_view(head: Head) -> AnyView {
        match head {
            Head::Project { name, group } => view! {
                <span class="gname" data-t="">{name}</span>
                {group.map(|g| view! { <span class="sub" data-t="">{g}</span> })}
            }
            .into_any(),
            Head::Account {
                label: name,
                occupant,
            } => {
                let occ = match occupant {
                    Reading::Known(Some(g)) => view! { <span data-t="">{g}</span> }.into_any(),
                    Reading::Known(None) => term("free_for_pipeline", label("free_for_pipeline")),
                    Reading::Unknown => state_icon(UNKNOWN),
                };
                view! {
                    <span class="gname mono">{name}</span>
                    <span class="sub">{occ}</span>
                }
                .into_any()
            }
            Head::NoAccount => view! {
                {state_icon(UNKNOWN)}
                <span class="gname">{label("st_unknown")}</span>
            }
            .into_any(),
            Head::Band(key) => {
                view! { <span class="gname">{term(key, label(key))}</span> }.into_any()
            }
        }
    }

    fn group_view(
        group: Group,
        at: EpochSecs,
        span: RwSignal<Span>,
        tick: ReadSignal<EpochSecs>,
        hb: States,
    ) -> AnyView {
        let n = group.rows.len();
        let head = group.head.map(|h| {
            view! { <div class=GHEAD>{head_view(h)}<span class="chip num">{n}</span></div> }
        });
        let rows = group
            .rows
            .into_iter()
            .map(|r| row_view(r, at, span, tick, hb))
            .collect_view();
        view! { {head}{rows} }.into_any()
    }

    /// 表の 1 行（経過の字は 1 秒の時計で組み直す）。
    fn row_view(
        row: SessRow,
        at: EpochSecs,
        span: RwSignal<Span>,
        tick: ReadSignal<EpochSecs>,
        hb: States,
    ) -> AnyView {
        let session = match row.session.clone() {
            Some(name) => view! { <span class="mono">{name}</span> }.into_any(),
            None => view! { <span class="sub">{term("seat_none", label("seat_none"))}</span> }
                .into_any(),
        };
        let account = match row.account.clone() {
            Some(a) => a.into_any(),
            None => view! { {state_icon(UNKNOWN)}<span class="sub">{label("st_unknown")}</span> }
                .into_any(),
        };
        let spans = row.spans.clone();
        let svg = move || match strip(&spans, at, span.get()) {
            Reading::Known(s) => view! { <div inner_html=s></div> }.into_any(),
            Reading::Unknown => NONE_MARK.into_any(),
        };
        let below = row.toggle.clone().map(|t| heartbeat::below(t, hb));
        let since = row.since;
        let elapsed = move || elapsed_at(since, at, tick.get());
        view! {
            <div class=row.class.clone()>
                <div class=C_PROJ data-t="">{row.project.clone()}</div>
                <div class=C_ROLE>{term(row.role_key, label(row.role_key))}</div>
                <div tabindex="0" use:attach=row.card.clone()>{session}</div>
                <div class="mono">{account}</div>
                <div class=C_STAGE>{stage_view(&row, at, tick, hb)}</div>
                <div class="num">{elapsed}</div>
                <div class=C_STRIP>{svg}</div>
            </div>
            {below}
        }
        .into_any()
    }

    /// 段の欄（退避までの残り秒は 1 秒の時計で電文の at からの差を引き直す）。
    fn stage_view(
        row: &SessRow,
        at: EpochSecs,
        tick: ReadSignal<EpochSecs>,
        hb: States,
    ) -> AnyView {
        let word = match row.stage {
            Some(s) => s.to_string(),
            None => label(state_key(row.state)),
        };
        let moving = row.moving.map(|m| match m {
            Move::Grace(secs) => {
                let left = move || term(m.key(), grace_text(grace_left(secs, at, tick.get())));
                view! {
                    <span class=m.class()>
                        <span aria-hidden="true">"⇥"</span>
                        <b class="num">{left}</b>
                    </span>
                }
                .into_any()
            }
            Move::Wait => {
                view! { <span class=m.class()>{term(m.key(), label(m.key()))}</span> }.into_any()
            }
        });
        view! {
            {state_icon(row.state)}
            <span class="sstg">{word}{moving}</span>
            {row.signs.clone().map(|s| signs_view(s, row.toggle.clone(), hb))}
        }
        .into_any()
    }

    fn sign_view(r: Reading<Sign>) -> AnyView {
        match r {
            Reading::Known(s) => {
                view! { <span class=s.class aria-hidden="true">{s.glyph}</span> }.into_any()
            }
            Reading::Unknown => state_icon(UNKNOWN),
        }
    }

    /// 合図の行（切り替えが在れば heartbeat の後に button）。
    fn signs_view(signs: Signs, toggle: Option<Toggle>, states: States) -> AnyView {
        let tk = tick_class(&signs.tick);
        let hb = hb_class(&signs.heartbeat);
        let hb_word = match signs.heartbeat {
            Reading::Known(w) => view! { <b>{w}</b> }.into_any(),
            Reading::Unknown => state_icon(UNKNOWN),
        };
        let button = toggle.map(|t| heartbeat::button(t, states));
        view! {
            <span class=TKHB>
                <span class=tk>{sign_view(signs.tick)}{hs("tick_health")}</span>
                <span class=hb>{hs("heartbeat")}{hb_word}</span>
                {button}
            </span>
        }
        .into_any()
    }
}
