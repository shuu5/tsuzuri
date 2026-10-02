//! 帯の印が開く窓の中身（行 g-win-parts・判断の記録 ADR-27 決定 (4)(8)・見本 board-v2 の stalledModal と noticeModal と
//! seatModal と gapsModal と legendModal と destModal）: 止まった run・知らせ・席と口座・抜けの検査の 4 つの窓と、
//! 設定（⚙）の中の記号の見方と表示先の窓。
//! 知らせ・席と口座・抜けの検査・記号の見方・表示先は、今の block の中身の関数（各 module の inner）を窓の中で描く
//! （頁から block を外したのは 1 枚の画面への切り替えの行）。席と口座の窓の稼働の記録は 24 時間の幅だけ。
//! 止まった run の窓は pipeline の口の止まりの列の札を、題の全体と段と経過と理由の全文と個別の頁への口で並べる。
//! 質問の窓は 1 問ずつの窓（askwin の frame・行 g-ask-win）。
//! 字と並びは純粋な関数にして host で試し、窓の DOM（`draw`）は wasm の target のときだけ組む。

use std::cmp::Reverse;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineCard, PipelineColumn, Reading};

use crate::frame::{Mode, node_href};
use crate::project::Body;
use crate::project::pipeline::{self, age_at, stage_word};
use crate::topbar::{SEAT_KEY, STALL_KEY, Win};
use crate::view::{Fetched, id_order, read_rows};

/// 窓の幅（px・見本の各窓の w）と題の語の鍵。
pub fn frame_of(win: Win) -> (u32, &'static str) {
    match win {
        Win::Ask => (760, "ask_open"),
        Win::Stalled => (720, STALL_KEY),
        Win::Notices => (600, "notice"),
        Win::Seat => (760, SEAT_KEY),
        Win::Gaps => (600, "gaps"),
        Win::Legend => (560, "status"),
        Win::Dest => (520, "stage_target"),
    }
}

/// 止まった run の窓の 1 枚（bead の id・題の全体・段の字・経過・理由の全文・個別の頁の URL）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stalled {
    pub id: String,
    pub title: Option<String>,
    pub stage: String,
    pub age: String,
    pub reason: Option<String>,
    pub href: String,
}

/// 止まった run が無い時の 1 行。
pub const STALLED_NONE: &str = "止まった run は無い";

/// 止まった run の窓の中身: pipeline の口の札のうち止まりの列（Questioned・Failed・Stopped）を、段を決めた時刻の新しい順
/// （時刻の無い札は後・同じなら bead の id の順）に並べる。題は台帳の一覧から引いた題の全体（引けなければ None）で、
/// 経過は今から、個別の頁は今の節点の頁の URL。口が読めなければ測れていない、止まりの札が無ければ空。
pub fn stalled(pipe: &Fetched, ledger: &Fetched, now: EpochSecs, mode: Mode) -> Body<Vec<Stalled>> {
    let cards = match pipeline::cards(pipe) {
        Ok(c) => c,
        Err(reason) => return Body::Unmeasured(reason),
    };
    let rows = match read_rows(ledger) {
        Reading::Known(rows) => rows,
        Reading::Unknown => Vec::new(),
    };
    let mut stop: Vec<&PipelineCard> = cards
        .iter()
        .filter(|c| c.stage.column() == PipelineColumn::QuestionedFailedStopped)
        .collect();
    stop.sort_by(|a, b| {
        let key = |c: &PipelineCard| (c.since.is_none(), Reverse(c.since));
        key(a)
            .cmp(&key(b))
            .then_with(|| id_order(a.contract.as_str(), b.contract.as_str()))
    });
    if stop.is_empty() {
        return Body::Empty(STALLED_NONE);
    }
    Body::Filled(
        stop.into_iter()
            .map(|c| {
                let id = c.contract.as_str();
                Stalled {
                    id: id.to_string(),
                    title: rows
                        .iter()
                        .find(|r| r.id.as_str() == id)
                        .map(|r| r.title.clone()),
                    stage: stage_word(c),
                    age: age_at(c.since, now),
                    reason: c.reason.clone(),
                    href: node_href(id, mode),
                }
            })
            .collect(),
    )
}

#[cfg(target_arch = "wasm32")]
pub use dom::draw;

#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::{Stalled, frame_of, stalled};
    use crate::frame::Mode;
    use crate::project::{
        Body, body_view, gaps, ledger, legend, notice, pipeline, seat, stage, unmeasured,
    };
    use crate::topbar::Win;
    use crate::vocab::label;
    use crate::widgets::modal::{Frame, WinCtx};

    /// 今の URL の表示の型。
    fn mode() -> Mode {
        Mode::from_query(&window().location().search().unwrap_or_default())
    }

    fn card_view(c: Stalled) -> AnyView {
        let head = c.title.clone().unwrap_or_else(|| c.id.clone());
        let reason = c
            .reason
            .map(|r| view! { <div class="stc-r"><b>{label("reason")}</b>" "{r}</div> });
        view! {
            <div class="stc">
                <div class="stc-h"><b>{head}</b><code>{c.id}</code><span class="sc">{format!("{} · {}", c.stage, c.age)}</span></div>
                {reason}
                <div class="stc-b"><a class="sm" href=c.href>{label("open_node")}" ›"</a></div>
            </div>
        }
        .into_any()
    }

    /// 止まった run の窓の本文（pipeline の口と台帳の一覧の口を読む）。
    fn stalled_view() -> AnyView {
        let pipe = crate::net::read(pipeline::PATH);
        let rows = crate::net::read(ledger::PATH);
        let list =
            move || match pipe.with(|p| rows.with(|l| stalled(p, l, crate::net::now(), mode()))) {
                Body::Unmeasured(reason) => unmeasured(reason),
                Body::Empty(line) => body_view(Body::Empty(line)),
                Body::Filled(cs) => cs.into_iter().map(card_view).collect_view().into_any(),
            };
        list.into_any()
    }

    /// 窓の名から中身を組む（窓の層 widgets::modal::layer の draw に渡す・`ctx` は質問の窓が全部に答えた後に閉じる口）。
    pub fn draw(win: Win, ctx: WinCtx<Win>) -> Frame {
        let (width, key) = frame_of(win);
        let body = match win {
            Win::Ask => return crate::askwin::frame(ctx),
            Win::Stalled => stalled_view(),
            Win::Notices => notice::inner(),
            Win::Seat => seat::inner(),
            Win::Gaps => gaps::inner(),
            Win::Legend => legend::inner(),
            Win::Dest => stage::inner(),
        };
        Frame {
            width,
            title: label(key).into_any(),
            body,
        }
    }
}
