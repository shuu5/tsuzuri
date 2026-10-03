//! 行 c-limit-resume の歯（面・接頭辞 lresume_）: header の席の pill の限度の再開の時刻と、席の card の ↻ の行・
//! seatpill の字・3 つの file の歯の名。fixture は tests/fixtures/surface/seat-card.json（読むだけ）で、
//! 欄 reopens は歯の中で置いて電文にする。時刻の字は hmd を撃った値と比べる。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::seat::{Reopens, SeatCard};
use tsuzuri_contract::wire;
use tsuzuri_surface::project::seat::{LIMIT_LINE, NEXT_TARGET, hmd};
use tsuzuri_surface::seatpill::{self, RESUME, RESUME_UNKNOWN};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::label;
use tsuzuri_surface::widgets::hover::ROW_CHARS;

/// 2026-09-27T13:00:00Z と 2026-09-28T00:00:00Z。
const T13: EpochSecs = 1_790_514_000;
const T24: EpochSecs = 1_790_553_600;

/// fixture の組の at。
const AT: EpochSecs = 1_790_510_400;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> BTreeMap<String, SeatCard> {
    wire::decode(&read("../../tests/fixtures/surface/seat-card.json"))
        .expect("fixture の組が電文として読める")
}

/// 組 `name` の reopens を置いた電文の本文。
fn body(name: &str, reopens: Reopens) -> Fetched {
    let mut card = fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture に組 {name} が在る"));
    card.reopens = reopens;
    Fetched::Body(wire::encode(&card).expect("電文"))
}

/// (7) pill の eta は、限度で時刻が在るときだけ時刻（今と at の大きい方で日を見る）。
#[test]
fn lresume_pill_eta() {
    assert_eq!(fixture()["limit"].reopens, Reopens::Unmeasured);
    for (t, now) in [
        (T13, 1_790_511_000),
        (T13, 1_790_500_000),
        (T24, 1_790_511_000),
        (T24, 1_790_560_000),
    ] {
        let p = seatpill::pill(&body("limit", Reopens::At(t)), now);
        assert_eq!(p.state, "limit");
        assert_eq!(p.eta, Some(hmd(t, now.max(AT))), "時刻 {t} 今 {now}");
    }
    for r in [Reopens::Clear, Reopens::Unmeasured, Reopens::Unknown] {
        let p = seatpill::pill(&body("limit", r), 1_790_511_000);
        assert_eq!(p.eta, None, "{r:?}");
    }
    for name in ["run", "wait", "unknown"] {
        let p = seatpill::pill(&body(name, Reopens::At(T13)), 1_790_511_000);
        assert_eq!(p.eta, None, "組 {name}");
    }
    let mut silent = fixture().remove("silent").expect("組 silent");
    silent.since = Some(1_790_509_800);
    silent.reopens = Reopens::At(T13);
    let f = Fetched::Body(wire::encode(&silent).expect("電文"));
    assert_eq!(
        seatpill::pill(&f, 1_790_510_400).eta.as_deref(),
        Some("10m")
    );
}

/// (8) card の ↻ の行は限度の行の直後にだけ在る。
#[test]
fn lresume_card_line() {
    assert_eq!(RESUME, "↻");
    assert_eq!(RESUME_UNKNOWN, "時刻が分からない");
    assert_eq!(label("st_unknown"), "測れていない");
    let head = vec!["model opus".to_string(), LIMIT_LINE.to_string()];
    let next = format!("{NEXT_TARGET} acct-6");
    let with = |third: Option<String>| {
        let mut v = head.clone();
        v.extend(third);
        v.push(next.clone());
        v
    };
    let clear = seatpill::card(&body("limit", Reopens::Clear), AT);
    for (r, third) in [
        (Reopens::At(T13), Some(format!("{RESUME} {}", hmd(T13, AT)))),
        (Reopens::At(T24), Some(format!("{RESUME} {}", hmd(T24, AT)))),
        (Reopens::Clear, None),
        (
            Reopens::Unmeasured,
            Some(format!("{RESUME} {}", label("st_unknown"))),
        ),
        (Reopens::Unknown, Some(format!("{RESUME} {RESUME_UNKNOWN}"))),
    ] {
        let c = seatpill::card(&body("limit", r), AT);
        assert_eq!(c.more, with(third), "{r:?}");
        for row in &c.more {
            assert!(row.chars().count() <= ROW_CHARS, "{r:?}: {row}");
        }
        assert_eq!(
            (&c.title, &c.kind, &c.value, &c.src),
            (&clear.title, &clear.kind, &clear.value, &clear.src),
            "{r:?}"
        );
    }
    let far = seatpill::card(&body("limit", Reopens::At(T24)), AT);
    assert_ne!(far.more[2], format!("{RESUME} {}", hmd(T24, T24)));
    for name in ["run", "wait", "silent", "unknown"] {
        let clear = seatpill::card(&body(name, Reopens::Clear), AT);
        for r in [Reopens::At(T13), Reopens::Unmeasured, Reopens::Unknown] {
            assert_eq!(
                seatpill::card(&body(name, r), AT),
                clear,
                "組 {name} の {r:?}"
            );
        }
    }
}

/// (9) seatpill の字: dom の eta の 2 つの span と、限度の時刻と ↻ の行の字。
#[test]
fn lresume_pill_text() {
    let text = read("src/seatpill.rs");
    let at = text.find("mod dom {").expect("seatpill に mod dom が在る");
    let (head, dom) = text.split_at(at);
    for (w, n) in [
        ("style=ETA_STYLE", 1),
        ("<span class=\"eta num\" style=ETA_STYLE>{e}</span>", 1),
        ("<span class=\"eta num\">{e}</span>", 1),
        ("state_value(SeatState::Silent)", 1),
    ] {
        assert_eq!(dom.matches(w).count(), n, "dom の {w}");
    }
    for w in [
        "pub const RESUME: &str = ",
        "pub const RESUME_UNKNOWN: &str = ",
        "fn resume_line(",
        "more.extend(resume_line(c.reopens, c.at));",
        "Reopens::Unmeasured => label(state_key(UNKNOWN)),",
        "Reopens::At(t)) => Some(hmd(t, now.max(c.at)))",
    ] {
        assert_eq!(head.matches(w).count(), 1, "mod dom の前の {w}");
    }
    assert!(!text.contains("\"/api/"), "seatpill に口の字が在る");
    assert!(!text.contains("decode::<"), "seatpill が電文を decode する");
}

/// filter の語（この行の接頭辞 lresume_ は並べない）。
const WORDS: [&str; 202] = [
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
    "lsnap_",
    "lspark_",
    "lstg_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
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
    "unow_",
    "urpanel_",
    "uword_",
    "wstrip_",
];

/// test の属性の行の次の fn の名。
fn test_names(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
                .to_string()
        })
        .collect()
}

/// (13) 3 つの file の歯の名の数と接頭辞と、filter の語を含まないこと。
#[test]
fn lresume_own_names_clean() {
    for (rel, n) in [
        ("../tsuzuri-contract/tests/teeth1/lresume.rs", 2),
        ("../tsuzuri-core/tests/teeth2/lresume.rs", 4),
        ("tests/teeth4/lresumepill.rs", 4),
    ] {
        let names = test_names(&read(rel));
        assert_eq!(names.len(), n, "{rel}: {names:?}");
        for name in &names {
            let rest = name
                .strip_prefix("lresume_")
                .unwrap_or_else(|| panic!("{name} が lresume_ で始まらない"));
            for w in WORDS {
                assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
            }
        }
    }
}
