//! 行 g-select の歯（接頭辞 bvsel_）: 一覧の組の頭の名と板の見出しの epic の chip で epic を選ぶと、選んだ組でない札を薄くし、
//! 札か一覧の行を押して吹き出しを開くと、その札と行と組の頭に輪の印を付ける（判断の記録 ADR-27 の決定 (8)・見本 board-v2 の
//! setEpic と applyMarks と renderEchips）。台帳の行と札と bead の事実は歯の中で組む（bead の id の接頭辞は fx-s）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::common::card;
use tsuzuri_contract::board::{PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::{BeadFact, BeadId, LedgerRow};
use tsuzuri_surface::ledgerlist::{
    Lgroup, OUTSIDE_KEY, dim, groups, head_ring, key_of, pick, ring,
};
use tsuzuri_surface::project::ledger::OUTSIDE;
use tsuzuri_surface::project::pipeline::{ALL_EPICS, Chip, Column, card_keys, chips, columns};

/// 板と一覧を組む今。
const NOW: u64 = 1_790_510_400;

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn row(id: &str, kind: &str, status: &str) -> LedgerRow {
    LedgerRow {
        id: BeadId::new(id).expect("id"),
        kind: kind.to_string(),
        title: format!("{id} の題"),
        status: status.to_string(),
        updated_at: NOW - 600,
        parent: None,
        labels: Vec::new(),
    }
}

fn fact(id: &str, short: &str) -> BeadFact {
    BeadFact {
        id: BeadId::new(id).expect("id"),
        created_at: Some(NOW - 3_600),
        short: short.to_string(),
        short_set: true,
        blocks: Vec::new(),
    }
}

/// 台帳: epic s1（便 3）と入れ子の epic s1.7（便 1）・epic s2（便 2）・台帳に無い epic の下の便 s9.1・epic の外の便 s0。
fn rows() -> Vec<LedgerRow> {
    vec![
        row("fx-s1", "epic", "open"),
        row("fx-s1.1", "task", "open"),
        row("fx-s1.2", "task", "open"),
        row("fx-s1.3", "task", "closed"),
        row("fx-s1.7", "epic", "open"),
        row("fx-s1.7.1", "task", "open"),
        row("fx-s2", "epic", "open"),
        row("fx-s2.1", "task", "open"),
        row("fx-s2.2", "task", "open"),
        row("fx-s0", "task", "open"),
    ]
}

/// 札: s1 の 2 枚（1 枚は 12 時間より前の着地で板に出ない）・s1.7 の 1 枚・s2 の 2 枚・epic の外の 1 枚。
fn cards() -> Vec<PipelineCard> {
    vec![
        card("fx-s1.1", Stage::Running, NOW - 60),
        card("fx-s1.3", Stage::Landed, NOW - 50_000),
        card("fx-s1.7.1", Stage::Queued, NOW - 120),
        card("fx-s2.1", Stage::Blocked, NOW - 300),
        card("fx-s2.2", Stage::Failed, NOW - 400),
        card("fx-s0", Stage::Queued, NOW - 500),
    ]
}

fn facts() -> BTreeMap<String, BeadFact> {
    [
        fact("fx-s1", "組 s1"),
        fact("fx-s1.7", "組 s1.7"),
        fact("fx-s2", "組 s2"),
    ]
    .into_iter()
    .map(|f| (f.id.to_string(), f))
    .collect()
}

/// bead の組の鍵は一覧の組と同じ読み（いちばん近い epic の祖先・入れ子の epic は内の epic・epic の祖先が無ければ epic の外）。
#[test]
fn bvsel_key_of_matches_groups() {
    let rows = rows();
    assert_eq!(key_of("fx-s1.1", &rows), "fx-s1");
    assert_eq!(key_of("fx-s1.7.1", &rows), "fx-s1.7");
    assert_eq!(key_of("fx-s1.7", &rows), "fx-s1");
    assert_eq!(key_of("fx-s0", &rows), OUTSIDE_KEY);
    assert_eq!(key_of("fx-s9.1", &rows), OUTSIDE_KEY);
    let gs = groups(&rows, &cards(), &facts(), NOW);
    let mut seen = 0;
    for g in &gs {
        for r in &g.rows {
            assert_eq!(key_of(&r.id, &rows), g.key, "{} の組", r.id);
            seen += 1;
        }
    }
    assert_eq!(seen, 6, "open の行の全部を照らした");
}

/// 押した後の選びは、空の鍵か今と同じ鍵で解き、ほかは押した鍵。
#[test]
fn bvsel_pick_toggles() {
    assert_eq!(pick(None, "fx-s1").as_deref(), Some("fx-s1"));
    assert_eq!(pick(Some("fx-s1"), "fx-s1"), None);
    assert_eq!(pick(Some("fx-s1"), "fx-s2").as_deref(), Some("fx-s2"));
    assert_eq!(pick(Some("fx-s1"), ""), None);
    assert_eq!(pick(None, ""), None);
}

/// 札は epic を選んでいて組の鍵が違う時だけ薄く、輪は開いている bead の id と同じ時だけ・組の頭の輪は開いている bead が組の open の行。
#[test]
fn bvsel_dim_and_ring() {
    assert!(!dim(None, Some("fx-s1")));
    assert!(!dim(Some("fx-s1"), Some("fx-s1")));
    assert!(dim(Some("fx-s1"), Some("fx-s2")));
    assert!(dim(Some("fx-s1"), Some(OUTSIDE_KEY)));
    // 台帳が読めず組の鍵が分からない札は薄くしない。
    assert!(!dim(Some("fx-s1"), None));
    assert!(!dim(None, None));
    assert!(!ring(None, "fx-s1.1"));
    assert!(ring(Some("fx-s1.1"), "fx-s1.1"));
    assert!(!ring(Some("fx-s1.1"), "fx-s1.2"));
    let rows = rows();
    let gs = groups(&rows, &cards(), &facts(), NOW);
    let g1: &Lgroup = gs.iter().find(|g| g.key == "fx-s1").expect("組 s1");
    assert!(head_ring(g1, Some("fx-s1.2")));
    assert!(head_ring(g1, Some("fx-s1.1")));
    assert!(
        !head_ring(g1, Some("fx-s1.3")),
        "閉じた行は組の open の行でない"
    );
    assert!(!head_ring(g1, Some("fx-s2.1")));
    assert!(!head_ring(g1, None));
}

/// 見出しの chip は、すべての chip（数は板に出ている札の全部）の後に、札を持つ組を札の多い順（同じ数は鍵の順）に並べ、
/// 名は epic の短い題（epic の外は OUTSIDE）・台帳が読めなければ札は全部 epic の外。
#[test]
fn bvsel_chips_by_count() {
    let rows = rows();
    let cards = cards();
    let cols = columns(&cards, &rows, NOW);
    let keys = card_keys(&cards, &Reading::Known(rows.clone()));
    assert_eq!(keys.get("fx-s1.7.1").map(String::as_str), Some("fx-s1.7"));
    let got = chips(&cols, &keys, &rows, &facts());
    let want = vec![
        Chip {
            key: String::new(),
            name: ALL_EPICS.to_string(),
            n: 5,
        },
        Chip {
            key: "fx-s2".to_string(),
            name: "組 s2".to_string(),
            n: 2,
        },
        Chip {
            key: OUTSIDE_KEY.to_string(),
            name: OUTSIDE.to_string(),
            n: 1,
        },
        Chip {
            key: "fx-s1".to_string(),
            name: "組 s1".to_string(),
            n: 1,
        },
        Chip {
            key: "fx-s1.7".to_string(),
            name: "組 s1.7".to_string(),
            n: 1,
        },
    ];
    assert_eq!(got, want);
    let unknown = card_keys(&cards, &Reading::Unknown);
    assert!(unknown.is_empty());
    let got = chips(&cols, &unknown, &[], &BTreeMap::new());
    assert_eq!(got.len(), 2);
    assert_eq!(got[1].key, OUTSIDE_KEY);
    assert_eq!(got[1].n, 5);
    let no_epic_row = chips(&cols, &keys, &rows[1..], &facts());
    assert!(
        no_epic_row
            .iter()
            .any(|c| c.key == "fx-s1" && c.name == "fx-s1")
    );
    chips_tie_by_id(&cols, &rows);
}

/// (7) の縁: 同じ数の組は鍵の id の自然な順（fx-t.9 が fx-t.10 の前・字の順なら逆）で、組の無い札は epic の外。
fn chips_tie_by_id(cols: &[Column], rows: &[LedgerRow]) {
    let ids: Vec<String> = cols
        .iter()
        .flat_map(|c| c.cards.iter().map(|k| k.id.clone()))
        .collect();
    assert_eq!(ids.len(), 5);
    let tie: BTreeMap<String, String> = ids
        .iter()
        .take(4)
        .enumerate()
        .map(|(i, id)| {
            (
                id.clone(),
                if i % 2 == 0 { "fx-t.10" } else { "fx-t.9" }.to_string(),
            )
        })
        .collect();
    let order: Vec<(String, usize)> = chips(cols, &tie, rows, &facts())
        .into_iter()
        .map(|c| (c.key, c.n))
        .collect();
    let want = |k: &str, n: usize| (k.to_string(), n);
    assert_eq!(
        order,
        [
            want("", 5),
            want("fx-t.9", 2),
            want("fx-t.10", 2),
            want(OUTSIDE_KEY, 1)
        ]
    );
}

/// 一覧の行は吹き出しの口の印を持ち押すと吹き出しを開き、組の頭の名は epic を選び、選んだ組と開いた bead の組を開く。
/// 板は見出しに epic の chip を置き、札を選びの包みに入れる。App は選びの状態を context に置き、class は stylesheet に在る。
/// 選びの chip の規則は入れ物 .pchips の下に絞り、既存の .pchip（account board の project の chip と seat の pill）を替えない。
#[test]
fn bvsel_dom_wiring_text() {
    let list = read("src/ledgerlist.rs");
    let ldom = &list[list.find("mod dom {").expect("mod dom")..];
    for want in [
        "<div class=\"ll-row\" class:ring=ringed data-id=r.id.clone() data-pop-row=r.id.clone() on:click=press",
        "p.press(&id, Via::Row);",
        "<a class=\"ll-ls\" href=href on:click=stay title=r.title.clone()>",
        "if crate::board::plain_click(&e) {",
        "sel.epic.update(|e| *e = pick(e.as_deref(), &key))",
        "<div class=head_class class:esel=esel class:ring=ringed>",
        "folds.update(|f| f.set(&key, true));",
        "move || head_ring(&g, shown().as_deref())",
        "for key in [epic, bead].into_iter().flatten() {",
        "e.prevent_default();",
    ] {
        assert!(ldom.contains(want), "ledgerlist の DOM に {want} が無い");
    }
    let pipe = read("src/project/pipeline.rs");
    let pdom = &pipe[pipe.find("mod dom {").expect("mod dom")..];
    for want in [
        "section(BLOCK, head.into_any(), body.into_any())",
        "let Reading::Known(list) = &ledger_rows else {",
        "let all = beads.with(|b| chips(&cols, &keys, list, &facts(b)));",
        "sel.epic.update(|e| *e = pick(e.as_deref(), &key))",
        "pick_view(c, keys, kcard_view(c, mode(), clock))",
        "<div class=\"kpick\" class:dim=dimmed class:ring=ringed class:hov=hovered>{inner}</div>",
        "let key = keys.with_value(|k| k.get(&card.id).cloned());",
        "s.epic.with(|e| dim(e.as_deref(), key.as_deref()))",
    ] {
        assert!(pdom.contains(want), "pipeline の DOM に {want} が無い");
    }
    let board = read("src/board.rs");
    assert!(board.contains("provide_context(SelCtx::default());"));
    let css = read("style.css");
    for rule in [
        ".pchips {",
        ".pchips .pchip {",
        ".pchips .pchip b {",
        ".pchips .pchip.on {",
        ".pchips .pchip.on b {",
        ".kpick.dim { opacity: .3; }",
        ".kpick.ring > .kcard {",
        ".kpick.dim.ring { opacity: 1; }",
        "button.ll-gname {",
        ".ll-gh.esel {",
        ".ll-gh.ring {",
        ".ll-row.ring {",
    ] {
        assert!(css.contains(rule), "stylesheet に {rule} が無い");
    }
    chip_rules_scoped(&css);
}

/// 選びの chip の規則は入れ物 .pchips の下だけに在り、絞りの無い .pchip の規則は既存の 2 つの字のまま。
fn chip_rules_scoped(css: &str) {
    for bare in [".pchip.on", ".pchip b"] {
        assert!(
            !css.lines().any(|l| l.starts_with(bare)),
            "stylesheet に絞りの無い {bare} の規則が在る"
        );
    }
    let bases: Vec<&str> = css.lines().filter(|l| l.starts_with(".pchip {")).collect();
    assert_eq!(
        bases,
        [
            ".pchip { display: inline-flex; align-items: center; gap: 4px; height: 26px; padding: 0 var(--s2); border-radius: 999px; border: 1px solid var(--line); font-size: 12px; font-weight: 600; }",
            ".pchip { max-width: 100%; }",
        ],
        "絞りの無い .pchip の規則は既存の 2 つだけ"
    );
}
