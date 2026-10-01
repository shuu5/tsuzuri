//! 行 g-unref-dash の歯（接頭辞 udash_・要件 FR13）: project board の台帳の block の未反映の数は、3 種とも分からなければ
//! account board の表と同じ数えない字 ― にし、1 種か 2 種が分からなければ数に測れていないの印を添える。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::{LedgerStats, UnreflectedKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::projects::NONE_MARK;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::ledger::{Metrics, NONE, Part, Unref, content};
use tsuzuri_surface::view::{Fetched, Screen};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 指標の段を組む今（歯 ledgerblock と同じ今）。
const NOW: u64 = 1_791_676_800;

/// fixture の組 filled の未反映の数と分からない種類だけを替えて組んだ中身。
fn metrics(count: u32, unknown: &[UnreflectedKind]) -> Metrics {
    let mut sets: BTreeMap<String, LedgerStats> =
        wire::decode(&read("../../tests/fixtures/surface/ledger-stats.json"))
            .expect("fixture の組が電文として読める");
    let mut s = sets.remove("filled").expect("fixture の組 filled");
    s.unreflected = count;
    s.unreflected_unknown = unknown.to_vec();
    let body = Fetched::Body(wire::encode(&Reading::Known(s)).expect("電文"));
    match content(&body, &Screen::initial(), NOW) {
        Body::Filled(m) => m,
        other => panic!("中身が無い: {other:?}"),
    }
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

/// (2) 電文から組んだ中身の text は、上段の箱と未反映の段の見出しのどちらも Unref の text の字。
#[test]
fn udash_metrics_text() {
    use UnreflectedKind::{Memo, Ruling, Utterance};
    for (count, unknown, want) in [
        (0, &[Memo, Ruling, Utterance][..], "―"),
        (3, &[Ruling, Utterance][..], "3"),
        (41, &[][..], "41"),
    ] {
        let m = metrics(count, unknown);
        for part in [Part::Unref, Part::UnrefCount] {
            assert_eq!(
                m.text(part),
                Some(want.to_string()),
                "{part:?} count {count} unknown {unknown:?}"
            );
        }
    }
}

/// (3) mod dom の字: 上段の箱の印の式と未反映の段の見出しの数の span を 1 度ずつ持ち、前の式の字を持たない。
#[test]
fn udash_dom_wiring() {
    let src = read("src/project/ledger.rs");
    let dom = &src[src.find("mod dom").expect("mod dom が在る")..];
    for want in [
        "(part == Part::Unref && m.unref.partial()).then(|| state_icon(UNKNOWN))",
        "<span class=unref_chip(m.unref.count)>{m.unref.text()}</span>",
    ] {
        assert_eq!(dom.matches(want).count(), 1, "{want}");
    }
    for gone in ["!m.unref.unknown.is_empty()", "{m.unref.count}</span>"] {
        assert!(!dom.contains(gone), "{gone}");
    }
}
