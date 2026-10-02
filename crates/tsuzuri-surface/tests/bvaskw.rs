//! 行 g-ask-win の歯: 質問の窓（判断の記録 ADR-27 決定 (4)・見本 board-v2 の qModal と qsend と qlater）の 1 問ずつの選び方と、
//! 送った後の次の問いと「あとで」、全部に答えた時の局面と閉じる待ち、題と点の class、下の段の並びと block の本文の分け、
//! 語の鍵と stylesheet の規則、窓の DOM（wasm の枝）の配線の字、問いの一覧や台帳が読めない時に問いが無いと見せないこと
//! （憲法 P-7.1・P-7.2）。
//! fixture は tests/fixtures/surface/question-list.json（読むだけ）。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::{AllQuestions, ProjectQuestions, QuestionList};
use tsuzuri_contract::wire;
use tsuzuri_surface::askwin::{
    ADVANCE_MS, DONE_KEY, DONE_MS, FOLDS, LATER_KEY, Phase, UNKNOWN_COUNT, current, dot_class,
    later, left, others_count, others_title, own_ids, phase, remaining, title, unread,
};
use tsuzuri_surface::project::{NOT_READ, ask};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn ids(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_string()).collect()
}

fn set(v: &[&str]) -> BTreeSet<String> {
    v.iter().map(|s| (*s).to_string()).collect()
}

/// 1 問ずつ: 選んだ問いが一覧に在り答えていなければそれ、ほかは一覧の頭から最初の答えていない問い。
#[test]
fn bvaskw_one_at_a_time_in_order() {
    let all = ids(&["q.1", "q.2", "q.3"]);
    assert_eq!(current(&all, None, &set(&[])), Some("q.1"));
    assert_eq!(current(&all, Some("q.3"), &set(&[])), Some("q.3"));
    assert_eq!(current(&all, Some("q.3"), &set(&["q.3"])), Some("q.1"));
    assert_eq!(current(&all, Some("q.9"), &set(&["q.1"])), Some("q.2"));
    assert_eq!(current(&all, None, &set(&["q.1", "q.2", "q.3"])), None);
    assert_eq!(current(&[], Some("q.1"), &set(&[])), None);
}

/// 送って記録された問いは 650 ms の後に答えた問いに数え、次は一覧の頭から最初の答えていない問い（今の後ろでなく頭から）。
#[test]
fn bvaskw_advance_after_send() {
    assert_eq!(ADVANCE_MS, 650);
    let all = ids(&["q.1", "q.2", "q.3"]);
    // q.2 を選んで答えた後は、q.3 でなく頭の q.1 へ移る。
    assert_eq!(current(&all, Some("q.2"), &set(&["q.2"])), Some("q.1"));
    // 一覧を読み直して答えた問いが消えても、残りの頭へ移る。
    let reloaded = ids(&["q.1", "q.3"]);
    assert_eq!(
        current(&reloaded, Some("q.2"), &set(&["q.2", "q.1"])),
        Some("q.3")
    );
    let src = read("src/project/ask.rs");
    let one = &src[src.find("pub fn one_view(").expect("one_view")..];
    let one = &one[..one.find("\n    }\n").expect("one_view の終わり")];
    assert!(
        one.contains("matches!(o, Some(Outcome::Recorded { .. }))"),
        "{one}"
    );
    assert!(one.contains("recorded.run(id.clone())"), "{one}");
}

/// あとで: 今の問いの後ろへ巡って最初の答えていない問い・ほかに無ければ None（窓を閉じる）。
#[test]
fn bvaskw_later_wraps() {
    let all = ids(&["q.1", "q.2", "q.3"]);
    assert_eq!(later(&all, "q.1", &set(&[])), Some("q.2"));
    assert_eq!(later(&all, "q.3", &set(&[])), Some("q.1"));
    assert_eq!(later(&all, "q.1", &set(&["q.2"])), Some("q.3"));
    assert_eq!(later(&all, "q.2", &set(&["q.1", "q.3"])), None);
    assert_eq!(later(&all, "q.9", &set(&["q.1"])), Some("q.2"));
    assert_eq!(later(&[], "q.1", &set(&[])), None);
}

/// 局面: 答えていない問いが在れば Walk・窓で答えた後に残りが無ければ Done（1200 ms の後に閉じる）・初めから無ければ Empty。
#[test]
fn bvaskw_close_when_none_left() {
    assert_eq!(DONE_MS, 1200);
    let all = ids(&["q.1", "q.2"]);
    assert_eq!(phase(None, &all, &set(&[])), Phase::Walk);
    assert_eq!(phase(None, &all, &set(&["q.1"])), Phase::Walk);
    assert_eq!(phase(None, &all, &set(&["q.1", "q.2"])), Phase::Done);
    assert_eq!(phase(None, &[], &set(&["q.1"])), Phase::Done);
    assert_eq!(phase(None, &[], &set(&[])), Phase::Empty);
    let src = read("src/askwin.rs");
    let dom = &src[src.find("mod dom {").expect("wasm の枝")..];
    for needle in [
        "if state.get() == Phase::Done {",
        "if ctx.top() == Some(Win::Ask) && state.get_untracked() == Phase::Done {",
        "Duration::from_millis(DONE_MS)",
        "None => ctx.close(),",
    ] {
        assert!(dom.contains(needle), "wasm の枝に {needle} が無い");
    }
}

/// 題は「答えを待つ質問 <残り>」で、点は今の問いが cur・答えた問いが ok。1 問ずつ出すのはこの project の問いだけ。
#[test]
fn bvaskw_dots_and_title() {
    let all = ids(&["q.1", "q.2", "q.3"]);
    let done = set(&["q.1"]);
    assert_eq!(left(&all, &done), 2);
    assert_eq!(title(Some(2)), "答えを待つ質問 2");
    let classes: Vec<&str> = all
        .iter()
        .map(|id| dot_class(id, Some("q.2"), &done))
        .collect();
    assert_eq!(classes, ["qd ok", "qd cur", "qd"]);
    assert_eq!(dot_class("q.1", Some("q.1"), &done), "qd cur ok");
    let own: QuestionList =
        wire::decode(&read("../../tests/fixtures/surface/question-list.json")).expect("問いの一覧");
    let Reading::Known(cards) = own.cards.clone() else {
        panic!("fixture の問い");
    };
    let mut other = cards[0].clone();
    other.id = BeadId::new("ox.1").expect("id");
    let all = AllQuestions {
        own,
        others: vec![ProjectQuestions {
            project: "other".to_string(),
            answerable: false,
            cards: Reading::Known(vec![other]),
        }],
    };
    let listed = ask::listed(&Fetched::Body(wire::encode(&all).expect("電文")));
    assert_eq!(listed.len(), 3);
    assert_eq!(own_ids(&listed), ["qa.2", "qa.10"]);
}

/// 下の段は ほかの project の質問・まとめて承認・全体への指示 の順で、まとめて承認と全体への指示は block の本文（inner）を描き、
/// 頁の block の view も同じ inner を描く。
#[test]
fn bvaskw_batch_policy_below() {
    assert_eq!(FOLDS, ["ask_others", "batch", "policy"]);
    let src = read("src/askwin.rs");
    let folds = &src[src.find("fn folds(").expect("folds")..];
    let b = folds.find("{batch::inner()}").expect("まとめて承認の段");
    let p = folds.find("{policy::inner()}").expect("全体への指示の段");
    let o = folds.find("{rows}").expect("ほかの project の段");
    assert!(o < b && b < p, "下の段の順");
    for file in ["src/project/batch.rs", "src/project/policy.rs"] {
        let text = read(file);
        assert!(
            text.contains("pub fn inner() -> leptos::prelude::AnyView {"),
            "{file}"
        );
        assert!(
            text.contains("section(BLOCK, ().into_any(), inner())"),
            "{file}"
        );
    }
    assert!(src.contains("body: view! { {ask::late_view()}{walk}{folds(w, focus)} }.into_any(),"));
}

/// 語の鍵の語と、質問の窓の規則が stylesheet の質問の塊（つながりの塊の前）に在ること。
#[test]
fn bvaskw_words_and_rules() {
    let v = vocab();
    for (key, want) in [
        (DONE_KEY, "全部答えました ✓"),
        (LATER_KEY, "あとで"),
        ("ask_others", "ほかの project の質問"),
        ("ask_open", "答えを待つ質問"),
        ("batch", "まとめて承認"),
        ("policy", "全体への指示"),
    ] {
        assert_eq!(v.term(key).map(|t| t.label.as_str()), Some(want), "{key}");
    }
    let css = read("style.css");
    let ask = css
        .find("/* ---------- 質問 ---------- */")
        .expect("質問の塊");
    let next = css
        .find("/* ---------- つながり（図） ---------- */")
        .expect("つながりの塊");
    for rule in [
        ".qdots .qd {",
        ".qdots .qd.cur {",
        ".qdots .qd.ok {",
        ".alldone {",
        ".qbtns .later {",
        ".folds {",
    ] {
        let i = css
            .find(rule)
            .unwrap_or_else(|| panic!("stylesheet に {rule} が無い"));
        assert!(ask < i && i < next, "{rule} は質問の塊の中");
    }
}

/// 窓の DOM（wasm の枝）の配線の字: 帯の窓の Ask は質問の窓を組み、1 問は質問の block と同じ card で描き、
/// 記録された問いを待ってから答えた問いに数え、点を押すとその問いを選ぶ。
#[test]
fn bvaskw_dom_wiring_text() {
    let wins = read("src/wins.rs");
    assert!(wins.contains("Win::Ask => return crate::askwin::frame(ctx),"));
    assert!(wins.contains("pub fn draw(win: Win, ctx: WinCtx<Win>) -> Frame {"));
    let src = read("src/askwin.rs");
    let dom = &src[src.find("mod dom {").expect("wasm の枝")..];
    for needle in [
        "crate::net::read(ask::PATH)",
        "{ask::one_view(card, recorded)}",
        "Duration::from_millis(super::ADVANCE_MS)",
        "on:click=move |_| picked.set(Some(id.clone()))",
        "on:click=later_click",
        "width: frame_of(Win::Ask).0,",
    ] {
        assert!(dom.contains(needle), "wasm の枝に {needle} が無い");
    }
    assert!(read("src/lib.rs").contains("\npub mod askwin;\n"));
}

fn own_list() -> QuestionList {
    wire::decode(&read("../../tests/fixtures/surface/question-list.json")).expect("問いの一覧")
}

fn with_others(own: QuestionList, others: Vec<ProjectQuestions>) -> Fetched {
    Fetched::Body(wire::encode(&AllQuestions { own, others }).expect("電文"))
}

/// 問いの一覧が読めない（まだ読んでいない・口が読めない・server が台帳を読めない・電文が読めない）時は、局面を問いが無い
/// （Empty）にも全部答えた（Done）にもせず理由を持つ Unmeasured にし、題の残りの数は 0 でなく「?」にする。
#[test]
fn bvaskw_unread_not_empty() {
    let unknown = QuestionList {
        cards: Reading::Unknown,
        answerable: true,
    };
    let cases = [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, ask::REASON),
        (with_others(unknown, Vec::new()), ask::CARDS_UNKNOWN),
        (Fetched::Body("{".to_string()), ask::UNREADABLE),
    ];
    for (fetched, reason) in &cases {
        assert_eq!(unread(fetched), Some(*reason));
    }
    let empty = QuestionList {
        cards: Reading::Known(Vec::new()),
        answerable: true,
    };
    assert_eq!(unread(&with_others(empty, Vec::new())), None);
    assert_eq!(unread(&with_others(own_list(), Vec::new())), None);
    let all = ids(&["q.1", "q.2"]);
    let r = ask::REASON;
    assert_eq!(phase(Some(r), &[], &set(&[])), Phase::Unmeasured(r));
    assert_eq!(phase(Some(r), &[], &set(&["q.1"])), Phase::Unmeasured(r));
    assert_eq!(phase(Some(r), &all, &set(&[])), Phase::Unmeasured(r));
    assert_eq!(remaining(Phase::Unmeasured(r), &all, &set(&[])), None);
    assert_eq!(remaining(Phase::Walk, &all, &set(&["q.1"])), Some(1));
    assert_eq!(remaining(Phase::Empty, &[], &set(&[])), Some(0));
    assert_eq!(UNKNOWN_COUNT, "?");
    assert_eq!(title(None), "答えを待つ質問 ?");
    assert_eq!(title(Some(0)), "答えを待つ質問 0");
}

/// ほかの project の質問の段の数: 問いの一覧が読めないか台帳が読めないほかの project が在れば 0 でなく「?」。
#[test]
fn bvaskw_others_unknown_shown() {
    let Reading::Known(cards) = own_list().cards else {
        panic!("fixture の問い");
    };
    let mut other = cards[0].clone();
    other.id = BeadId::new("ox.1").expect("id");
    let known = |p: &str| ProjectQuestions {
        project: p.to_string(),
        answerable: false,
        cards: Reading::Known(vec![other.clone()]),
    };
    let lost = ProjectQuestions {
        project: "lost".to_string(),
        answerable: false,
        cards: Reading::Unknown,
    };
    assert_eq!(others_count(&with_others(own_list(), Vec::new())), Some(0));
    assert_eq!(
        others_count(&with_others(own_list(), vec![known("other")])),
        Some(1)
    );
    let mixed = with_others(own_list(), vec![known("other"), lost.clone()]);
    assert_eq!(others_count(&mixed), None);
    assert_eq!(ask::unknown_projects(&mixed), ["lost"]);
    assert_eq!(others_count(&with_others(own_list(), vec![lost])), None);
    let unknown = QuestionList {
        cards: Reading::Unknown,
        answerable: true,
    };
    assert_eq!(
        others_count(&with_others(unknown, vec![known("other")])),
        None
    );
    assert_eq!(others_count(&Fetched::Failed), None);
    assert_eq!(others_title(None), "ほかの project の質問 ?");
    assert_eq!(others_title(Some(1)), "ほかの project の質問 1");
}

/// 読めない時の窓の DOM（wasm の枝）の字: 局面は問いの一覧の読めない理由から決め、Unmeasured は前の質問の block と同じ
/// 測れていないの形（body_view の Unmeasured）で理由を出し、題の残りの数は remaining で読む。ほかの project の質問の段は
/// 数を others_count で読み、台帳が読めない組は前の質問の block と同じ札と 1 行（ask の OTHER_UNKNOWN）を出す。窓の本文の
/// 頭は席に届いていない裁定の 1 行（ask の late_view・届いたかが読めない時の 1 行も持つ）。
#[test]
fn bvaskw_unread_dom_text() {
    let src = read("src/askwin.rs");
    let dom = &src[src.find("mod dom {").expect("wasm の枝")..];
    for needle in [
        "let reason = Memo::new(move |_| fetched.with(unread));",
        "phase(reason.get(), i, a)",
        "{move || title(ids.with(|i| answered.with(|a| remaining(state.get(), i, a))))}",
        "Phase::Unmeasured(reason) => body_view(Body::Unmeasured(reason)),",
        "let others = Memo::new(move |_| fetched.with(others_count));",
        "let unknown = Memo::new(move |_| fetched.with(ask::unknown_projects));",
        "<summary>{move || others_title(n.get())}</summary>",
        "<li class=\"small muted\"><span class=\"chip\">{p}</span>\" \"{ask::OTHER_UNKNOWN}</li>",
        "<ul class=\"items\">{rows}{unread_rows}</ul></details>",
        "body: view! { {ask::late_view()}{walk}",
    ] {
        assert!(dom.contains(needle), "wasm の枝に {needle} が無い");
    }
}
