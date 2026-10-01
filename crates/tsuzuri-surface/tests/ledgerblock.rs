//! 便 g-ledger の歯: 上段の 4 数と純減の字・判定の 5 値の表・burndown の座標・未反映の数と分からない種類・
//! 年齢の字・配置の表・測れていないと台帳の一覧・着地済みの外形と依存。
#![cfg(test)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use tsuzuri_boundary::server::ledger as server_ledger;
use tsuzuri_contract::board::{LedgerJudge, Reading};
use tsuzuri_contract::ledger::{LedgerList, LedgerRow};
use tsuzuri_contract::stats::{LedgerStats, UnreflectedKind};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ledger::{
    self, BURN_H, BURN_W, JUDGES, LAYOUT, METRICS, Metrics, NONE, Part, Tier, UNREF_KINDS, age,
    burn_svg, burndown, content, fixed1, judge, kind_name, layout, net, points, spark, spark_svg,
    stats,
};
use tsuzuri_surface::project::{Body, NO_CONTENT, NOT_READ};
use tsuzuri_surface::view::{Fetched, Screen};
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の組（組の名 → 電文）。
fn fixture() -> BTreeMap<String, LedgerStats> {
    wire::decode(&read("../../tests/fixtures/surface/ledger-stats.json"))
        .expect("fixture の組が電文として読める")
}

fn set(name: &str) -> LedgerStats {
    fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"))
}

/// 口の本文の形（指標を known の鍵で包んだ字）。
fn wrap(s: &LedgerStats) -> Fetched {
    Fetched::Body(wire::encode(&Reading::Known(s.clone())).expect("電文"))
}

fn body_of(name: &str) -> Fetched {
    wrap(&set(name))
}

fn ledger_rows() -> Vec<LedgerRow> {
    let text = read("../../tests/fixtures/ledger/board-8.jsonl");
    let Reading::Known(items) = server_ledger::parse(&text) else {
        panic!("fixture が server の読みで Unknown");
    };
    items.into_iter().map(|i| i.row).collect()
}

fn known_screen() -> Screen {
    let body = wire::encode(&LedgerList {
        rows: Reading::Known(ledger_rows()),
    })
    .expect("電文");
    Screen::initial().after_read(&Fetched::Body(body), 100)
}

/// 指標の段を組む今（fixture の組の時点）。
const NOW: u64 = 1_791_676_800;

fn filled(name: &str, screen: &Screen) -> Metrics {
    match content(&body_of(name), screen, NOW) {
        Body::Filled(m) => m,
        other => panic!("組 {name} が中身を出さない: {other:?}"),
    }
}

/// (1) 上段の 4 数（open の task・open の memo・未反映・純減 24h）と、純減の矢印と数の字。
#[test]
fn ledgerblock_top_four_and_net_text() {
    let names: Vec<String> = fixture().into_keys().collect();
    assert_eq!(names, vec!["empty", "filled"]);
    let m = filled("filled", &known_screen());
    let top: Vec<Option<String>> = [Part::Task, Part::Memo, Part::Unref, Part::Net24]
        .into_iter()
        .map(|p| m.text(p))
        .collect();
    let want: Vec<Option<String>> = ["5", "3", "3", "−2"]
        .into_iter()
        .map(|s| Some(s.to_string()))
        .collect();
    assert_eq!(top, want);
    assert_eq!(m.net24.arrow, "↓");
    assert_eq!(m.net24.class, "net net-down");
    let e = filled("empty", &Screen::initial());
    assert_eq!(e.text(Part::Task), Some("0".to_string()));
    assert_eq!(e.text(Part::Net24), Some("0".to_string()));
    assert_eq!(e.net24.arrow, "→");

    net_rows_and_rate(m, e);
}

/// 純減の値ごとの矢印と字と class・主な指標の行の字と小数 1 桁の丸め。
fn net_rows_and_rate(m: Metrics, e: Metrics) {
    for (value, arrow, text, class) in [
        (-3, "↓", "−3", "net net-down"),
        (2, "↑", "+2", "net net-up"),
        (0, "→", "0", "net net-flat"),
        (-1, "↓", "−1", "net net-down"),
        (15, "↑", "+15", "net net-up"),
    ] {
        let n = net(value);
        assert_eq!(
            (n.arrow, n.text.as_str(), n.class),
            (arrow, text, class),
            "値 {value}"
        );
    }
    // 主な指標の行: closed/日（小数 1 桁）と純減 7d。
    assert_eq!(m.text(Part::Rate), Some("1.7".to_string()));
    assert_eq!(m.text(Part::Net7), Some("−4".to_string()));
    assert_eq!(m.net7.arrow, "↓");
    assert_eq!(e.text(Part::Rate), Some("0.0".to_string()));
    assert_eq!(fixed1(2.25), "2.3");
    assert_eq!(fixed1(0.04), "0.0");
}

/// (2) 判定の 5 値は 1 か所の表で記号と class と語の鍵に写り、電文の判定をそのまま写す。
#[test]
fn ledgerblock_judge_table_five_values() {
    let want = [
        (LedgerJudge::Clogged, "!", "jdg j-bad", "j_bad"),
        (LedgerJudge::PilingUp, "↑", "jdg j-up", "j_up"),
        (LedgerJudge::Stalled, "→", "jdg j-stall", "j_stall"),
        (LedgerJudge::OnTrack, "✓", "jdg j-ok", "j_ok"),
        (LedgerJudge::NoLedger, "―", "jdg j-none", "j_none"),
    ];
    let got: Vec<_> = JUDGES
        .iter()
        .map(|j| (j.judge, j.symbol, j.class, j.key))
        .collect();
    assert_eq!(got, want);
    for v in LedgerJudge::ALL {
        assert_eq!(JUDGES.iter().filter(|j| j.judge == v).count(), 1, "{v:?}");
        assert_eq!(judge(v).judge, v);
    }
    assert_eq!(filled("filled", &Screen::initial()).judge.key, "j_ok");
    assert_eq!(filled("empty", &Screen::initial()).judge.key, "j_none");
    // 電文の判定を差し替えれば写しも変わる（面は数から判じ直さない）。
    for v in LedgerJudge::ALL {
        let mut s = set("filled");
        s.judge = v;
        let f = wrap(&s);
        let Body::Filled(m) = content(&f, &Screen::initial(), NOW) else {
            panic!("中身が無い");
        };
        assert_eq!(m.judge, judge(v));
    }
}

/// (3) burndown は 14 日で、棒と線の座標が見本の式（W 320・H 72）で計算した期待と一致する。
#[test]
fn ledgerblock_burndown_coordinates() {
    let s = set("filled");
    assert_eq!(s.days.len(), 14);
    let b = burndown(&s.days, BURN_W, BURN_H);
    assert_eq!(b.bars.len(), 14);
    assert_eq!(b.line.len(), 14);
    let xs: Vec<String> = b.bars.iter().map(|r| fixed1(r.x)).collect();
    assert_eq!(
        xs,
        [
            "5.0", "27.3", "49.6", "71.9", "94.1", "116.4", "138.7", "161.0", "183.3", "205.6",
            "227.9", "250.1", "272.4", "294.7"
        ]
    );
    let hs: Vec<String> = b.bars.iter().map(|r| fixed1(r.height)).collect();
    assert_eq!(
        hs,
        [
            "10.8", "0.0", "32.4", "21.6", "10.8", "21.6", "0.0", "10.8", "32.4", "21.6", "0.0",
            "10.8", "32.4", "21.6"
        ]
    );
    let ys: Vec<String> = b.bars.iter().map(|r| fixed1(r.y)).collect();
    assert_eq!(
        ys,
        [
            "61.2", "72.0", "39.6", "50.4", "61.2", "50.4", "72.0", "61.2", "39.6", "50.4", "72.0",
            "61.2", "39.6", "50.4"
        ]
    );
    assert!(b.bars.iter().all(|r| fixed1(r.width) == "20.3"));
    assert_eq!(
        points(&b.line),
        "15.1,10.0 37.4,7.0 59.7,16.0 82.0,13.0 104.3,13.0 126.6,19.0 148.9,13.0 171.1,4.0 \
         193.4,13.0 215.7,16.0 238.0,16.0 260.3,13.0 282.6,19.0 304.9,25.0"
    );
    let svg = burn_svg(&b);
    assert!(
        svg.starts_with(r#"<svg class="lburn" viewBox="0 0 320 72""#),
        "{svg}"
    );
    assert_eq!(svg.matches(r#"<rect class="lb-closed""#).count(), 14);
    assert!(
        svg.contains(r#"<rect class="lb-closed" x="5.0" y="61.2" width="20.3" height="10.8"/>"#)
    );
    assert!(svg.contains(r#"<polyline class="lb-open" fill="none" points="15.1,10.0 37.4,7.0"#));

    empty_days_and_spark(s);
}

/// 0 ばかりの日の burndown と、sparkline の座標。
fn empty_days_and_spark(s: LedgerStats) {
    // 0 ばかりの日でも割り算が落ちない（最大は 1 から）。
    let e = burndown(&set("empty").days, BURN_W, BURN_H);
    assert!(e.bars.iter().all(|r| r.height == 0.0 && r.y == BURN_H));
    assert!(e.line.iter().all(|(_, y)| fixed1(*y) == "40.0"));

    // sparkline（W 120・H 24・見本の spark14 の式）。
    let sp = spark(&s.days, 120.0, 24.0);
    assert_eq!(
        points(&sp.created),
        "1.0,12.0 10.1,17.0 19.2,22.0 28.2,7.0 37.3,17.0 46.4,22.0 55.5,12.0 64.5,2.0 \
         73.6,22.0 82.7,17.0 91.8,22.0 100.8,12.0 109.9,17.0 119.0,22.0"
    );
    assert_eq!(
        points(&sp.closed),
        "1.0,17.0 10.1,22.0 19.2,7.0 28.2,12.0 37.3,17.0 46.4,12.0 55.5,22.0 64.5,17.0 \
         73.6,7.0 82.7,12.0 91.8,22.0 100.8,17.0 109.9,7.0 119.0,12.0"
    );
    let svg = spark_svg(&sp);
    assert!(svg.contains(r#"<polyline class="ls-created""#));
    assert!(svg.contains(r#"<polyline class="ls-closed""#));
}

/// (4) 未反映は電文の数をそのまま出し、分からない種類は名を出す（0 と書かない）・台帳の一覧から数え直さない。
#[test]
fn ledgerblock_unreflected_copies_wire() {
    let screen = known_screen();
    let m = filled("filled", &screen);
    assert_eq!(m.unref.count, 3);
    assert_eq!(m.unref.unknown, vec!["ruling", "request"]);
    assert_eq!(m.text(Part::UnrefCount), Some("3".to_string()));
    // 台帳の一覧が在っても無くても同じ（面の側で数え直さない）。
    assert_eq!(filled("filled", &Screen::initial()).unref, m.unref);
    assert_eq!(filled("filled", &screen.after_lost()).unref, m.unref);
    let mut s = set("filled");
    s.unreflected = 41;
    s.unreflected_unknown = vec![];
    let f = wrap(&s);
    let Body::Filled(m2) = content(&f, &screen, NOW) else {
        panic!("中身が無い");
    };
    assert_eq!(m2.unref.count, 41);
    assert!(m2.unref.unknown.is_empty());

    let e = filled("empty", &screen);
    assert_eq!(e.unref.count, 0);
    assert_eq!(e.unref.unknown, vec!["memo", "ruling", "request"]);
    assert_eq!(
        UNREF_KINDS.map(|(k, _)| k),
        UnreflectedKind::ALL,
        "種類の表は 3 つの全部"
    );
    for k in UnreflectedKind::ALL {
        let name = kind_name(k);
        let wire_name = wire::encode(&k).expect("電文");
        assert_eq!(format!("\"{name}\""), wire_name);
    }
}

/// (5) 年齢と lead の字は 4 つの形。
#[test]
fn ledgerblock_age_four_forms() {
    assert_eq!(age(None), NONE);
    assert_eq!(NONE, "―");
    assert_eq!(age(Some(0.5)), "12h");
    assert_eq!(age(Some(0.01)), "0h");
    assert_eq!(age(Some(0.99)), "24h");
    assert_eq!(age(Some(1.0)), "1.0d");
    assert_eq!(age(Some(3.4)), "3.4d");
    assert_eq!(age(Some(9.94)), "9.9d");
    assert_eq!(age(Some(10.0)), "10d");
    assert_eq!(age(Some(12.3)), "12d");
    assert_eq!(age(Some(45.6)), "46d");
    memo_age_and_lead();
}

/// 指標の memo の年齢と lead の字・memo の段の残りの 3 数。
fn memo_age_and_lead() {
    let m = filled("filled", &Screen::initial());
    assert_eq!(m.text(Part::MemoAge), Some("3.4d".to_string()));
    assert_eq!(m.text(Part::Lead), Some("12h".to_string()));
    let e = filled("empty", &Screen::initial());
    assert_eq!(e.text(Part::MemoAge), Some("―".to_string()));
    assert_eq!(e.text(Part::Lead), Some("―".to_string()));
    // memo の段の残りの 3 数。
    let memo: Vec<Option<String>> = [Part::MemoOpen, Part::MemoWait, Part::MemoPromo7]
        .into_iter()
        .map(|p| m.text(p))
        .collect();
    assert_eq!(
        memo,
        vec![Some("3".into()), Some("1".into()), Some("2".into())]
    );
}

/// (6) 配置の表の段の並びと段ごとの項。作業中の数と最古の task はどの段にも無い。
#[test]
fn ledgerblock_layout_table() {
    let got: Vec<(&str, Vec<&str>)> = layout()
        .iter()
        .map(|(t, parts)| (t.name(), parts.iter().map(|p| p.key()).collect()))
        .collect();
    let want: Vec<(&str, Vec<&str>)> = vec![
        ("top", vec!["l_task", "l_memo", "l_unref", "l_net24"]),
        ("main", vec!["l_rate", "l_net7"]),
        ("burn", vec!["l_burn"]),
        ("memo", vec!["m_open", "m_wait", "m_promo7", "m_age"]),
        ("unref", vec!["unref"]),
        ("list", vec!["ledger_block"]),
    ];
    assert_eq!(got, want);
    assert_eq!(layout(), LAYOUT);
    assert_eq!(LAYOUT[0].0, Tier::Top);
    assert_eq!(
        LAYOUT[0].1.iter().map(|p| p.key()).collect::<Vec<_>>(),
        METRICS
    );
    tiers_and_keys();
}

/// 項の段・項は 1 度ずつで作業中の数と最古の task は無い・語の鍵は語の辞書に在る。
fn tiers_and_keys() {
    let tier_of = |part: Part| {
        LAYOUT
            .iter()
            .filter(|(_, ps)| ps.contains(&part))
            .map(|(t, _)| *t)
            .collect::<Vec<_>>()
    };
    for p in [Part::Rate, Part::Net7] {
        assert_eq!(tier_of(p), vec![Tier::Main], "{p:?}");
    }
    // 項は 1 度ずつ・作業中の数と最古の task の項は無い。
    let keys: Vec<&str> = LAYOUT
        .iter()
        .flat_map(|(_, ps)| ps.iter().map(|p| p.key()))
        .collect();
    let unique: BTreeSet<&str> = keys.iter().copied().collect();
    assert_eq!(unique.len(), keys.len());
    for k in &keys {
        for bad in ["wip", "in_progress", "progress", "oldest", "old"] {
            assert!(!k.contains(bad), "項 {k} は作業中か最古の task");
        }
    }
    // 項の語の鍵と、段の DOM が使う語の鍵は語の辞書に在る。
    let extra = ["memo_promo", "st_unknown"];
    for key in keys
        .iter()
        .copied()
        .chain(JUDGES.iter().map(|j| j.key))
        .chain(extra)
    {
        assert!(vocab().term(key).is_some(), "鍵 {key} が vocab に無い");
    }
    epics_and_more_counts();
}

/// epic の進みと、問い・epic・ready・blocked・stale の数。
fn epics_and_more_counts() {
    // epic の進み: 題は台帳の一覧から id で引き、引けなければ id だけ。
    let m = filled("filled", &known_screen());
    let eps: Vec<(&str, Option<&str>, u32, u32, u32)> = m
        .epics
        .iter()
        .map(|e| (e.id.as_str(), e.title.as_deref(), e.closed, e.total, e.pct))
        .collect();
    assert_eq!(
        eps,
        vec![
            ("bm", Some("最小の画面の見本の根"), 1, 4, 25),
            ("zz.9", None, 2, 3, 67)
        ]
    );
    let bare = filled("filled", &Screen::initial());
    assert!(bare.epics.iter().all(|e| e.title.is_none()));
    // open の問いの数と open の epic の数。
    assert_eq!(m.text(Part::Question), Some("2".to_string()));
    assert_eq!(m.text(Part::Epic), Some("2".to_string()));
    assert_eq!(m.text(Part::Ready), Some("3".to_string()));
    assert_eq!(m.text(Part::Blocked), Some("1".to_string()));
    assert_eq!(m.text(Part::Stale), Some("1".to_string()));
}

/// stylesheet の class の名（selector の `.名`）。
fn stylesheet_classes() -> BTreeSet<String> {
    let css = read("style.css");
    let chars: Vec<char> = css.chars().collect();
    let mut out = BTreeSet::new();
    for i in 0..chars.len() {
        let starts = chars
            .get(i + 1)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == '_' || *c == '-');
        if chars[i] == '.' && starts {
            let name: String = chars[i + 1..]
                .iter()
                .take_while(|c| c.is_ascii_alphanumeric() || **c == '-' || **c == '_')
                .collect();
            out.insert(name);
        }
    }
    out
}

/// 関数が組む class（判定・純減・上段の箱）は stylesheet に在る。
#[test]
fn ledgerblock_classes_in_stylesheet() {
    let css = stylesheet_classes();
    let mut used: Vec<&str> = Vec::new();
    used.extend(JUDGES.iter().flat_map(|j| j.class.split_whitespace()));
    for v in [-1, 0, 1] {
        used.extend(net(v).class.split_whitespace());
    }
    used.extend([
        "l4",
        "l4un",
        "on",
        "l4net",
        "lmid",
        "lcap",
        "lburn",
        "lb-closed",
        "lb-open",
        "lspark",
        "ls-created",
        "ls-closed",
        "mpro",
        "lep",
        "lchips",
        "ep",
        "bar",
    ]);
    let missing: Vec<&&str> = used.iter().filter(|c| !css.contains(**c)).collect();
    assert!(missing.is_empty(), "stylesheet に無い class: {missing:?}");
}

/// (7) 口が読めない・まだ読んでいない・電文が読めないは測れていない（0 でなく理由の 1 行）・台帳の一覧は今と同じ。
#[test]
fn ledgerblock_unmeasured_and_list_unchanged() {
    let screen = known_screen();
    for (fetched, want) in [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, ledger::METRICS_REASON),
        (Fetched::Body("{}".to_string()), NO_CONTENT),
        (Fetched::Body("まだ分からない".to_string()), NO_CONTENT),
        (
            Fetched::Body("\"unknown\"".to_string()),
            ledger::METRICS_UNKNOWN,
        ),
    ] {
        match content(&fetched, &screen, NOW) {
            Body::Unmeasured(reason) => {
                assert_eq!(reason, want, "{fetched:?}");
                assert!(!reason.trim().is_empty() && !reason.contains('\n'));
            }
            other => panic!("{fetched:?} が測れていないでない: {other:?}"),
        }
        assert_eq!(ledger::metrics(&fetched), Body::Unmeasured(want));
        assert_eq!(stats(&fetched), Err(want));
    }
    assert_eq!(ledger::metrics(&body_of("filled")), Body::Filled(()));
    assert_eq!(stats(&body_of("filled")), Ok(set("filled")));

    list_unchanged(screen);
}

/// 台帳の一覧は今と同じ・口を失えば数と一覧は測れていない。
fn list_unchanged(screen: Screen) {
    assert_eq!(ledger::count(&screen), Reading::Known(5));
    let Body::Filled(groups) = ledger::body(&screen) else {
        panic!("台帳の一覧が中身を出さない");
    };
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].head.as_ref().map(|h| h.id.as_str()), Some("bm"));
    let kids: Vec<&str> = groups[0].children.iter().map(|c| c.id.as_str()).collect();
    // 閉じた bm.3 は一覧に出さない（行 g-ledger-home）。
    assert_eq!(kids, vec!["bm.1", "bm.10", "bm.2"]);
    let lost = screen.after_lost();
    assert_eq!(ledger::count(&lost), Reading::Unknown);
    assert!(matches!(ledger::body(&lost), Body::Unmeasured(r) if !r.is_empty()));
}

/// (9) 実物の口の本文の写し（Reading で包んだ指標）は中身を出し、上段の open の task の数は写しの known の下の数。
#[test]
fn ledgerblock_real_metrics_body() {
    let text = read("../../tests/fixtures/surface/metrics-body.json");
    // 写しを鍵 → 指標の map として読み（Reading を通さず）、known の下の数を期待にする。
    let mut raw: BTreeMap<String, LedgerStats> =
        wire::decode(&text).expect("写しは鍵 known の下に指標を持つ");
    assert_eq!(raw.keys().collect::<Vec<_>>(), vec!["known"]);
    let inner = raw.remove("known").expect("鍵 known");
    assert_eq!(inner.open.task, 2);
    let fetched = Fetched::Body(text);
    let m = match content(&fetched, &known_screen(), NOW) {
        Body::Filled(m) => m,
        other => panic!("実物の本文が中身を出さない: {other:?}"),
    };
    assert_eq!(m.text(Part::Task), Some(inner.open.task.to_string()));
    assert_eq!(m.open, inner.open);
    assert_eq!(ledger::metrics(&fetched), Body::Filled(()));
    assert_eq!(stats(&fetched), Ok(inner));
}

/// (10) 本文が字 unknown なら台帳が読めない理由（口が読めない・電文が読めないとは違う字）・
/// known も unknown も持たない本文と、包まない指標そのものは電文が読めない理由。
#[test]
fn ledgerblock_unknown_body_reason() {
    let unknown = Fetched::Body(wire::encode(&Reading::<LedgerStats>::Unknown).expect("電文"));
    assert_eq!(
        wire::encode(&Reading::<LedgerStats>::Unknown).expect("電文"),
        "\"unknown\""
    );
    let reason = ledger::METRICS_UNKNOWN;
    assert_eq!(
        content(&unknown, &Screen::initial(), NOW),
        Body::Unmeasured(reason)
    );
    assert_eq!(ledger::metrics(&unknown), Body::Unmeasured(reason));
    assert_eq!(stats(&unknown), Err(reason));
    assert!(!reason.trim().is_empty() && !reason.contains('\n'));
    for other in [ledger::METRICS_REASON, NO_CONTENT, NOT_READ] {
        assert_ne!(reason, other);
    }
    for text in [
        "{}".to_string(),
        r#"{"other":1}"#.to_string(),
        wire::encode(&set("filled")).expect("電文"),
    ] {
        assert_eq!(
            stats(&Fetched::Body(text.clone())),
            Err(NO_CONTENT),
            "{text}"
        );
    }
}

/// (8) 着地済みの外形（BLOCK・口の path・metrics・一覧の body と count）と、足す外の依存は 0 本。
#[test]
fn ledgerblock_landed_shape_and_no_new_deps() {
    assert_eq!(ledger::BLOCK.id, "ledger");
    assert_eq!(ledger::BLOCK.heading, "ledger_block");
    assert_eq!(ledger::BLOCK.class, "panel");
    assert_eq!(ledger::PATH, "/api/ledger");
    assert_eq!(ledger::METRICS_PATH, "/api/metrics");
    let metrics: fn(&Fetched) -> Body<()> = ledger::metrics;
    assert!(matches!(
        metrics(&Fetched::Body("{}".to_string())),
        Body::Unmeasured(_)
    ));
    let _count: fn(&Screen) -> Reading<usize> = ledger::count;
    let _body: fn(&Screen) -> Body<Vec<ledger::Group>> = ledger::body;

    let manifest = read("Cargo.toml");
    let mut names = Vec::new();
    let mut inside = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line.ends_with("dependencies]");
        } else if inside
            && !line.starts_with('#')
            && let Some((k, _)) = line.split_once('=')
            && !k.trim().starts_with('"')
        {
            names.push(k.trim().to_string());
        }
    }
    names.sort();
    assert_eq!(
        names,
        vec![
            "leptos",
            "tsuzuri-boundary",
            "tsuzuri-contract",
            "wasm-bindgen-futures",
            "web-sys"
        ]
    );
}
