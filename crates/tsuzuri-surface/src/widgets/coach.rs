//! 最初の案内（coach mark・行 g-coach・的と字と置き場は行 g-help-sweep で 1 枚の画面の部品に合わせた）:
//! 初心者の mode の home の頁で、帯の次の一手・帯の印・段の tile・札・⚙ を順に輪で囲み、短い字の帯を出す。
//! 済み印は browser の保存に残し（持ち主の裁定 t3-hub.52.16・store の口）、query の `coach=1` で出し直す。
//! 本番の部品は電文を読んだ後に描くので、どれかの段の要素が出るまで 5 秒まで待ってから出す
//! （待ち切っても出なければ何も出さず済み印も書かない）。的は selector の片ごとの最初の要素のうち見える最初の要素で、
//! 見えない段（幅で隠れる tile など）は飛ばす。
//! 字の帯は帯の下に固定し、頁の上の余白をその高さだけ広げる（stylesheet の `body:has(.coach)`）ので、札や行に重ならない。
//! 窓の幅が替わるか頁が scroll すると、今の段から輪を置き直す（今の段が見えなければ次の見える段・無ければ頭から）。
//! 始めの決め・段の飛ばし・置き直し・button の字・輪の置き場は host でも組み立てて試し、DOM を撃つ所は wasm の target のときだけ組み立てる。

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

/// 5 段（帯の次の一手・帯の印・段の tile・札・⚙）。tile は幅 600 以下だけ、ほかは全部の幅で見える。
pub const STEPS: [Step; 5] = [
    Step {
        sel: "#bar .pill",
        text: "まずここ。次の一手を押すと進む。",
    },
    Step {
        sel: "#bar .chip",
        text: "帯の印を押すと、その窓が開く。",
    },
    Step {
        sel: ".ptile",
        text: "段の tile を押すと、その段の札が下に開く。",
    },
    Step {
        sel: ".board .kcard, .ptlist .kcard",
        text: "札や一覧の行を押すと、吹き出しが開く。",
    },
    Step {
        sel: "#bar .gearb",
        text: "⚙ の記号の見方に、画面の見方がある。",
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

/// 置き直しの段: 今の段から先で要素の見える最初の段、無ければ頭から（窓の幅が替わった時・どこにも無ければ None）。
pub fn again(step: usize, found: impl Fn(&str) -> bool) -> Option<usize> {
    first_found(step, &found).or_else(|| first_found(0, &found))
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

/// 輪の置き場（窓に固定した座標）: 要素の枠（窓の中の座標）を 4px 外へ広げる。
pub fn ring(el: Rect) -> Rect {
    Rect {
        left: el.left - 4.0,
        top: el.top - 4.0,
        width: el.width + 8.0,
        height: el.height + 8.0,
    }
}

/// 段の selector の組の各片（`,` で分けた順・DOM は片ごとに最初の要素を見て、見える最初の要素を的にする）。
pub fn parts(sel: &str) -> impl Iterator<Item = &str> {
    sel.split(',').map(str::trim).filter(|p| !p.is_empty())
}

/// 要素が見えるか（枠の幅と高さが 0 より大きい・display が none の要素と祖先が隠す要素は枠が 0）。
pub fn visible(el: Rect) -> bool {
    el.width > 0.0 && el.height > 0.0
}

#[cfg(target_arch = "wasm32")]
pub use dom::CoachLayer;

#[cfg(target_arch = "wasm32")]
mod dom {
    use std::time::Duration;

    use leptos::ev;
    use leptos::prelude::*;
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::wasm_bindgen::closure::Closure;

    use super::{
        COACH_KEY, DONE, LABEL, Rect, SKIP, STEPS, TRIES, WAIT_MS, again, dots, first_found,
        next_text, parts, ring, starts, visible,
    };
    use crate::frame::Mode;
    use crate::store;
    use crate::widgets::help::HelpCtx;

    const WAIT: Duration = Duration::from_millis(WAIT_MS);

    /// 出している段と輪の置き場。
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Shown {
        step: usize,
        ring: Rect,
    }

    fn rect_of(el: &web_sys::Element) -> Rect {
        let r = el.get_bounding_client_rect();
        Rect {
            left: r.left(),
            top: r.top(),
            width: r.width(),
            height: r.height(),
        }
    }

    /// selector の片ごとの最初の要素のうち見える最初の要素の枠（無ければ None）。
    fn seen(sel: &str) -> Option<Rect> {
        parts(sel)
            .filter_map(|p| document().query_selector(p).ok().flatten())
            .map(|el| rect_of(&el))
            .find(|r| visible(*r))
    }

    /// selector の要素が見えているか。
    fn found(sel: &str) -> bool {
        seen(sel).is_some()
    }

    /// 段の要素の枠から輪の置き場を決める（見える要素が無ければ None）。
    fn place(step: usize) -> Option<Shown> {
        let el = seen(STEPS.get(step)?.sel)?;
        Some(Shown {
            step,
            ring: ring(el),
        })
    }

    /// 段 `from` から先で要素の見える最初の段を置く（無ければ None）。
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

    /// 出している案内の輪を今の段から置き直す（窓の幅が替わった時と頁か面が scroll した時・置けなければ閉じる）。
    fn replace(shown: RwSignal<Option<Shown>>) {
        if let Some(s) = shown.get_untracked() {
            shown.set(again(s.step, found).and_then(place));
        }
    }

    /// 幅を替えた後と scroll の後に輪を置き直す（字の帯を出すと頁の余白が替わるので、段を出した直後にも 1 度置き直す）。
    /// scroll は面の中の scroll も拾う（捕らえの段で受ける）。返すのは幅の替わりの受け手（層の片付けで外す）。
    fn follow(shown: RwSignal<Option<Shown>>) -> WindowListenerHandle {
        let scrolled =
            Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| replace(shown));
        let _ = document().add_event_listener_with_callback_and_bool(
            "scroll",
            scrolled.as_ref().unchecked_ref(),
            true,
        );
        // 頁の一生の間ずっと持つ（層は home の頁に 1 つ）。
        scrolled.forget();
        Effect::new(move |prev: Option<Option<usize>>| {
            let step = shown.with(|s| s.map(|s| s.step));
            if step.is_some() && prev != Some(step) {
                request_animation_frame(move || replace(shown));
            }
            step
        });
        window_event_listener(ev::resize, move |_| replace(shown))
    }

    /// 案内の輪と字の帯（home の頁に 1 つ・初心者の mode で出す・Esc は閉じるだけ）。
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
        let resize = follow(shown);
        on_cleanup(move || {
            esc.remove();
            resize.remove();
        });
        let body = move || shown.get().map(|s| coach_view(s, skip, next));
        body.into_any()
    }

    /// 案内の輪と字の帯の中身（段の字・段の点・閉じると次の button）。
    fn coach_view(
        s: Shown,
        skip: impl Fn(ev::MouseEvent) + Copy + 'static,
        next: impl Fn(ev::MouseEvent) + Copy + 'static,
    ) -> impl IntoView {
        let text = STEPS.get(s.step).map_or("", |st| st.text);
        let marks = dots(s.step)
            .into_iter()
            .map(|on| view! { <i class=if on { "on" } else { "" }></i> })
            .collect_view();
        let ring_at = format!(
            "left:{}px;top:{}px;width:{}px;height:{}px",
            s.ring.left, s.ring.top, s.ring.width, s.ring.height
        );
        view! {
            <div class="coach-ring" style=ring_at></div>
            <div class="coach" role="dialog" aria-label=LABEL>
                <div class="ct">{text}</div>
                <div class="ft">
                    <span class="dots">{marks}</span>
                    <button type="button" on:click=skip>{SKIP}</button>
                    <button type="button" on:click=next>{next_text(s.step)}</button>
                </div>
            </div>
        }
    }
}
