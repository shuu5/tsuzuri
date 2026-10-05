//! block「次の一手」: 口 /api/next（契約の型の NextStep）の読みと、7 種の語の鍵と大きく出す 1 つの箱の字。
//! 判定は中核の crate が済ませていて、ここは写すだけ（大きく出す 1 つは電文の lead・各種の結果は電文の checks）。
//! 7 種の順は契約の型の宣言の順（`NextMove::ALL`）を引く。
//! 頁に block を置かない（1 枚の画面への切り替え）。帯の pill（topbar）と account board が
//! この module の純粋な関数（`step`・`key`・`unjudged`・`big`）を使う。block の DOM と、DOM だけが使った一覧の行・
//! 次の手の link・窓の探し方は消した（view は組み立ての script が module ごとに要るので空の中身）。

use tsuzuri_contract::board::NextMove;
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};
use tsuzuri_contract::wire;

use super::{Body, NO_CONTENT, NOT_READ, pipeline};
use crate::frame::Block;
use crate::view::Fetched;

pub const BLOCK: Block = Block {
    id: "next",
    heading: "next",
    class: "panel",
};

/// 読みの口（便 g-parts）。
pub const PATH: &str = "/api/next";

/// この file が字を持つ口の path（ほかの module の口を読む所は数えない・行 hs-derived）。
pub const PATHS: &[&str] = &[PATH];

/// この file の畳める段の開き閉じの鍵の形（無い・行 hs-derived）。
pub const FOLDS: &[&str] = &[];

/// 口が読めないときの理由。
pub const REASON: &str =
    "次の一手を判じる口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 7 種と語の鍵（1 か所の表）。並べる順は契約の型の宣言の順（`NextMove::ALL`）から引く。
pub const KEYS: [(NextMove, &str); 7] = [
    (NextMove::LimitOrMove, "nx_a"),
    (NextMove::Unresponsive, "nx_b"),
    (NextMove::StalledRun, "nx_c"),
    (NextMove::BatchApproval, "nx_d"),
    (NextMove::Question, "nx_e"),
    (NextMove::AwaitingEffect, "nx_f"),
    (NextMove::Nothing, "nx_g"),
];

/// なし（どれも当たらない）の箱の中身の字。
pub const NONE_LINE: &str = "orchestrator が動いている / 待っている";

/// なしを判じなかったときの大きい箱の語の鍵（見出しは「測れていない」・行 c-next-stall）。
pub const UNJUDGED_KEY: &str = "st_unknown";

/// なしを判じなかったときの大きい箱の中身の字。
pub const UNJUDGED_LINE: &str = "まだ判じていない種類があり、することが無いとは言えない";

/// account board の表と一覧の行に出す 1 行の字（なしの箱の字と測れていないの箱の字は空にして行の hover の card に任せる・
/// 行の数だけ同じ面の字が並んで最初の画面の散文を増やさない・行 g-accept-face）。
pub fn row_line(line: &str) -> &str {
    if line == NONE_LINE || line == UNJUDGED_LINE {
        ""
    } else {
        line
    }
}

/// 止まっている走行の箱の link の字（block「pipeline」へ頁の中で飛ぶ）。
pub const PIPE_LINK: &str = "run を見る ›";

/// 種類の語の鍵（表から引く）。
#[expect(clippy::expect_used, reason = "7 種の表は種類の全部を持つ")]
pub fn key(kind: NextMove) -> &'static str {
    KEYS.iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, key)| *key)
        .expect("7 種の表は種類の全部を持つ")
}

/// 頁の中の link。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub href: String,
    pub text: &'static str,
}

/// 大きく出す 1 つ（見本の `.nxbig`・なしは `.nxbig.none`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Big {
    pub kind: NextMove,
    pub key: &'static str,
    pub class: &'static str,
    /// 中身の字の class（なしは小さく淡く）。
    pub what_class: &'static str,
    pub what: String,
    pub link: Option<Link>,
    /// 電文の対象の id の字（無ければ None）。
    pub target: Option<String>,
}

/// 口の本文を電文に読む（まだ読んでいない・読めない・電文が読めないは理由）。
/// 電文が読めない本文の理由は、中身の無い block と同じ（便 g-parts の歯が 3 値の理由を pin する）。
pub fn step(fetched: &Fetched) -> Result<NextStep, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(REASON),
        Fetched::Body(text) => wire::decode::<NextStep>(text).map_err(|_| NO_CONTENT),
    }
}

/// 中身の有無（測れていない・中身あり・ほかの block と同じ 3 値の読みの形）。
pub fn body(fetched: &Fetched) -> Body<()> {
    match step(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(_) => Body::Filled(()),
    }
}

/// 種類の結果（電文に無い種類は None）。
fn check_of(step: &NextStep, kind: NextMove) -> Option<&NextCheck> {
    step.checks.iter().find(|c| c.kind == kind)
}

/// lead がなしで、電文のなしの結果が判じなかったか（なしの結果が電文に無ければ偽・行 c-next-stall）。
/// 判じるのは中核で、ここは電文の結果を読むだけ。
pub fn unjudged(step: &NextStep) -> bool {
    step.lead == NextMove::Nothing
        && check_of(step, NextMove::Nothing).is_some_and(|c| c.result == CheckResult::NotJudged)
}

/// 大きく出す 1 つの箱（止まっている走行は件数と block「pipeline」への link・質問は対象の id と件数・
/// なしは none の箱・ほかは件数と、在れば対象の id）。
pub fn big(kind: NextMove, check: Option<&NextCheck>) -> Big {
    let count = check.map_or(0, |c| c.count);
    let target = check.and_then(|c| c.target.as_ref());
    let counted = format!("{count} 件");
    let (what, link) = match kind {
        NextMove::Nothing => (NONE_LINE.to_string(), None),
        NextMove::StalledRun => (
            counted,
            Some(Link {
                href: format!("#{}", pipeline::BLOCK.id),
                text: PIPE_LINK,
            }),
        ),
        _ => match target {
            Some(t) => (format!("{t} · {counted}"), None),
            None => (counted, None),
        },
    };
    let none = kind == NextMove::Nothing;
    Big {
        kind,
        key: key(kind),
        class: if none { "nxbig none" } else { "nxbig" },
        what_class: if none { "what small muted" } else { "what" },
        what,
        link,
        target: target.map(ToString::to_string),
    }
}

/// 頁に置かない block の中身（空・組み立ての script の列挙 Module の view が module ごとに要る）。
#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    use leptos::prelude::*;

    ().into_any()
}
