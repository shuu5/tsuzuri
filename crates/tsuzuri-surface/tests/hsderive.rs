//! 行 hs-derived の歯（接頭辞 hsderive_・判断の記録 ADR-13）: block の module の口の path と畳める段の鍵の形は
//! module ごとの定数 PATHS と FOLDS・Module の paths と folds はその定数を返す・kit の fold_keys は Module の ALL の順に
//! folds をつないだ列・写しの snapshot は消した。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::project::{
    self, Module, ask, askpage, batch, gaps, ledger, legend, next, node, nodearound, notice,
    pipeline, policy, seat, stage, timeline,
};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// module の名・PATHS・FOLDS。
type Consts = (&'static str, &'static [&'static str], &'static [&'static str]);

/// 着地済みの module の定数（名の順）。
const CONSTS: &[Consts] = &[
    ("ask", ask::PATHS, ask::FOLDS),
    ("askpage", askpage::PATHS, askpage::FOLDS),
    ("batch", batch::PATHS, batch::FOLDS),
    ("gaps", gaps::PATHS, gaps::FOLDS),
    ("ledger", ledger::PATHS, ledger::FOLDS),
    ("legend", legend::PATHS, legend::FOLDS),
    ("next", next::PATHS, next::FOLDS),
    ("node", node::PATHS, node::FOLDS),
    ("nodearound", nodearound::PATHS, nodearound::FOLDS),
    ("notice", notice::PATHS, notice::FOLDS),
    ("pipeline", pipeline::PATHS, pipeline::FOLDS),
    ("policy", policy::PATHS, policy::FOLDS),
    ("seat", seat::PATHS, seat::FOLDS),
    ("stage", stage::PATHS, stage::FOLDS),
    ("timeline", timeline::PATHS, timeline::FOLDS),
];

/// (1) 着地済みの module の PATHS と FOLDS は着地の時の値（PATHS は定数の名で書いた path）。
#[test]
fn hsderive_consts_match_note() {
    let want: &[Consts] = &[
        (
            "ask",
            &["/api/questions", "/api/ruling", "/api/unreceived"],
            &["ask:around:{}"],
        ),
        ("askpage", &[], &["ask:hist"]),
        ("batch", &["/api/batch"], &[]),
        ("gaps", &[], &["gaps:{}"]),
        (
            "ledger",
            &["/api/ledger", "/api/metrics", "/api/unreflected"],
            &["ledger:unref"],
        ),
        ("legend", &[], &[]),
        ("next", &["/api/next"], &[]),
        ("node", &[], &[]),
        ("nodearound", &["/api/around"], &[]),
        ("notice", &["/api/notices"], &[]),
        ("pipeline", &["/api/pipeline"], &[]),
        ("policy", &["/api/policy"], &[]),
        ("seat", &["/api/seat"], &[]),
        ("stage", &[], &["stage:block"]),
        ("timeline", &[], &[]),
    ];
    assert_eq!(CONSTS, want);
    assert_eq!(
        ask::PATHS,
        [ask::PATH, ask::RULING_PATH, ask::UNRECEIVED_PATH]
    );
    assert_eq!(
        ledger::PATHS,
        [ledger::PATH, ledger::METRICS_PATH, ledger::UNREF_PATH]
    );
    for (paths, path) in [
        (batch::PATHS, batch::PATH),
        (next::PATHS, next::PATH),
        (nodearound::PATHS, nodearound::PATH),
        (notice::PATHS, notice::PATH),
        (pipeline::PATHS, pipeline::PATH),
        (policy::PATHS, policy::PATH),
        (seat::PATHS, seat::PATH),
    ] {
        assert_eq!(paths, [path]);
    }
    // PATHS は定数の名で書き、/api/ の字を新しく書かない（file の字の数は歯 parts_ が見る）。
    for &(name, _, _) in CONSTS {
        let text = read(&format!("src/project/{name}.rs"));
        let line = text
            .lines()
            .find(|l| l.starts_with("pub const PATHS: &[&str] = "))
            .unwrap_or_else(|| panic!("{name} に PATHS が無い"));
        assert!(!line.contains('"'), "{name} の PATHS が字を持つ: {line}");
        assert!(
            text.contains("\npub const FOLDS: &[&str] = "),
            "{name} に FOLDS が無い"
        );
    }
}

/// (2) Module の paths と folds は、名の同じ module の PATHS と FOLDS を返す。
#[test]
fn hsderive_module_returns_consts() {
    for m in Module::ALL {
        let (_, paths, folds) = CONSTS
            .iter()
            .copied()
            .find(|(n, _, _)| *n == m.name())
            .unwrap_or_else(|| panic!("{} の定数が表に無い", m.name()));
        assert_eq!(m.paths(), paths, "{} の paths", m.name());
        assert_eq!(m.folds(), folds, "{} の folds", m.name());
    }
    for &(name, _, _) in CONSTS {
        assert!(
            Module::ALL.iter().any(|m| m.name() == name),
            "{name} が Module の ALL に無い"
        );
    }
}

/// (3) kit.rs に FOLD_KEYS が無く、fold_keys は Module の ALL の順に各 module の folds をつないだ列で、
/// fold_key_ok はその形で判じる。
#[test]
fn hsderive_fold_forms_joined() {
    let kit = read("src/kit.rs");
    assert!(!kit.contains("FOLD_KEYS"), "kit.rs に FOLD_KEYS が在る");
    assert!(kit.contains("pub fn fold_keys() -> Vec<&'static str> {"));
    let want: Vec<&str> = Module::ALL
        .into_iter()
        .flat_map(|m| m.folds().iter().copied())
        .collect();
    assert_eq!(project::fold_keys(), want);
    for form in project::fold_keys() {
        let key = form.replace("{}", "x");
        assert!(project::fold_key_ok(&key), "{key}");
        if let Some(prefix) = form.strip_suffix("{}") {
            assert!(!project::fold_key_ok(prefix), "{prefix}");
        }
    }
    assert!(!project::fold_key_ok("ask:more"));
    assert!(!project::fold_key_ok(""));
}

/// (9) 写しの tests/snapshots/frame.json は無く、hspage.rs の字にその file の名が無い。
#[test]
fn hsderive_drops_old_copy() {
    assert!(
        !crate_dir().join("tests/snapshots/frame.json").exists(),
        "tests/snapshots/frame.json が在る"
    );
    assert!(!read("tests/hspage.rs").contains("frame.json"), "hspage.rs が frame.json を読む");
}

/// 着地済みの行とこの文書の行の verify の filter の語。
const FILTERS: [&str; 49] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hook_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "hbproc_",
    "hbconf_",
    "hbroute_",
    "hbpost_",
    "hsblock_",
    "hspage_",
    "hsderive_",
];

/// (7) この file の歯の名はどれも hsderive_ で始まり、残りの字は filter の語を含まない。
#[test]
fn hsderive_own_names_clean() {
    let text = read("tests/hsderive.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert!(names.len() >= 5, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("hsderive_")
            .unwrap_or_else(|| panic!("{name} が hsderive_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
