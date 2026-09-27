//! 面の server の最小の形（設計ノート surface-base §9・便 e-min）。
//! 標準 library だけで書く同期の server で、1 つの process と 1 つの port で動く。1 接続 1 thread。
//! 台帳は bd の読み取りの口を子 process で撃って読む（§10・便 e-src）。
//! 読む側の口の 4 つは、台帳と設計の索引と器の event log の字を集めて中核の関数に渡す（§11・便 e-read）。
//! 問いの一覧の口と裁定の受付の口は便 e-ask が足す（`ruling`）。束と方針の受付の口は便 e-batch が足す（`batch`・`policy`）。
//! account board の読みの口と停止の切り替えの口は行 h-wire が足す（`crate::acct`・`crate::accthb`）。
//! POST を受ける口は /api/ruling・/api/batch・/api/policy・/api/account/heartbeat だけで、
//! ほかの GET でない要求は 405 で何も書かない。server 自身は file を書かない（台帳に書くのは bdw・席へ送るのは器の CLI）。
//! 席の card の口は便 e-seat が足す（`seat`）。次の一手の口は、席の card が読めるときは席の card も受けて判じる。
//! 同じ時に届いた要求は、台帳の読みと設計の索引の読みを 1 本の子 process で分け合う（`coalesce`・便 e-coalesce）。
//! 分け合うのは起動で作る 1 つの `Source` と 1 つの `Design` とその clone（変化の見張りの読みも含む）で、
//! 読み終えた字は次の要求に持ち回さない。裁定の受付は合流せず、新しい子 process で読み直す。
//! - GET /api/ledger — 台帳の一覧（LedgerList）
//! - GET /api/ledger/<id> — 台帳の 1 本（LedgerItem）
//! - GET /api/pipeline — pipeline の板（PipelineBoard）
//! - GET /api/metrics — 台帳の指標（LedgerStats）
//! - GET /api/next — 次の一手（NextStep）
//! - GET /api/graph — 導出グラフ（GraphDoc）
//! - GET /api/graph/view — 地図のグラフの眺め（GraphView・便 e-view）
//! - GET /api/around?id=&k=&fold= — 節点の近傍（AroundDoc・便 e-view）
//! - GET /api/unreflected — 未反映の一覧（UnreflectedList・便 e-view）
//! - GET /api/questions — 問いの一覧（QuestionList）
//! - GET /api/seat — 席の card（SeatCard）
//! - POST /api/ruling — 裁定の受付（RulingRequest → RulingResponse か RefusalResponse）
//! - POST /api/batch — 束の受付（BatchRequest → BatchResponse か RefusalResponse）
//! - POST /api/policy — 方針の受付（PolicyRequest → PolicyResponse か RefusalResponse）
//! - GET /api/account — account board の読み（AccountDoc・行 h-wire）
//! - POST /api/account/heartbeat — 停止の切り替えの受付（HeartbeatRequest → HeartbeatResponse か字・行 h-wire）
//! - GET /api/surface/events — 変化の知らせ（SSE）
//! - それ以外の GET — 面の file の配布

pub mod batch;
pub mod board;
pub mod coalesce;
pub mod design;
pub mod events;
pub mod files;
pub mod http;
pub mod ledger;
pub mod policy;
pub mod ruling;
pub mod runs;
pub mod seat;

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Once, Weak};
use std::thread;
use std::time::Duration;

use tsuzuri_contract::account::{HEARTBEAT_PATH, PATH as ACCOUNT_PATH};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::Fold;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::{
    BatchRequest, PolicyRequest, Refusal, RefusalResponse, RulingRequest,
};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::around::{AROUND_STEPS, AROUND_STEPS_RANGE};

use crate::acct::Acct;
use crate::accthb;

use self::board::Sources;
use self::design::Design;
use self::events::Hub;
use self::files::Served;
use self::http::{Request, Response};
use self::ledger::Source;
use self::ruling::{Delivery, Outcome, Writer};
use self::runs::Runs;
use self::seat::Seats;

/// 起動の引数（repo の置き場・bind 先・面の file の置き場・bd の program・器の state dir・設計の道具の program・
/// bdw の program・席の target・器の CLI の program）。席の target と state dir の両方が在るときだけ、
/// 裁定を席へ配達し、席の card を組む（便 e-seat）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub repo: PathBuf,
    pub bind: SocketAddr,
    pub files: PathBuf,
    /// 台帳の読みに撃つ program（既定は `ledger::BD`）。
    pub bd: OsString,
    /// 器の state dir（None なら走行の出所は読めず、裁定を席へ配達しない）。
    pub state_dir: Option<PathBuf>,
    /// 設計の索引の読みに撃つ program（既定は `design::FOLIO`）。
    pub folio: OsString,
    /// 台帳の書きに撃つ program（既定は `tsuzuri_contract::ledger::BDW`・便 e-ask）。
    pub bdw: OsString,
    /// 裁定を配達し、card を組む席の target（None なら配達せず、card を組まない）。
    pub seat: Option<String>,
    /// 配達と席の読みに撃つ器の CLI の program（既定は `ruling::SCRIBE2`）。
    pub scribe2: OsString,
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
        Err(_) => Response::text(400, "bad-request"),
        Ok(req) if req.method == "POST" && req.path() == ruling::PATH => post_ruling(&req, shared),
        Ok(req) if req.method == "POST" && req.path() == batch::PATH => post_batch(&req, shared),
        Ok(req) if req.method == "POST" && req.path() == policy::PATH => post_policy(&req, shared),
        Ok(req) if req.method == "POST" && req.path() == HEARTBEAT_PATH => {
            post_heartbeat(&req, shared)
        }
        Ok(req) if req.method != "GET" => Response::text(405, "method").header("Allow", "GET"),
        Ok(req) if req.path() == "/api/surface/events" => {
            let _ = events::stream(&stream, &shared.hub);
            return;
        }
        Ok(req) => route(&req, shared),
    };
    let _ = response.write_to(&stream);
}

/// GET の口を選ぶ。
fn route(req: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let path = req.path();
    match path {
        "/api/ledger" => return json(200, wire::encode(&ledger::list(&sources.ledger))),
        "/api/pipeline" => {
            let texts = sources.gather(false, true);
            return json(200, wire::encode(&board::pipeline(&texts, events::now())));
        }
        "/api/metrics" => {
            let texts = sources.gather(false, false);
            return json(200, wire::encode(&board::metrics(&texts, events::now())));
        }
        "/api/next" => {
            // 席の card と台帳の字は並べて集める（待ちは 1 本分の上限まで）。
            let now = events::now();
            let (texts, card) = thread::scope(|s| {
                let card = s.spawn(|| shared.seats.known(now));
                let texts = sources.gather(false, true);
                (texts, card.join().ok().flatten())
            });
            let step = match &card {
                Some(card) => board::next_seat(&texts, card, now),
                None => board::next(&texts, now),
            };
            return json(200, wire::encode(&step));
        }
        seat::PATH => return json(200, wire::encode(&shared.seats.card(events::now()))),
        ACCOUNT_PATH => return account(shared),
        "/api/graph" => {
            let texts = sources.gather(true, true);
            return json(200, wire::encode(&board::graph(&texts)));
        }
        "/api/graph/view" => {
            let texts = sources.gather(true, true);
            return json(200, wire::encode(&board::view(&texts)));
        }
        "/api/around" => return around(req, sources),
        "/api/unreflected" => {
            let texts = sources.gather(false, false);
            return json(
                200,
                wire::encode(&board::unreflected(&texts, events::now())),
            );
        }
        "/api/questions" => {
            let text = sources.ledger.text().unwrap_or_default();
            return json(200, wire::encode(&tsuzuri_core::question::list(&text)));
        }
        _ => {}
    }
    if let Some(id) = path.strip_prefix("/api/ledger/") {
        let Ok(id) = BeadId::new(id) else {
            return Response::text(400, "id-shape");
        };
        return match ledger::item(&sources.ledger, &id) {
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

/// 節点の近傍（query は id・k・fold の順に読み、最初に当たった断りを返す）。
/// id が無いか空は 400 no-id・k が 1 から 3 の整数でなければ 400 steps・fold が 4 つの字のどれでもなければ 400 fold・
/// 節点が無ければ 404 no-node。字を集めるのは query が読めた後だけ。
fn around(req: &Request, sources: &Sources) -> Response {
    let Some(id) = req.query("id").filter(|id| !id.is_empty()) else {
        return Response::text(400, "no-id");
    };
    let steps = match req.query("k") {
        None => AROUND_STEPS,
        Some(k) => match k.parse::<u8>() {
            Ok(k) if AROUND_STEPS_RANGE.contains(&k) => k,
            _ => return Response::text(400, "steps"),
        },
    };
    let fold = match req.query("fold").as_deref() {
        None | Some("none") => Fold::None,
        Some("up") => Fold::Up,
        Some("down") => Fold::Down,
        Some("both") => Fold::Both,
        Some(_) => return Response::text(400, "fold"),
    };
    let texts = sources.gather(true, true);
    match board::around(&texts, &id, steps, fold) {
        Some(doc) => json(200, wire::encode(&doc)),
        None => Response::text(404, "no-node"),
    }
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

/// 裁定の受付（守りは `guarded`）。断りと 4xx と 5xx は、notes への追記の前なら何も書いていない。
fn post_ruling(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<RulingRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    let outcome = ruling::accept(&body, &shared.sources.ledger, &shared.writer, events::now());
    match outcome {
        Outcome::Recorded(response) => json(200, wire::encode(&response)),
        Outcome::Refused(reason) => refusal(reason),
        Outcome::LedgerUnknown => Response::text(503, "ledger-unknown"),
        Outcome::IdShape => Response::text(500, "ruling-id-shape"),
        Outcome::AppendFailed => Response::text(502, "ledger-append"),
        Outcome::CloseFailed(id) => Response::text(502, &format!("ledger-close {id}")),
    }
}

/// 束の受付（守りは `guarded`）。断りと 4xx と 5xx は何も書いていない。
/// 502 の本文は、2 回とも書き終えた行だけを書いたとして持つ BatchResponse。
fn post_batch(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<BatchRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    match batch::accept(&body, &shared.sources.ledger, &shared.writer, events::now()) {
        batch::Outcome::Recorded(response) => json(200, wire::encode(&response)),
        batch::Outcome::Refused(reason) => refusal(reason),
        batch::Outcome::Duplicate => Response::text(400, "duplicate"),
        batch::Outcome::LedgerUnknown => Response::text(503, "ledger-unknown"),
        batch::Outcome::IdShape => Response::text(500, "ruling-id-shape"),
        batch::Outcome::WriteFailed(response) => json(502, wire::encode(&response)),
    }
}

/// 方針の受付（守りは `guarded`）。断りと 4xx と 5xx は何も書いていない。
fn post_policy(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<PolicyRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    match policy::accept(&body, &shared.sources.ledger, &shared.writer, events::now()) {
        policy::Outcome::Recorded(response) => json(200, wire::encode(&response)),
        policy::Outcome::Refused(reason) => refusal(reason),
        policy::Outcome::LedgerUnknown => Response::text(503, "ledger-unknown"),
        policy::Outcome::BadScope => Response::text(400, "scope"),
        policy::Outcome::NoMemo => Response::text(503, "no-policy-memo"),
        policy::Outcome::IdShape => Response::text(500, "policy-id-shape"),
        policy::Outcome::AppendFailed => Response::text(502, "ledger-append"),
    }
}

/// account board の読み。state dir が無ければ器も git も撃たず、口座と群と移動が Unknown で列が空の電文。
/// 最初の要求で acct の印の取り直しを始める（要求は取り直しを待たない）。
fn account(shared: &Shared) -> Response {
    let doc = match &shared.acct {
        Some(acct) => {
            shared
                .acct_watch
                .call_once(|| watch_acct_marks(Arc::clone(acct), &shared.acct_marks));
            acct.doc(events::now())
        }
        None => tsuzuri_core::account::project::assemble(
            events::now(),
            Reading::Unknown,
            Reading::Unknown,
            Reading::Unknown,
            Vec::new(),
            Vec::new(),
        ),
    };
    json(200, wire::encode(&doc))
}

/// acct の印の一覧を、始めてすぐと `ACCT_MARKS_EVERY` ごとに `Acct::marks` で置き換える別の thread
/// （一覧の持ち手が落ちれば止まる）。
fn watch_acct_marks(acct: Arc<Acct>, marks: &Arc<Mutex<Vec<PathBuf>>>) {
    let weak: Weak<Mutex<Vec<PathBuf>>> = Arc::downgrade(marks);
    thread::spawn(move || {
        loop {
            let current = acct.marks();
            let Some(marks) = weak.upgrade() else {
                return;
            };
            *lock(&marks) = current;
            drop(marks);
            thread::sleep(ACCT_MARKS_EVERY);
        }
    });
}

/// 停止の切り替えの受付（守りは `guarded`・本文の読みは `accthb::accept`）。
/// state dir が無ければ器を撃たず 404 no-project。200 の本文は JSON、ほかは字。
fn post_heartbeat(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| Some(t.to_string())) {
        Ok(body) => body,
        Err(response) => return response,
    };
    let Some(acct) = &shared.acct else {
        return Response::text(404, accthb::NO_PROJECT);
    };
    match accthb::accept(acct, &body) {
        (200, text) => Response::json(200, text),
        (status, text) => Response::text(status, &text),
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// 電文の字の応答（字は契約の型の crate の `wire` が作る・境界の crate は serde に直接依存しない）。
fn json(status: u16, encoded: Result<String, wire::Error>) -> Response {
    match encoded {
        Ok(body) => Response::json(status, body),
        Err(_) => Response::text(500, "encode"),
    }
}
