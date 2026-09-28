//! 行 g-policy-face の歯: 全体への指示の block は server の書きの断り（no-root・ledger-create・ledger-append・
//! policy-id-shape・ledger-close）を持ち主の語の 1 行にし、読めない本文はほかの断りと同じ字にする・
//! 契約の方針の id の形は問いの形だけにする・自分の歯の名は verify の filter の語を含まない。

use std::path::PathBuf;

use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_surface::project::{ask, batch, policy};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 着地済みの行と並行の起草の行と計画の後の行の verify の filter の語（この行の接頭辞は並べない）。
const FILTER_WORDS: [&str; 151] = [
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
    "dnkind_",
    "dnrow_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "fprem_",
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
    "pgz_",
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
];

fn bead(s: &str) -> BeadId {
    BeadId::new(s).expect("bead の id")
}

fn ruling(s: &str) -> RulingId {
    RulingId::new(s).expect("記帳 id")
}

/// (1) 書きの断りは持ち主の語の 1 行（何が起きたかと送り直してよいか）になる。
#[test]
fn gpface_refusal_lines() {
    assert_eq!(policy::NO_ROOT, "指示を残す台帳の根が無い（何も書いていない）");
    assert_eq!(policy::NOT_CREATED, "指示を台帳に書けたかが分からない（書きが落ちた）");
    assert_eq!(
        policy::HALF_WRITTEN,
        "指示は記録できていない（送り直してよい）・途中で残った質問"
    );
    assert_eq!(policy::NOT_CLOSED, "質問を閉じる書きだけ落ちた（送り直さない）");

    for body in ["no-root", " no-root\n"] {
        let o = policy::outcome(Some((503, body)));
        assert_eq!(o, policy::Outcome::NoRoot, "{body:?}");
        assert_eq!(o.line(), policy::NO_ROOT);
    }
    let created = policy::outcome(Some((502, "ledger-create")));
    assert_eq!(created, policy::Outcome::NotCreated);
    assert_eq!(created.line(), policy::NOT_CREATED);

    let mut half = Vec::new();
    for (status, body) in [(502, "ledger-append fx-p.7"), (500, "policy-id-shape fx-p.7")] {
        let o = policy::outcome(Some((status, body)));
        assert_eq!(
            o,
            policy::Outcome::HalfWritten {
                question: bead("fx-p.7")
            },
            "{status} {body}"
        );
        assert_eq!(o.line(), format!("{} fx-p.7", policy::HALF_WRITTEN));
        half.push(o);
    }

    let closed = policy::outcome(Some((502, "ledger-close fx-p.7:20260928T0841Z-1")));
    assert_eq!(
        closed,
        policy::Outcome::NotClosed {
            policy: ruling("fx-p.7:20260928T0841Z-1")
        }
    );
    assert_eq!(
        closed.line(),
        format!("記録した fx-p.7:20260928T0841Z-1・{}", policy::NOT_CLOSED)
    );
    assert!(closed.line().starts_with(ask::RECORDED));
    assert!(!closed.keeps_text(), "記録した方針は送り直させない");

    let root = policy::outcome(Some((503, "no-root")));
    for o in [&root, &created, &half[0], &half[1]] {
        assert!(o.keeps_text(), "{o:?}");
    }
    for o in [&root, &created, &half[0], &half[1], &closed] {
        assert!(!o.reloads(), "{o:?}");
        assert!(!o.line().starts_with(ask::REFUSED), "{o:?}");
    }
}

/// (2) 読めない本文と今までの断りは、ほかの断りと同じ Refused の字になる。
#[test]
fn gpface_odd_bodies_fall_back() {
    for (status, body) in [
        (503, "no-policy-memo"),
        (503, "no-root fx-p.7"),
        (500, "no-root"),
        (502, "ledger-create fx-p.7"),
        (502, "ledger-append"),
        (502, "ledger-append fx-p.7 fx-p.8"),
        (500, "policy-id-shape -x"),
        (502, "policy-id-shape fx-p.7"),
        (502, "ledger-close"),
        (502, "ledger-close fx p"),
        (503, "ledger-unknown"),
    ] {
        let o = policy::outcome(Some((status, body)));
        assert_eq!(
            o,
            policy::Outcome::Refused(batch::refused_text(status, body)),
            "{status} {body}"
        );
        assert!(o.keeps_text(), "{status} {body}");
        assert!(!o.reloads(), "{status} {body}");
        assert!(o.line().starts_with(ask::REFUSED), "{status} {body}");
    }
}

/// (3) server の口は書いた本文を返し、面は今までの断りの写しを持たない。
#[test]
fn gpface_bodies_as_written() {
    let route = read("../tsuzuri-boundary/src/server/routes/policy.rs");
    for expr in [
        "Response::text(503, \"no-root\")",
        "Response::text(502, \"ledger-create\")",
        "Response::text(500, &format!(\"policy-id-shape {q}\"))",
        "Response::text(502, &format!(\"ledger-append {q}\"))",
        "Response::text(502, &format!(\"ledger-close {id}\"))",
    ] {
        assert_eq!(route.matches(expr).count(), 1, "口の file の {expr}");
    }
    assert!(!route.contains("no-policy-memo"), "口の file に no-policy-memo");

    let src = read("src/project/policy.rs");
    for word in ["no-policy-memo", "NO_MEMO", "NoMemo", "方針の memo"] {
        assert!(!src.contains(word), "policy.rs に {word} が在る");
    }
}

/// (4) 契約の方針の id は問いの形だけで、旧い形を作る手と字を持たない。
#[test]
fn gpface_contract_words() {
    let surface = read("../tsuzuri-contract/src/surface.rs");
    assert!(!surface.contains("fn for_policy"), "surface.rs に fn for_policy");
    assert!(!surface.contains("・方針は `policy:"), "surface.rs に旧い doc の字");
    for line in [
        "方針は、方針 1 つごとに作る閉じた問いの id を使う問いの形（`<方針の問い bead id>:<同じ時刻>-1`・`for_question` で作る・行 e-policy-q）。",
        "今までの memo「方針」の notes の行の `policy:<同じ時刻>-<n>` は読むだけで、もう発行しない。",
    ] {
        assert_eq!(surface.matches(line).count(), 1, "surface.rs の {line}");
    }

    let graph = read("../tsuzuri-contract/src/graph.rs");
    assert!(
        !graph.contains("根の直下の memo「方針」の notes の定型行から導く"),
        "graph.rs に旧い doc の字"
    );
    assert_eq!(
        graph
            .matches("全体への指示（方針 1 つごとに根の直下に作る閉じた問い〔label policy-scope:〕の notes の定型行「方針 id = 」から導く・")
            .count(),
        1,
        "graph.rs の NodeKind::Policy の doc"
    );

    let snap = read("../tsuzuri-contract/tests/snapshots/surface.json");
    assert!(!snap.contains("\"policy:"), "snapshot に旧い形の方針の id");
    assert!(
        snap.contains("\"policy\": \"fx-p.1:20260926T1437Z-1\""),
        "snapshot の方針の id"
    );
}

/// (5) この file の歯は 5 本で、名は gpface_ で始まり、filter の語を部分の字として含まない。
#[test]
fn gpface_own_names_clean() {
    let text = read("tests/gpface.rs");
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
    assert_eq!(names.len(), 5, "歯の数 {names:?}");
    for name in names {
        assert!(name.starts_with("gpface_"), "{name} が gpface_ で始まらない");
        for word in FILTER_WORDS {
            assert!(!name.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
