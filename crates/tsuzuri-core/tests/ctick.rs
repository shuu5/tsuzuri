//! 行 c-seat-tick の歯（中核）: 席の card の欄 tick（doctor の席の行の tick の語）と tick_at（合図の最後の判定の ts）。
//! fixture: tests/fixtures/seat/seat-inputs.json の組 run（組の席の名と今の時刻は期待の card の target と at）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{SeatCard, TickHealth};
use tsuzuri_core::seat::{SeatTexts, anchor, card};

const FIXTURE: &str = "tests/fixtures/seat/seat-inputs.json";

/// 組 run の doctor の席 proj-1:0.1 の行の末の字。
const OWN: &str = " heartbeat=on tick=healthy";

#[derive(Deserialize)]
struct Case {
    #[serde(flatten)]
    texts: SeatTexts,
    card: SeatCard,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn run() -> Case {
    let mut all: BTreeMap<String, Case> =
        serde_json::from_str(&read(FIXTURE)).expect("fixture の形");
    all.remove("run").expect("組 run")
}

/// server と同じ組み方（anchor は doctor の席の行から）。
fn build(c: &Case, texts: &SeatTexts) -> SeatCard {
    let target = &c.card.target;
    let anchor = texts.doctor.as_deref().and_then(|d| anchor(d, target));
    card(target, anchor.as_deref(), texts, c.card.at)
}

/// 組 run の doctor の字を替えて組む。
fn with_doctor(c: &Case, edit: impl Fn(&str) -> String) -> SeatCard {
    let mut texts = c.texts.clone();
    texts.doctor = texts.doctor.as_deref().map(edit);
    build(c, &texts)
}

/// 組 run の合図の最後の判定の字を替えて組んだ tick_at。
fn at_of(c: &Case, tick_last: Option<&str>) -> Option<u64> {
    let mut texts = c.texts.clone();
    texts.tick_last = tick_last.map(str::to_string);
    build(c, &texts).tick_at
}

/// (3) 組 run の card は fixture の期待と同じで、tick の語を替えると tick だけがその語になる。
#[test]
fn ctick_card_copies_word_and_ts() {
    let c = run();
    let doctor = c.texts.doctor.clone().expect("doctor");
    assert_eq!(doctor.matches(OWN).count(), 1, "席の行の末の字");
    let full = build(&c, &c.texts);
    assert_eq!(full, c.card);
    assert_eq!(full.tick, Reading::Known(TickHealth::Healthy));
    assert_eq!(full.tick_at, Some(1_790_510_390));
    for t in TickHealth::ALL {
        let got = with_doctor(&c, |d| {
            d.replace("tick=healthy", &format!("tick={}", t.as_str()))
        });
        let mut want = full.clone();
        want.tick = Reading::Known(t);
        assert_eq!(got, want, "{}", t.as_str());
    }
}

/// (4) 4 つの語のほかの語・空の語・語の無い行は tick だけが Unknown。
#[test]
fn ctick_other_words_unknown() {
    let c = run();
    let full = build(&c, &c.texts);
    let mut want = full.clone();
    want.tick = Reading::Unknown;
    for to in [
        " heartbeat=on tick=no-rule:missing",
        " heartbeat=on tick=",
        " heartbeat=on tick=Healthy",
        " heartbeat=on",
        "",
    ] {
        let got = with_doctor(&c, |d| d.replace(OWN, to));
        assert_eq!(got, want, "{to:?}");
    }
}

/// (5) tick の語は席の名の行だけから読む（host の行とほかの席の行の tick は読まない）。
#[test]
fn ctick_word_from_own_seat_line() {
    let c = run();
    let full = build(&c, &c.texts);
    let got = with_doctor(&c, |d| {
        let d = d
            .replace(OWN, "")
            .replace("pane=%2", "pane=%2 heartbeat=on tick=stale");
        format!("host-manifest=present tick=declared devices=d-1\n{d}")
    });
    let mut want = full.clone();
    want.tick = Reading::Unknown;
    assert_eq!(got, want);
}

/// (6) tick_at は合図の最後の判定の空でない最後の行の ts（数でなければ・無ければ None）。
#[test]
fn ctick_at_from_last_line() {
    let c = run();
    assert_eq!(
        at_of(
            &c,
            Some("ts=10 decision=noop reason=seat-busy\nts=20 decision=noop reason=seat-busy\n\n")
        ),
        Some(20)
    );
    for text in [
        Some("ts=abc decision=noop reason=seat-busy\n"),
        Some("decision=noop reason=seat-busy\n"),
        Some(""),
        None,
    ] {
        assert_eq!(at_of(&c, text), None, "{text:?}");
    }
}

/// filter の語（main ca1ca76 の contracts と計画の verify の filter の語を、ほかの語を部分の字として含まない語に畳んだ語）。
const FILTER: [&str; 153] = [
    "aaround_", "accept_", "acchold_", "account_", "acctcore_", "acctdoc_", "accthb_",
    "accthome_", "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_",
    "acctwire_", "afocus_", "aord_", "apop_", "askcard_", "athr_", "batchpanel_", "bhalf_",
    "board_min_", "bport_", "brand_", "btuck_", "cadopt_", "cdorm_", "cgdom_", "cmark_",
    "contract_form_", "cround_", "csled_", "denv_", "dnedge_", "dnrow_", "dnskip_", "ecache_",
    "epolq_", "flight_", "fmark_", "fprem_", "frame_", "fserve_", "fstop_", "gapspage_",
    "gbnote_", "gfix_", "gfresh_", "ghb_", "gjst_", "glabel_", "gnav_", "gpface_", "gpill_",
    "gpulse_", "graph_", "gsum_", "gtuck_", "gview_", "hacols_", "harest_", "hasplit_",
    "hbconf_", "hbmark_", "hbpost_", "hbproc_", "hbroute_", "hcard_", "hcled_", "hcnx_",
    "hcproj_", "hcsess_", "hdchip_", "hfig_", "hnunk_", "hook_", "hruling_", "hsblock_",
    "hsderive_", "hshist_", "hspage_", "hsym_", "iclose_", "ilink_", "kcli_", "klink_",
    "launch_", "lcard_", "ledgerblock_", "lhome_", "lidle_", "lresume_", "lsnap_", "lspark_",
    "lstore_", "mapview_", "mkeys_", "mlink_", "mstore_", "mtree_", "nact_", "nbatch_", "ncard_",
    "nextstep_", "nodepage_", "nstall_", "nsum_", "nsumw_", "ntime_", "nxact_", "parts_",
    "pclosed_", "pfold_", "pipe_", "plimit_", "pmisfit_", "pmore_", "pquest_", "pqueue_",
    "project_", "ptitle_", "punmap_", "pwhole_", "qblock_", "qgate_", "qkey_", "question_",
    "relay_", "rhold_", "runsdoc_", "rvk_", "saxis_", "sclosed_", "seatblock_", "seatcard_",
    "server_", "sesplit_", "shb_", "skeleton_", "smore_", "stage_", "stats_", "stcli_",
    "steady_", "sxaxis_", "ticker_", "tipx_", "topbar_", "tz_", "urpanel_", "uword_", "wstrip_",
];

/// 新しい歯の 3 つの file。
const FILES: [&str; 3] = [
    "crates/tsuzuri-contract/tests/ctick.rs",
    "crates/tsuzuri-core/tests/ctick.rs",
    "crates/tsuzuri-surface/tests/ctickface.rs",
];

/// (15) 3 つの file の歯の名は 12 本とも ctick_ で始まり、残りの字は filter の語を含まない。
#[test]
fn ctick_own_names_clean() {
    for w in FILTER {
        assert!(!"ctick_".contains(w), "接頭辞が {w} を含む");
    }
    let mut n = 0;
    for file in FILES {
        let text = read(file);
        let lines: Vec<&str> = text.lines().collect();
        for (i, l) in lines.iter().enumerate() {
            if l.trim() != "#[test]" {
                continue;
            }
            let f = lines[i + 1].trim();
            let name = f
                .strip_prefix("fn ")
                .and_then(|r| r.split('(').next())
                .unwrap_or_else(|| panic!("{file}: test の属性の次が fn でない: {f}"));
            let rest = name
                .strip_prefix("ctick_")
                .unwrap_or_else(|| panic!("{file}: {name} が ctick_ で始まらない"));
            for w in FILTER {
                assert!(!rest.contains(w), "{file}: {name} が filter の語 {w} を含む");
            }
            n += 1;
        }
    }
    assert_eq!(n, 12);
}
