//! tz の索引の読みの既定を tz 自身にする歯（行 k-tz-self・接頭辞 tzself_）。
//! tz の binary を PATH を空にして子の処理で撃ち、folio という名の program が無くても索引が読めることを見る。
#![cfg(test)]

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tsuzuri_boundary::cli::folio::program;

/// 行 k-tz-drop の 11 節の filter の語の一覧 331 語（main 7ea3a2d3 の契約表の verify の最後の字・字のまま・空白で区切る）。
const FILTER_WORDS: &str = concat!(
    "aaround_ aaround_src_ask_fold_embeds abss_ abst_ accept_ acchold_ account_ acctcore_ ",
    "acctcore_pure_and_no_new_dependencies acctdoc_ acctframe_ accthb_ accthome_ acctled_ acctlook_ ",
    "acctpcore_ acctproj_ acctsess_ acctwin_ acctwire_ acctwire_back_part_before_header ",
    "acctwire_back_steps aface_ afocus_ afocus_src_links_and_scroll alean_ aord_ aown_ apark_ apop_ ",
    "areread_ askcard_ askcard_page_frame_and_nav athr_ batchpanel_ ",
    "batchpanel_page_frame_two_columns bhalf_ board_min_ bport_ brand_ ",
    "brand_path_written_once_in_contract btuck_ cadl_ cadopt_ cadq_ cdorm_ cexcl_ cfsplit_ cg9_ ",
    "cgdom_ cishard_ cmark_ cnote_ cnret_ contract_form_ cround_ csled_ cspk_ ctick_ cupd_ cupdlist_ ",
    "denv_ dnedge_ dngrp_ dnrow_ dnskip_ dretry_ dstg_ ecache_ ecache_handle_wiring_text eheld_acct_ ",
    "eheld_design_ eheld_held_ eheld_marks_ eheld_vessel_ elazy_ epolq_ eretry_ esig_ evkind_ f123_ ",
    "f159_ f212_ f2ret_ fdlt_ fdlv_ flight_ fmark_ fprem_ frame_ frame_home_blocks_in_order_and_map_page ",
    "frame_one_module_per_block fserve_ fstop_ fstop_usage_errors_are_one fundl_ fxpre_ g3g7_ gacct_ ",
    "gapspage_ gapspage_page_frame_and_nav gatt_ gbnote_ gchip_ gcoach_ gext_ gfix_ gfresh_ ",
    "gfresh_boards_text ghb_ gins_ gjst_ glabel_ gmret_ gmretw_ gnav_ gpface_ gpill_ gpulse_ ",
    "gpulse_boards_text gquestion_ graph_ gsum_ gtuck_ gview_ gwv_ hacols_ harest_ hasplit_ hbconf_ ",
    "hbconf_config_moves_to_config_rs hbconf_main_parse_fills_the_rest_with_new hbmark_ hbon_ hbpost_ ",
    "hbpost_mod_rs_drops_post_fns hbproc_ hbroute_ hbroute_mod_rs_keeps_only_events hcard_ hcled_ ",
    "hcnx_ hcproj_ hcsess_ hdchip_ hfig_ hnunk_ hook_ hruling_ hsblock_ hsblock_dispatch_by_list ",
    "hsblock_ids_match_consts hsblock_kit_holds_shared hsderive_ hsderive_consts_match_note hshist_ ",
    "hspage_ hspage_snaps_match_files hspk_ hsym_ hthr_ http_ hwstore_ iclose_ ",
    "iclose_handle_wiring_text ilink_ inject_ jcount_ jrun_ kcli_ kdeny_ kg9_ kindlab_ klink_ klint_ ksum_ ",
    "lateface_ launch_ lcard_ ledgerblock_ lgrp_ lhome_ lidle_ lkind_ lresume_ lsnap_ lspark_ lstg_ ",
    "lstore_ mapgraph_ mapview_ mapview_classes_and_keys_exist mbig_ mkeys_ mlink_ mqask_ mqface_ mstore_ ",
    "mstore_module_text mtips_ mtree_ nact_ nbatch_ ncard_ nextstep_ nodepage_ ",
    "nodepage_frame_and_nav nstall_ nsum_ nsum_core_reads_no_files nsumw_ ntc_ ntime_ nxact_ nxorg_ ",
    "p106_ parts_ parts_paths_distinct_in_block_modules pci_ pclosed_ pfold_ pgsw_ pgz_ pipe_ pkac_ pkview_ ",
    "plimit_ pmisfit_ pmore_ popapp_ pquest_ pqueue_ project_ ptitle_ pubfp_ pubscan_ punmap_ pwhole_ ",
    "qblock_ qgate_ qkey_ qsig_ question_ rbusy_ relay_ rhold_ rhold_wiring_text rsid_ runsdoc_ rvk_ saxis_ ",
    "saxis_src_draws_legend_axis sclosed_ seatblock_ seatcard_ server::design::tests:: ",
    "server::events:: server::events::tests:: server_ server_acct_ server_ask_ server_ask_http_ ",
    "server_batch_ server_batch_policy_scope_and_refusals server_coalesce_ server_hb_ server_min_ ",
    "server_min_bind_judge server_min_epoch_secs_reads_rfc3339 server_min_http_ ",
    "server_min_hub_sends_to_each_subscriber server_min_parse_refuses_broken_lines ",
    "server_min_percent_decode server_read_ server_read_board_changed_within_5s ",
    "server_read_doc_copies_verdicts server_read_empty_texts_are_unread ",
    "server_read_watch_board_sends_on_marks server_reap_ server_seat_ server_src_ ",
    "server_src_marks_are_two_files server_src_unstartable_bd_is_unknown server_src_watch_ ",
    "server_src_watch_marks_trigger_reread server_view_ server_view_http_ sesplit_ sgrace_ shb_ ",
    "skeleton_ smore_ smore_dom_text stage_ stage_cdp_ stage_term_ stats_ ",
    "stats_detail_account_takes_the_last stats_epoch_secs_reads_rfc ",
    "stats_read_drops_tombstones_and_reads_parent_and_blocks stats_stage_of_closed_table stbp_ stcli_ ",
    "steady_ steady_details_use_fold_record steady_net_uses_settle_but_lose_all_does_not sthr_ ",
    "stnfy_ stskill_ sttgt_ sxaxis_ tgall_ tgown_ ticker_ ticker_net_holds_one_interval tipx_ ",
    "tipx_dom_text tkad_ tlic_ topbar_ topfit_ tz_ tzent_ tzpar_ udacct_ udash_ unow_ urpanel_ uword_ ",
    "wsteady_ wstrip_"
);

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn src(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("{path} を読む: {e}"))
}

/// 数えるのは字の出る数（歯の file 自身は字を持つので、数える字は別の file の中でだけ見る）。
fn count(text: &str, needle: &str) -> usize {
    text.matches(needle).count()
}

#[test]
fn tzself_program_prefers_value_then_self() {
    assert_eq!(program(Some("my-folio")), OsString::from("my-folio"));
    assert_eq!(program(Some("/opt/bin/folio")), OsString::from("/opt/bin/folio"));
    let exe = std::env::current_exe().expect("撃っている binary の path");
    assert_eq!(program(None), exe.into_os_string());
}

#[test]
fn tzself_design_view_reads_without_folio() {
    let work = std::env::temp_dir().join(format!("tzself-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    let repo = work.join("repo");
    let bare = work.join("bin");
    fs::create_dir_all(&bare).expect("空の PATH の dir");
    copy_dir(
        &root().join("folio2/design-intent"),
        &repo.join("design-intent"),
    );
    let run = |extra: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(["graph", "--design", "--repo"])
            .arg(&repo)
            .args(extra)
            .env("PATH", &bare)
            .current_dir(&work)
            .output()
            .expect("tz を撃つ");
        (
            out.status.code().expect("終了 code"),
            String::from_utf8(out.stdout).expect("標準出力は UTF-8"),
            String::from_utf8(out.stderr).expect("標準エラーは UTF-8"),
        )
    };
    let (rc, out, err) = run(&[]);
    assert_eq!(rc, 0, "{err}");
    assert!(out.starts_with(r#"{"nodes":[{"id":"#), "{err}");
    let (rc, _, err) = run(&["--folio", "folio"]);
    assert_eq!(rc, 2, "{err}");
    assert!(err.contains("設計の索引"), "{err}");
    let _ = fs::remove_dir_all(&work);
}

/// dir の下の file を写す（dir は作る）。
fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("写し先を作る");
    for entry in fs::read_dir(from).expect("写し元を読む") {
        let entry = entry.expect("dir の entry");
        let dest = to.join(entry.file_name());
        if entry.file_type().expect("file の種類").is_dir() {
            copy_dir(&entry.path(), &dest);
        } else {
            fs::copy(entry.path(), &dest).expect("file を写す");
        }
    }
}

#[test]
fn tzself_wiring_text() {
    let main = src("src/main.rs");
    let set = "config.folio = tsuzuri_boundary::cli::folio::program(folio);";
    assert_eq!(count(&main, set), 1, "main.rs の server の既定の代入");
    assert_eq!(count(&main, "(&mut config.folio, folio)"), 0, "program の列");
    let gate = src("src/hook/question_gate.rs");
    assert_eq!(count(&gate, "folio: crate::cli::folio::program(folio),"), 1);
    assert_eq!(count(&gate, "unwrap_or(FOLIO)"), 0);
    let graph = src("src/cli/graph.rs");
    assert_eq!(count(&graph, "folio: super::folio::program(folio),"), 1);
    assert_eq!(count(&graph, "unwrap_or(FOLIO)"), 0);
}

#[test]
fn tzself_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 331, "filter の語の数");
    let text = src("tests/tzself.rs");
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
            .strip_prefix("tzself_")
            .unwrap_or_else(|| panic!("{name} は tzself_ で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
