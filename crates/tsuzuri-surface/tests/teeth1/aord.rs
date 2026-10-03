//! 行 h-acct-order の歯: account board の口座の行と群の候補を名の自然な順にし、退役の口座を除く（裁定 t3-hub.53.14）。
//! 並べは電文を読む 1 か所（account の doc）で arrange を通し、doc を通して組む block はどれも同じ並びを受ける。
#![cfg(test)]

use std::path::{Path, PathBuf};

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::home::{Cells, MvKind};
use tsuzuri_surface::account::session::{Head, Sort};
use tsuzuri_surface::account::{self, home, session};
use tsuzuri_surface::project::{Body, NOT_READ};
use tsuzuri_surface::view::Fetched;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_text() -> String {
    read("../../tests/fixtures/account/acct-doc.json")
}

fn fixture() -> AccountDoc {
    wire::decode(&fixture_text()).expect("fixture が AccountDoc として読める")
}

fn strings(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| s.to_string()).collect()
}

/// 電文 D: accounts は acct-10（acct-1 の写しの名を替えた行）・acct-3（退役）・acct-2・acct-1、
/// Tier1 の候補は acct-10・acct-3・acct-2・acct-9、移動の 1 行目の from は acct-3。
fn doc_d() -> AccountDoc {
    let mut d = fixture();
    let Reading::Known(rows) = &d.accounts else {
        panic!("fixture の accounts は読める");
    };
    let (a1, a2, a3) = (rows[0].clone(), rows[1].clone(), rows[2].clone());
    assert!(a3.retired && !a1.retired && !a2.retired);
    let mut a10 = a1.clone();
    a10.label = "acct-10".to_string();
    d.accounts = Reading::Known(vec![a10, a3, a2, a1]);
    if let Reading::Known(g) = &mut d.groups {
        g[0].row.candidates = strings(&["acct-10", "acct-3", "acct-2", "acct-9"]);
    }
    if let Reading::Known(m) = &mut d.moves {
        m[0].from = Some("acct-3".to_string());
    }
    d
}

fn labels(doc: &AccountDoc) -> Vec<String> {
    match &doc.accounts {
        Reading::Known(rows) => rows.iter().map(|r| r.label.clone()).collect(),
        Reading::Unknown => panic!("accounts が読めない"),
    }
}

fn body(doc: &AccountDoc) -> Fetched {
    Fetched::Body(wire::encode(doc).expect("電文の字"))
}

fn filled<T: std::fmt::Debug>(body: Body<T>) -> T {
    match body {
        Body::Filled(v) => v,
        other => panic!("中身ありでない: {other:?}"),
    }
}

/// (1) 退役の行を除き自然な順に並べ、候補も退役の名を除いて自然な順・accounts が読めなければ並べるだけ・ほかの欄はそのまま。
#[test]
fn aord_arrange_rules() {
    let d = doc_d();
    let Reading::Known(rows) = &d.accounts else {
        panic!("D の accounts は読める");
    };
    let mut want = d.clone();
    want.accounts = Reading::Known(vec![rows[3].clone(), rows[2].clone(), rows[0].clone()]);
    if let Reading::Known(g) = &mut want.groups {
        g[0].row.candidates = strings(&["acct-2", "acct-9", "acct-10"]);
    }
    let got = account::arrange(d.clone());
    assert_eq!(got, want);
    assert_eq!(labels(&got), strings(&["acct-1", "acct-2", "acct-10"]));
    let Reading::Known(g) = &got.groups else {
        panic!("groups は読める");
    };
    assert_eq!(g[1].row.candidates, strings(&["acct-2"]));
    let Reading::Known(m) = &got.moves else {
        panic!("moves は読める");
    };
    assert_eq!(m[0].from.as_deref(), Some("acct-3"));

    // accounts が読めなければ候補は除かずに並べるだけ。
    let mut u = d.clone();
    u.accounts = Reading::Unknown;
    let mut want = u.clone();
    if let Reading::Known(g) = &mut want.groups {
        g[0].row.candidates = strings(&["acct-2", "acct-3", "acct-9", "acct-10"]);
    }
    assert_eq!(account::arrange(u), want);

    // fixture は acct-3 の行が消えるだけ。
    let f = fixture();
    let mut want = f.clone();
    if let Reading::Known(a) = &mut want.accounts {
        a.retain(|r| r.label != "acct-3");
    }
    assert_eq!(account::arrange(f), want);
}

/// (2) doc は読めた電文を arrange に通して返す・理由の 3 つは変わらない。
#[test]
fn aord_doc_reads_arranged() {
    let d = doc_d();
    assert_eq!(account::doc(&body(&d)), Ok(account::arrange(d.clone())));
    let f = account::doc(&Fetched::Body(fixture_text())).expect("fixture が読める");
    assert_eq!(labels(&f), strings(&["acct-1", "acct-2"]));
    assert_eq!(account::doc(&Fetched::NotRead), Err(NOT_READ));
    assert_eq!(account::doc(&Fetched::Failed), Err(account::UNREAD));
    assert_eq!(
        account::doc(&Fetched::Body("{}".to_string())),
        Err(account::BAD_BODY)
    );
}

/// (3) HOME の block と session の表の口座の並べも doc を通した並びになる・移動の行は電文の字のまま。
#[test]
fn aord_blocks_follow() {
    let d = doc_d();
    let h = home::content(&body(&d), d.at);
    let accounts = filled(h.accounts);
    let names: Vec<&str> = accounts.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(names, vec!["acct-1", "acct-2", "acct-10"]);
    assert!(accounts.iter().all(|r| r.cells != Cells::Retired));
    let groups = filled(h.groups);
    assert_eq!(groups[0].candidates, vec!["acct-2", "acct-9", "acct-10"]);
    let moves = filled(h.moves);
    let moved = moves
        .shown
        .iter()
        .find(|m| m.kind == MvKind::Moved)
        .expect("移動の記録の行");
    assert_eq!(moved.from, "acct-3");

    let mut s = d.clone();
    for (i, name) in ["acct-10", "acct-2", "acct-1"].into_iter().enumerate() {
        s.sessions[i].account = Some(name.to_string());
    }
    let table = filled(session::content(&body(&s), Sort::Account, s.at));
    let heads: Vec<&str> = table
        .groups
        .iter()
        .filter_map(|g| match &g.head {
            Some(Head::Account { label, .. }) => Some(label.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(heads, vec!["acct-1", "acct-2", "acct-10"]);
}

/// src の下の .rs の file の全部（path の順）。
fn sources() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("src を読む").flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let mut paths = Vec::new();
    walk(&crate_dir().join("src"), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).expect("src の file");
            (p, text)
        })
        .collect()
}

/// (4) 電文を読む所は account の mod.rs の doc の 1 つだけで、doc の本体が arrange を通す。
#[test]
fn aord_one_place() {
    let word = "decode::<AccountDoc>";
    let hits: Vec<(PathBuf, usize)> = sources()
        .into_iter()
        .map(|(p, text)| (p, text.matches(word).count()))
        .filter(|(_, n)| *n > 0)
        .collect();
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].1, 1, "{hits:?}");
    assert_eq!(hits[0].0, crate_dir().join("src/account/mod.rs"));

    let src = read("src/account/mod.rs");
    let start = src.find("pub fn doc(").expect("pub fn doc(");
    let rest = &src[start..];
    let end = rest.find("\n}\n").expect("doc の本体の終わり");
    assert!(rest[..end].contains("arrange"), "{}", &rest[..end]);
}

/// (7) この file の歯の名は aord_ で始まり、ほかの行の verify の語を部分の字として含まない。
#[test]
fn aord_names_stay_apart() {
    names_apart();
}

/// ほかの行の verify の語（この file の歯の名が部分の字として含まない語）。
const WORDS: [&str; 106] = [
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
    "nact_",
    "mtree_",
];

/// この file の #[test] の次の行の fn の名を読み、aord_ の後ろが語を含まないことを見る。
fn names_apart() {
    let src = read("tests/teeth1/aord.rs");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim() == "#[test]")
        .filter_map(|(i, _)| lines.get(i + 1))
        .filter_map(|l| l.trim().strip_prefix("fn "))
        .filter_map(|l| l.split('(').next())
        .collect();
    assert!(names.len() >= 5, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("aord_")
            .unwrap_or_else(|| panic!("{name} が aord_ で始まらない"));
        for word in WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
