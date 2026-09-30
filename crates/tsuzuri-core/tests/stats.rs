//! 台帳の指標・未反映の一覧・次の一手・pipeline の板の歯（便 d・接頭辞 stats_）。
//! fixture: tests/fixtures/ledger の judge.json・stats-30.json、tests/fixtures/pipeline の pipeline.json・next.json。
//! fixture の ledger は bd の出力の形の配列（JSON にして渡す）、events は字か 1 行 1 件の行の配列（改行で繋ぐ）。
//! 閾値と次の一手の順は、規則の行 R-18・R-24 の value の字の全体と比べる（rules の file は読むだけ・行の字を探す）。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tsuzuri_contract::board::{LedgerJudge, NextMove, PipelineCard, Reading, Stage};
use tsuzuri_contract::stats::{CheckResult, NextStep, UnreflectedKind};
use tsuzuri_core::ledger::{
    JudgeInput, THRESHOLDS, Thresholds, judge, judge_of, stats, unreflected,
};
use tsuzuri_core::next_step::next_step;
use tsuzuri_core::pipeline::board;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture(rel: &str) -> Value {
    serde_json::from_str(&read(rel)).unwrap_or_else(|e| panic!("{rel} は JSON: {e}"))
}

/// fixture の欄を入力の字にする。
fn input_text(v: &Value, key: &str) -> String {
    match &v[key] {
        Value::String(s) => s.clone(),
        Value::Array(items) if key == "events" => items
            .iter()
            .map(|item| match item {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Null => panic!("fixture に欄 {key} が無い"),
        other => other.to_string(),
    }
}

fn now_of(v: &Value) -> u64 {
    v["now"].as_u64().expect("fixture の now")
}

/// 値を比べる（数は 1e-9 まで・ほかは字のまま）。違いの在りかを返す。
fn diff(got: &Value, want: &Value, at: &str, out: &mut Vec<String>) {
    match (got, want) {
        (Value::Number(g), Value::Number(w)) => {
            let (g, w) = (g.as_f64().unwrap(), w.as_f64().unwrap());
            if (g - w).abs() > 1e-9 {
                out.push(format!("{at}: {g} ≠ {w}"));
            }
        }
        (Value::Object(g), Value::Object(w)) => {
            let keys: BTreeSet<&String> = g.keys().chain(w.keys()).collect();
            for k in keys {
                match (g.get(k), w.get(k)) {
                    (Some(gv), Some(wv)) => diff(gv, wv, &format!("{at}.{k}"), out),
                    _ => out.push(format!("{at}.{k}: 欄が片方にだけ在る")),
                }
            }
        }
        (Value::Array(g), Value::Array(w)) if g.len() == w.len() => {
            for (i, (gv, wv)) in g.iter().zip(w).enumerate() {
                diff(gv, wv, &format!("{at}[{i}]"), out);
            }
        }
        _ if got == want => {}
        _ => out.push(format!("{at}: {got} ≠ {want}")),
    }
}

/// rules の file の行の value の字（`{id: <id>, ` で始まる行の `value: "` から次の `"` まで）。
fn rule_value(id: &str) -> String {
    let rules = read("design-intent/rules.yaml");
    let head = format!("{{id: {id}, ");
    let line = rules
        .lines()
        .map(str::trim_start)
        .find(|l| l.trim_start_matches("- ").starts_with(&head))
        .unwrap_or_else(|| panic!("rules の file に行 {id} が無い"));
    let rest = line
        .split_once("value: \"")
        .unwrap_or_else(|| panic!("行 {id} に value が無い"))
        .1;
    rest.split_once('"').expect("value の終わり").0.to_string()
}

#[test]
fn stats_judge_matches_fixture() {
    let v = fixture("tests/fixtures/ledger/judge.json");
    let cases = v["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 7, "見本の自己検査と同じ 7 組");
    let mut seen = BTreeSet::new();
    for c in cases {
        let input: Option<JudgeInput> =
            serde_json::from_value(c["input"].clone()).expect("判定の材料");
        let want: LedgerJudge = serde_json::from_value(c["want"].clone()).expect("判定の語");
        assert_eq!(judge(input.as_ref()), want, "{}", c["case"]);
        seen.insert(format!("{want:?}"));
    }
    assert_eq!(
        seen.len(),
        LedgerJudge::ALL.len(),
        "5 値のどれもが 1 組で出る"
    );
}

#[test]
fn stats_30_matches_fixture() {
    let v = fixture("tests/fixtures/ledger/stats-30.json");
    assert_eq!(v["ledger"].as_array().expect("ledger").len(), 30);
    let Reading::Known(got) = stats(&input_text(&v, "ledger"), now_of(&v)) else {
        panic!("台帳が読めない");
    };
    let got_json = serde_json::to_value(&got).expect("電文");
    let mut out = Vec::new();
    diff(&got_json, &v["expected"], "stats", &mut out);
    assert!(out.is_empty(), "期待と違う: {out:#?}");
    assert_eq!(got.days.len(), 14);
    assert_eq!(judge_of(&Reading::Known(got)), LedgerJudge::PilingUp);
}

#[test]
fn stats_wire_has_no_in_progress_or_oldest() {
    let v = fixture("tests/fixtures/ledger/stats-30.json");
    let Reading::Known(got) = stats(&input_text(&v, "ledger"), now_of(&v)) else {
        panic!("台帳が読めない");
    };
    let json = serde_json::to_value(&got).expect("電文");
    let keys: BTreeSet<&str> = json
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    let want: BTreeSet<&str> = [
        "at",
        "judge",
        "open",
        "blocked",
        "ready",
        "stale",
        "net_drop_24h",
        "net_drop_7d",
        "closed_7d",
        "closed_per_day",
        "lead",
        "days",
        "epics",
        "memo",
        "unreflected",
        "unreflected_kinds",
        "unreflected_unknown",
    ]
    .into();
    assert_eq!(keys, want, "電文の欄");
    let all = json.to_string();
    for bad in ["in_progress", "working", "oldest"] {
        assert!(!all.contains(bad), "電文に {bad} の欄が在る: {all}");
    }
}

#[test]
fn stats_unreadable_ledger_is_no_ledger() {
    for text in ["", "  \n", "{", "{\"id\":\"x\"}"] {
        let got = stats(text, 0);
        assert_eq!(got, Reading::Unknown, "{text:?}");
        assert_eq!(judge_of(&got), LedgerJudge::NoLedger);
        let u = unreflected(text, 0);
        assert_eq!(u.unknown(), UnreflectedKind::ALL.to_vec(), "{text:?}");
    }
    // 空の台帳は読めた 0 件（台帳なしでない）。
    let empty = stats("[]", 0);
    assert!(matches!(empty, Reading::Known(_)));
    assert_ne!(judge_of(&empty), LedgerJudge::NoLedger);
}

#[test]
fn stats_rule_r18_text_matches_rules() {
    let want = rule_value("R-18");
    assert_eq!(THRESHOLDS.rule_text(), want, "閾値の定数と行 R-18 の字");
    // 定数のどれか 1 つを変えれば落ちる。
    let changed = [
        Thresholds {
            stale_ratio: 0.6,
            ..THRESHOLDS
        },
        Thresholds {
            gain_7d: 1,
            ..THRESHOLDS
        },
        Thresholds {
            idle_per_day: 0.2,
            ..THRESHOLDS
        },
        Thresholds {
            stale_days: 14,
            ..THRESHOLDS
        },
    ];
    for t in changed {
        assert_ne!(t.rule_text(), want, "{t:?} でも行 R-18 の字と一致する");
    }
}

#[test]
fn stats_rule_r24_order_matches_rules() {
    let value = rule_value("R-24");
    let want = value
        .split_once(" の順に判じ")
        .expect("行 R-24 の「 の順に判じ」")
        .0;
    let order = |moves: &[NextMove]| {
        moves
            .iter()
            .map(|m| m.label())
            .collect::<Vec<_>>()
            .join(" → ")
    };
    assert_eq!(order(&NextMove::ALL), want, "7 種の宣言の順と行 R-24 の字");
    // 順を入れ替えれば落ちる。
    let mut swapped = NextMove::ALL;
    swapped.swap(2, 4);
    assert_ne!(order(&swapped), want);
    // 次の一手の結果も宣言の順に並ぶ。
    let step = next_step("[]", "", 0);
    let kinds: Vec<NextMove> = step.checks.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, NextMove::ALL.to_vec());
}

#[test]
fn stats_next_step_matches_fixture() {
    let v = fixture("tests/fixtures/pipeline/next.json");
    let cases = v["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 4);
    let mut leads = BTreeSet::new();
    for c in cases {
        let got = next_step(
            &input_text(c, "ledger"),
            &input_text(c, "events"),
            now_of(&v),
        );
        let want: NextStep = serde_json::from_value(c["want"].clone()).expect("期待の電文");
        assert_eq!(got, want, "{}", c["case"]);
        leads.insert(got.lead);
    }
    assert_eq!(
        leads,
        [NextMove::StalledRun, NextMove::Question, NextMove::Nothing].into(),
        "判じられる 3 種のどれもが 1 組で大きく出る"
    );
    // 台帳が読めなければ質問も止まっている走行も束の承認も判じない。なしも判じない（行 c-next-stall・
    // 要件 NFR2）。大きく出す 1 つはなし（測れていないの意味）。
    let events = input_text(&cases[0], "events");
    let step = next_step("", &events, now_of(&v));
    for c in &step.checks {
        assert_eq!(c.result, CheckResult::NotJudged, "{c:?}");
    }
    assert_eq!(step.lead, NextMove::Nothing);
}

#[test]
fn stats_pipeline_matches_fixture() {
    let v = fixture("tests/fixtures/pipeline/pipeline.json");
    let got = board(
        &input_text(&v, "ledger"),
        &input_text(&v, "events"),
        now_of(&v),
    );
    let want: Vec<PipelineCard> = serde_json::from_value(v["cards"].clone()).expect("期待の札");
    assert_eq!(got.board.cards, Reading::Known(want.clone()));
    assert_eq!(u64::from(got.unmapped), v["unmapped"].as_u64().unwrap());
    // Stopped のほかの段は、どれも 1 枚以上の札で出る。
    let stages: BTreeSet<String> = want.iter().map(|c| format!("{:?}", c.stage)).collect();
    let all: BTreeSet<String> = Stage::ALL
        .iter()
        .filter(|s| **s != Stage::Stopped)
        .map(|s| format!("{s:?}"))
        .collect();
    assert_eq!(stages, all);
    // event log が読めなければ札は「まだ分からない」。
    for events in ["", "not json", "{\"kind\":\"RunCreated\"}\n["] {
        let b = board(&input_text(&v, "ledger"), events, now_of(&v));
        assert_eq!(b.board.cards, Reading::Unknown, "{events:?}");
    }
    // 台帳が読めなくても走行の札は組む（走行の無い契約の札だけが無い）。
    let b = board("", &input_text(&v, "events"), now_of(&v));
    let Reading::Known(cards) = b.board.cards else {
        panic!("札が読めない");
    };
    assert_eq!(cards.len(), want.iter().filter(|c| c.runs > 0).count());
}

#[test]
fn stats_core_deps_are_the_three() {
    let manifest = read("crates/tsuzuri-core/Cargo.toml");
    let deps: Vec<&str> = manifest
        .split("[dependencies]")
        .nth(1)
        .expect("[dependencies] の節")
        .lines()
        .take_while(|l| !l.starts_with('['))
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split(['=', '.']).next().map(str::trim))
        .collect();
    assert_eq!(deps, ["serde", "serde_json", "tsuzuri-contract"]);
    // 契約の型の crate も serde と serde_json だけ。
    let contract = read("crates/tsuzuri-contract/Cargo.toml");
    let deps: Vec<&str> = contract
        .split("[dependencies]")
        .nth(1)
        .expect("[dependencies] の節")
        .lines()
        .take_while(|l| !l.starts_with('['))
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split(['=', '.']).next().map(str::trim))
        .collect();
    assert_eq!(deps, ["serde", "serde_json"]);
}
