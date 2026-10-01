//! 行 g-graph-fold の歯（面）: 組の箱の card・語の辞書の 3 つの鍵・graph.rs の mod の行と外の依存・歯の名。
//! 開いた箱の列と口の path・組の箱と開き閉じの印の SVG・狭い幅の一覧の組の行・開けなかった行・DOM の配線の字の歯は
//! 行 m-map-graph で消した。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::graph::{BoxFold, GraphNode, GraphView, NodeKind, ViewNode};
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::band::{band_of, kind_key};
use tsuzuri_surface::widgets::nodecard::{NO_GIST, card_for, group_card, view_cards};
use tsuzuri_surface::vocab::{label, vocab};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 例のグラフの眺め（fixture の 8 節点・欄 group と fold は無いので偽と Leaf）。
fn fixture() -> GraphView {
    wire::decode(&read("../../tests/fixtures/surface/graph-view.json"))
        .expect("graph-view.json が眺めの電文として読める")
}

fn boxed(
    id: &str,
    kind: NodeKind,
    title: &str,
    (group, fold, kids): (bool, BoxFold, u32),
) -> ViewNode {
    ViewNode {
        node: GraphNode {
            id: id.to_string(),
            kind,
            file: None,
            digest: None,
            title: title.to_string(),
            line: None,
            plain: None,
            eng: None,
            updated: None,
        },
        status: (kind == NodeKind::Epic).then(|| "open".to_string()),
        rank: 0,
        kids,
        degree: 0,
        group,
        fold,
    }
}

fn rule_box() -> ViewNode {
    boxed("~rule", NodeKind::Rule, "規則行", (true, BoxFold::Folded, 27))
}

fn art_box() -> ViewNode {
    boxed("~art:P", NodeKind::Article, "条 P", (true, BoxFold::Open, 3))
}

fn epic_box() -> ViewNode {
    boxed("t3-hub.52", NodeKind::Epic, "面の epic", (false, BoxFold::Folded, 12))
}

/// fixture の 8 節点に組の箱 2 つと畳んだ epic を足した眺め。
fn mixed() -> GraphView {
    let mut v = fixture();
    v.nodes.extend([rule_box(), art_box(), epic_box()]);
    v
}

/// (4) 組の箱の card と view_cards の使い分け。
#[test]
fn btuck_group_card() {
    let c = group_card(&rule_box());
    assert_eq!(c.title, "規則行");
    assert_eq!(
        c.kind,
        format!(
            "{}・{}・{}",
            label(kind_key(NodeKind::Rule)),
            band_of(NodeKind::Rule).name(),
            label("gf_group")
        )
    );
    assert_eq!(c.value, format!("{} 27", label("children")));
    assert_eq!(c.src, label("gf_unfold"));
    assert!(c.more.is_empty());
    for (_, row) in c.rows() {
        assert!(!row.contains("~rule"), "{row}");
        assert!(!row.contains(NO_GIST), "{row}");
    }
    art_card_and_views();
}

/// 開いた組の箱の card と、view_cards の組の箱と組でない箱の使い分け。
fn art_card_and_views() {
    let c = group_card(&art_box());
    assert_eq!(c.title, "条 P");
    assert_eq!(
        c.kind,
        format!(
            "{}・{}・{}",
            label(kind_key(NodeKind::Article)),
            band_of(NodeKind::Article).name(),
            label("gf_group")
        )
    );
    assert_eq!(c.value, format!("{} 3", label("children")));
    assert_eq!(c.src, label("gf_fold"));
    assert!(c.more.is_empty());

    let v = mixed();
    let cards = view_cards(&v.nodes);
    assert_eq!(cards.len(), 11);
    for n in &v.nodes {
        let want = if n.group {
            group_card(n)
        } else {
            card_for(&n.node, n.status.as_deref())
        };
        assert_eq!(cards[&n.node.id], want, "{}", n.node.id);
    }
    assert_eq!(cards["~rule"], group_card(&rule_box()));
    assert_eq!(
        cards["t3-hub.52"],
        card_for(&epic_box().node, Some("open"))
    );
}

/// (6) 語の辞書の 3 つの鍵（見出しの語・注釈の記号と 1 行目の字数）と変えない鍵（開けなかった行の鍵は行 m-map-graph で外した）。
#[test]
fn btuck_vocab_keys() {
    for (key, want) in [
        ("gf_group", "組"),
        ("gf_unfold", "1 段開く"),
        ("gf_fold", "畳み直す"),
    ] {
        let t = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert_eq!(t.label, want, "{key}");
        assert_eq!(label(key), want, "{key}");
        assert_ne!(t.label, "畳む", "{key}");
        assert!(!t.note.is_empty() && !t.internal.is_empty(), "{key}");
        for text in [&t.note, &t.internal] {
            assert!(!text.contains("{fig:"), "{key}: {text}");
            assert!(!text.contains("{band:"), "{key}: {text}");
        }
        let first = t.note.lines().next().unwrap_or_default();
        assert!(first.chars().count() <= 40, "{key} の 1 行目: {first}");
    }
    let group = &vocab().term("gf_group").expect("gf_group").note;
    assert_eq!(
        group.lines().collect::<Vec<_>>(),
        ["同じ種類の節点を 1 つの箱に畳んだもの", "→ 右下の印を押すと 1 段開く"]
    );
    assert_eq!(label("children"), "子");
    assert_eq!(label("cut"), "表示した数");
}

/// Cargo.toml の表の中の鍵の名（`名 = …` の行の名・注釈の行は除く）。
fn table_names(manifest: &str, table: &str) -> Vec<String> {
    let mut inside = false;
    let mut names = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == table;
        } else if inside
            && !line.starts_with('#')
            && let Some((k, _)) = line.split_once('=')
        {
            names.push(k.trim().to_string());
        }
    }
    names
}

/// (7) graph.rs の mod の行・fold.rs に口の字が無い・(8) 外の依存は足さない（DOM の配線の字は行 m-map-graph で消した）。
#[test]
fn btuck_dom_wiring_text() {
    let host = read("src/mapview/graph.rs");
    assert!(
        host.lines().any(|l| l.starts_with("pub mod fold;")),
        "graph.rs に pub mod fold; の行が無い"
    );
    let fold = read("src/mapview/graph/fold.rs");
    assert!(!fold.contains("\"/api/"), "fold.rs に口の字が在る");

    let manifest = read("Cargo.toml");
    assert_eq!(table_names(&manifest, "[dependencies]"), ["tsuzuri-contract"]);
    assert_eq!(
        table_names(
            &manifest,
            "[target.'cfg(target_arch = \"wasm32\")'.dependencies]"
        ),
        ["leptos", "wasm-bindgen-futures", "web-sys"]
    );
}

/// 着地済みの行と第 3 波から第 9 波の行と表示面の行の verify の filter の語と流れている行の接頭辞（116 語）。
const FILTERS: [&str; 116] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
    "kcli_",
    "qgate_",
    "nsum_",
    "hcard_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
    "fstop_",
    "nsumw_",
    "fmark_",
    "fserve_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
    "wsteady_",
    "gsum_",
    "nstall_",
    "pfold_",
    "uword_",
    "cround_",
    "cgdom_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
    "rhold_",
    "wstrip_",
    "flight_",
    "qkey_",
    "pwhole_",
    "stage_term_",
    "stage_cdp_",
    "pclosed_",
    "gtuck_",
];

/// (9) この file の歯の名はどれも btuck_ で始まり、残りの字は filter の語を含まない（8 本以上）。
#[test]
fn btuck_own_names_clean() {
    let text = read("tests/btuck.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert!(names.len() >= 4, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("btuck_")
            .unwrap_or_else(|| panic!("{name} が btuck_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
