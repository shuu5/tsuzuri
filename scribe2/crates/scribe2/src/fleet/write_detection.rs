//! 書き込みの検出線の判定と doctor の行 — host の根の記録（`open` と `days.log`）と rules 行 4 本から、表の行ごとに今日・昨日・昨日で
//! 終わる窓の 1 日平均・越えた線・続いた越えの日数・持ち主の段を求める（設計 write-budget.md §5・§6・ADR-0112・FR113）。
//!
//! 判定は [`judge`] の 1 本だけが持ち、doctor の行（[`doctor_lines`]）と席の 1 行（hook の UserPromptSubmit）はその値を写す。
//! 値の無い日（`unmeasured` と行の無い日）は 0 と数えず、一部が欠けた日は下限（`:partial`）として数え、下限だけで線を越えた日も
//! 越えと数える（C10・NFR4）。線の行を読めない周は線に依る欄を `no-rule` にし、既定の値で埋めない（C5）。器は作業を止めない。
//! 記録の `open` は [`super::write_budget`] の読み手で読み、`days.log` の読み手はここに 1 つ置く。

use super::cli::format_utc;
use super::write_budget::{date_of, decimal, read_open, Day, Sighting, DAYS};
use crate::rules::int_row;
use crate::rules::manifest::Manifest;
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

/// 1 日の秒数（UTC）。
const DAY_S: u64 = 86_400;

/// 線の値の単位（10^9 byte・TBW と同じ十進）。
const GB: u64 = 1_000_000_000;

/// 平均の線の rules 行（10^9 byte）。
pub const ROW_AVG_GB: &str = "host.write_avg_gb";

/// 1 日の線の rules 行（10^9 byte）。
pub const ROW_DAY_GB: &str = "host.write_day_gb";

/// 平均の窓の日数の rules 行（昨日で終わる日数）。
pub const ROW_AVG_DAYS: &str = "host.write_avg_days";

/// 持ち主の段の連続日数の rules 行。
pub const ROW_OWNER_DAYS: &str = "host.write_owner_days";

/// 線を読めない周の語。
const NO_RULE: &str = "no-rule";

/// 記録を読めない周の語。
const UNREADABLE: &str = "unreadable";

/// 値の無い周の語。
const UNMEASURED: &str = "unmeasured";

/// 表の 1 行の判定の値（doctor の行の 11 key の値の字・key の順は [`Verdict::line`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// 表の name。
    pub name: String,
    /// 今日（UTC）の書き込み。
    pub today: String,
    /// 昨日の閉じた値。
    pub yesterday: String,
    /// 昨日で終わる窓の 1 日平均（`:partial` は下限）。
    pub avg: String,
    /// 窓のうち値の在る日（`<k>/<W>`）。
    pub days: String,
    /// 越えた線（`-`・`day`・`avg`・`day,avg`・`no-rule`）。
    pub over: String,
    /// 昨日から続いた越えの閉じた日の数。
    pub streak: String,
    /// 持ち主の段（`yes`・`no`）。
    pub owner: String,
    /// 線の値（`<平均の線 byte>/<1 日の線 byte>`）。
    pub cap: String,
    /// 最後に読めた標本の時刻（`open` の at）。
    pub sampled: String,
    /// 最後の読みの結果（`open` の probe）。
    pub probe: String,
}

impl Verdict {
    /// doctor の 1 行（key の順は固定・設計 §6）。
    pub fn line(&self) -> String {
        format!(
            "write-budget: name={} today={} yesterday={} avg={} days={} over={} streak={} owner={} cap={} sampled={} probe={}",
            self.name,
            self.today,
            self.yesterday,
            self.avg,
            self.days,
            self.over,
            self.streak,
            self.owner,
            self.cap,
            self.sampled,
            self.probe
        )
    }
}

/// 線と窓と段（線は byte に直した値・乗算は飽和）。
struct Lines {
    avg: u64,
    day: u64,
    window: u64,
    owner: u64,
}

/// 1 日の値（written の byte と、下限か）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Value {
    bytes: u64,
    partial: bool,
}

impl Value {
    fn word(self) -> String {
        if self.partial { format!("{}:partial", self.bytes) } else { self.bytes.to_string() }
    }
}

/// `days.log` の読み（date の字ごとの最後の行の値・`unmeasured` の日は値なし）。
type Days = HashMap<String, Option<Value>>;

/// `days.log` の 1 行（6 語がこの順で各 key= を持ち、schema が 1・date が `YYYY-MM-DD` の形・written が `-` なのは unmeasured の
/// ときだけ・数が 10 進・tail が `-` か 10 進のときだけ読める）。
fn day_of(line: &str) -> Option<(String, Option<Value>)> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let keys = ["schema", "date", "written", "state", "reboots", "tail"];
    if words.len() != keys.len() {
        return None;
    }
    let values: Vec<&str> = words.iter().zip(keys).map(|(word, key)| word.strip_prefix(key)?.strip_prefix('=')).collect::<Option<_>>()?;
    let [schema, date, written, state, reboots, tail] = values.as_slice() else {
        return None;
    };
    let shaped = date.len() == 10 && date.char_indices().all(|(at, found)| if at == 4 || at == 7 { found == '-' } else { found.is_ascii_digit() });
    if *schema != "1" || !shaped || decimal(reboots).is_none() || (*tail != "-" && decimal(tail).is_none()) {
        return None;
    }
    let value = match (Day::parse(state)?, *written) {
        (Day::Unmeasured, "-") => None,
        (Day::Measured, found) => Some(Value { bytes: decimal(found)?, partial: false }),
        (Day::Partial, found) => Some(Value { bytes: decimal(found)?, partial: true }),
        (Day::Unmeasured, _) => return None,
    };
    Some(((*date).to_owned(), value))
}

/// 記録の dir の `days.log` を読む（`None` は読めない＝形でない行が 1 行でも在る file か読めない file・無い file は空の読み）。
fn read_days(dir: &Path) -> Option<Days> {
    let text = match fs::read_to_string(dir.join(DAYS)) {
        Ok(text) => text,
        Err(err) if err.kind() == ErrorKind::NotFound => return Some(Days::new()),
        Err(_) => return None,
    };
    text.lines().map(day_of).collect::<Option<Vec<_>>>().map(|rows| rows.into_iter().collect())
}

/// 日の番号 `day`（UNIX 秒 ÷ 86400）の値（行の無い日と unmeasured の日は `None`）。
fn value_at(days: &Days, day: u64) -> Option<Value> {
    days.get(&date_of(day.saturating_mul(DAY_S))).copied().flatten()
}

/// `last` で終わる `window` 日の和の切り捨ての平均・値の在る日の数・下限か（全日 measured でない）。
fn mean(days: &Days, last: u64, window: u64) -> (u64, u64, bool) {
    let found: Vec<Value> = (0..window).filter_map(|back| last.checked_sub(back)).filter_map(|day| value_at(days, day)).collect();
    let sum = found.iter().fold(0_u64, |total, value| total.saturating_add(value.bytes));
    let count = u64::try_from(found.len()).unwrap_or(u64::MAX);
    (sum / window.max(1), count, count < window || found.iter().any(|value| value.partial))
}

/// 閉じた日 `day` の越え: 値が在り、その値が 1 日の線を越えるか、その日で終わる窓の平均が平均の線を越える。
fn over_on(days: &Days, day: u64, lines: &Lines) -> bool {
    value_at(days, day).is_some_and(|value| value.bytes > lines.day || mean(days, day, lines.window).0 > lines.avg)
}

/// 線と窓と段の 4 行（どれかを読めない周と窓が 0 の周は `None`）。
fn lines_of(manifest: &Manifest) -> Option<Lines> {
    let row = |id| int_row(manifest, id).ok();
    let (avg, day, window, owner) = (row(ROW_AVG_GB)?, row(ROW_DAY_GB)?, row(ROW_AVG_DAYS)?, row(ROW_OWNER_DAYS)?);
    (window > 0).then(|| Lines { avg: avg.saturating_mul(GB), day: day.saturating_mul(GB), window, owner })
}

/// 線に依る 6 欄（avg・days・over・streak・owner・cap）。`today` は今日の値、`log` は `days.log` の読み、`yesterday` は昨日の番号。
fn lined(lines: Option<&Lines>, log: Option<&Days>, today: Option<Value>, yesterday: u64) -> [String; 6] {
    let Some(lines) = lines else {
        return std::array::from_fn(|_| NO_RULE.to_owned());
    };
    let cap = format!("{}/{}", lines.avg, lines.day);
    let today_over = today.is_some_and(|value| value.bytes > lines.day);
    let Some(days) = log else {
        let over = if today_over { "day" } else { "-" };
        return [UNREADABLE.to_owned(), UNREADABLE.to_owned(), over.to_owned(), UNREADABLE.to_owned(), UNREADABLE.to_owned(), cap];
    };
    let (avg, count, partial) = mean(days, yesterday, lines.window);
    let avg_word = match (count, partial) {
        (0, _) => UNMEASURED.to_owned(),
        (_, true) => Value { bytes: avg, partial: true }.word(),
        (_, false) => avg.to_string(),
    };
    let day_over = today_over || value_at(days, yesterday).is_some_and(|value| value.bytes > lines.day);
    let over = match (day_over, count > 0 && avg > lines.avg) {
        (true, true) => "day,avg",
        (true, false) => "day",
        (false, true) => "avg",
        (false, false) => "-",
    };
    let streak = (0..=yesterday).rev().take_while(|day| over_on(days, *day, lines)).count();
    let owner = if u64::try_from(streak).unwrap_or(u64::MAX) >= lines.owner { "yes" } else { "no" };
    [avg_word, format!("{count}/{}", lines.window), over.to_owned(), streak.to_string(), owner.to_owned(), cap]
}

/// 表の 1 行の判定（記録の dir・線・今）。判定の順: 記録の読み（open・days.log）→ today / yesterday → 窓の和 → 越え → streak → owner。
fn verdict(name: &str, dir: &Path, lines: Option<&Lines>, now: u64) -> Verdict {
    let (open, log) = (read_open(dir), read_days(dir));
    let yesterday = (now / DAY_S).saturating_sub(1);
    let today = match &open {
        Sighting::Read(found) if found.date == date_of(now) => Some(Value { bytes: found.written, partial: found.state == Day::Partial }),
        _ => None,
    };
    let today_word = match (&open, today) {
        (Sighting::Unreadable, _) => UNREADABLE.to_owned(),
        (_, Some(value)) => value.word(),
        (_, None) => UNMEASURED.to_owned(),
    };
    let yesterday_word = log.as_ref().map_or_else(
        || UNREADABLE.to_owned(),
        |days| value_at(days, yesterday).map_or_else(|| UNMEASURED.to_owned(), Value::word),
    );
    let (sampled, probe) = match &open {
        Sighting::Read(found) => (format_utc(found.at), found.probe.as_str().to_owned()),
        _ => ("-".to_owned(), "-".to_owned()),
    };
    let [avg, days, over, streak, owner, cap] = lined(lines, log.as_ref(), today, yesterday);
    Verdict { name: name.to_owned(), today: today_word, yesterday: yesterday_word, avg, days, over, streak, owner, cap, sampled, probe }
}

/// 判定の 1 本: 置き場の host の面を合わせた manifest（`rules` は `--rules` の path・無ければ埋め込み）の表の行ごとの値を宣言順に返す。
/// 合わせた manifest を読めない周と表が 0 行の周は空の列。線の 4 行のどれかを読めない周と窓が 0 の周は線に依る欄が `no-rule`。
pub fn judge(state_dir: &Path, rules: Option<&str>, now: u64) -> Vec<Verdict> {
    let Ok(merged) = crate::rules::read(rules.map(Path::new), Some(state_dir)) else {
        return Vec::new();
    };
    let lines = lines_of(&merged);
    merged
        .write_budgets()
        .iter()
        .map(|row| verdict(row.name(), &crate::seat::host_write_budget_dir(state_dir, row.name()), lines.as_ref(), now))
        .collect()
}

/// doctor の行（表の行ごとに 1 行・宣言順・表の無い置き場と面を読めない置き場は 0 行）。
pub fn doctor_lines(state_dir: &Path, rules: Option<&str>) -> Vec<String> {
    judge(state_dir, rules, crate::seat::state::now_secs()).iter().map(Verdict::line).collect()
}
