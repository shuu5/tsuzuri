//! 便 g-map の歯: 種類から帯と語の鍵への閉じた表・URL の query・
//! 着地済みの外形と依存・使う class と語の鍵が在る。id は自然な順（便 g-graph が直した）。
//! 圧縮の面と一覧の面の歯は行 m-map-compact で、表の面の歯は行 m-map-tree で消した。
//! 近傍の歯は節点の頁の歯（nodepage.rs・便 g-node が近傍の測れていないの歯を消した）。
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_contract::graph::{GraphDoc, GraphSource, NodeKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::band::{
    BEADS_LANES, Band, KINDS, band_of, kind_from_name, kind_key, kind_name,
};
use tsuzuri_surface::mapview::{decode, encode, natural, param, set_param};
use tsuzuri_surface::project::{NO_CONTENT, NOT_READ, map};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_text() -> String {
    read("../../tests/fixtures/surface/graph-doc.json")
}

fn fixture() -> GraphDoc {
    wire::decode(&fixture_text()).expect("fixture が電文として読める")
}

#[test]
fn mapview_fixture_is_small() {
    let doc = fixture();
    assert!(doc.nodes.len() <= 40, "節点 {}", doc.nodes.len());
    assert!(doc.edges.len() <= 60, "辺 {}", doc.edges.len());
}

/// (1) 種類の 20 個の全部が 7 つの帯のちょうど 1 つに当たり、語の鍵も 20 個の全部を持つ。
#[test]
fn mapview_kind_table_is_closed() {
    let kinds: Vec<NodeKind> = KINDS.iter().map(|r| r.kind).collect();
    assert_eq!(
        kinds,
        NodeKind::ALL.to_vec(),
        "表は種類の全部を 1 度ずつ持つ"
    );
    let want: [(NodeKind, &str, &str); 20] = [
        (NodeKind::Article, "constitution", "k:条"),
        (NodeKind::Norm, "constitution", "k:規範文"),
        (NodeKind::Rule, "rules", "k:規則行"),
        (NodeKind::Goal, "SRS", "k:目的"),
        (NodeKind::Req, "SRS", "k:要件"),
        (NodeKind::Nfr, "SRS", "k:非機能要件"),
        (NodeKind::Ac, "SRS", "k:受入基準"),
        (NodeKind::Constraint, "SRS", "k:制約"),
        (NodeKind::Actor, "SRS", "k:登場人物"),
        (NodeKind::Output, "SRS", "k:出力"),
        (NodeKind::Adr, "ADR", "k:判断の記録"),
        (NodeKind::NoteRow, "design-note", "k:設計ノートの行"),
        (NodeKind::Epic, "beads", "k:epic"),
        (NodeKind::Task, "beads", "k:契約"),
        (NodeKind::Memo, "beads", "k:memo"),
        (NodeKind::Question, "beads", "k:問い"),
        (NodeKind::Ruling, "beads", "k:裁定"),
        (NodeKind::Receipt, "beads", "k:受け"),
        (NodeKind::Policy, "beads", "k:方針"),
        (NodeKind::Run, "pipeline", "k:走行"),
    ];
    for (kind, band, key) in want {
        assert_eq!(band_of(kind).name(), band, "{kind:?} の帯");
        assert_eq!(kind_key(kind), key, "{kind:?} の語の鍵");
        assert!(vocab().term(key).is_some(), "鍵 {key} が vocab に無い");
        assert_eq!(kind_from_name(kind_name(kind)), Some(kind));
        let hits = Band::ALL.iter().filter(|b| **b == band_of(kind)).count();
        assert_eq!(hits, 1);
    }
    band_names_and_paths();
}

/// 帯の名・語の鍵・class の名と、出所の path と出所の種類・beads の種類の行の順。
fn band_names_and_paths() {
    let names: Vec<&str> = Band::ALL.iter().map(|b| b.name()).collect();
    assert_eq!(
        names,
        vec![
            "constitution",
            "rules",
            "ADR",
            "SRS",
            "design-note",
            "beads",
            "pipeline"
        ]
    );
    for b in Band::ALL {
        assert_eq!(Band::from_name(b.name()), Some(b));
        assert_eq!(b.key(), format!("b:{}", b.name()));
        assert_eq!(b.class_name(), format!("band-{}", b.name().to_lowercase()));
        assert!(
            vocab().term(b.key()).is_some(),
            "鍵 {} が vocab に無い",
            b.key()
        );
    }
    // 出所の path は見本の BAND_PATH の字。
    let paths: Vec<&str> = Band::ALL.iter().map(|b| b.path()).collect();
    assert_eq!(
        paths,
        vec![
            "design-intent/constitution.yaml",
            "design-intent/rules.yaml",
            "design-intent/adr/ADR-n.yaml",
            "design-intent/srs.yaml",
            "design-intent/design-note/*.yaml・docs/design/*.md",
            ".beads/issues.jsonl",
            "<state dir>/fleet/events.jsonl",
        ]
    );
    let sources: Vec<GraphSource> = Band::ALL.iter().map(|b| b.source()).collect();
    assert_eq!(
        sources,
        [
            vec![GraphSource::Design; 5],
            vec![GraphSource::Ledger, GraphSource::Runs]
        ]
        .concat()
    );
    assert_eq!(
        BEADS_LANES.to_vec(),
        vec![
            NodeKind::Epic,
            NodeKind::Task,
            NodeKind::Memo,
            NodeKind::Question,
            NodeKind::Ruling,
            NodeKind::Receipt,
            NodeKind::Policy
        ]
    );
    assert_eq!(Band::from_name("nope"), None);
    assert_eq!(kind_from_name("nope"), None);
}

/// query の値の `%XX` の読み書きと、置き換えと消し。
#[test]
fn mapview_query_codec() {
    assert_eq!(decode(&encode("契約|epic")), "契約|epic");
    assert_eq!(encode("a-b_c.d~e f"), "a-b_c.d~e%20f");
    assert_eq!(decode("a+b%7Cc"), "a b|c");
    assert_eq!(decode("%zz%"), "%zz%");
    assert_eq!(decode("%E5"), "\u{FFFD}");
    assert_eq!(param("?a=1&b=%E5%95%8F", "b").as_deref(), Some("問"));
    assert_eq!(param("?a", "a").as_deref(), Some(""));
    assert_eq!(param("?a=1", "b"), None);
    assert_eq!(set_param("", "view", Some("list")), "?view=list");
    assert_eq!(set_param("?view=list", "view", None), "?");
    assert_eq!(
        set_param("?a=1&view=x&view=y", "view", Some("z")),
        "?a=1&view=z"
    );
    assert_eq!(set_param("?a=1&&b=2", "c", Some("")), "?a=1&b=2");
}

/// 自然な順（数の並びは数として比べる）。
#[test]
fn mapview_natural_order() {
    use std::cmp::Ordering::*;
    assert_eq!(natural("ADR-2", "ADR-10"), Less);
    assert_eq!(natural("a.2", "a.10"), Less);
    assert_eq!(natural("FR14", "FR2"), Greater);
    assert_eq!(natural("t3", "t3-hub.2"), Less);
    assert_eq!(natural("t3.q2#r", "t3.q10"), Less);
    assert_eq!(natural("x01", "x1"), Less);
    assert_eq!(natural("x1", "x01"), Greater);
    assert_eq!(natural("b", "a"), Greater);
    assert_eq!(natural("", "a"), Less);
    assert_eq!(natural("a", "a"), Equal);
}

/// (9) グラフの口の path と読み（地図の block は行 m-map-page で消した）と、電文の型として読めない本文は理由・
/// 外の依存は足さない。
#[test]
fn mapview_landed_shape_and_body() {
    assert_eq!(map::PATH, "/api/graph");
    assert_eq!(map::doc(&Fetched::NotRead), Err(NOT_READ));
    assert_eq!(map::doc(&Fetched::Failed), Err(map::REASON));
    for text in ["{}", "not json", "[]", r#"{"nodes": []}"#] {
        assert_eq!(
            map::doc(&Fetched::Body(text.to_string())),
            Err(NO_CONTENT),
            "{text}"
        );
    }
    assert_eq!(map::doc(&Fetched::Body(fixture_text())), Ok(fixture()));

    // 直接依存は着地前と同じ（[dependencies] は契約の型の crate だけ）。
    let manifest = read("Cargo.toml");
    let mut inside = false;
    let mut deps = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == "[dependencies]";
        } else if inside
            && !line.starts_with('#')
            && let Some((k, _)) = line.split_once('=')
        {
            deps.push(k.trim().to_string());
        }
    }
    assert_eq!(deps, vec!["tsuzuri-contract".to_string()]);
}

/// stylesheet の class の名（selector の `.名`・注釈を除く）。
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

/// 地図の部品が使う class は stylesheet に在り、語の鍵は vocab に在る（グラフの面の DOM は行 m-map-graph で消した）。
#[test]
fn mapview_classes_and_keys_exist() {
    let mut used = BTreeSet::new();
    let mut add = |s: &str| {
        for c in s.split_whitespace() {
            used.insert(c.to_string());
        }
    };
    let files = [
        "src/mapview/mod.rs",
        "src/mapview/band.rs",
        "src/mapview/graph.rs",
        "src/mapview/around.rs",
    ];
    for f in files {
        let text = read(f);
        let mut rest = text.as_str();
        while let Some(i) = rest.find("class=\"") {
            rest = &rest[i + 7..];
            let end = rest.find('"').expect("class の字の終わり");
            add(&rest[..end]);
            rest = &rest[end..];
        }
    }
    doc_classes(add);
    used_in_css(used);
}

/// 帯の印の class を足す（圧縮の面の札と一覧の面の行は行 m-map-compact で、表の面の升は行 m-map-tree で消した）。
fn doc_classes(mut add: impl FnMut(&str)) {
    for b in Band::ALL {
        add(&format!("shape fill {}", b.class_name()));
    }
}

/// 使う class は stylesheet に在り、語の鍵は vocab に在る。
fn used_in_css(used: BTreeSet<String>) {
    let css = stylesheet_classes();
    for c in ["node", "hit", "lk", "shape"] {
        assert!(used.contains(c), "地図の頁が class {c} を使わない");
    }
    let missing: Vec<&String> = used.iter().filter(|c| !css.contains(*c)).collect();
    assert!(missing.is_empty(), "stylesheet に無い class: {missing:?}");

    assert!(
        vocab().term("st_unknown").is_some(),
        "鍵 st_unknown が vocab に無い"
    );
}
