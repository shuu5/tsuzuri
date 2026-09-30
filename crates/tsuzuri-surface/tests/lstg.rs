//! 行 c-ledger-stage の歯: 台帳の一覧の下の項に、pipeline の板の札と同じ読み（`stages`）の段の記号と字を出す。
//! 札と台帳の行は歯の中で組む（fixture の file は使わない・bead の id の接頭辞は fx-l）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tsuzuri_contract::board::{Ci, PipelineBoard, PipelineCard, Reading, Stage};
use tsuzuri_contract::ledger::{BeadId, LedgerList, LedgerRow, MEMO_LABEL};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ledger::{Group, body, staged_body};
use tsuzuri_surface::project::pipeline::{CLOSED_STAGE, kcard, stage_word, stages};
use tsuzuri_surface::project::{Body, Item, Staged, item, staged_item};
use tsuzuri_surface::view::{Fetched, Screen};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 札を描く今（札の since はこの今から経過を引いた時刻）。
const NOW: u64 = 1_790_510_400;

/// 口座 None の札。
#[expect(
    clippy::too_many_arguments,
    reason = "引数が規則の行 R-4 の 5 を越える・R-4 の歯の行が直してこの属性を外す"
)]
fn card(
    id: &str,
    runs: u32,
    stage: Stage,
    reason: Option<&str>,
    elapsed: Option<u64>,
    ci: Option<Ci>,
) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("bead の id"),
        runs,
        stage,
        reason: reason.map(str::to_string),
        account: None,
        since: elapsed.map(|e| NOW - e),
        ci,
    }
}

/// 節の板 LS（8 枚）。
fn ls() -> Vec<PipelineCard> {
    vec![
        card("fx-l.1", 1, Stage::Running, None, Some(40), None),
        card("fx-l.2", 0, Stage::Queued, None, None, None),
        card("fx-l.3", 1, Stage::Failed, Some("verdict:FAIL x"), Some(90), None),
        card("fx-l.4", 1, Stage::Landed, None, Some(120), Some(Ci::Waiting)),
        card("fx-l.5", 1, Stage::Landed, None, Some(30), Some(Ci::Success)),
        card("fx-l.6", 1, Stage::Landed, None, Some(30), None),
        card("fx-l.7", 1, Stage::Landed, Some("closed:gave up"), Some(30), None),
        card(
            "fx-l.8",
            1,
            Stage::Failed,
            Some("terminal:ci:failure"),
            Some(30),
            Some(Ci::Failure),
        ),
    ]
}

fn board_text(cards: Reading<Vec<PipelineCard>>) -> String {
    wire::encode(&PipelineBoard {
        cards,
        misfits: Reading::Unknown,
    })
    .expect("板の電文")
}

fn staged(state: Option<&'static str>, closed: bool, word: &str) -> Staged {
    Staged {
        state,
        closed,
        word: word.to_string(),
    }
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

/// 節の台帳 LR（6 行・zz.3 は issue_type task と label intake:memo の open の行）。
fn lr() -> Vec<LedgerRow> {
    vec![
        row("fx-l", "epic", "open"),
        row("fx-l.1", "task", "open"),
        row("fx-l.4", "task", "in_progress"),
        row("fx-l.7", "task", "closed"),
        row("fx-l.9", "task", "open"),
        LedgerRow {
            labels: vec![MEMO_LABEL.to_string()],
            ..row("zz.3", "task", "open")
        },
    ]
}

fn screen_of(rows: Vec<LedgerRow>) -> Screen {
    let text = wire::encode(&LedgerList {
        rows: Reading::Known(rows),
    })
    .expect("電文");
    Screen::initial().after_read(&Fetched::Body(text), 100)
}

/// (1) 札の段の字は閉じた札なら CLOSED_STAGE・ほかは段の名で、札の hover の kind と同じ字。
#[test]
fn lstg_word_rules() {
    let want = [
        "Running",
        "Queued",
        "Failed",
        "Landed",
        "Landed",
        "Landed",
        "閉じた（着地せず）",
        "Failed",
    ];
    assert_eq!(CLOSED_STAGE, want[6]);
    let cards = ls();
    assert_eq!(cards.len(), want.len());
    for (c, w) in cards.iter().zip(want) {
        let word = stage_word(c);
        assert_eq!(word, w, "{}", c.contract);
        assert_eq!(kcard(c, &[], NOW).hover.kind, format!("run · {w}"), "{}", c.contract);
    }
}

/// (2) 板の電文から組んだ段は札ごとに札の読みの記号と閉じたかと、段の字に CI の語を足した字。
#[test]
fn lstg_stages_from_board() {
    let cards = ls();
    let got = stages(&Fetched::Body(board_text(Reading::Known(cards.clone()))), NOW);
    let want: BTreeMap<String, Staged> = [
        ("fx-l.1", staged(Some("run"), false, "Running")),
        ("fx-l.2", staged(Some("wait"), false, "Queued")),
        ("fx-l.3", staged(Some("wait"), false, "Failed")),
        ("fx-l.4", staged(Some("run"), false, "Landed · CI 中")),
        ("fx-l.5", staged(None, false, "Landed · CI 成功")),
        ("fx-l.6", staged(None, false, "Landed")),
        ("fx-l.7", staged(None, true, "閉じた（着地せず）")),
        ("fx-l.8", staged(Some("wait"), false, "Failed")),
    ]
    .into_iter()
    .map(|(id, s)| (id.to_string(), s))
    .collect();
    assert_eq!(got, want);
    for c in &cards {
        let k = kcard(c, &[], NOW);
        let s = &got[c.contract.as_str()];
        assert_eq!(s.state, k.state, "{}", c.contract);
        assert_eq!(s.closed, k.closed, "{}", c.contract);
    }

    for fetched in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
        Fetched::Body(board_text(Reading::Unknown)),
        Fetched::Body(board_text(Reading::Known(vec![]))),
    ] {
        assert!(stages(&fetched, NOW).is_empty(), "{fetched:?}");
    }
}

/// (3) 段の在る項は右の字を段の字と種類にし、無い項と epic の頭は bd の状態の語のまま。
#[test]
fn lstg_list_items() {
    let rows = lr();
    let by = |id: &str| rows.iter().find(|r| r.id.as_str() == id).expect("LR の行");
    let run = staged(Some("run"), false, "Running");

    let one = staged_item(by("fx-l.1"), Some(&run));
    assert_eq!(
        one,
        Item {
            shape: "shape band-beads".to_string(),
            alert: false,
            id: "fx-l.1".to_string(),
            title: "題".to_string(),
            aside: "Running · task".to_string(),
            stage: Some(run.clone()),
        }
    );
    let nine = staged_item(by("fx-l.9"), None);
    assert_eq!(nine, item(by("fx-l.9")));
    assert_eq!(nine.aside, "○ 未着手 · task");
    assert_eq!(nine.stage, None);

    let board = stages(&Fetched::Body(board_text(Reading::Known(ls()))), NOW);
    let mut with_epic = board.clone();
    with_epic.insert("fx-l".to_string(), run.clone());
    let screen = screen_of(rows.clone());
    assert_eq!(
        staged_body(&screen, &with_epic),
        Body::Filled(vec![
            Group {
                head: Some(item(by("fx-l"))),
                children: vec![
                    staged_item(by("fx-l.1"), board.get("fx-l.1")),
                    staged_item(by("fx-l.4"), board.get("fx-l.4")),
                    item(by("fx-l.9")),
                ],
            },
            Group {
                head: None,
                children: vec![item(by("zz.3"))],
            },
        ])
    );

    let Body::Filled(groups) = staged_body(&screen, &board) else {
        panic!("一覧が Filled でない");
    };
    let asides: Vec<&str> = groups
        .iter()
        .flat_map(|g| g.head.iter().chain(g.children.iter()))
        .map(|i| i.aside.as_str())
        .collect();
    assert_eq!(
        asides,
        [
            "○ 未着手 · epic",
            "Running · task",
            "Landed · CI 中 · task",
            "○ 未着手 · task",
            "? まだ分からない · memo",
        ]
    );
    assert_eq!(body(&screen), staged_body(&screen, &BTreeMap::new()));
}

/// 字 `mod dom {` より後の `start` から、次の 4 つの空白と閉じ波括弧だけの行までの字。
fn dom_fn<'a>(src: &'a str, start: &str) -> &'a str {
    let at = src.find("mod dom {").expect("mod dom の字");
    let dom = &src[at..];
    let from = dom
        .find(start)
        .unwrap_or_else(|| panic!("{start} の字が無い"));
    let rest = &dom[from..];
    let end = rest
        .find("\n    }\n")
        .unwrap_or_else(|| panic!("{start} の終わり"));
    &rest[..end]
}

/// src の下の .rs の file の相対の path と字（path の順）。
fn sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
        let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{dir:?} を読む: {e}"));
        for entry in entries {
            let path = entry.expect("dir の項").path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let rel = path
                    .strip_prefix(root)
                    .expect("src の下")
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = std::fs::read_to_string(&path).expect(".rs を読む");
                out.push((rel, text));
            }
        }
    }
    let root = crate_dir().join("src");
    let mut out = Vec::new();
    walk(&root, &root, &mut out);
    out.sort();
    out
}

/// 字の在りか（字が在る file ごとの数）。
fn places(files: &[(String, String)], needle: &str) -> BTreeMap<String, usize> {
    files
        .iter()
        .map(|(rel, text)| (rel.clone(), text.matches(needle).count()))
        .filter(|(_, n)| *n > 0)
        .collect()
}

/// (4) 一覧は板の口の読みから段を組み、項は札と同じ段の記号を出す。
#[test]
fn lstg_dom_wiring() {
    let ledger = read("src/project/ledger.rs");
    let list_view = dom_fn(&ledger, "fn list_view(");
    assert!(
        list_view.contains("let pipe = crate::net::read(pipeline::PATH);"),
        "{list_view}"
    );
    assert!(
        list_view.contains("staged_body(s, &stages(p, crate::net::now()))"),
        "{list_view}"
    );

    let kit = read("src/kit.rs");
    let item_view = dom_fn(&kit, "pub fn item_view(");
    assert!(
        item_view.contains("stage_sym(s.closed, s.state)"),
        "{item_view}"
    );
    assert!(
        item_view.contains(r#"<span class="aside">{sym}{item.aside.clone()}</span>"#),
        "{item_view}"
    );

    let pipeline = read("src/project/pipeline.rs");
    let kcard_view = dom_fn(&pipeline, "fn kcard_view(");
    assert!(
        kcard_view.contains("let sym = stage_sym(card.closed, card.state);"),
        "{kcard_view}"
    );
    assert!(pipeline.contains("pub use dom::stage_sym;"));
    assert_eq!(pipeline.matches("inner_html=CROSS").count(), 1);
    assert_eq!(pipeline.matches("inner_html=CHECK").count(), 1);

    let files = sources();
    let only = |rel: &str| BTreeMap::from([(rel.to_string(), 1)]);
    for (needle, rel) in [
        (
            "pub fn stage_sym(closed: bool, state: Option<&'static str>) -> AnyView {",
            "project/pipeline.rs",
        ),
        ("pub fn stages(", "project/pipeline.rs"),
        ("&stages(p, crate::net::now())", "project/ledger.rs"),
        (
            "staged_item(r, stages.get(r.id.as_str()))",
            "project/ledger.rs",
        ),
    ] {
        assert_eq!(places(&files, needle), only(rel), "{needle}");
    }
}

/// verify の filter の語（main の 164 語に畳んだ語と後の行 g-gz の接頭辞・165 語）。
const FILTERS: [&str; 165] = [
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
    "kindlab_",
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
    "nxorg_",
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
];

/// (6) この file の歯の名は 5 つで、どれも lstg_ で始まり、先頭の lstg_ を除いた字は filter の語を含まない。
#[test]
fn lstg_names_stay_apart() {
    let text = read("tests/lstg.rs");
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
    assert_eq!(names.len(), 5, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("lstg_")
            .unwrap_or_else(|| panic!("{name} が lstg_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
