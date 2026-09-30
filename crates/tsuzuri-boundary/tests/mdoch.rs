//! doctor の持ち回しの上限を 60 秒にする歯（行 m-doctor-hold・接頭辞 mdoch_・判断の記録 ADR-26 の決定 (2)）。
//! 頭ごとの上限の値と、境界の src の配線の字と、歯の名の重なりの無さを見る（撃ちの数は数えない）。
#![cfg(test)]

use std::fs;
use std::path::Path;
use std::time::Duration;

use tsuzuri_boundary::server::seat::{
    DOCTOR_ARGS, DOCTOR_HOLD, HOLD, SLOW_HOLD, TICK_ARGS, USAGE_ARGS, ceiling,
};

/// 行 m-doctor-hold の 19 節の filter の語の一覧 356 語（起草の時の main 754442db の契約表の verify の最後の字・字のまま・空白で区切る）。
const FILTER_WORDS: &str = concat!(
    "aaround_ aaround_src_ask_fold_embeds abss_ abst_ accept_ acchold_ account_ acctcore_ ",
    "acctcore_pure_and_no_new_dependencies acctdoc_ acctframe_ accthb_ accthome_ acctled_ acctlook_ ",
    "acctpcore_ acctproj_ acctsess_ acctwin_ acctwire_ acctwire_back_part_before_header ",
    "acctwire_back_steps aface_ afocus_ afocus_src_links_and_scroll alean_ anchor_ aord_ aown_ apark_ ",
    "apop_ areread_ askcard_ askcard_page_frame_and_nav athr_ batchpanel_ ",
    "batchpanel_page_frame_two_columns bhalf_ board_min_ bport_ brand_ ",
    "brand_path_written_once_in_contract btuck_ cadl_ cadopt_ cadq_ cdorm_ cexcl_ cfsplit_ cg9_ ",
    "cgdom_ check_canonical_design_intent_passes cishard_ cmark_ cnote_ cnret_ contract_form_ cround_ ",
    "csled_ cspk_ ctick_ cupd_ cupdlist_ denv_ dnedge_ dngrp_ dnrow_ dnskip_ dretry_ dstg_ ecache_ ",
    "ecache_handle_wiring_text eheld_acct_ eheld_design_ eheld_held_ eheld_marks_ eheld_vessel_ ",
    "elazy_ epolq_ eretry_ esig_ evkind_ f123_ f125_ f152_ f159_ f174_ f181_ f185_ f190_ f192_ f212_ ",
    "f2ret_ f76_ f89_ f98_ f99_ face_ fdlt_ fdlv_ fdrop_ flight_ fmark_ fprem_ frame_ ",
    "frame_home_blocks_in_order_and_map_page frame_one_module_per_block fserve_ fstop_ ",
    "fstop_usage_errors_are_one fundl_ fxpre_ g3g7_ gacct_ gapspage_ gapspage_page_frame_and_nav ",
    "gate_ gatt_ gbnote_ gchip_ gcoach_ gext_ gfix_ gfresh_ gfresh_boards_text ghb_ gins_ gjst_ ",
    "glabel_ gmret_ gmretw_ gnav_ gpface_ gpill_ gpulse_ gpulse_boards_text gquestion_ graph_ gsum_ ",
    "gtuck_ gview_ gwv_ hacols_ harest_ hasplit_ hbconf_ hbconf_config_moves_to_config_rs ",
    "hbconf_main_parse_fills_the_rest_with_new hbmark_ hbon_ hbpost_ hbpost_mod_rs_drops_post_fns ",
    "hbproc_ hbroute_ hbroute_mod_rs_keeps_only_events hcard_ hcled_ hcnx_ hcproj_ hcsess_ hdchip_ ",
    "hfig_ hnunk_ hook_ hruling_ hsblock_ hsblock_dispatch_by_list hsblock_ids_match_consts ",
    "hsblock_kit_holds_shared hsderive_ hsderive_consts_match_note hshist_ hspage_ ",
    "hspage_snaps_match_files hspk_ hsym_ hthr_ http_ hwstore_ iclose_ iclose_handle_wiring_text ",
    "ilink_ inject_ jcount_ jrun_ kcli_ kdeny_ kg9_ kindlab_ klink_ klint_ klintf_ ksize_ ksum_ ",
    "lateface_ launch_ lcard_ ledgerblock_ lgrp_ lhome_ lidle_ lineage_ lkind_ lresume_ lsnap_ ",
    "lspark_ lstg_ lstore_ mapgraph_ mapview_ mapview_classes_and_keys_exist mbig_ mkeys_ mlink_ ",
    "mqask_ mqface_ mstore_ mstore_module_text mtips_ mtree_ nact_ nbatch_ ncard_ nextstep_ nodepage_ ",
    "nodepage_frame_and_nav nstall_ nsum_ nsum_core_reads_no_files nsumw_ ntc_ ntime_ nxact_ nxorg_ ",
    "p106_ p1_commands_closed_list parts_ parts_paths_distinct_in_block_modules pci_ pclosed_ pfold_ ",
    "pgsw_ pgz_ pipe_ pkac_ pkview_ plimit_ pmisfit_ pmore_ popapp_ pquest_ pqueue_ project_ prose_ ",
    "ptitle_ pubfp_ pubscan_ punmap_ pwhole_ qblock_ qgate_ qkey_ qsig_ question_ rbusy_ relay_ ",
    "retired_render_ rhold_ rhold_wiring_text rsid_ runsdoc_ rvk_ saxis_ saxis_src_draws_legend_axis ",
    "sclosed_ seatblock_ seatcard_ server::design::tests:: server::events:: server::events::tests:: ",
    "server_ server_acct_ server_ask_ server_ask_http_ server_batch_ ",
    "server_batch_policy_scope_and_refusals server_coalesce_ server_hb_ server_min_ ",
    "server_min_bind_judge server_min_epoch_secs_reads_rfc3339 server_min_http_ ",
    "server_min_hub_sends_to_each_subscriber server_min_parse_refuses_broken_lines ",
    "server_min_percent_decode server_read_ server_read_board_changed_within_5s ",
    "server_read_doc_copies_verdicts server_read_empty_texts_are_unread ",
    "server_read_watch_board_sends_on_marks server_reap_ server_seat_ server_src_ ",
    "server_src_marks_are_two_files server_src_unstartable_bd_is_unknown server_src_watch_ ",
    "server_src_watch_marks_trigger_reread server_view_ server_view_http_ sesplit_ sgrace_ sha256_ ",
    "shb_ skeleton_ smore_ smore_dom_text stage_ stage_cdp_ stage_term_ stats_ ",
    "stats_detail_account_takes_the_last stats_epoch_secs_reads_rfc ",
    "stats_read_drops_tombstones_and_reads_parent_and_blocks stats_stage_of_closed_table stbp_ stcli_ ",
    "steady_ steady_details_use_fold_record steady_net_uses_settle_but_lose_all_does_not sthr_ ",
    "stnfy_ stskill_ sttgt_ sxaxis_ tgall_ tgown_ ticker_ ticker_net_holds_one_interval tipx_ ",
    "tipx_dom_text tkad_ tlic_ topbar_ topfit_ tz_ tzent_ tzpar_ tzself_ udacct_ udash_ unow_ ",
    "urpanel_ uword_ vocab_ wsteady_ wstrip_"
);

fn src(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("{path} を読む: {e}"))
}

/// fn の本体の字（字下げの後が `start` で始まる最初の行から、その行と同じ字下げの閉じの括弧だけの行まで）。
fn body(text: &str, start: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let first = lines
        .iter()
        .position(|l| l.trim_start().starts_with(start))
        .unwrap_or_else(|| panic!("{start} の行が無い"));
    let indent = &lines[first][..lines[first].len() - lines[first].trim_start().len()];
    let close = format!("{indent}}}");
    let last = lines[first..]
        .iter()
        .position(|l| **l == *close)
        .unwrap_or_else(|| panic!("{start} の閉じの括弧が無い"));
    lines[first..=first + last].join("\n")
}

fn count(text: &str, needle: &str) -> usize {
    text.matches(needle).count()
}

#[test]
fn mdoch_holds_by_head() {
    assert_eq!(DOCTOR_HOLD, Duration::from_secs(60));
    assert_eq!(SLOW_HOLD, Duration::from_secs(30));
    assert_eq!(HOLD, Duration::from_secs(5));
    assert_eq!(ceiling(&DOCTOR_ARGS), DOCTOR_HOLD);
    assert_eq!(ceiling(&USAGE_ARGS), SLOW_HOLD);
    assert_eq!(ceiling(&TICK_ARGS), HOLD);
}

#[test]
fn mdoch_reads_wiring_text() {
    let seat = src("src/server/seat.rs");
    let read_held = body(&seat, "pub fn read_held(");
    assert_eq!(count(&read_held, "ceiling(head)"), 1, "read_held の上限");
    let acct = src("src/acct.rs");
    let held_rule = body(&acct, "fn held_rule(");
    assert_eq!(count(&held_rule, "SLOW_HOLD"), 1, "held_rule の上限");
    assert_eq!(count(&acct, "DOCTOR_HOLD"), 0, "acct.rs は DOCTOR_HOLD を持たない");
}

#[test]
fn mdoch_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 356, "filter の語の数");
    let text = src("tests/mdoch.rs");
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
    let prefix = "mdoch_";
    for word in &words {
        assert!(!word.contains(prefix), "{word} が {prefix} を含む");
        assert!(!prefix.contains(word), "{word} が {prefix} に含まれる");
    }
    for name in names {
        let rest = name
            .strip_prefix(prefix)
            .unwrap_or_else(|| panic!("{name} は {prefix} で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
