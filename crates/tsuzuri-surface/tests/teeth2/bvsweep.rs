//! 行 g-list-sweep の歯（接頭辞 bvsweep_・判断の記録 ADR-27 決定 (8)・要件 FR13）: 前の台帳の一覧の組と件数の関数と、
//! 一覧の項に出していた板の段（型 Staged・段つきの項・pipeline の stages・項の段の記号）を src から消した。
//! 一覧の 1 項 Item は 5 つの欄（印・赤・id・題・右の字）だけで、kit の item_view は右の字の前に段の記号を出さない。
//! DOM は host で撃てないので、src の字で見る。
#![cfg(test)]

use std::path::Path;

use crate::common::{crate_dir, read};
use tsuzuri_contract::ledger::{BeadId, LedgerRow, QUESTION_LABEL};
use tsuzuri_surface::project::{Item, item};

/// src の下の .rs の file の相対の path と字（path の順）。
fn sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
        let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{dir:?} を読む: {e}"));
        for entry in entries {
            let path = entry.expect("dir の項").path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let rel = path
                    .strip_prefix(root)
                    .expect("src の下")
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = std::fs::read_to_string(&path).expect(".rs を読む");
                out.push((rel, text));
            }
        }
    }
    let root = crate_dir().join("src");
    let mut out = Vec::new();
    walk(&root, &root, &mut out);
    out.sort();
    out
}

/// (2) 前の一覧の組と件数の関数・段の型と段つきの項・pipeline の stages は src のどこにも無い。
#[test]
fn bvsweep_old_list_fns_gone() {
    let ledger = read("src/project/ledger.rs");
    for gone in [
        "pub struct Group",
        "pub fn count(",
        "pub fn body(",
        "pub fn staged_body(",
        "pub fn group_cards(",
        "fn cards_of(",
        "pub fn listed(",
        "Screen",
        "card_of",
        "LEDGER_UNREAD",
    ] {
        assert!(
            !ledger.contains(gone),
            "src/project/ledger.rs に {gone} が在る"
        );
    }
    assert_eq!(
        super_uses(&ledger),
        ["use super::{Body, NO_CONTENT, NOT_READ};"]
    );

    let kit = read("src/kit.rs");
    for gone in [
        "pub struct Staged",
        "pub fn staged_item(",
        "pub stage:",
        "stage: None",
        "stage_sym",
    ] {
        assert!(!kit.contains(gone), "src/kit.rs に {gone} が在る");
    }

    let pipeline = read("src/project/pipeline.rs");
    for gone in ["pub fn stages(", "`stages`", "c-ledger-stage"] {
        assert!(
            !pipeline.contains(gone),
            "src/project/pipeline.rs に {gone} が在る"
        );
    }

    for (rel, text) in sources() {
        for word in ["Staged", "staged_item", "staged_body", "group_cards"] {
            assert!(!text.contains(word), "src/{rel} に {word} が在る");
        }
    }
}

/// file の最上位（字下げ無し）の use の行のうち、module super（src/project の module）の名をじかに引く行。
/// 見える範囲の印（pub など）の有無を問わず、道の頭は super か crate::project で、その後が波括弧か名 1 つ
/// （super::seat::hm のような下の module の名は数えない）。
fn super_uses(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| {
            let bare = ["pub(crate) ", "pub(super) ", "pub "]
                .iter()
                .find_map(|v| l.strip_prefix(v))
                .unwrap_or(l);
            let Some(path) = bare.strip_prefix("use ") else {
                return false;
            };
            ["super::", "crate::project::"]
                .iter()
                .filter_map(|p| path.strip_prefix(p))
                .any(|tail| tail.starts_with('{') || !tail.contains("::"))
        })
        .collect()
}

/// 台帳の 1 行（題は「題」・時刻は 0・親は無し）。
fn row(id: &str, kind: &str, status: &str, labels: &[&str]) -> LedgerRow {
    LedgerRow {
        id: BeadId::new(id).expect("id"),
        kind: kind.to_string(),
        title: "題".to_string(),
        status: status.to_string(),
        updated_at: 0,
        parent: None,
        labels: labels.iter().map(|l| l.to_string()).collect(),
    }
}

/// (3) Item は 5 つの欄で、台帳の行の項の右の字は状態の印と語と種類のまま。item_view は右の字だけを描く。
/// item_view の行は印・題の link・右の字の 3 つをこの順で並べ、右の字の span は行の末に 1 つだけ（差し込みは 4 つ）。
#[test]
fn bvsweep_item_has_five_fields() {
    let want = |shape: &str, alert: bool, id: &str, aside: &str| Item {
        shape: shape.to_string(),
        alert,
        id: id.to_string(),
        title: "題".to_string(),
        aside: aside.to_string(),
    };
    for (r, w) in [
        (
            row("fx-w.1", "task", "open", &[]),
            want("shape band-beads", false, "fx-w.1", "○ 未着手 · task"),
        ),
        (
            row("fx-w.2", "task", "open", &[QUESTION_LABEL]),
            want("shape band-beads", true, "fx-w.2", "◷ 答え待ち · question"),
        ),
        (
            row("fx-w.3", "task", "closed", &[]),
            want("shape band-beads fill", false, "fx-w.3", "● 閉じた · task"),
        ),
        (
            row("fx-w", "epic", "in_progress", &[]),
            want("shape band-beads", false, "fx-w", "◐ 作業中 · epic"),
        ),
    ] {
        assert_eq!(item(&r), w, "{}", r.id);
    }

    let kit = read("src/kit.rs");
    let at = kit.find("pub struct Item {").expect("Item の宣言");
    let decl = &kit[at..at + kit[at..].find("\n}\n").expect("Item の宣言の終わり")];
    let fields: Vec<&str> = decl
        .lines()
        .filter_map(|l| l.strip_prefix("    pub "))
        .collect();
    assert_eq!(
        fields,
        [
            "shape: String,",
            "alert: bool,",
            "id: String,",
            "title: String,",
            "aside: String,"
        ]
    );

    let dom = &kit[kit.find("mod dom {").expect("mod dom の字")..];
    assert_eq!(
        dom.matches(r#"<span class="aside">{item.aside.clone()}</span>"#)
            .count(),
        1
    );
    for gone in ["let sym", "{sym}"] {
        assert!(!dom.contains(gone), "kit.rs の mod dom に {gone} が在る");
    }
    item_row_order(dom);
}

/// (3) item_view の行（li）は印 lead・題の link・右の字の span をこの順で並べ、右の字の span は li の閉じの直前に在り、
/// li の中の差し込み（波括弧）は印と id と題と右の字の 4 つだけ。
fn item_row_order(dom: &str) {
    let at = dom.find("pub fn item_view(").expect("item_view");
    let view = &dom[at..at + dom[at..].find("\n    }\n").expect("item_view の閉じ")];
    let li = &view[view.find("<li>").expect("li")..view.find("</li>").expect("li の閉じ")];
    let aside = r#"<span class="aside">{item.aside.clone()}</span>"#;
    assert!(
        li.contains("{lead}\n                <a class=\"ttl\""),
        "{li}"
    );
    assert!(li.trim_end().ends_with(aside), "{li}");
    assert_eq!(li.matches(aside).count(), 1, "{li}");
    assert_eq!(li.matches('{').count(), 4, "{li}");
}
