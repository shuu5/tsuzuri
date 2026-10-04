//! pipeline の札の段ごとの要の 1 行（行 g-pipe-cards・判断の記録 ADR-27 決定 (6)・見本 board-v2 の cardLine）:
//! Blocked は待つ相手と待ちの長さ・Queued は列に在る長さと起票からの経過（列に在る長さが `QUEUED_WARN_S`〔30 分・規則の行 R-37〕を
//! 越えると注意の印と色・列に入った時刻が無ければ付けない）・Running は回と経過と口座・Gated は gate と回と経過と口座・Failed と Stopped は
//! 20 字に切った理由と経過と回・Questioned は問いと経過と回・着地は着地の時刻と CI の語。待つ相手は吹き出しと同じ読み（`pop::wait_of` と
//! `pop::wait_on`）。今に依らない材料（`LineSrc`）は板を組む時に置き、字は描く時の今から組む（1 秒の時計で書き直す）。
//! Held（留め置き）は止めた者の印の語と止めてからの経過（行 g-held-col・判断の記録 ADR-42 決定 (7)）。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineCard, Reading, Stage};

use super::pop::{self, Src, Wait};
use crate::mapview::graph::cut;
use crate::project::pipeline::{
    CLOSED_STAGE, NO_ACCOUNT, NO_AGE, age_at, ci_key, closed_card, stage_word,
};
use crate::view::clock_short;
use crate::vocab::label;

/// Queued の札の注意の秒（列に在る長さが 30 分 を越えたら注意の色と印・規則の行 R-37・判断の記録 ADR-27 決定 (6)）。
pub const QUEUED_WARN_S: u64 = 1_800;

/// 注意の印（見本の ⚠）。
pub const WARN_MARK: &str = "⚠";

/// 要の 1 行の語の鍵（待ち・本待ち・列に・起票から・run の問い・着地）。
pub const LINE_KEYS: [&str; 6] = [
    "kl_wait",
    "kl_waits",
    "kl_queue",
    "kl_created",
    "kl_ask",
    "kl_landed",
];

/// 要の 1 行の class（ふつう・注意・止まり・問い）。
pub const LINE_CLASSES: [&str; 4] = ["kl", "kl klwarn", "kl klstop", "kl klask"];

/// 止まりの理由を切る字数（札の値の行と同じ 20）。
pub const WHY_CHARS: usize = 20;

/// 席の止めの理由の語（中核の crate の pipeline の `HOLD` の写し・面の crate は中核の crate に依存しない）。
pub const HOLD: &str = "hold";

/// 留め置きの札の止めた者（判断の記録 ADR-42 決定 (1)(7)・行 g-held-col）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeldBy {
    /// 席が器の止めの印で止めた（理由の語 `HOLD`）。
    Seat,
    /// 器の受付が断った（局面 contract-refused・理由は断りの名）。
    Intake,
}

/// 止めた者ごとの印の語の鍵と class（席の止め・受付の断り）。
pub const HELD_MARKS: [(HeldBy, &str, &str); 2] = [
    (HeldBy::Seat, "hm:seat", "hdm hd-seat"),
    (HeldBy::Intake, "hm:intake", "hdm hd-intake"),
];

impl HeldBy {
    /// 印の語の鍵。
    pub fn key(self) -> &'static str {
        self.mark().1
    }

    /// 印の class。
    pub fn class(self) -> &'static str {
        self.mark().2
    }

    #[expect(clippy::expect_used, reason = "印の表は止めた者の全部を持つ")]
    fn mark(self) -> (HeldBy, &'static str, &'static str) {
        HELD_MARKS
            .into_iter()
            .find(|(by, _, _)| *by == self)
            .expect("印の表は止めた者の全部を持つ")
    }
}

/// 札の止めた者（段 Held の札だけ・理由の語が `HOLD` なら席・ほかは受付・ほかの段は None）。
pub fn held_by(card: &PipelineCard) -> Option<HeldBy> {
    (card.stage == Stage::Held).then(|| {
        if card.reason.as_deref() == Some(HOLD) {
            HeldBy::Seat
        } else {
            HeldBy::Intake
        }
    })
}

/// 要の 1 行の材料（札の電文と、今に依らない字: Blocked の頭の字と起票の時刻）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineSrc {
    pub card: PipelineCard,
    /// Blocked の頭の字（相手が 1 つなら id と待ち・2 つ以上なら数と本待ち・承認待ちは持ち主の承認・分からなければ待ち）。
    pub wait: Option<String>,
    pub created: Option<EpochSecs>,
}

/// 要の 1 行（字と class）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLine {
    pub text: String,
    pub class: &'static str,
}

/// Queued の注意（段が Queued で列に入った時刻が在り、今から引いた経過が `QUEUED_WARN_S` を越える）。
pub fn queued_warn(card: &PipelineCard, now: EpochSecs) -> bool {
    card.stage == Stage::Queued
        && card
            .since
            .is_some_and(|s| now.saturating_sub(s) > QUEUED_WARN_S)
}

/// 要の 1 行の材料（待つ相手は吹き出しと同じ `pop::wait_of` と `pop::wait_on`・起票は bead の事実）。
pub fn line_src(card: &PipelineCard, src: &Src<'_>) -> LineSrc {
    let id = card.contract.as_str();
    let [wait, waits, ..] = LINE_KEYS;
    let lead = match (card.stage, pop::wait_of(src, card)) {
        (Stage::Blocked, Wait::Approval) => Some(label(pop::APPROVAL_KEYS[1])),
        (Stage::Blocked, Wait::Queue(..)) => Some(match pop::wait_on(src, id).as_deref() {
            Some([one]) => format!("{one} {}", label(wait)),
            Some(many) if many.len() > 1 => format!("{} {}", many.len(), label(waits)),
            _ => label(wait),
        }),
        _ => None,
    };
    let created = match src.facts {
        Reading::Known(f) => f
            .iter()
            .find(|x| x.id == card.contract)
            .and_then(|x| x.created_at),
        Reading::Unknown => None,
    };
    LineSrc {
        card: card.clone(),
        wait: lead,
        created,
    }
}

/// 段ごとの要の 1 行（閉じた（着地せず）の札は段の字と経過）。経過は今 now から引く。
pub fn key_line(line: &LineSrc, now: EpochSecs) -> KeyLine {
    let [wait, _, queue, created, ask, landed] = LINE_KEYS;
    let [plain, warn, stop, asked] = LINE_CLASSES;
    let card = &line.card;
    let age = age_at(card.since, now);
    let account = card.account.as_deref().unwrap_or(NO_ACCOUNT);
    let (text, class) = match card.stage {
        _ if closed_card(card) => (format!("{CLOSED_STAGE} · {age}"), plain),
        Stage::Blocked => {
            let lead = line.wait.clone().unwrap_or_else(|| label(wait));
            (format!("{lead} · {age}"), plain)
        }
        Stage::Held => {
            let mark = held_by(card).map_or_else(|| label(queue), |h| label(h.key()));
            (format!("{mark} · {age}"), plain)
        }
        Stage::Queued => {
            let made = age_at(line.created, now);
            let body = format!("{} {age} · {} {made}", label(queue), label(created));
            if queued_warn(card, now) {
                (format!("{WARN_MARK} {body}"), warn)
            } else {
                (body, plain)
            }
        }
        Stage::Running => (format!("run {} · {age} · {account}", card.runs), plain),
        Stage::Gated => (
            format!("gate · run {} · {age} · {account}", card.runs),
            plain,
        ),
        Stage::Failed | Stage::Stopped => {
            let why = card.reason.clone().unwrap_or_else(|| stage_word(card));
            let head = cut(&why, WHY_CHARS);
            (format!("{head} · {age} · run {}", card.runs), stop)
        }
        Stage::Questioned => (format!("{} · {age} · run {}", label(ask), card.runs), asked),
        Stage::Landed => {
            let at = card
                .since
                .map_or_else(|| NO_AGE.to_string(), |s| clock_short(s, now));
            let ci = card
                .ci
                .map_or_else(|| NO_AGE.to_string(), |c| label(ci_key(c)));
            (format!("{at} {} · CI {ci}", label(landed)), plain)
        }
    };
    KeyLine { text, class }
}
