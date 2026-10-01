//! 行 g-graph-label の歯（面）: 組の箱の題を id と種類から組み、組の箱と塊の箱と一覧の組の行と組の箱の card と
//! 開けなかった行で ~ の id を字にしない・組でない箱の字は変えない・歯の名。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::graph::{BoxFold, GraphNode, GraphView, NodeKind, ViewNode};
use tsuzuri_contract::wire;
use tsuzuri_surface::mapview::band::{band_of, kind_key};
use tsuzuri_surface::mapview::graph::fold::{GROUP_TITLE_CHARS, plain_title, refused_line};
use tsuzuri_surface::mapview::graph::{Pos, chain, node_svg};
use tsuzuri_surface::widgets::nodecard::{card_for, group_card, view_cards};
use tsuzuri_surface::vocab::label;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 例のグラフの眺め（fixture の 8 節点・どれも組でない）。
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

/// 電文と同じく題を id と同じ字にした組の箱。
fn group(id: &str, kind: NodeKind, kids: u32) -> ViewNode {
    boxed(id, kind, id, (true, BoxFold::Folded, kids))
}

/// 電文と同じく題を覆う先頭と末の id を字 … でつないだ塊。
fn chunk(id: &str, kind: NodeKind, first: &str, last: &str, kids: u32) -> ViewNode {
    boxed(
        id,
        kind,
        &format!("{first}…{last}"),
        (true, BoxFold::Folded, kids),
    )
}

fn epic_box() -> ViewNode {
    boxed(
        "t3-hub.52",
        NodeKind::Epic,
        "面の epic",
        (false, BoxFold::Folded, 12),
    )
}

fn art_a() -> ViewNode {
    group("~art:A", NodeKind::Article, 13)
}

fn ruling_box() -> ViewNode {
    group("~led:ruling", NodeKind::Ruling, 24)
}

fn hub_chunk() -> ViewNode {
    chunk("t3-hub.52~1-12", NodeKind::Task, "t3-hub.52.1", "t3-hub.52.12", 12)
}

const AT: Pos = Pos { x: 200, y: 40 };

/// 属性 data-key と data-fold の値を除いた字。
fn without_keys(svg: &str) -> String {
    let mut out = String::new();
    let mut rest = svg;
    loop {
        let hit = ["data-key=\"", "data-fold=\""]
            .iter()
            .filter_map(|a| rest.find(a).map(|i| (i, a.len())))
            .min();
        let Some((i, len)) = hit else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..i + len]);
        rest = &rest[i + len..];
        let end = rest.find('"').expect("属性の値の閉じの引用符");
        rest = &rest[end..];
    }
}

/// (1) 組の箱の題は id と種類から組み、組でない箱は電文の題。
#[test]
fn glabel_plain_titles() {
    for (id, kind, want) in [
        ("~art:A", NodeKind::Article, "条 A"),
        ("~rule", NodeKind::Rule, "規則行"),
        ("~adr", NodeKind::Adr, "判断の記録"),
        ("~srs:req", NodeKind::Req, "要件"),
        ("~srs:actor", NodeKind::Actor, "登場人物"),
        ("~led:ruling", NodeKind::Ruling, "裁定"),
        ("~led:task", NodeKind::Task, "契約"),
        ("~run", NodeKind::Run, "走行"),
        ("~note:surface-wave10a", NodeKind::NoteRow, "surface-wave10a の行"),
    ] {
        assert_eq!(plain_title(&group(id, kind, 3)), want, "{id}");
    }
    let band = boxed("~b:SRS", NodeKind::Goal, "SRS", (true, BoxFold::Open, 30));
    assert_eq!(plain_title(&band), "SRS");

    for (id, kind, first, last, want) in [
        ("t3-hub.52~1-12", NodeKind::Task, "t3-hub.52.1", "t3-hub.52.12", "t3-hub.52 の 1〜12"),
        ("h~577-619", NodeKind::Task, "h.577", "h.619", "h の 577〜619"),
        ("~led:ruling~13-24", NodeKind::Ruling, "r-13", "r-24", "裁定 の 13〜24"),
        ("~art:P~1-12", NodeKind::Article, "P-1", "P-12", "条 P の 1〜12"),
    ] {
        assert_eq!(plain_title(&chunk(id, kind, first, last, 12)), want, "{id}");
    }

    assert_eq!(plain_title(&epic_box()), "面の epic");
    // 組でない箱は ~ を持つ字でも電文の題のまま。
    let leaf = boxed("~rule", NodeKind::Rule, "規則", (false, BoxFold::Leaf, 0));
    assert_eq!(plain_title(&leaf), "規則");
    // ~ で始まらない組の箱は電文の題。
    let odd = boxed("x", NodeKind::Rule, "題 x", (true, BoxFold::Folded, 2));
    assert_eq!(plain_title(&odd), "題 x");
}

/// (2) 組の箱と塊の箱の 1 行目と aria-label は題で、~ の id を字にしない・組でない箱は id の字。
#[test]
fn glabel_box_first_line() {
    assert_eq!(GROUP_TITLE_CHARS, 20);
    let group_word = label("gf_group");

    let art = node_svg(&art_a(), AT);
    let kind = label(kind_key(NodeKind::Article));
    assert!(art.contains(">条 A</text>"), "{art}");
    assert!(art.contains(&format!("aria-label=\"条 A {kind} {group_word}\"")), "{art}");
    assert!(!art.contains(">~art:A<"), "{art}");

    let hub = node_svg(&hub_chunk(), AT);
    assert!(hub.contains(">t3-hub.52 の 1〜12</text>"), "{hub}");
    assert!(!hub.contains('…'), "{hub}");

    let deep = chunk(
        "t3-hub.52.13.2~1-12",
        NodeKind::Task,
        "t3-hub.52.13.2.1",
        "t3-hub.52.13.2.12",
        12,
    );
    let deep_svg = node_svg(&deep, AT);
    let head = "t3-hub.52.13.2 の 1〜…";
    assert_eq!(head.chars().count(), GROUP_TITLE_CHARS);
    assert!(deep_svg.contains(&format!(">{head}</text>")), "{deep_svg}");

    for n in [
        art_a(),
        ruling_box(),
        hub_chunk(),
        deep,
        group("~rule", NodeKind::Rule, 27),
        group("~note:surface-wave10a", NodeKind::NoteRow, 5),
        boxed("~b:SRS", NodeKind::Goal, "SRS", (true, BoxFold::Open, 30)),
        chunk("~led:ruling~13-24", NodeKind::Ruling, "r-13", "r-24", 12),
        chunk("~art:P~1-12", NodeKind::Article, "P-1", "P-12", 12),
    ] {
        let s = node_svg(&n, AT);
        assert!(!without_keys(&s).contains('~'), "{}: {s}", n.node.id);
    }

    let epic = node_svg(&epic_box(), AT);
    assert!(epic.contains(">t3-hub.52</text>"), "{epic}");
}

/// (3) 一覧の組の行の題は箱の題・fixture の行は変わらない。
#[test]
fn glabel_chain_titles() {
    let base: Vec<_> = chain(&fixture()).into_iter().flat_map(|b| b.rows).collect();
    let mut v = fixture();
    v.nodes.extend([art_a(), ruling_box(), hub_chunk()]);
    let rows: Vec<_> = chain(&v).into_iter().flat_map(|b| b.rows).collect();
    assert_eq!(rows.len(), 11);
    let find = |id: &str| rows.iter().find(|r| r.id == id).expect(id);
    assert_eq!(find("~art:A").title, "条 A");
    assert_eq!(find("~led:ruling").title, "裁定");
    assert_eq!(find("t3-hub.52~1-12").title, "t3-hub.52 の 1〜12");
    assert_eq!(rows.iter().filter(|r| r.group).count(), 3);
    assert_eq!(base.len(), 8);
    for r in &base {
        assert_eq!(find(&r.id), r, "{}", r.id);
    }
}

/// (4) 組の箱の card の題は箱の題で、ほかの欄は今のまま・4 行は ~ を持たない。
#[test]
fn glabel_card_titles() {
    for (n, want) in [(art_a(), "条 A"), (hub_chunk(), "t3-hub.52 の 1〜12")] {
        let c = group_card(&n);
        let kind = n.node.kind;
        assert_eq!(c.title, want);
        assert_eq!(
            c.kind,
            format!(
                "{}・{}・{}",
                label(kind_key(kind)),
                band_of(kind).name(),
                label("gf_group")
            )
        );
        assert_eq!(c.value, format!("{} {}", label("children"), n.kids));
        assert_eq!(c.src, label("gf_unfold"));
        assert!(c.more.is_empty());
        let rows: Vec<_> = c.rows().into_iter().collect();
        assert_eq!(rows.len(), 4);
        for (_, row) in rows {
            assert!(!row.contains('~'), "{row}");
        }
    }

    let mut v = fixture();
    v.nodes.extend([art_a(), ruling_box(), hub_chunk(), epic_box()]);
    let cards = view_cards(&v.nodes);
    for n in &v.nodes {
        let want = if n.group {
            group_card(n)
        } else {
            card_for(&n.node, n.status.as_deref())
        };
        assert_eq!(cards[&n.node.id], want, "{}", n.node.id);
    }
    assert_eq!(cards["~art:A"].title, "条 A");
}

/// (5) 開けなかった行は、同じ id の組の箱が在れば題・無ければ id のまま。
#[test]
fn glabel_refused_titles() {
    let mut v = fixture();
    assert_eq!(refused_line(&v), None);
    v.nodes.push(ruling_box());
    assert_eq!(refused_line(&v), None);
    v.refused = vec!["~led:ruling".to_string(), "x".to_string()];
    assert_eq!(
        refused_line(&v),
        Some(format!("{} 裁定・x", label("gf_refused")))
    );
    v.refused = vec!["~rule".to_string()];
    assert_eq!(
        refused_line(&v),
        Some(format!("{} ~rule", label("gf_refused")))
    );
}

/// 着地済みの行の verify の filter の語と、後の行の接頭辞。
const FILTERS: &[&str] = &[
    "aaround_", "accept_", "account_", "acctcore_", "acctdoc_", "accthb_", "accthome_", "acctled_",
    "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_", "afocus_", "aord_",
    "apop_", "askcard_", "athr_", "batchpanel_", "board_min_", "bport_", "brand_", "btuck_",
    "cadopt_", "cgdom_", "contract_form_", "cround_", "csled_", "flight_", "fmark_", "frame_",
    "fserve_", "fstop_", "gapspage_", "ghb_", "gnav_", "graph_", "gsum_", "gtuck_", "gview_",
    "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hcard_", "hcproj_", "hcsess_", "hfig_", "hook_",
    "hruling_", "hsblock_", "hsderive_", "hspage_", "hsym_", "ilink_", "kcli_", "klink_", "lcard_",
    "ledgerblock_", "lhome_", "lspark_", "mapview_", "mkeys_", "mlink_", "mstore_", "mtree_",
    "nact_", "nbatch_", "ncard_", "nextstep_", "nodepage_", "nstall_", "nsum_", "nsumw_", "ntime_",
    "nxact_", "parts_", "pclosed_", "pfold_", "pgz_", "pipe_", "plimit_", "pmore_", "project_",
    "ptitle_", "pwhole_", "qblock_", "qgate_", "qkey_", "question_", "rhold_", "runsdoc_",
    "saxis_", "seatblock_", "seatcard_", "server_", "sesplit_", "shb_", "skeleton_", "smore_",
    "stage_", "stats_", "steady_", "sxaxis_", "ticker_", "tipx_", "topbar_", "tz_", "urpanel_",
    "uword_", "wstrip_", "iclose_",
];

/// (7) この file の歯の名はどれも glabel_ で始まり、残りの字は filter の語を含まない（6 本以上）。
#[test]
fn glabel_own_names_clean() {
    let text = read("tests/glabel.rs");
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
    assert!(names.len() >= 6, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("glabel_")
            .unwrap_or_else(|| panic!("{name} が glabel_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
