//! 決定の頁の取り消しの歯（接頭辞 rvk_・設計ノート surface-wave12d 行 e-revoke・要件 FR7）。
//! 近傍の電文と問いの 1 本の引きの電文は歯の中で組む。DOM の歯は src/project/node.rs の字 `mod dom {` の後の字を見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::graph::{AroundDoc, AroundRow, EdgeType, Fold, GraphNode, NodeKind};
use tsuzuri_contract::ledger::{BeadId, LedgerItem, LedgerRow};
use tsuzuri_contract::surface::{RevokeRequest, RevokeResponse, RulingId};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ask::{Outcome, outcome};
use tsuzuri_surface::project::node::{
    PATHS, Revoke, item_path, revoke_body, revoke_target, shows_revoke,
};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

const ID: &str = "rv.1:20260928T0441Z-1";

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead id")
}

fn rid(id: &str) -> RulingId {
    RulingId::new(id).expect("裁定の id")
}

fn row(id: &str, kind: NodeKind, col: i8, via: Option<&str>) -> AroundRow {
    AroundRow {
        node: GraphNode {
            id: id.to_string(),
            kind,
            file: None,
            digest: None,
            title: format!("{id} の題"),
            line: None,
            plain: None,
            eng: None,
            updated: None,
        },
        status: None,
        col,
        via: via.map(str::to_string),
        edge_type: via.map(|_| EdgeType::Answers),
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
        impact: n.saturating_sub(1),
        shown: n,
        total: n,
        cut_hub: 0,
        cut_cap: 0,
        hubs: vec![],
        unread: vec![],
    }
}

fn target() -> Revoke {
    Revoke {
        question: bead("rv.1"),
        ruling: rid(ID),
    }
}

#[test]
fn rvk_target_of_center() {
    let with_question = doc_of(
        ID,
        vec![
            row(ID, NodeKind::Ruling, 0, None),
            row("rv.1", NodeKind::Question, 1, Some(ID)),
        ],
    );
    assert_eq!(revoke_target(&with_question), Some(target()));
    let alone = doc_of(ID, vec![row(ID, NodeKind::Ruling, 0, None)]);
    assert_eq!(revoke_target(&alone), Some(target()));
    for kind in [NodeKind::Question, NodeKind::Policy, NodeKind::Receipt] {
        let doc = doc_of(ID, vec![row(ID, kind, 0, None)]);
        assert_eq!(revoke_target(&doc), None, "{kind:?}");
    }
    for id in ["user 2026-09-28T00:40Z", "rv.1"] {
        let doc = doc_of(id, vec![row(id, NodeKind::Ruling, 0, None)]);
        assert_eq!(revoke_target(&doc), None, "{id}");
    }
    let no_center = doc_of(ID, vec![row(ID, NodeKind::Ruling, 1, Some("rv.1"))]);
    assert_eq!(revoke_target(&no_center), None);
}

#[test]
fn rvk_item_path_and_body() {
    assert_eq!(item_path(&bead("rv.1")), "/api/ledger/rv.1");
    let body = revoke_body(&target(), "待つ\n理由");
    assert_eq!(
        body,
        wire::encode(&RevokeRequest {
            question: bead("rv.1"),
            ruling: rid(ID),
            verbatim: "待つ\n理由".into(),
        })
        .expect("要求の電文")
    );
    assert!(body.contains("待つ\\n理由"), "{body}");
    for reopened_only in [false, true] {
        let reply = wire::encode(&RevokeResponse {
            ruling: rid(ID),
            recorded_at: 1_790_570_460,
            reopened_only,
        })
        .expect("応答の電文");
        assert_eq!(
            outcome(Some((200, &reply))),
            Outcome::Recorded {
                ruling: rid(ID),
                at: 1_790_570_460,
            },
            "{reply}"
        );
    }
}

/// 問いの 1 本の引きの電文（notes は行を改行でつなぐ）。
fn item(status: &str, question: bool, notes: &[&str]) -> Fetched {
    let item = LedgerItem {
        row: LedgerRow {
            id: bead("rv.1"),
            kind: "task".into(),
            title: "題 rv.1".into(),
            status: status.into(),
            updated_at: 1_790_570_460,
            parent: None,
            labels: if question {
                vec!["intake:question".into()]
            } else {
                vec![]
            },
        },
        description: String::new(),
        notes: notes.join("\n"),
    };
    Fetched::Body(wire::encode(&item).expect("1 本の電文"))
}

#[test]
fn rvk_shows_only_when_revocable() {
    let a = "裁定 id = rv.1:20260928T0100Z-1・問い = rv.1・逐語 = はい";
    let b = "裁定 id = rv.1:20260928T0200Z-1・問い = rv.1・束 = batch:20260928T0200Z-1・逐語 = はい";
    let r = "裁定 id = rv.1:20260928T0300Z-1・問い = rv.1・取り消す = rv.1:20260928T0200Z-1・逐語 = 待つ";
    let c = "裁定 id = rv.1:20260928T0400Z-1・問い = rv.1・逐語 = はい";
    let b_id = rid("rv.1:20260928T0200Z-1");
    assert!(shows_revoke(&item("closed", true, &[a, b]), &b_id));
    assert!(
        shows_revoke(&item("closed", true, &[a, b, r]), &b_id),
        "開き直しだけが落ちた後は同じ button で撃ち直す"
    );
    for (what, fetched) in [
        ("open", item("open", true, &[a, b])),
        ("label 無し", item("closed", false, &[a, b])),
        ("open に戻った", item("open", true, &[a, b, r])),
        ("答え直した", item("closed", true, &[a, b, r, c])),
        ("まだ読んでいない", Fetched::NotRead),
        ("読めない", Fetched::Failed),
        ("電文でない", Fetched::Body("no-item".into())),
    ] {
        assert!(!shows_revoke(&fetched, &b_id), "{what}");
    }
}

#[test]
fn rvk_vocab_word() {
    let v = vocab();
    let term = v.term("revoke").expect("鍵 revoke が語の辞書に在る");
    assert_eq!(term.label, "取り消す");
    assert!(term.note.contains("この決定を取り消す"), "{}", term.note);
    assert!(term.internal.contains("bd reopen"), "{}", term.internal);
    for text in [&term.note, &term.internal] {
        assert!(!text.contains('{') && !text.contains('}'), "{text}");
    }
    let source = tsuzuri_surface::vocab::SOURCE;
    let rephrase = source.find("\"rephrase\"").expect("欄 rephrase");
    assert!(
        source[rephrase..].contains("\"revoke\": {"),
        "鍵 revoke は欄 rephrase に在る"
    );
    assert_eq!(v.label("own_words"), "あなたの言葉（原文のまま）");
    assert_eq!(v.label("answer_here"), "質問の頁で答える");
    assert_eq!(v.label("k:裁定"), "あなたの決定");
}

#[test]
fn rvk_dom_wiring_text() {
    let text = read("src/project/node.rs");
    assert!(!text.contains("\"/api/"), "node.rs に /api/ の字");
    assert!(PATHS.is_empty());
    assert!(
        text.lines()
            .any(|l| l.trim() == "pub const PATHS: &[&str] = &[];"),
        "PATHS の宣言の行が空の配列でない"
    );
    let at = text.find("mod dom {").expect("mod dom");
    let dom = &text[at..];
    for word in [
        "revoke_target(&doc)",
        "item_path(&t.question)",
        "crate::net::read_path(",
        "shows_revoke(",
        "<textarea",
        "label(\"own_words\")",
        "label(\"revoke\")",
        "data-term=\"revoke\"",
        "can_send(",
        "revoke_body(",
        "crate::net::post(REVOKE_PATH",
        "outcome(",
        "crate::net::reload_all()",
    ] {
        assert!(dom.contains(word), "mod dom に {word} が無い");
    }
    let answer = dom.find("{answer}").expect("頭の {answer}");
    assert!(
        dom[answer..]
            .trim_start_matches("{answer}")
            .trim_start()
            .starts_with("{revoke}"),
        "頭の {{answer}} の後に {{revoke}} が無い"
    );
    assert!(!dom.contains("KeyAction"), "鍵で送る判定を持つ");
}

/// filter の語（contracts の verify の filter の語を畳んだ語と、後の行 g-gz の接頭辞）。
const FILTER: [&str; 129] = [
    "aaround_",
    "accept_",
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
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "ecache_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gfresh_",
    "ghb_",
    "glabel_",
    "gnav_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
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
    "lspark_",
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
    "pmore_",
    "pquest_",
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
];

#[test]
fn rvk_own_names_clean() {
    let text = read("tests/rvknode.rs");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            w[1].trim()
                .strip_prefix("fn ")
                .and_then(|r| r.split('(').next())
                .unwrap_or_else(|| panic!("test の属性の後の行が fn でない: {}", w[1]))
        })
        .collect();
    assert!(names.len() >= 6, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("rvk_")
            .unwrap_or_else(|| panic!("rvk_ で始まらない: {name}"));
        for word in FILTER {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
