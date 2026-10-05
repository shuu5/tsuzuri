//! host の負荷と書きの読み（要件 FR12・判断の記録 ADR-27 の決定 (5)・ADR-31 の決定 (5)）。
//! 入力は kernel の file の字（loadavg・動いている cpu の範囲・PSI の 3 種・meminfo・scope の memory.current と
//! memory.max・装置の stat）と、smartctl の JSON の字と、host の面の書きの測りの表の字。境界が読んで渡し、ここは
//! 字を値にして電文を組むだけ（file も時計も触らない）。線を越えたかの判じは `over` の 1 つだけで、線の値は
//! 規則の行 R-42 の値を写した定数（面は判じ直さない・条 P-21）。

use serde_json::Value;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::host::{
    DeviceWrite, Gauge, HostDoc, Load, Memory, Pressure, ScopeMemory, Stall, Wear,
};

/// 1 分の負荷の core 比の線（100 倍・1.0 を越えたら注意・規則の行 R-42）。
pub const LOAD_WARN: u32 = 100;

/// io の詰まりの線（PSI の full の avg60 の % の 100 倍・5% を越えたら注意・規則の行 R-42）。
pub const IO_FULL_WARN: u32 = 500;

/// memory の詰まりの線（PSI の full の avg60 の % の 100 倍・1% を越えたら注意・規則の行 R-42）。
pub const MEMORY_FULL_WARN: u32 = 100;

/// 装置の stat の区の大きさ（byte・kernel の数えの単位）。
pub const SECTOR_BYTES: u64 = 512;

/// smartctl の NVMe の書いた量の単位（byte・1,000 区）。
pub const DATA_UNIT_BYTES: u64 = 512_000;

/// 電文に載せる scope の数の上限（埋まりの多い順）。
pub const SCOPES_SHOWN: usize = 5;

/// host の面の書きの測りの表の見出し（器の表と同じ字）。
pub const BUDGET_HEADER: &str = "[[write-budget]]";

/// 上限を持つかもしれない scope の 1 つの字（cgroup の名・memory.current と memory.max の字・読めなければ None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeTexts {
    pub name: String,
    pub current: Option<String>,
    pub max: Option<String>,
}

/// 装置の 1 つの材料（表の名・境界が `rate` で導いた速さ・摩耗の記録の字〔表に path が無ければ None・
/// 在って読めなければ Some(None)〕）。
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceTexts {
    pub name: String,
    pub rate: Reading<u64>,
    pub wear: Option<Option<String>>,
}

/// 電文の材料の字（どれも読めなければ None・scope は dir を読めなければ None）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HostTexts {
    pub loadavg: Option<String>,
    pub online: Option<String>,
    pub psi_cpu: Option<String>,
    pub psi_memory: Option<String>,
    pub psi_io: Option<String>,
    pub meminfo: Option<String>,
    pub scopes: Option<Vec<ScopeTexts>>,
    pub devices: Vec<DeviceTexts>,
}

/// host の面の書きの測りの表の 1 行（名・stat の path・摩耗の記録の path）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetRow {
    pub name: String,
    pub stat: String,
    pub wear: Option<String>,
}

/// 10 進の字の 100 倍（小数は 2 桁まで読み、3 桁目からは捨てる・符号と空の字と数でない字は None）。
pub fn hundredths(text: &str) -> Option<u32> {
    let (whole, frac) = text.split_once('.').unwrap_or((text, ""));
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    if whole.is_empty() || !digits(whole) || !digits(frac) {
        return None;
    }
    let cents: String = frac.chars().chain("00".chars()).take(2).collect();
    whole
        .parse::<u32>()
        .ok()?
        .checked_mul(100)?
        .checked_add(cents.parse().ok()?)
}

/// 動いている cpu の範囲の字（例 0-31 や 0-3,5）の cpu の数。
pub fn cores(online: &str) -> Option<u32> {
    let mut count = 0u32;
    for part in online.trim().split(',') {
        let (lo, hi) = part.split_once('-').unwrap_or((part, part));
        let (lo, hi): (u32, u32) = (lo.parse().ok()?, hi.parse().ok()?);
        count = count.checked_add(hi.checked_sub(lo)?.checked_add(1)?)?;
    }
    Some(count)
}

/// loadavg の字の 3 つの負荷と cpu の数（cpu が 0 なら None）。
pub fn load(loadavg: &str, online: &str) -> Option<Load> {
    let mut words = loadavg.split_whitespace().map(hundredths);
    let (one, five, fifteen) = (words.next()??, words.next()??, words.next()??);
    let cores = cores(online).filter(|&c| c > 0)?;
    Some(Load {
        one,
        five,
        fifteen,
        cores,
    })
}

/// PSI の字の `kind`（some か full）の行の avg10 と avg60。
pub fn stall(psi: &str, kind: &str) -> Option<Stall> {
    let line = psi
        .lines()
        .find(|l| l.split_whitespace().next() == Some(kind))?;
    let field = |key: &str| {
        line.split_whitespace()
            .find_map(|w| w.strip_prefix(key)?.strip_prefix('='))
            .and_then(hundredths)
    };
    Some(Stall {
        avg10: field("avg10")?,
        avg60: field("avg60")?,
    })
}

/// meminfo の字の全体と使える量と swap（kB の字を byte にする・4 つのどれかが無ければ None）。
pub fn memory(meminfo: &str) -> Option<Memory> {
    let field = |key: &str| {
        meminfo.lines().find_map(|l| {
            let rest = l.strip_prefix(key)?.strip_prefix(':')?;
            let kb: u64 = rest.trim().strip_suffix("kB")?.trim().parse().ok()?;
            kb.checked_mul(1024)
        })
    };
    Some(Memory {
        total: field("MemTotal")?,
        available: field("MemAvailable")?,
        swap_total: field("SwapTotal")?,
        swap_free: field("SwapFree")?,
    })
}

/// 上限を持つ scope（上限の字が max・読めない・0 の scope と、今の量が読めない scope を除く）を、埋まり
/// （今の量と上限の比）の多い順・同じ埋まりは名の順に並べ、`SCOPES_SHOWN` まで。
pub fn scopes(raw: &[ScopeTexts]) -> Vec<ScopeMemory> {
    let num = |t: &Option<String>| t.as_deref().and_then(|s| s.trim().parse::<u64>().ok());
    let mut rows: Vec<ScopeMemory> = raw
        .iter()
        .filter_map(|s| {
            let max = num(&s.max).filter(|&m| m > 0)?;
            Some(ScopeMemory {
                name: s.name.clone(),
                current: num(&s.current)?,
                max,
            })
        })
        .collect();
    rows.sort_by(|a, b| {
        let fill =
            |s: &ScopeMemory, other: &ScopeMemory| u128::from(s.current) * u128::from(other.max);
        fill(b, a)
            .cmp(&fill(a, b))
            .then_with(|| a.name.cmp(&b.name))
    });
    rows.truncate(SCOPES_SHOWN);
    rows
}

/// 装置の stat の字の書いた区の数（7 つ目の欄）。
pub fn sectors_written(stat: &str) -> Option<u64> {
    stat.split_whitespace().nth(6)?.parse().ok()
}

/// 書きの速さ（byte/s）: 前の読み（区の数と ms の時刻）と今の読みの差。前が無い・数えが戻った・間が 0 なら Unknown。
pub fn rate(before: Option<(u64, u64)>, now: (u64, u64)) -> Reading<u64> {
    let Some((sectors, at_ms)) = before else {
        return Reading::Unknown;
    };
    let (Some(moved), Some(ms)) = (now.0.checked_sub(sectors), now.1.checked_sub(at_ms)) else {
        return Reading::Unknown;
    };
    if ms == 0 {
        return Reading::Unknown;
    }
    let bytes = u128::from(moved) * u128::from(SECTOR_BYTES) * 1000 / u128::from(ms);
    u64::try_from(bytes).map_or(Reading::Unknown, Reading::Known)
}

/// smartctl の JSON（-j）の摩耗（記録の時刻・使った割合・書いた量・予備の割合・媒体の誤り・どれかが無ければ None）。
pub fn wear(json: &str) -> Option<Wear> {
    let doc: Value = serde_json::from_str(json).ok()?;
    let log = doc.get("nvme_smart_health_information_log")?;
    let num = |v: Option<&Value>| v.and_then(Value::as_u64);
    let pct = |key: &str| num(log.get(key)).and_then(|n| u8::try_from(n).ok());
    Some(Wear {
        read_at: num(doc.get("local_time").and_then(|t| t.get("time_t")))?,
        used_pct: pct("percentage_used")?,
        written: num(log.get("data_units_written"))?.checked_mul(DATA_UNIT_BYTES)?,
        spare_pct: pct("available_spare")?,
        media_errors: num(log.get("media_errors"))?,
    })
}

/// 引用符 1 組で囲んだ字の中身（後ろに注釈のほかの字が続けば None）。
fn quoted(v: &str) -> Option<String> {
    let (body, rest) = v.strip_prefix('"')?.split_once('"')?;
    let rest = rest.trim();
    (rest.is_empty() || rest.starts_with('#')).then(|| body.to_string())
}

/// host の面の字の書きの測りの表の行（宣言の順・name か stat の読めない行は数えない・wear は無くてよい）。
/// ほかの見出しは別の表を始め、別の表の中の行は読まない。
pub fn budget_rows(host_toml: &str) -> Vec<BudgetRow> {
    let mut rows: Vec<(Option<String>, Option<String>, Option<String>)> = Vec::new();
    let mut inside = false;
    for line in host_toml.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line.split('#').next().unwrap_or(line).trim() == BUDGET_HEADER;
            if inside {
                rows.push((None, None, None));
            }
            continue;
        }
        let (Some((name, stat, wear)), true) = (rows.last_mut(), inside) else {
            continue;
        };
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let slot = match key.trim() {
            "name" => name,
            "stat" => stat,
            "wear" => wear,
            _ => continue,
        };
        *slot = quoted(value.trim());
    }
    rows.into_iter()
        .filter_map(|(name, stat, wear)| {
            Some(BudgetRow {
                name: name?,
                stat: stat?,
                wear,
            })
        })
        .collect()
}

/// 線を越えた測り（`Gauge::ALL` の順・ちょうどは越えない・読めない測りは越えない）。負荷は cpu の数で割らず、
/// 1 分の負荷の 100 倍と `LOAD_WARN` と cpu の数の積を比べる（割って切り捨てると core 比 1.00〜1.01 の越えを落とす）。
/// 詰まりは avg60 の 100 倍と線を直に比べる（PSI の字は小数 2 桁なので 100 倍に捨てる桁が無い）。
pub fn over(load: &Reading<Load>, pressure: &Pressure) -> Vec<Gauge> {
    let past = |r: &Reading<Stall>, line: u32| matches!(r, Reading::Known(s) if s.avg60 > line);
    let busy = |l: &Load| u64::from(l.one) > u64::from(LOAD_WARN) * u64::from(l.cores);
    Gauge::ALL
        .into_iter()
        .filter(|g| match g {
            Gauge::Load => matches!(load, Reading::Known(l) if busy(l)),
            Gauge::Io => past(&pressure.io_full, IO_FULL_WARN),
            Gauge::Memory => past(&pressure.memory_full, MEMORY_FULL_WARN),
        })
        .collect()
}

/// 読めた値は Known・読めなければ Unknown。
fn known<T>(v: Option<T>) -> Reading<T> {
    v.map_or(Reading::Unknown, Reading::Known)
}

/// 材料の字から電文を組む（読めない材料はその欄だけ Unknown）。
pub fn host_doc(texts: &HostTexts, now: EpochSecs) -> HostDoc {
    let psi = |t: &Option<String>, kind: &str| known(t.as_deref().and_then(|t| stall(t, kind)));
    let load = known(
        texts
            .loadavg
            .as_deref()
            .zip(texts.online.as_deref())
            .and_then(|(l, o)| load(l, o)),
    );
    let pressure = Pressure {
        cpu_some: psi(&texts.psi_cpu, "some"),
        memory_full: psi(&texts.psi_memory, "full"),
        io_full: psi(&texts.psi_io, "full"),
    };
    let devices = texts
        .devices
        .iter()
        .map(|d| DeviceWrite {
            name: d.name.clone(),
            rate: d.rate.clone(),
            wear: d.wear.as_ref().map(|t| known(t.as_deref().and_then(wear))),
        })
        .collect();
    HostDoc {
        at: now,
        over: over(&load, &pressure),
        load,
        pressure,
        memory: known(texts.meminfo.as_deref().and_then(memory)),
        scopes: known(texts.scopes.as_deref().map(scopes)),
        devices,
    }
}
