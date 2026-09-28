//! 便 c-q-blocking の歯（中核）: 問いの card の止めている task の列（電文の鍵と既定・台帳の種類 blocks の依存から写す・
//! 契約の型の snapshot の字・歯の名）。

use std::path::PathBuf;

use serde_json::{Value, json};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::question::{QuestionCard, QuestionList};
use tsuzuri_contract::wire;
use tsuzuri_core::question::{BLOCKS_TYPE, list, open_questions};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn dep(from: &str, to: &str, kind: &str) -> Value {
    json!({"issue_id": from, "depends_on_id": to, "type": kind})
}

fn task(id: &str, status: &str, deps: Vec<Value>) -> Value {
    json!({
        "id": id,
        "title": format!("{id} の題"),
        "issue_type": "task",
        "status": status,
        "labels": [],
        "created_at": "2026-09-27T09:00:00Z",
        "dependencies": deps,
    })
}

/// 節の台帳 LQ（問い 2 本と task 6 本）。
fn lq() -> Value {
    json!([
        {
            "id": "q-1",
            "title": "q-1 の題",
            "issue_type": "task",
            "status": "open",
            "labels": ["intake:question"],
            "created_at": "2026-09-27T10:00:00Z",
            "dependencies": [],
        },
        {
            "id": "q-2",
            "title": "q-2 の題",
            "issue_type": "task",
            "status": "open",
            "labels": ["intake:question"],
            "created_at": "2026-09-27T11:00:00Z",
            "dependencies": [dep("q-2", "q-1", "blocks")],
        },
        task(
            "t-3",
            "open",
            vec![dep("t-3", "q-1", "blocks"), dep("t-3", "q-2", "parent-child")]
        ),
        task("t-4", "closed", vec![dep("t-4", "q-1", "blocks")]),
        task(
            "t-5",
            "open",
            vec![dep("t-5", "q-1", "blocks"), dep("t-5", "q-1", "blocks")]
        ),
        task("t-6", "open", vec![dep("t-6", "q-2", "related")]),
        task("t-7", "open", vec![dep("t-7", "q-9", "blocks")]),
        {
            "id": "t-8",
            "title": "t-8 の題",
            "issue_type": "task",
            "status": "open",
            "labels": [],
            "created_at": "2026-09-27T09:00:00Z",
        },
    ])
}

/// LQ から全部の bead の鍵 dependencies を除いた台帳。
fn lq_bare() -> Value {
    let mut v = lq();
    for bead in v.as_array_mut().expect("配列") {
        bead.as_object_mut().expect("object").remove("dependencies");
    }
    v
}

fn known(list: QuestionList) -> Vec<QuestionCard> {
    match list.cards {
        Reading::Known(cards) => cards,
        Reading::Unknown => panic!("card が Unknown"),
    }
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn qblock_wire_key_and_default() {
    let cards = known(list(&lq().to_string()));
    let q1 = cards.iter().find(|c| c.id.as_str() == "q-1").expect("q-1");
    let text = wire::encode(q1).expect("encode");
    let keys = [
        "id",
        "title",
        "posted_at",
        "plain",
        "eng",
        "reason",
        "recommend",
        "a1",
        "touches",
        "blocking",
        "digest",
    ];
    let mut last = 0;
    for key in keys {
        let needle = format!("\"{key}\":");
        assert_eq!(text.matches(&needle).count(), 1, "{needle} は 1 度: {text}");
        let at = text.find(&needle).expect("鍵");
        assert!(at >= last, "{key} の位置が順でない: {text}");
        last = at;
    }
    let back: QuestionCard = wire::decode(&text).expect("decode");
    assert_eq!(&back, q1);
    // 鍵 blocking の無い字は空の列に読む。
    let mut value: Value = serde_json::from_str(&text).expect("JSON");
    value
        .as_object_mut()
        .expect("object")
        .remove("blocking")
        .expect("鍵 blocking");
    let bare: QuestionCard = wire::decode(&value.to_string()).expect("鍵の無い字の decode");
    assert!(bare.blocking.is_empty());
    assert_eq!(
        QuestionCard {
            blocking: q1.blocking.clone(),
            ..bare
        },
        *q1
    );
    let fixture: QuestionList =
        wire::decode(&read("../../tests/fixtures/surface/question-list.json")).expect("fixture");
    let fixture = known(fixture);
    assert!(!fixture.is_empty());
    for c in &fixture {
        assert!(c.blocking.is_empty(), "{c:?}");
    }
}

#[test]
fn qblock_ids_from_blocks_deps() {
    assert_eq!(BLOCKS_TYPE, "blocks");
    let ledger = lq().to_string();
    let cards = known(list(&ledger));
    let ids: Vec<&str> = cards.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["q-1", "q-2"]);
    assert_eq!(cards[0].blocking, strs(&["q-2", "t-3", "t-4", "t-5"]));
    assert!(cards[1].blocking.is_empty(), "{:?}", cards[1].blocking);
    let open = match open_questions(&ledger) {
        Reading::Known(qs) => qs,
        Reading::Unknown => panic!("open_questions が Unknown"),
    };
    let open_cards: Vec<QuestionCard> = open.into_iter().map(|q| q.card).collect();
    assert_eq!(open_cards, cards);
    // 依存の無い台帳は blocking のほかの欄が同じ。
    let bare = known(list(&lq_bare().to_string()));
    assert_eq!(bare.len(), cards.len());
    for (b, c) in bare.iter().zip(&cards) {
        assert!(b.blocking.is_empty(), "{b:?}");
        assert_eq!(
            *b,
            QuestionCard {
                blocking: vec![],
                ..c.clone()
            }
        );
    }
    let fixture = known(list(&read("../../tests/fixtures/surface/question-2.json")));
    assert!(!fixture.is_empty());
    for c in &fixture {
        assert!(c.blocking.is_empty(), "{c:?}");
    }
}

/// 入れ子の全部から鍵 a1 を持つ object を集める。
fn with_a1<'a>(v: &'a Value, out: &mut Vec<&'a serde_json::Map<String, Value>>) {
    match v {
        Value::Object(map) => {
            if map.contains_key("a1") {
                out.push(map);
            }
            for x in map.values() {
                with_a1(x, out);
            }
        }
        Value::Array(items) => {
            for x in items {
                with_a1(x, out);
            }
        }
        _ => {}
    }
}

#[test]
fn qblock_snapshot_carries_ids() {
    let text = read("../tsuzuri-contract/tests/snapshots/ledger.json");
    let root: Value = serde_json::from_str(&text).expect("snapshot の JSON");
    let mut cards = Vec::new();
    with_a1(&root, &mut cards);
    assert_eq!(cards.len(), 3, "{cards:?}");
    let mut seven = 0;
    let mut eight = 0;
    for c in &cards {
        let blocking = c.get("blocking").unwrap_or_else(|| panic!("鍵 blocking: {c:?}"));
        match c.get("id").and_then(Value::as_str) {
            Some("t3-hub.7") => {
                seven += 1;
                assert_eq!(*blocking, json!(["t3-hub.30", "t3-hub.31"]));
            }
            Some("t3-hub.8") => {
                eight += 1;
                assert_eq!(*blocking, json!([]));
            }
            other => panic!("知らない id {other:?}"),
        }
    }
    assert_eq!((seven, eight), (2, 1));
}

/// 着地済みの行と第 3 波から第 7 波のほかの行の verify の filter の語（92）。
const FILTERS: &[&str] = &[
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
    "mstore_",
    "athr_",
];

#[test]
fn qblock_own_names_clean() {
    assert_eq!(FILTERS.len(), 92);
    let distinct: std::collections::BTreeSet<&&str> = FILTERS.iter().collect();
    assert_eq!(distinct.len(), 92);
    let src = read("tests/qblock.rs");
    let lines: Vec<&str> = src.lines().collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() != "#[test]" {
            continue;
        }
        let next = lines
            .iter()
            .skip(i + 1)
            .find(|l| l.trim_start().starts_with("fn "))
            .expect("test の属性の次の fn");
        let name = next
            .trim_start()
            .trim_start_matches("fn ")
            .split('(')
            .next()
            .unwrap_or_default()
            .to_string();
        names.push(name);
    }
    assert_eq!(names.len(), 4, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("qblock_")
            .unwrap_or_else(|| panic!("{name} は qblock_ で始まる"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
