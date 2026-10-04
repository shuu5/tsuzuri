//! 行 g-unref-lc の歯（接頭辞 bvuphase_）: 台帳 open の一覧の memo と問いの行と見出しの未反映の数に、器の局面の出力の
//! 局面と手番を平易な字で添える（語は台帳の block の辞書の関数・出力がまだ無い間と知らない語は「まだ分からない」・
//! 読めない版の出力は「読めない」・古さの印の在る出力は「古い」・要件 FR13）。
//! 局面の出力と台帳の行は歯の中で組む（bead の id の接頭辞は fx-u）。指標は fixture tests/fixtures/surface/ledger-stats.json。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::case::{CaseDoc, CaseLinks, CasePart};
use tsuzuri_contract::ledger::{BeadId, LedgerRow, MEMO_LABEL, QUESTION_LABEL};
use tsuzuri_contract::stats::{LedgerStats, UnreflectedCount, UnreflectedKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::ledgerlist::{
    Lgroup, Phases, STALE_KEY, UNREADABLE_KEY, groups, phases, unref_head, with_phases,
};
use tsuzuri_surface::project::ledger::{
    PHASE_KEY, TURN_KEY, UNREF_PHASES, kind_phase_text, phase_text, plain_word,
};
use tsuzuri_surface::view::Fetched;

const NOW: u64 = 1_790_510_400;

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn row(id: &str, kind: &str, labels: &[&str]) -> LedgerRow {
    LedgerRow {
        id: BeadId::new(id).expect("id"),
        kind: kind.to_string(),
        title: format!("{id} の題"),
        status: "open".to_string(),
        updated_at: NOW,
        parent: None,
        labels: labels.iter().map(|l| (*l).to_string()).collect(),
    }
}

fn part(kind: &str, id: &str, phase: &str, turn: &str) -> CasePart {
    CasePart {
        part: kind.to_string(),
        id: id.to_string(),
        phase: phase.to_string(),
        turn: turn.to_string(),
        since: None,
        reason: None,
        why: None,
        closed: false,
        links: CaseLinks::default(),
    }
}

fn case_doc(parts: Reading<Vec<CasePart>>, stale: &[&str], unreadable: bool) -> Fetched {
    let doc = CaseDoc {
        generated_at: Some(NOW),
        stale: stale.iter().map(|s| (*s).to_string()).collect(),
        parts,
        unreadable,
    };
    Fetched::Body(wire::encode(&doc).expect("enc"))
}

fn cases(parts: Reading<Vec<CasePart>>) -> Fetched {
    case_doc(parts, &[], false)
}

/// memo の行 fx-u1.2 の局面の字。
fn memo_phase(gs: &[Lgroup], cases: &Fetched) -> Option<String> {
    with_phases(gs.to_vec(), cases)[0]
        .rows
        .iter()
        .find(|r| r.id == "fx-u1.2")
        .and_then(|r| r.phase.clone())
}

/// (2) 局面と手番の平易な字は語の辞書の鍵 lc: と turn: の見出しで、辞書に無い語は「まだ分からない」。
#[test]
fn bvuphase_phase_and_turn_words() {
    assert_eq!((PHASE_KEY, TURN_KEY), ("lc:", "turn:"));
    let words: Vec<(&str, String)> = [
        "memo-promoting",
        "memo-asking",
        "memo-actionable",
        "memo-waiting",
        "question-open",
        "ruling-unreflected",
        "utterance-open",
        "misfit",
    ]
    .into_iter()
    .map(|p| (p, plain_word(PHASE_KEY, p)))
    .collect();
    let want = [
        "昇格の途中",
        "問いの答え待ち",
        "処置の待ち",
        "引き金待ち",
        "答えの待ち",
        "反映待ち",
        "未仕分け",
        "形の崩れ",
    ];
    assert_eq!(
        words.iter().map(|(_, w)| w.as_str()).collect::<Vec<_>>(),
        want
    );
    let turns: Vec<String> = ["user", "seat", "vessel", "runner", "ci", "none"]
        .into_iter()
        .map(|t| plain_word(TURN_KEY, t))
        .collect();
    assert_eq!(
        turns,
        [
            "あなたの番",
            "席の番",
            "器の番",
            "実装の番",
            "CI の番",
            "誰の番でもない"
        ]
    );
    assert_eq!(phase_text("memo-actionable", "seat"), "処置の待ち · 席の番");
    assert_eq!(phase_text("memo-closed", "seat"), "まだ分からない · 席の番");
    assert_eq!(
        phase_text("question-open", "owner"),
        "答えの待ち · まだ分からない"
    );
}

/// (3) 未反映の 3 種の局面の表（memo は memo-actionable と misfit・裁定は ruling-unreflected・発話は utterance-open）
/// と、その平易な字（局面の字を・でつなぎ手番の席の字を添える）。
#[test]
fn bvuphase_unref_kind_phases() {
    let table: Vec<(UnreflectedKind, Vec<&str>)> =
        UNREF_PHASES.iter().map(|(k, p)| (*k, p.to_vec())).collect();
    assert_eq!(
        table,
        [
            (UnreflectedKind::Memo, vec!["memo-actionable", "misfit"]),
            (UnreflectedKind::Ruling, vec!["ruling-unreflected"]),
            (UnreflectedKind::Utterance, vec!["utterance-open"]),
        ]
    );
    assert_eq!(
        kind_phase_text(UnreflectedKind::Memo),
        "処置の待ち・形の崩れ · 席の番"
    );
    assert_eq!(
        kind_phase_text(UnreflectedKind::Ruling),
        "反映待ち · 席の番"
    );
    assert_eq!(
        kind_phase_text(UnreflectedKind::Utterance),
        "未仕分け · 席の番"
    );
}

/// (4) 行の局面: memo と問いの行は局面の出力の memo と question の部品の局面と手番の字・部品が無い行と出力がまだ無い
/// 間は「まだ分からない」・読めない版の出力の間は「読めない」・古さの印が在れば局面の字に「古い」を添える・便と epic の
/// 行は字を持たない・memo と question でない部品の同じ id は使わない。
#[test]
fn bvuphase_rows_get_phase() {
    let rows = vec![
        row("fx-u1", "epic", &[]),
        row("fx-u1.1", "task", &[]),
        row("fx-u1.2", "task", &[MEMO_LABEL]),
        row("fx-u1.3", "task", &[QUESTION_LABEL]),
        row("fx-u1.4", "task", &[MEMO_LABEL]),
        row("fx-u1.5", "task", &[MEMO_LABEL]),
    ];
    let gs = groups(&rows, &[], &BTreeMap::new(), NOW);
    let doc = cases(Reading::Known(vec![
        part("memo", "fx-u1.2", "memo-actionable", "seat"),
        part("question", "fx-u1.3", "question-open", "user"),
        part("contract", "fx-u1.5", "contract-queued", "vessel"),
        part("contract", "fx-u1.1", "contract-running", "none"),
    ]));
    let got: Vec<(String, Option<String>)> = with_phases(gs.clone(), &doc)[0]
        .rows
        .iter()
        .map(|r| (r.id.clone(), r.phase.clone()))
        .collect();
    let want = |id: &str, p: Option<&str>| (id.to_string(), p.map(str::to_string));
    assert_eq!(
        got,
        [
            want("fx-u1.3", Some("答えの待ち · あなたの番")),
            want("fx-u1.1", None),
            want("fx-u1.2", Some("処置の待ち · 席の番")),
            want("fx-u1.4", Some("まだ分からない")),
            want("fx-u1.5", Some("まだ分からない")),
        ]
    );
    assert!(gs[0].rows.iter().all(|r| r.phase.is_none()));
    for bad in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{".into()),
        cases(Reading::Unknown),
        case_doc(Reading::Unknown, &["lifecycle-old"], false),
    ] {
        assert_eq!(phases(&bad), Phases::Unknown);
        assert_eq!(memo_phase(&gs, &bad).as_deref(), Some("まだ分からない"));
    }
    unreadable_and_stale(&gs);
    let Phases::Known(table, stale) = phases(&doc) else {
        panic!("読めた出力の表");
    };
    assert!(stale.is_empty());
    assert_eq!(table.len(), 2);
    assert_eq!(
        table.get("fx-u1.3"),
        Some(&("question-open".to_string(), "user".to_string()))
    );
}

/// (4) の読めない版と古さの印の周: 読めない版（unreadable 真）は「読めない」で出力が無い周（unreadable 偽）と字が違い、
/// 古さの印が在れば局面の字に「古い」を添え、部品の無い行は「まだ分からない」のまま。
fn unreadable_and_stale(gs: &[Lgroup]) {
    // 読めない版（unreadable 真）は出力が無い周（unreadable 偽）と字が違う。
    let unreadable = case_doc(Reading::Unknown, &[], true);
    assert_eq!(phases(&unreadable), Phases::Unreadable);
    assert_eq!(UNREADABLE_KEY, "case_unreadable");
    assert_eq!(memo_phase(gs, &unreadable).as_deref(), Some("読めない"));
    assert_ne!(
        memo_phase(gs, &unreadable),
        memo_phase(gs, &cases(Reading::Unknown))
    );
    // 古さの印が在れば局面の字に「古い」を添え、部品の無い行は「まだ分からない」のまま。
    let parts = vec![part("memo", "fx-u1.2", "memo-actionable", "seat")];
    let old = case_doc(
        Reading::Known(parts.clone()),
        &["lifecycle-old", "input-moved"],
        false,
    );
    assert_eq!(STALE_KEY, "case_stale");
    assert_eq!(
        memo_phase(gs, &old).as_deref(),
        Some("処置の待ち · 席の番 古い")
    );
    let q = &with_phases(gs.to_vec(), &old)[0].rows[0];
    assert_eq!(
        (q.id.as_str(), q.phase.as_deref()),
        ("fx-u1.3", Some("まだ分からない"))
    );
    assert_eq!(
        memo_phase(gs, &cases(Reading::Known(parts))).as_deref(),
        Some("処置の待ち · 席の番")
    );
    let Phases::Known(_, stale) = phases(&old) else {
        panic!("古い出力の表");
    };
    assert_eq!(stale, ["lifecycle-old", "input-moved"]);
}

/// (5) 見出しの未反映の数は未反映の見出しの語と数と手番の席の字で、1〜2 種が分からなければ測れていないの印を添え、
/// 3 種とも分からなければ数は「まだ分からない」、読めない版の出力の周は「読めない」、古さの印が在れば「古い」を添える。
/// title は種類ごとの見出しと数（分からない種類は「まだ分からない」）と局面の平易な字で、古さの印の種類を末に並べる。
#[test]
fn bvuphase_unref_head() {
    let mut sets: BTreeMap<String, LedgerStats> =
        wire::decode(&read("../../tests/fixtures/surface/ledger-stats.json")).expect("fixture");
    let mut s = sets.remove("filled").expect("filled");
    s.unreflected = 4;
    s.unreflected_kinds = vec![
        UnreflectedCount {
            kind: UnreflectedKind::Memo,
            count: 3,
        },
        UnreflectedCount {
            kind: UnreflectedKind::Ruling,
            count: 1,
        },
    ];
    s.unreflected_unknown = vec![UnreflectedKind::Utterance];
    let read = phases(&cases(Reading::Known(Vec::new())));
    let head = unref_head(&s, &read);
    assert_eq!(head.text, "未反映 4 · 席の番");
    assert!(head.partial);
    let title = "memo 3（処置の待ち・形の崩れ · 席の番） / 裁定 1（反映待ち · 席の番） / 発話 まだ分からない（未仕分け · 席の番）";
    assert_eq!(head.title, title);
    // 2 種が分からなくても印を添え、全部が分かれば添えない。
    s.unreflected_unknown = vec![UnreflectedKind::Ruling, UnreflectedKind::Utterance];
    assert!(unref_head(&s, &read).partial);
    s.unreflected_unknown = Vec::new();
    assert!(!unref_head(&s, &read).partial);
    // 古さの印が在れば数に「古い」を添え、title の末に印の種類を並べる。
    s.unreflected_unknown = vec![UnreflectedKind::Utterance];
    let old = phases(&case_doc(
        Reading::Known(Vec::new()),
        &["lifecycle-old", "input-moved"],
        false,
    ));
    let head = unref_head(&s, &old);
    assert_eq!(head.text, "未反映 4 古い · 席の番");
    assert_eq!(
        head.title,
        format!("{title} / 古い（lifecycle-old・input-moved）")
    );
    // 3 種とも分からなければ数はまだ分からないで印は無く、読めない版の出力の周は「読めない」。
    s.unreflected = 0;
    s.unreflected_kinds = Vec::new();
    s.unreflected_unknown = UnreflectedKind::ALL.to_vec();
    let none = unref_head(&s, &phases(&cases(Reading::Unknown)));
    assert_eq!(none.text, "未反映 まだ分からない · 席の番");
    assert!(!none.partial);
    let unreadable = unref_head(&s, &phases(&case_doc(Reading::Unknown, &[], true)));
    assert_eq!(unreadable.text, "未反映 読めない · 席の番");
    assert!(!unreadable.partial);
    assert_ne!(unreadable.text, none.text);
}

/// (6) DOM の字: 一覧は局面の出力の口を契約の定数で読んで行に字を置き、行と見出しに出す。class は stylesheet に在る。
#[test]
fn bvuphase_dom_wiring_text() {
    let list = read("src/ledgerlist.rs");
    let dom = &list[list.find("mod dom {").expect("mod dom")..];
    for want in [
        "let cases = crate::net::read(case::PATH);",
        "cases.with(|c| with_phases(groups, c))",
        "cases.with(|c| unref_head(&s, &phases(c)))",
        "let mark = h.partial.then(|| state_icon(UNKNOWN));",
        "<span class=\"ll-unref num\" title=h.title>{h.text}{mark}</span>",
        "<span class=\"ll-ph\">{p}</span>",
    ] {
        assert!(dom.contains(want), "ledgerlist の DOM に {want} が無い");
    }
    assert!(!list.contains("\"/api/cases\""));
    let css = read("style.css");
    for class in [".ll-ph ", ".ll-unref "] {
        assert!(css.contains(class), "stylesheet に {class} が無い");
    }
}
