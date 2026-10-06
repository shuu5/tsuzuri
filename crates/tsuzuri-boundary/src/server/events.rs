//! 変化の知らせ（口 GET /api/surface/events・SSE）。
//! 変化の印（ledger の `Mark`: store の manifest の字と manifest が名指す file の長さ・store が無ければ jsonl の 2 file の
//! 更新時刻と長さ）を 500 ミリ秒ごとに見て、動いたら台帳を読み直す（規則の行 R-21: 合図なしは周期の読み 500 ms ごとで 1.5 秒以内・要件 NFR2）。
//! 印の取りこぼしを拾うために、印が動かなくても、store の印で前の読みが読めていれば 600 秒ごと、ほかは 5 秒ごとに
//! 読み直す（jsonl の印は器の素の bd close で動かないことがある）。
//! 受け手（`Subscription`）が 0 人の間は、印が動いても読まず印も置かず（口の読みが印の遅れを見て自分で読む）、
//! 0 人から 1 人以上になった周で 1 回読む。
//! 読みの結果が前と変わったときだけ、接続中の全員に 1 件ずつ送る。
//! 板の変化: 器の event log の file と設計文書の dir の下の全 file の印を 500 ミリ秒ごとに見て、
//! 動いたら board-changed を周に 1 件送る（要件 NFR2 の「器の event と台帳の変化は 5 秒以内に面へ届く」）。
//! 席の状態の file（`<state dir>/seat/<席の dir>/` の state.jsonl と tick-last）の印も板の印に足す。
//! 席の card の印と account board の印（`crate::acct` の `Acct::marks`）は同じ dir の停止の記録
//! heartbeat-off と明示の on の記録 heartbeat-on の印も持つ（停止と明示の on の切り替えで board-changed が出る）。
//! board-changed の data は契約の `BoardChanged`（`{"at":<epoch 秒>,"kinds":[…]}`・印の動いた種類）で、
//! ledger-changed の data は `{"at":<epoch 秒>}`。板の印は種類（`ChangeKind`）と file の組で、種類をまたいで
//! 一覧の並びだけが変わった周は送らない。
//! 問いの合図（口 POST /api/surface/questions・`NUDGE_PATH`）は `Hub::nudge` で台帳の見張りの周期の待ちを
//! 終わらせ、次の周を待たずに印を見る（規則の行 R-21: 問いの合図ありで 200 ms 以内・要件 NFR2）。
//! 合図は読みを強いず、印が動いていない周では読まない（bd の読みは台帳の store の錠を取るので、合図の数で読みを増やさない）。
//! 待ちの間に溜まった合図は 1 周にまとめる。
//! ほかの project の台帳の見張り（`Hub::watch_ledger_into`）は server の Hub に足し、同じ受け手の数えと
//! 読み直しの間隔で読んで、その Hub の受け手に ledger-changed を送る（問いの合図では起きない）。

use std::io::{self, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{LEDGER_CHANGED_EVENT, LedgerChanged};
use tsuzuri_contract::surface::{BOARD_CHANGED_EVENT, BoardChanged, ChangeKind};
use tsuzuri_contract::wire;

use super::ledger::{Mark, Source};

/// 印を見る間隔（規則の行 R-21）。
pub const POLL: Duration = Duration::from_millis(500);

/// 印が動かなくても読み直す間隔（取りこぼしを拾う）。
pub const REREAD: Duration = Duration::from_secs(5);

/// 台帳の store の印で見るとき、前の読みが読めていれば印が動かなくても読み直す間隔（安全の網）。
pub const STORE_REREAD: Duration = Duration::from_secs(600);

/// 周期の読みの間隔。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    /// 印を見る間隔。
    pub poll: Duration,
    /// 印が動かなくても読み直す間隔。
    pub reread: Duration,
    /// store の印で前の読みが読めていれば、印が動かなくても読み直す間隔。
    pub store_reread: Duration,
}

/// server の周期の読みの間隔。
pub const TIMING: Timing = Timing {
    poll: POLL,
    reread: REREAD,
    store_reread: STORE_REREAD,
};

/// 変化の無いときに送る注釈の行の間隔（切れた接続を見つけて片付ける）。
pub const KEEPALIVE: Duration = Duration::from_secs(15);

/// 問いの合図の口の path（設計の討論 B の契約の草案の合図の口）。
pub const NUDGE_PATH: &str = "/api/surface/questions";

/// 問いの合図を受けた応答の本文。
pub const NUDGED: &str = "nudged";

/// file の更新時刻と長さ（無ければ None）。
pub fn stamp(path: &Path) -> Option<(SystemTime, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// 接続中の SSE の受け手の集まり。
#[derive(Default)]
pub struct Hub {
    subscribers: Mutex<Vec<Sender<String>>>,
    seq: Mutex<u64>,
    live: Arc<()>,
    /// 台帳の見張りの周期の待ちを終わらせる送り手（見張りの無い Hub は None）。
    wake: Mutex<Option<Sender<()>>>,
}

/// 受け手の 1 人（Receiver として読み、落ちれば受け手の数から外れる）。
pub struct Subscription {
    rx: Receiver<String>,
    _live: Arc<()>,
}

impl Deref for Subscription {
    type Target = Receiver<String>;

    fn deref(&self) -> &Receiver<String> {
        &self.rx
    }
}

impl Hub {
    /// 台帳の周期の読みと、板の印の周期の見張りを始める（Hub が落ちればどちらも止まる）。
    /// `board` は板の印の種類と file の組の一覧を返す関数で、周ごとに撃ち直す（増えた file と消えた file も印の変化）。
    pub fn start<B>(source: Source, board: B) -> Arc<Hub>
    where
        B: FnMut() -> Vec<(ChangeKind, PathBuf)> + Send + 'static,
    {
        let marks = source.clone();
        let hub = Hub::watch_ledger(move || marks.mark(), move || source.read(), TIMING);
        Hub::watch_board(&hub, board, POLL);
        hub
    }

    /// 板の印の種類と file の組の一覧を `poll` ごとに取り直し、印（種類と path と更新時刻と長さ）が動いたら
    /// 印の動いた種類を載せた board-changed を周に 1 件送る。最初の印は戻る前に取る（戻った後の変化は取りこぼさない）。
    /// 周の途中（一覧を取った後で印を取る前）に消えた file は、同じ周で取り直した一覧にも同じ種類と path の組が
    /// 無ければその周の印から落とす（消し 1 回を 2 周続けての変化に数えない）。
    /// 印の列が動いても、どの種類の印の列も前と同じ周（種類をまたいで並びだけが変わった周）は送らない。
    pub fn watch_board<B>(hub: &Arc<Hub>, mut board: B, poll: Duration)
    where
        B: FnMut() -> Vec<(ChangeKind, PathBuf)> + Send + 'static,
    {
        let weak = Arc::downgrade(hub);
        let mut seen = board_stamps(&mut board);
        thread::spawn(move || {
            loop {
                thread::sleep(poll);
                if weak.strong_count() == 0 {
                    return;
                }
                let current = board_stamps(&mut board);
                if current == seen {
                    continue;
                }
                let kinds = moved(&seen, &current);
                seen = current;
                if kinds.is_empty() {
                    continue;
                }
                let Some(hub) = weak.upgrade() else {
                    return;
                };
                hub.board_changed(&kinds, now());
            }
        });
    }

    /// 印の file と読みの関数で周期の読みを始める。最初の印と読みは戻る前に取る
    /// （戻った後の変化は取りこぼさない）。
    pub fn watch<R, F>(marks: Vec<PathBuf>, read: F, timing: Timing) -> Arc<Hub>
    where
        R: PartialEq + Send + 'static,
        F: FnMut() -> R + Send + 'static,
    {
        watch_with(
            move || stamps(&marks),
            read,
            timing.poll,
            move |_, _| timing.reread,
        )
    }

    /// 台帳の印の値（`Mark`）と読みの関数で周期の読みを始める（行 e-marks）。最初の印と読みは戻る前に取る。
    /// 受け手が居る間は、印が動かなくても、印が Store で前の読みが Known なら `store_reread`、ほかは `reread` ごとに読み直す。
    pub fn watch_ledger<T, M, F>(mark: M, read: F, timing: Timing) -> Arc<Hub>
    where
        T: PartialEq + Send + 'static,
        M: FnMut() -> Mark + Send + 'static,
        F: FnMut() -> Reading<T> + Send + 'static,
    {
        watch_with(mark, read, timing.poll, ledger_reread(timing))
    }

    /// `watch_ledger` と同じ周期の読みを、在る Hub に足す（ほかの project の台帳の見張り・行 e-multi-ask）。
    /// 最初の印と読みは戻る前に取る。受け手の数えと送る先はその Hub のもので、問いの合図（`nudge`）では起きない。
    /// Hub が落ちれば止まる。
    pub fn watch_ledger_into<T, M, F>(hub: &Arc<Hub>, mut mark: M, mut read: F, timing: Timing)
    where
        T: PartialEq + Send + 'static,
        M: FnMut() -> Mark + Send + 'static,
        F: FnMut() -> Reading<T> + Send + 'static,
    {
        let weak = Arc::downgrade(hub);
        // 送り手は thread が持ち続けるので、待ちは合図で終わらず poll ごとに時間切れになる。
        let (tx, wake) = mpsc::channel();
        let seen = mark();
        let read_at = Instant::now();
        let last = read();
        let state = Watch {
            seen,
            read_at,
            last,
            listening: false,
        };
        thread::spawn(move || {
            let _held: Sender<()> = tx;
            let link = Link {
                hub: &weak,
                wake: &wake,
                poll: timing.poll,
            };
            watch(link, state, mark, read, ledger_reread(timing));
        });
    }

    /// 受け手を 1 人足す。
    pub fn subscribe(&self) -> Subscription {
        let (tx, rx) = mpsc::channel();
        lock(&self.subscribers).push(tx);
        Subscription {
            rx,
            _live: Arc::clone(&self.live),
        }
    }

    /// 受け手の数（落ちていない `Subscription` の数）。
    pub fn listeners(&self) -> usize {
        Arc::strong_count(&self.live) - 1
    }

    /// 台帳の見張りの周期の待ちを終わらせる（問いの合図・行 e-signal）。見張りが在り合図を送れれば真。
    /// 見張りは次の周を待たずに印を見て、印が動いていなければ読まない。
    pub fn nudge(&self) -> bool {
        lock(&self.wake)
            .as_ref()
            .is_some_and(|tx| tx.send(()).is_ok())
    }

    /// 台帳の変化を全員に送る（切れた受け手は外す・電文の字にできなければ送らない）。
    pub fn ledger_changed(&self, at: u64) {
        if let Ok(data) = wire::encode(&LedgerChanged { at }) {
            self.send(LEDGER_CHANGED_EVENT, &data);
        }
    }

    /// 板の変化（器の event log か設計文書か席か account か台帳の形の行）を、動いた種類を載せて全員に送る
    /// （切れた受け手は外す・電文の字にできなければ送らない）。
    pub fn board_changed(&self, kinds: &[ChangeKind], at: u64) {
        if let Ok(data) = wire::encode(&BoardChanged {
            at,
            kinds: kinds.to_vec(),
        }) {
            self.send(BOARD_CHANGED_EVENT, &data);
        }
    }

    /// event の名と data の字で frame を組んで全員に送る。
    fn send(&self, event: &str, data: &str) {
        let id = {
            let mut seq = lock(&self.seq);
            *seq += 1;
            *seq
        };
        let frame = format!("id: {id}\nevent: {event}\ndata: {data}\n\n");
        lock(&self.subscribers).retain(|tx| tx.send(frame.clone()).is_ok());
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// 今の時刻（UTC の epoch 秒・時計が 1970 年より前なら 0）。
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// 印の file の全部の更新時刻と長さ。
fn stamps(marks: &[PathBuf]) -> Vec<Option<(SystemTime, u64)>> {
    marks.iter().map(|m| stamp(m)).collect()
}

/// 板の印（種類と file の path と、その更新時刻と長さ）。
type BoardStamps = Vec<(ChangeKind, PathBuf, Option<(SystemTime, u64)>)>;

/// 1 つの周の板の印。無い file の印は、印を取った後に撃ち直した一覧にも同じ種類と path の組が在るときだけ残す
/// （一覧に載ったまま無い file は印に数え、周の途中で一覧から消えた file は落とす）。
fn board_stamps(board: &mut impl FnMut() -> Vec<(ChangeKind, PathBuf)>) -> BoardStamps {
    let mut stamps: BoardStamps = board()
        .into_iter()
        .map(|(k, f)| {
            let s = stamp(&f);
            (k, f, s)
        })
        .collect();
    if stamps.iter().any(|(_, _, s)| s.is_none()) {
        let again = board();
        stamps.retain(|(k, f, s)| s.is_some() || again.iter().any(|(ak, af)| ak == k && af == f));
    }
    stamps
}

/// 種類ごとの印の列（その種類の path と印を一覧の順に並べた列）が前の周と違う種類（`ChangeKind::ALL` の順）。
fn moved(seen: &BoardStamps, current: &BoardStamps) -> Vec<ChangeKind> {
    let of = |stamps: &BoardStamps, kind: ChangeKind| -> Vec<(PathBuf, Option<(SystemTime, u64)>)> {
        stamps
            .iter()
            .filter(|(k, _, _)| *k == kind)
            .map(|(_, f, s)| (f.clone(), *s))
            .collect()
    };
    ChangeKind::ALL
        .into_iter()
        .filter(|&kind| of(seen, kind) != of(current, kind))
        .collect()
}

/// 台帳の読み直しの間隔（印が Store で前の読みが Known なら `store_reread`、ほかは `reread`）。
fn ledger_reread<T>(timing: Timing) -> impl Fn(&Mark, &Reading<T>) -> Duration + Send + 'static {
    move |mark, last| match (mark, last) {
        (Mark::Store { .. }, Reading::Known(_)) => timing.store_reread,
        _ => timing.reread,
    }
}

/// 周期の読みの状態（見た印・最後に読んだ時刻・最後の読みの結果・前の周に受け手が居たか）。
struct Watch<K, R> {
    seen: K,
    read_at: Instant,
    last: R,
    listening: bool,
}

/// 印の値の関数と読みの関数で周期の読みを始める（`reread` は見た印と最後の読みから読み直しの間隔を決める）。
fn watch_with<K, R, M, F>(
    mut mark: M,
    mut read: F,
    poll: Duration,
    reread: impl Fn(&K, &R) -> Duration + Send + 'static,
) -> Arc<Hub>
where
    K: PartialEq + Send + 'static,
    R: PartialEq + Send + 'static,
    M: FnMut() -> K + Send + 'static,
    F: FnMut() -> R + Send + 'static,
{
    let (tx, wake) = mpsc::channel();
    let hub = Arc::new(Hub {
        wake: Mutex::new(Some(tx)),
        ..Hub::default()
    });
    let weak = Arc::downgrade(&hub);
    let seen = mark();
    let read_at = Instant::now();
    let last = read();
    let state = Watch {
        seen,
        read_at,
        last,
        listening: false,
    };
    thread::spawn(move || {
        let link = Link {
            hub: &weak,
            wake: &wake,
            poll,
        };
        watch(link, state, mark, read, reread)
    });
    hub
}

/// 周期の読みの thread が見る Hub への弱い参照と合図の受け口と待ちの上限。
struct Link<'a> {
    hub: &'a Weak<Hub>,
    wake: &'a Receiver<()>,
    poll: Duration,
}

fn watch<K: PartialEq, R: PartialEq>(
    Link { hub, wake, poll }: Link<'_>,
    mut state: Watch<K, R>,
    mut mark: impl FnMut() -> K,
    mut read: impl FnMut() -> R,
    reread: impl Fn(&K, &R) -> Duration,
) {
    loop {
        // 合図は待ちを終わらせるだけ（溜まった合図は 1 周にまとめる）。送り手が落ちれば Hub も落ちた。
        match wake.recv_timeout(poll) {
            Ok(()) => while wake.try_recv().is_ok() {},
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        let Some(listening) = hub.upgrade().map(|h| h.listeners() > 0) else {
            return;
        };
        // 受け手が 0 人の間は印が動いても読まず、見た印も置かない（次に付いた周に 1 回読む）。
        if !listening {
            state.listening = false;
            continue;
        }
        // 印は読みの前に取る（読みの途中の変化は次の周で拾う）。
        let current = mark();
        // 受け手が 0 人から 1 人以上になった周は 1 回読む。
        let attached = !state.listening;
        state.listening = true;
        let due = state.read_at.elapsed() >= reread(&current, &state.last);
        if current == state.seen && !attached && !due {
            continue;
        }
        state.seen = current;
        state.read_at = Instant::now();
        let reading = read();
        if reading == state.last {
            continue;
        }
        state.last = reading;
        let Some(hub) = hub.upgrade() else {
            return;
        };
        hub.ledger_changed(now());
    }
}

/// SSE の接続を持ち続ける（受け手を足してから頭を書くので、頭の後の変化は取りこぼさない）。
pub fn stream(mut w: impl Write, hub: &Hub) -> io::Result<()> {
    let rx = hub.subscribe();
    w.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\nretry: 1000\n\n",
    )?;
    w.flush()?;
    loop {
        let frame = match rx.recv_timeout(KEEPALIVE) {
            Ok(frame) => frame,
            Err(RecvTimeoutError::Timeout) => ": keepalive\n\n".to_string(),
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        };
        w.write_all(frame.as_bytes())?;
        w.flush()?;
    }
}

#[cfg(test)]
mod tests {
    use super::{Hub, Timing};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    /// 印の dir の置き場（一時の dir の下・名に pid と字 name）。
    fn place(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("tz-events-{}-{name}", std::process::id()))
    }

    /// 印の 2 つの file の置き場（2 つめは初めは無い）。
    fn marks(name: &str) -> Vec<PathBuf> {
        let dir = place(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("置き場");
        std::fs::write(dir.join("a"), "1").expect("印 a");
        vec![dir.join("a"), dir.join("b")]
    }

    /// 印の dir の外の隣の dir（印の dir の名に -put を足す）。
    fn put_dir(mark: &Path) -> PathBuf {
        let dir = mark.parent().expect("印の dir");
        let name = dir.file_name().expect("印の dir の名").to_string_lossy();
        dir.with_file_name(format!("{name}-put"))
    }

    /// 印の file の中身を丸ごと替える（隣の dir に書いてから移すので、見張りから書きかけは見えない）。
    fn put(mark: &Path, text: &str) {
        let dir = put_dir(mark);
        std::fs::create_dir_all(&dir).expect("移しの dir");
        let tmp = dir.join(mark.file_name().expect("印の名"));
        std::fs::write(&tmp, text).expect("移す前の印");
        std::fs::rename(&tmp, mark).expect("印を移す");
    }

    /// 印の 2 つの file を作って `body` を撃ち、終わりに印の dir と隣の移しの dir を消す（歯ごとの dir を一時の dir に
    /// 残さない・memo t3-hub.74.49.10）。
    fn with_marks<T>(name: &str, body: impl FnOnce(&[PathBuf]) -> T) -> T {
        let marks = marks(name);
        let got = body(&marks);
        let _ = std::fs::remove_dir_all(put_dir(&marks[0]));
        let _ = std::fs::remove_dir_all(place(name));
        got
    }

    /// 中身を替えられる読み（読んだ回数を数える）。
    fn reader() -> (
        Arc<Mutex<u32>>,
        Arc<AtomicUsize>,
        impl FnMut() -> u32 + Send,
    ) {
        let content = Arc::new(Mutex::new(0));
        let reads = Arc::new(AtomicUsize::new(0));
        let (c, r) = (Arc::clone(&content), Arc::clone(&reads));
        let read = move || {
            r.fetch_add(1, Ordering::SeqCst);
            *c.lock().expect("lock")
        };
        (content, reads, read)
    }

    #[test]
    fn server_src_watch_marks_trigger_reread() {
        with_marks("marks", |marks| {
            let (content, reads, read) = reader();
            let timing = Timing {
                poll: Duration::from_millis(20),
                reread: Duration::from_secs(60),
                store_reread: Duration::from_secs(60),
            };
            let hub = Hub::watch(marks.to_vec(), read, timing);
            assert_eq!(reads.load(Ordering::SeqCst), 1, "最初の読みは戻る前");
            // 受け手が付いた周で 1 回読む。
            let rx = hub.subscribe();
            thread::sleep(Duration::from_millis(200));
            assert_eq!(reads.load(Ordering::SeqCst), 2, "受け手が付いた周の読み");
            // 印が動かなければ読み直さない（中身が変わっても知らせない）。
            *content.lock().expect("lock") = 1;
            thread::sleep(Duration::from_millis(200));
            assert!(rx.try_recv().is_err());
            assert_eq!(reads.load(Ordering::SeqCst), 2);
            // 後の印（初めは無い file）ができると読み直して 1 件。
            put(&marks[1], "x");
            let frame = rx.recv_timeout(Duration::from_secs(5)).expect("1 件");
            assert!(frame.contains("event: ledger-changed\n"), "{frame}");
            thread::sleep(Duration::from_millis(200));
            assert!(rx.try_recv().is_err(), "印 1 回に 2 件");
            assert_eq!(reads.load(Ordering::SeqCst), 3);
            // 前の印の長さが動いても、読みの結果が同じなら知らせない。
            put(&marks[0], "22");
            let until = Instant::now() + Duration::from_secs(5);
            while reads.load(Ordering::SeqCst) < 4 {
                assert!(
                    Instant::now() < until,
                    "印 a の変化で 5 秒以内に読み直さない"
                );
                thread::sleep(Duration::from_millis(10));
            }
            thread::sleep(Duration::from_millis(200));
            assert_eq!(reads.load(Ordering::SeqCst), 4);
            assert!(rx.try_recv().is_err(), "同じ読みで知らせる");
        });
    }

    #[test]
    fn server_src_watch_rereads_without_marks() {
        with_marks("reread", |marks| {
            let (content, reads, read) = reader();
            let timing = Timing {
                poll: Duration::from_millis(20),
                reread: Duration::from_millis(150),
                store_reread: Duration::from_secs(60),
            };
            let hub = Hub::watch(marks.to_vec(), read, timing);
            let rx = hub.subscribe();
            *content.lock().expect("lock") = 1;
            rx.recv_timeout(Duration::from_secs(1))
                .expect("印なしの読み直しで 1 件");
            // 同じ中身の読み直しは続くが知らせない。
            let before = reads.load(Ordering::SeqCst);
            thread::sleep(Duration::from_millis(500));
            assert!(reads.load(Ordering::SeqCst) >= before + 2);
            assert!(rx.try_recv().is_err(), "同じ読みで知らせる");
            // Hub が落ちれば読みも止まる。
            drop(rx);
            drop(hub);
            thread::sleep(Duration::from_millis(100));
            let after = reads.load(Ordering::SeqCst);
            thread::sleep(Duration::from_millis(400));
            assert_eq!(reads.load(Ordering::SeqCst), after);
        });
    }

    #[test]
    fn server_read_watch_board_sends_on_marks() {
        with_marks("board", |marks| {
            let hub = Arc::new(Hub::default());
            let d = marks[0].parent().expect("置き場").to_path_buf();
            let files = move || {
                let mut f: Vec<PathBuf> = std::fs::read_dir(&d)
                    .expect("置き場")
                    .map(|e| e.expect("entry").path())
                    .collect();
                f.sort();
                f.into_iter().map(|p| (super::ChangeKind::Design, p)).collect::<Vec<_>>()
            };
            Hub::watch_board(&hub, files, Duration::from_millis(20));
            let rx = hub.subscribe();
            thread::sleep(Duration::from_millis(100));
            assert!(rx.try_recv().is_err(), "印が動かないのに知らせる");
            // 在る file の長さが動く・file が増える・file が消える、のどれも 1 件。
            for change in [
                Box::new(|| put(&marks[0], "22")) as Box<dyn Fn()>,
                Box::new(|| put(&marks[1], "x")),
                Box::new(|| std::fs::remove_file(&marks[1]).expect("印 b を消す")),
            ] {
                change();
                let frame = rx.recv_timeout(Duration::from_secs(5)).expect("1 件");
                assert!(frame.contains("event: board-changed\n"), "{frame}");
                assert!(frame.contains("data: {\"at\":"), "{frame}");
                thread::sleep(Duration::from_millis(100));
                assert!(rx.try_recv().is_err(), "印 1 回に 2 件");
            }
        });
    }

    /// 撃つ間は印の dir が在り（印 a が在る）、撃った後は印の dir と隣の移しの dir が無い。
    #[test]
    fn server_src_watch_marks_dir_is_gone_after_the_body() {
        let seen = with_marks("gone", |marks| {
            let moving = put_dir(&marks[0]);
            std::fs::create_dir_all(&moving).expect("移しの dir");
            (marks[0].is_file(), moving)
        });
        assert!(seen.0, "印の dir が在る間に撃った");
        assert!(!place("gone").exists(), "撃った後は印の dir が無い");
        assert!(!seen.1.exists(), "撃った後は移しの dir が無い");
    }

    #[test]
    fn server_min_hub_sends_to_each_subscriber() {
        let hub = Hub::default();
        let a = hub.subscribe();
        let b = hub.subscribe();
        drop(b);
        hub.ledger_changed(7);
        assert_eq!(
            a.try_recv().expect("1 件"),
            "id: 1\nevent: ledger-changed\ndata: {\"at\":7}\n\n"
        );
        assert!(a.try_recv().is_err());
        assert_eq!(hub.subscribers.lock().expect("lock").len(), 1);
    }
}
