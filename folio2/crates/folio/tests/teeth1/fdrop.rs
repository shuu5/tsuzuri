//! 口 inject と serve の退役の歯（行 k-tz-drop・接頭辞 fdrop_）。folio の binary を撃たず、lib の入口 folio::entry::run を
//! 歯の process の中で撃つ（行 k-tz-tests が folio の binary を退役させても残る）。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// 起草の時の main 7ea3a2d3 の契約表の verify の最後の字の filter の語 331 語（字のまま・空白で区切る）。
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
    "iclose_handle_wiring_text ilink_ inject_ jcount_ jrun_ kcli_ kdeny_ kg9_ kindlab_ klink_ klint_ ",
    "ksum_ lateface_ launch_ lcard_ ledgerblock_ lgrp_ lhome_ lidle_ lkind_ lresume_ lsnap_ lspark_ ",
    "lstg_ lstore_ mapgraph_ mapview_ mapview_classes_and_keys_exist mbig_ mkeys_ mlink_ mqask_ ",
    "mqface_ mstore_ mstore_module_text mtips_ mtree_ nact_ nbatch_ ncard_ nextstep_ nodepage_ ",
    "nodepage_frame_and_nav nstall_ nsum_ nsum_core_reads_no_files nsumw_ ntc_ ntime_ nxact_ nxorg_ ",
    "p106_ parts_ parts_paths_distinct_in_block_modules pci_ pclosed_ pfold_ pgsw_ pgz_ pipe_ pkac_ ",
    "pkview_ plimit_ pmisfit_ pmore_ popapp_ pquest_ pqueue_ project_ ptitle_ pubfp_ pubscan_ punmap_ ",
    "pwhole_ qblock_ qgate_ qkey_ qsig_ question_ rbusy_ relay_ rhold_ rhold_wiring_text rsid_ ",
    "runsdoc_ rvk_ saxis_ saxis_src_draws_legend_axis sclosed_ seatblock_ seatcard_ ",
    "server::design::tests:: server::events:: server::events::tests:: server_ server_acct_ ",
    "server_ask_ server_ask_http_ server_batch_ server_batch_policy_scope_and_refusals ",
    "server_coalesce_ server_hb_ server_min_ server_min_bind_judge server_min_epoch_secs_reads_rfc3339 ",
    "server_min_http_ server_min_hub_sends_to_each_subscriber server_min_parse_refuses_broken_lines ",
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

/// folio の package の根（この歯の file の置かれた crate の dir）。
fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 退役の置き場（crate の dir から ../../retired/crates/folio）。
fn retired() -> PathBuf {
    crate_dir().join("../../retired/crates/folio")
}

/// folio の lib の入口を撃つ（終了 code・標準出力・標準エラー）。頭の命令の名は folio。
fn entry(args: &[&str]) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let rc = folio::entry::run(args.iter().copied(), &mut out, &mut err);
    (
        rc,
        String::from_utf8(out).expect("標準出力は UTF-8"),
        String::from_utf8(err).expect("標準エラーは UTF-8"),
    )
}

/// 口 `name` が無い: 断りの字（clap の知らない subcommand）・命令の一覧に無い・src と tests の file が無く退役の置き場に在る・
/// src/lib.rs に module の宣言の行が無い。
fn assert_dropped(name: &str, args: &[&str]) {
    let (rc, out, err) = entry(args);
    assert_eq!(rc, 2, "{name}: {err}");
    assert_eq!(out, "", "{name}");
    assert!(
        err.lines().next().is_some_and(|l| l.starts_with("error: unrecognized subcommand")),
        "{name}: {err}"
    );
    assert!(err.contains(&format!("'{name}'")), "{name}: {err}");

    let (rc, help, err) = entry(&["folio", "--help"]);
    assert_eq!(rc, 0, "{err}");
    let body = help.split("Commands:").nth(1).expect("Commands: の節が無い");
    let body = body.split("\n\n").next().unwrap_or(body);
    assert!(
        !body
            .lines()
            .filter_map(|l| l.strip_prefix("  "))
            .any(|l| l.split_whitespace().next() == Some(name)),
        "{name} が命令の一覧に在る: {help}"
    );

    for rel in [format!("src/{name}.rs"), format!("tests/{name}.rs")] {
        assert!(!crate_dir().join(&rel).exists(), "{rel} が元の path に在る");
        assert!(retired().join(&rel).is_file(), "{rel} が退役の置き場に file として無い");
    }
    let lib = fs::read_to_string(crate_dir().join("src/lib.rs")).expect("src/lib.rs を読む");
    let decl = format!("mod {name};");
    assert!(
        !lib.lines().any(|l| {
            let l = l.trim();
            l.strip_prefix("pub ").unwrap_or(l) == decl
        }),
        "src/lib.rs に {decl} の行が在る"
    );
}

#[test]
fn fdrop_no_inject() {
    assert_dropped("inject", &["folio", "inject", "--print"]);
}

#[test]
fn fdrop_no_serve() {
    assert_dropped("serve", &["folio", "serve", "--dir", "/nonexistent/fdrop-serve"]);
}

#[test]
fn fdrop_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 331, "filter の語の数");
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/fdrop.rs"))
        .expect("tests/teeth1/fdrop.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 3, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("fdrop_")
            .unwrap_or_else(|| panic!("{name} は fdrop_ で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
