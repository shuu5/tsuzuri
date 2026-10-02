//! 便 g-batch の歯: 問いの頁の右の列（まとめて承認と全体への指示）の枠・行の一覧（fixture の 2 本）・
//! 関わる所の数と重なりの数・束を送る button の判定・束の要求の本文・束の応答の出し方・指示の本文の範囲はつねに all・
//! 指示を送る button の判定と応答の出し方・鍵の判定・口の path・測れていない・画面の外の保存の口の名が code に無い・
//! 足す外の依存は 0 本。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::title36;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::{QuestionCard, QuestionList};
use tsuzuri_contract::surface::{
    BatchItem, BatchItemResult, BatchRequest, BatchResponse, ItemOutcome, PolicyRequest,
    PolicyResponse, Refusal, RefusalResponse, RulingId,
};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ask::{self, KeyAction};
use tsuzuri_surface::project::{Body, NOT_READ, batch, policy};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn question_list() -> Fetched {
    Fetched::Body(read("../../tests/fixtures/surface/question-list.json"))
}

fn fixture_cards() -> Vec<QuestionCard> {
    ask::cards(&question_list()).expect("fixture の card")
}

fn fixture_rows() -> Vec<batch::Row> {
    match batch::body(&question_list()) {
        Body::Filled(rows) => rows,
        other => panic!("fixture の行が中身にならない: {other:?}"),
    }
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead の id")
}

fn ruling(id: &str) -> RulingId {
    RulingId::new(id).expect("記帳 id")
}

/// 歯の中で組む card（A-1 の印なし）。
fn card(id: &str, touches: &[&str]) -> QuestionCard {
    QuestionCard {
        id: bead(id),
        title: format!("{id} の題"),
        posted_at: 0,
        plain: None,
        eng: None,
        reason: None,
        recommend: None,
        a1: false,
        touches: touches.iter().map(|s| s.to_string()).collect(),
        blocking: vec![],
        digest: format!("{id}-digest"),
    }
}

fn list_of(cards: Vec<QuestionCard>) -> Fetched {
    Fetched::Body(
        wire::encode(&QuestionList {
            cards: Reading::Known(cards),
            answerable: true,
        })
        .expect("電文"),
    )
}



/// (2) 行は一覧の順の 1 から始まる番号と 36 字に切った題・A-1 の印を持つ 1 本目は選べず、2 本目は選べて初めは選ばれている。
#[test]
fn batchpanel_rows_from_fixture() {
    let cards = fixture_cards();
    let rows = fixture_rows();
    assert_eq!(rows.len(), 2);
    let got: Vec<(usize, &str, bool)> = rows
        .iter()
        .map(|r| (r.number, r.id.as_str(), r.selectable()))
        .collect();
    assert_eq!(got, vec![(1, "qa.2", false), (2, "qa.10", true)]);
    assert!(rows[0].a1);
    assert!(!rows[1].a1);
    assert_eq!(rows[0].title, "質問の頁の答えの欄は 1 問に 1 つでよいか");
    assert_eq!(rows[1].title, title36(&cards[1].title));
    assert_eq!(rows[1].title.chars().count(), 36);
    assert_eq!(
        rows[1].title,
        "これまでの決定の段を開いたままにするか閉じておくかをどちらにするか決めて"
    );
    // 初めは A-1 の印を持たない行だけが選ばれている。
    let chosen = batch::selected(&rows, &BTreeSet::new());
    let ids: Vec<&str> = chosen.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, vec!["qa.10"]);
    // 外した行は選ばれていない・読み直しで増えた行は初めのとおり選ばれている。
    let off = BTreeSet::from([bead("qa.10")]);
    assert!(batch::selected(&rows, &off).is_empty());
    let mut more = cards.clone();
    more.push(card("qa.11", &[]));
    let rows3 = batch::rows(&more);
    let ids: Vec<&str> = batch::selected(&rows3, &off)
        .iter()
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(ids, vec!["qa.11"]);
    assert_eq!(rows3[2].number, 3);
}

/// (3) 関わる所の数は touches の和集合の数・重なりの数は 2 つ以上の選んだ card が持つ touches の数。
#[test]
fn batchpanel_touch_counts() {
    let cards = vec![
        card("c.1", &["FR1", "FR2"]),
        card("c.2", &["FR2", "ADR-7"]),
        card("c.3", &["ADR-7"]),
    ];
    let rows = batch::rows(&cards);
    let all = batch::selected(&rows, &BTreeSet::new());
    assert_eq!(
        batch::counts(&all),
        batch::Counts {
            touches: 3,
            overlap: 2
        }
    );
    let off = BTreeSet::from([bead("c.2"), bead("c.3")]);
    let first = batch::selected(&rows, &off);
    assert_eq!(
        batch::counts(&first),
        batch::Counts {
            touches: 2,
            overlap: 0
        }
    );
    assert_eq!(
        batch::counts(&[]),
        batch::Counts {
            touches: 0,
            overlap: 0
        }
    );
    // 1 本の card の中で同じ字が 2 度在っても重なりに数えない。
    let twice = batch::rows(&[card("c.4", &["FR1", "FR1"])]);
    let chosen = batch::selected(&twice, &BTreeSet::new());
    assert_eq!(
        batch::counts(&chosen),
        batch::Counts {
            touches: 1,
            overlap: 0
        }
    );
    // fixture: A-1 の印を持つ 1 本目は選べないので、2 本目（touches 0）だけ。
    let rows = fixture_rows();
    let chosen = batch::selected(&rows, &BTreeSet::new());
    assert_eq!(
        batch::counts(&chosen),
        batch::Counts {
            touches: 0,
            overlap: 0
        }
    );
}

/// (4) 束を送る button は、逐語が空白だけか、選んだ card が 0 本か、送っている間は押せない。
#[test]
fn batchpanel_batch_send_button() {
    assert!(batch::can_send("まとめて進めてよい", 1, false));
    assert!(batch::can_send("  a ", 3, false));
    assert!(!batch::can_send("", 1, false));
    assert!(!batch::can_send(" \n\t　", 1, false));
    assert!(!batch::can_send("はい", 0, false));
    assert!(!batch::can_send("はい", 1, true));
    assert!(!batch::can_send("", 0, true));
}

/// (5) 束の要求の本文は選んだ card だけを一覧の順に・問いの id と card の digest と個別の逐語なし・束の逐語は欄の字。
#[test]
fn batchpanel_batch_request_body() {
    let cards = vec![
        card("c.1", &["FR1"]),
        card("c.2", &["FR2"]),
        card("c.3", &["FR3"]),
    ];
    let rows = batch::rows(&cards);
    let off = BTreeSet::from([bead("c.2")]);
    let words = "  おすすめどおりでよい。\n後で見直す  ";
    let body = batch::request_body(&batch::selected(&rows, &off), words);
    let req: BatchRequest = wire::decode(&body).expect("要求の電文");
    assert_eq!(
        req,
        BatchRequest {
            items: vec![
                BatchItem {
                    question: bead("c.1"),
                    seen_digest: "c.1-digest".to_string(),
                    verbatim: None,
                },
                BatchItem {
                    question: bead("c.3"),
                    seen_digest: "c.3-digest".to_string(),
                    verbatim: None,
                },
            ],
            verbatim: words.to_string(),
        }
    );
    // fixture: A-1 の印を持つ card は選べないので入らない（2 本目だけ）。
    let rows = fixture_rows();
    let body = batch::request_body(&batch::selected(&rows, &BTreeSet::new()), "ok");
    let req: BatchRequest = wire::decode(&body).expect("要求の電文");
    let got: Vec<(&str, &str, Option<&String>)> = req
        .items
        .iter()
        .map(|i| {
            (
                i.question.as_str(),
                i.seen_digest.as_str(),
                i.verbatim.as_ref(),
            )
        })
        .collect();
    assert_eq!(got, vec![("qa.10", "fedcba9876543210", None)]);
    // A-1 の印を持つ行を渡されても入れない。
    let all: Vec<&batch::Row> = rows.iter().collect();
    let req: BatchRequest = wire::decode(&batch::request_body(&all, "ok")).expect("要求の電文");
    assert_eq!(req.items.len(), 1);
    assert_eq!(req.items[0].question, bead("qa.10"));
}

fn batch_reply(written: usize, refused: usize) -> BatchResponse {
    let mut items = Vec::new();
    for i in 0..written {
        items.push(BatchItemResult {
            question: bead(&format!("w.{i}")),
            outcome: ItemOutcome::Written {
                ruling: ruling(&format!("w.{i}:20260927T1105Z-1")),
            },
        });
    }
    items.push(BatchItemResult {
        question: bead("s.1"),
        outcome: ItemOutcome::Skipped {
            ruling: ruling("s.1:20260927T1000Z-1"),
        },
    });
    for i in 0..refused {
        items.push(BatchItemResult {
            question: bead(&format!("r.{i}")),
            outcome: ItemOutcome::Refused {
                reason: Refusal::StaleVersion,
            },
        });
    }
    BatchResponse {
        batch: ruling("batch:20260927T1105Z-1"),
        items,
    }
}

/// (6) 束の応答の出し方: 200・409・502 の束の応答・ほかの 4xx と 5xx・届かない。
#[test]
fn batchpanel_batch_outcome() {
    let ok = wire::encode(&batch_reply(2, 0)).expect("電文");
    let recorded = batch::outcome(Some((200, &ok)), &[]);
    assert_eq!(
        recorded,
        batch::Outcome::Recorded {
            batch: ruling("batch:20260927T1105Z-1"),
            written: 2
        }
    );
    assert_eq!(
        recorded.line(),
        format!(
            "{} batch:20260927T1105Z-1 · {} 2",
            ask::RECORDED,
            batch::WRITTEN
        )
    );
    assert!(!recorded.keeps_text());
    assert!(recorded.reloads());

    let stale = batch::outcome(Some((409, "{\"reason\":\"stale-version\"}")), &[]);
    assert_eq!(stale, batch::Outcome::Stale);
    assert_eq!(stale.line(), ask::STALE);
    assert!(stale.keeps_text());
    assert!(stale.reloads());

    let partial_body = wire::encode(&batch_reply(1, 2)).expect("電文");
    let partial = batch::outcome(Some((502, &partial_body)), &[]);
    assert_eq!(
        partial,
        batch::Outcome::Partial {
            written: 1,
            left: vec![]
        }
    );
    assert!(partial.line().starts_with(ask::REFUSED));
    assert!(partial.line().ends_with(&format!("{} 1", batch::WRITTEN)));
    assert!(partial.keeps_text());
    assert!(partial.reloads());
    refused_batches();
}

/// 束のほかの 4xx と 5xx・読めない 502・届かない・読めない 200 は理由の字で、読み直さない。
fn refused_batches() {
    let a1 = wire::encode(&RefusalResponse {
        reason: Refusal::A1InBatch,
    })
    .expect("電文");
    let refused = batch::outcome(Some((400, &a1)), &[]);
    assert_eq!(
        refused,
        batch::Outcome::Refused(format!("{}（400）", ask::refusal_text(Refusal::A1InBatch)))
    );
    let text = batch::outcome(Some((500, "boom")), &[]);
    assert_eq!(text, batch::Outcome::Refused("状態 500: boom".to_string()));
    // 502 でも束の応答として読めなければ、ほかの 5xx と同じ。
    let gateway = batch::outcome(Some((502, "bad gateway")), &[]);
    assert_eq!(
        gateway,
        batch::Outcome::Refused("状態 502: bad gateway".to_string())
    );
    let lost = batch::outcome(None, &[]);
    assert_eq!(lost, batch::Outcome::Refused(ask::NOT_REACHED.to_string()));
    let bad = batch::outcome(Some((200, "not json")), &[]);
    assert!(matches!(bad, batch::Outcome::Refused(_)));
    for o in [&refused, &text, &gateway, &lost, &bad] {
        assert!(o.keeps_text(), "{o:?}");
        assert!(!o.reloads(), "{o:?}");
        assert!(o.line().starts_with(ask::REFUSED), "{o:?}");
    }
    assert_eq!(batch::written(&batch_reply(3, 1)), 3);
}

/// (8) 指示を送る button と要求の本文（範囲はつねに all）と応答の出し方。
#[test]
fn batchpanel_policy_send_and_outcome() {
    assert!(policy::can_send("方針", false));
    assert!(!policy::can_send(" \n　", false));
    assert!(!policy::can_send("方針", true));

    let words = "  全体に: 小さく刻む  ";
    let req: PolicyRequest =
        wire::decode(&policy::request_body(words)).expect("要求の電文");
    assert_eq!(
        req,
        PolicyRequest {
            scope: "all".to_string(),
            verbatim: words.to_string(),
        }
    );
    let req: PolicyRequest = wire::decode(&policy::request_body("x")).expect("要求の電文");
    assert_eq!(req.scope, "all");

    let ok = wire::encode(&PolicyResponse {
        policy: ruling("policy:20260927T1105Z-1"),
        recorded_at: 1_790_494_740,
    })
    .expect("電文");
    let recorded = policy::outcome(Some((200, &ok)));
    assert_eq!(
        recorded,
        policy::Outcome::Recorded {
            policy: ruling("policy:20260927T1105Z-1")
        }
    );
    assert_eq!(
        recorded.line(),
        format!("{} policy:20260927T1105Z-1", ask::RECORDED)
    );
    assert!(!recorded.keeps_text());
    assert!(!recorded.reloads(), "問いは閉じないので読み直さない");

    let scope = policy::outcome(Some((400, "scope")));
    assert_eq!(scope, policy::Outcome::NotCurrent);
    assert_eq!(scope.line(), policy::NOT_CURRENT);
    assert_eq!(policy::NOT_CURRENT, "範囲が今の問いでない");
    assert!(scope.keeps_text());
    assert!(scope.reloads());
    policy_refusals();
}

/// 指示の 409・ほかの 4xx と 5xx・届かない・読めない 200 の出し方（束と同じ）。
fn policy_refusals() {
    // ほかは束と同じ。
    let stale = policy::outcome(Some((409, "")));
    assert_eq!(stale, policy::Outcome::Stale);
    assert!(stale.keeps_text() && stale.reloads());
    let empty = wire::encode(&RefusalResponse {
        reason: Refusal::EmptyVerbatim,
    })
    .expect("電文");
    let refused = policy::outcome(Some((400, &empty)));
    assert_eq!(
        refused,
        policy::Outcome::Refused(format!(
            "{}（400）",
            ask::refusal_text(Refusal::EmptyVerbatim)
        ))
    );
    let other = policy::outcome(Some((503, "busy")));
    assert_eq!(
        other,
        policy::Outcome::Refused("状態 503: busy".to_string())
    );
    let lost = policy::outcome(None);
    assert_eq!(lost, policy::Outcome::Refused(ask::NOT_REACHED.to_string()));
    let bad = policy::outcome(Some((200, "{}")));
    assert!(matches!(bad, policy::Outcome::Refused(_)));
    for o in [&refused, &other, &lost, &bad] {
        assert!(o.keeps_text(), "{o:?}");
        assert!(!o.reloads(), "{o:?}");
        assert!(o.line().starts_with(ask::REFUSED), "{o:?}");
    }
}

/// (9) 2 つの欄は着地済みの鍵の判定の関数を使う: 変換の途中の Enter は何もしない・Ctrl か Meta と Enter で送る。
#[test]
fn batchpanel_key_action() {
    assert_eq!(ask::key_action(true, true, "Enter"), KeyAction::Nothing);
    assert_eq!(ask::key_action(true, false, "Enter"), KeyAction::Nothing);
    assert_eq!(ask::key_action(false, true, "Enter"), KeyAction::Send);
    assert_eq!(ask::key_action(false, false, "Enter"), KeyAction::Hold);
    assert!(
        read("src/project/batch.rs")
            .contains("use crate::project::ask::{self, KeyAction, key_action};"),
        "batch.rs が鍵の判定の関数を use しない"
    );
    for module in ["src/project/batch.rs", "src/project/policy.rs"] {
        let text = read(module);
        assert!(
            text.contains("key_action(composing,"),
            "{module} が鍵の判定の関数を使わない"
        );
        assert!(
            text.contains("ev.is_composing() || ev.key_code() == 229"),
            "{module} が変換の途中を見ない"
        );
        assert!(
            !text.contains("fn key_action"),
            "{module} が鍵の判定を持ち直す"
        );
    }
}

/// (10) 口の path の定数は batch と policy の module に 1 本ずつ・batch は問いの一覧を ask の module の口から読む
/// （policy は問いの一覧を読まない）。
#[test]
fn batchpanel_paths() {
    assert_eq!(batch::PATH, "/api/batch");
    assert_eq!(policy::PATH, "/api/policy");
    assert!(
        read("src/project/batch.rs").contains("crate::net::read(ask::PATH)"),
        "batch.rs が問いの一覧の口を読まない"
    );
    for (module, path) in [
        ("src/project/batch.rs", batch::PATH),
        ("src/project/policy.rs", policy::PATH),
    ] {
        let text = read(module);
        assert_eq!(text.matches(&format!("\"{path}\"")).count(), 1, "{module}");
        assert!(
            text.contains("crate::net::post(PATH, body)"),
            "{module} が自分の口へ送らない"
        );
    }
}

/// (11) 問いの一覧の口が読めない・まだ読んでいない・本文が電文として読めないときは、まとめて承認は測れていないと理由の 1 行
/// （全体への指示は問いの一覧を読まない）。
#[test]
fn batchpanel_unmeasured() {
    let unknown = Fetched::Body(
        wire::encode(&QuestionList {
            cards: Reading::Unknown,
            answerable: true,
        })
        .expect("電文"),
    );
    for fetched in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("not json".to_string()),
        Fetched::Body("{\"cards\": 3}".to_string()),
        unknown,
    ] {
        let Body::Unmeasured(reason) = batch::body(&fetched) else {
            panic!("まとめて承認が {fetched:?} で測れていないでない");
        };
        assert!(!reason.trim().is_empty() && !reason.contains('\n'));
    }
    assert_eq!(batch::body(&Fetched::NotRead), Body::Unmeasured(NOT_READ));
    assert_eq!(batch::body(&Fetched::Failed), Body::Unmeasured(ask::REASON));
    // 読めて 0 本は測れていないでない。
    assert_eq!(batch::body(&list_of(vec![])), Body::Empty(ask::EMPTY));
    let text = read("src/project/batch.rs") + &read("src/project/policy.rs");
    assert!(text.contains("unmeasured(reason)"));
}

/// (12) 2 つの module の code に画面の外の保存の口の名が無い。
#[test]
fn batchpanel_no_offscreen_storage_names() {
    for module in ["src/project/batch.rs", "src/project/policy.rs"] {
        let text = read(module).to_lowercase();
        for name in ["localStorage", "sessionStorage", "cookie"] {
            assert!(
                !text.contains(&name.to_lowercase()),
                "{module} に {name} の字が在る"
            );
        }
    }
}

/// この便が使う語の鍵は語の辞書に在る。
#[test]
fn batchpanel_vocab_keys() {
    for key in [
        "batch",
        "policy",
        "own_words",
        "touches",
        "a1",
        "st_unknown",
    ] {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }
}

/// (13) 足す外の依存は 0 本（面の crate の依存の節は着地のまま）。
#[test]
fn batchpanel_no_new_dependencies() {
    let manifest = read("Cargo.toml");
    let mut inside = false;
    let mut names = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if let Some(table) = line.strip_prefix('[') {
            inside = table.trim_end_matches(']').ends_with("dependencies");
        } else if inside
            && !line.starts_with('#')
            && let Some((k, _)) = line.split_once('=')
        {
            let k = k.trim();
            names.push(k.split('.').next().unwrap_or(k).to_string());
        }
    }
    names.sort();
    names.dedup();
    assert_eq!(
        names,
        vec![
            "leptos",
            "tsuzuri-boundary",
            "tsuzuri-contract",
            "wasm-bindgen-futures",
            "web-sys"
        ]
    );
}
