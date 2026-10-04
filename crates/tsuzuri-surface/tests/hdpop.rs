//! 留め置きの吹き出しの歯（面・接頭辞 hdpop_・設計ノート surface-wave28a 行 g-held-pop・判断の記録 ADR-42 決定 (7)）。
//! 段 Held の札の吹き出しは止めた者・理由・止めた時刻と経過・解く条件の 4 つ（席の止めと受付の断りで字が違う）。
//! 理由は局面の出力の契約の部品の欄 why で、無ければまだ分からない。ほかの段の札は留め置きの欄を持たない。否定の見本は 1 欄だけ替える。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineCard, Reading, Stage};
use tsuzuri_contract::case::{CaseLinks, CasePart};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::keyline::{HOLD, HeldBy};
use tsuzuri_surface::widgets::pop::{
    Fact, HELD_KEYS, HELD_WORDS, STAGE_KEYS, Src, UNKNOWN_KEY, Val, WHY_KEYS, held_facts, pop,
    stage_facts, with_why,
};

const NOW: EpochSecs = 1_790_000_000;
const AT: EpochSecs = NOW - 7_200;

/// 受付の断りの名と、止めの理由の字。
const NAME: &str = "teeth-outside-write-set";
const WHY: &str = "書く file の重なりを席が確かめる";

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn card(stage: Stage, reason: &str) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new("h-1").unwrap_or_else(|e| panic!("{e:?}")),
        runs: 0,
        stage,
        reason: Some(reason.to_string()),
        account: None,
        since: Some(AT),
        ci: None,
    }
}

fn part(phase: &str, reason: &str, why: Option<&str>) -> CasePart {
    CasePart {
        part: "contract".to_string(),
        id: "h-1".to_string(),
        phase: phase.to_string(),
        turn: "seat".to_string(),
        since: Some(AT),
        reason: Some(reason.to_string()),
        why: why.map(str::to_string),
        closed: false,
        links: CaseLinks::default(),
    }
}

fn src<'a>(cards: &'a [PipelineCard], parts: Reading<&'a [CasePart]>) -> Src<'a> {
    Src {
        facts: Reading::Unknown,
        rows: Reading::Unknown,
        cards,
        parts,
        graph: None,
    }
}

fn text(key: &str) -> Val {
    Val::Text(label(key))
}

fn keys(facts: &[Fact]) -> Vec<&'static str> {
    facts.iter().map(|f| f.key).collect()
}

#[test]
fn hdpop_seat_hold_facts() {
    let cards = [card(Stage::Held, HOLD)];
    let parts = [part("contract-queued", HOLD, Some(WHY))];
    let s = src(&cards, Reading::Known(&parts));
    let got = stage_facts(&s, &cards[0]);
    let want = [
        (HELD_KEYS[0], text("hb:seat")),
        (HELD_KEYS[1], Val::Text(WHY.to_string())),
        (HELD_KEYS[2], Val::At(AT)),
        (HELD_KEYS[3], text("hu:seat")),
    ];
    let got: Vec<(&str, Val)> = got.into_iter().map(|f| (f.key, f.val)).collect();
    assert_eq!(got, want);
    assert_eq!(held_facts(&s, &cards[0]), stage_facts(&s, &cards[0]));
}

#[test]
fn hdpop_intake_refusal_facts() {
    let cards = [card(Stage::Held, NAME)];
    let parts = [part("contract-refused", NAME, Some(WHY))];
    let s = src(&cards, Reading::Known(&parts));
    let got = stage_facts(&s, &cards[0]);
    assert_eq!(keys(&got), HELD_KEYS);
    assert_eq!(got[0].val, text("hb:intake"));
    assert_eq!(got[1].val, Val::Text(format!("{NAME}：{WHY}")));
    assert_eq!(got[2].val, Val::At(AT));
    assert_eq!(got[3].val, text("hu:intake"));
}

#[test]
fn hdpop_missing_why_and_since_unknown() {
    let unknown = label(UNKNOWN_KEY);
    for (reason, phase, want) in [
        (HOLD, "contract-queued", Val::Unknown),
        (
            NAME,
            "contract-refused",
            Val::Text(format!("{NAME}：{unknown}")),
        ),
    ] {
        let cards = [card(Stage::Held, reason)];
        let parts = [part(phase, reason, None)];
        let got = stage_facts(&src(&cards, Reading::Known(&parts)), &cards[0]);
        assert_eq!(got[1].val, want, "why の無い部品 {reason}");
        let got = stage_facts(&src(&cards, Reading::Unknown), &cards[0]);
        assert_eq!(got[1].val, want, "読めない局面の出力 {reason}");
        let mut no_since = cards[0].clone();
        no_since.since = None;
        let got = stage_facts(&src(&cards, Reading::Known(&parts)), &no_since);
        assert_eq!(got[2].val, Val::Unknown, "since の無い札 {reason}");
    }
}

#[test]
fn hdpop_other_stages_no_held_facts() {
    let parts = [part("contract-queued", HOLD, Some(WHY))];
    for stage in [Stage::Queued, Stage::Blocked] {
        let cards = [card(stage, HOLD)];
        let s = src(&cards, Reading::Known(&parts));
        let got = keys(&stage_facts(&s, &cards[0]));
        assert!(
            HELD_KEYS.iter().all(|k| !got.contains(k)),
            "段だけ替えた見本 {stage:?}: {got:?}"
        );
        assert!(held_facts(&s, &cards[0]).is_empty());
    }
    let queued = [card(Stage::Queued, HOLD)];
    let s = src(&queued, Reading::Known(&parts));
    assert_eq!(keys(&stage_facts(&s, &queued[0])), [STAGE_KEYS[2]]);
}

#[test]
fn hdpop_with_why_leaves_held() {
    let cards = [card(Stage::Held, HOLD)];
    let parts = [part("contract-queued", HOLD, Some(WHY))];
    let s = src(&cards, Reading::Known(&parts));
    let p = pop("h-1", &s);
    let after = with_why(p.clone(), &s);
    assert_eq!(after, p, "留め置きの札に起きない理由を足さない");
    assert!(keys(&after.facts).iter().all(|k| !WHY_KEYS.contains(k)));
    let tail: Vec<&str> = keys(&after.facts).into_iter().rev().take(4).collect();
    assert_eq!(tail, HELD_KEYS.into_iter().rev().collect::<Vec<_>>());
}

#[test]
fn hdpop_vocab_words() {
    let v = vocab();
    let term = |k: &str| v.term(k).unwrap_or_else(|| panic!("{k} が語の辞書に無い"));
    assert_eq!(
        HELD_KEYS,
        ["pf_held_by", "pf_held_why", "pf_held_at", "pf_held_until"]
    );
    assert_eq!(
        HELD_WORDS,
        [
            (HeldBy::Seat, "hb:seat", "hu:seat"),
            (HeldBy::Intake, "hb:intake", "hu:intake")
        ]
    );
    for (_, who, until) in HELD_WORDS {
        for k in HELD_KEYS.into_iter().chain([who, until]) {
            let t = term(k);
            assert!(!t.label.is_ascii() && !t.note.is_empty(), "{k}");
        }
    }
    assert_ne!(term("hb:seat").label, term("hb:intake").label);
    assert_ne!(term("hu:seat").label, term("hu:intake").label);
    let src = read("src/widgets/pop.rs");
    assert!(src.contains("        Stage::Held => held_facts(src, card),\n"));
}
