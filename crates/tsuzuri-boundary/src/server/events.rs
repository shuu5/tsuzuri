//! 変化の知らせ（口 GET /api/surface/events・SSE・便 e-src）。
//! 変化の印（ledger の `Mark`: store の manifest の中身と journal の長さ・store が無ければ jsonl の 2 file の
//! 更新時刻と長さ）を 500 ミリ秒ごとに見て、動いたら台帳を読み直す（規則の行 R-21: 合図なしは周期の読み 500 ms ごとで 1.5 秒以内・要件 NFR2）。
//! 印の取りこぼしを拾うために、印が動かなくても、store の印で前の読みが読めていれば 60 秒ごと、ほかは 5 秒ごとに
//! 読み直す（jsonl の印は器の素の bd close で動かないことがある・行 e-marks）。
//! 読みの結果が前と変わったときだけ、接続中の全員に 1 件ずつ送る。
//! 板の変化（便 e-read）: 器の event log の file と設計文書の dir の下の全 file の印を 500 ミリ秒ごとに見て、
//! 動いたら board-changed を 1 件送る（要件 NFR2 の「器の event と台帳の変化は 5 秒以内に面へ届く」）。
//! 便 e-seat は席の状態の file（`<state dir>/seat/<席の dir>/` の state.jsonl と tick-last）の印を板の印に足す。
//! board-changed の data は ledger-changed と同じ形（`{"at":<epoch 秒>}`）。

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{LEDGER_CHANGED_EVENT, LedgerChanged};
use tsuzuri_contract::surface::BOARD_CHANGED_EVENT;
use tsuzuri_contract::wire;

use super::ledger::{Mark, Source};

/// 印を見る間隔（規則の行 R-21）。
pub const POLL: Duration = Duration::from_millis(500);

/// 印が動かなくても読み直す間隔（取りこぼしを拾う）。
pub const REREAD: Duration = Duration::from_secs(5);

/// 台帳の store の印で見るとき、前の読みが読めていれば印が動かなくても読み直す間隔（安全の網）。
pub const STORE_REREAD: Duration = Duration::from_secs(60);

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
}

impl Hub {
    /// 台帳の周期の読みと、板の印の周期の見張りを始める（Hub が落ちればどちらも止まる）。
    /// `board` は板の印の file の一覧を返す関数で、周ごとに撃ち直す（増えた file と消えた file も印の変化）。
    pub fn start<B>(source: Source, board: B) -> Arc<Hub>
    where
        B: FnMut() -> Vec<PathBuf> + Send + 'static,
    {
        let marks = source.clone();
        let hub = Hub::watch_ledger(move || marks.mark(), move || source.read(), TIMING);
        Hub::watch_board(&hub, board, POLL);
        hub
    }

    /// 板の印の file の一覧を `poll` ごとに取り直し、印（path と更新時刻と長さ）が動いたら
    /// board-changed を送る。最初の印は戻る前に取る（戻った後の変化は取りこぼさない）。
    /// 周の途中（一覧を取った後で印を取る前）に消えた file は、同じ周で取り直した一覧にも
    /// 無ければその周の印から落とす（消し 1 回を 2 周続けての変化に数えない）。
    pub fn watch_board<B>(hub: &Arc<Hub>, mut board: B, poll: Duration)
    where
        B: FnMut() -> Vec<PathBuf> + Send + 'static,
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
                seen = current;
                let Some(hub) = weak.upgrade() else {
                    return;
                };
                hub.board_changed(now());
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
    /// 印が動かなくても、印が Store で前の読みが Known なら `store_reread`、ほかは `reread` ごとに読み直す。
    pub fn watch_ledger<T, M, F>(mark: M, read: F, timing: Timing) -> Arc<Hub>
    where
        T: PartialEq + Send + 'static,
        M: FnMut() -> Mark + Send + 'static,
        F: FnMut() -> Reading<T> + Send + 'static,
    {
        watch_with(mark, read, timing.poll, move |mark, last| {
            match (mark, last) {
                (Mark::Store(_), Reading::Known(_)) => timing.store_reread,
                _ => timing.reread,
            }
        })
    }

    /// 受け手を 1 人足す。
    pub fn subscribe(&self) -> Receiver<String> {
        let (tx, rx) = mpsc::channel();
        lock(&self.subscribers).push(tx);
        rx
    }

    /// 台帳の変化を全員に送る（切れた受け手は外す）。
    pub fn ledger_changed(&self, at: u64) {
        self.send(LEDGER_CHANGED_EVENT, at);
    }

    /// 板の変化（器の event log か設計文書）を全員に送る（切れた受け手は外す）。
    pub fn board_changed(&self, at: u64) {
        self.send(BOARD_CHANGED_EVENT, at);
    }

    fn send(&self, event: &str, at: u64) {
        let data = wire::encode(&LedgerChanged { at }).expect("LedgerChanged は JSON になる");
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

/// 板の印（file の path と、その更新時刻と長さ）。
type BoardStamps = Vec<(PathBuf, Option<(SystemTime, u64)>)>;

/// 1 つの周の板の印。無い file の印は、印を取った後に撃ち直した一覧にも在るときだけ残す
/// （一覧に載ったまま無い file は印に数え、周の途中で一覧から消えた file は落とす）。
fn board_stamps(board: &mut impl FnMut() -> Vec<PathBuf>) -> BoardStamps {
    let mut stamps: BoardStamps = board()
        .into_iter()
        .map(|f| {
            let s = stamp(&f);
            (f, s)
        })
        .collect();
    if stamps.iter().any(|(_, s)| s.is_none()) {
        let again = board();
        stamps.retain(|(f, s)| s.is_some() || again.contains(f));
    }
    stamps
}

/// 周期の読みの状態（見た印・最後に読んだ時刻・最後の読みの結果）。
struct Watch<K, R> {
    seen: K,
    read_at: Instant,
    last: R,
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
    let hub = Arc::new(Hub::default());
    let weak = Arc::downgrade(&hub);
    let seen = mark();
    let read_at = Instant::now();
    let last = read();
    let state = Watch {
        seen,
        read_at,
        last,
    };
    thread::spawn(move || watch(&weak, state, mark, read, poll, reread));
    hub
}

fn watch<K: PartialEq, R: PartialEq>(
    hub: &Weak<Hub>,
    mut state: Watch<K, R>,
    mut mark: impl FnMut() -> K,
    mut read: impl FnMut() -> R,
    poll: Duration,
    reread: impl Fn(&K, &R) -> Duration,
) {
    loop {
        thread::sleep(poll);
        if hub.strong_count() == 0 {
            return;
        }
        // 印は読みの前に取る（読みの途中の変化は次の周で拾う）。
        let current = mark();
        if current == state.seen && state.read_at.elapsed() < reread(&current, &state.last) {
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

    /// 印の 2 つの file の置き場（2 つめは初めは無い）。
    fn marks(name: &str) -> Vec<PathBuf> {
        let dir = std::env::temp_dir().join(format!("tz-events-{}-{name}", std::process::id()));
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
        let marks = marks("marks");
        let (content, reads, read) = reader();
        let timing = Timing {
            poll: Duration::from_millis(20),
            reread: Duration::from_secs(60),
            store_reread: Duration::from_secs(60),
        };
        let hub = Hub::watch(marks.clone(), read, timing);
        let rx = hub.subscribe();
        assert_eq!(reads.load(Ordering::SeqCst), 1, "最初の読みは戻る前");
        // 印が動かなければ読み直さない（中身が変わっても知らせない）。
        *content.lock().expect("lock") = 1;
        thread::sleep(Duration::from_millis(200));
        assert!(rx.try_recv().is_err());
        assert_eq!(reads.load(Ordering::SeqCst), 1);
        // 後の印（初めは無い file）ができると読み直して 1 件。
        put(&marks[1], "x");
        let frame = rx.recv_timeout(Duration::from_secs(5)).expect("1 件");
        assert!(frame.contains("event: ledger-changed\n"), "{frame}");
        thread::sleep(Duration::from_millis(200));
        assert!(rx.try_recv().is_err(), "印 1 回に 2 件");
        assert_eq!(reads.load(Ordering::SeqCst), 2);
        // 前の印の長さが動いても、読みの結果が同じなら知らせない。
        put(&marks[0], "22");
        let until = Instant::now() + Duration::from_secs(5);
        while reads.load(Ordering::SeqCst) < 3 {
            assert!(
                Instant::now() < until,
                "印 a の変化で 5 秒以内に読み直さない"
            );
            thread::sleep(Duration::from_millis(10));
        }
        thread::sleep(Duration::from_millis(200));
        assert_eq!(reads.load(Ordering::SeqCst), 3);
        assert!(rx.try_recv().is_err(), "同じ読みで知らせる");
        let dir = marks[0].parent().expect("置き場").to_path_buf();
        let _ = std::fs::remove_dir_all(put_dir(&marks[0]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn server_src_watch_rereads_without_marks() {
        let marks = marks("reread");
        let (content, reads, read) = reader();
        let timing = Timing {
            poll: Duration::from_millis(20),
            reread: Duration::from_millis(150),
            store_reread: Duration::from_secs(60),
        };
        let hub = Hub::watch(marks, read, timing);
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
    }

    #[test]
    fn server_read_watch_board_sends_on_marks() {
        let marks = marks("board");
        let dir = marks[0].parent().expect("置き場").to_path_buf();
        let hub = Arc::new(Hub::default());
        let d = dir.clone();
        let files = move || {
            let mut f: Vec<PathBuf> = std::fs::read_dir(&d)
                .expect("置き場")
                .map(|e| e.expect("entry").path())
                .collect();
            f.sort();
            f
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
        let _ = std::fs::remove_dir_all(put_dir(&marks[0]));
        let _ = std::fs::remove_dir_all(&dir);
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
