//! 札と一覧の行の吹き出し（判断の記録 ADR-27 決定 (7)・見本 board-v2 の popHTML と placePop と openPop と closePop）:
//! pipeline の札か台帳 open の一覧の行を押すと、その bead の吹き出しを頁に 1 つ出す（同じ口をもう 1 度押すと閉じる）。
//! 吹き出しは × と外の click と取り消しの鍵（Esc）で閉じ、閉じる判定は窓の枠の `modal::closes` を使う（2 つ目を書かない）。
//! どの段も題の全体・概要・id・種類・epic・設計の行・起票の時刻と経過・個別の頁への口を出し、段ごとの欄を足す
//! （Blocked は待つ相手〔相手の段つき・押すと相手の吹き出し〕と待ちの長さ・Queued は列に入った時刻・Running / Gated は
//! run の回と口座・止まりは理由の全文と run の回と口座と次の手・着地は着地の時刻と CI）。読めない欄は「まだ分からない」。
//! 板の札が読めない間は段ごとの欄を出せないので、吹き出しの末に板の読めない理由を測れていないの 1 行で出す（札が無いとは見せない）。
//! 材料は bead の事実の口（`BEADS_PATH`・短い題・起票・blocks の相手）と局面の出力の口（待つ相手の links.on）と、
//! 板と台帳の一覧とグラフの口（段・題・種類・親・設計の pointer）。幅 600 px 以下（規則の行 R-35）は下からの板（sheet）。
//! 欄の組みと置き場と開閉の判定は純粋な関数にして host で試し、層の DOM（`PopLayer`）は wasm の target だけ。
//! 札と行に口を付けるのは板の行と選びの行。
//! 段の流れと run の歴と着地の commit は、開いた bead の走行の読みの口を層が読み `runflow::with_runs` で足す。
//! Queued の起きない理由と Blocked の待つ理由・相手の便・承認の相手は `with_why` が足す: 理由の語は器の列の待ちの
//! 12 語（`REASONS`）を平易な字にした語の辞書の鍵 `qr:<語>` で引き、表に無い語と読めない理由はまだ分からない。
//! どの段の吹き出しも末の口の並びに相談の口を 1 つ置き、押すとその bead を題に入れた相談の窓を開く（hover では開かない）。
//! 概要は吹き出しを開いた時に開いた bead の 1 本の引きの口（`ITEM_PATH`）の本文から読み、表示の型の 1 つを部品 sumpick で選んで
//! 切らずに出し、`POP_SUM_MAX` 字を越える時は頭と … で畳んで口を押すと開く（判断の記録 ADR-30 決定 (2)(3)）。
//! Held（留め置き）は止めた者・理由・止めた時刻と経過・解く条件の 4 つ（`held_facts`・個別の頁も同じ関数で組む・
//! 判断の記録 ADR-42 決定 (7)）。理由は局面の出力の契約の部品の欄 why（無ければまだ分からない）。
//! 受付の断りの名は器の 27 語（`REFUSALS`）を平易な字にした語の辞書の鍵 `rf:<名>` で引く（表に無い名は名のまま）。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineCard, Reading, Stage};
use tsuzuri_contract::case::{CaseDoc, CasePart};
use tsuzuri_contract::graph::{GraphDoc, NodeKind};
use tsuzuri_contract::ledger::{BeadFact, BeadFacts, FACTS_PATH, ITEM_PATH, LedgerItem, LedgerRow};
use tsuzuri_contract::summary::{excerpt, summaries};
use tsuzuri_contract::wire;

use super::hover::{Point, Rect, Size};
use super::keyline::{HeldBy, held_by};
use super::modal::Hit;
use super::runflow::{Hist, Seg};
use super::sumpick::{self, POP_SUM_MAX, Picked};
use crate::frame::{self, Mode};
use crate::mapview::band::kind_key;
use crate::project::{pipeline, timeline};
use crate::topbar::{Win, win_href};
use crate::view::{Fetched, clock_short};
use crate::vocab::label;

/// bead の事実の口の path（契約の型の字・行 c-bead-route）。
pub const BEADS_PATH: &str = FACTS_PATH;

/// 吹き出しの箱の id（見本の `#pop`・頁に 1 つ）。
pub const ID: &str = "pop";

/// 幅 600 px 以下の幕の id（見本の `#popScrim`・stylesheet の media の規則でだけ見える）。
pub const SCRIM: &str = "popscrim";

/// 吹き出しの頭・本文・題の全体・概要・欄の表・足・相手の一覧・口・まだ分からないの class
/// （見本の `.ph`・`.pb`・`.pt`・`.psum`・`.facts`・`.pf`・`.bl`・`.lk`・`.faint`）。
pub const CLASSES: [&str; 9] = ["ph", "pb", "pt", "psum", "facts", "pf", "pbl", "plk", "pun"];

/// 札と行の口の印の属性（押した口の bead の id を値に持つ・外の click の判定と吹き出しの置き場が引く）。
pub const CARD_ATTR: &str = "data-pop-card";

/// 一覧の行の口の印の属性。
pub const ROW_ATTR: &str = "data-pop-row";

/// 口の矩形と吹き出しの間（px・見本の placePop の 8）。
pub const GAP_PX: f64 = 8.0;

/// 口の下へ回したときの間（px・見本の 6）。
pub const BELOW_PX: f64 = 6.0;

/// 吹き出しの上端の下限（px・上の帯の下・見本の 60）。
pub const TOP_MIN_PX: f64 = 60.0;

/// 口が見つからないときの上端（px・見本の 80）。
pub const NO_ANCHOR_TOP_PX: f64 = 80.0;

/// × の button の語の鍵（aria-label）。
pub const CLOSE_KEY: &str = "pop_close";

/// 個別の頁への口の語の鍵。
pub const PAGE_KEY: &str = "pop_page";

/// 読めない欄の語の鍵（まだ分からない）。
pub const UNKNOWN_KEY: &str = "pop_unknown";

/// 共通の欄の語の鍵（出す順・id・種類・epic・設計の行・起票）。
pub const COMMON_KEYS: [&str; 5] = ["pf_id", "pf_kind", "pf_epic", "pf_row", "pf_created"];

/// 段ごとの欄の語の鍵（待つ相手・待ちの長さ・列に入った・run の回・口座・理由・着地・CI）。
pub const STAGE_KEYS: [&str; 8] = [
    "pf_wait_on",
    "pf_waited",
    "pf_queued",
    "pf_runs",
    "pf_account",
    "pf_reason",
    "pf_landed",
    "pf_ci",
];

/// 止まりの次の手の語の鍵（問いに答える・run の記録）。
pub const NEXT_KEYS: [&str; 2] = ["pnext_ask", "pnext_runs"];

/// 設計の pointer の行の頭の字（中核のグラフの BeadAttr の pointers の行の頭）。
pub const POINTER_HEAD: &str = "design = ";

/// 器の列の待ちの理由の語（器の pipe/dispatch.rs の WAIT_REASONS の 12 語・宣言の順・行 g-pop-why）。
pub const REASONS: [&str; 12] = [
    "dependency",
    "overlap",
    "admission",
    "host-busy",
    "hold",
    "launched",
    "settled",
    "no-design-pointer",
    "unreflected-ruling",
    "floor",
    "reserved",
    "sibling",
];

/// 理由の語の辞書の鍵の頭（鍵は頭に理由の語を続けた字）。
pub const REASON_HEAD: &str = "qr:";

/// 起きない理由・待つ理由・相手の便の欄の語の鍵。
pub const WHY_KEYS: [&str; 3] = ["pf_why", "pf_wait_why", "pf_wait_runs"];

/// 承認待ちの相手（持ち主）と待つ理由（持ち主の承認）の語の鍵。
pub const APPROVAL_KEYS: [&str; 2] = ["pw_owner", "pw_approval"];

/// 概要の口の語の鍵（畳んだ形の開く口・開いた形の畳む口・行 g-pop-sum）。
pub const SUM_KEYS: [&str; 2] = ["psum_more", "psum_less"];

/// 概要の口の class。
pub const SUM_CLASS: &str = "pmore";

/// 留め置きの欄の語の鍵（止めた者・理由・止めた時刻・解く条件・行 g-held-pop）。
pub const HELD_KEYS: [&str; 4] = ["pf_held_by", "pf_held_why", "pf_held_at", "pf_held_until"];

/// 止めた者ごとの止めた者の語の鍵と解く条件の語の鍵（席の止め・受付の断り）。
pub const HELD_WORDS: [(HeldBy, &str, &str); 2] = [
    (HeldBy::Seat, "hb:seat", "hu:seat"),
    (HeldBy::Intake, "hb:intake", "hu:intake"),
];

/// 器の受付の断りの名（器の pipe/refuse.rs の REFUSALS の 27 語・宣言の順・行 g-held-name）。
pub const REFUSALS: [&str; 27] = [
    "not-a-repo",
    "duplicate-run",
    "write-set-overlap",
    "write-set-unreadable",
    "write-set-incomplete",
    "write-set-dir-without-slash",
    "contract-table",
    "write-set-item-unresolved",
    "cap-headroom",
    "name-unresolved",
    "write-set-drift",
    "teeth-place-unresolved",
    "also-names-rust",
    "tests-not-a-teeth-file",
    "fn-undeclared",
    "teeth-outside-write-set",
    "hand-written-contract",
    "same-kind-repeated",
    "finding-unaddressed",
    "promised-field-written",
    "promise-symbol-unresolved",
    "max-live",
    "entrance-not-red",
    "ruling-unresolved",
    "index-building",
    "code-facts",
    "code-facts-unmeasured",
];

/// 断りの名の語の辞書の鍵の頭（鍵は頭に断りの名を続けた字）。
pub const REFUSAL_HEAD: &str = "rf:";

/// 局面の出力の契約の列の待ちの局面の語（器の case-lifecycle §2・中核の pipeline の QUEUED_PHASE の写し）。
pub const QUEUED_PHASE: &str = "contract-queued";

/// 局面の出力の契約の段を最新の便の部品が持つ局面の語（札の段が Blocked なら便は承認待ちの run-blocked）。
pub const RUNNING_PHASE: &str = "contract-running";

/// 吹き出しを開いた口（見本の S.pop.via）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    /// pipeline の札。
    Card,
    /// 台帳 open の一覧の行。
    Row,
}

impl Via {
    /// 口の印の属性。
    pub fn attr(self) -> &'static str {
        match self {
            Via::Card => CARD_ATTR,
            Via::Row => ROW_ATTR,
        }
    }
}

/// 開いている吹き出し（bead の id と開いた口）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open {
    pub id: String,
    pub via: Via,
}

/// 口を押した後の吹き出し（同じ bead を同じ口で押すと閉じ・ほかは押した bead を開く・見本の act.card と act.row）。
pub fn toggle(now: Option<&Open>, next: Open) -> Option<Open> {
    (now != Some(&next)).then_some(next)
}

/// 頁の click の押し（吹き出しの中は中の click・口の上は口が自分で開け閉めするので None・ほかは外の click）。
pub fn hit_of(in_pop: bool, on_opener: bool) -> Option<Hit<'static>> {
    if in_pop {
        Some(Hit::Inside)
    } else if on_opener {
        None
    } else {
        Some(Hit::Outside)
    }
}

/// 口の要素を探す selector（押した口の種類を先に・無ければほかの種類・見本の anchorEl）。
pub fn anchor_selectors(open: &Open) -> [String; 2] {
    let other = match open.via {
        Via::Card => Via::Row,
        Via::Row => Via::Card,
    };
    [open.via, other].map(|v| format!("[{}=\"{}\"]", v.attr(), open.id))
}

/// 口の要素のどれかを探す selector（外の click の判定）。
pub fn opener_selector() -> String {
    format!("[{CARD_ATTR}],[{ROW_ATTR}]")
}

/// 吹き出しの左上（見本の placePop）: 口の右に置き、窓の右端を越えれば左、それでも左端を越えれば口の下へ回し、
/// 窓の下端を越えれば上へ寄せる（上端は `TOP_MIN_PX` より上へ出さない）。口が見つからなければ窓の真ん中の上。
pub fn place(anchor: Option<Rect>, pop: Size, window: Size) -> Point {
    let Some(r) = anchor else {
        return Point {
            x: ((window.width - pop.width) / 2.0).max(GAP_PX),
            y: NO_ANCHOR_TOP_PX,
        };
    };
    let mut x = r.left + r.width + GAP_PX;
    let mut y = r.top;
    if x + pop.width > window.width - GAP_PX {
        x = r.left - GAP_PX - pop.width;
    }
    if x < GAP_PX {
        x = (window.width - pop.width - GAP_PX).min(r.left).max(GAP_PX);
        y = r.top + r.height + BELOW_PX;
    }
    if y + pop.height > window.height - GAP_PX {
        y = (window.height - GAP_PX - pop.height).max(TOP_MIN_PX);
    }
    Point { x, y }
}

/// 待つ相手の 1 つ（id・短い題・相手の札の段〔札が無ければ None〕）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partner {
    pub id: String,
    pub short: String,
    pub stage: Option<Stage>,
}

/// 欄の値。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    /// 字のまま。
    Text(String),
    /// 等幅の字（id・設計の行）。
    Code(String),
    /// 時刻（日本時間の字と今からの経過を出す）。
    At(EpochSecs),
    /// 種類の語の鍵と札の段。
    Kind(&'static str, Option<Stage>),
    /// epic（id と短い題・押すと epic の吹き出し）。
    Epic(Partner),
    /// 待つ相手の一覧（押すと相手の吹き出し）。
    Partners(Vec<Partner>),
    /// 段の流れ（流れの段と表に無い段の語・行 g-pop-flow）。
    Flow(Vec<Seg>, Vec<String>),
    /// run の歴（行 g-pop-flow）。
    Hist(Vec<Hist>),
    /// まだ分からない。
    Unknown,
}

/// 欄の 1 行（語の鍵と値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    pub key: &'static str,
    pub val: Val,
}

/// 吹き出しの概要（行 g-pop-sum）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sum {
    /// 1 本の引きをまだ読めない（まだ分からない）。
    Unread,
    /// 2 つの概要も本文の頭の 1 行も無い（要約なし）。
    Empty,
    /// 表示の型で選んだ概要（字は切らない）。
    Picked(Picked),
}

/// 吹き出しの中身（頭の短い題・題の全体・概要・札の段・欄・止まりの次の手の語の鍵）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pop {
    pub id: String,
    pub short: String,
    pub title: Option<String>,
    pub summary: Sum,
    pub stage: Option<Stage>,
    pub facts: Vec<Fact>,
    pub next: Option<&'static str>,
}

/// 吹き出しの材料（口ごとの読み・読めない口の欄はまだ分からない）。
#[derive(Debug, Clone)]
pub struct Src<'a> {
    pub facts: Reading<&'a [BeadFact]>,
    pub rows: Reading<&'a [LedgerRow]>,
    pub cards: &'a [PipelineCard],
    pub parts: Reading<&'a [CasePart]>,
    pub graph: Option<&'a GraphDoc>,
}

/// 板の札の読めない理由（読めれば None・`pipeline::cards` の理由）。
pub fn board_unread(fetched: &Fetched) -> Option<&'static str> {
    pipeline::cards(fetched).err()
}

/// bead の事実の口の本文を読む（読めなければ Unknown）。
pub fn read_facts(fetched: &Fetched) -> Reading<Vec<BeadFact>> {
    match fetched {
        Fetched::Body(text) => wire::decode::<BeadFacts>(text).map_or(Reading::Unknown, |f| f.rows),
        Fetched::NotRead | Fetched::Failed => Reading::Unknown,
    }
}

/// 局面の出力の口の本文を部品の列に読む（読めなければ Unknown）。
pub fn read_parts(fetched: &Fetched) -> Reading<Vec<CasePart>> {
    match fetched {
        Fetched::Body(text) => wire::decode::<CaseDoc>(text).map_or(Reading::Unknown, |d| d.parts),
        Fetched::NotRead | Fetched::Failed => Reading::Unknown,
    }
}

/// 読みの中身を借りる。
pub fn known<T>(r: &Reading<Vec<T>>) -> Reading<&[T]> {
    match r {
        Reading::Known(v) => Reading::Known(v.as_slice()),
        Reading::Unknown => Reading::Unknown,
    }
}

fn fact_of<'a>(src: &Src<'a>, id: &str) -> Reading<Option<&'a BeadFact>> {
    match src.facts {
        Reading::Known(f) => Reading::Known(f.iter().find(|x| x.id.as_str() == id)),
        Reading::Unknown => Reading::Unknown,
    }
}

fn row_of<'a>(src: &Src<'a>, id: &str) -> Option<&'a LedgerRow> {
    match src.rows {
        Reading::Known(rows) => rows.iter().find(|r| r.id.as_str() == id),
        Reading::Unknown => None,
    }
}

/// bead の札（板に無ければ None）。
pub fn card_of<'a>(cards: &'a [PipelineCard], id: &str) -> Option<&'a PipelineCard> {
    cards.iter().find(|c| c.contract.as_str() == id)
}

/// 短い題（事実の short・事実が無ければ id）。
pub fn short_of(src: &Src<'_>, id: &str) -> String {
    match fact_of(src, id) {
        Reading::Known(Some(f)) => f.short.clone(),
        _ => id.to_string(),
    }
}

/// 相手の 1 つ（短い題と札の段）。
pub fn partner(src: &Src<'_>, id: &str) -> Partner {
    Partner {
        id: id.to_string(),
        short: short_of(src, id),
        stage: card_of(src.cards, id).map(|c| c.stage),
    }
}

/// 契約の部品（局面の出力の種類 contract で id が bead の id）。
pub fn contract_part<'a>(parts: &'a [CasePart], id: &str) -> Option<&'a CasePart> {
    parts.iter().find(|p| p.part == "contract" && p.id == id)
}

/// 待つ相手の id: 局面の出力の契約の部品が在ればその links.on、無ければ bead の事実の blocks のうち台帳で閉じていない相手。
/// 両方とも読めなければ None。
pub fn wait_on(src: &Src<'_>, id: &str) -> Option<Vec<String>> {
    if let Reading::Known(parts) = src.parts
        && let Some(p) = contract_part(parts, id)
    {
        return Some(p.links.on.clone());
    }
    let Reading::Known(Some(f)) = fact_of(src, id) else {
        return None;
    };
    let open = |b: &str| row_of(src, b).is_none_or(|r| r.status != "closed");
    Some(
        f.blocks
            .iter()
            .map(|b| b.to_string())
            .filter(|b| open(b))
            .collect(),
    )
}

/// 親をたどって最初の epic（自分は数えない・台帳が読めなければ None）。
pub fn epic_of(src: &Src<'_>, id: &str) -> Option<String> {
    let mut at = row_of(src, id)?.parent.clone();
    for _ in 0..16 {
        let row = row_of(src, at.as_ref()?.as_str())?;
        if row.node_kind() == NodeKind::Epic {
            return Some(row.id.to_string());
        }
        at.clone_from(&row.parent);
    }
    None
}

/// 設計の行（グラフの bead の pointers の最初の行から頭の `POINTER_HEAD` を外した字）。
/// pointer の行が無ければ、contracts がちょうど 1 つで空の字でない時にその id（ほかは None）。
pub fn row_pointer(doc: &GraphDoc, id: &str) -> Option<String> {
    let bead = doc.beads.get(id)?;
    if let Some(p) = bead.pointers.first() {
        return Some(p.strip_prefix(POINTER_HEAD).unwrap_or(p).trim().to_string());
    }
    match bead.contracts.as_slice() {
        [contract] if !contract.is_empty() => Some(contract.clone()),
        _ => None,
    }
}

/// 時刻の字（日本時間の短い字と今からの経過）。
pub fn at_text(at: EpochSecs, now: EpochSecs) -> String {
    format!(
        "{}（{}）",
        clock_short(at, now),
        pipeline::age(now.saturating_sub(at))
    )
}

fn or_unknown<T>(v: Option<T>, f: impl FnOnce(T) -> Val) -> Val {
    v.map_or(Val::Unknown, f)
}

/// 止まりの次の手（Questioned は問いに答える・Failed と Stopped は run の記録・ほかの段は無い）。
pub fn next_key(stage: Stage) -> Option<&'static str> {
    match stage {
        Stage::Questioned => Some(NEXT_KEYS[0]),
        Stage::Failed | Stage::Stopped => Some(NEXT_KEYS[1]),
        _ => None,
    }
}

/// 次の手の先（質問の窓を開いた home の頁か、個別の頁の run の時間軸の block）。
pub fn next_href(key: &str, id: &str, mode: Mode) -> String {
    if key == NEXT_KEYS[0] {
        win_href(Win::Ask, mode)
    } else {
        format!("{}#{}", frame::node_href(id, mode), timeline::BLOCK.id)
    }
}

/// 段ごとの欄（札が無い bead は無い）。
pub fn stage_facts(src: &Src<'_>, card: &PipelineCard) -> Vec<Fact> {
    let f = |key, val| Fact { key, val };
    let account = || or_unknown(card.account.clone(), Val::Text);
    let since = || or_unknown(card.since, Val::At);
    match card.stage {
        Stage::Blocked => {
            let on = wait_on(src, card.contract.as_str());
            let list = match on {
                Some(ids) if !ids.is_empty() => {
                    Val::Partners(ids.iter().map(|x| partner(src, x)).collect())
                }
                _ => Val::Unknown,
            };
            vec![f(STAGE_KEYS[0], list), f(STAGE_KEYS[1], since())]
        }
        Stage::Queued => vec![f(STAGE_KEYS[2], since())],
        Stage::Held => held_facts(src, card),
        Stage::Running | Stage::Gated => vec![
            f(STAGE_KEYS[3], Val::Text(card.runs.to_string())),
            f(STAGE_KEYS[4], account()),
        ],
        Stage::Questioned | Stage::Failed | Stage::Stopped => vec![
            f(STAGE_KEYS[5], or_unknown(card.reason.clone(), Val::Text)),
            f(STAGE_KEYS[3], Val::Text(card.runs.to_string())),
            f(STAGE_KEYS[4], account()),
        ],
        Stage::Landed => vec![
            f(STAGE_KEYS[6], since()),
            f(
                STAGE_KEYS[7],
                or_unknown(card.ci, |c| Val::Text(label(pipeline::ci_key(c)))),
            ),
        ],
    }
}

/// 留め置きの理由の字（局面の出力の契約の部品の欄 why・部品が無いか読めないか欄が無ければ None）。
pub fn held_why(src: &Src<'_>, card: &PipelineCard) -> Option<String> {
    match src.parts {
        Reading::Known(parts) => contract_part(parts, card.contract.as_str())?.why.clone(),
        Reading::Unknown => None,
    }
}

/// 受付の断りの名の平易な字（名の `:` の後の詳細は外して `REFUSALS` の語なら語の辞書の鍵 `rf:<名>` の字・表に無い名は
/// 名のまま・名が無ければまだ分からない・行 g-held-name）。
pub fn refusal_text(name: Option<&str>) -> String {
    let Some(name) = name else {
        return label(UNKNOWN_KEY);
    };
    let word = name.split(':').next().unwrap_or(name);
    if REFUSALS.contains(&word) {
        label(&format!("{REFUSAL_HEAD}{word}"))
    } else {
        name.to_string()
    }
}

/// 留め置きの欄（`HELD_KEYS` の順・段 Held でない札は空）。止めた者は席か器の受付、理由は席の止めなら why の字
/// （無ければまだ分からない）、受付の断りなら断りの名の字と why（無ければまだ分からない）を全角のコロンでつなぐ。
/// 止めた時刻は札の since（無ければまだ分からない）、解く条件は止めた者ごとの字。
pub fn held_facts(src: &Src<'_>, card: &PipelineCard) -> Vec<Fact> {
    let Some(by) = held_by(card) else {
        return Vec::new();
    };
    let [by_key, why_key, at_key, until_key] = HELD_KEYS;
    let (who, until) = HELD_WORDS
        .into_iter()
        .find(|(b, _, _)| *b == by)
        .map_or((UNKNOWN_KEY, UNKNOWN_KEY), |(_, w, u)| (w, u));
    let why = held_why(src, card);
    let reason = match by {
        HeldBy::Seat => or_unknown(why, Val::Text),
        HeldBy::Intake => Val::Text(format!(
            "{}：{}",
            refusal_text(card.reason.as_deref()),
            why.unwrap_or_else(|| label(UNKNOWN_KEY))
        )),
    };
    let f = |key, val| Fact { key, val };
    vec![
        f(by_key, Val::Text(label(who))),
        f(why_key, reason),
        f(at_key, or_unknown(card.since, Val::At)),
        f(until_key, Val::Text(label(until))),
    ]
}

/// 吹き出しの中身（共通の欄を `COMMON_KEYS` の順に・札が在れば段ごとの欄を足す）。
/// epic の欄は親をたどって epic が在るときだけ（台帳が読めなければまだ分からない）、設計の行は pointer が在るときだけ
/// （グラフが読めなければまだ分からない）。
pub fn pop(id: &str, src: &Src<'_>) -> Pop {
    let fact = fact_of(src, id);
    let row = row_of(src, id);
    let card = card_of(src.cards, id);
    let stage = card.map(|c| c.stage);
    let mut facts = vec![Fact {
        key: COMMON_KEYS[0],
        val: Val::Code(id.to_string()),
    }];
    let kind = or_unknown(row, |r| Val::Kind(kind_key(r.node_kind()), stage));
    facts.push(Fact {
        key: COMMON_KEYS[1],
        val: kind,
    });
    let epic = match src.rows {
        Reading::Known(_) => epic_of(src, id).map(|e| Val::Epic(partner(src, &e))),
        Reading::Unknown => Some(Val::Unknown),
    };
    let pointer = match src.graph {
        Some(doc) => row_pointer(doc, id).map(Val::Code),
        None => Some(Val::Unknown),
    };
    for (key, val) in [(COMMON_KEYS[2], epic), (COMMON_KEYS[3], pointer)] {
        if let Some(val) = val {
            facts.push(Fact { key, val });
        }
    }
    let created = match fact {
        Reading::Known(Some(f)) => f.created_at,
        _ => None,
    };
    facts.push(Fact {
        key: COMMON_KEYS[4],
        val: or_unknown(created, Val::At),
    });
    if let Some(c) = card {
        facts.extend(stage_facts(src, c));
    }
    let short = match fact {
        Reading::Known(Some(f)) => f.short.clone(),
        _ => id.to_string(),
    };
    Pop {
        id: id.to_string(),
        short,
        title: row.map(|r| r.title.clone()),
        summary: Sum::Unread,
        stage,
        facts,
        next: stage.and_then(next_key),
    }
}

/// 開いた bead の 1 本の引きの口の path（行 g-pop-sum）。
pub fn item_path(id: &str) -> String {
    format!("{ITEM_PATH}{id}")
}

/// 吹き出しに概要を置く（行 g-pop-sum）: 開いた bead の 1 本の引きの電文の本文から 2 つの概要と本文の頭の 1 行を読み、
/// 表示の型の 1 つを部品 sumpick で選ぶ。引きをまだ読めない・読めない・ほかの bead の電文の間は Unread、どれも無ければ Empty。
pub fn with_sum(mut p: Pop, item: &Fetched, mode: Mode) -> Pop {
    let read = match item {
        Fetched::Body(text) => wire::decode::<LedgerItem>(text).ok(),
        Fetched::NotRead | Fetched::Failed => None,
    };
    p.summary = match read {
        Some(it) if it.row.id.as_str() == p.id => {
            let two = summaries(&it.description);
            let body = excerpt(&it.description);
            let picked = sumpick::pick(
                mode,
                two.plain.as_deref(),
                two.eng.as_deref(),
                body.as_deref(),
            );
            picked.map_or(Sum::Empty, Sum::Picked)
        }
        _ => Sum::Unread,
    };
    p
}

/// 概要の見せる字と口の語の鍵: `POP_SUM_MAX` 字以下は全文で口なし、越えれば畳んだ形は頭と … と開く口、
/// 開いた形は全文と畳む口（行 g-pop-sum）。
pub fn sum_text(text: &str, whole: bool) -> (String, Option<&'static str>) {
    let f = sumpick::fold(text, POP_SUM_MAX);
    match (f.rest.is_some(), whole) {
        (false, _) => (f.head, None),
        (true, false) => (f.shut(), Some(SUM_KEYS[0])),
        (true, true) => (f.whole(), Some(SUM_KEYS[1])),
    }
}

/// 理由の語の辞書の鍵（語の `:` の後の詳細は外す・`REASONS` に無い語は None）。
pub fn reason_key(word: &str) -> Option<String> {
    let w = word.split(':').next().unwrap_or(word);
    REASONS.contains(&w).then(|| format!("{REASON_HEAD}{w}"))
}

/// 理由の欄の値（平易な字・理由が無いか表に無い語はまだ分からない）。
pub fn reason_val(word: Option<&str>) -> Val {
    word.and_then(reason_key)
        .map_or(Val::Unknown, |k| Val::Text(label(&k)))
}

/// 札の待ちの種類。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wait {
    /// 列の待ち（理由の語・契約の部品の links.runs）。
    Queue(Option<String>, Vec<String>),
    /// 便の承認待ち（器の run-blocked・相手は持ち主）。
    Approval,
}

/// 札の待ち: 局面の出力の契約の部品が `QUEUED_PHASE` なら部品の理由と links.runs、`RUNNING_PHASE` で段が Blocked なら承認待ち、
/// 部品が無いか読めなければ札の理由（読めない間は中核が理由を置かないのでまだ分からない）。
pub fn wait_of(src: &Src<'_>, card: &PipelineCard) -> Wait {
    let part = match src.parts {
        Reading::Known(parts) => contract_part(parts, card.contract.as_str()),
        Reading::Unknown => None,
    };
    match part {
        Some(p) if p.phase == QUEUED_PHASE => Wait::Queue(p.reason.clone(), p.links.runs.clone()),
        Some(p) if p.phase == RUNNING_PHASE && card.stage == Stage::Blocked => Wait::Approval,
        _ => Wait::Queue(card.reason.clone(), Vec::new()),
    }
}

/// 欄の値を替える（鍵の欄が無ければ何もしない）。
fn set_val(facts: &mut [Fact], key: &str, val: Val) {
    if let Some(f) = facts.iter_mut().find(|f| f.key == key) {
        f.val = val;
    }
}

/// 欄を鍵の後ろに差す（鍵の欄が無ければ末に足す）。
fn put_after(facts: &mut Vec<Fact>, after: &str, fact: Fact) {
    let at = facts
        .iter()
        .position(|f| f.key == after)
        .map_or(facts.len(), |i| i + 1);
    facts.insert(at, fact);
}

/// 吹き出しに待ちの欄を足す: Queued は列に入った時刻の後に起きない理由。Blocked は待つ相手の後に待つ理由と、
/// links.runs が在れば相手の便（重なりの相手の run）。承認待ちは待つ相手を持ち主にし、待つ理由を持ち主の承認にする。
/// ほかの段と札の無い bead は替えない。
pub fn with_why(mut p: Pop, src: &Src<'_>) -> Pop {
    let Some(card) = card_of(src.cards, &p.id) else {
        return p;
    };
    let [why, wait_why, wait_runs] = WHY_KEYS;
    match (card.stage, wait_of(src, card)) {
        (Stage::Queued, Wait::Queue(reason, _)) => {
            let val = reason_val(reason.as_deref());
            put_after(&mut p.facts, STAGE_KEYS[2], Fact { key: why, val });
        }
        (Stage::Blocked, Wait::Queue(reason, runs)) => {
            let val = reason_val(reason.as_deref());
            put_after(&mut p.facts, STAGE_KEYS[0], Fact { key: wait_why, val });
            if !runs.is_empty() {
                let val = Val::Code(runs.join(", "));
                put_after(
                    &mut p.facts,
                    wait_why,
                    Fact {
                        key: wait_runs,
                        val,
                    },
                );
            }
        }
        (Stage::Blocked, Wait::Approval) => {
            let [owner, approval] = APPROVAL_KEYS;
            set_val(&mut p.facts, STAGE_KEYS[0], Val::Text(label(owner)));
            let val = Val::Text(label(approval));
            put_after(&mut p.facts, STAGE_KEYS[0], Fact { key: wait_why, val });
        }
        _ => {}
    }
    p
}

#[cfg(target_arch = "wasm32")]
pub use dom::{PopCtx, PopLayer};

#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::html::Div;
    use leptos::prelude::*;
    use tsuzuri_contract::board::Stage;
    use tsuzuri_contract::case::PATH as CASES_PATH;
    use web_sys::wasm_bindgen::JsCast;

    use super::super::hover::{Point, Rect, Size};
    use super::super::modal::{Hit, closes};
    use super::super::runflow::{
        Hist, Seg, flow_text, read_runs, seg_secs, unknown_text, with_runs,
    };
    use super::{
        BEADS_PATH, CLASSES, CLOSE_KEY, ID, Open, PAGE_KEY, Partner, Pop, SCRIM, SUM_CLASS, Src,
        Sum, UNKNOWN_KEY, Val, Via, anchor_selectors, at_text, board_unread, hit_of, item_path,
        known, next_href, opener_selector, place, pop, read_facts, read_parts, sum_text, toggle,
        with_sum, with_why,
    };
    use crate::consultwin::{POP_KEY, consult_href};
    use crate::frame::{Mode, node_href};
    use crate::project::node::NO_SUMMARY;
    use crate::project::{ledger, map, pipeline, timeline, unmeasured};
    use crate::view::read_rows;
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, hs};

    /// 吹き出しの状態（App が context に置く・頁に 1 つ・札と行の口が `press` を呼ぶ）。
    #[derive(Clone, Copy)]
    pub struct PopCtx {
        open: RwSignal<Option<Open>>,
        /// 置いた左上（測って置くまでは None）。
        at: RwSignal<Option<Point>>,
        /// 概要を開いた形で出すか（開く bead が替われば畳んで始める・行 g-pop-sum）。
        whole: RwSignal<bool>,
        node: NodeRef<Div>,
    }

    impl Default for PopCtx {
        fn default() -> Self {
            Self {
                open: RwSignal::new(None),
                at: RwSignal::new(None),
                whole: RwSignal::new(false),
                node: NodeRef::new(),
            }
        }
    }

    fn window_size() -> Size {
        let w = window();
        let px = |v: Result<web_sys::wasm_bindgen::JsValue, _>| {
            v.ok().and_then(|v| v.as_f64()).unwrap_or(0.0)
        };
        Size {
            width: px(w.inner_width()),
            height: px(w.inner_height()),
        }
    }

    fn rect(el: &web_sys::Element) -> Rect {
        let r = el.get_bounding_client_rect();
        Rect {
            left: r.left(),
            top: r.top(),
            width: r.width(),
            height: r.height(),
        }
    }

    impl PopCtx {
        /// 口を押した（同じ口の 2 度目は閉じる）。
        pub fn press(self, id: &str, via: Via) {
            let next = Open {
                id: id.to_string(),
                via,
            };
            let now = self.open.with_untracked(|o| toggle(o.as_ref(), next));
            self.at.set(None);
            self.whole.set(false);
            self.open.set(now);
            request_animation_frame(move || self.settle());
        }

        /// 吹き出しを閉じる。
        pub fn close(self) {
            self.open.set(None);
            self.at.set(None);
        }

        /// 開いている bead の id（追う）。
        pub fn shown(self) -> Option<String> {
            self.open.with(|o| o.as_ref().map(|o| o.id.clone()))
        }

        /// 描いた吹き出しの大きさと口の矩形から置き場を決める。
        fn settle(self) {
            let Some(open) = self.open.get_untracked() else {
                return;
            };
            let Some(el) = self.node.get_untracked() else {
                return;
            };
            let me = rect(&el);
            let anchor = anchor_selectors(&open)
                .iter()
                .find_map(|s| document().query_selector(s).ok().flatten())
                .map(|a| rect(&a));
            let size = Size {
                width: me.width,
                height: me.height,
            };
            self.at.set(Some(place(anchor, size, window_size())));
        }

        /// 置き場の style（置くまでは窓の左上で見えない）。
        fn style(self) -> String {
            let p = self.at.get().unwrap_or(Point { x: 0.0, y: 0.0 });
            let shown = if self.at.with(Option::is_some) {
                ""
            } else {
                ";visibility:hidden"
            };
            format!("left:{}px;top:{}px{shown}", p.x, p.y)
        }

        fn hit(self, h: Hit<'_>) {
            if closes(h) {
                self.close();
            }
        }
    }

    /// 頁の click の押し（吹き出しの中か・口の上か）。
    fn page_hit(e: &ev::MouseEvent) -> Option<Hit<'static>> {
        let el = e.target()?.dyn_into::<web_sys::Element>().ok()?;
        let near = |s: &str| el.closest(s).ok().flatten().is_some();
        hit_of(near(&format!("#{ID}")), near(&opener_selector()))
    }

    fn partner_view(ctx: PopCtx, p: Partner) -> AnyView {
        let sym = p
            .stage
            .map(|s| pipeline::stage_sym(false, pipeline::lane(s.column()).state));
        let stage = p.stage.map(|s| view! { <small>{format!("{s:?}")}</small> });
        let id = p.id.clone();
        view! {
            <button type="button" class=CLASSES[7] on:click=move |_| ctx.press(&id, Via::Card)>
                {sym}<span>{p.short}</span><code>{p.id.clone()}</code>{stage}
            </button>
        }
        .into_any()
    }

    /// 段の流れ（長さの比の帯と字と表に無い段の語・帯の幅は 120 秒を下限にする）。
    fn flow_view(segs: Vec<Seg>, unknown: Vec<String>, now: u64) -> AnyView {
        let bars = segs
            .iter()
            .map(|s| {
                let w = seg_secs(s, now).unwrap_or(0).max(120);
                let class = if s.open { "plive" } else { "" };
                view! { <span class=class style=format!("flex:{w}")></span> }
            })
            .collect_view();
        let words = unknown.iter().map(|w| unknown_text(w)).collect::<Vec<_>>();
        let words =
            (!words.is_empty()).then(|| view! { <div class=CLASSES[8]>{words.join(" · ")}</div> });
        view! {
            <div class="ptl">{bars}</div>
            <div class="ptlt">{flow_text(&segs, now)}</div>
            {words}
        }
        .into_any()
    }

    /// run の歴の表（回・終わりの段・結び・FAIL を含む結びは止まりの色）。
    fn hist_view(rows: Vec<Hist>) -> AnyView {
        let rows = rows
            .into_iter()
            .map(|h| {
                let end = h.end.unwrap_or_else(|| label(UNKNOWN_KEY));
                let fail = h.verdict.as_deref().is_some_and(|v| v.contains("FAIL"));
                let verdict = h.verdict.unwrap_or_else(|| pipeline::NO_AGE.to_string());
                view! {
                    <tr>
                        <td>{h.n}</td>
                        <td>{end}</td>
                        <td class=if fail { "pfail" } else { "" }>{verdict}</td>
                    </tr>
                }
            })
            .collect_view();
        view! {
            <table class="phist">
                <tr><th>{label("ph_run")}</th><th>{label("ph_end")}</th><th>{label("ph_verdict")}</th></tr>
                {rows}
            </table>
        }
        .into_any()
    }

    /// 概要（行 g-pop-sum）: 読めない間はまだ分からない、どれも無ければ要約なし。表示の型の側でない字は頭に語の印を置き、
    /// 畳む字は口を押すと開いてもう 1 度押すと畳む（口の要素は残し語だけ替える）。字の置き場は台帳の字の印を持つ。
    fn sum_view(ctx: PopCtx, sum: Sum) -> AnyView {
        let class = CLASSES[3];
        let p = match sum {
            Sum::Unread => {
                return view! { <div class=class><span class=CLASSES[8]>{label(UNKNOWN_KEY)}</span></div> }
                    .into_any();
            }
            Sum::Empty => return view! { <div class=class>{NO_SUMMARY}</div> }.into_any(),
            Sum::Picked(p) => p,
        };
        let mark = p.marked.then(|| view! { <small>{label(p.key)}</small>" " });
        let folds = sum_text(&p.text, false).1.is_some();
        let text = p.text;
        let shown = move || sum_text(&text, ctx.whole.get());
        let word = shown.clone();
        let button = folds.then(|| {
            view! {
                <button type="button" class=SUM_CLASS on:click=move |_| ctx.whole.update(|w| *w = !*w)>
                    {move || word().1.map(label)}
                </button>
            }
        });
        view! {
            <div class=class>{mark}<span data-ledger-text="">{move || shown().0}</span>{button}</div>
        }
        .into_any()
    }

    fn val_view(ctx: PopCtx, val: Val, now: u64) -> AnyView {
        match val {
            Val::Text(t) => view! { <span>{t}</span> }.into_any(),
            Val::Code(t) => view! { <code>{t}</code> }.into_any(),
            Val::At(at) => view! { <span>{at_text(at, now)}</span> }.into_any(),
            Val::Kind(key, stage) => {
                let stage: Option<Stage> = stage;
                view! { <span>{label(key)}{stage.map(|s| format!(" · {s:?}"))}</span> }.into_any()
            }
            Val::Epic(p) => partner_view(ctx, p),
            Val::Partners(list) => {
                let items = list
                    .into_iter()
                    .map(|p| partner_view(ctx, p))
                    .collect_view();
                view! { <div class=CLASSES[6]>{items}</div> }.into_any()
            }
            Val::Flow(segs, unknown) => flow_view(segs, unknown, now),
            Val::Hist(rows) => hist_view(rows),
            Val::Unknown => view! { <span class=CLASSES[8]>{label(UNKNOWN_KEY)}</span> }.into_any(),
        }
    }

    fn pop_view(ctx: PopCtx, p: Pop, mode: Mode, now: u64) -> AnyView {
        let [head, main, title, _, table, foot, _, link, _] = CLASSES;
        let sym = p
            .stage
            .map(|s| pipeline::stage_sym(false, pipeline::lane(s.column()).state));
        let rows = p
            .facts
            .into_iter()
            .map(|f| view! { <tr><th>{hs(f.key)}</th><td>{val_view(ctx, f.val, now)}</td></tr> })
            .collect_view();
        let next = p.next.map(|key| {
            view! { <a class=link href=next_href(key, &p.id, mode)>{format!("{} ›", label(key))}</a> }
        });
        let unknown = || label(UNKNOWN_KEY);
        view! {
            <div class=head>
                {sym}<b>{p.short}</b>
                <button type="button" class="x" aria-label=label(CLOSE_KEY) on:click=move |_| ctx.hit(Hit::X)>"×"</button>
            </div>
            <div class=main>
                <div class=title>{p.title.unwrap_or_else(unknown)}</div>
                {sum_view(ctx, p.summary)}
                <table class=table>{rows}</table>
                {next}
            </div>
            <div class=foot>
                <a class=link href=node_href(&p.id, mode)>{format!("{} ›", label(PAGE_KEY))}</a>
                <a class=link href=consult_href(&p.id, mode)>{format!("{} ›", label(POP_KEY))}</a>
            </div>
        }
        .into_any()
    }

    /// 頁の取り消しの鍵と click を聞き、吹き出しが開いていれば窓の枠の閉じる判定に渡す（層の一生の間）。
    fn listen(ctx: PopCtx) {
        let keys = window_event_listener(ev::keydown, move |e| {
            if ctx.open.with_untracked(Option::is_some) {
                let key = e.key();
                ctx.hit(Hit::Key {
                    key: &key,
                    composing: e.is_composing(),
                });
            }
        });
        let clicks = window_event_listener(ev::click, move |e| {
            if ctx.open.with_untracked(Option::is_some)
                && let Some(h) = page_hit(&e)
            {
                ctx.hit(h);
            }
        });
        on_cleanup(move || {
            keys.remove();
            clicks.remove();
        });
    }

    /// 吹き出しの層（body に 1 つ・開いた bead の吹き出しを描く・× と外の click と取り消しの鍵で閉じる）。
    #[component]
    pub fn PopLayer() -> impl IntoView {
        let Some(ctx) = use_context::<PopCtx>() else {
            return ().into_any();
        };
        let help = use_context::<HelpCtx>();
        let beads = crate::net::read(BEADS_PATH);
        let cases = crate::net::read(CASES_PATH);
        let pipe = crate::net::read(pipeline::PATH);
        let rows = crate::net::read(ledger::PATH);
        let graph = crate::net::read(map::PATH);
        // 走行の読みの口は吹き出しを開いた時に読む（閉じていれば空の path で読まない・行 g-pop-flow）。
        let runs_path = Signal::derive(move || {
            ctx.shown()
                .map(|id| timeline::path(&id))
                .unwrap_or_default()
        });
        let runs = crate::net::read_path(runs_path);
        // 1 本の引きの口も吹き出しを開いた時に開いた bead の path で読む（閉じていれば空の path で読まない・行 g-pop-sum）。
        let item_at =
            Signal::derive(move || ctx.shown().map(|id| item_path(&id)).unwrap_or_default());
        let item = crate::net::read_path(item_at);
        listen(ctx);
        let style = move || ctx.style();
        let content = move || {
            let id = ctx.shown()?;
            let mode = help.map_or(Mode::Beginner, |h| h.mode.get());
            let facts = beads.with(read_facts);
            let parts = cases.with(read_parts);
            // 板の札が読めない間は、段ごとの欄を出せない理由を吹き出しの末に出す（札が無いとは見せない）。
            let unread = pipe.with(board_unread);
            let cards = pipe.with(|p| pipeline::cards(p).unwrap_or_default());
            let rows = rows.with(read_rows);
            let doc = graph.with(|g| map::doc(g).ok());
            let src = Src {
                facts: known(&facts),
                rows: known(&rows),
                cards: &cards,
                parts: known(&parts),
                graph: doc.as_ref(),
            };
            let lines = runs.with(|(f, _)| read_runs(f));
            let p = with_runs(pop(&id, &src), known(&lines));
            let p = with_why(p, &src);
            let p = item.with(|(f, _)| with_sum(p, f, mode));
            Some(
                view! { {pop_view(ctx, p, mode, crate::net::now())}{unread.map(unmeasured)} }
                    .into_any(),
            )
        };
        let open = move || ctx.open.with(Option::is_some);
        view! {
            <Show when=open>
                <div id=SCRIM></div>
                <div id=ID node_ref=ctx.node role="dialog" style=style>{content}</div>
            </Show>
        }
        .into_any()
    }
}
