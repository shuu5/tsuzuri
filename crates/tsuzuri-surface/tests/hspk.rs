//! 便 h-acct-spark の歯: HOME の口座 × 窓の 7 列（7 日の線の svg と測った時刻）・見出しと列の幅・語の辞書の 7 日。

use std::path::PathBuf;

use tsuzuri_contract::account::{AccountDoc, SPARK_SPAN_S, Spark, SparkLine, SparkPoint};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::home::{
    ACCT_HEADS, AcctRow, Measured, NONE, SPARK_H, SPARK_LINES, SPARK_W, home, measured, spark_svg,
};
use tsuzuri_surface::project::Body;
use tsuzuri_surface::vocab::vocab;

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

fn rows(doc: &AccountDoc) -> Vec<AcctRow> {
    match home(doc).accounts {
        Body::Filled(r) => r,
        other => panic!("口座の段が中身ありでない: {other:?}"),
    }
}

/// 辞書の字から鍵 spark の塊（`"spark": {` から閉じの `}` まで）を切り出す。
fn spark_block(text: &str) -> &str {
    let start = text.find("\n  \"spark\": {").expect("鍵 spark の塊");
    let rest = &text[start..];
    let end = rest.find('}').expect("塊の閉じ");
    &rest[..=end]
}

/// fixture の acct-1 の 7 日の線（電文の at 1790510400）。
const SVG_1: &str = concat!(
    r#"<svg viewBox="0 0 150 32" role="img" aria-label="acct-1 直近 7 日">"#,
    r#"<line x1="0" x2="150" y1="2.0" y2="2.0" stroke="var(--line-2)" stroke-dasharray="1 2"/>"#,
    r#"<polyline fill="none" stroke="var(--st-limit)" stroke-width="1.4" stroke-dasharray="3 2" points="128.5,23.8 149.9,21.6"/>"#,
    r#"<polyline fill="none" stroke="var(--band-beads)" stroke-width="1.6" points="146.4,26.6 148.1,20.2 149.9,18.2"/>"#,
    r#"<line x1="150.0" x2="150.0" y1="0" y2="32" stroke="var(--ink-3)"/></svg>"#,
);

/// (1) 見出しは 7 つ・図は 150 × 32・線は seven_day_model と five_hour の順・列の幅は stylesheet の 7 列・辞書の spark は 7 日。
#[test]
fn hspk_heads_width_and_vocab() {
    assert_eq!(
        ACCT_HEADS,
        [
            "accounts",
            "occupant",
            "five_hour",
            "seven_day",
            "seven_day_model",
            "spark",
            "measured_at"
        ]
    );
    assert_eq!((SPARK_W, SPARK_H), (150.0, 32.0));
    assert_eq!(SPARK_LINES.map(|(w, _)| w), ["seven_day_model", "five_hour"]);
    assert_eq!(SPARK_SPAN_S, 604_800);

    let css = read("style.css");
    assert!(css.contains(
        ".acct-grid { display: grid; grid-template-columns: 80px 156px repeat(3, minmax(0, 1fr)) 150px 76px;"
    ));
    let src = read("src/account/home.rs");
    for word in ["grid-template-columns", "ACCT_COLS"] {
        assert!(!src.contains(word), "home.rs に {word}");
    }
    assert!(src.contains("<div class=\"acct-grid\">"), "acct-grid の div は style を持たない");

    let term = vocab().term("spark").expect("鍵 spark");
    assert_eq!(term.label, "直近 7 日");
    for text in [&term.label, &term.note, &term.internal] {
        assert!(!text.contains("24"), "{text:?} に 24");
    }
    let surface = read("vocab.json");
    let mock = read("../../docs/design/mock3/vocab.json");
    let block = spark_block(&surface);
    assert!(block.contains("\"label\": \"直近 7 日\""), "{block}");
    assert!(!block.contains("24"), "{block}");
    assert_eq!(block, spark_block(&mock));
}

/// (2) fixture の acct-1 の線は SVG_1・退役は 100% の点線と今の縦線だけ・名は字の参照・7 日より前の点は x 0.0・読めない線は None。
#[test]
fn hspk_svg_on_fixture() {
    let doc = fixture();
    let r = rows(&doc);
    assert_eq!(r[0].spark.as_deref(), Some(SVG_1));
    let Reading::Known(accounts) = &doc.accounts else {
        panic!("fixture の accounts は読める");
    };
    let Reading::Known(s1) = &accounts[0].spark else {
        panic!("acct-1 の線は読める");
    };
    assert_eq!(spark_svg("acct-1", s1, doc.at), SVG_1);

    let retired = concat!(
        r#"<svg viewBox="0 0 150 32" role="img" aria-label="acct-3 直近 7 日">"#,
        r#"<line x1="0" x2="150" y1="2.0" y2="2.0" stroke="var(--line-2)" stroke-dasharray="1 2"/>"#,
        r#"<line x1="150.0" x2="150.0" y1="0" y2="32" stroke="var(--ink-3)"/></svg>"#,
    );
    assert_eq!(r[2].label, "acct-3");
    assert_eq!(r[2].spark.as_deref(), Some(retired));

    let odd = spark_svg("a<b&\"c", s1, doc.at);
    assert!(odd.contains(r#"aria-label="a&lt;b&amp;&quot;c 直近 7 日""#), "{odd}");

    let old = Spark {
        measured_at: Some(doc.at),
        lines: vec![SparkLine {
            window: "five_hour".to_string(),
            points: vec![
                SparkPoint {
                    at: doc.at - SPARK_SPAN_S - 3_600,
                    used_pct: 50,
                },
                SparkPoint {
                    at: doc.at,
                    used_pct: 100,
                },
            ],
        }],
    };
    let svg = spark_svg("x", &old, doc.at);
    assert!(svg.contains(r#"points="0.0,16.0 150.0,2.0""#), "{svg}");

    let mut d = doc.clone();
    if let Reading::Known(a) = &mut d.accounts {
        a[1].spark = Reading::Unknown;
    }
    assert_eq!(rows(&d)[1].spark, None);
}

/// (3) 測った時刻は窓が読めれば Known・読めなければ Unknown・線か時刻が無ければ Unknown の「―」・退役は Retired。
#[test]
fn hspk_measured_rules() {
    let doc = fixture();
    let got: Vec<Measured> = rows(&doc).into_iter().map(|r| r.measured).collect();
    assert_eq!(
        got,
        vec![
            Measured::Known("20:55 JST".to_string()),
            Measured::Known("20:53 JST".to_string()),
            Measured::Retired,
        ]
    );
    let Reading::Known(accounts) = &doc.accounts else {
        panic!("fixture の accounts は読める");
    };
    let a1 = &accounts[0];

    let mut r = a1.clone();
    r.usage = Reading::Unknown;
    assert_eq!(measured(&r, doc.at), Measured::Unknown("20:55 JST".to_string()));

    let mut r = a1.clone();
    r.spark = Reading::Unknown;
    assert_eq!(measured(&r, doc.at), Measured::Unknown(NONE.to_string()));

    let mut r = a1.clone();
    if let Reading::Known(s) = &mut r.spark {
        s.measured_at = None;
    }
    assert_eq!(measured(&r, doc.at), Measured::Unknown(NONE.to_string()));

    let mut r = a1.clone();
    if let Reading::Known(s) = &mut r.spark {
        s.measured_at = Some(1_790_424_000);
    }
    assert_eq!(measured(&r, doc.at), Measured::Known("09-26 21:00 JST".to_string()));

    let mut r = a1.clone();
    r.retired = true;
    assert_eq!(measured(&r, doc.at), Measured::Retired);
}
