//! 便 g-frame の歯: 頁の枠の snapshot・block の並び・見出しの語が vocab に在る・画面の class が stylesheet に在る・
//! data の口が無い block は測れていない・7 つの module・mode が URL に残る・「?」の注釈の分け方。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::server::ledger as server_ledger;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{LedgerList, LedgerRow};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::{self, HEADER, Mode, PageId};
use tsuzuri_surface::project::{
    self, Body, STATES, ask, ledger, legend, map, next, pipeline, seat, state_class,
};
use tsuzuri_surface::view::{Fetched, Screen};
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::help::{self, Inline, Line};
use tsuzuri_surface::widgets::hover::{Card, card_class};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_rows() -> Vec<LedgerRow> {
    let text = read("../../tests/fixtures/ledger/board-8.jsonl");
    let Reading::Known(items) = server_ledger::parse(&text) else {
        panic!("fixture が server の読みで Unknown");
    };
    items.into_iter().map(|i| i.row).collect()
}

fn known_screen() -> Screen {
    let body = wire::encode(&LedgerList {
        rows: Reading::Known(fixture_rows()),
    })
    .expect("電文");
    Screen::initial().after_read(&Fetched::Body(body), 100)
}

/// src の下の .rs の file の全部（path の順）。
fn sources() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("src を読む").flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let mut paths = Vec::new();
    walk(&crate_dir().join("src"), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).expect("src の file");
            (p, text)
        })
        .collect()
}

#[test]
fn frame_snapshot_matches_file() {
    let want = read("tests/snapshots/frame.json");
    let got = frame::snapshot();
    assert!(got == want, "頁の枠の値が snapshot と違う。今の値:\n{got}");
}

#[test]
fn frame_home_blocks_in_order_and_map_page() {
    assert_eq!(
        frame::home().block_ids(),
        vec!["next", "ask", "pipe", "orch", "ledger", "legend"]
    );
    assert_eq!(frame::map().block_ids(), vec!["map"]);
    assert_eq!(frame::map().id.id(), "map");
    assert_eq!(frame::home().id.id(), "home");
    let parts: Vec<&str> = HEADER.iter().map(|h| h.part).collect();
    assert_eq!(parts, vec!["brand", "nav", "updated", "mode"]);
}

/// 見出しの語の鍵（枠・header・nav・指標・凡例・注釈の部品）は全部 vocab の file に在る。
#[test]
fn frame_headings_come_from_vocab() {
    let mut keys: Vec<&str> = Vec::new();
    for page in frame::pages() {
        keys.push(page.heading);
        keys.extend(
            page.columns
                .iter()
                .flat_map(|c| c.blocks.iter().map(|b| b.heading)),
        );
    }
    for part in HEADER {
        keys.push(part.key);
        keys.extend(part.items);
    }
    keys.extend(ledger::METRICS);
    keys.extend(legend::states().iter().map(|e| e.key));
    keys.extend(STATES.iter().map(|(_, k)| *k));
    keys.extend(["p_more", "not_yet"]);
    for key in keys {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }
    assert_eq!(vocab().label("next"), "次の一手");
    assert_eq!(vocab().label("pipeline"), "pipeline dashboard");
    assert_eq!(
        vocab().label("no-such-key"),
        "〔語彙表に無い: no-such-key〕"
    );
}

/// 画面の code に日本語の見出しの字（vocab の語）を直に書かない（語彙の読み自身は除く）。
#[test]
fn frame_no_heading_words_in_code() {
    let words: Vec<String> = vocab()
        .keys()
        .filter_map(|k| vocab().term(k))
        .map(|t| t.label.clone())
        .filter(|l| !l.is_ascii())
        .collect();
    for (path, text) in sources() {
        if path.ends_with("vocab.rs") {
            continue;
        }
        for word in &words {
            let literal = format!("\"{word}\"");
            assert!(
                !text.contains(&literal),
                "{} に見出しの語 {literal} が直に在る",
                path.display()
            );
        }
    }
}

/// stylesheet の class の名（selector の `.名`）。
fn stylesheet_classes() -> BTreeSet<String> {
    let css = read("style.css");
    let mut text = String::new();
    let mut rest = css.as_str();
    while let Some(i) = rest.find("/*") {
        text.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |j| &rest[i + j + 2..]);
    }
    text.push_str(rest);
    let chars: Vec<char> = text.chars().collect();
    let mut out = BTreeSet::new();
    for i in 0..chars.len() {
        // `.名` の名は字か `-` か `_` で始まる（`.45` や `1.6s` の数は外れる・`.shape.fill` の続きは拾う）。
        let starts = chars
            .get(i + 1)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == '_' || *c == '-');
        if chars[i] == '.' && starts {
            let name: String = chars[i + 1..]
                .iter()
                .take_while(|c| c.is_ascii_alphanumeric() || **c == '-' || **c == '_')
                .collect();
            out.insert(name);
        }
    }
    out
}

/// 画面が使う class の名: src の `class="…"` の字・枠の値・関数が組む class。
fn used_classes() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut add = |s: &str| {
        for c in s.split_whitespace() {
            out.insert(c.to_string());
        }
    };
    for (_, text) in sources() {
        let mut rest = text.as_str();
        while let Some(i) = rest.find("class=\"") {
            rest = &rest[i + 7..];
            let end = rest.find('"').expect("class の字の終わり");
            add(&rest[..end]);
            rest = &rest[end..];
        }
    }
    for page in frame::pages() {
        add(page.class);
        for c in &page.columns {
            add(c.class);
            for b in &c.blocks {
                add(b.class);
            }
        }
    }
    for part in HEADER {
        add(part.class);
    }
    for page in [PageId::Home, PageId::Map] {
        for link in frame::nav_links(page) {
            add(link.class);
        }
    }
    for mode in Mode::ALL {
        add(mode.body_class());
    }
    for (v, _) in STATES {
        add(&state_class(v));
    }
    for row in fixture_rows() {
        add(&project::item(&row).shape);
    }
    for (open, pinned) in [(false, false), (true, false), (true, true)] {
        add(help::tip_class(open, pinned));
    }
    for open in [false, true] {
        add(card_class(open));
    }
    for (class, _) in Card::default().rows() {
        add(class);
    }
    out
}

#[test]
fn frame_classes_are_in_stylesheet() {
    let css = stylesheet_classes();
    // stylesheet は見本の ui.css の写し（見本の規則の class を持つ）。
    for c in [
        "page", "panel", "home", "stack", "top", "nav", "seg", "q", "tip", "legend5", "l4", "items",
    ] {
        assert!(css.contains(c), "stylesheet に見本の class {c} が無い");
    }
    let used = used_classes();
    assert!(used.len() > 30, "集めた class が少ない: {used:?}");
    let missing: Vec<&String> = used.iter().filter(|c| !css.contains(*c)).collect();
    assert!(missing.is_empty(), "stylesheet に無い class: {missing:?}");
}

/// data の口がまだ無い 4 つと ledger の指標の段は測れていない（理由の 1 行つき）・一覧と凡例は中身を出す。
#[test]
fn frame_blocks_without_data_are_unmeasured() {
    for fetched in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
    ] {
        for (id, body) in [
            (next::BLOCK.id, next::body(&fetched)),
            (pipeline::BLOCK.id, pipeline::body(&fetched)),
            (seat::BLOCK.id, seat::body(&fetched)),
            (map::BLOCK.id, map::body(&fetched)),
            ("ledger の指標", ledger::metrics(&fetched)),
        ] {
            match body {
                Body::Unmeasured(reason) => {
                    assert!(!reason.trim().is_empty(), "{id} の理由が空");
                }
                other => panic!("{id} が {fetched:?} で測れていないでない: {other:?}"),
            }
        }
    }

    let screen = known_screen();
    let Body::Filled(questions) = ask::body(&screen) else {
        panic!("問いの一覧が中身を出さない");
    };
    let ids: Vec<&str> = questions.iter().map(|q| q.id.as_str()).collect();
    assert_eq!(ids, vec!["bm.5", "bm.4"]);
    assert!(questions.iter().all(|q| q.alert));
    let Body::Filled(groups) = ledger::body(&screen) else {
        panic!("台帳の一覧が中身を出さない");
    };
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].head.as_ref().map(|h| h.id.as_str()), Some("bm"));
    let kids: Vec<&str> = groups[0].children.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(kids, vec!["bm.1", "bm.3", "bm.10", "bm.2"]);
    assert_eq!(legend::states().len(), 5);
    assert_eq!(legend::marks().len(), 5);

    // 読めない一覧は 0 件でなく測れていない・0 件は 0 件。
    let lost = screen.after_lost();
    assert!(matches!(ask::body(&lost), Body::Unmeasured(r) if !r.is_empty()));
    assert!(matches!(ledger::body(&lost), Body::Unmeasured(r) if !r.is_empty()));
    let empty_body = wire::encode(&LedgerList {
        rows: Reading::Known(vec![]),
    })
    .expect("電文");
    let empty = Screen::initial().after_read(&Fetched::Body(empty_body), 1);
    assert!(matches!(ask::body(&empty), Body::Empty(_)));
    assert!(matches!(ledger::body(&empty), Body::Empty(_)));
}

/// 印は閉じていれば塗り、open の問いは赤（見本の nodeShape と同じ）。
#[test]
fn frame_item_shape_follows_status() {
    let rows = fixture_rows();
    let by = |id: &str| {
        project::item(
            rows.iter()
                .find(|r| r.id.as_str() == id)
                .expect("fixture の行"),
        )
    };
    for row in &rows {
        let it = project::item(row);
        let open = matches!(row.status.as_str(), "open" | "in_progress");
        assert_eq!(it.shape.ends_with(" fill"), !open, "{}", row.id.as_str());
        assert!(it.aside.ends_with(&row.kind), "{}", it.aside);
    }
    assert!(by("bm.5").alert);
    assert!(!by("bm").alert);
}

/// block と地図の頁は project の下の 7 つの module に 1 つずつ・枠の module は中身を持たない。
#[test]
fn frame_one_module_per_block() {
    let modules = [
        ("next", next::BLOCK.id),
        ("ask", ask::BLOCK.id),
        ("pipeline", pipeline::BLOCK.id),
        ("seat", seat::BLOCK.id),
        ("ledger", ledger::BLOCK.id),
        ("legend", legend::BLOCK.id),
        ("map", map::BLOCK.id),
    ];
    let dir = crate_dir().join("src/project");
    let mut files: Vec<String> = std::fs::read_dir(&dir)
        .expect("src/project")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n != "mod.rs")
        .collect();
    files.sort();
    let mut want: Vec<String> = modules.iter().map(|(m, _)| format!("{m}.rs")).collect();
    want.sort();
    assert_eq!(files, want);
    for (module, id) in modules {
        let text = read(&format!("src/project/{module}.rs"));
        assert!(
            text.contains(&format!("id: \"{id}\"")),
            "{module}.rs が block {id} の枠を持たない"
        );
    }
    let mut all: Vec<&str> = frame::home().block_ids();
    all.extend(frame::map().block_ids());
    let mut declared: Vec<&str> = modules.iter().map(|(_, id)| *id).collect();
    all.sort_unstable();
    declared.sort_unstable();
    assert_eq!(all, declared, "枠の block と module の block が 1 対 1");
    let frame_src = read("src/frame.rs");
    for word in ["view!", "Screen", "Reading", "Body", "leptos"] {
        assert!(
            !frame_src.contains(word),
            "枠の module が中身の {word} を持つ"
        );
    }
}

/// 初心者と経験者の mode は URL の query に残る（頁の link も mode を持つ）。
#[test]
fn frame_mode_lives_in_url() {
    assert_eq!(Mode::from_query(""), Mode::Beginner);
    assert_eq!(Mode::from_query("?mode=expert"), Mode::Expert);
    assert_eq!(Mode::from_query("?page=map&mode=expert"), Mode::Expert);
    assert_eq!(Mode::from_query("?mode=bogus"), Mode::Beginner);
    assert_eq!(PageId::from_query("?page=map&mode=expert"), PageId::Map);
    assert_eq!(PageId::from_query("?mode=expert"), PageId::Home);
    assert_eq!(frame::with_param("", "mode", "expert"), "?mode=expert");
    assert_eq!(
        frame::with_param("?page=map", "mode", "expert"),
        "?page=map&mode=expert"
    );
    assert_eq!(
        frame::with_param("?mode=beginner&page=map", "mode", "expert"),
        "?mode=expert&page=map"
    );
    for mode in Mode::ALL {
        for page in [PageId::Home, PageId::Map] {
            let href = frame::href(page, mode);
            assert_eq!(Mode::from_query(&href), mode, "{href}");
            assert_eq!(PageId::from_query(&href), page, "{href}");
        }
    }
    let links = frame::nav_links(PageId::Map);
    let on: Vec<&str> = links
        .iter()
        .filter(|l| l.class == "on")
        .map(|l| l.key)
        .collect();
    assert_eq!(on, vec!["map"]);
}

/// 「?」の注釈: 1 行目 = 要点・項 = 記号 + 本文・「▸」の後 = 詳しく（見本の noteParts と同じ）。
#[test]
fn frame_help_note_splits_lines() {
    let n = help::note("st_unknown").expect("st_unknown の注釈");
    assert_eq!(n.label, "測れていない");
    assert_eq!(
        n.head,
        vec![
            Inline::Sym("st:unknown".to_string()),
            Inline::Text(" 記録が読めず状態が分からない".to_string())
        ]
    );
    assert_eq!(
        n.items,
        vec![Line {
            sym: vec![Inline::Text("✗".to_string())],
            text: vec![Inline::Text("0 件とは違う".to_string())],
        }]
    );
    assert!(n.more.is_empty());

    let s = help::note("status").expect("status の注釈");
    assert_eq!(s.items.len(), 4);
    assert_eq!(s.more.len(), 4);
    assert_eq!(s.more[0].sym, vec![Inline::Sym("st:unknown".to_string())]);
    let b = help::note("b:beads").expect("b:beads の注釈");
    assert_eq!(
        b.more,
        vec![Line {
            sym: vec![Inline::Sym("ic:file".to_string())],
            text: vec![Inline::Code(".beads/issues.jsonl".to_string())],
        }]
    );
    // rephrase の原語は内部の名の末尾に足す。
    let q = help::note("questions").expect("questions の注釈");
    assert_eq!(
        q.internal.last(),
        Some(&vec![Inline::Text("原語: 問い".to_string())])
    );

    assert_eq!(
        help::line("{fig:next}").text,
        vec![Inline::Sym("fig:next".to_string())]
    );
    assert_eq!(
        help::inline("a {b c"),
        vec![Inline::Text("a {b c".to_string())]
    );
    assert_eq!(help::tip_class(false, true), "tip");
    assert_eq!(help::tip_class(true, true), "tip on pin");
    assert_eq!(help::place(100.0, 50.0, 1280.0), (116.0, 62.0));
    assert_eq!(help::place(1200.0, 50.0, 1280.0), (892.0, 62.0));
    assert!(help::shows_internal(Mode::Expert));
    assert!(!help::shows_internal(Mode::Beginner));
}
