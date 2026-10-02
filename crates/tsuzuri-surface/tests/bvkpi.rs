//! 行 g-list-head の歯（接頭辞 bvkpi_）: 台帳 open の一覧の見出しの種類の切り替えと探す欄の絞りと、指標の小さな数と
//! 14 日の burndown の図（判断の記録 ADR-27 の決定 (8)・要件 FR13）。台帳の行と札は歯の中で組む（bead の id の接頭辞は fx-k）。
//! 指標は面の歯の fixture tests/fixtures/surface/ledger-stats.json の組 filled を読む。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::{PipelineCard, Reading, Stage};
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::{BeadFact, BeadId, LedgerRow, MEMO_LABEL, QUESTION_LABEL};
use tsuzuri_contract::stats::LedgerStats;
use tsuzuri_contract::wire;
use tsuzuri_surface::ledgerlist::{
    KINDS, KPI_UNKNOWN_KEY, Kind, Lgroup, NO_MATCH, SEARCH_HINT, filtered, filtering, groups, kpi,
    kpi_text, kpi_unknown, row_matches,
};
use tsuzuri_surface::project::ledger::{BURN_H, BURN_W, burn_svg, burndown};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::label;

const NOW: u64 = 1_790_510_400;

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn row(id: &str, kind: &str, title: &str, labels: &[&str]) -> LedgerRow {
    LedgerRow {
        id: BeadId::new(id).expect("id"),
        kind: kind.to_string(),
        title: title.to_string(),
        status: "open".to_string(),
        updated_at: NOW,
        parent: None,
        labels: labels.iter().map(|l| (*l).to_string()).collect(),
    }
}

/// 台帳: epic k1（便 2 と memo）・epic k2（問い 1）・epic の外の便。短い題は便 k1.1 だけが事実に在る。
fn sample() -> Vec<Lgroup> {
    let rows = vec![
        row("fx-k1", "epic", "一つ目の epic", &[]),
        row("fx-k1.1", "task", "便 a-row — 板の札を細くする", &[]),
        row("fx-k1.2", "task", "便 b-row — Blocked を分ける", &[]),
        row("fx-k1.3", "task", "控えの memo の題", &[MEMO_LABEL]),
        row("fx-k2", "epic", "二つ目の epic", &[]),
        row("fx-k2.1", "task", "問いの題 Width", &[QUESTION_LABEL]),
        row("fx-x1", "task", "外の便", &[]),
    ];
    let cards = vec![PipelineCard {
        contract: BeadId::new("fx-k1.1").expect("id"),
        runs: 2,
        stage: Stage::Running,
        reason: None,
        account: None,
        since: Some(NOW - 60),
        ci: None,
    }];
    let fact = BeadFact {
        id: BeadId::new("fx-k1.1").expect("id"),
        created_at: None,
        short: "札の幅".to_string(),
        short_set: true,
        summary: String::new(),
        blocks: Vec::new(),
    };
    let facts = BTreeMap::from([("fx-k1.1".to_string(), fact)]);
    groups(&rows, &cards, &facts, NOW)
}

fn ids(gs: &[Lgroup]) -> Vec<(String, Vec<String>)> {
    gs.iter()
        .map(|g| (g.key.clone(), g.rows.iter().map(|r| r.id.clone()).collect()))
        .collect()
}

fn filled() -> LedgerStats {
    let mut sets: BTreeMap<String, LedgerStats> =
        wire::decode(&read("../../tests/fixtures/surface/ledger-stats.json")).expect("fixture");
    sets.remove("filled").expect("組 filled")
}

/// (2) 種類の切り替えの表はすべて・便・memo・問いの 4 つ（合う行の種類つき）で、絞りが効くのは種類がすべてで
/// ないか探す字が空白だけでない時。
#[test]
fn bvkpi_kind_table_and_filtering() {
    let table: Vec<(Kind, &str, Option<NodeKind>)> = KINDS.to_vec();
    assert_eq!(
        table,
        [
            (Kind::All, "すべて", None),
            (Kind::Task, "便", Some(NodeKind::Task)),
            (Kind::Memo, "memo", Some(NodeKind::Memo)),
            (Kind::Question, "問い", Some(NodeKind::Question)),
        ]
    );
    assert!(!filtering(Kind::All, ""));
    assert!(!filtering(Kind::All, "  \t"));
    assert!(filtering(Kind::All, " a "));
    assert!(filtering(Kind::Memo, ""));
    assert_eq!(SEARCH_HINT, "題・id で探す");
}

/// (3) 行の合い: 種類は表の種類と同じ行だけ・探す字は前後の空白を除き大文字小文字を区別せず、短い題・題の全体・
/// id のどれかに含まれれば合う。すべてと空の探す字はどの行にも合う。
#[test]
fn bvkpi_search_short_title_id() {
    let gs = sample();
    let all: Vec<_> = gs.iter().flat_map(|g| g.rows.iter()).collect();
    let hit = |kind: Kind, q: &str| -> Vec<&str> {
        all.iter()
            .filter(|r| row_matches(r, kind, q))
            .map(|r| r.id.as_str())
            .collect()
    };
    assert_eq!(hit(Kind::All, "").len(), all.len());
    assert_eq!(hit(Kind::Task, ""), ["fx-k1.1", "fx-k1.2", "fx-x1"]);
    assert_eq!(hit(Kind::Memo, ""), ["fx-k1.3"]);
    assert_eq!(hit(Kind::Question, ""), ["fx-k2.1"]);
    assert_eq!(hit(Kind::All, " BLOCKED "), ["fx-k1.2"]);
    assert_eq!(hit(Kind::All, "width"), ["fx-k2.1"]);
    assert_eq!(hit(Kind::All, "FX-X1"), ["fx-x1"]);
    // 前後の空白（字の中に無い空白と tab）を除いて探す。
    assert_eq!(hit(Kind::All, "  FX-X1\t"), ["fx-x1"]);
    assert_eq!(hit(Kind::Memo, "便"), Vec::<&str>::new());
    // 短い題（事実の short）と題の全体の両方を探す: 短い題だけの字と題の全体だけの字のどちらでも合う。
    assert_eq!(hit(Kind::Task, "札の幅"), ["fx-k1.1"]);
    assert_eq!(hit(Kind::Task, "細く"), ["fx-k1.1"]);
}

/// (4) 絞った組: 絞りが効いていなければ組のまま・効いていれば合う行だけを残し、合う行の無い組は出さない。
/// 組の頭の数（閉じた数・全部の数・札の数）は替えない。
#[test]
fn bvkpi_filtered_keeps_matching_groups() {
    let gs = sample();
    assert_eq!(filtered(gs.clone(), Kind::All, " "), gs);
    let memo = filtered(gs.clone(), Kind::Memo, "");
    assert_eq!(
        ids(&memo),
        [("fx-k1".to_string(), vec!["fx-k1.3".to_string()])]
    );
    let k1 = gs.iter().find(|g| g.key == "fx-k1").expect("k1");
    assert_eq!(
        (memo[0].closed, memo[0].total, memo[0].live()),
        (k1.closed, k1.total, k1.live())
    );
    assert_eq!(memo[0].live(), 1);
    let task = filtered(gs.clone(), Kind::Task, "");
    let keys: Vec<&str> = task.iter().map(|g| g.key.as_str()).collect();
    assert_eq!(keys, ["fx-k1", "-"]);
    assert!(filtered(gs.clone(), Kind::All, "どこにも無い字").is_empty());
    assert_eq!(NO_MATCH, "合う行なし");
    // 縁: 閉じた行を持つ組と open の行の無い組。絞りが効かなければ組のまま（行の無い組も残す）、効けば閉じた数は替えない。
    let mut edge = gs;
    let at = edge.iter().position(|g| g.key == "fx-k1").expect("k1");
    edge[at].closed = 2;
    let bare = Lgroup {
        key: "fx-k9".to_string(),
        rows: Vec::new(),
        ..edge[at].clone()
    };
    edge.push(bare);
    assert_eq!(filtered(edge.clone(), Kind::All, " "), edge);
    let memo = filtered(edge, Kind::Memo, "");
    assert_eq!((memo.len(), memo[0].closed), (1, 2));
}

/// (5) 見出しの小さな数は open の便・memo・問いの数と純減 24h（負は −）の字で、14 日の図は台帳の block の burndown の
/// svg で sparkline でない。指標の口が読めない・まだ分からない・電文が読めない時は kpi_unknown の字（指標と語の鍵
/// KPI_UNKNOWN_KEY の「まだ分からない」）で図は無い。
#[test]
fn bvkpi_kpi_line_and_burn() {
    let s = filled();
    assert_eq!(kpi_text(&s), "便 5 · memo 3 · 問い 2 · 純減 24h −2");
    let body = Fetched::Body(wire::encode(&Reading::Known(s.clone())).expect("enc"));
    let (text, svg) = kpi(&body);
    assert_eq!(text, kpi_text(&s));
    assert_eq!(svg, Some(burn_svg(&burndown(&s.days, BURN_W, BURN_H))));
    let svg = svg.unwrap_or_default();
    assert!(svg.starts_with("<svg class=\"lburn\""), "{svg}");
    assert!(!svg.contains("lspark"), "{svg}");
    let unknown = Fetched::Body(wire::encode(&Reading::<LedgerStats>::Unknown).expect("enc"));
    for bad in [
        Fetched::NotRead,
        Fetched::Failed,
        unknown,
        Fetched::Body("{".into()),
    ] {
        assert_eq!(kpi(&bad), (kpi_unknown(), None));
    }
    assert_eq!(KPI_UNKNOWN_KEY, "pop_unknown");
    assert_eq!(label(KPI_UNKNOWN_KEY), "まだ分からない");
    assert_eq!(kpi_unknown(), "指標 まだ分からない");
}

/// (6) DOM の字: 見出しは指標の口を読み、種類の button と探す欄を signal に結ぶ。一覧は絞った組を描き、絞りが効いて
/// いる間は組を開く。class は stylesheet に在る。
#[test]
fn bvkpi_dom_wiring_text() {
    let list = read("src/ledgerlist.rs");
    let dom = &list[list.find("mod dom {").expect("mod dom")..];
    for want in [
        "crate::net::read(ledger::METRICS_PATH)",
        "metrics.with(kpi)",
        "on:click=move |_| kind.set(k)",
        "on:input=move |ev| query.set(event_target_value(&ev))",
        "let active = filtering(k, &q);",
        "let shown = filtered(groups, k, &q);",
        "group_view(g, folds, active)",
        "move || active || folds.with(|f| f.open(&key, initial))",
        "{head_line(kind, query)}",
    ] {
        assert!(dom.contains(want), "ledgerlist の DOM に {want} が無い");
    }
    let css = read("style.css");
    for class in [
        "ll-head", "ll-l1", "ll-l2", "ll-kchip", "ll-spark", "ll-seg", "ll-q",
    ] {
        assert!(
            css.contains(&format!(".{class} ")),
            "stylesheet に .{class} が無い"
        );
        assert!(dom.contains(class), "ledgerlist の DOM に {class} が無い");
    }
    assert!(css.contains(".ll-seg button.on "));
}
