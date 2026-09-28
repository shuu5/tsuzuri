//! block「次の一手」（見本の `#next` と index.html の nextHTML・便 g-next）: 大きく出す 1 つの箱と、残りの種類の一覧。
//! 中身は口 /api/next（契約の型の NextStep）から読む。判定は中核の crate が済ませていて、ここは写すだけ
//! （大きく出す 1 つは電文の lead・各種の結果は電文の checks）。7 種の順は契約の型の宣言の順（`NextMove::ALL`）を引く。
//! 字と並びは純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use tsuzuri_contract::board::NextMove;
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};
use tsuzuri_contract::wire;

use super::node::answer_href;
use super::{Body, NO_CONTENT, NOT_READ, batch, map, pipeline};
use crate::account::windows::ACCOUNT_WIN;
use crate::account::{Tab, tab_href};
use crate::frame::{Block, Mode, PageId, href};
use crate::view::Fetched;
use crate::widgets::hover::{Card, clip};
use crate::widgets::nodecard::card_of;

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

/// 当たらない種類の字。
pub const MISS: &str = "―";

/// なし（どれも当たらない）の箱の中身の字。
pub const NONE_LINE: &str = "orchestrator が動いている / 待っている";

/// なしを判じなかったときの大きい箱の語の鍵（見出しは「測れていない」・行 c-next-stall）。
pub const UNJUDGED_KEY: &str = "st_unknown";

/// なしを判じなかったときの大きい箱の中身の字。
pub const UNJUDGED_LINE: &str = "まだ判じていない種類があり、することが無いとは言えない";

/// 止まっている走行の箱の link の字（block「pipeline」へ頁の中で飛ぶ）。
pub const PIPE_LINK: &str = "run を見る ›";

/// 限度と移動の箱の link の字（account board の home の tab へ）。
pub const ACCOUNT_LINK: &str = "account board を見る ›";

/// 応答なしの箱の link の字（account board の session の tab へ）。
pub const SESSION_LINK: &str = "session を見る ›";

/// 束の承認の箱の link の字（問いの頁の束の block へ・見本の字）。
pub const BATCH_LINK: &str = "まとめて承認 ›";

/// 質問の箱の link の字（問いの頁の card へ・見本の字）。
pub const ANSWER_LINK: &str = "答える ›";

/// 発効待ちの箱の link の字（抜けの検査の頁へ・見本の字）。
pub const GAPS_LINK: &str = "orchestrator に任せる（見るだけ） ›";

/// 種類の語の鍵（表から引く）。
pub fn key(kind: NextMove) -> &'static str {
    KEYS.iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, key)| *key)
        .expect("7 種の表は種類の全部を持つ")
}

/// 一覧の右の字（当たった件数・当たらない「―」・判じなかった測れていないの記号）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Count(u32),
    Miss,
    Unmeasured,
}

/// 一覧の 1 行（見本の `ul.nxlist > li`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub kind: NextMove,
    pub key: &'static str,
    /// 当たった種類は on・ほかは off。
    pub class: &'static str,
    pub mark: Mark,
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

/// block の中身（大きく出す 1 つと、残りの種類の一覧）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Next {
    pub big: Big,
    pub rest: Vec<Row>,
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

/// 中身の有無（測れていない・中身あり）。中身は `content` が組む。
pub fn body(fetched: &Fetched) -> Body<()> {
    match step(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(_) => Body::Filled(()),
    }
}

/// block の中身（口が読めなければ測れていない）。
pub fn content(fetched: &Fetched) -> Body<Next> {
    match step(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(s) => Body::Filled(next(&s)),
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

/// 電文を中身に組む（大きく出すのは電文の lead・一覧は 7 種の順から lead を除いた並び）。
/// なしを判じなかったなら、大きい箱は測れていないの箱（`UNJUDGED_KEY` と `UNJUDGED_LINE`）。
pub fn next(step: &NextStep) -> Next {
    let rest = NextMove::ALL
        .into_iter()
        .filter(|&kind| kind != step.lead)
        .map(|kind| row(kind, check_of(step, kind)))
        .collect();
    let big = if unjudged(step) {
        Big {
            kind: NextMove::Nothing,
            key: UNJUDGED_KEY,
            class: "nxbig none",
            what_class: "what small muted",
            what: UNJUDGED_LINE.to_string(),
            link: None,
            target: None,
        }
    } else {
        big(step.lead, check_of(step, step.lead))
    };
    Next { big, rest }
}

/// 一覧の 1 行（電文に無い種類は判じなかったと同じ）。
pub fn row(kind: NextMove, check: Option<&NextCheck>) -> Row {
    let mark = match check.map(|c| (c.result, c.count)) {
        Some((CheckResult::Hit, n)) => Mark::Count(n),
        Some((CheckResult::Miss, _)) => Mark::Miss,
        Some((CheckResult::NotJudged, _)) | None => Mark::Unmeasured,
    };
    Row {
        kind,
        key: key(kind),
        class: if matches!(mark, Mark::Count(_)) {
            "on"
        } else {
            "off"
        },
        mark,
    }
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

/// 質問の大きい箱の中身（問いの節点の id・節点の題を 36 字に切った字・節点の card）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigTitle {
    pub id: String,
    pub text: String,
    pub card: Card,
}

/// 質問の大きい箱の題（見本の nx_e の箱: 問いの題の字を節点の頁への link にして節点の card を付ける）。
/// 質問のほかの種類・対象が無い・グラフの口が読めない・問いの節点が電文に無いときは None（中身は Big の what の字のまま）。
pub fn big_title(big: &Big, graph: &Fetched) -> Option<BigTitle> {
    if big.kind != NextMove::Question {
        return None;
    }
    let id = big.target.as_deref()?;
    let doc = map::doc(graph).ok()?;
    let card = card_of(&doc, id)?;
    Some(BigTitle {
        id: id.to_string(),
        text: clip(&card.title),
        card,
    })
}

/// 種類ごとの次の手の頁への link（見本の nextItems の button・止まっている走行は Big の link・なしは無い）。
/// 質問は対象の id が在ればその card へ、無ければ問いの頁へ。どれも mode を URL に残す。
pub fn action(kind: NextMove, target: Option<&str>, mode: Mode) -> Option<Link> {
    let (href, text) = match kind {
        NextMove::LimitOrMove => (tab_href(Tab::Home, mode), ACCOUNT_LINK),
        NextMove::Unresponsive => (tab_href(Tab::Session, mode), SESSION_LINK),
        NextMove::BatchApproval => (
            format!("{}#{}", href(PageId::Ask, mode), batch::BLOCK.id),
            BATCH_LINK,
        ),
        NextMove::Question => (
            target.map_or_else(|| href(PageId::Ask, mode), |id| answer_href(id, mode)),
            ANSWER_LINK,
        ),
        NextMove::AwaitingEffect => (href(PageId::Gaps, mode), GAPS_LINK),
        NextMove::StalledRun | NextMove::Nothing => return None,
    };
    Some(Link { href, text })
}

/// link を開く窓の名（account board へ飛ぶ種類は account board の名前つきの窓・ほかは今の窓）。
pub fn window_of(kind: NextMove) -> Option<&'static str> {
    match kind {
        NextMove::LimitOrMove | NextMove::Unresponsive => Some(ACCOUNT_WIN),
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 次の一手の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::{
        BLOCK, Big, BigTitle, MISS, Mark, Next, PATH, Row, action, big_title, content, window_of,
    };
    use crate::frame::{Mode, node_href};
    use crate::project::{Body, UNKNOWN, body_view, map, section, state_icon, unmeasured};
    use crate::widgets::help::{HelpCtx, hs};
    use crate::widgets::hover::attach;

    pub fn view() -> AnyView {
        let fetched = crate::net::read(PATH);
        let graph = crate::net::read(map::PATH);
        // 今の mode（context が無ければ今の URL の query から・link に mode を残す）。
        let ctx = use_context::<HelpCtx>();
        let url = Mode::from_query(&crate::mapview::current());
        let mode = move || ctx.map_or(url, |c| c.mode.get());
        let body = move || match fetched.with(content) {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Filled(next) => {
                let title = graph.with(|g| big_title(&next.big, g));
                next_view(next, title, mode)
            }
        };
        section(BLOCK, ().into_any(), body.into_any())
    }

    fn next_view(
        next: Next,
        title: Option<BigTitle>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let rows = next.rest.into_iter().map(row_view).collect_view();
        view! {
            {big_view(next.big, title, mode)}
            <ul class="nxlist">{rows}</ul>
        }
        .into_any()
    }

    /// 大きい箱（質問の箱で問いの節点が引ければ、中身は問いの題の字の節点の頁への link と節点の card・ほかは Big の what の字）。
    fn big_view(
        big: Big,
        title: Option<BigTitle>,
        mode: impl Fn() -> Mode + Copy + Send + Sync + 'static,
    ) -> AnyView {
        let what = match title {
            Some(t) => {
                let id = t.id.clone();
                let href = move || node_href(&id, mode());
                view! { <a href=href use:attach=t.card>{t.text}</a> }.into_any()
            }
            None => big.what.into_any(),
        };
        let (kind, link, target) = (big.kind, big.link, big.target);
        let win = window_of(kind);
        // Big の link（止まっている走行）が在ればそれを、無ければ種類の次の手の頁への link（mode で href が変わる）。
        let act = move || {
            link.clone()
                .or_else(|| action(kind, target.as_deref(), mode()))
                .map(|l| {
                    view! { <div class="act"><a class="btn primary" href=l.href target=win>{l.text}</a></div> }
                })
        };
        view! {
            <div class=big.class data-nx=big.key>
                {hs(big.key)}
                <div class=big.what_class>{what}</div>
                {act}
            </div>
        }
        .into_any()
    }

    fn row_view(row: Row) -> AnyView {
        let mark = match row.mark {
            Mark::Count(n) => view! { <span class="num">{n}</span> }.into_any(),
            Mark::Miss => MISS.into_any(),
            Mark::Unmeasured => state_icon(UNKNOWN),
        };
        view! {
            <li class=row.class data-nx=row.key>
                {hs(row.key)}
                <span class="v small">{mark}</span>
            </li>
        }
        .into_any()
    }
}
