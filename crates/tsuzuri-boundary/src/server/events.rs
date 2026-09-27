//! 変化の知らせ（口 GET /api/surface/events・SSE）。
//! 台帳の file の更新時刻と長さを 500 ミリ秒ごとに見て、変わったら接続中の全員に 1 件ずつ送る
//! （規則の行 R-21: 合図なしは周期の読み 500 ms ごとで 1.5 秒以内・要件 NFR2）。

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tsuzuri_contract::ledger::{LEDGER_CHANGED_EVENT, LedgerChanged};
use tsuzuri_contract::wire;

/// 周期の読みの間隔（規則の行 R-21）。
pub const POLL: Duration = Duration::from_millis(500);

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
    /// 台帳の file の周期の読みを始める（Hub が落ちれば読みも止まる）。
    pub fn start(ledger: PathBuf) -> Arc<Hub> {
        let hub = Arc::new(Hub::default());
        let weak = Arc::downgrade(&hub);
        thread::spawn(move || watch(&weak, &ledger));
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

fn watch(hub: &Weak<Hub>, ledger: &Path) {
    let mut last = stamp(ledger);
    loop {
        thread::sleep(POLL);
        let Some(hub) = hub.upgrade() else {
            return;
        };
        let current = stamp(ledger);
        if current != last {
            last = current;
            hub.ledger_changed(now());
        }
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
    use super::Hub;

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
