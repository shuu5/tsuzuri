//! 便 h-proj の歯: 各 project の表の 9 列の見出し・並べ方（query の psort）・need と group と judge と unref の並び・
//! 群の見出し・run の 4 列・accounts の印・開くの欄・決定待ちと未反映の数（台帳が Unknown は None）・
//! 読めないときは測れていない。

use std::path::PathBuf;

use tsuzuri_contract::account::{AccountDoc, RunCounts};
use tsuzuri_contract::board::{LedgerJudge, NextMove, Reading};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::projects::{
    self, Acc, COLUMNS, GroupHead, NO_ROWS, NONE_MARK, Open, PSort, RUNS, acc, content,
    group_order, hbm_class, judge_rank, need, need_rank, orch, psort_of, runs, sev, table,
    with_psort, without_param,
};
use tsuzuri_surface::account::windows::{NOT_YET_KEY, OPEN_NEW_KEY};
use tsuzuri_surface::account::{self, BAD_BODY, UNREAD};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::ledger::JUDGES;
use tsuzuri_surface::project::seat::{NG, OK};
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

/// 行の project の名（並びの順）。
fn names(doc: &AccountDoc, sort: PSort) -> Vec<String> {
    table(doc, sort, Mode::Beginner)
        .groups
        .iter()
        .flat_map(|g| g.rows.iter().map(|r| r.name.clone()))
        .collect()
}

/// (1) 列は 9 つで、見出しの語の鍵がこの順。
#[test]
fn acctproj_columns_in_order() {
    assert_eq!(
        COLUMNS,
        [
            "project",
            "p_need",
            "p_wait",
            "l_unref",
            "ledger_block",
            "runs4",
            "seat",
            "accounts",
            "p_open",
        ]
    );
    assert_eq!(projects::PTAB, "ptab");
    assert_eq!(projects::HROW, "prow hrow");
    assert_eq!(projects::PGH, "pgh");
}

/// (2) 並べ方は 4 つで、query の psort から決め（ほかの字と無いときは need）、書く関数は need のときに psort を消す。
#[test]
fn acctproj_psort_from_query_and_written_back() {
    let keys: Vec<&str> = PSort::ALL.iter().map(|s| s.key()).collect();
    assert_eq!(keys, vec!["need", "group", "judge", "unref"]);
    let labels: Vec<&str> = PSort::ALL.iter().map(|s| s.label_key()).collect();
    assert_eq!(
        labels,
        vec!["sort_need", "sort_group", "sort_judge", "sort_unref"]
    );
    assert_eq!(projects::PSORT_KEY, "psort");
    for (q, want) in [
        ("?psort=need", PSort::Need),
        ("?psort=group", PSort::Group),
        ("?board=account&tab=projects&psort=judge", PSort::Judge),
        ("psort=unref&mode=expert", PSort::Unref),
        ("", PSort::Need),
        ("?", PSort::Need),
        ("?psort", PSort::Need),
        ("?psort=", PSort::Need),
        ("?psort=Group", PSort::Need),
        ("?psort=project", PSort::Need),
        ("?sort=group", PSort::Need),
    ] {
        assert_eq!(psort_of(q), want, "{q}");
    }

    let q = "?board=account&tab=projects&mode=expert";
    assert_eq!(
        with_psort(q, PSort::Group),
        "?board=account&tab=projects&mode=expert&psort=group"
    );
    assert_eq!(
        with_psort("?board=account&psort=group&tab=projects", PSort::Judge),
        "?board=account&psort=judge&tab=projects"
    );
    assert_eq!(
        with_psort("?board=account&psort=group&tab=projects", PSort::Need),
        "?board=account&tab=projects"
    );
    assert_eq!(with_psort("?psort=unref", PSort::Need), "?");
    assert_eq!(with_psort("", PSort::Need), "?");
    assert_eq!(
        without_param("?psort=a&x=1&psort=b&psortx=2", "psort"),
        "?x=1&psortx=2"
    );
    for s in PSort::ALL {
        assert_eq!(psort_of(&with_psort(q, s)), s);
    }
}

/// (3) need の並べは lead の 7 種の順で next が Unknown はその後ろ・同じなら宣言の順。
#[test]
fn acctproj_need_order() {
    let doc = fixture();
    assert_eq!(names(&doc, PSort::Need), vec!["proj-a", "proj-b", "proj-c"]);

    let a = need(&doc.projects[0]);
    assert_eq!(a.lead, Some(NextMove::Question));
    assert_eq!(a.key, "nx_e");
    assert_eq!(a.sev, "mid");
    assert_eq!(a.line.as_deref(), Some("proj-a.7 · 1 件"));
    for p in &doc.projects[1..] {
        let n = need(p);
        assert_eq!(n.lead, None);
        assert_eq!(n.key, "st_unknown");
        assert_eq!(n.sev, "lo");
        assert_eq!(n.line, None);
    }
    assert_eq!(need_rank(&doc.projects[0]), 4);
    assert_eq!(need_rank(&doc.projects[1]), NextMove::ALL.len());
    for (i, k) in NextMove::ALL.iter().enumerate() {
        let mut p = doc.projects[0].clone();
        if let Reading::Known(s) = &mut p.next {
            s.lead = *k;
        }
        assert_eq!(need_rank(&p), i);
    }
    assert_eq!(sev(Some(NextMove::LimitOrMove)), "hi");
    assert_eq!(sev(Some(NextMove::StalledRun)), "hi");
    assert_eq!(sev(Some(NextMove::BatchApproval)), "mid");
    assert_eq!(sev(Some(NextMove::Nothing)), "lo");

    // Unknown の project を先に置いても、lead の在る project が前に来る（Unknown 同士は宣言の順）。
    let mut rev = doc.clone();
    rev.projects.reverse();
    assert_eq!(names(&rev, PSort::Need), vec!["proj-a", "proj-c", "proj-b"]);

    // 行の class は重さを持つ。
    let t = table(&doc, PSort::Need, Mode::Beginner);
    assert_eq!(t.groups.len(), 1);
    assert_eq!(t.groups[0].head, None);
    assert_eq!(t.groups[0].rows[0].class, "prow sev-mid");
    assert_eq!(t.groups[0].rows[1].class, "prow sev-lo");
    assert_eq!(t.order(), vec![0, 1, 2]);
}

/// (3) group の並べは群の宣言の順で群の無い project は末尾・群が変わる所に群の見出しの行。
#[test]
fn acctproj_group_order_and_heads() {
    let doc = fixture();
    assert_eq!(group_order(&doc), vec!["Tier1", "Tier2"]);
    let t = table(&doc, PSort::Group, Mode::Beginner);
    let heads: Vec<Option<GroupHead>> = t.groups.iter().map(|g| g.head.clone()).collect();
    assert_eq!(
        heads,
        vec![
            Some(GroupHead {
                group: Some("Tier1".into()),
                current: Some("acct-1".into()),
            }),
            Some(GroupHead {
                group: Some("Tier2".into()),
                current: Some("acct-2".into()),
            }),
        ]
    );
    let rows: Vec<Vec<&str>> = t
        .groups
        .iter()
        .map(|g| g.rows.iter().map(|r| r.name.as_str()).collect())
        .collect();
    assert_eq!(rows, vec![vec!["proj-a", "proj-c"], vec!["proj-b"]]);

    // 群の無い project は末尾の見出しの下・groups に無い群は groups の後に projects の順。
    let mut doc2 = doc.clone();
    doc2.projects[0].group = None;
    doc2.projects[1].group = Some("Tier9".into());
    let t = table(&doc2, PSort::Group, Mode::Beginner);
    let got: Vec<(Option<String>, Option<String>, Vec<&str>)> = t
        .groups
        .iter()
        .map(|g| {
            let h = g.head.clone().expect("group の並べは見出しを持つ");
            (h.group, h.current, g.rows.iter().map(|r| r.name.as_str()).collect())
        })
        .collect();
    assert_eq!(
        got,
        vec![
            (Some("Tier1".into()), Some("acct-1".into()), vec!["proj-c"]),
            (Some("Tier9".into()), None, vec!["proj-b"]),
            (None, None, vec!["proj-a"]),
        ]
    );

    // groups が Unknown なら群は projects の順で、今の口座は分からない。
    let mut doc3 = doc.clone();
    doc3.groups = Reading::Unknown;
    assert_eq!(group_order(&doc3), vec!["Tier1", "Tier2"]);
    let t = table(&doc3, PSort::Group, Mode::Beginner);
    assert!(t.groups.iter().all(|g| g.head.as_ref().unwrap().current.is_none()));
    assert_eq!(t.order(), vec![0, 2, 1]);
}

/// (3) judge の並べは JUDGES の表の順（台帳が Unknown は台帳なし）・unref の並べは未反映の多い順
/// （台帳が Unknown は後ろ・同じは宣言の順）。
#[test]
fn acctproj_judge_and_unref_order() {
    let doc = fixture();
    let rank = |j: LedgerJudge| JUDGES.iter().position(|x| x.judge == j).unwrap();
    assert_eq!(judge_rank(&doc.projects[0]), rank(LedgerJudge::OnTrack));
    assert_eq!(judge_rank(&doc.projects[1]), rank(LedgerJudge::NoLedger));
    assert_eq!(judge_rank(&doc.projects[2]), rank(LedgerJudge::NoLedger));
    assert_eq!(names(&doc, PSort::Judge), vec!["proj-a", "proj-b", "proj-c"]);

    // 悪い判定が前に来る（同じ判定は宣言の順）。
    let mut doc2 = doc.clone();
    doc2.projects.rotate_left(1);
    if let Reading::Known(s) = &mut doc2.projects[2].ledger {
        s.judge = LedgerJudge::Clogged;
    }
    assert_eq!(names(&doc2, PSort::Judge), vec!["proj-a", "proj-b", "proj-c"]);
    assert_eq!(table(&doc2, PSort::Judge, Mode::Beginner).order(), vec![2, 0, 1]);

    assert_eq!(names(&doc, PSort::Unref), vec!["proj-a", "proj-b", "proj-c"]);
    assert_eq!(names(&doc2, PSort::Unref), vec!["proj-a", "proj-b", "proj-c"]);
    assert_eq!(table(&doc2, PSort::Unref, Mode::Beginner).order(), vec![2, 0, 1]);

    // 台帳の欄は判定と、読めれば task の数。
    let t = table(&doc, PSort::Need, Mode::Beginner);
    let a = &t.groups[0].rows[0].led;
    assert_eq!(a.judge.key, "j_ok");
    assert_eq!(a.task, Some(9));
    assert_eq!(a.net24.as_ref().map(|n| n.text.as_str()), Some("+2"));
    let b = &t.groups[0].rows[1].led;
    assert_eq!(b.judge.key, "j_none");
    assert_eq!(b.task, None);
    assert_eq!(b.net24, None);
}

/// (4) run の 4 列は wait・run・stop・land の 4 つの数で、0 は class z・Unknown は「―」と class z。
#[test]
fn acctproj_runs4() {
    let doc = fixture();
    let cell = |i: usize| -> Vec<(String, String)> {
        runs(&doc.projects[i].runs)
            .into_iter()
            .map(|r| (r.class, r.text))
            .collect()
    };
    let pair = |c: &str, t: &str| (c.to_string(), t.to_string());
    assert_eq!(
        cell(0),
        vec![
            pair("rq q-wait", "1"),
            pair("rq q-run", "2"),
            pair("rq q-stop z", "0"),
            pair("rq q-land", "3"),
        ]
    );
    assert_eq!(
        cell(1),
        vec![
            pair("rq q-wait z", "0"),
            pair("rq q-run z", "0"),
            pair("rq q-stop", "1"),
            pair("rq q-land z", "0"),
        ]
    );
    assert_eq!(
        cell(2),
        vec![
            pair("rq q-wait z", NONE_MARK),
            pair("rq q-run z", NONE_MARK),
            pair("rq q-stop z", NONE_MARK),
            pair("rq q-land z", NONE_MARK),
        ]
    );
    let names: Vec<&str> = RUNS.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, vec!["wait", "run", "stop", "land"]);
    let one = runs(&Reading::Known(RunCounts {
        wait: 4,
        run: 5,
        stop: 6,
        land: 7,
    }));
    let texts: Vec<&str> = one.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(texts, vec!["4", "5", "6", "7"]);
}

/// (5) accounts の欄は席の口座と、群の今の口座と同じなら ✓・違えば !・分からなければ印なし。
#[test]
fn acctproj_accounts_mark() {
    let doc = fixture();
    assert_eq!(
        acc(&doc, &doc.projects[0]),
        Acc {
            account: Some("acct-1".into()),
            mark: Some(OK),
        }
    );
    assert_eq!(OK.glyph, "✓");
    assert_eq!(
        acc(&doc, &doc.projects[1]),
        Acc {
            account: Some("acct-1".into()),
            mark: Some(NG),
        }
    );
    assert_eq!(NG.glyph, "!");
    assert_eq!(
        acc(&doc, &doc.projects[2]),
        Acc {
            account: None,
            mark: None,
        }
    );

    // card の account が無い・groups が Unknown・群が無い・群が groups に無いは印なし。
    let mut p = doc.projects[0].clone();
    if let Reading::Known(card) = &mut p.seat {
        card.account = None;
    }
    assert_eq!(acc(&doc, &p).mark, None);
    let mut doc2 = doc.clone();
    doc2.groups = Reading::Unknown;
    assert_eq!(acc(&doc2, &doc.projects[0]).mark, None);
    assert_eq!(acc(&doc2, &doc.projects[0]).account.as_deref(), Some("acct-1"));
    let mut p = doc.projects[1].clone();
    p.group = None;
    assert_eq!(acc(&doc, &p).mark, None);
    p.group = Some("Tier9".into());
    assert_eq!(acc(&doc, &p).mark, None);
}

/// (6) 開くの欄は board を持つ project だけ開く button（open_new）・持たない project は not_yet。
#[test]
fn acctproj_open_cell() {
    let doc = fixture();
    let t = table(&doc, PSort::Need, Mode::Expert);
    let opens: Vec<(&str, Open)> = t.groups[0]
        .rows
        .iter()
        .map(|r| (r.name.as_str(), r.open.clone()))
        .collect();
    assert_eq!(
        opens,
        vec![
            ("proj-a", Open::New(":40001/?mode=expert".into())),
            ("proj-b", Open::New(":40002/?mode=expert".into())),
            ("proj-c", Open::NotYet),
        ]
    );
    assert_eq!(opens[0].1.key(), OPEN_NEW_KEY);
    assert_eq!(OPEN_NEW_KEY, "open_new");
    assert_eq!(opens[2].1.key(), NOT_YET_KEY);
    assert_eq!(NOT_YET_KEY, "not_yet");
    assert_eq!(
        projects::open(&doc.projects[0], Mode::Beginner),
        Open::New(":40001/?mode=beginner".into())
    );
}

/// (7) 決定待ちは台帳の open の問いの数・未反映は台帳の数と読めない種類（台帳が Unknown はどちらも None）・
/// orchestrator の欄は席の card の状態と合図。
#[test]
fn acctproj_wait_unref_dash_and_orch() {
    let doc = fixture();
    let t = table(&doc, PSort::Need, Mode::Beginner);
    let rows = &t.groups[0].rows;
    assert_eq!(rows[0].name, "proj-a");
    assert_eq!(rows[0].wait, Some(2));
    let u = rows[0].unref.as_ref().expect("proj-a の台帳は読める");
    assert_eq!(u.count, 3);
    assert_eq!(u.unknown, vec!["ruling", "request"]);
    for r in &rows[1..] {
        assert_eq!(r.wait, None, "{}", r.name);
        assert_eq!(r.unref, None, "{}", r.name);
    }
    let a = orch(&doc.projects[0]);
    assert_eq!(a.state, "run");
    assert_eq!(a.tick, Reading::Known(OK));
    assert_eq!(a.heartbeat, Reading::Known("on"));
    assert_eq!(hbm_class(&a.heartbeat), "hbm small");
    let b = orch(&doc.projects[1]);
    assert_eq!(b.state, "wait");
    assert_eq!(b.tick, Reading::Known(NG));
    assert_eq!(hbm_class(&b.heartbeat), "hbm small hb-off");
    let c = orch(&doc.projects[2]);
    assert_eq!(c.state, "unknown");
    assert_eq!(c.tick, Reading::Unknown);
    assert_eq!(c.heartbeat, Reading::Unknown);
    assert_eq!(hbm_class(&c.heartbeat), "hbm small");
}

/// 中身は口の本文から組み、読めない・まだ読んでいない・電文が読めないは測れていない・0 行は測れて 0 件。
/// 枠の body は変えない（便 h-frame の歯が pin する）。
#[test]
fn acctproj_content_reads_the_doc() {
    let body = Fetched::Body(fixture_text());
    match content(&body, PSort::Group, Mode::Beginner) {
        Body::Filled(t) => {
            assert_eq!(t.sort, PSort::Group);
            assert_eq!(t.order(), vec![0, 2, 1]);
        }
        other => panic!("中身が在るはず: {other:?}"),
    }
    assert_eq!(
        content(&Fetched::NotRead, PSort::Need, Mode::Beginner),
        Body::Unmeasured(NOT_READ)
    );
    assert_eq!(
        content(&Fetched::Failed, PSort::Need, Mode::Beginner),
        Body::Unmeasured(UNREAD)
    );
    assert_eq!(
        content(&Fetched::Body("{".into()), PSort::Need, Mode::Beginner),
        Body::Unmeasured(BAD_BODY)
    );
    let mut empty = fixture();
    empty.projects.clear();
    let text = wire::encode(&empty).expect("電文にできる");
    assert_eq!(
        content(&Fetched::Body(text), PSort::Need, Mode::Beginner),
        Body::Empty(NO_ROWS)
    );
    assert_eq!(projects::body(&body), Body::Unmeasured(NO_CONTENT));
    assert_eq!(projects::BLOCK.id, "ptab");
    assert_eq!(account::PATH, "/api/account");
}

/// (8) 語の鍵は語の辞書に在り、class は stylesheet に在り、面の crate の依存は契約の型の crate の 1 本のまま。
#[test]
fn acctproj_vocab_style_and_deps() {
    let mut keys: Vec<&str> = COLUMNS.to_vec();
    keys.extend(PSort::ALL.iter().map(|s| s.label_key()));
    keys.extend([
        "open_new",
        "not_yet",
        "current_account",
        "st_unknown",
        "sort_by",
        "tick_health",
        "dashboards",
    ]);
    keys.extend(NextMove::ALL.iter().map(|k| need(&{
        let mut p = fixture().projects[0].clone();
        if let Reading::Known(s) = &mut p.next {
            s.lead = *k;
        }
        p
    }).key));
    for key in keys {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }

    let css = read("style.css");
    let mut classes: Vec<String> = vec![
        projects::PTAB,
        projects::PROW,
        projects::HROW,
        projects::PGH,
        projects::C_PN,
        projects::C_NEED,
        projects::C_WAIT,
        projects::C_UN2,
        projects::C_LED,
        projects::C_RUN,
        projects::RC4,
        projects::ZERO,
        projects::C_ORCH,
        projects::C_ACC,
        projects::C_OPEN,
        projects::TKM,
        projects::HBM_OFF,
        projects::NOT_YET_CLASS,
        "sev-hi",
        "sev-mid",
        "sev-lo",
        "pchip",
        "grp",
        "sortbar",
        "seg",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    classes.extend(RUNS.iter().map(|(_, c)| c.to_string()));
    classes.extend(COLUMNS.iter().map(|k| format!("h-{k}")));
    for class in classes {
        for c in class.split_whitespace() {
            assert!(
                css.contains(&format!(".{c}")),
                "stylesheet に class {c} が無い"
            );
        }
    }

    // 依存は足さない（直接の依存は契約の型の crate だけ）。
    let manifest = read("Cargo.toml");
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|s| s.split("\n[").next())
        .expect("[dependencies]");
    let names: Vec<&str> = deps
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split('=').next())
        .map(str::trim)
        .collect();
    assert_eq!(names, vec!["tsuzuri-contract"]);
}
