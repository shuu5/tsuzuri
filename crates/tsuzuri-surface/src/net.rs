//! 通信（wasm の target のときだけ組み立てる）: 台帳の一覧の口を読み、変化の知らせ（SSE）を受けたら読み直す。
//! 知らせが切れている間は、今の一覧が正しいと言えないので測れていないにする（要件 NFR2）。

use leptos::prelude::*;
use leptos::task::spawn_local;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::ledger::LEDGER_CHANGED_EVENT;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::Date;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::closure::Closure;
use web_sys::{EventSource, Response};

use crate::view::{Fetched, Screen};

/// 台帳の一覧の口（server の便 e-min）。
pub const LEDGER_PATH: &str = "/api/ledger";

/// 変化の知らせの口（SSE・server の便 e-min）。
pub const EVENTS_PATH: &str = "/api/surface/events";

/// 今の時刻（epoch 秒）。
fn now() -> EpochSecs {
    (Date::now() / 1000.0) as EpochSecs
}

/// 一覧の口を読む（届かない・200 でない・本文が字でなければ Failed）。
async fn fetch_ledger() -> Fetched {
    let Some(window) = web_sys::window() else {
        return Fetched::Failed;
    };
    let Ok(value) = JsFuture::from(window.fetch_with_str(LEDGER_PATH)).await else {
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

/// 一覧を読み直して画面の状態を進める。
fn reload(screen: RwSignal<Screen>) {
    spawn_local(async move {
        let fetched = fetch_ledger().await;
        screen.update(|s| *s = s.after_read(&fetched, now()));
    });
}

/// 最初の 1 回を読み、知らせを受け続ける（開いた・変わったで読み直し、切れたら測れていない）。
/// 知らせの接続は画面の一生の間ずっと持つ（切れても EventSource が繋ぎ直す）。
pub fn follow(screen: RwSignal<Screen>) {
    reload(screen);
    let Ok(source) = EventSource::new(EVENTS_PATH) else {
        screen.update(|s| *s = s.after_lost());
        return;
    };
    let on_open = Closure::<dyn FnMut()>::new(move || reload(screen));
    let on_change = Closure::<dyn FnMut()>::new(move || reload(screen));
    let on_error = Closure::<dyn FnMut()>::new(move || screen.update(|s| *s = s.after_lost()));
    source.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    source.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    if source
        .add_event_listener_with_callback(LEDGER_CHANGED_EVENT, on_change.as_ref().unchecked_ref())
        .is_err()
    {
        screen.update(|s| *s = s.after_lost());
    }
    on_open.forget();
    on_change.forget();
    on_error.forget();
    std::mem::forget(source);
}
