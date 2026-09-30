//! websocket の client の側（行 i-3・RFC 6455）。
//! 行 i-2 が ssh の -L で張る unix socket に繋ぎ、字の message だけを運ぶ（binary と拡張と subprotocol は持たない）。
//! 外の部品を足さず、SHA-1 と base64 と乱れた値（std の RandomState）を std だけで書く。
//! 字を運ぶ `Socket::send` を使うのは cdp の Session だけ（撃てる命令の語彙は cdp の steps が決める・
//! 窓を足す手は動いている Chrome へ渡す起動の引数で、頁の target を作る命令は撃たない・行 i-board-win）。

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

/// 握手の応答の鍵を組む字（RFC 6455 の 1.3 節）。
pub const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// 1 つの字の message の上限（64 MiB・写真の base64 の字を受ける）。
pub const MAX_MESSAGE: usize = 67_108_864;

/// 握手の応答の頭の上限。
const HEAD_MAX: usize = 8192;

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const TEXT: u8 = 1;
const CLOSE: u8 = 8;
const PING: u8 = 9;
const PONG: u8 = 10;

/// 鍵に応える Sec-WebSocket-Accept の字（鍵と GUID をつないだ字の SHA-1 の base64）。
pub fn accept(key: &str) -> String {
    base64(&sha1(format!("{key}{GUID}").as_bytes()))
}

/// 握手を終えた websocket の接続。
pub struct Socket {
    stream: UnixStream,
    /// 握手の応答の頭の後に続けて届いた byte（最初の frame の読みに使う）。
    carry: Vec<u8>,
    /// 自分が close を送ったか。
    closed: bool,
}

impl Socket {
    /// unix socket の path に繋ぎ、resource へ握手する（読みと書きの上限は timeout）。
    pub fn connect(path: &Path, resource: &str, timeout: Duration) -> Result<Socket, String> {
        if resource.is_empty() || resource.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(format!("resource の字 {resource:?}"));
        }
        let place = path.display();
        let mut stream = UnixStream::connect(path).map_err(|e| format!("{place} に繋ぐ: {e}"))?;
        stream
            .set_read_timeout(Some(timeout))
            .and_then(|()| stream.set_write_timeout(Some(timeout)))
            .map_err(|e| format!("{place} の上限の時間: {e}"))?;
        let mut nonce = [0u8; 16];
        nonce[..8].copy_from_slice(&random8());
        nonce[8..].copy_from_slice(&random8());
        let key = base64(&nonce);
        let request = format!(
            "GET {resource} HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("{place} へ握手を書く: {e}"))?;
        let mut head = Vec::new();
        let mut chunk = [0u8; 1024];
        let end = loop {
            if let Some(at) = head.windows(4).position(|w| w == b"\r\n\r\n") {
                break at;
            }
            if head.len() > HEAD_MAX {
                return Err(format!("{place} の握手の応答の頭が {HEAD_MAX} byte を越える"));
            }
            let n = stream
                .read(&mut chunk)
                .map_err(|e| format!("{place} の握手の応答を読む: {e}"))?;
            if n == 0 {
                return Err(format!("{place} が握手の応答の前に閉じた"));
            }
            head.extend_from_slice(&chunk[..n]);
        };
        let carry = head.split_off(end + 4);
        let text = String::from_utf8_lossy(&head);
        let mut lines = text.split("\r\n");
        let status = lines.next().unwrap_or("");
        if status.split_whitespace().nth(1) != Some("101") {
            return Err(format!("{place} の握手の状態の行 {status:?}"));
        }
        let want = accept(&key);
        let agreed = lines.filter_map(|l| l.split_once(':')).any(|(name, value)| {
            name.trim().eq_ignore_ascii_case("Sec-WebSocket-Accept") && value.trim() == want
        });
        if !agreed {
            return Err(format!("{place} の Sec-WebSocket-Accept が無いか違う"));
        }
        Ok(Socket {
            stream,
            carry,
            closed: false,
        })
    }

    /// 字を 1 つの frame（FIN と opcode 1・mask つき）で送る。
    pub fn send(&mut self, text: &str) -> Result<(), String> {
        self.write_frame(TEXT, text.as_bytes())
    }

    /// 次の字の message を読む（ping には pong を返し、pong は読み捨てる・相手の close には close を返して None）。
    pub fn recv(&mut self) -> Result<Option<String>, String> {
        let mut text: Option<Vec<u8>> = None;
        loop {
            let before = text.as_ref().map_or(0, Vec::len);
            let (fin, opcode, payload) = self.read_frame(before)?;
            match opcode {
                TEXT => {
                    if text.is_some() {
                        return Err("字の message の途中の opcode 1".to_string());
                    }
                    if fin {
                        return utf8(payload).map(Some);
                    }
                    text = Some(payload);
                }
                0 => {
                    let Some(buf) = text.as_mut() else {
                        return Err("前の字の無い opcode 0".to_string());
                    };
                    buf.extend_from_slice(&payload);
                    if fin {
                        return utf8(text.take().unwrap_or_default()).map(Some);
                    }
                }
                PING => self.write_frame(PONG, &payload)?,
                PONG => {}
                CLOSE => {
                    if !self.closed {
                        self.write_frame(CLOSE, &payload[..payload.len().min(2)])?;
                        self.closed = true;
                    }
                    return Ok(None);
                }
                _ => return Err(format!("受けない opcode {opcode}")),
            }
        }
    }

    /// 状態の code 1000 の close を送り、相手の close か接続の終わりか誤りまで読む。
    pub fn close(mut self) -> Result<(), String> {
        if self.closed {
            return Ok(());
        }
        self.write_frame(CLOSE, &1000u16.to_be_bytes())?;
        self.closed = true;
        loop {
            match self.read_frame(0) {
                Ok((_, CLOSE, _)) | Err(_) => return Ok(()),
                Ok(_) => {}
            }
        }
    }

    /// 次の読みの上限の時間を替える（cdp の Session が待ちの残りの時間を渡す）。
    pub(crate) fn wait(&self, timeout: Duration) -> Result<(), String> {
        self.stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| format!("読みの上限の時間: {e}"))
    }

    /// 1 つの frame を mask つきで書く（長さは最も短い形）。
    fn write_frame(&mut self, opcode: u8, payload: &[u8]) -> Result<(), String> {
        let mut frame = Vec::with_capacity(payload.len() + 14);
        frame.push(0x80 | opcode);
        match payload.len() {
            n @ 0..=125 => frame.push(0x80 | n as u8),
            n @ 126..=65535 => {
                frame.push(0x80 | 126);
                frame.extend_from_slice(&(n as u16).to_be_bytes());
            }
            n => {
                frame.push(0x80 | 127);
                frame.extend_from_slice(&(n as u64).to_be_bytes());
            }
        }
        let random = random8();
        let key = [random[0], random[1], random[2], random[3]];
        frame.extend_from_slice(&key);
        frame.extend(payload.iter().enumerate().map(|(i, b)| b ^ key[i % 4]));
        self.stream
            .write_all(&frame)
            .map_err(|e| format!("websocket へ書く: {e}"))
    }

    /// 1 つの frame を読み（FIN・opcode・payload）、形の外れた frame は payload を読む前に断る。
    /// before はその message で既に読んだ長さ。
    fn read_frame(&mut self, before: usize) -> Result<(bool, u8, Vec<u8>), String> {
        let mut head = [0u8; 2];
        self.fill(&mut head)?;
        let fin = head[0] & 0x80 != 0;
        if head[0] & 0x70 != 0 {
            return Err("server の frame に RSV の bit".to_string());
        }
        let opcode = head[0] & 0x0f;
        if !matches!(opcode, 0 | TEXT | CLOSE | PING | PONG) {
            return Err(format!("受けない opcode {opcode}"));
        }
        if head[1] & 0x80 != 0 {
            return Err("server の frame に MASK の bit".to_string());
        }
        let len = match head[1] & 0x7f {
            126 => {
                let mut b = [0u8; 2];
                self.fill(&mut b)?;
                u64::from(u16::from_be_bytes(b))
            }
            127 => {
                let mut b = [0u8; 8];
                self.fill(&mut b)?;
                u64::from_be_bytes(b)
            }
            n => u64::from(n),
        };
        if opcode >= CLOSE && (!fin || len > 125) {
            return Err(format!("control の frame の形（opcode {opcode}）"));
        }
        if len.saturating_add(before as u64) > MAX_MESSAGE as u64 {
            return Err(format!("message が {MAX_MESSAGE} byte を越える"));
        }
        let mut payload = vec![0u8; len as usize];
        self.fill(&mut payload)?;
        Ok((fin, opcode, payload))
    }

    /// 持ち越した byte から先に、足りない分を接続から読む。
    fn fill(&mut self, buf: &mut [u8]) -> Result<(), String> {
        let n = buf.len().min(self.carry.len());
        buf[..n].copy_from_slice(&self.carry[..n]);
        self.carry.drain(..n);
        self.stream
            .read_exact(&mut buf[n..])
            .map_err(|e| format!("websocket を読む: {e}"))
    }
}

fn utf8(bytes: Vec<u8>) -> Result<String, String> {
    String::from_utf8(bytes).map_err(|e| format!("UTF-8 でない字の message: {e}"))
}

/// 乱れた 8 byte（RandomState の新しい値の hasher の finish）。
fn random8() -> [u8; 8] {
    RandomState::new().build_hasher().finish().to_le_bytes()
}

/// 字 = の詰めを付ける base64。
fn base64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let n = (u32::from(chunk[0]) << 16) | (u32::from(b1) << 8) | u32::from(b2);
        for (i, shift) in [18u32, 12, 6, 0].into_iter().enumerate() {
            if i <= chunk.len() {
                out.push(char::from(BASE64[((n >> shift) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// SHA-1 の 20 byte。
fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x6745_2301, 0xEFCD_AB89, 0x98BA_DCFE, 0x1032_5476, 0xC3D2_E1F0];
    let mut msg = data.to_vec();
    let bits = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bits.to_be_bytes());
    for block in msg.as_chunks::<64>().0 {
        let mut w = [0u32; 80];
        for (slot, word) in w.iter_mut().zip(block.as_chunks::<4>().0) {
            *slot = u32::from_be_bytes(*word);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = h;
        for (i, word) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let t = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = t;
        }
        for (slot, v) in h.iter_mut().zip([a, b, c, d, e]) {
            *slot = slot.wrapping_add(v);
        }
    }
    let mut out = [0u8; 20];
    for (dst, v) in out.as_chunks_mut::<4>().0.iter_mut().zip(h) {
        *dst = v.to_be_bytes();
    }
    out
}
