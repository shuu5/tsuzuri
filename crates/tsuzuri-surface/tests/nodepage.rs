//! 便 g-node の歯: 節点の頁の枠（nav に出さない）・口の path と URL の query の読み書き・畳みの移り方・
//! fixture の近傍の図の配置と線の端・頭と概要・畳みの button の数・数の行・狭い幅の一覧・頁の 4 つの状態・
//! id の無い URL は口を読まない・全部の節点に id が出る・語の鍵・fixture の大きさ・足す外の依存は 0 本。

use std::path::PathBuf;

use tsuzuri_contract::graph::{
    AroundDoc, AroundRow, EdgeEnd, EdgeType, Fold, GraphNode, NodeKind, basis_end,
};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::{self, Mode, PageId};
use tsuzuri_surface::mapview::around::{
    self, COL_KEYS, ChainItem, as_view, chain, count_line, edge_path, edges, expert_line, layout,
    line_ends, svg,
};
use tsuzuri_surface::mapview::band::Band;
use tsuzuri_surface::mapview::graph::{self, LineStyle, highlight, line_style};
use tsuzuri_surface::project::nodearound::{
    self, FoldSide, PageState, buttons, fold_of, id_of, k_of, path, request, state, toggle,
    unmeasured_reason, with_fold, with_k,
};
use tsuzuri_surface::project::{NO_CONTENT, NOT_READ, node};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

const FIXTURE: &str = "../../tests/fixtures/surface/around-doc.json";

fn fixture_text() -> String {
    read(FIXTURE)
}

fn fixture() -> AroundDoc {
    wire::decode(&fixture_text()).expect("fixture が契約の型の AroundDoc として読める")
}

fn row(id: &str, kind: NodeKind, status: Option<&str>, col: i8, via: Option<&str>) -> AroundRow {
    AroundRow {
        node: GraphNode {
            id: id.to_string(),
            kind,
            file: None,
            digest: None,
            title: format!("{id} の題"),
        },
        status: status.map(str::to_string),
        col,
        via: via.map(str::to_string),
        edge_type: via.map(|_| EdgeType::Touches),
        degree: 1,
    }
}

fn doc_of(center: &str, rows: Vec<AroundRow>) -> AroundDoc {
    let n = u32::try_from(rows.len()).expect("数");
    AroundDoc {
        center: center.to_string(),
        steps: 2,
        fold: Fold::None,
        rows,
        basis: 0,
        impact: n - 1,
        shown: n,
        total: n,
        cut_hub: 0,
        cut_cap: 0,
        hubs: vec![],
        unread: vec![],
    }
}

/// 語の列 `want` が `all` の中にこの順で含まれる。
fn in_order(all: &[&str], want: &[&str]) -> bool {
    let mut rest = all.iter();
    want.iter().all(|w| rest.any(|a| a == w))
}

/// (1) 節点の頁の枠（block は node と around）・page=node で開く・nav は 4 つをこの順に含み節点の頁を含まない・
/// snapshot と同じ。
#[test]
fn nodepage_frame_and_nav() {
    let page = frame::page(PageId::Node);
    assert_eq!(page.id, PageId::Node);
    assert_eq!(page.id.id(), "node");
    assert_eq!(page.block_ids(), vec!["node", "around"]);
    let blocks: Vec<(&str, &str, &str)> = page
        .columns
        .iter()
        .flat_map(|c| c.blocks.iter().map(|b| (b.id, b.heading, b.class)))
        .collect();
    assert_eq!(
        blocks,
        vec![
            ("node", "summary_plain", "stack"),
            ("around", "around", "panel")
        ]
    );
    assert_eq!(node::BLOCK.id, "node");
    assert_eq!(nodearound::BLOCK.id, "around");
    assert_eq!(PageId::from_query("?page=node&id=FR1"), PageId::Node);
    assert_eq!(PageId::from_query("?page=node"), PageId::Node);
    assert!(PageId::ALL.contains(&PageId::Node));
    // nav の頁の一覧と header の link は 4 つをこの順に含み、節点の頁を含まない。
    let ids: Vec<&str> = frame::pages().iter().map(|p| p.id.id()).collect();
    assert!(in_order(&ids, &["home", "ask", "map", "gaps"]), "{ids:?}");
    assert!(!ids.contains(&"node"), "{ids:?}");
    let nav_keys = frame::nav_keys();
    assert!(
        in_order(&nav_keys, &["home", "questions", "map", "gaps"]),
        "{nav_keys:?}"
    );
    assert!(!nav_keys.contains(&"nb_self"), "{nav_keys:?}");
    let keys: Vec<&str> = frame::nav_links(PageId::Node)
        .iter()
        .map(|l| l.key)
        .collect();
    assert_eq!(keys, nav_keys);
    // 節点の頁への link の字。
    assert_eq!(
        frame::node_href("FR1", Mode::Expert),
        "?page=node&id=FR1&mode=expert"
    );
    assert_eq!(
        frame::node_href("e.2:20260927T0000Z-1", Mode::Beginner),
        "?page=node&id=e.2%3A20260927T0000Z-1&mode=beginner"
    );
    assert_eq!(
        frame::page_snapshot(PageId::Node),
        read("tests/snapshots/pages/node.json"),
        "snapshot の file と違う"
    );
}

/// (2) 口の path（段数 2 と畳みなしは付けない・id は `%XX`）。
#[test]
fn nodepage_path() {
    assert_eq!(nodearound::PATH, "/api/around");
    assert_eq!(path("FR1", 2, Fold::None), "/api/around?id=FR1");
    assert_eq!(path("FR1", 3, Fold::Up), "/api/around?id=FR1&k=3&fold=up");
    assert_eq!(
        path("e.2:20260927T0000Z-1", 2, Fold::None),
        "/api/around?id=e.2%3A20260927T0000Z-1"
    );
    assert_eq!(
        path("FR1", 1, Fold::Both),
        "/api/around?id=FR1&k=1&fold=both"
    );
    assert_eq!(path("FR1", 2, Fold::Down), "/api/around?id=FR1&fold=down");
    assert_eq!(
        request("?page=node&id=FR1&k=3&fold=up&mode=expert").as_deref(),
        Some("/api/around?id=FR1&k=3&fold=up")
    );
    assert_eq!(
        request("?page=node&id=e.2%3A20260927T0000Z-1").as_deref(),
        Some("/api/around?id=e.2%3A20260927T0000Z-1")
    );
}

/// (3) URL の query の読み（知らない値と無いときは段数 2・畳みなし）と書き（2 と畳みなしは消す）。
#[test]
fn nodepage_query_read_write() {
    for (search, want) in [
        ("?k=1", 1),
        ("?k=2", 2),
        ("?k=3", 3),
        ("?k=0", 2),
        ("?k=4", 2),
        ("?k=x", 2),
        ("?k=", 2),
        ("?page=node", 2),
        ("", 2),
    ] {
        assert_eq!(k_of(search), want, "{search}");
    }
    for (search, want) in [
        ("?fold=up", Fold::Up),
        ("?fold=down", Fold::Down),
        ("?fold=both", Fold::Both),
        ("?fold=none", Fold::None),
        ("?fold=UP", Fold::None),
        ("?fold=", Fold::None),
        ("?page=node", Fold::None),
    ] {
        assert_eq!(fold_of(search), want, "{search}");
    }
    let s = "?page=node&id=FR1&mode=expert";
    assert_eq!(with_k(s, 3), "?page=node&id=FR1&mode=expert&k=3");
    assert_eq!(with_k("?page=node&k=3&id=FR1", 2), "?page=node&id=FR1");
    assert_eq!(with_k("?page=node&k=3&id=FR1", 1), "?page=node&k=1&id=FR1");
    assert_eq!(
        with_fold(s, Fold::Up),
        "?page=node&id=FR1&mode=expert&fold=up"
    );
    assert_eq!(
        with_fold("?page=node&fold=up&id=FR1", Fold::None),
        "?page=node&id=FR1"
    );
    assert_eq!(
        with_fold("?page=node&fold=up&id=FR1", Fold::Both),
        "?page=node&fold=both&id=FR1"
    );
    for k in [1, 2, 3] {
        assert_eq!(k_of(&with_k(s, k)), k);
    }
    for f in Fold::ALL {
        assert_eq!(fold_of(&with_fold(s, f)), f);
    }
}

/// (4) 畳みの button を押した後の畳み。
#[test]
fn nodepage_fold_toggle() {
    assert_eq!(toggle(Fold::None, FoldSide::Basis), Fold::Up);
    assert_eq!(toggle(Fold::Up, FoldSide::Impact), Fold::Both);
    assert_eq!(toggle(Fold::Both, FoldSide::Basis), Fold::Down);
    assert_eq!(toggle(Fold::Down, FoldSide::Impact), Fold::None);
    assert_eq!(toggle(Fold::None, FoldSide::Impact), Fold::Down);
    assert_eq!(toggle(Fold::Up, FoldSide::Basis), Fold::None);
    for f in Fold::ALL {
        for side in FoldSide::ALL {
            assert_eq!(toggle(toggle(f, side), side), f, "{f:?} {side:?}");
        }
    }
}

/// (5) fixture の近傍の図の配置が節の表と一致する。
#[test]
fn nodepage_fixture_layout() {
    let doc = fixture();
    let lay = layout(&doc);
    assert_eq!(lay.cols, vec![-1, 0, 1, 2]);
    assert_eq!(
        (lay.col_w, lay.box_w, lay.chars, lay.width, lay.height),
        (275, 255, 19, 1228, 422)
    );
    let bands: Vec<(Band, u32, u32)> = lay
        .bands
        .iter()
        .map(|b| (b.band, b.top, b.height))
        .collect();
    assert_eq!(
        bands,
        vec![
            (Band::Constitution, 28, 60),
            (Band::Adr, 88, 60),
            (Band::Srs, 148, 60),
            (Band::Beads, 208, 210),
        ]
    );
    let lanes: Vec<(NodeKind, u32, u32)> = lay
        .band(Band::Beads)
        .expect("beads の帯")
        .lanes
        .iter()
        .map(|l| (l.kind, l.top, l.height))
        .collect();
    assert_eq!(
        lanes,
        vec![
            (NodeKind::Epic, 208, 18),
            (NodeKind::Task, 226, 18),
            (NodeKind::Memo, 244, 18),
            (NodeKind::Question, 262, 60),
            (NodeKind::Ruling, 322, 60),
            (NodeKind::Receipt, 382, 18),
            (NodeKind::Policy, 400, 18),
        ]
    );
    assert_eq!(lay.lane(NodeKind::Question).map(|l| l.count), Some(1));
    let boxes: Vec<(&str, u32, u32)> =
        ["P-1", "ADR-1", "FR1", "AC1", "e.2", "e.2:20260927T0000Z-1"]
            .into_iter()
            .map(|id| {
                let p = lay.pos(id).unwrap_or_else(|| panic!("{id} の箱"));
                (id, p.x, p.y)
            })
            .collect();
    assert_eq!(
        boxes,
        vec![
            ("P-1", 138, 34),
            ("ADR-1", 138, 94),
            ("FR1", 413, 154),
            ("AC1", 688, 154),
            ("e.2", 688, 268),
            ("e.2:20260927T0000Z-1", 963, 328),
        ]
    );
    assert_eq!(lay.boxes.len(), 6);

    // 列の幅は 168 から 300 に収める（7 列は 168・1 列は 300）。
    let mut all = doc_of("c", vec![row("c", NodeKind::Req, None, 0, None)]);
    let one = layout(&all);
    assert_eq!(
        (one.cols.clone(), one.col_w, one.width),
        (vec![0], 300, 428)
    );
    for c in [-3_i8, -2, -1, 1, 2, 3] {
        all.rows
            .push(row(&format!("n{c}"), NodeKind::Req, None, c, Some("c")));
    }
    let seven = layout(&all);
    assert_eq!(seven.cols, vec![-3, -2, -1, 0, 1, 2, 3]);
    assert_eq!((seven.col_w, seven.box_w, seven.chars), (168, 148, 10));
    assert_eq!(seven.width, 128 + 7 * 168);
    // 同じ升の節点は rows の順に 48 ずつ下へ・beads でない帯の高さは節点の数に 48 を掛けて 12 を足す。
    assert_eq!(seven.band(Band::Srs).map(|b| b.height), Some(60));
    let mut two = doc_of(
        "c",
        vec![
            row("c", NodeKind::Req, None, 0, None),
            row("z", NodeKind::Ac, None, 1, Some("c")),
            row("a", NodeKind::Ac, None, 1, Some("c")),
        ],
    );
    let l2 = layout(&two);
    assert_eq!(l2.band(Band::Srs).map(|b| b.height), Some(108));
    assert_eq!(l2.pos("z").map(|p| p.y), Some(34));
    assert_eq!(l2.pos("a").map(|p| p.y), Some(82));
    two.rows.swap(1, 2);
    assert_eq!(layout(&two).pos("a").map(|p| p.y), Some(34));
    for (key, want) in COL_KEYS.iter().zip([
        "nb_up3", "nb_up2", "nb_up", "nb_self", "nb_down", "nb_down2", "nb_down3",
    ]) {
        assert_eq!(*key, want);
    }
    assert_eq!(around::col_key(-2), Some("nb_up2"));
    assert_eq!(around::col_key(4), None);
}

/// (6) 線は 1 つ前の節点の箱から着いた節点の箱へ・例の FR1 から P-1 の線の道の字。
#[test]
fn nodepage_lines_from_via() {
    let doc = fixture();
    let lay = layout(&doc);
    assert_eq!(
        line_ends(&lay, "FR1", "P-1"),
        Some(((413.0, 174.0), (393.0, 54.0)))
    );
    let p1 = doc.rows.iter().find(|r| r.node.id == "P-1").expect("P-1");
    assert_eq!(
        edge_path(&lay, p1).as_deref(),
        Some("M413 174 C 403 174, 403 54, 393 54")
    );
    // 右に在れば 1 つ前の箱の右の辺から着いた箱の左の辺へ。
    assert_eq!(
        line_ends(&lay, "FR1", "AC1"),
        Some(((668.0, 174.0), (688.0, 174.0)))
    );
    assert_eq!(
        line_ends(&lay, "e.2", "e.2:20260927T0000Z-1"),
        Some(((943.0, 288.0), (963.0, 348.0)))
    );
    // 同じ列なら横の中点どうし。
    assert_eq!(
        line_ends(&lay, "AC1", "e.2"),
        Some(((815.5, 174.0), (815.5, 288.0)))
    );
    assert_eq!(line_ends(&lay, "FR1", "nope"), None);
    let center = doc.rows.iter().find(|r| r.col == 0).expect("中心");
    assert_eq!(edge_path(&lay, center), None);

    // 線の形はグラフの module の表・根拠の側の端（to）は中心に近いほう。
    let es = edges(&doc);
    let pairs: Vec<(&str, &str, EdgeType)> = es
        .iter()
        .map(|e| (e.from.as_str(), e.to.as_str(), e.edge_type))
        .collect();
    assert_eq!(
        pairs,
        vec![
            ("FR1", "P-1", EdgeType::Basis),
            ("FR1", "ADR-1", EdgeType::Adrs),
            ("AC1", "FR1", EdgeType::Verifies),
            ("e.2", "FR1", EdgeType::Touches),
            ("e.2:20260927T0000Z-1", "e.2", EdgeType::Answers),
        ]
    );
    assert_eq!(line_style(EdgeType::Basis), LineStyle::Solid);
    let picture = svg(&doc, &lay);
    assert!(picture.contains(r#"d="M413 174 C 403 174, 403 54, 393 54""#));
    assert!(picture.contains(r#"<g class="e" data-i="0" data-u="P-1" data-d="FR1">"#));
    assert!(picture.contains(r#"viewBox="0 0 1228 422""#));
    // 中心の箱は太い縁（3）・open の問いは止まりの色の縁。
    assert!(picture.contains(
        r#"<rect class="hit" x="413" y="154" width="255" height="40" rx="4" fill="var(--panel)" stroke="var(--ink)" stroke-width="3"/>"#
    ));
    assert!(picture.contains(r#"x="688" y="268" width="255" height="40" rx="4" fill="var(--panel)" stroke="var(--s-stop)" stroke-width="2""#));
    // 列の見出しと帯の名は語の辞書の語。
    assert!(picture.contains(&vocab().label("nb_self")));
    assert!(picture.contains(&vocab().label("nb_down2")));

    // 光らせ方はグラフの module の関数（中心 FR1 から根拠の側 2・影響の側 3）。
    let gv = as_view(&doc);
    let h = highlight(&gv.edges, &graph::degrees(&gv), "FR1");
    assert_eq!(h.basis(), vec!["P-1", "ADR-1"]);
    assert_eq!(h.impact(), vec!["AC1", "e.2", "e.2:20260927T0000Z-1"]);
    assert_eq!(h.lit, vec![0, 1, 2, 3, 4]);
    // 根拠の側の端は契約の型の決め方と同じ（to が根拠の側）。
    assert_eq!(
        basis_end(EdgeType::Basis, NodeKind::Req, NodeKind::Article),
        EdgeEnd::To
    );
    let lg = graph::legend(&gv);
    assert_eq!(
        lg.bands,
        vec![Band::Constitution, Band::Adr, Band::Srs, Band::Beads]
    );
    assert_eq!(
        lg.types,
        vec![
            EdgeType::Basis,
            EdgeType::Adrs,
            EdgeType::Verifies,
            EdgeType::Answers,
            EdgeType::Touches
        ]
    );
}

/// (7) 頭（種類の語の鍵・帯・id・題 36 字・状態・出所）と、open の問いの中心だけが質問の頁への link を持つ。
#[test]
fn nodepage_head() {
    let doc = fixture();
    let h = node::head(&doc).expect("頭");
    assert_eq!(h.kind_key, "k:要件");
    assert_eq!(h.band, Band::Srs);
    assert_eq!(h.id, "FR1");
    assert_eq!(h.title, "project board を 1 枚の頁で見る");
    assert_eq!(h.state, None);
    assert_eq!(h.src, "srs.yaml");
    assert!(!h.answer);
    assert!(!h.alert);
    assert_eq!(h.shape, "shape band-srs big fill");
    assert!(vocab().term(h.kind_key).is_some());

    // open の問いが中心: 状態の字・出所なし・質問の頁への link。
    let q = doc_of(
        "e.2",
        vec![
            row("e.2", NodeKind::Question, Some("open"), 0, None),
            row("x", NodeKind::Req, None, -1, Some("e.2")),
        ],
    );
    let h = node::head(&q).expect("頭");
    assert_eq!(h.kind_key, "k:問い");
    assert_eq!(h.band, Band::Beads);
    assert_eq!(h.state.as_deref(), Some("open"));
    assert_eq!(h.src, node::NO_SRC);
    assert_eq!(node::NO_SRC, "出所なし");
    assert!(h.answer && h.alert);
    assert_eq!(h.shape, "shape band-beads big");
    assert_eq!(
        node::answer_href("e.2", Mode::Expert),
        "?page=ask&id=e.2&mode=expert"
    );
    // 閉じた問い・open の task は link を持たない。
    for (kind, status) in [
        (NodeKind::Question, "closed"),
        (NodeKind::Task, "open"),
        (NodeKind::Question, "in_progress"),
    ] {
        let d = doc_of("t", vec![row("t", kind, Some(status), 0, None)]);
        let h = node::head(&d).expect("頭");
        assert!(!h.answer, "{kind:?} {status}");
        assert_eq!(h.state.as_deref(), Some(status));
    }
    // 走行は段の字・設計文書の節点は状態を出さない。
    let run = doc_of("r", vec![row("r", NodeKind::Run, Some("Landed"), 0, None)]);
    assert_eq!(
        node::head(&run).and_then(|h| h.state).as_deref(),
        Some("Landed")
    );
    let req = doc_of("q", vec![row("q", NodeKind::Req, Some("x"), 0, None)]);
    assert_eq!(node::head(&req).and_then(|h| h.state), None);
    // 題は 36 字で切る。
    let mut long = doc_of("L", vec![row("L", NodeKind::Req, None, 0, None)]);
    long.rows[0].node.title = "字".repeat(50);
    assert_eq!(node::head(&long).map(|h| h.title.chars().count()), Some(36));
    // 中心の行が無ければ頭は無い。
    let none = doc_of("c", vec![row("x", NodeKind::Req, None, 1, Some("c"))]);
    assert_eq!(node::head(&none), None);
}

/// (8) 概要の 2 つの箱は要約なしの字と class none。
#[test]
fn nodepage_summary() {
    let doc = fixture();
    let c = node::center(&doc).expect("中心");
    assert_eq!(c.node.id, "FR1");
    let boxes = node::summary(c);
    let keys: Vec<&str> = boxes.iter().map(|b| b.key).collect();
    assert_eq!(keys, vec!["summary_plain", "summary_eng"]);
    for b in boxes {
        assert_eq!(b.text, node::NO_SUMMARY);
        assert_eq!(b.class, "sumbox none");
        assert!(b.class.split_whitespace().any(|c| c == "none"));
    }
    assert_eq!(node::NO_SUMMARY, "要約なし");
}

/// (9) 畳みの button の数は電文の basis と impact・畳んだ側の数も消えない。
#[test]
fn nodepage_fold_buttons() {
    let mut doc = fixture();
    let [up, down] = buttons(&doc);
    assert_eq!(
        (up.key, up.count, up.folded, up.glyph),
        ("nb_up", 2, false, "▾")
    );
    assert_eq!(
        (down.key, down.count, down.folded, down.glyph),
        ("nb_down", 3, false, "▾")
    );
    for (fold, want) in [
        (Fold::Up, [true, false]),
        (Fold::Down, [false, true]),
        (Fold::Both, [true, true]),
    ] {
        doc.fold = fold;
        let bs = buttons(&doc);
        assert_eq!(bs.map(|b| b.count), [2, 3], "{fold:?}");
        assert_eq!(bs.map(|b| b.folded), want, "{fold:?}");
        for b in bs {
            assert_eq!(b.glyph, if b.folded { "▸" } else { "▾" });
        }
    }
}

/// (10) 数の行と経験者向けの 1 行。
#[test]
fn nodepage_count_line() {
    let doc = fixture();
    assert_eq!(count_line(&doc), "6 / 6");
    assert_eq!(
        expert_line(&doc),
        "shown=6 total=6 up=2 down=2 cut.hub=0 cut.cap=0"
    );
    let mut cut = fixture();
    cut.shown = 12;
    cut.total = 621;
    cut.cut_cap = 609;
    assert_eq!(count_line(&cut), "12 / 621 ・ ✂ 609");
    assert_eq!(
        expert_line(&cut),
        "shown=12 total=621 up=2 down=2 cut.hub=0 cut.cap=609"
    );
    cut.cut_hub = 3;
    assert_eq!(count_line(&cut), "12 / 621 ・ ✂ 612");
    cut.fold = Fold::Up;
    assert_eq!(
        expert_line(&cut),
        "shown=12 total=621 up=0 down=2 cut.hub=3 cut.cap=609"
    );
    cut.fold = Fold::Both;
    cut.steps = 3;
    assert!(expert_line(&cut).contains(" up=0 down=0 "));
}

fn ids(items: &[ChainItem]) -> Vec<(&str, Option<&str>, Vec<&str>)> {
    items
        .iter()
        .map(|i| {
            (
                i.id.as_str(),
                i.edge.as_deref(),
                i.nest.iter().map(|n| n.id.as_str()).collect(),
            )
        })
        .collect()
}

/// (11) 狭い幅の一覧は根拠の側と影響の側の 2 段・1 段目の下にその先を入れ子・行の無い側は出さない。
#[test]
fn nodepage_chain() {
    let doc = fixture();
    let sides = chain(&doc);
    let keys: Vec<&str> = sides.iter().map(|s| s.key).collect();
    assert_eq!(keys, vec!["nb_up", "nb_down"]);
    assert_eq!(
        ids(&sides[0].items),
        vec![
            ("P-1", Some("basis"), vec![]),
            ("ADR-1", Some("adrs"), vec![])
        ]
    );
    assert_eq!(
        ids(&sides[1].items),
        vec![
            ("AC1", Some("verifies"), vec![]),
            ("e.2", Some("touches"), vec!["e.2:20260927T0000Z-1"]),
        ]
    );
    let q = &sides[1].items[1];
    assert!(q.alert);
    assert_eq!(q.shape, "shape band-beads");
    assert_eq!(q.nest[0].edge.as_deref(), Some("answers"));
    assert_eq!(sides[0].items[0].shape, "shape band-constitution fill");

    // 影響の側の行が無ければ根拠の側だけ。
    let up_only = doc_of(
        "c",
        vec![
            row("a", NodeKind::Req, None, -2, Some("b")),
            row("b", NodeKind::Req, None, -1, Some("c")),
            row("c", NodeKind::Req, None, 0, None),
        ],
    );
    let sides = chain(&up_only);
    assert_eq!(sides.len(), 1);
    assert_eq!(sides[0].key, "nb_up");
    assert_eq!(
        ids(&sides[0].items),
        vec![("b", Some("touches"), vec!["a"])]
    );
    assert!(chain(&doc_of("c", vec![row("c", NodeKind::Req, None, 0, None)])).is_empty());
}

/// (12) 頁の 4 つの状態（404 は見つからない・読めない・まだ読んでいない・電文の型として読めない本文は測れていない）。
#[test]
fn nodepage_page_state() {
    assert_eq!(state(&Fetched::Failed, Some(404)), PageState::NotFound);
    assert_eq!(
        state(&Fetched::Body("no-node".to_string()), Some(404)),
        PageState::NotFound
    );
    assert_eq!(state(&Fetched::NotRead, None), PageState::NotRead);
    for status in [None, Some(500), Some(503)] {
        assert_eq!(
            state(&Fetched::Failed, status),
            PageState::Unread(nodearound::REASON)
        );
    }
    for text in ["{}", "no-node", "[]"] {
        assert_eq!(
            state(&Fetched::Body(text.to_string()), Some(200)),
            PageState::Unread(NO_CONTENT),
            "{text}"
        );
    }
    let ok = state(&Fetched::Body(fixture_text()), Some(200));
    assert_eq!(ok, PageState::Doc(fixture()));
    assert_eq!(unmeasured_reason(&ok), None);
    assert_eq!(unmeasured_reason(&PageState::NotFound), None);
    assert_eq!(unmeasured_reason(&PageState::NotRead), Some(NOT_READ));
    assert_eq!(
        unmeasured_reason(&PageState::Unread(nodearound::REASON)),
        Some(nodearound::REASON)
    );
    assert!(!nodearound::REASON.trim().is_empty());
    assert!(!nodearound::REASON.contains('\n'));
    assert!(vocab().term("not_found").is_some());
}

/// (13) URL に id が無いか空なら口を読まない（口の path が無い）。
#[test]
fn nodepage_no_id_reads_nothing() {
    for search in ["", "?page=node", "?page=node&id=", "?page=node&id&k=3"] {
        assert_eq!(id_of(search), None, "{search}");
        assert_eq!(request(search), None, "{search}");
    }
    assert_eq!(id_of("?page=node&id=e.2%3Ax").as_deref(), Some("e.2:x"));
}

/// (14) 図と一覧の全部の節点に id が出る。
#[test]
fn nodepage_every_node_shows_id() {
    let doc = fixture();
    let lay = layout(&doc);
    let picture = svg(&doc, &lay);
    for r in &doc.rows {
        assert!(
            picture.contains(&format!(r#"data-key="{}""#, r.node.id)),
            "{} の箱",
            r.node.id
        );
        assert!(
            picture.contains(&format!(">{}</text>", r.node.id)),
            "{} の id の字",
            r.node.id
        );
    }
    let mut listed: Vec<String> = chain(&doc)
        .iter()
        .flat_map(|s| &s.items)
        .flat_map(|i| std::iter::once(i.id.clone()).chain(i.nest.iter().map(|n| n.id.clone())))
        .collect();
    listed.push(node::head(&doc).expect("頭").id);
    listed.sort();
    let mut want: Vec<String> = doc.rows.iter().map(|r| r.node.id.clone()).collect();
    want.sort();
    assert_eq!(listed, want);
    assert!(listed.iter().all(|id| !id.is_empty()));
}

/// 語の鍵は語の辞書に在り、fixture は 5000 byte 以下・足す外の依存は 0 本。
#[test]
fn nodepage_keys_fixture_and_deps() {
    for key in [
        "not_found",
        "around",
        "src",
        "answer_here",
        "summary_plain",
        "summary_eng",
        "nb_up",
        "nb_down",
        "nb_self",
        "nb_up2",
        "nb_up3",
        "nb_down2",
        "nb_down3",
        "nb_k",
        "cut",
        "st_unknown",
        "nb_self",
        "open_node",
        "pinned",
        "unpin",
    ] {
        assert!(vocab().term(key).is_some(), "鍵 {key} が vocab に無い");
    }
    assert!(fixture_text().len() <= 5000, "{}", fixture_text().len());
    let doc = fixture();
    assert_eq!(
        (doc.center.as_str(), doc.steps, doc.fold, doc.rows.len()),
        ("FR1", 2, Fold::None, 6)
    );
    assert_eq!((doc.basis, doc.impact, doc.shown, doc.total), (2, 3, 6, 6));
    assert!(doc.hubs.is_empty() && doc.unread.is_empty());

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
