//! 語彙の部品の歯（面・接頭辞 vcparts_・設計ノート surface-wave29b 行 g-vocab-parts・判断の記録 ADR-58）。
//! 部品の表は vocab の dir の .json の名の順・vocab() は基と部品の和・鍵の重なりは file の中（基も部品も・同じ欄に 2 度と
//! 両方の欄）でも file を跨いでも Err・受入の測りは同じ和を引く。否定の見本は 1 欄だけ替える。
//! 置き場の部品は鍵を持たないので、vocab() と受入の vocab_text が部品を読む事は関数の本体の範囲の字で測る。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::audit;
use tsuzuri_surface::vocab::{PARTS, SOURCE, Vocab, sources, vocab};

fn crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(rel: &str) -> String {
    fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 新しい鍵を 1 つ持つ部品の見本（欄 col に鍵 key）。
fn part(col: &str, key: &str) -> String {
    let other = if col == "english" {
        "rephrase"
    } else {
        "english"
    };
    format!(
        r#"{{"{col}": {{"{key}": {{"label": "語 {key}", "note": "注 {key}", "plain": "注 {key}", "internal": "内 {key}"}}}}, "{other}": {{}}}}"#
    )
}

/// 欄の鍵の頭の行（字下げ 2 の `"鍵": {` の行）の数。
fn heads(text: &str) -> usize {
    text.lines()
        .filter(|l| l.starts_with("  \"") && l.ends_with(": {"))
        .count()
}

/// file の字 src の中の、頭の行 head（file に 1 つだけ）から次の行頭の `}` の前までの関数の本体。
fn fn_body<'a>(src: &'a str, head: &str) -> &'a str {
    assert_eq!(src.matches(head).count(), 1, "頭 {head}");
    let at = src.find(head).expect("頭");
    let end = at + src[at..].find("\n}\n").expect("関数の末");
    &src[at..end]
}

/// (1) 部品の表は vocab の dir の .json の file の名の byte の順で、名と中身の字が file と同じ。
#[test]
fn vcparts_table_follows_dir() {
    let mut names: Vec<String> = fs::read_dir(crate_dir().join("vocab"))
        .expect("vocab の dir を読む")
        .map(|e| {
            e.expect("dir の項")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n.ends_with(".json"))
        .collect();
    names.sort();
    assert!(names.contains(&"000-about.json".to_string()), "{names:?}");
    let want: Vec<String> = names.iter().map(|n| format!("vocab/{n}")).collect();
    assert_eq!(PARTS.iter().map(|(n, _)| *n).collect::<Vec<_>>(), want);
    for (name, text) in PARTS {
        assert_eq!(text, read(name), "{name} の中身");
    }
    assert_eq!(sources().first(), Some(&("vocab.json", SOURCE)));
    assert_eq!(&sources()[1..], &PARTS[..]);
}

/// (2) vocab() は基と全部の部品の鍵を持ち（数は基の鍵の頭の行と、部品を 1 つずつ読んだ鍵の数の和）、
/// 部品だけに在る鍵の語・注・内部の名を引ける。
#[test]
fn vcparts_part_keys_read() {
    let in_parts: usize = PARTS
        .iter()
        .map(|p| {
            Vocab::parse_all(&[*p])
                .expect("部品は 1 つでも読める")
                .keys()
                .count()
        })
        .sum();
    let want = heads(SOURCE) + in_parts;
    assert!(want > 400, "鍵の頭の行が少ない: {want}");
    assert_eq!(vocab().keys().count(), want);
    let en = part("english", "zz-en");
    let re = part("rephrase", "zz-re");
    let all = Vocab::parse_all(&[
        ("vocab.json", SOURCE),
        ("vocab/a.json", &en),
        ("vocab/b.json", &re),
    ])
    .expect("新しい鍵だけの部品");
    assert_eq!(all.keys().count(), heads(SOURCE) + 2);
    let term = all.term("zz-en").expect("部品の english の鍵");
    assert_eq!(
        (
            term.label.as_str(),
            term.note.as_str(),
            term.internal.as_str()
        ),
        ("語 zz-en", "注 zz-en", "内 zz-en")
    );
    assert_eq!(all.label("zz-re"), "語 zz-re");
    assert_eq!(all.label("park"), "区画");
}

/// (3) 基と部品の間か部品どうしで同じ鍵が在れば、鍵と 2 つの在りかを書いた Err。
#[test]
fn vcparts_duplicate_key_refused() {
    let park = part("rephrase", "park");
    assert_eq!(
        Vocab::parse_all(&[("vocab.json", SOURCE), ("vocab/x.json", &park)]),
        Err(
            "鍵 park が 2 度在る（vocab.json の欄 rephrase と vocab/x.json の欄 rephrase）"
                .to_string()
        )
    );
    let a = part("english", "zz-a");
    let b = part("rephrase", "zz-a");
    assert_eq!(
        Vocab::parse_all(&[
            ("vocab.json", SOURCE),
            ("vocab/a.json", &a),
            ("vocab/b.json", &b)
        ]),
        Err(
            "鍵 zz-a が 2 度在る（vocab/a.json の欄 english と vocab/b.json の欄 rephrase）"
                .to_string()
        )
    );
}

/// (4) 1 つの file の中の重なり（基の vocab.json の同じ欄に 2 度・部品の同じ欄に 2 度・部品の両方の欄）も Err。
#[test]
fn vcparts_in_file_duplicate_refused() {
    let head = " \"rephrase\": {\n";
    let at = SOURCE.find(head).expect("欄 rephrase の頭") + head.len();
    let base = format!(
        "{}  \"park\": {{\n   \"label\": \"二度目\"\n  }},\n{}",
        &SOURCE[..at],
        &SOURCE[at..]
    );
    assert_eq!(heads(&base), heads(SOURCE) + 1);
    assert_eq!(
        Vocab::parse_all(&[("vocab.json", &base)]),
        Err(
            "鍵 park が 2 度在る（vocab.json の欄 rephrase と vocab.json の欄 rephrase）"
                .to_string()
        )
    );
    let twice =
        r#"{"rephrase": {"zz-t": {"label": "一"}, "zz-t": {"label": "二"}}, "english": {}}"#;
    assert_eq!(
        Vocab::parse_all(&[("vocab.json", SOURCE), ("vocab/t.json", twice)]),
        Err(
            "鍵 zz-t が 2 度在る（vocab/t.json の欄 rephrase と vocab/t.json の欄 rephrase）"
                .to_string()
        )
    );
    let both = r#"{"rephrase": {"zz-b": {"label": "一"}}, "english": {"zz-b": {"label": "二"}}}"#;
    assert_eq!(
        Vocab::parse_all(&[("vocab.json", SOURCE), ("vocab/b.json", both)]),
        Err(
            "鍵 zz-b が 2 度在る（vocab/b.json の欄 rephrase と vocab/b.json の欄 english）"
                .to_string()
        )
    );
}

/// (5) 受入の測りの語彙（audit::merge_vocab の和）は、どの鍵も面の読みと同じ見出しを引く（部品だけの鍵も）。
#[test]
fn vcparts_accept_reads_parts() {
    let en = part("english", "zz-en");
    let re = part("rephrase", "zz-re");
    let texts: Vec<&str> = sources()
        .iter()
        .map(|(_, t)| *t)
        .chain([en.as_str(), re.as_str()])
        .collect();
    let merged = audit::merge_vocab(&texts).expect("語彙の和");
    let mut files: Vec<(&str, &str)> = sources();
    files.extend([("vocab/a.json", en.as_str()), ("vocab/b.json", re.as_str())]);
    let all = Vocab::parse_all(&files).expect("面の読み");
    assert_eq!(all.keys().count(), vocab().keys().count() + 2);
    for key in all.keys() {
        assert_eq!(audit::label(&merged, key), Some(all.label(key)), "鍵 {key}");
    }
    assert_eq!(
        audit::merge_vocab(&[SOURCE, r#"{"english": {}}"#]),
        Err("語彙表の 1 番目の字に object の欄 rephrase が無い".to_string())
    );
}

/// (6) 関数 vocab() の本体は基と部品の並び sources() を parse_all に渡し、基の字 SOURCE を直に読まない。
#[test]
fn vcparts_vocab_reads_sources() {
    let src = read("src/vocab.rs");
    let body = fn_body(&src, "pub fn vocab() -> &'static Vocab {");
    let counts = [
        "Vocab::parse_all(&sources())",
        "sources",
        "SOURCE",
        "::parse(",
    ]
    .map(|s| body.matches(s).count());
    assert_eq!(counts, [1, 1, 0, 0], "{body}");
}

/// (7) 受入の sweep は語彙を vocab_text から得て、vocab_text は部品の dir の .json を名の順に読み、基を先頭に置いて
/// merge_vocab に渡す（並びを途中で縮める字も無い）。
#[test]
fn vcparts_accept_reads_dir() {
    let src = read("../../xtask/src/accept.rs");
    let dir = "const VOCAB_PARTS: &str = \"crates/tsuzuri-surface/vocab\";";
    assert_eq!(src.matches(dir).count(), 1);
    let sweep = fn_body(
        &src,
        "fn sweep(flags: &Flags, root: &Path) -> Result<(usize, usize), String> {",
    );
    let counts =
        ["let vocab = vocab_text(root)?;", "&vocab)", "VOCAB"].map(|s| sweep.matches(s).count());
    assert_eq!(counts, [1, 1, 0], "{sweep}");
    let body = fn_body(
        &src,
        "fn vocab_text(root: &Path) -> Result<String, String> {",
    );
    let steps = [
        "let dir = root.join(VOCAB_PARTS);",
        "fs::read_dir(&dir)",
        "is_some_and(|x| x == \"json\")",
        "files.sort();",
        "files.insert(0, root.join(VOCAB));",
        "for file in &files {",
        "texts.push(",
        "audit::merge_vocab(&texts.iter().map(String::as_str).collect::<Vec<_>>())",
    ];
    let counts: Vec<usize> = steps.iter().map(|s| body.matches(s).count()).collect();
    assert_eq!(counts, [1; 8], "{body}");
    let at: Vec<usize> = steps.iter().filter_map(|s| body.find(s)).collect();
    assert!(at.is_sorted(), "{at:?}");
    assert_eq!(
        (body.matches("files").count(), body.matches("texts").count()),
        (4, 3),
        "{body}"
    );
}
