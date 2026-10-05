//! 便 g-next の歯: fixture の 4 組の大きく出す 1 つ・なしの箱・測れていない・
//! 7 種と語の鍵の 1 か所の表・電文の lead を写すだけ・着地済みの外形と依存。
//! 残りの一覧（Row・Mark・content・next）と block の DOM の class の歯は行 g-dead-sweep-a で消した（大きく出す 1 つは big で組む）。
#![cfg(test)]

use std::collections::BTreeMap;

use crate::common::read;
use tsuzuri_contract::board::NextMove;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::next::{self, Big, KEYS, Link, NONE_LINE, PIPE_LINK, big, key, step};
use tsuzuri_surface::project::{Body, NO_CONTENT, NOT_READ, pipeline};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

/// fixture の 4 組（組の名 → 電文の字）。
fn fixture() -> BTreeMap<String, String> {
    let sets: BTreeMap<String, NextStep> =
        wire::decode(&read("../../tests/fixtures/surface/next-step.json"))
            .expect("fixture の組が電文として読める");
    sets.into_iter()
        .map(|(name, s)| (name, wire::encode(&s).expect("電文")))
        .collect()
}

/// 組の大きく出す 1 つ（電文の lead と、その種類の結果）。
fn filled(name: &str) -> Big {
    let text = fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"));
    let s = step(&Fetched::Body(text)).unwrap_or_else(|e| panic!("組 {name} が読めない: {e}"));
    big(s.lead, s.checks.iter().find(|c| c.kind == s.lead))
}

/// (1) fixture の 4 組で、大きく出す 1 つの語の鍵と中身の字（と箱の class・link）が期待と一致する。
#[test]
fn nextstep_big_matches_each_fixture_set() {
    let names: Vec<String> = fixture().into_keys().collect();
    assert_eq!(names, vec!["not-judged", "nothing", "question", "stalled"]);
    let pipe = Some(Link {
        href: "#pipe".to_string(),
        text: PIPE_LINK,
    });
    let cases: [(&str, &str, &str, &str, Option<Link>); 4] = [
        ("stalled", "nx_c", "nxbig", "2 件", pipe),
        ("question", "nx_e", "nxbig", "nq.4 · 1 件", None),
        ("nothing", "nx_g", "nxbig none", NONE_LINE, None),
        ("not-judged", "nx_d", "nxbig", "4 件", None),
    ];
    for (name, want_key, want_class, want_what, want_link) in cases {
        let big = filled(name);
        assert_eq!(big.key, want_key, "{name} の語の鍵");
        assert_eq!(big.class, want_class, "{name} の箱の class");
        assert_eq!(big.what, want_what, "{name} の中身の字");
        assert_eq!(big.link, want_link, "{name} の link");
    }
    // 止まっている走行の link は block「pipeline」の id へ頁の中で飛ぶ。
    assert_eq!(format!("#{}", pipeline::BLOCK.id), "#pipe");
}

/// (3) どれも当たらない組は、なし（nx_g）を none の箱で大きく出す。
#[test]
fn nextstep_nothing_set_leads_nx_g() {
    let b = filled("nothing");
    assert_eq!(b.kind, NextMove::Nothing);
    assert_eq!(b.key, "nx_g");
    assert_eq!(b.class, "nxbig none");
    assert_eq!(b.what_class, "what small muted");
}

/// (4) 口が読めない・まだ読んでいない・電文として読めない本文は、0 件でなく測れていないと理由の 1 行。
#[test]
fn nextstep_unreadable_is_unmeasured() {
    for (fetched, want) in [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, next::REASON),
        (Fetched::Body("{}".to_string()), NO_CONTENT),
        (Fetched::Body("not json".to_string()), NO_CONTENT),
    ] {
        assert_eq!(next::body(&fetched), Body::Unmeasured(want), "{fetched:?}");
        assert_eq!(step(&fetched), Err(want));
        assert!(!want.trim().is_empty() && !want.contains('\n'));
    }
    for text in fixture().into_values() {
        assert_eq!(next::body(&Fetched::Body(text)), Body::Filled(()));
    }
}

/// (5) 7 種と語の鍵の対応は 1 か所の表で、順は契約の型の 7 種の宣言の順（NextMove::ALL）。
#[test]
fn nextstep_keys_one_table_in_declared_order() {
    let kinds: Vec<NextMove> = KEYS.iter().map(|(k, _)| *k).collect();
    assert_eq!(kinds, NextMove::ALL.to_vec());
    let keys: Vec<&str> = NextMove::ALL.into_iter().map(key).collect();
    assert_eq!(
        keys,
        vec!["nx_a", "nx_b", "nx_c", "nx_d", "nx_e", "nx_f", "nx_g"]
    );
    let src = read("src/project/next.rs");
    for k in &keys {
        let literal = format!("\"{k}\"");
        assert_eq!(
            src.matches(&literal).count(),
            1,
            "{literal} が表の外にも在る"
        );
    }
}

fn check(kind: NextMove, result: CheckResult, count: u32, target: Option<&str>) -> NextCheck {
    NextCheck {
        kind,
        result,
        count,
        target: target.map(|t| BeadId::new(t).expect("id")),
    }
}

/// 面は判じない: 大きく出すのは電文の lead（結果が当たったかを見直さない）。
#[test]
fn nextstep_copies_lead_without_judging() {
    let s = NextStep {
        checks: vec![
            check(NextMove::StalledRun, CheckResult::Hit, 7, Some("nx.1")),
            check(NextMove::Question, CheckResult::Miss, 0, None),
        ],
        lead: NextMove::Question,
    };
    let lead = next::big(s.lead, s.checks.iter().find(|c| c.kind == s.lead));
    assert_eq!(lead.kind, NextMove::Question);
    assert_eq!(lead.what, "0 件");
    // 対象の id が在るほかの種類は id と件数（止まっている走行は件数と link だけ）。
    let b: Big = next::big(
        NextMove::LimitOrMove,
        Some(&check(
            NextMove::LimitOrMove,
            CheckResult::Hit,
            1,
            Some("nx.9"),
        )),
    );
    assert_eq!(
        (b.key, b.what.as_str(), b.link),
        ("nx_a", "nx.9 · 1 件", None)
    );
    let c = next::big(
        NextMove::StalledRun,
        Some(&check(
            NextMove::StalledRun,
            CheckResult::Hit,
            2,
            Some("nx.7"),
        )),
    );
    assert_eq!(c.what, "2 件");
}

/// 語の鍵は語の辞書に在る（block の DOM の class の歯は行 g-dead-sweep-a で消した）。
#[test]
fn nextstep_keys_in_vocab_and_classes_in_stylesheet() {
    for (_, k) in KEYS {
        assert!(vocab().term(k).is_some(), "鍵 {k} が vocab に無い");
    }
    for k in ["next", "st_unknown"] {
        assert!(vocab().term(k).is_some(), "鍵 {k} が vocab に無い");
    }
}

/// (6) 着地済みの外形（BLOCK・口の path・body の名と引数と返りの型）と、面の crate の直接依存が増えない。
#[test]
fn nextstep_landed_shape_and_no_new_deps() {
    assert_eq!(next::BLOCK.id, "next");
    assert_eq!(next::BLOCK.heading, "next");
    assert_eq!(next::BLOCK.class, "panel");
    assert_eq!(next::PATH, "/api/next");
    let body: fn(&Fetched) -> Body<()> = next::body;
    assert!(matches!(
        body(&Fetched::Body("{}".to_string())),
        Body::Unmeasured(_)
    ));

    let manifest = read("Cargo.toml");
    let mut names = Vec::new();
    let mut inside = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line.ends_with("dependencies]");
        } else if inside
            && !line.starts_with('#')
            && let Some((k, _)) = line.split_once('=')
            && !k.trim().starts_with('"')
        {
            names.push(k.trim().to_string());
        }
    }
    names.sort();
    assert_eq!(
        names,
        vec![
            "leptos",
            "tsuzuri-boundary",
            "tsuzuri-contract",
            "wasm-bindgen-futures",
            "web-sys"
        ]
    );
}
