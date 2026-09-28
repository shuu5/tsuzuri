//! block「pipeline」（見本の `#pipe` と ui.js の kcardHTML・便 g-pipe）: 4 列の板・札・「+n」・0 件の帯・hover の card。
//! 板は口 /api/pipeline（契約の型の PipelineBoard）から、札の題は台帳の一覧の口（block ledger の定数）から読む。
//! 段から列への対応は契約の型の関数（`Stage::column`）を呼び、ここに対応の表を書かない。
//! 並べ方・字・札の中身・開いた列の query は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineBoard, PipelineCard, PipelineColumn, Reading};
use tsuzuri_contract::graph::{GraphDoc, title36};
use tsuzuri_contract::ledger::LedgerRow;
use tsuzuri_contract::wire;

use super::{Body, NO_CONTENT, NOT_READ, map};
use crate::frame::{self, Block};
use crate::mapview::graph::cut;
use crate::view::{Fetched, id_order, read_rows};
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
    pub class: &'static str,
    pub hover: Card,
    /// 節点の card の値の行（見本の cardContent の data-run の枝・回数と段の名と 20 字に切った理由）。
    pub run_line: String,
    /// 節点の card の詳しく（理由が 20 字を越えれば 34 字以下の行に折った列・越えなければ空）。
    pub run_more: Vec<String>,
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
            &self.cards[..self.cards.len().min(SHOW)]
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
            }) => Ok(cards),
            Ok(PipelineBoard {
                cards: Reading::Unknown,
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

/// 1 日の秒。
const DAY: EpochSecs = 86_400;

/// 今日（UTC）の着地か（段が Landed で経過が在り、今の時刻から経過を引いた時刻が今の時刻を含む UTC の日の始まり以上）。
/// 経過は server が読んだ時の今からの秒なので、読んだ時と描く時の差の数秒は許す。
pub fn landed_today(card: &PipelineCard, now: EpochSecs) -> bool {
    card.stage.column() == PipelineColumn::Landed
        && card
            .elapsed_s
            .and_then(|e| now.checked_sub(e))
            .is_some_and(|at| at >= now / DAY * DAY)
}

/// 板の中身（4 列・Landed の列は今日の着地だけ）。札の題は台帳の一覧の口の読みから引く（読めなければ全部の札が id だけ）。
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

/// 札を 4 列に組む（列は板の順・列の中は経過の短い順・経過の無い札は後・同じなら bead の id の順）。
/// Landed の列は今日（UTC）の着地だけ（`landed_today`）。
pub fn columns(cards: &[PipelineCard], rows: &[LedgerRow], now: EpochSecs) -> Vec<Column> {
    LANES
        .into_iter()
        .map(|lane| {
            let mut mine: Vec<&PipelineCard> = cards
                .iter()
                .filter(|c| c.stage.column() == lane.column)
                .filter(|c| lane.column != PipelineColumn::Landed || landed_today(c, now))
                .collect();
            mine.sort_by(|a, b| {
                let key = |c: &PipelineCard| (c.elapsed_s.is_none(), c.elapsed_s);
                key(a)
                    .cmp(&key(b))
                    .then_with(|| id_order(a.contract.as_str(), b.contract.as_str()))
            });
            let empty = if mine.is_empty() { " is-empty" } else { "" };
            Column {
                lane,
                class: format!("col c-{}{empty}", lane.name),
                cards: mine.into_iter().map(|c| kcard(c, rows)).collect(),
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

/// 1 枚の札（止まった列は回数の代わりに段の理由・理由が空なら段の名）。
/// 閉じた（着地せず）の札は段の字を `CLOSED_STAGE` にし、hover の詳しくに閉じた理由を折って出す。
pub fn kcard(card: &PipelineCard, rows: &[LedgerRow]) -> Kcard {
    let lane = lane(card.stage.column());
    let id = card.contract.to_string();
    let title = title_of(rows, &id);
    let closed = closed_card(card);
    let stage = if closed {
        CLOSED_STAGE.to_string()
    } else {
        format!("{:?}", card.stage)
    };
    let age = card.elapsed_s.map_or_else(|| NO_AGE.to_string(), age);
    let why = card.reason.clone().unwrap_or_else(|| stage.clone());
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
        state: lane.state,
        lead,
        age,
        class: if lane.stops() {
            "kcard why-stop"
        } else {
            "kcard"
        },
        hover,
        run_line,
        run_more,
    }
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

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 板の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;
    use tsuzuri_contract::board::PipelineColumn;

    use super::{
        BLOCK, CLOSE, Column, Kcard, Lead, PATH, card_href, columns, content, open_columns,
        with_closed, with_nodes, with_open,
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
        let body = move || {
            let now = crate::net::now();
            match pipe.with(|p| rows.with(|l| graph.with(|g| with_nodes(content(p, l, now), g)))) {
                Body::Unmeasured(reason) => unmeasured(reason),
                Body::Empty(key) => view! {
                    <div class="empty"><span>{label(key)}</span><b class="num">"0"</b></div>
                    {board_view(columns(&[], &[], now), open, mode)}
                }
                .into_any(),
                Body::Filled(cols) => board_view(cols, open, mode),
            }
        };
        section(BLOCK, ().into_any(), body.into_any())
    }

    fn board_view(
        cols: Vec<Column>,
        open: RwSignal<Vec<PipelineColumn>>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let cols = cols
            .into_iter()
            .map(|c| column_view(c, open, mode))
            .collect_view();
        view! { <div class="board">{cols}</div> }.into_any()
    }

    fn column_view(
        col: Column,
        open: RwSignal<Vec<PipelineColumn>>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
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
                .map(|c| kcard_view(c, mode()))
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
    fn kcard_view(card: &Kcard, mode: Mode) -> AnyView {
        let sym = match card.state {
            Some(v) => state_icon(v),
            None => view! { <span class="st" style="color:var(--s-land)" inner_html=CHECK></span> }
                .into_any(),
        };
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
                    <span><span inner_html=CLOCK></span><span class="num">{card.age.clone()}</span></span>
                </div>
            </a>
        }
        .into_any()
    }
}
