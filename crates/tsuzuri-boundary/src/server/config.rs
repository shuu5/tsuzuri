//! 起動の引数（行 hb-config）。欄を足す行は、この file と main.rs だけを書く。
//! 歯と main.rs は `Config::new` と残りの欄の埋め（struct の更新の形）で組む。

use std::ffi::OsString;
use std::net::SocketAddr;
use std::path::PathBuf;

use tsuzuri_contract::ledger::BDW;

use super::{design, ledger, ruling};

/// 起動の引数（repo の置き場・bind 先・面の file の置き場・bd の program・器の state dir・設計の道具の program・
/// bdw の program・席の target・器の CLI の program・読むだけか・ほかの project の置き場・知らせの記録の dir）。席の target と state dir の両方が在るときだけ、
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
    /// 答えと方針の口を 403 で断り、問いの一覧に答えを受けないと書くか（既定は偽・行 e-ask-own-only）。
    pub read_only: bool,
    /// 問いの一覧に混ぜるほかの project の repo の置き場（引数の順・既定は空・行 e-multi-ask）。
    pub projects: Vec<PathBuf>,
    /// 席の「見て」の知らせの記録の dir（None なら口 /api/notices は 503・main.rs は tz stage notify と同じ環境の字で引く・行 i-11）。
    pub notify: Option<PathBuf>,
}

impl Config {
    /// 省けない 3 つの欄を受け、ほかの欄を既定の値で埋める（state dir と席の target は無し）。
    pub fn new(repo: PathBuf, bind: SocketAddr, files: PathBuf) -> Config {
        Config {
            repo,
            bind,
            files,
            bd: ledger::BD.into(),
            state_dir: None,
            folio: design::FOLIO.into(),
            bdw: BDW.into(),
            seat: None,
            scribe2: ruling::SCRIBE2.into(),
            read_only: false,
            projects: Vec::new(),
            notify: None,
        }
    }
}
