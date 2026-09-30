//! 便 h-led-spark の歯: account board の台帳の表の詳しくの段の 14 日の sparkline
//! （見本の ledMore の `mi sp` と ledger.js の spark14）。字は project の ledger の module の spark と spark_svg で組む。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::LedgerStats;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::ledger::{Cells, LedRow, MORE, Sort, table};
use tsuzuri_surface::project::ledger::{SPARK_H, SPARK_W, spark, spark_svg};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

fn known(row: &LedRow) -> &Cells {
    match &row.cells {
        Reading::Known(c) => c,
        Reading::Unknown => panic!("{} の cells が Unknown", row.name),
    }
}

fn row<'a>(rows: &'a [LedRow], name: &str) -> &'a LedRow {
    rows.iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("{name} の行が無い"))
}

/// polyline の class の points の字。
fn points<'a>(svg: &'a str, class: &str) -> &'a str {
    let head = format!(r#"<polyline class="{class}" fill="none" points=""#);
    let at = svg.find(&head).unwrap_or_else(|| panic!("{class} の polyline が無い: {svg}"));
    let rest = &svg[at + head.len()..];
    &rest[..rest.find('"').expect("points の字が閉じる")]
}

/// (1) 台帳が Known の行の spark は spark_svg に spark（days・SPARK_W・SPARK_H）を渡した字と同じ。
#[test]
fn lspark_known_equals_svg_of_days() {
    let doc = fixture();
    let t = table(&doc, Sort::Judge);
    let mut seen = 0;
    for r in &t.rows {
        if let (Reading::Known(c), Reading::Known(s)) = (&r.cells, &doc.projects[r.index].ledger) {
            assert_eq!(c.spark, spark_svg(&spark(&s.days, SPARK_W, SPARK_H)), "{}", r.name);
            seen += 1;
        }
    }
    assert!(seen >= 1, "台帳が Known の行が fixture に在る");
    for sort in Sort::ALL {
        let other = table(&doc, sort);
        for r in &other.rows {
            assert_eq!(r.cells, row(&t.rows, &r.name).cells, "{} の並べで {}", sort.key(), r.name);
        }
    }
}

/// (2) judge の並べの 1 行目（proj-a）の spark は class lspark の svg で、2 本の線の点の字は spark14 の式から出る 14 点。
#[test]
fn lspark_first_line_points() {
    let t = table(&fixture(), Sort::Judge);
    let first = &t.rows[0];
    assert_eq!(first.name, "proj-a");
    let svg = &known(first).spark;
    assert!(svg.starts_with(r#"<svg class="lspark" viewBox="0 0 120 24""#), "{svg}");
    assert!(svg.contains(r#"role="img""#) && svg.contains("aria-label="), "{svg}");
    assert!(svg.ends_with("</svg>"), "{svg}");

    let created = points(svg, "ls-created");
    assert_eq!(created.split(' ').count(), 14, "{created}");
    assert!(created.starts_with("1.0,12.0 10.1,22.0 19.2,2.0 "), "{created}");
    assert!(created.ends_with(" 119.0,22.0"), "{created}");

    let closed = points(svg, "ls-closed");
    assert_eq!(closed.split(' ').count(), 14, "{closed}");
    assert!(closed.starts_with("1.0,22.0 10.1,12.0 "), "{closed}");
    assert!(closed.ends_with(" 119.0,12.0"), "{closed}");
}

/// (3) days が空の台帳の行の spark は空の days の spark の字・台帳が Unknown の行の cells は Unknown のまま。
#[test]
fn lspark_empty_days_and_unknown() {
    let mut doc = fixture();
    let empty: LedgerStats = match &mut doc.projects[0].ledger {
        Reading::Known(s) => {
            s.days.clear();
            s.clone()
        }
        Reading::Unknown => panic!("fixture の proj-a の台帳が Known でない"),
    };
    let t = table(&doc, Sort::Judge);
    let a = known(row(&t.rows, "proj-a"));
    assert_eq!(a.spark, spark_svg(&spark(&empty.days, SPARK_W, SPARK_H)));
    assert_eq!(a.spark, spark_svg(&spark(&[], SPARK_W, SPARK_H)));
    for name in ["proj-b", "proj-c"] {
        assert_eq!(row(&t.rows, name).cells, Reading::Unknown, "{name}");
    }
}

/// (4) 詳しくの段の 5 つの鍵と組は変わらない。
#[test]
fn lspark_more_keeps_five() {
    assert_eq!(MORE, ["l_ready", "l_blocked", "l_stale", "l_net7", "l_lead"]);
    let t = table(&fixture(), Sort::Judge);
    let got = row(&t.rows, "proj-a").more();
    let want = [
        ("l_ready", "5"),
        ("l_blocked", "3"),
        ("l_stale", "1"),
        ("l_net7", "+4"),
        ("l_lead", "4.0d"),
    ];
    assert_eq!(got.len(), want.len());
    for ((k, v), (wk, wv)) in got.iter().zip(want) {
        assert_eq!((*k, v.as_str()), (wk, wv));
    }
}

/// (5) DOM の部分に `class="mi sp"` と inner_html= が在り、足す class は style.css に在る。
#[test]
fn lspark_dom_after_more() {
    let src = read("src/account/ledger.rs");
    let at = src.find("mod dom {").expect("ledger.rs に mod dom { が在る");
    let dom = &src[at..];
    assert!(dom.contains(r#"class="mi sp""#), "DOM に class=\"mi sp\" が在る");
    assert!(dom.contains("inner_html="), "DOM に inner_html= が在る");
    let more = &dom[dom.find("fn more_view").expect("more_view が在る")..];
    let items = more.find(r#"class="mi""#).expect("項の span が在る");
    let sp = more.find(r#"class="mi sp""#).expect("sparkline の span が在る");
    assert!(items < sp, "sparkline は 5 つの項の後");

    let css = read("style.css");
    for class in ["mi", "sp", "lspark", "ls-created", "ls-closed"] {
        let dot = format!(".{class}");
        let found = css.match_indices(&dot).any(|(i, _)| {
            css[i + dot.len()..]
                .chars()
                .next()
                .is_some_and(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        });
        assert!(found, "style.css に .{class} が在る");
    }
}

/// 着地済みの filter の語と同じ波の行の接頭辞。
const FILTERS: [&str; 65] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
];

/// (8) この file の test の fn の名はどれも lspark_ で始まり、残りの字は filter の語を含まない。
#[test]
fn lspark_names_avoid_filters() {
    let src = read("tests/ledspark.rs");
    let lines: Vec<&str> = src.lines().collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() != "#[test]" {
            continue;
        }
        let f = lines[i + 1..]
            .iter()
            .find(|l| l.trim_start().starts_with("fn "))
            .expect("#[test] の後に fn が在る");
        let name = f.trim_start()["fn ".len()..]
            .split('(')
            .next()
            .expect("fn の名")
            .to_string();
        names.push(name);
    }
    assert!(names.len() >= 6, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("lspark_")
            .unwrap_or_else(|| panic!("{name} は lspark_ で始まる"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含まない");
        }
    }
}
