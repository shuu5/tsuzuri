//! 係の型の考える量の段の歯（接頭辞 ageff_・設計ノート surface-wave29d 行 ag-effort-high・台帳の問い t3-hub.87.37 の推奨）。
//! plugin/agents/ の型 3 本の頭の欄 effort を見る。起草係 drafter だけが欄をちょうど 1 つ値 high で持ち、調べ役 researcher と
//! 検証役 verifier は欄を持たず席の段を継ぐ。型の定義は Claude Code が係を起こす時に読む設定の字で、歯から係を起こして段を
//! 測れないので字を照らす。否定の見本は今の file から句を 1 つだけ崩して作る。外の依存を使わない。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// 係の型の file の置き場（workspace の根から）。
const AGENTS: &str = "plugin/agents";

/// 型の file の名の幹と、頭の欄 effort に望む値の列（空は欄を持たず席の段を継ぐ）。
const WANT: [(&str, &[&str]); 3] = [
    ("drafter", &["high"]),
    ("researcher", &[]),
    ("verifier", &[]),
];

/// 起草係の頭の欄 effort の行。
const LINE: &str = "effort: high\n";

/// 否定の見本で欄を差し込む所（型 3 本の頭に 1 度ずつ在る行）。
const MODEL: &str = "model: opus\n";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// 今の型 3 本の幹と中身（WANT の順）。
fn current() -> Vec<(&'static str, String)> {
    WANT.iter()
        .map(|(stem, _)| {
            let rel = format!("{AGENTS}/{stem}.md");
            let text = fs::read_to_string(root().join(&rel))
                .unwrap_or_else(|e| panic!("{rel} を読む: {e}"));
            (*stem, text)
        })
        .collect()
}

/// 頭の --- の間の欄 effort の値の列（頭の外の字は数えない）。頭が読めなければ None。
fn efforts(text: &str) -> Option<Vec<&str>> {
    let rest = text.strip_prefix("---\n")?;
    let (head, _) = rest.split_once("\n---\n")?;
    Some(
        head.lines()
            .filter_map(|l| l.strip_prefix("effort:"))
            .map(str::trim)
            .collect(),
    )
}

/// 型の幹と中身の組のうち、頭の欄 effort の値の列が WANT と違う型の幹（WANT の順・頭が読めない型と無い型も名指す）。
fn faults(files: &[(&str, String)]) -> Vec<&'static str> {
    WANT.iter()
        .filter(|(stem, want)| {
            let got = files
                .iter()
                .find(|(s, _)| s == stem)
                .and_then(|(_, t)| efforts(t));
            got.as_deref() != Some(*want)
        })
        .map(|(stem, _)| *stem)
        .collect()
}

/// (2) plugin/agents/ の file は drafter.md・researcher.md・verifier.md の 3 本だけで、drafter.md の頭は欄 effort を
/// ちょうど 1 つ値 high で持ち、researcher.md と verifier.md の頭は欄 effort を持たない。
#[test]
fn ageff_only_the_drafter_sets_effort() {
    let mut names: Vec<String> = fs::read_dir(root().join(AGENTS))
        .expect("plugin/agents を読む")
        .map(|e| e.expect("項").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let want: Vec<String> = WANT.iter().map(|(s, _)| format!("{s}.md")).collect();
    assert_eq!(names, want);
    let now = current();
    for (stem, text) in &now {
        assert!(efforts(text).is_some(), "{stem} の頭");
    }
    assert_eq!(faults(&now), Vec::<&str>::new());
}

/// (3) 今の file から句を 1 つだけ崩した見本（起草係の欄を外す・値を xhigh にする・欄を 2 つにする・欄を頭の外の本文の末へ
/// 動かす・調べ役か検証役の頭に欄 effort: high を足す）は、どれも外れとして名指されるのがその型だけになる。
#[test]
fn ageff_one_clause_broken_names_its_type() {
    let now = current();
    let text = |stem: &str| {
        now.iter()
            .find(|(s, _)| *s == stem)
            .map(|(_, t)| t.clone())
            .expect("型")
    };
    let drafter = text("drafter");
    let add = |stem: &str| text(stem).replacen(MODEL, &format!("{MODEL}{LINE}"), 1);
    let moved = format!("{}{LINE}", drafter.replacen(LINE, "", 1));
    assert!(moved.contains(LINE));
    let cases = [
        ("drafter", drafter.replacen(LINE, "", 1)),
        ("drafter", drafter.replacen(LINE, "effort: xhigh\n", 1)),
        ("drafter", drafter.replacen(LINE, &LINE.repeat(2), 1)),
        ("drafter", moved),
        ("researcher", add("researcher")),
        ("verifier", add("verifier")),
    ];
    for (stem, sample) in cases {
        assert_ne!(sample, text(stem), "{stem}");
        let files: Vec<(&str, String)> = now
            .iter()
            .map(|(s, t)| (*s, if *s == stem { &sample } else { t }.clone()))
            .collect();
        assert_eq!(faults(&files), [stem], "{stem}");
    }
}
