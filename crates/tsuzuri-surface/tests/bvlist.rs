//! 行 g-list-groups の歯（接頭辞 bvlist_）: 台帳 open の一覧を epic ごとの組にし、組の頭に進みの数と板で動いている札の
//! 列ごとの数を持たせ、既定で開く組を決め、行を 1 行の短い題と右の字にする（判断の記録 ADR-27 の決定 (8)）。
//! 台帳の行と札と bead の事実は歯の中で組む（fixture の file は使わない・bead の id の接頭辞は fx-g）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::{PipelineBoard, PipelineCard, Reading, Stage};
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::{
    BeadFact, BeadFacts, BeadId, LedgerList, LedgerRow, MEMO_LABEL, QUESTION_LABEL,
};
use tsuzuri_contract::wire;
use tsuzuri_surface::ledgerlist::{
    COUNTS_UNKNOWN, CREATED_KEY, FACTS_PATH, KIND_TAGS, Lgroup, OUTSIDE_KEY, content, facts,
    groups, head_counts, home, kind_tag, row_rank,
};
use tsuzuri_surface::project::ledger::{EMPTY, NO_OPEN, OUTSIDE};
use tsuzuri_surface::project::pipeline::{age_at, stage_word};
use tsuzuri_surface::project::{Body, LEDGER_UNREAD};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::label;
use tsuzuri_surface::widgets::pop::board_unread;

/// 一覧を組む今。
const NOW: u64 = 1_790_510_400;

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn row(id: &str, kind: &str, status: &str, labels: &[&str]) -> LedgerRow {
    LedgerRow {
        id: BeadId::new(id).expect("id"),
        kind: kind.to_string(),
        title: format!("{id} の題 — 長い説明"),
        status: status.to_string(),
        updated_at: NOW - 600,
        parent: None,
        labels: labels.iter().map(|l| (*l).to_string()).collect(),
    }
}

fn card(id: &str, stage: Stage) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new(id).expect("id"),
        runs: 1,
        stage,
        reason: None,
        account: None,
        since: Some(NOW - 120),
        ci: None,
    }
}

fn fact(id: &str, short: &str, created: Option<u64>) -> BeadFact {
    BeadFact {
        id: BeadId::new(id).expect("id"),
        created_at: created,
        short: short.to_string(),
        short_set: true,
        summary: String::new(),
        blocks: Vec::new(),
    }
}

/// 台帳: 動く epic g1（便 3・memo・問い・閉じた便）・入れ子の epic g2 の下の便・open の行の無い epic g3・
/// 閉じた epic g4（閉じた子だけ）・閉じた epic g5（子は閉じたが札が走る）・epic の外の便と閉じた memo。
fn rows() -> Vec<LedgerRow> {
    vec![
        row("fx-g1", "epic", "open", &[]),
        row("fx-g1.1", "task", "open", &[]),
        row("fx-g1.2", "task", "closed", &[]),
        row("fx-g1.3", "task", "in_progress", &[]),
        row("fx-g1.4", "task", "open", &[MEMO_LABEL]),
        row("fx-g1.5", "task", "open", &[QUESTION_LABEL]),
        row("fx-g1.6", "task", "open", &[]),
        row("fx-g1.7", "epic", "open", &[]),
        row("fx-g1.7.1", "task", "open", &[]),
        row("fx-g3", "epic", "open", &[]),
        row("fx-g3.1", "task", "closed", &[]),
        row("fx-g4", "epic", "closed", &[]),
        row("fx-g4.1", "task", "closed", &[]),
        row("fx-g5", "epic", "closed", &[]),
        row("fx-g5.1", "task", "closed", &[]),
        row("fx-x9", "task", "open", &[]),
        row("fx-x8", "task", "closed", &[MEMO_LABEL]),
    ]
}

/// 札: g1 に止まり・Gated・Queued・Blocked・着地、g1.7 に走り、g5 に走り（台帳では閉じた子）・外の便に Queued。
fn cards() -> Vec<PipelineCard> {
    vec![
        card("fx-g1.1", Stage::Failed),
        card("fx-g1.3", Stage::Gated),
        card("fx-g1.6", Stage::Queued),
        card("fx-g1.9", Stage::Blocked),
        card("fx-g1.2", Stage::Landed),
        card("fx-g1.7.1", Stage::Running),
        card("fx-g5.1", Stage::Running),
        card("fx-x9", Stage::Questioned),
    ]
}

fn facts_map() -> BTreeMap<String, BeadFact> {
    [
        fact("fx-g1", "動く epic", Some(NOW - 86_400)),
        fact("fx-g1.4", "memo の短い題", Some(NOW - 7_200)),
        fact("fx-g1.5", "問いの短い題", None),
    ]
    .into_iter()
    .map(|f| (f.id.to_string(), f))
    .collect()
}

fn keys(gs: &[Lgroup]) -> Vec<&str> {
    gs.iter().map(|g| g.key.as_str()).collect()
}

fn group<'a>(gs: &'a [Lgroup], key: &str) -> &'a Lgroup {
    gs.iter()
        .find(|g| g.key == key)
        .unwrap_or_else(|| panic!("組 {key} が無い"))
}

/// (2) 組は id の階層のいちばん近い epic の祖先ごと（入れ子の epic は自分の組・memo と問いも入る）で、閉じた epic は
/// open の行も動いている札も無ければ出さない。並べは動いている札の多い組・open の行を持つ組・ほかの順（同じ位は
/// epic の id の順）で、epic の外の組は最後。
#[test]
fn bvlist_groups_by_epic_outside_last() {
    let gs = groups(&rows(), &cards(), &facts_map(), NOW);
    assert_eq!(
        keys(&gs),
        ["fx-g1", "fx-g1.7", "fx-g5", "fx-g3", OUTSIDE_KEY]
    );
    let ids =
        |key: &str| -> Vec<String> { group(&gs, key).rows.iter().map(|r| r.id.clone()).collect() };
    assert_eq!(
        ids("fx-g1"),
        ["fx-g1.1", "fx-g1.3", "fx-g1.6", "fx-g1.5", "fx-g1.4"]
    );
    assert_eq!(ids("fx-g1.7"), ["fx-g1.7.1"]);
    assert_eq!(ids(OUTSIDE_KEY), ["fx-x9"]);
    assert!(ids("fx-g3").is_empty() && ids("fx-g5").is_empty());
    let out = group(&gs, OUTSIDE_KEY);
    assert_eq!((out.epic.as_deref(), out.name.as_str()), (None, OUTSIDE));
    // 祖先の epic: 近い順にたどり、自分は数えない。
    let all = rows();
    let epics: Vec<&LedgerRow> = all
        .iter()
        .filter(|r| r.node_kind() == NodeKind::Epic)
        .collect();
    assert_eq!(
        home("fx-g1.7.1", &epics).map(|e| e.id.as_str()),
        Some("fx-g1.7")
    );
    assert_eq!(
        home("fx-g1.7", &epics).map(|e| e.id.as_str()),
        Some("fx-g1")
    );
    assert_eq!(home("fx-g1", &epics), None);
    // 親が epic でなければ祖父母へたどる（memo の下の問いは memo の epic の組）。
    assert_eq!(
        home("fx-g1.4.1", &epics).map(|e| e.id.as_str()),
        Some("fx-g1")
    );
    // 外の組の行が全部閉じていれば外の組は出さない。
    let closed_out: Vec<LedgerRow> = all
        .iter()
        .filter(|r| r.id.as_str() != "fx-x9")
        .cloned()
        .collect();
    assert!(!keys(&groups(&closed_out, &cards(), &facts_map(), NOW)).contains(&OUTSIDE_KEY));
    order_case();
}

/// 並べの 3 つの位と同じ位の順（歯 bvlist_groups_by_epic_outside_last が呼ぶ）: 札の多い組が先（id の順と逆）・札の数が
/// 同じ組は epic の id の順（台帳の並びが逆でも）・open の行だけの組（閉じた epic でも出す）は札の在る組の後で、行も札も
/// 無い組の前。
fn order_case() {
    let order = vec![
        row("fx-g9", "epic", "open", &[]),
        row("fx-g9.1", "task", "open", &[]),
        row("fx-g9.2", "task", "open", &[]),
        row("fx-g8", "epic", "closed", &[]),
        row("fx-g8.1", "task", "open", &[]),
        row("fx-g7", "epic", "open", &[]),
        row("fx-g7.1", "task", "open", &[]),
        row("fx-g6", "epic", "open", &[]),
        row("fx-g6.1", "task", "open", &[]),
        row("fx-g2", "epic", "open", &[]),
    ];
    let order_cards = [
        card("fx-g9.1", Stage::Running),
        card("fx-g9.2", Stage::Queued),
        card("fx-g7.1", Stage::Running),
        card("fx-g6.1", Stage::Blocked),
    ];
    assert_eq!(
        keys(&groups(&order, &order_cards, &BTreeMap::new(), NOW)),
        ["fx-g9", "fx-g6", "fx-g7", "fx-g8", "fx-g2"]
    );
}

/// (3) 既定で開く組は板で動いている札を持つ組か open の行を持つ組だけ（open の epic で行も札も無い組は畳む・
/// 閉じた epic でも札が走れば開く）。
#[test]
fn bvlist_default_open_live_or_open_kids() {
    let gs = groups(&rows(), &cards(), &facts_map(), NOW);
    let open: Vec<(&str, bool)> = gs
        .iter()
        .map(|g| (g.key.as_str(), g.open_default()))
        .collect();
    assert_eq!(
        open,
        [
            ("fx-g1", true),
            ("fx-g1.7", true),
            ("fx-g5", true),
            ("fx-g3", false),
            (OUTSIDE_KEY, true)
        ]
    );
    // 札が着地だけの組は動いていない: open の行を閉じれば畳む。
    let landed_only = vec![
        row("fx-g7", "epic", "open", &[]),
        row("fx-g7.1", "task", "closed", &[]),
    ];
    let g = groups(
        &landed_only,
        &[card("fx-g7.1", Stage::Landed)],
        &BTreeMap::new(),
        NOW,
    );
    assert_eq!(keys(&g), ["fx-g7"]);
    assert_eq!((g[0].live(), g[0].open_default()), (0, false));
    // 札が走れば行が無くても開く。
    let g = groups(
        &landed_only,
        &[card("fx-g7.1", Stage::Running)],
        &BTreeMap::new(),
        NOW,
    );
    assert_eq!((g[0].live(), g[0].open_default()), (1, true));
    // open の行だけの組（札は無い）も開く。
    let kids_only = vec![
        row("fx-g7", "epic", "open", &[]),
        row("fx-g7.1", "task", "open", &[]),
    ];
    let g = groups(&kids_only, &[], &BTreeMap::new(), NOW);
    assert_eq!((g[0].live(), g[0].open_default()), (0, true));
}

/// (4) 組の頭: 閉じた数と全部の数は組の行（閉じた行も memo と問いも数え、入れ子の epic の行は数えない）・百分率・
/// 札の列ごとの数は Landed を除く 4 列を板の順（Blocked・Queued・Running / Gated・止まり）で 0 の列も持つ。
#[test]
fn bvlist_head_progress_and_lane_counts() {
    let gs = groups(&rows(), &cards(), &facts_map(), NOW);
    let g1 = group(&gs, "fx-g1");
    assert_eq!((g1.closed, g1.total, g1.pct()), (1, 6, 16));
    assert_eq!(g1.name, "動く epic");
    assert_eq!(g1.epic.as_deref(), Some("fx-g1"));
    let counts: Vec<(&str, usize)> = g1.counts.iter().map(|(l, n)| (l.name, *n)).collect();
    assert_eq!(
        counts,
        [("block", 1), ("queue", 1), ("run", 1), ("stop", 1)]
    );
    assert_eq!(g1.live(), 4);
    let g17 = group(&gs, "fx-g1.7");
    assert_eq!((g17.closed, g17.total, g17.live()), (0, 1, 1));
    assert_eq!(g17.name, "fx-g1.7 の題 — 長い説明");
    let g3 = group(&gs, "fx-g3");
    assert_eq!((g3.closed, g3.total, g3.pct(), g3.live()), (1, 1, 100, 0));
    let out = group(&gs, OUTSIDE_KEY);
    let stop: Vec<usize> = out.counts.iter().map(|(_, n)| *n).collect();
    assert_eq!(stop, [0, 0, 0, 1]);
    assert_eq!((out.closed, out.total), (1, 2));
    // 行の無い組の百分率は 0。
    let lone = groups(
        &[row("fx-g8", "epic", "open", &[])],
        &[],
        &BTreeMap::new(),
        NOW,
    );
    assert_eq!((lone[0].total, lone[0].pct()), (0, 0));
}

/// (5) 行: 短い題は bead の事実の short（無ければ台帳の題）・題の全体は台帳の字・右の字は札が在れば札の段の字、
/// 無ければ起票と起票の時刻からの経過（時刻が無ければ経過の無い字）・並べの位は止まり・走り・Queued・Blocked・着地・
/// 札の無い問い・便・memo の順・種類の札の字。
#[test]
fn bvlist_row_short_title_and_right() {
    let gs = groups(&rows(), &cards(), &facts_map(), NOW);
    let g1 = group(&gs, "fx-g1");
    let by = |id: &str| g1.rows.iter().find(|r| r.id == id).expect("行");
    let memo = by("fx-g1.4");
    assert_eq!(memo.short, "memo の短い題");
    assert_eq!(memo.title, "fx-g1.4 の題 — 長い説明");
    assert_eq!(
        memo.right,
        format!("{} {}", label(CREATED_KEY), age_at(Some(NOW - 7_200), NOW))
    );
    // 右の字の頭は語の辞書の鍵 pf_created の見出し（吹き出しの起票の欄と同じ語）。
    assert_eq!(
        (CREATED_KEY, label(CREATED_KEY).as_str()),
        ("pf_created", "起票")
    );
    assert_eq!(memo.right, "起票 2h");
    assert_eq!(memo.kind, NodeKind::Memo);
    let q = by("fx-g1.5");
    assert_eq!(
        (q.short.as_str(), q.right.as_str()),
        ("問いの短い題", "起票 ―")
    );
    let failed = by("fx-g1.1");
    assert_eq!(failed.short, failed.title);
    assert_eq!(failed.right, stage_word(&card("fx-g1.1", Stage::Failed)));
    assert_eq!(by("fx-g1.3").right, "Gated");
    let ranks: Vec<u8> = g1.rows.iter().map(|r| r.rank).collect();
    assert_eq!(ranks, [0, 1, 2, 5, 7]);
    row_order_case();
    let c = |s| card("fx-g1.1", s);
    let order: Vec<u8> = [
        Stage::Stopped,
        Stage::Running,
        Stage::Queued,
        Stage::Blocked,
        Stage::Landed,
    ]
    .into_iter()
    .map(|s| row_rank(NodeKind::Task, Some(&c(s))))
    .chain([NodeKind::Question, NodeKind::Task, NodeKind::Memo].map(|k| row_rank(k, None)))
    .collect();
    assert_eq!(order, [0, 1, 2, 3, 4, 5, 6, 7]);
    let tags: Vec<&str> = KIND_TAGS.iter().map(|(_, t, _)| *t).collect();
    assert_eq!(tags, ["便", "memo", "問い", "epic"]);
    assert_eq!(kind_tag(NodeKind::Question), ("問い", "ll-kt k-question"));
}

/// 同じ位の行は id の順（歯 bvlist_row_short_title_and_right が呼ぶ・台帳の並びが逆でも・札の無い便の 3 つが同じ位）。
fn row_order_case() {
    let mut rev = rows();
    rev.reverse();
    let gs = groups(&rev, &[], &facts_map(), NOW);
    let ids: Vec<&str> = group(&gs, "fx-g1")
        .rows
        .iter()
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(ids, ["fx-g1.5", "fx-g1.1", "fx-g1.3", "fx-g1.6", "fx-g1.4"]);
}

/// 台帳の一覧の口の本文。
fn ledger_body(rows: Vec<LedgerRow>) -> Fetched {
    Fetched::Body(
        wire::encode(&LedgerList {
            rows: Reading::Known(rows),
        })
        .expect("enc"),
    )
}

/// 板の口の本文（札は `cards`）。
fn pipe_body() -> Fetched {
    let board = PipelineBoard {
        cards: Reading::Known(cards()),
        misfits: Reading::Unknown,
    };
    Fetched::Body(wire::encode(&board).expect("enc"))
}

/// bead の事実の口の本文（事実は `facts_map`）。
fn beads_body() -> Fetched {
    let facts = BeadFacts {
        rows: Reading::Known(facts_map().into_values().collect()),
    };
    Fetched::Body(wire::encode(&facts).expect("enc"))
}

/// (6) 中身: 台帳が読めなければ測れていない・行の無い台帳は EMPTY・出す組が無ければ NO_OPEN。板の札と bead の
/// 事実が読めない間は段の数が 0 で行の右の字は起票の経過・短い題は台帳の題。事実の口の本文は id の表になる。
#[test]
fn bvlist_content_reads_three_paths() {
    let (ledger, pipe, beads) = (ledger_body(rows()), pipe_body(), beads_body());
    assert_eq!(
        content(&ledger, &pipe, &beads, NOW),
        Body::Filled(groups(&rows(), &cards(), &facts_map(), NOW))
    );
    for bad in [Fetched::NotRead, Fetched::Failed, Fetched::Body("{".into())] {
        assert_eq!(
            content(&bad, &pipe, &beads, NOW),
            Body::Unmeasured(LEDGER_UNREAD)
        );
        let Body::Filled(gs) = content(&ledger, &bad, &bad, NOW) else {
            panic!("札と事実が読めなくても組は出す");
        };
        assert!(gs.iter().all(|g| g.live() == 0));
        let g1 = group(&gs, "fx-g1");
        assert!(
            g1.rows
                .iter()
                .all(|r| r.short == r.title && r.right == "起票 ―")
        );
        assert!(facts(&bad).is_empty());
    }
    let unknown = Fetched::Body(
        wire::encode(&BeadFacts {
            rows: Reading::Unknown,
        })
        .expect("enc"),
    );
    assert!(facts(&unknown).is_empty());
    assert_eq!(facts(&beads).len(), 3);
    let empty = ledger_body(vec![]);
    assert_eq!(content(&empty, &pipe, &beads, NOW), Body::Empty(EMPTY));
    let closed = vec![
        row("fx-g4", "epic", "closed", &[]),
        row("fx-g4.1", "task", "closed", &[]),
    ];
    let closed = ledger_body(closed);
    assert_eq!(
        content(&closed, &Fetched::NotRead, &Fetched::NotRead, NOW),
        Body::Empty(NO_OPEN)
    );
    assert_eq!(FACTS_PATH, "/api/beads");
}

/// (7) DOM の字: 台帳の block の一覧の段は 2 つの枝とも ledgerlist の view を描き、前の一覧の DOM の関数は無い。
/// 一覧の view は台帳・板・事実の 3 つの口を読み、組の開き閉じを Folds の signal で持つ。class は stylesheet に在る。
#[test]
fn bvlist_dom_wiring_text() {
    let ledger = read("src/project/ledger.rs");
    let dom = &ledger[ledger.find("mod dom {").expect("mod dom")..];
    assert_eq!(dom.matches("crate::ledgerlist::view()").count(), 2);
    for gone in [
        "fn list_view(",
        "fn group_view(",
        "staged_body(s,",
        "group_cards(&groups",
    ] {
        assert!(!dom.contains(gone), "ledger.rs の DOM に {gone}");
    }
    let list = read("src/ledgerlist.rs");
    let ldom = &list[list.find("mod dom {").expect("mod dom")..];
    for want in [
        "crate::net::read(ledger::PATH)",
        "crate::net::read(pipeline::PATH)",
        "crate::net::read(FACTS_PATH)",
        "RwSignal::new(Folds::default())",
        "folds.with(|f| f.open(&key, initial))",
        "folds.update(|f| f.set(&key, !now))",
        "title=r.title.clone()>{r.short.clone()}</a>",
    ] {
        assert!(ldom.contains(want), "ledgerlist の DOM に {want} が無い");
    }
    let css = read("style.css");
    for class in [
        "ll-body", "ll-grp", "ll-gh", "ll-tog", "ll-gname", "ll-gid", "ll-prog", "ll-pn",
        "ll-sdots", "ll-gopen", "ll-gnone", "ll-row", "ll-kt", "ll-ls", "ll-rt",
    ] {
        assert!(
            css.contains(&format!(".{class} ")),
            "stylesheet に .{class} が無い"
        );
        assert!(list.contains(class), "ledgerlist に {class} が無い");
    }
    for lane in ["block", "queue", "run", "stop"] {
        assert!(
            css.contains(&format!(".ll-sd.sd-{lane} ")),
            "stylesheet に sd-{lane}"
        );
    }
}

/// 板の札が読めない間は、一覧の view が頭に板の読めない理由を測れていないの 1 行で出す（組は出すが、段の数と行の段の字が
/// 無いのは札が無いからでないと見せる）。理由は widgets の pop の board_unread（読めた板は None）。
#[test]
fn bvlist_board_unread_line() {
    assert_eq!(
        board_unread(&Fetched::Failed),
        Some(tsuzuri_surface::project::pipeline::REASON)
    );
    assert_eq!(board_unread(&pipe_body()), None);
    let list = read("src/ledgerlist.rs");
    let ldom = &list[list.find("mod dom {").expect("mod dom")..];
    for want in [
        "let unread = move || pipe.with(board_unread).map(unmeasured);",
        "<div class=\"ll-body\">{unread}{list}</div>",
    ] {
        assert!(ldom.contains(want), "ledgerlist の DOM に {want} が無い");
    }
}

/// 板が読めない間は組の頭に段の数を描かない（head_counts が None で COUNTS_UNKNOWN の 1 つ・0 の字を出さない）。
/// 読めた板では 0 でない列の数だけを描く。
#[test]
fn bvlist_head_counts_unknown() {
    assert_eq!(COUNTS_UNKNOWN, "?");
    let ledger = ledger_body(rows());
    let unread = board_unread(&Fetched::Failed).is_some();
    let Body::Filled(gs) = content(&ledger, &Fetched::Failed, &beads_body(), NOW) else {
        panic!("組");
    };
    assert!(gs.iter().all(|g| head_counts(g, unread).is_none()));
    let Body::Filled(gs) = content(&ledger, &pipe_body(), &beads_body(), NOW) else {
        panic!("組");
    };
    let drawn: Vec<usize> = gs
        .iter()
        .filter_map(|g| head_counts(g, false))
        .flatten()
        .map(|(_, n)| n)
        .collect();
    assert!(
        !drawn.is_empty() && drawn.iter().all(|n| *n > 0),
        "{drawn:?}"
    );
    // 読めた板で札の無い組は空の点（? を出さない）。
    assert_eq!(head_counts(group(&gs, "fx-g3"), false), Some(Vec::new()));
    let list = read("src/ledgerlist.rs");
    let ldom = &list[list.find("mod dom {").expect("mod dom")..];
    for want in [
        "let unread = crate::net::read(pipeline::PATH).with_untracked(|p| board_unread(p).is_some());",
        "let dots = match head_counts(g, unread) {",
        "{COUNTS_UNKNOWN}</b>",
    ] {
        assert!(ldom.contains(want), "ledgerlist の DOM に {want} が無い");
    }
}
