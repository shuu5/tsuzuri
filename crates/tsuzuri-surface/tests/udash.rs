//! 行 g-unref-dash の歯（接頭辞 udash_・要件 FR13）: project board の未反映の段（行 g-ledger-trim で台帳の block から
//! 抜けの検査の窓へ移した）の数は、3 種とも分からなければ account board の表と同じ数えない字 ― にし、1 種か 2 種が
//! 分からなければ数に測れていないの印を添える。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::stats::{LedgerStats, UnreflectedKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::projects::NONE_MARK;
use tsuzuri_surface::project::ledger::{NONE, Unref};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の組 filled の未反映の数と分からない種類だけを替えた電文。
fn stats(count: u32, unknown: &[UnreflectedKind]) -> LedgerStats {
    let mut sets: BTreeMap<String, LedgerStats> =
        wire::decode(&read("../../tests/fixtures/surface/ledger-stats.json"))
            .expect("fixture の組が電文として読める");
    let mut s = sets.remove("filled").expect("fixture の組 filled");
    s.unreflected = count;
    s.unreflected_unknown = unknown.to_vec();
    s
}

fn unref(count: u32, unknown: &[&'static str]) -> Unref {
    Unref {
        count,
        unknown: unknown.to_vec(),
    }
}

/// (1) 3 種とも分からなければ字 ―（count に関わらず）・ほかは数の字。印は 1 種か 2 種が分からない時だけ。
#[test]
fn udash_text_and_partial() {
    assert_eq!(NONE, "―");
    assert_eq!(NONE, NONE_MARK, "account board の表の字と同じ");
    let all = ["memo", "ruling", "utterance"];
    assert_eq!(all.len(), UnreflectedKind::ALL.len());
    for count in [0, 4] {
        let u = unref(count, &all);
        assert_eq!(u.text(), NONE, "3 種とも分からない count {count}");
        assert!(!u.partial(), "3 種とも分からない count {count}");
    }
    for (count, unknown, text, partial) in [
        (0, &["ruling", "utterance"][..], "0", true),
        (3, &["ruling", "utterance"][..], "3", true),
        (2, &["memo"][..], "2", true),
        (0, &[][..], "0", false),
        (41, &[][..], "41", false),
    ] {
        let u = unref(count, unknown);
        assert_eq!(u.text(), text, "count {count} unknown {unknown:?}");
        assert_eq!(u.partial(), partial, "count {count} unknown {unknown:?}");
    }
}

/// (2) 電文から組んだ未反映の段の見出しの数（Unref::of の text）は、電文の数と分からない種類から決まる字。
#[test]
fn udash_metrics_text() {
    use UnreflectedKind::{Memo, Ruling, Utterance};
    for (count, unknown, want) in [
        (0, &[Memo, Ruling, Utterance][..], "―"),
        (3, &[Ruling, Utterance][..], "3"),
        (41, &[][..], "41"),
    ] {
        let u = Unref::of(&stats(count, unknown));
        assert_eq!(u.text(), want, "count {count} unknown {unknown:?}");
        assert_eq!(u.count, count);
    }
}

/// (3) mod dom の字: 未反映の段の見出しの印の式と数の span を 1 度ずつ持ち、前の式の字を持たない。
#[test]
fn udash_dom_wiring() {
    let src = read("src/project/ledger.rs");
    let dom = &src[src.find("mod dom").expect("mod dom が在る")..];
    for want in [
        "u.partial().then(|| state_icon(UNKNOWN))",
        "<span class=unref_chip(u.count)>{u.text()}</span>",
    ] {
        assert_eq!(dom.matches(want).count(), 1, "{want}");
    }
    for gone in ["!u.unknown.is_empty()", "{u.count}</span>"] {
        assert!(!dom.contains(gone), "{gone}");
    }
}
