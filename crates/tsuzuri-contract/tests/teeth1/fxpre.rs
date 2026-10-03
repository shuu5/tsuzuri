//! 契約の歯の見本の裁定 id の見張りの歯（接頭辞 fxpre_・設計ノート surface-wave15c 行 t-fx-prefix）。
//! 器の決まり（FR83 の A）が本物の引きとして数える形（問いの部分が台帳の接頭辞で始まる裁定 id の形）を、
//! 契約の crate の tests の dir の下の全部の .rs と .json の file が持たないことを見る。
//! 見張りは器より厳しく、語の途中から始まる台帳の接頭辞の形も数える。
//! この file の見本の字は LEDGER から組み、file の字に台帳の接頭辞の裁定 id の形を持たない。
#![cfg(test)]

use std::path::{Path, PathBuf};

/// 台帳の接頭辞。
const LEDGER: &str = "t3-hub";

/// 字の頭が コロン・数字 8 桁・T・数字 4 桁・Z・ハイフン・数字 1 桁以上 なら、その byte の長さ。
fn minute_tail(s: &[u8]) -> Option<usize> {
    let digits = |from: usize| {
        s[from.min(s.len())..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count()
    };
    let mut at = 0;
    if s.first() != Some(&b':') {
        return None;
    }
    at += 1;
    if digits(at) < 8 {
        return None;
    }
    at += 8;
    if s.get(at) != Some(&b'T') {
        return None;
    }
    at += 1;
    if digits(at) < 4 {
        return None;
    }
    at += 4;
    if s.get(at) != Some(&b'Z') || s.get(at + 1) != Some(&b'-') {
        return None;
    }
    at += 2;
    match digits(at) {
        0 => None,
        n => Some(at + n),
    }
}

/// 字の中の台帳の接頭辞の裁定 id の形の字（LEDGER の現れのそれぞれから、問いの部分と時刻と連番まで）。
fn ledger_forms(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    text.match_indices(LEDGER)
        .filter_map(|(start, _)| {
            let part = bytes[start..]
                .iter()
                .take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
                .count();
            let tail = minute_tail(&bytes[start + part..])?;
            Some(text[start..start + part + tail].to_string())
        })
        .collect()
}

/// dir の下の全部の .rs と .json の file（下の dir も読む・path の順）。
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display()))
        .map(|e| e.expect("dir の項").path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            out.extend(files(&p));
        } else if p.extension().is_some_and(|e| e == "rs" || e == "json") {
            out.push(p);
        }
    }
    out
}

#[test]
fn fxpre_scan_counts_ledger_forms() {
    let id = format!("{LEDGER}.5:20260926T1437Z-1");
    let whole = format!("{LEDGER}:20260926T1437Z-12");
    let found = [
        (format!("裁定 {id}・逐語 = よい"), vec![id.clone()]),
        (format!("fx-{id}"), vec![id.clone()]),
        (whole.clone(), vec![whole.clone()]),
        (format!("{LEDGER}.5"), vec![]),
        (format!("{LEDGER}.5:20260926T1437Z-"), vec![]),
        (format!("{LEDGER}.5:2026092T1437Z-1"), vec![]),
        (format!("{LEDGER}.5:20260926T1437-1"), vec![]),
        ("fx-c.5:20260926T1437Z-1".to_string(), vec![]),
    ];
    for (text, want) in found {
        assert_eq!(ledger_forms(&text), want, "{text}");
    }
}

#[test]
fn fxpre_contract_tests_hold_none() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let all = files(&dir);
    for name in [
        "teeth1/cform_surface.rs",
        "teeth1/cform_ledger.rs",
        "snapshots/surface.json",
        "snapshots/ledger.json",
    ] {
        assert!(all.contains(&dir.join(name)), "読んだ file に {name} が無い");
    }
    let mut hits = Vec::new();
    for path in &all {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
        for form in ledger_forms(&text) {
            hits.push((path.display().to_string(), form));
        }
    }
    assert!(hits.is_empty(), "台帳の接頭辞の裁定 id の形の字: {hits:?}");
}
