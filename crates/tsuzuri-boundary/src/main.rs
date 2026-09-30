//! tz の入口（便 e-min）。終了 code は 合格 0・不合格 1・まだ分からない 2。
//! tz surface serve --repo <dir> --bind <住所:port> --files <dir> [--bd <program>] [--state-dir <dir>] [--folio <program>]
//! bind 先は loopback か tailnet の住所だけ（条 N-6）。tailnet の住所はこの引数で受ける（行 D-4）。
//! --bd は台帳の読みに撃つ program（既定 bd・便 e-src）。
//! --state-dir は器の state dir（省けば走行の出所は読めない）・--folio は設計の索引の読みに撃つ program
//! （既定 folio）（便 e-read）。host 固有の置き場は code に書かず、この引数で受ける（行 D-4）。
//! --bdw は台帳の書きに撃つ program（既定 bdw）・--seat は裁定を配達する席の target（--state-dir と両方が
//! 在るときだけ配達する）・--scribe2 は配達に撃つ器の CLI（既定 scribe2）（便 e-ask）。
//! --seat と --state-dir の両方が在るときだけ、口 /api/seat が席の card を組む（器の読みも --scribe2 で撃つ・便 e-seat）。
//! --read-only は値を取らず、答えと方針の口を 403 で断り、問いの一覧に答えを受けないと書く（ほかの project の
//! board を読むだけで起こす・行 e-ask-own-only）。
//! --project はほかの project の repo の置き場で、何度でも受け、その台帳の open の問いを dir の名の札つきで問いの一覧に
//! 混ぜる（答えは受けない・行 e-multi-ask）。
//! --tz は表示先の設定と窓を開く頼みに撃つ tz の program（省けば server 自身の binary・行 e-stage-target）。
//! 席の「見て」の知らせの記録の dir は引数でなく環境の XDG_STATE_HOME と HOME から tz stage notify と同じ決めで引く（行 i-11）。
//! tz graph [--check | --design] [--repo <dir>] [--bd <program>] [--folio <program>] [--state-dir <dir>]（行 k-graph）。
//! tz hook stop --repo <dir> [--bd <program>] [--bdw <program>]（行 f-stop・席の停止の hook・rc は 0 か 1）。
//! tz hook question-gate --repo <dir> [--bd <program>] [--folio <program>]（行 f-gate・問いの起票の門・rc は 0 か 1）。
//! tz hook question-signal --repo <dir>（行 e-signal-send・問いの合図の送り手・rc は 0 か 1）。
//! tz hook deliver --repo <dir> [--bd <program>]（行 f-deliver・配達の指し示しの逐語を席の文脈に足す・rc は 0 か 1）。
//! tz hook deliver-tool --repo <dir> [--bd <program>] [--bdw <program>]（行 f-deliver-tool・tool の呼びの組の後に未読の逐語を席の文脈に足す・rc は 0 か 1）。
//! tz stage <命令> --to <端末の名> [命令の旗] [--repo <dir>]（行 i-5・表示面の命令・rc は 0 か 1）。

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_boundary::stage::notify;
use tsuzuri_core::account::project_name;

const USAGE: &str = "usage: tz surface serve --repo <dir> --bind <住所:port> --files <dir> [--bd <program>] [--state-dir <dir>] [--folio <program>] [--bdw <program>] [--seat <target>] [--scribe2 <program>] [--tz <program>] [--read-only] [--project <dir>]...";

/// 読むだけの server を名指す値を取らない引数（行 e-ask-own-only）。
const READ_ONLY: &str = "--read-only";

/// 問いの一覧に混ぜるほかの project の置き場を名指す何度でも受ける引数（行 e-multi-ask）。
const PROJECT: &str = "--project";

/// 不合格（断り・使い方の誤り）。
const FAIL: u8 = 1;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rc = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["surface", "serve", rest @ ..] => serve(rest),
        ["graph", rest @ ..] => tsuzuri_boundary::cli::graph::run(rest),
        ["hook", "stop", rest @ ..] => tsuzuri_boundary::hook::stop::run(rest),
        ["hook", "deliver", rest @ ..] => tsuzuri_boundary::hook::deliver::run(rest),
        ["hook", "deliver-tool", rest @ ..] => tsuzuri_boundary::hook::deliver_tool::run(rest),
        ["hook", "question-gate", rest @ ..] => tsuzuri_boundary::hook::question_gate::run(rest),
        ["hook", "question-signal", rest @ ..] => {
            tsuzuri_boundary::hook::question_signal::run(rest)
        }
        ["stage", rest @ ..] => tsuzuri_boundary::stage::cli::run(rest),
        _ => usage("subcommand"),
    };
    ExitCode::from(rc)
}

fn usage(what: &str) -> u8 {
    eprintln!("tz: {what}\n{USAGE}");
    FAIL
}

/// `--名 値` か `--名=値` の 3 つの引数と、省ける --bd・--state-dir・--folio・--bdw・--seat・--scribe2・--tz と、
/// 値を取らない --read-only と、何度でも受ける --project を読む（省ける引数の空の値と 2 度の引数は断る・
/// --project は dir の名の無い値と前と同じ dir の名を断る）。
fn parse(rest: &[&str]) -> Result<Config, String> {
    let (mut repo, mut bind, mut files, mut bd) = (None, None, None, None);
    let (mut state_dir, mut folio) = (None, None);
    let (mut bdw, mut seat, mut scribe2, mut tz) = (None, None, None, None);
    let mut read_only = false;
    let (mut projects, mut labels) = (Vec::new(), Vec::new());
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        if *arg == READ_ONLY {
            if std::mem::replace(&mut read_only, true) {
                return Err(format!("{READ_ONLY} が 2 度ある"));
            }
            continue;
        }
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, v),
            None => (*arg, *it.next().ok_or_else(|| format!("{arg} の値が無い"))?),
        };
        if name == PROJECT {
            let label = project_name(value);
            if label.is_empty() {
                return Err(format!("{PROJECT} の値 {value} に dir の名が無い"));
            }
            if labels.contains(&label) {
                return Err(format!("{PROJECT} の名 {label} が 2 度ある"));
            }
            labels.push(label);
            projects.push(PathBuf::from(value));
            continue;
        }
        let slot = match name {
            "--repo" => &mut repo,
            "--bind" => &mut bind,
            "--files" => &mut files,
            "--bd" => &mut bd,
            "--state-dir" => &mut state_dir,
            "--folio" => &mut folio,
            "--bdw" => &mut bdw,
            "--seat" => &mut seat,
            "--scribe2" => &mut scribe2,
            "--tz" => &mut tz,
            _ => return Err(format!("知らない引数 {name}")),
        };
        if slot.replace(value).is_some() {
            return Err(format!("{name} が 2 度ある"));
        }
    }
    let (Some(repo), Some(bind), Some(files)) = (repo, bind, files) else {
        return Err("--repo と --bind と --files の 3 つが要る".into());
    };
    for (name, value) in [
        ("--bd", bd),
        ("--state-dir", state_dir),
        ("--folio", folio),
        ("--bdw", bdw),
        ("--seat", seat),
        ("--scribe2", scribe2),
        ("--tz", tz),
    ] {
        if value == Some("") {
            return Err(format!("{name} の値が空"));
        }
    }
    let bind = bind
        .parse::<SocketAddr>()
        .map_err(|_| format!("bind 先 {bind} は 住所:port の形でない"))?;
    let mut config = Config {
        state_dir: state_dir.map(PathBuf::from),
        seat: seat.map(str::to_string),
        read_only,
        projects,
        ..Config::new(PathBuf::from(repo), bind, PathBuf::from(files))
    };
    // 省いた program は Config::new の既定の値のまま。
    for (slot, value) in [
        (&mut config.bd, bd),
        (&mut config.folio, folio),
        (&mut config.bdw, bdw),
        (&mut config.scribe2, scribe2),
    ] {
        if let Some(value) = value {
            *slot = value.into();
        }
    }
    // --tz を省けば server 自身の binary（起きている server の PATH に tz は無いことが在る・行 e-stage-target）。
    match (tz, std::env::current_exe()) {
        (Some(value), _) => config.tz = value.into(),
        (None, Ok(exe)) => config.tz = exe.into(),
        (None, Err(_)) => {}
    }
    Ok(config)
}

fn serve(rest: &[&str]) -> u8 {
    let mut config = match parse(rest) {
        Ok(config) => config,
        Err(e) => return usage(&e),
    };
    // 席の「見て」の知らせの記録の dir は tz stage notify と同じ環境の字で引く（起動の引数は足さない・行 i-11）。
    config.notify = notify::dir(
        std::env::var_os("XDG_STATE_HOME").as_deref(),
        std::env::var_os("HOME").as_deref(),
    );
    let server = match Server::bind(&config) {
        Ok(server) => server,
        Err(e) => {
            eprintln!("tz surface serve: 起動を断る: {e}");
            return FAIL;
        }
    };
    match server.local_addr() {
        Ok(addr) => eprintln!("tz surface serve: http://{addr}/"),
        Err(e) => eprintln!("tz surface serve: 口の住所が読めない: {e}"),
    }
    server.run()
}
