//! 便 g-gaps の歯: 抜けの検査の頁の枠と header の link・数の札（3 枚）・一覧の並びと最初に開く段・
//! 12 本の名の表（表に無い id は id の字）・20 件の切り方・測れていないと 0 本の区別・class と語の鍵・
//! fixture の大きさ・足す外の依存は 0 本。

use std::path::PathBuf;

use tsuzuri_contract::graph::{InvariantCheck, Verdict};
use tsuzuri_surface::frame::{self, HEADER, Mode, PageId};
use tsuzuri_surface::project::{Body, NOT_READ, gaps, map};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

const FIXTURE: &str = "../../tests/fixtures/surface/invariants.json";

fn fixture() -> Fetched {
    Fetched::Body(read(FIXTURE))
}

fn filled() -> gaps::Gaps {
    match gaps::body(&fixture()) {
        Body::Filled(g) => g,
        other => panic!("fixture が中身にならない: {other:?}"),
    }
}

fn check(id: &str, verdict: Verdict, ids: &[&str]) -> InvariantCheck {
    InvariantCheck {
        id: id.to_string(),
        verdict,
        violations: ids.len() as u32,
        ids: ids.iter().map(|s| s.to_string()).collect(),
    }
}

/// 頁の順は home・ask・map・gaps、抜けの検査の頁の block は gaps の 1 つ、header の link は 4 つ。
#[test]
fn gapspage_page_frame_and_nav() {
    let ids: Vec<&str> = frame::pages().iter().map(|p| p.id.id()).collect();
    assert_eq!(ids, vec!["home", "ask", "map", "gaps"]);
    assert_eq!(PageId::ALL.map(PageId::id), ["home", "ask", "map", "gaps"]);
    let page = frame::gaps();
    assert_eq!(page.id, PageId::Gaps);
    assert_eq!(page.heading, "gaps");
    assert_eq!(page.block_ids(), vec!["gaps"]);
    assert_eq!(gaps::BLOCK.id, "gaps");
    assert_eq!(gaps::BLOCK.heading, "gaps");
    assert_eq!(PageId::from_query("?page=gaps&mode=expert"), PageId::Gaps);
    assert_eq!(
        frame::href(PageId::Gaps, Mode::Beginner),
        "?page=gaps&mode=beginner"
    );
    assert_eq!(HEADER[1].items, ["home", "questions", "map", "gaps"]);
    let links = frame::nav_links(PageId::Gaps);
    let keys: Vec<&str> = links.iter().map(|l| l.key).collect();
    assert_eq!(keys, vec!["home", "questions", "map", "gaps"]);
    let on: Vec<&str> = links
        .iter()
        .filter(|l| l.class == "on")
        .map(|l| l.key)
        .collect();
    assert_eq!(on, vec!["gaps"]);
    assert_eq!(vocab().label("gaps"), "抜けの検査");
    let snapshot = read("tests/snapshots/frame.json");
    assert!(snapshot.contains("{\"id\": \"gaps\", \"heading\": \"gaps\", \"class\": \"panel\"}"));
}

/// 数の札は 3 枚（違反・まだ分からない・合格の順）・数は fixture の判定の数・記号の字と class と語の鍵。
#[test]
fn gapspage_tiles_count_fixture() {
    let g = filled();
    let got: Vec<(&str, &str, &str, usize)> = g
        .tiles
        .iter()
        .map(|t| (t.mark.class, t.mark.glyph, t.mark.key, t.count))
        .collect();
    assert_eq!(
        got,
        vec![
            ("ng", "✕", "gap_ng", 2),
            ("unknown", "?", "gap_unknown", 3),
            ("ok", "✓", "gap_ok", 7),
        ]
    );
    assert_eq!(g.tiles.iter().map(|t| t.count).sum::<usize>(), 12);
    assert_eq!(gaps::mark_class(gaps::mark(Verdict::Violation)), "gi ng");
    // 3 値のどれも記号を持つ（閉じた一覧の全部）。
    for v in Verdict::ALL {
        assert!(gaps::ORDER.contains(&v));
    }
}

/// 一覧は 12 行・違反・まだ分からない・合格の順（同じ判定の中は id の数の順）・違反の行だけが最初は開く。
#[test]
fn gapspage_rows_order_and_open() {
    let rows = filled().rows;
    let ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "g-2", "g-10", "g-3", "g-9", "g-12", "g-1", "g-4", "g-5", "g-6", "g-7", "g-8", "g-11"
        ]
    );
    let open: Vec<&str> = rows
        .iter()
        .filter(|r| r.open)
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(open, vec!["g-2", "g-10"]);
    for r in &rows {
        assert_eq!(r.mark, gaps::mark(r.verdict));
        assert!(!r.name.is_empty());
    }
    let counts: Vec<Option<u32>> = rows.iter().map(|r| r.count).collect();
    let mut want = vec![Some(2), Some(25)];
    want.extend([None; 10]);
    assert_eq!(counts, want);
    assert_eq!(rows[0].name, "open の task は design-note の行を 1 つ指す");
    assert_eq!(rows[0].named, vec!["t3-hub.9", "t3-hub.15"]);
    assert_eq!(rows[0].more, None);
    assert_eq!(rows[2].mark.glyph, "?");
    assert_eq!(rows[11].mark.glyph, "✓");
    // 並びは電文の順に依らない（逆に並べた電文でも同じ）。
    let mut reversed = vec![
        check("g-11", Verdict::Pass, &[]),
        check("g-2", Verdict::Pass, &[]),
        check("g-10", Verdict::Unknown, &[]),
        check("g-9", Verdict::Unknown, &[]),
        check("g-12", Verdict::Violation, &["x"]),
        check("g-1", Verdict::Violation, &["y"]),
    ];
    reversed.reverse();
    let order: Vec<String> = gaps::rows(&reversed).into_iter().map(|r| r.id).collect();
    assert_eq!(order, vec!["g-1", "g-12", "g-9", "g-10", "g-2", "g-11"]);
}

/// 12 本の id と名の表は 12 行で g-1 から g-12 の全部・表に無い id は id の字を名の代わりに出す。
#[test]
fn gapspage_names_table() {
    assert_eq!(gaps::NAMES.len(), 12);
    let ids: Vec<&str> = gaps::NAMES.iter().map(|(id, _)| *id).collect();
    let want: Vec<String> = (1..=12).map(|n| format!("g-{n}")).collect();
    assert_eq!(ids, want);
    for (id, name) in gaps::NAMES {
        assert!(!name.is_empty(), "{id} の名が空");
        assert_eq!(gaps::name(id), name);
    }
    assert_eq!(gaps::name("g-1"), "線の両端が実在する");
    assert_eq!(gaps::name("g-12"), "止まった run の question が記録にある");
    assert_eq!(gaps::name("g-99"), "g-99");
    let rows = gaps::rows(&[
        check("g-99", Verdict::Pass, &[]),
        check("g-3", Verdict::Pass, &[]),
    ]);
    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "発効した ADR と rule に、あなたの決定が結ばれている",
            "g-99"
        ]
    );
}

/// 名指しの一覧は 20 件まで・超える分は残りの数の 1 行。
#[test]
fn gapspage_named_cut_at_twenty() {
    let rows = filled().rows;
    let many = &rows[1];
    assert_eq!(many.id, "g-10");
    assert_eq!(many.named.len(), gaps::LIMIT);
    assert_eq!(gaps::LIMIT, 20);
    assert_eq!(many.named.first().map(String::as_str), Some("n-1"));
    assert_eq!(many.named.last().map(String::as_str), Some("n-20"));
    assert_eq!(many.more, Some(5));
    let ids = |n: usize| (1..=n).map(|i| format!("x-{i}")).collect::<Vec<_>>();
    assert_eq!(gaps::cut(&ids(0)), (vec![], None));
    assert_eq!(gaps::cut(&ids(20)), (ids(20), None));
    assert_eq!(gaps::cut(&ids(21)), (ids(20), Some(1)));
}

/// 口が読めない・まだ読んでいない・本文が電文の型として読めないは、合格と出さず測れていないと理由の 1 行。
#[test]
fn gapspage_unmeasured_never_pass() {
    for fetched in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("not json".to_string()),
        Fetched::Body("{}".to_string()),
        Fetched::Body("{\"invariants\": []}".to_string()),
    ] {
        let Body::Unmeasured(reason) = gaps::body(&fetched) else {
            panic!("{fetched:?} で測れていないでない");
        };
        assert!(!reason.trim().is_empty() && !reason.contains('\n'));
        assert!(gaps::checks(&fetched).is_err());
    }
    assert_eq!(gaps::body(&Fetched::NotRead), Body::Unmeasured(NOT_READ));
    assert_eq!(gaps::body(&Fetched::Failed), Body::Unmeasured(gaps::REASON));
    assert_eq!(
        gaps::body(&Fetched::Body("not json".to_string())),
        Body::Unmeasured(gaps::UNREADABLE)
    );
    // 判定が 0 本の電文は 0 本（測れていないとは分ける）。
    let text = read(FIXTURE);
    let start = text.find("\"invariants\"").expect("invariants の欄");
    let end = text.find("\"skipped\"").expect("skipped の欄");
    let none = format!("{}\"invariants\": [],\n  {}", &text[..start], &text[end..]);
    assert_eq!(gaps::body(&Fetched::Body(none)), Body::Empty(gaps::EMPTY));
}

/// 口は地図と同じ /api/graph の定数を使い、面の code は判定を数えない（電文の節点と線を見ない）。
#[test]
fn gapspage_reads_graph_and_copies_verdicts() {
    assert_eq!(map::PATH, "/api/graph");
    let src = read("src/project/gaps.rs");
    assert!(src.contains("map::PATH"));
    for word in [".nodes", ".edges", ".beads", ".runs", "\"/api/"] {
        assert!(!src.contains(word), "gaps.rs に {word} が在る");
    }
    for name in ["localStorage", "sessionStorage", "cookie"] {
        assert!(!src.to_lowercase().contains(&name.to_lowercase()));
    }
}

/// この頁が使う class は stylesheet に在り、語の鍵は語の辞書に在る。
#[test]
fn gapspage_classes_and_keys_exist() {
    let css = read("style.css");
    let mut used: Vec<String> = Vec::new();
    let src = read("src/project/gaps.rs");
    let mut rest = src.as_str();
    while let Some(i) = rest.find("class=\"") {
        rest = &rest[i + 7..];
        let end = rest.find('"').expect("class の字の終わり");
        used.extend(rest[..end].split_whitespace().map(str::to_string));
        rest = &rest[end..];
    }
    for v in gaps::ORDER {
        used.extend(
            gaps::mark_class(gaps::mark(v))
                .split_whitespace()
                .map(str::to_string),
        );
    }
    used.extend(gaps::BLOCK.class.split_whitespace().map(str::to_string));
    for c in [
        "gtiles", "gtile", "gi", "ok", "ng", "unknown", "n", "gitem", "ttl", "aside", "body",
        "items", "panel",
    ] {
        assert!(used.iter().any(|u| u == c), "gaps が class {c} を使わない");
    }
    for c in &used {
        assert!(css.contains(&format!(".{c}")), "stylesheet に .{c} が無い");
    }
    for key in ["gaps", "gap_ok", "gap_ng", "gap_unknown", "st_unknown"] {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty());
    }
}

/// fixture は 12 本（違反 2・まだ分からない 3・合格 7・うち 1 本は違反の id を 25 個）で 5000 byte 以下。
#[test]
fn gapspage_fixture_shape_and_size() {
    let len = read(FIXTURE).len();
    assert!(len <= 5000, "{FIXTURE} が {len} byte");
    let checks = gaps::checks(&fixture()).expect("fixture の電文");
    assert_eq!(checks.len(), 12);
    assert_eq!(checks.iter().filter(|c| c.ids.len() == 25).count(), 1);
}

/// 足す外の依存は 0 本（面の crate の依存の名は便 g-parts の着地のまま）。
#[test]
fn gapspage_no_new_dependencies() {
    let text = read("Cargo.toml");
    let mut inside = false;
    let mut names: Vec<String> = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(table) = line.strip_prefix('[') {
            let table = table.trim_end_matches(']');
            inside = table.ends_with("dependencies");
            if let Some((head, name)) = table.rsplit_once('.')
                && head.ends_with("dependencies")
            {
                names.push(name.to_string());
            }
        } else if inside
            && !line.starts_with('#')
            && let Some((k, _)) = line.split_once('=')
        {
            let k = k.trim();
            names.push(k.split('.').next().unwrap_or(k).to_string());
        }
    }
    names.sort();
    names.dedup();
    assert_eq!(
        names,
        vec![
            "leptos",
            "tsuzuri-boundary",
            "tsuzuri-contract",
            "wasm-bindgen-futures",
            "web-sys"
        ]
    );
}
