//! 個別の頁の本文と記録の開き閉じをその browser に残す歯（行 g-fold-keep・接頭辞 fkeep_・判断の記録 ADR-30 決定 (4)・要件 FR1）。
//! 保存の鍵と保存の字の決め方を純な関数で断言し、保存の口（wasm の枝）と block の DOM の配線の字を照らす。
#![cfg(test)]

use crate::common::{read, span};
use tsuzuri_surface::project::nodebody::FOLDS;
use tsuzuri_surface::store::{
    FOLD_KEYS, FOLD_OPEN, FOLD_SHUT, fold_start, fold_store_key, fold_text,
};

const WASM: &str = "#[cfg(target_arch = \"wasm32\")]";

#[test]
fn fkeep_keys_by_part() {
    assert_eq!(
        FOLD_KEYS,
        [
            ("node:body", "tz-node-body"),
            ("node:notes", "tz-node-notes")
        ]
    );
    let parts: Vec<&str> = FOLD_KEYS.iter().map(|(f, _)| *f).collect();
    assert_eq!(parts, FOLDS, "保存に残す段は個別の頁の本文と記録の 2 つ");
    for (fold, key) in FOLD_KEYS {
        assert!(key.starts_with("tz-"), "{key} の頭が tz- でない");
        assert_eq!(fold_store_key(fold), Some(key), "{fold}");
    }
    for other in [
        "ledger:more",
        "seat:hist",
        "ask:around:t-1.2",
        "node:body:t-1.2",
        "",
    ] {
        assert_eq!(fold_store_key(other), None, "{other} は保存に書かない");
    }
}

#[test]
fn fkeep_saved_start() {
    assert_eq!((FOLD_OPEN, FOLD_SHUT), ("open", "shut"));
    assert!(fold_start(Some("open")));
    assert!(!fold_start(None), "保存が無ければ閉じて始める");
    assert!(!fold_start(Some("shut")));
    assert!(!fold_start(Some("")), "読めない字は閉じて始める");
    assert!(!fold_start(Some("Open")), "知らない字は閉じて始める");
    assert_eq!(fold_text(true), "open");
    assert_eq!(fold_text(false), "shut");
    for open in [true, false] {
        assert_eq!(
            fold_start(Some(fold_text(open))),
            open,
            "書いた字の読み戻し"
        );
    }
}

#[test]
fn fkeep_store_wasm_only() {
    let src = read("src/store.rs");
    let lines: Vec<&str> = src.lines().map(str::trim).collect();
    for head in [
        "pub fn fold_open(fold: &str) -> bool {",
        "pub fn keep_fold(fold: &str, open: bool) {",
    ] {
        assert!(
            lines.windows(2).any(|w| w[0] == WASM && w[1] == head),
            "store.rs の {head} が wasm の target のときだけの行の次に無い"
        );
    }
    let open = span(&src, "pub fn fold_open(", "\n}\n");
    assert!(
        open.contains("fold_store_key(fold).is_some_and(|k| fold_start(get(k).as_deref()))"),
        "読みは保存の鍵の値を fold_start で決める"
    );
    let keep = span(&src, "pub fn keep_fold(", "\n}\n");
    for want in [
        "if let Some(k) = fold_store_key(fold) {",
        "set(k, fold_text(open));",
    ] {
        assert!(keep.contains(want), "keep_fold に {want} が無い");
    }
}

#[test]
fn fkeep_node_dom_wiring() {
    let src = read("src/project/nodebody.rs");
    let dom = &src[src.find("mod dom {").expect("wasm の枝")..];
    for (key, part) in [("node:body", "text"), ("node:notes", "notes")] {
        let call = format!("fold(\"{key}\".to_string(), || store::fold_open(\"{key}\"));");
        assert_eq!(dom.matches(&call).count(), 1, "{call}");
        let wrap = format!("let toggle_{part} = kept(\"{key}\", toggle_{part});");
        assert_eq!(dom.matches(&wrap).count(), 1, "{wrap}");
    }
    let kept = span(dom, "fn kept(", "\n    }\n");
    assert!(
        kept.contains("store::keep_fold(key, now);"),
        "toggle で今の開き閉じを保存へ写す"
    );
    assert!(kept.contains("toggle(ev);"), "畳める段の記録へも書き戻す");
    assert!(
        kept.contains("let now = event_target::<web_sys::Element>(&ev).has_attribute(\"open\");"),
        "今の開き閉じは details の open の属性"
    );
}
