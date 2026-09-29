//! 行 g-tick-adopt の歯: 問いの card の経過の chip・pipeline の札の経過・次の一手の質問の行の経過を
//! net の 1 秒の時計（ticker）で書き直すこと、問いの chip の経験者だけの注釈（投稿の時刻）。
//! 字と経過の値は host で組み、DOM は wasm の target のときだけなので、src の file の字で付け方を見る
//! （DOM が組めることは xtask の surface-build）。

use std::path::PathBuf;

use tsuzuri_contract::board::{PipelineCard, Stage};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::ask::{age as ask_age, posted_tip};
use tsuzuri_surface::project::pipeline::{Kcard, NO_AGE, age, age_at, kcard};
use tsuzuri_surface::widgets::help::{Inline, expert_note};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// file の DOM の部分（字「mod dom」より後）。
fn dom(rel: &str) -> String {
    let text = read(rel);
    let at = text
        .find("mod dom")
        .unwrap_or_else(|| panic!("{rel} に mod dom が在る"));
    text[at..].to_string()
}

/// 字 from から字 to の前まで（to は from より後を探す）。
fn range<'a>(text: &'a str, from: &str, to: &str) -> &'a str {
    let start = text
        .find(from)
        .unwrap_or_else(|| panic!("字 {from} が無い"));
    let rest = &text[start..];
    let end = rest
        .find(to)
        .unwrap_or_else(|| panic!("字 {from} の後に字 {to} が無い"));
    &rest[..end]
}

/// 札を描く今。
const NOW: u64 = 1_790_510_400;

/// 段 Running と段を決めた時刻の札（contract は fx-t.1）。
fn running(since: Option<u64>) -> PipelineCard {
    PipelineCard {
        contract: BeadId::new("fx-t.1").expect("id"),
        runs: 1,
        stage: Stage::Running,
        reason: None,
        account: None,
        since,
        ci: None,
    }
}

/// (1) 問いの chip の注釈の字は posted_at と日本時間の投稿の時刻で、経験者の mode だけで 1 行の注釈になる。
#[test]
fn tkad_posted_tip_rules() {
    let tip = posted_tip(1_790_488_800);
    assert_eq!(tip, "posted_at 2026-09-27 15:00:00 JST");
    assert_eq!(
        expert_note(&tip, Mode::Expert),
        Some(vec![vec![Inline::Text(tip.clone())]])
    );
    assert_eq!(expert_note(&tip, Mode::Beginner), None);
    assert_eq!(ask_age(1_790_488_861, 1_790_488_800), "1m");
}

/// (2) 描く時の経過は今から段を決めた時刻を引いた字（今より後は 0s）で、札は電文の時刻を持つ。
#[test]
fn tkad_age_at_rules() {
    assert_eq!(age_at(Some(910), 1000), "1m");
    assert_eq!(age_at(Some(910), 1030), "2m");
    assert_eq!(age_at(Some(910), 900), "0s");
    assert_eq!(age_at(Some(941), 1000), "59s");
    assert_eq!(age_at(Some(10), 3610), "1h");
    assert_eq!(age_at(None, 50), NO_AGE);
    for elapsed in [None, Some(0), Some(3700)] {
        let card = running(elapsed.map(|e: u64| NOW - e));
        let k: Kcard = kcard(&card, &[], NOW);
        assert_eq!(k.since, card.since, "{elapsed:?}");
        assert_eq!(k.age, age_at(card.since, NOW), "{elapsed:?}");
        assert_eq!(
            k.age,
            elapsed.map_or_else(|| NO_AGE.to_string(), age),
            "{elapsed:?}"
        );
    }
}

/// (3) 3 つの file の DOM は 1 秒の時計を読み、問いの chip に投稿の時刻の注釈を付ける。
#[test]
fn tkad_dom_wiring() {
    let ask = dom("src/project/ask.rs");
    assert!(ask.contains("let clock = crate::net::ticker();"));
    assert!(!ask.contains("crate::net::now()"));
    let head = range(&ask, "Part::Head", "Part::Summary");
    let tagged = head.split("<span").skip(1).any(|t| {
        let tag = &t[..t.find('>').expect("span の開きの tag の >")];
        tag.contains(r#"class="chip num""#) && tag.contains("use:expert_tip=posted_tip(posted)")
    });
    assert!(tagged, "頭の部分の chip の span に注釈が無い");
    assert!(head.contains("{move || age(tick(), posted)}"));

    let pipe = dom("src/project/pipeline.rs");
    assert!(pipe.contains("let tick = crate::net::ticker();"));
    assert!(range(&pipe, "let tick = ", "let body").contains("let clock = move || tick.get();"));
    assert!(!pipe.contains("Memo::new("));
    let card = range(&pipe, "fn kcard_view(", "pub fn stage_sym(");
    assert!(card.contains("age_at(since, clock())"));
    assert!(!card.contains("card.age"));

    let next = dom("src/project/next.rs");
    assert!(next.contains("waited(&for_wait, q, clock.get())"));
    assert!(next.contains("let clock = crate::net::ticker();"));
    assert!(!next.contains("crate::net::now()"));
}

/// verify の filter の語（main fb2cc24 の 177 語と同じノートのほかの 2 行の接頭辞）。
const FILTERS: [&str; 179] = [
    "aaround_",
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
    "cadl_",
    "cadopt_",
    "cadq_",
    "cdorm_",
    "cfsplit_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "denv_",
    "dnedge_",
    "dngrp_",
    "dnrow_",
    "dnskip_",
    "ecache_",
    "epolq_",
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
    "kindlab_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lresume_",
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
    "pgz_",
    "pipe_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
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
    "steady_",
    "stbp_",
    "stcli_",
    "sttgt_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "gwv_",
    "ksum_",
];

/// (4) この file の歯は 4 つで、名は tkad_ で始まり、残りの字は filter の語を含まない。
#[test]
fn tkad_names_clean() {
    let text = read("tests/tkad.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<String> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .map(|w| {
            let name = w[1].strip_prefix("fn ").expect("#[test] の次の行は fn");
            name[..name.find('(').expect("fn の名の後に ( が在る")].to_string()
        })
        .collect();
    assert_eq!(names.len(), 4);
    for name in &names {
        let tail = name
            .strip_prefix("tkad_")
            .unwrap_or_else(|| panic!("{name} は tkad_ で始まる"));
        for word in FILTERS {
            assert!(!tail.contains(word), "{name} は {word} を含まない");
        }
    }
}
