//! block「pipeline」（見本の `#pipe` と ui.js の kcardHTML・便 g-pipe）: 4 列の板・札・「+n」・0 件の帯・hover の card。
//! 4 列の下に要修正の行（形の崩れた open の bead・0 本なら出さない・行 g-pipe-misfit）。
//! 板は口 /api/pipeline（契約の型の PipelineBoard）から、札の題は台帳の一覧の口（block ledger の定数）から読む。
//! 段から列への対応は契約の型の関数（`Stage::column`）を呼び、ここに対応の表を書かない。
//! 並べ方・字・札の中身・開いた列の query は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。
//! 着地の後の CI の読み（札の欄 ci・中核が判じた値を写すだけ・行 c-pipe-ci）: CI を待つ札は日を問わず Landed の列に出し、
//! 状態の記号を動いている印にする。結果の語は止まった列の札ではいつも、ほかの札では今までの経過が `CI_MARK_S` 以下の間だけ出す（`ci_shown`）。
//! 語は語の辞書の `CI_KEYS` の鍵から引く。
//! 台帳の一覧の項に出す段は札と同じ読みから `stages` 1 つで組む（行 c-ledger-stage のつなぎ）。
//! 札の欄 since は段を決めた時刻で、経過は面の時計の今から引く（行 c-abs-time）。札の meta の経過は 1 秒の時計（net の ticker）で
//! `age_at` から書き直し、CI の語と今日の着地と hover の card の値の行は block を組む時の今で決める（行 g-tick-adopt）。

use std::collections::BTreeMap;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{Ci, Misfit, PipelineBoard, PipelineCard, PipelineColumn, Reading};
use tsuzuri_contract::graph::{GraphDoc, title36};
use tsuzuri_contract::ledger::LedgerRow;
use tsuzuri_contract::wire;

use super::{Body, NO_CONTENT, NOT_READ, Staged, map};
use crate::frame::{self, Block};
use crate::mapview::graph::cut;
use crate::view::{Fetched, id_order, jst, read_rows};
use crate::vocab::label;
use crate::widgets::hover::Card;
use crate::widgets::nodecard::card_of;

pub const BLOCK: Block = Block {
    id: "pipe",
    heading: "pipeline",
    class: "panel",
};

/// 読みの口（便 g-parts）。
pub const PATH: &str = "/api/pipeline";

/// この file が字を持つ口の path（ほかの module の口を読む所は数えない・行 hs-derived）。
pub const PATHS: &[&str] = &[PATH];

/// この file の畳める段の開き閉じの鍵の形（無い・行 hs-derived）。
pub const FOLDS: &[&str] = &[];

/// 口が読めないときの理由。
pub const REASON: &str =
    "run の段を読む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 札が「まだ分からない」ときの理由（器の state dir が読めない）。
pub const UNKNOWN_REASON: &str = "器の state dir が読めず run の段がまだ分からない";

/// 0 件の帯の語の鍵（帯は語と数の 0）。
pub const RUN_KEY: &str = "run";

/// 列ごとに出す札の数（超える分は「+n」）。
pub const SHOW: usize = 3;

/// 開いた列の名を残す URL の query の鍵。
pub const QUERY_KEY: &str = "col";

/// 開いた列を畳む button の字（見出しでなく、「+n」と同じく file の定数の字・行 g-pipe-fold）。
pub const CLOSE: &str = "畳む";

/// hover の card の出所の行。
pub const SOURCE: &str = "fleet/events.jsonl";

/// 経過の値が無い札の字。
pub const NO_AGE: &str = "―";

/// 口座の値が無い札の字。
pub const NO_ACCOUNT: &str = "―";

/// 台帳で閉じた bead の札の段の理由の頭の字（中核の crate の pipeline の `CLOSED_TAG` の写し・面の crate は中核の crate に依存しない）。
pub const CLOSED_TAG: &str = "closed:";

/// 台帳で閉じた bead の着地しなかった札の段の字（行 c-pipe-closed）。
pub const CLOSED_STAGE: &str = "閉じた（着地せず）";

/// 要修正の行の見出しの語の鍵（行 g-pipe-misfit）。
pub const MISFIT_KEY: &str = "misfit";

/// 見本の案 A の横長の行の class。
pub const MISFIT_CLASS: &str = "fixrow";

/// 控えの印も設計の参照も無い bead の札の崩れの字（見本の案 A の字）。
pub const NEITHER_TEXT: &str = "印が無い — 設計の参照も控えの印も無い";

/// 控えの印と設計の参照の両方が在る bead の札の崩れの字（NEITHER_TEXT の対の字）。
pub const BOTH_TEXT: &str = "印が両方 — 設計の参照と控えの印の両方が在る";

/// 着地の後の CI の読みと語の辞書の鍵（読みの宣言の順・行 c-pipe-ci）。
pub const CI_KEYS: [(Ci, &str); 7] = [
    (Ci::Waiting, "ci_wait"),
    (Ci::Success, "ci_success"),
    (Ci::Failure, "ci_failure"),
    (Ci::Unmeasurable, "ci_unmeasurable"),
    (Ci::PushFailed, "ci_push_failed"),
    (Ci::CloseFailed, "ci_close_failed"),
    (Ci::Unreadable, "ci_unreadable"),
];

/// 結果の語を Landed の列ほかの札に出す経過の上限の秒（止まった列と CI を待つ札は上限なし）。
pub const CI_MARK_S: u64 = 600;

/// CI を待つ札の状態の記号の値（動いている）。
pub const CI_WAIT_STATE: &str = "run";

/// 1 つの列の見せ方（列・URL と class の名・見出しの語の鍵・札の状態の記号）。
/// 状態の記号が None の列（Landed）は取り込みの印を出す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lane {
    pub column: PipelineColumn,
    pub name: &'static str,
    pub key: &'static str,
    pub state: Option<&'static str>,
}

/// 4 列（板の順）。
pub const LANES: [Lane; 4] = [
    Lane {
        column: PipelineColumn::QueuedBlocked,
        name: "wait",
        key: "col_wait",
        state: Some("wait"),
    },
    Lane {
        column: PipelineColumn::RunningGated,
        name: "run",
        key: "col_run",
        state: Some("run"),
    },
    Lane {
        column: PipelineColumn::QuestionedFailedStopped,
        name: "stop",
        key: "col_stop",
        state: Some("wait"),
    },
    Lane {
        column: PipelineColumn::Landed,
        name: "land",
        key: "col_land",
        state: None,
    },
];

impl Lane {
    /// 止まった run の列（札は回数の代わりに段の理由を出す）。
    pub fn stops(self) -> bool {
        self.column == PipelineColumn::QuestionedFailedStopped
    }
}

/// 列の見せ方（4 列の表から引く）。
#[expect(clippy::expect_used, reason = "4 列の表は列の全部を持つ")]
pub fn lane(column: PipelineColumn) -> Lane {
    LANES
        .into_iter()
        .find(|l| l.column == column)
        .expect("4 列の表は列の全部を持つ")
}

/// 札の meta の 1 つ目（回数か、止まった列では段の理由）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lead {
    Runs(u32),
    Why(String),
}

/// 1 枚の札（見本の `.kcard`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kcard {
    pub id: String,
    /// 題（36 字に切った字・台帳の一覧から引けなければ None）。
    pub title: Option<String>,
    /// 状態の記号の値（None は取り込みの印）。
    pub state: Option<&'static str>,
    pub lead: Lead,
    pub age: String,
    /// 段を決めた時刻（電文の値のまま・描く時の経過は `age_at` が今から引く・行 g-tick-adopt）。
    pub since: Option<EpochSecs>,
    pub class: &'static str,
    pub hover: Card,
    /// 節点の card の値の行（見本の cardContent の data-run の枝・回数と段の名と 20 字に切った理由）。
    pub run_line: String,
    /// 節点の card の詳しく（理由が 20 字を越えれば 34 字以下の行に折った列・越えなければ空）。
    pub run_more: Vec<String>,
    /// 台帳で閉じた（着地せず）の札か（札の表の記号と meta の段の字を替える・行 g-closed-mark）。
    pub closed: bool,
    /// Landed の列ほかの札の meta に足す着地の後の CI の読み（`ci_shown` の値・止まった列の札は None で、語は lead に出る）。
    pub ci: Option<Ci>,
}

/// 値の行に出す理由の字数（見本の cut の 20）。
const WHY_CHARS: usize = 20;

/// 詳しくの 1 行の字数の上限（見本の chunk の 34）。
const MORE_CHARS: usize = 34;

/// 字を句切りの字（、。・，）の直後で片に分け、前から n 字以下の行に詰める（n 字を越える片は n 字ずつに切る・見本の chunk）。
fn chunk(s: &str, n: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Vec<char> = Vec::new();
    for piece in s.split_inclusive(['、', '。', '・', '，', '）']) {
        let mut p: Vec<char> = piece.chars().collect();
        if cur.len() + p.len() <= n {
            cur.extend(p);
            continue;
        }
        if !cur.is_empty() {
            out.push(cur.drain(..).collect());
        }
        while p.len() > n {
            out.push(p.drain(..n).collect());
        }
        cur = p;
    }
    if !cur.is_empty() {
        out.push(cur.into_iter().collect());
    }
    out
}

/// 1 つの列（見出しの語の鍵・class・経過の短い順の札の全部）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub lane: Lane,
    pub class: String,
    pub cards: Vec<Kcard>,
}

impl Column {
    /// 出す札（開いた列は全部・閉じた列は 3 枚まで）。
    pub fn shown(&self, open: bool) -> &[Kcard] {
        if open {
            &self.cards
        } else {
            self.cards.get(..SHOW).unwrap_or(&self.cards)
        }
    }

    /// 「+n」の n（開いた列と、3 枚以下の列は None）。
    pub fn more(&self, open: bool) -> Option<usize> {
        (!open && self.cards.len() > SHOW).then(|| self.cards.len() - SHOW)
    }

    /// 畳む button を出すか（開いた列で札が 3 枚を越えるときだけ・「+n」と同じ列に同時には出ない）。
    pub fn closable(&self, open: bool) -> bool {
        open && self.cards.len() > SHOW
    }
}

/// 描く時の経過の字（今から段を決めた時刻を引いて `age` に渡す・今より後の時刻は 0・時刻が無ければ `NO_AGE`・行 g-tick-adopt）。
pub fn age_at(since: Option<EpochSecs>, now: EpochSecs) -> String {
    since.map_or_else(|| NO_AGE.to_string(), |s| age(now.saturating_sub(s)))
}

/// 経過の字（60 秒未満は s・60 分未満は m・24 時間未満は h・それ以上は d・端数は切り捨て）。
pub fn age(secs: u64) -> String {
    match secs {
        s if s < 60 => format!("{s}s"),
        s if s < 3_600 => format!("{}m", s / 60),
        s if s < 86_400 => format!("{}h", s / 3_600),
        s => format!("{}d", s / 86_400),
    }
}

/// 口の本文を板の札に読む（まだ読んでいない・読めない・電文が読めない・札が「まだ分からない」は理由）。
/// 電文が読めない本文の理由は、中身の無い block と同じ（便 g-parts の歯が 3 値の理由を pin する）。
pub fn cards(fetched: &Fetched) -> Result<Vec<PipelineCard>, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(REASON),
        Fetched::Body(text) => match wire::decode::<PipelineBoard>(text) {
            Ok(PipelineBoard {
                cards: Reading::Known(cards),
                ..
            }) => Ok(cards),
            Ok(PipelineBoard {
                cards: Reading::Unknown,
                ..
            }) => Err(UNKNOWN_REASON),
            Err(_) => Err(NO_CONTENT),
        },
    }
}

/// 中身の有無（測れていない・0 件・札あり）。札の中身は `content` が組む。
pub fn body(fetched: &Fetched) -> Body<()> {
    match cards(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(c) if c.is_empty() => Body::Empty(RUN_KEY),
        Ok(_) => Body::Filled(()),
    }
}

/// 今日（日本の日）の着地か（段が Landed で段を決めた時刻が在り、その日本の日が今の日本の日と同じ）。
pub fn landed_today(card: &PipelineCard, now: EpochSecs) -> bool {
    card.stage.column() == PipelineColumn::Landed
        && card.since.is_some_and(|at| jst(at).0 == jst(now).0)
}

/// 着地の後の CI の読みの語の辞書の鍵。
#[expect(clippy::expect_used, reason = "鍵の表は読みの全部を持つ")]
pub fn ci_key(ci: Ci) -> &'static str {
    CI_KEYS
        .into_iter()
        .find(|(c, _)| *c == ci)
        .map(|(_, key)| key)
        .expect("鍵の表は読みの全部を持つ")
}

/// 札に出す CI の読み（CI を待つ札と止まった列の札はいつも・ほかは今までの経過が `CI_MARK_S` 以下のときだけ・読みが無ければ None）。
pub fn ci_shown(card: &PipelineCard, now: EpochSecs) -> Option<Ci> {
    let ci = card.ci?;
    (ci == Ci::Waiting
        || lane(card.stage.column()).stops()
        || card.since.is_some_and(|s| now.saturating_sub(s) <= CI_MARK_S))
    .then_some(ci)
}

/// CI の読みの語の style（待つは動いている色・成功は取り込みの色・ほかは止まった色の太字）。
pub fn ci_style(ci: Ci) -> &'static str {
    match ci {
        Ci::Waiting => "color:var(--s-run)",
        Ci::Success => "color:var(--s-land)",
        Ci::Failure | Ci::Unmeasurable | Ci::PushFailed | Ci::CloseFailed | Ci::Unreadable => {
            "color:var(--s-stop);font-weight:600"
        }
    }
}

/// 板の中身（4 列・Landed の列は今日の着地と CI を待つ札）。札の題は台帳の一覧の口の読みから引く（読めなければ全部の札が id だけ）。
pub fn content(pipe: &Fetched, ledger: &Fetched, now: EpochSecs) -> Body<Vec<Column>> {
    match cards(pipe) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(c) if c.is_empty() => Body::Empty(RUN_KEY),
        Ok(c) => {
            let rows = match read_rows(ledger) {
                Reading::Known(rows) => rows,
                Reading::Unknown => Vec::new(),
            };
            Body::Filled(columns(&c, &rows, now))
        }
    }
}

/// 札の題（台帳の一覧から bead の id で引いて 36 字に切る・引けなければ None）。
pub fn title_of(rows: &[LedgerRow], id: &str) -> Option<String> {
    rows.iter()
        .find(|r| r.id.as_str() == id)
        .map(|r| title36(&r.title))
}

/// 札を 4 列に組む（列は板の順・列の中は段を決めた時刻の新しい順・時刻の無い札は後・同じなら bead の id の順）。
/// Landed の列は今日（日本の日）の着地（`landed_today`）と、日を問わず CI を待つ札（欄 ci が Waiting）。
pub fn columns(cards: &[PipelineCard], rows: &[LedgerRow], now: EpochSecs) -> Vec<Column> {
    LANES
        .into_iter()
        .map(|lane| {
            let mut mine: Vec<&PipelineCard> = cards
                .iter()
                .filter(|c| c.stage.column() == lane.column)
                .filter(|c| {
                    lane.column != PipelineColumn::Landed
                        || landed_today(c, now)
                        || c.ci == Some(Ci::Waiting)
                })
                .collect();
            mine.sort_by(|a, b| {
                let key = |c: &PipelineCard| (c.since.is_none(), std::cmp::Reverse(c.since));
                key(a)
                    .cmp(&key(b))
                    .then_with(|| id_order(a.contract.as_str(), b.contract.as_str()))
            });
            let empty = if mine.is_empty() { " is-empty" } else { "" };
            Column {
                lane,
                class: format!("col c-{}{empty}", lane.name),
                cards: mine.into_iter().map(|c| kcard(c, rows, now)).collect(),
            }
        })
        .collect()
}

/// 台帳で閉じた bead の着地しなかった札か（段の列が Landed で、段の理由が `CLOSED_TAG` で始まる）。
pub fn closed_card(card: &PipelineCard) -> bool {
    card.stage.column() == PipelineColumn::Landed
        && card
            .reason
            .as_deref()
            .is_some_and(|r| r.starts_with(CLOSED_TAG))
}

/// 札の段の字（閉じた（着地せず）の札は `CLOSED_STAGE`・ほかは段の名）。
pub fn stage_word(card: &PipelineCard) -> String {
    if closed_card(card) {
        CLOSED_STAGE.to_string()
    } else {
        format!("{:?}", card.stage)
    }
}

/// 1 枚の札（止まった列は回数の代わりに段の理由・理由が空なら段の名）。
/// 閉じた（着地せず）の札は段の字を `CLOSED_STAGE` にし、hover の詳しくに閉じた理由を折って出す。
/// CI の読みを出す札（`ci_shown`）は理由の代わりに読みの語を出し、CI を待つ札の状態の記号は `CI_WAIT_STATE`。
/// 欄 ci は止まった列でなければ出す読み（止まった列の札は lead が読みの語なので None）。経過の字と CI の語は今 now で決める。
pub fn kcard(card: &PipelineCard, rows: &[LedgerRow], now: EpochSecs) -> Kcard {
    let lane = lane(card.stage.column());
    let id = card.contract.to_string();
    let title = title_of(rows, &id);
    let closed = closed_card(card);
    let stage = stage_word(card);
    let age = age_at(card.since, now);
    let shown = ci_shown(card, now);
    let why = match shown {
        Some(ci) => label(ci_key(ci)),
        None => card.reason.clone().unwrap_or_else(|| stage.clone()),
    };
    let run_line = format!("↻{} · {stage} · {}", card.runs, cut(&why, WHY_CHARS));
    let run_more = if why.chars().count() > WHY_CHARS {
        chunk(&why, MORE_CHARS)
    } else {
        Vec::new()
    };
    let more = if closed {
        chunk(&why, MORE_CHARS)
    } else {
        Vec::new()
    };
    let lead = if lane.stops() {
        Lead::Why(why)
    } else {
        Lead::Runs(card.runs)
    };
    let hover = Card {
        title: title.clone().unwrap_or_else(|| id.clone()),
        kind: format!("{RUN_KEY} · {stage}"),
        value: format!(
            "↻{} · {} · {age}",
            card.runs,
            card.account.as_deref().unwrap_or(NO_ACCOUNT)
        ),
        src: SOURCE.to_string(),
        more,
    };
    Kcard {
        id,
        title,
        state: if shown == Some(Ci::Waiting) {
            Some(CI_WAIT_STATE)
        } else {
            lane.state
        },
        lead,
        age,
        since: card.since,
        class: if lane.stops() {
            "kcard why-stop"
        } else {
            "kcard"
        },
        hover,
        run_line,
        run_more,
        closed,
        ci: if lane.stops() { None } else { shown },
    }
}

/// 台帳の一覧の項に出す板の段（札の bead の id の字の鍵・記号と閉じたかは札の読みの `kcard` の値・字は `stage_word` に
/// 札の meta の CI の語を ` · ` で足した字）。口が読めない・札がまだ分からない・電文が読めない間は空。
/// 器の局面の出力を読む後の行 c-ledger-lc はこの 1 つを替える。CI の語は今 now で決める。
pub fn stages(fetched: &Fetched, now: EpochSecs) -> BTreeMap<String, Staged> {
    let Ok(cards) = cards(fetched) else {
        return BTreeMap::new();
    };
    cards
        .iter()
        .map(|c| {
            let k = kcard(c, &[], now);
            let word = match k.ci {
                Some(ci) => format!("{} · {}", stage_word(c), label(ci_key(ci))),
                None => stage_word(c),
            };
            let staged = Staged {
                state: k.state,
                closed: k.closed,
                word,
            };
            (k.id, staged)
        })
        .collect()
}

/// 節点の card を札に付ける値（見本の cardContent の data-run の枝: 題と種類と帯と状態は節点から、
/// 値と出所と詳しくは走行から）。札の id の節点が電文に無ければ札の hover のまま。
pub fn node_hover(doc: &GraphDoc, card: &Kcard) -> Card {
    match card_of(doc, &card.id) {
        Some(node) => Card {
            title: node.title,
            kind: node.kind,
            value: card.run_line.clone(),
            src: card.hover.src.clone(),
            more: card.run_more.clone(),
        },
        None => card.hover.clone(),
    }
}

/// 板の全部の札の hover を節点の card に替える（グラフの口が読めない・まだ読んでいない間は受けた値のまま）。
pub fn with_nodes(body: Body<Vec<Column>>, graph: &Fetched) -> Body<Vec<Column>> {
    let Body::Filled(mut cols) = body else {
        return body;
    };
    if let Ok(doc) = map::doc(graph) {
        for card in cols.iter_mut().flat_map(|c| c.cards.iter_mut()) {
            card.hover = node_hover(&doc, card);
        }
    }
    Body::Filled(cols)
}

/// 札を押した先（契約 bead と同じ id の節点の頁・近傍と問いの card と同じ頁へ行く）。
pub fn card_href(card: &Kcard, mode: frame::Mode) -> String {
    frame::node_href(&card.id, mode)
}

/// URL の query から開いた列（`col=wait,land` の形・知らない名は読み捨てる・板の順）。
pub fn open_columns(search: &str) -> Vec<PipelineColumn> {
    let names: Vec<&str> = frame::param(search, QUERY_KEY)
        .map(|v| v.split(',').collect())
        .unwrap_or_default();
    LANES
        .into_iter()
        .filter(|l| names.contains(&l.name))
        .map(|l| l.column)
        .collect()
}

/// 列を開いた後の URL の query（開いた列の名を板の順で `col=` に残す・ほかの値と順はそのまま）。
pub fn with_open(search: &str, column: PipelineColumn) -> String {
    let mut open = open_columns(search);
    if !open.contains(&column) {
        open.push(column);
    }
    let names: Vec<&str> = LANES
        .into_iter()
        .filter(|l| open.contains(&l.column))
        .map(|l| l.name)
        .collect();
    frame::with_param(search, QUERY_KEY, &names.join(","))
}

/// 列を畳んだ後の URL の query（残りの開いた列の名を板の順で `col=` に残す・残りが無ければ `col` の片を全部外す）。
/// 空の字は返さない（history に空の字を渡すと今の URL が残るので、片が無ければ `?` だけ）。
pub fn with_closed(search: &str, column: PipelineColumn) -> String {
    let open = open_columns(search);
    let names: Vec<&str> = LANES
        .into_iter()
        .filter(|l| l.column != column && open.contains(&l.column))
        .map(|l| l.name)
        .collect();
    if !names.is_empty() {
        return frame::with_param(search, QUERY_KEY, &names.join(","));
    }
    let rest: Vec<&str> = search
        .strip_prefix('?')
        .unwrap_or(search)
        .split('&')
        .filter(|p| p.split('=').next() != Some(QUERY_KEY))
        .collect();
    format!("?{}", rest.join("&"))
}

/// 要修正の行の 1 枚の札（id・36 字に切った題・崩れの字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MisfitCard {
    pub id: String,
    pub title: String,
    pub why: &'static str,
}

/// 崩れの字（電文の語を写すだけで面は判じない）。
pub fn misfit_text(misfit: Misfit) -> &'static str {
    match misfit {
        Misfit::Neither => NEITHER_TEXT,
        Misfit::Both => BOTH_TEXT,
    }
}

/// 口の本文の欄 misfits を要修正の札に読む（器の判定の順のまま）。
/// 読んでいない・読めない・電文が読めない・欄が「まだ分からない」ときは空（行を出さない）。
pub fn misfit_cards(fetched: &Fetched) -> Vec<MisfitCard> {
    let Fetched::Body(text) = fetched else {
        return Vec::new();
    };
    match wire::decode::<PipelineBoard>(text) {
        Ok(PipelineBoard {
            misfits: Reading::Known(beads),
            ..
        }) => beads
            .iter()
            .map(|b| MisfitCard {
                id: b.bead.to_string(),
                title: title36(&b.title),
                why: misfit_text(b.misfit),
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// 要修正の札を押した先（bead と同じ id の節点の頁・pipeline の札の `card_href` と同じ先）。
pub fn misfit_href(card: &MisfitCard, mode: frame::Mode) -> String {
    frame::node_href(&card.id, mode)
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

#[cfg(target_arch = "wasm32")]
pub use dom::stage_sym;

/// 板の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;
    use tsuzuri_contract::EpochSecs;
    use tsuzuri_contract::board::PipelineColumn;

    use super::{
        BLOCK, CLOSE, CLOSED_STAGE, Column, Kcard, Lead, MISFIT_CLASS, MISFIT_KEY, MisfitCard, PATH,
        age_at, card_href, ci_key, ci_style, columns, content, misfit_cards, misfit_href,
        open_columns, with_closed, with_nodes, with_open,
    };
    use crate::frame::Mode;
    use crate::project::{Body, ledger, map, section, state_icon, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, hs};
    use crate::widgets::hover::attach;

    /// 回数の印（見本の IC.redo）。
    const REDO: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M20 12a8 8 0 1 1-2.3-5.7"/><path d="M20 4v5h-5"/></svg>"#;

    /// 経過の印（見本の IC.clock）。
    const CLOCK: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg>"#;

    /// 取り込みの印（見本の IC.check）。
    const CHECK: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12l5 5 9-10"/></svg>"#;

    /// 閉じた（着地せず）の印（見本の IC.cross・行 g-closed-mark）。
    const CROSS: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18"/></svg>"#;

    /// 止まった理由の印（見本の IC.stop）。
    const STOP: &str = r#"<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="6" y="6" width="12" height="12" rx="2"/></svg>"#;

    /// 今の URL の query（読めなければ空）。
    fn search() -> String {
        window().location().search().unwrap_or_default()
    }

    /// 列を開き、開いた列の名を URL の query に残す（頁は読み直さない）。
    fn open_column(open: RwSignal<Vec<PipelineColumn>>, column: PipelineColumn) {
        let url = with_open(&search(), column);
        if let Ok(history) = window().history() {
            let _ = history.replace_state_with_url(
                &web_sys::wasm_bindgen::JsValue::NULL,
                "",
                Some(&url),
            );
        }
        open.set(open_columns(&url));
    }

    /// 列を畳み、その列の名を URL の query から外す（頁は読み直さない・履歴に積まない）。
    fn close_column(open: RwSignal<Vec<PipelineColumn>>, column: PipelineColumn) {
        let url = with_closed(&search(), column);
        if let Ok(history) = window().history() {
            let _ = history.replace_state_with_url(
                &web_sys::wasm_bindgen::JsValue::NULL,
                "",
                Some(&url),
            );
        }
        open.set(open_columns(&url));
    }

    pub fn view() -> AnyView {
        let pipe = crate::net::read(PATH);
        let rows = crate::net::read(ledger::PATH);
        let graph = crate::net::read(map::PATH);
        let open = RwSignal::new(open_columns(&search()));
        let ctx = use_context::<HelpCtx>();
        let mode = move || match ctx {
            Some(c) => c.mode.get(),
            None => Mode::from_query(&search()),
        };
        // 札の経過は 1 秒の時計で書き直す（時計の今から段を決めた時刻を引く）。
        let tick = crate::net::ticker();
        let clock = move || tick.get();
        let body = move || {
            let now = crate::net::now();
            let board = match pipe.with(|p| rows.with(|l| graph.with(|g| with_nodes(content(p, l, now), g)))) {
                Body::Unmeasured(reason) => unmeasured(reason),
                Body::Empty(key) => view! {
                    <div class="empty"><span>{label(key)}</span><b class="num">"0"</b></div>
                    {board_view(columns(&[], &[], now), open, mode, clock)}
                }
                .into_any(),
                Body::Filled(cols) => board_view(cols, open, mode, clock),
            };
            let fix = pipe.with(|p| misfit_view(misfit_cards(p), mode()));
            view! { {board}{fix} }.into_any()
        };
        section(BLOCK, ().into_any(), body.into_any())
    }

    /// 4 列の下の要修正の行（見本の案 A・札が 0 枚なら出さない・札は押すと bead と同じ id の節点の頁へ）。
    fn misfit_view(cards: Vec<MisfitCard>, mode: Mode) -> Option<AnyView> {
        if cards.is_empty() {
            return None;
        }
        let cards = cards
            .into_iter()
            .map(|c| {
                view! {
                    <a class="fixcard" href=misfit_href(&c, mode)>
                        <span class="kid">{c.id.clone()}</span>
                        <span class="tt">{c.title.clone()}</span>
                        <span class="why">{c.why}</span>
                    </a>
                }
            })
            .collect_view();
        Some(
            view! {
                <div class=MISFIT_CLASS>
                    <header>{hs(MISFIT_KEY)}</header>
                    <div class="fixcards">{cards}</div>
                </div>
            }
            .into_any(),
        )
    }

    fn board_view(
        cols: Vec<Column>,
        open: RwSignal<Vec<PipelineColumn>>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
        clock: impl Fn() -> EpochSecs + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let cols = cols
            .into_iter()
            .map(|c| column_view(c, open, mode, clock))
            .collect_view();
        view! { <div class="board">{cols}</div> }.into_any()
    }

    fn column_view(
        col: Column,
        open: RwSignal<Vec<PipelineColumn>>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
        clock: impl Fn() -> EpochSecs + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let column = col.lane.column;
        let is_open = move || open.with(|o| o.contains(&column));
        let count = col.cards.len();
        let class = col.class.clone();
        let key = col.lane.key;
        let shown = col.clone();
        let closing = col.clone();
        let cards = move || {
            shown
                .shown(is_open())
                .iter()
                .map(|c| kcard_view(c, mode(), clock))
                .collect_view()
        };
        let more = move || {
            col.more(is_open()).map(|n| {
                view! {
                    <button type="button" class="more num" aria-label=n.to_string() on:click=move |_| open_column(open, column)>
                        {format!("+{n}")}
                    </button>
                }
            })
        };
        let close = move || {
            closing.closable(is_open()).then(|| {
                view! {
                    <button type="button" class="more" on:click=move |_| close_column(open, column)>
                        {CLOSE}
                    </button>
                }
            })
        };
        view! {
            <div class=class>
                <header><span class="dot" aria-hidden="true"></span>{hs(key)}<span class="cnt num">{count}</span></header>
                <div class="cards">{cards}</div>
                {more}
                {close}
            </div>
        }
        .into_any()
    }

    /// 1 枚の札（押すと契約 bead と同じ id の節点の頁へ・指を置くと hover の card）。
    fn kcard_view(
        card: &Kcard,
        mode: Mode,
        clock: impl Fn() -> EpochSecs + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let sym = stage_sym(card.closed, card.state);
        let since = card.since;
        let age = move || age_at(since, clock());
        let closed = card.closed.then(|| view! { <span>{CLOSED_STAGE}</span> });
        let ci = card
            .ci
            .map(|c| view! { <span style=ci_style(c)>{label(ci_key(c))}</span> });
        let title = card
            .title
            .clone()
            .map(|t| view! { <span class="tt" data-t="">{t}</span> });
        let lead = match &card.lead {
            Lead::Runs(n) => view! { <span><span inner_html=REDO></span>{*n}</span> }.into_any(),
            Lead::Why(w) => {
                view! { <span class="why"><span inner_html=STOP></span>{w.clone()}</span> }
                    .into_any()
            }
        };
        view! {
            <a class=card.class href=card_href(card, mode) use:attach=card.hover.clone()>
                <div class="t">{sym}{title}</div>
                <div class="m">
                    <span class="kid">{card.id.clone()}</span>
                    {lead}
                    {closed}
                    {ci}
                    <span><span inner_html=CLOCK></span><span class="num">{age}</span></span>
                </div>
            </a>
        }
        .into_any()
    }

    /// 札と台帳の一覧の項の段の記号（閉じた（着地せず）は ✕・状態の記号の値が在ればその記号・無ければ取り込みの印）。
    pub fn stage_sym(closed: bool, state: Option<&'static str>) -> AnyView {
        if closed {
            view! {
                <span class="st" style="color:var(--ink-3)" aria-label=CLOSED_STAGE inner_html=CROSS></span>
            }
            .into_any()
        } else {
            match state {
                Some(v) => state_icon(v),
                None => {
                    view! { <span class="st" style="color:var(--s-land)" inner_html=CHECK></span> }
                        .into_any()
                }
            }
        }
    }
}
