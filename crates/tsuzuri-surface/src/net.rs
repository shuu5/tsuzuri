//! 通信（wasm の target のときだけ組み立てる）: block ごとの読みの口を読み、変化の知らせ（SSE）で読み直す。
//! block は自分の口の path を `read` に渡し、読みの結果（3 値）の signal を受ける。
//! 知らせの接続は頁に 1 本だけ持ち、開いたで登録された口を全部読み直し、読み直しの合図の event を受けたで
//! 合図の種類（view の `changed_kinds`）を読む口（view の `reloads`）だけを読み直す。
//! 初めての口は接続が開くまで読まず、開いた時の 1 巡を最初の読みにする（開かなければ 1 秒で読む）。
//! 口ごとの読みの印（flight の Flight）で、読みの途中の呼びは読みの後の 1 回にまとめ、
//! 見ている部品が無い（片付いた）口の呼びは部品が戻ったときの 1 回にまとめる。
//! 読みの後は view の決め方で置くかを決める（同じ本文は置かない・一度の読めないは 1 秒後に読み直す）。
//! 知らせが切れても最後に読めた中身を READ_HOLD_S 秒まで出し続け、越えたら登録された口を全部「読めない」にする
//! （要件 NFR2・決め方を通さない）。読みの応答の頭（最後に読めた時からの秒）と切れた時刻は fresh の
//! Fresh に置き、上端の帯の最終の記録と読み込み不良の印が読む。台帳の読みが落ちている間は HELD_POLL_MS ごとに読み直す。
//! 頁を開いてから READ_HOLD_S 秒の間に知らせの接続が一度も開かなければ（待たされたまま誤りも来ない）、同じく全部を
//! 「読めない」にし、理由は Fresh の印の card に出す（最初の読みが開きを待つ形は残す）。
//! 読みの印を変えるたびに読みの途中の口を数え直し、上端の帯の読みの脈が読む。
//! 書きの口へは本文つきの POST を送り、状態の数と本文の字を返す。
//! 読みの重い口（表示先）は `get` で 1 回だけ読む（登録せず知らせでも読み直さない）。
//! path が query で変わる口（節点の近傍）は `read_path` に path の字の signal を渡し、
//! 読みの結果と応答の状態の数の組を受ける。path が変わったときと知らせの合図で読み直し、接続は同じ 1 本を使う。

use std::cell::{Cell, RefCell};
use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::ledger::{READ_AGE_HEADER, READ_HOLD_S};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::Date;
use web_sys::wasm_bindgen::closure::Closure;
use web_sys::wasm_bindgen::{JsCast, JsValue};
use web_sys::{EventSource, Headers, MessageEvent, Request, RequestInit, Response};

use crate::flight::{Flight, OPEN_WAIT_MS};
use crate::fresh::{Fresh, HELD_POLL_MS, read_age};
use crate::view::{Fetched, RELOAD_EVENTS, RETRY_MS, Settle, changed_kinds, reloads, settle};

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
    /// 中身の古さの signal（初めての呼びで作る・頁に 1 本だけ）。
    static FRESH: RefCell<Option<ArcRwSignal<Fresh>>> = const { RefCell::new(None) };
    /// 台帳の読みが落ちている間の読み直しを待っているか。
    static POLLING: Cell<bool> = const { Cell::new(false) };
    /// 読みの途中の口の数の signal（初めての呼びで作る・頁に 1 本だけ）。
    static BUSY: RefCell<Option<ArcRwSignal<usize>>> = const { RefCell::new(None) };
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

/// 中身の古さの signal（初めての呼びで作る）。
fn fresh_signal() -> ArcRwSignal<Fresh> {
    FRESH.with_borrow_mut(|fresh| {
        fresh
            .get_or_insert_with(|| ArcRwSignal::new(Fresh::default()))
            .clone()
    })
}

/// 中身の古さ（上端の帯の最終の記録と読み込み不良の印が読む）。
pub fn fresh() -> ReadSignal<Fresh> {
    ReadSignal::from(fresh_signal().read_only())
}

/// 中身の古さに `f` を撃ち、値が変わった時だけ signal に置く。
fn change<R>(f: impl FnOnce(&mut Fresh) -> R) -> R {
    let signal = fresh_signal();
    let mut next = signal.get_untracked();
    let out = f(&mut next);
    if signal.with_untracked(|now| *now != next) {
        signal.set(next);
    }
    out
}

/// 口 `path` の読めた応答の頭の秒（頭が無ければ None）を古さに置き、読みが落ちている間は後で読み直す
/// （待ちは 1 つだけ・読み直しの応答でまだ落ちていれば次の待ちを置く・見張りが戻りを知らせない場合の拾い）。
fn note(path: &str, age: Option<u64>) {
    change(|f| f.got(path, now(), age));
    if fresh_signal().with_untracked(Fresh::held) && !POLLING.replace(true) {
        set_timeout(
            || {
                POLLING.set(false);
                if fresh_signal().with_untracked(Fresh::held) {
                    reload_all();
                }
            },
            Duration::from_millis(HELD_POLL_MS),
        );
    }
}

/// 知らせのつながりが切れた（切れた最初の誤りから READ_HOLD_S 秒の後もまだ切れていれば全部「読めない」にする・
/// 繋ぎ直しのたびの誤りは待ちを増やさない）。
fn link_lost() {
    let at = now();
    if !change(|f| f.lose(at)) {
        return;
    }
    set_timeout(
        move || {
            if fresh_signal().with_untracked(Fresh::lost_since) == Some(at) {
                lose_all();
            }
        },
        Duration::from_secs(READ_HOLD_S),
    );
}

/// 頁を開いた時刻 `at` から READ_HOLD_S 秒の後も知らせの接続が一度も開いていなければ、登録された口を全部「読めない」に
/// する（待たされたまま誤りも来ない接続・理由は古さの card に出す・行 g-accept-face）。
fn link_unopened(at: EpochSecs) {
    if change(|f| f.unopened(at, now())) {
        lose_all();
    }
}

/// 知らせのつながりが開いた（切れた時刻を消す）。
fn link_back() {
    change(Fresh::back);
}

/// 口を読む（届かない・200 でない・本文が字でなければ Failed）。
async fn fetch(path: &str) -> Fetched {
    fetch_status(path).await.0
}

/// 口を読み、応答の状態の数も返す（応答が無ければ状態の数は None）。
/// 読めた応答の頭（最後に読めた時からの秒）は古さに置く（200 でない・届かない応答は置かない）。
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
        Some(body) => {
            let header = response.headers().get(READ_AGE_HEADER).ok().flatten();
            let age = read_age(header.as_deref());
            note(path, age);
            (Fetched::Body(body), status)
        }
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

/// 口を 1 回だけ読み、状態の数と本文の字を返す（届かない・応答が読めなければ None）。
/// 登録も知らせの接続もせず、古さにも置かない（読みの重い口を block が開いた時と書きの後だけ読むために使う）。
pub async fn get(path: &str) -> Option<(u16, String)> {
    let window = web_sys::window()?;
    let value = JsFuture::from(window.fetch_with_str(path)).await.ok()?;
    let response = value.dyn_into::<Response>().ok()?;
    let text = JsFuture::from(response.text().ok()?)
        .await
        .ok()?
        .as_string()?;
    Some((response.status(), text))
}

/// 読みの途中の口の数の signal（初めての呼びで作る）。
fn busy_signal() -> ArcRwSignal<usize> {
    BUSY.with_borrow_mut(|busy| busy.get_or_insert_with(|| ArcRwSignal::new(0)).clone())
}

/// 読みの途中の口の数（上端の帯の読みの脈が読む・行 g-pulse）。
pub fn busy() -> ReadSignal<usize> {
    ReadSignal::from(busy_signal().read_only())
}

/// 読みの途中の口を数え直し、数が変わった時だけ signal に置く
/// （READS と WATCHES を借りている間は撃たない・二重の借りになる）。
fn count_busy() {
    let reads = READS.with_borrow(|reads| reads.iter().filter(|(_, _, f)| f.busy()).count());
    let watches = WATCHES.with_borrow(|w| w.iter().filter(|(_, _, f)| f.busy()).count());
    let n = reads + watches;
    let signal = busy_signal();
    if signal.get_untracked() != n {
        signal.set(n);
    }
}

/// 登録された口の `slot` 番目の読みの印に `f` を撃つ（枠が無ければ None・撃った後に読みの途中の口を数え直す）。
fn read_flight<R>(slot: usize, f: impl FnOnce(&mut Flight) -> R) -> Option<R> {
    let out = READS.with_borrow_mut(|reads| reads.get_mut(slot).map(|(_, _, flight)| f(flight)));
    count_busy();
    out
}

/// path が変わる口の `slot` 番目の読みの印に `f` を撃つ（枠が無ければ None・撃った後に読みの途中の口を数え直す）。
fn watch_flight<R>(slot: usize, f: impl FnOnce(&mut Flight) -> R) -> Option<R> {
    let out = WATCHES.with_borrow_mut(|w| w.get_mut(slot).map(|(_, _, flight)| f(flight)));
    count_busy();
    out
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
    reload_if(|_| true);
}

/// 登録された口のうち path が `pick` に当たる口を読み直す（まとめ方は `reload_all` と同じ）。
fn reload_if(pick: impl Fn(&str) -> bool) {
    for slot in 0..READS.with_borrow(Vec::len) {
        let hit = READS.with_borrow(|reads| reads.get(slot).is_some_and(|(p, _, _)| pick(p)));
        if hit && read_flight(slot, |flight| flight.call()) == Some(true) {
            load_at(slot, 1);
        }
    }
    for slot in 0..WATCHES.with_borrow(Vec::len) {
        let go = WATCHES.with_borrow_mut(|w| {
            let (path, _, flight) = w.get_mut(slot)?;
            (!path.is_empty() && pick(path) && flight.call()).then(|| flight.round())
        });
        if let Some(round) = go {
            load_watch(slot, round, 1);
        }
    }
    count_busy();
}

/// 読み直しの合図を受けた（合図の種類を読む口だけを読み直す・行 c-ev-kind）。
fn reload_changed(event: MessageEvent) {
    let kinds = changed_kinds(&event.type_(), event.data().as_string().as_deref());
    reload_if(|path| reloads(path, &kinds));
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
            // 前の path の古さは数えない。
            if let Some(old) = &before {
                change(|f| f.forget(old));
            }
            let go = WATCHES.with_borrow_mut(|w| {
                let (p, _, flight) = w.get_mut(slot)?;
                p.clone_from(&now);
                let round = flight.renew();
                (LIVE.get() && !now.is_empty() && flight.start()).then_some(round)
            });
            count_busy();
            out.set((Fetched::NotRead, None));
            if let Some(round) = go {
                load_watch(slot, round, 1);
            }
        }
        now
    });
    on_cleanup(move || {
        let users = watch_flight(slot, |flight| {
            flight.detach();
            flight.users()
        });
        // 見ている部品が無くなった口の古さは数えない。
        if users == Some(0) {
            let now = WATCHES.with_borrow(|w| w.get(slot).map(|(p, _, _)| p.clone()));
            if let Some(p) = now {
                change(|f| f.forget(&p));
            }
        }
    });
    connect();
    ReadSignal::from(signal.read_only())
}

/// 口の読みの結果の signal（初めての path は登録し、接続が開いた後なら 1 回読む・
/// 片付いていた口は見ていない間に呼びが来ていれば読む・知らせの接続がまだ無ければ張る）。
pub fn read(path: &'static str) -> ReadSignal<Fetched> {
    let (slot, signal, go) = READS.with_borrow_mut(|reads| {
        if let Some((slot, (_, signal, flight))) = reads
            .iter_mut()
            .enumerate()
            .find(|(_, (p, _, _))| *p == path)
        {
            return (slot, signal.clone(), flight.attach());
        }
        let signal = ArcRwSignal::new(Fetched::NotRead);
        let mut flight = Flight::default();
        flight.attach();
        let go = LIVE.get() && flight.start();
        reads.push((path, signal.clone(), flight));
        (reads.len() - 1, signal, go)
    });
    count_busy();
    if go {
        load_at(slot, 1);
    }
    on_cleanup(move || {
        let users = read_flight(slot, |flight| {
            flight.detach();
            flight.users()
        });
        // 見ている部品が無くなった口の古さは数えない。
        if users == Some(0) {
            change(|f| f.forget(path));
        }
    });
    connect();
    ReadSignal::from(signal.read_only())
}

/// 知らせの接続を張る（頁に 1 本だけ・開いたで最初の読みを許して全部を読み直し・合図で種類を読む口を読み直し・
/// 切れたら切れた時刻を置き READ_HOLD_S 秒の後もまだ切れていれば全部「読めない」・開いたの event で切れた時刻を消す）。
/// 接続は頁の一生の間ずっと持つ（切れても EventSource が繋ぎ直し、開いたで読み直す）。
/// 開くのを待つのは上限まで（越えたら開くのを待たずに 1 巡読み、後で開いたときにもう 1 巡読む）。
/// 上限の timer は切れた時刻を消さない（開く前に切れた接続の時刻を残す）。
/// 頁を開いてから READ_HOLD_S 秒の後も一度も開いていなければ全部「読めない」にする（行 g-accept-face）。
fn connect() {
    if CONNECTED.replace(true) {
        return;
    }
    let at = now();
    let Ok(source) = EventSource::new(EVENTS_PATH) else {
        lose_all();
        go_live();
        return;
    };
    let on_open = Closure::<dyn FnMut()>::new(go_live);
    let on_change = Closure::<dyn FnMut(MessageEvent)>::new(reload_changed);
    let on_error = Closure::<dyn FnMut()>::new(link_lost);
    let on_back = Closure::<dyn FnMut()>::new(link_back);
    source.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    source.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    if source
        .add_event_listener_with_callback("open", on_back.as_ref().unchecked_ref())
        .is_err()
    {
        lose_all();
    }
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
    on_back.forget();
    std::mem::forget(source);
    set_timeout(
        || {
            if !LIVE.get() {
                go_live();
            }
        },
        Duration::from_millis(OPEN_WAIT_MS),
    );
    set_timeout(move || link_unopened(at), Duration::from_secs(READ_HOLD_S));
}
