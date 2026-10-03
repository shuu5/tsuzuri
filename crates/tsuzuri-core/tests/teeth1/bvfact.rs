//! 行 c-bead-facts の歯（中核・接頭辞 bvfact_）: 台帳の字から bead の事実の一覧（起票の時刻・短い題・概要・
//! blocks の相手）を組む。台帳の字は歯の中で組む（fixture の file は使わない）。
#![cfg(test)]

use serde_json::{Value, json};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadFact, BeadId};
use tsuzuri_core::ledger::facts;
use tsuzuri_core::ledger::facts::SUMMARY_MAX;

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("見本の bead id")
}

/// 題と本文と metadata（無ければ Null）を持つ open の task の 1 本。
fn line(id: &str, title: &str, description: &str, metadata: Value) -> Value {
    let mut b = json!({
        "id": id,
        "title": title,
        "description": description,
        "status": "open",
        "issue_type": "task",
        "created_at": "2026-09-27T00:00:00Z",
    });
    if !metadata.is_null() {
        b["metadata"] = metadata;
    }
    b
}

/// 台帳の字から組んだ行（読めなければ落ちる）。
fn rows(beads: &[Value]) -> Vec<BeadFact> {
    match facts(&Value::Array(beads.to_vec()).to_string()).rows {
        Reading::Known(rows) => rows,
        Reading::Unknown => panic!("読める台帳の字が Unknown"),
    }
}

/// 1 本だけの台帳の字から組んだ行。
fn one(bead: Value) -> BeadFact {
    let mut got = rows(&[bead]);
    assert_eq!(got.len(), 1, "{got:?}");
    got.remove(0)
}

#[test]
fn bvfact_short_from_metadata() {
    let set = one(line(
        "fx.1",
        "便 r4-table — 長い題",
        "",
        json!({"short": "表の上限", "touches": ["A-1"]}),
    ));
    assert_eq!(set.short, "表の上限");
    assert!(set.short_set, "metadata の鍵 short の字は short_set が真");
    // metadata が object を JSON にした字でも読む。
    let text = one(line(
        "fx.2",
        "題",
        "",
        json!("{\"short\":\"字の metadata\"}"),
    ));
    assert_eq!(text.short, "字の metadata");
    assert!(text.short_set);
    // 鍵 short が空の字なら機械で作る。
    let empty = one(line("fx.3", "問い — 色", "", json!({"short": ""})));
    assert_eq!(empty.short, "問い");
    assert!(!empty.short_set, "空の short は機械で作る");
}

#[test]
fn bvfact_short_fallback_row_id() {
    let row = one(line(
        "fx.1",
        "便 r4-table — 表の上限を直す",
        "",
        Value::Null,
    ));
    assert_eq!(row.short, "r4-table");
    assert!(!row.short_set, "題から作った字は short_set が偽");
    let bare = one(line("fx.2", "便 e-reap", "", Value::Null));
    assert_eq!(bare.short, "e-reap", "区切りの無い便の題は頭の後の字の全体");
}

#[test]
fn bvfact_short_fallback_dash() {
    let dash = one(line("fx.1", "  問い — 新しい方の問い", "", Value::Null));
    assert_eq!(dash.short, "問い", "「—」の前の字の前後の空白を除く");
    assert!(!dash.short_set);
    let whole = one(line("fx.2", "問い: 色を替えてよいか ", "", Value::Null));
    assert_eq!(
        whole.short, "問い: 色を替えてよいか",
        "「—」が無ければ題の全体"
    );
    let lead = one(line("fx.3", "— だけの題", "", Value::Null));
    assert_eq!(lead.short, "— だけの題", "「—」の前が空なら題の全体");
}

#[test]
fn bvfact_summary_from_observation() {
    let description = "## memo\n### 出所\n出所の行。\n### 観測\n\n  観測の 1 行目。  \n2 行目\n";
    let row = one(line("fx.1", "題", description, Value::Null));
    assert_eq!(row.summary, "観測の 1 行目。");
    // 観測の下の行も頭の「概要 =」を外す。
    let label = one(line(
        "fx.2",
        "題",
        "前置き\n### 観測\n概要 = 見た事",
        Value::Null,
    ));
    assert_eq!(label.summary, "見た事");
}

#[test]
fn bvfact_summary_skips_heading_and_label() {
    let row = one(line(
        "fx.1",
        "題",
        "\n## 見出し\n\n概要 =   板を 5 列にする\n技術 = 列の型\n",
        Value::Null,
    ));
    assert_eq!(row.summary, "板を 5 列にする");
    let plain = one(line(
        "fx.2",
        "題",
        "# 頭\n最初の段落の行\n次の行",
        Value::Null,
    ));
    assert_eq!(plain.summary, "最初の段落の行");
    let none = one(line("fx.3", "題", "# 見出しだけ\n\n", Value::Null));
    assert_eq!(none.summary, "", "行が無ければ空の字");
}

#[test]
fn bvfact_summary_cut_120() {
    assert_eq!(SUMMARY_MAX, 120);
    let long = "あ".repeat(SUMMARY_MAX + 5);
    let row = one(line("fx.1", "題", &format!("概要 = {long}"), Value::Null));
    assert_eq!(row.summary.chars().count(), SUMMARY_MAX);
    assert_eq!(row.summary, "あ".repeat(SUMMARY_MAX));
    let short = "a".repeat(SUMMARY_MAX);
    assert_eq!(one(line("fx.2", "題", &short, Value::Null)).summary, short);
}

#[test]
fn bvfact_blocks_all_targets() {
    let mut b = line("fx.1", "題", "", Value::Null);
    b["dependencies"] = json!([
        {"depends_on_id": "fx", "type": "parent-child"},
        {"depends_on_id": "fx.3", "type": "blocks"},
        {"depends_on_id": "fx.2", "type": "blocks"},
        {"depends_on_id": "fx.9", "type": "relates-to"},
    ]);
    assert_eq!(one(b).blocks, vec![bead("fx.3"), bead("fx.2")]);
    assert!(one(line("fx.4", "題", "", Value::Null)).blocks.is_empty());
}

#[test]
fn bvfact_created_epoch() {
    let row = one(line("fx.1", "題", "", Value::Null));
    assert_eq!(row.created_at, Some(1_790_467_200));
    let mut bad = line("fx.2", "題", "", Value::Null);
    bad["created_at"] = json!("2026-09-27");
    assert_eq!(one(bad).created_at, None, "読めない時刻は None");
    let mut gone = line("fx.3", "題", "", Value::Null);
    gone.as_object_mut().expect("object").remove("created_at");
    assert_eq!(one(gone).created_at, None, "欄の無い時刻は None");
}

#[test]
fn bvfact_unknown_when_unreadable() {
    assert_eq!(facts("").rows, Reading::Unknown);
    assert_eq!(facts("{").rows, Reading::Unknown);
    assert_eq!(facts("{}").rows, Reading::Unknown, "配列でない字");
    assert_eq!(facts("[]").rows, Reading::Known(vec![]), "0 件は読めた");
}

#[test]
fn bvfact_rows_skip_tombstone_keep_order() {
    let mut gone = line("fx.2", "消えた", "", Value::Null);
    gone["status"] = json!("tombstone");
    let got = rows(&[
        line("fx.3", "題 3", "", Value::Null),
        gone,
        line("bad id", "形の合わない id", "", Value::Null),
        line("fx.1", "題 1", "", Value::Null),
    ]);
    let ids: Vec<&str> = got.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(
        ids,
        ["fx.3", "fx.1"],
        "tombstone と形の合わない id を外し台帳の順"
    );
}
