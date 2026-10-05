//! 未反映の一覧を局面の出力から読む歯（接頭辞 bvunref_・設計ノート surface-wave26a 行 c-unref-lc の完了の条件）。
//! fixture: tests/fixtures/case/unref.json（未反映の 3 種と数えない局面の部品を持つ小さな字）と lifecycle.stale（読むだけ）。
#![cfg(test)]

use crate::common::fixture;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::{UnreflectedCount, UnreflectedKind};
use tsuzuri_core::ledger::unreflected::PHASES;
use tsuzuri_core::ledger::{Unreflected, UnreflectedItem, stats, unreflected, with_unreflected};

/// 題と作った時刻を引く台帳（memo の t3-hub.920 と問いの t3-hub.930 だけ・t3-hub.921 は台帳に無い）。
const LEDGER: &str = r#"[
  {"id":"t3-hub.920","title":"[memo] 処置を待つ控え","status":"open","issue_type":"task","labels":["memo"],"created_at":"2026-09-30T00:00:00Z"},
  {"id":"t3-hub.930","title":"[問い] 反映を待つ裁定","status":"closed","issue_type":"task","labels":["question"],"created_at":"2026-09-29T00:00:00Z","closed_at":"2026-10-01T08:00:00Z"}
]"#;

/// 2026-09-30T00:00:00Z・2026-09-29T00:00:00Z・2026-10-01T11:22:33Z の epoch 秒。
const MEMO_AT: u64 = 1_790_726_400;
const RULING_AT: u64 = 1_790_640_000;
const UTTER_AT: u64 = 1_790_853_753;

fn item(id: &str, title: &str, created: Option<u64>) -> UnreflectedItem {
    UnreflectedItem {
        id: id.to_string(),
        title: title.to_string(),
        created,
    }
}

/// fixture の出力と古さの印から読んだ一覧。
fn read_fixture() -> Unreflected {
    unreflected(
        LEDGER,
        &fixture("unref.json"),
        Some(&fixture("lifecycle.stale")),
    )
}

/// 出力の unmeasured を置き替えた字。
fn with_unmeasured(value: &str) -> String {
    fixture("unref.json").replace(
        r#""unmeasured": [],"#,
        &format!(r#""unmeasured": {value},"#),
    )
}

/// (1) memo は種類 memo の部品のうち局面 memo-actionable と misfit の物だけ（出力の順・memo-waiting と
/// 種類 commit の misfit は数えない）。題と作った時刻は台帳から引き、台帳に無い bead の題は空で時刻は None。
#[test]
fn bvunref_memo_actionable_and_misfit() {
    assert_eq!(
        read_fixture().memos,
        Reading::Known(vec![
            item("t3-hub.920", "[memo] 処置を待つ控え", Some(MEMO_AT)),
            item("t3-hub.921", "", None),
        ])
    );
}

/// (2) 裁定は種類 question の部品のうち局面 ruling-unreflected の物だけ（question-open と question-closed は数えない）。
#[test]
fn bvunref_ruling_unreflected() {
    assert_eq!(
        read_fixture().rulings,
        Reading::Known(vec![item(
            "t3-hub.930",
            "[問い] 反映を待つ裁定",
            Some(RULING_AT)
        )])
    );
}

/// (3) 発話は種類 utterance の部品のうち局面 utterance-open の物だけ（utterance-sorted は数えない）。
/// 題は空、作った時刻は id の発話の ts（秒より下の桁を捨てる）。
#[test]
fn bvunref_utterance_open() {
    assert_eq!(
        read_fixture().utterances,
        Reading::Known(vec![item("2026-10-01T11:22:33.456Z", "", Some(UTTER_AT))])
    );
}

/// (4) 出力が無いか読めない（空の字・JSON でない・版が 1 でない・部品の列が無い）か、古さの印の file が読めないか、
/// unmeasured の欄が無いか形が違えば、3 種とも「まだ分からない」で古さの印は空（出力が JSON の組は印を 1 つ持つ
/// 古さの印の file を渡す）。台帳が読めなくても一覧は出力から読む。
#[test]
fn bvunref_unknown_when_unread() {
    let json = fixture("unref.json");
    let marked = fixture("lifecycle.stale");
    let all = UnreflectedKind::ALL.to_vec();
    for (name, out, stale) in [
        ("空の字", String::new(), None),
        ("JSON でない", "{".to_string(), None),
        (
            "版 2",
            json.replace(r#""version": 1"#, r#""version": 2"#),
            Some(marked.clone()),
        ),
        (
            "部品の列が無い",
            json.replace(r#""parts""#, r#""items""#),
            Some(marked.clone()),
        ),
        ("印が読めない", json.clone(), Some(String::new())),
        (
            "unmeasured が無い",
            json.replace(r#""unmeasured""#, r#""skipped""#),
            Some(marked.clone()),
        ),
        (
            "unmeasured の形",
            with_unmeasured(r#"[{"reason":"ledger-prefix"}]"#),
            Some(marked.clone()),
        ),
    ] {
        let got = unreflected(LEDGER, &out, stale.as_deref());
        assert_eq!(got.unknown(), all, "{name}");
        assert!(got.stale.is_empty(), "{name}");
    }
    let unread_ledger = unreflected("{", &json, None);
    assert!(unread_ledger.unknown().is_empty());
    assert_eq!(
        unread_ledger.memos,
        Reading::Known(vec![
            item("t3-hub.920", "", None),
            item("t3-hub.921", "", None)
        ])
    );
}

/// (5) unmeasured が名指す部品の種類（question・memo・utterance）の種類だけ「まだ分からない」で、ほかは読めた一覧。
#[test]
fn bvunref_unmeasured_kind_unknown() {
    for (part, kind) in [
        ("question", UnreflectedKind::Ruling),
        ("memo", UnreflectedKind::Memo),
        ("utterance", UnreflectedKind::Utterance),
    ] {
        let out = with_unmeasured(&format!(
            r#"[{{"part":"{part}","reason":"ledger-prefix"}}]"#
        ));
        assert_eq!(
            unreflected(LEDGER, &out, None).unknown(),
            vec![kind],
            "{part}"
        );
    }
    let rows = with_unmeasured(r#"[{"part":"row","reason":"table-unreadable"}]"#);
    assert!(unreflected(LEDGER, &rows, None).unknown().is_empty());
}

/// (6) 古さの印が在れば印の種類を一覧に添え（一覧は読めたまま）、印の file が無いか印が 0 件なら空。
#[test]
fn bvunref_stale_marked() {
    let got = read_fixture();
    assert_eq!(got.stale, ["ledger-gate"]);
    assert!(got.unknown().is_empty());
    let json = fixture("unref.json");
    assert!(unreflected(LEDGER, &json, None).stale.is_empty());
    let none = r#"{"version":1,"marks":[]}"#;
    assert!(unreflected(LEDGER, &json, Some(none)).stale.is_empty());
}

/// (7) 指標の未反映の 3 欄は一覧から数え直し（読めた種類ごとの件数・和・分からない種類）、読めなければ 3 種とも分からない。
#[test]
fn bvunref_counted_per_kind() {
    let Reading::Known(base) = stats("[]", UTTER_AT) else {
        panic!("空の台帳は読める");
    };
    let got = with_unreflected(base.clone(), &read_fixture());
    let count = |kind, count| UnreflectedCount { kind, count };
    assert_eq!(
        got.unreflected_kinds,
        vec![
            count(UnreflectedKind::Memo, 2),
            count(UnreflectedKind::Ruling, 1),
            count(UnreflectedKind::Utterance, 1),
        ]
    );
    assert_eq!(got.unreflected, 4);
    assert!(got.unreflected_unknown.is_empty());
    let out = with_unmeasured(r#"[{"part":"question","reason":"unreflected-unreadable"}]"#);
    let part = with_unreflected(base.clone(), &unreflected(LEDGER, &out, None));
    assert_eq!(part.unreflected, 3);
    assert_eq!(part.unreflected_unknown, vec![UnreflectedKind::Ruling]);
    let none = with_unreflected(base, &unreflected(LEDGER, "", None));
    assert_eq!((none.unreflected, none.unreflected_kinds.len()), (0, 0));
    assert_eq!(none.unreflected_unknown, UnreflectedKind::ALL.to_vec());
}

/// (8) 種類ごとの部品の種類と局面の語の表は 1 か所で、種類の順は UnreflectedKind の ALL の順。
#[test]
fn bvunref_phase_table() {
    let got: Vec<(UnreflectedKind, &str, Vec<&str>)> = PHASES
        .iter()
        .map(|(k, p, w)| (*k, *p, w.to_vec()))
        .collect();
    assert_eq!(
        got,
        vec![
            (
                UnreflectedKind::Memo,
                "memo",
                vec!["memo-actionable", "misfit"]
            ),
            (
                UnreflectedKind::Ruling,
                "question",
                vec!["ruling-unreflected"]
            ),
            (
                UnreflectedKind::Utterance,
                "utterance",
                vec!["utterance-open"]
            ),
        ]
    );
}
