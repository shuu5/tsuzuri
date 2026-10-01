//! 契約の型の外形の歯（接頭辞 contract_form_）のうち ledger の群（module ledger と question）の型の見本。
//! snapshot は tests/snapshots の群の名の json（ledger.json）で、字の比べは標準 library だけで行う。
//! 共通の手は tests/common/mod.rs に 1 つだけ在る。
//! 字が違えば今の字を CARGO_TARGET_TMPDIR の群の名の json に書いて落ちる（見て正しければ snapshot へ写す）。
#![cfg(test)]

mod common;

use std::collections::BTreeSet;

use common::{AT, Form, bead, form};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::{
    BDW, BdLine, BeadId, ChildType, Effect, LedgerChanged, LedgerItem, LedgerList, LedgerRow, LedgerWrite,
    MEMO_LABEL, NOTES_REPLACE_FLAG, PARENT_FLAG, QUESTION_LABEL, fnv1a64,
};
use tsuzuri_contract::question::{AllQuestions, ProjectQuestions, QuestionCard, QuestionList};
use tsuzuri_contract::surface::RulingId;
use tsuzuri_contract::wire;

fn ledger_row() -> LedgerRow {
    LedgerRow {
        id: bead("t3-hub.5"),
        kind: "task".into(),
        title: "契約の型".into(),
        status: "open".into(),
        updated_at: AT,
        parent: Some(bead("t3-hub")),
        labels: vec![QUESTION_LABEL.into()],
    }
}

/// 親も label も無い行（根の bead）。
fn root_row() -> LedgerRow {
    LedgerRow {
        id: bead("t3-hub"),
        kind: "epic".into(),
        title: "根".into(),
        status: "closed".into(),
        updated_at: AT,
        parent: None,
        labels: vec![],
    }
}

fn ledger_writes() -> Vec<LedgerWrite> {
    vec![
        LedgerWrite::AppendNotes {
            id: bead("t3-hub.5"),
            line: "裁定 fx-c.5:20260926T1437Z-1・逐語 = よい".into(),
        },
        LedgerWrite::CloseItem {
            id: bead("t3-hub.5"),
            reason: "裁定 fx-c.5:20260926T1437Z-1".into(),
        },
        LedgerWrite::CreateChild {
            parent: bead("t3-hub"),
            title: "方針".into(),
            child_type: ChildType::Task,
            description: "全体への指示の控え".into(),
            labels: vec![QUESTION_LABEL.into(), "policy-scope:all".into()],
            effect: Some(Effect::Operation),
        },
        // 人の字が旗の形でも旗に化けない見本。
        LedgerWrite::AppendNotes {
            id: bead("t3-hub.5"),
            line: "--notes".into(),
        },
        LedgerWrite::CreateChild {
            parent: bead("t3-hub"),
            title: "--notes=x".into(),
            child_type: ChildType::Epic,
            description: "--parent=".into(),
            labels: vec![NOTES_REPLACE_FLAG.into()],
            effect: None,
        },
        LedgerWrite::ReopenItem {
            id: bead("t3-hub.5"),
            reason: "裁定 fx-c.5:20260926T1440Z-1・取り消す = fx-c.5:20260926T1437Z-1".into(),
        },
    ]
}

fn ledger_item() -> LedgerItem {
    LedgerItem {
        row: ledger_row(),
        description: "本文".into(),
        notes: "裁定 fx-c.5:20260926T1437Z-1".into(),
    }
}

/// 定型行を全部持つ問いの card。
fn question_card() -> QuestionCard {
    QuestionCard {
        id: bead("t3-hub.7"),
        title: "画面の色".into(),
        posted_at: AT,
        plain: Some("画面の色を決める".into()),
        eng: Some("CSS の変数を 2 組持つ".into()),
        reason: Some("夜に読む".into()),
        recommend: Some("暗い色を既定にする".into()),
        a1: true,
        touches: vec!["surface-base#b-cards".into(), "ADR-7".into()],
        blocking: vec!["t3-hub.30".into(), "t3-hub.31".into()],
        digest: ledger_item().digest(),
    }
}

/// ledger の群の型の見本（この順が snapshot の file の key の順）。
fn forms() -> Vec<Box<dyn Form>> {
    vec![
        // ledger
        form("ledger::BeadId", vec![bead("t3-hub"), bead("t3-hub.5")]),
        form("ledger::ChildType", vec![ChildType::Task, ChildType::Epic]),
        form("ledger::Effect", vec![Effect::Document, Effect::Operation]),
        form("ledger::LedgerRow", vec![ledger_row(), root_row()]),
        form("ledger::LedgerItem", vec![ledger_item()]),
        form(
            "ledger::LedgerList",
            vec![
                LedgerList {
                    rows: Reading::Known(vec![ledger_row()]),
                },
                LedgerList {
                    rows: Reading::Known(vec![]),
                },
                LedgerList {
                    rows: Reading::Unknown,
                },
            ],
        ),
        form("ledger::LedgerChanged", vec![LedgerChanged { at: AT }]),
        form("ledger::LedgerWrite", ledger_writes()),
    ]
    .into_iter()
    .chain(question_forms())
    .collect()
}

/// ledger の群の型の見本の後半（module question の型）。
fn question_forms() -> Vec<Box<dyn Form>> {
    vec![
        // question
        form(
            "question::QuestionCard",
            vec![
                question_card(),
                // 定型行も A-1 の印も名指しも無い問い。
                QuestionCard {
                    id: bead("t3-hub.8"),
                    title: "素の問い".into(),
                    posted_at: AT + 60,
                    plain: None,
                    eng: None,
                    reason: None,
                    recommend: None,
                    a1: false,
                    touches: vec![],
                    blocking: vec![],
                    digest: "cbf29ce484222325".into(),
                },
            ],
        ),
        form(
            "question::QuestionList",
            vec![
                QuestionList {
                    cards: Reading::Known(vec![question_card()]),
                    answerable: true,
                },
                QuestionList {
                    cards: Reading::Known(vec![]),
                    answerable: true,
                },
                QuestionList {
                    cards: Reading::Unknown,
                    answerable: true,
                },
            ],
        ),
        form("question::ProjectQuestions", project_questions()),
        form(
            "question::AllQuestions",
            vec![
                AllQuestions {
                    own: own_questions(),
                    others: vec![],
                },
                AllQuestions {
                    own: own_questions(),
                    others: project_questions(),
                },
            ],
        ),
    ]
}

/// ほかの project の問いの一覧の見本（card の列は空・card の形は question::QuestionCard の見本が持つ）。
fn project_questions() -> Vec<ProjectQuestions> {
    vec![
        ProjectQuestions {
            project: "proj-x".into(),
            answerable: false,
            cards: Reading::Known(vec![]),
        },
        ProjectQuestions {
            project: "proj-y".into(),
            answerable: false,
            cards: Reading::Unknown,
        },
    ]
}

/// 全部の問いの一覧の見本の自分の問いの一覧。
fn own_questions() -> QuestionList {
    QuestionList {
        cards: Reading::Known(vec![]),
        answerable: true,
    }
}

#[test]
fn contract_form_snapshot_matches() {
    common::snapshot_matches("ledger", &forms());
}

#[test]
fn contract_form_roundtrip_all_types() {
    common::roundtrip_all(&forms());
}

#[test]
fn contract_form_ledger_write_argv() {
    let writes = ledger_writes();
    let mut variants = BTreeSet::new();
    for w in &writes {
        // 全 variant を数える（variant が増えれば match が網羅でなくなり組めない）。
        variants.insert(match w {
            LedgerWrite::AppendNotes { .. } => "append-notes",
            LedgerWrite::CloseItem { .. } => "close-item",
            LedgerWrite::CreateChild { .. } => "create-child",
            LedgerWrite::ReopenItem { .. } => "reopen-item",
        });
        let argv = w.argv();
        assert!(!argv.is_empty(), "{w:?}: argv が空");
        assert_ne!(argv[0], BDW, "{w:?}: argv は program の名を含めない");
        // 旗として読まれるのは `--` の前の語だけ。
        let flags: Vec<&String> = argv.iter().take_while(|a| a.as_str() != "--").collect();
        assert!(
            !flags.iter().any(|a| {
                a.as_str() == NOTES_REPLACE_FLAG
                    || a.starts_with(&format!("{NOTES_REPLACE_FLAG}="))
                    || a.as_str() == "-n"
            }),
            "{w:?}: notes を置き換える旗を持つ {argv:?}"
        );
        let parents: Vec<&str> = flags
            .iter()
            .filter_map(|a| a.strip_prefix(&format!("{PARENT_FLAG}=")))
            .collect();
        if w.creates_child() {
            assert_eq!(argv[0], "create", "{w:?}");
            assert_eq!(
                parents.len(),
                1,
                "{w:?}: 子を作る argv は親を 1 つ持つ {argv:?}"
            );
            assert!(
                BeadId::new(parents[0]).is_ok(),
                "{w:?}: 親が bead id でない {argv:?}"
            );
        } else {
            assert_ne!(argv[0], "create", "{w:?}: 子を作らない書きが create を撃つ");
        }
        // 人の字は旗の値か `--` の後にだけ在り、旗の位置に素で立たない。
        for a in &flags[1..] {
            assert!(
                !a.starts_with('-') || a.starts_with("--") && a.contains('='),
                "{w:?}: 値の無い旗 {a} {argv:?}"
            );
        }
    }
    assert_eq!(variants.len(), 4,"全 variant の見本が要る: {variants:?}");
}

#[test]
fn contract_form_ids_refuse_bad_shape() {
    for bad in ["\"\"", "\"-x\"", "\"a b\"", "\"t3:hub\""] {
        assert!(
            wire::decode::<BeadId>(bad).is_err(),
            "bead id が {bad} を通す"
        );
    }
    for bad in ["\"\"", "\"-x\"", "\"a@b#1\""] {
        assert!(
            wire::decode::<RulingId>(bad).is_err(),
            "記帳 id が {bad} を通す"
        );
    }
    let long = format!("\"{}\"", "a".repeat(65));
    assert!(wire::decode::<RulingId>(&long).is_err());
    // 親の無い子の作成は電文でも組めない。
    let orphan = r#"{"op":"create-child","title":"x","child_type":"task","description":"y"}"#;
    assert!(wire::decode::<LedgerWrite>(orphan).is_err());
    // 閉じた enum は知らない語を断る。
    assert!(
        wire::decode::<LedgerWrite>(r#"{"op":"replace-notes","id":"t3-hub.5","notes":"x"}"#)
            .is_err()
    );
    assert!(wire::decode::<NodeKind>("\"file\"").is_err());
}

#[test]
fn contract_form_bd_line_reads() {
    // 知らない欄は読み捨て、省かれた種類・本文・notes・親・label は空で読む。
    let line = r#"{"id":"t3-hub.5","title":"契約の型","status":"open","priority":2,"updated_at":"2026-09-27T07:39:00Z","labels":["x"]}"#;
    let bd: BdLine = wire::decode(line).expect("bd の行");
    assert!(!bd.is_tombstone());
    let item = bd.into_item(AT);
    assert_eq!(item.row.id, bead("t3-hub.5"));
    assert_eq!(item.row.kind, "");
    assert_eq!(item.row.updated_at, AT);
    assert_eq!(item.row.parent, None);
    assert_eq!(item.row.labels, vec!["x".to_string()]);
    assert_eq!(item.description, "");
    // 親と label は行へ写る。
    let child = r#"{"id":"t3-hub.7","title":"問い","status":"closed","updated_at":"2026-09-27T07:39:00Z","parent":"t3-hub","labels":["intake:question"],"dependencies":[{"issue_id":"t3-hub.7","depends_on_id":"t3-hub","type":"parent-child"}]}"#;
    let row = wire::decode::<BdLine>(child)
        .expect("bd の行")
        .into_item(AT)
        .row;
    assert_eq!(row.parent, Some(bead("t3-hub")));
    assert!(row.is_question() && !row.is_memo(), "{row:?}");
    // 形の悪い id と親と欠けた更新時刻は読まない。
    for bad in [
        r#"{"id":"-x","title":"t","status":"open","updated_at":"2026-09-27T07:39:00Z"}"#,
        r#"{"id":"t3-hub.5","title":"t","status":"open"}"#,
        r#"{"id":"t3-hub.5","title":"t","status":"open","updated_at":"2026-09-27T07:39:00Z","parent":"a b"}"#,
    ] {
        assert!(wire::decode::<BdLine>(bad).is_err(), "{bad} を読む");
    }
    let gone =
        r#"{"id":"t3-hub.6","title":"t","status":"tombstone","updated_at":"2026-09-27T07:39:00Z"}"#;
    assert!(
        wire::decode::<BdLine>(gone)
            .expect("bd の行")
            .is_tombstone()
    );
}

#[test]
fn contract_form_ledger_row_labels() {
    assert_eq!(QUESTION_LABEL, "intake:question");
    assert_eq!(MEMO_LABEL, "intake:memo");
    let question = ledger_row();
    assert!(question.is_question() && !question.is_memo());
    let memo = LedgerRow {
        labels: vec!["surface".into(), MEMO_LABEL.into()],
        ..ledger_row()
    };
    assert!(memo.is_memo() && !memo.is_question());
    let plain = root_row();
    assert!(!plain.is_memo() && !plain.is_question());
    // label の語の一部だけでは見分けない。
    let near = LedgerRow {
        labels: vec!["intake:questions".into(), "intake".into()],
        ..ledger_row()
    };
    assert!(!near.is_question() && !near.is_memo());
}

#[test]
fn contract_form_ledger_item_digest() {
    // FNV-1a 64 bit の公開の見本。
    assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    let item = ledger_item();
    let digest = item.digest();
    assert_eq!(digest, "2f890a3dc04a8b5a");
    assert_eq!(
        digest,
        format!(
            "{:016x}",
            fnv1a64("t3-hub.5\n契約の型\nopen\n本文\n裁定 fx-c.5:20260926T1437Z-1".as_bytes())
        )
    );
    // 同じ中身なら同じ値（数えない欄は値を動かさない）。
    assert_eq!(item.clone().digest(), digest);
    let other_row = LedgerItem {
        row: LedgerRow {
            kind: "epic".into(),
            updated_at: AT + 1,
            parent: None,
            labels: vec![],
            ..ledger_row()
        },
        ..ledger_item()
    };
    assert_eq!(other_row.digest(), digest);
    digest_changes(digest);
}

/// 1 字を変えた item の digest が元の値とも互いにも違う 16 進の小文字 16 字であることを見る。
fn digest_changes(digest: String) {
    // どの欄の 1 字が変わっても値が変わる（区切りの位置が動くだけでも変わる）。
    let changed = [
        LedgerItem {
            row: LedgerRow {
                id: bead("t3-hub.6"),
                ..ledger_row()
            },
            ..ledger_item()
        },
        LedgerItem {
            row: LedgerRow {
                title: "契約の形".into(),
                ..ledger_row()
            },
            ..ledger_item()
        },
        LedgerItem {
            row: LedgerRow {
                status: "opem".into(),
                ..ledger_row()
            },
            ..ledger_item()
        },
        LedgerItem {
            description: "本分".into(),
            ..ledger_item()
        },
        LedgerItem {
            notes: "裁定 fx-c.5:20260926T1437Z-2".into(),
            ..ledger_item()
        },
        LedgerItem {
            description: "本文\n裁定".into(),
            notes: " fx-c.5:20260926T1437Z-1".into(),
            ..ledger_item()
        },
    ];
    let mut seen = BTreeSet::from([digest.clone()]);
    for c in &changed {
        let d = c.digest();
        assert_eq!(d.len(), 16, "{d}");
        assert!(
            d.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "16 進の小文字でない {d}"
        );
        assert!(seen.insert(d.clone()), "{c:?}: 値が変わらない {d}");
    }
}
