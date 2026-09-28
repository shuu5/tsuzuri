//! 便 h-led の歯: account board の台帳の処理状況の表・並べ方と query・並べの決まり・行の字・
//! 列の最大と最小の印・台帳が Unknown の行・詳しくの段の開き閉じは URL と画面の外の保存に書かない。

use std::path::PathBuf;

use tsuzuri_contract::account::{AccountDoc, ProjectRow};
use tsuzuri_contract::board::{LedgerJudge, Reading};
use tsuzuri_contract::stats::{LedgerStats, OpenCounts};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::ledger::{
    self, COLUMNS, HI, HROW, JTAB, LEGEND, LO, MORE, SORT_KEY, Sort, UNKNOWN_KEY, extremes, mark,
    order, sort_of, table, with_sort,
};
use tsuzuri_surface::account::{self, BAD_BODY, UNREAD};
use tsuzuri_surface::frame::param;
use tsuzuri_surface::project::ledger::{JUDGES, NONE};
use tsuzuri_surface::project::{Body, NO_CONTENT, NOT_READ};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

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

/// fixture の proj-a の台帳（Known）の複製。
fn base_stats() -> LedgerStats {
    match &fixture().projects[0].ledger {
        Reading::Known(s) => s.clone(),
        Reading::Unknown => panic!("fixture の proj-a の台帳が Known でない"),
    }
}

/// 台帳の欄を変えた LedgerStats。
fn stats(judge: LedgerJudge, task: u32, net24: i64, net7: i64, rate: f64) -> LedgerStats {
    let mut s = base_stats();
    s.judge = judge;
    s.open = OpenCounts {
        task,
        ..s.open
    };
    s.net_drop_24h = net24;
    s.net_drop_7d = net7;
    s.closed_per_day = rate;
    s
}

/// fixture の proj-a の行を名と台帳だけ変えて並べた電文。
fn doc_of(rows: Vec<(&str, Reading<LedgerStats>)>) -> AccountDoc {
    let mut doc = fixture();
    let template: ProjectRow = doc.projects[0].clone();
    doc.projects = rows
        .into_iter()
        .map(|(name, ledger)| ProjectRow {
            name: name.to_string(),
            ledger,
            ..template.clone()
        })
        .collect();
    doc
}

fn names(doc: &AccountDoc, idx: &[usize]) -> Vec<String> {
    idx.iter().map(|&i| doc.projects[i].name.clone()).collect()
}

/// query の鍵（順）。
fn keys(search: &str) -> Vec<String> {
    search
        .trim_start_matches('?')
        .split('&')
        .filter(|kv| !kv.is_empty())
        .map(|kv| kv.split_once('=').map_or(kv, |(k, _)| k).to_string())
        .collect()
}

/// (1) 並べ方は 4 つで、query の lsort から決め（ほかの字と無いときは judge）、judge のときは lsort を消す。
#[test]
fn acctled_sort_from_query_and_write() {
    let got: Vec<&str> = Sort::ALL.iter().map(|s| s.key()).collect();
    assert_eq!(got, vec!["judge", "project", "net", "backlog"]);
    let labels: Vec<&str> = Sort::ALL.iter().map(|s| s.label_key()).collect();
    assert_eq!(
        labels,
        vec!["sort_judge", "sort_project", "sort_net", "sort_backlog"]
    );
    assert_eq!(SORT_KEY, "lsort");
    for s in Sort::ALL {
        assert_eq!(sort_of(&format!("?lsort={}", s.key())), s);
        assert_eq!(
            sort_of(&format!("?board=account&tab=session&lsort={}&mode=expert", s.key())),
            s
        );
    }
    for q in [
        "",
        "?",
        "?lsort",
        "?lsort=",
        "?lsort=Judge",
        "?lsort=need",
        "?lsort=bogus",
        "?sort=net",
        "?board=account&tab=session",
    ] {
        assert_eq!(sort_of(q), Sort::Judge, "{q}");
    }

    let base = "?board=account&tab=session&mode=expert";
    assert_eq!(with_sort(base, Sort::Judge), base);
    assert_eq!(
        with_sort("?board=account&lsort=net&tab=session", Sort::Judge),
        "?board=account&tab=session"
    );
    assert_eq!(with_sort("?lsort=net&lsort=backlog", Sort::Judge), "?");
    assert_eq!(
        with_sort(base, Sort::Net),
        "?board=account&tab=session&mode=expert&lsort=net"
    );
    assert_eq!(
        with_sort("?board=account&lsort=net&tab=session", Sort::Backlog),
        "?board=account&lsort=backlog&tab=session"
    );
    // (6) 書く関数は lsort のほかの鍵を増やさず、読みと書きが往復する。
    for q in [
        "",
        base,
        "?board=account&lsort=net&tab=session",
        "?sort=stage&span=24h",
    ] {
        let others: Vec<String> = keys(q).into_iter().filter(|k| k != SORT_KEY).collect();
        for s in Sort::ALL {
            let out = with_sort(q, s);
            assert_eq!(sort_of(&out), s, "{q} → {out}");
            let got: Vec<String> = keys(&out).into_iter().filter(|k| k != SORT_KEY).collect();
            assert_eq!(got, others, "{q} → {out}");
            assert_eq!(
                param(&out, SORT_KEY).is_some(),
                s != Sort::Judge,
                "{q} → {out}"
            );
        }
    }
}

/// (2) 並べの決まり: judge は JUDGES の表の順（Unknown は台帳なしと同じ位）・project は電文の順・
/// net は 7 日で同じなら 24 時間の小さい順・backlog は open の task の多い順（net と backlog は Unknown を末尾）・同じ値は電文の順。
#[test]
fn acctled_order_rules() {
    let doc = doc_of(vec![
        ("u1", Reading::Unknown),
        ("ok", Reading::Known(stats(LedgerJudge::OnTrack, 5, 0, -3, 1.0))),
        ("none", Reading::Known(stats(LedgerJudge::NoLedger, 0, 0, 0, 0.0))),
        ("bad", Reading::Known(stats(LedgerJudge::Clogged, 12, 2, 4, 0.2))),
        ("up", Reading::Known(stats(LedgerJudge::PilingUp, 7, -1, -3, 0.5))),
        ("stall", Reading::Known(stats(LedgerJudge::Stalled, 7, 3, 4, 0.0))),
        ("bad2", Reading::Known(stats(LedgerJudge::Clogged, 12, -2, -3, 0.7))),
        ("u2", Reading::Unknown),
    ]);
    let p = &doc.projects;

    // JUDGES の表は悪い順（滞り・積み増し・停滞・順調・台帳なし）。
    assert_eq!(
        JUDGES.map(|j| j.judge),
        [
            LedgerJudge::Clogged,
            LedgerJudge::PilingUp,
            LedgerJudge::Stalled,
            LedgerJudge::OnTrack,
            LedgerJudge::NoLedger
        ]
    );
    assert_eq!(
        names(&doc, &order(p, Sort::Judge)),
        vec!["bad", "bad2", "up", "stall", "ok", "u1", "none", "u2"]
    );
    assert_eq!(ledger::rank(&Reading::Unknown), 4);
    assert_eq!(
        names(&doc, &order(p, Sort::Project)),
        vec!["u1", "ok", "none", "bad", "up", "stall", "bad2", "u2"]
    );
    // net: 7 日（-3 が 3 つ・0・4 が 2 つ）で、同じなら 24 時間の小さい順（-2・-1・0）、なお同じなら電文の順。
    assert_eq!(
        names(&doc, &order(p, Sort::Net)),
        vec!["bad2", "up", "ok", "none", "bad", "stall", "u1", "u2"]
    );
    // backlog: open の task の多い順（12 が 2 つ・7 が 2 つ・5・0）で、Unknown は末尾。
    assert_eq!(
        names(&doc, &order(p, Sort::Backlog)),
        vec!["bad", "bad2", "up", "stall", "ok", "none", "u1", "u2"]
    );
    // 表の行の並びは order と同じ。
    for s in Sort::ALL {
        assert_eq!(table(&doc, s).order(), order(p, s), "{s:?}");
        assert_eq!(table(&doc, s).sort, s);
    }
}

/// (3)(5) fixture の行の字: proj-a は ✓・task 9・24h の +2・closed/日 0.9・未反映 3・詳しくの段・proj-b と proj-c は st_unknown と「―」。
#[test]
fn acctled_rows_on_fixture() {
    let doc = fixture();
    let t = table(&doc, Sort::Judge);
    assert_eq!(t.names(), vec!["proj-a", "proj-b", "proj-c"]);

    let a = &t.rows[0];
    let Reading::Known(c) = &a.cells else {
        panic!("proj-a の台帳が Known でない");
    };
    assert_eq!(c.judge.judge, LedgerJudge::OnTrack);
    assert_eq!(c.judge.symbol, "✓");
    assert_eq!(a.judge_key(), "j_ok");
    assert_eq!(a.class, "jrow j-ok");
    assert_eq!(c.task, 9);
    assert_eq!(c.net24.text, "+2");
    assert_eq!(c.rate, "0.9");
    assert_eq!(
        a.numbers(),
        [
            "9".to_string(),
            "+2".to_string(),
            "0.9".to_string(),
            "3".to_string()
        ]
    );
    let more: Vec<(&str, String)> = a.more().to_vec();
    assert_eq!(
        more,
        vec![
            ("l_ready", "5".to_string()),
            ("l_blocked", "3".to_string()),
            ("l_stale", "1".to_string()),
            ("l_net7", "+4".to_string()),
            ("l_lead", "4.0d".to_string())
        ]
    );
    assert_eq!(c.net7.class, "net net-up");
    // Known の値が 1 つなので印は付かない。
    assert_eq!(a.marks(), [None, None]);
    assert_eq!(c.task_class, "c-n c-task");
    assert_eq!(c.rate_class, "c-n c-rate");

    for (row, name) in t.rows[1..].iter().zip(["proj-b", "proj-c"]) {
        assert_eq!(row.name, name);
        assert_eq!(row.cells, Reading::Unknown);
        assert_eq!(row.judge_key(), UNKNOWN_KEY);
        assert_eq!(UNKNOWN_KEY, "st_unknown");
        assert_eq!(row.class, "jrow j-none");
        assert!(row.numbers().iter().all(|n| n == NONE), "{name}");
        assert!(row.more().iter().all(|(_, v)| v == NONE), "{name}");
        assert_eq!(row.marks(), [None, None]);
    }

    // lead が無ければ「―」。
    let mut no_lead = base_stats();
    no_lead.lead = None;
    let d = doc_of(vec![("x", Reading::Known(no_lead))]);
    assert_eq!(table(&d, Sort::Judge).rows[0].more()[4].1, NONE);

    // 読めない・まだ読んでいない・電文が読めないは測れていない、projects が空は 0 件。
    assert_eq!(
        ledger::content(&Fetched::NotRead, Sort::Judge),
        Body::Unmeasured(NOT_READ)
    );
    assert_eq!(
        ledger::content(&Fetched::Failed, Sort::Judge),
        Body::Unmeasured(UNREAD)
    );
    assert_eq!(
        ledger::content(&Fetched::Body("{}".to_string()), Sort::Judge),
        Body::Unmeasured(BAD_BODY)
    );
    assert_eq!(
        ledger::content(&Fetched::Body(fixture_text()), Sort::Net),
        Body::Filled(table(&doc, Sort::Net))
    );
    let mut empty = doc.clone();
    empty.projects.clear();
    let text = wire::encode(&empty).expect("電文の字");
    assert_eq!(
        ledger::content(&Fetched::Body(text), Sort::Judge),
        Body::Empty(ledger::NO_ROWS)
    );
    // 枠の body は変えない（中身は content が組む）。
    assert_eq!(
        ledger::body(&Fetched::Body(fixture_text())),
        Body::Unmeasured(NO_CONTENT)
    );
    assert_eq!(account::page(account::Tab::Session).block_ids()[1], "ledger");
}

/// (4) 列の最大と最小の印は、Known の値が 2 つ以上在って最大と最小が違うときだけ付く（Unknown の行は数えない）。
#[test]
fn acctled_extremes_marks() {
    assert_eq!(extremes(&[]), None);
    assert_eq!(extremes(&[3.0]), None);
    assert_eq!(extremes(&[3.0, 3.0, 3.0]), None);
    let e = extremes(&[2.0, 9.0, 5.0]).expect("最大と最小");
    assert_eq!((e.max, e.min), (9.0, 2.0));
    assert_eq!(mark(Some(e), 9.0), Some(HI));
    assert_eq!(mark(Some(e), 2.0), Some(LO));
    assert_eq!(mark(Some(e), 5.0), None);
    assert_eq!(mark(None, 9.0), None);
    assert_eq!((HI, LO), ("hi", "lo"));

    let doc = doc_of(vec![
        ("a", Reading::Known(stats(LedgerJudge::OnTrack, 9, 0, 0, 0.9))),
        ("u", Reading::Unknown),
        ("b", Reading::Known(stats(LedgerJudge::Stalled, 2, 0, 0, 0.0))),
        ("c", Reading::Known(stats(LedgerJudge::PilingUp, 5, 0, 0, 1.4))),
    ]);
    let t = table(&doc, Sort::Project);
    let marks: Vec<(&str, [Option<&str>; 2])> =
        t.rows.iter().map(|r| (r.name.as_str(), r.marks())).collect();
    assert_eq!(
        marks,
        vec![
            ("a", [Some(HI), None]),
            ("u", [None, None]),
            ("b", [Some(LO), Some(LO)]),
            ("c", [None, Some(HI)])
        ]
    );
    let Reading::Known(c) = &t.rows[0].cells else {
        panic!("a の台帳");
    };
    assert_eq!(c.task_class, "c-n c-task hi");
    // 並べを変えても印は同じ行に付く。
    let t2 = table(&doc, Sort::Backlog);
    assert_eq!(t2.names(), vec!["a", "c", "b", "u"]);
    assert_eq!(t2.rows[0].marks(), [Some(HI), None]);

    // 同じ値だけの列は印を付けない（Unknown の行は数えない）。
    let flat = doc_of(vec![
        ("a", Reading::Known(stats(LedgerJudge::OnTrack, 4, 0, 0, 0.5))),
        ("u", Reading::Unknown),
        ("b", Reading::Known(stats(LedgerJudge::OnTrack, 4, 0, 0, 0.5))),
        ("c", Reading::Known(stats(LedgerJudge::OnTrack, 4, 0, 0, 0.5))),
    ]);
    for r in table(&flat, Sort::Judge).rows {
        assert_eq!(r.marks(), [None, None], "{}", r.name);
    }

    // 凡例は xm hi と xm lo。
    assert_eq!(LEGEND.map(|(c, _)| c), ["xm hi", "xm lo"]);
}

/// (6)(7) 開き閉じは ledger.rs の signal だけ・保存の口の字は無い・語の鍵は辞書に在り class は stylesheet に在る。
#[test]
fn acctled_fold_signal_keys_and_classes() {
    let src = read("src/account/ledger.rs");
    for word in ["localStorage", "sessionStorage", "cookie"] {
        assert!(!src.contains(word), "ledger.rs に {word} の字が在る");
    }
    assert!(src.contains("RwSignal::new(BTreeMap::new())"));
    assert!(src.contains("pub fn view()"));
    assert!(src.contains("content(f, sort.get())"));

    let mut keys: Vec<&str> = COLUMNS.iter().map(|(_, k)| *k).collect();
    keys.extend(MORE);
    keys.extend(Sort::ALL.map(|s| s.label_key()));
    keys.push(UNKNOWN_KEY);
    keys.extend(JUDGES.map(|j| j.key));
    for key in &keys {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }
    assert_eq!(
        COLUMNS.map(|(_, k)| k),
        ["l_judge", "project", "l_task", "l_rate", "l_unref"]
    );
    assert_eq!(
        MORE,
        ["l_ready", "l_blocked", "l_stale", "l_net7", "l_lead"]
    );

    let css = read("style.css");
    let mut classes: Vec<&str> = vec![JTAB, HROW, "c-n", "rbar", "rowmore", "c-more"];
    classes.extend(COLUMNS.map(|(c, _)| c));
    classes.extend(LEGEND.map(|(c, _)| c));
    for class in classes.iter().flat_map(|c| c.split_whitespace()) {
        assert!(
            css.contains(&format!(".{class}")),
            "stylesheet に class {class} が無い"
        );
    }
}
