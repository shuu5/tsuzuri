//! 面の server の最小の形（設計ノート surface-base §9）。
//! 標準 library だけで書く同期の server で、1 つの process と 1 つの port で動く。1 接続 1 thread。
//! 台帳は bd の読み取りの口を子 process で撃って読む（§10）。
//! 読む側の口の 4 つは、台帳と設計の索引と器の event log の字を集めて中核の関数に渡す（§11）。
//! 問いの一覧の口と裁定の受付の口は `ruling` が持つ。束と方針の受付の口は `batch` と `policy` が持つ。
//! account board の読みの口と停止の切り替えの口は `crate::acct` と `crate::accthb` が持つ。
//! POST を受ける口は /api/ruling・/api/batch・/api/policy・/api/account/heartbeat・/api/seat/heartbeat と、
//! 表示先の設定と窓を開く 3 つ（/api/stage/target・/api/stage/targets・/api/stage/open・頭 Origin の無い要求は断り、
//! 読むだけの server も受ける・tz の口を撃つだけ）だけで、
//! ほかの GET でない要求は 405 で何も書かない（問いの合図の口 /api/surface/questions も POST を受けるが、何も書かず
//! 台帳の見張りの周期の待ちを終わらせるだけで、読むだけの server も受ける）。server 自身は file を書かない（台帳に書くのは bdw・席へ送るのは器の CLI）。
//! 読むだけの server（`Config::read_only`）は答えと方針の口を受付の前に 403 で断り（`read_only`）、
//! 問いの一覧の電文に答えを受けないと書く（心拍の口は受ける）。
//! 起動の引数 --project の置き場ごとに、その台帳を読み取りの bd で見張り、問いの一覧の電文の鍵 others に札つきで
//! 載せる（答えは受けず、答えの口はその問いを自分の台帳に無い問いとして断る・`others`）。
//! 席の card の口は `seat` が持つ。次の一手の口は、席の card が読めるときは席の card も受けて判じる。
//! 同じ時に届いた要求は、設計の索引の読みを 1 本の子 process で分け合う（`coalesce`）。
//! 台帳の GET の口は、起動で作る 1 つの `Source`（`Source::watched`）とその clone の印が見張りの最後の読みの前と同じ間は
//! bd を撃たず、見張りの最後の読みの字を返し、見張りが読みの途中ならその終わりを待って同じ字を返す。
//! 印が違えば（受け手が 0 人の間に印が動いた後）口が自分で読む。見張りの読みが落ちたときは
//! 最後に読めた字を `ledger::READ_HOLD`（60 秒）まで返し、その応答の頭（`READ_AGE_HEADER`）に最後に読めた時からの秒を
//! 付ける。裁定の受付は合流せず、新しい子 process で読み直す。
//! GET の口と POST の 5 つの口は src/server/routes の下に 1 口 1 file で置き（各 file の doc が自分の path を書く）、
//! 口の列 `Route` は組み立ての script が dir から生成する（`route`・判断の記録 ADR-13）。
//! 変化の知らせ（SSE）はここに在り、どの口にも当たらない GET は面の file の配布。
//! 板の印は走行・設計・席・account・席の「見て」の知らせの記録の種類（`ChangeKind`）を付けて見張りに渡す
//! （知らせが動いた種類を載せる・知らせの記録は `crate::stage::notify` が書く）。
//! 相談の窓の 3 つの口（一覧・未受け・頼みの POST /api/consult/request）は `consult` が持つ（頼みの口は方針の口と同じ守りで、
//! 読むだけの server は 403・置き場に相談の頼みの行を 1 行足すだけで窓を開かず配達を撃たない）。
//! 器の局面の出力の口は state dir の fleet/lifecycle.json と lifecycle.stale を要求のたびに読む（`cases`）。
//! その 2 つの file は板の印と同じ間隔で見張り、面が読む中身が動いた時だけ局面の出力の種類の board-changed を送る
//! （`Cases::watch`）。
//! host の口は kernel の file と host の面の書きの測りの表を要求のたびに読み（`host`）、受け手が居る周だけ
//! `host::HOST_POLL` ごとに読んで中身が動いた時だけ host の種類の board-changed を送る（`Host::watch`）。
//! 席の target と state dir の両方が在るときだけ、台帳の見張りの読みの周の台帳の字で器の doctor の台帳の形の行を撃ち、
//! その字と組で持つ（`form`）。撃ちは口 /api/pipeline の最初の要求か、
//! 知らせの接続が受け手を足す前に許す（受け手の付いた周の見張りの読みが撃つ）。

pub mod batch;
pub mod board;
pub mod cases;
pub mod clock;
pub mod coalesce;
mod config;
pub mod consult;
pub mod design;
pub mod events;
pub mod files;
pub mod form;
pub mod held;
pub mod host;
pub mod http;
pub mod ledger;
mod others;
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

use tsuzuri_contract::ledger::{BeadId, READ_AGE_HEADER};
use tsuzuri_contract::surface::{ChangeKind, Refusal, RefusalResponse};
use tsuzuri_contract::wire;

use crate::acct::Acct;
use crate::out::emit_err;
use crate::stage::notify;
use crate::stagecall::Caller;

use self::board::Sources;
use self::cases::Cases;
use self::consult::Consult;
use self::design::Design;
use self::events::Hub;
use self::form::Form;
use self::held::Held;
use self::host::Host;
use self::http::{Request, Response};
use self::ledger::Source;
use self::others::Others;
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
    /// 器の局面の出力の読み（state dir が無ければ出所の無い読み・行 c-case-read）。
    cases: Cases,
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
    /// 答えと方針の口を断る読むだけの server か（`Config::read_only`・行 e-ask-own-only）。
    read_only: bool,
    /// ほかの project の問いの読み（`Config::projects`・行 e-multi-ask）。
    others: Others,
    /// 席の「見て」の知らせの記録の dir（`Config::notify`・行 i-11）。
    notify: Option<PathBuf>,
    /// 表示先の設定と窓を開く頼みの撃ち先（tz と器の program と --repo・行 e-stage-target）。
    stage: Caller,
    /// 相談の窓の口の置き場の材料（行 cs-server）。
    consult: Consult,
    /// host の負荷と書きの読み（`host`・口 /api/host と host の種類の見張り）。
    host: Arc<Host>,
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
    /// account board の読みは自分の repo と --project の置き場の anchor の台帳を見張りの Source で読み、
    /// 印が見張りの最後の読みの前と同じ間は bd を撃たない（行 a-lean・行 e-ledger-lazy）。
    pub fn bind_with(config: &Config, git: &OsStr) -> Result<Server, StartError> {
        let files = Server::checked_files(config)?;
        let listener = TcpListener::bind(config.bind).map_err(StartError::Io)?;
        let form = match (&config.seat, &config.state_dir) {
            (Some(_), Some(state_dir)) => Some(Form::new(&config.scribe2, state_dir, &config.repo)),
            _ => None,
        };
        let sources = Sources {
            ledger: Source::new(&config.repo, &config.bd)
                .watched()
                .with_form(form.clone()),
            design: Design::new(&config.repo, &config.folio),
            runs: Runs::new(config.state_dir.as_deref()),
        };
        let held = Held::new();
        let seats = Seats::new(
            &config.scribe2,
            config.state_dir.as_deref(),
            config.seat.as_deref(),
            &config.repo,
            held.clone(),
        );
        let seat_marks = seats.marks();
        let others = Others::new(&config.projects, &config.bd);
        let acct = Server::acct_of(config, git, &sources, &others, &held);
        let acct_marks = Arc::new(Mutex::new(Vec::new()));
        let held_marks = Arc::clone(&acct_marks);
        let notify_dir = config.notify.clone();
        let cases = Cases::new(config.state_dir.as_deref());
        let hub = Server::start_hub(&sources, &cases, seat_marks, held_marks, notify_dir);
        if let Some(form) = &form {
            form.notify(&hub);
        }
        others.watch(&hub);
        let host = Arc::new(Host::new(config.state_dir.as_deref()));
        host.watch(&hub, host::HOST_POLL);
        let writer = Server::writer_of(config);
        Ok(Server {
            listener,
            shared: Arc::new(Shared {
                sources,
                cases,
                files,
                hub,
                writer,
                seats,
                acct,
                acct_marks,
                acct_watch: Once::new(),
                read_only: config.read_only,
                others,
                notify: config.notify.clone(),
                stage: Caller {
                    tz: config.tz.clone(),
                    scribe2: config.scribe2.clone(),
                    repo: config.repo.clone(),
                },
                consult: Consult::new(config, git),
                host,
            }),
        })
    }

    /// bind 先と repo と面の file の置き場を確かめ、面の file の置き場の正しい path を返す。
    fn checked_files(config: &Config) -> Result<PathBuf, StartError> {
        if !bind_allowed(config.bind.ip()) {
            return Err(StartError::BindRefused(config.bind));
        }
        if !config.repo.is_dir() {
            return Err(StartError::NotDir {
                what: "repo ",
                path: config.repo.clone(),
            });
        }
        config
            .files
            .canonicalize()
            .ok()
            .filter(|p| p.is_dir())
            .ok_or_else(|| StartError::NotDir {
                what: "面の file ",
                path: config.files.clone(),
            })
    }

    /// state dir が在るときだけ account board の読みを作る（自分と --project の台帳は見張りの Source で読む）。
    fn acct_of(
        config: &Config,
        git: &OsStr,
        sources: &Sources,
        others: &Others,
        held: &Held,
    ) -> Option<Arc<Acct>> {
        config.state_dir.as_ref().map(|state_dir| {
            Arc::new(
                Acct::new(
                    config.scribe2.clone(),
                    git,
                    config.bd.clone(),
                    state_dir.clone(),
                    config.repo.clone(),
                )
                .with_own(sources.ledger.clone())
                .with_watched(others.sources())
                .with_held(held.clone()),
            )
        })
    }

    /// 台帳の周期の読みと板の印（走行・設計・席・account・知らせ）の見張りと、局面の出力の中身の見張り
    /// （`Cases::watch`・行 c-cases-watch）を始める。
    fn start_hub(
        sources: &Sources,
        cases: &Cases,
        seat_marks: Vec<PathBuf>,
        held_marks: Arc<Mutex<Vec<PathBuf>>>,
        notify_dir: Option<PathBuf>,
    ) -> Arc<Hub> {
        let (design, runs) = (sources.design.clone(), sources.runs.clone());
        let kinded = |kind: ChangeKind, files: Vec<PathBuf>| -> Vec<(ChangeKind, PathBuf)> {
            files.into_iter().map(|f| (kind, f)).collect()
        };
        let hub = Hub::start(sources.ledger.clone(), move || {
            let mut marks = kinded(ChangeKind::Runs, runs.marks());
            marks.extend(kinded(ChangeKind::Design, design.marks()));
            marks.extend(kinded(ChangeKind::Seat, seat_marks.clone()));
            marks.extend(kinded(ChangeKind::Account, lock(&held_marks).clone()));
            // 知らせの記録の file（dir が無いか読めなければ足さない・行 i-11）。
            let notices = notify_dir.as_deref().map(notify::files).and_then(Result::ok);
            marks.extend(kinded(ChangeKind::Notice, notices.unwrap_or_default()));
            marks
        });
        cases.watch(&hub, events::POLL);
        hub
    }

    /// 裁定の書きの持ち物（席の target と state dir の両方が在るときだけ配達の先を持つ）。
    fn writer_of(config: &Config) -> Writer {
        let delivery = match (&config.seat, &config.state_dir) {
            (Some(target), Some(state_dir)) => Some(Delivery {
                program: config.scribe2.clone(),
                state_dir: state_dir.clone(),
                target: target.clone(),
            }),
            _ => None,
        };
        Writer {
            repo: config.repo.clone(),
            bdw: config.bdw.clone(),
            delivery,
        }
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
                Err(e) => emit_err(&format!("tz surface serve: accept: {e}")),
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
            // 受け手を足す前に撃ちを許す（受け手の付いた周の見張りの読みが撃つ・行 c-misfit-pair）。
            if let Some(form) = shared.sources.ledger.form() {
                form.arm();
            }
            let _ = events::stream(&stream, &shared.hub);
            return;
        }
        // 口（`route::dispatch`・method を問わない）に当たらなければ、GET でない要求は 405、GET は面の file の配布。
        Ok(req) => match route::dispatch(&req, shared) {
            Some(response) => response,
            None if req.method != "GET" => Response::text(405, "method").header("Allow", "GET"),
            None => files::respond(&req, &shared.files),
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

/// 頭 Origin の無い要求も断る POST の口の守り（表示先の設定と窓を開く口・行 e-stage-target）。
/// browser の POST は必ず頭 Origin を付けるので持ち主の button は通り、席が curl で撃つ要求は 403 origin で断る。
/// 頭 Origin が在れば `guarded` と同じ。
fn guarded_strict<T>(
    req: &Request,
    decode: impl FnOnce(&str) -> Option<T>,
) -> Result<T, Response> {
    if req.origin.is_none() {
        return Err(Response::text(403, "origin"));
    }
    guarded(req, decode)
}

/// 断りの応答（状態の code は `Refusal::http_status`・本文は RefusalResponse）。
fn refusal(reason: Refusal) -> Response {
    json(
        reason.http_status(),
        wire::encode(&RefusalResponse { reason }),
    )
}

/// 読むだけの server の断り（状態 403・本文 `ruling::READ_ONLY`）。`refusal_line` の 1 行を標準エラーに書いて返す
/// （`questions` は要求の問いの id・行 e-ask-own-only）。
fn read_only(path: &str, questions: &[&BeadId]) -> Response {
    let response = Response::text(403, ruling::READ_ONLY);
    emit_err(&ruling::refusal_line(
        path,
        response.status,
        &response.body,
        questions,
    ));
    response
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
