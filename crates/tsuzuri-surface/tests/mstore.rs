//! 行 g-mode-store の歯: mode を browser の保存に残す（見本の store と initMode・持ち主の裁定 t3-hub.52.16・要件 FR1）。
//! 決め方は host の純粋な関数 mode_start で撃ち、保存と URL を撃つ所の配線は source の字で見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::store::{MODE_KEY, ModeStart, mode_start};
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = crate_dir().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} が読めない: {e}", path.display()))
}

/// 字 start から、その後の最初の字 end までの本文（無ければ空）。
fn body<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    text.find(start).map_or("", |i| {
        let rest = &text[i..];
        rest.find(end).map_or(rest, |j| &rest[..j])
    })
}

/// Eq を求める型の値を受ける（derive の Eq を組み立てで見る）。
fn same<T: Eq>(a: &T, b: &T) -> bool {
    a == b
}

/// 表の 1 組（query・保存の値 → mode・keep・url）。
type Row = (&'static str, Option<&'static str>, Mode, Option<&'static str>, Option<&'static str>);

/// 節の表の 12 組（query・保存の値 → mode・keep・url）。
#[test]
fn mstore_start_rules() {
    let table: [Row; 12] = [
        ("?mode=expert", None, Mode::Expert, Some("expert"), None),
        ("?mode=beginner", Some("expert"), Mode::Beginner, Some("beginner"), None),
        ("?board=account&mode=expert", Some("beginner"), Mode::Expert, Some("expert"), None),
        ("", Some("expert"), Mode::Expert, None, Some("?mode=expert")),
        ("?board=account", Some("expert"), Mode::Expert, None, Some("?board=account&mode=expert")),
        (
            "?page=node&id=g-seat",
            Some("expert"),
            Mode::Expert,
            None,
            Some("?page=node&id=g-seat&mode=expert"),
        ),
        ("?mode=bogus", Some("expert"), Mode::Expert, None, Some("?mode=expert")),
        ("?page=ask&mode=Expert", Some("expert"), Mode::Expert, None, Some("?page=ask&mode=expert")),
        ("?mode=", Some("expert"), Mode::Expert, None, Some("?mode=expert")),
        ("?page=map", None, Mode::Beginner, None, None),
        ("", Some("beginner"), Mode::Beginner, None, None),
        ("?mode=bogus", Some("bogus"), Mode::Beginner, None, None),
    ];
    for (query, saved, mode, keep, url) in table {
        let got = mode_start(query, saved);
        let want = ModeStart {
            mode,
            keep,
            url: url.map(str::to_string),
        };
        assert_eq!(got, want, "query {query:?} と保存 {saved:?}");
        let copy = got.clone();
        assert!(same(&copy, &want), "clone が同じでない: {query:?}");
    }
}

/// 8 つの query と 4 つの保存の値の全部の組で、URL が先・URL と mode が食い違わない。
#[test]
fn mstore_url_wins() {
    let queries = [
        "",
        "?mode=expert",
        "?mode=beginner",
        "?mode=bogus",
        "?mode=",
        "?board=account",
        "?page=ask&mode=Expert",
        "?board=account&tab=session&mode=expert",
    ];
    let saved = [None, Some("beginner"), Some("expert"), Some("bogus")];
    for query in queries {
        let in_url = tsuzuri_surface::frame::param(query, "mode");
        let url_mode = Mode::ALL.into_iter().find(|m| in_url == Some(m.key()));
        for s in saved {
            let got = mode_start(query, s);
            let seen = got.url.as_deref().unwrap_or(query);
            assert_eq!(Mode::from_query(seen), got.mode, "{query:?} と {s:?}");
            if let Some(m) = url_mode {
                assert_eq!(got.mode, m, "URL の mode が先でない: {query:?} と {s:?}");
            }
            assert_eq!(
                got.url.is_some(),
                got.mode == Mode::Expert && url_mode.is_none(),
                "url の在る組: {query:?} と {s:?}"
            );
            assert_eq!(got.keep.is_some(), url_mode.is_some(), "keep の在る組: {query:?} と {s:?}");
        }
    }
}

/// 鍵は見本の tz-mode で、見本の ui.js と語の辞書の注釈に同じ字が在る。
#[test]
fn mstore_key_is_mock_key() {
    assert_eq!(MODE_KEY, "tz-mode");
    let js = read("../../docs/design/mock3/ui.js");
    assert!(js.contains("store('tz-mode'"), "見本の ui.js に store('tz-mode' が無い");
    let term = vocab().term("mode").unwrap_or_else(|| panic!("鍵 mode が vocab に無い"));
    assert!(
        term.internal.contains("localStorage tz-mode"),
        "語の辞書の mode の internal: {:?}",
        term.internal
    );
}

const WASM: &str = "#[cfg(target_arch = \"wasm32\")]";

/// lib.rs は store を target に限らず置き、store.rs の保存の口は wasm の target のときだけ。
#[test]
fn mstore_module_text() {
    let lib = read("src/lib.rs");
    let lines: Vec<&str> = lib.lines().map(str::trim).collect();
    let at = lines
        .iter()
        .position(|l| *l == "pub mod store;")
        .unwrap_or_else(|| panic!("lib.rs に pub mod store; が無い"));
    assert!(at == 0 || lines[at - 1] != WASM, "pub mod store; が wasm の target のときだけ");
    for m in ["pub mod board;", "pub mod net;"] {
        assert!(
            lines.windows(2).any(|w| w[0] == WASM && w[1] == m),
            "lib.rs の {m} が wasm の target のときだけでない"
        );
    }
    let src = read("src/store.rs");
    let lines: Vec<&str> = src.lines().map(str::trim).collect();
    for head in ["pub fn get(", "pub fn set(", "pub fn settle_mode(", "pub fn keep_mode("] {
        assert!(
            lines.windows(2).any(|w| w[0] == WASM && w[1].starts_with(head)),
            "store.rs の {head} が wasm の target のときだけの行の次に無い"
        );
    }
    for word in [
        "local_storage()",
        "get_item(",
        "set_item(",
        "replace_state_with_url(",
        "mode_start(",
        "MODE_KEY",
    ] {
        assert!(src.contains(word), "store.rs に {word} が無い");
    }
    for word in ["unwrap(", "expect(", "panic!"] {
        assert!(!src.contains(word), "store.rs に {word} が在る");
    }
}

/// 2 つの board は mount で App の前に settle_mode を撃ち、mode の押しで keep_mode を撃つ。
#[test]
fn mstore_board_calls() {
    for rel in ["src/board.rs", "src/account/board.rs"] {
        let text = read(rel);
        let mount = body(&text, "pub fn mount()", "\n}");
        let settle = mount.find("store::settle_mode()");
        let body_at = mount.find("mount_to_body");
        assert!(
            settle.is_some() && body_at.is_some() && settle < body_at,
            "{rel} の mount で settle_mode が mount_to_body の前に無い"
        );
        let keep = body(&text, "fn keep_mode_in_url(", "\n}");
        let pick = body(&text, "let pick = move |_| {", "};");
        assert!(
            keep.contains("store::keep_mode(") || pick.contains("store::keep_mode("),
            "{rel} の mode の押しで keep_mode を撃たない"
        );
    }
}

/// manifest の依存の節の名（target ごとの節も・`[dependencies.x]` の節も）。
fn dependency_names(text: &str, only: Option<&str>) -> Vec<String> {
    let mut inside = false;
    let mut names = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(table) = line.strip_prefix('[') {
            let table = table.trim_end_matches(']');
            inside = match only {
                Some(t) => table == t,
                None => table.ends_with("dependencies"),
            };
            if only.is_none()
                && let Some((head, name)) = table.rsplit_once('.')
                && head.ends_with("dependencies")
            {
                names.push(name.to_string());
            }
        } else if inside
            && !line.starts_with('#')
            && let Some((k, _)) = line.split_once('=')
        {
            let k = k.trim();
            names.push(k.split('.').next().unwrap_or(k).to_string());
        }
    }
    names.sort();
    names.dedup();
    names
}

/// web-sys の features に Storage と StorageEvent を足し、依存の名は増やさない。
#[test]
fn mstore_manifest_features() {
    let text = read("Cargo.toml");
    let list = text
        .find("web-sys = {")
        .map_or("", |i| body(&text[i..], "web-sys = {", "]"));
    for name in ["Storage", "StorageEvent", "Window", "History", "Location"] {
        assert!(list.contains(&format!("\"{name}\"")), "web-sys の features に {name} が無い");
    }
    assert_eq!(
        dependency_names(&text, None),
        vec![
            "leptos",
            "tsuzuri-boundary",
            "tsuzuri-contract",
            "wasm-bindgen-futures",
            "web-sys"
        ]
    );
    assert_eq!(dependency_names(&text, Some("dependencies")), vec!["tsuzuri-contract"]);
}

/// 着地済みの行の verify の filter の語（この行の接頭辞は並べない）。
const FILTERS: [&str; 92] = [
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
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
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
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
    "kcli_",
    "hcard_",
    "qgate_",
    "nsum_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
    "fmark_",
    "fstop_",
    "fserve_",
    "nsumw_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "athr_",
    "qblock_",
];

/// 自分の file の test の名はどれも接頭辞で始まり、残りの字はほかの行の filter の語を含まない。
#[test]
fn mstore_own_names_clean() {
    let text = read("tests/mstore.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .filter_map(|w| w[1].strip_prefix("fn "))
        .map(|rest| rest.split('(').next().unwrap_or(rest))
        .collect();
    assert_eq!(
        names.len(),
        lines.iter().filter(|l| **l == "#[test]").count(),
        "test の属性の次の行が fn でない"
    );
    assert!(names.len() >= 7, "test の名が少ない: {names:?}");
    for name in names {
        let rest = name
            .strip_prefix("mstore_")
            .unwrap_or_else(|| panic!("{name} が mstore_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
