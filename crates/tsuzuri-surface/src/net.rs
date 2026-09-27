//! 通信（wasm の target のときだけ組み立てる・便 g-parts）: block ごとの読みの口を読み、変化の知らせ（SSE）で読み直す。
//! block は自分の口の path を `read` に渡し、読みの結果（3 値）の signal を受ける。登録した口は頁を開いたときに 1 回読む。
//! 知らせの接続は頁に 1 本だけ持ち、開いた・読み直しの合図の event を受けたで登録された口を全部読み直す。
//! 読みの後は view の決め方で置くかを決める（同じ本文は置かない・一度の読めないは 1 秒後に読み直す・便 g-steady）。
//! 知らせが切れている間は、今の中身が正しいと言えないので登録された口を全部「読めない」にする（要件 NFR2・決め方を通さない）。
//! 書きの口へは本文つきの POST を送り、状態の数と本文の字を返す（便 g-ask）。
//! path が query で変わる口（節点の近傍・便 g-node）は `read_path` に path の字の signal を渡し、
//! 読みの結果と応答の状態の数の組を受ける。path が変わったときと知らせの合図で読み直し、接続は同じ 1 本を使う。

use std::cell::{Cell, RefCell};
use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use tsuzuri_contract::EpochSecs;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::Date;
use web_sys::wasm_bindgen::closure::Closure;
use web_sys::wasm_bindgen::{JsCast, JsValue};
use web_sys::{EventSource, Headers, Request, RequestInit, Response};

use crate::view::{Fetched, RELOAD_EVENTS, RETRY_MS, Settle, settle};

/// 変化の知らせの口（SSE・server の便 e-min）。
pub const EVENTS_PATH: &str = "/api/surface/events";

thread_local! {
    /// 登録された口（path と読みの結果）。同じ path は 1 つの signal を分ける。
    static READS: RefCell<Vec<(&'static str, ArcRwSignal<Fetched>)>> = const { RefCell::new(Vec::new()) };
    /// path が変わる口（今の path と、読みの結果と応答の状態の数の組）。頁の一生の間だけ持つ。
    static WATCHES: RefCell<Vec<(String, ArcRwSignal<Status>)>> = const { RefCell::new(Vec::new()) };
    /// 知らせの接続を張ったか（頁に 1 本だけ）。
    static CONNECTED: Cell<bool> = const { Cell::new(false) };
}

/// 読みの結果と応答の状態の数（応答が無ければ None）の組。
type Status = (Fetched, Option<u16>);

/// 今の時刻（epoch 秒）。
pub fn now() -> EpochSecs {
    (Date::now() / 1000.0) as EpochSecs
}

/// 口を読む（届かない・200 でない・本文が字でなければ Failed）。
async fn fetch(path: &str) -> Fetched {
    fetch_status(path).await.0
}

/// 口を読み、応答の状態の数も返す（応答が無ければ状態の数は None）。
async fn fetch_status(path: &str) -> Status {
    let Some(window) = web_sys::window() else {
        return (Fetched::Failed, None);
    };
    let Ok(value) = JsFuture::from(window.fetch_with_str(path)).await else {
        return (Fetched::Failed, None);
    };
    let Ok(response) = value.dyn_into::<Response>() else {
        return (Fetched::Failed, None);
    };
    let status = Some(response.status());
    if !response.ok() {
        return (Fetched::Failed, status);
    }
    let Ok(text) = response.text() else {
        return (Fetched::Failed, status);
    };
    match JsFuture::from(text).await.ok().and_then(|t| t.as_string()) {
        Some(body) => (Fetched::Body(body), status),
        None => (Fetched::Failed, status),
    }
}

/// 本文つきの POST を送り、状態の数と本文の字を返す（届かない・応答が読めなければ None）。
/// 本文は引数の字だけを送り、ほかの所に書かない。
pub async fn post(path: &str, body: String) -> Option<(u16, String)> {
    let window = web_sys::window()?;
    let init = RequestInit::new();
    init.set_method("POST");
    init.set_body(&JsValue::from_str(&body));
    let headers = Headers::new().ok()?;
    headers.set("Content-Type", "application/json").ok()?;
    init.set_headers(&headers);
    let request = Request::new_with_str_and_init(path, &init).ok()?;
    let value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .ok()?;
    let response = value.dyn_into::<Response>().ok()?;
    let text = JsFuture::from(response.text().ok()?)
        .await
        .ok()?
        .as_string()?;
    Some((response.status(), text))
}

/// 1 つの口を読み、置くかを決め方（view の settle）で決めて signal に置く。
fn load(path: &'static str, signal: ArcRwSignal<Fetched>) {
    load_at(path, signal, 1);
}

/// `attempt` 回目の読み（同じ本文は置かない・一度の読めないは 1 秒後に読み直す）。
fn load_at(path: &'static str, signal: ArcRwSignal<Fetched>, attempt: u32) {
    spawn_local(async move {
        let fetched = fetch(path).await;
        match signal.with_untracked(|now| settle(now, &fetched, attempt)) {
            Settle::Set => signal.set(fetched),
            Settle::Keep => {}
            Settle::Retry => set_timeout(
                move || load_at(path, signal, attempt + 1),
                Duration::from_millis(RETRY_MS),
            ),
        }
    });
}

/// 登録された口の全部。
fn registered() -> Vec<(&'static str, ArcRwSignal<Fetched>)> {
    READS.with_borrow(|reads| reads.clone())
}

/// path が変わる口の `slot` 番目を `attempt` 回目に読む（読む間に path が変われば置かない・決め方は `load_at` と同じ）。
fn load_watch(slot: usize, attempt: u32) {
    let Some((path, signal)) = WATCHES.with_borrow(|w| w.get(slot).cloned()) else {
        return;
    };
    if path.is_empty() {
        return;
    }
    spawn_local(async move {
        let got = fetch_status(&path).await;
        let moved = WATCHES.with_borrow(|w| w.get(slot).is_none_or(|(p, _)| *p != path));
        if moved || signal.with_untracked(|now| *now == got) {
            return;
        }
        match signal.with_untracked(|now| settle(&now.0, &got.0, attempt)) {
            Settle::Retry => set_timeout(
                move || load_watch(slot, attempt + 1),
                Duration::from_millis(RETRY_MS),
            ),
            Settle::Set | Settle::Keep => signal.set(got),
        }
    });
}

/// 登録された口を全部読み直す（書きの口へ送った後に block も呼ぶ）。
pub fn reload_all() {
    for (path, signal) in registered() {
        load(path, signal);
    }
    for slot in 0..WATCHES.with_borrow(Vec::len) {
        load_watch(slot, 1);
    }
}

/// 登録された口を全部「読めない」にする。
fn lose_all() {
    for (_, signal) in registered() {
        signal.set(Fetched::Failed);
    }
    for (_, signal) in WATCHES.with_borrow(|w| w.clone()) {
        signal.set((Fetched::Failed, None));
    }
}

/// path が変わる口の読みの組の signal（path が変わるたびに「まだ読んでいない」に戻して読み直す・
/// 知らせの合図でも読み直す・知らせの接続は `read` と同じ 1 本）。
pub fn read_path(path: Signal<String>) -> ReadSignal<(Fetched, Option<u16>)> {
    let signal = ArcRwSignal::new((Fetched::NotRead, None));
    let slot = WATCHES.with_borrow_mut(|w| {
        w.push((String::new(), signal.clone()));
        w.len() - 1
    });
    let out = signal.clone();
    Effect::new(move |before: Option<String>| {
        let now = path.get();
        if before.as_ref() != Some(&now) {
            WATCHES.with_borrow_mut(|w| {
                if let Some(entry) = w.get_mut(slot) {
                    entry.0.clone_from(&now);
                }
            });
            out.set((Fetched::NotRead, None));
            load_watch(slot, 1);
        }
        now
    });
    connect();
    ReadSignal::from(signal.read_only())
}

/// 口の読みの結果の signal（初めての path は登録して 1 回読む・知らせの接続がまだ無ければ張る）。
pub fn read(path: &'static str) -> ReadSignal<Fetched> {
    let found = READS.with_borrow(|reads| {
        reads
            .iter()
            .find(|(p, _)| *p == path)
            .map(|(_, s)| s.clone())
    });
    let signal = match found {
        Some(signal) => signal,
        None => {
            let signal = ArcRwSignal::new(Fetched::NotRead);
            READS.with_borrow_mut(|reads| reads.push((path, signal.clone())));
            load(path, signal.clone());
            signal
        }
    };
    connect();
    ReadSignal::from(signal.read_only())
}

/// 知らせの接続を張る（頁に 1 本だけ・開いた・合図で読み直し・切れたら全部「読めない」）。
/// 接続は頁の一生の間ずっと持つ（切れても EventSource が繋ぎ直し、開いたで読み直す）。
fn connect() {
    if CONNECTED.replace(true) {
        return;
    }
    let Ok(source) = EventSource::new(EVENTS_PATH) else {
        lose_all();
        return;
    };
    let on_open = Closure::<dyn FnMut()>::new(reload_all);
    let on_change = Closure::<dyn FnMut()>::new(reload_all);
    let on_error = Closure::<dyn FnMut()>::new(lose_all);
    source.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    source.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    for name in RELOAD_EVENTS {
        if source
            .add_event_listener_with_callback(name, on_change.as_ref().unchecked_ref())
            .is_err()
        {
            lose_all();
        }
    }
    on_open.forget();
    on_change.forget();
    on_error.forget();
    std::mem::forget(source);
}
