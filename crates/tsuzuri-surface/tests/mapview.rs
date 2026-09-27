//! 便 g-map の歯: 種類から帯と語の鍵への閉じた表・圧縮の面の帯の順と札の並び・読めない帯と 0 件の帯・
//! 一覧の面の絞りと並べ替え・表の面の行列の数と cell の組の絞り・URL の query・全部の札と行の id・
//! グラフの面と近傍は測れていない・着地済みの外形と依存・使う class と語の鍵が在る。

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_contract::graph::{GraphDoc, GraphSource, NodeKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::Block;
use tsuzuri_surface::mapview::band::{
    BEADS_LANES, Band, KINDS, band_of, kind_from_name, kind_key, kind_name, unread_reason,
};
use tsuzuri_surface::mapview::compact::{BandBox, Cards, compact, cut30};
use tsuzuri_surface::mapview::list::{
    Listing, NO_STATE, Query, Sort, listing, pair_value, parse_pair, with_choice, with_pair,
};
use tsuzuri_surface::mapview::table::{Matrix, matrix};
use tsuzuri_surface::mapview::{
    View, around, decode, encode, graph, natural, param, set_param, with_view,
};
use tsuzuri_surface::project::{Body, NO_CONTENT, NOT_READ, map};
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

fn boxes(doc: &GraphDoc) -> Vec<BandBox> {
    compact(doc)
}

fn rows(l: &Listing) -> Vec<&str> {
    l.rows.iter().map(|r| r.id.as_str()).collect()
}

fn list_of(doc: &GraphDoc, search: &str) -> Vec<String> {
    rows(&listing(doc, &Query::from_search(search)))
        .into_iter()
        .map(str::to_string)
        .collect()
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

/// (2) 圧縮の面の帯の順と、帯ごとの札の id の並び・beads は種類の 7 行。
#[test]
fn mapview_compact_bands_and_tags_in_order() {
    let doc = fixture();
    let bs = boxes(&doc);
    let bands: Vec<Band> = bs.iter().map(|b| b.band).collect();
    assert_eq!(bands, Band::ALL.to_vec());
    let classes: Vec<String> = bs.iter().map(BandBox::class).collect();
    assert_eq!(classes[0], "band band-constitution");
    assert_eq!(classes[2], "band band-adr");
    assert_eq!(classes[4], "band band-design-note");
    let ids: Vec<Vec<&str>> = bs.iter().map(BandBox::ids).collect();
    assert_eq!(
        ids,
        vec![
            vec![
                "P-1", "P-1.1", "P-1.2", "A-2", "A-2.1", "N-1", "N-1.2", "N-1.10", "P-9.1"
            ],
            vec!["R-25", "D-3", "R-4"],
            vec!["ADR-2", "ADR-7", "ADR-10"],
            vec!["AC-3", "FR2", "FR14", "G-1", "NFR2"],
            vec!["surface-board#g-frame", "surface-board#g-map"],
            vec![
                "t3",
                "t3-hub.2",
                "t3-hub.9",
                "t3-hub.15",
                "t3.m1",
                "t3.q2",
                "t3.q10",
                "t3.q2#r",
                "t3.p"
            ],
            vec!["r-1", "r-3", "r-12"],
        ]
    );
    let counts: Vec<Option<usize>> = bs.iter().map(|b| b.count).collect();
    assert_eq!(
        counts,
        vec![
            Some(9),
            Some(3),
            Some(3),
            Some(5),
            Some(2),
            Some(9),
            Some(3)
        ]
    );

    // constitution: 条の札は電文の順・規範文は条の下に自然な順・条の無い規範文は後に札で出す。
    let Cards::Articles { articles, loose } = &bs[0].cards else {
        panic!("constitution が条の札でない: {:?}", bs[0].cards);
    };
    let arts: Vec<(&str, Vec<&str>)> = articles
        .iter()
        .map(|a| {
            (
                a.tag.id.as_str(),
                a.norms.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(
        arts,
        vec![
            ("P-1", vec!["P-1.1", "P-1.2"]),
            ("A-2", vec!["A-2.1"]),
            ("N-1", vec!["N-1.2", "N-1.10"]),
        ]
    );
    assert_eq!(
        loose.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
        vec!["P-9.1"]
    );

    // beads: 種類の 7 行（0 件の行も出す）。
    let Cards::Lanes(lanes) = &bs[5].cards else {
        panic!("beads が種類の行でない: {:?}", bs[5].cards);
    };
    let lane_kinds: Vec<NodeKind> = lanes.iter().map(|l| l.kind).collect();
    assert_eq!(lane_kinds, BEADS_LANES.to_vec());
    let lane_ids: Vec<Vec<&str>> = lanes
        .iter()
        .map(|l| l.tags.iter().map(|t| t.id.as_str()).collect())
        .collect();
    assert_eq!(
        lane_ids,
        vec![
            vec!["t3"],
            vec!["t3-hub.2", "t3-hub.9", "t3-hub.15"],
            vec!["t3.m1"],
            vec!["t3.q2", "t3.q10"],
            vec!["t3.q2#r"],
            vec![],
            vec!["t3.p"],
        ]
    );
    let keys: Vec<&str> = lanes.iter().map(|l| l.key).collect();
    assert_eq!(
        keys,
        vec![
            "k:epic", "k:契約", "k:memo", "k:問い", "k:裁定", "k:受け", "k:方針"
        ]
    );

    // 札: 題は 30 字・open の問いは赤・動いている節点は印を塗らない。
    let Cards::Tags(rules) = &bs[1].cards else {
        panic!("rules が札でない");
    };
    assert_eq!(rules[2].title, "abcdefghij abcdefghij abcdefgh");
    assert_eq!(cut30("  a   b  "), "a b");
    let q10 = &lanes[3].tags[1];
    assert!(q10.alert);
    assert_eq!(q10.class, "tag band-beads open-q");
    assert_eq!(q10.shape, "shape band-beads");
    let q2 = &lanes[3].tags[0];
    assert!(!q2.alert);
    assert_eq!(q2.class, "tag band-beads");
    assert_eq!(q2.shape, "shape band-beads fill");
    assert_eq!(rules[0].shape, "shape band-rules fill");
    assert_eq!(
        lanes[1].tags[1].shape, "shape band-beads",
        "in_progress は塗らない"
    );
}

/// (3) 読めなかった出所の帯は測れていない（0 件でない）・読めて 0 件の帯は 0 件の帯。
#[test]
fn mapview_unread_band_is_unmeasured_and_zero_band_is_empty() {
    let mut doc = fixture();
    doc.unread = vec![GraphSource::Runs];
    let bs = boxes(&doc);
    assert_eq!(bs[6].count, None);
    assert_eq!(
        bs[6].cards,
        Cards::Unmeasured(unread_reason(GraphSource::Runs))
    );
    assert!(bs[..6].iter().all(|b| b.count.is_some()));

    doc.unread = vec![GraphSource::Design];
    let bs = boxes(&doc);
    for b in &bs[..5] {
        assert_eq!(
            b.cards,
            Cards::Unmeasured(unread_reason(GraphSource::Design))
        );
        assert_eq!(b.count, None);
    }
    assert!(matches!(bs[5].cards, Cards::Lanes(_)));
    let l = listing(&doc, &Query::from_search(""));
    assert_eq!(l.unread, vec![unread_reason(GraphSource::Design)]);

    doc.unread = vec![GraphSource::Ledger];
    assert_eq!(
        boxes(&doc)[5].cards,
        Cards::Unmeasured(unread_reason(GraphSource::Ledger))
    );

    let mut zero = fixture();
    zero.nodes.retain(|n| n.kind != NodeKind::Run);
    let bs = boxes(&zero);
    assert_eq!(bs[6].count, Some(0));
    assert_eq!(bs[6].cards, Cards::Empty);
    for s in GraphSource::ALL {
        assert!(!unread_reason(s).trim().is_empty());
    }
}

/// (4) 一覧の面の絞り（帯・種類・種類の組）と並べ替え（id・状態）。
#[test]
fn mapview_list_filters_and_sorts() {
    let doc = fixture();
    assert_eq!(
        list_of(&doc, "?page=map&view=list"),
        vec![
            "P-1",
            "P-1.2",
            "P-1.1",
            "A-2",
            "A-2.1",
            "N-1",
            "N-1.10",
            "N-1.2",
            "P-9.1",
            "R-25",
            "D-3",
            "R-4",
            "ADR-2",
            "ADR-7",
            "ADR-10",
            "AC-3",
            "FR2",
            "FR14",
            "G-1",
            "NFR2",
            "surface-board#g-frame",
            "surface-board#g-map",
            "t3",
            "t3-hub.2",
            "t3-hub.9",
            "t3-hub.15",
            "t3.m1",
            "t3.p",
            "t3.q2",
            "t3.q2#r",
            "t3.q10",
            "r-1",
            "r-3",
            "r-12",
        ]
    );
    assert_eq!(
        list_of(&doc, "?view=list&band=beads&sort=state"),
        vec![
            "t3.q10",
            "t3",
            "t3-hub.9",
            "t3-hub.15",
            "t3.m1",
            "t3.p",
            "t3.q2#r",
            "t3-hub.2",
            "t3.q2",
        ]
    );
    assert_eq!(
        list_of(&doc, "?band=pipeline&sort=state"),
        vec!["r-12", "r-1", "r-3"]
    );
    let kind = format!("?kind={}", encode("要件"));
    assert_eq!(list_of(&doc, &kind), vec!["FR2", "FR14"]);
    assert_eq!(list_of(&doc, "?kind=要件"), vec!["FR2", "FR14"]);

    let pair = format!(
        "?pair={}",
        encode(&pair_value(NodeKind::Task, NodeKind::Epic))
    );
    let l = listing(&doc, &Query::from_search(&pair));
    assert_eq!(rows(&l), vec!["t3", "t3-hub.2", "t3-hub.9", "t3-hub.15"]);
    let note = l.pair.expect("組の札");
    assert_eq!(
        (note.from, note.to, note.count),
        (NodeKind::Task, NodeKind::Epic, 4)
    );
    let pair_state = format!(
        "?sort=state&pair={}",
        encode(&pair_value(NodeKind::Question, NodeKind::Task))
    );
    assert_eq!(list_of(&doc, &pair_state), vec!["t3.q10", "t3-hub.15"]);
    let pair_band = format!("{pair}&kind={}", encode("契約"));
    assert_eq!(
        list_of(&doc, &pair_band),
        vec!["t3-hub.2", "t3-hub.9", "t3-hub.15"]
    );

    // 状態の語: bead は状態・走行は段・設計文書の節点と段の無い走行は状態なし。
    let all = listing(&doc, &Query::from_search(""));
    let word = |id: &str| {
        all.rows
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.state_word().to_string())
            .expect("行")
    };
    assert_eq!(word("t3-hub.9"), "in_progress");
    assert_eq!(word("r-1"), "Landed");
    assert_eq!(word("r-12"), NO_STATE);
    assert_eq!(word("P-1"), NO_STATE);
    let r4 = all.rows.iter().find(|r| r.id == "R-4").expect("R-4");
    assert_eq!(r4.title, "abcdefghij abcdefghij abcdefghij abc");
    assert!(
        all.rows
            .iter()
            .find(|r| r.id == "t3.q10")
            .is_some_and(|r| r.alert)
    );
    // 種類の選択肢は電文に在る種類（契約の型の順）。
    assert!(!all.kinds.contains(&NodeKind::Receipt));
    assert_eq!(all.kinds.first(), Some(&NodeKind::Article));
    assert_eq!(all.kinds.last(), Some(&NodeKind::Run));
    assert!(all.pair.is_none());
    assert!(all.unread.is_empty());
}

/// (5) 表の面の行列の数は fixture の辺から数えた期待と一致し、cell の組で一覧の面が絞られる。
#[test]
fn mapview_table_counts_and_pair_filter() {
    let doc = fixture();
    let m: Matrix = matrix(&doc);
    use NodeKind::*;
    assert_eq!(
        m.kinds,
        vec![
            Article, Norm, Rule, Goal, Req, Adr, NoteRow, Epic, Task, Question, Ruling, Run
        ]
    );
    let cells: Vec<(NodeKind, NodeKind, usize)> =
        m.cells.iter().map(|c| (c.from, c.to, c.count)).collect();
    assert_eq!(
        cells,
        vec![
            (Norm, Article, 5),
            (Rule, Article, 2),
            (Req, Goal, 2),
            (Adr, Article, 1),
            (Adr, Rule, 1),
            (Task, NoteRow, 2),
            (Task, Epic, 3),
            (Question, Epic, 1),
            (Question, Task, 1),
            (Ruling, Question, 1),
            (Run, Task, 3),
        ]
    );
    // 数は辺の数（両端の在る辺だけ）と合う。
    let known: BTreeSet<&str> = doc.nodes.iter().map(|n| n.id.as_str()).collect();
    let counted = doc
        .edges
        .iter()
        .filter(|e| known.contains(e.from.as_str()) && known.contains(e.to.as_str()))
        .count();
    assert_eq!(m.cells.iter().map(|c| c.count).sum::<usize>(), counted);
    assert_eq!(m.count(Article, Norm), 0);
    assert_eq!(m.count(Norm, Article), 5);
    let adr = m.cell(Adr, Article).expect("cell");
    assert_eq!(adr.types_text(), "relations.articles");
    assert_eq!(adr.shape(), "shape band-adr fill");

    // cell を押した後の URL で一覧の面が組の両端に絞られる。
    let url = with_pair("?page=map&view=table&band=rules", Some((Run, Task)));
    assert_eq!(View::from_query(&url), View::List);
    assert_eq!(
        list_of(&doc, &url),
        vec!["t3-hub.2", "t3-hub.15", "r-1", "r-3", "r-12"]
    );
    for c in &m.cells {
        let url = with_pair("", Some((c.from, c.to)));
        let l = listing(&doc, &Query::from_search(&url));
        assert!(!l.rows.is_empty());
        assert!(
            l.rows.iter().all(|r| r.kind == c.from || r.kind == c.to),
            "{:?}",
            (c.from, c.to)
        );
    }
}

/// (6) 面の切り替えと絞りと並べ替えが URL の query に残り、知らない view の値は圧縮の面。
#[test]
fn mapview_query_keeps_view_and_filters() {
    assert_eq!(View::from_query(""), View::Compact);
    assert_eq!(View::from_query("?page=map"), View::Compact);
    assert_eq!(View::from_query("?view=bogus"), View::Compact);
    assert_eq!(View::from_query("?view="), View::Compact);
    for v in View::ALL {
        let url = with_view("?page=map&mode=expert", v);
        assert_eq!(View::from_query(&url), v);
        assert_eq!(param(&url, "mode").as_deref(), Some("expert"));
        assert_eq!(param(&url, "page").as_deref(), Some("map"));
        assert!(
            vocab().term(v.key()).is_some(),
            "鍵 {} が vocab に無い",
            v.key()
        );
    }
    assert_eq!(
        with_view("?page=map&mode=expert", View::List),
        "?page=map&mode=expert&view=list"
    );
    assert_eq!(
        with_view("?view=list&page=map", View::Table),
        "?view=table&page=map"
    );

    let q = Query::from_search(&format!(
        "?page=map&view=list&band=beads&kind={}&sort=state",
        encode("問い")
    ));
    assert_eq!(q.band, Some(Band::Beads));
    assert_eq!(q.kind, Some(NodeKind::Question));
    assert_eq!(q.sort, Sort::State);
    assert_eq!(q.pair, None);
    let q = Query::from_search("?band=nope&kind=nope&sort=nope&pair=a|b");
    assert_eq!(
        (q.band, q.kind, q.sort, q.pair),
        (None, None, Sort::Id, None)
    );

    let s = with_choice("?page=map&view=list", "sort", "state");
    assert_eq!(s, "?page=map&view=list&sort=state");
    let s = with_choice(&s, "band", "rules");
    assert_eq!(Query::from_search(&s).band, Some(Band::Rules));
    let s = with_choice(&s, "band", "");
    assert_eq!(s, "?page=map&view=list&sort=state");

    let url = with_pair(
        "?page=map&view=table&band=rules&kind=x&mode=expert",
        Some((NodeKind::Task, NodeKind::Epic)),
    );
    assert_eq!(
        url,
        format!(
            "?page=map&view=list&mode=expert&pair={}",
            encode("契約|epic")
        )
    );
    let q = Query::from_search(&url);
    assert_eq!(q.pair, Some((NodeKind::Task, NodeKind::Epic)));
    assert_eq!((q.band, q.kind), (None, None));
    assert_eq!(with_pair(&url, None), "?page=map&view=list&mode=expert");
    assert_eq!(
        parse_pair("契約|epic"),
        Some((NodeKind::Task, NodeKind::Epic))
    );
    assert_eq!(parse_pair("契約"), None);
    for s in Sort::ALL {
        assert!(
            vocab().term(s.key()).is_some(),
            "鍵 {} が vocab に無い",
            s.key()
        );
    }
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

/// (7) 圧縮と一覧の全部の札と行に節点の id が出る（1 度ずつ）。
#[test]
fn mapview_every_tag_and_row_shows_id() {
    let doc = fixture();
    let mut want: Vec<&str> = doc.nodes.iter().map(|n| n.id.as_str()).collect();
    want.sort_unstable();
    let bs = boxes(&doc);
    let mut shown: Vec<&str> = bs.iter().flat_map(BandBox::ids).collect();
    shown.sort_unstable();
    assert_eq!(shown, want);
    let l = listing(&doc, &Query::from_search(""));
    let mut listed = rows(&l);
    listed.sort_unstable();
    assert_eq!(listed, want);
    assert!(l.rows.iter().all(|r| !r.id.is_empty()));
}

/// (8) グラフの面と近傍は測れていないと理由の 1 行。
#[test]
fn mapview_graph_and_around_are_unmeasured() {
    assert_eq!(graph::body(), Body::Unmeasured(graph::REASON));
    assert_eq!(around::body(), Body::Unmeasured(around::REASON));
    assert!(!graph::REASON.trim().is_empty());
    assert!(!around::REASON.trim().is_empty());
}

/// (9) 着地済みの外形（BLOCK・口の path・body）と、電文の型として読めない本文は測れていない・外の依存は足さない。
#[test]
fn mapview_landed_shape_and_body() {
    assert_eq!(
        map::BLOCK,
        Block {
            id: "map",
            heading: "map",
            class: "panel"
        }
    );
    assert_eq!(map::PATH, "/api/graph");
    let body: fn(&Fetched) -> Body<()> = map::body;
    assert_eq!(body(&Fetched::NotRead), Body::Unmeasured(NOT_READ));
    assert_eq!(body(&Fetched::Failed), Body::Unmeasured(map::REASON));
    for text in ["{}", "not json", "[]", r#"{"nodes": []}"#] {
        assert_eq!(
            body(&Fetched::Body(text.to_string())),
            Body::Unmeasured(NO_CONTENT),
            "{text}"
        );
    }
    assert_eq!(body(&Fetched::Body(fixture_text())), Body::Filled(()));
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

/// 地図の頁が使う class は stylesheet に在り、語の鍵は vocab に在る。
#[test]
fn mapview_classes_and_keys_exist() {
    let mut used = BTreeSet::new();
    let mut add = |s: &str| {
        for c in s.split_whitespace() {
            used.insert(c.to_string());
        }
    };
    let files = [
        "src/project/map.rs",
        "src/mapview/mod.rs",
        "src/mapview/band.rs",
        "src/mapview/compact.rs",
        "src/mapview/list.rs",
        "src/mapview/table.rs",
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
    let doc = fixture();
    for b in boxes(&doc) {
        add(&b.class());
        match &b.cards {
            Cards::Tags(tags) => tags.iter().for_each(|t| {
                add(&t.class);
                add(&t.shape);
            }),
            Cards::Articles { articles, loose } => {
                for t in articles.iter().map(|a| &a.tag).chain(loose) {
                    add(&t.class);
                    add(&t.shape);
                }
            }
            Cards::Lanes(lanes) => lanes.iter().flat_map(|l| &l.tags).for_each(|t| {
                add(&t.class);
                add(&t.shape);
            }),
            Cards::Unmeasured(_) | Cards::Empty => {}
        }
    }
    for r in &listing(&doc, &Query::from_search("")).rows {
        add(&r.shape);
        add(&r.band.chip_class());
    }
    for c in &matrix(&doc).cells {
        add(&c.shape());
    }
    for b in Band::ALL {
        add(&format!("art7 shape fill {}", b.class_name()));
    }
    let css = stylesheet_classes();
    for c in [
        "tabs", "band", "art7", "kids", "tag", "tid", "tt", "subh", "cards7", "filters", "items",
        "rows", "rowb", "nid", "ttl", "meta", "bchip", "gist", "none", "mx", "mxwrap", "mxlist",
        "pair", "pairnote", "path", "bn", "shape", "empty", "open-q",
    ] {
        assert!(used.contains(c), "地図の頁が class {c} を使わない");
    }
    let missing: Vec<&String> = used.iter().filter(|c| !css.contains(*c)).collect();
    assert!(missing.is_empty(), "stylesheet に無い class: {missing:?}");

    for key in [
        "map",
        "views",
        "bands",
        "matrix",
        "col_band",
        "col_kind",
        "col_id",
        "col_state",
        "sort",
        "st_unknown",
    ] {
        assert!(vocab().term(key).is_some(), "鍵 {key} が vocab に無い");
    }
}
