//! 区画の行を群の枠の列と口座の占有に数えない歯（行 c-acct-park・接頭辞 apark_）。
//! fixture: tests/fixtures/account/acct-inputs.json（host の側の字と、期待の口座の列・群の枠の列）。
//! 区画の宣言と doctor の区画の行は fixture に足さず、歯の中で字に足す。

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tsuzuri_contract::account::{AccountRow, GroupCard};
use tsuzuri_contract::board::{GroupRow, Reading};
use tsuzuri_contract::seat::Pressure;
use tsuzuri_core::account::host::{HostTexts, PARK_KIND, accounts, declaration, groups};
use tsuzuri_core::seat::{SeatTexts, card};

const FIXTURE: &str = "tests/fixtures/account/acct-inputs.json";

/// 区画の宣言（fixture の群の宣言の後ろ・[seat] の前に足す・heartbeat の key は読み捨てる）。
const LOT_TOML: &str = "[[account-group]]\nname = \"lot\"\nanchors = [\"/work/proj-d\"]\naccounts = [\"acct-3\", \"acct-4\"]\nheartbeat = \"off\"\n\n";

/// 区画の anchor。
const LOT_ANCHOR: &str = "/work/proj-d";

/// 器の草稿の区画の行の pressure の字の 7 つ。
const PRESSURES: [&str; 7] = [
    "5h:91/90",
    "7d:80/85",
    "model:95/90",
    "-",
    "unmeasured",
    "unreadable",
    "no-rule",
];

#[derive(Deserialize)]
struct Inputs {
    texts: HostTexts,
    accounts: Reading<Vec<AccountRow>>,
    groups: Reading<Vec<GroupCard>>,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    let path: &Path = &root().join(path);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

fn inputs() -> Inputs {
    serde_json::from_str(&read(FIXTURE)).expect("fixture の形")
}

fn known<T: std::fmt::Debug>(r: Reading<T>) -> T {
    match r {
        Reading::Known(v) => v,
        Reading::Unknown => panic!("Unknown"),
    }
}

/// 字の一部を置き換えた字。
fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "{from} が字に無い");
    text.replacen(from, to, 1)
}

/// fixture の群の宣言の後ろに区画の宣言を足した字。
fn lot_toml(host: &str) -> String {
    edit(host, "[seat]\n", &format!("{LOT_TOML}[seat]\n"))
}

/// 区画の名・kind の欄・current・pressure の区画の行（kind が None なら欄 kind の無い行）。
fn row(name: &str, kind: Option<&str>, current: &str, pressure: &str) -> String {
    let kind = kind.map(|k| format!(" kind={k}")).unwrap_or_default();
    format!(
        "group={name}{kind} accounts=acct-3,acct-4 anchors=1 seat-accounts=none current={current} next=- refused=- pressure={pressure}\n"
    )
}

/// doctor の群の行の前（`before`）か後ろに行を足した字。
fn with_row(doctor: &str, line: &str, before: bool) -> String {
    let at = if before {
        "group=main "
    } else {
        "account=acct-1 "
    };
    edit(doctor, at, &format!("{line}{at}"))
}

/// 宣言と doctor を差し替えた host の字。
fn texts_with(base: &HostTexts, host: String, doctor: String) -> HostTexts {
    HostTexts {
        host_toml: Some(host),
        doctor: Some(doctor),
        ..base.clone()
    }
}

#[test]
fn apark_lot_left_out() {
    assert_eq!(PARK_KIND, "park");
    let i = inputs();
    let host = i.texts.host_toml.clone().expect("宣言");
    let doctor = i.texts.doctor.clone().expect("doctor");
    let host = lot_toml(&host);
    // 区画の宣言は 3 つ目の群として読まれる（heartbeat の key は読み捨てる）。
    let d = declaration(&host);
    let names: Vec<&str> = d.groups.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, ["main", "aux", "lot"]);
    assert_eq!(
        d.groups[2].anchors.as_deref(),
        Some(&[LOT_ANCHOR.to_string()][..])
    );
    assert_eq!(
        d.groups[2].accounts.as_deref(),
        Some(&["acct-3".to_string(), "acct-4".to_string()][..])
    );
    let mut seen = 0;
    for pressure in PRESSURES {
        for before in [true, false] {
            let line = row("lot", Some(PARK_KIND), "-", pressure);
            let t = texts_with(&i.texts, host.clone(), with_row(&doctor, &line, before));
            assert_eq!(groups(&t), i.groups, "{pressure} {before}");
            assert_eq!(accounts(&t), i.accounts, "{pressure} {before}");
            // 名を Tier9 にしても同じ。
            let tier = texts_with(
                &i.texts,
                edit(&host, "name = \"lot\"", "name = \"Tier9\""),
                with_row(
                    &doctor,
                    &line.replacen("group=lot", "group=Tier9", 1),
                    before,
                ),
            );
            assert_eq!(groups(&tier), i.groups, "Tier9 {pressure} {before}");
            seen += 1;
        }
    }
    assert_eq!(seen, 14);
}

#[test]
fn apark_kind_alone_decides() {
    let i = inputs();
    let fixture_groups = known(i.groups.clone());
    let host = i.texts.host_toml.clone().expect("宣言");
    let doctor = i.texts.doctor.clone().expect("doctor");
    let lot = lot_toml(&host);
    // doctor に区画の名の行が無ければ Unknown のまま。
    assert_eq!(
        groups(&texts_with(&i.texts, lot.clone(), doctor.clone())),
        Reading::Unknown
    );
    // kind の無い行・kind=lot・kind=Park・名 Tier9 の kind の無い行は群の枠として 3 つ目に並ぶ。
    let cases = [
        ("lot", None),
        ("lot", Some("lot")),
        ("lot", Some("Park")),
        ("Tier9", None),
    ];
    for (name, kind) in cases {
        let host = edit(&lot, "name = \"lot\"", &format!("name = \"{name}\""));
        let line = row(name, kind, "-", "-");
        let t = texts_with(&i.texts, host, with_row(&doctor, &line, false));
        let cards = known(groups(&t));
        assert_eq!(cards.len(), 3, "{name} {kind:?}");
        assert_eq!(cards[..2], fixture_groups[..], "{name} {kind:?}");
        assert_eq!(cards[2].row.group, name);
        assert_eq!(cards[2].row.account, "-", "今の口座は行の current の写し");
        assert_eq!(cards[2].row.candidates, ["acct-3", "acct-4"]);
    }
    // 宣言に無い名の区画の行は読まない。
    let ghost = row("ghost", Some(PARK_KIND), "acct-4", "-");
    let t = texts_with(&i.texts, host.clone(), with_row(&doctor, &ghost, true));
    assert_eq!(groups(&t), i.groups);
    assert_eq!(accounts(&t), i.accounts);
    // current=acct-3 の区画の行は acct-3 の占有に数えない。
    let occupied = |kind: Option<&str>| {
        let line = row("lot", kind, "acct-3", "-");
        let t = texts_with(&i.texts, lot.clone(), with_row(&doctor, &line, true));
        known(accounts(&t))
            .into_iter()
            .find(|a| a.label == "acct-3")
            .expect("acct-3")
            .occupant
    };
    assert_eq!(occupied(Some(PARK_KIND)), None);
    // kind の無い行なら占有に数える（歯が見分けを測っている）。
    assert_eq!(occupied(None).as_deref(), Some("lot"));
}

#[test]
fn apark_seat_card_copies_lot() {
    let i = inputs();
    let host = lot_toml(i.texts.host_toml.as_deref().expect("宣言"));
    let doctor = i.texts.doctor.clone().expect("doctor");
    let expected: [(&str, Reading<Option<Pressure>>); 7] = [
        ("5h:91/90", Reading::Known(Some(pressure("5h", 91, 90)))),
        ("7d:80/85", Reading::Known(Some(pressure("7d", 80, 85)))),
        (
            "model:95/90",
            Reading::Known(Some(pressure("model", 95, 90))),
        ),
        ("-", Reading::Known(None)),
        ("unmeasured", Reading::Unknown),
        ("unreadable", Reading::Unknown),
        ("no-rule", Reading::Unknown),
    ];
    for (word, want) in expected {
        let line = row("lot", Some(PARK_KIND), "-", word);
        let texts = SeatTexts {
            doctor: Some(with_row(&doctor, &line, false)),
            usage: i.texts.usage.clone(),
            host_toml: Some(host.clone()),
            ..SeatTexts::default()
        };
        let c = card("proj-d:0.1", Some(LOT_ANCHOR), &texts, 1_790_510_400);
        assert_eq!(
            c.group,
            Reading::Known(GroupRow {
                group: "lot".into(),
                account: "-".into(),
                candidates: vec!["acct-3".into(), "acct-4".into()],
                next_account: Some("-".into()),
                remaining: Vec::new(),
            }),
            "{word}"
        );
        assert_eq!(c.pressure, want, "{word}");
        assert_eq!(c.refused, Reading::Known(None), "{word}");
    }
}

fn pressure(window: &str, used: u32, cap: u32) -> Pressure {
    Pressure {
        window: window.into(),
        used,
        cap,
    }
}

#[test]
fn apark_own_names_clean() {
    assert_eq!(WORDS.len(), 291);
    let text = read("crates/tsuzuri-core/tests/apark.rs");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<String> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の次の行は fn");
            rest[..rest.find('(').expect("fn の名の後に (")].to_string()
        })
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("apark_")
            .unwrap_or_else(|| panic!("{name} は apark_ で始まる"));
        for w in WORDS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}

/// 起草の時の main（3fcb0d18）の契約表の verify の nextest の最後の引数の filter の語（291 語）。
const WORDS: &[&str] = &[
    "aaround_",
    "aaround_src_ask_fold_embeds",
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
    "cfsplit_",
    "cg9_",
    "cgdom_",
    "cmark_",
    "cnote_",
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
    "ecache_",
    "ecache_handle_wiring_text",
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "frame_home_blocks_in_order_and_map_page",
    "frame_one_module_per_block",
    "fserve_",
    "fstop_",
    "fstop_usage_errors_are_one",
    "fxpre_",
    "g3g7_",
    "gacct_",
    "gapspage_",
    "gapspage_page_frame_and_nav",
    "gatt_",
    "gbnote_",
    "gchip_",
    "gcoach_",
    "gfix_",
    "gfresh_",
    "gfresh_boards_text",
    "ghb_",
    "gins_",
    "gjst_",
    "glabel_",
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
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
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
    "ticker_",
    "ticker_net_holds_one_interval",
    "tipx_",
    "tipx_dom_text",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "wsteady_",
    "wstrip_",
];
