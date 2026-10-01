//! 行 h-acct-rest の歯: account board の各 project の表の決定待ちの 2 段目（束の承認の件数）・
//! 未反映の 2 段目（種類ごとの件数と見出し）・未反映の列の見出しの和（Σ）・台帳の表の未反映の列の最大と最小の印と、DOM の部分の字。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::stats::{CheckResult, LedgerStats, UnreflectedCount, UnreflectedKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::ledger::{self, HI, LO, Sort, un_class};
use tsuzuri_surface::account::projects::{
    PSort, UnrefKinds, UnrefSum, batch_of, batch_text, kinds_of, table, unref_break, unref_sum,
};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::ledger::{UNREF_KIND_KEY, kind_label, kind_name};
use tsuzuri_surface::vocab::{SOURCE, vocab};

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

/// proj-a の台帳の未反映を memo だけの n にした複製。
fn memo_only(doc: &AccountDoc, n: u32) -> LedgerStats {
    let mut s = a_stats(doc);
    s.unreflected = n;
    s.unreflected_kinds = vec![UnreflectedCount {
        kind: UnreflectedKind::Memo,
        count: n,
    }];
    s.unreflected_unknown = Vec::new();
    s
}

/// 3 つの行の台帳を差し替えた電文。
fn with_ledgers(ledgers: [Reading<LedgerStats>; 3]) -> AccountDoc {
    let mut doc = fixture();
    for (p, l) in doc.projects.iter_mut().zip(ledgers) {
        p.ledger = l;
    }
    doc
}

/// proj-a の束の承認の判じを変えた電文。
fn with_batch(result: CheckResult, count: u32) -> AccountDoc {
    let mut doc = fixture();
    let Reading::Known(step) = &mut doc.projects[0].next else {
        panic!("proj-a の次の一手が Known でない");
    };
    let check = step
        .checks
        .iter_mut()
        .find(|c| c.kind == NextMove::BatchApproval)
        .expect("束の承認の判じ");
    check.result = result;
    check.count = count;
    doc
}

/// 字の「mod dom」より後（DOM の部分）。
fn dom_of(rel: &str) -> String {
    let text = read(rel);
    let at = text
        .find("mod dom")
        .unwrap_or_else(|| panic!("{rel} に mod dom が無い"));
    text[at..].to_string()
}

/// (5) 決定待ちの 2 段目は中核の束の承認の判じの件数（判じなかったか次の一手が Unknown は None と空の字）。
#[test]
fn harest_batch_line() {
    let doc = fixture();
    let got: Vec<Option<u32>> = doc.projects.iter().map(batch_of).collect();
    assert_eq!(got, vec![None, None, None]);
    assert_eq!(batch_text(None), "");

    let hit = with_batch(CheckResult::Hit, 3);
    assert_eq!(batch_of(&hit.projects[0]), Some(3));
    assert_eq!(batch_text(batch_of(&hit.projects[0])), "束 3");
    let miss = with_batch(CheckResult::Miss, 0);
    assert_eq!(batch_of(&miss.projects[0]), Some(0));
    assert_eq!(batch_text(batch_of(&miss.projects[0])), "束 0");
    let unjudged = with_batch(CheckResult::NotJudged, 0);
    assert_eq!(batch_of(&unjudged.projects[0]), None);
    let mut unknown = hit.clone();
    unknown.projects[0].next = Reading::Unknown;
    assert_eq!(batch_of(&unknown.projects[0]), None);

    for d in [&doc, &hit, &miss] {
        for sort in PSort::ALL {
            let t = table(d, sort, Mode::Beginner);
            for r in t.groups.iter().flat_map(|g| g.rows.iter()) {
                assert_eq!(
                    r.batch,
                    batch_of(&d.projects[r.index]),
                    "{sort:?} {}",
                    r.name
                );
            }
        }
    }
}

/// (6) 未反映の 2 段目は読めた種類の 1 以上の件数と読めない種類の見出し（電文の順・語の辞書の見出し）。
#[test]
fn harest_unref_kinds() {
    let doc = fixture();
    let a = a_stats(&doc);
    assert_eq!(
        unref_break(&a),
        UnrefKinds {
            known: vec![("memo".to_string(), 3)],
            unknown: vec!["裁定".to_string(), "発話".to_string()],
        }
    );

    let mut mixed = a.clone();
    mixed.unreflected_kinds = vec![
        UnreflectedCount {
            kind: UnreflectedKind::Memo,
            count: 0,
        },
        UnreflectedCount {
            kind: UnreflectedKind::Ruling,
            count: 2,
        },
        UnreflectedCount {
            kind: UnreflectedKind::Utterance,
            count: 1,
        },
    ];
    mixed.unreflected = 3;
    mixed.unreflected_unknown = Vec::new();
    assert_eq!(
        unref_break(&mixed),
        UnrefKinds {
            known: vec![("裁定".to_string(), 2), ("発話".to_string(), 1)],
            unknown: vec![],
        }
    );

    let got: Vec<UnrefKinds> = doc.projects.iter().map(kinds_of).collect();
    assert_eq!(
        got,
        vec![
            unref_break(&a),
            UnrefKinds::default(),
            UnrefKinds::default()
        ]
    );
    assert_eq!(
        UnrefKinds::default(),
        UnrefKinds {
            known: vec![],
            unknown: vec![],
        }
    );
    for sort in PSort::ALL {
        let t = table(&doc, sort, Mode::Beginner);
        for r in t.groups.iter().flat_map(|g| g.rows.iter()) {
            assert_eq!(
                r.kinds,
                kinds_of(&doc.projects[r.index]),
                "{sort:?} {}",
                r.name
            );
        }
    }
}

/// (7) 種類の見出しは語の辞書の鍵 unref: と kind_name の字の label（memo は english・裁定と発話は rephrase）。
#[test]
fn harest_kind_labels() {
    assert_eq!(UNREF_KIND_KEY, "unref:");
    let got: Vec<String> = UnreflectedKind::ALL.into_iter().map(kind_label).collect();
    assert_eq!(got, vec!["memo", "裁定", "発話"]);

    let en_at = SOURCE
        .find("\n \"english\": {")
        .expect("欄 english の始まり");
    let re_at = SOURCE
        .find("\n \"rephrase\": {")
        .expect("欄 rephrase の始まり");
    let (english, rephrase) = if en_at < re_at {
        (&SOURCE[en_at..re_at], &SOURCE[re_at..])
    } else {
        (&SOURCE[en_at..], &SOURCE[re_at..en_at])
    };
    for (kind, in_english) in [
        (UnreflectedKind::Memo, true),
        (UnreflectedKind::Ruling, false),
        (UnreflectedKind::Utterance, false),
    ] {
        let key = format!("{UNREF_KIND_KEY}{}", kind_name(kind));
        let entry = format!("\"{key}\": {{");
        assert_eq!(english.contains(&entry), in_english, "欄 english の {key}");
        assert_eq!(
            rephrase.contains(&entry),
            !in_english,
            "欄 rephrase の {key}"
        );
        let term = vocab()
            .term(&key)
            .unwrap_or_else(|| panic!("語の辞書に {key} が無い"));
        assert_eq!(term.label, kind_label(kind), "{key}");
        assert!(!term.note.is_empty(), "{key} の注釈が空");
    }

    let dom = dom_of("src/account/projects.rs");
    assert!(
        !dom.contains("kind_name"),
        "projects.rs の DOM の部分に字 kind_name が在る"
    );
}

/// (8) 未反映の列の見出しの和は Known の台帳の和で、Unknown の台帳か読めない種類が在れば部分の和。
#[test]
fn harest_unref_sum() {
    let doc = fixture();
    let sum = unref_sum(&doc);
    assert_eq!(
        sum,
        UnrefSum {
            total: 3,
            partial: true,
            projects: 3,
        }
    );
    assert_eq!(sum.text(), "Σ 3");
    assert_eq!(sum.title(), "3 project の合計");
    assert_eq!(table(&doc, PSort::Need, Mode::Beginner).sum, sum);

    let full = with_ledgers([
        Reading::Known(memo_only(&doc, 3)),
        Reading::Known(memo_only(&doc, 5)),
        Reading::Known(memo_only(&doc, 0)),
    ]);
    let s = unref_sum(&full);
    assert_eq!((s.total, s.partial, s.projects), (8, false, 3));
    assert_eq!(s.text(), "Σ 8");
    for sort in PSort::ALL {
        assert_eq!(table(&full, sort, Mode::Beginner).sum, s, "{sort:?}");
    }

    let part = with_ledgers([
        Reading::Known(memo_only(&doc, 3)),
        Reading::Known(memo_only(&doc, 5)),
        Reading::Known(a_stats(&doc)),
    ]);
    let s = unref_sum(&part);
    assert_eq!((s.total, s.partial), (11, true));

    let mut empty = doc.clone();
    empty.projects.clear();
    assert_eq!(
        unref_sum(&empty),
        UnrefSum {
            total: 0,
            partial: false,
            projects: 0,
        }
    );
}

/// (9) 台帳の表の未反映の列は Known の行の最大と最小の印を持ち、class の語の順は c-n c-un・印・on。
#[test]
fn harest_ledger_un_marks() {
    assert_eq!(un_class(None, 0), "c-n c-un");
    assert_eq!(un_class(None, 4), "c-n c-un on");
    assert_eq!(un_class(Some(HI), 9), "c-n c-un hi on");
    assert_eq!(un_class(Some(LO), 0), "c-n c-un lo");

    let doc = fixture();
    let three = with_ledgers([
        Reading::Known(memo_only(&doc, 9)),
        Reading::Known(memo_only(&doc, 0)),
        Reading::Known(memo_only(&doc, 4)),
    ]);
    let t = ledger::table(&three, Sort::Project);
    assert_eq!(t.names(), vec!["proj-a", "proj-b", "proj-c"]);
    let third: Vec<Option<&str>> = t.rows.iter().map(|r| r.marks()[2]).collect();
    assert_eq!(third, vec![Some(HI), Some(LO), None]);
    let classes: Vec<String> = t
        .rows
        .iter()
        .map(|r| match &r.cells {
            Reading::Known(c) => c.un_class.clone(),
            Reading::Unknown => panic!("{} の台帳が Known でない", r.name),
        })
        .collect();
    assert_eq!(
        classes,
        vec!["c-n c-un hi on", "c-n c-un lo", "c-n c-un on"]
    );

    let mut gap = three.clone();
    gap.projects[1].ledger = Reading::Unknown;
    let t = ledger::table(&gap, Sort::Project);
    let third: Vec<Option<&str>> = t.rows.iter().map(|r| r.marks()[2]).collect();
    assert_eq!(third, vec![Some(HI), None, Some(LO)]);

    let t = ledger::table(&doc, Sort::Project);
    for r in &t.rows {
        assert_eq!(r.marks()[2], None, "{}", r.name);
    }
}

/// (11) DOM の部分の字と stylesheet の規則。
#[test]
fn harest_dom_words() {
    let dom = dom_of("src/account/projects.rs");
    for w in [
        "<span class=\"l2 small muted\">{batch_text(row.batch)}</span>",
        "unref_view(row.unref.as_ref(), &row.kinds)",
        "fn unref_view(unref: Option<&Unref>, kinds: &UnrefKinds)",
        "sum_view(sum)",
        "fn sum_view(sum: UnrefSum)",
        "class=\"small muted num\"",
        "title=sum.title()",
        "sum.text()",
        "sum.partial.then(|| state_icon(UNKNOWN))",
    ] {
        assert!(dom.contains(w), "projects.rs の DOM の部分に字 {w} が無い");
    }
    let packed: String = dom.chars().filter(|c| !c.is_whitespace()).collect();
    let counts = packed
        .find("letcounts=kinds.known.iter()")
        .expect("読めた種類の件数の並び");
    let unknown = packed
        .find("letunknown=kinds.unknown.iter()")
        .expect("読めない種類の見出しの並び");
    assert!(counts < unknown, "読めた種類を先に組む");
    assert!(
        packed.contains("{counts}{unknown}"),
        "読めた種類の後に読めない種類"
    );

    let led = read("src/account/ledger.rs");
    for w in [
        "extremes(&unrefs)",
        "unref_of(p).and_then(|u| mark(ext_un, f64::from(u.count))),",
        "[of(&c.task_class), of(&c.rate_class), of(&c.un_class)]",
    ] {
        assert!(led.contains(w), "ledger.rs に字 {w} が無い");
    }
    let css = read("style.css");
    for w in [
        ".prow .l2 {",
        ".prow .c-wait .l2",
        ".hi {",
        ".lo {",
        ".h-l_unref",
    ] {
        assert!(css.contains(w), "style.css に字 {w} が無い");
    }
    let hacols = read("tests/hacols.rs");
    assert!(hacols.contains("\"unref_view(row.unref.as_ref(), &row.kinds)\""));
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

/// (12) この file の歯の名は 7 つで harest_ で始まり、名の全体は filter の語を含まない。
#[test]
fn harest_own_names_clean() {
    assert_eq!(FILTERS.len(), 149);
    let text = read("tests/harest.rs");
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
    assert_eq!(names.len(), 7, "{names:?}");
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
