//! 行 g-next-rows の歯: 次の一手の一覧の行の対象の id・当たった行の次の手の link・当たった質問の行の経過・
//! DOM が link と経過を組む字・歯の名。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::QuestionList;
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::next::{
    ACCOUNT_LINK, ANSWER_LINK, BATCH_LINK, GAPS_LINK, Link, Mark, Next, PIPE_LINK, Row,
    SESSION_LINK, action, content, next, row, row_link, waited,
};
use tsuzuri_surface::project::{Body, ask};
use tsuzuri_surface::view::Fetched;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

const MODES: [Mode; 2] = [Mode::Beginner, Mode::Expert];

/// fixture の 4 組（組の名 → 電文の字）。
fn fixture() -> BTreeMap<String, String> {
    let sets: BTreeMap<String, NextStep> =
        wire::decode(&read("../../tests/fixtures/surface/next-step.json"))
            .expect("fixture の組が電文として読める");
    sets.into_iter()
        .map(|(name, s)| (name, wire::encode(&s).expect("電文")))
        .collect()
}

fn filled(name: &str) -> Next {
    let text = fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"));
    match content(&Fetched::Body(text)) {
        Body::Filled(n) => n,
        other => panic!("組 {name} が中身を出さない: {other:?}"),
    }
}

fn rest_row<'a>(n: &'a Next, key: &str) -> &'a Row {
    n.rest
        .iter()
        .find(|r| r.key == key)
        .unwrap_or_else(|| panic!("一覧に {key} が無い"))
}

fn check(kind: NextMove, result: CheckResult, count: u32, target: Option<&str>) -> NextCheck {
    NextCheck {
        kind,
        result,
        count,
        target: target.map(|t| BeadId::new(t).expect("id")),
    }
}

/// lead がなしで、6 種を同じ結果にした電文（なしの check は置かない）。
fn six(result: CheckResult) -> NextStep {
    NextStep {
        checks: vec![
            check(NextMove::LimitOrMove, result, 1, None),
            check(NextMove::Unresponsive, result, 1, None),
            check(NextMove::StalledRun, result, 2, Some("nx.7")),
            check(NextMove::BatchApproval, result, 4, None),
            check(NextMove::Question, result, 3, Some("nq.4")),
            check(NextMove::AwaitingEffect, result, 1, None),
        ],
        lead: NextMove::Nothing,
    }
}

fn link(href: String, text: &'static str) -> Option<Link> {
    Some(Link { href, text })
}

/// (1) 一覧の行は電文の対象の id を字で持ち、ほかの欄は今のまま。
#[test]
fn nact_row_carries_target() {
    let stalled = filled("stalled");
    let e = rest_row(&stalled, "nx_e");
    assert_eq!(e.target.as_deref(), Some("nx.2"));
    assert_eq!(e.class, "on");
    assert_eq!(e.mark, Mark::Count(3));
    assert_eq!(e.kind, NextMove::Question);

    let nj = filled("not-judged");
    assert_eq!(rest_row(&nj, "nx_e").target.as_deref(), Some("nq.1"));
    assert_eq!(rest_row(&nj, "nx_c").target, None);

    for name in ["question", "nothing"] {
        for r in &filled(name).rest {
            assert_eq!(r.target, None, "{name} の {}", r.key);
        }
    }

    let none = row(NextMove::Question, None);
    assert_eq!(none.target, None);
    assert_eq!(none.key, "nx_e");
    assert_eq!(none.class, "off");
    assert_eq!(none.mark, Mark::Unmeasured);
    check_target();
}

/// check の target の有り無しと行の対象。
fn check_target() {
    // check が在っても target が無ければ None。
    let c = check(NextMove::StalledRun, CheckResult::Hit, 2, None);
    assert_eq!(row(NextMove::StalledRun, Some(&c)).target, None);
    let c = check(NextMove::StalledRun, CheckResult::Hit, 2, Some("nx.7"));
    let r = row(NextMove::StalledRun, Some(&c));
    assert_eq!(r.target.as_deref(), Some("nx.7"));
    assert_eq!((r.key, r.class, r.mark), ("nx_c", "on", Mark::Count(2)));
}

/// (2) 当たった行だけが次の手の link を持つ（止まっている走行は block「pipeline」へ・ほかは action）。
#[test]
fn nact_row_link_rules() {
    for mode in MODES {
        let m = mode.key();
        let hit = next(&six(CheckResult::Hit));
        let want = [
            link(format!("?board=account&tab=home&mode={m}"), ACCOUNT_LINK),
            link(format!("?board=account&tab=session&mode={m}"), SESSION_LINK),
            link("#pipe".to_string(), PIPE_LINK),
            link(format!("?mode={m}&win=ask"), BATCH_LINK),
            link(format!("?mode={m}&win=ask&id=nq.4"), ANSWER_LINK),
            link(format!("?mode={m}&win=gaps"), GAPS_LINK),
        ];
        assert_eq!(hit.rest.len(), 6);
        for (r, w) in hit.rest.iter().zip(want) {
            let got = row_link(r, mode);
            assert_eq!(got, w, "{} {m}", r.key);
            if r.kind != NextMove::StalledRun {
                assert_eq!(got, action(r.kind, r.target.as_deref(), mode), "{} {m}", r.key);
            }
        }
        for result in [CheckResult::Miss, CheckResult::NotJudged] {
            for r in &next(&six(result)).rest {
                assert_eq!(row_link(r, mode), None, "{result:?} {} {m}", r.key);
            }
        }
        // lead が質問でなしが当たった電文の一覧のなしの行は None。
        let s = NextStep {
            checks: vec![
                check(NextMove::Question, CheckResult::Hit, 1, Some("nq.4")),
                check(NextMove::Nothing, CheckResult::Hit, 1, None),
            ],
            lead: NextMove::Question,
        };
        let n = next(&s);
        assert_eq!(row_link(rest_row(&n, "nx_g"), mode), None, "{m}");
        // fixture の組 stalled の一覧は nx_e の行だけが link を持つ。
        let stalled = filled("stalled");
        for r in &stalled.rest {
            let want = if r.key == "nx_e" {
                link(format!("?mode={m}&win=ask&id=nx.2"), ANSWER_LINK)
            } else {
                None
            };
            assert_eq!(row_link(r, mode), want, "stalled {} {m}", r.key);
        }
    }
}

fn question_row(result: CheckResult, target: Option<&str>) -> Row {
    let c = check(NextMove::Question, result, 2, target);
    row(NextMove::Question, Some(&c))
}

/// (3) 当たった質問の行だけが、電文の対象の問いの経過の字を持つ。
#[test]
fn nact_waited_rules() {
    let list = Fetched::Body(read("../../tests/fixtures/surface/question-list.json"));
    let q2 = question_row(CheckResult::Hit, Some("qa.2"));
    let q10 = question_row(CheckResult::Hit, Some("qa.10"));
    let cases: [(&Row, u64, &str, u64); 3] = [
        (&q2, 1_790_492_700, "◷ 1h05", 1_790_488_800),
        (&q2, 1_790_748_000, "◷ 3d", 1_790_488_800),
        (&q10, 1_790_494_440, "◷ 4m", 1_790_494_200),
    ];
    for (r, now, want, posted) in cases {
        let got = waited(r, &list, now);
        assert_eq!(got.as_deref(), Some(want), "{:?} {now}", r.target);
        assert_eq!(got, Some(format!("◷ {}", ask::age(now, posted))));
    }
    let now = 1_790_492_700;
    for r in [
        question_row(CheckResult::Hit, Some("nq.4")),
        question_row(CheckResult::Hit, None),
        question_row(CheckResult::Miss, Some("qa.2")),
        row(
            NextMove::StalledRun,
            Some(&check(NextMove::StalledRun, CheckResult::Hit, 2, Some("qa.2"))),
        ),
    ] {
        assert_eq!(waited(&r, &list, now), None, "{r:?}");
    }
    let unknown = wire::encode(&QuestionList {
        cards: Reading::Unknown,
        answerable: true,
    })
    .expect("電文");
    for fetched in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
        Fetched::Body(unknown),
    ] {
        assert_eq!(waited(&q2, &fetched, now), None, "{fetched:?}");
    }
}

/// (4) DOM（mod dom から file の終わり）は問いの一覧の口を読み、row_link と waited で行を組む。
#[test]
fn nact_dom_wiring() {
    let src = read("src/project/next.rs");
    let at = src.find("mod dom {").expect("mod dom が在る");
    let (head, dom) = src.split_at(at);
    for word in ["crate::net::read(ask::PATH)", "row_link(", "waited("] {
        assert!(dom.contains(word), "mod dom に {word} が無い");
    }
    for word in ["pub fn row_link(", "pub fn waited("] {
        assert!(head.contains(word), "mod dom より前に {word} が無い");
    }
}

/// 着地済みの行と同じ波の行の verify の filter の語（106 語）。
const FILTERS: [&str; 106] = [
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
    "aord_",
    "mtree_",
];

/// (6) この file の歯の名は nact_ で始まり、残りの字は着地済みと同じ波の filter の語を含まない。
#[test]
fn nact_own_names_clean() {
    let src = read("tests/nact.rs");
    let lines: Vec<&str> = src.lines().map(str::trim).collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if *line != "#[test]" {
            continue;
        }
        let fn_line = lines[i + 1..]
            .iter()
            .find(|l| l.starts_with("fn "))
            .expect("test の属性の次に fn が在る");
        let name = fn_line["fn ".len()..]
            .split('(')
            .next()
            .expect("fn の名")
            .to_string();
        names.push(name);
    }
    assert!(names.len() >= 5, "歯の名 {names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("nact_")
            .unwrap_or_else(|| panic!("{name} が nact_ で始まらない"));
        for f in FILTERS {
            assert!(!rest.contains(f), "{name} が filter の語 {f} を含む");
        }
    }
}
