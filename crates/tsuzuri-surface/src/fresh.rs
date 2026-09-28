//! 中身の古さ（行 g-fresh・規則の行 R-21・要件 NFR2）: 台帳の読みが落ちたときと知らせのつながりが切れたときは、
//! 最後に読めた中身を 60 秒まで出し続け、15 秒を越えれば上端の帯に読み込み不良の印を 1 つ出す（60 秒を越えた中身は
//! server が 測れていない の電文にし、つながりの切れは net が全部の口を「読めない」にする）。
//! 古さは server が読みの落ちた応答に載せる頭（最後に読めた時からの秒）と、面の時計の経過で数える。
//! 1 つでも口が読みの途中なら最終の記録の横に短い脈を出し、読み終えてから PULSE_MS まで残す（行 g-pulse）。
//! 決め方は host でも組んで試し、印と脈の DOM（`mark`・`pulse`）は wasm の target のときだけ組む。

use std::collections::BTreeMap;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::ledger::{READ_AGE_HEADER, READ_HOLD_S, READ_WARN_S};

use crate::widgets::hover::Card;

/// 台帳の読みが落ちている間に読み直しを撃つ間（ミリ秒・見張りが戻りを知らせない場合の拾い）。
pub const HELD_POLL_MS: u64 = 5000;

/// 印の語（card の題と aria-label）。
pub const WARN_WORD: &str = "読み込み不良";

/// 古さの起点が台帳の読みの落ちのときの種類の語。
pub const HELD_WORD: &str = "台帳の読み";

/// 古さの起点が知らせのつながりの切れのときの種類の語。
pub const LOST_WORD: &str = "知らせのつながり";

/// つながりの切れの出所（変化の知らせの口）。
pub const LOST_SRC: &str = "SSE";

/// card の種類と出所の語をつなぐ字。
const JOIN: &str = "・";

/// 頭の値の字を秒にする（前後の空白を除いた 10 進の数字だけの字・ほかは None）。
pub fn read_age(value: Option<&str>) -> Option<u64> {
    let text = value?.trim();
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// 中身の古さの状態（頁に 1 つ・net が応答とつながりの切れのたびに進める）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fresh {
    /// 頭の無い応答を最後に受けた時刻。
    seen: Option<EpochSecs>,
    /// 頭の在る口の path から最後に読めた時刻。
    held: BTreeMap<String, EpochSecs>,
    /// つながりが切れた時刻。
    lost: Option<EpochSecs>,
}

impl Fresh {
    /// 口 `path` の応答を時刻 `at` に受けた（`age` は頭の秒・頭が無ければ None）。
    pub fn got(&mut self, path: &str, at: EpochSecs, age: Option<u64>) {
        match age {
            None => {
                self.held.remove(path);
                self.seen = self.seen.max(Some(at));
            }
            Some(secs) => {
                self.held.insert(path.to_string(), at.saturating_sub(secs));
            }
        }
    }

    /// 最終の記録（読みの落ちた口の最も古い最後に読めた時刻・無ければ頭の無い応答を最後に受けた時刻）。
    pub fn record(&self) -> Option<EpochSecs> {
        self.held.values().min().copied().or(self.seen)
    }

    /// 中身の古さ（読みの落ちとつながりの切れの古い方から今まで・どちらも無ければ None）。
    pub fn age(&self, now: EpochSecs) -> Option<u64> {
        let since = self.held.values().copied().chain(self.lost).min()?;
        Some(now.saturating_sub(since))
    }

    /// 印を出すか（古さが READ_WARN_S を越えたとき）。
    pub fn warn(&self, now: EpochSecs) -> bool {
        self.age(now).is_some_and(|a| a > READ_WARN_S)
    }

    /// 読みの落ちた口が在るか。
    pub fn held(&self) -> bool {
        !self.held.is_empty()
    }

    /// つながりが切れた（切れた最初の誤りなら真・切れている間の誤りは時刻を動かさない）。
    pub fn lose(&mut self, at: EpochSecs) -> bool {
        if self.lost.is_some() {
            return false;
        }
        self.lost = Some(at);
        true
    }

    /// つながりが開いた。
    pub fn back(&mut self) {
        self.lost = None;
    }

    /// つながりが切れた時刻。
    pub fn lost_since(&self) -> Option<EpochSecs> {
        self.lost
    }

    /// 見ていない口の古さを数えない。
    pub fn forget(&mut self, path: &str) {
        self.held.remove(path);
    }

    /// 印の card（印を出さない間は None）。
    pub fn card(&self, now: EpochSecs) -> Option<Card> {
        if !self.warn(now) {
            return None;
        }
        let age = self.age(now)?;
        let mut kinds = Vec::new();
        let mut srcs = Vec::new();
        if self.held() {
            kinds.push(HELD_WORD);
            srcs.push(READ_AGE_HEADER);
        }
        if self.lost.is_some() {
            kinds.push(LOST_WORD);
            srcs.push(LOST_SRC);
        }
        Some(Card {
            title: WARN_WORD.to_string(),
            kind: kinds.join(JOIN),
            value: format!("中身の古さ {age} 秒・{READ_HOLD_S} 秒で測れていない"),
            src: srcs.join(JOIN),
            more: Vec::new(),
        })
    }
}

/// 読みの脈を読みの後に残す間（ミリ秒・見本の beat の 800 ミリ秒・行 g-pulse）。
pub const PULSE_MS: u64 = 800;

/// 読みの脈の class（出す間は動いているの記号と短い輪・出さない間も同じ幅の枠）。
pub fn pulse_class(on: bool) -> &'static str {
    if on { "st st-run beat" } else { "st" }
}

/// 読みの脈の状態（読みの途中の口の数と、最後に 0 になった時刻〔ミリ秒〕）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pulse {
    /// 読みの途中の口の数。
    busy: usize,
    /// 読みの途中の口が最後に 0 になった時刻。
    ended: Option<u64>,
}

impl Pulse {
    /// 読みの途中の口の数 `busy` を時刻 `at` に受けた（1 以上から 0 になった時だけ終わりの時刻を置く）。
    pub fn set(&mut self, busy: usize, at: u64) {
        if self.busy > 0 && busy == 0 {
            self.ended = Some(at);
        }
        self.busy = busy;
    }

    /// 脈を出すか（読みの途中か、最後に 0 になってから PULSE_MS より前）。
    pub fn on(&self, now: u64) -> bool {
        self.busy > 0 || self.ended.is_some_and(|e| now.saturating_sub(e) < PULSE_MS)
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::{mark, pulse};

#[cfg(target_arch = "wasm32")]
mod dom {
    use std::time::Duration;

    use leptos::ev;
    use leptos::prelude::*;
    use web_sys::js_sys::Date;

    use super::{PULSE_MS, Pulse, WARN_WORD, pulse_class};
    use crate::widgets::hover::delegate;

    /// 印（見本の IC.warn）。
    const WARN: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M12 3l10 18H2z"/><path d="M12 10v5"/><circle cx="12" cy="18" r=".8" fill="currentColor"/></svg>"#;

    /// 読み込み不良の印（古さが READ_WARN_S を越えている間だけ出す・指を置くと card を出す）。
    pub fn mark() -> AnyView {
        let fresh = crate::net::fresh();
        let tick = crate::net::ticker();
        let on = Memo::new(move |_| fresh.with(|f| f.warn(tick.get())));
        let hc = delegate();
        let shown = move || {
            on.get().then(|| {
                let over = move |ev: ev::MouseEvent| {
                    if let Some(card) = fresh.with_untracked(|f| f.card(crate::net::now())) {
                        hc.show(&ev, card);
                    }
                };
                let out = move |ev: ev::MouseEvent| hc.leave(&ev);
                view! {
                    <span class="warn" role="status" tabindex="0" aria-label=WARN_WORD on:mouseenter=over on:mouseleave=out>
                        <span inner_html=WARN></span>
                        {WARN_WORD}
                    </span>
                }
            })
        };
        shown.into_any()
    }

    /// 読みの脈（1 つでも口が読みの途中なら最終の記録の横に短い輪を出し、読み終えてから PULSE_MS まで残す・
    /// 飾りなので読み上げない・出さない間も同じ幅を持つ）。
    pub fn pulse() -> AnyView {
        let busy = crate::net::busy();
        let state = RwSignal::new(Pulse::default());
        let clock = RwSignal::new(Date::now() as u64);
        Effect::new(move |_| {
            let n = busy.get();
            let now = Date::now() as u64;
            state.update(|p| p.set(n, now));
            clock.set(now);
            if n == 0 {
                // 脈を消す組み直し（時計が戻っても 0 になった時刻から PULSE_MS より前に置かない）。
                let end = now + PULSE_MS;
                set_timeout(
                    move || clock.set((Date::now() as u64).max(end)),
                    Duration::from_millis(PULSE_MS),
                );
            }
        });
        let class = move || pulse_class(state.with(|p| p.on(clock.get())));
        view! { <span class=class aria-hidden="true"><i></i></span> }.into_any()
    }
}
