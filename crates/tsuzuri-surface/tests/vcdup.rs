//! 語彙の鍵の重なりの歯（面・接頭辞 vcdup_・設計ノート surface-wave29b 行 g-vocab-dupkey・検証役 qv184 の名指し）。
//! 同じ欄に 2 度在る鍵と english と rephrase の両方に在る鍵は、読みが鍵と 2 つの在りかを書いた Err にする
//! （前は黙って後と english が勝ち、2 本の便が同じ鍵を足しても差は当たり歯も拾わなかった）。否定の見本は鍵の字 1 つだけを替える。
#![cfg(test)]

use tsuzuri_surface::vocab::{SOURCE, Vocab};

/// 見本の語彙（rephrase に鍵 a と b・english に鍵 c と d）。
fn sample(a: &str, b: &str, c: &str, d: &str) -> String {
    let re =
        |k: &str| format!(r#""{k}": {{"label": "語 {k}", "plain": "注 {k}", "orig": "原 {k}"}}"#);
    let en = |k: &str| {
        format!(r#""{k}": {{"label": "語 {k}", "note": "注 {k}", "internal": "内 {k}"}}"#)
    };
    format!(
        "{{\"rephrase\": {{{}, {}}}, \"english\": {{{}, {}}}}}",
        re(a),
        re(b),
        en(c),
        en(d)
    )
}

/// 置き場の vocab.json は読め、表の鍵の数は 2 欄の鍵の頭の行（字下げ 2 の `"鍵": {` の行）の数と同じ。
#[test]
fn vcdup_source_parses() {
    let vocab = Vocab::parse(SOURCE).expect("置き場の vocab.json は読める");
    let heads = SOURCE
        .lines()
        .filter(|l| l.starts_with("  \"") && l.ends_with(": {"))
        .count();
    assert!(heads > 400, "鍵の頭の行が少ない: {heads}");
    assert_eq!(vocab.keys().count(), heads);
}

/// 同じ欄に 2 度在る鍵（rephrase と english の見本）は Err。
#[test]
fn vcdup_same_column_refused() {
    assert_eq!(
        Vocab::parse(&sample("ra", "ra", "ec", "ed")),
        Err(
            "鍵 ra が 2 度在る（vocab.json の欄 rephrase と vocab.json の欄 rephrase）".to_string()
        )
    );
    assert_eq!(
        Vocab::parse(&sample("ra", "rb", "ec", "ec")),
        Err("鍵 ec が 2 度在る（vocab.json の欄 english と vocab.json の欄 english）".to_string())
    );
}

/// english と rephrase の両方に在る鍵は Err（english が勝たない）。
#[test]
fn vcdup_cross_column_refused() {
    assert_eq!(
        Vocab::parse(&sample("ra", "rb", "rb", "ed")),
        Err("鍵 rb が 2 度在る（vocab.json の欄 rephrase と vocab.json の欄 english）".to_string())
    );
}

/// 対照: 重なりの無い見本は 4 つの鍵の語・注・内部の名（rephrase は原語の行）を返す。
#[test]
fn vcdup_distinct_keys_read() {
    let vocab = Vocab::parse(&sample("ra", "rb", "ec", "ed")).expect("重なりの無い見本");
    assert_eq!(vocab.keys().collect::<Vec<_>>(), ["ec", "ed", "ra", "rb"]);
    let term = vocab.term("rb").expect("鍵 rb");
    assert_eq!(
        (
            term.label.as_str(),
            term.note.as_str(),
            term.internal.as_str()
        ),
        ("語 rb", "注 rb", "\n原語: 原 rb")
    );
    let term = vocab.term("ed").expect("鍵 ed");
    assert_eq!(
        (
            term.label.as_str(),
            term.note.as_str(),
            term.internal.as_str()
        ),
        ("語 ed", "注 ed", "内 ed")
    );
}
