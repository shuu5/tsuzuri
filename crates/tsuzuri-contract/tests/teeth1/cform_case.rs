//! 契約の型の外形の歯（接頭辞 bvcform_・設計ノート surface-wave26a 行 c-case-read）のうち case の群の型の見本。
//! snapshot は tests/snapshots の群の名の json（case.json）で、字の比べは標準 library だけで行う。
//! 共通の手は tests/teeth1/common/mod.rs に 1 つだけ在る。
//! 字が違えば今の字を CARGO_TARGET_TMPDIR の群の名の json に書いて落ちる（見て正しければ snapshot へ写す）。
#![cfg(test)]

use crate::common;

use common::{AT, Form, form};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::case::{CaseDoc, CaseLinks, CasePart, PATH};
use tsuzuri_contract::wire;

/// 依存で待つ契約と、知らない局面の語を持つ便の部品。
fn parts() -> Vec<CasePart> {
    vec![
        CasePart {
            part: "contract".into(),
            id: "t3-hub.902".into(),
            phase: "contract-queued".into(),
            turn: "vessel".into(),
            since: Some(AT - 600),
            reason: Some("dependency".into()),
            why: None,
            closed: false,
            links: CaseLinks {
                on: vec!["t3-hub.903".into()],
                runs: vec![],
            },
        },
        CasePart {
            part: "run".into(),
            id: "t3-hub.905-20261001T120000Z".into(),
            phase: "run-parked".into(),
            turn: "user".into(),
            since: None,
            reason: None,
            why: None,
            closed: false,
            links: CaseLinks::default(),
        },
    ]
}

/// case の群の型の見本（この順が snapshot の file の key の順）。
fn forms() -> Vec<Box<dyn Form>> {
    vec![
        form(
            "case::CaseDoc",
            vec![
                CaseDoc {
                    generated_at: Some(AT),
                    stale: vec!["ledger-gate".into()],
                    parts: Reading::Known(parts()),
                    unreadable: false,
                },
                // 出力が無い（まだ分からない）。
                CaseDoc {
                    generated_at: None,
                    stale: vec![],
                    parts: Reading::Unknown,
                    unreadable: false,
                },
                // 出力は在るが読めない版（読めない）。
                CaseDoc {
                    generated_at: None,
                    stale: vec![],
                    parts: Reading::Unknown,
                    unreadable: true,
                },
            ],
        ),
        form("case::CasePart", parts()),
        form(
            "case::CaseLinks",
            vec![
                CaseLinks::default(),
                CaseLinks {
                    on: vec!["t3-hub.905".into()],
                    runs: vec!["t3-hub.905-20261001T120000Z".into()],
                },
            ],
        ),
    ]
}

#[test]
fn bvcform_snapshot_matches() {
    common::snapshot_matches("case", &forms());
}

#[test]
fn bvcform_roundtrip_all() {
    common::roundtrip_all(&forms());
}

/// 欄 unreadable は false なら電文に字を置かず、鍵の無い前の電文は false に読む（行 c-case-unreadable）。
#[test]
fn bvcform_unreadable_defaults_false() {
    let old = r#"{"generated_at":null,"stale":[],"parts":"unknown"}"#;
    let doc: CaseDoc = wire::decode(old).expect("鍵の無い前の電文");
    assert!(!doc.unreadable, "鍵の無い電文は読めた周");
    assert_eq!(doc.parts, Reading::Unknown);
    let text = wire::encode(&doc).expect("電文");
    assert_eq!(text, old, "false は字を置かない");
    let broken = CaseDoc {
        unreadable: true,
        ..doc
    };
    let text = wire::encode(&broken).expect("電文");
    assert!(text.ends_with(r#","unreadable":true}"#), "{text}");
}

#[test]
fn bvcform_path_is_api_cases() {
    assert_eq!(PATH, "/api/cases");
}
