//! 束の書きが途中で落ちた 502 の出し方の歯（接頭辞 bhalf_・設計ノート surface-wave12e の行 e-batch-partial）:
//! 残った行を送った行の題で名指す・閉じていない行は括弧で包んだ字を足す・送る所は送った行を応答と一緒に読む。

use std::path::PathBuf;

use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::{BatchItemResult, BatchResponse, ItemOutcome, RulingId};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ask;
use tsuzuri_surface::project::batch::{self, Left, Outcome, Row};

/// 着地済みの filter の語を畳んだ語と、計画の後の行 g-gz の接頭辞（この行の歯の名はどれも部分の字として含まない）。
const FILTER_WORDS: [&str; 125] = [
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

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead の id")
}

fn rid(id: &str) -> RulingId {
    RulingId::new(id).expect("記帳 id")
}

/// 送った 3 行（題は 色の数・字の大きさ・余白）。
fn sent() -> Vec<Row> {
    ["色の数", "字の大きさ", "余白"]
        .iter()
        .enumerate()
        .map(|(i, title)| Row {
            number: i + 1,
            id: bead(&format!("fx-h.{}", i + 1)),
            title: title.to_string(),
            a1: false,
            touches: vec![],
            digest: format!("{:016x}", i + 1),
        })
        .collect()
}

fn item(q: &str, outcome: ItemOutcome) -> BatchItemResult {
    BatchItemResult {
        question: bead(q),
        outcome,
    }
}

fn reply(items: Vec<BatchItemResult>) -> String {
    wire::encode(&BatchResponse {
        batch: rid("batch:20260928T0441Z-1"),
        items,
    })
    .expect("電文")
}

/// 書いた・閉じていない・書いていないの 3 行。
fn half() -> Vec<BatchItemResult> {
    vec![
        item(
            "fx-h.1",
            ItemOutcome::Written {
                ruling: rid("fx-h.1:20260928T0441Z-1"),
            },
        ),
        item(
            "fx-h.2",
            ItemOutcome::Unclosed {
                ruling: rid("fx-h.2:20260928T0441Z-1"),
            },
        ),
        item("fx-h.3", ItemOutcome::Unwritten),
    ]
}

/// 自分の file の test の属性の付いた fn の名。
fn test_names(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .filter_map(|w| w[1].strip_prefix("fn "))
        .map(|rest| rest.split('(').next().unwrap_or(rest).to_string())
        .collect()
}

/// (5) 502 の束の応答の残った行を送った行の題で名指す（無ければ問いの id の字）。
#[test]
fn bhalf_outcome_names_left() {
    assert_eq!(batch::LEFT, "残り");
    assert_eq!(batch::UNCLOSED, "記録したが閉じていない");
    let rows = sent();
    let body = reply(half());
    let got = batch::outcome(Some((502, &body)), &rows);
    assert_eq!(
        got,
        Outcome::Partial {
            written: 1,
            left: vec![
                Left {
                    question: bead("fx-h.2"),
                    title: "字の大きさ".to_string(),
                    ruling: Some(rid("fx-h.2:20260928T0441Z-1")),
                },
                Left {
                    question: bead("fx-h.3"),
                    title: "余白".to_string(),
                    ruling: None,
                },
            ],
        }
    );
    assert_eq!(
        got.line(),
        format!(
            "{}（502） · {} 1 · {} 2 · 字の大きさ（{}）、余白",
            ask::REFUSED,
            batch::WRITTEN,
            batch::LEFT,
            batch::UNCLOSED
        )
    );
    assert_eq!(
        got.line(),
        "送れなかった（502） · 書いた行 1 · 残り 2 · 字の大きさ（記録したが閉じていない）、余白"
    );
    assert!(got.keeps_text());
    assert!(got.reloads());

    // 送った行に無い id は問いの id の字。
    let only = batch::outcome(Some((502, &body)), &rows[..1]);
    assert_eq!(
        only.line(),
        format!(
            "{}（502） · {} 1 · {} 2 · fx-h.2（{}）、fx-h.3",
            ask::REFUSED,
            batch::WRITTEN,
            batch::LEFT,
            batch::UNCLOSED
        )
    );

    // 書いたの 1 行だけの 502 は残りが空で、字は今のまま。
    let done = reply(half()[..1].to_vec());
    let written_only = batch::outcome(Some((502, &done)), &rows);
    assert_eq!(
        written_only,
        Outcome::Partial {
            written: 1,
            left: vec![],
        }
    );
    assert_eq!(
        written_only.line(),
        format!("{}（502） · {} 1", ask::REFUSED, batch::WRITTEN)
    );

    // 200 の束の応答の値は送った行に依らない。
    let ok = reply(half()[..1].to_vec());
    assert_eq!(
        batch::outcome(Some((200, &ok)), &rows),
        batch::outcome(Some((200, &ok)), &[])
    );
}

/// (7) 送る所は送る前に選んだ行を写し、応答をその写しと一緒に読む（DOM は wasm の target だけなので字で見る）。
#[test]
fn bhalf_submit_passes_sent() {
    let text = read("src/project/batch.rs");
    let at = text.find("fn submit(").expect("fn submit");
    let tail = &text[at..];
    assert_eq!(tail.matches("outcome(").count(), 1, "{tail}");
    assert!(!tail.contains("&[]"), "送った行を渡さない");
    assert!(!tail.contains("Vec::new()"), "送った行を渡さない");
    assert!(tail.contains("request_body("));
}

/// (9) この file の歯の名はどれも bhalf_ で始まり、並べた filter の語を部分の字として含まない。
#[test]
fn bhalf_view_names_clean() {
    let names = test_names(&read("tests/bhalfpanel.rs"));
    assert_eq!(names.len(), 3, "{names:?}");
    for name in &names {
        assert!(name.starts_with("bhalf_"), "{name}");
        for word in FILTER_WORDS {
            assert!(!name.contains(word), "{name} が {word} を含む");
        }
    }
}
