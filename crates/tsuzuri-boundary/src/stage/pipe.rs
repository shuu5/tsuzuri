//! 席の目の Chrome の pipe の運び手（行 i-4・要件 FR16・判断の記録 ADR-15 の決定 (5)）。
//! 子の標準入力と標準出力（子の側では fd 3 と fd 4 に写す）で、NUL で終わる字の message を運ぶ。
//! 口座の外から届かない（TCP の port も unix socket も開かない・見積りの T3）。
//! 付いた後の message の頭に鍵 sessionId を足す（browser の段に繋がるので、頁の session へ flatten の形で送る）。

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{ChildStdin, ChildStdout};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use super::json;
use super::ws::MAX_MESSAGE;

/// 子の Chrome への pipe（読みは別の thread が NUL までを 1 つの message として渡す）。
pub struct Pipe {
    stdin: ChildStdin,
    replies: Receiver<Result<String, String>>,
    session: Option<String>,
}

impl Pipe {
    /// 子の標準入力と標準出力で pipe を組み、読みの thread を起こす。
    pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Pipe {
        let (tx, replies) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut buf = Vec::new();
                let limit = MAX_MESSAGE as u64 + 1;
                let got = match (&mut reader).take(limit).read_until(0, &mut buf) {
                    Ok(0) => return,
                    Ok(_) if buf.last() == Some(&0) => {
                        buf.pop();
                        String::from_utf8(buf).map_err(|e| format!("UTF-8 でない字の message: {e}"))
                    }
                    Ok(_) => Err(format!(
                        "NUL で終わらないか {MAX_MESSAGE} byte を越える message"
                    )),
                    Err(e) => Err(format!("pipe を読む: {e}")),
                };
                let last = got.is_err();
                if tx.send(got).is_err() || last {
                    return;
                }
            }
        });
        Pipe {
            stdin,
            replies,
            session: None,
        }
    }

    /// 付いた頁の session の id を置く（この後の message の頭に鍵 sessionId を足す）。
    pub fn join(&mut self, session: &str) {
        self.session = Some(session.to_string());
    }

    /// 字を 1 つの message（末に NUL）として書く。
    pub fn send(&mut self, text: &str) -> Result<(), String> {
        let mut bytes = Vec::with_capacity(text.len() + 32);
        match (&self.session, text.strip_prefix('{')) {
            (Some(id), Some(rest)) => {
                bytes.push(b'{');
                bytes.extend_from_slice(format!("\"sessionId\":{},", json::escape(id)).as_bytes());
                bytes.extend_from_slice(rest.as_bytes());
            }
            _ => bytes.extend_from_slice(text.as_bytes()),
        }
        bytes.push(0);
        self.stdin
            .write_all(&bytes)
            .and_then(|()| self.stdin.flush())
            .map_err(|e| format!("pipe へ書く: {e}"))
    }

    /// 次の字の message を timeout まで待つ（読みの thread が終わって受けた字が尽きれば None）。
    pub fn recv(&mut self, timeout: Duration) -> Result<Option<String>, String> {
        match self.replies.recv_timeout(timeout) {
            Ok(got) => got.map(Some),
            Err(RecvTimeoutError::Timeout) => Err("時間切れ".to_string()),
            Err(RecvTimeoutError::Disconnected) => Ok(None),
        }
    }

    /// 標準入力を閉じる（Chrome は pipe の終わりで終わる）。
    pub fn close(self) -> Result<(), String> {
        drop(self.stdin);
        Ok(())
    }
}
