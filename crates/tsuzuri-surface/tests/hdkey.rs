//! 見出しの語の鍵の歯（接頭辞 hdkey_）。
//! host の窓と相談の窓の段の見出しが語の鍵の印 data-v を持ち、受入の測りと同じ和の語彙表の label が出ている字と同じであることを測る段である。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::audit;
use tsuzuri_surface::consultwin::PART_KEYS as CONSULT_KEYS;
use tsuzuri_surface::hostwin::PART_KEYS as HOST_KEYS;
use tsuzuri_surface::vocab::{PARTS, Vocab, sources, vocab};

const PART_FILE: &str = "vocab/t-heading-keys.json";
const CONSULT_LABELS: [&str; 3] = ["開いている相談の窓", "処分の無い所見", "受けの無い頼み"];

fn crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(rel: &str) -> String {
    fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// dir を辿った .rs の file の字（path と字）。
fn rs_files(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display()))
        .map(|e| e.expect("dir の項").path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let text =
                fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} を読む: {e}", p.display()));
            out.push((p, text));
        }
    }
}

/// 相談の窓の 3 つの鍵は部品 vocab/t-heading-keys.json だけが持ち、label は出ている字・注釈は空でない。
#[test]
fn hdkey_consult_keys_in_part_file() {
    assert_eq!(CONSULT_KEYS, ["cs_windows", "cs_findings", "cs_requests"]);
    let hits: Vec<&(&str, &str)> = PARTS.iter().filter(|(n, _)| *n == PART_FILE).collect();
    assert_eq!(hits.len(), 1, "部品の表の {PART_FILE}");
    let only = Vocab::parse_all(&[*hits[0]]).expect("部品だけで読める");
    assert_eq!(
        only.keys().collect::<Vec<_>>(),
        ["cs_findings", "cs_requests", "cs_windows"]
    );
    for (key, want) in CONSULT_KEYS.into_iter().zip(CONSULT_LABELS) {
        let term = vocab().term(key).unwrap_or_else(|| panic!("語彙の {key}"));
        assert_eq!(term.label, want, "{key}");
        assert!(!term.note.is_empty(), "{key} の注釈が空");
    }
}

/// 見出しの鍵を受入の測りの和（audit::merge_vocab）で引くと出ている字と同じで、空の鍵は引けない（runner が違反に数える形）。
#[test]
fn hdkey_headings_match_vocab_labels() {
    let texts: Vec<&str> = sources().iter().map(|(_, t)| *t).collect();
    let merged = audit::merge_vocab(&texts).expect("語彙の和");
    let host = ["負荷", "詰まり", "メモリ", "書きの速さ", "SSD の摩耗"];
    for (key, want) in HOST_KEYS.into_iter().zip(host) {
        assert_eq!(audit::label(&merged, key).as_deref(), Some(want), "{key}");
    }
    for (key, want) in CONSULT_KEYS.into_iter().zip(CONSULT_LABELS) {
        assert_eq!(audit::label(&merged, key).as_deref(), Some(want), "{key}");
    }
    assert_eq!(audit::label(&merged, ""), None);
}

/// 見出しの要素は語の鍵の印 data-v を持つ（DOM は wasm の target でだけ組むので字で照らす）。
#[test]
fn hdkey_heading_marks_in_dom() {
    let host = read("src/hostwin.rs");
    assert_eq!(
        host.matches("<h4 data-v=p.key>{label(p.key)}</h4>").count(),
        1
    );
    let consult = read("src/consultwin.rs");
    let mut last = 0;
    for i in 0..3 {
        let want = format!("<h4 data-v=PART_KEYS[{i}]>{{label(PART_KEYS[{i}])}}</h4>");
        assert_eq!(consult.matches(&want).count(), 1, "{want}");
        let at = consult.find(&want).expect("見出し");
        assert!(at > last, "{want} の並び");
        last = at;
    }
    let mut files = Vec::new();
    rs_files(&crate_dir().join("src"), &mut files);
    assert!(files.len() > 10, "src の file が少ない");
    for (p, text) in &files {
        assert_eq!(
            text.matches("<h4>").count(),
            0,
            "{} に印の無い見出し",
            p.display()
        );
    }
}
