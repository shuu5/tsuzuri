//! host の負荷と書きの印と窓の中身（判断の記録 ADR-27 の決定 (3)(4) の帯の印と窓の作り）: 帯の右の短い印
//! （1 分の負荷の core 比と装置の書きの速さの和）と、印を押すと開く窓の段（負荷・詰まり・メモリと上限を持つ scope・
//! 装置ごとの書きの速さ・摩耗と記録の時刻）。account board の HOME の block も同じ中身の関数（`content`）を描く。
//! 口は契約の型の host の PATH。線を越えたかは中核が判じた電文の over を写すだけで、面は値と線を比べない（条 P-21）。
//! 読めない欄はその欄だけ「?」にし、口が読めなければ全体を測れていないにする（条 P-7）。
//! 字と並びは純粋な関数にして host で試し、DOM（`body`）は wasm の target のときだけ組む。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::host::{DeviceWrite, Gauge, HostDoc, Load, Memory, ScopeMemory, Stall};
use tsuzuri_contract::wire;

use crate::project::{Body, NOT_READ};
use crate::view::{Fetched, clock_short};

pub use tsuzuri_contract::host::PATH;

/// 帯の印の aria・窓の題・account board の block の見出しの語の鍵。
pub const HOST_KEY: &str = "host_load";

/// 窓の幅（px）。
pub const WIDTH: u32 = 600;

/// 口が読めないときの理由。
pub const UNREAD: &str =
    "host の口が読めない（届かない・知らせが切れた）ので今の負荷と書きが正しいと言えない";

/// 本文が電文として読めないときの理由。
pub const BAD_BODY: &str = "host の口の本文が電文（HostDoc）として読めない";

/// 読めない欄の字。
pub const UNKNOWN_TEXT: &str = "?";

/// 印の class（線の内・線を越えた測りが在る・口が読めない）。
pub const MARK_CLASSES: [&str; 3] = ["hostb", "hostb w", "hostb u"];

/// 窓の段の見出しの語の鍵（段の順: 負荷・詰まり・メモリ・書きの速さ・摩耗）。
pub const PART_KEYS: [&str; 5] = ["hw_load", "hw_stall", "hw_memory", "hw_write", "hw_wear"];

/// 装置が 0 の時の書きの段の 1 行。
pub const NO_DEVICES: &str = "書きの測りの表に装置が無い";

/// 摩耗の記録を持つ装置が 0 の時の摩耗の段の 1 行。
pub const NO_WEAR: &str = "摩耗の記録を持つ装置が無い";

/// 上限を持つ scope が読めない時の行の名。
pub const SCOPE_NAME: &str = "scope";

/// 帯の印（class と字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostMark {
    pub class: &'static str,
    pub text: String,
}

/// 窓の段の 1 行（名・値の字・線を越えた測りの行か）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub name: String,
    pub value: String,
    pub warn: bool,
}

/// 窓の段（見出しの語の鍵・行・行が無い時の 1 行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub key: &'static str,
    pub lines: Vec<Line>,
    pub none: Option<&'static str>,
}

/// 口の本文を電文に読む（まだ読んでいない・読めない・電文が読めないは理由）。
pub fn doc(fetched: &Fetched) -> Result<HostDoc, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(UNREAD),
        Fetched::Body(text) => wire::decode::<HostDoc>(text).map_err(|_| BAD_BODY),
    }
}

/// 100 倍の整数を小数 2 桁の字にする（42 は `0.42`・1344 は `13.44`）。
pub fn fixed2(hundredths: u32) -> String {
    format!("{}.{:02}", hundredths / 100, hundredths % 100)
}

/// 1 分の負荷の core 比の 100 倍（表示の丸め・切り捨て・core の数 0 は 1 で割る・線の判じは中核の over）。
pub fn ratio(load: &Load) -> u32 {
    load.one / load.cores.max(1)
}

/// byte/s を MB/s（10 の 6 乗 byte）の字にする（10 未満は小数 1 桁・ほかは整数・どちらも切り捨て）。
pub fn mbps(bytes: u64) -> String {
    let mb = 1_000_000;
    if bytes < 10 * mb {
        format!("{}.{}", bytes / mb, bytes % mb / (mb / 10))
    } else {
        (bytes / mb).to_string()
    }
}

/// byte を GiB の小数 1 桁の字にする（切り捨て）。
pub fn gib(bytes: u64) -> String {
    let tenths = u128::from(bytes) * 10 / (1u128 << 30);
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// byte を TB（10 の 12 乗 byte）の小数 1 桁の字にする（切り捨て）。
pub fn tb(bytes: u64) -> String {
    let tenths = bytes / 100_000_000_000;
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// 装置の書きの速さの和（装置が 0・どれかが読めない時は None）。
pub fn write_total(devices: &[DeviceWrite]) -> Option<u64> {
    if devices.is_empty() {
        return None;
    }
    devices.iter().try_fold(0u64, |sum, d| match d.rate {
        Reading::Known(r) => Some(sum.saturating_add(r)),
        Reading::Unknown => None,
    })
}

/// 帯の印: 字は「負荷 <core 比> · 書き <和> MB/s」（読めない値は「?」）で、class は電文の over が空でなければ
/// `hostb w`・空なら `hostb`。口が読めなければ `hostb u` の「負荷 ? · 書き ?」。
pub fn mark(fetched: &Fetched) -> HostMark {
    let Ok(d) = doc(fetched) else {
        return HostMark {
            class: MARK_CLASSES[2],
            text: format!("負荷 {UNKNOWN_TEXT} · 書き {UNKNOWN_TEXT}"),
        };
    };
    let load = match &d.load {
        Reading::Known(l) => fixed2(ratio(l)),
        Reading::Unknown => UNKNOWN_TEXT.to_string(),
    };
    let write = write_total(&d.devices)
        .map_or_else(|| UNKNOWN_TEXT.to_string(), |b| format!("{} MB/s", mbps(b)));
    HostMark {
        class: if d.over.is_empty() {
            MARK_CLASSES[0]
        } else {
            MARK_CLASSES[1]
        },
        text: format!("負荷 {load} · 書き {write}"),
    }
}

fn line(name: &str, value: String, warn: bool) -> Line {
    Line {
        name: name.to_string(),
        value,
        warn,
    }
}

/// 行が無い時だけ `none` の 1 行を持つ段。
fn part(key: &'static str, lines: Vec<Line>, none: &'static str) -> Part {
    let none = lines.is_empty().then_some(none);
    Part { key, lines, none }
}

/// 行が欠けない段（読めない値は行の値が「?」）。
fn fixed(key: &'static str, lines: Vec<Line>) -> Part {
    Part {
        key,
        lines,
        none: None,
    }
}

/// 負荷の段: core 比（over に Load が在れば注意）・1 と 5 と 15 分の負荷・core の数。
fn load_part(load: &Reading<Load>, over: &[Gauge]) -> Part {
    let (r, all, cores) = match load {
        Reading::Known(l) => (
            fixed2(ratio(l)),
            format!(
                "{} · {} · {}",
                fixed2(l.one),
                fixed2(l.five),
                fixed2(l.fifteen)
            ),
            l.cores.to_string(),
        ),
        Reading::Unknown => (
            UNKNOWN_TEXT.into(),
            UNKNOWN_TEXT.into(),
            UNKNOWN_TEXT.into(),
        ),
    };
    let lines = vec![
        line("core 比", r, over.contains(&Gauge::Load)),
        line("1 · 5 · 15 分", all, false),
        line("core", cores, false),
    ];
    fixed(PART_KEYS[0], lines)
}

/// 詰まりの段: cpu の some・memory の full・io の full の avg10 と avg60（memory と io は over に在れば注意）。
fn stall_part(d: &HostDoc) -> Part {
    let text = |r: &Reading<Stall>| match r {
        Reading::Known(s) => format!("avg10 {}% · avg60 {}%", fixed2(s.avg10), fixed2(s.avg60)),
        Reading::Unknown => UNKNOWN_TEXT.to_string(),
    };
    let p = &d.pressure;
    let lines = vec![
        line("cpu some", text(&p.cpu_some), false),
        line(
            "memory full",
            text(&p.memory_full),
            d.over.contains(&Gauge::Memory),
        ),
        line("io full", text(&p.io_full), d.over.contains(&Gauge::Io)),
    ];
    fixed(PART_KEYS[1], lines)
}

/// メモリの段: 使っている量と全体・swap の使っている量と全体・上限を持つ scope（電文の順・今の量と上限と埋まりの %）。
fn memory_part(memory: &Reading<Memory>, scopes: &Reading<Vec<ScopeMemory>>) -> Part {
    let used =
        |total: u64, free: u64| format!("{} / {} GiB", gib(total.saturating_sub(free)), gib(total));
    let (mem, swap) = match memory {
        Reading::Known(m) => (used(m.total, m.available), used(m.swap_total, m.swap_free)),
        Reading::Unknown => (UNKNOWN_TEXT.into(), UNKNOWN_TEXT.into()),
    };
    let mut lines = vec![line("使っている", mem, false), line("swap", swap, false)];
    match scopes {
        Reading::Known(list) => lines.extend(list.iter().map(|s| {
            let pct = u128::from(s.current) * 100 / u128::from(s.max.max(1));
            line(
                &s.name,
                format!("{} / {} GiB · {pct}%", gib(s.current), gib(s.max)),
                false,
            )
        })),
        Reading::Unknown => lines.push(line(SCOPE_NAME, UNKNOWN_TEXT.into(), false)),
    }
    fixed(PART_KEYS[2], lines)
}

/// 書きの速さの段: 装置ごとの MB/s（表の順・読めない速さは「?」）。
fn write_part(devices: &[DeviceWrite]) -> Part {
    let lines = devices
        .iter()
        .map(|d| {
            let v = match d.rate {
                Reading::Known(r) => format!("{} MB/s", mbps(r)),
                Reading::Unknown => UNKNOWN_TEXT.to_string(),
            };
            line(&d.name, v, false)
        })
        .collect();
    part(PART_KEYS[3], lines, NO_DEVICES)
}

/// 摩耗の段: 記録を持つ装置ごとの使った割合・書いた量・予備・媒体の誤り・記録の時刻（読めない記録は「?」・
/// 記録の path を持たない装置は出さない）。
fn wear_part(devices: &[DeviceWrite], now: EpochSecs) -> Part {
    let lines = devices
        .iter()
        .filter_map(|d| {
            let v = match d.wear.as_ref()? {
                Reading::Known(w) => format!(
                    "使った {}% · 書いた {} TB · 予備 {}% · 誤り {} · 記録 {}",
                    w.used_pct,
                    tb(w.written),
                    w.spare_pct,
                    w.media_errors,
                    clock_short(w.read_at, now)
                ),
                Reading::Unknown => UNKNOWN_TEXT.to_string(),
            };
            Some(line(&d.name, v, false))
        })
        .collect();
    part(PART_KEYS[4], lines, NO_WEAR)
}

/// 窓の段の全部（`PART_KEYS` の順）。
pub fn parts(d: &HostDoc, now: EpochSecs) -> Vec<Part> {
    vec![
        load_part(&d.load, &d.over),
        stall_part(d),
        memory_part(&d.memory, &d.scopes),
        write_part(&d.devices),
        wear_part(&d.devices, now),
    ]
}

/// 窓と block の中身: 口が読めなければ測れていない（理由の 1 行）、読めれば段の全部。
pub fn content(fetched: &Fetched, now: EpochSecs) -> Body<Vec<Part>> {
    match doc(fetched) {
        Err(reason) => Body::Unmeasured(reason),
        Ok(d) => Body::Filled(parts(&d, now)),
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::body;

#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::{PATH, Part, content};
    use crate::project::{Body, body_view, unmeasured};
    use crate::vocab::label;

    fn part_view(p: Part) -> AnyView {
        let lines = p
            .lines
            .into_iter()
            .map(|l| {
                let class = if l.warn { "hw-l w" } else { "hw-l" };
                view! { <div class=class><span class="hw-n">{l.name}</span><span class="hw-v num">{l.value}</span></div> }
            })
            .collect_view();
        let none = p
            .none
            .map(|n| view! { <div class="hw-l"><span class="hw-v muted">{n}</span></div> });
        view! { <div class="hw-p"><h4 data-v=p.key>{label(p.key)}</h4>{lines}{none}</div> }
            .into_any()
    }

    /// 窓と block の本文（host の口を読む）。
    pub fn body() -> AnyView {
        let fetched = crate::net::read(PATH);
        let inner = move || match fetched.with(|f| content(f, crate::net::now())) {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Filled(ps) => {
                view! { <div class="hw">{ps.into_iter().map(part_view).collect_view()}</div> }
                    .into_any()
            }
        };
        inner.into_any()
    }
}
