//! 便 h-sess の歯: session の表の並べ方（query の sort）・束の見出し・fixture の 4 行の順・稼働の記録・
//! orchestrator の行の合図（tick の健康・heartbeat・退避までの残り秒・移動待ち）・席なしの行・読めないときは測れていない。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::{Reading, Stage};
use tsuzuri_contract::seat::{SeatSpan, SeatState};
use tsuzuri_contract::surface::SeatRole;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::session::{
    self, BANDS, COLUMNS, Head, Move, NO_ROWS, STALLED_STAGES, Sort, band_key, content, grace_text,
    hb_class, moving, rank, sort_of, strip, table, tick_class, with_sort, without_param,
};
use tsuzuri_surface::account::{self, BAD_BODY, UNREAD};
use tsuzuri_surface::project::seat::{
    NG, OK, SPAN_KEY, Span, rects, span_of, strip_svg, with_span,
};
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

/// 行の順（fixture の行の番号・1 から）。
fn order(doc: &AccountDoc, sort: Sort) -> Vec<usize> {
    table(doc, sort)
        .order()
        .into_iter()
        .map(|i| i + 1)
        .collect()
}

/// (1) 並べ方は 4 つで、query の sort から決め（ほかの字と無いときは project）、書く関数は project のときに sort を消す。
#[test]
fn acctsess_sort_from_query_and_written_back() {
    let keys: Vec<&str> = Sort::ALL.iter().map(|s| s.key()).collect();
    assert_eq!(keys, vec!["project", "account", "stage", "elapsed"]);
    let labels: Vec<&str> = Sort::ALL.iter().map(|s| s.label_key()).collect();
    assert_eq!(
        labels,
        vec!["sort_project", "sort_account", "sort_stage", "sort_elapsed"]
    );
    assert_eq!(session::SORT_KEY, "sort");
    for (q, want) in [
        ("?sort=project", Sort::Project),
        ("?sort=account", Sort::Account),
        ("?board=account&tab=session&sort=stage", Sort::Stage),
        ("sort=elapsed&mode=expert", Sort::Elapsed),
        ("", Sort::Project),
        ("?", Sort::Project),
        ("?sort", Sort::Project),
        ("?sort=", Sort::Project),
        ("?sort=Stage", Sort::Project),
        ("?sort=need", Sort::Project),
        ("?lsort=account", Sort::Project),
    ] {
        assert_eq!(sort_of(q), want, "{q}");
    }

    let q = "?board=account&tab=session&mode=expert";
    assert_eq!(
        with_sort(q, Sort::Stage),
        "?board=account&tab=session&mode=expert&sort=stage"
    );
    assert_eq!(
        with_sort("?board=account&sort=stage&tab=session", Sort::Elapsed),
        "?board=account&sort=elapsed&tab=session"
    );
    assert_eq!(
        with_sort("?board=account&sort=stage&tab=session", Sort::Project),
        "?board=account&tab=session"
    );
    assert_eq!(with_sort(q, Sort::Project), q);
    assert_eq!(with_sort("?sort=account", Sort::Project), "?");
    for sort in Sort::ALL {
        let written = with_sort(q, sort);
        assert_eq!(sort_of(&written), sort, "{written}");
        assert_eq!(
            written.contains("sort="),
            sort != Sort::Project,
            "{written}"
        );
        // ほかの鍵は残る。
        for kv in ["board=account", "tab=session", "mode=expert"] {
            assert!(written.contains(kv), "{written}");
        }
    }
    // 消す関数は同じ鍵を全部消し、似た名の鍵とほかの値の順は残す。
    assert_eq!(
        without_param("?sort=a&lsort=b&sort=c&x=1", "sort"),
        "?lsort=b&x=1"
    );
    assert_eq!(without_param("", "sort"), "?");
    assert_eq!(without_param("?sort", "sort"), "?");
}

/// (2) fixture の 4 行の順が節の期待と一致する（project 1・2・3・4、account 1・3・2・4、stage 3・1・2・4、elapsed 1・2・3・4）。
#[test]
fn acctsess_orders_on_fixture() {
    let doc = fixture();
    let names: Vec<&str> = doc.sessions.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "proj-a-orch",
            "proj-a.3-20260927T113000Z",
            "proj-b-orch",
            "proj-c-orch"
        ]
    );
    assert_eq!(order(&doc, Sort::Project), vec![1, 2, 3, 4]);
    assert_eq!(order(&doc, Sort::Account), vec![1, 3, 2, 4]);
    assert_eq!(order(&doc, Sort::Stage), vec![3, 1, 2, 4]);
    assert_eq!(order(&doc, Sort::Elapsed), vec![1, 2, 3, 4]);

    let heads = |sort: Sort| -> Vec<(Option<Head>, usize)> {
        table(&doc, sort)
            .groups
            .into_iter()
            .map(|g| (g.head, g.rows.len()))
            .collect()
    };
    let project = |name: &str, group: &str| {
        Some(Head::Project {
            name: name.to_string(),
            group: Some(group.to_string()),
        })
    };
    assert_eq!(
        heads(Sort::Project),
        vec![
            (project("proj-a", "Tier1"), 2),
            (project("proj-b", "Tier2"), 1),
            (project("proj-c", "Tier1"), 1)
        ]
    );
    // acct-2 と acct-3 は行が無いので見出しを出さない・口座の無い行は末尾。
    assert_eq!(
        heads(Sort::Account),
        vec![
            (
                Some(Head::Account {
                    label: "acct-1".to_string(),
                    occupant: Reading::Known(Some("Tier1".to_string()))
                }),
                3
            ),
            (Some(Head::NoAccount), 1)
        ]
    );
    // 止まっている束は行が無いので出さない。
    assert_eq!(
        heads(Sort::Stage),
        vec![
            (Some(Head::Band("moving_group")), 3),
            (Some(Head::Band("no_record")), 1)
        ]
    );
    assert_eq!(heads(Sort::Elapsed), vec![(None, 4)]);
    for sort in Sort::ALL {
        let t = table(&doc, sort);
        assert_eq!(t.sort, sort);
        assert_eq!(t.at, doc.at);
    }
}

/// (2) 並べの決まり: 見出しの中は orchestrator が先で名の順・口座の見出しは占有の群か free_for_pipeline・
/// stage は位で 3 つの束（止まった run を含む）で束の中は位と since の順・elapsed は since の無い行が末尾・同じ値は電文の順。
#[test]
fn acctsess_group_rules() {
    let base = fixture();

    // project: 見出しの中は orchestrator が先・ほかは name の字の順（同じ名は電文の順）。
    let mut d = base.clone();
    let mut p2 = d.sessions[1].clone();
    p2.name = "proj-a.1-20260927T100000Z".to_string();
    let mut p3 = d.sessions[1].clone();
    p3.name = "proj-a.1-20260927T100000Z".to_string();
    p3.since = Some(1);
    d.sessions.insert(0, p2); // 行 1
    d.sessions.push(p3); // 行 6
    // 行 1 = pipeline .1・行 2 = orch・行 3 = pipeline .3・行 4 = proj-b・行 5 = proj-c・行 6 = pipeline .1
    assert_eq!(order(&d, Sort::Project), vec![2, 1, 6, 3, 4, 5]);

    // projects に無い project の行はその後に（群は無し）・projects の順が見出しの順。
    let mut d = base.clone();
    d.projects.reverse();
    let mut extra = base.sessions[0].clone();
    extra.project = "proj-z".to_string();
    d.sessions.insert(0, extra);
    assert_eq!(order(&d, Sort::Project), vec![5, 4, 2, 3, 1]);
    let last = table(&d, Sort::Project).groups.pop().expect("見出し");
    assert_eq!(
        last.head,
        Some(Head::Project {
            name: "proj-z".to_string(),
            group: None
        })
    );

    // account: 電文の accounts の順・占有の無い口座は None（free_for_pipeline）・中は orchestrator が先で project の名の順。
    let mut d = base.clone();
    d.sessions[0].account = Some("acct-2".to_string());
    d.sessions[1].account = Some("acct-3".to_string());
    d.sessions[2].account = Some("acct-3".to_string());
    let mut pipe_b = base.sessions[1].clone();
    pipe_b.project = "proj-b".to_string();
    pipe_b.account = Some("acct-3".to_string());
    let mut pipe_a = base.sessions[1].clone();
    pipe_a.account = Some("acct-3".to_string());
    d.sessions.insert(0, pipe_b); // 行 1
    d.sessions.push(pipe_a); // 行 6
    // 行 1 = pipe proj-b acct-3・行 2 = orch a acct-2・行 3 = pipe a acct-3・行 4 = orch b acct-3・行 5 = orch c 無し・行 6 = pipe a acct-3
    assert_eq!(order(&d, Sort::Account), vec![2, 4, 3, 6, 1, 5]);
    let heads: Vec<Option<Head>> = table(&d, Sort::Account)
        .groups
        .into_iter()
        .map(|g| g.head)
        .collect();
    assert_eq!(
        heads,
        vec![
            Some(Head::Account {
                label: "acct-2".to_string(),
                occupant: Reading::Known(Some("Tier2".to_string()))
            }),
            Some(Head::Account {
                label: "acct-3".to_string(),
                occupant: Reading::Known(None)
            }),
            Some(Head::NoAccount)
        ]
    );
    // 電文の accounts に無い口座（accounts が読めない）の行も落とさず、占有は測れていない。
    let mut d = base.clone();
    d.accounts = Reading::Unknown;
    let t = table(&d, Sort::Account);
    assert_eq!(t.order(), vec![0, 2, 1, 3]);
    assert_eq!(
        t.groups[0].head,
        Some(Head::Account {
            label: "acct-1".to_string(),
            occupant: Reading::Unknown
        })
    );

    // stage: 位の決まり。
    let line = |state: SeatState, role: SeatRole, stage: Option<Stage>| {
        let mut l = base.sessions[1].clone();
        l.state = state;
        l.role = role;
        l.stage = stage;
        l
    };
    assert_eq!(
        STALLED_STAGES,
        [Stage::Questioned, Stage::Failed, Stage::Stopped]
    );
    for stage in Stage::ALL {
        let stalled = STALLED_STAGES.contains(&stage);
        for state in [SeatState::Run, SeatState::Wait, SeatState::Unknown] {
            let want = if stalled {
                2
            } else {
                match state {
                    SeatState::Wait => 3,
                    SeatState::Run => 4,
                    _ => 5,
                }
            };
            assert_eq!(
                rank(&line(state, SeatRole::Pipeline, Some(stage))),
                want,
                "{state:?} {stage:?}"
            );
            // orchestrator の行は段を見ない。
            assert_eq!(
                rank(&line(state, SeatRole::Orchestrator, Some(stage))),
                match state {
                    SeatState::Wait => 3,
                    SeatState::Run => 4,
                    _ => 5,
                }
            );
        }
        assert_eq!(
            rank(&line(SeatState::Limit, SeatRole::Pipeline, Some(stage))),
            0
        );
        assert_eq!(
            rank(&line(SeatState::Silent, SeatRole::Pipeline, Some(stage))),
            1
        );
    }
    assert_eq!(rank(&line(SeatState::Run, SeatRole::Pipeline, None)), 4);
    let bands: Vec<&str> = (0..=5).map(band_key).collect();
    assert_eq!(
        bands,
        vec![
            "stopped_group",
            "stopped_group",
            "stopped_group",
            "moving_group",
            "moving_group",
            "no_record"
        ]
    );
    assert_eq!(BANDS, ["stopped_group", "moving_group", "no_record"]);

    // stage: 束の中は位の順・同じ位は since の古い順（since の無い行は末尾）・同じ値は電文の順。
    let mut d = base.clone();
    let at = |since: Option<u64>, mut l: tsuzuri_contract::account::SessionLine| {
        l.since = since;
        l
    };
    d.sessions = vec![
        at(
            Some(50),
            line(SeatState::Run, SeatRole::Pipeline, Some(Stage::Failed)),
        ), // 1: 位 2
        at(
            Some(90),
            line(SeatState::Silent, SeatRole::Orchestrator, None),
        ), // 2: 位 1
        at(None, line(SeatState::Limit, SeatRole::Pipeline, None)), // 3: 位 0
        at(
            Some(10),
            line(SeatState::Limit, SeatRole::Orchestrator, None),
        ), // 4: 位 0
        at(Some(20), line(SeatState::Run, SeatRole::Orchestrator, None)), // 5: 位 4
        at(
            Some(30),
            line(SeatState::Wait, SeatRole::Orchestrator, None),
        ), // 6: 位 3
        at(None, line(SeatState::Unknown, SeatRole::Orchestrator, None)), // 7: 位 5
        at(
            Some(5),
            line(SeatState::Unknown, SeatRole::Orchestrator, None),
        ), // 8: 位 5
        at(Some(20), line(SeatState::Run, SeatRole::Pipeline, None)), // 9: 位 4
    ];
    assert_eq!(order(&d, Sort::Stage), vec![4, 3, 2, 1, 6, 5, 9, 8, 7]);
    let sizes: Vec<(Option<Head>, usize)> = table(&d, Sort::Stage)
        .groups
        .into_iter()
        .map(|g| (g.head, g.rows.len()))
        .collect();
    assert_eq!(
        sizes,
        vec![
            (Some(Head::Band("stopped_group")), 4),
            (Some(Head::Band("moving_group")), 3),
            (Some(Head::Band("no_record")), 2)
        ]
    );
    // elapsed: since の古い順・since の無い行は末尾・同じ値は電文の順。
    assert_eq!(order(&d, Sort::Elapsed), vec![8, 4, 5, 9, 6, 1, 2, 3, 7]);

    // 行が 0 の電文は測れて 0 件（見出しも束も出さない）。
    let mut d = base.clone();
    d.sessions.clear();
    for sort in Sort::ALL {
        assert!(table(&d, sort).groups.is_empty());
    }
}

/// (3) 稼働の記録は seat の module の幅と矩形と SVG で組み、窓の右端は電文の at・縦線は無い・区間が読めなければ測れていない。
#[test]
fn acctsess_strip_from_seat_module() {
    let doc = fixture();
    assert_eq!(SPAN_KEY, "span");
    assert_eq!(span_of("?board=account&tab=session&span=6h"), Span::H6);
    assert_eq!(span_of("?sort=stage"), Span::H24);
    assert_eq!(
        with_span("?board=account&sort=stage", Span::H3),
        "?board=account&sort=stage&span=3h"
    );

    let t = table(&doc, Sort::Project);
    let rows: Vec<_> = t.groups.iter().flat_map(|g| g.rows.iter()).collect();
    for span in Span::ALL {
        let Reading::Known(spans) = &doc.sessions[0].spans else {
            panic!("行 1 の区間が読める");
        };
        let want = strip_svg(&rects(spans, doc.at, span), &[]);
        assert_eq!(
            strip(&rows[0].spans, t.at, span),
            Reading::Known(want.clone())
        );
        assert!(!want.contains("mk-acct"), "縦線は出さない");
        assert!(want.contains("mk-now"));
        // 行 2 と行 4 は区間が読めない。
        assert_eq!(strip(&rows[1].spans, t.at, span), Reading::Unknown);
        assert_eq!(strip(&rows[3].spans, t.at, span), Reading::Unknown);
    }
    // 窓の右端は電文の at: 右端で終わる区間は幅の右端まで届く。
    let spans = vec![SeatSpan {
        from: doc.at - 3_600,
        to: doc.at,
        state: SeatState::Run,
    }];
    let r = rects(&spans, doc.at, Span::H3);
    assert_eq!(r.len(), 1);
    assert!((r[0].x + r[0].width - 288.0).abs() < 1e-9);
    assert_eq!(
        strip(&Reading::Known(spans.clone()), doc.at, Span::H3),
        Reading::Known(strip_svg(&r, &[]))
    );
    // at を 1 時間進めると同じ区間は左へ寄る（右端は電文の at）。
    let later = rects(&spans, doc.at + 3_600, Span::H3);
    assert!(later[0].x + later[0].width < 288.0 - 1.0);

    // 経過の字は電文の at からの差（since が無ければ「―」）。
    let els: Vec<&str> = rows.iter().map(|r| r.elapsed.as_str()).collect();
    assert_eq!(els, vec!["1h", "30m", "25m", session::NONE_MARK]);
}

/// (4) orchestrator の行の合図: 同じ project の席の card の tick と heartbeat・残り秒は mvgrace・残り秒の無い行の不一致は mvwait。
#[test]
fn acctsess_signals_on_fixture() {
    let doc = fixture();
    let t = table(&doc, Sort::Project);
    let rows: Vec<_> = t.groups.iter().flat_map(|g| g.rows.iter()).collect();
    let (a, pipe, b, c) = (rows[0], rows[1], rows[2], rows[3]);

    let sa = a.signs.as_ref().expect("proj-a の合図");
    assert_eq!(sa.tick, Reading::Known(OK));
    assert_eq!(sa.heartbeat, Reading::Known("on"));
    assert_eq!(tick_class(&sa.tick), "tk tk-healthy");
    assert_eq!(hb_class(&sa.heartbeat), "hb");
    assert_eq!(a.moving, None);

    let sb = b.signs.as_ref().expect("proj-b の合図");
    assert_eq!(sb.tick, Reading::Known(NG));
    assert_eq!(OK.glyph, "✓");
    assert_eq!(NG.glyph, "!");
    assert_eq!(sb.heartbeat, Reading::Known("off"));
    assert_eq!(tick_class(&sb.tick), "tk tk-stale");
    assert_eq!(hb_class(&sb.heartbeat), "hb hb-off");
    // proj-b は席の口座 acct-1 が群の今の口座 acct-2 と違うが、残り秒が在るので mvwait は出さない。
    assert_eq!(b.moving, Some(Move::Grace(1_790_511_501)));
    let m = b.moving.expect("残り秒");
    assert_eq!((m.class(), m.key()), ("mvgrace", "move_grace"));
    assert_eq!(grace_text(1101), "1101 秒");

    let sc = c.signs.as_ref().expect("proj-c の合図");
    assert_eq!(sc.tick, Reading::Unknown);
    assert_eq!(sc.heartbeat, Reading::Unknown);
    assert_eq!(tick_class(&sc.tick), "tk");
    assert_eq!(hb_class(&sc.heartbeat), "hb");
    assert_eq!(c.moving, None);

    // pipeline の行は合図を持たない。
    assert_eq!(pipe.signs, None);
    assert_eq!(pipe.moving, None);
    assert_eq!(pipe.stage, Some("Running"));
    assert_eq!(a.stage, None);

    // 行の値（class・役の語の鍵・状態）。
    let got: Vec<(&str, &str, &str)> = rows
        .iter()
        .map(|r| (r.class.as_str(), r.role_key, r.state))
        .collect();
    assert_eq!(
        got,
        vec![
            ("srow r-orchestrator", "role:orchestrator", "run"),
            ("srow r-pipeline", "role:pipeline", "run"),
            ("srow r-orchestrator", "role:orchestrator", "wait"),
            ("srow r-orchestrator", "role:orchestrator", "unknown")
        ]
    );
    assert_eq!(c.account, None);
    assert_eq!(a.account.as_deref(), Some("acct-1"));
}

/// (4) mvwait の決まり: 残り秒の無い行で、行の口座と群の今の口座が両方分かって違うときだけ。
#[test]
fn acctsess_move_wait_rules() {
    let base = fixture();
    let b = |d: &AccountDoc| moving(d, &d.sessions[2]);

    // 残り秒を消すと、acct-1 ≠ acct-2 で移動待ち。
    let mut d = base.clone();
    d.projects[1].move_until = None;
    assert_eq!(b(&d), Some(Move::Wait));
    assert_eq!(
        (Move::Wait.class(), Move::Wait.key()),
        ("mvwait", "seat_mismatch")
    );
    // 行の口座が群の今の口座と同じなら出さない。
    let mut same = d.clone();
    same.sessions[2].account = Some("acct-2".to_string());
    assert_eq!(b(&same), None);
    // 行の口座が無い・groups が読めない・群が無い・群の名が groups に無い行は出さない。
    let mut x = d.clone();
    x.sessions[2].account = None;
    assert_eq!(b(&x), None);
    let mut x = d.clone();
    x.groups = Reading::Unknown;
    assert_eq!(b(&x), None);
    let mut x = d.clone();
    x.projects[1].group = None;
    assert_eq!(b(&x), None);
    let mut x = d.clone();
    x.projects[1].group = Some("Tier9".to_string());
    assert_eq!(b(&x), None);
    // projects に無い project の行は合図を測れない（移動の印も出さない）。
    let mut x = d.clone();
    x.sessions[2].project = "proj-z".to_string();
    assert_eq!(b(&x), None);
    let r = session::row(&x, 2);
    let s = r.signs.expect("orchestrator の行");
    assert_eq!((s.tick, s.heartbeat), (Reading::Unknown, Reading::Unknown));

    // 残り秒は mvwait より先（一致しても残り秒を出す）。
    let mut g = base.clone();
    g.sessions[2].account = Some("acct-2".to_string());
    assert_eq!(b(&g), Some(Move::Grace(1_790_511_501)));
    // pipeline の行は移動の印を出さない。
    let mut p = d.clone();
    p.sessions[2].role = SeatRole::Pipeline;
    assert_eq!(session::row(&p, 2).moving, None);
    assert_eq!(session::row(&p, 2).signs, None);
}

/// (5) 席が無い project の行（name が空の字の orchestrator の行）は session の欄に語の鍵 seat_none を出す。
#[test]
fn acctsess_seat_none_row() {
    let mut d = fixture();
    d.sessions[3].name = String::new();
    let r = session::row(&d, 3);
    assert_eq!(r.session, None);
    assert_eq!(r.role_key, "role:orchestrator");
    let named = session::row(&d, 0);
    assert_eq!(named.session.as_deref(), Some("proj-a-orch"));
    // DOM は None の欄に語の鍵 seat_none を出す。
    let src = read("src/account/session.rs");
    assert!(src.contains("term(\"seat_none\", label(\"seat_none\"))"));
    assert!(vocab().term("seat_none").is_some());
}

/// (6) 口が読めない・まだ読んでいない・本文が電文として読めないときは、表の全体が測れていないと理由の 1 行。
#[test]
fn acctsess_unmeasured_until_read() {
    let cases = [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, UNREAD),
        (Fetched::Body("{}".to_string()), BAD_BODY),
        (Fetched::Body("not json".to_string()), BAD_BODY),
        (Fetched::Body(String::new()), BAD_BODY),
    ];
    for (fetched, want) in cases {
        for sort in Sort::ALL {
            assert_eq!(
                content(&fetched, sort, 1_790_510_400),
                Body::Unmeasured(want),
                "{fetched:?}"
            );
        }
    }
    let doc = fixture();
    let Body::Filled(t) = content(&Fetched::Body(fixture_text()), Sort::Account, doc.at) else {
        panic!("fixture は中身あり");
    };
    assert_eq!(t, table(&doc, Sort::Account));
    // 行が 0 の電文は測れて 0 件（測れていないと分ける）。
    let mut empty = doc.clone();
    empty.sessions.clear();
    let text = wire::encode(&empty).expect("電文");
    assert_eq!(
        content(&Fetched::Body(text), Sort::Project, doc.at),
        Body::Empty(NO_ROWS)
    );
    assert!(!NO_ROWS.contains('\n'));
    // 枠の body は便 h-frame のまま。
    assert_eq!(
        session::body(&Fetched::Body(fixture_text())),
        Body::Unmeasured(NO_CONTENT)
    );
    // 口は account の mod.rs の PATH を net の read に渡す。
    let src = read("src/account/session.rs");
    assert!(src.contains("crate::net::read(PATH)"));
    assert!(src.contains("use crate::account::PATH;"));
    assert_eq!(account::PATH, "/api/account");
}

/// (7) 語の鍵は語の辞書に在り、class は stylesheet に在り、面の crate の依存は契約の型の crate の 1 本のまま。
#[test]
fn acctsess_vocab_style_and_deps() {
    let mut keys: Vec<&str> = COLUMNS.to_vec();
    keys.extend(Sort::ALL.iter().map(|s| s.label_key()));
    keys.extend(BANDS);
    keys.extend([
        "sessions",
        "span",
        "sort_by",
        "seat_none",
        "tick_health",
        "heartbeat",
        "move_grace",
        "seat_mismatch",
        "free_for_pipeline",
        "st_unknown",
        "role:orchestrator",
        "role:pipeline",
    ]);
    for key in keys {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }

    let css = read("style.css");
    for class in [
        session::SESS,
        session::HROW,
        session::GHEAD,
        session::C_PROJ,
        session::C_ROLE,
        session::C_STAGE,
        session::C_STRIP,
        session::TKHB,
        "mvgrace",
        "mvwait",
        "sortbar",
        "seg",
        "r-orchestrator",
        "r-pipeline",
        "tk-healthy",
        "tk-stale",
        "hb-off",
    ] {
        for c in class.split_whitespace() {
            assert!(
                css.contains(&format!(".{c}")),
                "stylesheet に class {c} が無い"
            );
        }
    }
    // stylesheet に規則の無い列の class は付けない。
    let src = read("src/account/session.rs");
    for c in ["c-ses", "c-acct", "c-el"] {
        assert!(!src.contains(c), "session.rs に {c} が在る");
    }

    // session の止まった run（見本の runStopped）は、中核の crate の next_step の止まっている走行の段に
    // Questioned を足した段（行 c-next-stall が次の一手の段から Questioned を除いた）。
    let core = read("../tsuzuri-core/src/next_step.rs");
    assert!(
        core.contains("pub const STALLED_STAGES: [Stage; 2] = [Stage::Failed, Stage::Stopped];")
    );

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
