//! 行 h-thr-home の歯: account board の HOME の閾値の印・逼迫の強調・断りの理由・移動の段の知らせの行を
//! 電文の caps と notices から描く（面は使った割合と閾値を比べない・R-22）。
#![cfg(test)]

use crate::common::{filled, read};
use tsuzuri_contract::account::{AccountDoc, GroupNotice, WindowCap};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire::decode;
use tsuzuri_surface::account::home::{
    GroupView, MOVES_UNREAD, MvKind, NO_MOVES, cap_of, caps, content, history, home,
    latest_notices, notice_row,
};
use tsuzuri_surface::project::Body;
use tsuzuri_surface::view::Fetched;

fn fixture() -> AccountDoc {
    decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

fn groups(doc: &AccountDoc) -> Vec<GroupView> {
    filled(home(doc).groups)
}

fn group<'a>(gs: &'a [GroupView], name: &str) -> &'a GroupView {
    gs.iter()
        .find(|g| g.name == name)
        .unwrap_or_else(|| panic!("群 {name} が無い"))
}

fn cap_texts(g: &GroupView) -> Vec<&str> {
    g.pressure.iter().map(|p| p.cap.as_str()).collect()
}

fn hots(g: &GroupView) -> Vec<bool> {
    g.pressure.iter().map(|p| p.hot).collect()
}

fn pressure(
    at: u64,
    group: &str,
    account: &str,
    window: &str,
    (used, cap): (u64, u64),
) -> GroupNotice {
    GroupNotice::Pressure {
        at,
        group: group.to_string(),
        account: account.to_string(),
        window: window.to_string(),
        used,
        cap,
        sent: 1,
    }
}

fn refused(at: u64, group: &str, account: &str, reason: &str) -> GroupNotice {
    GroupNotice::Refused {
        at,
        group: group.to_string(),
        account: account.to_string(),
        reason: reason.to_string(),
    }
}

/// 節の組 K（seven_day_model の Known 300 と five_hour の Unknown）。
fn caps_k() -> Vec<WindowCap> {
    vec![
        WindowCap {
            window: "seven_day_model".to_string(),
            rule: "r-m".to_string(),
            cap: Reading::Known(300),
        },
        WindowCap {
            window: "five_hour".to_string(),
            rule: "r-5".to_string(),
            cap: Reading::Unknown,
        },
    ]
}

/// 節の列 M1（口座は acct-9・使った割合が閾値より小さい行も在る）。
fn notices_m1() -> Vec<GroupNotice> {
    vec![
        pressure(500, "Tier2", "acct-9", "seven_day_model", (10, 95)),
        pressure(500, "Tier2", "acct-9", "seven_day", (1, 95)),
        refused(500, "Tier2", "acct-9", "x-word"),
        pressure(400, "Tier1", "acct-9", "seven_day", (99, 95)),
        refused(300, "Tier1", "acct-9", "old-word"),
        pressure(600, "Tier9", "acct-9", "five_hour", (99, 85)),
    ]
}

/// 節の列 M2。
fn notices_m2() -> Vec<GroupNotice> {
    vec![
        refused(100, "Tier1", "acct-9", "old-word"),
        pressure(200, "Tier1", "acct-9", "five_hour", (50, 85)),
    ]
}

fn n1() -> GroupNotice {
    pressure(
        1_790_510_340,
        "Tier1",
        "acct-9",
        "seven_day_model",
        (97, 95),
    )
}

fn n2() -> GroupNotice {
    refused(1_790_510_400, "Tier2", "acct-9", "no-candidate")
}

/// (1) 閾値は電文の caps の行の写し（窓の名が等しい最初の行・無いか読めなければ None で字 / ?）。
#[test]
fn hthr_caps_copy_rows() {
    let d = fixture();
    let want = vec![Some(85), Some(95), Some(95)];
    assert_eq!(caps(&d), want);
    assert_eq!(home(&d).caps, want);
    let gs = groups(&d);
    assert_eq!(cap_texts(group(&gs, "Tier1")), vec!["/ 85", "/ 95", "/ 95"]);

    let mut k = d.clone();
    k.caps = caps_k();
    assert_eq!(cap_of(&k.caps, "five_hour"), None);
    assert_eq!(cap_of(&k.caps, "seven_day"), None);
    assert_eq!(cap_of(&k.caps, "seven_day_model"), Some(300));
    assert_eq!(caps(&k), vec![None, None, Some(300)]);
    let gs = groups(&k);
    assert_eq!(cap_texts(group(&gs, "Tier2")), vec!["/ ?", "/ ?", "/ 300"]);

    assert!(content(&Fetched::Failed, d.at).caps.is_empty());
}

/// (2) 強調と断りの理由は群の最も新しい知らせの組だけから出す（比べの式を持たない）。
#[test]
fn hthr_marks_from_latest_notices() {
    let d = fixture();
    let gs = groups(&d);
    let (t1, t2) = (group(&gs, "Tier1"), group(&gs, "Tier2"));
    assert_eq!(hots(t1), vec![true, false, false]);
    assert_eq!(t1.refused, None);
    // Tier2 の 5h は 100% で閾値 85 を越えても知らせの行が無いので強調しない。
    assert_eq!(t2.pressure[0].used, "100%");
    assert_eq!(hots(t2), vec![false, false, false]);
    assert_eq!(t2.refused.as_deref(), Some("no-candidate"));

    let mut m1 = d.clone();
    m1.notices = Reading::Known(notices_m1());
    let gs = groups(&m1);
    let (t1, t2) = (group(&gs, "Tier1"), group(&gs, "Tier2"));
    assert_eq!(hots(t1), vec![false, true, false]);
    assert_eq!(t1.refused, None);
    assert_eq!(hots(t2), vec![false, true, true]);
    assert_eq!(t2.refused.as_deref(), Some("x-word"));
    let latest = latest_notices(&m1.notices, "Tier2");
    assert_eq!(latest.len(), 3);
    assert_eq!(latest, notices_m1().iter().take(3).collect::<Vec<_>>());
    assert!(latest_notices(&m1.notices, "Tier3").is_empty());

    latest_m2_and_unknown(d);
}

/// 節の列 M2 と読めない知らせの列の強調と断りの理由。
fn latest_m2_and_unknown(d: AccountDoc) {
    let mut m2 = d.clone();
    m2.notices = Reading::Known(notices_m2());
    let gs = groups(&m2);
    let t1 = group(&gs, "Tier1");
    assert_eq!(hots(t1), vec![true, false, false]);
    assert_eq!(t1.refused, None);

    let mut u = d.clone();
    u.notices = Reading::Unknown;
    for g in groups(&u) {
        assert_eq!(hots(&g), vec![false, false, false], "{}", g.name);
        assert_eq!(g.refused, None, "{}", g.name);
    }
    assert!(latest_notices(&u.notices, "Tier1").is_empty());
}

/// (3) 知らせの行は移動の段の行になり、移動の記録と at の新しい順に 1 つの列で畳む。
#[test]
fn hthr_history_rows() {
    let d = fixture();
    let r1 = notice_row(&n1(), d.at);
    assert_eq!(r1.at, "20:59 JST");
    assert_eq!(r1.group, "Tier1");
    assert_eq!(r1.from, "acct-9");
    assert_eq!(r1.to, "");
    assert_eq!(r1.kind, MvKind::Pressure("model 97% ≥ 95".to_string()));
    let r2 = notice_row(&n2(), d.at);
    assert_eq!(r2.kind, MvKind::Refused("no-candidate".to_string()));
    assert_eq!(r2.at, "21:00 JST");

    let Reading::Known(moves) = &d.moves else {
        panic!("fixture の moves は読める");
    };
    let h = history(
        moves,
        &[refused(1_790_503_200, "Tier2", "acct-9", "w"), n1()],
        d.at,
    );
    let got: Vec<(&str, &MvKind)> = h.shown.iter().map(|r| (r.group.as_str(), &r.kind)).collect();
    assert_eq!(
        got,
        vec![
            ("Tier1", &MvKind::Pressure("model 97% ≥ 95".to_string())),
            ("Tier1", &MvKind::Moved),
            ("Tier2", &MvKind::Refused("w".to_string())),
            ("Tier2", &MvKind::Moved),
        ]
    );
    assert!(h.folded.is_empty());
    assert_eq!(h.more, None);

    folded_and_home_moves(d);
}

/// 9 行の知らせの畳みと、頁の移動の欄の知らせの行・空・読めない。
fn folded_and_home_moves(d: AccountDoc) {
    let nine: Vec<GroupNotice> = (0..9)
        .map(|i| refused(d.at - 60 * i, "Tier1", "acct-9", "w"))
        .collect();
    let h = history(&[], &nine, d.at);
    assert_eq!(h.shown.len(), 8);
    assert_eq!(h.folded.len(), 1);
    assert_eq!(h.more.as_deref(), Some("+1"));

    let mut only = d.clone();
    only.moves = Reading::Known(vec![]);
    only.notices = Reading::Known(vec![n1()]);
    let m = filled(home(&only).moves);
    assert_eq!(m.shown.len(), 1);
    assert_eq!(m.shown[0].kind, MvKind::Pressure("model 97% ≥ 95".to_string()));

    let mut u = d.clone();
    u.notices = Reading::Unknown;
    let m = filled(home(&u).moves);
    assert_eq!(m.shown.len(), 2);
    assert!(m.shown.iter().all(|r| r.kind == MvKind::Moved), "{m:?}");
    u.moves = Reading::Known(vec![]);
    assert_eq!(home(&u).moves, Body::Empty(NO_MOVES));
    u.moves = Reading::Unknown;
    assert_eq!(home(&u).moves, Body::Unmeasured(MOVES_UNREAD));
}

/// (4) DOM の部分は閾値の線・強調・閾値の字・断りの理由・知らせの行を描き、class は stylesheet に在る。
#[test]
fn hthr_dom_wiring() {
    let src = read("src/account/home.rs");
    let dom = &src[src.find("mod dom {").expect("mod dom が在る")..];
    for s in [
        r#"<span class="cap" style=format!("left:{c}%")></span>"#,
        r#"if p.hot { "pw hot" } else { "pw" }"#,
        r#"<span class="cap num">{p.cap}</span>"#,
        r#"<span class="why">{format!("GroupMoveRefused {r}")}</span>"#,
        r#"<span class="ic pressure" aria-hidden="true">"!"</span>"#,
        r#"<span class="ic refused" aria-hidden="true">"×"</span>"#,
        r#"<span class="sub">"GroupPressureNotified"</span>"#,
        r#"<span class="sub">{format!("GroupMoveRefused {reason}")}</span>"#,
        "meter(w, caps.get(i).copied().flatten())",
    ] {
        assert_eq!(dom.matches(s).count(), 1, "DOM に {s} が 1 度でない");
    }
    let css = read("style.css");
    for c in [
        ".meter .bar .cap",
        ".pw .cap",
        ".pw.hot",
        ".mv .ic.pressure",
        ".mv .ic.refused",
        ".why",
    ] {
        assert!(css.contains(c), "style.css に {c} が無い");
    }
}

/// (5) この file の歯は 5 つで、名はどれも hthr_ で始まる。
#[test]
fn hthr_own_names() {
    let me = read("tests/teeth3/hthr.rs");
    let lines: Vec<&str> = me.lines().map(str::trim).collect();
    let mut n = 0;
    for (i, l) in lines.iter().enumerate() {
        if *l != "#[test]" {
            continue;
        }
        let name = lines[i + 1]
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("test の属性の次が fn でない: {}", lines[i + 1]));
        assert!(name.starts_with("hthr_"), "{name} が hthr_ で始まらない");
        n += 1;
    }
    assert_eq!(n, 5);
}
