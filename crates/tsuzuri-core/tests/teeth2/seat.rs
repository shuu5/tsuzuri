//! 席の card と、席の card から判じる次の一手の歯（便 e-seat・接頭辞 seatcard_）。
//! fixture: tests/fixtures/seat/seat-inputs.json（組の名 → 7 つの字と期待の card）と、
//! tests/fixtures/pipeline/next.json（着地済みの次の一手の組）。
//! 組の席の名と今の時刻は期待の card の target と at から取る。anchor は server と同じく doctor の席の行から取る。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use tsuzuri_contract::board::{GroupRow, NextMove, QuotaLeft, Reading};
use tsuzuri_contract::seat::{AccountMove, QuotaUsed, SeatCard, SeatSpan, SeatState, TickHealth};
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};
use tsuzuri_core::next_step::{next_step, next_step_seat};
use tsuzuri_core::seat::{SeatTexts, anchor, card, group_name};

const FIXTURE: &str = "tests/fixtures/seat/seat-inputs.json";

/// 6 組の名。
const CASES: [&str; 6] = ["run", "wait", "limit", "silent", "no-state", "unread"];

#[derive(Deserialize)]
struct Case {
    #[serde(flatten)]
    texts: SeatTexts,
    card: SeatCard,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn cases() -> BTreeMap<String, Case> {
    serde_json::from_str(&read(FIXTURE)).expect("fixture の形")
}

fn case(name: &str) -> Case {
    cases()
        .remove(name)
        .unwrap_or_else(|| panic!("組 {name} が無い"))
}

/// server と同じ組み方（anchor は doctor の席の行から）。
fn build(target: &str, texts: &SeatTexts, now: u64) -> SeatCard {
    let anchor = texts.doctor.as_deref().and_then(|d| anchor(d, target));
    card(target, anchor.as_deref(), texts, now)
}

/// 組の 7 つの字から組んだ card。
fn built(c: &Case) -> SeatCard {
    build(&c.card.target, &c.texts, c.card.at)
}

#[test]
fn seatcard_fixture_six_cases() {
    let all = cases();
    let names: Vec<&str> = all.keys().map(String::as_str).collect();
    let mut want = CASES.to_vec();
    want.sort_unstable();
    assert_eq!(names, want);
    let mut states = Vec::new();
    for (name, c) in &all {
        let got = built(c);
        assert_eq!(got, c.card, "組 {name}");
        states.push(got.state);
    }
    // 状態は 5 つ全部が出る（動いている・待っている・限度・応答なし・測れていない）。
    for s in SeatState::ALL {
        assert!(states.contains(&s), "{s:?} の組が無い");
    }
    // 合図の健康と停止の切り替えは yes/on が真・no/off が偽。
    let run = &all["run"].card;
    assert_eq!(
        (&run.tick_healthy, &run.heartbeat),
        (&Reading::Known(true), &Reading::Known(true))
    );
    let wait = &all["wait"].card;
    assert_eq!(
        (&wait.tick_healthy, &wait.heartbeat),
        (&Reading::Known(false), &Reading::Known(false))
    );
    // tick の語は doctor の席の行の tick から（4 つの語のほかと行か語が無ければ Unknown）・
    // tick_at は合図の最後の判定の空でない最後の行の ts。
    let ticks: Vec<(&str, Reading<TickHealth>, Option<u64>)> = all
        .iter()
        .map(|(n, c)| (n.as_str(), c.card.tick.clone(), c.card.tick_at))
        .collect();
    assert_eq!(
        ticks,
        [
            ("limit", Reading::Unknown, Some(1_790_503_200)),
            ("no-state", Reading::Unknown, Some(1_790_510_390)),
            (
                "run",
                Reading::Known(TickHealth::Healthy),
                Some(1_790_510_390)
            ),
            ("silent", Reading::Unknown, Some(1_790_509_000)),
            ("unread", Reading::Unknown, Some(1_790_510_390)),
            (
                "wait",
                Reading::Known(TickHealth::Stale),
                Some(1_790_510_000)
            ),
        ]
    );
}

#[test]
fn seatcard_missing_tick_line_only_two_fields() {
    let c = case("run");
    let mut texts = c.texts.clone();
    texts.tick_status = Some(
        "seat tick status: target=proj-2:0.1 last=- age=- healthy=yes heartbeat=on step=5 next=0\n"
            .into(),
    );
    let got = build(&c.card.target, &texts, c.card.at);
    let mut want = c.card.clone();
    want.tick_healthy = Reading::Unknown;
    want.heartbeat = Reading::Unknown;
    assert_eq!(got, want);
    // 字が無いときも同じ 2 つだけ。
    texts.tick_status = None;
    assert_eq!(build(&c.card.target, &texts, c.card.at), want);
    // 真偽の語でない値も「まだ分からない」。
    texts.tick_status = Some(
        "seat tick status: target=proj-1:0.1 last=- age=- healthy=maybe heartbeat=on step=5 next=0\n"
            .into(),
    );
    let got = build(&c.card.target, &texts, c.card.at);
    assert_eq!(
        (got.tick_healthy, got.heartbeat),
        (Reading::Unknown, Reading::Known(true))
    );
}

/// run の組の残量の字だけを替えて組む。
fn with_usage(usage: Option<&str>, model: &str) -> SeatCard {
    let c = case("run");
    let mut texts = c.texts.clone();
    texts.usage = usage.map(str::to_string);
    texts.doctor = texts.doctor.map(|d| {
        d.replace(
            "account=acct-1 model=opus",
            &format!("account=acct-1 model={model}"),
        )
    });
    build(&c.card.target, &texts, c.card.at)
}

fn used(window: &str, pct: u8, resets: Option<u64>, counted: bool) -> QuotaUsed {
    QuotaUsed {
        window: window.into(),
        used_pct: pct,
        resets_at: resets,
        counted,
    }
}

#[test]
fn seatcard_usage_windows() {
    let line = "usage: account=acct-1 five_hour=100% resets=2026-09-27T12:00:00Z seven_day=300% resets=none model=claude-Opus-5:120% resets=2026-09-28T00:00:00Z\n";
    let got = with_usage(Some(line), "claude-opus-5");
    assert_eq!(
        got.usage,
        Reading::Known(vec![
            used("five_hour", 100, Some(1_790_510_400), true),
            used("seven_day", 255, None, true),
            used("seven_day_model", 120, Some(1_790_553_600), true),
        ])
    );
    // model の名が違えば model の窓だけ数えない。
    let got = with_usage(Some(line), "sonnet");
    let Reading::Known(windows) = &got.usage else {
        panic!("読める行が Unknown");
    };
    let counted: Vec<bool> = windows.iter().map(|w| w.counted).collect();
    assert_eq!(counted, [true, true, false]);
    // 83 は 83。
    let got = with_usage(
        Some(
            "usage: account=acct-1 five_hour=83% resets=none seven_day=0% resets=none model=opus:1% resets=none\n",
        ),
        "opus",
    );
    assert_eq!(
        got.usage,
        Reading::Known(vec![
            used("five_hour", 83, None, true),
            used("seven_day", 0, None, true),
            used("seven_day_model", 1, None, true),
        ])
    );
    // 測れていない行・行が無い・字が無い・読めない時刻は「まだ分からない」。
    for usage in [
        Some("usage: account=acct-1 unmeasured reason=no-token\n"),
        Some(
            "usage: account=acct-2 five_hour=1% resets=none seven_day=1% resets=none model=opus:1% resets=none\n",
        ),
        Some(
            "usage: account=acct-1 five_hour=1% resets=tomorrow seven_day=1% resets=none model=opus:1% resets=none\n",
        ),
        Some("usage: account=acct-1 five_hour=x% resets=none\n"),
        None,
    ] {
        assert_eq!(
            with_usage(usage, "opus").usage,
            Reading::Unknown,
            "{usage:?}"
        );
    }
}

fn left(window: &str, pct: u8) -> QuotaLeft {
    QuotaLeft {
        window: window.into(),
        left_pct: pct,
    }
}

#[test]
fn seatcard_group_from_declaration_and_doctor() {
    let c = case("run");
    // 宣言の順で最初に anchor を含む群（末尾の「/」は同じ path）。
    let host = c.texts.host_toml.clone().expect("群の宣言");
    assert_eq!(
        group_name(&host, "/srv/host-1/proj-1").as_deref(),
        Some("g-a")
    );
    assert_eq!(
        group_name(&host, "/srv/host-1/proj-2/").as_deref(),
        Some("g-b")
    );
    assert_eq!(group_name(&host, "/srv/host-1/proj-9"), None);
    let swapped = host
        .replace("\"g-a\"", "\"g-x\"")
        .replace("\"g-c\"", "\"g-a\"");
    assert_eq!(
        group_name(&swapped, "/srv/host-1/proj-1").as_deref(),
        Some("g-x")
    );
    assert_eq!(
        c.card.group,
        Reading::Known(GroupRow {
            group: "g-a".into(),
            account: "acct-1".into(),
            candidates: vec!["acct-1".into(), "acct-2".into()],
            next_account: Some("acct-2".into()),
            remaining: vec![
                left("five_hour", 17),
                left("seven_day", 0),
                left("seven_day_model", 0)
            ],
            park: false,
        })
    );
    assert_eq!(built(&c), c.card);
    // 次の移り先が none なら無し・今の口座の残量の行が無いか測れていなければ空の列。
    let mut texts = c.texts.clone();
    texts.doctor = texts.doctor.map(|d| d.replace("next=acct-2", "next=none"));
    texts.usage = Some("usage: account=acct-1 unmeasured reason=no-token\n".into());
    let Reading::Known(g) = build(&c.card.target, &texts, c.card.at).group else {
        panic!("群が Unknown");
    };
    assert_eq!((g.next_account, g.remaining), (None, vec![]));
    texts.usage = None;
    let Reading::Known(g) = build(&c.card.target, &texts, c.card.at).group else {
        panic!("群が Unknown");
    };
    assert!(g.remaining.is_empty());
    group_unknown(&c);
}

/// 群の宣言や doctor の字が群を引けなければ、群は「まだ分からない」。
fn group_unknown(c: &Case) {
    // 群の宣言の字が無い・1 行に収まらない配列の群しか含まない・doctor に群の行が無い・doctor が無い。
    let multi = "[[account-group]]\nname = \"g-a\"\nanchors = [\"/srv/host-1/proj-1\",\n  \"/srv/host-1/proj-0\"]\n";
    assert_eq!(group_name(multi, "/srv/host-1/proj-1"), None);
    for (what, edit) in [
        (
            "宣言が無い",
            Box::new(|t: &mut SeatTexts| t.host_toml = None) as Box<dyn Fn(&mut SeatTexts)>,
        ),
        (
            "1 行に収まらない",
            Box::new(move |t: &mut SeatTexts| t.host_toml = Some(multi.into())),
        ),
        (
            "群の行が無い",
            Box::new(|t: &mut SeatTexts| {
                t.doctor = t
                    .doctor
                    .as_ref()
                    .map(|d| d.replace("group=g-a", "group=g-z"));
            }),
        ),
    ] {
        let mut texts = c.texts.clone();
        edit(&mut texts);
        let got = build(&c.card.target, &texts, c.card.at);
        assert_eq!(got.group, Reading::Unknown, "{what}");
    }
}

#[test]
fn seatcard_spans_last_24_hours() {
    let c = case("run");
    let now = c.card.at;
    let day = 86_400;
    let line = |state: &str, ts: u64| {
        format!(
            "{{\"schema\":1,\"state\":\"{state}\",\"event\":\"e\",\"ts\":{ts},\"sid\":\"s\"}}\n"
        )
    };
    let mut texts = c.texts.clone();
    // 24 時間前ちょうどの行は入り、その 1 秒前の行は入らない。ts の順に並べ直す。
    texts.state_log = Some(
        [
            line("busy", now - day - 1),
            line("idle", now - 100),
            line("busy", now - day),
            line("busy", now - 500),
            line("idle", now - 50),
        ]
        .concat(),
    );
    let got = build(&c.card.target, &texts, now);
    assert_eq!(
        got.spans,
        Reading::Known(vec![
            SeatSpan {
                from: now - day,
                to: now - 100,
                state: SeatState::Run
            },
            SeatSpan {
                from: now - 100,
                to: now,
                state: SeatState::Wait
            },
        ])
    );
    // 状態は file の最後の行（idle）で、から の時刻は file の末尾から続く idle の行（1 行）の ts。
    assert_eq!((got.state, got.since), (SeatState::Wait, Some(now - 50)));
    // 24 時間の中に行が無ければ空の列・読める行が無ければ状態は unknown・字が無ければ「まだ分からない」。
    texts.state_log = Some(line("busy", now - day - 1));
    assert_eq!(
        build(&c.card.target, &texts, now).spans,
        Reading::Known(vec![])
    );
    texts.state_log = Some("not json\n".into());
    let got = build(&c.card.target, &texts, now);
    assert_eq!(
        (got.spans, got.state, got.since),
        (Reading::Known(vec![]), SeatState::Unknown, None)
    );
    texts.state_log = None;
    assert_eq!(build(&c.card.target, &texts, now).spans, Reading::Unknown);
}

#[test]
fn seatcard_moves_oldest_first() {
    let c = case("run");
    let mut texts = c.texts.clone();
    texts.records = vec![
        "account=acct-2\nts=2026-09-27T11:00:00Z\nreason=account-pressed\nprevious=acct-1\n".into(),
        "account=acct-1\nts=2026-09-27T10:00:00Z\nreason=initial\nprevious=\n".into(),
        "reason=broken\n".into(),
        "account=acct-3\nts=1790500000\nreason=manual\n".into(),
    ];
    let got = build(&c.card.target, &texts, c.card.at);
    let mv = |at: u64, from: Option<&str>, to: &str| AccountMove {
        at,
        from: from.map(str::to_string),
        to: to.into(),
    };
    assert_eq!(
        got.moves,
        Reading::Known(vec![
            mv(1_790_500_000, None, "acct-3"),
            mv(1_790_503_200, None, "acct-1"),
            mv(1_790_506_800, Some("acct-1"), "acct-2"),
        ])
    );
    // 群の名が得られなければ「まだ分からない」。
    texts.host_toml = None;
    assert_eq!(
        build(&c.card.target, &texts, c.card.at).moves,
        Reading::Unknown
    );
}

#[test]
fn seatcard_unread_text_touches_only_its_fields() {
    let c = case("run");
    let full = built(&c);
    unread_each(&c, &full);
}

/// 字を 1 つずつ無くした card は、その字の欄だけが変わる。
fn unread_each(c: &Case, full: &SeatCard) {
    type Edit = fn(&mut SeatTexts);
    let edits: [(&str, Edit); 6] = [
        ("tick_status", |t| t.tick_status = None),
        ("doctor", |t| t.doctor = None),
        ("usage", |t| t.usage = None),
        ("state_log", |t| t.state_log = None),
        ("tick_last", |t| t.tick_last = None),
        ("host_toml", |t| t.host_toml = None),
    ];
    for (what, edit) in edits {
        let mut texts = c.texts.clone();
        edit(&mut texts);
        let got = build(&c.card.target, &texts, c.card.at);
        let mut want = full.clone();
        match what {
            "tick_status" => {
                want.tick_healthy = Reading::Unknown;
                want.heartbeat = Reading::Unknown;
            }
            "doctor" => {
                want.account = None;
                want.model = None;
                want.tick = Reading::Unknown;
                want.group = Reading::Unknown;
                want.usage = Reading::Unknown;
                want.moves = Reading::Unknown;
                want.refused = Reading::Unknown;
            }
            "usage" => {
                want.usage = Reading::Unknown;
                if let Reading::Known(g) = &mut want.group {
                    g.remaining.clear();
                }
            }
            "state_log" => {
                want.state = SeatState::Unknown;
                want.since = None;
                want.spans = Reading::Unknown;
            }
            // 判定が限度でも応答なしでもない組なので、合図の最後の判定が無くても状態は変わらず、
            // 判定の時刻が無くなり、時点は状態の記録の最後の行（1790505000）になって窓もそこから 1 日前になる。
            "tick_last" => {
                let span = |from, to, state| SeatSpan { from, to, state };
                want.tick_at = None;
                want.at = 1_790_505_000;
                want.spans = Reading::Known(vec![
                    span(1_790_420_000, 1_790_430_000, SeatState::Wait),
                    span(1_790_430_000, 1_790_440_000, SeatState::Run),
                    span(1_790_440_000, 1_790_500_000, SeatState::Wait),
                    span(1_790_500_000, 1_790_505_000, SeatState::Run),
                ]);
            }
            "host_toml" => {
                want.group = Reading::Unknown;
                want.moves = Reading::Unknown;
                want.refused = Reading::Unknown;
            }
            other => panic!("知らない欄の名: {other}"),
        }
        assert_eq!(got, want, "{what} が無い");
    }
}

/// 着地済みの次の一手の組（台帳の字・event log の字・今の時刻）。
fn next_inputs() -> Vec<(String, String, u64)> {
    let v: Value = serde_json::from_str(&read("tests/fixtures/pipeline/next.json")).expect("JSON");
    let text = |c: &Value, key: &str| match &c[key] {
        Value::String(s) => s.clone(),
        Value::Array(items) if key == "events" => items
            .iter()
            .map(|i| i.as_str().map_or_else(|| i.to_string(), str::to_string))
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    };
    let now = v["now"].as_u64().expect("now");
    v["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .map(|c| (text(c, "ledger"), text(c, "events"), now))
        .collect()
}

fn check_of(step: &NextStep, kind: NextMove) -> &NextCheck {
    step.checks
        .iter()
        .find(|c| c.kind == kind)
        .unwrap_or_else(|| panic!("{kind:?} の結果が無い"))
}

#[test]
fn seatcard_next_step_from_card() {
    let limit = case("limit").card;
    let silent = case("silent").card;
    let run = case("run").card;
    let wait = case("wait").card; // 登録の口座 acct-2 が群の今の口座 acct-1 と違う。
    let unknown = case("no-state").card;
    let (ledger, events) = ("[]", "");
    for (name, card, limit_hit, silent_hit) in [
        ("limit", &limit, CheckResult::Hit, CheckResult::Miss),
        ("moved", &wait, CheckResult::Hit, CheckResult::Miss),
        ("silent", &silent, CheckResult::Miss, CheckResult::Hit),
        ("run", &run, CheckResult::Miss, CheckResult::Miss),
    ] {
        let step = next_step_seat(ledger, events, 0, Some(card));
        let l = check_of(&step, NextMove::LimitOrMove);
        let u = check_of(&step, NextMove::Unresponsive);
        assert_eq!((l.result, u.result), (limit_hit, silent_hit), "{name}");
        if l.result == CheckResult::Hit {
            assert_eq!((l.count, &l.target), (1, &None), "{name}");
        }
        let kinds: Vec<NextMove> = step.checks.iter().map(|c| c.kind).collect();
        assert_eq!(kinds, NextMove::ALL.to_vec(), "{name}");
        let want_lead = if limit_hit == CheckResult::Hit {
            NextMove::LimitOrMove
        } else if silent_hit == CheckResult::Hit {
            NextMove::Unresponsive
        } else {
            NextMove::Nothing
        };
        assert_eq!(step.lead, want_lead, "{name}");
        // 空の event log では止まっている走行を判じないので、2 種が当たらない card のなしは判じなかった
        // （行 c-next-stall）。どちらかが当たればなしは当たらない。
        let nothing = check_of(&step, NextMove::Nothing).result;
        let want_nothing = if want_lead == NextMove::Nothing {
            CheckResult::NotJudged
        } else {
            CheckResult::Miss
        };
        assert_eq!(nothing, want_nothing, "{name}");
    }
    // card が無い・状態が unknown なら 2 種とも判じない（着地済みの関数と同じ値）。
    for card in [None, Some(&unknown)] {
        let step = next_step_seat(ledger, events, 0, card);
        for kind in [NextMove::LimitOrMove, NextMove::Unresponsive] {
            assert_eq!(
                check_of(&step, kind).result,
                CheckResult::NotJudged,
                "{card:?}"
            );
        }
        assert_eq!(step, next_step(ledger, events, 0));
    }
}

#[test]
fn seatcard_next_step_keeps_landed_values() {
    let limit = case("limit").card;
    let run = case("run").card;
    for (ledger, events, now) in next_inputs() {
        let landed = next_step(&ledger, &events, now);
        assert_eq!(next_step_seat(&ledger, &events, now, None), landed);
        // 2 種が当たらない card では、なしのほかの種類の結果も大きく出す 1 つも着地済みのまま
        // （なしは card の有無で判じた・判じなかったが変わるので比べない・行 c-next-stall）。
        let missed = next_step_seat(&ledger, &events, now, Some(&run));
        assert_eq!(missed.lead, landed.lead);
        for (m, l) in missed.checks.iter().zip(&landed.checks) {
            if matches!(m.kind, NextMove::LimitOrMove | NextMove::Unresponsive) {
                assert_eq!(m.result, CheckResult::Miss);
            } else if m.kind != NextMove::Nothing {
                assert_eq!(m, l);
            }
        }
        // 限度と移動が当たれば大きく出す 1 つは限度と移動で、なしは当たらない。
        let hit = next_step_seat(&ledger, &events, now, Some(&limit));
        assert_eq!(hit.lead, NextMove::LimitOrMove);
        assert_eq!(check_of(&hit, NextMove::Nothing).result, CheckResult::Miss);
    }
}

#[test]
fn seatcard_fixture_names_and_size() {
    let text = read(FIXTURE);
    assert!(text.len() <= 20_000, "fixture は {} byte", text.len());
    let (mut accounts, mut hosts) = (Vec::new(), Vec::new());
    for (_, c) in cases() {
        let t = &c.texts;
        let texts = [
            &t.tick_status,
            &t.doctor,
            &t.usage,
            &t.state_log,
            &t.tick_last,
            &t.host_toml,
        ]
        .into_iter()
        .flatten()
        .chain(&t.records);
        for line in texts.flat_map(|x| x.lines()) {
            // 合図の健康の行の next は数。
            let tick = line.starts_with("seat tick status:");
            for token in line.split_whitespace().filter(|_| !tick) {
                let Some((key, value)) = token.split_once('=') else {
                    continue;
                };
                match key {
                    "account" | "accounts" | "seat-accounts" | "current" | "next" | "previous" => {
                        accounts.extend(
                            value
                                .split(',')
                                .filter(|v| !matches!(*v, "" | "none" | "-"))
                                .map(str::to_string),
                        );
                    }
                    "name" if line.starts_with("host:") => hosts.push(value.to_string()),
                    _ => {}
                }
            }
            // path の /srv/<host>/ と、host の塊の name。
            for seg in line.split("/srv/").skip(1) {
                hosts.push(seg.split('/').next().unwrap_or_default().to_string());
            }
            if let Some(v) = line.strip_prefix("name = \"host") {
                hosts.push(format!("host{}", v.trim_end_matches('"')));
            }
        }
        let card = &c.card;
        accounts.extend(card.account.clone());
        if let Reading::Known(g) = &card.group {
            accounts.push(g.account.clone());
            accounts.extend(g.candidates.clone());
            accounts.extend(g.next_account.clone());
        }
        if let Reading::Known(moves) = &card.moves {
            for m in moves {
                accounts.push(m.to.clone());
                accounts.extend(m.from.clone());
            }
        }
    }
    fixture_names(accounts, hosts);
}

/// 口座の名はどれも acct- で、host の名はどれも host- で始まる（どちらの列も空でない）。
fn fixture_names(accounts: Vec<String>, hosts: Vec<String>) {
    assert!(!accounts.is_empty() && !hosts.is_empty());
    for a in &accounts {
        assert!(a.starts_with("acct-"), "口座の名 {a}");
    }
    for h in &hosts {
        assert!(h.starts_with("host-"), "host の名 {h}");
    }
}
