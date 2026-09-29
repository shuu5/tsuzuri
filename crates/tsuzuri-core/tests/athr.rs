//! 窓ごとの逼迫の閾値と群の逼迫の知らせと移動の断りの歯（設計ノート surface-wave7 便 c-acct-thr・接頭辞 athr_）。
//! 閾値は器の rules 行の出力の字を、知らせと断りは器の event log の行を写すだけで、tsuzuri は判じない。
//! fixture: tests/fixtures/account/acct-doc.json（読むだけ）。event log の字は歯の中で組む。今は 2026-09-27T12:00:00Z。

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::{Value, json};
use tsuzuri_contract::account::{AccountDoc, GroupNotice, WindowCap};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire::{decode, encode};
use tsuzuri_core::account::host::{
    self, CAP_ROWS, HostTexts, PRESSURE_EVENT, REFUSED_EVENT, WINDOW_WORDS,
};
use tsuzuri_core::account::project::{self, ProjectTexts, assemble, project_rows, session_lines};

/// 今（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 群の宣言の字（Tier1 の anchors は proj-a と proj-b・Tier2 は proj-c）。
const HOST: &str = "[[account-group]]\nname = \"Tier1\"\nanchors = [\"/work/proj-a\", \"/work/proj-b\"]\n\n\
[[account-group]]\nname = \"Tier2\"\nanchors = [\"/work/proj-c\"]\n";

const UNKNOWN: Reading<u64> = Reading::Unknown;

const PRESSURE: &str = "GroupPressureNotified";
const REFUSED: &str = "GroupMoveRefused";

/// event の 1 行（ts は `HH:MM` なら 2026-09-27 の UTC・ほかの字はそのまま・account が None なら欄を置かない）。
fn ev(ts: &str, kind: &str, account: Option<&str>, detail: &str) -> String {
    let ts = if ts.contains(':') {
        format!("2026-09-27T{ts}:00Z")
    } else {
        ts.to_string()
    };
    let mut event = json!({"schema": 1, "ts": ts, "kind": kind, "actor": "machine", "detail": detail});
    if let Some(a) = account {
        event["account"] = json!(a);
    }
    event.to_string()
}

fn log(lines: &[String]) -> String {
    lines.iter().map(|l| format!("{l}\n")).collect()
}

fn l1() -> String {
    log(&[
        ev("11:30", PRESSURE, Some("acct-2"), "group=Tier1 window=5h used=90 cap=85 sent=1"),
        ev("11:40", REFUSED, Some("acct-3"), "group=Tier2 reason=no-candidate"),
        ev("11:40", PRESSURE, Some("acct-1"), "group=Tier1 window=model used=97 cap=95 sent=2"),
        // 宣言に無い群。
        ev("11:45", PRESSURE, Some("acct-2"), "group=Tier9 window=7d used=99 cap=95 sent=1"),
        // 窓の語でない。
        ev("11:46", PRESSURE, Some("acct-2"), "group=Tier1 window=1h used=99 cap=95 sent=1"),
        // 数でない。
        ev("11:47", PRESSURE, Some("acct-2"), "group=Tier1 window=7d used=lots cap=95 sent=1"),
        // sent の無い行。
        ev("11:44", PRESSURE, Some("acct-2"), "group=Tier1 window=7d used=99 cap=95"),
        // ts の読めない行。
        ev("soon", REFUSED, Some("acct-3"), "group=Tier2 reason=no-candidate"),
        // account の無い行。
        ev("11:48", REFUSED, None, "group=Tier2 reason=no-candidate"),
        // reason の無い行。
        ev("11:49", REFUSED, Some("acct-3"), "group=Tier2"),
        ev("11:50", "GroupMoved", Some("acct-2"), "name"),
        "{".to_string(),
        ev("11:51", "GroupMovePending", Some("acct-2"), "group=Tier1 reason=not-a-shell"),
    ])
}

fn l2() -> String {
    log(&[
        ev("11:40", REFUSED, Some("acct-3"), "group=Tier2 reason=no-candidate"),
        ev("11:55", REFUSED, Some("acct-1"), "group=Tier1 reason=no-candidate"),
        ev("11:30", PRESSURE, Some("acct-2"), "group=Tier1 window=5h used=91 cap=85 sent=1"),
        ev("11:20", PRESSURE, Some("acct-1"), "group=Tier2 window=7d used=96 cap=95 sent=1"),
    ])
}

fn l3() -> String {
    log(&[ev("11:59", REFUSED, Some("acct-3"), "group=Tier2 reason=no-candidate")])
}

fn l4() -> String {
    log(&[ev("11:58", PRESSURE, Some("acct-1"), "group=Tier1 window=5h used=99 cap=85 sent=1")])
}

fn pressure(at: u64, group: &str, account: &str, window: &str, used: u64, cap: u64, sent: u64) -> GroupNotice {
    GroupNotice::Pressure {
        at,
        group: group.into(),
        account: account.into(),
        window: window.into(),
        used,
        cap,
        sent,
    }
}

fn refused(at: u64, group: &str, account: &str, reason: &str) -> GroupNotice {
    GroupNotice::Refused {
        at,
        group: group.into(),
        account: account.into(),
        reason: reason.into(),
    }
}

/// 節の表（HOST と L1 と L2 の順の知らせ・6 行）。
fn table() -> Vec<GroupNotice> {
    vec![
        refused(1_790_510_100, "Tier1", "acct-1", "no-candidate"),
        pressure(1_790_509_200, "Tier1", "acct-1", "seven_day_model", 97, 95, 2),
        refused(1_790_509_200, "Tier2", "acct-3", "no-candidate"),
        pressure(1_790_508_600, "Tier1", "acct-2", "five_hour", 90, 85, 1),
        pressure(1_790_508_600, "Tier1", "acct-2", "five_hour", 91, 85, 1),
        pressure(1_790_508_000, "Tier2", "acct-1", "seven_day", 96, 95, 1),
    ]
}

fn host_texts() -> HostTexts {
    HostTexts {
        host_toml: Some(HOST.to_string()),
        ..HostTexts::default()
    }
}

fn caps_of(rows: &[(&str, &str)]) -> HostTexts {
    HostTexts {
        caps: rows
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..HostTexts::default()
    }
}

/// 閾値の列の値だけ（CAP_ROWS の順の窓と行の id を見たうえで）。
fn cap_values(caps: &[WindowCap]) -> Vec<Reading<u64>> {
    let rows: Vec<(&str, &str)> = caps
        .iter()
        .map(|c| (c.window.as_str(), c.rule.as_str()))
        .collect();
    assert_eq!(rows, CAP_ROWS);
    caps.iter().map(|c| c.cap.clone()).collect()
}

fn fixture() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/account/acct-doc.json");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

/// 字の中の、引用符で囲んだ鍵の名とコロンをつないだ字の最初の位置。
fn key_at(text: &str, key: &str) -> Option<usize> {
    text.find(&format!("\"{key}\":"))
}

fn is_eq<T: Eq>(_: &T) {}

#[test]
fn athr_wire_shape_pinned() {
    let doc: AccountDoc = decode(&fixture()).expect("fixture の形");
    let text = encode(&doc).expect("電文");
    let keys = [
        "accounts", "caps", "groups", "moves", "notices", "projects", "sessions", "dormant",
    ];
    let at: Vec<usize> = keys
        .iter()
        .map(|k| key_at(&text, k).unwrap_or_else(|| panic!("鍵 {k} が無い")))
        .collect();
    assert!(at.windows(2).all(|w| w[0] < w[1]), "鍵の順: {at:?}");
    // fixture の閾値と知らせ。
    assert_eq!(
        doc.caps,
        vec![
            WindowCap {
                window: "five_hour".into(),
                rule: "fleet.group_pressure_5h_pct".into(),
                cap: Reading::Known(85),
            },
            WindowCap {
                window: "seven_day".into(),
                rule: "fleet.group_pressure_7d_pct".into(),
                cap: Reading::Known(95),
            },
            WindowCap {
                window: "seven_day_model".into(),
                rule: "fleet.group_pressure_model_pct".into(),
                cap: Reading::Known(95),
            },
        ]
    );
    assert_eq!(
        doc.notices,
        Reading::Known(vec![
            refused(1_790_509_800, "Tier2", "acct-2", "no-candidate"),
            pressure(1_790_508_600, "Tier1", "acct-2", "five_hour", 100, 85, 1),
        ])
    );
    // 知らせの行の鍵の順と kind の値。
    let n_p = pressure(1_790_508_600, "Tier1", "acct-2", "five_hour", 90, 85, 1);
    let n_r = refused(1_790_509_200, "Tier2", "acct-3", "no-candidate");
    for (n, keys, kind) in [
        (
            &n_p,
            &["kind", "at", "group", "account", "window", "used", "cap", "sent"][..],
            "pressure",
        ),
        (&n_r, &["kind", "at", "group", "account", "reason"][..], "refused"),
    ] {
        let text = encode(n).expect("電文");
        let at: Vec<usize> = keys
            .iter()
            .map(|k| {
                let pat = format!("\"{k}\":");
                assert_eq!(text.matches(&pat).count(), 1, "鍵 {k} は 1 度: {text}");
                key_at(&text, k).expect("鍵")
            })
            .collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]), "鍵の順: {text}");
        let v: Value = serde_json::from_str(&text).expect("JSON");
        assert_eq!(v["kind"], kind);
        assert_eq!(&decode::<GroupNotice>(&text).expect("戻る"), n);
    }
    let moved = encode(&n_r)
        .expect("電文")
        .replace("\"kind\":\"refused\"", "\"kind\":\"moved\"");
    assert!(moved.contains("\"moved\""));
    assert!(decode::<GroupNotice>(&moved).is_err(), "閉じた 2 つの種類");
    let unknown = WindowCap {
        window: "five_hour".into(),
        rule: "fleet.group_pressure_5h_pct".into(),
        cap: Reading::Unknown,
    };
    let v: Value = serde_json::from_str(&encode(&unknown).expect("電文")).expect("JSON");
    assert_eq!(v["cap"], "unknown");
    is_eq(&n_p);
}

#[test]
fn athr_caps_copy_rules_rows() {
    assert_eq!(
        CAP_ROWS,
        [
            ("five_hour", "fleet.group_pressure_5h_pct"),
            ("seven_day", "fleet.group_pressure_7d_pct"),
            ("seven_day_model", "fleet.group_pressure_model_pct"),
        ]
    );
    let c1 = caps_of(&[
        ("fleet.group_pressure_5h_pct", "70\n"),
        ("fleet.group_pressure_7d_pct", "  88  "),
        ("fleet.group_pressure_model_pct", "soon"),
    ]);
    assert_eq!(
        cap_values(&host::caps(&c1)),
        [Reading::Known(70), Reading::Known(88), Reading::Unknown]
    );
    let c2 = caps_of(&[
        ("fleet.group_pressure_5h_pct", ""),
        ("fleet.group_pressure_7d_pct", "-1"),
        ("fleet.group_pressure_model_pct", "300"),
    ]);
    assert_eq!(
        cap_values(&host::caps(&c2)),
        [Reading::Unknown, Reading::Unknown, Reading::Known(300)]
    );
    let c3 = caps_of(&[("seat.move_grace_s", "1800")]);
    for texts in [HostTexts::default(), c3] {
        assert_eq!(cap_values(&host::caps(&texts)), [UNKNOWN; 3]);
    }
}

#[test]
fn athr_notices_copy_events() {
    assert_eq!(PRESSURE_EVENT, PRESSURE);
    assert_eq!(REFUSED_EVENT, REFUSED);
    assert_eq!(
        WINDOW_WORDS,
        [
            ("5h", "five_hour"),
            ("7d", "seven_day"),
            ("model", "seven_day_model"),
        ]
    );
    let h = host_texts();
    let (a, b) = (l1(), l2());
    assert_eq!(host::notices(&h, &[&a, &b]), Reading::Known(table()));
    let mut swapped = table();
    swapped.swap(3, 4);
    assert_eq!(host::notices(&h, &[&b, &a]), Reading::Known(swapped));
    assert_eq!(host::notices(&h, &[&a]), Reading::Known(table()[1..4].to_vec()));
    // 同じ log を 2 度渡しても 1 度だけ。
    assert_eq!(host::notices(&h, &[&a, &b, &a]), Reading::Known(table()));
    assert_eq!(host::notices(&h, &[""]), Reading::Known(Vec::new()));
    assert_eq!(host::notices(&h, &[]), Reading::Unknown);
    assert_eq!(host::notices(&HostTexts::default(), &[&a]), Reading::Unknown);
}

fn project_map() -> BTreeMap<String, ProjectTexts> {
    let p = |known: bool, events: String| ProjectTexts {
        state_dir_known: known,
        events: Some(events),
        ..ProjectTexts::default()
    };
    BTreeMap::from([
        ("/work/proj-a".to_string(), p(true, l1())),
        ("/work/proj-b".to_string(), p(true, l2())),
        ("/work/proj-c".to_string(), p(false, l3())),
        ("/work/proj-z".to_string(), p(true, l4())),
    ])
}

#[test]
fn athr_doc_carries_both() {
    let h = HostTexts {
        caps: BTreeMap::from([("fleet.group_pressure_5h_pct".to_string(), "70".to_string())]),
        ..host_texts()
    };
    let ps = project_map();
    let d = project::doc(&h, &ps, NOW);
    assert_eq!(
        cap_values(&d.caps),
        [Reading::Known(70), Reading::Unknown, Reading::Unknown]
    );
    assert_eq!(d.caps, host::caps(&h));
    let (a, b) = (l1(), l2());
    assert_eq!(d.notices, host::notices(&h, &[&a, &b]));
    assert_eq!(d.notices, Reading::Known(table()));
    // ほかの欄は assemble に同じ入力を渡した電文のまま。
    let base = assemble(
        host::accounts(&h),
        host::groups(&h),
        host::moves(&h),
        project_rows(&h, &ps, NOW),
        session_lines(&h, &ps, NOW),
    );
    let back = AccountDoc {
        caps: base.caps.clone(),
        notices: base.notices.clone(),
        ..d
    };
    assert_eq!(back, base);
}

#[test]
fn athr_assemble_leaves_unknown() {
    let d = assemble(
        Reading::Unknown,
        Reading::Unknown,
        Reading::Unknown,
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(cap_values(&d.caps), [UNKNOWN; 3]);
    assert_eq!(d.caps, host::caps(&HostTexts::default()));
    assert_eq!(d.notices, Reading::Unknown);
}

/// 着地済みの filter の語（歯の名から先頭の athr_ を除いた字はどれも含まない）。
const WORDS: [&str; 92] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
    "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_",
    "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_",
    "pmore_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "hcard_", "qgate_",
    "nsum_", "ntime_", "ptitle_", "ncard_", "hsym_", "smore_", "lcard_", "aaround_", "hruling_",
    "sxaxis_", "plimit_", "bport_", "fmark_", "fstop_", "fserve_", "nsumw_", "cadopt_", "tipx_",
    "hcsess_", "hcproj_", "sesplit_", "mstore_", "qblock_",
];

#[test]
fn athr_own_names_clean() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/athr.rs");
    let text = std::fs::read_to_string(&path).expect("自分の file");
    let mut lines = text.lines().map(str::trim);
    let mut names = Vec::new();
    while let Some(line) = lines.next() {
        if line != "#[test]" {
            continue;
        }
        let f = lines.next().expect("属性の次の行");
        let name = f
            .strip_prefix("fn ")
            .and_then(|r| r.split_once('('))
            .map(|(n, _)| n)
            .unwrap_or_else(|| panic!("属性の次が fn でない: {f}"));
        names.push(name.to_string());
    }
    assert_eq!(names.len(), 6, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("athr_")
            .unwrap_or_else(|| panic!("接頭辞: {name}"));
        for w in WORDS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
