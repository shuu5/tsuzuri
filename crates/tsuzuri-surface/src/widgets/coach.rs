//! 最初の案内（coach mark・見本 mock v3 の ui.js の COACH・startCoach・drawCoach・closeCoach と同じ形・行 g-coach）:
//! 初心者の mode の home の頁で最初の 3 手（次の一手・orchestrator の状態・pipeline の札）を輪で囲み、短い字の箱を出す。
//! 済み印は browser の保存に残し（持ち主の裁定 t3-hub.52.16・store の口）、query の `coach=1` で出し直す。
//! 見本との違い: 本番の block は電文を読んだ後に描くので、どれかの段の要素が出るまで 5 秒まで待ってから出す
//! （待ち切っても出なければ何も出さず済み印も書かない）。箱の横の収めは箱の幅を stylesheet の広い方の 240px で見込む。
//! 始めの決め・段の飛ばし・button の字・輪と箱の置き場は host でも組み立てて試し、DOM を撃つ所は wasm の target のときだけ組み立てる。

use crate::frame::{self, Mode};

/// 済み印の保存の鍵（見本の鍵と同じ）。
pub const COACH_KEY: &str = "tz-coach";

/// 済み印の値。
pub const DONE: &str = "done";

/// 済み印によらず出し直す query の名と値（`?coach=1`）。
pub const AGAIN_PARAM: &str = "coach";
pub const AGAIN: &str = "1";

/// 案内の 1 段（輪で囲む要素の selector と箱の字）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub sel: &'static str,
    pub text: &'static str,
}

/// 3 段（見本の COACH の字のまま・2 段目の selector は本番の orchestrator の block の id）。
pub const STEPS: [Step; 3] = [
    Step {
        sel: "#next .nxbig",
        text: "まずここ。押すと進む。",
    },
    Step {
        sel: "#orch .big",
        text: "orchestrator の状態は記号で。砂時計は利用枠の限度で停止中。",
    },
    Step {
        sel: ".kcard",
        text: "札に指を置くと中身、押すと詳しい頁へ。",
    },
];

/// 閉じる button の字。
pub const SKIP: &str = "閉じる";

/// 次の段へ進む button の字。
pub const NEXT: &str = "次へ";

/// 最後の段の button の字。
pub const GOT: &str = "分かった";

/// 箱の aria-label。
pub const LABEL: &str = "手引き";

/// 要素が出るのを待つ間（ミリ秒）と回数（200ms ごとに 25 回 = 5 秒）。
pub const WAIT_MS: u64 = 200;
pub const TRIES: u32 = 25;

/// 箱の幅の見込み（stylesheet の `.coach` の広い方の幅・ピクセル）。
pub const BOX_W: f64 = 240.0;

/// 頁を開いたときに始めるか: 初心者で、済み印が無いか query の `coach=1` のときだけ（見本の startCoach）。
pub fn starts(mode: Mode, saved: Option<&str>, search: &str) -> bool {
    mode == Mode::Beginner
        && (saved != Some(DONE) || frame::param(search, AGAIN_PARAM) == Some(AGAIN))
}

/// 段 `from` から先で要素の在る最初の段（見本の drawCoach の要素の無い段の飛ばし・無ければ None）。
pub fn first_found(from: usize, found: impl Fn(&str) -> bool) -> Option<usize> {
    STEPS
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, step)| found(step.sel))
        .map(|(i, _)| i)
}

/// 進む button の字（最後の段は分かった・ほかは次へ）。
pub fn next_text(step: usize) -> &'static str {
    if step + 1 == STEPS.len() { GOT } else { NEXT }
}

/// 点の列（今の段だけ真）。
pub fn dots(step: usize) -> Vec<bool> {
    (0..STEPS.len()).map(|i| i == step).collect()
}

/// 矩形（左上と幅と高さ・ピクセル）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

/// 輪の置き場: 要素の枠（窓の中の座標）を scroll で頁の座標にして 4px 外へ広げる。
pub fn ring(el: Rect, scroll_x: f64, scroll_y: f64) -> Rect {
    Rect {
        left: el.left + scroll_x - 4.0,
        top: el.top + scroll_y - 4.0,
        width: el.width + 8.0,
        height: el.height + 8.0,
    }
}

/// 箱の左上: 要素の下 12px で、横は 8px の余白で窓に収める（左と上の組）。
pub fn box_at(el: Rect, scroll_x: f64, scroll_y: f64, view_width: f64) -> (f64, f64) {
    let left = (el.left + scroll_x)
        .max(8.0)
        .min(scroll_x + view_width - BOX_W - 8.0);
    (left, el.top + el.height + scroll_y + 12.0)
}

#[cfg(target_arch = "wasm32")]
pub use dom::CoachLayer;

#[cfg(target_arch = "wasm32")]
mod dom {
    use std::time::Duration;

    use leptos::ev;
    use leptos::prelude::*;

    use super::{
        COACH_KEY, DONE, LABEL, Rect, SKIP, STEPS, TRIES, WAIT_MS, box_at, dots, first_found,
        next_text, ring, starts,
    };
    use crate::frame::Mode;
    use crate::store;
    use crate::widgets::help::HelpCtx;

    const WAIT: Duration = Duration::from_millis(WAIT_MS);

    /// 出している段と、輪と箱の置き場。
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Shown {
        step: usize,
        ring: Rect,
        at: (f64, f64),
    }

    /// selector の要素が在るか。
    fn found(sel: &str) -> bool {
        document().query_selector(sel).ok().flatten().is_some()
    }

    /// 段の要素の枠から輪と箱の置き場を決める（要素が無ければ None）。
    fn place(step: usize) -> Option<Shown> {
        let el = document().query_selector(STEPS.get(step)?.sel).ok().flatten()?;
        let r = el.get_bounding_client_rect();
        let win = window();
        let sx = win.scroll_x().unwrap_or(0.0);
        let sy = win.scroll_y().unwrap_or(0.0);
        let vw = document()
            .document_element()
            .map_or(0.0, |d| f64::from(d.client_width()));
        let rect = Rect {
            left: r.left(),
            top: r.top(),
            width: r.width(),
            height: r.height(),
        };
        Some(Shown {
            step,
            ring: ring(rect, sx, sy),
            at: box_at(rect, sx, sy, vw),
        })
    }

    /// 段 `from` から先で要素の在る最初の段を置く（無ければ None）。
    fn show_from(from: usize) -> Option<Shown> {
        first_found(from, found).and_then(place)
    }

    /// どれかの段の要素が出るまで WAIT ごとに `tries` 回まで待ってから `go` を撃つ（出なければ撃たない）。
    fn wait(tries: u32, go: impl FnOnce() + 'static) {
        if first_found(0, found).is_some() {
            go();
        } else if tries > 0 {
            set_timeout(move || wait(tries - 1, go), WAIT);
        }
    }

    /// 案内の輪と箱（home の頁に 1 つ・初心者の mode で出す・Esc は閉じるだけ）。
    #[component]
    pub fn CoachLayer() -> impl IntoView {
        let Some(c) = use_context::<HelpCtx>() else {
            return ().into_any();
        };
        let shown = RwSignal::new(None::<Shown>);
        // 始めと閉じの回（待ちの間に閉じたか始め直したかを見分ける）。
        let run = RwSignal::new(0_u32);
        let close = move || {
            run.update(|n| *n += 1);
            shown.set(None);
        };
        let start = move || {
            close();
            let now = run.get_untracked();
            wait(TRIES, move || {
                if run.get_untracked() == now && c.mode.get_untracked() == Mode::Beginner {
                    shown.set(show_from(0));
                }
            });
        };
        // 初めの 1 度は starts で決め、後は初心者へ替わると済み印によらず始め、経験者へ替わると閉じる（見本の setMode）。
        Effect::new(move |prev: Option<Mode>| {
            let mode = c.mode.get();
            match prev {
                None => {
                    let search = window().location().search().unwrap_or_default();
                    if starts(mode, store::get(COACH_KEY).as_deref(), &search) {
                        start();
                    }
                }
                Some(p) if p == mode => {}
                Some(_) if mode == Mode::Beginner => start(),
                Some(_) => close(),
            }
            mode
        });
        let skip = move |_| {
            close();
            store::set(COACH_KEY, DONE);
        };
        let next = move |_| {
            let from = shown.with_untracked(|s| s.map_or(STEPS.len(), |s| s.step + 1));
            match show_from(from) {
                Some(s) => shown.set(Some(s)),
                None => {
                    close();
                    store::set(COACH_KEY, DONE);
                }
            }
        };
        let esc = window_event_listener(ev::keydown, move |e| {
            if e.key() == "Escape" && shown.with_untracked(Option::is_some) {
                close();
            }
        });
        on_cleanup(move || esc.remove());
        let body = move || {
            shown.get().map(|s| {
                let text = STEPS.get(s.step).map_or("", |st| st.text);
                let marks = dots(s.step)
                    .into_iter()
                    .map(|on| view! { <i class=if on { "on" } else { "" }></i> })
                    .collect_view();
                let ring_at = format!(
                    "left:{}px;top:{}px;width:{}px;height:{}px",
                    s.ring.left, s.ring.top, s.ring.width, s.ring.height
                );
                let box_at = format!("left:{}px;top:{}px", s.at.0, s.at.1);
                view! {
                    <div class="coach-ring" style=ring_at></div>
                    <div class="coach" role="dialog" aria-label=LABEL style=box_at>
                        <div>{text}</div>
                        <div class="ft">
                            <span class="dots">{marks}</span>
                            <button type="button" on:click=skip>{SKIP}</button>
                            <button type="button" on:click=next>{next_text(s.step)}</button>
                        </div>
                    </div>
                }
            })
        };
        body.into_any()
    }
}
