//! 窓の枠（行 g-win-frame・判断の記録 ADR-27 決定 (4)・見本 board-v2 の openModal と drawModal と closeModal と topModal）:
//! 帯の印や窓の中の口を押すと開く窓を、頁に 1 枚ずつ出す。窓は × と外の click と取り消しの鍵（Esc）で閉じる。
//! 窓の中の口から別の窓を開くと前の窓を積んで残し、頭に前の窓へ戻る口（‹ 戻る）を出す。
//! 幅 600 px 以下（規則の行 R-35 のスマホの形）の窓は全画面（stylesheet の media の規則）。
//! 閉じる判定（`closes`）と窓の積み（`Stack`）と幅の字は純粋な関数にして host で試し、層の DOM（`layer`）は wasm の target だけ。
//! 吹き出し（行 g-pop）も同じ閉じる判定を使う。窓の中身は窓を足す行が描き、層を頁に置くのは 1 枚の画面への切り替えの行。

/// 窓と吹き出しに届いた押し（× の button・外の click・中の click・鍵）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit<'a> {
    /// × の button。
    X,
    /// 窓か吹き出しの外の click（窓は幕そのものの click）。
    Outside,
    /// 窓か吹き出しの中の click。
    Inside,
    /// 鍵（KeyboardEvent の key の字と、仮名漢字の変換の途中か）。
    Key { key: &'a str, composing: bool },
}

/// 取り消しの鍵の字（KeyboardEvent の key）。
pub const ESC: &str = "Escape";

/// 押しで閉じるか: × と外の click と取り消しの鍵で閉じ、中の click とほかの鍵では閉じない。
/// 変換の途中の取り消しの鍵は変換を取り消す鍵なので閉じない（答えの欄の書きかけを消さない）。
pub fn closes(hit: Hit<'_>) -> bool {
    match hit {
        Hit::X | Hit::Outside => true,
        Hit::Inside => false,
        Hit::Key { key, composing } => key == ESC && !composing,
    }
}

/// 開いている窓の積み（見本の MS・頂が見えている窓・空なら窓は閉じている）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack<W> {
    wins: Vec<W>,
}

impl<W> Default for Stack<W> {
    fn default() -> Self {
        Self { wins: Vec::new() }
    }
}

impl<W: Copy> Stack<W> {
    /// 窓を開く: 窓の中の口から開くとき（`from_win`）は前の窓を残して積み、ほかは前の窓を全部閉じて開く。
    pub fn open(&mut self, win: W, from_win: bool) {
        if !from_win {
            self.wins.clear();
        }
        self.wins.push(win);
    }

    /// 前の窓に戻る（頂を外す・残りが無ければ窓は閉じる）。
    pub fn back(&mut self) {
        self.wins.pop();
    }

    /// 窓を全部閉じる。
    pub fn close(&mut self) {
        self.wins.clear();
    }

    /// 見えている窓（閉じていれば None）。
    pub fn top(&self) -> Option<W> {
        self.wins.last().copied()
    }

    /// 戻る口を出すか（積みが 2 枚以上）。
    pub fn can_back(&self) -> bool {
        self.wins.len() > 1
    }
}

/// 窓の幅の style（見本の drawModal の `min(<幅>, 94vw)`）。
pub fn width_style(px: u32) -> String {
    format!("width:min({px}px, 94vw)")
}

/// 窓の全画面の幅の上限（px・規則の行 R-35 のスマホの形・stylesheet の media の値）。
pub const FULL_MAX_PX: u32 = 600;

/// 幕の id（見本の `#scrim`・頁に 1 つ）。
pub const SCRIM: &str = "scrim";

/// 窓の箱・頭・本文・戻る口・× の class（見本の `.modal`・`.mh`・`.mb`・`.back`・`.x`）。
pub const CLASSES: [&str; 5] = ["modal", "mh", "mb", "back", "x"];

/// × の button の語の鍵（aria-label）。
pub const CLOSE_KEY: &str = "mclose";

/// 戻る口の語の鍵。
pub const BACK_KEY: &str = "mback";

/// 戻る口の頭の印（見本の `‹ 戻る`）。
pub const BACK_MARK: &str = "‹";

#[cfg(target_arch = "wasm32")]
pub use dom::{Frame, WinCtx, layer};

#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::ev;
    use leptos::prelude::*;

    use super::{BACK_KEY, BACK_MARK, CLASSES, CLOSE_KEY, Hit, SCRIM, Stack, closes, width_style};
    use crate::vocab::label;

    /// 窓の 1 枚の中身（幅 px・題・本文）。
    pub struct Frame {
        pub width: u32,
        pub title: AnyView,
        pub body: AnyView,
    }

    /// 窓の積みの状態（頁に 1 つ・帯と窓の中の口が open を呼ぶ）。
    pub struct WinCtx<W: Send + Sync + 'static> {
        stack: RwSignal<Stack<W>>,
    }

    impl<W: Send + Sync + 'static> Clone for WinCtx<W> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<W: Send + Sync + 'static> Copy for WinCtx<W> {}

    impl<W: Copy + Send + Sync + 'static> Default for WinCtx<W> {
        fn default() -> Self {
            Self {
                stack: RwSignal::new(Stack::default()),
            }
        }
    }

    impl<W: Copy + Send + Sync + 'static> WinCtx<W> {
        /// 窓を開く（`from_win` は窓の中の口から開くとき）。
        pub fn open(self, win: W, from_win: bool) {
            self.stack.update(|s| s.open(win, from_win));
        }

        /// 前の窓に戻る。
        pub fn back(self) {
            self.stack.update(Stack::back);
        }

        /// 窓を全部閉じる。
        pub fn close(self) {
            self.stack.update(Stack::close);
        }

        /// 見えている窓（追う）。
        pub fn top(self) -> Option<W> {
            self.stack.with(Stack::top)
        }

        fn open_now(self) -> bool {
            self.stack.with_untracked(|s| s.top().is_some())
        }
    }

    /// 押しで閉じるなら窓を全部閉じる。
    fn hit<W: Copy + Send + Sync + 'static>(ctx: WinCtx<W>, h: Hit<'_>) {
        if closes(h) {
            ctx.close();
        }
    }

    /// 幕の click が幕そのものか（窓の中の click は幕へ上がっても外でない）。
    fn outside(e: &ev::MouseEvent) -> bool {
        e.target().is_some() && e.target() == e.current_target()
    }

    /// 窓の層（body に 1 つ・頂の窓だけを描く・× と幕の click と取り消しの鍵で閉じる）。
    /// `draw` は窓の名から中身を組む（窓を足す行が持つ）。
    pub fn layer<W, F>(ctx: WinCtx<W>, draw: F) -> AnyView
    where
        W: Copy + Send + Sync + 'static,
        F: Fn(W) -> Frame + Send + Sync + 'static,
    {
        let keys = window_event_listener(ev::keydown, move |e| {
            if ctx.open_now() {
                let key = e.key();
                hit(
                    ctx,
                    Hit::Key {
                        key: &key,
                        composing: e.is_composing(),
                    },
                );
            }
        });
        on_cleanup(move || keys.remove());
        let [boxed, head, main, back_class, x] = CLASSES;
        let shown = move || {
            let (win, can_back) = ctx.stack.with(|s| (s.top(), s.can_back()));
            let f = draw(win?);
            let back = can_back.then(|| {
                view! {
                    <button type="button" class=back_class on:click=move |_| ctx.back()>
                        {format!("{BACK_MARK} {}", label(BACK_KEY))}
                    </button>
                }
            });
            Some(view! {
                <div id=SCRIM on:click=move |e: ev::MouseEvent| hit(ctx, if outside(&e) { Hit::Outside } else { Hit::Inside })>
                    <div class=boxed role="dialog" aria-modal="true" style=width_style(f.width)>
                        <div class=head>
                            {back}
                            <h3>{f.title}</h3>
                            <button type="button" class=x aria-label=label(CLOSE_KEY) on:click=move |_| hit(ctx, Hit::X)>"×"</button>
                        </div>
                        <div class=main>{f.body}</div>
                    </div>
                </div>
            })
        };
        view! { {shown} }.into_any()
    }
}
