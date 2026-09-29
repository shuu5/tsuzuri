//! 行 c-seat-grace の歯（面）: 席の card の器の移動の 4 つの欄（move_to・grace_until・refused・pressure）の
//! 電文の鍵と、project board の席の block の状態の帯の猶予の内・移り先の無い断り・逼迫の行と、
//! 詳しくの写しの行（器の鍵の名・`-` と `?`）。残り秒は終わる時刻から card の at を引いた値（行 c-abs-seat）。
//! fixture: tests/fixtures/surface/seat-card.json（読むだけ・4 つの鍵を持たない）。器の欄は歯の中で組む。

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{Pressure, SeatCard};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::home::{EXPERT_CHARS, wrap_words};
use tsuzuri_surface::project::seat::{
    self, Band, GRACE_NEXT, LIMIT_LINE, MOVE_WAIT, MOVING, NEXT_TARGET, PRESSED, REFUSED_NEXT,
    copied_line, hmd, more,
};
use tsuzuri_surface::vocab::label;

/// fixture の組の時点（2026-09-27 12:00Z）。
const AT: u64 = 1_790_510_400;

/// 同じ日本の日の断りの時刻（2026-09-27 09:00Z）。
const REFUSED_AT: u64 = 1_790_503_200;

/// 前の日本の日の断りの時刻。
const REFUSED_OLD: u64 = 1_790_410_400;

/// 4 つの欄の電文の鍵。
const KEYS: [&str; 4] = ["move_to", "grace_until", "refused", "pressure"];

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_text() -> String {
    read("../../tests/fixtures/surface/seat-card.json")
}

/// fixture の 5 組（組の名 → 電文の型）。
fn fixture() -> BTreeMap<String, SeatCard> {
    wire::decode(&fixture_text()).expect("fixture の組が電文として読める")
}

fn card(name: &str) -> SeatCard {
    fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"))
}

fn pressure(window: &str, used: u32, cap: u32) -> Pressure {
    Pressure {
        window: window.to_string(),
        used,
        cap,
    }
}

/// 猶予の内の card（器が `to` へ移し、残り `left` 秒）。
fn grace(name: &str, to: &str, left: u64) -> SeatCard {
    let mut c = card(name);
    c.move_to = Reading::Known(Some(to.to_string()));
    c.grace_until = Reading::Known(Some(c.at + left));
    c
}

fn refused_line(at: u64) -> String {
    format!("{} ◷ {}", label("no_target"), hmd(at, AT))
}

#[test]
fn sgrace_wire_keys() {
    let text = fixture_text();
    for key in KEYS {
        assert!(!text.contains(&format!("\"{key}\"")), "fixture に鍵 {key}");
    }
    let fx = fixture();
    assert_eq!(fx.len(), 5);
    for (name, c) in &fx {
        assert_eq!(c.move_to, Reading::Unknown, "組 {name}");
        assert_eq!(c.grace_until, Reading::Unknown, "組 {name}");
        assert_eq!(c.refused, Reading::Unknown, "組 {name}");
        assert_eq!(c.pressure, Reading::Unknown, "組 {name}");
        let t = wire::encode(c).expect("電文");
        for key in KEYS {
            assert!(t.contains(&format!("\"{key}\":\"unknown\"")), "組 {name} の {key}");
        }
    }

    let mut full = grace("run", "acct-5", 120);
    full.refused = Reading::Known(Some(REFUSED_AT));
    full.pressure = Reading::Known(Some(pressure("5h", 92, 85)));
    let t = wire::encode(&full).expect("電文");
    for want in [
        r#""move_to":{"known":"acct-5"}"#,
        r#""grace_until":{"known":1790510520}"#,
        r#""refused":{"known":1790503200}"#,
        r#""pressure":{"known":{"window":"5h","used":92,"cap":85}}"#,
    ] {
        assert!(t.contains(want), "{want} が無い: {t}");
    }
    assert_eq!(wire::decode::<SeatCard>(&t).expect("往復"), full);

    let mut none = card("run");
    none.move_to = Reading::Known(None);
    none.grace_until = Reading::Known(None);
    none.refused = Reading::Known(None);
    none.pressure = Reading::Known(None);
    let t = wire::encode(&none).expect("電文");
    for key in KEYS {
        assert!(t.contains(&format!("\"{key}\":{{\"known\":null}}")), "{key}: {t}");
    }
    assert_eq!(wire::decode::<SeatCard>(&t).expect("往復"), none);
}

#[test]
fn sgrace_band_grace() {
    assert!(MOVING.starts_with("移動中"));
    assert!(GRACE_NEXT.contains("/exit"));
    // 移動待ちの組: 猶予の内の行が移動待ちの行に代わり、2 行目は移る先の口座。
    assert_eq!(
        seat::band(&grace("wait", "acct-5", 120)),
        Some(Band {
            l1: vec![format!("{MOVING} 120 秒")],
            l2: Some(format!("{GRACE_NEXT} acct-5")),
        })
    );
    // 限度の組: 限度の行の後に猶予の内の行（残り 0 も器の字のまま）。
    assert_eq!(
        seat::band(&grace("limit", "acct-6", 0)),
        Some(Band {
            l1: vec![LIMIT_LINE.to_string(), format!("{MOVING} 0 秒")],
            l2: Some(format!("{GRACE_NEXT} acct-6")),
        })
    );
    // 平時の組も猶予の内なら帯を開く。
    assert_eq!(
        seat::band(&grace("run", "acct-5", 1700)),
        Some(Band {
            l1: vec![format!("{MOVING} 1700 秒")],
            l2: Some(format!("{GRACE_NEXT} acct-5")),
        })
    );
    // どちらかが Known(None) か Unknown なら着地の帯のまま。
    let landed = |name: &str| seat::band(&card(name));
    assert_eq!(
        landed("wait"),
        Some(Band {
            l1: vec![format!("{MOVE_WAIT} acct-4 → acct-5")],
            l2: None,
        })
    );
    type Half = (Reading<Option<String>>, Reading<Option<u64>>);
    let halves: [Half; 5] = [
        (Reading::Known(Some("acct-5".into())), Reading::Known(None)),
        (Reading::Known(Some("acct-5".into())), Reading::Unknown),
        (Reading::Known(None), Reading::Known(Some(120))),
        (Reading::Unknown, Reading::Known(Some(120))),
        (Reading::Known(None), Reading::Known(None)),
    ];
    for name in ["run", "wait", "limit", "silent", "unknown"] {
        for (move_to, left) in &halves {
            let mut c = card(name);
            c.move_to = move_to.clone();
            c.grace_until = left.clone();
            assert_eq!(seat::band(&c), landed(name), "組 {name} の {move_to:?} {left:?}");
        }
    }
    // 帯の字は起点を書かず、語 move_grace を引かない。
    let Some(b) = seat::band(&grace("run", "acct-5", 120)) else {
        panic!("猶予の内の帯が無い");
    };
    let text = format!("{} {:?}", b.l1.join(" "), b.l2);
    assert!(!text.contains(&label("move_grace")), "{text}");
}

#[test]
fn sgrace_band_refused_pressed() {
    assert_eq!(label("no_target"), "移り先なし");
    // 断り: 語 no_target と ◷ と時刻の行・2 行目は REFUSED_NEXT。
    let mut run = card("run");
    run.refused = Reading::Known(Some(REFUSED_AT));
    assert_eq!(
        seat::band(&run),
        Some(Band {
            l1: vec![refused_line(REFUSED_AT)],
            l2: Some(REFUSED_NEXT.to_string()),
        })
    );
    // 前の日の断りは月日を前に付けた時刻。
    run.refused = Reading::Known(Some(REFUSED_OLD));
    assert_eq!(seat::band(&run).expect("帯").l1, vec![refused_line(REFUSED_OLD)]);
    assert_ne!(hmd(REFUSED_OLD, AT), hmd(REFUSED_AT, AT));
    // 移動待ちの組の断り: 移動待ちの行の後・2 行目は REFUSED_NEXT。
    let mut wait = card("wait");
    wait.refused = Reading::Known(Some(REFUSED_AT));
    assert_eq!(
        seat::band(&wait),
        Some(Band {
            l1: vec![
                format!("{MOVE_WAIT} acct-4 → acct-5"),
                refused_line(REFUSED_AT)
            ],
            l2: Some(REFUSED_NEXT.to_string()),
        })
    );
    // 猶予の内の断り: 2 行目は GRACE_NEXT のまま。
    let mut both = grace("run", "acct-5", 120);
    both.refused = Reading::Known(Some(REFUSED_AT));
    assert_eq!(
        seat::band(&both),
        Some(Band {
            l1: vec![format!("{MOVING} 120 秒"), refused_line(REFUSED_AT)],
            l2: Some(format!("{GRACE_NEXT} acct-5")),
        })
    );
    // 逼迫: 群の今の口座と窓と割合と閾値の行・2 行目は着地の次の移り先のまま。
    let mut pressed = card("run");
    pressed.pressure = Reading::Known(Some(pressure("5h", 92, 85)));
    assert_eq!(
        seat::band(&pressed),
        Some(Band {
            l1: vec![format!("{PRESSED} acct-4 5h 92% ≥ 85")],
            l2: Some(format!("{NEXT_TARGET} acct-5")),
        })
    );
    // 限度の間は逼迫の行を出さない。
    let mut limit = card("limit");
    limit.pressure = Reading::Known(Some(pressure("7d", 81, 80)));
    assert_eq!(seat::band(&limit), seat::band(&card("limit")));
    // 断りと逼迫が Known(None) か Unknown なら行を出さない（平時は帯が開かない）。
    for r in [Reading::Known(None), Reading::Unknown] {
        let mut c = card("run");
        c.refused = r.clone();
        c.pressure = match r {
            Reading::Known(_) => Reading::Known(None),
            Reading::Unknown => Reading::Unknown,
        };
        assert_eq!(seat::band(&c), None, "{r:?}");
    }
}

#[test]
fn sgrace_more_copied() {
    // fixture の 5 組は 4 つとも測れていない。
    for (name, c) in fixture() {
        assert_eq!(
            copied_line(&c),
            "move=? grace_left=? refused=? pressure=?",
            "組 {name}"
        );
        let m = more(&c);
        assert_eq!(m.copied, vec![copied_line(&c)], "組 {name}");
        assert_eq!(seat::seat(&c).more, m, "組 {name}");
    }
    let mut none = card("run");
    none.move_to = Reading::Known(None);
    none.grace_until = Reading::Known(None);
    none.refused = Reading::Known(None);
    none.pressure = Reading::Known(None);
    assert_eq!(copied_line(&none), "move=- grace_left=- refused=- pressure=-");

    let mut full = grace("run", "acct-5", 120);
    full.refused = Reading::Known(Some(REFUSED_AT));
    full.pressure = Reading::Known(Some(pressure("model", 100, 95)));
    assert_eq!(
        copied_line(&full),
        format!(
            "move=acct-5 grace_left=120 refused={} pressure=model:100/95",
            hmd(REFUSED_AT, AT)
        )
    );
    // 前の日の断りは行が長くなり、doctor の行と同じ字数に畳む。
    full.refused = Reading::Known(Some(REFUSED_OLD));
    let line = copied_line(&full);
    assert!(line.chars().count() > EXPERT_CHARS, "{line}");
    let m = more(&full);
    assert_eq!(m.copied, wrap_words(&line, EXPERT_CHARS));
    assert!(m.copied.len() > 1);
    for l in &m.copied {
        assert!(l.chars().count() <= EXPERT_CHARS, "{l}");
    }
    assert_eq!(m.copied.join(" "), line);
    // doctor の行は着地のまま。
    assert_eq!(m.doctor, more(&card("run")).doctor);
}

#[test]
fn sgrace_dom_text() {
    let dom = read("src/project_dom/seat.rs");
    let start = dom.find("fn more_view(").expect("more_view が在る");
    let len = dom[start..].find("\n}").expect("more_view の閉じ");
    let body = &dom[start..start + len];
    assert_eq!(body.matches(".chain(more.copied)").count(), 1);
    let chain = body.find(".chain(more.copied)").expect("chain");
    assert!(body.find(".doctor").expect("doctor") < chain);
    assert!(chain < body.find("</details>").expect("</details>"));
    assert_eq!(dom.matches(".chain(more.copied)").count(), 1);

    let src = read("src/project/seat.rs");
    for word in ["move_grace_s", "grace_secs", "move-signal", "\"move_grace\""] {
        assert!(!src.contains(word), "src/project/seat.rs に {word} が在る");
    }
}

/// main の契約表の verify の filter の語（210 語・歯の名の見張りの WORDS と同じ）。
const WORDS: &[&str] = &[
    "aaround_",
    "abst_",
    "accept_",
    "acchold_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "aface_",
    "afocus_",
    "alean_",
    "aord_",
    "aown_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadl_",
    "cadopt_",
    "cadq_",
    "cdorm_",
    "cfsplit_",
    "cg9_",
    "cgdom_",
    "cmark_",
    "cnote_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "cupd_",
    "cupdlist_",
    "denv_",
    "dnedge_",
    "dngrp_",
    "dnrow_",
    "dnskip_",
    "ecache_",
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "g3g7_",
    "gapspage_",
    "gatt_",
    "gbnote_",
    "gchip_",
    "gcoach_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gins_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "gwv_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hdchip_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hshist_",
    "hspage_",
    "hspk_",
    "hsym_",
    "hthr_",
    "http_",
    "hwstore_",
    "iclose_",
    "ilink_",
    "jcount_",
    "jrun_",
    "kcli_",
    "kg9_",
    "kindlab_",
    "klink_",
    "ksum_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lgrp_",
    "lhome_",
    "lidle_",
    "lkind_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstg_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mqask_",
    "mqface_",
    "mstore_",
    "mtips_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntc_",
    "ntime_",
    "nxact_",
    "nxorg_",
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
    "pgsw_",
    "pgz_",
    "pipe_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "pubscan_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "qsig_",
    "question_",
    "rbusy_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server::events::",
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stbp_",
    "stcli_",
    "steady_",
    "stnfy_",
    "stskill_",
    "sttgt_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "wstrip_",
];

#[test]
fn sgrace_own_names_clean() {
    assert_eq!(WORDS.len(), 210);
    let mut names = Vec::new();
    for rel in ["tests/sgrace.rs", "../tsuzuri-core/tests/sgrace.rs"] {
        let text = read(rel);
        let lines: Vec<&str> = text.lines().collect();
        for w in lines.windows(2).filter(|w| w[0].trim() == "#[test]") {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の次の行は fn");
            names.push(rest[..rest.find('(').expect("fn の名の後に (")].to_string());
        }
    }
    assert_eq!(names.len(), 9, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("sgrace_")
            .unwrap_or_else(|| panic!("{name} は sgrace_ で始まる"));
        for w in WORDS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
