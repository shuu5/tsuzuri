//! 依存の監査の歯（行 k-deny・条 P-26.5）: 根の deny.toml の節と鍵が閉じていること、licenses の allow が規則の行 R-2 の
//! 値の許可一覧の写しであること、ci.yml の job deny が全面の 1 行を撃つこと、この file の歯の名が filter の語と重ならないこと。
//! 外の依存を使わず、repo の根からの相対の path で file の字を読む。
#![cfg(test)]

use std::path::PathBuf;

const RUN_LINE: &str = "cargo deny check -D unmatched-skip -D advisory-not-detected";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask は workspace の root の直下に在る")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// deny.toml の 1 つの鍵: 名と、値（1 行の字か、複数行の配列の 1 行 1 つの項目）。
struct Key {
    name: String,
    value: String,
    items: Vec<String>,
}

/// 節の見出しと `鍵 = 値` の行と、複数行の配列（行 `]` で閉じる・1 行 1 つの項目）を読む。注と空の行は飛ばす。
fn parse(text: &str) -> Vec<(String, Vec<Key>)> {
    let mut sections: Vec<(String, Vec<Key>)> = Vec::new();
    let mut open: Option<Key> = None;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if let Some(key) = open.as_mut() {
            if t == "]" {
                let key = open.take().expect("開いた配列");
                sections.last_mut().expect("節の中").1.push(key);
            } else {
                key.items.push(t.trim_end_matches(',').to_string());
            }
            continue;
        }
        if let Some(name) = t.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            sections.push((name.to_string(), Vec::new()));
            continue;
        }
        let (name, value) = t
            .split_once(" = ")
            .unwrap_or_else(|| panic!("鍵 = 値の行でも節の見出しでもない: {t}"));
        let key = Key {
            name: name.to_string(),
            value: value.to_string(),
            items: Vec::new(),
        };
        if value == "[" {
            open = Some(key);
        } else {
            sections.last_mut().expect("節の中").1.push(key);
        }
    }
    assert!(open.is_none(), "閉じない配列が在る");
    sections
}

/// inline の表 `{ a = "x", b = "y" }` の (鍵, 字) の列。
fn fields(item: &str) -> Vec<(String, String)> {
    let body = item
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or_else(|| panic!("inline の表でない: {item}"))
        .trim();
    let mut out = Vec::new();
    let mut rest = body;
    while !rest.is_empty() {
        let (name, after) = rest
            .split_once(" = \"")
            .unwrap_or_else(|| panic!("鍵 = \"字\" の形でない: {rest}"));
        let end = after
            .find('"')
            .unwrap_or_else(|| panic!("字が閉じない: {after}"));
        out.push((name.trim().to_string(), after[..end].to_string()));
        rest = after[end + 1..].trim_start_matches(',').trim();
    }
    out
}

fn key<'a>(section: &'a [Key], name: &str) -> &'a Key {
    section
        .iter()
        .find(|k| k.name == name)
        .unwrap_or_else(|| panic!("鍵 {name} が無い"))
}

/// 規則の行 R-2 の value の字 `ライセンスは … の許可一覧` の間を ` / ` で割った名の列。
fn rule_r2_licenses() -> Vec<String> {
    let rules = read("design-intent/rules.yaml");
    let row = rules
        .lines()
        .find(|l| l.contains("{id: R-2,"))
        .expect("規則の行 R-2");
    let after = row.split_once("ライセンスは ").expect("R-2 の字 ライセンスは").1;
    let list = after.split_once(" の許可一覧").expect("R-2 の字 の許可一覧").0;
    list.split(" / ").map(str::to_string).collect()
}

#[test]
fn kdeny_allow_copies_rule_r2() {
    let toml = parse(&read("deny.toml"));
    let licenses = &toml.iter().find(|(n, _)| n == "licenses").expect("節 licenses").1;
    let allow: Vec<String> = key(licenses, "allow")
        .items
        .iter()
        .map(|i| i.trim_matches('"').to_string())
        .collect();
    assert_eq!(allow, rule_r2_licenses(), "allow は R-2 の許可一覧の写し");
}

#[test]
fn kdeny_config_closed() {
    let toml = parse(&read("deny.toml"));
    let shape: Vec<(&str, Vec<&str>)> = toml
        .iter()
        .map(|(n, ks)| (n.as_str(), ks.iter().map(|k| k.name.as_str()).collect()))
        .collect();
    assert_eq!(
        shape,
        [
            ("advisories", vec!["version", "ignore"]),
            ("bans", vec!["multiple-versions", "skip"]),
            ("licenses", vec!["include-dev", "include-build", "allow"]),
            ("sources", vec!["unknown-registry", "unknown-git"]),
        ],
        "節と鍵"
    );
    let get = |section: &str, name: &str| -> &Key {
        key(&toml.iter().find(|(n, _)| n == section).expect("節").1, name)
    };
    assert_eq!(get("advisories", "version").value, "2");
    assert_eq!(get("bans", "multiple-versions").value, "\"deny\"");
    assert_eq!(get("licenses", "include-dev").value, "true");
    assert_eq!(get("licenses", "include-build").value, "true");
    assert_eq!(get("sources", "unknown-registry").value, "\"deny\"");
    assert_eq!(get("sources", "unknown-git").value, "\"deny\"");

    let field = |item: &str, want: &str| -> String {
        let f = fields(item);
        assert_eq!(f.len(), 2, "entry の鍵は {want} と reason だけ: {item}");
        assert_eq!(f[0].0, want, "{item}");
        assert_eq!(f[1].0, "reason", "{item}");
        assert!(!f[1].1.trim().is_empty(), "reason が空: {item}");
        f[0].1.clone()
    };

    let skip = &get("bans", "skip").items;
    let crates: Vec<String> = skip.iter().map(|i| field(i, "crate")).collect();
    assert_eq!(
        crates,
        [
            "convert_case@0.6.0",
            "syn@2.0.119",
            "thiserror@1.0.69",
            "thiserror-impl@1.0.69"
        ],
        "skip の crate"
    );

    let ignore = &get("advisories", "ignore").items;
    let ids: Vec<String> = ignore.iter().map(|i| field(i, "id")).collect();
    assert_eq!(ids, ["RUSTSEC-2024-0436", "RUSTSEC-2026-0173"], "ignore の id");
    for item in ignore {
        for want in [
            "判断の記録 ADR-3 の決定 (3)",
            "裁定 id = user 2026-09-24T22:07Z",
        ] {
            assert!(item.contains(want), "ignore の reason に {want} が無い: {item}");
        }
    }
}

/// job deny の行（見出し `  deny:` の次から、4 つの空白で始まる行が続く間）。
fn job_deny(ci: &str) -> Vec<&str> {
    ci.lines()
        .skip_while(|l| *l != "  deny:")
        .skip(1)
        .take_while(|l| l.starts_with("    "))
        .collect()
}

#[test]
fn kdeny_ci_job_runs_full_check() {
    let ci = read(".github/workflows/ci.yml");
    let job = job_deny(&ci);
    assert!(!job.is_empty(), "ci.yml に job deny が在る");
    let lines: Vec<&str> = job.iter().map(|l| l.trim()).collect();

    let pinned = lines.iter().any(|l| {
        l.strip_prefix("- uses: actions/checkout@")
            .is_some_and(|r| r.split_whitespace().next().is_some_and(is_sha))
    });
    assert!(pinned, "actions/checkout は SHA で pin する: {lines:?}");
    assert!(
        lines.contains(&"tool: cargo-deny@0.20.2"),
        "install-action の tool: {lines:?}"
    );

    let runs: Vec<&&str> = lines.iter().filter(|l| l.starts_with("- run:")).collect();
    assert_eq!(runs, [&format!("- run: {RUN_LINE}").as_str()], "run の行は 1 つ");

    let with_words = ci.lines().filter(|l| l.contains("cargo deny")).count();
    assert_eq!(with_words, 1, "ci.yml の中で字 cargo deny を持つ行は 1 つ");
}

fn is_sha(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// 起草の時の main a2a39947 の契約表の verify の最後の字（この行の語を除く）。
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
    "f159_",
    "f2ret_",
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
    "inject_",
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
fn kdeny_own_names_clean() {
    assert_eq!(WORDS.len(), 321);
    let text = read("xtask/tests/teeth1/deny.rs");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の次の行は fn");
            &rest[..rest.find('(').expect("fn の名の後に (")]
        })
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("kdeny_")
            .unwrap_or_else(|| panic!("{name} は kdeny_ で始まる"));
        for w in WORDS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
