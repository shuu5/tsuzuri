//! 面の server の最小の形（設計ノート surface-base §9・便 e-min）。
//! 標準 library だけで書く同期の server で、1 つの process と 1 つの port で動く。1 接続 1 thread。
//! 口は 4 つで、書く口は持たない（GET でない要求は 405 で何も書かない）。
//! - GET /api/ledger — 台帳の一覧（LedgerList）
//! - GET /api/ledger/<id> — 台帳の 1 本（LedgerItem）
//! - GET /api/surface/events — 変化の知らせ（SSE）
//! - それ以外の GET — 面の file の配布

pub mod events;
pub mod files;
pub mod http;
pub mod ledger;

use std::fmt;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::wire;

use self::events::Hub;
use self::files::Served;
use self::http::Response;

/// 起動の引数（repo の置き場・bind 先・面の file の置き場）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub repo: PathBuf,
    pub bind: SocketAddr,
    pub files: PathBuf,
}

/// tailnet の IPv4 の範囲（100.64.0.0/10）。
pub const TAILNET_V4: (Ipv4Addr, u32) = (Ipv4Addr::new(100, 64, 0, 0), 10);

/// tailnet の IPv6 の範囲（fd7a:115c:a1e0::/48）。
pub const TAILNET_V6: (Ipv6Addr, u32) = (Ipv6Addr::new(0xfd7a, 0x115c, 0xa1e0, 0, 0, 0, 0, 0), 48);

/// bind 先の判定（条 N-6.1）: 同じ端末の中（loopback）か tailnet の中だけを通す。
/// 全部の宛先（0.0.0.0・::）も LAN も公開の住所も断る。tailnet の住所は引数で受け、file に書かない（行 D-4）。
pub fn bind_allowed(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(v4) => {
            let (net, len) = TAILNET_V4;
            let mask = u32::MAX << (32 - len);
            v4.is_loopback() || u32::from(v4) & mask == u32::from(net) & mask
        }
        IpAddr::V6(v6) => {
            let (net, len) = TAILNET_V6;
            let mask = u128::MAX << (128 - len);
            v6.is_loopback() || u128::from(v6) & mask == u128::from(net) & mask
        }
    }
}

/// 起動を断る理由。
#[derive(Debug)]
pub enum StartError {
    /// bind 先が loopback でも tailnet でもない。
    BindRefused(SocketAddr),
    /// 置き場が dir でない。
    NotDir {
        what: &'static str,
        path: PathBuf,
    },
    Io(io::Error),
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StartError::BindRefused(addr) => write!(
                f,
                "bind 先 {addr} は同じ端末の中（loopback）でも tailnet の中でもない（条 N-6）"
            ),
            StartError::NotDir { what, path } => {
                write!(f, "{what}の置き場 {} が dir でない", path.display())
            }
            StartError::Io(e) => write!(f, "bind できない: {e}"),
        }
    }
}

impl std::error::Error for StartError {}

/// 口を開いた server（`run` で要求を受け始める）。
pub struct Server {
    listener: TcpListener,
    shared: Arc<Shared>,
}

struct Shared {
    ledger: PathBuf,
    files: PathBuf,
    hub: Arc<Hub>,
}

/// 要求の頭を読む時間の上限。
const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// 応答を書く時間の上限（止まった相手の接続を片付ける）。
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

impl Server {
    /// bind 先を判定し、置き場を確かめ、口を開き、台帳の周期の読みを始める。
    pub fn bind(config: &Config) -> Result<Server, StartError> {
        if !bind_allowed(config.bind.ip()) {
            return Err(StartError::BindRefused(config.bind));
        }
        if !config.repo.is_dir() {
            return Err(StartError::NotDir {
                what: "repo ",
                path: config.repo.clone(),
            });
        }
        let files = config
            .files
            .canonicalize()
            .ok()
            .filter(|p| p.is_dir())
            .ok_or_else(|| StartError::NotDir {
                what: "面の file ",
                path: config.files.clone(),
            })?;
        let listener = TcpListener::bind(config.bind).map_err(StartError::Io)?;
        let ledger = ledger::path(&config.repo);
        let hub = Hub::start(ledger.clone());
        Ok(Server {
            listener,
            shared: Arc::new(Shared { ledger, files, hub }),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// 要求を受け続ける（戻らない）。
    pub fn run(self) -> ! {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let shared = Arc::clone(&self.shared);
                    thread::spawn(move || handle(stream, &shared));
                }
                Err(e) => eprintln!("tz surface serve: accept: {e}"),
            }
        }
    }
}

fn handle(stream: TcpStream, shared: &Shared) {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
    let response = match http::read_request(&stream) {
        Err(_) => Response::text(400, "bad-request"),
        Ok(req) if req.method != "GET" => Response::text(405, "method").header("Allow", "GET"),
        Ok(req) if req.path() == "/api/surface/events" => {
            let _ = events::stream(&stream, &shared.hub);
            return;
        }
        Ok(req) => route(req.path(), shared),
    };
    let _ = response.write_to(&stream);
}

/// GET の口を選ぶ。
fn route(path: &str, shared: &Shared) -> Response {
    if path == "/api/ledger" {
        return json(200, wire::encode(&ledger::list(&shared.ledger)));
    }
    if let Some(id) = path.strip_prefix("/api/ledger/") {
        let Ok(id) = BeadId::new(id) else {
            return Response::text(400, "id-shape");
        };
        return match ledger::item(&shared.ledger, &id) {
            ledger::Lookup::Found(item) => json(200, wire::encode(&item)),
            ledger::Lookup::Missing => Response::text(404, "no-item"),
            ledger::Lookup::Unknown => Response::text(503, "ledger-unknown"),
        };
    }
    match files::serve(&shared.files, path) {
        Served::File { content_type, body } => Response::new(200, content_type, body),
        Served::Outside => Response::text(403, "outside"),
        Served::Missing => Response::text(404, "no-file"),
    }
}

/// 電文の字の応答（字は契約の型の crate の `wire` が作る・境界の crate は serde に直接依存しない）。
fn json(status: u16, encoded: Result<String, wire::Error>) -> Response {
    match encoded {
        Ok(body) => Response::json(status, body),
        Err(_) => Response::text(500, "encode"),
    }
}
