//! 便 h-cards-proj の歯: 各 project の表の行の 5 つの欄（要対応・台帳・run の数・orchestrator・口座）と
//! 群の見出しの chip の hover の card（見本の nx・ledCard・pcnt・seatCard・gproj・group の枝）と、DOM の部分の字。

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::{AccountDoc, ProjectRow};
use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::stats::CheckResult;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::cards::{
    GPROJ_SRC, LED_SRC, NX_MISS, NX_SRC, RUNS_UNKNOWN, RowCards, WAIT_NOTE, gproj_card, grp_card,
    led_card, nx_card, orch_card, pcnt_card, row_cards,
};
use tsuzuri_surface::account::projects::{PSort, ProjLine, table};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::hover::Card;

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

/// card の 4 行と詳しくを比べる。
fn check(card: &Card, title: &str, kind: &str, value: &str, src: &str, more: &[&str]) {
    assert_eq!(card.title, title);
    assert_eq!(card.kind, kind, "{title} の種類");
    assert_eq!(card.value, value, "{title} の値");
    assert_eq!(card.src, src, "{title} の出所");
    assert_eq!(card.more, more, "{title} の詳しく");
}

/// 行の next を書き換える（Known の行だけ）。
fn with_next(p: &mut ProjectRow, f: impl FnOnce(&mut tsuzuri_contract::stats::NextStep)) {
    match &mut p.next {
        Reading::Known(s) => f(s),
        Reading::Unknown => panic!("{} の next は Known", p.name),
    }
}

/// 行の席の card を書き換える（Known の行だけ）。
fn with_seat(p: &mut ProjectRow, f: impl FnOnce(&mut tsuzuri_contract::seat::SeatCard)) {
    match &mut p.seat {
        Reading::Known(c) => f(c),
        Reading::Unknown => panic!("{} の席は Known", p.name),
    }
}

/// (1) 要対応の欄の card。
#[test]
fn hcproj_nx_card() {
    assert_eq!(NX_SRC, "中核の next_step_seat");
    assert_eq!(NX_MISS, "なし");
    let f: fn(&ProjectRow, EpochSecs) -> Card = nx_card;
    let doc = fixture();
    assert_eq!(doc.at, 1790510400);
    check(
        &f(&doc.projects[0], doc.at),
        "proj-a · (e) 質問",
        "各 project の次の一手 · e",
        "proj-a.7 · 1 件",
        "中核の next_step_seat · ◷ 21:00 JST",
        &[
            "(a) 限度 / 移動 なし",
            "(b) 応答なし なし",
            "(c) 止まっている run なし",
            "(d) 束の承認 ―",
            "(e) 質問 · proj-a.7 · 1 件",
            "(f) 発効待ち ―",
        ],
    );
    let unknown_more = [
        "(a) 限度 / 移動 ―",
        "(b) 応答なし ―",
        "(c) 止まっている run ―",
        "(d) 束の承認 ―",
        "(e) 質問 ―",
        "(f) 発効待ち ―",
    ];
    for (i, name) in [(1, "proj-b"), (2, "proj-c")] {
        check(
            &nx_card(&doc.projects[i], doc.at),
            &format!("{name} · 測れていない"),
            "各 project の次の一手 · ―",
            "―",
            "中核の next_step_seat · ◷ 21:00 JST",
            &unknown_more,
        );
    }

    // なしだけ当たり・ほかは当たらない・lead はなし。
    let mut nothing = doc.projects[0].clone();
    with_next(&mut nothing, |s| {
        s.lead = NextMove::Nothing;
        for c in &mut s.checks {
            c.result = if c.kind == NextMove::Nothing {
                CheckResult::Hit
            } else {
                CheckResult::Miss
            };
        }
    });
    check(
        &nx_card(&nothing, doc.at),
        "proj-a · (g) 次の一手なし",
        "各 project の次の一手 · g",
        "orchestrator が動いている / 待っている",
        "中核の next_step_seat · ◷ 21:00 JST",
        &[
            "(a) 限度 / 移動 なし",
            "(b) 応答なし なし",
            "(c) 止まっている run なし",
            "(d) 束の承認 なし",
            "(e) 質問 なし",
            "(f) 発効待ち なし",
        ],
    );

    // checks が空。
    let mut empty = doc.projects[0].clone();
    with_next(&mut empty, |s| s.checks.clear());
    let card = nx_card(&empty, doc.at);
    assert_eq!(card.kind, "各 project の次の一手 · ―");
    assert_eq!(card.value, "0 件");
    assert_eq!(card.more, unknown_more);

    // 時刻は at から。
    assert_eq!(
        nx_card(&doc.projects[0], 1790485620).src,
        "中核の next_step_seat · ◷ 14:07 JST"
    );
}

/// (2) 台帳の欄の card。
#[test]
fn hcproj_led_card() {
    assert_eq!(LED_SRC, "bd list --all の bead");
    let f: fn(&ProjectRow) -> Card = led_card;
    let doc = fixture();
    check(
        &f(&doc.projects[0]),
        "proj-a · 台帳の処理状況",
        "↑+2 24h · ↑+4 7d · closed/日 0.9",
        "task 9（ready 5 / blocked 3）",
        "bd list --all の bead · 時点 21:00 JST",
        &["memo 3 · stale 1", "question 2 · epic 2", "lead p50 4.0d / p90 12d"],
    );
    for (i, name) in [(1, "proj-b"), (2, "proj-c")] {
        check(
            &led_card(&doc.projects[i]),
            &format!("{name} · 台帳なし"),
            "台帳の処理状況",
            "測れていない",
            "bd list --all の bead",
            &[],
        );
    }

    let mut p = doc.projects[0].clone();
    let Reading::Known(s) = &mut p.ledger else {
        panic!("proj-a の台帳は Known");
    };
    s.lead = None;
    s.net_drop_24h = -3;
    s.net_drop_7d = 0;
    let card = led_card(&p);
    assert_eq!(card.kind, "↓−3 24h · →0 7d · closed/日 0.9");
    assert_eq!(card.more[2], "lead p50 ― / p90 ―");
}

/// (3) run の数の欄の card。
#[test]
fn hcproj_pcnt_card() {
    assert_eq!(RUNS_UNKNOWN, "state dir か event log が読めない");
    assert_eq!(WAIT_NOTE, "― = 質問の台帳を読んでいない");
    let f: fn(&ProjectRow, EpochSecs) -> Card = pcnt_card;
    let doc = fixture();
    check(
        &f(&doc.projects[0], doc.at),
        "run の数 · proj-a",
        "wait 1 · run 2 · stop 0 · land 3",
        "あなたの決定待ち 2",
        "fleet/events.jsonl · ◷ 21:00 JST",
        &[
            "Queued / Blocked 1",
            "Running / Gated 2",
            "Questioned / Failed / Stopped 0",
            "Landed（今日） 3",
        ],
    );
    let b = pcnt_card(&doc.projects[1], doc.at);
    assert_eq!(b.title, "run の数 · proj-b");
    assert_eq!(b.kind, "wait 0 · run 0 · stop 1 · land 0");
    assert_eq!(b.value, "あなたの決定待ち ―");
    assert_eq!(b.more.last().map(String::as_str), Some(WAIT_NOTE));
    check(
        &pcnt_card(&doc.projects[2], doc.at),
        "run の数 · proj-c",
        "測れていない",
        "あなたの決定待ち ―",
        "fleet/events.jsonl · ◷ 21:00 JST",
        &["state dir か event log が読めない", "― = 質問の台帳を読んでいない"],
    );
    assert_eq!(
        pcnt_card(&doc.projects[0], 1790485620).src,
        "fleet/events.jsonl · ◷ 14:07 JST"
    );
}

/// (4) orchestrator の欄の card（席の無い行は None）。
#[test]
fn hcproj_orch_card() {
    let f: fn(&AccountDoc, &ProjectRow) -> Option<Card> = orch_card;
    let doc = fixture();
    check(
        &f(&doc, &doc.projects[0]).expect("proj-a は席を持つ"),
        "proj-a-orch",
        "動いている · ◷ 19:31 JST から",
        "口座 acct-1 · tick healthy · hb on",
        "seat/proj-a-orch/state.jsonl ほか",
        &["model opus"],
    );
    check(
        &orch_card(&doc, &doc.projects[1]).expect("proj-b は席を持つ"),
        "proj-b-orch",
        "待っている · ◷ 20:35 JST から",
        "口座 acct-1 · tick stale · hb off",
        "seat/proj-b-orch/state.jsonl ほか",
        &["model sonnet", "移動待ち acct-1 → acct-2", "退避までの残り 1101 秒"],
    );
    assert_eq!(orch_card(&doc, &doc.projects[2]), None);

    let kind_with = |f: &dyn Fn(&mut tsuzuri_contract::seat::SeatCard)| {
        let mut p = doc.projects[0].clone();
        with_seat(&mut p, f);
        orch_card(&doc, &p).expect("席は Known").kind
    };
    assert_eq!(kind_with(&|c| c.since = None), "動いている");
    assert_eq!(
        kind_with(&|c| c.since = Some(1790427900)),
        "動いている · ◷ 09-26 22:05 JST から"
    );
    assert_eq!(
        kind_with(&|c| c.state = SeatState::Limit),
        "限度で止まっている · ◷ 19:31 JST から"
    );

    let mut blank = doc.projects[0].clone();
    with_seat(&mut blank, |c| {
        c.tick_healthy = Reading::Unknown;
        c.heartbeat = Reading::Unknown;
        c.account = None;
        c.model = None;
    });
    let card = orch_card(&doc, &blank).expect("席は Known");
    assert_eq!(card.value, "口座 ? · tick ? · hb ?");
    assert_eq!(card.more, ["model ?"]);

    let mut no_groups = doc.clone();
    no_groups.groups = Reading::Unknown;
    assert_eq!(
        orch_card(&no_groups, &no_groups.projects[1])
            .expect("席は Known")
            .more,
        ["model sonnet", "退避までの残り 1101 秒"]
    );

    let mut stay = doc.projects[1].clone();
    stay.move_left_s = None;
    assert_eq!(
        orch_card(&doc, &stay).expect("席は Known").more,
        ["model sonnet", "移動待ち acct-1 → acct-2"]
    );
}

/// (5) 口座の欄の card（席の無い project は群の今の口座だけ）。
#[test]
fn hcproj_gproj_card() {
    assert_eq!(GPROJ_SRC, "doctor の席の行と群の今の記録");
    let f: fn(&AccountDoc, &ProjectRow) -> Card = gproj_card;
    let doc = fixture();
    let src = "doctor の席の行と群の今の記録";
    check(
        &f(&doc, &doc.projects[0]),
        "proj-a-orch",
        "✓ 今の口座で動いている",
        "登録 acct-1 = 群 acct-1",
        src,
        &["動いている"],
    );
    check(
        &gproj_card(&doc, &doc.projects[1]),
        "proj-b-orch",
        "! 移動待ち",
        "登録 acct-1 ≠ 群 acct-2",
        src,
        &["待っている"],
    );
    check(
        &gproj_card(&doc, &doc.projects[2]),
        "proj-c",
        "session なし",
        "Tier1 の今の口座 acct-1",
        src,
        &[],
    );

    let mut no_groups = doc.clone();
    no_groups.groups = Reading::Unknown;
    let card = gproj_card(&no_groups, &no_groups.projects[0]);
    assert_eq!(card.kind, "測れていない");
    assert_eq!(card.value, "登録 acct-1 ? 群 ?");

    let mut no_acct = doc.projects[0].clone();
    with_seat(&mut no_acct, |c| c.account = None);
    let card = gproj_card(&doc, &no_acct);
    assert_eq!(card.kind, "測れていない");
    assert_eq!(card.value, "登録 ? ? 群 acct-1");

    let mut loose = doc.projects[2].clone();
    loose.group = None;
    assert_eq!(gproj_card(&doc, &loose).value, "― の今の口座 ?");
}

/// (6) 群の見出しの chip の card（群が読めないか無ければ None）。
#[test]
fn hcproj_grp_card() {
    let f: fn(&AccountDoc, &str) -> Option<Card> = grp_card;
    let doc = fixture();
    check(
        &f(&doc, "Tier1").expect("Tier1 は在る"),
        "Tier1 · 今の口座 acct-1",
        "project の群 · 記録 1 件",
        "◷ 19:00 JST から · ← 前 acct-2",
        "groups/Tier1.account ほか",
        &[
            "候補 2 口座 → 候補と次の移り先",
            "閾値 5h 85% · 7d 95% · model 95%",
            "anchor 2 件",
            "・ proj-a",
            "・ proj-c",
        ],
    );
    check(
        &grp_card(&doc, "Tier2").expect("Tier2 は在る"),
        "Tier2 · 今の口座 acct-2",
        "project の群 · 記録 1 件",
        "記録なし · ← 前 ―",
        "groups/Tier2.account ほか",
        &[
            "候補 1 口座 → 候補と次の移り先",
            "閾値 5h 85% · 7d 95% · model 95%",
            "anchor 1 件",
            "・ proj-b",
        ],
    );
    assert_eq!(grp_card(&doc, "Tier9"), None);

    let mut no_groups = doc.clone();
    no_groups.groups = Reading::Unknown;
    assert_eq!(grp_card(&no_groups, "Tier1"), None);

    let mut no_moves = doc.clone();
    no_moves.moves = Reading::Unknown;
    assert_eq!(
        grp_card(&no_moves, "Tier1").expect("Tier1 は在る").kind,
        "project の群 · 記録 ―"
    );

    let tier1 = |f: &dyn Fn(&mut tsuzuri_contract::account::GroupCard)| {
        let mut d = doc.clone();
        let Reading::Known(cards) = &mut d.groups else {
            panic!("groups は Known");
        };
        let g = cards
            .iter_mut()
            .find(|g| g.row.group == "Tier1")
            .expect("Tier1 は在る");
        f(g);
        grp_card(&d, "Tier1").expect("Tier1 は在る").value
    };
    assert_eq!(
        tier1(&|g| g.since = Some(1790427900)),
        "◷ 09-26 22:05 JST から · ← 前 acct-2"
    );
    assert_eq!(tier1(&|g| g.recorded = false), "記録なし · ← 前 acct-2");
}

/// 欄ごとに row_cards の値と比べる。
fn same_cards(got: &RowCards, want: &RowCards, at: &str) {
    assert_eq!(got.need, want.need, "{at} の need");
    assert_eq!(got.led, want.led, "{at} の led");
    assert_eq!(got.runs, want.runs, "{at} の runs");
    assert_eq!(got.orch, want.orch, "{at} の orch");
    assert_eq!(got.acc, want.acc, "{at} の acc");
}

/// (7) 表の行と束が card を持つ。
#[test]
fn hcproj_rows_carry_cards() {
    let f: fn(&AccountDoc, &ProjectRow) -> RowCards = row_cards;
    let doc = fixture();
    for (i, p) in doc.projects.iter().enumerate() {
        let c = f(&doc, p);
        assert_eq!(c.need, nx_card(p, doc.at));
        assert_eq!(c.led, led_card(p));
        assert_eq!(c.runs, pcnt_card(p, doc.at));
        assert_eq!(c.orch, orch_card(&doc, p));
        let seated = i < 2;
        assert_eq!(c.orch.is_some(), seated, "{} の orch", p.name);
        assert_eq!(c.acc.is_some(), seated, "{} の acc", p.name);
        if seated {
            assert_eq!(c.acc, Some(gproj_card(&doc, p)));
        }
        // fixture の値の card の 4 行はどれも 36 字以下。
        let cards: Vec<&Card> = [&c.need, &c.led, &c.runs]
            .into_iter()
            .chain(c.orch.iter())
            .chain(c.acc.iter())
            .collect();
        for card in cards {
            for line in [&card.title, &card.kind, &card.value, &card.src] {
                assert!(line.chars().count() <= 36, "{} の行 {line} が 36 字を超える", p.name);
            }
        }
    }

    for sort in PSort::ALL {
        let t = table(&doc, sort, Mode::Beginner);
        let rows: Vec<&ProjLine> = t.groups.iter().flat_map(|g| g.rows.iter()).collect();
        assert_eq!(rows.len(), doc.projects.len(), "{sort:?}");
        for r in rows {
            let want = row_cards(&doc, &doc.projects[r.index]);
            same_cards(&r.cards, &want, &format!("{sort:?} の {}", r.name));
        }
        for g in &t.groups {
            let name = g.head.as_ref().and_then(|h| h.group.as_deref());
            match (sort, name) {
                (PSort::Group, Some(n)) => {
                    assert_eq!(g.card, grp_card(&doc, n), "{n} の束");
                    assert!(g.card.is_some(), "{n} の束は card を持つ");
                }
                _ => assert_eq!(g.card, None, "{sort:?} の束"),
            }
        }
    }

    let group = table(&doc, PSort::Group, Mode::Beginner);
    let names: Vec<Option<&str>> = group
        .groups
        .iter()
        .map(|g| g.head.as_ref().and_then(|h| h.group.as_deref()))
        .collect();
    assert_eq!(names, [Some("Tier1"), Some("Tier2")]);

    // 群の無い見出しの束は card を持たない。
    let mut loose = doc.clone();
    loose.projects[0].group = None;
    let t = table(&loose, PSort::Group, Mode::Beginner);
    let last = t.groups.last().expect("束が在る");
    assert_eq!(last.head.as_ref().and_then(|h| h.group.clone()), None);
    assert_eq!(last.card, None);
}

/// projects.rs の字「mod dom {」より後（DOM の部分）。
fn dom_part() -> String {
    let text = read("src/account/projects.rs");
    let at = text.find("mod dom {").expect("projects.rs に mod dom が在る");
    text[at + "mod dom {".len()..].to_string()
}

/// DOM の部分の fn の本体（宣言の字から、次の行頭 4 空白の fn か pub fn の宣言の前まで・無ければ終わりまで）。
fn fn_body(dom: &str, name: &str) -> String {
    let start = dom
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("DOM の部分に fn {name} が無い"));
    let rest = &dom[start..];
    let first = rest.find('\n').map_or(rest.len(), |i| i + 1);
    let mut end = rest.len();
    let mut at = first;
    for line in rest[first..].split_inclusive('\n') {
        if line.starts_with("    fn ") || line.starts_with("    pub fn ") {
            end = at;
            break;
        }
        at += line.len();
    }
    rest[..end].to_string()
}

/// 本体の中の字 head の所から最初の > までの tag。
fn tag(body: &str, head: &str) -> String {
    let at = body
        .find(head)
        .unwrap_or_else(|| panic!("本体に字 {head} が無い"));
    let rest = &body[at..];
    rest[..rest.find('>').unwrap_or(rest.len())].to_string()
}

fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// (8) DOM の部分が 5 つの欄と 4 つの span と群の chip に card を付ける。
#[test]
fn hcproj_dom_wiring() {
    let text = read("src/account/projects.rs");
    let dom = dom_part();
    assert!(dom.contains("widgets::hover"), "DOM の部分に字 widgets::hover が無い");
    assert!(!text.contains("fn attach("), "projects.rs に自前の fn attach が在る");
    assert!(
        squeeze(&dom).contains("fnattach_some(el:web_sys::Element,card:Option<Card>)"),
        "fn attach_some の宣言の形が違う"
    );
    assert!(
        squeeze(&fn_body(&dom, "attach_some")).contains("attach(el,card)"),
        "fn attach_some が attach に el と card を渡さない"
    );

    let tab = "tabindex=\"0\"";
    let row = fn_body(&dom, "row_view");
    for (head, attr, focus) in [
        ("<div class=C_NEED", "use:attach=row.cards.need.clone()", true),
        ("<div class=C_LED", "use:attach=row.cards.led.clone()", true),
        ("<div class=C_RUN", "use:attach=row.cards.runs.clone()", true),
        ("<div class=C_ORCH", "use:attach_some=row.cards.orch.clone()", false),
        ("<div class=C_ACC", "use:attach_some=row.cards.acc.clone()", false),
    ] {
        let t = tag(&row, head);
        assert!(t.contains(attr), "row_view の {head} の tag {t} に {attr} が無い");
        if focus {
            assert!(t.contains(tab), "row_view の {head} の tag {t} に {tab} が無い");
        }
    }

    let more = fn_body(&dom, "more_view");
    for (class, attr) in [
        ("mi m-run", "use:attach=row.cards.runs.clone()"),
        ("mi m-orch", "use:attach_some=row.cards.orch.clone()"),
        ("mi m-acc", "use:attach_some=row.cards.acc.clone()"),
        ("mi m-led", "use:attach=row.cards.led.clone()"),
    ] {
        let t = tag(&more, &format!("<span class=\"{class}\""));
        assert!(t.contains(attr), "more_view の {class} の tag {t} に {attr} が無い");
    }

    let head = fn_body(&dom, "head_view");
    assert!(
        squeeze(&head).starts_with("fnhead_view(head:GroupHead,card:Option<Card>)->AnyView"),
        "fn head_view の宣言の形が違う"
    );
    let chip = "<span class=\"pchip grp\"";
    let chips: Vec<String> = head
        .match_indices(chip)
        .map(|(i, _)| tag(&head[i..], chip))
        .collect();
    assert!(
        chips.iter().any(|t| t.contains("use:attach_some=card")),
        "head_view の群の chip の tag {chips:?} に use:attach_some=card が無い"
    );
    assert!(
        fn_body(&dom, "group_view").contains("group.card"),
        "group_view が束の card を渡さない"
    );

    // 行の前提: account の board.rs の頁が card の層を持つ。
    let board = read("src/account/board.rs");
    assert!(board.contains("provide_context(HoverCtx::default())"));
    assert!(board.contains("<CardLayer/>"));
}

/// (9) card が引く語の鍵は語の辞書に在って語が空でない。
#[test]
fn hcproj_vocab_keys() {
    for key in [
        "next_all",
        "nx_a",
        "nx_b",
        "nx_c",
        "nx_d",
        "nx_e",
        "nx_f",
        "nx_g",
        "st_unknown",
        "ledger_state",
        "j_none",
        "l_rate",
        "runs4",
        "waiting_you",
        "col_wait",
        "col_run",
        "col_stop",
        "col_land",
        "st_run",
        "st_wait",
        "st_limit",
        "st_silent",
        "seat_match",
        "seat_mismatch",
        "seat_none",
        "move_grace",
        "group",
        "current_account",
        "candidates",
        "no_record",
    ] {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }
}

/// (11) この file の歯の名は hcproj_ で始まり、残りの字は filter の語を含まない。
#[test]
fn hcproj_names_clean() {
    const WORDS: [&str; 92] = [
        "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
        "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
        "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
        "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
        "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
        "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
        "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_",
        "gnav_", "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_",
        "ticker_", "hfig_", "pmore_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_",
        "kcli_", "qgate_", "nsum_", "hcard_", "ntime_", "ptitle_", "ncard_", "hsym_", "smore_",
        "lcard_", "aaround_", "hruling_", "sxaxis_", "plimit_", "bport_", "fmark_", "fstop_",
        "fserve_", "nsumw_", "cadopt_", "tipx_", "hcsess_", "sesplit_", "mstore_", "athr_",
        "qblock_",
    ];
    let text = read("tests/hcproj.rs");
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
    assert_eq!(names.len(), 10, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("hcproj_")
            .unwrap_or_else(|| panic!("歯の名 {name} が hcproj_ で始まらない"));
        for w in WORDS {
            assert!(!rest.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
