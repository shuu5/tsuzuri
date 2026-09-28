//! 通信（wasm の target のときだけ組み立てる・便 g-parts）: block ごとの読みの口を読み、変化の知らせ（SSE）で読み直す。
//! block は自分の口の path を `read` に渡し、読みの結果（3 値）の signal を受ける。
//! 知らせの接続は頁に 1 本だけ持ち、開いた・読み直しの合図の event を受けたで登録された口を全部読み直す。
//! 初めての口は接続が開くまで読まず、開いた時の 1 巡を最初の読みにする（開かなければ 1 秒で読む・行 g-reads）。
//! 口ごとの読みの印（flight の Flight）で、読みの途中の呼びは読みの後の 1 回にまとめ、
//! 見ている部品が無い（片付いた）口の呼びは部品が戻ったときの 1 回にまとめる。
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

use crate::flight::{Flight, OPEN_WAIT_MS};
use crate::view::{Fetched, RELOAD_EVENTS, RETRY_MS, Settle, settle};

/// 変化の知らせの口（SSE・server の便 e-min）。
pub const EVENTS_PATH: &str = "/api/surface/events";

/// 登録された口の枠（path と読みの結果と読みの印）。
type ReadSlot = (&'static str, ArcRwSignal<Fetched>, Flight);

/// path が変わる口の枠（今の path と、読みの結果と応答の状態の数の組と、読みの印）。
type WatchSlot = (String, ArcRwSignal<Status>, Flight);

thread_local! {
    /// 登録された口。同じ path は 1 つの signal を分ける。
    static READS: RefCell<Vec<ReadSlot>> = const { RefCell::new(Vec::new()) };
    /// path が変わる口。頁の一生の間だけ持つ。
    static WATCHES: RefCell<Vec<WatchSlot>> = const { RefCell::new(Vec::new()) };
    /// 知らせの接続を張ったか（頁に 1 本だけ）。
    static CONNECTED: Cell<bool> = const { Cell::new(false) };
    /// 最初の読みを撃ってよいか（接続が開いたか、開くのを待つ上限を越えた）。
    static LIVE: Cell<bool> = const { Cell::new(false) };
    /// 1 秒の時計の signal（初めて `ticker` を呼んだときに作る・頁に 1 本だけ）。
    static TICK: RefCell<Option<ArcRwSignal<EpochSecs>>> = const { RefCell::new(None) };
}

/// 時計の signal を書き直す間（ミリ秒）。
pub const TICK_MS: u64 = 1000;

/// 読みの結果と応答の状態の数（応答が無ければ None）の組。
type Status = (Fetched, Option<u16>);

/// 今の時刻（epoch 秒）。
pub fn now() -> EpochSecs {
    (Date::now() / 1000.0) as EpochSecs
}

/// 1 秒ごとに今の時刻を置く signal（初めての呼びで作り interval を 1 本張る・2 度目からは同じ signal を返す）。
pub fn ticker() -> ReadSignal<EpochSecs> {
    let signal = TICK.with_borrow_mut(|tick| {
        tick.get_or_insert_with(|| {
            let signal = ArcRwSignal::new(now());
            let out = signal.clone();
            set_interval(move || out.set(now()), Duration::from_millis(TICK_MS));
            signal
        })
        .clone()
    });
    ReadSignal::from(signal.read_only())
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

/// 登録された口の `slot` 番目の読みの印に `f` を撃つ（枠が無ければ None）。
fn read_flight<R>(slot: usize, f: impl FnOnce(&mut Flight) -> R) -> Option<R> {
    READS.with_borrow_mut(|reads| reads.get_mut(slot).map(|(_, _, flight)| f(flight)))
}

/// path が変わる口の `slot` 番目の読みの印に `f` を撃つ（枠が無ければ None）。
fn watch_flight<R>(slot: usize, f: impl FnOnce(&mut Flight) -> R) -> Option<R> {
    WATCHES.with_borrow_mut(|w| w.get_mut(slot).map(|(_, _, flight)| f(flight)))
}

/// 登録された口の `slot` 番目を `attempt` 回目に読む（同じ本文は置かない・一度の読めないは 1 秒後に読み直す・
/// 読みの途中に呼びが来ていれば続けてもう 1 回読む）。
fn load_at(slot: usize, attempt: u32) {
    let Some((path, signal)) =
        READS.with_borrow(|reads| reads.get(slot).map(|(p, s, _)| (*p, s.clone())))
    else {
        return;
    };
    spawn_local(async move {
        let fetched = fetch(path).await;
        match signal.with_untracked(|now| settle(now, &fetched, attempt)) {
            Settle::Set => signal.set(fetched),
            Settle::Keep => {}
            Settle::Retry => {
                set_timeout(
                    move || load_at(slot, attempt + 1),
                    Duration::from_millis(RETRY_MS),
                );
                return;
            }
        }
        if read_flight(slot, |flight| flight.end()) == Some(true) {
            load_at(slot, 1);
        }
    });
}

/// 登録された口の全部。
fn registered() -> Vec<(&'static str, ArcRwSignal<Fetched>)> {
    READS.with_borrow(|reads| reads.iter().map(|(p, s, _)| (*p, s.clone())).collect())
}

/// path が変わる口の `slot` 番目を `round` の path で `attempt` 回目に読む
/// （読む間に path が変われば置かない・決め方は `load_at` と同じ）。
fn load_watch(slot: usize, round: u32, attempt: u32) {
    let Some((path, signal)) = WATCHES.with_borrow(|w| {
        w.get(slot)
            .filter(|(_, _, flight)| flight.round() == round)
            .map(|(p, s, _)| (p.clone(), s.clone()))
    }) else {
        return;
    };
    if path.is_empty() {
        return;
    }
    spawn_local(async move {
        let got = fetch_status(&path).await;
        if watch_flight(slot, |flight| flight.round()) != Some(round) {
            return;
        }
        if signal.with_untracked(|now| *now != got) {
            match signal.with_untracked(|now| settle(&now.0, &got.0, attempt)) {
                Settle::Retry => {
                    set_timeout(
                        move || load_watch(slot, round, attempt + 1),
                        Duration::from_millis(RETRY_MS),
                    );
                    return;
                }
                Settle::Set | Settle::Keep => signal.set(got),
            }
        }
        let more = watch_flight(slot, |flight| flight.round() == round && flight.end());
        if more == Some(true) {
            load_watch(slot, round, 1);
        }
    });
}

/// 登録された口を全部読み直す（書きの口へ送った後に block も呼ぶ・読みの途中の口と見ていない口は後の 1 回にまとめる）。
pub fn reload_all() {
    for slot in 0..READS.with_borrow(Vec::len) {
        if read_flight(slot, |flight| flight.call()) == Some(true) {
            load_at(slot, 1);
        }
    }
    for slot in 0..WATCHES.with_borrow(Vec::len) {
        let go = WATCHES.with_borrow_mut(|w| {
            let (path, _, flight) = w.get_mut(slot)?;
            (!path.is_empty() && flight.call()).then(|| flight.round())
        });
        if let Some(round) = go {
            load_watch(slot, round, 1);
        }
    }
}

/// 最初の読みを許し、登録された口を全部読む（接続が開いた・開くのを待つ上限を越えた・接続を張れない）。
fn go_live() {
    LIVE.set(true);
    reload_all();
}

/// 登録された口を全部「読めない」にする。
fn lose_all() {
    for (_, signal) in registered() {
        signal.set(Fetched::Failed);
    }
    for (_, signal, _) in WATCHES.with_borrow(|w| w.clone()) {
        signal.set((Fetched::Failed, None));
    }
}

/// path が変わる口の読みの組の signal（path が変わるたびに「まだ読んでいない」に戻して読み直す・
/// 知らせの合図でも読み直す・片付いた後は読まない・知らせの接続は `read` と同じ 1 本）。
pub fn read_path(path: Signal<String>) -> ReadSignal<(Fetched, Option<u16>)> {
    let signal = ArcRwSignal::new((Fetched::NotRead, None));
    let slot = WATCHES.with_borrow_mut(|w| {
        let mut flight = Flight::default();
        flight.attach();
        w.push((String::new(), signal.clone(), flight));
        w.len() - 1
    });
    let out = signal.clone();
    Effect::new(move |before: Option<String>| {
        let now = path.get();
        if before.as_ref() != Some(&now) {
            let go = WATCHES.with_borrow_mut(|w| {
                let (p, _, flight) = w.get_mut(slot)?;
                p.clone_from(&now);
                let round = flight.renew();
                (LIVE.get() && !now.is_empty() && flight.start()).then_some(round)
            });
            out.set((Fetched::NotRead, None));
            if let Some(round) = go {
                load_watch(slot, round, 1);
            }
        }
        now
    });
    on_cleanup(move || {
        watch_flight(slot, |flight| flight.detach());
    });
    connect();
    ReadSignal::from(signal.read_only())
}

/// 口の読みの結果の signal（初めての path は登録し、接続が開いた後なら 1 回読む・
/// 片付いていた口は見ていない間に呼びが来ていれば読む・知らせの接続がまだ無ければ張る）。
pub fn read(path: &'static str) -> ReadSignal<Fetched> {
    let (slot, signal, go) = READS.with_borrow_mut(|reads| {
        if let Some(slot) = reads.iter().position(|(p, _, _)| *p == path) {
            let (_, signal, flight) = &mut reads[slot];
            return (slot, signal.clone(), flight.attach());
        }
        let signal = ArcRwSignal::new(Fetched::NotRead);
        let mut flight = Flight::default();
        flight.attach();
        let go = LIVE.get() && flight.start();
        reads.push((path, signal.clone(), flight));
        (reads.len() - 1, signal, go)
    });
    if go {
        load_at(slot, 1);
    }
    on_cleanup(move || {
        read_flight(slot, |flight| flight.detach());
    });
    connect();
    ReadSignal::from(signal.read_only())
}

/// 知らせの接続を張る（頁に 1 本だけ・開いたで最初の読みを許して読み直し・合図で読み直し・切れたら全部「読めない」）。
/// 接続は頁の一生の間ずっと持つ（切れても EventSource が繋ぎ直し、開いたで読み直す）。
/// 開くのを待つのは上限まで（越えたら開くのを待たずに 1 巡読み、後で開いたときにもう 1 巡読む）。
fn connect() {
    if CONNECTED.replace(true) {
        return;
    }
    let Ok(source) = EventSource::new(EVENTS_PATH) else {
        lose_all();
        go_live();
        return;
    };
    let on_open = Closure::<dyn FnMut()>::new(go_live);
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
    set_timeout(
        || {
            if !LIVE.get() {
                go_live();
            }
        },
        Duration::from_millis(OPEN_WAIT_MS),
    );
}
