//! 行 m-more-drop の歯（接頭辞 bvmore_・判断の記録 ADR-27 決定 (2)）: project board の block「orchestrator と口座」から
//! 口座の履歴と「詳しく」とその畳みの鍵 seat:hist・seat:more を消し、台帳の block の指標の段から「詳しく」の段と
//! 畳みの鍵 ledger:more を消した形。電文（SeatCard）と状態の帯と 3 つの幅の稼働の記録は残る。
//! DOM は host で撃てないので、src の字と xtask の surface-build で組めることで見る。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::home::EXPERT_CHARS;
use tsuzuri_surface::project::ledger::{self, Part, layout};
use tsuzuri_surface::project::seat::{self, MOVING, Seat, Span};
use tsuzuri_surface::project::{fold_key_ok, fold_keys};

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

/// 席の block の host の file と DOM の file の字。
const SEAT_FILES: [&str; 2] = ["src/project/seat.rs", "src/project_dom/seat.rs"];

/// 席の block の file に残ってはならない字（口座の履歴と詳しくの型・関数・定数・畳みの鍵・DOM の class と語の鍵）。
const SEAT_GONE: [&str; 19] = [
    "HistRow",
    "pub struct More",
    "pub hist:",
    "pub more:",
    "pub fn hist(",
    "pub fn more(",
    "pub const MORE:",
    "MORE_SRC",
    "seat_line",
    "copied_line",
    "NO_CURRENT",
    "seat:hist",
    "seat:more",
    "fn hist_view(",
    "fn more_view(",
    "acct_hist",
    "gmore",
    "ahist",
    "<details",
];

/// (1) 席の block は畳める段を持たず、口座の履歴と詳しくの字が host の file にも DOM の file にも無い。
#[test]
fn bvmore_seat_no_hist_no_more() {
    assert!(seat::FOLDS.is_empty(), "{:?}", seat::FOLDS);
    for rel in SEAT_FILES {
        let text = read(rel);
        for w in SEAT_GONE {
            assert!(!text.contains(w), "{rel} に {w} が在る");
        }
    }
    let forms = fold_keys();
    for key in ["seat:hist", "seat:more"] {
        assert!(!forms.contains(&key), "fold_keys に {key} が在る");
        assert!(!fold_key_ok(key), "{key} が鍵の形に合う");
    }
}

/// (2) 中身は上段・3 つの幅の稼働の記録・下段・状態の帯の 4 つで、DOM はその 4 つを描く（電文は替えない）。
#[test]
fn bvmore_seat_strip_band_kept() {
    for (name, card) in fixture() {
        let Seat {
            top,
            strips,
            low,
            band,
        } = seat::seat(&card);
        assert_eq!(top.state, seat::state_value(card.state), "組 {name}");
        let spans: Vec<Span> = strips.iter().map(|s| s.span).collect();
        assert_eq!(spans, Span::ALL.to_vec(), "組 {name}");
        assert_eq!(low.account, card.account, "組 {name}");
        assert_eq!(band, seat::band(&card), "組 {name}");
    }
    let mut grace = fixture().remove("run").expect("組 run");
    grace.move_to = Reading::Known(Some("acct-5".to_string()));
    grace.grace_until = Reading::Known(Some(grace.at + 30));
    let l1 = seat::seat(&grace).band.expect("猶予の帯").l1;
    assert!(l1.contains(&format!("{MOVING} 30 秒")), "{l1:?}");

    let dom = read("src/project_dom/seat.rs");
    let start = dom.find("fn seat_view(").expect("seat_view が在る");
    let len = dom[start..].find("\n}\n").expect("seat_view の閉じ");
    let body = &dom[start..start + len];
    for w in [
        "{band.map(band_view)}",
        "{top_view(top, states)}",
        "{strip_view(strips, span)}",
        "{low_view(low, group_doc)}",
    ] {
        assert_eq!(body.matches(w).count(), 1, "seat_view の {w}");
    }
}

/// (3) 台帳の指標の段は上段・主な指標・burndown・memo・未反映の数・一覧の 6 段で、詳しくの段と鍵 ledger:more が無い。
/// 詳しくの段の項（ready から epic の数までの 8 項）はどの段にも置かない。
#[test]
fn bvmore_ledger_no_more_tier() {
    let tiers: Vec<&str> = layout().iter().map(|(t, _)| t.name()).collect();
    assert_eq!(tiers, ["top", "main", "burn", "memo", "unref", "list"]);
    for p in [
        Part::Ready,
        Part::Blocked,
        Part::Lead,
        Part::Stale,
        Part::Spark,
        Part::Epics,
        Part::Question,
        Part::Epic,
    ] {
        assert!(
            layout().iter().all(|(_, ps)| !ps.contains(&p)),
            "{p:?} が配置の表に在る"
        );
    }
    assert_eq!(ledger::FOLDS, ["ledger:unref"]);
    assert!(!fold_keys().contains(&"ledger:more"));
    assert!(!fold_key_ok("ledger:more"));
    let src = read("src/project/ledger.rs");
    for w in [
        "Tier::More",
        "ledger:more",
        "p_more",
        "gmore",
        "\"gm1 num\"",
    ] {
        assert!(!src.contains(w), "src/project/ledger.rs に {w} が在る");
    }
}

/// (4) 経験者向けの 1 行の字数は規則の行 R-19 の数と同じ（消した詳しくの歯の file から移した見張り）。
#[test]
fn bvmore_expert_chars_rule() {
    let rules = read("../../design-intent/rules.yaml");
    let line = rules
        .lines()
        .find(|l| l.contains("id: R-19,"))
        .expect("規則の行 R-19 が在る");
    let key = "経験者向けの 1 行は";
    let rest = &line[line.find(key).expect("経験者向けの 1 行の字") + key.len()..];
    let num: String = rest[..rest.find('字').expect("字数の単位")]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert_eq!(num.parse::<usize>().expect("数"), EXPERT_CHARS);
    assert_eq!(EXPERT_CHARS, 60);
}

/// main の契約表の verify の filter の語（358 語）。
const FILTERS: [&str; 358] = [
    "aaround_",
    "aaround_src_ask_fold_embeds",
    "abss_",
    "abst_",
    "accept_",
    "acchold_",
    "account_",
    "acctcore_",
    "acctcore_pure_and_no_new_dependencies",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "acctwire_back_part_before_header",
    "acctwire_back_steps",
    "aface_",
    "afocus_",
    "afocus_src_links_and_scroll",
    "alean_",
    "anchor_",
    "aord_",
    "aown_",
    "apark_",
    "apop_",
    "areread_",
    "askcard_",
    "askcard_page_frame_and_nav",
    "athr_",
    "batchpanel_",
    "batchpanel_page_frame_two_columns",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "brand_path_written_once_in_contract",
    "btuck_",
    "cadl_",
    "cadopt_",
    "cadq_",
    "cdorm_",
    "cexcl_",
    "cfsplit_",
    "cg9_",
    "cgdom_",
    "check_canonical_design_intent_passes",
    "cishard_",
    "cmark_",
    "cnote_",
    "cnret_",
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
    "dretry_",
    "dstg_",
    "ecache_",
    "ecache_handle_wiring_text",
    "eheld_acct_",
    "eheld_design_",
    "eheld_held_",
    "eheld_marks_",
    "eheld_vessel_",
    "elazy_",
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
    "f123_",
    "f125_",
    "f142_",
    "f152_",
    "f159_",
    "f174_",
    "f181_",
    "f185_",
    "f190_",
    "f192_",
    "f212_",
    "f2ret_",
    "f76_",
    "f89_",
    "f98_",
    "f99_",
    "face_",
    "fdlt_",
    "fdlv_",
    "fdrop_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "frame_home_blocks_in_order_and_map_page",
    "frame_one_module_per_block",
    "fserve_",
    "fstop_",
    "fstop_usage_errors_are_one",
    "fundl_",
    "fxpre_",
    "g3g7_",
    "gacct_",
    "gapspage_",
    "gapspage_page_frame_and_nav",
    "gate_",
    "gatt_",
    "gbnote_",
    "gchip_",
    "gcoach_",
    "gext_",
    "gfix_",
    "gfresh_",
    "gfresh_boards_text",
    "ghb_",
    "gins_",
    "gjst_",
    "glabel_",
    "gmret_",
    "gmretw_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "gpulse_boards_text",
    "gquestion_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "gwv_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbconf_config_moves_to_config_rs",
    "hbconf_main_parse_fills_the_rest_with_new",
    "hbmark_",
    "hbon_",
    "hbpost_",
    "hbpost_mod_rs_drops_post_fns",
    "hbproc_",
    "hbroute_",
    "hbroute_mod_rs_keeps_only_events",
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
    "hsblock_dispatch_by_list",
    "hsblock_ids_match_consts",
    "hsblock_kit_holds_shared",
    "hsderive_",
    "hsderive_consts_match_note",
    "hshist_",
    "hspage_",
    "hspage_snaps_match_files",
    "hspk_",
    "hsym_",
    "hthr_",
    "http_",
    "hwstore_",
    "iclose_",
    "iclose_handle_wiring_text",
    "ilink_",
    "inject_",
    "jcount_",
    "jrun_",
    "kcli_",
    "kdeny_",
    "kg9_",
    "kindlab_",
    "klink_",
    "klint_",
    "klintf_",
    "ksize_",
    "ksum_",
    "lateface_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lgrp_",
    "lhome_",
    "lidle_",
    "lineage_",
    "lkind_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstg_",
    "lstore_",
    "mapgraph_",
    "mapview_",
    "mapview_classes_and_keys_exist",
    "mbig_",
    "mdoch_",
    "mkeys_",
    "mlink_",
    "mqask_",
    "mqface_",
    "mstore_",
    "mstore_module_text",
    "mtips_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nodepage_frame_and_nav",
    "nstall_",
    "nsum_",
    "nsum_core_reads_no_files",
    "nsumw_",
    "ntc_",
    "ntime_",
    "nxact_",
    "nxorg_",
    "p106_",
    "p1_commands_closed_list",
    "parts_",
    "parts_paths_distinct_in_block_modules",
    "pci_",
    "pclosed_",
    "pfold_",
    "pgsw_",
    "pgz_",
    "pipe_",
    "pkac_",
    "pkview_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "popapp_",
    "pquest_",
    "pqueue_",
    "project_",
    "prose_",
    "ptitle_",
    "pubfp_",
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
    "retired_render_",
    "rhold_",
    "rhold_wiring_text",
    "rsid_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "saxis_src_draws_legend_axis",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server::design::tests::",
    "server::events::",
    "server::events::tests::",
    "server_",
    "server_acct_",
    "server_ask_",
    "server_ask_http_",
    "server_batch_",
    "server_batch_policy_scope_and_refusals",
    "server_coalesce_",
    "server_hb_",
    "server_min_",
    "server_min_bind_judge",
    "server_min_epoch_secs_reads_rfc3339",
    "server_min_http_",
    "server_min_hub_sends_to_each_subscriber",
    "server_min_parse_refuses_broken_lines",
    "server_min_percent_decode",
    "server_read_",
    "server_read_board_changed_within_5s",
    "server_read_doc_copies_verdicts",
    "server_read_empty_texts_are_unread",
    "server_read_watch_board_sends_on_marks",
    "server_reap_",
    "server_seat_",
    "server_src_",
    "server_src_marks_are_two_files",
    "server_src_unstartable_bd_is_unknown",
    "server_src_watch_",
    "server_src_watch_marks_trigger_reread",
    "server_view_",
    "server_view_http_",
    "sesplit_",
    "sgrace_",
    "sha256_",
    "shb_",
    "skeleton_",
    "smore_",
    "smore_dom_text",
    "stage_",
    "stage_cdp_",
    "stage_term_",
    "stats_",
    "stats_detail_account_takes_the_last",
    "stats_epoch_secs_reads_rfc",
    "stats_read_drops_tombstones_and_reads_parent_and_blocks",
    "stats_stage_of_closed_table",
    "stbp_",
    "stcli_",
    "steady_",
    "steady_details_use_fold_record",
    "steady_net_uses_settle_but_lose_all_does_not",
    "sthr_",
    "stnfy_",
    "stskill_",
    "sttgt_",
    "sxaxis_",
    "tgall_",
    "tgown_",
    "ticker_",
    "ticker_net_holds_one_interval",
    "tipx_",
    "tipx_dom_text",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "tzent_",
    "tzpar_",
    "tzself_",
    "udacct_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "vocab_",
    "wsteady_",
    "wstrip_",
];

/// (5) この file の歯の名はちょうど 5 で、どれも bvmore_ で始まり、接頭辞と残りの字は filter の語と重ならない。
#[test]
fn bvmore_own_names_clean() {
    let prefix = "bvmore_";
    for w in FILTERS {
        assert!(!prefix.contains(w), "接頭辞が filter の語 {w} を含む");
        assert!(!w.contains(prefix), "filter の語 {w} が接頭辞を含む");
    }
    let text = read("tests/bvmore.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .map(|w| {
            w[1].strip_prefix("fn ")
                .and_then(|r| r.split('(').next())
                .expect("test の属性の次の行は fn")
        })
        .collect();
    assert_eq!(names.len(), 5, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix(prefix)
            .unwrap_or_else(|| panic!("{name} が {prefix} で始まらない"));
        for w in FILTERS {
            assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
        }
    }
}
