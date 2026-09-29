//! 一覧の面の更新の時刻の歯（行 c-updated・要件 FR5）。
//! 並べ替えは id・更新・状態の 3 つで、更新は新しい順・時刻の無い節点は後ろ・同じなら id の自然な順。
//! 行は更新の時刻を日本時間の月日と時分で持ち、無ければ ―。節の電文は歯の中で組む。

use std::path::PathBuf;

use tsuzuri_contract::graph::GraphDoc;
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::list::{Listing, NO_UPDATED, Query, Sort, listing};

/// 節の電文（節点 6 つ・fx-10 は null・FR2 と fx-9 は鍵なし・辺と台帳と走行の属性は空）。
const DOC: &str = r#"{
  "nodes": [
    {"id":"fx-10","kind":"task","file":null,"digest":null,"title":"十","updated":null},
    {"id":"fx-a","kind":"task","file":null,"digest":null,"title":"あ","updated":1790494756},
    {"id":"FR2","kind":"要件","file":"srs.yaml","digest":"00000000","title":"要件"},
    {"id":"fx-c","kind":"task","file":null,"digest":null,"title":"し","updated":1790494756},
    {"id":"fx-9","kind":"task","file":null,"digest":null,"title":"九"},
    {"id":"fx-b","kind":"task","file":null,"digest":null,"title":"び","updated":1790494800}
  ],
  "edges": [],
  "unread": [],
  "beads": {},
  "runs": {},
  "invariants": [],
  "skipped": {"design": 0, "ledger": 0, "design_nodes": 0}
}"#;

fn doc() -> GraphDoc {
    wire::decode(DOC).expect("節の電文が読める")
}

fn ids(l: &Listing) -> Vec<&str> {
    l.rows.iter().map(|r| r.id.as_str()).collect()
}

#[test]
fn cupdlist_sorts_by_updated() {
    let q = Query::from_search("?view=list&sort=updated");
    assert_eq!(q.sort, Sort::Updated);
    let l = listing(&doc(), &q);
    assert_eq!(ids(&l), ["fx-b", "fx-a", "fx-c", "FR2", "fx-9", "fx-10"]);
}

#[test]
fn cupdlist_sort_choices() {
    let names: Vec<&str> = Sort::ALL.iter().map(|s| s.name()).collect();
    assert_eq!(names, ["id", "updated", "state"]);
    let keys: Vec<&str> = Sort::ALL.iter().map(|s| s.key()).collect();
    assert_eq!(keys, ["col_id", "col_updated", "col_state"]);
}

#[test]
fn cupdlist_updated_word() {
    let l = listing(&doc(), &Query::from_search("?sort=updated"));
    let words: Vec<String> = l.rows.iter().map(|r| r.updated_word()).collect();
    assert_eq!(
        words,
        [
            "09-27 16:40 JST",
            "09-27 16:39 JST",
            "09-27 16:39 JST",
            "―",
            "―",
            "―"
        ]
    );
    assert_eq!(NO_UPDATED, "―");
}

#[test]
fn cupdlist_wiring() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/mapview/list.rs");
    let src = std::fs::read_to_string(&path).expect("list.rs を読む");
    let wired = r#"<span class="meta">{band_chip(r.band)}<span>{meta}</span><span class="num">{r.updated_word()}</span></span>"#;
    assert_eq!(src.matches(wired).count(), 1, "行の meta の字は 1 度");
    assert_eq!(
        src.matches("{r.updated_word()}").count(),
        1,
        "更新の時刻の語を出す字は 1 か所"
    );
}
