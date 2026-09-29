//! host の側の部分（口座の列・群の枠・移動の列・設計ノート surface-base 便 e-acct-host）。
//! 入力は群の宣言の字（host.toml）・残量の出力の字・host の doctor の字（群の行と口座の行を読む）・
//! anchor ごとの doctor の字の表（席の行を読む）・群の記録の字（今の記録と history の記録）。
//! doctor の区画の行（欄 kind が `PARK_KIND` の行・行 c-acct-park）は口座を占有せず、群の行に数えない。
//! 群の宣言は TOML の読み手を使わず、`[[account]]` の label と `[[account-group]]` の name・anchors・accounts だけを読む。
//! 読めない字の決まり（要件 NFR2）: host の doctor の字が無ければ口座の列と群の列が、群の宣言の字が無ければ
//! 3 つの列とも「まだ分からない」。残量の字が無いか口座の行が測れていなければ、その口座の usage だけが「まだ分からない」。
//! 窓ごとの逼迫の閾値は器の rules 行の出力の字を、群の逼迫の知らせと移動の断りは器の event log の行を写すだけで、
//! 閾値の数を持たず判じない（便 c-acct-thr・規則の行 R-22）。
//! 口座の 7 日の線は引数の state dir の event log の測りの行を 1 度だけ読み、今から 1 時間の刻みごとに最も新しい行を
//! 点にする（行 c-acct-spark）。log の字が無ければ線は「まだ分からない」で、最後に測った時刻の古さは判じない。

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;
use tsuzuri_contract::account::{
    AccountRow, GroupCard, GroupMember, GroupNotice, MoveRow, SPARK_SPAN_S, SPARK_STEP_S, Spark,
    SparkLine, SparkPoint, WindowCap,
};
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{GroupRow, Reading};
use tsuzuri_contract::seat::QuotaUsed;

use super::{field, project_name, same_path, value};
use crate::ledger::epoch_secs;
use crate::seat::{GROUP_HEADER, WINDOWS};

/// 口座の宣言の塊の頭の行。
pub const ACCOUNT_HEADER: &str = "[[account]]";

/// 群の記録のうち、移動の行と今の記録に読む種類（file の名の最初の点の後）。
pub const RECORD_KIND: &str = "account";

/// 群の枠の席に数える役。
pub const ORCHESTRATOR: &str = "orchestrator";

/// 窓ごとの逼迫の閾値の器の rules 行（窓の名・行の id・`WINDOWS` の順・値は器の行から読み code に書かない）。
pub const CAP_ROWS: [(&str, &str); 3] = [
    (WINDOWS[0], "fleet.group_pressure_5h_pct"),
    (WINDOWS[1], "fleet.group_pressure_7d_pct"),
    (WINDOWS[2], "fleet.group_pressure_model_pct"),
];

/// 器の逼迫の知らせの event の種類。
pub const PRESSURE_EVENT: &str = "GroupPressureNotified";

/// 器の移動の断りの event の種類。
pub const REFUSED_EVENT: &str = "GroupMoveRefused";

/// 器の口座の測りの event の種類（口座の線の材料・行 c-acct-spark）。
pub const MEASURED_EVENT: &str = "AllowanceMeasured";

/// doctor の区画の行の欄 kind の字（口座を占有しない区画・群の行は kind を持たない）。
pub const PARK_KIND: &str = "park";

/// 器の知らせの窓の語と窓の名。
pub const WINDOW_WORDS: [(&str, &str); 3] = [
    ("5h", WINDOWS[0]),
    ("7d", WINDOWS[1]),
    ("model", WINDOWS[2]),
];

/// 残量の出力の行の頭。
const USAGE_PREFIX: &str = "usage:";

/// doctor の席の行の頭。
const SEAT_PREFIX: &str = "seat:";

/// host の側の材料の字（無い字は None）。面の歯の fixture を同じ形で読むために電文から読める形にしておく。
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct HostTexts {
    /// 群の宣言（`<state dir>/host.toml`）。
    pub host_toml: Option<String>,
    /// 残量の出力（`fleet usage --show`）。
    pub usage: Option<String>,
    /// host の doctor の出力（群の行と口座の行を読む）。
    pub doctor: Option<String>,
    /// anchor の path → その anchor の state dir の doctor の出力（席の行を読む・引けない anchor は無い）。
    #[serde(default)]
    pub seat_doctors: BTreeMap<String, String>,
    /// groups の下の file の名 → 字（今の記録は `<群>.account`）。
    #[serde(default)]
    pub records: BTreeMap<String, String>,
    /// groups/history の下の file の名 → 字（`<群>.account.<時刻>.<番号>` ほか）。
    #[serde(default)]
    pub history: BTreeMap<String, String>,
    /// 器の rules 行の id → `rules get <id>` の出力（撃てない行と rc 0 でない行は無い）。
    #[serde(default)]
    pub caps: BTreeMap<String, String>,
    /// 席の名 → その席の状態の記録（`state.jsonl`）の字（読めない file の席は無い・行 c-dormant）。
    #[serde(default)]
    pub seat_logs: BTreeMap<String, String>,
    /// 引数の state dir の event log（`<state dir>/fleet/events.jsonl`）の字（口座の線の材料・行 c-acct-spark）。
    pub events: Option<String>,
}

/// 群の宣言の 1 つの群（配列の欄が無ければ空の列・読めなければ None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredGroup {
    pub name: String,
    pub anchors: Option<Vec<String>>,
    pub accounts: Option<Vec<String>>,
}

/// 群の宣言（口座の label と群を宣言の順に）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Declaration {
    pub accounts: Vec<String>,
    pub groups: Vec<DeclaredGroup>,
}

/// 読んでいる途中の塊。
enum Table {
    Other,
    Account(Option<String>),
    Group {
        name: Option<String>,
        anchors: Option<Vec<String>>,
        accounts: Option<Vec<String>>,
    },
}

/// 引用符で囲んだ字の中身（後ろに注釈のほかの字が続けば None）。
fn quoted(s: &str) -> Option<String> {
    let q = s.chars().next().filter(|c| matches!(c, '"' | '\''))?;
    let (body, rest) = s[1..].split_once(q)?;
    let rest = rest.trim();
    (rest.is_empty() || rest.starts_with('#')).then(|| body.to_string())
}

/// 配列の 1 行の読み（閉じた・まだ続く・読めない形）。
enum Scan<'a> {
    Closed(&'a str),
    Open,
    Bad,
}

/// 配列の 1 行の中の、引用符で囲んだ字を `items` に足す。
fn scan<'a>(line: &'a str, items: &mut Vec<String>) -> Scan<'a> {
    let mut s = line;
    loop {
        s = s.trim_start_matches(|c: char| c.is_whitespace() || c == ',');
        match s.chars().next() {
            None | Some('#') => return Scan::Open,
            Some(']') => return Scan::Closed(&s[1..]),
            Some(q @ ('"' | '\'')) => {
                let Some((body, after)) = s[1..].split_once(q) else {
                    return Scan::Bad;
                };
                items.push(body.to_string());
                s = after;
            }
            _ => return Scan::Bad,
        }
    }
}

/// 配列（`[` で始まる字・中は引用符で囲んだ字）。閉じの括弧が同じ行に無ければ続く行を読み進める。
/// 読めない形の配列と、閉じの括弧が無い配列は None（読めない配列の残りの行は読み捨てる）。
fn array<'a>(first: &'a str, rest: &mut impl Iterator<Item = &'a str>) -> Option<Vec<String>> {
    let mut line = first.strip_prefix('[')?;
    let mut items = Vec::new();
    loop {
        match scan(line, &mut items) {
            Scan::Closed(after) => {
                let after = after.trim();
                return (after.is_empty() || after.starts_with('#')).then_some(items);
            }
            Scan::Bad => {
                if !line.contains(']') {
                    rest.find(|l| l.contains(']'));
                }
                return None;
            }
            Scan::Open => line = rest.next()?,
        }
    }
}

/// 群の宣言の字を読む（label の読めない口座と name の読めない群は数えない）。
pub fn declaration(host_toml: &str) -> Declaration {
    let mut out = Declaration::default();
    let mut current = Table::Other;
    let flush = |table: Table, out: &mut Declaration| match table {
        Table::Account(Some(label)) => out.accounts.push(label),
        Table::Group {
            name: Some(name),
            anchors,
            accounts,
        } => out.groups.push(DeclaredGroup {
            name,
            anchors,
            accounts,
        }),
        _ => {}
    };
    let mut lines = host_toml.lines().map(str::trim);
    while let Some(line) = lines.next() {
        if line.starts_with('[') {
            flush(std::mem::replace(&mut current, Table::Other), &mut out);
            current = match line.split('#').next().unwrap_or(line).trim() {
                ACCOUNT_HEADER => Table::Account(None),
                GROUP_HEADER => Table::Group {
                    name: None,
                    anchors: Some(Vec::new()),
                    accounts: Some(Vec::new()),
                },
                _ => Table::Other,
            };
            continue;
        }
        let Some((key, v)) = line.split_once('=') else {
            continue;
        };
        let v = v.trim();
        match (&mut current, key.trim()) {
            (Table::Account(label), "label") => *label = quoted(v),
            (Table::Group { name, .. }, "name") => *name = quoted(v),
            (Table::Group { anchors, .. }, "anchors") => *anchors = array(v, &mut lines),
            (Table::Group { accounts, .. }, "accounts") => *accounts = array(v, &mut lines),
            _ => {}
        }
    }
    flush(current, &mut out);
    out
}

/// 残量の出力のうち、account が口座と同じ最初の行（頭を除いた字）。
fn usage_line<'a>(usage: &'a str, account: &str) -> Option<&'a str> {
    usage
        .lines()
        .filter_map(|l| l.trim().strip_prefix(USAGE_PREFIX))
        .find(|l| field(l, "account") == Some(account))
}

/// 残量の行の model の窓の model の名。
fn model_name(line: &str) -> Option<String> {
    value(line, "model")
        .and_then(|v| v.rsplit_once(':'))
        .map(|(m, _)| m)
        .filter(|m| !m.is_empty())
        .map(str::to_string)
}

/// 測れた行の窓ごとの使った割合（使った割合は 255 で止める・unmeasured の行と形の読めない行は None）。
fn windows(line: &str) -> Option<Vec<QuotaUsed>> {
    let mut tokens = line.split_whitespace();
    let mut out = Vec::new();
    while let Some(token) = tokens.next() {
        if token == "unmeasured" {
            return None;
        }
        let Some((key, v)) = token.split_once('=') else {
            continue;
        };
        let (name, pct) = match key {
            "five_hour" => (WINDOWS[0], v),
            "seven_day" => (WINDOWS[1], v),
            "model" => (WINDOWS[2], v.rsplit_once(':')?.1),
            _ => continue,
        };
        let used: u64 = pct.strip_suffix('%')?.parse().ok()?;
        let resets_at = match tokens.next()?.strip_prefix("resets=")? {
            "none" => None,
            t => Some(epoch_secs(t)?),
        };
        out.push(QuotaUsed {
            window: name.to_string(),
            used_pct: u8::try_from(used).unwrap_or(u8::MAX),
            resets_at,
            counted: true,
        });
    }
    (!out.is_empty()).then_some(out)
}

/// doctor の頭が `group=` の行（群の行と区画の行）。
fn headed_lines(doctor: &str) -> impl Iterator<Item = &str> {
    doctor
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("group="))
}

/// 区画の行か（欄 kind の字が `PARK_KIND` ちょうど・kind の無い行とほかの字は群の行）。
fn parked(line: &str) -> bool {
    field(line, "kind") == Some(PARK_KIND)
}

/// doctor の群の行（頭が `group=` の行のうち区画の行を除く）。
fn group_lines(doctor: &str) -> impl Iterator<Item = &str> {
    headed_lines(doctor).filter(|l| !parked(l))
}

/// 群の宣言が区画か（doctor に同じ名の区画の行が在る）。
fn in_park(doctor: &str, group: &DeclaredGroup) -> bool {
    headed_lines(doctor).any(|l| parked(l) && field(l, "group") == Some(group.name.as_str()))
}

/// doctor の口座の行のうち、account が口座と同じ最初の行。
fn account_line<'a>(doctor: &'a str, account: &str) -> Option<&'a str> {
    doctor
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("account="))
        .find(|l| field(l, "account") == Some(account))
}

/// 口座の列（群の宣言の label の順）。群の宣言か host の doctor の字が無ければ「まだ分からない」。
/// 占有の群は doctor の群の行のうち current が口座と同じ最初の行の群、退役は doctor の口座の行の retired=yes。
/// 線は「まだ分からない」（線を埋めるのは `accounts_at`）。
pub fn accounts(texts: &HostTexts) -> Reading<Vec<AccountRow>> {
    let (Some(host), Some(doctor)) = (texts.host_toml.as_deref(), texts.doctor.as_deref()) else {
        return Reading::Unknown;
    };
    Reading::Known(
        declaration(host)
            .accounts
            .into_iter()
            .map(|label| {
                let line = texts.usage.as_deref().and_then(|u| usage_line(u, &label));
                AccountRow {
                    retired: account_line(doctor, &label)
                        .is_some_and(|l| field(l, "retired") == Some("yes")),
                    occupant: group_lines(doctor)
                        .find(|l| field(l, "current") == Some(label.as_str()))
                        .and_then(|l| value(l, "group"))
                        .map(str::to_string),
                    model: line.and_then(model_name),
                    usage: line
                        .and_then(windows)
                        .map_or(Reading::Unknown, Reading::Known),
                    spark: Reading::Unknown,
                    label,
                }
            })
            .collect(),
    )
}

/// 読みかけの口座の線（最後に測った時刻と、`WINDOWS` の順の窓ごとの刻み → 刻みの中の最も新しい点）。
type Draft = (Option<EpochSecs>, [BTreeMap<u64, SparkPoint>; 3]);

/// 測りの行を口座の名・窓の位置・点にする（種類が `MEASURED_EVENT` で口座が空でなく、窓が `WINDOWS` のどれかで、
/// ts が読めて今より後でなく、used_pct が整数の行だけ・使った割合は 255 で止める）。字 `MEASURED_EVENT` を含まない行は
/// JSON として読まない。
fn measured_row(line: &str, now: EpochSecs) -> Option<(String, usize, SparkPoint)> {
    if !line.contains(MEASURED_EVENT) {
        return None;
    }
    let event: Value = serde_json::from_str(line).ok()?;
    if event_text(&event, "kind")? != MEASURED_EVENT {
        return None;
    }
    let account = event_text(&event, "account").filter(|a| !a.is_empty())?;
    let window = event_text(&event, "window")?;
    let window = WINDOWS.iter().position(|w| *w == window)?;
    let at = epoch_secs(event_text(&event, "ts")?)?;
    if at > now {
        return None;
    }
    let used = event.get("used_pct")?.as_u64()?;
    Some((
        account.to_string(),
        window,
        SparkPoint {
            at,
            used_pct: u8::try_from(used).unwrap_or(u8::MAX),
        },
    ))
}

/// 点の無い 3 本の線（最後に測った時刻は None）。
fn bare() -> Spark {
    Spark {
        measured_at: None,
        lines: WINDOWS
            .iter()
            .map(|w| SparkLine {
                window: w.to_string(),
                points: Vec::new(),
            })
            .collect(),
    }
}

/// log を 1 度だけ読んだ口座の名 → 線の表。点は今からの秒が `SPARK_SPAN_S` 未満の行で、刻みは今からの秒を
/// `SPARK_STEP_S` で割った商、刻みの中は ts の最も新しい行（同じ ts は log の後の行）、線の点は at の古い順。
fn sparks(log: &str, now: EpochSecs) -> BTreeMap<String, Spark> {
    let mut drafts: BTreeMap<String, Draft> = BTreeMap::new();
    for (account, window, point) in log.lines().filter_map(|l| measured_row(l, now)) {
        let (measured_at, steps) = drafts.entry(account).or_default();
        *measured_at = Some(measured_at.map_or(point.at, |m| m.max(point.at)));
        let age = now - point.at;
        if age >= SPARK_SPAN_S {
            continue;
        }
        let slot = steps[window].entry(age / SPARK_STEP_S).or_insert(point);
        if slot.at <= point.at {
            *slot = point;
        }
    }
    drafts
        .into_iter()
        .map(|(account, (measured_at, steps))| {
            let mut spark = bare();
            spark.measured_at = measured_at;
            for (line, points) in spark.lines.iter_mut().zip(steps) {
                line.points = points.into_values().rev().collect();
            }
            (account, spark)
        })
        .collect()
}

/// 口座の 7 日の線（引数の state dir の event log の測りの行）。log の字が無ければ「まだ分からない」、
/// 口座の読めた行が無ければ最後に測った時刻は None で点の無い 3 本。
pub fn spark(texts: &HostTexts, account: &str, now: EpochSecs) -> Reading<Spark> {
    let Some(log) = texts.events.as_deref() else {
        return Reading::Unknown;
    };
    Reading::Known(sparks(log, now).remove(account).unwrap_or_else(bare))
}

/// `accounts` の口座の列の線を、log の 1 度の読みで埋める（log の字が無ければ `accounts` と同じ・
/// 退役の口座も線を持つ）。
pub fn accounts_at(texts: &HostTexts, now: EpochSecs) -> Reading<Vec<AccountRow>> {
    let mut rows = accounts(texts);
    if let (Reading::Known(rows), Some(log)) = (&mut rows, texts.events.as_deref()) {
        let table = sparks(log, now);
        for row in rows {
            row.spark = Reading::Known(table.get(&row.label).cloned().unwrap_or_else(bare));
        }
    }
    rows
}

/// 群の記録の `鍵=値` の行の値（空なら None）。
fn record_value<'a>(record: &'a str, key: &str) -> Option<&'a str> {
    record
        .lines()
        .find_map(|l| l.trim().strip_prefix(key)?.strip_prefix('='))
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

/// doctor の席の行のうち、役が orchestrator で anchor が同じ path の最初の行。
fn orchestrator_line<'a>(doctor: &'a str, anchor: &str) -> Option<&'a str> {
    doctor
        .lines()
        .filter_map(|l| l.trim().strip_prefix(SEAT_PREFIX))
        .find(|l| {
            field(l, "role") == Some(ORCHESTRATOR)
                && field(l, "anchor").is_some_and(|a| same_path(a, anchor))
        })
}

/// 群の project（その anchor の doctor の字が無いか、席の行か席の口座が無ければ一致は「まだ分からない」）。
fn member(texts: &HostTexts, anchor: &str, current: &str) -> GroupMember {
    let seat_account = texts
        .seat_doctors
        .iter()
        .find(|(a, _)| same_path(a, anchor))
        .and_then(|(_, d)| orchestrator_line(d, anchor))
        .and_then(|l| value(l, "account"))
        .map(str::to_string);
    GroupMember {
        project: project_name(anchor),
        matches: seat_account
            .as_deref()
            .map_or(Reading::Unknown, |a| Reading::Known(a == current)),
        seat_account,
    }
}

/// 1 つの群の枠（doctor に群の行が無いか今の口座が無い・anchors が読めなければ None）。
fn card(texts: &HostTexts, doctor: &str, group: &DeclaredGroup) -> Option<GroupCard> {
    let line = group_lines(doctor).find(|l| field(l, "group") == Some(group.name.as_str()))?;
    let current = value(line, "current")?;
    let anchors = group.anchors.as_ref()?;
    let record = texts
        .records
        .get(&format!("{}.{RECORD_KIND}", group.name))
        .map(String::as_str);
    Some(GroupCard {
        row: GroupRow {
            group: group.name.clone(),
            account: current.to_string(),
            candidates: value(line, "accounts")
                .unwrap_or_default()
                .split(',')
                .filter(|a| !a.is_empty())
                .map(str::to_string)
                .collect(),
            next_account: value(line, "next")
                .filter(|v| *v != "none")
                .map(str::to_string),
            remaining: Vec::new(),
        },
        since: record
            .and_then(|r| record_value(r, "ts"))
            .and_then(epoch_secs),
        previous: record
            .and_then(|r| record_value(r, "previous"))
            .map(str::to_string),
        recorded: record.is_some(),
        members: anchors.iter().map(|a| member(texts, a, current)).collect(),
        refused: value(line, "refused")
            .filter(|v| *v != "-")
            .map(str::to_string),
    })
}

/// 群の枠の列（群の宣言の順・doctor に同じ名の区画の行が在る宣言は列に入れない）。群の宣言か host の doctor の
/// 字が無いか、残りの宣言の群のどれかの枠が組めなければ（doctor に群の行か今の口座が無い・anchors が読めない）
/// 「まだ分からない」。
pub fn groups(texts: &HostTexts) -> Reading<Vec<GroupCard>> {
    let (Some(host), Some(doctor)) = (texts.host_toml.as_deref(), texts.doctor.as_deref()) else {
        return Reading::Unknown;
    };
    declaration(host)
        .groups
        .iter()
        .filter(|g| !in_park(doctor, g))
        .map(|g| card(texts, doctor, g))
        .collect::<Option<Vec<_>>>()
        .map_or(Reading::Unknown, Reading::Known)
}

/// file の名が account の記録なら群の名（`<群>.account` か `<群>.account.<…>`）。
fn record_group(file: &str) -> Option<&str> {
    let (group, kind) = file.split_once('.')?;
    let rest = kind.strip_prefix(RECORD_KIND)?;
    (rest.is_empty() || rest.starts_with('.')).then_some(group)
}

/// 群の記録の 1 つを移動の行にする（account か ts が読めなければ None・previous が無いか空なら from は無し）。
fn move_row(group: &str, record: &str) -> Option<MoveRow> {
    Some(MoveRow {
        at: epoch_secs(record_value(record, "ts")?)?,
        group: group.to_string(),
        from: record_value(record, "previous").map(str::to_string),
        to: record_value(record, "account")?.to_string(),
    })
}

/// 移動の列（今の記録と history の account の記録の全部を ts の新しい順・同じ ts は群の宣言の順）。
/// 宣言に無い群の記録と request・refused の記録は読まない。群の宣言の字が無ければ「まだ分からない」。
pub fn moves(texts: &HostTexts) -> Reading<Vec<MoveRow>> {
    let Some(host) = texts.host_toml.as_deref() else {
        return Reading::Unknown;
    };
    let names: Vec<String> = declaration(host)
        .groups
        .into_iter()
        .map(|g| g.name)
        .collect();
    let mut rows: Vec<(usize, MoveRow)> = texts
        .records
        .iter()
        .chain(&texts.history)
        .filter_map(|(file, text)| {
            let group = record_group(file)?;
            let order = names.iter().position(|n| n == group)?;
            Some((order, move_row(group, text)?))
        })
        .collect();
    rows.sort_by(|(ga, a), (gb, b)| b.at.cmp(&a.at).then(ga.cmp(gb)));
    Reading::Known(rows.into_iter().map(|(_, m)| m).collect())
}

/// 窓ごとの逼迫の閾値（`CAP_ROWS` の順）。行の字の前後の空白を除いた字が数として読めればその値、
/// 字が無いか読めなければ「まだ分からない」。数の範囲は見ない（器の値のまま）。
pub fn caps(texts: &HostTexts) -> Vec<WindowCap> {
    CAP_ROWS
        .iter()
        .map(|&(window, rule)| WindowCap {
            window: window.to_string(),
            rule: rule.to_string(),
            cap: texts
                .caps
                .get(rule)
                .and_then(|t| t.trim().parse::<u64>().ok())
                .map_or(Reading::Unknown, Reading::Known),
        })
        .collect()
}

/// event の欄の字。
fn event_text<'a>(event: &'a Value, key: &str) -> Option<&'a str> {
    event.get(key).and_then(Value::as_str)
}

/// event の 1 行を知らせか断りの行にする（宣言の群の順の位置と行・読めない行は None）。
fn notice(names: &[String], event: &Value) -> Option<(usize, GroupNotice)> {
    let kind = event_text(event, "kind")?;
    if kind != PRESSURE_EVENT && kind != REFUSED_EVENT {
        return None;
    }
    let at = epoch_secs(event_text(event, "ts")?)?;
    let account = event_text(event, "account").filter(|a| !a.is_empty())?;
    let detail = event_text(event, "detail")?;
    let group = value(detail, "group")?;
    let order = names.iter().position(|n| n == group)?;
    let (group, account) = (group.to_string(), account.to_string());
    let row = if kind == PRESSURE_EVENT {
        let word = value(detail, "window")?;
        let (_, window) = WINDOW_WORDS.iter().find(|(w, _)| *w == word)?;
        let number = |key: &str| value(detail, key)?.parse::<u64>().ok();
        GroupNotice::Pressure {
            at,
            group,
            account,
            window: window.to_string(),
            used: number("used")?,
            cap: number("cap")?,
            sent: number("sent")?,
        }
    } else {
        GroupNotice::Refused {
            at,
            group,
            account,
            reason: value(detail, "reason")?.to_string(),
        }
    };
    Some((order, row))
}

fn notice_at(n: &GroupNotice) -> EpochSecs {
    match n {
        GroupNotice::Pressure { at, .. } | GroupNotice::Refused { at, .. } => *at,
    }
}

/// 群の逼迫の知らせと移動の断りの列（渡した event log の順に、log の中は行の順に読み、at の新しい順・
/// 同じ at は群の宣言の順・それも同じなら読んだ順）。宣言に無い群の行・欄の欠けた行・ほかの種類の行・
/// JSON でない行は読まず、全部の欄が同じ行は 1 度だけ。群の宣言の字が無いか log の字が無ければ「まだ分からない」。
pub fn notices(texts: &HostTexts, logs: &[&str]) -> Reading<Vec<GroupNotice>> {
    let Some(host) = texts.host_toml.as_deref() else {
        return Reading::Unknown;
    };
    if logs.is_empty() {
        return Reading::Unknown;
    }
    let names: Vec<String> = declaration(host)
        .groups
        .into_iter()
        .map(|g| g.name)
        .collect();
    let mut rows: Vec<(usize, GroupNotice)> = Vec::new();
    for line in logs.iter().flat_map(|log| log.lines()) {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(row) = notice(&names, &event) else {
            continue;
        };
        if !rows.iter().any(|(_, r)| *r == row.1) {
            rows.push(row);
        }
    }
    rows.sort_by(|(ga, a), (gb, b)| notice_at(b).cmp(&notice_at(a)).then(ga.cmp(gb)));
    Reading::Known(rows.into_iter().map(|(_, n)| n).collect())
}
