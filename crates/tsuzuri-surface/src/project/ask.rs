//! block「答えを待つ質問」（問いの頁・便 g-ask）: 口 /api/questions の card を電文の順（古い順）に番号つきで出し、
//! card ごとに答えの欄から口 /api/ruling へ答えを送る。見本は docs/design/mock3/ask.html の qcard。
//! 問いの見分けは server の側（label intake:question）が済ませていて、面は bd の種類で選ばず電文を写すだけ。
//! card の部分の並び（配置の表）・card の中身・送る button の状態・鍵の判定・要求の本文・応答から card の状態を決める関数は
//! 純粋な関数にして host で試し、DOM と通信は wasm の target のときだけ組み立てる。
//! 持ち主の字は送る要求の本文の外に書かない（URL にも、画面の外の保存の口にも残さない）。
//! card の題は節点の頁への link で、URL の `?id=` で名指された card は class target を足して画面の上端へ寄せる（便 g-ask-focus）。
//! 題の link にはグラフの口の電文から引いた問いの節点の hover の card を付ける（電文に無い問いは付けない・行 g-card-adopt-b）。
//! 経過の chip は 1 秒の時計（net の ticker）で書き直し、経験者の mode には投稿の時刻の注釈を付ける（行 g-tick-adopt）。
//! 送っている間は送る button の字を替え、server の台帳の断りは理由と次の手の字にして目立つ 1 行で出す（行 g-ruling-busy）。
//! 電文の answerable が偽（読むだけの server）なら、送る欄の代わりにチャットで答える 1 行を出す（行 e-ask-own-only）。
//! 電文の鍵 others のほかの project の問いは札つきで投稿の時刻の順に 1 つの一覧へ混ぜ、題は link にせず、つながりの段を
//! 出さず、送る欄の代わりにチャットで答える 1 行を出す。台帳が読めない組は札と 1 行を一覧の下に出す（行 e-multi-ask）。
//! 束の block と次の一手と問いの件数（`cards`・`count`・`answerable`）は自分の問いだけを読む。

use std::collections::BTreeMap;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::{AllQuestions, ProjectQuestions, QuestionCard, QuestionList};
use tsuzuri_contract::surface::{Refusal, RefusalResponse, RulingId, RulingRequest, RulingResponse};
use tsuzuri_contract::wire;

use super::{Body, NOT_READ};
use crate::frame::Block;
use crate::view::{Fetched, clock};
use crate::widgets::hover::{self, clip};
use crate::widgets::nodecard::card_of;

pub const BLOCK: Block = Block {
    id: "ask",
    heading: "ask_open",
    class: "panel",
};

/// 問いの一覧の口（契約の型の QuestionList）。
pub const PATH: &str = "/api/questions";

/// 答えを送る口（POST・契約の型の RulingRequest・server の便 e-ask）。
pub const RULING_PATH: &str = "/api/ruling";

/// 席に届いていない裁定の口（契約の型の board の Reading と surface の RulingId・server の行 f-undelivered）。
pub const UNRECEIVED_PATH: &str = "/api/unreceived";

/// この file が字を持つ口の path（ほかの module の口を読む所は数えない・行 hs-derived）。
pub const PATHS: &[&str] = &[PATH, RULING_PATH, UNRECEIVED_PATH];

/// 裁定の id の分の終わりから、席の受けを待つ秒（これより早く届いていないと出さない・要件 FR9）。
pub const RECEIPT_WAIT_S: EpochSecs = 120;

/// 届いていない裁定の 1 行の頭。
pub const UNRECEIVED_HEAD: &str = "席に届いていない";

/// 届いていない裁定の口が読めないときの 1 行（前の版の server は口を持たない）。
pub const UNRECEIVED_UNKNOWN: &str = "席に届いたかが読めない（届いていない裁定の口が読めない）";

/// この file の畳める段の開き閉じの鍵の形（`{}` は問いの id・行 hs-derived）。
pub const FOLDS: &[&str] = &["ask:around:{}"];

/// 口が読めないときの理由。
pub const REASON: &str =
    "問いの一覧の口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 本文が電文として読めないときの理由。
pub const UNREADABLE: &str = "問いの一覧の本文が電文として読めない";

/// server が台帳を読めず card が「まだ分からない」ときの理由。
pub const CARDS_UNKNOWN: &str = "server が台帳を読めず、答えを待つ質問が分からない";

/// ほかの project の台帳が読めないときに札の後に出す 1 行（行 e-multi-ask）。
pub const OTHER_UNKNOWN: &str = "台帳が読めず、答えを待つ質問が分からない";

/// 測れて 0 件のときの 1 行。
pub const EMPTY: &str = "答えを待つ question は無い";

/// 概要が両方無いときの 1 行。
pub const NO_SUMMARY: &str = "要約なし";

/// 概要の片方が無いときの字。
pub const MISSING: &str = "―";

/// 200 の応答の後に card に出す字（記録した id と時刻の前）。
pub const RECORDED: &str = "記録した";

/// 409 の応答の後に card に出す字（一覧は読み直す）。
pub const STALE: &str = "質問が更新された";

/// 口に届かないときの理由の字。
pub const NOT_REACHED: &str = "口に届かない";

/// 200 の応答の本文が電文として読めないときの理由の字。
pub const BAD_REPLY: &str = "応答が電文として読めない";

/// 断られたときの字（理由の前）。
pub const REFUSED: &str = "送れなかった";

/// 送っている間の送る button の字（行 g-ruling-busy）。
pub const SENDING: &str = "送っています…";

/// 503 の ledger-unknown のときの理由と次の手の字。
pub const LEDGER_BUSY: &str =
    "台帳が混んでいて読めなかった（503）・何も書いていない・少し待ってもう一度押す";

/// 502 の ledger-append のときの理由と次の手の字。
pub const APPEND_FAILED: &str = "台帳に書けなかった（502）・何も書いていない・もう一度押す";

/// 502 の ledger-close のときの理由と次の手の字（後に裁定の id を付ける）。
pub const CLOSE_FAILED: &str =
    "記録したが問いを閉じられなかった（502）・もう一度押さず席に知らせる";

/// 読むだけの server の問いの card と束の block に、送る欄の代わりに出す 1 行の語の鍵（行 e-ask-own-only）。
pub const CHAT_KEY: &str = "answer_in_chat";

/// card の部分（見本の qcard の順: 題・概要・理由・推奨・答えの欄・つながり）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// 頭: 番号・題・A-1 の印・置かれてからの経過。
    Head,
    Summary,
    Reason,
    Recommend,
    Answer,
    Around,
}

/// 配置の表の 1 行（部分・class・語の鍵・畳める段なら最初に開いているか）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub part: Part,
    pub class: &'static str,
    pub key: Option<&'static str>,
    pub open: Option<bool>,
}

/// card の部分の並び（DOM はこの表を順にたどって組み立てる）。
pub const LAYOUT: [Slot; 6] = [
    Slot {
        part: Part::Head,
        class: "head",
        key: None,
        open: None,
    },
    Slot {
        part: Part::Summary,
        class: "qsum",
        key: None,
        open: None,
    },
    Slot {
        part: Part::Reason,
        class: "qreason",
        key: Some("reason"),
        open: None,
    },
    Slot {
        part: Part::Recommend,
        class: "rec",
        key: Some("recommend"),
        open: None,
    },
    Slot {
        part: Part::Answer,
        class: "answer",
        key: Some("own_words"),
        open: None,
    },
    Slot {
        part: Part::Around,
        class: "nb-d",
        key: Some("around"),
        open: Some(false),
    },
];

/// 概要の 1 行（class・語の鍵・字・エンジニア向けか）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SumLine {
    pub class: &'static str,
    pub key: Option<&'static str>,
    pub text: String,
    pub eng: bool,
}

/// 1 本の card の中身（番号は 1 から・題は 36 字で切る）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Card {
    pub number: usize,
    pub id: BeadId,
    pub title: String,
    pub a1: bool,
    pub posted_at: EpochSecs,
    pub summary: Vec<SumLine>,
    pub reason: String,
    pub recommend: String,
    pub touches: Vec<String>,
    /// 答えを待って止まった task の id（chip は数だけを出す）。
    pub blocking: Vec<String>,
    pub digest: String,
    /// ほかの project の札（自分の問いは None・行 e-multi-ask）。
    pub project: Option<String>,
    /// この card に答えを送れるか（自分の問いは真・ほかの project の問いは電文のその組の answerable）。
    pub answerable: bool,
}

/// 口の本文を card の列に読む（まだ読んでいない・読めない・電文が読めない・まだ分からないは理由）。
pub fn cards(fetched: &Fetched) -> Result<Vec<QuestionCard>, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(REASON),
        Fetched::Body(text) => match wire::decode::<QuestionList>(text) {
            Ok(QuestionList {
                cards: Reading::Known(cards),
                ..
            }) => Ok(cards),
            Ok(QuestionList {
                cards: Reading::Unknown,
                ..
            }) => Err(CARDS_UNKNOWN),
            Err(_) => Err(UNREADABLE),
        },
    }
}

/// server が答えを受けるか（本文を電文に読めればその欄 answerable・読めない本文とまだ読んでいないと読めないは真・
/// 行 e-ask-own-only）。
pub fn answerable(fetched: &Fetched) -> bool {
    match fetched {
        Fetched::Body(text) => wire::decode::<QuestionList>(text).map_or(true, |l| l.answerable),
        Fetched::NotRead | Fetched::Failed => true,
    }
}

/// 問いの件数（測れていなければ Unknown・0 件と書かない）。
pub fn count(fetched: &Fetched) -> Reading<usize> {
    match cards(fetched) {
        Ok(cards) => Reading::Known(cards.len()),
        Err(_) => Reading::Unknown,
    }
}

/// 本文を全部の問いの一覧に読んだ欄 others（読めない本文・まだ読んでいない・読めないは空の列・行 e-multi-ask）。
pub fn others(fetched: &Fetched) -> Vec<ProjectQuestions> {
    match fetched {
        Fetched::Body(text) => wire::decode::<AllQuestions>(text).map_or_else(|_| Vec::new(), |a| a.others),
        Fetched::NotRead | Fetched::Failed => Vec::new(),
    }
}

/// card が Unknown のほかの project の札（電文の順）。
pub fn unknown_projects(fetched: &Fetched) -> Vec<String> {
    others(fetched)
        .into_iter()
        .filter(|p| p.cards == Reading::Unknown)
        .map(|p| p.project)
        .collect()
}

/// block の中身の card の数（ほかの project の問いを含む・測れていなければ Unknown・0 件は Known の 0）。
pub fn total(fetched: &Fetched) -> Reading<usize> {
    match body(fetched) {
        Body::Unmeasured(_) => Reading::Unknown,
        Body::Empty(_) => Reading::Known(0),
        Body::Filled(cards) => Reading::Known(cards.len()),
    }
}

/// block の中身（自分の問いの列に、ほかの project の読めた組の列を電文の順に 1 つずつ投稿の時刻で混ぜ、番号を 1 から付ける）。
pub fn body(fetched: &Fetched) -> Body<Vec<Card>> {
    let own = match cards(fetched) {
        Err(reason) => return Body::Unmeasured(reason),
        Ok(cards) => cards.iter().map(|q| card(0, q)).collect(),
    };
    let all = others(fetched)
        .into_iter()
        .fold(own, |all, p| match p.cards {
            Reading::Known(cards) => {
                let tagged = cards
                    .iter()
                    .map(|q| Card {
                        project: Some(p.project.clone()),
                        answerable: p.answerable,
                        ..card(0, q)
                    })
                    .collect();
                merged(all, tagged)
            }
            Reading::Unknown => all,
        });
    if all.is_empty() {
        return Body::Empty(EMPTY);
    }
    Body::Filled(
        all.into_iter()
            .enumerate()
            .map(|(i, c)| Card { number: i + 1, ..c })
            .collect(),
    )
}

/// 2 つの列をどちらも順を変えずに 1 つにする（先頭どうしの投稿の時刻を比べ、後の列の先頭が古いときだけ後の列から取る）。
fn merged(front: Vec<Card>, back: Vec<Card>) -> Vec<Card> {
    let mut out = Vec::with_capacity(front.len() + back.len());
    let (mut front, mut back) = (front.into_iter().peekable(), back.into_iter().peekable());
    loop {
        let from_back = match (front.peek(), back.peek()) {
            (Some(f), Some(b)) => b.posted_at < f.posted_at,
            (None, Some(_)) => true,
            (Some(_), None) => false,
            (None, None) => return out,
        };
        out.extend(if from_back { back.next() } else { front.next() });
    }
}

/// 鍵つきの一覧の鍵（番号を 0 にした中身・前の card が答えられて番号が詰まっても変わらない）。
pub fn card_key(card: &Card) -> Card {
    Card {
        number: 0,
        ..card.clone()
    }
}

/// block の形（Filled の中身を捨てた値・形が同じなら外枠を組み直さない）。
pub fn outline(fetched: &Fetched) -> Body<()> {
    match body(fetched) {
        Body::Unmeasured(reason) => Body::Unmeasured(reason),
        Body::Empty(line) => Body::Empty(line),
        Body::Filled(_) => Body::Filled(()),
    }
}

/// card の列（Filled でなければ空の列）。
pub fn listed(fetched: &Fetched) -> Vec<Card> {
    match body(fetched) {
        Body::Filled(cards) => cards,
        _ => Vec::new(),
    }
}

/// 問いの id ごとの節点の hover の card（グラフの口が読めなければ空・電文に無い問いは持たない）。
pub fn node_cards(graph: &Fetched, cards: &[Card]) -> BTreeMap<String, hover::Card> {
    let Ok(doc) = super::map::doc(graph) else {
        return BTreeMap::new();
    };
    cards
        .iter()
        .filter_map(|c| card_of(&doc, c.id.as_str()).map(|n| (c.id.to_string(), n)))
        .collect()
}

/// 電文の 1 本を自分の問いの card の中身にする（札なし・答えを送れる）。
pub fn card(number: usize, q: &QuestionCard) -> Card {
    Card {
        number,
        id: q.id.clone(),
        title: clip(&q.title),
        a1: q.a1,
        posted_at: q.posted_at,
        summary: summary(q.plain.as_deref(), q.eng.as_deref()),
        reason: q.reason.clone().unwrap_or_default(),
        recommend: q.recommend.clone().unwrap_or_default(),
        touches: q.touches.clone(),
        blocking: q.blocking.clone(),
        digest: q.digest.clone(),
        project: None,
        answerable: true,
    }
}

/// 概要の行（2 行・片方だけ無ければ「―」・両方無ければ「要約なし」の 1 行）。
pub fn summary(plain: Option<&str>, eng: Option<&str>) -> Vec<SumLine> {
    if plain.is_none() && eng.is_none() {
        return vec![SumLine {
            class: "ln plain muted",
            key: None,
            text: NO_SUMMARY.to_string(),
            eng: false,
        }];
    }
    vec![
        SumLine {
            class: "ln plain",
            key: Some("summary_plain"),
            text: plain.unwrap_or(MISSING).to_string(),
            eng: false,
        },
        SumLine {
            class: "ln eng",
            key: Some("summary_eng"),
            text: eng.unwrap_or(MISSING).to_string(),
            eng: true,
        },
    ]
}

/// 置かれてからの経過の字（60 分未満は分・48 時間未満は時間と分・それ以上は日・見本の durMs）。
pub fn age(now: EpochSecs, posted_at: EpochSecs) -> String {
    let minutes = now.saturating_sub(posted_at) / 60;
    if minutes < 60 {
        return format!("{minutes}m");
    }
    let hours = minutes / 60;
    if hours < 48 {
        return match minutes % 60 {
            0 => format!("{hours}h"),
            m => format!("{hours}h{m:02}"),
        };
    }
    format!("{}d", hours / 24)
}

/// 経過の chip の経験者だけの注釈の字（`posted_at` と日本時間の投稿の時刻・見本の chip の `data-tip-expert`・行 g-tick-adopt）。
pub fn posted_tip(posted_at: EpochSecs) -> String {
    format!("posted_at {}", clock(posted_at))
}

/// 送る button を押せるか（答えの欄が空白だけのときと送っている間は押せない）。
pub fn can_send(text: &str, sending: bool) -> bool {
    !sending && !text.trim().is_empty()
}

/// 送る button の字（送っている間は SENDING・ほかは決定の語に空白と › を足した字）。
pub fn send_text(sending: bool, ruling: &str) -> String {
    if sending {
        SENDING.to_string()
    } else {
        format!("{ruling} ›")
    }
}

/// 答えの欄の鍵の判定の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// 何もしない（字の変換の途中）。
    Nothing,
    /// 送る（Ctrl か Meta と Enter）。
    Send,
    /// 送らない（字の欄のふつうの動き）。
    Hold,
}

/// 鍵の判定（変換の途中か・Ctrl か Meta が押されているか・鍵の名）。
pub fn key_action(composing: bool, ctrl_or_meta: bool, key: &str) -> KeyAction {
    if composing {
        KeyAction::Nothing
    } else if ctrl_or_meta && key == "Enter" {
        KeyAction::Send
    } else {
        KeyAction::Hold
    }
}

/// 要求の本文（見た版の要約値は card の digest の字のまま・逐語は答えの欄の字のまま・電文の字にできなければ空の本文で、server が断る）。
pub fn request_body(card: &Card, verbatim: &str) -> String {
    wire::encode(&RulingRequest {
        question: card.id.clone(),
        seen_digest: card.digest.clone(),
        verbatim: verbatim.to_string(),
    })
    .unwrap_or_default()
}

/// 送った後の card の状態。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 200: 記録した id と時刻。
    Recorded { ruling: RulingId, at: EpochSecs },
    /// 409: 質問が更新された（一覧を読み直す）。
    Stale,
    /// ほかの 4xx・5xx・届かない: 理由の字。
    Refused(String),
}

impl Outcome {
    /// card に出す 1 行。
    pub fn line(&self) -> String {
        match self {
            Outcome::Recorded { ruling, at } => format!("{RECORDED} {ruling} · {}", clock(*at)),
            Outcome::Stale => STALE.to_string(),
            Outcome::Refused(reason) => format!("{REFUSED}: {reason}"),
        }
    }

    /// 答えの欄の字を残すか（200 の外は残す）。
    pub fn keeps_text(&self) -> bool {
        !matches!(self, Outcome::Recorded { .. })
    }

    /// 答えの欄を開いたままにするか（200 は閉じる）。
    pub fn answer_open(&self) -> bool {
        self.keeps_text()
    }

    /// 問いの一覧を読み直すか（200 と 409）。
    pub fn reloads(&self) -> bool {
        matches!(self, Outcome::Recorded { .. } | Outcome::Stale)
    }

    /// 1 行の class と role（断りは warn と alert・ほかは small と status）。
    pub fn note(&self) -> (&'static str, &'static str) {
        match self {
            Outcome::Refused(_) => ("warn", "alert"),
            _ => ("small", "status"),
        }
    }
}

/// server の台帳の断りの理由と次の手の字（本文の前後の空白を除いて読む・ほかは None）。
pub fn server_reason(status: u16, text: &str) -> Option<String> {
    match (status, text.trim()) {
        (503, "ledger-unknown") => Some(LEDGER_BUSY.to_string()),
        (502, "ledger-append") => Some(APPEND_FAILED.to_string()),
        (502, body) => body
            .strip_prefix("ledger-close ")
            .map(|id| format!("{CLOSE_FAILED}（{id}）")),
        _ => None,
    }
}

/// 断りの理由の字（閉じた 6 値）。
pub fn refusal_text(reason: Refusal) -> &'static str {
    match reason {
        Refusal::EmptyVerbatim => "言葉が空",
        Refusal::UnknownQuestion => "台帳に無い質問",
        Refusal::StaleVersion => STALE,
        Refusal::A1NeedsOwnVerbatim => "A-1 の質問には個別の言葉が要る",
        Refusal::A1InBatch => "A-1 の質問は束に入れない",
        Refusal::EmptyBatch => "束が空",
    }
}

/// 応答から card の状態を決める（`reply` は状態の数と本文の字・届かなければ None）。
pub fn outcome(reply: Option<(u16, &str)>) -> Outcome {
    match reply {
        None => Outcome::Refused(NOT_REACHED.to_string()),
        Some((200, text)) => match wire::decode::<RulingResponse>(text) {
            Ok(r) => Outcome::Recorded {
                ruling: r.ruling,
                at: r.recorded_at,
            },
            Err(_) => Outcome::Refused(format!("{BAD_REPLY}（200）")),
        },
        Some((409, _)) => Outcome::Stale,
        Some((status, text)) => Outcome::Refused(match wire::decode::<RefusalResponse>(text) {
            Ok(r) => format!("{}（{status}）", refusal_text(r.reason)),
            Err(_) => server_reason(status, text).unwrap_or_else(|| format!("状態 {status}")),
        }),
    }
}

/// URL の query で問いを名指す鍵（`?id=`・見本の ask.html の終わりの script）。
pub const FOCUS_KEY: &str = "id";

/// URL の query で名指された問いの id（`%XX` を戻した字・無いか空なら None）。
pub fn focus(query: &str) -> Option<String> {
    crate::frame::param(query, FOCUS_KEY)
        .map(crate::mapview::decode)
        .filter(|id| !id.is_empty())
}

/// card の要素の id（字 q と 0 から数えた位置・番号 1 は q0＝見本の qcard の id）。
pub fn anchor(number: usize) -> String {
    format!("q{}", number.saturating_sub(1))
}

/// card の要素の class（名指された card は target を足す・見本の `.qcard.target`）。
pub fn card_class(target: bool) -> &'static str {
    if target { "qcard target" } else { "qcard" }
}

/// 名指された id の card の番号（一覧に無ければ None＝答え済み）。
pub fn target_number(cards: &[Card], id: &str) -> Option<usize> {
    cards.iter().find(|c| c.id.as_str() == id).map(|c| c.number)
}

/// 裁定の id の分の終わりの epoch 秒（最後のコロンの後の `YYYYMMDDTHHMMZ-<数>` の分の始まりに 60 秒を足す・
/// 読めない id と 1970 年より前の分は None）。
pub fn minute_end(id: &str) -> Option<EpochSecs> {
    let (_, tail) = id.rsplit_once(':')?;
    let bytes = tail.as_bytes();
    let stamp: &[u8; 14] = bytes.get(..14)?.try_into().ok()?;
    let count = bytes.get(14..)?.strip_prefix(b"-")?;
    if count.is_empty() || !count.iter().all(u8::is_ascii_digit) || stamp[8] != b'T' || stamp[13] != b'Z' {
        return None;
    }
    let digits = |s: &[u8]| -> Option<u64> {
        s.iter()
            .all(u8::is_ascii_digit)
            .then(|| s.iter().fold(0, |n, d| n * 10 + u64::from(d - b'0')))
    };
    let (year, month, day) = (digits(&stamp[..4])?, digits(&stamp[4..6])?, digits(&stamp[6..8])?);
    let (hour, minute) = (digits(&stamp[9..11])?, digits(&stamp[11..13])?);
    if year < 1970 || !(1..=12).contains(&month) || hour > 23 || minute > 59 {
        return None;
    }
    let days_in = |y: u64, m: u64| match m {
        2 if y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400)) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if !(1..=days_in(year, month)).contains(&day) {
        return None;
    }
    // 1970-01-01 からの日数（年と月の頭までの日数に日を足す）。
    let years: u64 = (1970..year).map(|y| (1..=12).map(|m| days_in(y, m)).sum::<u64>()).sum();
    let months: u64 = (1..month).map(|m| days_in(year, m)).sum();
    Some((years + months + day - 1) * 86_400 + hour * 3_600 + minute * 60 + 60)
}

/// 分の終わりから `RECEIPT_WAIT_S` 秒経った裁定の id と、分の読めない id（待たずに出す・条 P-7）を元の順に返す。
pub fn late(ids: &[RulingId], now: EpochSecs) -> Vec<RulingId> {
    ids.iter()
        .filter(|id| minute_end(id.as_str()).is_none_or(|end| now >= end + RECEIPT_WAIT_S))
        .cloned()
        .collect()
}

/// 口の本文を電文に読む（まだ読んでいなければ None・口が読めない・電文が読めないは Unknown）。
pub fn unreceived(fetched: &Fetched) -> Option<Reading<Vec<RulingId>>> {
    match fetched {
        Fetched::NotRead => None,
        Fetched::Failed => Some(Reading::Unknown),
        Fetched::Body(text) => Some(wire::decode(text).unwrap_or(Reading::Unknown)),
    }
}

/// 届いていない裁定の 1 行（まだ読んでいない・届いていないものが無い・どの裁定もまだ待つ間なら None・
/// 読めなければ `UNRECEIVED_UNKNOWN`・逐語は出さず id だけを並べる）。
pub fn unreceived_line(fetched: &Fetched, now: EpochSecs) -> Option<String> {
    let ids = match unreceived(fetched)? {
        Reading::Unknown => return Some(UNRECEIVED_UNKNOWN.to_string()),
        Reading::Known(ids) => late(&ids, now),
    };
    if ids.is_empty() {
        return None;
    }
    let listed: Vec<&str> = ids.iter().map(RulingId::as_str).collect();
    Some(format!("{UNRECEIVED_HEAD} {} 件: {}", ids.len(), listed.join("・")))
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 質問の窓の 1 問（行 g-ask-win・中身は dom の one_view）。
#[cfg(target_arch = "wasm32")]
pub use dom::one_view;

/// 届いていない裁定の 1 行（一覧の上と次の一手の箱の上に出す・wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
pub fn late_view() -> leptos::prelude::AnyView {
    dom::late_view()
}

/// 問いの card の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;
    use leptos::task::spawn_local;
    use tsuzuri_contract::board::Reading;
    use tsuzuri_contract::ledger::BeadId;

    use super::{
        BLOCK, CHAT_KEY, Card, KeyAction, LAYOUT, OTHER_UNKNOWN, Outcome, PATH, Part, RULING_PATH,
        Slot, UNRECEIVED_PATH, age, anchor, answerable, can_send, card_class, card_key, focus, key_action, listed,
        node_cards, outcome, outline, posted_tip, request_body, send_text, target_number, total,
        unknown_projects,
    };
    use crate::frame::{Mode, node_href};
    use crate::project::map;
    use crate::project::nodearound::{Embeds, embeds};
    use crate::project::{Body, body_view, fold, section, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, expert_tip};
    use crate::widgets::hover::{self, attach_some};

    /// 見本の IC.warn・IC.clock・IC.person・IC.code・IC.check・IC.link・IC.stop。
    const WARN: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M12 3l10 18H2z"/><path d="M12 10v5"/><circle cx="12" cy="18" r=".8" fill="currentColor"/></svg>"#;
    const CLOCK: &str = r#"<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg>"#;
    const PERSON: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="8" r="4"/><path d="M4 21c1-4 4-6 8-6s7 2 8 6"/></svg>"#;
    const CODE: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M8 7l-5 5 5 5M16 7l5 5-5 5M14 4l-4 16"/></svg>"#;
    const CHECK: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" aria-hidden="true"><path d="M5 12l5 5 9-10"/></svg>"#;
    const LINK: &str = r#"<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1"/><path d="M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"/></svg>"#;
    const STOP: &str = r#"<svg width="12" height="12" viewBox="0 0 24 24" aria-hidden="true"><rect x="6" y="6" width="12" height="12" rx="1" fill="currentColor"/></svg>"#;

    /// 1 本の card の答えの状態（一覧を読み直しても残すので、問いの id ごとに block が持つ）。
    #[derive(Clone)]
    struct Draft {
        text: ArcRwSignal<String>,
        sending: ArcRwSignal<bool>,
        outcome: ArcRwSignal<Option<Outcome>>,
    }

    type Drafts = StoredValue<Vec<(BeadId, Draft)>>;

    /// server が答えを受けるか（block が context で置き、答えの欄が読む・card_view の引数は替えない）。
    #[derive(Clone, Copy)]
    struct CanAnswer(Memo<bool>);

    /// 1 本の card の、一覧を読み直すと動く値（今の番号と題の節点の card）。
    #[derive(Clone, Copy)]
    struct Live {
        nb: Memo<usize>,
        node: Memo<Option<hover::Card>>,
    }

    /// 問いの id の答えの状態（初めての id は空で作る）。
    fn draft(drafts: Drafts, id: &BeadId) -> Draft {
        let found = drafts.with_value(|v| v.iter().find(|(k, _)| k == id).map(|(_, d)| d.clone()));
        found.unwrap_or_else(|| {
            let d = Draft {
                text: ArcRwSignal::new(String::new()),
                sending: ArcRwSignal::new(false),
                outcome: ArcRwSignal::new(None),
            };
            drafts.update_value(|v| v.push((id.clone(), d.clone())));
            d
        })
    }

    /// 名指された card を画面の上端へ寄せる（描いた後の frame で・要素が無ければ何もしない）。
    fn scroll_to(number: usize) {
        let id = anchor(number);
        request_animation_frame(move || {
            if let Some(el) = document().get_element_by_id(&id) {
                el.scroll_into_view_with_bool(true);
            }
        });
    }

    pub fn view() -> AnyView {
        let search = window().location().search().unwrap_or_default();
        let target = focus(&search);
        let fallback = Mode::from_query(&search);
        let mode = use_context::<HelpCtx>().map(|c| c.mode);
        let scrolled = StoredValue::new(false);
        let fetched = crate::net::read(PATH);
        let drafts: Drafts = StoredValue::new(Vec::new());
        // 読むだけの server なら答えの欄は送る欄の代わりにチャットで答える 1 行を出す。
        provide_context(CanAnswer(Memo::new(move |_| fetched.with(answerable))));
        // つながりの段の図の読みはこの block が持つ（一覧の読み直しで card を組み直しても作り直さない）。
        let places = embeds();
        // 形と card の列は値が前と同じなら知らせない（本文が替わっても形が同じなら外枠を組み直さない）。
        let shape = Memo::new(move |_| fetched.with(outline));
        let cards = Memo::new(move |_| fetched.with(listed));
        // 題の節点の card はグラフの口から引く（問いの一覧より後に読めても、後から card が付く）。
        let graph = crate::net::read(map::PATH);
        let nodes = Memo::new(move |_| cards.with(|v| graph.with(|g| node_cards(g, v))));
        // 経過は 1 秒の時計で書き直し、link は mode を替えれば替わる。
        let clock = crate::net::ticker();
        let tick = move || clock.get();
        let current = move || mode.map_or(fallback, |m| m.get());
        let extra = move || match fetched.with(total) {
            Reading::Known(n) => view! { <span class="chip num">{n}</span> }.into_any(),
            Reading::Unknown => ().into_any(),
        };
        // 名指しの card を 1 度だけ画面の上端へ寄せる。
        let focused = target.clone();
        Effect::new(move |_| {
            if let Some(id) = focused.as_deref()
                && !scrolled.get_value()
                && let Some(n) = cards.with(|v| target_number(v, id))
            {
                scrolled.set_value(true);
                scroll_to(n);
            }
        });
        // 台帳が読めないほかの project は、札と 1 行を一覧の下に出す。
        let unknown = move || {
            fetched
                .with(unknown_projects)
                .into_iter()
                .map(|p| view! { <div class="small muted"><span class="chip">{p}</span>" "{OTHER_UNKNOWN}</div> })
                .collect_view()
        };
        let list = move || match shape.get() {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Filled(()) => {
                let target = target.clone();
                // 鍵は番号を除いた中身（変わらない card の DOM は残り、欄の focus と変換の途中の字も残る）。
                view! {
                    <For each=move || cards.get() key=card_key children=move |c: Card| {
                        let d = draft(drafts, &c.id);
                        let on = target.as_deref() == Some(c.id.as_str());
                        let id = c.id.to_string();
                        let key = id.clone();
                        let nb = Memo::new(move |_| cards.with(|v| target_number(v, &id)).unwrap_or(0));
                        let node = Memo::new(move |_| nodes.with(|m| m.get(&key).cloned()));
                        card_view(c, d, Live { nb, node }, on, Shared { tick, mode: current, places })
                    }/>
                }
                .into_any()
            }
        };
        let content = view! { {late_view()}{list}{unknown} }.into_any();
        section(BLOCK, extra.into_any(), content)
    }

    /// 質問の窓の 1 問（行 g-ask-win）: `pick` の card を block と同じ card（つながりの図と答えの欄も同じ）で描き、
    /// 答えを記録した（200）問いの id を `recorded` に渡す。答えの状態は問いの id ごとに窓の一生の間持つ。
    pub fn one_view(pick: Memo<Option<Card>>, recorded: Callback<BeadId>) -> AnyView {
        let fallback = Mode::from_query(&window().location().search().unwrap_or_default());
        let mode = use_context::<HelpCtx>().map(|c| c.mode);
        let current = move || mode.map_or(fallback, |m| m.get());
        let fetched = crate::net::read(PATH);
        provide_context(CanAnswer(Memo::new(move |_| fetched.with(answerable))));
        let cards = Memo::new(move |_| fetched.with(listed));
        let places = embeds();
        let drafts: Drafts = StoredValue::new(Vec::new());
        let clock = crate::net::ticker();
        let tick = move || clock.get();
        let one = move || {
            pick.get().map(|c| {
                let d = draft(drafts, &c.id);
                let (id, outcome) = (c.id.clone(), d.outcome.clone());
                Effect::new(move |_| {
                    if outcome.with(|o| matches!(o, Some(Outcome::Recorded { .. }))) {
                        recorded.run(id.clone());
                    }
                });
                let key = c.id.to_string();
                let live = Live {
                    nb: Memo::new(move |_| cards.with(|v| target_number(v, &key)).unwrap_or(0)),
                    node: Memo::new(|_| None),
                };
                card_view(c, d, live, false, Shared { tick, mode: current, places })
            })
        };
        one.into_any()
    }

    /// 席に届いていない裁定の 1 行（口を読み、1 秒の時計で書き直す・無いときは何も出さない・行 f-undelivered）。
    pub fn late_view() -> AnyView {
        let fetched = crate::net::read(UNRECEIVED_PATH);
        let clock = crate::net::ticker();
        let line = move || {
            fetched
                .with(|f| super::unreceived_line(f, clock.get()))
                .map(|text| view! { <div class="small" role="status">{text}</div> })
        };
        view! { {line} }.into_any()
    }

    fn card_view<T, M>(
        card: Card,
        d: Draft,
        live: Live,
        target: bool,
        shared: Shared<T, M>,
    ) -> AnyView
    where
        T: Fn() -> u64 + Copy + Send + Sync + 'static,
        M: Fn() -> Mode + Copy + Send + Sync + 'static,
    {
        let parts = LAYOUT
            .iter()
            .map(|slot| part_view(*slot, &card, &d, live, shared))
            .collect_view();
        view! {
            <article class=card_class(target) id=move || anchor(live.nb.get()) data-q=card.id.to_string()>{parts}</article>
        }
        .into_any()
    }

    /// 一覧の card がみな共にする値（経過の時計・link の mode・つながりの図の置き場）。
    #[derive(Clone, Copy)]
    struct Shared<T, M> {
        tick: T,
        mode: M,
        places: Embeds,
    }

    fn part_view<T, M>(
        slot: Slot,
        card: &Card,
        d: &Draft,
        live: Live,
        shared: Shared<T, M>,
    ) -> AnyView
    where
        T: Fn() -> u64 + Copy + Send + Sync + 'static,
        M: Fn() -> Mode + Copy + Send + Sync + 'static,
    {
        let Shared { tick, mode, places } = shared;
        match slot.part {
            Part::Head => {
                let a1 = card.a1.then(|| {
                    view! { <span class="warn" data-term="a1" tabindex="0" aria-label=label("a1") inner_html=WARN></span> }
                });
                // ほかの project の card は札を番号の後に置き、題をその project の地図を読まない字だけで出す。
                let tag = card.project.clone().map(|p| view! { <span class="chip">{p}</span> });
                let other = card.project.is_some();
                let (id, posted, title) = (card.id.to_string(), card.posted_at, card.title.clone());
                // 題の a は節点の card が替わったときだけ組み直す（番号とほかの部分は組み直さない）。
                let link = move || {
                    let (id, title) = (id.clone(), title.clone());
                    if other {
                        return view! { <span data-t="">{title}</span> }.into_any();
                    }
                    view! {
                        <a class="t" href=move || node_href(&id, mode()) use:attach_some=live.node.get()><span data-t="">{title}</span></a>
                    }
                    .into_any()
                };
                view! {
                    <div class=slot.class>
                        <span class="nb">{move || live.nb.get()}</span>
                        {tag}
                        {link}
                        {a1}
                        <span class="chip num" use:expert_tip=posted_tip(posted)><span inner_html=CLOCK></span><span>{move || age(tick(), posted)}</span></span>
                    </div>
                }
                .into_any()
            }
            Part::Summary => summary_view(slot, card),
            Part::Reason => {
                let key = slot.key.unwrap_or_default();
                view! {
                    <div class=slot.class>
                        <b data-term=key tabindex="0">{label(key)}</b>
                        <span data-t="">{card.reason.clone()}</span>
                    </div>
                }
                .into_any()
            }
            Part::Recommend => {
                let key = slot.key.unwrap_or_default();
                view! {
                    <div class=slot.class data-term=key>
                        <span inner_html=CHECK></span>
                        <div><b>{label(key)}</b>" "<span data-t="">{card.recommend.clone()}</span></div>
                    </div>
                }
                .into_any()
            }
            Part::Answer => answer_view(slot, card.clone(), d.clone()),
            // ほかの project の card はつながりの段を出さない（その project の地図を読まない）。
            Part::Around if card.project.is_some() => ().into_any(),
            Part::Around => around_view(slot, card, places),
        }
    }

    /// 要約の行（人の字と作りの字の印・字）。
    fn summary_view(slot: Slot, card: &Card) -> AnyView {
        let lines = card
            .summary
            .iter()
            .map(|l| {
                let icon = if l.eng { CODE } else { PERSON };
                view! {
                    <div class=l.class data-term=l.key>
                        <span inner_html=icon></span>
                        <span data-t="">{l.text.clone()}</span>
                    </div>
                }
            })
            .collect_view();
        view! { <div class=slot.class>{lines}</div> }.into_any()
    }

    /// つながりの段（開いたときに図の口を読み始め、開き閉じを記録へ書き戻す）。
    fn around_view(slot: Slot, card: &Card, places: Embeds) -> AnyView {
        let key = slot.key.unwrap_or_default();
        let initial = slot.open.unwrap_or(false);
        let (open, record) = fold(format!("ask:around:{}", card.id), move || initial);
        // つながりは開いたときに組む（段が開いた event で口を読み始め、開き閉じは記録へ書き戻す）。
        let center = card.id.to_string();
        let toggle = move |ev: web_sys::Event| {
            if event_target::<web_sys::Element>(&ev).has_attribute("open") {
                places.open(&center);
            }
            record(ev);
        };
        view! {
            <details class=slot.class prop:open=open on:toggle=toggle>
                <summary>
                    <span data-term=key>{label(key)}</span>
                    <span class="chip num" data-term="touches"><span inner_html=LINK></span>{label("touches")}" "{card.touches.len()}</span>
                    <span class="chip num" data-term="blocking"><span inner_html=STOP></span>{label("blocking")}" "{card.blocking.len()}</span>
                </summary>
                <div class="nb-body">{places.view(card.id.to_string())}</div>
            </details>
        }
        .into_any()
    }

    /// 答えの欄（字の欄と送る button）と、送った後の 1 行。200 の後は欄を閉じる。
    /// 読むだけの server の card とほかの project の card は、送る欄の代わりにチャットで答える 1 行を出す
    /// （context が無ければ server は答えを受ける）。
    fn answer_view(slot: Slot, card: Card, d: Draft) -> AnyView {
        let key = slot.key.unwrap_or_default();
        let own_ok = card.answerable;
        let can_answer = use_context::<CanAnswer>().map(|c| c.0);
        let answerable = move || own_ok && can_answer.is_none_or(|m| m.get());
        let open = {
            let outcome = d.outcome.clone();
            move || answerable() && outcome.with(|o| o.as_ref().is_none_or(Outcome::answer_open))
        };
        let chat = move || {
            (!answerable()).then(|| view! { <div class="small muted" data-term=CHAT_KEY>{label(CHAT_KEY)}</div> })
        };
        let form = {
            let d = d.clone();
            move || open().then(|| form_view(slot, key, &card, &d))
        };
        let note = {
            let outcome = d.outcome.clone();
            move || {
                outcome.get().map(|o| {
                    let (class, role) = o.note();
                    view! { <div class=class role=role>{o.line()}</div> }
                })
            }
        };
        view! { {form}{chat}{note} }.into_any()
    }

    /// 答えの字の欄と送る button（Ctrl か ⌘ と Enter でも送る）。
    fn form_view(slot: Slot, key: &'static str, card: &Card, d: &Draft) -> impl IntoView + use<> {
        let (text, sending) = (d.text.clone(), d.sending.clone());
        let busy = sending.clone();
        let send = move || send_text(busy.get(), &label("ruling"));
        let disabled = move || !can_send(&text.get(), sending.get());
        let value = {
            let text = d.text.clone();
            move || text.get()
        };
        let input = {
            let text = d.text.clone();
            move |ev: ev::Event| text.set(event_target_value(&ev))
        };
        let keydown = {
            let (card, d) = (card.clone(), d.clone());
            move |ev: ev::KeyboardEvent| {
                let composing = ev.is_composing() || ev.key_code() == 229;
                let action = key_action(composing, ev.ctrl_key() || ev.meta_key(), &ev.key());
                if action == KeyAction::Send {
                    ev.prevent_default();
                    submit(&card, &d);
                }
            }
        };
        let click = {
            let (card, d) = (card.clone(), d.clone());
            move |_: ev::MouseEvent| submit(&card, &d)
        };
        view! {
            <div class=slot.class>
                <textarea rows="2" aria-label=label(key) placeholder=label(key) prop:value=value on:input=input on:keydown=keydown></textarea>
                <button type="button" class="btn primary send" disabled=disabled on:click=click>{send}</button>
            </div>
        }
    }

    /// 答えを送る（押せないときは何もしない）。応答で card の状態を決め、200 と 409 は一覧を読み直す。
    fn submit(card: &Card, d: &Draft) {
        let text = d.text.get_untracked();
        if !can_send(&text, d.sending.get_untracked()) {
            return;
        }
        d.sending.set(true);
        let body = request_body(card, &text);
        let d = d.clone();
        spawn_local(async move {
            let reply = crate::net::post(RULING_PATH, body).await;
            let out = outcome(reply.as_ref().map(|(s, t)| (*s, t.as_str())));
            if !out.keeps_text() {
                d.text.set(String::new());
            }
            let reload = out.reloads();
            d.outcome.set(Some(out));
            d.sending.set(false);
            if reload {
                crate::net::reload_all();
            }
        });
    }
}
