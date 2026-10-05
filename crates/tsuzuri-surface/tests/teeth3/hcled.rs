//! 便 h-cards-led の歯: account board の台帳の処理状況の表の project の欄の hover の card（見本の ledCard）と、
//! DOM の部分の字と、この file の歯の名。
#![cfg(test)]

use crate::common::read;
use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::ledger::{Sort, order, table};
use tsuzuri_surface::account::cards::led_card;
use tsuzuri_surface::widgets::hover::Card;

fn fixture() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

/// (1) 表の各行の card は電文の projects の index の位置の行を led_card に渡した値で、ほかの欄と並びは変わらない。
#[test]
fn hcled_rows_carry_cards() {
    let doc = fixture();
    for sort in Sort::ALL {
        let t = table(&doc, sort);
        assert_eq!(t.sort, sort);
        assert_eq!(t.order(), order(&doc.projects, sort), "{sort:?}");
        assert_eq!(t.rows.len(), doc.projects.len());
        for row in &t.rows {
            let p = &doc.projects[row.index];
            assert_eq!(row.card, led_card(p), "{sort:?} の {}", row.name);
            assert_eq!(row.name, p.name);
        }
    }

    let t = table(&doc, Sort::Judge);
    assert_eq!(t.names(), vec!["proj-a", "proj-b", "proj-c"]);
    assert_eq!(t.order(), vec![0, 1, 2]);
    judge_rows(t);
}

/// 判定の順に並べた表の proj-a の行の card の字と、台帳の無い 2 行の card を見る。
fn judge_rows(t: tsuzuri_surface::account::ledger::LedTable) {
    let a = &t.rows[0];
    assert!(matches!(a.cells, Reading::Known(_)));
    assert_eq!(a.class, "jrow j-ok");
    assert_eq!(a.card.title, "proj-a · 台帳の処理状況");
    assert_eq!(a.card.kind, "↑+2 24h · ↑+4 7d · closed/日 0.9");
    assert_eq!(a.card.value, "task 9（ready 5 / blocked 3）");
    // 時点の後の時刻の字は固めない（後の行が時刻の書き方を替えても直さずに済む）。
    assert!(
        a.card.src.starts_with("bd list --all の bead · 時点 "),
        "{}",
        a.card.src
    );
    assert_eq!(
        a.card.more,
        vec![
            "memo 3 · stale 1".to_string(),
            "question 2 · epic 2".to_string(),
            "lead p50 4.0d / p90 12d".to_string()
        ]
    );

    for (row, name) in t.rows[1..].iter().zip(["proj-b", "proj-c"]) {
        assert_eq!(row.name, name);
        assert_eq!(row.cells, Reading::Unknown);
        assert_eq!(row.class, "jrow j-none");
        assert_eq!(
            row.card,
            Card {
                title: format!("{name} · 台帳なし"),
                kind: "台帳の処理状況".to_string(),
                value: "測れていない".to_string(),
                src: "bd list --all の bead".to_string(),
                more: Vec::new(),
            }
        );
    }
}

/// ledger.rs の字 `mod dom {` より後の部分。
fn dom_part() -> String {
    let text = read("src/account/ledger.rs");
    let at = text.find("mod dom {").expect("ledger.rs に mod dom が在る");
    text[at..].to_string()
}

/// 関数の本体（`fn <name>(` から次の `fn ` の宣言の手前まで）。
fn fn_body(text: &str, name: &str) -> String {
    let head = format!("fn {name}(");
    let at = text
        .find(&head)
        .unwrap_or_else(|| panic!("fn {name} が無い"));
    let rest = &text[at + head.len()..];
    let end = rest.find("\n    fn ").unwrap_or(rest.len());
    text[at..at + head.len() + end].to_string()
}

/// (2) row_view の project の欄の tag はどれも tabindex と card を持ち、mod dom は hover の attach を使う。
#[test]
fn hcled_dom_wiring() {
    let dom = dom_part();
    assert!(
        dom.contains("use crate::widgets::hover::attach;"),
        "mod dom の use に crate::widgets::hover::attach が無い"
    );
    let row = fn_body(&dom, "row_view");
    let head = "<div class=\"c-p\"";
    let tags: Vec<&str> = row
        .match_indices(head)
        .map(|(i, _)| {
            let rest = &row[i..];
            &rest[..rest.find('>').expect("tag が > で閉じる")]
        })
        .collect();
    assert!(!tags.is_empty(), "row_view に project の欄の tag が無い");
    for t in &tags {
        assert!(t.contains("tabindex=\"0\""), "tag {t} に tabindex が無い");
        assert!(t.contains("use:attach="), "tag {t} に use:attach= が無い");
    }
}

/// (4) この file の歯の名は hcled_ で始まり、残りの字は filter の語を含まない。
#[test]
fn hcled_names_clean() {
    const WORDS: &[&str] = &[
        "aaround_", "accept_", "account_", "acctcore_", "acctdoc_", "accthb_", "accthome_",
        "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
        "afocus_", "aord_", "apop_", "askcard_", "athr_", "batchpanel_", "board_min_", "bport_",
        "brand_", "btuck_", "cadopt_", "cgdom_", "contract_form_", "cround_", "csled_", "denv_",
        "flight_", "fmark_", "frame_", "fserve_", "fstop_", "gapspage_", "ghb_", "glabel_",
        "gnav_", "graph_", "gsum_", "gtuck_", "gview_", "hbconf_", "hbpost_", "hbproc_",
        "hbroute_", "hcard_", "hcnx_", "hcproj_", "hcsess_", "hfig_", "hook_", "hruling_",
        "hsblock_", "hsderive_", "hspage_", "hsym_", "iclose_", "ilink_", "kcli_", "klink_",
        "lcard_", "ledgerblock_", "lhome_", "lspark_", "mapview_", "mkeys_", "mlink_", "mstore_",
        "mtree_", "nact_", "nbatch_", "ncard_", "nextstep_", "nodepage_", "nstall_", "nsum_",
        "nsumw_", "ntime_", "nxact_", "parts_", "pclosed_", "pfold_", "pipe_", "plimit_",
        "pmore_", "project_", "ptitle_", "pwhole_", "qblock_", "qgate_", "qkey_", "question_",
        "rhold_", "runsdoc_", "saxis_", "seatblock_", "seatcard_", "server_", "sesplit_",
        "shb_", "skeleton_", "smore_", "stage_", "stats_", "steady_", "sxaxis_", "ticker_",
        "tipx_", "topbar_", "tz_", "urpanel_", "uword_", "wstrip_",
    ];
    let text = read("tests/teeth3/hcled.rs");
    let lines: Vec<&str> = text.lines().collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines[i + 1..]
            .iter()
            .find(|l| l.trim_start().starts_with("fn "))
            .expect("test の属性の後に fn が在る");
        let name = decl.trim_start()["fn ".len()..]
            .split('(')
            .next()
            .unwrap_or_default()
            .to_string();
        names.push(name);
    }
    assert!(names.len() >= 3, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("hcled_")
            .unwrap_or_else(|| panic!("歯の名 {name} が hcled_ で始まらない"));
        for w in WORDS {
            assert!(!rest.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
