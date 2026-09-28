//! 行 h-next-unknown の歯: account board の HOME の次の一手の行と各 project の表の要対応の欄は、
//! なしを判じなかった電文（lead がなしで、なしの結果が判じなかった）を読めない project と同じ測れていないの字と後ろの順にする。
//! fixture は tests/fixtures/account/acct-doc.json（読むだけ）。電文 U と M は proj-a の行の写しから組む。

use std::path::PathBuf;

use tsuzuri_contract::account::{AccountDoc, ProjectRow};
use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::stats::CheckResult;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::home::{self, UNKNOWN_KEY};
use tsuzuri_surface::account::projects::{PSort, need, need_rank, nx_card, table};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::next::{NONE_LINE, UNJUDGED_LINE};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

fn proj<'a>(doc: &'a AccountDoc, name: &str) -> &'a ProjectRow {
    doc.projects
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("fixture に {name} が無い"))
}

/// proj-a の写しの checks の結果（種ごと）と lead を替えた行。
fn with(row: &ProjectRow, results: &[(NextMove, CheckResult)], lead: NextMove) -> ProjectRow {
    let mut row = row.clone();
    let Reading::Known(step) = &mut row.next else {
        panic!("proj-a の next は Known");
    };
    for (kind, result) in results {
        step.checks
            .iter_mut()
            .find(|c| c.kind == *kind)
            .unwrap_or_else(|| panic!("proj-a の checks に {kind:?} が無い"))
            .result = *result;
    }
    step.lead = lead;
    row
}

/// 電文 U（question を Miss・nothing を NotJudged・lead をなし）。
fn row_u(doc: &AccountDoc) -> ProjectRow {
    with(
        proj(doc, "proj-a"),
        &[
            (NextMove::Question, CheckResult::Miss),
            (NextMove::Nothing, CheckResult::NotJudged),
        ],
        NextMove::Nothing,
    )
}

/// 電文 M（U の nothing を Miss）。
fn row_m(doc: &AccountDoc) -> ProjectRow {
    with(&row_u(doc), &[(NextMove::Nothing, CheckResult::Miss)], NextMove::Nothing)
}

fn renamed(mut row: ProjectRow, name: &str) -> ProjectRow {
    row.name = name.to_string();
    row
}

/// (2) の電文（u1・proj-b・g1・proj-a の順）。
fn mixed(doc: &AccountDoc) -> AccountDoc {
    let mut d = doc.clone();
    d.projects = vec![
        renamed(row_u(doc), "u1"),
        proj(doc, "proj-b").clone(),
        renamed(row_m(doc), "g1"),
        proj(doc, "proj-a").clone(),
    ];
    d
}

/// (1) なしを判じなかった行は読めない行と同じ測れていないの行で、判じたなしの行は今のまま。
#[test]
fn hnunk_home_row_unmeasured() {
    let doc = fixture();
    let u = row_u(&doc);
    let r = home::nx_row(&u, doc.at);
    assert_eq!(r.lead, None);
    assert_eq!(r.key, UNKNOWN_KEY);
    assert_eq!(r.key, "st_unknown");
    assert_eq!(r.line, UNJUDGED_LINE);
    assert_eq!(r.line, "まだ判じていない種類があり、することが無いとは言えない");
    assert_eq!(r.class, "nxrow dim");
    let classes: Vec<&str> = r.marks.iter().map(|m| m.class).collect();
    assert_eq!(
        classes,
        ["nxm off", "nxm off", "nxm off", "nxm na", "nxm off", "nxm na"]
    );
    assert_eq!(r.marks, home::marks(&u.next));
    assert_eq!(r.card, nx_card(&u, doc.at));

    let m = row_m(&doc);
    let g = home::nx_row(&m, doc.at);
    assert_eq!(g.lead, Some(NextMove::Nothing));
    assert_eq!(g.key, "nx_g");
    assert_eq!(g.line, NONE_LINE);
    assert_eq!(g.line, "orchestrator が動いている / 待っている");
    assert_eq!(g.class, "nxrow dim");
    assert_eq!(g.card, nx_card(&m, doc.at));
}

/// (2) next_all はなしを判じなかった行を読めない行と同じ後ろの位に置く（同じ位の中は電文の順）。
#[test]
fn hnunk_home_order_last() {
    let doc = mixed(&fixture());
    let rows = home::next_all(&doc);
    let names: Vec<&str> = rows.iter().map(|r| r.project.as_str()).collect();
    assert_eq!(names, ["proj-a", "g1", "u1", "proj-b"]);
    let keys: Vec<&str> = rows.iter().map(|r| r.key).collect();
    assert_eq!(keys, ["nx_e", "nx_g", "st_unknown", "st_unknown"]);
}

/// (3) 表の要対応の欄はなしを判じなかった行を測れていないにし、並べは読めない行と同じ後ろ。
#[test]
fn hnunk_projects_need_unmeasured() {
    let fx = fixture();
    let u = row_u(&fx);
    let n = need(&u);
    assert_eq!(n.lead, None);
    assert_eq!(n.key, "st_unknown");
    assert_eq!(n.sev, "lo");
    assert_eq!(n.line.as_deref(), Some(UNJUDGED_LINE));
    assert_eq!(need_rank(&u), NextMove::ALL.len());
    assert_eq!(need_rank(&u), 7);

    let m = row_m(&fx);
    let g = need(&m);
    assert_eq!(g.lead, Some(NextMove::Nothing));
    assert_eq!(g.key, "nx_g");
    assert_eq!(g.line.as_deref(), Some(NONE_LINE));
    assert_eq!(need_rank(&m), 6);

    let doc = mixed(&fx);
    let t = table(&doc, PSort::Need, Mode::Beginner);
    assert_eq!(t.groups.len(), 1);
    let rows = &t.groups[0].rows;
    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["proj-a", "g1", "u1", "proj-b"]);
    let u1 = &rows[2];
    assert_eq!(u1.need, n);
    let card = &u1.cards.need;
    assert_eq!(*card, nx_card(&renamed(u.clone(), "u1"), doc.at));
    assert_eq!(card.title, "u1 · 測れていない");
    assert_eq!(card.kind, "各 project の次の一手 · ―");
    assert_eq!(card.value, UNJUDGED_LINE);
    assert_eq!(
        card.more,
        [
            "(a) 限度 / 移動 なし",
            "(b) 応答なし なし",
            "(c) 止まっている run なし",
            "(d) 束の承認 ―",
            "(e) 質問 なし",
            "(f) 発効待ち ―",
        ]
    );
}

/// (5) この file の歯の名は hnunk_ で始まり、残りの字は filter の語を含まない。
#[test]
fn hnunk_names_clean() {
    const WORDS: [&str; 115] = [
        "aaround_", "accept_", "account_", "acctcore_", "acctdoc_", "accthb_", "accthome_",
        "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
        "afocus_", "aord_", "apop_", "askcard_", "athr_", "batchpanel_", "board_min_", "bport_",
        "brand_", "btuck_", "cadopt_", "cgdom_", "cmark_", "contract_form_", "cround_", "csled_",
        "denv_", "ecache_", "flight_", "fmark_", "frame_", "fserve_", "fstop_", "gapspage_",
        "gfresh_", "ghb_", "glabel_", "gnav_", "graph_", "gsum_", "gtuck_", "gview_", "hbconf_",
        "hbpost_", "hbproc_", "hbroute_", "hcard_", "hcled_", "hcnx_", "hcproj_", "hcsess_",
        "hfig_", "hook_", "hruling_", "hsblock_", "hsderive_", "hspage_", "hsym_", "iclose_",
        "ilink_", "kcli_", "klink_", "lcard_", "ledgerblock_", "lhome_", "lspark_", "mapview_",
        "mkeys_", "mlink_", "mstore_", "mtree_", "nact_", "nbatch_", "ncard_", "nextstep_",
        "nodepage_", "nstall_", "nsum_", "nsumw_", "ntime_", "nxact_", "parts_", "pclosed_",
        "pfold_", "pipe_", "plimit_", "pmore_", "pquest_", "project_", "ptitle_", "pwhole_",
        "qblock_", "qgate_", "qkey_", "question_", "rhold_", "runsdoc_", "saxis_", "sclosed_",
        "seatblock_", "seatcard_", "server_", "sesplit_", "shb_", "skeleton_", "smore_",
        "stage_", "stats_", "steady_", "sxaxis_", "ticker_",
    ];
    const MORE: [&str; 6] = ["tipx_", "topbar_", "tz_", "urpanel_", "uword_", "wstrip_"];
    let text = read("tests/hnunk.rs");
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
    assert!(names.len() >= 4, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("hnunk_")
            .unwrap_or_else(|| panic!("歯の名 {name} が hnunk_ で始まらない"));
        for w in WORDS.iter().chain(MORE.iter()) {
            assert!(!rest.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
