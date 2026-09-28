//! 面の server の最小の形（設計ノート surface-base §9・便 e-min）。
//! 標準 library だけで書く同期の server で、1 つの process と 1 つの port で動く。1 接続 1 thread。
//! 台帳は bd の読み取りの口を子 process で撃って読む（§10・便 e-src）。
//! 読む側の口の 4 つは、台帳と設計の索引と器の event log の字を集めて中核の関数に渡す（§11・便 e-read）。
//! 問いの一覧の口と裁定の受付の口は便 e-ask が足す（`ruling`）。束と方針の受付の口は便 e-batch が足す（`batch`・`policy`）。
//! account board の読みの口と停止の切り替えの口は行 h-wire が足す（`crate::acct`・`crate::accthb`）。
//! POST を受ける口は /api/ruling・/api/batch・/api/policy・/api/account/heartbeat・/api/seat/heartbeat だけで、
//! ほかの GET でない要求は 405 で何も書かない。server 自身は file を書かない（台帳に書くのは bdw・席へ送るのは器の CLI）。
//! 席の card の口は便 e-seat が足す（`seat`）。次の一手の口は、席の card が読めるときは席の card も受けて判じる。
//! 同じ時に届いた要求は、台帳の読みと設計の索引の読みを 1 本の子 process で分け合う（`coalesce`・便 e-coalesce）。
//! 分け合うのは起動で作る 1 つの `Source` と 1 つの `Design` とその clone（変化の見張りの読みも含む）で、
//! 台帳の読めた字は、次の読みが落ちたときだけ `ledger::READ_HOLD`（60 秒）まで返し、その応答の頭
//! （`READ_AGE_HEADER`）に最後に読めた時からの秒を付ける（行 e-hold）。裁定の受付は合流せず、新しい子 process で読み直す。
//! GET の口と POST の 5 つの口は src/server/routes の下に 1 口 1 file で置き（各 file の doc が自分の path を書く）、
//! 口の列 `Route` は組み立ての script が dir から生成する（`route`・判断の記録 ADR-13・行 hb-post）。
//! 変化の知らせ（SSE）はここに在り、どの口にも当たらない GET は面の file の配布。

pub mod batch;
pub mod board;
pub mod clock;
pub mod coalesce;
mod config;
pub mod design;
pub mod events;
pub mod files;
pub mod http;
pub mod ledger;
pub mod policy;
pub mod proc;
pub mod route;
pub mod ruling;
pub mod runs;
pub mod seat;

/// 口の列（組み立ての script が src/server/routes から生成する）。
mod routes {
    include!(concat!(env!("OUT_DIR"), "/routes.rs"));
}

use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Once};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_contract::ledger::READ_AGE_HEADER;
use tsuzuri_contract::surface::{Refusal, RefusalResponse};
use tsuzuri_contract::wire;

use crate::acct::Acct;

use self::board::Sources;
use self::design::Design;
use self::events::Hub;
use self::files::Served;
use self::http::{Request, Response};
use self::ledger::Source;
use self::ruling::{Delivery, Writer};
use self::runs::Runs;
use self::seat::Seats;

pub use self::config::Config;
pub use self::routes::Route;

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
    sources: Sources,
    files: PathBuf,
    hub: Arc<Hub>,
    writer: Writer,
    seats: Seats,
    /// account board の読み（state dir が無ければ None）。
    acct: Option<Arc<Acct>>,
    /// acct の変化の印の一覧（初めは空・口 /api/account の最初の要求から `ACCT_MARKS_EVERY` ごとに取り直す）。
    acct_marks: Arc<Mutex<Vec<PathBuf>>>,
    /// acct の印の取り直しを 1 回だけ始める。
    acct_watch: Once,
}

/// 既定の git の program の名（account board の読みが anchor の state dir を引く）。
pub const GIT: &str = "git";

/// acct の変化の印の一覧を取り直す間隔（一覧が変わるのは席が移ったときだけ・file の変化は `events::POLL` ごとに見る）。
pub const ACCT_MARKS_EVERY: Duration = Duration::from_secs(60);

/// 要求の頭を読む時間の上限。
const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// 応答を書く時間の上限（止まった相手の接続を片付ける）。
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

impl Server {
    /// bind 先を判定し、置き場を確かめ、口を開き、台帳の周期の読みと板の印の見張りを始める
    /// （最初の読みは戻る前に取るので、bd が返さなければ `ledger::BD_TIMEOUT` まで待つ）。
    /// state dir は確かめない（無い置き場は走行の出所が読めない扱い）。git の program は `GIT`。
    pub fn bind(config: &Config) -> Result<Server, StartError> {
        Server::bind_with(config, OsStr::new(GIT))
    }

    /// `bind` と同じで、account board の読みが撃つ git の program を受ける。
    /// state dir が在るときだけ account board の読みを作る（器は口 /api/account の要求まで撃たない）。
    pub fn bind_with(config: &Config, git: &OsStr) -> Result<Server, StartError> {
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
        let sources = Sources {
            ledger: Source::new(&config.repo, &config.bd),
            design: Design::new(&config.repo, &config.folio),
            runs: Runs::new(config.state_dir.as_deref()),
        };
        let seats = Seats::new(
            &config.scribe2,
            config.state_dir.as_deref(),
            config.seat.as_deref(),
            &config.repo,
        );
        let (design, runs) = (sources.design.clone(), sources.runs.clone());
        let seat_marks = seats.marks();
        let acct = config.state_dir.as_ref().map(|state_dir| {
            Arc::new(Acct::new(
                config.scribe2.clone(),
                git,
                config.bd.clone(),
                state_dir.clone(),
                config.repo.clone(),
            ))
        });
        let acct_marks = Arc::new(Mutex::new(Vec::new()));
        let held_marks = Arc::clone(&acct_marks);
        let hub = Hub::start(sources.ledger.clone(), move || {
            let mut marks = runs.marks();
            marks.extend(design.marks());
            marks.extend(seat_marks.iter().cloned());
            marks.extend(lock(&held_marks).iter().cloned());
            marks
        });
        let delivery = match (&config.seat, &config.state_dir) {
            (Some(target), Some(state_dir)) => Some(Delivery {
                program: config.scribe2.clone(),
                state_dir: state_dir.clone(),
                target: target.clone(),
            }),
            _ => None,
        };
        let writer = Writer {
            repo: config.repo.clone(),
            bdw: config.bdw.clone(),
            delivery,
        };
        Ok(Server {
            listener,
            shared: Arc::new(Shared {
                sources,
                files,
                hub,
                writer,
                seats,
                acct,
                acct_marks,
                acct_watch: Once::new(),
            }),
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
        // 1 byte も届かずに閉じたか時間切れの接続（browser が先に張る接続）には何も書かない
        // （400 を書くと、使い回した接続で後の要求の応答として読まれる）。
        Err(e) if http::no_bytes(&e) => return,
        Err(_) => Response::text(400, "bad-request"),
        Ok(req) if req.method == "GET" && req.path() == "/api/surface/events" => {
            let _ = events::stream(&stream, &shared.hub);
            return;
        }
        // 口（`route::dispatch`・method を問わない）に当たらなければ、GET でない要求は 405、GET は面の file の配布。
        Ok(req) => match route::dispatch(&req, shared) {
            Some(response) => response,
            None if req.method != "GET" => Response::text(405, "method").header("Allow", "GET"),
            None => match files::serve(&shared.files, req.path()) {
                Served::File { content_type, body } => Response::new(200, content_type, body),
                Served::Outside => Response::text(403, "outside"),
                Served::Missing => Response::text(404, "no-file"),
            },
        },
    };
    let _ = response.write_to(&stream);
}

/// POST の口の守り（Origin が Host と違えば 403・本文が `http::BODY_MAX` を越えれば 413・
/// 契約の型として読めない本文は 400 bad-body）。通れば `decode` で読んだ本文
/// （境界の crate は serde に直接依存しないので、読みは呼ぶ側が `wire::decode` で渡す）。
fn guarded<T>(req: &Request, decode: impl FnOnce(&str) -> Option<T>) -> Result<T, Response> {
    if !req.same_origin() {
        return Err(Response::text(403, "origin"));
    }
    if req.too_large() {
        return Err(Response::text(413, "too-large"));
    }
    std::str::from_utf8(&req.body)
        .ok()
        .and_then(decode)
        .ok_or_else(|| Response::text(400, "bad-body"))
}

/// 断りの応答（状態の code は `Refusal::http_status`・本文は RefusalResponse）。
fn refusal(reason: Refusal) -> Response {
    json(
        reason.http_status(),
        wire::encode(&RefusalResponse { reason }),
    )
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// 台帳の読みが落ちて最後に読めた字を返した応答に、頭 `READ_AGE_HEADER` で最後に読めた時からの秒を足す
/// （`stale` が None なら応答をそのまま返す・行 e-hold）。
fn aged(response: Response, stale: Option<Instant>) -> Response {
    match stale {
        Some(at) => response.header(READ_AGE_HEADER, at.elapsed().as_secs().to_string()),
        None => response,
    }
}

/// 電文の字の応答（字は契約の型の crate の `wire` が作る・境界の crate は serde に直接依存しない）。
fn json(status: u16, encoded: Result<String, wire::Error>) -> Response {
    match encoded {
        Ok(body) => Response::json(status, body),
        Err(_) => Response::text(500, "encode"),
    }
}
