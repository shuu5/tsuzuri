//! 便 h-home の歯: HOME の 4 段（次の一手の並び・群の枠・口座 × 窓・移動）の並べと字と class・
//! 段ごとの測れていない・移動の畳み・出さない物・語の辞書と stylesheet と依存。

use std::path::PathBuf;

use tsuzuri_contract::account::{AccountDoc, MoveRow};
use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::stats::{CheckResult, NextCheck, NextStep};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::{self, home};
use tsuzuri_surface::account::home::{
    ACCOUNTS_UNREAD, ACCT_HEADS, AcctRow, Cells, GROUPS_UNREAD, GroupView, Home, MARK_KINDS,
    MOVES_UNREAD, Moves, MvKind, MvRow, NONE, NO_ACCOUNTS, NO_GROUPS, NO_MOVES, NO_PROJECTS, NxRow,
    Occupant, SHOW_MV, UNKNOWN_SIGN, content,
};
use tsuzuri_surface::project::next::NONE_LINE;
use tsuzuri_surface::project::seat::{NG, OK, window_row};
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

fn filled<T: std::fmt::Debug>(body: Body<T>) -> T {
    match body {
        Body::Filled(v) => v,
        other => panic!("中身ありでない: {other:?}"),
    }
}

fn fixture_home() -> Home {
    home::home(&fixture())
}

fn classes(row: &NxRow) -> Vec<(&str, &str)> {
    row.marks.iter().map(|m| (m.glyph, m.class)).collect()
}

fn step(lead: NextMove, checks: &[(NextMove, CheckResult)]) -> Reading<NextStep> {
    Reading::Known(NextStep {
        checks: checks
            .iter()
            .map(|(kind, result)| NextCheck {
                kind: *kind,
                result: *result,
                count: 2,
                target: None,
            })
            .collect(),
        lead,
    })
}

/// (1)(2) fixture の並びは proj-a・proj-b・proj-c で、proj-a の 1 行と 6 つの印・読めない project は測れていない。
#[test]
fn accthome_next_all_on_fixture() {
    let rows = filled(fixture_home().next);
    let names: Vec<&str> = rows.iter().map(|r| r.project.as_str()).collect();
    assert_eq!(names, vec!["proj-a", "proj-b", "proj-c"]);

    let a = &rows[0];
    assert_eq!(a.group, "Tier1");
    assert_eq!(a.lead, Some(NextMove::Question));
    assert_eq!(a.key, "nx_e");
    assert_eq!(a.line, "proj-a.7 · 1 件");
    assert_eq!(
        classes(a),
        vec![
            ("a", "nxm off"),
            ("b", "nxm off"),
            ("c", "nxm off"),
            ("d", "nxm na"),
            ("e", "nxm on"),
            ("f", "nxm na"),
        ]
    );
    assert_eq!(a.class, "nxrow");

    for (row, group) in [(&rows[1], "Tier2"), (&rows[2], "Tier1")] {
        assert_eq!(row.group, group);
        assert_eq!(row.lead, None);
        assert_eq!(row.key, "st_unknown");
        assert_eq!(row.line, NONE);
        assert!(row.marks.iter().all(|m| m.class == "nxm na"), "{row:?}");
        assert_eq!(row.class, "nxrow dim");
    }
    let kinds: Vec<NextMove> = a.marks.iter().map(|m| m.kind).collect();
    assert_eq!(kinds, MARK_KINDS.to_vec());
    assert_eq!(MARK_KINDS.to_vec(), NextMove::ALL[..6].to_vec());
    assert!(!MARK_KINDS.contains(&NextMove::Nothing));
}

/// (1) 並びは lead の 7 種の順・同じ種は電文の順・読めない行は後ろに電文の順。
#[test]
fn accthome_next_all_order_rule() {
    let base = fixture();
    let template = base.projects[0].clone();
    let unknown = base.projects[1].clone();
    let mut doc = base.clone();
    let plan: [(&str, Option<NextMove>); 9] = [
        ("u1", None),
        ("q1", Some(NextMove::Question)),
        ("g1", Some(NextMove::Nothing)),
        ("l1", Some(NextMove::LimitOrMove)),
        ("u2", None),
        ("q2", Some(NextMove::Question)),
        ("s1", Some(NextMove::StalledRun)),
        ("a1", Some(NextMove::AwaitingEffect)),
        ("l2", Some(NextMove::LimitOrMove)),
    ];
    doc.projects = plan
        .iter()
        .map(|(name, lead)| {
            let mut p = match lead {
                Some(_) => template.clone(),
                None => unknown.clone(),
            };
            p.name = name.to_string();
            if let (Some(l), Reading::Known(s)) = (lead, &mut p.next) {
                s.lead = *l;
            }
            p
        })
        .collect();
    let rows = home::next_all(&doc);
    let names: Vec<&str> = rows.iter().map(|r| r.project.as_str()).collect();
    assert_eq!(
        names,
        vec!["l1", "l2", "s1", "q1", "q2", "a1", "g1", "u1", "u2"]
    );
    let keys: Vec<&str> = rows.iter().map(|r| r.key).collect();
    assert_eq!(
        keys,
        vec![
            "nx_a",
            "nx_a",
            "nx_c",
            "nx_e",
            "nx_e",
            "nx_f",
            "nx_g",
            "st_unknown",
            "st_unknown"
        ]
    );
    // 並べは電文の projects を変えない。
    assert_eq!(doc.projects[0].name, "u1");

    // 7 種の順は NextMove::ALL の順（全部の種を 1 つずつ逆順に置いても ALL の順に並ぶ）。
    doc.projects = NextMove::ALL
        .iter()
        .rev()
        .map(|lead| {
            let mut p = template.clone();
            p.name = format!("{lead:?}");
            if let Reading::Known(s) = &mut p.next {
                s.lead = *lead;
            }
            p
        })
        .collect();
    let leads: Vec<Option<NextMove>> = home::next_all(&doc).iter().map(|r| r.lead).collect();
    assert_eq!(
        leads,
        NextMove::ALL.iter().map(|l| Some(*l)).collect::<Vec<_>>()
    );

    let mut empty = base.clone();
    empty.projects.clear();
    assert_eq!(home::home(&empty).next, Body::Empty(NO_PROJECTS));
}

/// (2) 1 行の字は next の module の big の what・印は判じた当たりが on・当たらないが off・判じなかったと無い種が na。
#[test]
fn accthome_next_row_line_and_marks() {
    let mut p = fixture().projects[0].clone();
    p.group = None;
    p.next = step(
        NextMove::StalledRun,
        &[
            (NextMove::LimitOrMove, CheckResult::Hit),
            (NextMove::Unresponsive, CheckResult::NotJudged),
            (NextMove::StalledRun, CheckResult::Hit),
            (NextMove::Question, CheckResult::Miss),
            (NextMove::Nothing, CheckResult::Hit),
        ],
    );
    let row = home::nx_row(&p, fixture().at);
    assert_eq!(row.group, NONE);
    assert_eq!(row.key, "nx_c");
    assert_eq!(row.line, "2 件");
    assert_eq!(
        classes(&row),
        vec![
            ("a", "nxm on"),
            ("b", "nxm na"),
            ("c", "nxm on"),
            ("d", "nxm na"),
            ("e", "nxm off"),
            ("f", "nxm na"),
        ]
    );

    p.next = step(NextMove::Nothing, &[]);
    let row = home::nx_row(&p, fixture().at);
    assert_eq!(row.key, "nx_g");
    assert_eq!(row.line, NONE_LINE);
    assert_eq!(row.class, "nxrow dim");
    assert!(row.marks.iter().all(|m| m.class == "nxm na"));

    for (i, kind) in MARK_KINDS.into_iter().enumerate() {
        p.next = step(kind, &[(kind, CheckResult::Hit)]);
        let row = home::nx_row(&p, fixture().at);
        assert_eq!(row.class, "nxrow");
        for (j, m) in row.marks.iter().enumerate() {
            let want = if i == j { "nxm on" } else { "nxm na" };
            assert_eq!(m.class, want, "{kind:?} の印 {j}");
        }
    }
}

/// (3) 群の枠は群の宣言の順で、今の口座・いつから・前の口座・窓の割合・project の印・候補と次の移り先を持つ。
#[test]
fn accthome_groups_on_fixture() {
    let groups: Vec<GroupView> = filled(fixture_home().groups);
    let names: Vec<&str> = groups.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, vec!["Tier1", "Tier2"]);

    let t1 = &groups[0];
    assert_eq!(t1.account, "acct-1");
    assert!(t1.recorded);
    assert_eq!(t1.since, "19:00 JST");
    assert_eq!(t1.previous, "acct-2");
    let pw: Vec<(&str, &str, &str)> = t1
        .pressure
        .iter()
        .map(|p| (p.window, p.short.as_str(), p.used.as_str()))
        .collect();
    assert_eq!(
        pw,
        vec![
            ("five_hour", "5h", "42%"),
            ("seven_day", "7d", "61%"),
            ("seven_day_model", "model", "30%")
        ]
    );
    let members: Vec<(&str, &str, &str)> = t1
        .members
        .iter()
        .map(|m| (m.project.as_str(), m.sign.glyph, m.sign.class))
        .collect();
    assert_eq!(
        members,
        vec![("proj-a", "✓", "gi ok"), ("proj-c", "", "gi unknown")]
    );
    assert_eq!(t1.members[0].sign, OK);
    assert_eq!(t1.members[1].sign, UNKNOWN_SIGN);
    assert_eq!(t1.candidates, vec!["acct-1", "acct-2"]);
    assert_eq!(t1.next_account.as_deref(), Some("acct-2"));
    assert_eq!(t1.next, "acct-2");

    let t2 = &groups[1];
    assert_eq!(t2.account, "acct-2");
    assert!(!t2.recorded);
    assert_eq!(t2.since, vocab().label("no_record"));
    assert_eq!(t2.previous, NONE);
    let used: Vec<&str> = t2.pressure.iter().map(|p| p.used.as_str()).collect();
    assert_eq!(used, vec!["100%", "81%", "12%"]);
    assert_eq!(t2.members.len(), 1);
    assert_eq!(t2.members[0].project, "proj-b");
    assert_eq!(t2.members[0].sign, NG);
    assert_eq!((NG.glyph, NG.class), ("!", "gi ng"));
    assert_eq!(t2.candidates, vec!["acct-2"]);
    assert_eq!(t2.next_account, None);
    assert_eq!(t2.next, vocab().label("no_target"));
}

/// (3) 窓の割合は今の口座の行から・読めない（列・行・usage・窓）は「―」で 0 と出さない。記録なしは since が在っても記録なしの字。
#[test]
fn accthome_groups_rules() {
    let base = fixture();
    let used = |doc: &AccountDoc, g: usize| -> Vec<String> {
        filled(home::home(doc).groups)[g]
            .pressure
            .iter()
            .map(|p| p.used.clone())
            .collect()
    };
    let dashes = vec![NONE.to_string(); 3];

    let mut d = base.clone();
    d.accounts = Reading::Unknown;
    assert_eq!(used(&d, 0), dashes);
    assert_eq!(used(&d, 1), dashes);

    let mut d = base.clone();
    if let Reading::Known(a) = &mut d.accounts {
        a.retain(|r| r.label != "acct-1");
    }
    assert_eq!(used(&d, 0), dashes);
    assert_eq!(used(&d, 1), vec!["100%", "81%", "12%"]);

    let mut d = base.clone();
    if let Reading::Known(a) = &mut d.accounts {
        a[0].usage = Reading::Unknown;
    }
    assert_eq!(used(&d, 0), dashes);

    let mut d = base.clone();
    if let Reading::Known(a) = &mut d.accounts
        && let Reading::Known(u) = &mut a[0].usage
    {
        u.retain(|q| q.window != "seven_day");
        u[0].used_pct = 0;
    }
    assert_eq!(used(&d, 0), vec!["0%", NONE, "30%"]);

    // 今の口座を替えると、その口座の行から読む。
    let mut d = base.clone();
    if let Reading::Known(g) = &mut d.groups {
        g[0].row.account = "acct-2".to_string();
    }
    assert_eq!(used(&d, 0), vec!["100%", "81%", "12%"]);

    // 記録なしの群は since が在っても記録なしの字・前の口座は在れば出す。
    let mut d = base.clone();
    if let Reading::Known(g) = &mut d.groups {
        g[0].recorded = false;
        g[1].previous = Some("acct-9".to_string());
    }
    let gs = filled(home::home(&d).groups);
    assert_eq!(gs[0].since, vocab().label("no_record"));
    assert_eq!(gs[1].previous, "acct-9");

    // いつからは電文の at から見た hmd（違う日は月日を前に付ける）。
    let mut d = base.clone();
    d.at += 86_400;
    assert_eq!(filled(home::home(&d).groups)[0].since, "09-27 19:00 JST");

    let mut d = base.clone();
    d.groups = Reading::Known(vec![]);
    assert_eq!(home::home(&d).groups, Body::Empty(NO_GROUPS));
}

/// (4) 口座 × 窓は口座の行の順・占有・3 つの窓は window_row の値のまま・退役は窓の代わりに retired。
#[test]
fn accthome_accounts_on_fixture() {
    let doc = fixture();
    let rows: Vec<AcctRow> = filled(home::home(&doc).accounts);
    let labels: Vec<&str> = rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, vec!["acct-1", "acct-2", "acct-3"]);
    assert_eq!(
        ACCT_HEADS,
        [
            "accounts",
            "occupant",
            "five_hour",
            "seven_day",
            "seven_day_model",
            "spark",
            "measured_at"
        ]
    );

    let Reading::Known(accounts) = &doc.accounts else {
        panic!("fixture の accounts は読める");
    };
    for (row, src, group) in [(&rows[0], &accounts[0], "Tier1"), (&rows[1], &accounts[1], "Tier2")] {
        assert_eq!(row.occupant, Occupant::Group(group.to_string()));
        assert_eq!(row.occupant.class(), "occ grp");
        assert_eq!(row.occupant.text(), group);
        let Reading::Known(usage) = &src.usage else {
            panic!("usage が読める");
        };
        let want: Vec<_> = usage.iter().map(|q| Some(window_row(q, doc.at))).collect();
        assert_eq!(row.cells, Cells::Windows(want));
    }
    let Cells::Windows(w2) = &rows[1].cells else {
        panic!("acct-2 は窓を持つ");
    };
    let bars: Vec<&str> = w2.iter().map(|w| w.as_ref().expect("窓").bar_class).collect();
    assert_eq!(bars, vec!["w100", "w80", ""]);

    assert_eq!(rows[2].occupant, Occupant::Retired);
    assert_eq!(rows[2].occupant.text(), NONE);
    assert_eq!(rows[2].cells, Cells::Retired);
    assert_eq!(home::RETIRED_KEY, "retired");
}

/// (4) 占有が無い口座は free_for_pipeline・usage が読めない行と無い窓は「―」・退役は占有が在っても「―」。
#[test]
fn accthome_accounts_rules() {
    let base = fixture();
    let rows = |d: &AccountDoc| filled(home::home(d).accounts);

    let mut d = base.clone();
    if let Reading::Known(a) = &mut d.accounts {
        a[0].occupant = None;
        a[1].usage = Reading::Unknown;
        a[2].occupant = Some("Tier3".to_string());
    }
    let r = rows(&d);
    assert_eq!(r[0].occupant, Occupant::Free);
    assert_eq!(r[0].occupant.class(), "occ free");
    assert_eq!(r[0].occupant.text(), vocab().label("free_for_pipeline"));
    assert_eq!(r[1].cells, Cells::Windows(vec![None, None, None]));
    assert_eq!(r[2].occupant, Occupant::Retired);
    assert_eq!(r[2].cells, Cells::Retired);

    // 窓は窓の名の順に置く（電文の順が違っても・無い窓は None）。
    let mut d = base.clone();
    if let Reading::Known(a) = &mut d.accounts
        && let Reading::Known(u) = &mut a[0].usage
    {
        u.reverse();
        u.retain(|q| q.window != "five_hour");
    }
    let Cells::Windows(ws) = &rows(&d)[0].cells else {
        panic!("窓を持つ");
    };
    let got: Vec<Option<&str>> = ws
        .iter()
        .map(|w| w.as_ref().map(|w| w.window.as_str()))
        .collect();
    assert_eq!(got, vec![None, Some("seven_day"), Some("seven_day_model")]);

    // 退役は usage が読めても窓を出さない。
    let mut d = base.clone();
    if let Reading::Known(a) = &mut d.accounts {
        a[0].retired = true;
    }
    assert_eq!(rows(&d)[0].cells, Cells::Retired);

    let mut d = base.clone();
    d.accounts = Reading::Known(vec![]);
    assert_eq!(home::home(&d).accounts, Body::Empty(NO_ACCOUNTS));
}

fn mv(at: u64, group: &str, from: Option<&str>, to: &str) -> MoveRow {
    MoveRow {
        at,
        group: group.to_string(),
        from: from.map(str::to_string),
        to: to.to_string(),
    }
}

fn row(at: &str, group: &str, from: &str, to: &str, kind: MvKind) -> MvRow {
    MvRow {
        at: at.to_string(),
        group: group.to_string(),
        from: from.to_string(),
        to: to.to_string(),
        kind,
    }
}

/// (5) 移動は電文の順・初めの 8 行・残りは畳める段で見出しは + と残りの数（fixture は知らせの行と 1 つの列）。
#[test]
fn accthome_moves_fold() {
    let m: Moves = filled(fixture_home().moves);
    assert_eq!(
        m.shown,
        vec![
            row(
                "20:50 JST",
                "Tier2",
                "acct-2",
                "",
                MvKind::Refused("no-candidate".to_string())
            ),
            row(
                "20:30 JST",
                "Tier1",
                "acct-2",
                "",
                MvKind::Pressure("5h 100% ≥ 85".to_string())
            ),
            row("19:00 JST", "Tier1", "acct-2", "acct-1", MvKind::Moved),
            row("07:05 JST", "Tier2", NONE, "acct-2", MvKind::Moved),
        ]
    );
    assert!(m.folded.is_empty());
    assert_eq!(m.more, None);
    assert_eq!(SHOW_MV, 8);

    let at = 1_790_510_400;
    for (n, shown, more) in [
        (0, 0, None),
        (1, 1, None),
        (8, 8, None),
        (9, 8, Some("+1")),
        (11, 8, Some("+3")),
        (20, 8, Some("+12")),
    ] {
        let rows: Vec<MoveRow> = (0..n)
            .map(|i| mv(at - 60 * i as u64, &format!("G{i}"), Some("x"), "y"))
            .collect();
        let got = home::moves(&rows, at);
        assert_eq!(got.shown.len(), shown, "{n}");
        assert_eq!(got.folded.len(), n - shown, "{n}");
        assert_eq!(got.more.as_deref(), more, "{n}");
        let order: Vec<String> = got
            .shown
            .iter()
            .chain(&got.folded)
            .map(|r| r.group.clone())
            .collect();
        assert_eq!(order, (0..n).map(|i| format!("G{i}")).collect::<Vec<_>>());
    }

    let mut d = fixture();
    d.moves = Reading::Known(vec![]);
    d.notices = Reading::Known(vec![]);
    assert_eq!(home::home(&d).moves, Body::Empty(NO_MOVES));
}

/// (6) 本文が読めなければ 4 段とも測れていない・読めたら Unknown の列の段だけ測れていない。
#[test]
fn accthome_unmeasured_per_block() {
    for (fetched, want) in [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, account::UNREAD),
        (Fetched::Body("{}".to_string()), account::BAD_BODY),
        (Fetched::Body("not json".to_string()), account::BAD_BODY),
        (Fetched::Body(String::new()), account::BAD_BODY),
    ] {
        let h = content(&fetched, 1_790_510_400);
        assert_eq!(h.next, Body::Unmeasured(want), "{fetched:?}");
        assert_eq!(h.groups, Body::Unmeasured(want), "{fetched:?}");
        assert_eq!(h.accounts, Body::Unmeasured(want), "{fetched:?}");
        assert_eq!(h.moves, Body::Unmeasured(want), "{fetched:?}");
    }
    assert_eq!(
        content(&Fetched::Body(fixture_text()), fixture().at),
        home::home(&account::arrange(fixture()))
    );
    let h = fixture_home();
    assert!(matches!(h.next, Body::Filled(_)));
    assert!(matches!(h.groups, Body::Filled(_)));
    assert!(matches!(h.accounts, Body::Filled(_)));
    assert!(matches!(h.moves, Body::Filled(_)));

    let base = fixture();
    let mut d = base.clone();
    d.groups = Reading::Unknown;
    let h = home::home(&d);
    assert_eq!(h.groups, Body::Unmeasured(GROUPS_UNREAD));
    assert!(matches!(h.next, Body::Filled(_)));
    assert!(matches!(h.accounts, Body::Filled(_)));
    assert!(matches!(h.moves, Body::Filled(_)));

    let mut d = base.clone();
    d.accounts = Reading::Unknown;
    let h = home::home(&d);
    assert_eq!(h.accounts, Body::Unmeasured(ACCOUNTS_UNREAD));
    assert!(matches!(h.groups, Body::Filled(_)));
    assert!(matches!(h.next, Body::Filled(_)));
    assert!(matches!(h.moves, Body::Filled(_)));

    let mut d = base.clone();
    d.moves = Reading::Unknown;
    let h = home::home(&d);
    assert_eq!(h.moves, Body::Unmeasured(MOVES_UNREAD));
    assert!(matches!(h.groups, Body::Filled(_)));
    assert!(matches!(h.accounts, Body::Filled(_)));
    assert!(matches!(h.next, Body::Filled(_)));

    let mut d = base.clone();
    d.groups = Reading::Unknown;
    d.accounts = Reading::Unknown;
    d.moves = Reading::Unknown;
    let h = home::home(&d);
    assert_eq!(h.groups, Body::Unmeasured(GROUPS_UNREAD));
    assert_eq!(h.accounts, Body::Unmeasured(ACCOUNTS_UNREAD));
    assert_eq!(h.moves, Body::Unmeasured(MOVES_UNREAD));
    assert!(matches!(h.next, Body::Filled(_)));

    // 理由は 1 行で重ならず、0 件の字とも重ならない。
    let lines = [
        NOT_READ,
        account::UNREAD,
        account::BAD_BODY,
        NO_CONTENT,
        GROUPS_UNREAD,
        ACCOUNTS_UNREAD,
        MOVES_UNREAD,
        NO_PROJECTS,
        NO_GROUPS,
        NO_ACCOUNTS,
        NO_MOVES,
    ];
    for (i, a) in lines.iter().enumerate() {
        assert!(!a.trim().is_empty() && !a.contains('\n'), "{a}");
        for b in &lines[i + 1..] {
            assert_ne!(a, b, "字が重なる");
        }
    }
}

/// (7) 面は使った割合と閾値を比べない（比べの字と出さない列の字が source に無い・R-22）。
#[test]
fn accthome_no_pressure_formula() {
    let src = read("src/account/home.rs");
    for word in [
        ">= cap",
        "> cap",
        "<= used",
        "< used",
        "used_pct >",
        "used_pct <",
        "used >",
        "used <",
        "c-occ",
        "remaining",
    ] {
        assert!(!src.contains(word), "home.rs に {word}");
    }
}

/// (8) 使う語の鍵は語の辞書に在り、class は stylesheet に在る。外の依存を足さない。着地済みの枠は変えない。
#[test]
fn accthome_vocab_css_deps_and_frame() {
    let mut keys: Vec<&str> = vec![
        "nx_a",
        "nx_b",
        "nx_c",
        "nx_d",
        "nx_e",
        "nx_f",
        "nx_g",
        "group",
        "current_account",
        "since_rec",
        "previous",
        "pressure",
        "members",
        "candidates",
        "next_target",
        home::NO_TARGET_KEY,
        home::NO_RECORD_KEY,
        home::FREE_KEY,
        home::RETIRED_KEY,
        home::UNKNOWN_KEY,
        "not_yet",
        "open_new",
    ];
    keys.extend(ACCT_HEADS);
    for key in keys {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が語の辞書に無い"));
        assert!(!term.label.is_empty(), "{key}");
    }
    for r in filled(fixture_home().next) {
        assert!(vocab().term(r.key).is_some(), "{}", r.key);
    }

    let css = read("style.css");
    let mut classes: Vec<&str> = vec![
        "nxm on", "nxm off", "nxm na", "nxrow dim", "occ grp", "occ free", "gi ok", "gi ng",
        "gi unknown",
    ];
    let src = read("src/account/home.rs");
    let mut rest = src.as_str();
    while let Some(i) = rest.find("class=\"") {
        rest = &rest[i + 7..];
        let end = rest.find('"').expect("class の字の終わり");
        classes.push(&rest[..end]);
        rest = &rest[end..];
    }
    for class in [
        "nxall", "nxp", "nxtop", "nxk", "nxl", "nxms", "gcard", "gcur", "gpw", "pws", "pw",
        "projs", "pchip", "acct-grid", "c-name", "aname", "c-w", "meter", "mv", "fold",
    ] {
        assert!(
            src.contains(&format!("\"{class}\"")) || src.contains(&format!("\"{class} ")),
            "home.rs が class {class} を使わない"
        );
    }
    for class in classes.iter().flat_map(|c| c.split_whitespace()) {
        assert!(
            css.contains(&format!(".{class}")),
            "stylesheet に class {class} が無い"
        );
    }

    let manifest = read("Cargo.toml");
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|s| s.split("[target.").next())
        .expect("dependencies の節");
    let names: Vec<&str> = deps
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split('=').next())
        .map(str::trim)
        .collect();
    assert_eq!(names, vec!["tsuzuri-contract"]);

    // 枠の定数と body は便 h-frame のまま・board は home の view に block を渡す。
    assert_eq!(
        home::BLOCKS.map(|b| (b.id, b.heading, b.class)),
        [
            ("nxall", "next_all", "panel"),
            ("groups", "group", "gtop"),
            ("allowance", "allowance", "panel"),
            ("moves", "moves", "panel fold mvp")
        ]
    );
    assert_eq!(
        home::body(&Fetched::Body(fixture_text())),
        Body::Unmeasured(NO_CONTENT)
    );
    assert!(read("src/account/board.rs").contains("home::view(block)"));
    assert!(src.contains("pub fn view(block: Block) -> leptos::prelude::AnyView"));
    // 口の読みは mod.rs の PATH を net の read に渡し、開く button は windows の module の開く関数を呼ぶ。
    assert!(src.contains("crate::net::read(PATH)"));
    assert!(src.contains("open(&p.name, &u)"));
}
