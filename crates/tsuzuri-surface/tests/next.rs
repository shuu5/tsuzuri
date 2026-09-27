//! 便 g-next の歯: fixture の 4 組の大きく出す 1 つ・残りの一覧の順と字・なしの箱・測れていない・
//! 7 種と語の鍵の 1 か所の表・電文の lead を写すだけ・着地済みの外形と依存。

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::NextMove;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::next::{
    self, Big, KEYS, Link, MISS, Mark, NONE_LINE, Next, PIPE_LINK, content, key, step,
};
use tsuzuri_surface::project::{Body, NO_CONTENT, NOT_READ, pipeline};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の 4 組（組の名 → 電文の字）。
fn fixture() -> BTreeMap<String, String> {
    let sets: BTreeMap<String, NextStep> =
        wire::decode(&read("../../tests/fixtures/surface/next-step.json"))
            .expect("fixture の組が電文として読める");
    sets.into_iter()
        .map(|(name, s)| (name, wire::encode(&s).expect("電文")))
        .collect()
}

fn filled(name: &str) -> Next {
    let text = fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"));
    match content(&Fetched::Body(text)) {
        Body::Filled(n) => n,
        other => panic!("組 {name} が中身を出さない: {other:?}"),
    }
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
        let big = filled(name).big;
        assert_eq!(big.key, want_key, "{name} の語の鍵");
        assert_eq!(big.class, want_class, "{name} の箱の class");
        assert_eq!(big.what, want_what, "{name} の中身の字");
        assert_eq!(big.link, want_link, "{name} の link");
    }
    // 止まっている走行の link は block「pipeline」の id へ頁の中で飛ぶ。
    assert_eq!(format!("#{}", pipeline::BLOCK.id), "#pipe");
}

fn rest(name: &str) -> Vec<(&'static str, &'static str, Mark)> {
    filled(name)
        .rest
        .iter()
        .map(|r| (r.key, r.class, r.mark))
        .collect()
}

/// (2) 残りの一覧は 7 種の順から大きく出した 1 つを除いた並びで、当たった種類は件数・当たらない種類は「―」・
/// 判じなかった種類は測れていないの記号。
#[test]
fn nextstep_rest_in_declared_order_without_lead() {
    use Mark::{Count, Miss, Unmeasured};
    assert_eq!(
        rest("stalled"),
        vec![
            ("nx_a", "off", Unmeasured),
            ("nx_b", "off", Unmeasured),
            ("nx_d", "off", Unmeasured),
            ("nx_e", "on", Count(3)),
            ("nx_f", "off", Unmeasured),
            ("nx_g", "off", Miss),
        ]
    );
    assert_eq!(
        rest("question"),
        vec![
            ("nx_a", "off", Unmeasured),
            ("nx_b", "off", Unmeasured),
            ("nx_c", "off", Miss),
            ("nx_d", "off", Unmeasured),
            ("nx_f", "off", Unmeasured),
            ("nx_g", "off", Miss),
        ]
    );
    assert_eq!(
        rest("not-judged"),
        vec![
            ("nx_a", "off", Unmeasured),
            ("nx_b", "off", Miss),
            ("nx_c", "off", Unmeasured),
            ("nx_e", "on", Count(5)),
            ("nx_f", "off", Unmeasured),
            ("nx_g", "off", Miss),
        ]
    );
    // 当たらない字は「―」で、0 とも測れていないとも違う。
    assert_eq!(MISS, "―");
    assert_ne!(Mark::Count(0), Mark::Miss);
    assert_ne!(Mark::Miss, Mark::Unmeasured);
    // どの組も一覧は 6 行で、大きく出した 1 つを含まない。
    for name in fixture().into_keys() {
        let n = filled(&name);
        assert_eq!(n.rest.len(), 6, "{name}");
        assert!(n.rest.iter().all(|r| r.kind != n.big.kind), "{name}");
    }
}

/// (3) どれも当たらない組は、なし（nx_g）を none の箱で大きく出し、一覧は 6 種が全部「―」。
#[test]
fn nextstep_nothing_set_leads_nx_g() {
    let n = filled("nothing");
    assert_eq!(n.big.kind, NextMove::Nothing);
    assert_eq!(n.big.key, "nx_g");
    assert_eq!(n.big.class, "nxbig none");
    assert_eq!(n.big.what_class, "what small muted");
    let keys: Vec<&str> = n.rest.iter().map(|r| r.key).collect();
    assert_eq!(keys, vec!["nx_a", "nx_b", "nx_c", "nx_d", "nx_e", "nx_f"]);
    assert!(
        n.rest
            .iter()
            .all(|r| r.mark == Mark::Miss && r.class == "off")
    );
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
        assert_eq!(content(&fetched), Body::Unmeasured(want), "{fetched:?}");
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

/// 面は判じない: 大きく出すのは電文の lead（結果が当たったかを見直さない）・一覧の字は電文の結果のまま・
/// 電文に無い種類は判じなかったと同じ。
#[test]
fn nextstep_copies_lead_without_judging() {
    let s = NextStep {
        checks: vec![
            check(NextMove::StalledRun, CheckResult::Hit, 7, Some("nx.1")),
            check(NextMove::Question, CheckResult::Miss, 0, None),
        ],
        lead: NextMove::Question,
    };
    let n = next::next(&s);
    assert_eq!(n.big.kind, NextMove::Question);
    assert_eq!(n.big.what, "0 件");
    let marks: Vec<(NextMove, Mark)> = n.rest.iter().map(|r| (r.kind, r.mark)).collect();
    assert_eq!(
        marks,
        vec![
            (NextMove::LimitOrMove, Mark::Unmeasured),
            (NextMove::Unresponsive, Mark::Unmeasured),
            (NextMove::StalledRun, Mark::Count(7)),
            (NextMove::BatchApproval, Mark::Unmeasured),
            (NextMove::AwaitingEffect, Mark::Unmeasured),
            (NextMove::Nothing, Mark::Unmeasured),
        ]
    );
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

/// 語の鍵は語の辞書に、class は stylesheet に在る。
#[test]
fn nextstep_keys_in_vocab_and_classes_in_stylesheet() {
    for (_, k) in KEYS {
        assert!(vocab().term(k).is_some(), "鍵 {k} が vocab に無い");
    }
    for k in ["next", "st_unknown"] {
        assert!(vocab().term(k).is_some(), "鍵 {k} が vocab に無い");
    }
    let css = read("style.css");
    let has = |name: &str| {
        let dot = format!(".{name}");
        css.match_indices(&dot).any(|(i, _)| {
            css[i + dot.len()..]
                .chars()
                .next()
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        })
    };
    let mut used: Vec<String> = Vec::new();
    for name in fixture().into_keys() {
        let n = filled(&name);
        used.extend(n.big.class.split_whitespace().map(str::to_string));
        used.extend(n.big.what_class.split_whitespace().map(str::to_string));
        for r in &n.rest {
            used.push(r.class.to_string());
        }
    }
    used.extend(["nxlist", "act", "btn", "primary", "v", "small", "num"].map(str::to_string));
    for c in ["nxbig", "none", "on", "off", "what", "muted"] {
        assert!(
            used.iter().any(|u| u == c),
            "組の中身が class {c} を使わない"
        );
    }
    for c in used {
        assert!(has(&c), "stylesheet に class {c} が無い");
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
