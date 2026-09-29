//! 行 g-ledger-home の歯: HOME の台帳の一覧は閉じた bead を出さない（持ち主の裁定 t3-hub.52.31）・
//! 閉じた epic の頭は閉じていない下の項が在るときだけ残す・残る組が 0 なら NO_OPEN・件数と指標は閉じた行も数えたまま。

use std::path::PathBuf;

use tsuzuri_boundary::server::ledger as server_ledger;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerList, LedgerRow, MEMO_LABEL, QUESTION_LABEL};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ledger::{
    self, CLOSED, EMPTY, FOLDS, Group, LAYOUT, NO_OPEN, Part, Tier, listed,
};
use tsuzuri_surface::project::{Body, LEDGER_UNREAD, item};
use tsuzuri_surface::view::{Fetched, Screen};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 台帳の 1 行（題は「題」・時刻は 0・親と labels は無し）。
fn row(id: &str, kind: &str, status: &str) -> LedgerRow {
    LedgerRow {
        id: BeadId::new(id).expect("id"),
        kind: kind.to_string(),
        title: "題".to_string(),
        status: status.to_string(),
        updated_at: 0,
        parent: None,
        labels: vec![],
    }
}

/// 本物の台帳（bd）の形: 欄 kind の字 memo と question の行を issue_type task と label にする（ほかはそのまま）。
fn real(row: LedgerRow) -> LedgerRow {
    let label = match row.kind.as_str() {
        "memo" => MEMO_LABEL,
        "question" => QUESTION_LABEL,
        _ => return row,
    };
    LedgerRow {
        kind: "task".to_string(),
        labels: vec![label.to_string()],
        ..row
    }
}

fn screen_of(rows: Vec<LedgerRow>) -> Screen {
    let body = wire::encode(&LedgerList {
        rows: Reading::Known(rows),
    })
    .expect("電文");
    Screen::initial().after_read(&Fetched::Body(body), 100)
}

/// 節の台帳 LH（15 行・memo と問いは fn real で本物の台帳の形にする）。
fn lh() -> Vec<LedgerRow> {
    [
        ("ea", "epic", "open"),
        ("ea.1", "task", "open"),
        ("ea.2", "task", "closed"),
        ("ea.3", "memo", "in_progress"),
        ("eb", "epic", "in_progress"),
        ("eb.1", "task", "closed"),
        ("ec", "epic", "closed"),
        ("ec.1", "task", "closed"),
        ("ec.2", "memo", "closed"),
        ("ed", "epic", "closed"),
        ("ed.1", "task", "blocked"),
        ("ed.2", "task", "closed"),
        ("z.1", "task", "closed"),
        ("z.2", "memo", "open"),
        ("qq.1", "question", "open"),
    ]
    .into_iter()
    .map(|(id, kind, status)| real(row(id, kind, status)))
    .collect()
}

/// 節の台帳 LH2（3 行とも閉じた行）。
fn lh2() -> Vec<LedgerRow> {
    vec![
        row("ec", "epic", "closed"),
        row("ec.1", "task", "closed"),
        row("z.1", "task", "closed"),
    ]
}

/// (1) listed は状態が closed の行だけ偽。
#[test]
fn lhome_listed_hides_closed() {
    assert_eq!(CLOSED, "closed");
    assert_eq!(NO_OPEN, "閉じていない bead は無い");
    assert!(!listed(&row("a.1", "task", CLOSED)));
    for status in ["open", "in_progress", "blocked", "deferred", ""] {
        assert!(listed(&row("a.1", "task", status)), "{status:?} が出ない");
    }
}

/// (2) 節の台帳 LH の body は節の表 T の 4 つの組。
#[test]
fn lhome_groups_rules() {
    let rows = lh();
    let by = |id: &str| item(rows.iter().find(|r| r.id.as_str() == id).expect("LH の行"));
    let want = vec![
        Group {
            head: Some(by("ea")),
            children: vec![by("ea.1"), by("ea.3")],
        },
        Group {
            head: Some(by("eb")),
            children: vec![],
        },
        Group {
            head: Some(by("ed")),
            children: vec![by("ed.1")],
        },
        Group {
            head: None,
            children: vec![by("z.2")],
        },
    ];
    let screen = screen_of(rows.clone());
    assert_eq!(ledger::body(&screen), Body::Filled(want));
    // 閉じた epic ed の頭は塗った印（閉じていない ea の頭は塗らない）。
    assert!(by("ed").shape.contains("fill"));
    assert!(!by("ea").shape.contains("fill"));
}

/// (3) fixture board-8 の body は閉じた bm.3 を出さず、画面の台帳の一覧と count は今のまま。
#[test]
fn lhome_fixture_hides_closed() {
    let text = read("../../tests/fixtures/ledger/board-8.jsonl");
    let Reading::Known(items) = server_ledger::parse(&text) else {
        panic!("fixture が server の読みで Unknown");
    };
    let screen = screen_of(items.into_iter().map(|i| i.row).collect());
    let Body::Filled(groups) = ledger::body(&screen) else {
        panic!("台帳の一覧が中身を出さない");
    };
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].head.as_ref().map(|h| h.id.as_str()), Some("bm"));
    let kids: Vec<&str> = groups[0].children.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(kids, vec!["bm.1", "bm.10", "bm.2"]);

    let Reading::Known(board) = &screen.board else {
        panic!("画面の台帳が読めていない");
    };
    let bm = board
        .groups
        .iter()
        .find(|g| g.epic.as_ref().is_some_and(|e| e.id.as_str() == "bm"))
        .expect("bm の組");
    let all: Vec<&str> = bm.children.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(all, vec!["bm.1", "bm.3", "bm.10", "bm.2"]);
    assert_eq!(ledger::count(&screen), Reading::Known(5));
}

/// (4) 閉じた行だけの台帳は NO_OPEN・行の無い台帳は EMPTY・読めない台帳は測れていない・count は閉じた行も数える。
#[test]
fn lhome_empty_and_count() {
    let closed = screen_of(lh2());
    assert_eq!(ledger::body(&closed), Body::Empty(NO_OPEN));
    assert_ne!(NO_OPEN, EMPTY);
    assert!(!NO_OPEN.trim().is_empty() && !NO_OPEN.contains('\n'));
    assert_eq!(ledger::count(&closed), Reading::Known(3));
    assert_eq!(ledger::count(&screen_of(lh())), Reading::Known(14));
    assert_eq!(ledger::body(&screen_of(vec![])), Body::Empty(EMPTY));
    assert_eq!(
        ledger::body(&Screen::initial()),
        Body::Unmeasured(LEDGER_UNREAD)
    );
}

/// (5) 畳みと配置の表は今のまま・一覧の段は body の値を描き、見出しの件数は count を描く。
#[test]
fn lhome_list_draws_body() {
    assert_eq!(FOLDS, &["ledger:more", "ledger:unref"]);
    let (tier, parts) = LAYOUT[LAYOUT.len() - 1];
    assert_eq!(tier, Tier::List);
    assert_eq!(parts, &[Part::List]);

    let src = read("src/project/ledger.rs");
    let at = src.find("mod dom {").expect("mod dom の字");
    let dom = &src[at..];
    let start = dom.find("fn list_view(").expect("list_view の字");
    let rest = &dom[start..];
    let end = rest.find("\n    }\n").expect("list_view の終わり");
    let list_view = &rest[..end];
    assert!(
        list_view.contains("screen.with(|s| staged_body(s, &stages(p, crate::net::now())))"),
        "{list_view}"
    );
    assert!(
        list_view.contains("map(|g| group_view(g, &cards))"),
        "{list_view}"
    );
    assert!(dom.contains("Tier::List => list_view(screen, graph)"));
    assert!(dom.contains("screen.with(count)"));
}

/// 着地済みの行と第 3 波から第 8 波の行の verify の filter の語。
const FILTERS: &[&str] = &[
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
    "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_",
    "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_",
    "pmore_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "qgate_", "hcard_",
    "ntime_", "ptitle_", "ncard_", "hsym_", "smore_", "lcard_", "aaround_", "hruling_",
    "sxaxis_", "plimit_", "bport_", "fstop_", "nsum_", "nsumw_", "fmark_", "fserve_", "cadopt_",
    "tipx_", "hcsess_", "hcproj_", "sesplit_", "mstore_", "athr_", "qblock_", "wsteady_",
    "gsum_", "nstall_", "pfold_", "uword_", "cround_", "cgdom_", "csled_", "shb_", "ghb_",
    "nact_", "aord_", "mtree_",
];

/// (8) この file の歯の名はどれも lhome_ で始まり、残りの字は filter の語を含まない。
#[test]
fn lhome_own_names_clean() {
    assert_eq!(FILTERS.len(), 106);
    let src = read("tests/lhome.rs");
    let mut names = Vec::new();
    let mut lines = src.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let next = lines.next().expect("属性の次の行");
        let name = next
            .trim()
            .strip_prefix("fn ")
            .and_then(|s| s.split('(').next())
            .unwrap_or_else(|| panic!("属性の次が fn でない: {next}"));
        names.push(name.to_string());
    }
    assert_eq!(names.len(), 6, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("lhome_")
            .unwrap_or_else(|| panic!("{name} が lhome_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
