//! 配達済みの印と未配達の裁定の歯（行 f-mark・接頭辞 fmark_）。
//! fixture: tests/fixtures/stop/ledger.json（bd の一覧の形・bead 8 本）と ledger-done.json（その 3 本）。
//! 行 f-stop と行 f-mark-serve の歯も同じ 2 つの fixture を読む。
#![cfg(test)]

use std::path::Path;

use serde_json::Value;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_core::delivery::{self, MARK_PREFIX, Pending, RULING_PREFIX, Route};
use tsuzuri_core::graph::build::TYPED_LINES;

const LEDGER: &str = "tests/fixtures/stop/ledger.json";
const LEDGER_DONE: &str = "tests/fixtures/stop/ledger-done.json";
const MINUTE: &str = "20260928T0110Z";

const P1: &str = r#"{"session_id":"s-1","transcript_path":"/tmp/t.jsonl","cwd":"/tmp","hook_event_name":"Stop","stop_hook_active":false}"#;
const P2: &str = "{\"session_id\":\"s-1\",\"transcript_path\":\"/tmp/t.jsonl\",\"cwd\":\"/tmp\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":true}\n";
const P3: &str = r#"{"session_id":"s-1"}"#;
const P4: &str = r#"{"stop_hook_active":"true"}"#;
const P5: &str = "";
const P6: &str = "not json";
const P7: &str = "[]";

/// 行 f-mark の外で決めた filter の語（この行の接頭辞 fmark_ は並べない）。
const FILTER_WORDS: [&str; 92] = [
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
    "nsumw_",
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
    "fstop_",
    "fserve_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
];

type MarkLine = fn(&RulingId, Route, &str) -> String;
type Undelivered = fn(&str) -> Reading<Vec<Pending>>;
type Marked = fn(&str, &BeadId, &RulingId) -> bool;
type StopActive = fn(&str) -> Option<bool>;
type Block = fn(&[Pending]) -> Option<String>;

const MARK_LINE: MarkLine = delivery::mark_line;
const UNDELIVERED: Undelivered = delivery::undelivered;
const MARKED: Marked = delivery::marked;
const STOP_ACTIVE: StopActive = delivery::stop_active;
const BLOCK: Block = delivery::block;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead の id")
}

fn ruling(id: &str) -> RulingId {
    RulingId::new(id).expect("裁定の id")
}

fn pending(question: &str, id: &str) -> Pending {
    Pending {
        question: bead(question),
        ruling: ruling(id),
    }
}

/// 節の 3 つの Pending（台帳の配列の順）。
fn three() -> Vec<Pending> {
    vec![
        pending("fx-s.2", "fx-s.2:20260928T0101Z-1"),
        pending("fx-s.3", "fx-s.3:20260928T0102Z-1"),
        pending("fx-s.4", "fx-s.4:20260928T0105Z-1"),
    ]
}

fn known(ledger: &str) -> Vec<Pending> {
    match UNDELIVERED(ledger) {
        Reading::Known(p) => p,
        Reading::Unknown => panic!("読める台帳が Unknown"),
    }
}

#[test]
fn fmark_mark_line_and_prefixes() {
    assert_eq!(RULING_PREFIX, "裁定 id = ");
    assert_eq!(RULING_PREFIX, TYPED_LINES[0].0, "導出グラフの裁定の頭と同じ字");
    assert_eq!(MARK_PREFIX, "配達 = ");
    for (head, _) in TYPED_LINES {
        assert!(
            !head.starts_with(MARK_PREFIX) && !MARK_PREFIX.starts_with(head),
            "印の頭と {head} は重ならない"
        );
    }
    assert_eq!(Route::Deliver.word(), "配達の口");
    assert_eq!(Route::Stop.word(), "停止");
    let id = ruling("fx-s.2:20260928T0101Z-1");
    assert_eq!(
        MARK_LINE(&id, Route::Stop, MINUTE),
        "配達 = fx-s.2:20260928T0101Z-1・経路 = 停止・時刻 = 20260928T0110Z"
    );
    assert_eq!(
        MARK_LINE(&id, Route::Deliver, MINUTE),
        "配達 = fx-s.2:20260928T0101Z-1・経路 = 配達の口・時刻 = 20260928T0110Z"
    );
}

#[test]
fn fmark_pending_in_ledger_order() {
    assert_eq!(known(&read(LEDGER)), three());
}

#[test]
fn fmark_marks_close_pending() {
    let mut beads: Value = serde_json::from_str(&read(LEDGER)).expect("fixture は JSON");
    let marks = [
        ("fx-s.2", "fx-s.2:20260928T0101Z-1", Route::Deliver),
        ("fx-s.3", "fx-s.3:20260928T0102Z-1", Route::Stop),
        ("fx-s.4", "fx-s.4:20260928T0105Z-1", Route::Stop),
    ];
    for (id, rid, route) in marks {
        let b = beads
            .as_array_mut()
            .expect("配列")
            .iter_mut()
            .find(|b| b["id"] == id)
            .unwrap_or_else(|| panic!("{id} が無い"));
        let notes = b["notes"].as_str().expect("notes").to_string();
        b["notes"] = Value::String(format!(
            "{notes}\n{}",
            MARK_LINE(&ruling(rid), route, MINUTE)
        ));
    }
    let text = beads.to_string();
    assert_eq!(UNDELIVERED(&text), Reading::Known(Vec::new()));
}

#[test]
fn fmark_pending_unknown_or_empty() {
    for text in ["", "not json", "{}"] {
        assert_eq!(UNDELIVERED(text), Reading::Unknown, "{text:?}");
    }
    assert_eq!(UNDELIVERED("[]"), Reading::Known(Vec::new()));
    assert_eq!(UNDELIVERED(&read(LEDGER_DONE)), Reading::Known(Vec::new()));
}

#[test]
fn fmark_marked_same_bead_only() {
    let ledger = read(LEDGER);
    let cases = [
        ("fx-s.1", "fx-s.1:20260928T0100Z-1", true),
        ("fx-s.4", "fx-s.4:20260928T0103Z-1", true),
        ("fx-s.4", "fx-s.4:20260928T0105Z-1", false),
        ("fx-s.2", "fx-s.2:20260928T0101Z-1", false),
        ("fx-s.9", "fx-s.9:20260928T0100Z-1", false),
    ];
    for (q, r, want) in cases {
        assert_eq!(MARKED(&ledger, &bead(q), &ruling(r)), want, "{q} {r}");
    }
    assert!(!MARKED(
        "not json",
        &bead("fx-s.1"),
        &ruling("fx-s.1:20260928T0100Z-1")
    ));
}

#[test]
fn fmark_active_flag_from_payload() {
    assert_eq!(STOP_ACTIVE(P2), Some(true));
    for p in [P1, P3, P4] {
        assert_eq!(STOP_ACTIVE(p), Some(false), "{p}");
    }
    for p in [P5, P6, P7] {
        assert_eq!(STOP_ACTIVE(p), None, "{p:?}");
    }
}

#[test]
fn fmark_block_names_each_ruling() {
    assert_eq!(BLOCK(&[]), None);
    let text = BLOCK(&three()).expect("未配達が在れば答えを組む");
    let v: Value = serde_json::from_str(&text).expect("答えは JSON");
    let o = v.as_object().expect("object");
    let mut keys: Vec<&str> = o.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["decision", "reason"]);
    assert_eq!(o["decision"], "block");
    assert_eq!(
        o["reason"],
        "席に届いていない持ち主の裁定が 3 件ある（逐語は台帳の問いの notes の裁定の行）。\n\
         裁定 fx-s.2:20260928T0101Z-1 が届いた（問い fx-s.2）\n\
         裁定 fx-s.3:20260928T0102Z-1 が届いた（問い fx-s.3）\n\
         裁定 fx-s.4:20260928T0105Z-1 が届いた（問い fx-s.4）"
    );
}

#[test]
fn fmark_delivery_is_pure() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/delivery.rs");
    let src = std::fs::read_to_string(&path).expect("src/delivery.rs を読む");
    for word in [
        "std::fs",
        "std::process",
        "std::net",
        "SystemTime",
        "Instant",
    ] {
        assert!(!src.contains(word), "src/delivery.rs に {word}");
    }
}

#[test]
fn fmark_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/fmark.rs");
    let src = std::fs::read_to_string(&path).expect("tests/teeth1/fmark.rs を読む");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 9, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("fmark_")
            .unwrap_or_else(|| panic!("{name} は fmark_ で始まらない"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
