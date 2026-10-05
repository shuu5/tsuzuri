//! host の負荷と書きの電文（要件 FR12・判断の記録 ADR-27 の決定 (5)・ADR-31 の決定 (5)）。
//! 材料は kernel の file（負荷・詰まり・メモリ・scope ごとのメモリ・装置の書いた区の数）と、host の面の
//! 書きの測りの表が名指す摩耗の記録で、server はその場で読んで返す（database に書かない・条 P-20）。
//! 値は整数で持つ（負荷と詰まりは 100 倍・量は byte）。線を越えたかは中核の 1 つの関数が判じ、面は判じ直さない。

use serde::{Deserialize, Serialize};

use crate::EpochSecs;
use crate::board::Reading;

/// host の口の path（面と server はこの定数を使う）。
pub const PATH: &str = "/api/host";

/// 注意の色の線を持つ測り（規則の行 R-42）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Gauge {
    /// 1 分の負荷の core 比。
    Load,
    /// io の詰まり（PSI の full の avg60）。
    Io,
    /// memory の詰まり（PSI の full の avg60）。
    Memory,
}

impl Gauge {
    /// 全部の測り（この順）。
    pub const ALL: [Gauge; 3] = [Gauge::Load, Gauge::Io, Gauge::Memory];
}

/// 負荷（1・5・15 分の load の 100 倍と、動いている core の数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Load {
    pub one: u32,
    pub five: u32,
    pub fifteen: u32,
    pub cores: u32,
}

/// 詰まりの 1 種（PSI の avg10 と avg60 の % の 100 倍）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stall {
    pub avg10: u32,
    pub avg60: u32,
}

/// 詰まり（cpu の some・memory の full・io の full・種ごとに読めなければ Unknown）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pressure {
    pub cpu_some: Reading<Stall>,
    pub memory_full: Reading<Stall>,
    pub io_full: Reading<Stall>,
}

/// メモリ（byte・全体と使える量と swap の全体と空き）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    pub total: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

/// 上限を持つ scope の 1 つ（cgroup の名・今の量・上限・byte）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeMemory {
    pub name: String,
    pub current: u64,
    pub max: u64,
}

/// 摩耗（smartctl の JSON の値・記録の時刻・使った割合・書いた byte・予備の割合・媒体の誤りの数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wear {
    pub read_at: EpochSecs,
    pub used_pct: u8,
    pub written: u64,
    pub spare_pct: u8,
    pub media_errors: u64,
}

/// 装置の 1 つ（host の面の書きの測りの表の名・書きの速さ byte/s・摩耗）。
/// 速さは前の読みとの差から導き、前の読みが無い・数えが戻った・間が 0 の時は Unknown。
/// 摩耗は表の行が記録の path を持たなければ None、持って読めなければ Some(Unknown)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceWrite {
    pub name: String,
    pub rate: Reading<u64>,
    pub wear: Option<Reading<Wear>>,
}

/// host の口の電文（組んだ時刻・負荷・詰まり・メモリ・上限を持つ scope〔埋まりの多い順〕・装置〔表の順〕・
/// 線を越えた測り〔`Gauge::ALL` の順・空なら注意なし〕）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostDoc {
    pub at: EpochSecs,
    pub load: Reading<Load>,
    pub pressure: Pressure,
    pub memory: Reading<Memory>,
    pub scopes: Reading<Vec<ScopeMemory>>,
    pub devices: Vec<DeviceWrite>,
    pub over: Vec<Gauge>,
}
