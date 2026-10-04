//! 台帳 open の一覧の段の字と組の頭の段の数に板の札と同じ段の記号を付ける歯（行 g-list-sym・接頭辞 lsym_・判断の記録 ADR-30 決定 (9)）。
//! 行の記号の材料を板の札の値と照らし、札の無い行は記号を持たないことを断言し、DOM（wasm の枝）の配線の字と stylesheet の規則を照らす。
//! 台帳の行と札は歯の中で組む（bead の id の接頭辞は fx-y）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::{Ci, PipelineCard, Stage};
use tsuzuri_contract::ledger::{BeadId, LedgerRow, MEMO_LABEL, QUESTION_LABEL};
use tsuzuri_surface::ledgerlist::{Lrow, groups};
use tsuzuri_surface::project::pipeline::{CLOSED_TAG, card_sym, kcard};

/// 一覧を組む今。
const NOW: u64 = 1_790_510_400;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// file の字の start の字から end の字の前まで（どちらも在ることを断言する）。
fn span(src: &str, start: &str, end: &str) -> String {
    let at = src.find(start).unwrap_or_else(|| panic!("{start} が無い"));
    let rest = &src[at..];
    let to = rest.find(end).unwrap_or_else(|| panic!("{end} が無い"));
    rest[..to].to_string()
}

fn row(id: &str, labels: &[&str]) -> LedgerRow {
    LedgerRow {
        id: BeadId::new(id).expect("id"),
        kind: "task".to_string(),
        title: format!("{id} の題"),
        status: "open".to_string(),
        updated_at: NOW - 600,
        parent: None,
        labels: labels.iter().map(|l| (*l).to_string()).collect(),
    }
}

fn card(id: &str, stage: Stage) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("id"),
        runs: 1,
        stage,
        reason: None,
        account: None,
        since: Some(NOW - 120),
        ci: None,
    }
}

/// 台帳: epic fx-y の下に便 8 本（札を持つ 7 本と札の無い 1 本）と memo と問い。
fn rows() -> Vec<LedgerRow> {
    let mut rows = vec![LedgerRow {
        kind: "epic".to_string(),
        ..row("fx-y", &[])
    }];
    rows.extend((1..=8).map(|n| row(&format!("fx-y.{n}"), &[])));
    rows.push(row("fx-y.9", &[MEMO_LABEL]));
    rows.push(row("fx-y.10", &[QUESTION_LABEL]));
    rows
}

/// 札: 5 つの段と、台帳で閉じた（着地せず）の札と、CI を待つ着地の札（fx-y.8 は札なし）。
fn cards() -> Vec<PipelineCard> {
    let closed = PipelineCard {
        reason: Some(format!("{CLOSED_TAG} 取り下げ")),
        ..card("fx-y.6", Stage::Landed)
    };
    let waiting = PipelineCard {
        ci: Some(Ci::Waiting),
        since: Some(NOW - 86_400),
        ..card("fx-y.7", Stage::Landed)
    };
    vec![
        card("fx-y.1", Stage::Blocked),
        card("fx-y.2", Stage::Queued),
        card("fx-y.3", Stage::Running),
        card("fx-y.4", Stage::Failed),
        card("fx-y.5", Stage::Landed),
        closed,
        waiting,
    ]
}

/// 一覧の行を id で引く表。
fn listed() -> BTreeMap<String, Lrow> {
    groups(&rows(), &cards(), &BTreeMap::new(), NOW)
        .into_iter()
        .flat_map(|g| g.rows)
        .map(|r| (r.id.clone(), r))
        .collect()
}

#[test]
fn lsym_row_sym_like_card() {
    let listed = listed();
    let rows = rows();
    for c in cards() {
        let id = c.contract.to_string();
        let got = listed
            .get(&id)
            .unwrap_or_else(|| panic!("{id} の行が無い"))
            .sym;
        let board = kcard(&c, &rows, NOW);
        assert_eq!(got, Some((board.closed, board.state)), "{id} の行と板の札");
        assert_eq!(got, Some(card_sym(&c, NOW)), "{id} の行は card_sym の値");
    }
    let want = [
        ("fx-y.1", (false, Some("wait"))),
        ("fx-y.2", (false, Some("wait"))),
        ("fx-y.3", (false, Some("run"))),
        ("fx-y.4", (false, Some("wait"))),
        ("fx-y.5", (false, None)),
        ("fx-y.6", (true, None)),
        ("fx-y.7", (false, Some("run"))),
    ];
    for (id, sym) in want {
        assert_eq!(listed[id].sym, Some(sym), "{id} の段の記号の材料");
    }
}

#[test]
fn lsym_no_card_plain() {
    let listed = listed();
    for id in ["fx-y.8", "fx-y.9", "fx-y.10"] {
        assert_eq!(listed[id].sym, None, "札の無い行 {id} は記号を持たない");
    }
    // 札の無い便の行に札を 1 枚だけ足すと記号を持つ（行の違いは札の有無だけ）。
    let mut cards = cards();
    cards.push(card("fx-y.8", Stage::Queued));
    let with_card = groups(&rows(), &cards, &BTreeMap::new(), NOW)
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == "fx-y.8")
        .expect("fx-y.8 の行");
    assert_eq!(with_card.sym, Some((false, Some("wait"))));
}

#[test]
fn lsym_row_and_head_dom() {
    let list = read("src/ledgerlist.rs");
    let dom = &list[list.find("mod dom {").expect("wasm の枝")..];
    let row = span(dom, "fn row_view(", "\n    }\n");
    assert!(
        row.contains(".map(|(closed, state)| pipeline::stage_sym(closed, state));"),
        "行の記号は札と同じ stage_sym で描く"
    );
    assert!(
        row.contains("<span class=\"ll-rt\">{sym}{r.right.clone()}</span>"),
        "記号は右の段の字の前"
    );
    let head = span(dom, "fn head_view(", "\n    }\n");
    assert!(
        head.contains("let sym = pipeline::stage_sym(false, lane.state);"),
        "組の頭の数の記号は列の状態の記号"
    );
    assert!(
        head.contains("title=label(lane.key)>{sym}{n}</b>"),
        "記号は段の数の前"
    );
    assert!(
        head.contains("<b class=\"ll-sd\" title=label(state_key(UNKNOWN))>{COUNTS_UNKNOWN}</b>"),
        "板が読めない時の ? の 1 つは記号を持たない"
    );
    let css = read("style.css");
    assert_eq!(
        css.matches(".ll-rt, .ll-sd { display: inline-flex; align-items: center; gap: 3px; }")
            .count(),
        1,
        "記号と字を横に並べる規則"
    );
}

#[test]
fn lsym_kcard_shares_sym() {
    let pipe = read("src/project/pipeline.rs");
    let body = span(&pipe, "pub fn kcard(", "\n}\n");
    assert!(
        body.contains("let (closed, state) = card_sym(card, now);"),
        "板の札も card_sym の値で記号を決める"
    );
    assert!(
        !body.contains("CI_WAIT_STATE"),
        "kcard が状態の記号を別に決める"
    );
    assert!(
        !body.contains("closed_card("),
        "kcard が閉じた札を別に決める"
    );
}
