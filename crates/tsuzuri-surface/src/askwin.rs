//! 質問の窓（行 g-ask-win・判断の記録 ADR-27 決定 (4)・見本 board-v2 の qModal と qsend と qlater）:
//! この project の問いを 1 問ずつ出し、答えを送って記録されると次の問いへ進み、全部に答えると閉じる。
//! 題の横の点で問いを選べ、「あとで」は後ろの答えていない問いへ移る。1 問の card と答えの口と記帳は今の質問の block と同じ
//! （project の ask の one_view）。窓の下の段は畳んだ段で、ほかの project の質問（読むだけ）・まとめて承認・全体への指示。
//! 質問の頁は 1 枚の画面への切り替えの行で消した。窓を開いた link が問いの id を持てば、その問いから出す（`AskFocus`・行 g-one-screen-b）。
//! 問いの一覧が読めない時は問いが無いと見せず、本文に「測れていない」と理由を、題の残りの数に「?」を出す。窓の上に席に
//! 届いていない裁定の 1 行（ask の late_view）を、ほかの project の質問の段に台帳が読めない組の札と 1 行を、前の質問の
//! block と同じく出す（憲法 P-7.1・P-7.2）。
//! 問いの選び方と局面と点の class は純粋な関数にして host で試し、窓の DOM（`frame`）は wasm の target のときだけ組む。

use std::collections::BTreeSet;

use crate::project::Body;
use crate::project::ask::{self, Card};
use crate::view::Fetched;
use crate::vocab::label;

/// 送って記録されてから次の問いへ移るまでの待ち（ms・「記録した」の 1 行を見せる・見本の qsend の 650）。
pub const ADVANCE_MS: u64 = 650;

/// 全部に答えてから窓を閉じるまでの待ち（ms・見本の qsend の 1200）。
pub const DONE_MS: u64 = 1200;

/// 全部に答えた時の字の語の鍵。
pub const DONE_KEY: &str = "ask_done";

/// 「あとで」の語の鍵。
pub const LATER_KEY: &str = "ask_later";

/// 窓の下の段（畳んだ段）の語の鍵の順: ほかの project の質問・まとめて承認・全体への指示。
pub const FOLDS: [&str; 3] = ["ask_others", "batch", "policy"];

/// 数が分からない時の字（題の残りの数とほかの project の質問の数・帯のやる事の数と同じ）。
pub const UNKNOWN_COUNT: &str = "?";

/// 窓の局面（問いの一覧が読めない・この project の問いが無い・1 問ずつ答えている・全部に答えた）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// 問いの一覧が読めない（理由の 1 行・問いが無いとは見せない）。
    Unmeasured(&'static str),
    Empty,
    Walk,
    Done,
}

/// 1 問ずつ出す問い（この project の問いの id・一覧の順・ほかの project の問いは外す）。
pub fn own_ids(cards: &[Card]) -> Vec<String> {
    cards
        .iter()
        .filter(|c| c.project.is_none())
        .map(|c| c.id.to_string())
        .collect()
}

/// 問いの一覧が読めない時の理由（ask の body が Unmeasured の時だけ・測れて 0 件と中身ありは None）。
pub fn unread(fetched: &Fetched) -> Option<&'static str> {
    match ask::body(fetched) {
        Body::Unmeasured(reason) => Some(reason),
        Body::Empty(_) | Body::Filled(_) => None,
    }
}

/// ほかの project の問いの数（問いの一覧が読めないか、台帳が読めないほかの project が在れば None・0 と書かない）。
pub fn others_count(fetched: &Fetched) -> Option<usize> {
    if unread(fetched).is_some() || !ask::unknown_projects(fetched).is_empty() {
        return None;
    }
    Some(
        ask::listed(fetched)
            .iter()
            .filter(|c| c.project.is_some())
            .count(),
    )
}

/// 局面: 一覧が読めなければ読めない（窓で答えた問いが在っても）、答えていない問いが在れば 1 問ずつ、無くて窓で答えた問いが
/// 在れば全部答えた、どちらも無ければ問いが無い。
pub fn phase(unread: Option<&'static str>, ids: &[String], answered: &BTreeSet<String>) -> Phase {
    if let Some(reason) = unread {
        Phase::Unmeasured(reason)
    } else if ids.iter().any(|id| !answered.contains(id)) {
        Phase::Walk
    } else if answered.is_empty() {
        Phase::Empty
    } else {
        Phase::Done
    }
}

/// 今の問い: 選んだ問いが一覧に在り答えていなければそれ、ほかは一覧の頭から最初の答えていない問い
/// （送った後も頭から探す・見本の qsend の findIndex）。
pub fn current<'a>(
    ids: &'a [String],
    picked: Option<&str>,
    answered: &BTreeSet<String>,
) -> Option<&'a str> {
    let open = |id: &&String| !answered.contains(*id);
    picked
        .and_then(|p| ids.iter().filter(open).find(|id| id.as_str() == p))
        .or_else(|| ids.iter().find(open))
        .map(String::as_str)
}

/// あとで: 今の問いの後ろへ巡って最初の答えていない問い（ほかに無ければ None で窓を閉じる・見本の qlater）。
/// 今の問いが一覧に無ければ一覧の頭から探す。
pub fn later<'a>(ids: &'a [String], cur: &str, answered: &BTreeSet<String>) -> Option<&'a str> {
    let n = ids.len();
    let at = ids.iter().position(|id| id == cur).map_or(n, |i| i + 1);
    (0..n)
        .map(|k| (at + k) % n.max(1))
        .filter_map(|i| ids.get(i))
        .find(|id| id.as_str() != cur && !answered.contains(*id))
        .map(String::as_str)
}

/// 答えていない問いの数。
pub fn left(ids: &[String], answered: &BTreeSet<String>) -> usize {
    ids.iter().filter(|id| !answered.contains(*id)).count()
}

/// 題の残りの数（一覧が読めない局面は None・ほかは答えていない問いの数）。
pub fn remaining(state: Phase, ids: &[String], answered: &BTreeSet<String>) -> Option<usize> {
    match state {
        Phase::Unmeasured(_) => None,
        Phase::Empty | Phase::Walk | Phase::Done => Some(left(ids, answered)),
    }
}

/// 数の字（分からなければ UNKNOWN_COUNT）。
fn count_text(n: Option<usize>) -> String {
    n.map_or_else(|| UNKNOWN_COUNT.to_string(), |n| n.to_string())
}

/// 窓の題の字（見本の「答えを待つ質問 <残り>」・残りが分からなければ「?」）。
pub fn title(left: Option<usize>) -> String {
    format!("{} {}", label("ask_open"), count_text(left))
}

/// 下の段のほかの project の質問の段の題（語とほかの project の問いの数・分からなければ「?」）。
pub fn others_title(n: Option<usize>) -> String {
    format!("{} {}", label(FOLDS[0]), count_text(n))
}

/// 題の横の点の class（今の問いは cur・答えた問いは ok・見本の `.qd`）。
pub fn dot_class(id: &str, cur: Option<&str>, answered: &BTreeSet<String>) -> &'static str {
    match (cur == Some(id), answered.contains(id)) {
        (true, true) => "qd cur ok",
        (true, false) => "qd cur",
        (false, true) => "qd ok",
        (false, false) => "qd",
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::{AskFocus, frame};

#[cfg(target_arch = "wasm32")]
mod dom {
    use std::collections::BTreeSet;
    use std::time::Duration;

    use leptos::prelude::*;
    use tsuzuri_contract::ledger::BeadId;

    use super::{
        DONE_KEY, DONE_MS, FOLDS, LATER_KEY, Phase, current, dot_class, later, others_count,
        others_title, own_ids, phase, remaining, title, unread,
    };
    use crate::project::{Body, ask, batch, body_view, policy};
    use crate::topbar::Win;
    use crate::vocab::label;
    use crate::widgets::modal::{Frame, WinCtx};
    use crate::wins::frame_of;

    /// 窓の下の段（ほかの project の質問は読むだけで、台帳が読めない組は札と 1 行・まとめて承認と全体への指示は block の本文）。
    fn folds(w: Walk) -> AnyView {
        let Walk {
            cards,
            others: n,
            unknown,
            ..
        } = w;
        let [_, batch_key, policy_key] = FOLDS;
        let others = move || {
            cards.with(|v| {
                v.iter()
                    .filter_map(|c| {
                        c.project
                            .clone()
                            .map(|p| (p, c.title.clone(), c.id.to_string()))
                    })
                    .collect::<Vec<_>>()
            })
        };
        let rows = move || {
            others()
                .into_iter()
                .map(|(p, t, id)| view! { <li><span class="chip">{p}</span><span class="ttl">{t}</span><code>{id}</code></li> })
                .collect_view()
        };
        // 台帳が読めないほかの project は、札と 1 行を読めた問いの後に出す（字は前の質問の block と同じ）。
        let unread_rows = move || {
            unknown
                .get()
                .into_iter()
                .map(|p| view! { <li class="small muted"><span class="chip">{p}</span>" "{ask::OTHER_UNKNOWN}</li> })
                .collect_view()
        };
        view! {
            <div class="folds">
                <details class="fold"><summary>{move || others_title(n.get())}</summary><ul class="items">{rows}{unread_rows}</ul></details>
                <details class="fold"><summary>{label(batch_key)}</summary>{batch::inner()}</details>
                <details class="fold"><summary>{label(policy_key)}</summary>{policy::inner()}</details>
            </div>
        }
        .into_any()
    }

    /// 窓の状態（問いの一覧・ほかの project の問いの数と台帳が読めない組・1 問ずつ出す問い・選んだ問い・窓で答えた問い・
    /// 今の問い・局面）。
    #[derive(Clone, Copy)]
    struct Walk {
        cards: Memo<Vec<ask::Card>>,
        others: Memo<Option<usize>>,
        unknown: Memo<Vec<String>>,
        ids: Memo<Vec<String>>,
        picked: RwSignal<Option<String>>,
        answered: RwSignal<BTreeSet<String>>,
        cur: Memo<Option<String>>,
        state: Memo<Phase>,
    }

    fn walk() -> Walk {
        let fetched = crate::net::read(ask::PATH);
        let cards = Memo::new(move |_| fetched.with(ask::listed));
        let others = Memo::new(move |_| fetched.with(others_count));
        let unknown = Memo::new(move |_| fetched.with(ask::unknown_projects));
        let reason = Memo::new(move |_| fetched.with(unread));
        let ids = Memo::new(move |_| cards.with(|v| own_ids(v)));
        // 窓を開いた link が問いの id を持てば、その問いから出す（行 g-one-screen-b）。
        let first = use_context::<AskFocus>().and_then(|f| f.0.get_untracked());
        let picked = RwSignal::new(first);
        let answered = RwSignal::new(BTreeSet::<String>::new());
        let cur = Memo::new(move |_| {
            ids.with(|i| {
                picked.with(|p| answered.with(|a| current(i, p.as_deref(), a).map(str::to_string)))
            })
        });
        let state = Memo::new(move |_| ids.with(|i| answered.with(|a| phase(reason.get(), i, a))));
        Walk {
            cards,
            others,
            unknown,
            ids,
            picked,
            answered,
            cur,
            state,
        }
    }

    /// 全部に答えたら少し待って閉じる（待つ間に別の窓へ移ったか、新しい問いが来たら閉じない）。
    fn close_when_done(ctx: WinCtx<Win>, state: Memo<Phase>) {
        Effect::new(move |_| {
            if state.get() == Phase::Done {
                set_timeout(
                    move || {
                        if ctx.top() == Some(Win::Ask) && state.get_untracked() == Phase::Done {
                            ctx.close();
                        }
                    },
                    Duration::from_millis(DONE_MS),
                );
            }
        });
    }

    /// 題（残りの数と、押すとその問いを選ぶ点）。
    fn head(w: Walk) -> AnyView {
        let Walk {
            ids,
            picked,
            answered,
            cur,
            state,
            ..
        } = w;
        let dots = move || {
            ids.get()
                .into_iter()
                .map(|id| {
                    let class = {
                        let id = id.clone();
                        move || cur.with(|c| answered.with(|a| dot_class(&id, c.as_deref(), a)))
                    };
                    let aria = id.clone();
                    view! { <button type="button" class=class aria-label=aria on:click=move |_| picked.set(Some(id.clone()))></button> }
                })
                .collect_view()
        };
        view! {
            {move || title(ids.with(|i| answered.with(|a| remaining(state.get(), i, a))))}
            <span class="qdots">{dots}</span>
        }
        .into_any()
    }

    /// 1 問の段（局面ごと・1 問は質問の block と同じ card・記録された問いは待ってから答えた問いに数える）。
    fn steps(ctx: WinCtx<Win>, w: Walk) -> AnyView {
        let Walk {
            cards,
            ids,
            picked,
            answered,
            cur,
            state,
            ..
        } = w;
        let card = Memo::new(move |_| {
            cur.with(|c| {
                c.as_ref()
                    .and_then(|id| cards.with(|v| v.iter().find(|k| k.id.as_str() == id).cloned()))
            })
        });
        let recorded = Callback::new(move |id: BeadId| {
            set_timeout(
                move || {
                    answered.update(|s| {
                        s.insert(id.to_string());
                    })
                },
                Duration::from_millis(super::ADVANCE_MS),
            );
        });
        let later_click = move |_| {
            let next = cur.get_untracked().and_then(|c| {
                ids.with_untracked(|i| {
                    answered.with_untracked(|a| later(i, &c, a).map(str::to_string))
                })
            });
            match next {
                Some(n) => picked.set(Some(n)),
                None => ctx.close(),
            }
        };
        let walk = move || {
            match state.get() {
            Phase::Unmeasured(reason) => body_view(Body::Unmeasured(reason)),
            Phase::Empty => body_view(Body::Empty(ask::EMPTY)),
            Phase::Done => view! { <div class="alldone">{label(DONE_KEY)}</div> }.into_any(),
            Phase::Walk => view! {
                {ask::one_view(card, recorded)}
                <div class="qbtns"><button type="button" class="later" on:click=later_click>{label(LATER_KEY)}</button></div>
            }
            .into_any(),
        }
        };
        walk.into_any()
    }

    /// 質問の窓を開いた link の問いの id（App が context に置く・窓を開く前に置く・行 g-one-screen-b）。
    #[derive(Clone, Copy)]
    pub struct AskFocus(pub RwSignal<Option<String>>);

    /// 質問の窓（帯の窓の層が開く・`ctx` は全部に答えた後に閉じるための窓の積み・本文の頭は席に届いていない裁定の 1 行）。
    pub fn frame(ctx: WinCtx<Win>) -> Frame {
        let w = walk();
        close_when_done(ctx, w.state);
        let walk = steps(ctx, w);
        Frame {
            width: frame_of(Win::Ask).0,
            title: head(w),
            body: view! { {ask::late_view()}{walk}{folds(w)} }.into_any(),
        }
    }
}
