//! 変化の知らせ（口 GET /api/surface/events・SSE・便 e-src）。
//! 変化の印（.beads の issues.jsonl と interactions.jsonl の更新時刻と長さ）を 500 ミリ秒ごとに見て、
//! どちらかが動いたら台帳を読み直す（規則の行 R-21: 合図なしは周期の読み 500 ms ごとで 1.5 秒以内・要件 NFR2）。
//! 印の取りこぼしを拾うために、印が動かなくても 5 秒ごとに読み直す。
//! 読みの結果が前と変わったときだけ、接続中の全員に 1 件ずつ送る。

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_contract::ledger::{LEDGER_CHANGED_EVENT, LedgerChanged};
use tsuzuri_contract::wire;

use super::ledger::Source;

/// 印を見る間隔（規則の行 R-21）。
pub const POLL: Duration = Duration::from_millis(500);

/// 印が動かなくても読み直す間隔（取りこぼしを拾う）。
pub const REREAD: Duration = Duration::from_secs(5);

/// 周期の読みの 2 つの間隔。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    /// 印を見る間隔。
    pub poll: Duration,
    /// 印が動かなくても読み直す間隔。
    pub reread: Duration,
}

/// server の周期の読みの間隔。
pub const TIMING: Timing = Timing {
    poll: POLL,
    reread: REREAD,
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
    /// 台帳の周期の読みを始める（Hub が落ちれば読みも止まる）。
    pub fn start(source: Source) -> Arc<Hub> {
        Hub::watch(source.marks(), move || source.read(), TIMING)
    }

    /// 印の file と読みの関数で周期の読みを始める。最初の印と読みは戻る前に取る
    /// （戻った後の変化は取りこぼさない）。
    pub fn watch<R, F>(marks: Vec<PathBuf>, mut read: F, timing: Timing) -> Arc<Hub>
    where
        R: PartialEq + Send + 'static,
        F: FnMut() -> R + Send + 'static,
    {
        let hub = Arc::new(Hub::default());
        let weak = Arc::downgrade(&hub);
        let seen = stamps(&marks);
        let read_at = Instant::now();
        let last = read();
        let state = Watch {
            marks,
            seen,
            read_at,
            last,
        };
        thread::spawn(move || watch(&weak, state, read, timing));
        hub
    }

    /// 受け手を 1 人足す。
    pub fn subscribe(&self) -> Receiver<String> {
        let (tx, rx) = mpsc::channel();
        lock(&self.subscribers).push(tx);
        rx
    }

    /// 台帳の変化を全員に送る（切れた受け手は外す）。
    pub fn ledger_changed(&self, at: u64) {
        let data = wire::encode(&LedgerChanged { at }).expect("LedgerChanged は JSON になる");
        let id = {
            let mut seq = lock(&self.seq);
            *seq += 1;
            *seq
        };
        let frame = format!("id: {id}\nevent: {LEDGER_CHANGED_EVENT}\ndata: {data}\n\n");
        lock(&self.subscribers).retain(|tx| tx.send(frame.clone()).is_ok());
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// 印の file の全部の更新時刻と長さ。
fn stamps(marks: &[PathBuf]) -> Vec<Option<(SystemTime, u64)>> {
    marks.iter().map(|m| stamp(m)).collect()
}

/// 周期の読みの状態（見た印・最後に読んだ時刻・最後の読みの結果）。
struct Watch<R> {
    marks: Vec<PathBuf>,
    seen: Vec<Option<(SystemTime, u64)>>,
    read_at: Instant,
    last: R,
}

fn watch<R: PartialEq>(
    hub: &Weak<Hub>,
    mut state: Watch<R>,
    mut read: impl FnMut() -> R,
    timing: Timing,
) {
    loop {
        thread::sleep(timing.poll);
        if hub.strong_count() == 0 {
            return;
        }
        // 印は読みの前に取る（読みの途中の変化は次の周で拾う）。
        let current = stamps(&state.marks);
        if current == state.seen && state.read_at.elapsed() < timing.reread {
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
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    /// 印の 2 つの file の置き場（2 つめは初めは無い）。
    fn marks(name: &str) -> Vec<PathBuf> {
        let dir = std::env::temp_dir().join(format!("tz-events-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("置き場");
        std::fs::write(dir.join("a"), "1").expect("印 a");
        vec![dir.join("a"), dir.join("b")]
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
        std::fs::write(&marks[1], "x").expect("印 b");
        let frame = rx.recv_timeout(Duration::from_secs(1)).expect("1 件");
        assert!(frame.contains("event: ledger-changed\n"), "{frame}");
        thread::sleep(Duration::from_millis(200));
        assert!(rx.try_recv().is_err(), "印 1 回に 2 件");
        assert_eq!(reads.load(Ordering::SeqCst), 2);
        // 前の印の長さが動いても、読みの結果が同じなら知らせない。
        std::fs::write(&marks[0], "22").expect("印 a");
        thread::sleep(Duration::from_millis(200));
        assert_eq!(reads.load(Ordering::SeqCst), 3);
        assert!(rx.try_recv().is_err(), "同じ読みで知らせる");
    }

    #[test]
    fn server_src_watch_rereads_without_marks() {
        let marks = marks("reread");
        let (content, reads, read) = reader();
        let timing = Timing {
            poll: Duration::from_millis(20),
            reread: Duration::from_millis(150),
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
