//! 持ち込んだ folio2/ の下の、道具が名で探す file の退役（行 t-carry-retire・判断の記録 ADR-18 の決定 (6)・要件 NFR3）。
//! 外の依存を使わず、repo の根（xtask の manifest の dir の 1 つ上）からの相対の path で木を見る。

use std::collections::BTreeSet;
use std::path::PathBuf;

const FOLIO2: &str = "folio2";
const RETIRED: &str = "folio2/retired";

/// 移した 11 の対（元・先）。元は folio2/ の下の元の path・先は folio2/retired/ の下。
const MOVED: [(&str, &str); 11] = [
    ("folio2/CLAUDE.md", "folio2/retired/claude-md.txt"),
    (
        "folio2/.beads/.gitignore",
        "folio2/retired/beads/gitignore.txt",
    ),
    ("folio2/.beads/PRIME.md", "folio2/retired/beads/PRIME.md"),
    ("folio2/.beads/README.md", "folio2/retired/beads/README.md"),
    (
        "folio2/.beads/config.yaml",
        "folio2/retired/beads/config.yaml",
    ),
    (
        "folio2/.beads/metadata.json",
        "folio2/retired/beads/metadata.json",
    ),
    (
        "folio2/.github/workflows/ci.yml",
        "folio2/retired/github/workflows/ci.yml",
    ),
    ("folio2/.vessel", "folio2/retired/vessel.txt"),
    ("folio2/.vessel.toml", "folio2/retired/vessel.toml"),
    (
        "folio2/rust-toolchain.toml",
        "folio2/retired/rust-toolchain.txt",
    ),
    ("folio2/scripts/bdw", "folio2/retired/scripts/bdw"),
];

/// 道具が名で探す名。
const TOOL_NAMES: [&str; 7] = [
    ".beads",
    ".github",
    ".vessel",
    ".vessel.toml",
    "CLAUDE.md",
    "rust-toolchain",
    "rust-toolchain.toml",
];

/// 残す 8 本（folio の歯の材料の CLAUDE.md 5 本を含む）。
const KEPT: [&str; 8] = [
    "folio2/.gitignore",
    "folio2/contracts/example.toml",
    "folio2/contracts/schema.toml",
    "folio2/tests/fixtures/inject/drift/CLAUDE.md",
    "folio2/tests/fixtures/inject/no-marker/CLAUDE.md",
    "folio2/tests/fixtures/inject/ok/CLAUDE.md",
    "folio2/tests/fixtures/inject/outside/CLAUDE.md",
    "folio2/tests/fixtures/inject/over-limit/CLAUDE.md",
];

/// 歩かない dir（cargo の出力）。
const SKIP: &str = "folio2/target";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask の dir の 1 つ上")
        .to_path_buf()
}

/// dir の下の全項目（file・dir・symlink）を repo の根からの相対の path で返す。symlink は辿らず、SKIP の dir は入らない。
fn entries(rel: &str, out: &mut Vec<String>) {
    let dir = repo_root().join(rel);
    let read = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{rel} を読む: {e}"));
    for entry in read {
        let entry = entry.unwrap_or_else(|e| panic!("{rel} の項目: {e}"));
        let name = entry.file_name().to_string_lossy().into_owned();
        let child = format!("{rel}/{name}");
        if child == SKIP {
            continue;
        }
        let kind = entry
            .file_type()
            .unwrap_or_else(|e| panic!("{child} の種別: {e}"));
        out.push(child.clone());
        if kind.is_dir() {
            entries(&child, out);
        }
    }
}

fn is_file(rel: &str) -> bool {
    std::fs::symlink_metadata(repo_root().join(rel)).is_ok_and(|m| m.file_type().is_file())
}

fn exists(rel: &str) -> bool {
    std::fs::symlink_metadata(repo_root().join(rel)).is_ok()
}

#[test]
fn f2ret_files_moved_to_retired() {
    for (from, to) in MOVED {
        assert!(!exists(from), "{from} が元の path に在る");
        assert!(is_file(to), "{to} が file として無い");
    }
    let mut all = Vec::new();
    entries(RETIRED, &mut all);
    let files: BTreeSet<&str> = all.iter().map(String::as_str).filter(|p| is_file(p)).collect();
    let want: BTreeSet<&str> = MOVED.iter().map(|(_, to)| *to).collect();
    assert_eq!(files, want, "{RETIRED} の下の file は移した 11 本だけ");
}

#[test]
fn f2ret_no_tool_names_under_folio2() {
    for kept in KEPT {
        assert!(is_file(kept), "{kept} が file として無い");
    }
    let mut all = Vec::new();
    entries(FOLIO2, &mut all);
    let found: Vec<&str> = all
        .iter()
        .map(String::as_str)
        .filter(|p| {
            let name = p.rsplit('/').next().unwrap_or(p);
            TOOL_NAMES.contains(&name)
        })
        .filter(|p| !KEPT.contains(p))
        .collect();
    assert!(found.is_empty(), "道具が名で探す名の項目が残る: {found:?}");
}

/// 起草の時の main 58c1da55 の契約表の verify の最後の字（この行の語を除く）。
const WORDS: &[&str] = &[
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
    "fdlt_",
    "fdlv_",
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
    "jcount_",
    "jrun_",
    "kcli_",
    "kg9_",
    "kindlab_",
    "klink_",
    "ksum_",
    "lateface_",
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
    "mapgraph_",
    "mapview_",
    "mapview_classes_and_keys_exist",
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
    "pquest_",
    "pqueue_",
    "project_",
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
    "rhold_",
    "rhold_wiring_text",
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
    "udacct_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "wsteady_",
    "wstrip_",
];

#[test]
fn f2ret_own_names_clean() {
    assert_eq!(WORDS.len(), 317);
    let text = std::fs::read_to_string(repo_root().join("xtask/tests/retire.rs"))
        .expect("xtask/tests/retire.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の次の行は fn");
            &rest[..rest.find('(').expect("fn の名の後に (")]
        })
        .collect();
    assert_eq!(names.len(), 3, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("f2ret_")
            .unwrap_or_else(|| panic!("{name} は f2ret_ で始まる"));
        for w in WORDS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
