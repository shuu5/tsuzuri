//! 表示の型で概要を 1 つ選ぶ部品と畳みの歯（行 g-sum-pick・接頭辞 mpick_・判断の記録 ADR-30 決定 (2)(3)・規則の行 R-19）。
//! 2 つの概要を違う字にした fixture を、2 つの表示の型で選びの関数と個別の頁の概要の箱に通し、両向きで断言する。
#![cfg(test)]

use std::path::PathBuf;

use crate::common::{E, P, X};
use tsuzuri_contract::graph::AroundDoc;
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::node::{self, SUMMARY_MARKED};
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::sumpick::{
    BODY_KEY, ELLIPSIS, ENG_KEY, PLAIN_KEY, POP_SUM_MAX, Picked, fold, other, pick,
};

fn picked(key: &'static str, text: &str, marked: bool) -> Option<Picked> {
    Some(Picked {
        key,
        text: text.to_string(),
        marked,
    })
}

/// 中心の節点の plain と eng を替えた近傍の電文（fixture は着地した歯 gsum と同じ file）。
fn around(plain: Option<&str>, eng: Option<&str>) -> AroundDoc {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/surface/around-doc.json");
    let text = std::fs::read_to_string(&path).expect("around-doc.json が読める");
    let mut doc: AroundDoc = wire::decode(&text).expect("around-doc.json が電文として読める");
    let c = doc
        .rows
        .iter_mut()
        .find(|r| r.col == 0)
        .expect("fixture の中心の行");
    c.node.plain = plain.map(str::to_string);
    c.node.eng = eng.map(str::to_string);
    doc
}

/// (1) 2 つの字が違う時、初心者は非エンジニア向けの字だけ・経験者はエンジニア向けの字だけを印なしで返す。
#[test]
fn mpick_mode_picks_one() {
    let b = pick(Mode::Beginner, Some(P), Some(E), Some(X));
    assert_eq!(b, picked(PLAIN_KEY, P, false));
    let e = pick(Mode::Expert, Some(P), Some(E), Some(X));
    assert_eq!(e, picked(ENG_KEY, E, false));
    let (b, e) = (b.expect("初心者"), e.expect("経験者"));
    assert!(!b.text.contains("E-字") && !e.text.contains("P-字"));
    assert_eq!((PLAIN_KEY, ENG_KEY), ("summary_plain", "summary_eng"));
}

/// (2) 表示の型の側が無い（None・空・空白だけ）時はもう一方を印つきで、両方無ければ本文の頭の 1 行を鍵 sum_body の印つきで返し、
/// それも無ければ None。鍵 sum_body の語は 本文から。
#[test]
fn mpick_fallback_marks() {
    for gone in [None, Some(""), Some("  ")] {
        assert_eq!(
            pick(Mode::Beginner, gone, Some(E), Some(X)),
            picked(ENG_KEY, E, true)
        );
        assert_eq!(
            pick(Mode::Expert, Some(P), gone, Some(X)),
            picked(PLAIN_KEY, P, true)
        );
    }
    for mode in Mode::ALL {
        assert_eq!(pick(mode, None, None, Some(X)), picked(BODY_KEY, X, true));
        assert_eq!(pick(mode, None, None, None), None);
    }
    assert_eq!(BODY_KEY, "sum_body");
    assert_eq!(vocab().label(BODY_KEY), "本文から");
}

/// (3) 200 字以下は畳まず、201 字以上は頭の 199 字と残りに分け、畳んだ形は頭と … の 200 字、頭と残りをつなぐと元の字。
#[test]
fn mpick_fold_lossless() {
    assert_eq!((POP_SUM_MAX, ELLIPSIS), (200, '…'));
    let at = "あ".repeat(200);
    let f = fold(&at, POP_SUM_MAX);
    assert_eq!((f.head.as_str(), f.rest.as_deref()), (at.as_str(), None));
    assert_eq!((f.shut(), f.whole()), (at.clone(), at.clone()));
    let over = format!("{}い字", "あ".repeat(199));
    let f = fold(&over, POP_SUM_MAX);
    assert_eq!(f.head, "あ".repeat(199));
    assert_eq!(f.rest.as_deref(), Some("い字"));
    assert_eq!(f.shut(), format!("{}…", "あ".repeat(199)));
    assert_eq!(f.shut().chars().count(), 200);
    assert_eq!(f.whole(), over);
}

/// (4) 個別の頁の概要の箱は 1 つで表示の型の字を切らずに持ち、もう一方は箱の下に畳む字として持つ。
/// 表示の型の側が無ければもう一方を印の class で出し畳む字は無い。block の見出しの鍵は nb_summary（語 概要）。
#[test]
fn mpick_node_one_box() {
    let long = format!("{P}{}", "あ".repeat(300));
    let doc = around(Some(&long), Some(E));
    let c = node::center(&doc).expect("中心の行");
    let b = node::summary(c, Mode::Beginner);
    assert_eq!(
        (b.key, b.class, b.text.as_str()),
        (PLAIN_KEY, "sumbox", long.as_str())
    );
    assert_eq!(b.other, picked(ENG_KEY, E, true));
    let e = node::summary(c, Mode::Expert);
    assert_eq!((e.key, e.class, e.text.as_str()), (ENG_KEY, "sumbox", E));
    assert_eq!(e.other, picked(PLAIN_KEY, &long, true));
    assert_eq!(
        other(Mode::Expert, Some(P), Some(E)),
        picked(PLAIN_KEY, P, true)
    );
    let doc = around(None, Some(E));
    let c = node::center(&doc).expect("中心の行");
    let m = node::summary(c, Mode::Beginner);
    assert_eq!(
        (m.key, m.class, m.text.as_str()),
        (ENG_KEY, SUMMARY_MARKED, E)
    );
    assert_eq!(m.other, None);
    assert_eq!(node::BLOCK.heading, "nb_summary");
    assert_eq!(vocab().label("nb_summary"), "概要");
}
