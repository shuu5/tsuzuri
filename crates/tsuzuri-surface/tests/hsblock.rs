//! 行 hs-blocks の歯: block の列を組み立ての script が src/project の dir から生成し（列挙 Module）、
//! 共通の部品を kit へ移し、board の描き分けを生成した列の 1 つの関数にし、台帳の block は画面の状態を context で受ける。
//! block_view と ledger の view の本文は wasm の target のときだけなので字を読んで見る（DOM は surface-build で組めることで見る）。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::frame::Block;
use tsuzuri_surface::project::{
    self, Module, ask, askpage, batch, gaps, ledger, legend, next, node, nodearound, pipeline,
    policy, seat,
};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 生成した字（組み立ての出力の dir の project_blocks.rs）。
fn generated() -> String {
    let path = PathBuf::from(env!("OUT_DIR")).join("project_blocks.rs");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

/// src/project の下の mod.rs でない .rs の file の名（拡張子を除く・名の順）。
fn dir_names() -> Vec<String> {
    let dir = crate_dir().join("src/project");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .expect("src/project")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n != "mod.rs")
        .filter_map(|n| n.strip_suffix(".rs").map(str::to_string))
        .collect();
    names.sort();
    names
}

/// 着地済みの 12 の module と、その BLOCK（地図の block は行 m-map-page で消した）。
const LANDED: [(&str, Block); 12] = [
    ("ask", ask::BLOCK),
    ("askpage", askpage::BLOCK),
    ("batch", batch::BLOCK),
    ("gaps", gaps::BLOCK),
    ("ledger", ledger::BLOCK),
    ("legend", legend::BLOCK),
    ("next", next::BLOCK),
    ("node", node::BLOCK),
    ("nodearound", nodearound::BLOCK),
    ("pipeline", pipeline::BLOCK),
    ("policy", policy::BLOCK),
    ("seat", seat::BLOCK),
];

/// 名の生成した module（無ければ落とす）。
fn module(name: &str) -> Module {
    Module::ALL
        .into_iter()
        .find(|m| m.name() == name)
        .unwrap_or_else(|| panic!("Module の ALL に {name} が無い"))
}

/// fn の本文（`fn <名>(` から、行の頭の `}` まで）。
fn fn_body<'a>(text: &'a str, name: &str) -> &'a str {
    let start = text
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("fn {name} が無い"));
    let rest = &text[start..];
    let end = rest.find("\n}").unwrap_or_else(|| panic!("fn {name} の終わりが無い"));
    &rest[..end]
}

/// `word` の後に `::` が続き、前が識別子の字でない所が在るか。
fn has_path(text: &str, word: &str) -> bool {
    let pat = format!("{word}::");
    text.match_indices(&pat).any(|(i, _)| {
        !text[..i]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

/// (1) Module の ALL の名の列は dir の file の名の列と同じで、着地済みの 12 が全部在る（多い分は許す）。
#[test]
fn hsblock_names_follow_dir() {
    let names: Vec<String> = Module::ALL.iter().map(|m| m.name().to_string()).collect();
    assert_eq!(names, dir_names(), "Module の ALL と src/project の file の名");
    for (name, _) in LANDED {
        assert!(names.iter().any(|n| n == name), "{name} が Module の ALL に無い");
    }
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "Module の ALL は名の順");
}

/// (2) Module の block の値は各 module の BLOCK と同じ。
#[test]
fn hsblock_ids_match_consts() {
    for (name, block) in LANDED {
        assert_eq!(module(name).block(), block, "{name} の BLOCK");
    }
    assert_eq!(Module::Askpage.name(), "askpage");
    assert_eq!(Module::Nodearound.block(), nodearound::BLOCK);
}

/// (3) project の mod.rs は kit の再公開と生成した file の include だけ・kit.rs が共通の部品を持つ。
#[test]
fn hsblock_kit_holds_shared() {
    let modrs = read("src/project/mod.rs");
    let code: Vec<&str> = modrs
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect();
    assert!(
        !code.iter().any(|l| l.starts_with("pub mod ") || l.starts_with("mod ")),
        "mod.rs が mod の行を持つ"
    );
    assert!(!code.iter().any(|l| l.contains("fn ")), "mod.rs が fn を持つ");
    assert!(code.contains(&"pub use crate::kit::*;"), "mod.rs に kit の再公開が無い");
    assert!(
        code.contains(&r#"include!(concat!(env!("OUT_DIR"), "/project_blocks.rs"));"#),
        "mod.rs に生成した file の include が無い"
    );
    kit_defs_and_dom();
}

/// kit.rs が共通の部品の定義を持ち、DOM の部品は wasm の target の mod dom の中で pub use で出す。
fn kit_defs_and_dom() {
    let kit = read("src/kit.rs");
    for def in [
        "pub enum Body<",
        "pub const STATES:",
        "pub const UNKNOWN:",
        "pub fn state_key(",
        "pub fn state_class(",
        "pub struct Item ",
        "pub const ALERT_STYLE:",
        "pub fn item(",
        "pub const LEDGER_UNREAD:",
        "pub const NOT_READ:",
        "pub const NO_CONTENT:",
        "pub fn pending(",
        "pub fn fold_keys(",
        "pub fn fold_key_ok(",
        "pub struct Folds(",
    ] {
        assert!(kit.contains(def), "kit.rs に {def} が無い");
    }
    let dom = kit
        .find("#[cfg(target_arch = \"wasm32\")]\nmod dom {")
        .expect("kit.rs に wasm の target のときだけの mod dom が無い");
    for def in ["section", "state_icon", "unmeasured", "body_view", "item_view", "fold"] {
        let at = kit
            .find(&format!("    pub fn {def}("))
            .unwrap_or_else(|| panic!("kit.rs に {def} が無い"));
        assert!(at > dom, "{def} が mod dom の外");
    }
    assert!(
        kit.contains(
            "#[cfg(target_arch = \"wasm32\")]\npub use dom::{body_view, fold, item_view, section, state_icon, unmeasured};"
        ),
        "kit.rs が dom の部品を pub use で出さない"
    );
    host_parts_via_project();
}

/// host の部品の値と型を crate::project の path で引く。
fn host_parts_via_project() {
    // 今までの crate::project の path で host の部品が使える。
    assert_eq!(project::state_key("run"), "st_run");
    assert_eq!(project::state_class("wait"), "st st-wait");
    assert!(project::fold_key_ok("gaps:x"));
    assert_eq!(project::STATES.len(), 5);
    assert_eq!(project::UNKNOWN, "unknown");
    assert_eq!(
        project::fold_keys().len(),
        Module::ALL.iter().map(|m| m.folds().len()).sum::<usize>()
    );
    assert!(project::Folds::default().recorded("ledger:more").is_none());
    assert!(!project::ALERT_STYLE.is_empty());
    assert!(!project::LEDGER_UNREAD.is_empty());
    assert!(matches!(
        project::pending(&tsuzuri_surface::view::Fetched::NotRead, project::NO_CONTENT),
        project::Body::Unmeasured(project::NOT_READ)
    ));
    let _: fn(&tsuzuri_contract::ledger::LedgerRow) -> project::Item = project::item;
}

/// (4) board の block_view は生成した列から選び、App は画面の状態を context に置き、台帳の view は引数を持たない。
#[test]
fn hsblock_dispatch_by_list() {
    let board = read("src/board.rs");
    let body = fn_body(&board, "block_view");
    for m in Module::ALL {
        assert!(
            !has_path(body, m.name()),
            "block_view が module {} の path を持つ",
            m.name()
        );
    }
    assert!(body.contains("Module::ALL"), "block_view が Module の ALL から選ばない");
    assert!(
        body.contains(".block().id == block.id"),
        "block_view が枠の id で選ばない"
    );
    assert!(body.contains("Module::view"), "block_view が module の view を呼ばない");
    // App は画面の状態（Screen）を作らない（読み手が無くなり行 g-dead-sweep-b で消した）。
    let app = fn_body(&board, "App");
    assert!(!app.contains("Screen"), "App が画面の状態を作る");
    assert!(
        board.contains("fn page_view(page: PageId) ->"),
        "page_view が画面の状態を受ける"
    );
    let led = read("src/project/ledger.rs");
    assert!(
        led.contains("#[cfg(target_arch = \"wasm32\")]\npub fn view() -> leptos::prelude::AnyView {"),
        "台帳の view が引数を持つ"
    );
    let view = fn_body(&led, "view");
    // 台帳の block は画面の状態を読まなくなった（指標の段と件数の chip は行 g-ledger-trim で外した）。
    assert!(view.contains("dom::view()"), "台帳の view が dom の view を呼ばない");
    assert!(!view.contains("use_context::<RwSignal<Screen>>()"), "台帳の view が画面の状態を受ける");
    // lib.rs の wasm の target のときだけの board の 2 行は崩さない。
    assert!(read("src/lib.rs").contains("#[cfg(target_arch = \"wasm32\")]\npub mod board;"));
}

/// (6) 組み立ての script の rerun の 2 行・生成した字の path の属性と pub mod の宣言が名の順。
#[test]
fn hsblock_script_reruns_and_emits() {
    let script = read("build.rs");
    for line in [
        "\"cargo:rerun-if-changed=src/project\"",
        "\"cargo:rerun-if-changed=build.rs\"",
    ] {
        assert!(script.contains(line), "build.rs に {line} が無い");
    }
    for word in [".unwrap()", ".expect(", "panic!"] {
        assert!(!script.contains(word), "build.rs が {word} を使う");
    }
    assert!(script.contains("fn main() -> Result<"), "build.rs の main が Result を返さない");
    let text = generated();
    assert!(!text.contains("//!") && !text.contains("#!["), "生成した字が内側の doc か属性を持つ");
    let mut from = 0;
    for name in dir_names() {
        let path = crate_dir().join("src").join("project").join(format!("{name}.rs"));
        let decl = format!(
            "#[path = {:?}]\npub mod {name};\n",
            path.to_str().expect("UTF-8 の path")
        );
        let at = text[from..]
            .find(&decl)
            .unwrap_or_else(|| panic!("生成した字に {name} の宣言が名の順に無い"));
        from += at + decl.len();
    }
    assert!(text.contains("pub enum Module {"));
    assert!(text.contains("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum Module"));
    assert!(text.contains("    #[cfg(target_arch = \"wasm32\")]\n    pub fn view(self) -> leptos::prelude::AnyView {"));
}

/// (7) 規則の行 R-2 の測り: before_s と after_s の秒・差が 0.3 を越えれば reason の行。
#[test]
fn hsblock_measure_r2_lines() {
    let text = read("../../docs/measure/r2-surface-build.txt");
    let value = |key: &str| -> Option<&str> {
        text.lines().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == key).then(|| v.trim())
        })
    };
    let secs = |key: &str| -> f64 {
        value(key)
            .unwrap_or_else(|| panic!("{key} の行が無い"))
            .parse()
            .unwrap_or_else(|e| panic!("{key} が秒の数でない: {e}"))
    };
    let (before, after) = (secs("before_s"), secs("after_s"));
    assert!(before > 0.0 && after > 0.0);
    if after - before > 0.3 {
        assert!(
            value("reason").is_some_and(|r| !r.is_empty()),
            "差が 0.3 秒を越えるのに reason の行が無い"
        );
    }
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

/// (9) この file の歯の名はどれも hsblock_ で始まり、残りの字は filter の語を含まない。
#[test]
fn hsblock_own_names_clean() {
    let text = read("tests/hsblock.rs");
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
    assert!(names.len() >= 7, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("hsblock_")
            .unwrap_or_else(|| panic!("{name} が hsblock_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
