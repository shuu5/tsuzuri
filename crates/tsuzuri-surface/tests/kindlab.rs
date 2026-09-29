//! 行 g-kind-label の歯（要件 FR13 と NFR1）: 未反映の種類の見出し（語の辞書の鍵 unref: と種類の名の label）は
//! 面の ledger の module の 1 か所に在り、account board と project board が同じ語（memo・裁定・要望）で出す。
//! project board の DOM は wasm の target のときだけなので、ledger.rs の字で見る。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::{
    LedgerStats, UnreflectedCount, UnreflectedKind, UnreflectedList, UnreflectedRow,
};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::projects::unref_break;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::ledger::{
    UNREF_KIND_KEY, kind_label, kind_name, name_label, panel, unref_list,
};
use tsuzuri_surface::view::{Fetched, Screen};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の指標の組（tests/fixtures/surface/ledger-stats.json）の 1 組。
fn stats(set: &str) -> LedgerStats {
    let mut sets: BTreeMap<String, LedgerStats> =
        wire::decode(&read("../../tests/fixtures/surface/ledger-stats.json"))
            .expect("fixture の組が電文として読める");
    sets.remove(set)
        .unwrap_or_else(|| panic!("fixture の組 {set}"))
}

/// `dir` の下の .rs の file（crate の dir からの相対の path と字）を辿る。
fn rs_files(dir: &Path, out: &mut Vec<(String, String)>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("dir の項").path();
        if path.is_dir() {
            rs_files(&path, out);
        } else if path.extension().is_some_and(|x| x == "rs") {
            let rel = path
                .strip_prefix(crate_dir())
                .expect("crate の dir の下")
                .to_string_lossy()
                .replace('\\', "/");
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
            out.push((rel, text));
        }
    }
}

/// (1) 鍵の接頭と 2 つの見出しの関数は面の ledger の module の 1 か所に在り、3 種の見出しは memo・裁定・要望。
#[test]
fn kindlab_one_place() {
    assert_eq!(UNREF_KIND_KEY, "unref:");
    let got: Vec<String> = UnreflectedKind::ALL.into_iter().map(kind_label).collect();
    assert_eq!(got, vec!["memo", "裁定", "要望"]);
    for kind in UnreflectedKind::ALL {
        assert_eq!(name_label(kind_name(kind)), kind_label(kind), "{kind:?}");
    }

    let mut files = Vec::new();
    rs_files(&crate_dir().join("src"), &mut files);
    assert!(files.len() > 10, "src の file の数 {}", files.len());
    for word in [
        "pub fn kind_label(",
        "pub fn name_label(",
        "pub const UNREF_KIND_KEY",
        "\"unref:\"",
    ] {
        let found: Vec<(&str, usize)> = files
            .iter()
            .map(|(rel, text)| (rel.as_str(), text.matches(word).count()))
            .filter(|(_, n)| *n > 0)
            .collect();
        assert_eq!(found, vec![("src/project/ledger.rs", 1)], "字 {word}");
    }
}

fn row(id: &str) -> UnreflectedRow {
    UnreflectedRow {
        id: id.to_string(),
        title: format!("{id} の題"),
        created: None,
    }
}

/// 指標の段と一覧を組む今（fixture の組の時点）。
const NOW: u64 = 1_791_676_800;

fn filled(list: &UnreflectedList) -> tsuzuri_surface::project::ledger::UnrefList {
    let fetched = Fetched::Body(wire::encode(list).expect("電文"));
    match unref_list(&fetched, NOW) {
        Body::Filled(l) => l,
        other => panic!("中身が無い: {other:?}"),
    }
}

/// (2) account board の unref_break と project board の指標の段・未反映の一覧が同じ見出しの語を出す。
#[test]
fn kindlab_boards_same_words() {
    let screen = Screen::initial();

    let s = stats("filled");
    let acct = unref_break(&s);
    assert_eq!(acct.known, vec![("memo".to_string(), 3)]);
    assert_eq!(acct.unknown, vec!["裁定", "要望"]);
    let proj: Vec<String> = panel(&s, &screen, NOW)
        .unref
        .unknown
        .iter()
        .map(|k| name_label(k))
        .collect();
    assert_eq!(proj, acct.unknown);

    let e = stats("empty");
    let acct = unref_break(&e);
    assert_eq!(acct.unknown, vec!["memo", "裁定", "要望"]);
    let proj: Vec<String> = panel(&e, &screen, NOW)
        .unref
        .unknown
        .iter()
        .map(|k| name_label(k))
        .collect();
    assert_eq!(proj, acct.unknown);

    let mut all = stats("filled");
    all.unreflected_kinds = UnreflectedKind::ALL
        .into_iter()
        .map(|kind| UnreflectedCount { kind, count: 1 })
        .collect();
    all.unreflected_unknown = Vec::new();
    let heads: Vec<String> = unref_break(&all).known.into_iter().map(|(h, _)| h).collect();
    assert_eq!(heads, vec!["memo", "裁定", "要望"]);

    let three = UnreflectedList {
        memos: Reading::Known(vec![row("m-1")]),
        rulings: Reading::Known(vec![row("r-1")]),
        requests: Reading::Known(vec![row("q-1")]),
    };
    let rows: Vec<String> = filled(&three)
        .rows
        .iter()
        .map(|r| name_label(r.kind))
        .collect();
    assert_eq!(rows, heads);

    let memo_only = UnreflectedList {
        memos: Reading::Known(vec![row("m-1")]),
        rulings: Reading::Unknown,
        requests: Reading::Unknown,
    };
    let unknown: Vec<String> = filled(&memo_only)
        .unknown
        .iter()
        .map(|k| name_label(k))
        .collect();
    assert_eq!(unknown, vec!["裁定", "要望"]);
}

/// `start` から末尾までの字。
fn after<'a>(text: &'a str, start: &str) -> &'a str {
    let at = text
        .find(start)
        .unwrap_or_else(|| panic!("字 {start} が無い"));
    &text[at..]
}

/// 宣言の字から、次の 4 つの空白と閉じ波括弧だけの行までの字。
fn fn_body<'a>(text: &'a str, decl: &str) -> &'a str {
    let rest = after(text, decl);
    let end = rest.find("\n    }\n").expect("本文の終わり");
    &rest[..end]
}

/// (3) project board の DOM の 3 か所（指標の段の chip・一覧の読めない種類の行・一覧の種類の欄）は name_label の見出しを出す。
#[test]
fn kindlab_ledger_dom_words() {
    let text = read("src/project/ledger.rs");
    let dom = after(&text, "mod dom {");

    let chip = fn_body(dom, "fn unref_view(");
    let want = "<span class=\"chip num\">{name_label(k)}\" \"{state_icon(UNKNOWN)}</span>";
    assert!(chip.contains(want), "{chip}");

    let rows = fn_body(dom, "fn unref_rows(");
    let want = "<div class=\"small muted\">{name_label(k)}\" \"{state_icon(UNKNOWN)}\" \"{label(\"gap_unknown\")}</div>";
    assert!(rows.contains(want), "{rows}");

    let row = fn_body(dom, "fn unref_row_view(");
    assert!(row.contains("<span>{name_label(r.kind)}</span>"), "{row}");

    for bad in ["{*k}", "{r.kind}"] {
        assert!(!dom.contains(bad), "ledger.rs の mod dom に {bad} が在る");
    }
}

/// main 947fb4a の contracts の verify の filter の語を畳んだ語と、後の行 g-gz と同じノートの行 g-next-acct-origin の接頭辞。
const FILTERS: &[&str] = &[
    "aaround_",
    "accept_",
    "acchold_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "afocus_",
    "aord_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadl_",
    "cadopt_",
    "cadq_",
    "cdorm_",
    "cfsplit_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "denv_",
    "dnedge_",
    "dnrow_",
    "dnskip_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "g3g7_",
    "gapspage_",
    "gbnote_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hdchip_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hshist_",
    "hspage_",
    "hspk_",
    "hsym_",
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mstore_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntime_",
    "nxact_",
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
    "pipe_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stcli_",
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "pgz_",
    "nxorg_",
];

/// (5) この file の歯の名はどれも kindlab_ で始まり、残りの字は filter の語を含まない。
#[test]
fn kindlab_names_stay_apart() {
    assert_eq!(FILTERS.len(), 164);
    let text = read("tests/kindlab.rs");
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
            .strip_prefix("kindlab_")
            .unwrap_or_else(|| panic!("{name} が kindlab_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
