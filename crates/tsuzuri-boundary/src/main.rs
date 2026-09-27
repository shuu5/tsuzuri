//! tz の入口（便 e-min）。終了 code は 合格 0・不合格 1・まだ分からない 2。
//! tz surface serve --repo <dir> --bind <住所:port> --files <dir> [--bd <program>] [--state-dir <dir>] [--folio <program>]
//! bind 先は loopback か tailnet の住所だけ（条 N-6）。tailnet の住所はこの引数で受ける（行 D-4）。
//! --bd は台帳の読みに撃つ program（既定 bd・便 e-src）。
//! --state-dir は器の state dir（省けば走行の出所は読めない）・--folio は設計の索引の読みに撃つ program
//! （既定 folio）（便 e-read）。host 固有の置き場は code に書かず、この引数で受ける（行 D-4）。
//! --bdw は台帳の書きに撃つ program（既定 bdw）・--seat は裁定を配達する席の target（--state-dir と両方が
//! 在るときだけ配達する）・--scribe2 は配達に撃つ器の CLI（既定 scribe2）（便 e-ask）。
//! --seat と --state-dir の両方が在るときだけ、口 /api/seat が席の card を組む（器の読みも --scribe2 で撃つ・便 e-seat）。
//! tz graph [--check | --design] [--repo <dir>] [--bd <program>] [--folio <program>] [--state-dir <dir>]（行 k-graph）。

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use tsuzuri_boundary::server::{Config, Server};

const USAGE: &str = "usage: tz surface serve --repo <dir> --bind <住所:port> --files <dir> [--bd <program>] [--state-dir <dir>] [--folio <program>] [--bdw <program>] [--seat <target>] [--scribe2 <program>]";

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
        _ => usage("subcommand"),
    };
    ExitCode::from(rc)
}

fn usage(what: &str) -> u8 {
    eprintln!("tz: {what}\n{USAGE}");
    FAIL
}

/// `--名 値` か `--名=値` の 3 つの引数と、省ける --bd・--state-dir・--folio・--bdw・--seat・--scribe2 を読む
/// （省ける引数の空の値は断る）。
fn parse(rest: &[&str]) -> Result<Config, String> {
    let (mut repo, mut bind, mut files, mut bd) = (None, None, None, None);
    let (mut state_dir, mut folio) = (None, None);
    let (mut bdw, mut seat, mut scribe2) = (None, None, None);
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, v),
            None => (*arg, *it.next().ok_or_else(|| format!("{arg} の値が無い"))?),
        };
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
    Ok(config)
}

fn serve(rest: &[&str]) -> u8 {
    let config = match parse(rest) {
        Ok(config) => config,
        Err(e) => return usage(&e),
    };
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
