//! 行 h-acct-cols の歯: account board の各 project の表の決定待ちと未反映の欄の数・並べ unref・
//! 群の card の閾値の行・台帳の表の未反映の欄と、DOM の部分の字。

use std::path::PathBuf;

use tsuzuri_contract::account::{AccountDoc, WindowCap};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::{LedgerStats, UnreflectedKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::ledger::{self, Sort};
use tsuzuri_surface::account::cards::{WAIT_NOTE, grp_card, pcnt_card, thr_line};
use tsuzuri_surface::account::projects::{
    NONE_MARK, PSort, count_text, table, unref_class, unref_count, unref_of, wait_class, wait_of,
};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::ledger::{Unref, panel};
use tsuzuri_surface::view::Screen;

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

/// fixture の proj-a の台帳（Known）の複製。
fn a_stats(doc: &AccountDoc) -> LedgerStats {
    match &doc.projects[0].ledger {
        Reading::Known(s) => s.clone(),
        Reading::Unknown => panic!("fixture の proj-a の台帳が Known でない"),
    }
}

/// 字の「mod dom」より後（DOM の部分）。
fn dom_of(rel: &str) -> String {
    let text = read(rel);
    let at = text
        .find("mod dom")
        .unwrap_or_else(|| panic!("{rel} に mod dom が無い"));
    text[at..].to_string()
}

/// (1) 決定待ちは台帳の open の問いの数（台帳が Unknown は None）・run の数の card の値の行と WAIT_NOTE。
#[test]
fn hacols_wait_numbers() {
    let doc = fixture();
    let got: Vec<Option<u32>> = doc.projects.iter().map(wait_of).collect();
    assert_eq!(got, vec![Some(2), None, None]);

    let t = table(&doc, PSort::Need, Mode::Beginner);
    let rows: Vec<_> = t.groups.iter().flat_map(|g| g.rows.iter()).collect();
    assert_eq!(rows.len(), 3);
    for r in rows {
        assert_eq!(r.wait, wait_of(&doc.projects[r.index]), "{}", r.name);
    }

    assert_eq!(count_text(None), "―");
    assert_eq!(count_text(None), NONE_MARK);
    assert_eq!(count_text(Some(0)), "0");
    assert_eq!(count_text(Some(2)), "2");
    assert_eq!(wait_class(Some(2)), "num on");
    assert_eq!(wait_class(Some(1)), "num on");
    assert_eq!(wait_class(Some(0)), "num");
    assert_eq!(wait_class(None), "num");

    let a = pcnt_card(&doc.projects[0], doc.at);
    assert_eq!(a.value, "あなたの決定待ち 2");
    assert!(!a.more.iter().any(|l| l == WAIT_NOTE), "{:?}", a.more);
    for p in &doc.projects[1..] {
        let c = pcnt_card(p, doc.at);
        assert_eq!(c.value, "あなたの決定待ち ―", "{}", p.name);
        assert_eq!(c.more.last().map(String::as_str), Some(WAIT_NOTE), "{}", p.name);
    }
}

/// (2) 未反映は project board の指標の段の panel の欄 unref と同じ組み方（部分の和と読めない種類）・台帳が Unknown は None。
#[test]
fn hacols_unref_partial() {
    let doc = fixture();
    let s = a_stats(&doc);
    let u = unref_count(&s);
    assert_eq!(
        u,
        Unref {
            count: 3,
            unknown: vec!["ruling", "request"],
        }
    );
    let screen = Screen::initial();
    assert_eq!(u, panel(&s, &screen, doc.at).unref);

    let mut full = s.clone();
    full.unreflected = 5;
    full.unreflected_unknown = Vec::new();
    assert_eq!(unref_count(&full), panel(&full, &screen, doc.at).unref);
    assert_eq!(unref_count(&full).count, 5);
    assert!(unref_count(&full).unknown.is_empty());

    let mut none = s.clone();
    none.unreflected = 0;
    none.unreflected_unknown = vec![
        UnreflectedKind::Memo,
        UnreflectedKind::Ruling,
        UnreflectedKind::Request,
    ];
    assert_eq!(unref_count(&none), panel(&none, &screen, doc.at).unref);
    assert_eq!(unref_count(&none).unknown, vec!["memo", "ruling", "request"]);

    assert_eq!(unref_of(&doc.projects[0]), Some(u.clone()));
    assert_eq!(unref_of(&doc.projects[1]), None);
    assert_eq!(unref_of(&doc.projects[2]), None);

    let t = table(&doc, PSort::Need, Mode::Beginner);
    for r in t.groups.iter().flat_map(|g| g.rows.iter()) {
        assert_eq!(r.unref, unref_of(&doc.projects[r.index]), "{}", r.name);
    }

    assert_eq!(unref_class(Some(&u)), "c-un2 on");
    assert_eq!(unref_class(Some(&unref_count(&none))), "c-un2");
    assert_eq!(unref_class(None), "c-un2");
}

/// (3) 並べ unref は未反映の多い順・台帳が Unknown は後ろ・同じは電文の順。
#[test]
fn hacols_unref_sort() {
    let doc = fixture();
    assert_eq!(table(&doc, PSort::Unref, Mode::Beginner).order(), vec![0, 1, 2]);

    let mut doc2 = doc.clone();
    let mut five = a_stats(&doc);
    five.unreflected = 5;
    doc2.projects[1].ledger = Reading::Known(five);
    let mut d = doc.projects[0].clone();
    d.name = "proj-d".to_string();
    doc2.projects.push(d);
    let t = table(&doc2, PSort::Unref, Mode::Beginner);
    assert_eq!(t.groups.len(), 1);
    assert_eq!(t.order(), vec![1, 0, 3, 2]);
}

/// (4) 閾値の行（窓の短い字と値と %・読めない値は ?・空の列は ―）と、群の card の詳しくの 2 行目。
#[test]
fn hacols_thr_line() {
    let doc = fixture();
    assert_eq!(thr_line(&doc.caps), "閾値 5h 85% · 7d 95% · model 95%");

    let mut caps = doc.caps.clone();
    caps[0].cap = Reading::Unknown;
    caps.push(WindowCap {
        window: "x".to_string(),
        rule: "fleet.x".to_string(),
        cap: Reading::Known(50),
    });
    assert_eq!(thr_line(&caps), "閾値 5h ? · 7d 95% · model 95% · x 50%");
    assert_eq!(thr_line(&[]), "閾値 ―");

    for name in ["Tier1", "Tier2"] {
        let card = grp_card(&doc, name).unwrap_or_else(|| panic!("{name} は在る"));
        assert!(card.more[0].starts_with("候補 "), "{name}: {:?}", card.more);
        assert_eq!(card.more[1], thr_line(&doc.caps), "{name}");
        assert!(card.more[2].starts_with("anchor "), "{name}: {:?}", card.more);
    }
}

/// (5) 台帳の表の Known の行の未反映の数と class・Unknown の行は ―。
#[test]
fn hacols_ledger_unref() {
    let doc = fixture();
    let t = ledger::table(&doc, Sort::Judge);
    assert_eq!(t.names(), vec!["proj-a", "proj-b", "proj-c"]);
    let a = &t.rows[0];
    let Reading::Known(c) = &a.cells else {
        panic!("proj-a の台帳が Known でない");
    };
    assert_eq!(c.unref, unref_count(&a_stats(&doc)));
    assert_eq!(c.un_class, "c-n c-un on");
    assert_eq!(a.numbers()[3], "3");
    for r in &t.rows[1..] {
        assert_eq!(r.numbers()[3], "―", "{}", r.name);
    }

    let mut doc0 = doc.clone();
    let mut zero = a_stats(&doc);
    zero.unreflected = 0;
    doc0.projects[0].ledger = Reading::Known(zero);
    let t0 = ledger::table(&doc0, Sort::Judge);
    let a0 = &t0.rows[0];
    assert_eq!(a0.name, "proj-a");
    let Reading::Known(c0) = &a0.cells else {
        panic!("proj-a の台帳が Known でない");
    };
    assert_eq!(c0.un_class, "c-n c-un");
    assert_eq!(a0.numbers()[3], "0");
}

/// (6) DOM の部分の字と stylesheet の規則。
#[test]
fn hacols_dom_words() {
    let dom = dom_of("src/account/projects.rs");
    for w in [
        "class=wait_class(row.wait)",
        "count_text(row.wait)",
        "class=unref_class(row.unref.as_ref())",
        "unref_view(row.unref.as_ref(), &row.kinds)",
        "fn unref_view(",
        "state_icon(UNKNOWN)",
        "class=\"l2 small muted\"",
    ] {
        assert!(dom.contains(w), "projects.rs の DOM の部分に字 {w} が無い");
    }
    let led = dom_of("src/account/ledger.rs");
    for w in [
        "class=cells.un_class.clone()",
        "cells.unref.count",
        "cells.unref.unknown.is_empty()",
    ] {
        assert!(led.contains(w), "ledger.rs の DOM の部分に字 {w} が無い");
    }
    let css = read("style.css");
    for w in [".c-wait b.on", ".c-un2.on b", ".c-un.on b"] {
        assert!(css.contains(w), "style.css に字 {w} が無い");
    }
}

/// filter の語（main の verify の filter の語を畳んだ語と、後の行の接頭辞）。
const FILTERS: &[&str] = &[
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

/// (8) この file の歯の名は hacols_ で始まり、残りの字は filter の語を含まない。
#[test]
fn hacols_own_names_clean() {
    assert_eq!(FILTERS.len(), 126);
    let text = read("tests/hacols.rs");
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
    assert!(names.len() >= 7, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("hacols_")
            .unwrap_or_else(|| panic!("歯の名 {name} が hacols_ で始まらない"));
        for w in FILTERS {
            assert!(!rest.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
