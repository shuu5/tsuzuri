//! block「pipeline」（見本の `#pipe` と ui.js の kcardHTML・便 g-pipe）: 5 列の板・札・「+n」・0 件の帯。
//! 列は Blocked と Queued を分けた 5 つ（判断の記録 ADR-27 決定 (6)・行 c-pipe-five）。
//! 5 列の下に要修正の行（形の崩れた open の bead・0 本なら出さない・行 g-pipe-misfit）。
//! 板は口 /api/pipeline（契約の型の PipelineBoard）から、札の題は台帳の一覧の口（block ledger の定数）から読む。
//! 段から列への対応は契約の型の関数（`Stage::column`）を呼び、ここに対応の表を書かない。
//! 並べ方・字・札の中身・開いた列の query は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。
//! 着地の後の CI の読み（札の欄 ci・中核が判じた値を写すだけ・行 c-pipe-ci）: CI を待つ札は着地の時刻を問わず Landed の列に出し、
//! 状態の記号を動いている印にする。結果の語は止まった列の札ではいつも、ほかの札では今までの経過が `CI_MARK_S` 以下の間だけ出す（`ci_shown`）。
//! 語は語の辞書の `CI_KEYS` の鍵から引く。
//! 札の欄 since は段を決めた時刻で、経過は面の時計の今から引く（行 c-abs-time）。札の meta の経過は 1 秒の時計（net の ticker）で
//! `age_at` から書き直し、CI の語と直近の着地は block を組む時の今で決める（行 g-tick-adopt）。
//! 着地の列は今から `LAND_WINDOW_S`（12 時間・規則の行 R-36）の内の着地を出す（`landed_recent`・行 c-landed-12h）。
//! 札は短い題（bead の事実の short・無ければ 36 字の題・無ければ id）と段ごとの要の 1 行（widgets の keyline・判断の記録 ADR-27 決定 (6)・
//! Queued の 30 分越えの注意は規則の行 R-37）を出し、押すと吹き出し（widgets の pop）を開く。hover の card は札に付けない（行 g-pipe-cards・
//! 札の hover の card の値の組みと節点の card への替えと札の link の先は行 g-dead-sweep-a で消した）。
//! 見出しの epic の chip（`chips`・見本の renderEchips）と一覧の組の頭の名は epic を選び、選んだ組でない札を薄くし、吹き出しの
//! 開いている札に輪の印を付ける（組の鍵は一覧の組と同じ `ledgerlist::key_of`・行 g-select）。
//! 留め置きの札は Blocked の列に出し、席の止めと受付の断りで別の印（widgets の keyline の `HeldBy`）を札の meta に置く。Blocked の札は印を持たない。
//! Blocked の列の見出しの数は Blocked と Held の内訳（`count_text`・判断の記録 ADR-42 決定 (3)(7)・行 g-held-col）。

use std::collections::BTreeMap;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{Ci, Misfit, PipelineBoard, PipelineCard, PipelineColumn, Reading};
use tsuzuri_contract::graph::title36;
use tsuzuri_contract::ledger::{BeadFact, LedgerRow};
use tsuzuri_contract::wire;

use super::ledger::OUTSIDE;
use super::{Body, NO_CONTENT, NOT_READ};
use crate::frame::{self, Block};
use crate::ledgerlist::{OUTSIDE_KEY, key_of, short_of};
use crate::view::{Fetched, id_order, read_rows};
use crate::vocab::label;
use crate::widgets::keyline::{HeldBy, LineSrc, held_by, line_src};
use crate::widgets::pop::{self, Src};

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

/// 着地の列に出す着地の範囲の秒（今から 12 時間 以内・規則の行 R-36・判断の記録 ADR-27 決定 (6)）。
pub const LAND_WINDOW_S: u64 = 43_200;

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

/// 5 列（板の順・Blocked・Queued・Running / Gated・止まり・着地）。
pub const LANES: [Lane; 5] = [
    Lane {
        column: PipelineColumn::Blocked,
        name: "block",
        key: "col_block",
        state: Some("wait"),
    },
    Lane {
        column: PipelineColumn::Queued,
        name: "queue",
        key: "col_queue",
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
        key: "col_land_12h",
        state: None,
    },
];

impl Lane {
    /// 止まった run の列（札は回数の代わりに段の理由を出す）。
    pub fn stops(self) -> bool {
        self.column == PipelineColumn::QuestionedFailedStopped
    }
}

/// 列の見せ方（5 列の表から引く）。
#[expect(clippy::expect_used, reason = "5 列の表は列の全部を持つ")]
pub fn lane(column: PipelineColumn) -> Lane {
    LANES
        .into_iter()
        .find(|l| l.column == column)
        .expect("5 列の表は列の全部を持つ")
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
    /// 台帳で閉じた（着地せず）の札か（札の表の記号と meta の段の字を替える・行 g-closed-mark）。
    pub closed: bool,
    /// Landed の列ほかの札の meta に足す着地の後の CI の読み（`ci_shown` の値・止まった列の札は None で、語は lead に出る）。
    pub ci: Option<Ci>,
    /// 短い題（bead の事実の short・`with_lines` が置く・置くまでは None・行 g-pipe-cards）。
    pub short: Option<String>,
    /// 要の 1 行の材料（`with_lines` が置く・置くまでは None で要の 1 行を出さない）。
    pub line: Option<LineSrc>,
    /// 留め置きの札の止めた者（印を出す・ほかの段は None・行 g-held-col）。
    pub held: Option<HeldBy>,
}


/// 札の短い題（bead の事実の short・事実に無ければ 36 字の題・題も無ければ id）。
pub fn short_title(facts: &Reading<&[BeadFact]>, card: &Kcard) -> String {
    let found = match facts {
        Reading::Known(f) => f.iter().find(|x| x.id.as_str() == card.id),
        Reading::Unknown => None,
    };
    found.map_or_else(
        || card.title.clone().unwrap_or_else(|| card.id.clone()),
        |x| x.short.clone(),
    )
}

/// 板の全部の札に短い題と要の 1 行の材料を置く（材料の札の電文は材料の札の列から id で引く・中身の無い板は受けた値のまま）。
pub fn with_lines(body: Body<Vec<Column>>, src: &Src<'_>) -> Body<Vec<Column>> {
    let Body::Filled(mut cols) = body else {
        return body;
    };
    for card in cols.iter_mut().flat_map(|c| c.cards.iter_mut()) {
        card.short = Some(short_title(&src.facts, card));
        card.line = pop::card_of(src.cards, &card.id).map(|c| line_src(c, src));
    }
    Body::Filled(cols)
}

/// 止まった列の札の class。
const STOP_CLASS: &str = "kcard why-stop";

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

/// 列の見出しの数（Blocked の列は Blocked と Held の内訳 `<Blocked の数>·<Held の数>`・ほかの列は札の数・行 g-held-col）。
/// 内訳は札の要の 1 行と同じ中黒（U+00B7）でつなぎ、空白を挟まない（斜線は分数と読まれ、空白は狭い列で数の中の折れ目になる）。
pub fn count_text(col: &Column) -> String {
    let n = col.cards.len();
    if col.lane.column != PipelineColumn::Blocked {
        return n.to_string();
    }
    let held = col.cards.iter().filter(|c| c.held.is_some()).count();
    format!("{}·{held}", n - held)
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

/// 直近の着地か（段が Landed で段を決めた時刻が在り、今からその時刻を引いた経過が `LAND_WINDOW_S` 以下・今より後の時刻は経過 0）。
pub fn landed_recent(card: &PipelineCard, now: EpochSecs) -> bool {
    card.stage.column() == PipelineColumn::Landed
        && card
            .since
            .is_some_and(|at| now.saturating_sub(at) <= LAND_WINDOW_S)
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

/// 板の中身（5 列・Landed の列は直近の着地と CI を待つ札）。札の題は台帳の一覧の口の読みから引く（読めなければ全部の札が id だけ）。
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

/// 札を 5 列に組む（列は板の順・列の中は段を決めた時刻の新しい順・時刻の無い札は後・同じなら bead の id の順）。
/// Landed の列は直近の着地（`landed_recent`）と、着地の時刻を問わず CI を待つ札（欄 ci が Waiting）。
pub fn columns(cards: &[PipelineCard], rows: &[LedgerRow], now: EpochSecs) -> Vec<Column> {
    LANES
        .into_iter()
        .map(|lane| {
            let mut mine: Vec<&PipelineCard> = cards
                .iter()
                .filter(|c| c.stage.column() == lane.column)
                .filter(|c| {
                    lane.column != PipelineColumn::Landed
                        || landed_recent(c, now)
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

/// 札の段の記号の材料（台帳で閉じた（着地せず）の札か・状態の記号の値〔CI を待つ札は `CI_WAIT_STATE`・ほかは列の値〕）。
/// 板の札と台帳 open の一覧の行の段の字の頭が、同じ値で `stage_sym` を描く（行 g-list-sym・判断の記録 ADR-30 決定 (9)）。
pub fn card_sym(card: &PipelineCard, now: EpochSecs) -> (bool, Option<&'static str>) {
    let state = if ci_shown(card, now) == Some(Ci::Waiting) {
        Some(CI_WAIT_STATE)
    } else {
        lane(card.stage.column()).state
    };
    (closed_card(card), state)
}

/// 1 枚の札（止まった列は回数の代わりに段の理由・理由が空なら段の名）。
/// 閉じた（着地せず）の札は段の字を `CLOSED_STAGE` にする。
/// CI の読みを出す札（`ci_shown`）は理由の代わりに読みの語を出し、CI を待つ札の状態の記号は `CI_WAIT_STATE`。
/// 欄 ci は止まった列でなければ出す読み（止まった列の札は lead が読みの語なので None）。経過の字と CI の語は今 now で決める。
pub fn kcard(card: &PipelineCard, rows: &[LedgerRow], now: EpochSecs) -> Kcard {
    let lane = lane(card.stage.column());
    let id = card.contract.to_string();
    let title = title_of(rows, &id);
    let (closed, state) = card_sym(card, now);
    let stage = stage_word(card);
    let age = age_at(card.since, now);
    let shown = ci_shown(card, now);
    let why = match shown {
        Some(ci) => label(ci_key(ci)),
        None => card.reason.clone().unwrap_or_else(|| stage.clone()),
    };
    let lead = if lane.stops() {
        Lead::Why(why)
    } else {
        Lead::Runs(card.runs)
    };
    Kcard {
        id,
        title,
        state,
        lead,
        age,
        since: card.since,
        class: if lane.stops() { STOP_CLASS } else { "kcard" },
        closed,
        ci: if lane.stops() { None } else { shown },
        short: None,
        line: None,
        held: held_by(card),
    }
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

/// すべての epic の chip の字（押すと選びを解く）。
pub const ALL_EPICS: &str = "すべて";

/// 見出しの epic の chip の 1 つ（組の鍵・名・板に出ている札の数・すべての chip の鍵は空）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chip {
    pub key: String,
    pub name: String,
    pub n: usize,
}

/// 板の札の組の鍵（札の bead の id から一覧の組と同じ読み・台帳が読めなければ空の表）。
pub fn card_keys(
    cards: &[PipelineCard],
    rows: &Reading<Vec<LedgerRow>>,
) -> BTreeMap<String, String> {
    let Reading::Known(rows) = rows else {
        return BTreeMap::new();
    };
    cards
        .iter()
        .map(|c| (c.contract.to_string(), key_of(c.contract.as_str(), rows)))
        .collect()
}

/// 見出しの epic の chip（頭はすべての chip で数は板に出ている札の全部、続けて板に出ている札を持つ組を札の多い順〔同じ数は
/// 鍵の id の順〕に・名は一覧の組の頭と同じ epic の短い題〔epic の外の組は `OUTSIDE`・台帳に無い epic は鍵のまま〕・見本の renderEchips）。
pub fn chips(
    cols: &[Column],
    keys: &BTreeMap<String, String>,
    rows: &[LedgerRow],
    facts: &BTreeMap<String, BeadFact>,
) -> Vec<Chip> {
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    let all: Vec<&Kcard> = cols.iter().flat_map(|c| c.cards.iter()).collect();
    for card in &all {
        let key = keys.get(&card.id).map_or(OUTSIDE_KEY, String::as_str);
        *count.entry(key).or_default() += 1;
    }
    let name = |key: &str| {
        if key == OUTSIDE_KEY {
            return OUTSIDE.to_string();
        }
        rows.iter()
            .find(|r| r.id.as_str() == key)
            .map_or_else(|| key.to_string(), |r| short_of(r, facts))
    };
    let mut groups: Vec<Chip> = count
        .into_iter()
        .map(|(key, n)| Chip {
            key: key.to_string(),
            name: name(key),
            n,
        })
        .collect();
    groups.sort_by(|a, b| b.n.cmp(&a.n).then_with(|| id_order(&a.key, &b.key)));
    let head = Chip {
        key: String::new(),
        name: ALL_EPICS.to_string(),
        n: all.len(),
    };
    std::iter::once(head).chain(groups).collect()
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

    use std::collections::BTreeMap;

    use super::{
        BLOCK, CLOSE, CLOSED_STAGE, Column, Kcard, Lead, MISFIT_CLASS, MISFIT_KEY, MisfitCard,
        PATH, age_at, card_keys, chips, ci_key, ci_style, columns, content, count_text,
        misfit_cards, misfit_href, open_columns, with_closed, with_lines, with_open,
    };
    use crate::frame::Mode;
    use crate::ledgerlist::{SelCtx, dim, facts, hover_lit, pick, ring};
    use crate::tiles::{open_of, press, tiles};
    use tsuzuri_contract::case::PATH as CASES_PATH;

    use crate::project::{Body, ledger, section, state_icon, unmeasured};
    use crate::view::read_rows;
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, hs};
    use crate::widgets::keyline::key_line;
    use crate::widgets::pop::{BEADS_PATH, PopCtx, Src, Via, known, read_facts, read_parts};
    use tsuzuri_contract::board::Reading;

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
        let beads = crate::net::read(BEADS_PATH);
        let cases = crate::net::read(CASES_PATH);
        let open = RwSignal::new(open_columns(&search()));
        // スマホの段の tile を押した記録（押すまでは None で、開く段は札の在る一番急ぐ段・行 g-layout）。
        let picked = RwSignal::new(None);
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
            let facts = beads.with(read_facts);
            let parts = cases.with(read_parts);
            let ledger_rows = rows.with(read_rows);
            let cards = pipe.with(|p| super::cards(p).unwrap_or_default());
            let keys = StoredValue::new(card_keys(&cards, &ledger_rows));
            let src = Src {
                facts: known(&facts),
                rows: known(&ledger_rows),
                cards: &cards,
                parts: known(&parts),
                graph: None,
            };
            let board = match pipe.with(|p| rows.with(|l| with_lines(content(p, l, now), &src))) {
                Body::Unmeasured(reason) => unmeasured(reason),
                Body::Empty(key) => view! {
                    <div class="empty"><span>{label(key)}</span><b class="num">"0"</b></div>
                    {board_view(columns(&[], &[], now), open, mode, clock, keys)}
                }
                .into_any(),
                Body::Filled(cols) => with_tiles(cols, open, mode, clock, (keys, picked)),
            };
            let fix = pipe.with(|p| misfit_view(misfit_cards(p), mode()));
            view! { {board}{fix} }.into_any()
        };
        // 見出しの epic の chip（押すと epic を選び・すべての chip か同じ chip で解く・行 g-select）。
        let sel = use_context::<SelCtx>();
        let head = move || {
            let sel = sel?;
            let Body::Filled(cols) = pipe.with(|p| rows.with(|l| content(p, l, crate::net::now())))
            else {
                return None;
            };
            // 台帳が読めない間は組が分からないので chip を出さない。
            let ledger_rows = rows.with(read_rows);
            let Reading::Known(list) = &ledger_rows else {
                return None;
            };
            let keys = card_keys(
                &pipe.with(|p| super::cards(p).unwrap_or_default()),
                &ledger_rows,
            );
            let all = beads.with(|b| chips(&cols, &keys, list, &facts(b)));
            Some(chips_view(all, sel))
        };
        section(BLOCK, head.into_any(), body.into_any())
    }

    /// 見出しの epic の chip の並び（選んでいる組の chip は on・何も選んでいない間はすべての chip が on）。
    fn chips_view(all: Vec<super::Chip>, sel: SelCtx) -> AnyView {
        let items = all
            .into_iter()
            .map(|c| {
                let on = {
                    let key = c.key.clone();
                    move || {
                        sel.epic
                            .with(|e| e.as_deref().map_or(key.is_empty(), |s| s == key))
                    }
                };
                let key = c.key;
                let choose = move |_| sel.epic.update(|e| *e = pick(e.as_deref(), &key));
                view! {
                    <button type="button" class="pchip" class:on=on on:click=choose>
                        {c.name}<b class="num">{c.n}</b>
                    </button>
                }
            })
            .collect_view();
        view! { <div class="pchips" role="group">{items}</div> }.into_any()
    }

    /// 札の選びの包み（選んだ epic の組でない札は薄く・吹き出しの開いている札は輪・行 g-select・
    /// 一覧の行にマウスの pointer が載っている札は輪より薄い hover の印・行 g-list-hover）。
    fn pick_view(
        card: &Kcard,
        keys: StoredValue<BTreeMap<String, String>>,
        inner: AnyView,
    ) -> AnyView {
        let sel = use_context::<SelCtx>();
        let pop = use_context::<PopCtx>();
        let key = keys.with_value(|k| k.get(&card.id).cloned());
        let dimmed =
            move || sel.is_some_and(|s| s.epic.with(|e| dim(e.as_deref(), key.as_deref())));
        let id = card.id.clone();
        let hovered = {
            let id = id.clone();
            move || {
                let shown = pop.and_then(PopCtx::shown);
                sel.is_some_and(|s| {
                    s.hover
                        .with(|h| hover_lit(shown.as_deref(), h.as_deref(), &id))
                })
            }
        };
        let ringed = move || ring(pop.and_then(PopCtx::shown).as_deref(), &id);
        view! { <div class="kpick" class:dim=dimmed class:ring=ringed class:hov=hovered>{inner}</div> }
            .into_any()
    }

    /// 5 列の下の要修正の行（見本の案 A・札が 0 枚なら出さない・札は押すと bead と同じ id の節点の頁へ）。
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
        keys: StoredValue<BTreeMap<String, String>>,
    ) -> AnyView {
        let cols = cols
            .into_iter()
            .map(|c| column_view(c, open, mode, clock, keys))
            .collect_view();
        view! { <div class="board">{cols}</div> }.into_any()
    }

    /// 板とスマホの段の tile（`marks` は札の組の鍵と tile を押した記録・行 g-layout）。
    fn with_tiles(
        cols: Vec<Column>,
        open: RwSignal<Vec<PipelineColumn>>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
        clock: impl Fn() -> EpochSecs + Copy + Send + Sync + 'static,
        marks: (Keys, RwSignal<Option<Option<PipelineColumn>>>),
    ) -> AnyView {
        let (keys, picked) = marks;
        let tiles = tiles_view(cols.clone(), mode, clock, keys, picked);
        view! { {board_view(cols, open, mode, clock, keys)}{tiles} }.into_any()
    }

    /// 札の組の鍵の表（札の bead の id から一覧の組の鍵）。
    type Keys = StoredValue<BTreeMap<String, String>>;

    /// スマホの段の tile と開いた段の札の並び（幅 600 px 以下だけ stylesheet が出す・行 g-layout・見本の mpipe）。
    fn tiles_view(
        cols: Vec<Column>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
        clock: impl Fn() -> EpochSecs + Copy + Send + Sync + 'static,
        keys: StoredValue<BTreeMap<String, String>>,
        picked: RwSignal<Option<Option<PipelineColumn>>>,
    ) -> AnyView {
        let cols = StoredValue::new(cols);
        let open = move || cols.with_value(|c| open_of(picked.get(), c));
        let row = move || {
            let now = open();
            cols.with_value(|c| tiles(c, now, clock()))
                .into_iter()
                .map(|t| {
                    let column = t.lane.column;
                    let class = format!("ptile c-{}{}", t.lane.name, if t.open { " on" } else { "" });
                    let warn = t.warn.then(|| view! { <span class="ptwarn">"⚠"</span> });
                    view! {
                        <button type="button" class=class aria-expanded=t.open.to_string() on:click=move |_| picked.set(Some(press(now, column)))>
                            <span class="pt1"><b class="num">{t.n}</b>{warn}</span>
                            <small>{label(t.lane.key)}</small>
                        </button>
                    }
                })
                .collect_view()
        };
        let list = move || {
            let now = open()?;
            let col = cols.with_value(|c| c.iter().find(|c| c.lane.column == now).cloned())?;
            let cards = col
                .cards
                .iter()
                .map(|c| pick_view(c, keys, kcard_view(c, mode(), clock)))
                .collect_view();
            Some(view! {
                <div class="ptlist">
                    <div class="ptlh">{hs(col.lane.key)}</div>
                    {cards}
                </div>
            })
        };
        view! { <div class="ptiles"><div class="ptrow">{row}</div>{list}</div> }.into_any()
    }

    fn column_view(
        col: Column,
        open: RwSignal<Vec<PipelineColumn>>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
        clock: impl Fn() -> EpochSecs + Copy + Send + Sync + 'static,
        keys: StoredValue<BTreeMap<String, String>>,
    ) -> AnyView {
        let column = col.lane.column;
        let is_open = move || open.with(|o| o.contains(&column));
        let count = count_text(&col);
        let class = col.class.clone();
        let key = col.lane.key;
        let shown = col.clone();
        let closing = col.clone();
        let cards = move || {
            shown
                .shown(is_open())
                .iter()
                .map(|c| pick_view(c, keys, kcard_view(c, mode(), clock)))
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

    /// 1 枚の札（押すと吹き出しを開き・もう 1 度押すと閉じる・hover の card は付けない・行 g-pipe-cards）。
    fn kcard_view(
        card: &Kcard,
        _mode: Mode,
        clock: impl Fn() -> EpochSecs + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let sym = stage_sym(card.closed, card.state);
        let since = card.since;
        let age = move || age_at(since, clock());
        let closed = card.closed.then(|| view! { <span>{CLOSED_STAGE}</span> });
        let ci = card
            .ci
            .map(|c| view! { <span style=ci_style(c)>{label(ci_key(c))}</span> });
        let held = card
            .held
            .map(|h| view! { <span class=h.class()>{label(h.key())}</span> });
        let title = card
            .short
            .clone()
            .or_else(|| card.title.clone())
            .map(|t| view! { <span class="tt" data-t="">{t}</span> });
        // 要の 1 行は 1 秒の時計で書き直す（材料が無ければ出さない）。
        let line = card.line.clone().map(|src| {
            let src = StoredValue::new(src);
            let now = move || src.with_value(|s| key_line(s, clock()));
            view! { <div class=move || now().class>{move || now().text}</div> }
        });
        let pop = use_context::<PopCtx>();
        let id = card.id.clone();
        let press = move |_| {
            if let Some(p) = pop {
                p.press(&id, Via::Card);
            }
        };
        let lead = match &card.lead {
            Lead::Runs(n) => view! { <span><span inner_html=REDO></span>{*n}</span> }.into_any(),
            Lead::Why(w) => {
                view! { <span class="why"><span inner_html=STOP></span>{w.clone()}</span> }
                    .into_any()
            }
        };
        view! {
            <button type="button" class=card.class data-pop-card=card.id.clone() on:click=press>
                <div class="t">{sym}{title}</div>
                <div class="m">
                    <span class="kid">{card.id.clone()}</span>
                    {lead}
                    {held}
                    {closed}
                    {ci}
                    <span><span inner_html=CLOCK></span><span class="num">{age}</span></span>
                </div>
                {line}
            </button>
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
