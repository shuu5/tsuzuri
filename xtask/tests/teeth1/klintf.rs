//! 行 k-lint-folio-tests と k-lint-folio-deny の歯（接頭辞 klintf_）: 持ち込んだ folio の歯の根が #![cfg(test)] を置くこと・
//! folio の 2 つの根が lint の属性を置かず workspace の lint の表が覆うこと・allow の属性を置かないこと。
//! 行 k-lint-folio-roots が 2 つの根の lint の一覧を消し、表が一覧 COVER の 15 の lint を持つことを見る形にした。
//! 外の依存を使わず、repo の根（xtask の manifest の dir の 1 つ上）からの相対の path で字を読む。
#![cfg(test)]

use crate::common::repo_root;
use std::fs;
use std::path::{Path, PathBuf};

/// 契約表の verify の filter の語（起草の時の main f17e983f の 357 語・空白で区切る）。
const FILTER_WORDS: &str = concat!(
    "aaround_ aaround_src_ask_fold_embeds abss_ abst_ accept_ acchold_ account_ acctcore_ ",
    "acctcore_pure_and_no_new_dependencies acctdoc_ acctframe_ accthb_ accthome_ acctled_ acctlook_ ",
    "acctpcore_ acctproj_ acctsess_ acctwin_ acctwire_ acctwire_back_part_before_header ",
    "acctwire_back_steps aface_ afocus_ afocus_src_links_and_scroll alean_ anchor_ aord_ aown_ apark_ apop_ ",
    "areread_ askcard_ askcard_page_frame_and_nav athr_ batchpanel_ batchpanel_page_frame_two_columns ",
    "bhalf_ board_min_ bport_ brand_ brand_path_written_once_in_contract btuck_ cadl_ cadopt_ cadq_ ",
    "cdorm_ cexcl_ cfsplit_ cg9_ cgdom_ check_canonical_design_intent_passes cishard_ cmark_ cnote_ ",
    "cnret_ contract_form_ cround_ csled_ cspk_ ctick_ cupd_ cupdlist_ denv_ dnedge_ dngrp_ dnrow_ ",
    "dnskip_ dretry_ dstg_ ecache_ ecache_handle_wiring_text eheld_acct_ eheld_design_ eheld_held_ ",
    "eheld_marks_ eheld_vessel_ elazy_ epolq_ eretry_ esig_ evkind_ f123_ f125_ f152_ f159_ f174_ f181_ f185_ ",
    "f190_ f192_ f212_ f2ret_ f76_ f89_ f98_ f99_ face_ fdlt_ fdlv_ fdrop_ flight_ fmark_ fprem_ frame_ frame_home_blocks_in_order_and_map_page ",
    "frame_one_module_per_block fserve_ fstop_ fstop_usage_errors_are_one fundl_ fxpre_ g3g7_ gacct_ ",
    "gapspage_ gapspage_page_frame_and_nav gate_ gatt_ gbnote_ gchip_ gcoach_ gext_ gfix_ gfresh_ ",
    "gfresh_boards_text ghb_ gins_ gjst_ glabel_ gmret_ gmretw_ gnav_ gpface_ gpill_ gpulse_ ",
    "gpulse_boards_text gquestion_ graph_ gsum_ gtuck_ gview_ gwv_ hacols_ harest_ hasplit_ hbconf_ ",
    "hbconf_config_moves_to_config_rs hbconf_main_parse_fills_the_rest_with_new hbmark_ hbon_ hbpost_ ",
    "hbpost_mod_rs_drops_post_fns hbproc_ hbroute_ hbroute_mod_rs_keeps_only_events hcard_ hcled_ ",
    "hcnx_ hcproj_ hcsess_ hdchip_ hfig_ hnunk_ hook_ hruling_ hsblock_ hsblock_dispatch_by_list ",
    "hsblock_ids_match_consts hsblock_kit_holds_shared hsderive_ hsderive_consts_match_note hshist_ ",
    "hspage_ hspage_snaps_match_files hspk_ hsym_ hthr_ http_ hwstore_ iclose_ ",
    "iclose_handle_wiring_text ilink_ inject_ jcount_ jrun_ kcli_ kdeny_ kg9_ kindlab_ klink_ klint_ ",
    "klintf_ ksize_ ksum_ lateface_ launch_ lcard_ ledgerblock_ lgrp_ lhome_ lidle_ lineage_ lkind_ lresume_ ",
    "lsnap_ lspark_ lstg_ lstore_ mapgraph_ mapview_ mapview_classes_and_keys_exist mbig_ mdoch_ mkeys_ ",
    "mlink_ mqask_ mqface_ mstore_ mstore_module_text mtips_ mtree_ nact_ nbatch_ ncard_ nextstep_ ",
    "nodepage_ nodepage_frame_and_nav nstall_ nsum_ nsum_core_reads_no_files nsumw_ ntc_ ntime_ ",
    "nxact_ nxorg_ p106_ p1_commands_closed_list parts_ parts_paths_distinct_in_block_modules pci_ ",
    "pclosed_ pfold_ pgsw_ pgz_ pipe_ pkac_ pkview_ plimit_ pmisfit_ pmore_ popapp_ pquest_ pqueue_ ",
    "project_ prose_ ptitle_ pubfp_ pubscan_ punmap_ pwhole_ qblock_ qgate_ qkey_ qsig_ question_ rbusy_ ",
    "relay_ retired_render_ rhold_ rhold_wiring_text rsid_ runsdoc_ rvk_ saxis_ saxis_src_draws_legend_axis sclosed_ ",
    "seatblock_ seatcard_ server::design::tests:: server::events:: server::events::tests:: server_ ",
    "server_acct_ server_ask_ server_ask_http_ server_batch_ server_batch_policy_scope_and_refusals ",
    "server_coalesce_ server_hb_ server_min_ server_min_bind_judge ",
    "server_min_epoch_secs_reads_rfc3339 server_min_http_ server_min_hub_sends_to_each_subscriber ",
    "server_min_parse_refuses_broken_lines server_min_percent_decode server_read_ ",
    "server_read_board_changed_within_5s server_read_doc_copies_verdicts ",
    "server_read_empty_texts_are_unread server_read_watch_board_sends_on_marks server_reap_ ",
    "server_seat_ server_src_ server_src_marks_are_two_files server_src_unstartable_bd_is_unknown ",
    "server_src_watch_ server_src_watch_marks_trigger_reread server_view_ server_view_http_ sesplit_ ",
    "sgrace_ sha256_ shb_ skeleton_ smore_ smore_dom_text stage_ stage_cdp_ stage_term_ stats_ ",
    "stats_detail_account_takes_the_last stats_epoch_secs_reads_rfc ",
    "stats_read_drops_tombstones_and_reads_parent_and_blocks stats_stage_of_closed_table stbp_ stcli_ ",
    "steady_ steady_details_use_fold_record steady_net_uses_settle_but_lose_all_does_not sthr_ ",
    "stnfy_ stskill_ sttgt_ sxaxis_ tgall_ tgown_ ticker_ ticker_net_holds_one_interval tipx_ ",
    "tipx_dom_text tkad_ tlic_ topbar_ topfit_ tz_ tzent_ tzpar_ tzself_ udacct_ udash_ unow_ ",
    "urpanel_ uword_ vocab_ wsteady_ wstrip_",
);

/// 2 つの根の頭の一覧が持っていた lint（lint・表・level）。規則の行 R-10 の 13 と条 P-24.3 の 2 つで、
/// 行 k-lint-folio-roots が根の一覧を消した後は根の Cargo.toml の workspace の lint の表が持つ。
const COVER: [(&str, &str, &str); 15] = [
    ("unused_must_use", "workspace.lints.rust", "deny"),
    ("unsafe_code", "workspace.lints.rust", "forbid"),
    ("unwrap_used", "workspace.lints.clippy", "deny"),
    ("expect_used", "workspace.lints.clippy", "deny"),
    ("panic", "workspace.lints.clippy", "deny"),
    ("todo", "workspace.lints.clippy", "deny"),
    ("unimplemented", "workspace.lints.clippy", "deny"),
    ("unreachable", "workspace.lints.clippy", "deny"),
    ("exit", "workspace.lints.clippy", "deny"),
    ("indexing_slicing", "workspace.lints.clippy", "deny"),
    ("dbg_macro", "workspace.lints.clippy", "deny"),
    ("print_stdout", "workspace.lints.clippy", "deny"),
    ("print_stderr", "workspace.lints.clippy", "deny"),
    ("allow_attributes", "workspace.lints.clippy", "deny"),
    ("allow_attributes_without_reason", "workspace.lints.clippy", "deny"),
];

fn folio_dir() -> PathBuf {
    repo_root().join("folio2/crates/folio")
}

/// folio の歯の file の根（tests/ の直下の .rs と tests/<dir>/main.rs）。
fn folio_test_roots() -> Vec<PathBuf> {
    let dir = folio_dir().join("tests");
    let mut roots = Vec::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display())) {
        let path = entry.expect("dir の 1 件").path();
        if path.is_dir() {
            let main = path.join("main.rs");
            if main.is_file() {
                roots.push(main);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            roots.push(path);
        }
    }
    roots.sort();
    roots
}

/// 見出し `[name]` の次の行から次の見出しの前までの、井桁で始まる行と空の行を除いた行（trim・見出しが無ければ空）。
fn section(text: &str, name: &str) -> Vec<String> {
    let head = format!("[{name}]");
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            inside = t == head;
        } else if inside && !t.is_empty() && !t.starts_with('#') {
            out.push(t.to_string());
        }
    }
    out
}

/// dir の下（下の dir も辿る）の字 .rs で終わる file。
fn rs_files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{} を読む: {e}", dir.display())) {
        let path = entry.expect("dir の 1 件").path();
        if path.is_dir() {
            rs_files_under(&path, out);
        } else if path.to_string_lossy().ends_with(".rs") {
            out.push(path);
        }
    }
}

#[test]
fn klintf_test_roots_cfg_test() {
    let mut seen = 0;
    for path in folio_test_roots() {
        let text =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
        let lines: Vec<&str> = text.lines().collect();
        let head = lines.iter().take_while(|l| l.starts_with("//!")).count();
        assert_eq!(
            lines.get(head).copied(),
            Some("#![cfg(test)]"),
            "{} の頭の //! の続きの直後が #![cfg(test)] でない",
            path.display()
        );
        let count = lines.iter().filter(|l| l.trim() == "#![cfg(test)]").count();
        assert_eq!(count, 1, "{} の #![cfg(test)] が {count} 行", path.display());
        seen += 1;
    }
    assert!(seen >= 1, "見た file が無い");
}

#[test]
fn klintf_src_roots_lint_attrs() {
    // binary の根 src/main.rs は行 k-tz-tests で folio2/retired/ へ退役した（folio の binary は無い）。
    for rel in ["src/lib.rs", "build.rs"] {
        let path = folio_dir().join(rel);
        let text =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
        for attr in ["#![forbid(", "#![deny(", "#![warn("] {
            assert!(!text.contains(attr), "{} が {attr} を含む", path.display());
        }
    }
    let manifest_path = folio_dir().join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("{} を読む: {e}", manifest_path.display()));
    assert_eq!(section(&manifest, "lints"), vec!["workspace = true"], "folio の manifest の節 lints");
    let root_path = repo_root().join("Cargo.toml");
    let root = fs::read_to_string(&root_path)
        .unwrap_or_else(|e| panic!("{} を読む: {e}", root_path.display()));
    for (lint, table, level) in COVER {
        let want = format!("{lint} = \"{level}\"");
        let rows = section(&root, table);
        assert!(rows.contains(&want), "根の Cargo.toml の節 {table} に {want} の行が無い");
    }
}

#[test]
fn klintf_no_allow_attrs() {
    let dir = folio_dir();
    let mut files = vec![dir.join("build.rs")];
    rs_files_under(&dir.join("src"), &mut files);
    rs_files_under(&dir.join("tests"), &mut files);
    assert!(files.len() > 1, "見た file が無い");
    for path in files {
        let text =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()));
        for (i, line) in text.lines().enumerate() {
            assert!(!line.contains("allow("), "{} の {} 行目が allow( を含む", path.display(), i + 1);
        }
    }
}

#[test]
fn klintf_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 357, "filter の語の数");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/klintf.rs");
    let text = fs::read_to_string(&path).expect("tests/teeth1/klintf.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 4, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("klintf_")
            .unwrap_or_else(|| panic!("{name} は klintf_ で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
