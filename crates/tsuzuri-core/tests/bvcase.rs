//! 器の局面の出力の読みの歯（接頭辞 bvcase_・設計ノート surface-wave26a 行 c-case-read の完了の条件）。
//! fixture: tests/fixtures/case/lifecycle.json と lifecycle.stale（器の実物の形を写した小さな字・読むだけ）。短い字の組は歯の中に書く。
#![cfg(test)]

use std::path::{Path, PathBuf};

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::case::{CaseDoc, CaseLinks, CasePart};
use tsuzuri_core::case::cases_of;

fn fixture(name: &str) -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/case")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// 読めた部品の列（Unknown なら落ちる）。
fn known(doc: &CaseDoc) -> &[CasePart] {
    match &doc.parts {
        Reading::Known(parts) => parts,
        Reading::Unknown => panic!("部品が読めたはず"),
    }
}

/// 部品を id で引く。
fn by_id<'a>(parts: &'a [CasePart], id: &str) -> &'a CasePart {
    parts
        .iter()
        .find(|p| p.id == id)
        .unwrap_or_else(|| panic!("部品 {id} が無い"))
}

#[test]
fn bvcase_fields_copied_from_fixture() {
    let doc = cases_of(&fixture("lifecycle.json"), None);
    assert_eq!(doc.generated_at, Some(1_790_866_402));
    assert!(doc.stale.is_empty(), "古さの印の file が無ければ古くない");
    let parts = known(&doc);
    assert_eq!(parts.len(), 9, "欄の欠けた 1 つを除く 9 つ");
    assert_eq!(
        by_id(parts, "t3-hub.902"),
        &CasePart {
            part: "contract".into(),
            id: "t3-hub.902".into(),
            phase: "contract-queued".into(),
            turn: "vessel".into(),
            since: Some(1_790_863_200),
            reason: Some("dependency".into()),
            closed: false,
            links: CaseLinks {
                on: ids(&["t3-hub.903"]),
                runs: vec![],
            },
        }
    );
    let overlap = by_id(parts, "t3-hub.904");
    assert_eq!(overlap.links.on, ids(&["t3-hub.905"]));
    assert!(overlap.links.runs.is_empty());
    let reserved = by_id(parts, "t3-hub.910");
    assert_eq!(
        (reserved.reason.as_deref(), &reserved.links.on),
        (Some("reserved"), &ids(&["t3-hub.909"])),
        "理由 reserved の相手も結びの on のまま"
    );
    let running = by_id(parts, "t3-hub.905");
    assert_eq!(running.links.runs, ids(&["t3-hub.905-20261001T120000Z"]));
    let run = by_id(parts, "t3-hub.905-20261001T120000Z");
    assert_eq!(
        (run.part.as_str(), run.phase.as_str(), run.turn.as_str()),
        ("run", "run-blocked", "user")
    );
    assert_eq!(run.links, CaseLinks::default(), "欠けた key は空の列");
    let question = by_id(parts, "t3-hub.901");
    assert_eq!(
        (question.closed, question.since, question.reason.as_deref()),
        (true, Some(1_790_644_171), None)
    );
    assert_eq!(by_id(parts, "t3-hub.900").since, None, "null の since");
}

#[test]
fn bvcase_unknown_words_kept_as_text() {
    let doc = cases_of(&fixture("lifecycle.json"), None);
    let odd = by_id(known(&doc), "t3-hub.907");
    assert_eq!(
        (odd.phase.as_str(), odd.turn.as_str(), odd.reason.as_deref()),
        ("contract-parked", "later", Some("new-reason")),
        "知らない局面と手番と理由の語も字のまま"
    );
    let json = r#"{"version":1,"generated_at":"x","parts":[
        {"part":"novel","id":"n-1","phase":"novel-open","turn":"none","since":"not-a-time","reason":null,"closed":false,"links":{"on":["a"],"future":[1]}}]}"#;
    let doc = cases_of(json, None);
    let part = by_id(known(&doc), "n-1");
    assert_eq!(part.part, "novel", "知らない種類の部品も落とさない");
    assert_eq!(part.since, None, "読めない時刻は None");
    assert_eq!(part.links.on, ids(&["a"]), "知らない結びの key は読まない");
    assert_eq!(doc.generated_at, None, "読めない生成の時刻は None");
}

#[test]
fn bvcase_broken_piece_dropped() {
    let doc = cases_of(&fixture("lifecycle.json"), None);
    assert!(
        known(&doc).iter().all(|p| p.id != "t3-hub.908"),
        "closed の欠けた部品は落とす"
    );
    let head = r#"{"version":1,"generated_at":"2026-10-01T14:53:22Z","parts":["#;
    let good = r#"{"part":"epic","id":"e-1","phase":"epic-open","turn":"none","since":null,"reason":null,"closed":false,"links":{}}"#;
    for bad in [
        r#"{"part":"epic","id":"e-2","phase":"epic-open","turn":"none","since":null,"reason":null,"closed":"no","links":{}}"#,
        r#"{"part":"epic","id":"e-2","phase":"epic-open","turn":"none","since":null,"reason":null,"closed":false}"#,
        r#"{"part":"epic","id":"e-2","phase":"epic-open","turn":"none","since":null,"reason":null,"closed":false,"links":{"on":[7]}}"#,
        r#"{"part":"epic","id":"e-2","phase":"epic-open","turn":"none","reason":null,"closed":false,"links":{}}"#,
        r#"{"part":"epic","id":"e-2","phase":3,"turn":"none","since":null,"reason":null,"closed":false,"links":{}}"#,
        r#"{"part":"epic","id":"e-2","phase":"epic-open","turn":"none","since":null,"reason":5,"closed":false,"links":{}}"#,
    ] {
        let doc = cases_of(&format!("{head}{good},{bad}]}}"), None);
        let got: Vec<&str> = known(&doc).iter().map(|p| p.id.as_str()).collect();
        assert_eq!(got, ["e-1"], "{bad}");
    }
}

#[test]
fn bvcase_unreadable_is_unknown() {
    let json = fixture("lifecycle.json");
    let unknown = CaseDoc {
        generated_at: None,
        stale: vec![],
        parts: Reading::Unknown,
    };
    for (out, stale, why) in [
        ("", None, "出力が無い"),
        ("not json", None, "出力が JSON でない"),
        (r#"{"version":2,"parts":[]}"#, None, "出力の版が 1 でない"),
        (r#"{"version":1}"#, None, "部品の列が無い"),
        (r#"{"version":1,"parts":{}}"#, None, "部品が列でない"),
        (json.as_str(), Some(""), "古さの印の file を読めない"),
        (json.as_str(), Some("not json"), "古さの印が JSON でない"),
        (
            json.as_str(),
            Some(r#"{"version":2,"marks":[]}"#),
            "古さの印の版が 1 でない",
        ),
        (json.as_str(), Some(r#"{"version":1}"#), "印の列が無い"),
        (
            json.as_str(),
            Some(r#"{"version":1,"marks":[{"at":"x"}]}"#),
            "印の種類が無い",
        ),
        (
            json.as_str(),
            Some(r#"{"version":1,"marks":[{"kind":"merge-gate"},{"kind":"merge-gate"}]}"#),
            "同じ種類の印が 2 つ",
        ),
    ] {
        assert_eq!(cases_of(out, stale), unknown, "{why}");
    }
}

#[test]
fn bvcase_stale_marks_listed() {
    let json = fixture("lifecycle.json");
    let doc = cases_of(&json, Some(&fixture("lifecycle.stale")));
    assert_eq!(doc.stale, ids(&["ledger-gate"]));
    assert_eq!(known(&doc).len(), 9, "古くても部品は読む");
    let none = cases_of(&json, Some(r#"{"version":1,"marks":[]}"#));
    assert!(none.stale.is_empty(), "0 件の印は古くない");
    let three = r#"{"version":1,"marks":[{"kind":"unreadable"},{"kind":"merge-gate"},{"kind":"next-gate"}]}"#;
    assert_eq!(
        cases_of(&json, Some(three)).stale,
        ids(&["unreadable", "merge-gate", "next-gate"]),
        "file の順で・知らない種類も字のまま"
    );
}
