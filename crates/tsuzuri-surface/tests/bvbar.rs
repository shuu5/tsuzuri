//! 行 g-topbar の歯: 上の固定の帯（判断の記録 ADR-27 決定 (3)・見本 board-v2 の `#bar`）の次の一手の pill と開く窓、
//! やる事の 3 つの数と色、席からの最新の知らせと件数、席と口座の印（状態・直近 3 時間の細い帯・5 時間と 7 日）、
//! 抜けの検査の印、語の鍵と stylesheet の規則、帯の DOM（wasm の枝）の配線の字。
//! fixture は tests/fixtures/surface/next-step.json・seat-card.json・invariants.json（読むだけ）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::NextMove;
use tsuzuri_contract::notice::{Notice, Notices};
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::stats::NextStep;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::topbar::{
    BAR, GAPS_KEY, GEAR, GEAR_KEY, GO, PILL_GO, PILL_NONE, RECENT_KEY, SEAT_KEY, STALL_KEY,
    STRIP_SPAN, TODOS, USE_WINDOWS, Win, counts, gaps_mark, lead, lead_win, notice_line, seat_bar,
};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;

/// seat-card.json の at（UTC の日の正午・日本時間の 21 時）。
const AT: u64 = 1_790_510_400;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn next_body(name: &str) -> Fetched {
    let mut sets: BTreeMap<String, NextStep> =
        wire::decode(&read("../../tests/fixtures/surface/next-step.json")).expect("next の組");
    let s = sets.remove(name).unwrap_or_else(|| panic!("組 {name}"));
    Fetched::Body(wire::encode(&s).expect("電文"))
}

fn seat_body(name: &str) -> Fetched {
    let mut sets: BTreeMap<String, SeatCard> =
        wire::decode(&read("../../tests/fixtures/surface/seat-card.json")).expect("席の組");
    let c = sets.remove(name).unwrap_or_else(|| panic!("組 {name}"));
    Fetched::Body(wire::encode(&c).expect("電文"))
}

fn graph_body() -> Fetched {
    Fetched::Body(read("../../tests/fixtures/surface/invariants.json"))
}

fn unread() -> [Fetched; 3] {
    [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
    ]
}

/// 次の一手の pill: 電文の lead と件数の字と開く窓、なしと判じなかったと読めない口は窓を開かない。
#[test]
fn bvbar_lead_from_next() {
    assert_eq!(GO, "▶");
    let l = lead(&next_body("stalled"));
    assert_eq!(
        (l.class, l.text.as_str(), l.win),
        (PILL_GO, "▶ 止まった run 2 件を確かめる", Some(Win::Stalled))
    );
    let l = lead(&next_body("question"));
    assert_eq!(
        (l.class, l.text.as_str(), l.win),
        (PILL_GO, "▶ 質問 1 件に答える", Some(Win::Ask))
    );
    let l = lead(&next_body("not-judged"));
    assert_eq!(
        (l.class, l.text.as_str(), l.win),
        (PILL_GO, "▶ 束の承認 4 件", Some(Win::Ask))
    );
    let l = lead(&next_body("nothing"));
    assert_eq!(
        (l.class, l.text.as_str(), l.win),
        (PILL_NONE, "次の一手なし", None)
    );
    for f in unread() {
        let l = lead(&f);
        assert_eq!(
            (l.class, l.text.as_str(), l.win),
            (PILL_NONE, "測れていない", None),
            "{f:?}"
        );
    }
    let wins: Vec<Option<Win>> = NextMove::ALL.into_iter().map(lead_win).collect();
    assert_eq!(
        wins,
        [
            Some(Win::Seat),
            Some(Win::Seat),
            Some(Win::Stalled),
            Some(Win::Ask),
            Some(Win::Ask),
            Some(Win::Gaps),
            None,
        ]
    );
}

/// やる事の 3 つの数は質問・止まった run・抜けの検査の順で、1 以上の物だけ色の class、0 と分からない物は chip zero。
#[test]
fn bvbar_three_counts_colored_when_positive() {
    assert_eq!(
        TODOS,
        [
            ("questions", "org", Win::Ask),
            (STALL_KEY, "red", Win::Stalled),
            (GAPS_KEY, "amb", Win::Gaps),
        ]
    );
    let row = |f: &Fetched, g: &Fetched| -> Vec<(String, String, Option<u32>, Win)> {
        counts(f, g)
            .into_iter()
            .map(|c| (c.class, c.text, c.n, c.win))
            .collect()
    };
    let s = |a: &str| a.to_string();
    assert_eq!(
        row(&next_body("stalled"), &graph_body()),
        [
            (s("chip org"), s("質問 3"), Some(3), Win::Ask),
            (s("chip red"), s("止まった run 2"), Some(2), Win::Stalled),
            (s("chip amb"), s("抜け 2"), Some(2), Win::Gaps),
        ]
    );
    assert_eq!(
        row(&next_body("question"), &Fetched::Failed),
        [
            (s("chip org"), s("質問 1"), Some(1), Win::Ask),
            (s("chip zero"), s("止まった run 0"), Some(0), Win::Stalled),
            (s("chip zero"), s("抜け ?"), None, Win::Gaps),
        ]
    );
    assert_eq!(
        row(&next_body("not-judged"), &graph_body())[1],
        (s("chip zero"), s("止まった run ?"), None, Win::Stalled)
    );
    for f in unread() {
        let r = row(&f, &f);
        assert!(
            r.iter().all(|c| c.0 == "chip zero" && c.2.is_none()),
            "{r:?}"
        );
    }
}

fn notices(project: &str, latest: &[(&str, u64, &str)], unread: &[&str]) -> Fetched {
    let doc = Notices {
        at: AT,
        project: project.to_string(),
        latest: latest
            .iter()
            .map(|(p, at, title)| Notice {
                at: *at,
                project: (*p).to_string(),
                title: (*title).to_string(),
                url: String::new(),
            })
            .collect(),
        unread: unread.iter().map(|s| (*s).to_string()).collect(),
    };
    Fetched::Body(wire::encode(&doc).expect("電文"))
}

/// 席からの最新の知らせ: この project の最新の 1 つの時刻と題と件数で、ほかの project の知らせは数えない。
#[test]
fn bvbar_notice_latest_and_count() {
    let f = notices(
        "tsuzuri",
        &[
            ("other", AT, "ほかの知らせ"),
            ("tsuzuri", AT - 3_600, "着地した"),
        ],
        &[],
    );
    let Body::Filled(l) = notice_line(&f) else {
        panic!("知らせの 1 行");
    };
    assert_eq!(
        (l.when.as_str(), l.title.as_str(), l.count),
        ("20:00", "着地した", 1)
    );
    let f = notices("tsuzuri", &[("other", AT, "ほかの知らせ")], &[]);
    assert!(matches!(notice_line(&f), Body::Empty(_)));
    let f = notices("tsuzuri", &[("tsuzuri", AT, "x")], &["tsuzuri"]);
    assert!(matches!(notice_line(&f), Body::Unmeasured(_)));
    for f in unread() {
        assert!(matches!(notice_line(&f), Body::Unmeasured(_)), "{f:?}");
    }
}

/// 席と口座の印の口座の窓は 5 時間と 7 日の 2 つだけ（model の窓は出さない）で、割合と棒の長さと class を写す。
#[test]
fn bvbar_usage_two_windows() {
    assert_eq!(USE_WINDOWS, ["five_hour", "seven_day"]);
    let b = seat_bar(&seat_body("run"), AT);
    assert_eq!(b.account.as_deref(), Some("acct-4"));
    let u: Vec<(&str, Option<u8>, u8, &str)> = b
        .usage
        .iter()
        .map(|u| (u.short.as_str(), u.used, u.width, u.class))
        .collect();
    assert_eq!(u, [("5h", Some(42), 42, ""), ("7d", Some(85), 85, "w80")]);
    let b = seat_bar(&seat_body("limit"), AT);
    assert_eq!(
        (b.usage[0].used, b.usage[0].width, b.usage[0].class),
        (Some(100), 100, "w100")
    );
    let b = seat_bar(&seat_body("wait"), AT);
    assert!(b.usage.iter().all(|u| u.used.is_none() && u.width == 0));
    let b = seat_bar(&seat_body("silent"), AT);
    let u: Vec<Option<u8>> = b.usage.iter().map(|u| u.used).collect();
    assert_eq!(u, [None, Some(12)]);
}

/// 細い帯は直近 3 時間の区間だけを描く（窓の前の区間は置かず、窓の端で切る）。
#[test]
fn bvbar_strip_three_hours() {
    assert_eq!(STRIP_SPAN.secs(), 10_800);
    let b = seat_bar(&seat_body("run"), AT);
    assert_eq!(b.strip.matches("<rect ").count(), 3, "{}", b.strip);
    assert!(
        b.strip.contains("<rect class=\"sg-wait\" x=\"0\" "),
        "{}",
        b.strip
    );
    assert!(b.strip.contains(" width=\"144\" "), "{}", b.strip);
    assert!(b.strip.contains("class=\"sg-run\""));
    assert!(!b.strip.contains("sg-limit"), "{}", b.strip);
    let b = seat_bar(&seat_body("wait"), AT);
    assert_eq!(b.strip.matches("<rect ").count(), 0);
    let b = seat_bar(&Fetched::Failed, AT);
    assert_eq!(b.strip.matches("<rect ").count(), 0);
}

/// 席と口座の印の状態と から の時刻と tick と heartbeat の字、読めない口は測れていない。
#[test]
fn bvbar_seat_state_and_tick() {
    let b = seat_bar(&seat_body("run"), AT);
    assert_eq!(
        (b.state, b.key, b.since.as_deref(), b.tick.as_str()),
        ("run", "st_run", Some("18:50〜"), "tick ✓ · hb on")
    );
    let b = seat_bar(&seat_body("wait"), AT);
    assert_eq!((b.state, b.tick.as_str()), ("wait", "tick ✗ · hb off"));
    let b = seat_bar(&seat_body("silent"), AT);
    assert_eq!(
        (b.state, b.since.as_deref(), b.tick.as_str()),
        ("silent", None, "tick ? · hb ?")
    );
    for f in unread() {
        let b = seat_bar(&f, AT);
        assert_eq!(
            (b.state, b.key, b.since, b.account),
            ("unknown", "st_unknown", None, None)
        );
    }
}

/// 抜けの検査の印: 抜けありが在れば gp f・まだ分からないだけなら gp u・どちらも無ければ gp p・読めなければ gp u の ?。
#[test]
fn bvbar_gaps_mark() {
    let g = gaps_mark(&graph_body());
    assert_eq!((g.class, g.text.as_str()), ("gp f", "抜け 2 · ?3"));
    let only_unknown =
        read("../../tests/fixtures/surface/invariants.json").replace("\"violation\"", "\"pass\"");
    let g = gaps_mark(&Fetched::Body(only_unknown.clone()));
    assert_eq!((g.class, g.text.as_str()), ("gp u", "抜け 0 · ?3"));
    let all_pass = only_unknown.replace("\"unknown\"", "\"pass\"");
    let g = gaps_mark(&Fetched::Body(all_pass));
    assert_eq!((g.class, g.text.as_str()), ("gp p", "抜け 0 · ?0"));
    for f in unread() {
        let g = gaps_mark(&f);
        assert_eq!((g.class, g.text.as_str()), ("gp u", "抜け ?"), "{f:?}");
    }
}

/// 帯の語の鍵の語と、設定の中の口と、帯の規則が stylesheet の上端の帯の塊（頁の枠の前）に在ること。
#[test]
fn bvbar_words_and_rules() {
    let v = vocab();
    for (key, want) in [
        (STALL_KEY, "止まった run"),
        (GAPS_KEY, "抜け"),
        (RECENT_KEY, "直近 3h"),
        (GEAR_KEY, "設定"),
        (SEAT_KEY, "席と口座"),
        ("questions", "質問"),
        ("nx_g", "次の一手なし"),
        ("status", "記号の見方"),
        ("stage_target", "表示先"),
    ] {
        assert_eq!(v.term(key).map(|t| t.label.as_str()), Some(want), "{key}");
    }
    assert_eq!(GEAR, [("status", Win::Legend), ("stage_target", Win::Dest)]);
    assert_eq!(BAR, "bar");
    let css = read("style.css");
    let at = css
        .find("#bar { position: fixed; top: 0;")
        .expect("帯の規則");
    let frame = css
        .find("/* ---------- 頁の枠 ---------- */")
        .expect("頁の枠");
    let top = css
        .find("/* ---------- 上端の帯 ---------- */")
        .expect("上端の帯");
    assert!(top < at && at < frame, "帯の規則は上端の帯の塊の末");
    for rule in [
        "#bar .pill.go {",
        "#bar .pill.none {",
        "#bar .chip.org {",
        "#bar .chip.red {",
        "#bar .chip.amb {",
        "#bar .chip.zero {",
        "#bar .notice .nc {",
        "#bar .seatb {",
        "#bar .ub s.w80 {",
        "#bar .gp.f {",
        "#bar .gp.u {",
        "#bar .gp.p {",
        "#bar .gmenu {",
    ] {
        let i = css
            .find(rule)
            .unwrap_or_else(|| panic!("stylesheet に {rule} が無い"));
        assert!(i < frame, "{rule} は頁の枠の前");
    }
}

/// 帯の DOM（wasm の枝）の配線の字: 4 つの口を読み、印を押すと窓を開き、設定は外の click と取り消しの鍵で畳む。
#[test]
fn bvbar_dom_wiring_text() {
    let src = read("src/topbar.rs");
    let dom = &src[src.find("mod dom {").expect("wasm の枝")..];
    for needle in [
        "crate::net::read(next::PATH)",
        "crate::net::read(notice::PATH)",
        "crate::net::read(seat::PATH)",
        "crate::net::read(map::PATH)",
        "wins.open(w, false)",
        "wins.open(win, false)",
        "wins.open(Win::Notices, false)",
        "wins.open(Win::Seat, false)",
        "wins.open(Win::Gaps, false)",
        "bar.wins.open(win, false)",
        "el.closest(\".gearw\")",
        "closes(Hit::Outside)",
        "composing: e.is_composing()",
        "bar.pick.run(m)",
        "bar.back.run(())",
        "<header id=BAR>",
    ] {
        assert!(dom.contains(needle), "wasm の枝に {needle} が無い");
    }
    assert!(read("src/lib.rs").contains("\npub mod topbar;\n"));
}
