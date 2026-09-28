//! 行 h-acct-rest の中核の歯: 台帳の指標の未反映の種類ごとの件数（unreflected_kinds）と、
//! LedgerStats を持つ fixture の値が同じ定義（件数の和が未反映の数・種類は閉じた一覧の順）に揃うこと。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::{LedgerStats, UnreflectedCount, UnreflectedKind};
use tsuzuri_contract::wire;
use tsuzuri_core::ledger::{stats, unreflected};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture(rel: &str) -> Value {
    serde_json::from_str(&read(rel)).unwrap_or_else(|e| panic!("{rel} は JSON: {e}"))
}

fn stats_of(v: &Value, what: &str) -> LedgerStats {
    serde_json::from_value(v.clone()).unwrap_or_else(|e| panic!("{what} は LedgerStats: {e}"))
}

fn count_of(kind: UnreflectedKind, count: u32) -> UnreflectedCount {
    UnreflectedCount { kind, count }
}

/// bd の出力の形の memo の 1 行（closed なら closed_at を持つ）。
fn memo_bead(id: &str, status: &str) -> Value {
    let mut b = json!({
        "id": id,
        "title": format!("[memo] {id}"),
        "status": status,
        "issue_type": "task",
        "labels": ["intake:memo"],
        "created_at": "2026-09-20T00:00:00Z",
        "updated_at": "2026-09-20T00:00:00Z",
    });
    if status == "closed" {
        b["closed_at"] = json!("2026-09-21T00:00:00Z");
    }
    b
}

/// (1) 未反映の数は種類ごとの件数の和で、件数は読めた種類だけ（件数 0 も持つ）。
#[test]
fn harest_kinds_from_one_list() {
    let v = fixture("tests/fixtures/ledger/stats-30.json");
    let ledger = v["ledger"].to_string();
    let now = v["now"].as_u64().expect("now");
    let Reading::Known(s) = stats(&ledger, now) else {
        panic!("台帳が読めない");
    };
    assert_eq!(
        s.unreflected_kinds,
        vec![count_of(UnreflectedKind::Memo, 3)]
    );
    assert_eq!(s.unreflected, 3);
    assert_eq!(
        s.unreflected_unknown,
        vec![UnreflectedKind::Ruling, UnreflectedKind::Request]
    );
    let Reading::Known(memos) = unreflected(&ledger, now).memos else {
        panic!("memo の一覧が読めない");
    };
    assert_eq!(memos.len(), 3);

    let now = 1_790_553_600;
    let two = json!([memo_bead("fx-m.1", "open"), memo_bead("fx-m.2", "closed")]).to_string();
    let Reading::Known(s) = stats(&two, now) else {
        panic!("2 つの memo の台帳が読めない");
    };
    assert_eq!(
        s.unreflected_kinds,
        vec![count_of(UnreflectedKind::Memo, 1)]
    );
    assert_eq!(s.unreflected, 1);

    let Reading::Known(s) = stats("[]", now) else {
        panic!("空の台帳が読めない");
    };
    assert_eq!(
        s.unreflected_kinds,
        vec![count_of(UnreflectedKind::Memo, 0)]
    );
    assert_eq!(s.unreflected, 0);

    assert_eq!(stats("{", now), Reading::Unknown);
}

/// (2) LedgerStats を持つ fixture の 5 つの値は同じ定義に揃う。
#[test]
fn harest_fixtures_agree() {
    let doc: AccountDoc = wire::decode(&read("tests/fixtures/account/acct-doc.json"))
        .expect("acct-doc は AccountDoc");
    let Reading::Known(acct) = doc.projects[0].ledger.clone() else {
        panic!("proj-a の台帳が Known でない");
    };
    let ls = fixture("tests/fixtures/surface/ledger-stats.json");
    let metrics: Reading<LedgerStats> =
        wire::decode(&read("tests/fixtures/surface/metrics-body.json")).expect("metrics-body");
    let Reading::Known(known) = metrics else {
        panic!("metrics-body の known が無い");
    };
    let expected = stats_of(
        &fixture("tests/fixtures/ledger/stats-30.json")["expected"],
        "stats-30 の expected",
    );
    let cases = [
        ("acct-doc", acct, vec![count_of(UnreflectedKind::Memo, 3)]),
        (
            "filled",
            stats_of(&ls["filled"], "filled"),
            vec![count_of(UnreflectedKind::Memo, 3)],
        ),
        ("empty", stats_of(&ls["empty"], "empty"), vec![]),
        (
            "metrics-body",
            known,
            vec![count_of(UnreflectedKind::Memo, 1)],
        ),
        (
            "stats-30",
            expected,
            vec![count_of(UnreflectedKind::Memo, 3)],
        ),
    ];
    for (name, s, want) in cases {
        assert_eq!(s.unreflected_kinds, want, "{name}");
        let sum: u32 = s.unreflected_kinds.iter().map(|k| k.count).sum();
        assert_eq!(sum, s.unreflected, "{name}: 件数の和が未反映の数");
        let pos = |k: UnreflectedKind| UnreflectedKind::ALL.iter().position(|x| *x == k);
        let order: Vec<_> = s.unreflected_kinds.iter().map(|k| pos(k.kind)).collect();
        let mut sorted = order.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(order, sorted, "{name}: 閉じた一覧の順");
        for k in UnreflectedKind::ALL {
            let known = s.unreflected_kinds.iter().any(|c| c.kind == k);
            let unknown = s.unreflected_unknown.contains(&k);
            assert!(known != unknown, "{name}: {k:?} は片方にだけ在る");
        }
    }
}

/// filter の語（main の verify の filter の語を畳んだ語と、並行の行と後の行の接頭辞）。
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
    "cadopt_",
    "cdorm_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "dnrow_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gbnote_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
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
    "hspage_",
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
    "fprem_",
    "gpface_",
    "dnkind_",
];

/// (12) この file の歯の名は 3 つで harest_ で始まり、名の全体は filter の語を含まない。
#[test]
fn harest_core_names_clean() {
    assert_eq!(FILTERS.len(), 149);
    let text = read("crates/tsuzuri-core/tests/harestcore.rs");
    let lines: Vec<&str> = text.lines().collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines[i + 1..]
            .iter()
            .find(|l| l.trim_start().starts_with("fn "))
            .expect("test の属性の後に fn が在る");
        let name = decl.trim_start()["fn ".len()..]
            .split('(')
            .next()
            .unwrap_or_default()
            .to_string();
        names.push(name);
    }
    assert_eq!(names.len(), 3, "{names:?}");
    for name in names {
        assert!(
            name.starts_with("harest_"),
            "歯の名 {name} が harest_ で始まらない"
        );
        for w in FILTERS {
            assert!(!name.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
