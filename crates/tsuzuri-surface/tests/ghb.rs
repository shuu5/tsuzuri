//! 行 g-seat-hb の歯: project board の席の block の停止の切り替え（席の card からの切り替え・Top の欄・
//! 送り先の口と本文・DOM の繋ぎの字）と、この file の歯の名。fixture は tests/fixtures/surface/seat-card.json（読むだけ）。

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::account::{self as contract_account, Heartbeat};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::seathb::{self, SeatHeartbeatRequest};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::heartbeat::{
    Dest, HEARTBEAT_PATH, SEAT_HEARTBEAT_PATH, Toggle, panel, seat_toggle,
};
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::seat::content;
use tsuzuri_surface::view::Fetched;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の 5 組（組の名 → 電文の型）。
fn fixture() -> BTreeMap<String, SeatCard> {
    wire::decode(&read("../../tests/fixtures/surface/seat-card.json"))
        .expect("fixture の組が電文として読める")
}

fn card(name: &str) -> SeatCard {
    fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"))
}

/// fixture の席の切り替えの期待（project は空の字・席の名は tsuzuri-orch・撃つ向きは今の逆）。
fn want(now: Heartbeat, to: Heartbeat) -> Toggle {
    Toggle {
        project: String::new(),
        target: "tsuzuri-orch".to_string(),
        now,
        to,
    }
}

/// 字 `from` の在る所から、その後で最初の `end` までの字（`from` が無ければ落ちる）。
fn span_of<'a>(text: &'a str, from: &str, end: &str) -> &'a str {
    let at = text
        .find(from)
        .unwrap_or_else(|| panic!("字 {from} が無い"));
    let rest = &text[at..];
    let stop = rest
        .find(end)
        .unwrap_or_else(|| panic!("字 {from} の後に終わりの並びが無い"));
    &rest[..stop]
}

/// (1) 席の card の切り替えは席の名が在り heartbeat が読めるときだけ在り、撃つ字は account board と同じ形。
#[test]
fn ghb_card_toggle_on_fixture() {
    let names: Vec<String> = fixture().into_keys().collect();
    assert_eq!(names, vec!["limit", "run", "silent", "unknown", "wait"]);

    let cases: [(&str, Option<Toggle>); 5] = [
        ("run", Some(want(Heartbeat::On, Heartbeat::Off))),
        ("limit", Some(want(Heartbeat::On, Heartbeat::Off))),
        ("wait", Some(want(Heartbeat::Off, Heartbeat::On))),
        ("silent", None),
        ("unknown", None),
    ];
    for (name, expect) in cases {
        assert_eq!(seat_toggle(&card(name)), expect, "組 {name}");
    }

    let mut no_target = card("run");
    no_target.target = String::new();
    assert_eq!(seat_toggle(&no_target), None);
    let mut hb_unknown = card("run");
    hb_unknown.heartbeat = Reading::Unknown;
    assert_eq!(seat_toggle(&hb_unknown), None);

    let r = seat_toggle(&card("run")).expect("run");
    let w = seat_toggle(&card("wait")).expect("wait");
    assert_eq!(r.project, "");
    assert_eq!(r.button_key(), "hb_to_off");
    assert_eq!(w.button_key(), "hb_to_on");
    assert_eq!(
        panel(&r).command,
        "scribe2 seat heartbeat off --target tsuzuri-orch"
    );
    assert_eq!(
        panel(&w).command,
        "scribe2 seat heartbeat on --target tsuzuri-orch"
    );
}

/// (2) block の中身の Top の toggle は、同じ card を seat_toggle に渡した値。
#[test]
fn ghb_top_carries_toggle() {
    for (name, c) in fixture() {
        let text = wire::encode(&c).expect("電文");
        let seat = match content(&Fetched::Body(text), c.at) {
            Body::Filled(s) => s,
            other => panic!("組 {name} が中身を出さない: {other:?}"),
        };
        assert_eq!(seat.top.toggle, seat_toggle(&c), "組 {name}");
    }
    assert!(
        matches!(
            content(
                &Fetched::Body(wire::encode(&card("run")).expect("電文")),
                card("run").at
            ),
            Body::Filled(s) if s.top.toggle.is_some()
        ),
        "run の Top は切り替えを持つ"
    );
}

fn copied<T: Copy + Eq + std::fmt::Debug>(v: T) -> (T, String) {
    let c = v;
    (c, format!("{v:?}"))
}

/// (3) 席の口の本文は向きだけの SeatHeartbeatRequest・account の口の本文は request_body・口の path は契約の型の定数。
#[test]
fn ghb_seat_body_and_path() {
    let r = seat_toggle(&card("run")).expect("run");
    let w = seat_toggle(&card("wait")).expect("wait");
    assert_eq!(Dest::Seat.body(&r), r#"{"to":"off"}"#);
    assert_eq!(Dest::Seat.body(&w), r#"{"to":"on"}"#);
    assert_eq!(
        wire::decode::<SeatHeartbeatRequest>(&Dest::Seat.body(&r)).expect("本文が要求として読める"),
        SeatHeartbeatRequest { to: Heartbeat::Off }
    );
    assert_eq!(
        wire::decode::<SeatHeartbeatRequest>(&Dest::Seat.body(&w)).expect("本文が要求として読める"),
        SeatHeartbeatRequest { to: Heartbeat::On }
    );
    for t in [&r, &w] {
        assert!(!Dest::Seat.body(t).contains("tsuzuri-orch"));
    }

    let a = Toggle {
        project: "proj-a".to_string(),
        ..r.clone()
    };
    assert_eq!(Dest::Account.body(&a), a.request_body());
    assert_eq!(Dest::Account.body(&a), r#"{"project":"proj-a","to":"off"}"#);

    assert_eq!(SEAT_HEARTBEAT_PATH, seathb::PATH);
    assert_eq!(HEARTBEAT_PATH, contract_account::HEARTBEAT_PATH);

    let (acc, acc_dbg) = copied(Dest::Account);
    let (seat, seat_dbg) = copied(Dest::Seat);
    assert_eq!(acc, Dest::Account);
    assert_eq!(seat, Dest::Seat);
    assert_ne!(acc, seat);
    assert_eq!((acc_dbg.as_str(), seat_dbg.as_str()), ("Account", "Seat"));

    let src = read("src/account/heartbeat.rs");
    for call in [
        "crate::net::post(HEARTBEAT_PATH, body)",
        "crate::net::post(SEAT_HEARTBEAT_PATH, body)",
    ] {
        assert_eq!(src.matches(call).count(), 1, "heartbeat.rs の {call} の数");
    }
}

/// (4) 段の本文は below_to に移り、席の block の top_view が button と below_to を呼び、view が States を作る。
#[test]
fn ghb_dom_wiring_text() {
    let hb = read("src/account/heartbeat.rs");
    assert!(hb.contains("pub fn below_to("));
    let below = span_of(&hb, "pub fn below(", "\n    }");
    assert!(
        below.contains("below_to(Dest::Account, t, states)"),
        "below の本文: {below}"
    );

    let dom = read("src/project_dom/seat.rs");
    let top = span_of(&dom, "fn top_view(", "\n}\n");
    for call in ["heartbeat::button(", "heartbeat::below_to(Dest::Seat,"] {
        assert_eq!(dom.matches(call).count(), 1, "{call} の数");
        assert!(top.contains(call), "top_view の本文に {call} が無い");
    }
    let view = span_of(&dom, "pub fn view() -> AnyView {", "\n}\n");
    assert!(view.contains("States = RwSignal::new("), "view の本文: {view}");
}

/// filter の語（着地済みの行の verify の filter の語と、第 3 波から第 8 波の行の接頭辞・この行の接頭辞は並べない）。
const FILTER: [&str; 106] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_", "server_",
    "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_", "mkeys_",
    "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_", "pmore_",
    "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "qgate_", "hcard_", "ntime_",
    "ptitle_", "ncard_", "hsym_", "smore_", "lcard_", "aaround_", "hruling_", "sxaxis_",
    "plimit_", "bport_", "fstop_", "nsum_", "nsumw_", "fmark_", "fserve_", "cadopt_", "tipx_",
    "hcsess_", "hcproj_", "sesplit_", "mstore_", "athr_", "qblock_", "wsteady_", "gsum_",
    "nstall_", "pfold_", "uword_", "cround_", "cgdom_", "csled_", "lhome_", "shb_", "nact_",
    "aord_", "mtree_",
];

/// (7) この file の歯の名は ghb_ で始まり、残りの字は filter の語を含まない。
#[test]
fn ghb_own_names_clean() {
    let me = read("tests/ghb.rs");
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
            .strip_prefix("ghb_")
            .unwrap_or_else(|| panic!("{name} が ghb_ で始まらない"));
        for w in FILTER {
            assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
        }
        n += 1;
    }
    assert_eq!(n, 5);
}
