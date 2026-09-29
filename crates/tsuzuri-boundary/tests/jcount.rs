//! 受入 12 条の数えの歯（行 j-count・接頭辞 jcount_）。
//! fixture は tests/fixtures/surface/accept-12/ の、違反 0 の clean.json と、clean.json に 1 件だけ仕込んだ
//! 12 の file と、english と rephrase に同じ鍵 pipeline を持つ小さな vocab.json。

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::audit::{
    Facts, Heading, Node, PROSE_MAX, RULES, TITLE_MAX, count, facts, head, label, line,
    prose_chars,
};

/// 仕込んだ 12 の file（番号と鍵の名）。
const PLANTED: [&str; 12] = [
    "01-overflow.json",
    "02-overlap.json",
    "03-hscroll.json",
    "04-hover.json",
    "05-heading.json",
    "06-prose.json",
    "07-error.json",
    "08-budget.json",
    "09-nodeid.json",
    "10-url.json",
    "11-verbatim.json",
    "12-library.json",
];

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/surface/accept-12")
}

fn read(name: &str) -> String {
    fs::read_to_string(dir().join(name)).unwrap_or_else(|e| panic!("{name} を読む: {e}"))
}

fn vocab() -> String {
    read("vocab.json")
}

fn clean() -> Facts {
    facts(&read("clean.json")).expect("clean.json を読む")
}

/// 鍵 `key` の条だけが `n` でほかは 0 の数の列。
fn only(key: &str, n: usize) -> [usize; 12] {
    let at = RULES
        .iter()
        .position(|(k, _)| *k == key)
        .unwrap_or_else(|| panic!("条 {key} が無い"));
    let mut want = [0; 12];
    want[at] = n;
    want
}

#[test]
fn jcount_rules_in_order() {
    let keys: Vec<&str> = RULES.iter().map(|(k, _)| *k).collect();
    assert_eq!(
        keys,
        [
            "overflow", "overlap", "hscroll", "hover", "heading", "prose", "error", "budget",
            "nodeid", "url", "verbatim", "library",
        ]
    );
    assert_eq!(RULES[5].1, "最初の画面の散文 300 字 超");
    assert_eq!(PROSE_MAX, 300);
    assert_eq!(TITLE_MAX, 36);
}

#[test]
fn jcount_clean_is_zero() {
    assert_eq!(count(&clean(), &vocab()), [0; 12]);
    let mut names: Vec<String> = fs::read_dir(dir())
        .expect("accept-12 の dir を読む")
        .map(|e| e.expect("dir の項").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut want: Vec<String> = PLANTED.iter().map(ToString::to_string).collect();
    want.push("clean.json".to_string());
    want.push("vocab.json".to_string());
    want.sort();
    assert_eq!(names, want, "accept-12 の file は clean と vocab と仕込んだ 12 だけ");
}

#[test]
fn jcount_each_planted_counts_one() {
    let vocab = vocab();
    for (name, (key, _)) in PLANTED.iter().zip(RULES) {
        assert!(name.ends_with(&format!("-{key}.json")), "{name} の名は鍵 {key}");
        let planted = facts(&read(name)).unwrap_or_else(|e| panic!("{name} を読む: {e}"));
        assert_eq!(count(&planted, &vocab), only(key, 1), "{name}");
    }
}

#[test]
fn jcount_edges() {
    let vocab = vocab();
    assert_eq!(prose_chars("P-1 は 12:30Z の「字」・ab"), 7);

    let base = clean();
    let used: usize = base.first.iter().map(|p| prose_chars(p)).sum();
    let mut at_max = base.clone();
    at_max.first[0].push_str(&"あ".repeat(PROSE_MAX - used));
    assert_eq!(at_max.first.iter().map(|p| prose_chars(p)).sum::<usize>(), 300);
    assert_eq!(count(&at_max, &vocab), [0; 12]);
    let mut over = at_max.clone();
    over.first[0].push('あ');
    assert_eq!(count(&over, &vocab), only("prose", 1));

    assert_eq!(base.titles[0].chars().count(), 36);
    assert_eq!(count(&base, &vocab), [0; 12]);
    let mut long = base.clone();
    long.titles.push(format!("{}字", base.titles[0]));
    assert_eq!(count(&long, &vocab), only("budget", 1));

    for (hscroll, want) in [(-3, [0; 12]), (0, [0; 12]), (1, only("hscroll", 1))] {
        let wide = Facts {
            hscroll,
            ..base.clone()
        };
        assert_eq!(count(&wide, &vocab), want, "hscroll {hscroll}");
    }
}

#[test]
fn jcount_rule_readings() {
    let vocab = vocab();
    assert_eq!(
        label(&vocab, "pipeline").as_deref(),
        Some("pipeline dashboard")
    );
    assert_eq!(label(&vocab, "gaps").as_deref(), Some("すき間"));
    assert_eq!(label(&vocab, "lanes"), None);
    let base = clean();

    let mut heading = base.clone();
    heading.headings[0] = Heading {
        key: "lanes".to_string(),
        text: "pipeline dashboard".to_string(),
    };
    assert_eq!(count(&heading, &vocab), only("heading", 1));

    for (piece, n) in [
        ("帯・札・印", 1),
        ("帯（札（印））", 1),
        ("帯(札(印))", 1),
        ("帯（札(印)）", 1),
        ("帯・札（印）", 0),
        ("帯（札）（印）", 0),
    ] {
        let mut heavy = base.clone();
        heavy.first.push(piece.to_string());
        assert_eq!(count(&heavy, &vocab), only("budget", n), "片 {piece}");
    }

    for (words, n) in [
        ("user 2026-09-26T07:51Z", 1),
        ("裁定 user 2026-09-26T07:51Z と user 2026-09-27T23:57Z", 2),
        ("user 2026-09-26T07", 0),
        ("user 2026-09-26T07-51", 0),
        ("user 2026-09-26 07:51", 0),
        ("user 26-09-26T07:51", 0),
        ("user2026-09-26T07:51", 0),
        ("user が 2026-09-26T07:51 に決めた", 0),
    ] {
        let mut said = base.clone();
        said.text = format!("{}\n{words}", base.text);
        assert_eq!(count(&said, &vocab), only("verbatim", n), "字 {words}");
    }

    assert_eq!(base.url, "http://127.0.0.1:4801/?page=pipeline&mode=beginner");
    for (source, n) in [
        ("//cdn.example.com/lib.js", 1),
        ("https://127.0.0.1:4801/lib.js", 1),
        ("http://127.0.0.1:4802/lib.js", 1),
        ("http://localhost:4801/lib.js", 1),
        ("https://cdn.example.com/lib.js", 1),
        ("http://127.0.0.1:4801/lib.js", 0),
        ("HTTP://127.0.0.1:4801/lib.js", 0),
        ("/lib.js", 0),
        ("lib.js", 0),
        ("../lib.js", 0),
        ("blob:http://127.0.0.1:4801/0f3c", 0),
        ("data:text/javascript,void 0", 0),
    ] {
        let mut loads = base.clone();
        loads.libraries.push(source.to_string());
        assert_eq!(count(&loads, &vocab), only("library", n), "読み先 {source}");
    }

    let mut unnamed = base.clone();
    unnamed.nodes.push(Node {
        id: "t3-hub.53".to_string(),
        text: "t3-hub.5 面".to_string(),
    });
    assert_eq!(count(&unnamed, &vocab), only("nodeid", 1));
}

/// clean.json の鍵を 1 つずつ替えるための、鍵と値の字の組の列から組んだ事実の字。
fn doc(skip: &str, hscroll: &str) -> String {
    let pairs = [
        ("url", "\"http://127.0.0.1:4801/\"".to_string()),
        ("hscroll", hscroll.to_string()),
        ("overflow", "[]".to_string()),
        ("overlap", "[]".to_string()),
        ("nocard", "[]".to_string()),
        ("headings", "[]".to_string()),
        ("first", "[]".to_string()),
        ("errors", "[]".to_string()),
        ("titles", "[]".to_string()),
        ("nodes", "[]".to_string()),
        ("switches", "[]".to_string()),
        ("text", "\"\"".to_string()),
        ("libraries", "[]".to_string()),
        ("extra", "{\"a\": [1, 2]}".to_string()),
    ];
    let body: Vec<String> = pairs
        .iter()
        .filter(|(k, _)| *k != skip)
        .map(|(k, v)| format!("\"{k}\": {v}"))
        .collect();
    format!("{{{}}}", body.join(", "))
}

#[test]
fn jcount_facts_refuse_bad_shape() {
    let whole = facts(&doc("", "0")).expect("欠けの無い字");
    assert_eq!(count(&whole, "{}"), [0; 12]);
    for key in ["url", "hscroll", "switches", "libraries"] {
        let err = facts(&doc(key, "0")).expect_err(key);
        assert!(err.contains(key), "{key} の欠け: {err}");
    }
    let err = facts(&doc("", "\"3\"")).expect_err("hscroll が字の値");
    assert!(err.contains("hscroll"), "{err}");
    for text in ["[]", "\"clean\"", "3", ""] {
        assert!(facts(text).is_err(), "object でない字 {text:?}");
    }
    let err = facts(&doc("", "0").replace("\"nodes\": []", "\"nodes\": [{\"id\": \"x\"}]"))
        .expect_err("札の text の欠け");
    assert!(err.contains("nodes"), "{err}");
}

#[test]
fn jcount_report_lines() {
    let words: Vec<String> = ["width", "mode", "url"]
        .iter()
        .map(ToString::to_string)
        .chain(RULES.iter().map(|(k, _)| k.to_string()))
        .chain(["計".to_string()])
        .collect();
    assert_eq!(head(), words.join(" "));

    let hscroll = facts(&read("03-hscroll.json")).expect("03-hscroll.json を読む");
    let counts = count(&hscroll, &vocab());
    assert_eq!(
        line(390, "expert", &hscroll.url, &counts),
        format!(
            "390 expert {} 0 0 1 0 0 0 0 0 0 0 0 0 計 1",
            hscroll.url
        )
    );
}
