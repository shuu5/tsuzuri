//! 持ち込む code への規則の除外の表 carry-exclusions.toml（行 t-carry-ex・判断の記録 ADR-18 の決定 (5)・ADR-19 の決定 (2)・要件 NFR3）。
//! 行 v-carry-ex（判断の記録 ADR-33 の決定 (14)）からは、持ち込んだ器 scribe2 の 4 行と器の錠の外の部品の数も見る。
//! 外の依存を使わず、repo の根（xtask の manifest の dir の 1 つ上）からの相対の path で字を読む。
#![cfg(test)]

use crate::common::repo_root;
use std::collections::BTreeSet;

/// 除外の表の file と、裁定 id を引く 2 つの判断の記録（repo の根からの相対）。
/// 持ち込んだ folio2/design-intent/adr/ の同じ名の file は読まない。
const TABLE: &str = "carry-exclusions.toml";
const ADRS: [&str; 2] = [
    "design-intent/adr/ADR-18.yaml",
    "design-intent/adr/ADR-19.yaml",
];
const LOCK: &str = "Cargo.lock";
/// 根の直下の dir のうち、持ち込んだ木の根の閉じた一覧。
const CARRIED: [&str; 2] = ["folio2", "scribe2"];

/// 欄 rule の閉じた一覧。
const RULES: [&str; 4] = ["R-4", "R-10", "R-2", "N-3"];
/// 表の 1 行が持つ鍵（この順・これだけ）。
const KEYS: [&str; 5] = ["rule", "path", "scope", "ruling", "removed_by"];

const RULING: &str = "t3-hub.67.4:20260929T0058Z-1";
const FOLIO_PATH: &str = "folio2/crates/folio/";
const SCOPE_R10: &str = "lint で deny にする書き方（unwrap・expect・panic・直接の print ほか）";

/// 器 scribe2 の行（行 v-carry-ex）。N-3 の行の裁定 id は ADR-19 の束の頭の字（裁定 id = t3-hub.67.7:… ほか）で読む。
const VESSEL_CRATES: &str = "scribe2/crates/";
const VESSEL_RULES: &str = "scribe2/rules/";
const RULING_N3: &str = "t3-hub.67.8:20260929T0609Z-1";
/// 器の錠（repo の根からの相対）と、その外の部品（行 source を持つ塊）の数（判断の記録 ADR-33 の決定 (5)）。
const VESSEL_LOCK: &str = "scribe2/Cargo.lock";
const VESSEL_OUTSIDE: usize = 35;

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

#[derive(Debug)]
struct Row {
    rule: String,
    path: String,
    scope: String,
    ruling: String,
    removed_by: String,
}

/// 表の読み。行は見出し [[exclusion]] と、鍵 = "字" の行だけ（空行と # の行は読み飛ばす）。
fn parse_table(text: &str) -> Result<Vec<Row>, String> {
    let mut rows: Vec<Vec<(String, String)>> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        let at = i + 1;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[exclusion]]" {
            rows.push(Vec::new());
            continue;
        }
        let Some((key, value)) = line.split_once(" = ") else {
            return Err(format!("{at} 行目: 鍵 = \"字\" の形でない: {line}"));
        };
        let Some(value) = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .filter(|v| !v.contains('"'))
        else {
            return Err(format!("{at} 行目: 値が引用符で囲んだ字でない: {line}"));
        };
        if !KEYS.contains(&key) {
            return Err(format!("{at} 行目: 知らない鍵 {key}"));
        }
        let Some(row) = rows.last_mut() else {
            return Err(format!("{at} 行目: 見出しの前の鍵 {key}"));
        };
        if row.iter().any(|(k, _)| k == key) {
            return Err(format!("{at} 行目: 鍵 {key} が 2 度"));
        }
        row.push((key.to_string(), value.to_string()));
    }
    rows.into_iter()
        .map(|pairs| {
            let get = |key: &str| {
                pairs
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone())
                    .ok_or_else(|| format!("鍵 {key} の無い行: {pairs:?}"))
            };
            Ok(Row {
                rule: get("rule")?,
                path: get("path")?,
                scope: get("scope")?,
                ruling: get("ruling")?,
                removed_by: get("removed_by")?,
            })
        })
        .collect()
}

/// path は字 / で終わり、節 .. と . を持たない dir で、根の直下の dir が持ち込んだ木の根の一覧 CARRIED のどれか。
fn judge_path(path: &str) -> Result<(), String> {
    let Some(body) = path.strip_suffix('/') else {
        return Err(format!("path {path} が字 / で終わらない"));
    };
    let parts: Vec<&str> = body.split('/').collect();
    if parts
        .iter()
        .any(|p| p.is_empty() || *p == "." || *p == "..")
    {
        return Err(format!("path {path} が空の節・. ・.. を持つ"));
    }
    if !repo_root().join(body).is_dir() {
        return Err(format!("path {path} は木の dir でない"));
    }
    if !CARRIED.contains(&parts[0]) {
        return Err(format!("{} は持ち込んだ木の根の一覧に無い", parts[0]));
    }
    Ok(())
}

fn is_removed_by_id(id: &str) -> bool {
    id.starts_with(|c: char| c.is_ascii_lowercase())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// 裁定 id が判断の記録の字に在るか。字 裁定 id = に続けて在るか、問いの id が全角の丸括弧で囲まれて在り、
/// 同じ時刻の束の頭の字（裁定 id = と、頭の問いの id と字 : と時刻と、空白と字 ほか）が在る。
fn ruling_in(adr: &str, ruling: &str) -> bool {
    const HEAD: &str = "裁定 id = ";
    if adr.contains(&format!("{HEAD}{ruling}")) {
        return true;
    }
    let Some((question, stamp)) = ruling.split_once(':') else {
        return false;
    };
    adr.contains(&format!("（{question}）"))
        && adr.match_indices(HEAD).any(|(at, _)| {
            adr[at + HEAD.len()..]
                .split_once(' ')
                .is_some_and(|(id, rest)| {
                    id.split_once(':').is_some_and(|(_, s)| s == stamp) && rest.starts_with("ほか")
                })
        })
}

/// 表の行の列の、repo の木での確かめ。
fn judge_rows(rows: &[Row]) -> Result<(), String> {
    let adr = ADRS.map(read).join("\n");
    let mut seen = BTreeSet::new();
    for row in rows {
        if !RULES.contains(&row.rule.as_str()) {
            return Err(format!("規則 {} は {RULES:?} のどれでもない", row.rule));
        }
        judge_path(&row.path)?;
        if !ruling_in(&adr, &row.ruling) {
            return Err(format!("裁定 id {} が判断の記録に無い", row.ruling));
        }
        if !is_removed_by_id(&row.removed_by) {
            return Err(format!("外す行の id {:?} の形が違う", row.removed_by));
        }
        if row.scope.is_empty() {
            return Err(format!("{} の scope が空", row.rule));
        }
        if !seen.insert((row.rule.clone(), row.path.clone())) {
            return Err(format!("規則 {} と path {} の 2 度目", row.rule, row.path));
        }
    }
    Ok(())
}

fn row_text(rule: &str, path: &str, scope: &str, ruling: &str, removed_by: &str) -> String {
    format!(
        "[[exclusion]]\nrule = \"{rule}\"\npath = \"{path}\"\nscope = \"{scope}\"\nruling = \"{ruling}\"\nremoved_by = \"{removed_by}\"\n"
    )
}

fn good_r10() -> String {
    row_text("R-10", FOLIO_PATH, SCOPE_R10, RULING, "k-lint-folio")
}

fn accepts(text: &str) -> Result<(), String> {
    judge_rows(&parse_table(text)?)
}

#[test]
fn cexcl_table_rows() {
    let rows = parse_table(&read(TABLE)).expect("表の読み");
    let got: Vec<[&str; 4]> = rows
        .iter()
        .map(|r| [&*r.rule, &*r.path, &*r.ruling, &*r.removed_by])
        .collect();
    assert_eq!(
        got,
        [
            ["R-4", VESSEL_CRATES, RULING, "v-size"],
            ["R-10", VESSEL_CRATES, RULING, "v-lint"],
            ["R-2", VESSEL_CRATES, RULING, "v-join"],
            ["N-3", VESSEL_RULES, RULING_N3, "v-flag-drop"],
        ],
        "表の行は器の 4 行（この順）"
    );
    judge_rows(&rows).expect("表の行の列は repo の木の確かめを通る");
}

#[test]
fn cexcl_refuses_bad_rows() {
    accepts(&good_r10()).expect("良い 1 行");
    let with = |rule, path, ruling, removed_by| row_text(rule, path, SCOPE_R10, ruling, removed_by);
    let bad: [(&str, String); 12] = [
        (
            "知らない鍵 enabled",
            format!("{}enabled = \"true\"\n", good_r10()),
        ),
        (
            "鍵 removed_by の無い行",
            good_r10().replace("removed_by = \"k-lint-folio\"\n", ""),
        ),
        ("規則 R-5", with("R-5", FOLIO_PATH, RULING, "k-lint-folio")),
        (
            "tsuzuri の core の path",
            with("R-10", "crates/tsuzuri-core/", RULING, "k-lint-folio"),
        ),
        (
            "xtask の path",
            with("R-10", "xtask/", RULING, "k-lint-folio"),
        ),
        (
            "節 .. を持つ path",
            with("R-10", "folio2/../crates/", RULING, "k-lint-folio"),
        ),
        (
            "字 / で終わらない path",
            with("R-10", "folio2/crates/folio", RULING, "k-lint-folio"),
        ),
        (
            "木に無い dir の path",
            with("R-10", "folio2/no-such-dir/", RULING, "k-lint-folio"),
        ),
        (
            "判断の記録に無い裁定 id",
            with(
                "R-10",
                FOLIO_PATH,
                "t3-hub.0.0:19700101T0000Z-1",
                "k-lint-folio",
            ),
        ),
        ("空の removed_by", with("R-10", FOLIO_PATH, RULING, "")),
        (
            "同じ規則と path の 2 行",
            format!(
                "{}{}",
                good_r10(),
                with("R-10", FOLIO_PATH, RULING, "k-lint-folio-2")
            ),
        ),
        ("見出しの前の鍵", format!("rule = \"R-10\"\n{}", good_r10())),
    ];
    for (label, text) in &bad {
        assert!(accepts(text).is_err(), "{label} を断らない");
    }
}

#[test]
fn cexcl_ruling_batch_form() {
    let scope = "器の規則の表の旗";
    let n3 = |ruling| row_text("N-3", VESSEL_RULES, scope, ruling, "v-flag-drop");
    accepts(&n3(RULING_N3)).expect("束の頭の字で読む N-3 の裁定 id");
    accepts(&row_text(
        "R-10",
        VESSEL_CRATES,
        SCOPE_R10,
        RULING,
        "v-lint",
    ))
    .expect("器の crate の R-10");
    for (label, ruling) in [
        ("束の時刻の違う裁定 id", "t3-hub.67.8:20260929T0610Z-1"),
        (
            "判断の記録が名指さない問いの id",
            "t3-hub.67.98:20260929T0609Z-1",
        ),
    ] {
        assert!(accepts(&n3(ruling)).is_err(), "{label} を断らない");
    }
    let adr = "（q.2）を足す。裁定 id = q.1:S-1 ほか・束 b:S-1";
    assert!(ruling_in(adr, "q.2:S-1"), "束の頭の字と問いの id");
    assert!(
        ruling_in("裁定 id = q.5:T-1（束 b）", "q.5:T-1"),
        "字 裁定 id = に続けて在る"
    );
    for (label, text) in [
        (
            "字 ほか の無い束の頭",
            "（q.2）を足す。裁定 id = q.1:S-1 まで・束 b:S-1",
        ),
        (
            "時刻の違う束の頭",
            "（q.2）を足す。裁定 id = q.1:S-2 ほか・束 b:S-2",
        ),
        (
            "問いの id の無い記録",
            "（q.3）を足す。裁定 id = q.1:S-1 ほか・束 b:S-1",
        ),
        (
            "丸括弧で囲まない問いの id",
            "q.2 を足す。裁定 id = q.1:S-1 ほか・束 b:S-1",
        ),
    ] {
        assert!(!ruling_in(text, "q.2:S-1"), "{label} を通す");
    }
}

/// 錠の 1 つの塊（見出し [[package]] から次の見出しまで）。
struct Package {
    name: String,
    dependencies: Vec<String>,
}

/// 錠の file を字で塊に割る。塊の頭の name = の行と、dependencies = [ … ] の行だけを見る。
fn lock_packages(text: &str) -> Vec<Package> {
    let mut packages: Vec<Package> = Vec::new();
    let mut in_deps = false;
    for line in text.lines() {
        if line == "[[package]]" {
            packages.push(Package {
                name: String::new(),
                dependencies: Vec::new(),
            });
            in_deps = false;
            continue;
        }
        let Some(package) = packages.last_mut() else {
            continue;
        };
        if in_deps {
            if line == "]" {
                in_deps = false;
            } else if let Some(entry) = line.trim().trim_end_matches(',').strip_prefix('"') {
                let entry = entry.trim_end_matches('"');
                package
                    .dependencies
                    .push(entry.split(' ').next().unwrap_or(entry).to_string());
            }
        } else if line == "dependencies = [" {
            in_deps = true;
        } else if package.name.is_empty() {
            package.name = line
                .strip_prefix("name = \"")
                .map(|name| name.trim_end_matches('"').to_string())
                .unwrap_or_default();
        }
    }
    packages
}

#[test]
fn cexcl_lock_drops_encoding() {
    let packages = lock_packages(&read(LOCK));
    assert!(packages.iter().all(|p| !p.name.is_empty()), "名の無い塊");
    assert!(
        packages.iter().all(|p| p.name != "encoding_rs"),
        "encoding_rs が錠に在る"
    );
    let folio: Vec<&Package> = packages.iter().filter(|p| p.name == "folio").collect();
    assert_eq!(folio.len(), 1, "folio の塊");
    let mut deps = folio[0].dependencies.clone();
    deps.sort();
    assert_eq!(deps, ["clap", "yaml-rust2"], "folio の dependencies");
    let rows = parse_table(&read(TABLE)).expect("表の読み");
    assert!(
        rows.iter()
            .all(|r| r.rule != "R-2" || !r.path.starts_with("folio2/")),
        "表に folio の R-2 の行が在る"
    );
}

/// 錠の file の塊（見出し [[package]]）のうち、行 source を持つ塊の数。
fn outside_packages(text: &str) -> usize {
    text.split("[[package]]")
        .skip(1)
        .filter(|block| block.lines().any(|l| l.starts_with("source = ")))
        .count()
}

#[test]
fn cexcl_vessel_lock_counts_row() {
    assert_eq!(
        outside_packages(&read(VESSEL_LOCK)),
        VESSEL_OUTSIDE,
        "器の錠の外の部品"
    );
    let rows = parse_table(&read(TABLE)).expect("表の読み");
    let r2: Vec<&Row> = rows
        .iter()
        .filter(|r| r.rule == "R-2" && r.path == VESSEL_CRATES)
        .collect();
    assert_eq!(r2.len(), 1, "器の R-2 の行");
    assert!(
        r2[0]
            .scope
            .contains(&format!("外の部品 {VESSEL_OUTSIDE} 本")),
        "器の R-2 の行の本数の字: {}",
        r2[0].scope
    );
    let two = "[[package]]\nname = \"a\"\n\n[[package]]\nname = \"b\"\nsource = \"registry\"\n";
    assert_eq!(outside_packages(two), 1, "行 source の在る塊だけ");
    assert_eq!(outside_packages("# 塊の無い錠\n"), 0);
}

#[test]
fn cexcl_folio_joins_root() {
    let members: Vec<String> = read("Cargo.toml")
        .lines()
        .skip_while(|l| l.trim() != "members = [")
        .skip(1)
        .take_while(|l| l.trim() != "]")
        .map(|l| l.trim().trim_end_matches(',').trim_matches('"').to_string())
        .collect();
    assert!(
        members.iter().any(|m| m == "folio2/crates/folio"),
        "根の members に folio が無い: {members:?}"
    );
    for gone in ["folio2/Cargo.toml", "folio2/Cargo.lock"] {
        assert!(!repo_root().join(gone).exists(), "{gone} が在る");
    }
    assert!(
        read("folio2/crates/folio/Cargo.toml")
            .lines()
            .all(|l| l.trim() != "[workspace]"),
        "folio の manifest が行 [workspace] を持つ"
    );
}

/// 起草の時の main 821ed79 の契約表の verify の最後の字（この行の語を除く）。
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

#[test]
fn cexcl_own_names_clean() {
    assert_eq!(WORDS.len(), 294);
    let text = read("xtask/tests/teeth1/carry.rs");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の次の行は fn");
            &rest[..rest.find('(').expect("fn の名の後に (")]
        })
        .collect();
    assert_eq!(names.len(), 7, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("cexcl_")
            .unwrap_or_else(|| panic!("{name} は cexcl_ で始まる"));
        for w in WORDS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
