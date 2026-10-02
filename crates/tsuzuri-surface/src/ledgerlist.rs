//! 台帳 open の一覧（判断の記録 ADR-27 の決定 (8)・行 g-list-groups）: epic ごとの組にし、組の頭の 1 行に
//! 進みの棒（組の行の閉じた数と全部の数）と、板で動いている札の段ごとの数と、open の行の数を出す。行は 1 行の短い題。
//! 既定で開く組は、板で動いている札を持つ組と open の行を持つ組だけで、ほかは畳む（開き閉じは頁の一生の間だけ持つ）。
//! 行は口 /api/ledger の台帳の行、段の数と行の段の字は口 /api/pipeline の札、短い題と起票の時刻は口 /api/beads の
//! bead の事実から引く。組の親は id の階層（`a.1` の親は `a`）のいちばん近い epic の祖先で、epic の祖先の無い行は最後の組。
//! 一覧の見出しは、種類の切り替え（すべて・便・memo・問い）と探す欄（短い題・題の全体・id）と、指標の口 /api/metrics の
//! 小さな数（open の便・memo・問いと純減 24h）と 14 日の burndown の図（棒 = 閉じた数 / 日・線 = open の task）を出す
//! （行 g-list-head・要件 FR13 の指標で sparkline は出さない）。絞りが効いている間は合う行の在る組だけを開いて出す。
//! 14 日の burndown の図には、block の前の burndown の段の card を付ける（block の指標の段を外して見出しへ縮めた・行 g-ledger-trim）。
//! memo と問いの行と見出しの未反映の数には、口 /api/cases（器の局面の出力の部品）の局面と手番の平易な字を添える
//! （語は台帳の block の辞書の関数・出力がまだ無い間と知らない語は「まだ分からない」・読めない版の出力は「読めない」・
//! 古さの印の在る出力は「古い」と添える・要件 FR13・行 g-unref-lc）。
//! 組の頭の名を押すと epic を選び（もう 1 度押すと解く）、板のその組でない札を薄くし、行を押すと吹き出しを開き、吹き出しの
//! 開いている bead の行と組の頭と札に輪の印を付け、選んだ組と開いた bead の組は開く（行 g-select・見本の setEpic と applyMarks）。
//! 組と頭の数と既定の開きと並べと絞りは純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use std::collections::BTreeMap;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineCard, PipelineColumn, Reading};
use tsuzuri_contract::case::CaseDoc;
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::{BeadFact, BeadFacts, LedgerRow};
use tsuzuri_contract::stats::{LedgerStats, UnreflectedKind};
use tsuzuri_contract::wire;

use crate::project::ledger::{
    BURN_H, BURN_W, CLOSED, EMPTY, NO_OPEN, NONE, OUTSIDE, TURN_KEY, UNKNOWN_WORD_KEY, Unref,
    burn_svg, burndown, kind_label, kind_name, kind_phase_text, net, phase_text, plain_word, stats,
};
use crate::project::pipeline::{LANES, Lane, age_at, cards, stage_word};
use crate::project::{Body, LEDGER_UNREAD};
use crate::view::{Fetched, id_order, read_rows};
use crate::vocab::label;

/// bead の事実の口（短い題と起票の時刻・行 c-bead-route）。
pub const FACTS_PATH: &str = "/api/beads";

/// epic の外の組の鍵（epic の id と重ならない字）。
pub const OUTSIDE_KEY: &str = "-";

/// 札の無い行の右の字の頭の語の鍵（起票からの経過・吹き出しの起票の欄と同じ語・字は語の辞書の見出し）。
pub const CREATED_KEY: &str = "pf_created";

/// 開いた組に open の行が無いときの 1 行。
pub const NO_KIDS: &str = "open の子なし（全部着地）";

/// 一覧の行の種類の札（種類・字・class）。
pub const KIND_TAGS: [(NodeKind, &str, &str); 4] = [
    (NodeKind::Task, "便", "ll-kt k-task"),
    (NodeKind::Memo, "memo", "ll-kt k-memo"),
    (NodeKind::Question, "問い", "ll-kt k-question"),
    (NodeKind::Epic, "epic", "ll-kt k-epic"),
];

/// 種類の札の字と class（表に無い種類は便の札）。
pub fn kind_tag(kind: NodeKind) -> (&'static str, &'static str) {
    KIND_TAGS
        .iter()
        .find(|(k, _, _)| *k == kind)
        .map_or(("便", "ll-kt k-task"), |(_, t, c)| (*t, *c))
}

/// 一覧の 1 行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lrow {
    pub id: String,
    pub kind: NodeKind,
    /// 短い題（bead の事実の short・引けなければ台帳の題）。
    pub short: String,
    /// 題の全体（台帳の字のまま）。
    pub title: String,
    /// 右の字（札が在れば札の段の字・無ければ起票からの経過）。
    pub right: String,
    /// 並べの位（`row_rank`）。
    pub rank: u8,
    /// 局面と手番の平易な字（memo と問いの行だけ・`with_phases` が置く・行 g-unref-lc）。
    pub phase: Option<String>,
}

/// 一覧の 1 組（epic の組・epic の外の組は `epic` が None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lgroup {
    /// 組の鍵（epic の id・epic の外の組は `OUTSIDE_KEY`）。
    pub key: String,
    pub epic: Option<String>,
    /// 頭の名（epic の短い題・epic の外の組は `OUTSIDE`）。
    pub name: String,
    /// 組の行（閉じた行も数える）のうち閉じた数と全部の数。
    pub closed: usize,
    pub total: usize,
    /// 板で動いている札（Landed の列を除く 4 列）の列ごとの数（板の列の順・0 の列も持つ）。
    pub counts: Vec<(Lane, usize)>,
    /// open の行（`row_rank` の順）。
    pub rows: Vec<Lrow>,
}

impl Lgroup {
    /// 板で動いている札の数。
    pub fn live(&self) -> usize {
        self.counts.iter().map(|(_, n)| n).sum()
    }

    /// 既定で開くか（板で動いている札か open の行を持つ組だけ）。
    pub fn open_default(&self) -> bool {
        self.live() > 0 || !self.rows.is_empty()
    }

    /// 進みの棒の幅の百分率（全部が 0 なら 0）。
    pub fn pct(&self) -> usize {
        (self.closed * 100).checked_div(self.total).unwrap_or(0)
    }
}

/// bead の事実の口の本文を id の表にする（読めない・まだ読んでいない・台帳が読めないは空の表）。
pub fn facts(fetched: &Fetched) -> BTreeMap<String, BeadFact> {
    let Fetched::Body(text) = fetched else {
        return BTreeMap::new();
    };
    match wire::decode::<BeadFacts>(text) {
        Ok(BeadFacts {
            rows: Reading::Known(rows),
        }) => rows.into_iter().map(|f| (f.id.to_string(), f)).collect(),
        _ => BTreeMap::new(),
    }
}

/// 短い題（bead の事実の short・事実に無ければ台帳の題）。
pub fn short_of(row: &LedgerRow, facts: &BTreeMap<String, BeadFact>) -> String {
    facts
        .get(row.id.as_str())
        .map_or_else(|| row.title.clone(), |f| f.short.clone())
}

/// id のいちばん近い epic の祖先（id の階層で親をたどる・自分は数えない）。
pub fn home<'a>(id: &str, epics: &[&'a LedgerRow]) -> Option<&'a LedgerRow> {
    std::iter::successors(id.rsplit_once('.').map(|(p, _)| p), |p| {
        p.rsplit_once('.').map(|(q, _)| q)
    })
    .find_map(|a| epics.iter().find(|e| e.id.as_str() == a).copied())
}

/// 行の並べの位（札が在れば止まり・走り・Queued・Blocked・着地の順、無ければ問い・便・memo の順）。
pub fn row_rank(kind: NodeKind, card: Option<&PipelineCard>) -> u8 {
    match card.map(|c| c.stage.column()) {
        Some(PipelineColumn::QuestionedFailedStopped) => 0,
        Some(PipelineColumn::RunningGated) => 1,
        Some(PipelineColumn::Queued) => 2,
        Some(PipelineColumn::Blocked) => 3,
        Some(PipelineColumn::Landed) => 4,
        None => match kind {
            NodeKind::Question => 5,
            NodeKind::Task => 6,
            NodeKind::Memo => 7,
            _ => 8,
        },
    }
}

/// 台帳の行を一覧の 1 行にする（右の字は札が在れば段の字・無ければ起票からの経過・今 now から引く）。
fn lrow(
    row: &LedgerRow,
    card: Option<&PipelineCard>,
    facts: &BTreeMap<String, BeadFact>,
    now: EpochSecs,
) -> Lrow {
    let created = facts.get(row.id.as_str()).and_then(|f| f.created_at);
    Lrow {
        id: row.id.to_string(),
        kind: row.node_kind(),
        short: short_of(row, facts),
        title: row.title.clone(),
        right: card.map_or_else(
            || format!("{} {}", label(CREATED_KEY), age_at(created, now)),
            stage_word,
        ),
        rank: row_rank(row.node_kind(), card),
        phase: None,
    }
}

/// 1 組（epic の外の組は epic が None・組の行と組の札から: 閉じた数と全部の数・Landed を除く 4 列の札の数・open の行を並べの位と id の順に）。
/// 行の札は組の札から id で引く（札の bead と行は同じ id なので親の epic も同じ）。
fn group_of(
    epic: Option<&LedgerRow>,
    all: &[&LedgerRow],
    mine: &[&PipelineCard],
    facts: &BTreeMap<String, BeadFact>,
    now: EpochSecs,
) -> Lgroup {
    let card_of = |id: &str| mine.iter().find(|c| c.contract.as_str() == id).copied();
    let mut open: Vec<Lrow> = all
        .iter()
        .filter(|r| r.status != CLOSED)
        .map(|r| lrow(r, card_of(r.id.as_str()), facts, now))
        .collect();
    open.sort_by(|a, b| a.rank.cmp(&b.rank).then_with(|| id_order(&a.id, &b.id)));
    let counts = LANES
        .iter()
        .filter(|l| l.column != PipelineColumn::Landed)
        .map(|l| {
            (
                *l,
                mine.iter().filter(|c| c.stage.column() == l.column).count(),
            )
        })
        .collect();
    Lgroup {
        key: epic.map_or(OUTSIDE_KEY, |e| e.id.as_str()).to_string(),
        epic: epic.map(|e| e.id.to_string()),
        name: epic.map_or_else(|| OUTSIDE.to_string(), |e| short_of(e, facts)),
        closed: all.iter().filter(|r| r.status == CLOSED).count(),
        total: all.len(),
        counts,
        rows: open,
    }
}

/// 組を作る: 台帳の epic ごとに、いちばん近い epic の祖先がその epic の行（epic の外も含む・epic の行は自分の組の頭）と
/// 札を集める。epic が閉じていて open の行も動いている札も無い組は出さない。並べは、動いている札を持つ組（多い順）・
/// open の行を持つ組・ほか（同じ位は epic の id の順）で、epic の外の組（open の行が在るときだけ）は最後。
pub fn groups(
    rows: &[LedgerRow],
    cards: &[PipelineCard],
    facts: &BTreeMap<String, BeadFact>,
    now: EpochSecs,
) -> Vec<Lgroup> {
    let epics: Vec<&LedgerRow> = rows
        .iter()
        .filter(|r| r.node_kind() == NodeKind::Epic)
        .collect();
    let key_of = |id: &str| home(id, &epics).map_or(OUTSIDE_KEY, |e| e.id.as_str());
    let mut members: BTreeMap<&str, Vec<&LedgerRow>> = BTreeMap::new();
    for row in rows.iter().filter(|r| r.node_kind() != NodeKind::Epic) {
        members
            .entry(key_of(row.id.as_str()))
            .or_default()
            .push(row);
    }
    let mut owned: BTreeMap<&str, Vec<&PipelineCard>> = BTreeMap::new();
    for card in cards {
        owned
            .entry(key_of(card.contract.as_str()))
            .or_default()
            .push(card);
    }
    let group = |key: &str, epic: Option<&LedgerRow>| {
        let all = members.get(key).map(Vec::as_slice).unwrap_or_default();
        let mine = owned.get(key).map(Vec::as_slice).unwrap_or_default();
        group_of(epic, all, mine, facts, now)
    };
    let mut out: Vec<Lgroup> = epics
        .iter()
        .map(|e| (group(e.id.as_str(), Some(e)), e.status != CLOSED))
        .filter(|(g, open)| *open || g.open_default())
        .map(|(g, _)| g)
        .collect();
    let rank = |g: &Lgroup| match (g.live() > 0, g.rows.is_empty()) {
        (true, _) => 0,
        (false, false) => 1,
        (false, true) => 2,
    };
    out.sort_by(|a, b| {
        rank(a)
            .cmp(&rank(b))
            .then_with(|| b.live().cmp(&a.live()))
            .then_with(|| id_order(&a.key, &b.key))
    });
    let outside = group(OUTSIDE_KEY, None);
    if !outside.rows.is_empty() {
        out.push(outside);
    }
    out
}

/// 板が読めない間に組の頭の段の数の代わりに出す字（数を描かない・0 と見せない）。
pub const COUNTS_UNKNOWN: &str = "?";

/// 組の頭に描く段の数（板が読めない間は None で数を描かず `COUNTS_UNKNOWN` の 1 つにする・読めれば 0 でない列だけ）。
pub fn head_counts(g: &Lgroup, unread: bool) -> Option<Vec<(Lane, usize)>> {
    (!unread).then(|| g.counts.iter().filter(|(_, n)| *n > 0).copied().collect())
}

/// 一覧の中身（台帳が読めなければ測れていない・行の無い台帳は EMPTY・出す組が無ければ NO_OPEN）。
/// 板の札と bead の事実が読めない間は、段の数を 0・行の段の字を起票からの経過・短い題を台帳の題にする（板が読めない間は
/// view が一覧の頭に板の読めない理由を測れていないの 1 行で出し、札が無いとは見せない・widgets の pop の `board_unread`）。
pub fn content(
    ledger: &Fetched,
    pipe: &Fetched,
    beads: &Fetched,
    now: EpochSecs,
) -> Body<Vec<Lgroup>> {
    let rows = match read_rows(ledger) {
        Reading::Known(rows) => rows,
        Reading::Unknown => return Body::Unmeasured(LEDGER_UNREAD),
    };
    if rows.is_empty() {
        return Body::Empty(EMPTY);
    }
    let out = groups(&rows, &cards(pipe).unwrap_or_default(), &facts(beads), now);
    if out.is_empty() {
        Body::Empty(NO_OPEN)
    } else {
        Body::Filled(out)
    }
}

/// 種類の切り替えの値（見本の kseg）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    All,
    Task,
    Memo,
    Question,
}

/// 種類の切り替えの表（値・button の字・合う行の種類・すべては None）。
pub const KINDS: [(Kind, &str, Option<NodeKind>); 4] = [
    (Kind::All, "すべて", None),
    (Kind::Task, "便", Some(NodeKind::Task)),
    (Kind::Memo, "memo", Some(NodeKind::Memo)),
    (Kind::Question, "問い", Some(NodeKind::Question)),
];

/// 探す欄の置き字（短い題・題の全体・id を探す）。
pub const SEARCH_HINT: &str = "題・id で探す";

/// 絞りに合う行が 1 つも無いときの 1 行。
pub const NO_MATCH: &str = "合う行なし";

/// 指標の口が読めないときの見出しの小さな数の語の鍵（吹き出しの読めない欄と同じ「まだ分からない」・憲法 P-7）。
pub const KPI_UNKNOWN_KEY: &str = crate::widgets::pop::UNKNOWN_KEY;

/// 絞りが効いているか（種類がすべてでないか、探す字が空白だけでない）。
pub fn filtering(kind: Kind, q: &str) -> bool {
    kind != Kind::All || !q.trim().is_empty()
}

/// 行が絞りに合うか（種類が表の種類と同じで、探す字〔前後の空白を除いた小文字〕が短い題・題の全体・id の
/// 小文字のどれかに含まれる・すべてと空の探す字はどの行にも合う）。
pub fn row_matches(row: &Lrow, kind: Kind, q: &str) -> bool {
    let want = KINDS
        .iter()
        .find(|(k, _, _)| *k == kind)
        .and_then(|(_, _, n)| *n);
    let q = q.trim().to_lowercase();
    want.is_none_or(|n| n == row.kind)
        && [&row.short, &row.title, &row.id]
            .iter()
            .any(|t| t.to_lowercase().contains(&q))
}

/// 絞った組（絞りが効いていなければ組のまま・効いていれば合う行だけを残し、合う行の無い組は出さない・頭の数は替えない）。
pub fn filtered(groups: Vec<Lgroup>, kind: Kind, q: &str) -> Vec<Lgroup> {
    if !filtering(kind, q) {
        return groups;
    }
    groups
        .into_iter()
        .filter_map(|mut g| {
            g.rows.retain(|r| row_matches(r, kind, q));
            (!g.rows.is_empty()).then_some(g)
        })
        .collect()
}

/// 見出しの指標の小さな数の字（open の便・memo・問いの数と純減 24h・見本の kchip）。
pub fn kpi_text(s: &LedgerStats) -> String {
    format!(
        "便 {} · memo {} · 問い {} · 純減 24h {}",
        s.open.task,
        s.open.memo,
        s.open.question,
        net(s.net_drop_24h).text
    )
}

/// 指標の口が読めないときの見出しの小さな数の字（指標と語 KPI_UNKNOWN_KEY の見出しの語）。
pub fn kpi_unknown() -> String {
    format!("指標 {}", label(KPI_UNKNOWN_KEY))
}

/// 見出しの小さな数と 14 日の図の svg（図は台帳の block の burndown・指標の口が読めなければ kpi_unknown の字と図なし）。
pub fn kpi(fetched: &Fetched) -> (String, Option<String>) {
    match stats(fetched) {
        Ok(s) => (
            kpi_text(&s),
            Some(burn_svg(&burndown(&s.days, BURN_W, BURN_H))),
        ),
        Err(_) => (kpi_unknown(), None),
    }
}

/// 読めない版の局面の出力の周に出す字の鍵（読めない・電文の unreadable が真・要件 FR13）。
pub const UNREADABLE_KEY: &str = "case_unreadable";

/// 古さの印の在る局面の出力に添える字の鍵（古い・電文の stale が空でない・要件 FR13）。
pub const STALE_KEY: &str = "case_stale";

/// 局面の出力の口の本文の読み（要件 FR13 の周: 出力がまだ無い・読めない版・読めた〔古さの印の種類つき〕）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phases {
    /// まだ読んでいない・口が読めない・電文が読めない・出力がまだ無い（「まだ分からない」）。
    Unknown,
    /// 出力の file は在るのに読めない版（電文の unreadable が真・「読めない」・行 c-case-unreadable）。
    Unreadable,
    /// memo と問いの部品の id の表（局面の語と手番の語）と古さの印の種類（空でなければ出力は古い）。
    Known(BTreeMap<String, (String, String)>, Vec<String>),
}

/// 局面の出力の口の本文を読む（memo と問いの部品を id の表にし、古さの印の種類を添える）。
pub fn phases(fetched: &Fetched) -> Phases {
    let Fetched::Body(text) = fetched else {
        return Phases::Unknown;
    };
    match wire::decode::<CaseDoc>(text) {
        Ok(CaseDoc {
            parts: Reading::Known(parts),
            stale,
            ..
        }) => Phases::Known(
            parts
                .into_iter()
                .filter(|p| matches!(p.part.as_str(), "memo" | "question"))
                .map(|p| (p.id, (p.phase, p.turn)))
                .collect(),
            stale,
        ),
        Ok(CaseDoc {
            unreadable: true, ..
        }) => Phases::Unreadable,
        _ => Phases::Unknown,
    }
}

/// 古さの印の在る出力の字の末に添える字（印が無ければ空・在れば空白と「古い」）。
fn old_mark(stale: &[String]) -> String {
    if stale.is_empty() {
        String::new()
    } else {
        format!(" {}", label(STALE_KEY))
    }
}

/// memo と問いの行に局面と手番の平易な字を置く（読めない版は「読めない」・表に id が無い行と出力がまだ無い間は
/// 「まだ分からない」・古さの印が在れば局面の字に「古い」を添える・ほかの行は None のまま）。
pub fn with_phases(groups: Vec<Lgroup>, cases: &Fetched) -> Vec<Lgroup> {
    let read = phases(cases);
    groups
        .into_iter()
        .map(|mut g| {
            for r in &mut g.rows {
                if matches!(r.kind, NodeKind::Memo | NodeKind::Question) {
                    r.phase = Some(match &read {
                        Phases::Known(table, stale) => table.get(&r.id).map_or_else(
                            || label(UNKNOWN_WORD_KEY),
                            |(phase, turn)| {
                                format!("{}{}", phase_text(phase, turn), old_mark(stale))
                            },
                        ),
                        Phases::Unreadable => label(UNREADABLE_KEY),
                        Phases::Unknown => label(UNKNOWN_WORD_KEY),
                    });
                }
            }
            g
        })
        .collect()
}

/// 見出しの未反映の数（字・title・測れていないの印を添えるか）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrefHead {
    pub text: String,
    pub title: String,
    pub partial: bool,
}

/// 見出しの未反映の数（数は台帳の block の未反映の数に手番の席の字を添え、1〜2 種が分からなければ測れていないの印、
/// 3 種とも分からなければ「まだ分からない」、読めない版の出力の周は「読めない」、古さの印が在れば「古い」を添える・
/// title は種類ごとの数と局面の平易な字・数の分からない種類は「まだ分からない」・古さの印の種類を末に並べる）。
pub fn unref_head(s: &LedgerStats, cases: &Phases) -> UnrefHead {
    let unref = Unref {
        count: s.unreflected,
        unknown: s
            .unreflected_unknown
            .iter()
            .map(|k| kind_name(*k))
            .collect(),
    };
    let stale: &[String] = match cases {
        Phases::Known(_, stale) => stale,
        _ => &[],
    };
    let count = match (cases, unref.text()) {
        (Phases::Unreadable, _) => label(UNREADABLE_KEY),
        (_, n) if n == NONE => label(KPI_UNKNOWN_KEY),
        (_, n) => n,
    };
    let text = format!(
        "{} {count}{} · {}",
        label("l_unref"),
        old_mark(stale),
        plain_word(TURN_KEY, "seat")
    );
    let title = UnreflectedKind::ALL
        .iter()
        .map(|k| {
            let n = s
                .unreflected_kinds
                .iter()
                .find(|c| c.kind == *k)
                .map_or_else(|| label(UNKNOWN_WORD_KEY), |c| c.count.to_string());
            format!("{} {n}（{}）", kind_label(*k), kind_phase_text(*k))
        })
        .chain((!stale.is_empty()).then(|| format!("{}（{}）", label(STALE_KEY), stale.join("・"))))
        .collect::<Vec<_>>()
        .join(" / ");
    UnrefHead {
        text,
        title,
        partial: unref.partial() && !matches!(cases, Phases::Unreadable),
    }
}

/// bead の組の鍵（一覧の組と同じ読み: id の階層のいちばん近い epic の祖先の id・epic の祖先が無ければ `OUTSIDE_KEY`）。
pub fn key_of(id: &str, rows: &[LedgerRow]) -> String {
    let epics: Vec<&LedgerRow> = rows
        .iter()
        .filter(|r| r.node_kind() == NodeKind::Epic)
        .collect();
    home(id, &epics)
        .map_or(OUTSIDE_KEY, |e| e.id.as_str())
        .to_string()
}

/// 押した後の選んだ epic の組の鍵（押した鍵が空か今と同じなら解いて None・ほかは押した鍵・見本の setEpic）。
pub fn pick(now: Option<&str>, key: &str) -> Option<String> {
    (!key.is_empty() && now != Some(key)).then(|| key.to_string())
}

/// 札を薄くするか（epic の組を選んでいて、札の組の鍵が分かりそれと違う・見本の applyMarks の dim）。
/// 台帳が読めない間は札の組の鍵が分からない（None）ので薄くしない（選んだ組の外と見せない・憲法 P-7.2）。
pub fn dim(sel: Option<&str>, key: Option<&str>) -> bool {
    matches!((sel, key), (Some(s), Some(k)) if s != k)
}

/// 印を付けるか（開いている bead の id か選んだ組の鍵が印の鍵と同じ・札と行の輪は bead の id・組の頭の esel は組の鍵）。
pub fn ring(shown: Option<&str>, key: &str) -> bool {
    shown == Some(key)
}

/// 組の頭に輪の印を付けるか（吹き出しの開いている bead が組の open の行のどれか・見本の applyMarks の gh）。
pub fn head_ring(g: &Lgroup, shown: Option<&str>) -> bool {
    shown.is_some_and(|s| g.rows.iter().any(|r| r.id == s))
}

#[cfg(target_arch = "wasm32")]
pub use dom::view;

#[cfg(target_arch = "wasm32")]
pub use dom::SelCtx;

/// 一覧の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;

    use tsuzuri_contract::board::Reading;
    use tsuzuri_contract::case;

    use super::{
        COUNTS_UNKNOWN, FACTS_PATH, KINDS, Kind, Lgroup, Lrow, NO_KIDS, NO_MATCH, SEARCH_HINT,
        content, filtered, filtering, head_counts, head_ring, key_of, kind_tag, kpi, phases, pick,
        ring, unref_head, with_phases,
    };
    use crate::frame::{Mode, node_href};
    use crate::kit::Folds;
    use crate::project::ledger::{burn_card, stats};
    use crate::project::{Body, UNKNOWN, ledger, pipeline, state_icon, state_key, unmeasured};
    use crate::view::read_rows;
    use crate::vocab::label;
    use crate::widgets::help::HelpCtx;
    use crate::widgets::hover::attach_some;
    use crate::widgets::pop::{PopCtx, Via, board_unread};

    /// 選びの状態（App が context に置く・頁に 1 つ・一覧と板が読む・行 g-select）: 選んだ epic の組の鍵。
    /// 選んだ bead は吹き出しの開いている bead（`PopCtx::shown`）で、ここには持たない。
    #[derive(Clone, Copy)]
    pub struct SelCtx {
        pub epic: RwSignal<Option<String>>,
    }

    impl Default for SelCtx {
        fn default() -> Self {
            Self {
                epic: RwSignal::new(None),
            }
        }
    }

    /// 吹き出しの開いている bead の id（吹き出しの層が無ければ None）。
    fn shown() -> Option<String> {
        use_context::<PopCtx>().and_then(PopCtx::shown)
    }

    /// 一覧（見出しと組・台帳と板と bead の事実の 3 つの口を読む・組の開き閉じと絞りは頁の一生の間だけ持つ・板が読めない間は一覧の頭に理由の 1 行）。
    pub fn view() -> AnyView {
        let rows = crate::net::read(ledger::PATH);
        let pipe = crate::net::read(pipeline::PATH);
        let beads = crate::net::read(FACTS_PATH);
        let cases = crate::net::read(case::PATH);
        let folds = RwSignal::new(Folds::default());
        let kind = RwSignal::new(Kind::All);
        let query = RwSignal::new(String::new());
        // 選んだ epic の組と吹き出しの開いた bead の組は、開き閉じの記録に開くと書く（見本の setEpic と ensureListed）。
        let sel = use_context::<SelCtx>();
        Effect::new(move |_| {
            let epic = sel.and_then(|s| s.epic.get());
            let bead = shown().and_then(|id| {
                rows.with_untracked(|l| match read_rows(l) {
                    Reading::Known(r) => Some(key_of(&id, &r)),
                    Reading::Unknown => None,
                })
            });
            for key in [epic, bead].into_iter().flatten() {
                folds.update(|f| f.set(&key, true));
            }
        });
        let list = move || {
            let got =
                rows.with(|l| pipe.with(|p| beads.with(|b| content(l, p, b, crate::net::now()))));
            match got {
                Body::Unmeasured(reason) => unmeasured(reason),
                Body::Empty(line) => {
                    view! { <div class="empty"><span>{line}</span></div> }.into_any()
                }
                Body::Filled(groups) => {
                    let (k, q) = (kind.get(), query.get());
                    let active = filtering(k, &q);
                    let shown = filtered(cases.with(|c| with_phases(groups, c)), k, &q);
                    if shown.is_empty() {
                        view! { <div class="ll-gnone">{NO_MATCH}</div> }.into_any()
                    } else {
                        shown
                            .into_iter()
                            .map(|g| group_view(g, folds, active))
                            .collect_view()
                            .into_any()
                    }
                }
            }
        };
        // 板の札が読めない間は、段の数と行の段の字が出ない理由を一覧の頭に出す（札が無いとは見せない）。
        let unread = move || pipe.with(board_unread).map(unmeasured);
        view! { {head_line(kind, query)}<div class="ll-body">{unread}{list}</div> }.into_any()
    }

    /// 一覧の見出し（指標の小さな数と 14 日の図・種類の切り替え・探す欄）。
    fn head_line(kind: RwSignal<Kind>, query: RwSignal<String>) -> AnyView {
        let metrics = crate::net::read(ledger::METRICS_PATH);
        let cases = crate::net::read(case::PATH);
        let kpis = move || {
            let (text, svg) = metrics.with(kpi);
            let card = metrics.with(|m| stats(m).ok().map(|s| burn_card(&s)));
            let fig = svg.map(|s| {
                view! { <span class="ll-spark" tabindex="0" use:attach_some=card inner_html=s></span> }
            });
            let unref = metrics
                .with(|m| stats(m).ok())
                .map(|s| cases.with(|c| unref_head(&s, &phases(c))))
                .map(|h| {
                    let mark = h.partial.then(|| state_icon(UNKNOWN));
                    view! { <span class="ll-unref num" title=h.title>{h.text}{mark}</span> }
                });
            view! { <span class="ll-kchip num">{text}</span>{unref}{fig} }
        };
        let seg = KINDS
            .into_iter()
            .map(|(k, text, _)| {
                let class = move || if kind.get() == k { "on" } else { "" };
                view! { <button type="button" class=class on:click=move |_| kind.set(k)>{text}</button> }
            })
            .collect_view();
        view! {
            <div class="ll-head">
                <div class="ll-l1">{kpis}</div>
                <div class="ll-l2">
                    <div class="ll-seg" role="group">{seg}</div>
                    <input class="ll-q" type="search" autocomplete="off" placeholder=SEARCH_HINT
                        prop:value=move || query.get() on:input=move |ev| query.set(event_target_value(&ev))/>
                </div>
            </div>
        }
        .into_any()
    }

    /// 1 組（頭の 1 行と、開いていれば行・絞りが効いている間は開く）。
    fn group_view(g: Lgroup, folds: RwSignal<Folds>, active: bool) -> AnyView {
        let initial = g.open_default();
        let is_open = {
            let key = g.key.clone();
            move || active || folds.with(|f| f.open(&key, initial))
        };
        let toggle = {
            let (key, is_open) = (g.key.clone(), is_open.clone());
            move |_| {
                let now = is_open();
                folds.update(|f| f.set(&key, !now));
            }
        };
        let mark = {
            let is_open = is_open.clone();
            move || if is_open() { "▾" } else { "▸" }
        };
        let head_class = if g.open_default() {
            "ll-gh"
        } else {
            "ll-gh empty"
        };
        // 選んだ epic の組の頭は esel・吹き出しの開いている bead の組の頭は輪（行 g-select）。
        let sel = use_context::<SelCtx>();
        let esel = {
            let key = g.key.clone();
            move || sel.is_some_and(|s| s.epic.with(|e| ring(e.as_deref(), &key)))
        };
        let ringed = {
            let g = g.clone();
            move || head_ring(&g, shown().as_deref())
        };
        let rows = g.rows.clone();
        let body = move || {
            is_open().then(|| {
                if rows.is_empty() {
                    view! { <div class="ll-gnone">{NO_KIDS}</div> }.into_any()
                } else {
                    rows.iter().map(row_view).collect_view().into_any()
                }
            })
        };
        view! {
            <div class="ll-grp" data-gid=g.key.clone()>
                <div class=head_class class:esel=esel class:ring=ringed>
                    <button class="ll-tog" type="button" aria-label="開閉" on:click=toggle>{mark}</button>
                    {head_view(&g)}
                </div>
                {body}
            </div>
        }
        .into_any()
    }

    /// 組の頭の名と数（epic の名と id・進みの棒と閉じた数 / 全部の数・札の在る列の数・open の行の数）。
    fn head_view(g: &Lgroup) -> AnyView {
        let prog = g.epic.as_ref().map(|_| {
            let ratio = format!("{}/{}", g.closed, g.total);
            view! {
                <span class="ll-prog" role="img" aria-label=ratio.clone()><i style=format!("width:{}%", g.pct())></i></span>
                <span class="ll-pn num">{ratio}</span>
            }
        });
        // 板が読めない間は段の数を描かず「?」の 1 つにする（0 の札と見せない・list が板を読み直すと組み直す）。
        let unread = crate::net::read(pipeline::PATH).with_untracked(|p| board_unread(p).is_some());
        let dots = match head_counts(g, unread) {
            Some(counts) => counts
                .into_iter()
                .map(|(lane, n)| {
                    view! { <b class=format!("ll-sd sd-{}", lane.name) title=label(lane.key)>{n}</b> }
                })
                .collect_view()
                .into_any(),
            None => view! { <b class="ll-sd" title=label(state_key(UNKNOWN))>{COUNTS_UNKNOWN}</b> }
                .into_any(),
        };
        let gid = g
            .epic
            .clone()
            .map(|id| view! { <span class="ll-gid mono">{id}</span> });
        // epic の組の名は押すと epic を選ぶ・もう 1 度押すと解く（epic の外の組は選べない・行 g-select）。
        let name = match (&g.epic, use_context::<SelCtx>()) {
            (Some(_), Some(sel)) => {
                let key = g.key.clone();
                let choose = move |_| sel.epic.update(|e| *e = pick(e.as_deref(), &key));
                view! { <button type="button" class="ll-gname" on:click=choose>{g.name.clone()}</button> }
                    .into_any()
            }
            _ => view! { <span class="ll-gname">{g.name.clone()}</span> }.into_any(),
        };
        view! {
            {name}
            {gid}
            {prog}
            <span class="ll-sdots">{dots}</span>
            <span class="ll-gopen num">{format!("open {}", g.rows.len())}</span>
        }
        .into_any()
    }

    /// 1 行（種類の札・短い題・右の字・短い題は節点の頁への link で、題の全体を title に置く）。
    fn row_view(r: &Lrow) -> AnyView {
        let (tag, class) = kind_tag(r.kind);
        let ctx = use_context::<HelpCtx>();
        let id = r.id.clone();
        let href = move || {
            let mode = match ctx {
                Some(c) => c.mode.get(),
                None => Mode::from_query(&window().location().search().unwrap_or_default()),
            };
            node_href(&id, mode)
        };
        // 行を押すと吹き出しを開き・もう 1 度押すと閉じ、吹き出しの開いている行は輪（行 g-select）。
        let pop = use_context::<PopCtx>();
        let press = {
            let id = r.id.clone();
            move |_| {
                if let Some(p) = pop {
                    p.press(&id, Via::Row);
                }
            }
        };
        let ringed = {
            let id = r.id.clone();
            move || ring(shown().as_deref(), &id)
        };
        // 短い題の link の普通の押しは頁を移らず行の押しに任せる（新しい窓や tab で開く押しは link のまま）。
        let stay = |e: ev::MouseEvent| {
            if crate::board::plain_click(&e) {
                e.prevent_default();
            }
        };
        view! {
            <div class="ll-row" class:ring=ringed data-id=r.id.clone() data-pop-row=r.id.clone() on:click=press>
                <span class=class>{tag}</span>
                <a class="ll-ls" href=href on:click=stay title=r.title.clone()>{r.short.clone()}</a>
                {r.phase.clone().map(|p| view! { <span class="ll-ph">{p}</span> })}
                <span class="ll-rt">{r.right.clone()}</span>
            </div>
        }
        .into_any()
    }
}
