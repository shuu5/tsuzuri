//! 行 g-pop-why の歯: 吹き出しの Queued の起きない理由と Blocked の待つ理由（器の列の待ちの 12 語を平易な字にした語の辞書）と、
//! 表に無い理由の語のまだ分からないと、待つ相手（links.on・links.runs・承認待ちは持ち主）と、層が with_why を通す配線の字。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{PipelineCard, Reading, Stage};
use tsuzuri_contract::case::{CaseLinks, CasePart};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::pop::{
    APPROVAL_KEYS, Fact, Partner, REASON_HEAD, REASONS, STAGE_KEYS, Src, Val, WHY_KEYS, pop,
    reason_key, reason_val, with_why,
};

const NOW: EpochSecs = 1_790_000_000;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn card(x: &str, stage: Stage, runs: u32, reason: Option<&str>) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(x).unwrap_or_else(|e| panic!("{x}: {e:?}")),
        runs,
        stage,
        reason: reason.map(str::to_string),
        account: None,
        since: Some(NOW - 60),
        ci: None,
    }
}

fn part(x: &str, phase: &str, reason: Option<&str>, on: &[&str], runs: &[&str]) -> CasePart {
    CasePart {
        part: "contract".to_string(),
        id: x.to_string(),
        phase: phase.to_string(),
        turn: "vessel".to_string(),
        since: Some(NOW - 60),
        reason: reason.map(str::to_string),
        closed: false,
        links: CaseLinks {
            on: on.iter().map(|s| s.to_string()).collect(),
            runs: runs.iter().map(|s| s.to_string()).collect(),
        },
    }
}

fn facts(cards: &[PipelineCard], parts: Reading<&[CasePart]>) -> Vec<Fact> {
    let src = Src {
        facts: Reading::Unknown,
        rows: Reading::Unknown,
        cards,
        parts,
        graph: None,
    };
    with_why(pop(cards[0].contract.as_str(), &src), &src).facts
}

fn keys(f: &[Fact]) -> Vec<&'static str> {
    f.iter().map(|x| x.key).collect()
}

fn val<'a>(f: &'a [Fact], key: &str) -> &'a Val {
    &f.iter()
        .find(|x| x.key == key)
        .unwrap_or_else(|| panic!("欄 {key} が無い"))
        .val
}

/// 理由の語は器の pipe/dispatch.rs の WAIT_REASONS の 12 語（宣言の順）で、語ごとに鍵 qr:<語> の平易な字が語の辞書に在る。
#[test]
fn bvwhy_twelve_reason_words_in_vocab() {
    assert_eq!(
        REASONS,
        [
            "dependency",
            "overlap",
            "admission",
            "host-busy",
            "hold",
            "launched",
            "settled",
            "no-design-pointer",
            "unreflected-ruling",
            "floor",
            "reserved",
            "sibling"
        ]
    );
    assert_eq!(REASON_HEAD, "qr:");
    for w in REASONS {
        let key = format!("qr:{w}");
        let t = vocab()
            .term(&key)
            .unwrap_or_else(|| panic!("{key} が語の辞書に無い"));
        assert!(!t.label.is_ascii() && !t.note.is_empty(), "{key}");
        assert_eq!(reason_key(w).as_deref(), Some(key.as_str()));
    }
    for k in WHY_KEYS.into_iter().chain(APPROVAL_KEYS) {
        assert!(vocab().term(k).is_some(), "{k} が語の辞書に無い");
    }
}

/// 理由の語の : の後の詳細は外して引き、表に無い語と理由の無い札はまだ分からない。
#[test]
fn bvwhy_unknown_reason_word() {
    assert_eq!(
        reason_val(Some("dependency:t-1,t-2")),
        Val::Text(label("qr:dependency"))
    );
    assert_eq!(reason_val(Some("warped")), Val::Unknown);
    assert_eq!(reason_val(Some("")), Val::Unknown);
    assert_eq!(reason_val(None), Val::Unknown);
    let cards = [card("t-1", Stage::Queued, 0, Some("warped"))];
    let f = facts(&cards, Reading::Unknown);
    assert_eq!(keys(&f)[5..], [STAGE_KEYS[2], WHY_KEYS[0]]);
    assert_eq!(*val(&f, WHY_KEYS[0]), Val::Unknown);
}

/// Queued の起きない理由は局面の出力の契約の部品の理由（無ければ札の理由）を列に入った時刻の後に出す。
#[test]
fn bvwhy_queued_why_from_case() {
    let cards = [card("t-1", Stage::Queued, 0, Some("hold"))];
    let parts = [part("t-1", "contract-queued", Some("floor"), &[], &[])];
    let f = facts(&cards, Reading::Known(&parts));
    assert_eq!(keys(&f)[5..], [STAGE_KEYS[2], WHY_KEYS[0]]);
    assert_eq!(*val(&f, WHY_KEYS[0]), Val::Text(label("qr:floor")));
    let f = facts(&cards, Reading::Unknown);
    assert_eq!(*val(&f, WHY_KEYS[0]), Val::Text(label("qr:hold")));
}

/// Blocked の待つ相手は links.on の bead・待つ理由は部品の理由・links.runs が在れば相手の便を待つ理由の後に出す。
/// 予約（reserved）で links.on が空なら相手はまだ分からない。
#[test]
fn bvwhy_partners_from_links() {
    let cards = [card("t-1", Stage::Blocked, 0, None)];
    let parts = [part(
        "t-1",
        "contract-queued",
        Some("overlap"),
        &["t-5"],
        &["t-5-20261001T000000Z"],
    )];
    let f = facts(&cards, Reading::Known(&parts));
    assert_eq!(
        keys(&f)[5..],
        [STAGE_KEYS[0], WHY_KEYS[1], WHY_KEYS[2], STAGE_KEYS[1]]
    );
    assert_eq!(
        *val(&f, STAGE_KEYS[0]),
        Val::Partners(vec![Partner {
            id: "t-5".to_string(),
            short: "t-5".to_string(),
            stage: None
        }])
    );
    assert_eq!(*val(&f, WHY_KEYS[1]), Val::Text(label("qr:overlap")));
    assert_eq!(
        *val(&f, WHY_KEYS[2]),
        Val::Code("t-5-20261001T000000Z".to_string())
    );
    let parts = [part("t-1", "contract-queued", Some("reserved"), &[], &[])];
    let f = facts(&cards, Reading::Known(&parts));
    assert_eq!(keys(&f)[5..], [STAGE_KEYS[0], WHY_KEYS[1], STAGE_KEYS[1]]);
    assert_eq!(*val(&f, STAGE_KEYS[0]), Val::Unknown);
    assert_eq!(*val(&f, WHY_KEYS[1]), Val::Text(label("qr:reserved")));
    let parts = [part(
        "t-1",
        "contract-queued",
        Some("reserved"),
        &["t-6"],
        &[],
    )];
    let f = facts(&cards, Reading::Known(&parts));
    assert!(matches!(val(&f, STAGE_KEYS[0]), Val::Partners(p) if p[0].id == "t-6"));
}

/// 承認待ち（契約の部品が contract-running で札の段が Blocked）は相手を持ち主・待つ理由を持ち主の承認にする。
/// 部品が読めない間は承認待ちと決めず、札の理由（語の辞書に無い器の detail の字）はまだ分からない。
#[test]
fn bvwhy_approval_partner_is_owner() {
    let cards = [card("t-1", Stage::Blocked, 2, Some("approval:merge"))];
    let parts = [part(
        "t-1",
        "contract-running",
        Some("run-blocked"),
        &[],
        &[],
    )];
    let f = facts(&cards, Reading::Known(&parts));
    assert_eq!(keys(&f)[5..], [STAGE_KEYS[0], WHY_KEYS[1], STAGE_KEYS[1]]);
    assert_eq!(*val(&f, STAGE_KEYS[0]), Val::Text(label(APPROVAL_KEYS[0])));
    assert_eq!(*val(&f, WHY_KEYS[1]), Val::Text(label(APPROVAL_KEYS[1])));
    let f = facts(&cards, Reading::Unknown);
    assert_eq!(*val(&f, STAGE_KEYS[0]), Val::Unknown);
    assert_eq!(*val(&f, WHY_KEYS[1]), Val::Unknown);
}

/// ほかの段の札は替えない。層は with_runs の後に with_why を通す。
#[test]
fn bvwhy_dom_wiring_text() {
    for stage in [Stage::Running, Stage::Failed, Stage::Landed] {
        let cards = [card("t-1", stage, 1, Some("dependency"))];
        let src = Src {
            facts: Reading::Unknown,
            rows: Reading::Unknown,
            cards: &cards,
            parts: Reading::Unknown,
            graph: None,
        };
        let p = pop("t-1", &src);
        assert_eq!(with_why(p.clone(), &src), p);
    }
    let src = read("src/widgets/pop.rs");
    assert!(src.contains(
        "let p = with_runs(pop(&id, &src), known(&lines));\n            let p = with_why(p, &src);"
    ));
}
