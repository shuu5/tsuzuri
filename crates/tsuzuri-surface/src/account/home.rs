//! account board の HOME の block（便 h-frame の枠・便 h-home の中身）: 各 project の次の一手・群の枠・口座 × 窓・移動。
//! 見本は account/index.html の render の home の枝（`#nxall` の nxrows・`.gtop` の over・口座 × 窓の arows・`details#moves` の mvli）と
//! acct.js の nextAll・meter。並べと字と class は純粋な関数（`content`）で組み、DOM は wasm の target のときだけ組む。
//! 群の列・口座の列・移動の列は電文で別々に Unknown になりうるので、その段だけ測れていないにし、ほかの段は出す（要件 NFR2）。
//! 逼迫の印（見本の上限の字と強調の class）・候補ごとの門で落ちた理由・24 時間の線・測った時刻の列は出さない（未決・R-22）。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::{
    AccountDoc, AccountRow, GroupCard, MoveRow, ProjectRow, SessionLine,
};
use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::seat::{QuotaUsed, SeatState};
use tsuzuri_contract::stats::{CheckResult, NextStep};

use super::cards::{gproj_card, nx_card};
use crate::frame::Block;
use crate::project::Body;
use crate::project::next::{UNJUDGED_LINE, big, key, unjudged};
use crate::project::seat::{NG, OK, Sign, WINDOWS, WindowRow, hmd, short, window_row};
use crate::view::Fetched;
use crate::vocab::label;
use crate::widgets::hover::Card;

/// 各 project の次の一手（1 段目の左）。
pub const NXALL: Block = Block {
    id: "nxall",
    heading: "next_all",
    class: "panel",
};

/// 群の枠（2 段目・見本の `.gtop`）。
pub const GROUPS: Block = Block {
    id: "groups",
    heading: "group",
    class: "gtop",
};

/// 口座 × 窓（3 段目）。
pub const ALLOWANCE: Block = Block {
    id: "allowance",
    heading: "allowance",
    class: "panel",
};

/// 移動（4 段目・畳める段）。
pub const MOVES: Block = Block {
    id: "moves",
    heading: "moves",
    class: "panel fold mvp",
};

/// この module が描く block（HOME の段の順）。
pub const BLOCKS: [Block; 4] = [NXALL, GROUPS, ALLOWANCE, MOVES];

/// block の中身の有無（便 h-frame の枠の値・中身は `content` が組む）。
pub fn body(fetched: &Fetched) -> Body<()> {
    super::pending(fetched)
}

/// 無い値の字（群の無い project・前の口座の無い群・読めない割合・退役の口座の占有・移る前の口座が無い移動）。
pub const NONE: &str = "―";

/// 印の 6 種（`NextMove::ALL` の先頭の 6 種・なしを除く）。
pub const MARK_KINDS: [NextMove; 6] = [
    NextMove::LimitOrMove,
    NextMove::Unresponsive,
    NextMove::StalledRun,
    NextMove::BatchApproval,
    NextMove::Question,
    NextMove::AwaitingEffect,
];

/// 次の一手が読めない project の lead の語の鍵。
pub const UNKNOWN_KEY: &str = "st_unknown";

/// 次の一手の 1 つの印（種・字・class）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NxMark {
    pub kind: NextMove,
    /// 語の鍵の末尾の字（a から f）。
    pub glyph: &'static str,
    /// 判じた当たりは `nxm on`・当たらないは `nxm off`・判じなかった（電文に無い）は `nxm na`。
    pub class: &'static str,
}

/// 次の一手の並びの 1 行（見本の `.nxrow`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NxRow {
    pub project: String,
    /// 群の名（無しは「―」）。
    pub group: String,
    /// lead の種（次の一手が読めなければ None）。
    pub lead: Option<NextMove>,
    /// lead の語の鍵（nx_a から nx_g・読めなければ st_unknown）。
    pub key: &'static str,
    /// 1 行の字（next の module の big の what・読めなければ「―」）。
    pub line: String,
    /// 6 つの印（`MARK_KINDS` の順）。
    pub marks: Vec<NxMark>,
    /// 行の class（なしと読めない行は淡く）。
    pub class: &'static str,
    /// 行の hover の card（見本の `__tz_card` の nx の枝・`nx_card`）。
    pub card: Card,
}

/// 並べの順位（lead の `NextMove::ALL` の位置・読めない行となしを判じなかった行は後ろ）。
fn rank(next: &Reading<NextStep>) -> usize {
    match next {
        Reading::Known(s) if unjudged(s) => NextMove::ALL.len(),
        Reading::Known(s) => NextMove::ALL
            .iter()
            .position(|k| *k == s.lead)
            .expect("lead は 7 種のどれか"),
        Reading::Unknown => NextMove::ALL.len(),
    }
}

/// 印の class（電文にその種が無ければ判じなかったと同じ）。
fn mark_class(result: Option<CheckResult>) -> &'static str {
    match result {
        Some(CheckResult::Hit) => "nxm on",
        Some(CheckResult::Miss) => "nxm off",
        Some(CheckResult::NotJudged) | None => "nxm na",
    }
}

/// 6 つの印（次の一手が読めなければ全部 na）。
pub fn marks(next: &Reading<NextStep>) -> Vec<NxMark> {
    MARK_KINDS
        .into_iter()
        .map(|kind| {
            let result = match next {
                Reading::Known(s) => s.checks.iter().find(|c| c.kind == kind).map(|c| c.result),
                Reading::Unknown => None,
            };
            NxMark {
                kind,
                glyph: &key(kind)[3..],
                class: mark_class(result),
            }
        })
        .collect()
}

/// project の行を並びの 1 行にする（card の時刻は電文の at・なしを判じなかった行は読めない行と同じ測れていない）。
pub fn nx_row(row: &ProjectRow, at: EpochSecs) -> NxRow {
    let (lead, k, line) = match &row.next {
        Reading::Known(s) if unjudged(s) => (None, UNKNOWN_KEY, UNJUDGED_LINE.to_string()),
        Reading::Known(s) => {
            let check = s.checks.iter().find(|c| c.kind == s.lead);
            (Some(s.lead), key(s.lead), big(s.lead, check).what)
        }
        Reading::Unknown => (None, UNKNOWN_KEY, NONE.to_string()),
    };
    NxRow {
        project: row.name.clone(),
        group: row.group.clone().unwrap_or_else(|| NONE.to_string()),
        lead,
        key: k,
        line,
        marks: marks(&row.next),
        class: match lead {
            Some(NextMove::Nothing) | None => "nxrow dim",
            Some(_) => "nxrow",
        },
        card: nx_card(row, at),
    }
}

/// 次の一手の並び（lead の 7 種の順・同じ種は電文の順・読めない行は後ろに電文の順）。
pub fn next_all(doc: &AccountDoc) -> Vec<NxRow> {
    let mut rows: Vec<&ProjectRow> = doc.projects.iter().collect();
    rows.sort_by_key(|p| rank(&p.next));
    rows.into_iter().map(|p| nx_row(p, doc.at)).collect()
}

/// 群の枠の窓の 1 つ（見本の `.pw`・逼迫の印は持たない）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pw {
    /// 窓の名（語の鍵）。
    pub window: &'static str,
    /// 窓の短い字（`5h`）。
    pub short: String,
    /// 使った割合の字（`42%`・読めなければ「―」）。
    pub used: String,
}

/// 群の project の 1 つ（名と一致の印）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub project: String,
    pub sign: Sign,
    /// chip の hover の card（見本の `__tz_card` の gproj の枝・電文に同じ名の行が無ければ None）。
    pub card: Option<Card>,
}

/// 席の口座が分からない project の印（字は無い）。
pub const UNKNOWN_SIGN: Sign = Sign {
    glyph: "",
    class: "gi unknown",
};

/// 記録なしの群のいつからの語の鍵。
pub const NO_RECORD_KEY: &str = "no_record";

/// 次の移り先が無い群の語の鍵。
pub const NO_TARGET_KEY: &str = "no_target";

/// 群の枠（見本の `.gcard`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupView {
    pub name: String,
    /// 今の口座。
    pub account: String,
    /// 今の記録が在るか（偽ならいつからは記録なしの字）。
    pub recorded: bool,
    /// いつからの字（hmd か、記録なしの字）。
    pub since: String,
    /// 前の口座（無しは「―」）。
    pub previous: String,
    /// 今の口座の窓ごとの使った割合（`WINDOWS` の順）。
    pub pressure: Vec<Pw>,
    pub members: Vec<Member>,
    pub candidates: Vec<String>,
    /// 次の移り先（無しは None）。
    pub next_account: Option<String>,
    /// 次の移り先の字（無しは語の鍵 no_target の字）。
    pub next: String,
    /// 群の project の限度で止まった session の数（見出しの横の `.kpi-s`）。
    pub limited: usize,
    /// 枠の末尾の詳しくの段（見本の `.gmore`）。
    pub more: GroupMore,
}

/// 経験者向けの 1 行の字数（要件 FR14）。
pub const EXPERT_CHARS: usize = 60;

/// 空白で区切った語を 1 行の字数に畳む（字数を超える語は字数ごとの片に切る・見本の doctorLine の畳み）。
pub fn wrap_words(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let pieces = text.split_whitespace().flat_map(|w| {
        let chars: Vec<char> = w.chars().collect();
        chars
            .chunks(width)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<_>>()
    });
    let mut lines: Vec<String> = Vec::new();
    for p in pieces {
        match lines.last_mut() {
            Some(l) if l.chars().count() + 1 + p.chars().count() <= width => {
                l.push(' ');
                l.push_str(&p);
            }
            _ => lines.push(p),
        }
    }
    lines
}

/// 器の doctor の群の行の形（見本の doctorLine と同じ 6 つの欄の順）。
pub fn doctor_line(card: &GroupCard) -> String {
    let row = &card.row;
    let mut seats: Vec<&str> = Vec::new();
    for s in card.members.iter().filter_map(|m| m.seat_account.as_deref()) {
        if !seats.contains(&s) {
            seats.push(s);
        }
    }
    let seats = if seats.is_empty() {
        "none".to_string()
    } else {
        seats.join(",")
    };
    format!(
        "group={} accounts={} anchors={} seat-accounts={} current={} next={}",
        row.group,
        row.candidates.join(","),
        card.members.len(),
        seats,
        row.account,
        row.next_account.as_deref().unwrap_or("none"),
    )
}

/// 群の枠の詳しくの段（記録の数・出所・doctor の群の行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMore {
    /// 記録の数の字（`記録 1 件`・移動が読めなければ `記録 ―`）。
    pub records: String,
    /// 出所の字（見本の gm1 src）。
    pub src: String,
    /// doctor の群の行を 60 字に畳んだ行。
    pub doctor: Vec<String>,
}

/// 群の詳しくの段を組む（記録 1 件は移動の段の 1 行）。
pub fn group_more(card: &GroupCard, moves: &Reading<Vec<MoveRow>>) -> GroupMore {
    let name = &card.row.group;
    let records = match moves {
        Reading::Known(m) => format!("記録 {} 件", m.iter().filter(|r| &r.group == name).count()),
        Reading::Unknown => format!("記録 {NONE}"),
    };
    GroupMore {
        records,
        src: format!("groups/{name}.account・host.toml の群の行"),
        doctor: wrap_words(&doctor_line(card), EXPERT_CHARS),
    }
}

/// 群の project の session のうち限度で止まった数（電文の state の写しだけを数える）。
pub fn limited(card: &GroupCard, sessions: &[SessionLine]) -> usize {
    sessions
        .iter()
        .filter(|s| s.state == SeatState::Limit)
        .filter(|s| card.members.iter().any(|m| m.project == s.project))
        .count()
}

/// 口座の行の窓（行が無い・usage が読めない・窓が無いは None）。
fn usage_of<'a>(row: Option<&'a AccountRow>, window: &str) -> Option<&'a QuotaUsed> {
    match &row?.usage {
        Reading::Known(u) => u.iter().find(|q| q.window == window),
        Reading::Unknown => None,
    }
}

/// 群の枠を 1 つ組む（`accounts` は電文の口座の列・読めなければ None・session と移動の列と時刻は電文の欄）。
pub fn group_view(card: &GroupCard, accounts: Option<&[AccountRow]>, doc: &AccountDoc) -> GroupView {
    let at = doc.at;
    let row = &card.row;
    let acct = accounts.and_then(|a| a.iter().find(|r| r.label == row.account));
    let since = if card.recorded {
        card.since.map_or_else(|| NONE.to_string(), |s| hmd(s, at))
    } else {
        label(NO_RECORD_KEY)
    };
    GroupView {
        name: row.group.clone(),
        account: row.account.clone(),
        recorded: card.recorded,
        since,
        previous: card.previous.clone().unwrap_or_else(|| NONE.to_string()),
        pressure: WINDOWS
            .into_iter()
            .map(|w| Pw {
                window: w,
                short: short(w),
                used: usage_of(acct, w).map_or_else(|| NONE.to_string(), |q| format!("{}%", q.used_pct)),
            })
            .collect(),
        members: card
            .members
            .iter()
            .map(|m| Member {
                project: m.project.clone(),
                sign: match m.matches {
                    Reading::Known(true) => OK,
                    Reading::Known(false) => NG,
                    Reading::Unknown => UNKNOWN_SIGN,
                },
                card: doc
                    .projects
                    .iter()
                    .find(|p| p.name == m.project)
                    .map(|p| gproj_card(doc, p)),
            })
            .collect(),
        candidates: row.candidates.clone(),
        next_account: row.next_account.clone(),
        next: row
            .next_account
            .clone()
            .unwrap_or_else(|| label(NO_TARGET_KEY)),
        limited: limited(card, &doc.sessions),
        more: group_more(card, &doc.moves),
    }
}

/// 口座 × 窓の占有の欄。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Occupant {
    /// 占有している群の名。
    Group(String),
    /// どの群の今の口座でもない（pipeline に使える）。
    Free,
    /// 退役の口座（「―」）。
    Retired,
}

/// 空いている口座の語の鍵。
pub const FREE_KEY: &str = "free_for_pipeline";

/// 退役の口座の語の鍵。
pub const RETIRED_KEY: &str = "retired";

impl Occupant {
    /// 欄の class（見本の `.occ.grp`・`.occ.free`・退役は無し）。
    pub fn class(&self) -> &'static str {
        match self {
            Occupant::Group(_) => "occ grp",
            Occupant::Free => "occ free",
            Occupant::Retired => "",
        }
    }

    /// 欄の字。
    pub fn text(&self) -> String {
        match self {
            Occupant::Group(g) => g.clone(),
            Occupant::Free => label(FREE_KEY),
            Occupant::Retired => NONE.to_string(),
        }
    }
}

/// 口座 × 窓の窓の欄（退役は窓の代わりに語の鍵 retired）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cells {
    Retired,
    /// `WINDOWS` の順の 3 つ（無い窓と usage が読めない行は None で「―」）。
    Windows(Vec<Option<WindowRow>>),
}

/// 口座 × 窓の 1 行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcctRow {
    pub label: String,
    pub occupant: Occupant,
    pub cells: Cells,
    /// 名の欄の hover の card（`acct_card`）。
    pub card: Card,
}

/// 口座の行を組む器の出力の名（口座の card の出所）。
pub const ACCT_SRC: &str = "fleet usage --show・doctor の口座の行";

/// 口座の名の欄の hover の card（見本の `__tz_card` の acct の枝・窓ごとの戻る時刻と model と動く session）。
pub fn acct_card(row: &AccountRow, sessions: &[SessionLine], at: EpochSecs) -> Card {
    let occ = if row.retired {
        label(RETIRED_KEY)
    } else {
        match &row.occupant {
            Some(g) => format!("{} {g}", label("occupant")),
            None => label(FREE_KEY),
        }
    };
    let known = match &row.usage {
        Reading::Known(u) if !row.retired => Some(u),
        _ => None,
    };
    let value = if row.retired {
        NONE.to_string()
    } else if let Some(u) = known {
        let parts: Vec<String> = WINDOWS
            .into_iter()
            .filter_map(|w| u.iter().find(|q| q.window == w))
            .map(|q| format!("{} {}%", short(&q.window), q.used_pct))
            .collect();
        if parts.is_empty() {
            NONE.to_string()
        } else {
            parts.join(" · ")
        }
    } else {
        label(UNKNOWN_KEY)
    };
    let mut more: Vec<String> = Vec::new();
    if let Some(u) = known {
        for w in WINDOWS {
            let Some(r) = u.iter().find(|q| q.window == w).and_then(|q| q.resets_at) else {
                continue;
            };
            let mut line = format!("{} ↻ {}", short(w), hmd(r, at));
            if let (true, Some(m)) = (w == "seven_day_model", &row.model) {
                line.push_str(&format!(" · {m}"));
            }
            more.push(line);
        }
    }
    let mine: Vec<&str> = sessions
        .iter()
        .filter(|s| s.account.as_deref() == Some(row.label.as_str()))
        .map(|s| s.name.as_str())
        .collect();
    more.push(if mine.is_empty() {
        "session 0".to_string()
    } else {
        let names: Vec<&str> = mine.iter().take(2).copied().collect();
        format!("session {} · {}", mine.len(), names.join(" / "))
    });
    Card {
        title: row.label.clone(),
        kind: format!("{} · {occ}", label("accounts")),
        value,
        src: ACCT_SRC.to_string(),
        more,
    }
}

/// 口座 × 窓の見出しの語の鍵（名・占有・3 つの窓）。
pub const ACCT_HEADS: [&str; 5] = ["accounts", "occupant", "five_hour", "seven_day", "seven_day_model"];

/// 口座 × 窓の列の幅（stylesheet の 7 列から出さない 2 列を除いた 5 列）。
pub const ACCT_COLS: &str = "grid-template-columns: 80px 156px repeat(3, minmax(0, 1fr))";

/// 口座の行を 1 行にする（窓の時刻は電文の at・card の session は電文の列）。
pub fn acct_row(row: &AccountRow, sessions: &[SessionLine], at: EpochSecs) -> AcctRow {
    let card = acct_card(row, sessions, at);
    if row.retired {
        return AcctRow {
            label: row.label.clone(),
            occupant: Occupant::Retired,
            cells: Cells::Retired,
            card,
        };
    }
    AcctRow {
        card,
        label: row.label.clone(),
        occupant: row
            .occupant
            .clone()
            .map_or(Occupant::Free, Occupant::Group),
        cells: Cells::Windows(
            WINDOWS
                .into_iter()
                .map(|w| usage_of(Some(row), w).map(|q| window_row(q, at)))
                .collect(),
        ),
    }
}

/// 移動の 1 行（時刻・群・前 → 後）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MvRow {
    pub at: String,
    pub group: String,
    pub from: String,
    pub to: String,
}

/// 移動の行の hover の card（見本の `__tz_card` の mv の枝・記録の種類と出所）。
pub fn mv_card(m: &MvRow) -> Card {
    Card {
        title: format!("{} → {}", m.from, m.to),
        kind: format!("{} · {}", label("moves"), m.group),
        value: format!("◷ 記録 {}", m.at),
        src: format!("groups/history/{} ほか", m.group),
        more: Vec::new(),
    }
}

/// 移動の段（初めの行と畳める残り・残りの見出しの字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moves {
    pub shown: Vec<MvRow>,
    pub folded: Vec<MvRow>,
    /// 畳める段の見出しの字（`+3`・残りが無ければ None）。
    pub more: Option<String>,
}

/// 畳まずに出す移動の行の数（見本の SHOWMV）。
pub const SHOW_MV: usize = 8;

/// 移動の行を 1 行にする（時刻は電文の at から見た hmd）。
pub fn mv_row(m: &MoveRow, at: EpochSecs) -> MvRow {
    MvRow {
        at: hmd(m.at, at),
        group: m.group.clone(),
        from: m.from.clone().unwrap_or_else(|| NONE.to_string()),
        to: m.to.clone(),
    }
}

/// 移動の段（電文の順・初めの 8 行と残り）。
pub fn moves(rows: &[MoveRow], at: EpochSecs) -> Moves {
    let mut shown: Vec<MvRow> = rows.iter().map(|m| mv_row(m, at)).collect();
    let folded = shown.split_off(shown.len().min(SHOW_MV));
    let more = (!folded.is_empty()).then(|| format!("+{}", folded.len()));
    Moves {
        shown,
        folded,
        more,
    }
}

/// 群の列が読めないときの理由。
pub const GROUPS_UNREAD: &str = "群の記録か host.toml の群の行が読めないので群の枠が正しいと言えない";

/// 口座の列が読めないときの理由。
pub const ACCOUNTS_UNREAD: &str = "口座の行が読めないので口座ごとの窓と占有が正しいと言えない";

/// 移動の列が読めないときの理由。
pub const MOVES_UNREAD: &str = "移動の記録が読めないので移動の一覧が正しいと言えない";

/// project が 0 の 1 行。
pub const NO_PROJECTS: &str = "project が 0 件";

/// 群が 0 の 1 行。
pub const NO_GROUPS: &str = "群が 0 件";

/// 口座が 0 の 1 行。
pub const NO_ACCOUNTS: &str = "口座が 0 件";

/// 移動が 0 の 1 行。
pub const NO_MOVES: &str = "移動が 0 件";

/// HOME の 4 つの block の中身。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home {
    pub next: Body<Vec<NxRow>>,
    pub groups: Body<Vec<GroupView>>,
    pub accounts: Body<Vec<AcctRow>>,
    pub moves: Body<Moves>,
}

fn list<T>(rows: Vec<T>, none: &'static str) -> Body<Vec<T>> {
    if rows.is_empty() {
        Body::Empty(none)
    } else {
        Body::Filled(rows)
    }
}

/// 読めた電文から 4 つの block の中身を組む（Unknown の列の段だけ測れていない）。
pub fn home(doc: &AccountDoc) -> Home {
    let accounts = match &doc.accounts {
        Reading::Known(a) => Some(a.as_slice()),
        Reading::Unknown => None,
    };
    Home {
        next: list(next_all(doc), NO_PROJECTS),
        groups: match &doc.groups {
            Reading::Known(g) => list(
                g.iter()
                    .map(|c| group_view(c, accounts, doc))
                    .collect(),
                NO_GROUPS,
            ),
            Reading::Unknown => Body::Unmeasured(GROUPS_UNREAD),
        },
        accounts: match accounts {
            Some(a) => list(
                a.iter().map(|r| acct_row(r, &doc.sessions, doc.at)).collect(),
                NO_ACCOUNTS,
            ),
            None => Body::Unmeasured(ACCOUNTS_UNREAD),
        },
        moves: match &doc.moves {
            Reading::Known(m) if m.is_empty() => Body::Empty(NO_MOVES),
            Reading::Known(m) => Body::Filled(moves(m, doc.at)),
            Reading::Unknown => Body::Unmeasured(MOVES_UNREAD),
        },
    }
}

/// 口の読みの結果から 4 つの block の中身（本文が電文として読めなければ 4 つとも測れていないと理由）。
pub fn content(fetched: &Fetched) -> Home {
    match super::doc(fetched) {
        Ok(d) => home(&d),
        Err(reason) => Home {
            next: Body::Unmeasured(reason),
            groups: Body::Unmeasured(reason),
            accounts: Body::Unmeasured(reason),
            moves: Body::Unmeasured(reason),
        },
    }
}

/// block の DOM（移動は畳める段・ほかは section）。
#[cfg(target_arch = "wasm32")]
pub fn view(block: Block) -> leptos::prelude::AnyView {
    match block.id {
        id if id == NXALL.id => dom::next_view(block),
        id if id == GROUPS.id => dom::groups_view(block),
        id if id == ALLOWANCE.id => dom::allowance_view(block),
        _ => dom::moves_view(block),
    }
}

/// HOME の 4 つの block の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;
    use tsuzuri_contract::account::ProjectRow;

    use super::{
        ACCT_COLS, ACCT_HEADS, AcctRow, Cells, GroupView, Home, Moves, MvRow, NxRow, RETIRED_KEY,
        UNKNOWN_KEY, content,
    };
    use crate::account::windows::{NOT_YET_KEY, button_text, open, open_url};
    use crate::account::{PATH, doc};
    use crate::frame::{Block, Mode};
    use crate::project::seat::WindowRow;
    use crate::project::{Body, UNKNOWN, body_view, section, state_icon, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, h2, hs, shows_internal};
    use crate::widgets::hover::{Card, attach};

    /// card が在れば要素に付ける（電文に同じ名の行の無い群の project の chip は card を持たない）。
    fn attach_some(el: web_sys::Element, card: Option<Card>) {
        if let Some(card) = card {
            attach(el, card);
        }
    }

    /// 測れていない・0 件の段（中身ありは `filled` が組む）。
    fn body_or<T>(body: Body<T>, filled: impl FnOnce(T) -> AnyView) -> AnyView {
        match body {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Filled(v) => filled(v),
        }
    }

    fn read() -> Memo<Home> {
        let fetched = crate::net::read(PATH);
        Memo::new(move |_| fetched.with(content))
    }

    pub fn next_view(block: Block) -> AnyView {
        let fetched = crate::net::read(PATH);
        let projects = Memo::new(move |_| {
            fetched.with(|f| doc(f).map(|d| d.projects).unwrap_or_default())
        });
        let home = read();
        let mode = use_context::<HelpCtx>().map(|c| c.mode);
        let count = move || {
            home.with(|h| match &h.next {
                Body::Filled(rows) => Some(view! { <span class="chip num">{rows.len()}</span> }),
                _ => None,
            })
        };
        let body = move || {
            body_or(home.get().next, |rows| {
                let all = projects.get();
                let rows = rows
                    .into_iter()
                    .map(|r| {
                        let p = all.iter().find(|p| p.name == r.project).cloned();
                        nx_row_view(r, p, mode)
                    })
                    .collect_view();
                view! { <div class="nxall">{rows}</div> }.into_any()
            })
        };
        section(block, count.into_any(), body.into_any())
    }

    fn nx_row_view(r: NxRow, p: Option<ProjectRow>, mode: Option<RwSignal<Mode>>) -> AnyView {
        let lead = if r.lead.is_some() {
            let glyph = &r.key[3..];
            view! { <span class="nxk">{format!("({glyph}) {}", label(r.key))}</span> }.into_any()
        } else {
            view! { {state_icon(UNKNOWN)}<span class="nxk">{label(UNKNOWN_KEY)}</span> }.into_any()
        };
        let marks = r
            .marks
            .iter()
            .map(|m| {
                let aria = label(super::key(m.kind));
                view! { <span class=m.class aria-label=aria>{m.glyph}</span> }
            })
            .collect_view();
        let go = match p.filter(|p| p.board.is_some()) {
            Some(p) => {
                let click = move |_| {
                    let m = mode.map_or(Mode::Beginner, |m| m.get_untracked());
                    if let Some(u) = open_url(&p, m) {
                        open(&p.name, &u);
                    }
                };
                view! { <button type="button" class="btn sm" on:click=click>{button_text(None)}</button> }
                    .into_any()
            }
            None => view! { <span class="btn static dis">{label(NOT_YET_KEY)}</span> }.into_any(),
        };
        let card = r.card;
        view! {
            <div class=r.class tabindex="0" use:attach=card>
                <span class="nxp"><span data-t="">{r.project}</span><span class="small muted" data-t="">{r.group}</span></span>
                <span class="nxtop">{lead}<span class="nxl">{r.line}</span></span>
                <span class="nxms">{marks}</span>
                <span class="go">{go}</span>
            </div>
        }
        .into_any()
    }

    /// 群の枠の段（見本の `.gtop`・段の見出しは置かず、群の card の列だけ）。
    pub fn groups_view(block: Block) -> AnyView {
        let home = read();
        let mode = use_context::<HelpCtx>().map(|c| c.mode);
        let body = move || {
            body_or(home.get().groups, |cards| {
                cards
                    .into_iter()
                    .map(|g| group_card(g, mode))
                    .collect_view()
                    .into_any()
            })
        };
        view! { <section class=block.class id=block.id>{body}</section> }.into_any()
    }

    fn group_card(g: GroupView, mode: Option<RwSignal<Mode>>) -> AnyView {
        // 経験者は詳しくを初めから開く（見本の gmore の open）。
        let expert = mode.is_some_and(|m| shows_internal(m.get_untracked()));
        let limit = (g.limited > 0).then(|| state_icon("limit"));
        let doctor = g
            .more
            .doctor
            .into_iter()
            .map(|l| view! { <div><code>{l}</code></div> })
            .collect_view();
        let since = if g.recorded {
            view! { <span class="num">{g.since}</span> }.into_any()
        } else {
            view! { <span class="sub">{g.since}</span> }.into_any()
        };
        let pws = g
            .pressure
            .into_iter()
            .map(|p| {
                view! { <span class="pw" data-term=p.window tabindex="0"><span class="wl">{p.short}</span><b class="num">{p.used}</b></span> }
            })
            .collect_view();
        let members = g
            .members
            .into_iter()
            .map(|m| {
                view! { <span class="pchip" tabindex="0" use:attach_some=m.card><span class=m.sign.class aria-hidden="true">{m.sign.glyph}</span><span data-t="">{m.project}</span></span> }
            })
            .collect_view();
        let next = g.next_account.clone();
        let cands = g
            .candidates
            .into_iter()
            .map(|c| {
                let class = if next.as_deref() == Some(c.as_str()) { "next" } else { "" };
                view! { <li class=class><span class="mono">{c}</span></li> }
            })
            .collect_view();
        let data = g.name.clone();
        view! {
            <section class="gcard" data-group=data>
                <header>
                    {hs("group")}<b class="gname" data-t="">{g.name}</b>
                    <span class="kpi-s">{limit}{hs("limited")}<b class="num">{g.limited}</b></span>
                </header>
                <div class="gcur">
                    <div class="lab">{hs("current_account")}</div>
                    <div class="acc mono">{g.account}</div>
                    <div class="meta">
                        <span>{hs("since_rec")}{since}</span>
                        <span>{hs("previous")}<span class="mono">{g.previous}</span></span>
                    </div>
                </div>
                <div class="gpw">{hs("used")}<span class="pws">{pws}</span></div>
                <div class="glab">{hs("members")}</div>
                <div class="projs">{members}</div>
                <div class="glab">{hs("candidates")}</div>
                <ul class="cands2 smpl">{cands}</ul>
                <div class="glab">{hs("next_target")}<span class="mono">{g.next}</span></div>
                <details class="gmore" open=expert>
                    <summary><span class="rm-t">{label("p_more")}</span><span class="rm-a" aria-hidden="true">"▸"</span></summary>
                    <div class="gm">
                        <div class="gm1"><span class="num">{g.more.records}</span></div>
                        <div class="gm1 src">{g.more.src}</div>
                        <div class="gm1 int xo">{doctor}</div>
                    </div>
                </details>
            </section>
        }
        .into_any()
    }

    pub fn allowance_view(block: Block) -> AnyView {
        let home = read();
        let body = move || {
            body_or(home.get().accounts, |rows| {
                let heads = ACCT_HEADS
                    .into_iter()
                    .map(|k| view! { <div class="hrow">{hs(k)}</div> })
                    .collect_view();
                let rows = rows.into_iter().map(acct_row_view).collect_view();
                view! { <div class="acct-grid" style=ACCT_COLS>{heads}{rows}</div> }.into_any()
            })
        };
        section(block, ().into_any(), body.into_any())
    }

    fn acct_row_view(a: AcctRow) -> AnyView {
        let occ = match a.occupant.class() {
            "" => view! { <span class="sub">{a.occupant.text()}</span> }.into_any(),
            class => view! { <span class=class><span data-t="">{a.occupant.text()}</span></span> }
                .into_any(),
        };
        let cells = match a.cells {
            Cells::Retired => view! {
                <div class="c-w c-ret">{state_icon(UNKNOWN)}<span>{label(RETIRED_KEY)}</span></div>
            }
            .into_any(),
            Cells::Windows(ws) => ws
                .into_iter()
                .map(|w| view! { <div class="c-w">{meter(w)}</div> })
                .collect_view()
                .into_any(),
        };
        let card = a.card;
        view! {
            <div class="c-name" tabindex="0" use:attach=card><span class="aname">{a.label}</span></div>
            <div>{occ}</div>
            {cells}
        }
        .into_any()
    }

    /// 窓の棒（見本の acct.js の meter・上限の線は置かない・無い窓は「―」）。
    fn meter(w: Option<WindowRow>) -> AnyView {
        match w {
            Some(w) => {
                let style = format!("width:{}%", w.width);
                view! {
                    <div class="meter">
                        <div class="bar"><i class=w.bar_class style=style></i></div>
                        <div class="v"><b class="num">{w.used}</b><span class="num">{format!("↻ {}", w.reset)}</span></div>
                    </div>
                }
                .into_any()
            }
            None => view! { <span class="sub">{super::NONE}</span> }.into_any(),
        }
    }

    pub fn moves_view(block: Block) -> AnyView {
        let home = read();
        let count = move || {
            home.with(|h| match &h.moves {
                Body::Filled(m) => Some(m.shown.len() + m.folded.len()),
                Body::Empty(_) => Some(0),
                Body::Unmeasured(_) => None,
            })
            .map(|n| view! { <span class="chip num">{n}</span> })
        };
        let body = move || body_or(home.get().moves, moves_body);
        view! {
            <details class=block.class id=block.id>
                <summary>{h2(block.heading)}{count}</summary>
                {body}
            </details>
        }
        .into_any()
    }

    fn moves_body(m: Moves) -> AnyView {
        let shown = m.shown.into_iter().map(mv_li).collect_view();
        let folded = m.more.map(|more| {
            let rest = m.folded.into_iter().map(mv_li).collect_view();
            view! {
                <details class="fold">
                    <summary class="num">{more}</summary>
                    <ul class="mv">{rest}</ul>
                </details>
            }
        });
        view! {
            <ul class="mv">{shown}</ul>
            {folded}
        }
        .into_any()
    }

    fn mv_li(m: MvRow) -> AnyView {
        let card = super::mv_card(&m);
        view! {
            <li tabindex="0" use:attach=card>
                <span class="ic moved" aria-hidden="true">"→"</span>
                <span class="num">{m.at}</span>
                <span class="what"><b data-t="">{m.group}</b>" "<span class="mono">{format!("{} → {}", m.from, m.to)}</span></span>
            </li>
        }
        .into_any()
    }
}
