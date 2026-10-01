//! 行 g-jst の歯: 面が人に見せる時刻は日本時間（UTC に 9 時間を足す固定）で、字の印は JST、
//! 同じ日と今日は日本の日、稼働の記録の目盛は日本時間の tick の倍数、決定の link は id の UTC の時分を日本時間にする。
//! 時差の定数は view の 1 つだけで、src に UTC の印の字と時差の数を直に書かない。この file の歯の名。
#![cfg(test)]

use std::path::{Path, PathBuf};

use tsuzuri_contract::EpochSecs;
use tsuzuri_surface::project::askpage::ruling_text;
use tsuzuri_surface::project::seat::{Span, hm, hmd, span_ticks};
use tsuzuri_surface::view::{JST, JST_OFFSET, clock, clock_short, hhmm, jst};
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// (1) 時差の定数と印、日本時間の日の番号とその日の 0 時からの秒。
#[test]
fn gjst_offset_and_day_number() {
    assert_eq!(JST_OFFSET, 32_400);
    assert_eq!(JST, "JST");
    let f: fn(EpochSecs) -> (EpochSecs, EpochSecs) = jst;
    assert_eq!(f(0), (0, 32_400));
    assert_eq!(jst(1_790_467_200), (20_723, 32_400));
    assert_eq!(jst(1_790_521_199), (20_723, 86_399));
    assert_eq!(jst(1_790_521_200), (20_724, 0));
}

/// (2) 長い字は日本時間の年月日と時分秒と JST（日と年の変わり目・閏日）、hhmm は印なしの時分。
#[test]
fn gjst_clock_crosses_the_date() {
    for (at, want) in [
        (1_790_521_199, "2026-09-27 23:59:59 JST"),
        (1_790_521_200, "2026-09-28 00:00:00 JST"),
        (1_767_193_199, "2025-12-31 23:59:59 JST"),
        (1_767_193_200, "2026-01-01 00:00:00 JST"),
        (1_709_132_400, "2024-02-29 00:00:00 JST"),
    ] {
        assert_eq!(clock(at), want, "{at}");
    }
    let f: fn(EpochSecs) -> String = hhmm;
    assert_eq!(f(1_790_521_199), "23:59");
    assert_eq!(hhmm(1_790_521_200), "00:00");
}

/// (3) 短い字は時分と空白と JST。clock_short は 20 時間で、hmd は日本の日で月日を付けるか決める。
#[test]
fn gjst_short_forms_mark() {
    assert_eq!(clock_short(1_790_521_199, 1_790_521_200), "23:59 JST");
    assert_eq!(clock_short(1_790_521_200, 1_790_593_201), "09-28 00:00 JST");
    assert_eq!(hm(1_790_521_200), "00:00 JST");
    // UTC では同じ日・日本の日が違う。
    assert_eq!(hmd(1_790_521_199, 1_790_521_200), "09-27 23:59 JST");
    // UTC では違う日・日本の日が同じ。
    assert_eq!(hmd(1_790_460_300, 1_790_510_400), "07:05 JST");
    assert_eq!(hmd(1_790_521_200, 1_790_607_599), "00:00 JST");
}

/// (5) 決定の link は id の UTC の時分を日本時間の時分と JST にする（日を越えても時分だけ）。
#[test]
fn gjst_ruling_link_wraps() {
    for (id, want) in [
        ("fx.1:20260927T1500Z-1", "00:00 JST"),
        ("fx.1:20260927T1459Z-2", "23:59 JST"),
        ("fx.1:20260927T2359Z-3", "08:59 JST"),
        ("fx.1:20260927T0000Z-1", "09:00 JST"),
    ] {
        assert_eq!(ruling_text(id), want, "{id}");
    }
}

/// (6) 目盛は日本時間で tick の倍数の時刻に置く。
#[test]
fn gjst_axis_on_japan_hours() {
    let pairs: Vec<(String, String)> = span_ticks(1_790_521_200, Span::H24)
        .into_iter()
        .map(|t| (t.left, t.label))
        .collect();
    let want: Vec<(String, String)> = [
        ("0.00", "00:00"),
        ("25.00", "06:00"),
        ("50.00", "12:00"),
        ("75.00", "18:00"),
    ]
    .iter()
    .map(|(l, t)| (l.to_string(), t.to_string()))
    .collect();
    assert_eq!(pairs, want);
    for i in 0..48 {
        let at = 1_790_510_400 + i * 1_237;
        for t in span_ticks(at, Span::H24) {
            assert!(
                ["00:00", "06:00", "12:00", "18:00"].contains(&t.label.as_str()),
                "{at} H24 {}",
                t.label
            );
        }
        for t in span_ticks(at, Span::H6) {
            assert!(t.label.ends_with(":00"), "{at} H6 {}", t.label);
        }
        for t in span_ticks(at, Span::H3) {
            assert!(
                t.label.ends_with(":00") || t.label.ends_with(":30"),
                "{at} H3 {}",
                t.label
            );
        }
    }
}

/// dir の下の .rs の file を再帰に集める（crate の dir からの相対の path と中身）。
fn rs_files(dir: &Path, out: &mut Vec<(String, String)>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display()))
        .map(|e| e.expect("dir の項").path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let rel = p
                .strip_prefix(crate_dir())
                .expect("crate の下")
                .to_string_lossy()
                .replace('\\', "/");
            let text = std::fs::read_to_string(&p).expect("src の file");
            out.push((rel, text));
        }
    }
}

/// (7) src に UTC の印の字と時差の数を直に書かず、時差の定数は view の 1 つだけ。
#[test]
fn gjst_src_one_offset_no_utc_marks() {
    let mut files = Vec::new();
    rs_files(&crate_dir().join("src"), &mut files);
    assert!(files.len() >= 40, "src の .rs の file が {} しか無い", files.len());
    let banned = [
        "}Z\"",
        "UTC\"",
        "trim_end_matches('Z')",
        "32400",
        "9 * 3600",
        "9 * 3_600",
    ];
    let mut offsets = 0;
    for (rel, text) in &files {
        for b in banned {
            assert!(!text.contains(b), "{rel} に {b} が在る");
        }
        let n = text.matches("32_400").count();
        if rel != "src/view.rs" {
            assert_eq!(n, 0, "{rel} に 32_400 が在る");
        }
        offsets += n;
    }
    assert_eq!(offsets, 1, "32_400 は src/view.rs にちょうど 1 度");
    let view = read("src/view.rs");
    assert!(view.contains("pub const JST_OFFSET"));
    assert!(view.contains(": EpochSecs = 32_400;"));
}

/// (8) 語の辞書の Landed の列の注は日本時間の今日。
#[test]
fn gjst_vocab_land_note() {
    let term = vocab().term("col_land").expect("鍵 col_land が vocab に無い");
    assert_eq!(term.label, "Landed（今日）");
    assert_eq!(
        term.note,
        "今日（日本時間）、本線に取り込まれたか、着地せずに閉じた run と、日を問わず CI を待つ run"
    );
}

/// 着地済みの行と並行の起草の行と計画の後の行の verify の filter の語。
const FILTERS: [&str; 143] = [
    "aaround_",
    "accept_",
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
    "afocus_",
    "aord_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadopt_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "dnrow_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gbnote_",
    "gfresh_",
    "ghb_",
    "glabel_",
    "gnav_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
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
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "hsym_",
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lsnap_",
    "lspark_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mstore_",
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
    "parts_",
    "pclosed_",
    "pfold_",
    "pipe_",
    "plimit_",
    "pmore_",
    "pqueue_",
    "project_",
    "pquest_",
    "ptitle_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
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
    "stcli_",
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "pgz_",
    "pmisfit_",
    "gfix_",
    "hasplit_",
];

/// (18) この file の歯の名はちょうど 8 で gjst_ で始まり、残りの字は filter の語を含まない。
#[test]
fn gjst_own_names_clean() {
    let src = read("tests/gjst.rs");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1]
                .trim()
                .strip_prefix("fn ")
                .unwrap_or_else(|| panic!("test の属性の次が fn でない: {}", w[1]));
            &rest[..rest.find('(').expect("fn の名の終わり")]
        })
        .collect();
    assert_eq!(names.len(), 8, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("gjst_")
            .unwrap_or_else(|| panic!("歯 {name} が gjst_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "歯 {name} が filter の語 {word} を含む");
        }
    }
}
