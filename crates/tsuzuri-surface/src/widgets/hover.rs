//! hover の card（見本の ui.js の card4 と同じ 4 行 + 「詳しく ▸」・便 g-parts）。節点と札に指を置くと出す。
//! card の層は頁に 1 つ（`CardLayer`）で、block は要素に card を付ける関数（`attach`）を呼ぶだけにする。
//! 置き場・猶予の判定・行の切り方は純粋な関数にして host で試す。値は規則の行 R-20（右 16 px・上 20 px・猶予 150 ms）と
//! 行 R-19（4 行・1 行 36 字）の値で、歯が rules の file の字と照らす。

/// card の左上を pointer の右へずらす幅（ピクセル・規則の行 R-20）。窓の右端を越えるときは pointer の左へ同じ幅。
pub const OFFSET_X: i32 = 16;

/// card の左上を pointer の上へずらす高さ（ピクセル・規則の行 R-20）。
pub const OFFSET_Y: i32 = 20;

/// 要素を出てから card を残せる猶予（ミリ秒・規則の行 R-20）。
pub const GRACE_MS: u32 = 150;

/// card の行の数（題・種類・値・出所・規則の行 R-19）。
pub const ROWS: usize = 4;

/// card の 1 行の字数の上限（規則の行 R-19）。超える行は `ROW_CHARS - 1` 字と「…」に切る。
pub const ROW_CHARS: usize = 36;

/// 切った行の末尾に付ける字。
pub const ELLIPSIS: char = '…';

/// 窓の中の点（ピクセル・窓の左上が原点）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// 幅と高さ（ピクセル）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// 矩形（左上と幅と高さ・ピクセル）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    /// 点から矩形までの距離（矩形の中なら 0）。
    pub fn distance(&self, p: Point) -> f64 {
        let dx = (self.left - p.x)
            .max(p.x - (self.left + self.width))
            .max(0.0);
        let dy = (self.top - p.y)
            .max(p.y - (self.top + self.height))
            .max(0.0);
        dx.hypot(dy)
    }
}

/// card の左上の置き場。pointer の右 16 px・上 20 px に置く。
/// card の右端が窓の右端を越えるときは pointer の左 16 px に card の右端を置く（それでも左端を越えるなら左端に寄せる）。
/// card が窓の上端か下端を越えるときは、窓に収まるまで縦に寄せる（窓より高い card は上端に揃える）。
pub fn place(pointer: Point, card: Size, window: Size) -> Point {
    let dx = f64::from(OFFSET_X);
    let right = pointer.x + dx;
    let x = if right + card.width > window.width {
        (pointer.x - dx - card.width).max(0.0)
    } else {
        right
    };
    let y = (pointer.y - f64::from(OFFSET_Y))
        .min(window.height - card.height)
        .max(0.0);
    Point { x, y }
}

/// 猶予の判定: 要素を出てから 150 ms 以内で、今の pointer から card の矩形までの距離が出た点より縮んでいれば残す。
pub fn keeps(exit: Point, now: Point, card: Rect, elapsed_ms: f64) -> bool {
    (0.0..=f64::from(GRACE_MS)).contains(&elapsed_ms) && card.distance(now) < card.distance(exit)
}

/// 行の切り方: 36 字を超える行は 35 字と「…」にする（字は Unicode の scalar で数える）。
pub fn clip(line: &str) -> String {
    if line.chars().count() <= ROW_CHARS {
        return line.to_string();
    }
    let mut out: String = line.chars().take(ROW_CHARS - 1).collect();
    out.push(ELLIPSIS);
    out
}

/// card の中身（題・種類の行・値の行・出所・詳しく）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Card {
    pub title: String,
    pub kind: String,
    pub value: String,
    /// 出所の file と行（見本は card に出所を必ず持たせる）。
    pub src: String,
    pub more: Vec<String>,
}

impl Card {
    /// 4 行の class と字（見本の `.c4 .r.ti` などの行の順・36 字を超える行は切る）。
    pub fn rows(&self) -> [(&'static str, String); ROWS] {
        [
            ("r ti", clip(&self.title)),
            ("r k", clip(&self.kind)),
            ("r v", clip(&self.value)),
            ("r src", clip(&self.src)),
        ]
    }
}

/// card の箱の class（出ているか）。
pub fn card_class(open: bool) -> &'static str {
    if open { "hcard c4 on" } else { "hcard c4" }
}

/// 委ねで指が入ったときに card を出し直すか（出している card と同じ値なら出し直さず残す）。
pub fn over_shows(shown: Option<&Card>, card: &Card) -> bool {
    shown != Some(card)
}

/// 委ねで指が出たときに猶予に入るか（出た節点の鍵が在り、移った先が同じ節点でないときだけ・同じ節点の子の間の移りは数えない）。
pub fn leaves(from: Option<&str>, to: Option<&str>) -> bool {
    from.is_some() && from != to
}

/// 外した要素の後始末で card を閉じるか（要素の出した card の番号 `shown` が層の今の番号 `now` と同じときだけ・
/// 番号 0 はまだ出していない要素・後から別の要素が出した card と猶予の後に消えた card には触らない）。
pub fn unmount_hides(shown: u64, now: u64) -> bool {
    shown != 0 && shown == now
}

#[cfg(target_arch = "wasm32")]
pub use dom::{CardLayer, Delegate, HoverCtx, attach, attach_some, delegate};

#[cfg(target_arch = "wasm32")]
mod dom {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    use leptos::ev;
    use leptos::html::Div;
    use leptos::prelude::*;
    use web_sys::js_sys::Date;
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::wasm_bindgen::closure::Closure;

    use super::{
        Card, GRACE_MS, Point, Rect, Size, card_class, keeps, over_shows, place, unmount_hides,
    };
    use crate::vocab::label;

    /// 要素を出た点と時刻（ミリ秒）。
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Leaving {
        from: Point,
        at_ms: f64,
    }

    /// card の層の状態（App が context に置く・頁に 1 つ）。
    #[derive(Clone, Copy)]
    pub struct HoverCtx {
        card: RwSignal<Option<Card>>,
        /// 置いた左上（測って置くまでは None）。
        at: RwSignal<Option<Point>>,
        pointer: StoredValue<Point>,
        leaving: StoredValue<Option<Leaving>>,
        /// 出すたび・消すたびに進める番号（古い猶予の timer が新しい card を消さないように）。
        seq: StoredValue<u64>,
        node: NodeRef<Div>,
    }

    impl Default for HoverCtx {
        fn default() -> Self {
            Self {
                card: RwSignal::new(None),
                at: RwSignal::new(None),
                pointer: StoredValue::new(Point { x: 0.0, y: 0.0 }),
                leaving: StoredValue::new(None),
                seq: StoredValue::new(0),
                node: NodeRef::new(),
            }
        }
    }

    fn point(ev: &web_sys::MouseEvent) -> Point {
        Point {
            x: f64::from(ev.client_x()),
            y: f64::from(ev.client_y()),
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

    impl HoverCtx {
        /// 層の箱の矩形（まだ描いていなければ None）。
        fn rect(self) -> Option<Rect> {
            let r = self.node.get_untracked()?.get_bounding_client_rect();
            Some(Rect {
                left: r.left(),
                top: r.top(),
                width: r.width(),
                height: r.height(),
            })
        }

        /// card を出す（描いた後に大きさを測って置く）。
        fn show(self, card: Card, pointer: Point) {
            self.seq.update_value(|n| *n += 1);
            self.leaving.set_value(None);
            self.pointer.set_value(pointer);
            self.at.set(None);
            self.card.set(Some(card));
            request_animation_frame(move || self.settle());
        }

        /// 描いた card の大きさと窓の大きさから置き場を決める。
        fn settle(self) {
            if let Some(r) = self.rect() {
                let size = Size {
                    width: r.width,
                    height: r.height,
                };
                self.at
                    .set(Some(place(self.pointer.get_value(), size, window_size())));
            }
        }

        /// 要素を出た: 猶予の間だけ残し、猶予が尽きても card に入っていなければ消す。
        fn leave(self, from: Point) {
            if self.card.with_untracked(Option::is_none) {
                return;
            }
            self.leaving.set_value(Some(Leaving {
                from,
                at_ms: Date::now(),
            }));
            let seq = self.seq.get_value();
            set_timeout(
                move || {
                    if self.seq.get_value() == seq && self.leaving.get_value().is_some() {
                        self.hide();
                    }
                },
                Duration::from_millis(u64::from(GRACE_MS)),
            );
        }

        /// 猶予の間に pointer が動いた: card へ近づいていなければ消す。
        fn moved(self, now: Point) {
            let Some(l) = self.leaving.get_value() else {
                return;
            };
            let Some(rect) = self.rect() else {
                return;
            };
            if !keeps(l.from, now, rect, Date::now() - l.at_ms) {
                self.hide();
            }
        }

        /// card に入った（猶予を止めて残す）。
        fn hold(self) {
            self.leaving.set_value(None);
        }

        /// 外した要素の後始末: その要素の出した card がまだ出ていれば閉じる（頁を閉じて層が先に捨てられていれば何もしない）。
        fn release(self, shown: u64) {
            if self
                .seq
                .try_get_value()
                .is_some_and(|now| unmount_hides(shown, now))
            {
                self.hide();
            }
        }

        fn hide(self) {
            self.seq.update_value(|n| *n += 1);
            self.leaving.set_value(None);
            self.card.set(None);
            self.at.set(None);
        }
    }

    /// 要素に card を付ける（指を置くと出し、出たら猶予の判定で残すか消す）。
    /// block は要素を描いた後にこれを呼ぶだけ（`use:attach=card` の directive の形でも使える）。
    /// DOM を組み直して要素を外すと出の事件が来ないので、要素の反応の範囲が捨てられるときに、その要素の出した card を閉じる。
    pub fn attach(el: web_sys::Element, card: Card) {
        let Some(ctx) = use_context::<HoverCtx>() else {
            return;
        };
        // 要素が最後に出した card の番号（0 はまだ出していない・後始末の閉包と listener が分け合う）。
        let shown = Arc::new(AtomicU64::new(0));
        let mine = Arc::clone(&shown);
        let enter =
            Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |ev: web_sys::PointerEvent| {
                ctx.show(card.clone(), point(&ev));
                mine.store(ctx.seq.get_value(), Ordering::Relaxed);
            });
        let leave =
            Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |ev: web_sys::PointerEvent| {
                ctx.leave(point(&ev));
            });
        let _ = el.add_event_listener_with_callback("pointerenter", enter.as_ref().unchecked_ref());
        let _ = el.add_event_listener_with_callback("pointerleave", leave.as_ref().unchecked_ref());
        // 要素の一生の間ずっと持つ（要素を外すと listener も届かなくなる）。
        enter.forget();
        leave.forget();
        on_cleanup(move || ctx.release(shown.load(Ordering::Relaxed)));
    }

    /// card が在れば要素に付ける（`use:attach_some=card` の directive の形で使う・節点が引けなければ card を出さない）。
    pub fn attach_some(el: web_sys::Element, card: Option<Card>) {
        if let Some(card) = card {
            attach(el, card);
        }
    }

    /// 委ねの口（字の SVG のように要素に `attach` を付けられない面が、事件の点で card を出し消す）。
    /// 置き場と猶予は `attach` と同じ層の show と leave を通る（規則の行 R-20）。
    #[derive(Clone, Copy)]
    pub struct Delegate {
        ctx: Option<HoverCtx>,
    }

    /// 頁の card の層を引いた委ねの口（描くときに呼ぶ・層が無ければ何もしない口）。
    pub fn delegate() -> Delegate {
        Delegate {
            ctx: use_context::<HoverCtx>(),
        }
    }

    impl Delegate {
        /// 指が入った: 出している card と同じなら出し直さず猶予を止め、違えば card を出す。
        pub fn show(self, ev: &web_sys::MouseEvent, card: Card) {
            let Some(ctx) = self.ctx else {
                return;
            };
            if ctx.card.with_untracked(|c| over_shows(c.as_ref(), &card)) {
                ctx.show(card, point(ev));
            } else {
                ctx.hold();
            }
        }

        /// 指が出た: 猶予に入る。
        pub fn leave(self, ev: &web_sys::MouseEvent) {
            if let Some(ctx) = self.ctx {
                ctx.leave(point(ev));
            }
        }
    }

    /// card の 4 行と「詳しく」。
    fn card_view(card: &Card) -> AnyView {
        let rows = card
            .rows()
            .into_iter()
            .map(|(class, text)| view! { <div class=class>{text}</div> })
            .collect_view();
        let more = (!card.more.is_empty()).then(|| {
            let lines = card
                .more
                .iter()
                .map(|m| view! { <div class="r2">{m.clone()}</div> })
                .collect_view();
            view! {
                <details class="more">
                    <summary>{format!("{} ▸", label("p_more"))}</summary>
                    {lines}
                </details>
            }
        });
        view! { {rows}{more} }.into_any()
    }

    /// card の層（body に 1 つ・pointer が動くたびに猶予の判定をする）。
    #[component]
    pub fn CardLayer() -> impl IntoView {
        let Some(c) = use_context::<HoverCtx>() else {
            return ().into_any();
        };
        let moved = window_event_listener(ev::pointermove, move |ev| c.moved(point(&ev)));
        on_cleanup(move || moved.remove());
        let class = move || card_class(c.card.with(Option::is_some) && c.at.with(Option::is_some));
        // 置くまでは窓の左上で描いて大きさを測る（`on` が無い間は見えない）。
        let style = move || {
            let p = c.at.get().unwrap_or(Point { x: 0.0, y: 0.0 });
            format!("left:{}px;top:{}px", p.x, p.y)
        };
        let content = move || c.card.with(|card| card.as_ref().map(card_view));
        view! {
            <div
                node_ref=c.node
                class=class
                role="tooltip"
                style=style
                on:pointerenter=move |_| c.hold()
                on:pointerleave=move |_| c.hide()
            >
                {content}
            </div>
        }
        .into_any()
    }
}
