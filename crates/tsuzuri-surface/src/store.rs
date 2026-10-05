//! browser の保存（見本 mock v3 の ui.js の store と同じ口）。
//! 要件 FR1: mode と account board から開いた窓の一覧と最初の案内の済み印の 3 つを
//! その browser に残す。これは便利のための写しで正本ではなく、保存が消えても面は URL と server の読みから同じ画面を組む。
//! 決め方（mode_start）は host でも組み立てて試し、保存と URL を撃つ所は wasm の target のときだけ組み立てる。
//! 保存は origin ごとなので、port の違う account board と各 project board はそれぞれ自分の写しを持つ。
//! on_change は同じ origin のほかの tab が鍵の値を書き替えたときに撃つ（窓の一覧の置き直し）。
//! 個別の頁の本文と記録の 2 つの畳める段の開き閉じも、段の種類ごとの鍵でその browser に残す（節点ごとでない・
//! 判断の記録 ADR-30 決定 (4)）。ほかの畳める段は頁の一生の間だけ持つ。

use crate::frame::{self, Mode};

/// mode の保存の鍵（見本の鍵と同じ・2 つの board で同じ）。
pub const MODE_KEY: &str = "tz-mode";

/// 頁を開くときに決めた mode と、保存に書く字と、履歴を置き換える query の字。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeStart {
    pub mode: Mode,
    /// 保存に書く字（URL の mode が beginner か expert のときだけ）。
    pub keep: Option<&'static str>,
    /// 履歴を置き換える query の字（URL に mode が無く、保存が expert のときだけ）。
    pub url: Option<String>,
}

/// 見本の initMode と同じ決め方: URL の mode が beginner か expert ならそれが先で保存に写し、
/// そうでなければ保存の値が expert のときだけ経験者（ほかは初心者）。
/// 見本との違いは 1 つで、経験者に決めたときは URL の mode を expert に揃える
/// （頁と戻ると進むは URL から mode を読むので、URL と保存の決めが食い違わない）。
pub fn mode_start(query: &str, saved: Option<&str>) -> ModeStart {
    let from_url = Mode::ALL
        .into_iter()
        .find(|m| frame::param(query, "mode") == Some(m.key()));
    if let Some(mode) = from_url {
        return ModeStart {
            mode,
            keep: Some(mode.key()),
            url: None,
        };
    }
    let mode = if saved == Some(Mode::Expert.key()) {
        Mode::Expert
    } else {
        Mode::Beginner
    };
    let url =
        (mode == Mode::Expert).then(|| frame::with_param(query, "mode", Mode::Expert.key()));
    ModeStart {
        mode,
        keep: None,
        url,
    }
}

/// 保存に残す畳める段の鍵と、その保存の鍵（個別の頁の本文と記録・段の種類ごとに 1 つ・行 g-fold-keep）。
pub const FOLD_KEYS: [(&str, &str); 2] = [
    ("node:body", "tz-node-body"),
    ("node:notes", "tz-node-notes"),
];

/// 開いた段の保存の字。
pub const FOLD_OPEN: &str = "open";

/// 閉じた段の保存の字。
pub const FOLD_SHUT: &str = "shut";

/// 畳める段の鍵の保存の鍵（`FOLD_KEYS` に無い鍵は None で、保存に書かない）。
pub fn fold_store_key(fold: &str) -> Option<&'static str> {
    FOLD_KEYS.iter().find(|(f, _)| *f == fold).map(|(_, k)| *k)
}

/// 保存の字から決める段の初めの開き閉じ（`FOLD_OPEN` だけ開く・保存が無い・読めない・ほかの字は閉じて始める）。
pub fn fold_start(saved: Option<&str>) -> bool {
    saved == Some(FOLD_OPEN)
}

/// 段の開き閉じを保存に書く字。
pub fn fold_text(open: bool) -> &'static str {
    if open { FOLD_OPEN } else { FOLD_SHUT }
}

/// 保存の口（window が無い・使えないときは None）。
#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

/// 保存の値を読む（無い・使えない・読めないときは None）。
#[cfg(target_arch = "wasm32")]
pub fn get(key: &str) -> Option<String> {
    storage()?.get_item(key).ok().flatten()
}

/// 保存に値を書く（使えない・書けないときは何もしない）。
#[cfg(target_arch = "wasm32")]
pub fn set(key: &str, value: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item(key, value);
    }
}

/// 保存の鍵 `key` の値がほかの tab で書き替わるたびに `f` を撃つ（storage の event は同じ origin のほかの tab の書きだけが届く）。
#[cfg(target_arch = "wasm32")]
pub fn on_change(key: &'static str, f: impl Fn() + 'static) {
    let _ = leptos::prelude::window_event_listener(leptos::ev::storage, move |e| {
        if e.key().as_deref() == Some(key) {
            f();
        }
    });
}

/// 頁を開くときに mode を決める: URL の mode を保存に写すか、保存の経験者を URL に揃える（頁は読み直さない）。
#[cfg(target_arch = "wasm32")]
pub fn settle_mode() {
    let Some(win) = web_sys::window() else {
        return;
    };
    let query = win.location().search().unwrap_or_default();
    let start = mode_start(&query, get(MODE_KEY).as_deref());
    if let Some(keep) = start.keep {
        set(MODE_KEY, keep);
    }
    if let Some(url) = start.url
        && let Ok(history) = win.history()
    {
        let _ = history.replace_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, "", Some(&url));
    }
}

/// 押しで選んだ mode を保存に写す。
#[cfg(target_arch = "wasm32")]
pub fn keep_mode(mode: Mode) {
    set(MODE_KEY, mode.key());
}

/// 畳める段の初めの開き閉じを保存から読む（`FOLD_KEYS` に無い鍵と、保存が無い・読めない時は閉じ）。
#[cfg(target_arch = "wasm32")]
pub fn fold_open(fold: &str) -> bool {
    fold_store_key(fold).is_some_and(|k| fold_start(get(k).as_deref()))
}

/// 畳める段の開き閉じを保存に写す（`FOLD_KEYS` に無い鍵は書かない）。
#[cfg(target_arch = "wasm32")]
pub fn keep_fold(fold: &str, open: bool) {
    if let Some(k) = fold_store_key(fold) {
        set(k, fold_text(open));
    }
}
