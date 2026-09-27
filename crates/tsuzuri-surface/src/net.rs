//! 通信（wasm の target のときだけ組み立てる・便 g-parts）: block ごとの読みの口を読み、変化の知らせ（SSE）で読み直す。
//! block は自分の口の path を `read` に渡し、読みの結果（3 値）の signal を受ける。登録した口は頁を開いたときに 1 回読む。
//! 知らせの接続は頁に 1 本だけ持ち、開いた・読み直しの合図の event を受けたで登録された口を全部読み直す。
//! 知らせが切れている間は、今の中身が正しいと言えないので登録された口を全部「読めない」にする（要件 NFR2）。
//! 書きの口へは本文つきの POST を送り、状態の数と本文の字を返す（便 g-ask）。

use std::cell::{Cell, RefCell};

use leptos::prelude::*;
use leptos::task::spawn_local;
use tsuzuri_contract::EpochSecs;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::Date;
use web_sys::wasm_bindgen::closure::Closure;
use web_sys::wasm_bindgen::{JsCast, JsValue};
use web_sys::{EventSource, Headers, Request, RequestInit, Response};

use crate::view::{Fetched, RELOAD_EVENTS};

/// 変化の知らせの口（SSE・server の便 e-min）。
pub const EVENTS_PATH: &str = "/api/surface/events";

thread_local! {
    /// 登録された口（path と読みの結果）。同じ path は 1 つの signal を分ける。
    static READS: RefCell<Vec<(&'static str, ArcRwSignal<Fetched>)>> = const { RefCell::new(Vec::new()) };
    /// 知らせの接続を張ったか（頁に 1 本だけ）。
    static CONNECTED: Cell<bool> = const { Cell::new(false) };
}

/// 今の時刻（epoch 秒）。
pub fn now() -> EpochSecs {
    (Date::now() / 1000.0) as EpochSecs
}

/// 口を読む（届かない・200 でない・本文が字でなければ Failed）。
async fn fetch(path: &str) -> Fetched {
    let Some(window) = web_sys::window() else {
        return Fetched::Failed;
    };
    let Ok(value) = JsFuture::from(window.fetch_with_str(path)).await else {
        return Fetched::Failed;
    };
    let Ok(response) = value.dyn_into::<Response>() else {
        return Fetched::Failed;
    };
    if !response.ok() {
        return Fetched::Failed;
    }
    let Ok(text) = response.text() else {
        return Fetched::Failed;
    };
    match JsFuture::from(text).await.ok().and_then(|t| t.as_string()) {
        Some(body) => Fetched::Body(body),
        None => Fetched::Failed,
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
    let text = JsFuture::from(response.text().ok()?).await.ok()?.as_string()?;
    Some((response.status(), text))
}

/// 1 つの口を読み、結果を signal に置く。
fn load(path: &'static str, signal: ArcRwSignal<Fetched>) {
    spawn_local(async move {
        let fetched = fetch(path).await;
        signal.set(fetched);
    });
}

/// 登録された口の全部。
fn registered() -> Vec<(&'static str, ArcRwSignal<Fetched>)> {
    READS.with_borrow(|reads| reads.clone())
}

/// 登録された口を全部読み直す（書きの口へ送った後に block も呼ぶ）。
pub fn reload_all() {
    for (path, signal) in registered() {
        load(path, signal);
    }
}

/// 登録された口を全部「読めない」にする。
fn lose_all() {
    for (_, signal) in registered() {
        signal.set(Fetched::Failed);
    }
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
