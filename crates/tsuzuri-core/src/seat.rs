//! 席の card（設計ノート surface-base 便 e-seat・規則の行 R-22）。
//! 入力は席の名・anchor の path・器の 3 つの出力（合図の健康・doctor・残量）と 3 つの file の字
//! （状態の記録・合図の最後の判定・群の宣言）・群の記録の字の列・今の時刻。どの関数も file も子 process も
//! 時計も触らない。状態の判定は器の値を写すだけで、閾値を持たない。読めない字はその字から組む欄だけを
//! 「まだ分からない」か無しにする。群の宣言（host.toml）は TOML の読み手を使わず行の字を読み、
//! 1 行に収まらない配列は読めない扱いにする。
//! 限度の再開の時刻は合図の健康の出力の席の行の欄 `reopens=` を写すだけで、窓の実測や rules 行から計算しない。

use serde::Deserialize;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{GroupRow, QuotaLeft, Reading};
use tsuzuri_contract::seat::{
    AccountMove, QuotaUsed, Reopens, SeatCard, SeatSpan, SeatState, TickHealth,
};

use crate::ledger::{DAY, epoch_secs};

/// 窓の名（5 時間窓・7 日窓・model の 7 日窓の順）。
pub const WINDOWS: [&str; 3] = ["five_hour", "seven_day", "seven_day_model"];

/// 合図の最後の判定の理由のうち、限度にする語。
pub const PRESSED: &str = "account-pressed";

/// 合図の最後の判定の理由のうち、応答なしにする語。
pub const STALE: &str = "state-stale";

/// 群の宣言の塊の頭の行。
pub const GROUP_HEADER: &str = "[[account-group]]";

/// 合図の健康の出力の行の頭。
const TICK_PREFIX: &str = "seat tick status:";

/// doctor の席の行の頭。
const SEAT_PREFIX: &str = "seat:";

/// 残量の出力の行の頭。
const USAGE_PREFIX: &str = "usage:";

/// 席の card の材料の字（無い字は None・群の記録は読めた file の字の列）。
/// 面の歯の fixture を同じ形で読むために電文から読める形にしておく。
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct SeatTexts {
    /// (a) 合図の健康の出力（`seat tick status --state-dir <dir>`）。
    pub tick_status: Option<String>,
    /// (b) doctor の出力（`doctor --state-dir <dir>`）。
    pub doctor: Option<String>,
    /// (c) 残量の出力（`fleet usage --show --state-dir <dir>`）。
    pub usage: Option<String>,
    /// (d) 状態の記録（`<state dir>/seat/<席の dir>/state.jsonl`）。
    pub state_log: Option<String>,
    /// (e) 合図の最後の判定（`<state dir>/seat/<席の dir>/tick-last`）。
    pub tick_last: Option<String>,
    /// (f) 群の宣言（`<state dir>/host.toml`）。
    pub host_toml: Option<String>,
    /// (g) 群の記録（今の記録と過去の記録の字）。
    #[serde(default)]
    pub records: Vec<String>,
}

/// 行の `鍵=値` の欄（空白で区切った字のうち `=` を持つもの）。
fn fields(line: &str) -> impl Iterator<Item = (&str, &str)> {
    line.split_whitespace().filter_map(|t| t.split_once('='))
}

/// 行の最初の `鍵=値` の値。
fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    fields(line).find(|(k, _)| *k == key).map(|(_, v)| v)
}

/// 頭が `prefix` の行のうち、`key` の値が `value` の最初の行（頭を除いた字）。
fn line_of<'a>(text: &'a str, prefix: &str, key: &str, value: &str) -> Option<&'a str> {
    text.lines()
        .filter_map(|l| l.trim().strip_prefix(prefix))
        .find(|l| field(l, key) == Some(value))
}

/// doctor の出力のうち、target が席の名と同じ席の行。
fn seat_line<'a>(doctor: &'a str, target: &str) -> Option<&'a str> {
    line_of(doctor, SEAT_PREFIX, "target", target)
}

/// doctor の出力の席の行の anchor（server はこの値で群の名を決める）。
pub fn anchor(doctor: &str, target: &str) -> Option<String> {
    seat_line(doctor, target)
        .and_then(|l| field(l, "anchor"))
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

/// 2 つの字が真偽の語のどちらかなら真偽、どちらでもなければ「まだ分からない」。
fn flag(value: Option<&str>, yes: &str, no: &str) -> Reading<bool> {
    match value {
        Some(v) if v == yes => Reading::Known(true),
        Some(v) if v == no => Reading::Known(false),
        _ => Reading::Unknown,
    }
}

/// 群の宣言の 1 つの塊（名の行と anchors の行・1 行に収まらない anchors は None）。
#[derive(Debug, Default)]
struct Declared {
    name: Option<String>,
    anchors: Option<Vec<String>>,
}

/// 引用符で囲んだ字の中身（後ろに注釈のほかの字が続けば None）。
fn quoted(s: &str) -> Option<String> {
    let q = s.chars().next().filter(|c| matches!(c, '"' | '\''))?;
    let (body, rest) = s[1..].split_once(q)?;
    let rest = rest.trim();
    (rest.is_empty() || rest.starts_with('#')).then(|| body.to_string())
}

/// 1 行の配列（角括弧で囲み、中は引用符で囲んだ字をコンマで区切る）。閉じの括弧が無い配列は None。
fn one_line_array(s: &str) -> Option<Vec<String>> {
    let (inner, rest) = s.strip_prefix('[')?.split_once(']')?;
    let rest = rest.trim();
    if !(rest.is_empty() || rest.starts_with('#')) {
        return None;
    }
    inner
        .split(',')
        .map(str::trim)
        .filter(|i| !i.is_empty())
        .map(quoted)
        .collect()
}

/// 群の宣言の塊を宣言の順に読む。
fn declared(host_toml: &str) -> Vec<Declared> {
    let mut out = Vec::new();
    let mut current: Option<Declared> = None;
    for line in host_toml.lines().map(str::trim) {
        if line.starts_with('[') {
            out.extend(current.take());
            let head = line.split('#').next().unwrap_or(line).trim();
            if head == GROUP_HEADER {
                current = Some(Declared::default());
            }
            continue;
        }
        let Some(group) = current.as_mut() else {
            continue;
        };
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "name" => group.name = quoted(value.trim()),
            "anchors" => group.anchors = one_line_array(value.trim()),
            _ => {}
        }
    }
    out.extend(current);
    out
}

/// 末尾の「/」を除いて同じ path か。
fn same_path(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

/// 群の宣言の字の中で、宣言の順で最初に anchor を含む群の名（無ければ None）。
/// anchors の配列が 1 行に収まらない群は読めないので含むと数えない。
pub fn group_name(host_toml: &str, anchor: &str) -> Option<String> {
    declared(host_toml)
        .into_iter()
        .filter(|g| g.name.is_some())
        .find(|g| {
            g.anchors
                .as_ref()
                .is_some_and(|a| a.iter().any(|p| same_path(p, anchor)))
        })
        .and_then(|g| g.name)
}

/// 残量の出力の窓の 1 つ（使った百分率・戻る時刻・model の窓なら model の名）。
struct Window {
    name: &'static str,
    used: u64,
    resets: Option<EpochSecs>,
    model: Option<String>,
}

/// 残量の出力のうち、account が口座と同じ行。
fn usage_line<'a>(usage: &'a str, account: &str) -> Option<&'a str> {
    line_of(usage, USAGE_PREFIX, "account", account)
}

/// 測れた行の窓（unmeasured の行と、形の読めない行は None）。
fn windows(line: &str) -> Option<Vec<Window>> {
    let mut tokens = line.split_whitespace();
    let mut out = Vec::new();
    while let Some(token) = tokens.next() {
        if token == "unmeasured" {
            return None;
        }
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };
        let (name, model, pct) = match key {
            "five_hour" => (WINDOWS[0], None, value),
            "seven_day" => (WINDOWS[1], None, value),
            "model" => {
                let (m, p) = value.rsplit_once(':')?;
                (WINDOWS[2], Some(m.to_string()), p)
            }
            _ => continue,
        };
        let used = pct.strip_suffix('%')?.parse().ok()?;
        let resets = match tokens.next()?.strip_prefix("resets=")? {
            "none" => None,
            t => Some(epoch_secs(t)?),
        };
        out.push(Window {
            name,
            used,
            resets,
            model,
        });
    }
    (!out.is_empty()).then_some(out)
}

/// 口座の測れた窓（字が無い・行が無い・unmeasured なら None）。
fn measured(usage: Option<&str>, account: &str) -> Option<Vec<Window>> {
    windows(usage_line(usage?, account)?)
}

/// 席の口座の窓ごとの使った割合（使った割合は 255 で止める・model の窓は席の model と同じときだけ数える）。
fn quota_used(
    usage: Option<&str>,
    account: Option<&str>,
    model: Option<&str>,
) -> Reading<Vec<QuotaUsed>> {
    let Some(windows) = account.and_then(|a| measured(usage, a)) else {
        return Reading::Unknown;
    };
    Reading::Known(
        windows
            .into_iter()
            .map(|w| QuotaUsed {
                window: w.name.to_string(),
                used_pct: u8::try_from(w.used).unwrap_or(u8::MAX),
                resets_at: w.resets,
                counted: w
                    .model
                    .as_deref()
                    .is_none_or(|m| model.is_some_and(|s| s.eq_ignore_ascii_case(m))),
            })
            .collect(),
    )
}

/// 今の口座の窓ごとの残り（100 から使った割合を引く・0 より小さくしない・測れていなければ空の列）。
fn quota_left(usage: Option<&str>, account: &str) -> Vec<QuotaLeft> {
    measured(usage, account)
        .unwrap_or_default()
        .into_iter()
        .map(|w| QuotaLeft {
            window: w.name.to_string(),
            left_pct: u8::try_from(100 - w.used.min(100)).unwrap_or(0),
        })
        .collect()
}

/// 群の行（群の名と doctor の同じ名の群の行から組む・どちらかが無ければ「まだ分からない」）。
fn group(texts: &SeatTexts, name: Option<&str>) -> Reading<GroupRow> {
    let line = name.zip(texts.doctor.as_deref()).and_then(|(n, d)| {
        d.lines()
            .map(str::trim)
            .find(|l| l.starts_with("group=") && field(l, "group") == Some(n))
    });
    let (Some(name), Some(line)) = (name, line) else {
        return Reading::Unknown;
    };
    let Some(current) = field(line, "current").filter(|v| !v.is_empty()) else {
        return Reading::Unknown;
    };
    Reading::Known(GroupRow {
        group: name.to_string(),
        account: current.to_string(),
        candidates: field(line, "accounts")
            .unwrap_or_default()
            .split(',')
            .filter(|a| !a.is_empty())
            .map(str::to_string)
            .collect(),
        next_account: field(line, "next")
            .filter(|v| !v.is_empty() && *v != "none")
            .map(str::to_string),
        remaining: quota_left(texts.usage.as_deref(), current),
    })
}

/// 状態の記録の 1 行（読む欄だけ）。
#[derive(Deserialize)]
struct StateLine {
    state: String,
    ts: serde_json::Number,
}

/// 状態の記録の読めた行（ts と、busy は run・idle は wait に写した状態）を file の順に。
/// JSON でない行・state が busy でも idle でもない行・ts が数でない行は飛ばす。
fn state_rows(text: &str) -> Vec<(EpochSecs, SeatState)> {
    text.lines()
        .filter_map(|l| serde_json::from_str::<StateLine>(l.trim()).ok())
        .filter_map(|r| {
            let state = match r.state.as_str() {
                "busy" => SeatState::Run,
                "idle" => SeatState::Wait,
                _ => return None,
            };
            let ts = r.ts.as_u64().or_else(|| {
                r.ts.as_f64()
                    .filter(|f| f.is_finite() && *f >= 0.0)
                    .map(|f| f as u64)
            })?;
            Some((ts, state))
        })
        .collect()
}

/// doctor の席の行の tick の語（4 つの語のどれにも当たらなければ「まだ分からない」）。
fn tick_health(value: Option<&str>) -> Reading<TickHealth> {
    value
        .and_then(|v| TickHealth::ALL.into_iter().find(|t| t.as_str() == v))
        .map_or(Reading::Unknown, Reading::Known)
}

/// 席の行の欄 reopens の値（`-` は当たっていない・`unknown` は時刻が無い・
/// `YYYY-MM-DDTHH:MM:SSZ` は時刻・ほかと欄が無いのは測れていない）。
fn reopens(value: Option<&str>) -> Reopens {
    match value {
        Some("-") => Reopens::Clear,
        Some("unknown") => Reopens::Unknown,
        Some(t) if t.len() == 20 && t.ends_with('Z') => {
            epoch_secs(t).map_or(Reopens::Unmeasured, Reopens::At)
        }
        _ => Reopens::Unmeasured,
    }
}

/// 合図の最後の判定の字の空でない最後の行。
fn last_tick(tick_last: Option<&str>) -> Option<&str> {
    tick_last?.lines().map(str::trim).rfind(|l| !l.is_empty())
}

/// 合図の最後の判定の行の時刻（ts の数・読めなければ None）。
fn tick_ts(line: &str) -> Option<EpochSecs> {
    field(line, "ts").and_then(|t| t.parse().ok())
}

/// 状態と、今の状態になった時刻。合図の最後の判定の理由が account-pressed なら limit・
/// state-stale なら silent（どちらも時刻は判定の ts）。それ以外は状態の記録の最後の行の状態で、
/// 時刻は末尾から同じ状態が続く行のうち最も古い ts。記録が無いか読めた行が無ければ unknown。
fn state_of(
    tick_last: Option<&str>,
    rows: Option<&[(EpochSecs, SeatState)]>,
) -> (SeatState, Option<EpochSecs>) {
    if let Some(line) = last_tick(tick_last) {
        let ts = tick_ts(line);
        match field(line, "reason") {
            Some(PRESSED) => return (SeatState::Limit, ts),
            Some(STALE) => return (SeatState::Silent, ts),
            _ => {}
        }
    }
    let Some(rows) = rows else {
        return (SeatState::Unknown, None);
    };
    let Some(&(_, last)) = rows.last() else {
        return (SeatState::Unknown, None);
    };
    let since = rows
        .iter()
        .rev()
        .take_while(|(_, s)| *s == last)
        .map(|(t, _)| *t)
        .min();
    (last, since)
}

/// 今の時刻の 24 時間前以後の行を ts の順に並べ、同じ状態が続く行を 1 つの区間にまとめる
/// （区間の終わりは次の区間の始まり・最後の区間は今の時刻）。
fn spans(rows: &[(EpochSecs, SeatState)], now: EpochSecs) -> Vec<SeatSpan> {
    let from = now.saturating_sub(DAY);
    let mut rows: Vec<_> = rows.iter().copied().filter(|(t, _)| *t >= from).collect();
    rows.sort_by_key(|(t, _)| *t);
    let mut out: Vec<SeatSpan> = Vec::new();
    for (t, state) in rows {
        if out.last().is_some_and(|s| s.state == state) {
            continue;
        }
        if let Some(prev) = out.last_mut() {
            prev.to = t;
        }
        out.push(SeatSpan {
            from: t,
            to: now,
            state,
        });
    }
    out
}

/// 群の記録の時刻（epoch 秒の数か、RFC 3339 の時刻）。
fn record_time(s: &str) -> Option<EpochSecs> {
    if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) {
        return s.parse().ok();
    }
    epoch_secs(s)
}

/// 群の記録の 1 つを移動にする（account か ts が読めなければ None・previous が無いか空なら from は無し）。
fn account_move(record: &str) -> Option<AccountMove> {
    let value = |key: &str| {
        record
            .lines()
            .find_map(|l| l.trim().strip_prefix(key)?.strip_prefix('='))
            .map(str::trim)
    };
    let to = value("account").filter(|v| !v.is_empty())?;
    Some(AccountMove {
        at: record_time(value("ts")?)?,
        from: value("previous")
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        to: to.to_string(),
    })
}

/// 群の記録の字の列を時刻の古い順の移動にする（読めない記録は飛ばす）。
fn moves(records: &[String]) -> Vec<AccountMove> {
    let mut out: Vec<AccountMove> = records.iter().filter_map(|r| account_move(r)).collect();
    out.sort_by_key(|m| m.at);
    out
}

/// 席の card を組む（`anchor` は席の anchor の path・`now` は今の時刻）。
pub fn card(target: &str, anchor: Option<&str>, texts: &SeatTexts, now: EpochSecs) -> SeatCard {
    let seat = texts.doctor.as_deref().and_then(|d| seat_line(d, target));
    let seat_value = |key: &str| {
        seat.and_then(|l| field(l, key))
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    };
    let (account, model) = (seat_value("account"), seat_value("model"));
    let tick = texts
        .tick_status
        .as_deref()
        .and_then(|t| line_of(t, TICK_PREFIX, "target", target));
    let rows = texts.state_log.as_deref().map(state_rows);
    let (state, since) = state_of(texts.tick_last.as_deref(), rows.as_deref());
    let name = texts
        .host_toml
        .as_deref()
        .zip(anchor)
        .and_then(|(h, a)| group_name(h, a));
    SeatCard {
        at: now,
        target: target.to_string(),
        state,
        since,
        tick_healthy: tick.map_or(Reading::Unknown, |l| flag(field(l, "healthy"), "yes", "no")),
        heartbeat: tick.map_or(Reading::Unknown, |l| {
            flag(field(l, "heartbeat"), "on", "off")
        }),
        tick: tick_health(seat.and_then(|l| field(l, "tick"))),
        tick_at: last_tick(texts.tick_last.as_deref()).and_then(tick_ts),
        reopens: reopens(tick.and_then(|l| field(l, "reopens"))),
        group: group(texts, name.as_deref()),
        usage: quota_used(texts.usage.as_deref(), account.as_deref(), model.as_deref()),
        spans: rows.map_or(Reading::Unknown, |r| Reading::Known(spans(&r, now))),
        moves: if name.is_some() {
            Reading::Known(moves(&texts.records))
        } else {
            Reading::Unknown
        },
        account,
        model,
    }
}
