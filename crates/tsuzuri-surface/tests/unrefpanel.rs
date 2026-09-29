//! 行 g-unref-panel の歯（接頭辞 urpanel_・要件 FR13 と NFR2）: 台帳の block の未反映の段は口 /api/unreflected の電文を
//! 行の順と数のまま一覧にし（20 件まで・超える分は残りの数の 1 行）、分からない種類と読めない口を「測れていない」と出す。
//! DOM は host で撃てないので、ledger.rs の字と xtask の surface-build で組めることで見る。

use std::collections::BTreeSet;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::{UnreflectedList, UnreflectedRow};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ledger::{
    self, FOLDS, PATHS, UNREF_EMPTY, UNREF_MAX, UNREF_OPEN, UNREF_PATH, UNREF_REASON,
    UNREF_UNKNOWN, UnrefList, UnrefRow, more_line, unref_chip, unref_list,
};
use tsuzuri_surface::project::{Body, NO_CONTENT, NOT_READ};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 一覧を組む今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 今 NOW から年齢の秒を引いた時刻に作った 1 件。
fn row(id: &str, title: &str, age: Option<u64>) -> UnreflectedRow {
    UnreflectedRow {
        id: id.to_string(),
        title: title.to_string(),
        created: age.map(|a| NOW - a),
    }
}

fn body(list: &UnreflectedList) -> Fetched {
    Fetched::Body(wire::encode(list).expect("電文"))
}

fn filled(fetched: &Fetched) -> UnrefList {
    match unref_list(fetched, NOW) {
        Body::Filled(l) => l,
        other => panic!("中身が無い: {other:?}"),
    }
}

/// 行の比べる形（id・題・種類の名・年齢の字・次の 1 手の語の鍵）。
fn cells(r: &UnrefRow) -> (&str, &str, &str, &str, &str) {
    (&r.id, &r.title, r.kind, &r.age, r.next)
}

/// 組 A（memos が Known の 2 行・rulings と requests が Unknown）。
fn set_a() -> UnreflectedList {
    UnreflectedList {
        memos: Reading::Known(vec![
            row("t3-hub.9", "[memo] 控え", Some(604_800)),
            row("t3-hub.10", "[memo] 時刻の無い控え", None),
        ]),
        rulings: Reading::Unknown,
        requests: Reading::Unknown,
    }
}

/// 組 A を鍵 known と字 unknown で書いた電文の字。
const SET_A_TEXT: &str = r#"{
  "memos": {
    "known": [
      { "id": "t3-hub.9", "title": "[memo] 控え", "created": 1789905600 },
      { "id": "t3-hub.10", "title": "[memo] 時刻の無い控え", "created": null }
    ]
  },
  "rulings": "unknown",
  "requests": "unknown"
}"#;

/// (2) 組 A と組 C: 行は種類の順に電文の順のまま、年齢の字と次の 1 手の鍵を添える。
#[test]
fn urpanel_rows_from_wire() {
    let want_a = vec![
        ("t3-hub.9", "[memo] 控え", "memo", "7.0d", "nx_promote"),
        ("t3-hub.10", "[memo] 時刻の無い控え", "memo", "―", "nx_promote"),
    ];
    for fetched in [body(&set_a()), Fetched::Body(SET_A_TEXT.to_string())] {
        let l = filled(&fetched);
        assert_eq!(l.rows.iter().map(cells).collect::<Vec<_>>(), want_a);
        assert_eq!(l.more, None);
        assert_eq!(l.unknown, vec!["ruling", "request"]);
    }
    // 組 C（3 種が Known の 1 行ずつ）。
    let c = UnreflectedList {
        memos: Reading::Known(vec![row("m-1", "memo の 1", Some(3600))]),
        rulings: Reading::Known(vec![row("r-1", "裁定の 1", Some(950_400))]),
        requests: Reading::Known(vec![row("q-1", "要望の 1", None)]),
    };
    let l = filled(&body(&c));
    assert_eq!(
        l.rows.iter().map(cells).collect::<Vec<_>>(),
        vec![
            ("m-1", "memo の 1", "memo", "1h", "nx_promote"),
            ("r-1", "裁定の 1", "ruling", "11d", "nx_declare"),
            ("q-1", "要望の 1", "request", "―", "nx_reflect"),
        ]
    );
    assert_eq!(l.more, None);
    assert!(l.unknown.is_empty());
}

/// (2) 組 B: 行は先頭から 20 まで、超えた数は more、分からない種類は unknown。
#[test]
fn urpanel_cut_at_twenty() {
    let memos: Vec<UnreflectedRow> = (1..=25)
        .map(|i| row(&format!("m-{i}"), &format!("控え {i}"), Some(3600)))
        .collect();
    let b = UnreflectedList {
        memos: Reading::Known(memos),
        rulings: Reading::Known(vec![row("r-1", "裁定", None)]),
        requests: Reading::Unknown,
    };
    let l = filled(&body(&b));
    assert_eq!(UNREF_MAX, 20);
    assert_eq!(l.rows.len(), 20);
    let ids: Vec<String> = (1..=20).map(|i| format!("m-{i}")).collect();
    assert_eq!(l.rows.iter().map(|r| r.id.clone()).collect::<Vec<_>>(), ids);
    for r in &l.rows {
        assert_eq!((r.kind, r.age.as_str(), r.next), ("memo", "1h", "nx_promote"));
    }
    assert_eq!(l.more, Some(6));
    assert_eq!(l.unknown, vec!["request"]);
    // ちょうど 20 行なら more は無い。
    let twenty = UnreflectedList {
        memos: Reading::Known(
            (1..=20)
                .map(|i| row(&format!("m-{i}"), "控え", None))
                .collect(),
        ),
        rulings: Reading::Known(vec![]),
        requests: Reading::Known(vec![]),
    };
    let l = filled(&body(&twenty));
    assert_eq!((l.rows.len(), l.more), (20, None));
}

/// (3) 全部分からない・0 行・まだ読んでいない・読めない・電文でない本文の理由（どれも空でない 1 行）。
#[test]
fn urpanel_unknown_kinds_and_reasons() {
    let all_unknown = UnreflectedList {
        memos: Reading::Unknown,
        rulings: Reading::Unknown,
        requests: Reading::Unknown,
    };
    assert_eq!(
        unref_list(&body(&all_unknown), NOW),
        Body::Unmeasured(UNREF_UNKNOWN)
    );
    let empty = UnreflectedList {
        memos: Reading::Known(vec![]),
        rulings: Reading::Known(vec![]),
        requests: Reading::Known(vec![]),
    };
    assert_eq!(unref_list(&body(&empty), NOW), Body::Empty(UNREF_EMPTY));
    for (fetched, reason) in [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, UNREF_REASON),
        (Fetched::Body("not json".to_string()), NO_CONTENT),
        (Fetched::Body("{}".to_string()), NO_CONTENT),
    ] {
        assert_eq!(unref_list(&fetched, NOW), Body::Unmeasured(reason), "{fetched:?}");
    }
    for reason in [UNREF_UNKNOWN, UNREF_EMPTY, UNREF_REASON, NOT_READ, NO_CONTENT] {
        assert!(!reason.trim().is_empty());
        assert!(!reason.contains('\n'), "{reason}");
    }
}

/// (1) 口の path・FOLDS の鍵・段の初めの値（値の並びは歯 hsderive_ が比べる）。
#[test]
fn urpanel_consts_paths_folds() {
    assert_eq!(UNREF_PATH, "/api/unreflected");
    assert!(PATHS.contains(&UNREF_PATH));
    assert!(PATHS.contains(&ledger::PATH));
    assert!(PATHS.contains(&ledger::METRICS_PATH));
    assert!(FOLDS.contains(&"ledger:unref"));
    assert!(FOLDS.contains(&"ledger:more"));
    const { assert!(UNREF_OPEN) };
    assert_eq!(UNREF_MAX, 20);
}

/// (4) 残りの数の 1 行と、数の chip の class。
#[test]
fn urpanel_more_line_and_chip() {
    assert_eq!(more_line(6), "ほか 6 件");
    assert_eq!(unref_chip(0), "chip num unref-n z");
    assert_eq!(unref_chip(3), "chip num unref-n");
    assert_eq!(unref_chip(1), "chip num unref-n");
}

/// stylesheet の class の名（selector の `.名`）。
fn stylesheet_classes() -> BTreeSet<String> {
    let css = read("style.css");
    let chars: Vec<char> = css.chars().collect();
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

/// (6) 段が使う語の鍵は語の辞書に、class は stylesheet に在る。
#[test]
fn urpanel_keys_and_classes() {
    for key in [
        "unref",
        "u_kind",
        "u_age",
        "u_next",
        "nx_promote",
        "nx_declare",
        "nx_reflect",
        "col_title",
        "gap_unknown",
    ] {
        assert!(vocab().term(key).is_some(), "鍵 {key} が vocab に無い");
    }
    for (_, key) in ledger::UNREF_NEXT {
        assert!(vocab().term(key).is_some(), "鍵 {key} が vocab に無い");
    }
    let css = stylesheet_classes();
    for class in [
        "fold", "lep", "lchips", "ulist", "urow", "hrow", "u-id", "u-t", "u-n", "unref-n", "z",
        "mono", "chip", "num", "small", "muted",
    ] {
        assert!(css.contains(class), "class {class} が stylesheet に無い");
    }
    for count in [0, 3] {
        for class in unref_chip(count).split(' ') {
            assert!(css.contains(class), "class {class} が stylesheet に無い");
        }
    }
}

/// (5) ledger.rs の mod dom は口を読み、一覧を組み、段を記録の鍵の details にし、行の id を節点の頁への link にする。
#[test]
fn urpanel_src_fold_and_read() {
    let text = read("src/project/ledger.rs");
    let at = text.find("\nmod dom {").expect("mod dom");
    let dom = &text[at..];
    for want in [
        "read(UNREF_PATH)",
        "unref_list(",
        "fold(\"ledger:unref\"",
        "UNREF_OPEN",
        "node_href(",
        "<details class=\"fold lep\"",
        "id=\"unref\"",
        "class=\"ulist\"",
        "class=\"urow hrow\"",
        "class=\"urow\"",
        "class=\"u-id mono\"",
        "class=\"u-t\"",
        "class=\"u-n\"",
    ] {
        assert!(dom.contains(want), "mod dom に {want} が無い");
    }
    // 足した定数は pub fn view より前（mod dom は file の終わり）。
    let view_at = text.find("pub fn view()").expect("pub fn view");
    let unref_at = text.find("pub const UNREF_OPEN").expect("UNREF_OPEN");
    assert!(unref_at < view_at);
    assert!(text.find("pub fn unref_list(").expect("unref_list") < view_at);
}

/// 着地済みの行と同じ波の行の verify の filter の語。
const FILTERS: [&str; 65] = [
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
    "hook_",
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
    "hbproc_",
    "hbconf_",
    "hbroute_",
    "hbpost_",
    "hsblock_",
    "hspage_",
    "hsderive_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
];

/// (9) この file の歯の名はどれも urpanel_ で始まり、残りの字は filter の語を含まない。
#[test]
fn urpanel_own_names_clean() {
    let text = read("tests/unrefpanel.rs");
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
    assert!(names.len() >= 7, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("urpanel_")
            .unwrap_or_else(|| panic!("{name} が urpanel_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
