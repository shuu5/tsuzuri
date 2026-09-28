//! 行 h-cards-home の歯: account board の HOME の口座の名の欄と移動の行の hover の card、
//! 群の枠の限度で止まった session の数と詳しくの段（記録の数・出所・doctor の群の行）。

use std::path::PathBuf;

use tsuzuri_contract::account::{AccountDoc, GroupCard, MoveRow};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::home::{
    ACCT_SRC, EXPERT_CHARS, GroupMore, MvRow, NONE, acct_card, doctor_line, group_more, home,
    limited, moves, mv_card, wrap_words,
};
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::seat::hmd;
use tsuzuri_surface::vocab::{label, vocab};
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

fn filled<T: std::fmt::Debug>(body: Body<T>) -> T {
    match body {
        Body::Filled(v) => v,
        other => panic!("中身ありでない: {other:?}"),
    }
}

fn groups(doc: &AccountDoc) -> Vec<GroupCard> {
    match &doc.groups {
        Reading::Known(g) => g.clone(),
        Reading::Unknown => panic!("fixture の群が読めない"),
    }
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

/// (1) 語を字数に畳み、字数を超える語は片に切る。
#[test]
fn hcard_wrap_words() {
    assert_eq!(EXPERT_CHARS, 60);
    assert_eq!(wrap_words("a b c", 3), s(&["a b", "c"]));
    assert_eq!(wrap_words("aaa bbb", 7), s(&["aaa bbb"]));
    assert_eq!(wrap_words("aaa bbb", 6), s(&["aaa", "bbb"]));
    assert_eq!(wrap_words("abcdefgh ij", 4), s(&["abcd", "efgh", "ij"]));
    assert_eq!(wrap_words("abcdefg h", 5), s(&["abcde", "fg h"]));
    assert_eq!(wrap_words("あいう えお", 6), s(&["あいう えお"]));
    assert_eq!(wrap_words("あいう えお", 5), s(&["あいう", "えお"]));
    assert!(wrap_words("", 5).is_empty());
    assert_eq!(wrap_words("  a   b  ", 3), s(&["a b"]));
}

/// (2)(3) doctor の群の行の形と、記録の数の数え方。
#[test]
fn hcard_group_more_rules() {
    let doc = fixture();
    let g = groups(&doc);
    assert_eq!(
        doctor_line(&g[0]),
        "group=Tier1 accounts=acct-1,acct-2 anchors=2 seat-accounts=acct-1 current=acct-1 next=acct-2"
    );
    assert_eq!(
        doctor_line(&g[1]),
        "group=Tier2 accounts=acct-2 anchors=1 seat-accounts=acct-1 current=acct-2 next=none"
    );
    let mut t1 = g[0].clone();
    t1.members[1].seat_account = Some("acct-2".into());
    assert!(doctor_line(&t1).contains(" seat-accounts=acct-1,acct-2 "));
    t1.members[1].seat_account = Some("acct-1".into());
    assert!(doctor_line(&t1).contains(" seat-accounts=acct-1 "));
    t1.members[0].seat_account = None;
    t1.members[1].seat_account = None;
    assert!(doctor_line(&t1).contains(" seat-accounts=none "));

    assert_eq!(group_more(&g[0], &Reading::Unknown).records, "記録 ―");
    assert_eq!(group_more(&g[0], &Reading::Known(vec![])).records, "記録 0 件");
    let mut mv = match &doc.moves {
        Reading::Known(m) => m.clone(),
        Reading::Unknown => panic!("fixture の移動が読めない"),
    };
    for at in [1790400000, 1790300000] {
        mv.push(MoveRow {
            at,
            group: "Tier1".into(),
            from: Some("acct-1".into()),
            to: "acct-2".into(),
        });
    }
    let mv = Reading::Known(mv);
    assert_eq!(group_more(&g[0], &mv).records, "記録 3 件");
    assert_eq!(group_more(&g[1], &mv).records, "記録 1 件");
}

/// (3)(4) fixture の群の詳しくの段と、home の群の枠が同じ値を持つこと。
#[test]
fn hcard_group_more_on_fixture() {
    let doc = fixture();
    let g = groups(&doc);
    assert_eq!(
        group_more(&g[0], &doc.moves),
        GroupMore {
            records: "記録 1 件".into(),
            src: "groups/Tier1.account・host.toml の群の行".into(),
            doctor: s(&[
                "group=Tier1 accounts=acct-1,acct-2 anchors=2",
                "seat-accounts=acct-1 current=acct-1 next=acct-2",
            ]),
        }
    );
    assert_eq!(
        group_more(&g[1], &doc.moves),
        GroupMore {
            records: "記録 1 件".into(),
            src: "groups/Tier2.account・host.toml の群の行".into(),
            doctor: s(&[
                "group=Tier2 accounts=acct-2 anchors=1 seat-accounts=acct-1",
                "current=acct-2 next=none",
            ]),
        }
    );
    for d in [&doc, &limited_doc()] {
        let views = filled(home(d).groups);
        assert_eq!(views.len(), 2);
        for (v, c) in views.iter().zip(&g) {
            assert_eq!(v.name, c.row.group);
            assert_eq!(v.limited, limited(c, &d.sessions));
            assert_eq!(v.more, group_more(c, &d.moves));
        }
    }
    let views = filled(home(&limited_doc()).groups);
    assert_eq!((views[0].limited, views[1].limited), (2, 1));
}

/// proj-a-orch・proj-c-orch・proj-b-orch を Limit にした電文。
fn limited_doc() -> AccountDoc {
    let mut d = fixture();
    for s in &mut d.sessions {
        if ["proj-a-orch", "proj-c-orch", "proj-b-orch"].contains(&s.name.as_str()) {
            s.state = SeatState::Limit;
        }
    }
    d
}

/// (4) 群の project の Limit の session だけを数える。
#[test]
fn hcard_limited_count() {
    let doc = fixture();
    let g = groups(&doc);
    assert_eq!(limited(&g[0], &doc.sessions), 0);
    assert_eq!(limited(&g[1], &doc.sessions), 0);
    let mut ss = doc.sessions.clone();
    ss[0].state = SeatState::Limit;
    assert_eq!(limited(&g[0], &ss), 1);
    ss[3].state = SeatState::Limit;
    assert_eq!(limited(&g[0], &ss), 2);
    assert_eq!(limited(&g[1], &ss), 0);
    ss[2].state = SeatState::Limit;
    assert_eq!(limited(&g[1], &ss), 1);
    let mut z = doc.sessions[0].clone();
    z.project = "proj-z".into();
    z.state = SeatState::Limit;
    ss.push(z);
    assert_eq!((limited(&g[0], &ss), limited(&g[1], &ss)), (2, 1));
    for st in [SeatState::Run, SeatState::Wait, SeatState::Silent, SeatState::Unknown] {
        let mut ss = doc.sessions.clone();
        for x in &mut ss {
            x.state = st;
        }
        assert_eq!(limited(&g[0], &ss), 0);
        assert_eq!(limited(&g[1], &ss), 0);
    }
}

fn accounts(doc: &mut AccountDoc) -> &mut Vec<tsuzuri_contract::account::AccountRow> {
    match &mut doc.accounts {
        Reading::Known(a) => a,
        Reading::Unknown => panic!("fixture の口座が読めない"),
    }
}

/// (5) 口座の card の決まり（占有・usage の有無・窓の順・model・session）。
#[test]
fn hcard_acct_rules() {
    let mut doc = fixture();
    let at = doc.at;
    let ss = doc.sessions.clone();
    let rows = accounts(&mut doc).clone();

    let mut r = rows[0].clone();
    r.occupant = None;
    assert_eq!(acct_card(&r, &ss, at).kind, "口座 · pipeline が使える");
    assert_eq!(
        acct_card(&r, &ss, at).kind,
        format!("{} · {}", label("accounts"), label("free_for_pipeline"))
    );

    let mut r = rows[1].clone();
    r.usage = Reading::Unknown;
    let c = acct_card(&r, &ss, at);
    assert_eq!(c.value, "測れていない");
    assert_eq!(c.more, s(&["session 0"]));

    let mut r = rows[0].clone();
    r.usage = Reading::Known(vec![]);
    let c = acct_card(&r, &ss, at);
    assert_eq!(c.value, NONE);
    assert_eq!(c.more, s(&["session 3 · proj-a-orch / proj-a.3-20260927T113000Z"]));

    let base = acct_card(&rows[0], &ss, at);
    let mut r = rows[0].clone();
    if let Reading::Known(u) = &mut r.usage {
        u.reverse();
    }
    let rev = acct_card(&r, &ss, at);
    assert_eq!((rev.value, rev.more), (base.value.clone(), base.more.clone()));

    let mut r = rows[0].clone();
    r.model = None;
    assert!(acct_card(&r, &ss, at).more.contains(&"model ↻ 09-29 21:00 JST".to_string()));

    let mut r = rows[0].clone();
    if let Reading::Known(u) = &mut r.usage {
        let mut o = u[0].clone();
        o.window = "other".into();
        o.used_pct = 7;
        u.push(o);
    }
    let c = acct_card(&r, &ss, at);
    assert_eq!(c.value, base.value);
    assert_eq!(c.more, base.more);

    let mut none = ss.clone();
    for x in &mut none {
        x.account = None;
    }
    assert_eq!(acct_card(&rows[0], &none, at).more.last().map(String::as_str), Some("session 0"));
}

/// (6) fixture の 3 口座の card と、home の口座の行の card。
#[test]
fn hcard_acct_on_fixture() {
    let mut doc = fixture();
    let at = doc.at;
    let ss = doc.sessions.clone();
    let rows = accounts(&mut doc).clone();
    assert_eq!(ACCT_SRC, "fleet usage --show・doctor の口座の行");
    assert_eq!(hmd(1790512200, at), "21:30 JST");
    let want = [
        Card {
            title: "acct-1".into(),
            kind: "口座 · 占有 Tier1".into(),
            value: "5h 42% · 7d 61% · model 30%".into(),
            src: ACCT_SRC.into(),
            more: s(&[
                "5h ↻ 21:30 JST",
                "7d ↻ 09-29 21:00 JST",
                "model ↻ 09-29 21:00 JST · opus",
                "session 3 · proj-a-orch / proj-a.3-20260927T113000Z",
            ]),
        },
        Card {
            title: "acct-2".into(),
            kind: "口座 · 占有 Tier2".into(),
            value: "5h 100% · 7d 81% · model 12%".into(),
            src: ACCT_SRC.into(),
            more: s(&["5h ↻ 21:45 JST", "model ↻ 09-28 21:00 JST · sonnet", "session 0"]),
        },
        Card {
            title: "acct-3".into(),
            kind: "口座 · retired".into(),
            value: "―".into(),
            src: ACCT_SRC.into(),
            more: s(&["session 0"]),
        },
    ];
    for (r, w) in rows.iter().zip(&want) {
        assert_eq!(&acct_card(r, &ss, at), w);
    }
    let views = filled(home(&fixture()).accounts);
    assert_eq!(views.len(), 3);
    for (v, r) in views.iter().zip(&rows) {
        assert_eq!(v.label, r.label);
        assert_eq!(v.card, acct_card(r, &ss, at));
    }
}

/// (7) 移動の行の card。
#[test]
fn hcard_move_on_fixture() {
    let doc = fixture();
    let rows = match &doc.moves {
        Reading::Known(m) => moves(m, doc.at).shown,
        Reading::Unknown => panic!("fixture の移動が読めない"),
    };
    let cards: Vec<Card> = rows.iter().map(mv_card).collect();
    assert_eq!(
        cards,
        vec![
            Card {
                title: "acct-2 → acct-1".into(),
                kind: "口座の移動 · Tier1".into(),
                value: "◷ 記録 19:00 JST".into(),
                src: "groups/history/Tier1 ほか".into(),
                more: vec![],
            },
            Card {
                title: "― → acct-2".into(),
                kind: "口座の移動 · Tier2".into(),
                value: "◷ 記録 07:05 JST".into(),
                src: "groups/history/Tier2 ほか".into(),
                more: vec![],
            },
        ]
    );
    let m = MvRow {
        at: "21:00 JST".into(),
        group: "G".into(),
        from: "a".into(),
        to: "b".into(),
    };
    assert_eq!(mv_card(&m).kind, format!("{} · G", label("moves")));
}

/// tag（`needle` の前の `<` から後の `>` まで）を全部返す。
fn tags_with<'a>(text: &'a str, needle: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find(needle) {
        let at = from + i;
        let open = text[..=at].rfind('<').expect("tag の始まり");
        let close = at + text[at..].find('>').expect("tag の終わり");
        out.push(&text[open..=close]);
        from = at + needle.len();
    }
    out
}

/// (8) DOM の部分の字（card の付け方・限度の数・詳しくの段）と account board の card の層。
#[test]
fn hcard_dom_wiring() {
    let src = read("src/account/home.rs");
    let dom = &src[src.find("mod dom {").expect("mod dom が在る")..];
    assert!(dom.matches("use:attach=").count() >= 2);
    let names = tags_with(dom, "class=\"c-name\"");
    assert!(!names.is_empty());
    assert!(names.iter().all(|t| t.contains("use:attach=")), "{names:?}");
    assert!(
        tags_with(dom, "<li").iter().any(|t| t.starts_with("<li") && t.contains("use:attach=")),
        "li に card が無い"
    );
    for s in [
        "mv_card(",
        "shows_internal(",
        "class=\"kpi-s\"",
        "class=\"gmore\"",
        "class=\"gm\"",
        "class=\"gm1 src\"",
        "class=\"gm1 int xo\"",
        "hs(\"limited\")",
        "\"p_more\"",
    ] {
        assert!(dom.contains(s), "DOM に {s} が無い");
    }
    let board = read("src/account/board.rs");
    assert!(board.contains("provide_context(HoverCtx::default())"));
    assert!(board.contains("<CardLayer/>"));
}

/// (9) 語の鍵は辞書に在り、class は stylesheet に在る。
#[test]
fn hcard_vocab_css() {
    for k in [
        "limited",
        "p_more",
        "accounts",
        "occupant",
        "moves",
        "free_for_pipeline",
        "retired",
        "st_unknown",
    ] {
        let t = vocab().term(k).unwrap_or_else(|| panic!("語の鍵 {k} が辞書に無い"));
        assert!(!t.label.is_empty(), "語の鍵 {k} の語が空");
    }
    let css = read("style.css");
    for c in ["kpi-s", "gmore", "gm", "gm1", "src", "int", "xo", "rm-t", "rm-a"] {
        let dotted = format!(".{c}");
        let found = css.match_indices(&dotted).any(|(i, _)| {
            css[i + dotted.len()..]
                .chars()
                .next()
                .is_none_or(|n| !(n.is_ascii_alphanumeric() || n == '-' || n == '_'))
        });
        assert!(found, "style.css に .{c} が無い");
    }
}

/// filter の語（着地済みの歯の接頭辞）。
const FILTER: [&str; 69] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_", "server_",
    "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_", "mkeys_",
    "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_", "pmore_",
    "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "qgate_", "nsum_",
];

/// (11) この file の歯の名は hcard_ で始まり、残りの字は filter の語を含まない。
#[test]
fn hcard_names_filtered() {
    let me = read("tests/hcard.rs");
    let lines: Vec<&str> = me.lines().collect();
    let mut n = 0;
    for (i, l) in lines.iter().enumerate() {
        if l.trim() != "#[test]" {
            continue;
        }
        let f = lines[i + 1].trim();
        let name = f
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("test の属性の次が fn でない: {f}"));
        let rest = name
            .strip_prefix("hcard_")
            .unwrap_or_else(|| panic!("{name} が hcard_ で始まらない"));
        for w in FILTER {
            assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
        }
        n += 1;
    }
    assert_eq!(n, 10);
}
