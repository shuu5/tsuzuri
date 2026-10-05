//! 便 g-ask の歯: 問いの頁の枠と header の数の印・card の部分の並び（配置の表）・card の字（fixture の 2 本）・
//! 番号とつながりの段・送る button の状態・鍵の判定・要求の本文・応答から card の状態・これまでの決定・
//! 0 件と測れていないの区別・画面の外の保存の口の名が code に無い・足す外の依存は 0 本。
#![cfg(test)]

use crate::common::{bead, ledger_questions, question_list, read};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::LedgerList;
use tsuzuri_contract::question::QuestionList;
use tsuzuri_contract::surface::{
    Refusal, RefusalResponse, RulingId, RulingRequest, RulingResponse,
};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::{Body, NOT_READ, ask, askpage};
use tsuzuri_surface::view::{Fetched, clock};
use tsuzuri_surface::vocab::vocab;

fn filled_cards() -> Vec<ask::Card> {
    match ask::body(&question_list()) {
        Body::Filled(cards) => cards,
        other => panic!("fixture の card が中身にならない: {other:?}"),
    }
}




/// card の部分の並びは 題・概要・理由・推奨・答えの欄・つながり で、つながりの段だけが畳める段（最初は閉じる）。
#[test]
fn askcard_layout_order() {
    use ask::Part;
    let parts: Vec<Part> = ask::LAYOUT.iter().map(|s| s.part).collect();
    assert_eq!(
        parts,
        vec![
            Part::Head,
            Part::Summary,
            Part::Reason,
            Part::Recommend,
            Part::Answer,
            Part::Around
        ]
    );
    let classes: Vec<&str> = ask::LAYOUT.iter().map(|s| s.class).collect();
    assert_eq!(
        classes,
        vec!["head", "qsum", "qreason", "rec", "answer", "nb-d"]
    );
    let folds: Vec<Option<bool>> = ask::LAYOUT.iter().map(|s| s.open).collect();
    assert_eq!(folds, vec![None, None, None, None, None, Some(false)]);
    let keys: Vec<Option<&str>> = ask::LAYOUT.iter().map(|s| s.key).collect();
    assert_eq!(
        keys,
        vec![
            None,
            None,
            Some("reason"),
            Some("recommend"),
            Some("own_words"),
            Some("around")
        ]
    );
}

/// fixture の 2 本の card の字（番号は電文の順に 1 から・題は 36 字で切る・A-1 の印は印を持つ card だけ）。
#[test]
fn askcard_fixture_cards_text() {
    let cards = filled_cards();
    assert_eq!(cards.len(), 2);
    let first = &cards[0];
    assert_eq!(first.number, 1);
    assert_eq!(first.id.as_str(), "qa.2");
    assert_eq!(first.title, "質問の頁の答えの欄は 1 問に 1 つでよいか");
    assert!(first.a1);
    assert_eq!(first.posted_at, 1_790_488_800);
    assert_eq!(
        (first.plain.as_deref(), first.eng.as_deref()),
        (
            Some("答えを書く欄を質問ごとに 1 つ置く"),
            Some("qcard ごとに textarea と送る button を 1 組")
        )
    );
    assert_eq!(
        first.reason,
        "束の承認は後の便が足すので、先に 1 問ずつ答えられるようにしたい"
    );
    assert_eq!(first.recommend, "1 問に 1 つの欄にする");
    assert_eq!(first.digest, "0123456789abcdef");
    second_card(&cards, first);
}

/// 2 本目の card の字（題は 36 字で切る・概要の無い card は要約なし）と、置かれてからの経過。
fn second_card(cards: &[ask::Card], first: &ask::Card) {
    let second = &cards[1];
    assert_eq!(second.number, 2);
    assert_eq!(second.id.as_str(), "qa.10");
    assert_eq!(
        second.title,
        "これまでの決定の段を開いたままにするか閉じておくかをどちらにするか決め…"
    );
    assert_eq!(second.title.chars().count(), 36);
    assert!(!second.a1);
    assert_eq!((second.plain.as_deref(), second.eng.as_deref()), (None, None));
    assert_eq!(
        ask::summary(None, None, Mode::Beginner),
        ask::SumLine {
            class: "ln plain muted",
            key: None,
            text: ask::NO_SUMMARY.to_string(),
            eng: false,
            marked: false,
        }
    );
    assert_eq!(ask::NO_SUMMARY, "要約なし");
    // 理由はつねに出す（在れば字・無ければ空）・推奨の無い card は空の字。
    assert_eq!(second.reason, "見本では閉じている");
    assert_eq!(second.recommend, "");
    assert_eq!(second.digest, "fedcba9876543210");
    card_ages(first);
}

/// 1 本目の card の置かれた時刻からの経過の字。
fn card_ages(first: &ask::Card) {
    // 置かれてからの経過（見本の durMs）。
    assert_eq!(ask::age(1_790_488_800 + 59 * 60, first.posted_at), "59m");
    assert_eq!(ask::age(1_790_488_800 + 2 * 3600, first.posted_at), "2h");
    assert_eq!(
        ask::age(1_790_488_800 + 2 * 3600 + 5 * 60, first.posted_at),
        "2h05"
    );
    assert_eq!(ask::age(1_790_488_800 + 72 * 3600, first.posted_at), "3d");
    assert_eq!(ask::age(0, first.posted_at), "0m");
}

/// 概要の片方だけ無い card は、表示の型の側が無ければもう一方の 1 行を印つきで出す（行 g-ask-mode で 2 行と「―」をやめた）。
#[test]
fn askcard_summary_one_side_missing() {
    let only_plain = ask::summary(Some("やさしい説明"), None, Mode::Expert);
    assert_eq!(
        (only_plain.class, only_plain.text.as_str(), only_plain.marked),
        ("ln plain marked", "やさしい説明", true)
    );
    let only_eng = ask::summary(None, Some("statements[0]"), Mode::Beginner);
    assert_eq!(
        (only_eng.class, only_eng.text.as_str(), only_eng.marked),
        ("ln eng marked", "statements[0]", true)
    );
    assert_eq!(ask::summary(None, None, Mode::Expert).key, None);
}

/// 番号は電文の順（古い順）のまま 1 から・つながりの見出しの数と中の id は card の touches と同じ。
#[test]
fn askcard_numbers_and_touches() {
    let text = read("../../tests/fixtures/surface/question-list.json");
    let list: QuestionList = wire::decode(&text).expect("fixture の電文");
    let Reading::Known(wire_cards) = list.cards else {
        panic!("fixture の card が読めない");
    };
    let cards = filled_cards();
    for (i, (c, w)) in cards.iter().zip(&wire_cards).enumerate() {
        assert_eq!(c.number, i + 1);
        assert_eq!(c.id, w.id);
        assert_eq!(c.touches, w.touches);
        assert_eq!(c.touches.len(), w.touches.len());
    }
    assert_eq!(cards[0].touches, vec!["qa.1", "ADR-7", "R-20"]);
    assert!(cards[1].touches.is_empty());
    // 電文の順が逆なら番号も逆（面は並べ替えない）。
    let reversed = QuestionList {
        cards: Reading::Known(wire_cards.iter().rev().cloned().collect()),
        answerable: true,
    };
    let body = Fetched::Body(wire::encode(&reversed).expect("電文"));
    let Body::Filled(cards) = ask::body(&body) else {
        panic!("中身にならない");
    };
    let ids: Vec<&str> = cards.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, vec!["qa.10", "qa.2"]);
    assert_eq!(cards[0].number, 1);
}

/// 送る button は答えの欄が空白だけのときと送っている間は押せない。
#[test]
fn askcard_send_button_state() {
    assert!(ask::can_send("はい", false));
    assert!(ask::can_send("  a  ", false));
    assert!(!ask::can_send("", false));
    assert!(!ask::can_send(" \n\t　", false));
    assert!(!ask::can_send("はい", true));
    assert!(!ask::can_send("", true));
}

/// 鍵の判定: 変換の途中は何もしない・Ctrl か Meta と Enter は送る・Enter だけは送らない。
#[test]
fn askcard_key_action() {
    use ask::KeyAction::{Hold, Nothing, Send};
    assert_eq!(ask::key_action(true, false, "Enter"), Nothing);
    assert_eq!(ask::key_action(true, true, "Enter"), Nothing);
    assert_eq!(ask::key_action(false, true, "Enter"), Send);
    assert_eq!(ask::key_action(false, false, "Enter"), Hold);
    assert_eq!(ask::key_action(false, true, "a"), Hold);
    assert_eq!(ask::key_action(false, false, "a"), Hold);
}

/// 要求の本文は RulingRequest の字で、見た版の要約値は card の digest・逐語は答えの欄の字のまま。
#[test]
fn askcard_request_body() {
    let cards = filled_cards();
    let words = "  1 問に 1 つでよい。\n理由は後で  ";
    let body = ask::request_body(&cards[0], words);
    let req: RulingRequest = wire::decode(&body).expect("要求の電文");
    assert_eq!(
        req,
        RulingRequest {
            question: bead("qa.2"),
            seen_digest: "0123456789abcdef".to_string(),
            verbatim: words.to_string(),
        }
    );
    assert_eq!(req.seen_digest, cards[0].digest);
    assert_eq!(ask::RULING_PATH, "/api/ruling");
    assert_eq!(ask::PATH, "/api/questions");
}

/// 応答から card の状態: 200 は記録した id と時刻・409 は読み直し・ほかと届かないは理由の字。
#[test]
fn askcard_outcome_from_reply() {
    let ruling = RulingId::new("qa.2:20260927T1105Z-1").expect("記帳 id");
    let ok = wire::encode(&RulingResponse {
        ruling: ruling.clone(),
        recorded_at: 1_790_494_740,
    })
    .expect("電文");
    let recorded = ask::outcome(Some((200, &ok)));
    assert_eq!(
        recorded,
        ask::Outcome::Recorded {
            ruling,
            at: 1_790_494_740
        }
    );
    assert_eq!(
        recorded.line(),
        format!(
            "{} qa.2:20260927T1105Z-1 · {}",
            ask::RECORDED,
            clock(1_790_494_740)
        )
    );
    assert!(!recorded.keeps_text());
    assert!(!recorded.answer_open());
    assert!(recorded.reloads());

    let stale = wire::encode(&RefusalResponse {
        reason: Refusal::StaleVersion,
    })
    .expect("電文");
    let conflict = ask::outcome(Some((409, &stale)));
    assert_eq!(conflict, ask::Outcome::Stale);
    assert_eq!(conflict.line(), "質問が更新された");
    assert!(conflict.keeps_text());
    assert!(conflict.answer_open());
    assert!(conflict.reloads());
    refused_replies();
}

/// 400・404・503・届かない・読めない 200 は理由の字で、断りの理由は 6 値とも空でない字。
fn refused_replies() {
    let empty = wire::encode(&RefusalResponse {
        reason: Refusal::EmptyVerbatim,
    })
    .expect("電文");
    let refused = ask::outcome(Some((400, &empty)));
    assert_eq!(
        refused,
        ask::Outcome::Refused(format!(
            "{}（400）",
            ask::refusal_text(Refusal::EmptyVerbatim)
        ))
    );
    let unknown = wire::encode(&RefusalResponse {
        reason: Refusal::UnknownQuestion,
    })
    .expect("電文");
    assert_eq!(
        ask::outcome(Some((404, &unknown))),
        ask::Outcome::Refused("台帳に無い質問（404）".to_string())
    );
    let server = ask::outcome(Some((503, "busy")));
    assert_eq!(server, ask::Outcome::Refused("状態 503".to_string()));
    let lost = ask::outcome(None);
    assert_eq!(lost, ask::Outcome::Refused(ask::NOT_REACHED.to_string()));
    for o in [&refused, &server, &lost] {
        assert!(o.keeps_text(), "{o:?}");
        assert!(o.answer_open(), "{o:?}");
        assert!(!o.reloads(), "{o:?}");
        assert!(o.line().starts_with(ask::REFUSED), "{o:?}");
    }
    // 200 でも応答が読めなければ記録したと書かない。
    let bad = ask::outcome(Some((200, "not json")));
    assert!(matches!(bad, ask::Outcome::Refused(_)));
    assert!(bad.keeps_text());
    // 断りの理由は 6 値とも空でない字。
    for r in [
        Refusal::EmptyVerbatim,
        Refusal::UnknownQuestion,
        Refusal::StaleVersion,
        Refusal::A1NeedsOwnVerbatim,
        Refusal::A1InBatch,
        Refusal::EmptyBatch,
    ] {
        assert!(!ask::refusal_text(r).is_empty(), "{r:?}");
    }
}

/// これまでの決定: label intake:question を持つ closed の 3 本だけを id の数の順（2・9・10）に・件数 3・最初は閉じる。
#[test]
fn askcard_rulings_from_ledger_fixture() {
    let fetched = ledger_questions();
    let rows = askpage::rows(&fetched).expect("fixture の行");
    assert_eq!(rows.len(), 5);
    let order: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(order, vec!["lq.2", "lq.10", "lq.9", "lq.3", "lq.4"]);
    let done = askpage::rulings(&rows);
    let ids: Vec<&str> = done.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, vec!["lq.2", "lq.9", "lq.10"]);
    assert_eq!(askpage::count(&fetched), Reading::Known(3));
    let Body::Filled(items) = askpage::body(&fetched) else {
        panic!("これまでの決定が中身を出さない");
    };
    let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, vec!["lq.2", "lq.9", "lq.10"]);
    const { assert!(!askpage::OPEN, "これまでの決定の段は最初は閉じている") };
    assert_eq!(askpage::BLOCK.id, "hist");
    assert_eq!(askpage::BLOCK.heading, "rulings");
    assert_eq!(askpage::BLOCK.class, "fold panel");
}

/// 読めて 0 本は 0 件の 1 行・まだ読んでいない・読めない・電文が読めない・まだ分からないは測れていない。
#[test]
fn askcard_empty_is_not_unmeasured() {
    let empty_cards = Fetched::Body(
        wire::encode(&QuestionList {
            cards: Reading::Known(vec![]),
            answerable: true,
        })
        .expect("電文"),
    );
    assert_eq!(ask::body(&empty_cards), Body::Empty(ask::EMPTY));
    let empty_rows = Fetched::Body(
        wire::encode(&LedgerList {
            rows: Reading::Known(vec![]),
        })
        .expect("電文"),
    );
    assert_eq!(askpage::body(&empty_rows), Body::Empty(askpage::EMPTY));
    assert_eq!(askpage::count(&empty_rows), Reading::Known(0));
    unmeasured_reads();
}

/// まだ読んでいない・読めない・電文が読めない・まだ分からないは card も決定も測れていない。
fn unmeasured_reads() {
    let unknown_cards = Fetched::Body(
        wire::encode(&QuestionList {
            cards: Reading::Unknown,
            answerable: true,
        })
        .expect("電文"),
    );
    let unknown_rows = Fetched::Body(
        wire::encode(&LedgerList {
            rows: Reading::Unknown,
        })
        .expect("電文"),
    );
    let broken = Fetched::Body("{\"cards\": 3}".to_string());
    for fetched in [
        Fetched::NotRead,
        Fetched::Failed,
        broken.clone(),
        Fetched::Body("not json".to_string()),
        unknown_cards,
    ] {
        let Body::Unmeasured(reason) = ask::body(&fetched) else {
            panic!("問いの card が {fetched:?} で測れていないでない");
        };
        assert!(!reason.trim().is_empty() && !reason.contains('\n'));
    }
    for fetched in [
        Fetched::NotRead,
        Fetched::Failed,
        broken,
        Fetched::Body("not json".to_string()),
        unknown_rows,
    ] {
        let Body::Unmeasured(reason) = askpage::body(&fetched) else {
            panic!("これまでの決定が {fetched:?} で測れていないでない");
        };
        assert!(!reason.trim().is_empty() && !reason.contains('\n'));
        assert_eq!(askpage::count(&fetched), Reading::Unknown);
    }
    assert_eq!(ask::body(&Fetched::NotRead), Body::Unmeasured(NOT_READ));
    assert_eq!(ask::body(&Fetched::Failed), Body::Unmeasured(ask::REASON));
    // 問いの一覧の電文は台帳の一覧の電文と取り違えない。
    assert!(matches!(ask::body(&ledger_questions()), Body::Unmeasured(_)));
    assert!(matches!(
        askpage::body(&question_list()),
        Body::Unmeasured(_)
    ));
}

/// 問いの頁の 2 つの module の code に画面の外の保存の口の名が無い。
#[test]
fn askcard_no_offscreen_storage_names() {
    for module in ["src/project/ask.rs", "src/project/askpage.rs"] {
        let text = read(module);
        for name in ["localStorage", "sessionStorage", "cookie"] {
            assert!(
                !text.to_lowercase().contains(&name.to_lowercase()),
                "{module} に {name} の字が在る"
            );
        }
    }
}

/// この便が使う語の鍵は 12 個とも語の辞書に在る。
#[test]
fn askcard_vocab_keys() {
    for key in [
        "questions",
        "ask_open",
        "rulings",
        "reason",
        "recommend",
        "own_words",
        "ruling",
        "around",
        "touches",
        "summary_plain",
        "summary_eng",
        "a1",
    ] {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }
}

/// manifest の依存の節の名（target ごとの節も・`[dependencies.x]` の節も）。
fn dependency_names(text: &str) -> Vec<String> {
    let mut inside = false;
    let mut names = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(table) = line.strip_prefix('[') {
            let table = table.trim_end_matches(']');
            inside = table.ends_with("dependencies");
            if let Some((head, name)) = table.rsplit_once('.')
                && head.ends_with("dependencies")
            {
                names.push(name.to_string());
            }
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
    names
}

/// 足す外の依存は 0 本（面の crate の依存の名は便 g-parts の着地のまま）。
#[test]
fn askcard_no_new_dependencies() {
    let names = dependency_names(&read("Cargo.toml"));
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

/// fixture は 2 つとも 5000 byte 以下。
#[test]
fn askcard_fixtures_are_small() {
    for rel in [
        "../../tests/fixtures/surface/question-list.json",
        "../../tests/fixtures/surface/ledger-questions.json",
    ] {
        let len = read(rel).len();
        assert!(len <= 5000, "{rel} が {len} byte");
    }
}
