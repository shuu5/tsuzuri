//! 入れ子の workspace の段（行 v-gate・判断の記録 ADR-18 の決定 (2)・ADR-22 の決定 (7)）。
//! 根の直下の dir のうち、Cargo.toml が行 `[workspace]` を持つ dir（入れ子の workspace）を字の順で数えて 1 行で出し（0 も出す）、
//! 各々の dir で build・歯（根と同じ partition）・clippy・その workspace の `cargo xtask check` を順に撃ち、最初に落ちた段の rc を返す。
//! 入れ子の組み立ては根の target の下の nested/<dir の名> に書く（根の xtask と入れ子の xtask の binary を同じ dir に置かない）。

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{emit_err, step_args};

/// 入れ子の dir ごとの段（順に撃つ）。頭の語が nextest の段にだけ、根と同じ partition の字を足す（step_args）。
pub const STEPS: &[&[&str]] = &[
    &["build", "--workspace"],
    &["nextest", "run", "--workspace"],
    &[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ],
    &["xtask", "check"],
];

/// 入れ子の組み立ての置き場（根の target の下の dir の名）。
pub const TARGET_SUB: &str = "nested";

/// 入れ子の段の 1 つ（入れ子の dir の名・撃つ dir・組み立ての置き場・cargo の引数）。
#[derive(Debug, PartialEq, Eq)]
pub struct Shot {
    pub name: String,
    pub dir: PathBuf,
    pub target: PathBuf,
    pub args: Vec<String>,
}

/// 根の直下の dir のうち、Cargo.toml の行（前後の空白を除いた字）が `[workspace]` に等しい行を持つ dir の名（字の順）。
/// 下の dir と、`[workspace.package]` などの節だけを持つ manifest は数えない。
pub fn find(root: &Path) -> Result<Vec<String>, String> {
    let entries =
        std::fs::read_dir(root).map_err(|e| format!("{} を読めない: {e}", root.display()))?;
    let mut dirs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("{} の項目を読めない: {e}", root.display()))?;
        let manifest = entry.path().join("Cargo.toml");
        if !entry.path().is_dir() || !manifest.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&manifest)
            .map_err(|e| format!("{} を読めない: {e}", manifest.display()))?;
        if text.lines().any(|l| l.trim() == "[workspace]") {
            dirs.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    dirs.sort();
    Ok(dirs)
}

/// 数えた 1 行の字（0 なら字 nested workspaces=0 だけ・在れば後に dirs= と名を , でつないだ字）。
pub fn summary(dirs: &[String]) -> String {
    if dirs.is_empty() {
        return "nested workspaces=0".to_string();
    }
    format!("nested workspaces={} dirs={}", dirs.len(), dirs.join(","))
}

/// 入れ子の組み立ての置き場の根: 環境変数 CARGO_TARGET_DIR の字（相対なら根から・空なら無いと読む）か根の target の下の nested。
pub fn target_base(root: &Path, env: Option<OsString>) -> PathBuf {
    let target = env
        .filter(|v| !v.is_empty())
        .map_or_else(|| PathBuf::from("target"), PathBuf::from);
    root.join(target).join(TARGET_SUB)
}

/// 段の列: 入れ子の dir の順に、dir ごとに STEPS の順で撃つ段（撃つ dir は根の下・置き場は base の下の dir の名）。
pub fn plan(root: &Path, base: &Path, dirs: &[String], part: Option<&str>) -> Vec<Shot> {
    dirs.iter()
        .flat_map(|name| {
            STEPS.iter().map(move |step| Shot {
                name: name.clone(),
                dir: root.join(name),
                target: base.join(name),
                args: step_args(step, part),
            })
        })
        .collect()
}

/// 入れ子の workspace の段: 数えた 1 行を出し、段の列を順に撃ち、最初に落ちた段の rc を返す（入れ子が無いか全部通れば 0）。
/// 入れ子の dir を読めなければ段を撃たずに rc 1 を返す。
pub fn run(root: &Path, cargo: &str, part: Option<&str>) -> i32 {
    let dirs = match find(root) {
        Ok(dirs) => dirs,
        Err(e) => {
            emit_err(&format!("xtask check: nested: {e}"));
            emit_err("xtask check: 落ちた段 nested (rc 1)");
            return 1;
        }
    };
    emit_err(&format!("xtask check: {}", summary(&dirs)));
    let base = target_base(root, std::env::var_os("CARGO_TARGET_DIR"));
    for shot in plan(root, &base, &dirs, part) {
        let line = format!("nested {}: cargo {}", shot.name, shot.args.join(" "));
        emit_err(&format!("xtask check: {line}"));
        let status = Command::new(cargo)
            .args(&shot.args)
            .current_dir(&shot.dir)
            .env("CARGO_TARGET_DIR", &shot.target)
            .status();
        let rc = match status {
            Ok(s) if s.success() => continue,
            Ok(s) => s.code().unwrap_or(1),
            Err(e) => {
                emit_err(&format!("xtask check: cargo を起動できない: {e}"));
                1
            }
        };
        emit_err(&format!("xtask check: 落ちた段 {line} (rc {rc})"));
        return rc;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// 起草の時の main afa65b32 の契約表の verify の最後の字と、ノート surface-wave27a の 5 行と、
    /// docs/design/vessel-keys.md の行の字（409 語）。歯の名から接頭辞を除いた字はどの語も含まない。
    const FILTER_WORDS: &str = "
aaround_ aaround_src_ask_fold_embeds abss_ abst_ accept_ acchold_ account_ acctcore_
acctcore_pure_and_no_new_dependencies acctdoc_ acctframe_ accthb_ accthome_ acctled_ acctlook_
acctpcore_ acctproj_ acctsess_ acctwin_ acctwire_ acctwire_back_part_before_header
acctwire_back_steps aface_ afocus_ afocus_src_links_and_scroll alean_ anchor_ aord_ aown_ apark_
apop_ areread_ askcard_ askcard_page_frame_and_nav athr_ batchpanel_
batchpanel_page_frame_two_columns bhalf_ board_min_ bport_ brand_
brand_path_written_once_in_contract btuck_ bvacc_ bvafix_ bvaskw_ bvbar_ bvcase_ bvcform_ bvcols_
bvcroute_ bvcull_ bvdsweep_ bvfact_ bvfive_ bvfront_ bvfroute_ bvhelp_ bvkpi_ bvland_ bvlay_ bvlc_
bvlcw_ bvlex_ bvlist_ bvmore_ bvmpage_ bvnav_ bvopen_ bvpop_ bvqcol_ bvqhist_ bvqsrv_ bvrread_
bvsel_ bvsettle_ bvshed_ bvstep_ bvsweep_ bvtrim_ bvunref_ bvuphase_ bvuroute_ bvwhy_ bvwin_
bvwins_ bvwurl_ cadl_ cadopt_ cadq_ cdorm_ cexcl_ cfsplit_ cg9_ cgdom_
check_canonical_design_intent_passes cishard_ cmark_ cnote_ cnret_ contract_form_ cround_ csled_
cspk_ ctick_ cupd_ cupdlist_ cwarg_ cwfnd_ cwlin_ cwquo_ cwty_ denv_ dnedge_ dngrp_ dnrow_ dnskip_
dretry_ dstg_ ecache_ ecache_handle_wiring_text eheld_acct_ eheld_design_ eheld_held_ eheld_marks_
eheld_vessel_ elazy_ epolq_ eretry_ esig_ evkind_ f123_ f125_ f142_ f152_ f159_ f174_ f181_ f185_
f190_ f192_ f212_ f2ret_ f76_ f89_ f98_ f99_ face_ fdlt_ fdlv_ fdrop_ flight_ fmark_ fprem_ frame_
frame_home_blocks_in_order_and_map_page frame_one_module_per_block fsch_ fserve_ fstop_
fstop_usage_errors_are_one fundl_ fxpre_ g3g7_ gacct_ gapspage_ gapspage_page_frame_and_nav gate_
gatt_ gbnote_ gchip_ gcoach_ gext_ gfix_ gfresh_ gfresh_boards_text ghb_ gins_ gjst_ glabel_ gmret_
gmretw_ gnav_ gpface_ gpill_ gpulse_ gpulse_boards_text gquestion_ graph_ gsum_ gtuck_ gview_ gwv_
hacols_ harest_ hasplit_ hbconf_ hbconf_config_moves_to_config_rs
hbconf_main_parse_fills_the_rest_with_new hbmark_ hbon_ hbpost_ hbpost_mod_rs_drops_post_fns
hbproc_ hbroute_ hbroute_mod_rs_keeps_only_events hcard_ hcled_ hcnx_ hcproj_ hcsess_ hdchip_ hfig_
hnunk_ hook_ hruling_ hsblock_ hsblock_dispatch_by_list hsblock_ids_match_consts
hsblock_kit_holds_shared hsderive_ hsderive_consts_match_note hshist_ hspage_
hspage_snaps_match_files hspk_ hsym_ hthr_ http_ hwstore_ iclose_ iclose_handle_wiring_text ilink_
inject_ jcount_ jrun_ kcli_ kdeny_ kg9_ kindlab_ klink_ klint_ klintf_ ksize_ ksum_ lateface_
launch_ lcard_ ledgerblock_ lgrp_ lhome_ lidle_ lineage_ lkind_ lresume_ lsnap_ lspark_ lstg_
lstore_ mapgraph_ mapview_ mapview_classes_and_keys_exist mbig_ mdoch_ mkeys_ mlink_ mqask_ mqface_
mstore_ mstore_module_text mtips_ mtree_ nact_ nbatch_ ncard_ nextstep_ nodepage_
nodepage_frame_and_nav nstall_ nsum_ nsum_core_reads_no_files nsumw_ ntc_ ntime_ nxact_ nxorg_
p106_ p1_commands_closed_list parts_ parts_paths_distinct_in_block_modules pci_ pclosed_ pfold_
pgsw_ pgz_ pipe_ pkac_ pkview_ plimit_ pmisfit_ pmore_ popapp_ pquest_ pqueue_ project_ prose_
ptitle_ pubfp_ pubscan_ punmap_ pwhole_ qblock_ qgate_ qkey_ qsig_ question_ rbusy_ relay_
relay_eyes_argv_snapshot retired_render_ rhold_ rhold_wiring_text rsid_ runsdoc_ rvk_ saxis_
saxis_src_draws_legend_axis sclosed_ seatblock_ seatcard_ server_ server_acct_ server_ask_
server_ask_http_ server_batch_ server_batch_policy_scope_and_refusals server_coalesce_
server::design::tests:: server::events:: server::events::tests:: server_hb_ server_min_
server_min_bind_judge server_min_epoch_secs_reads_rfc3339 server_min_http_
server_min_hub_sends_to_each_subscriber server_min_parse_refuses_broken_lines
server_min_percent_decode server_read_ server_read_board_changed_within_5s
server_read_doc_copies_verdicts server_read_empty_texts_are_unread
server_read_watch_board_sends_on_marks server_reap_ server_seat_ server_src_
server_src_marks_are_two_files server_src_unstartable_bd_is_unknown server_src_watch_
server_src_watch_marks_trigger_reread server_view_ server_view_http_ sesplit_ sgrace_ sha256_ shb_
skeleton_ smore_ smore_dom_text stage_ stage_cdp_ stage_term_ stats_
stats_detail_account_takes_the_last stats_epoch_secs_reads_rfc
stats_read_drops_tombstones_and_reads_parent_and_blocks stats_stage_of_closed_table stbp_ stcli_
steady_ steady_details_use_fold_record steady_net_uses_settle_but_lose_all_does_not sthr_ stnfy_
stskill_ sttgt_ sxaxis_ tgall_ tgown_ ticker_ ticker_net_holds_one_interval tipx_ tipx_dom_text
tkad_ tlic_ topbar_ topfit_ tz_ tzent_ tzpar_ tzself_ udacct_ udash_ unow_ urpanel_ uword_ vocab_
wsteady_ wstrip_
";

    /// 一時の dir（名に pid と字 tag）を作り直す。
    fn fresh(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tsuzuri-vnest-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("一時の dir");
        dir
    }

    fn put(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().expect("親の dir")).expect(rel);
        std::fs::write(&path, body).expect(rel);
    }

    fn strings(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn vnest_finds_marked_dirs_in_order() {
        let root = fresh("find");
        put(&root, "Cargo.toml", "[workspace]\nmembers = [\"b\"]\n");
        put(&root, "b/Cargo.toml", "[workspace]\nmembers = [\"x\"]\n");
        put(
            &root,
            "a/Cargo.toml",
            "# 注\n  [workspace]\nresolver = \"2\"\n",
        );
        put(&root, "Z/Cargo.toml", "[workspace]\n");
        put(&root, "c/Cargo.toml", "[package]\nname = \"c\"\n");
        put(&root, "d/sub/Cargo.toml", "[workspace]\n");
        put(
            &root,
            "e/Cargo.toml",
            "[workspace.package]\nversion = \"0.1.0\"\n",
        );
        put(
            &root,
            "f/Cargo.toml",
            "# [workspace]\n[package]\nname = \"f\"\n",
        );
        put(&root, "g/Cargo.toml/inner", "[workspace]\n");
        put(&root, "h", "[workspace]\n");
        assert_eq!(find(&root), Ok(strings(&["Z", "a", "b"])));
        let empty = fresh("none");
        assert_eq!(find(&empty), Ok(Vec::new()));
        let missing = find(&root.join("no-such-dir")).expect_err("無い dir");
        assert!(missing.contains("no-such-dir"), "{missing}");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&empty);
    }

    #[test]
    fn vnest_shots_with_and_without_partition() {
        assert_eq!(
            STEPS,
            [
                &["build", "--workspace"][..],
                &["nextest", "run", "--workspace"],
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings"
                ],
                &["xtask", "check"],
            ]
        );
        let root = Path::new("/r");
        let base = Path::new("/t/nested");
        let dirs = strings(&["a", "b"]);
        for part in [None, Some("count:3/8")] {
            let shots = plan(root, base, &dirs, part);
            assert_eq!(shots.len(), 8, "{part:?}");
            let mut want = Vec::new();
            for name in ["a", "b"] {
                for step in STEPS {
                    let mut args = strings(step);
                    if step.first() == Some(&"nextest")
                        && let Some(p) = part
                    {
                        args.extend(strings(&["--partition", p]));
                    }
                    want.push(Shot {
                        name: name.to_string(),
                        dir: root.join(name),
                        target: base.join(name),
                        args,
                    });
                }
            }
            assert_eq!(shots, want, "{part:?}");
        }
        let nextest = plan(root, base, &strings(&["s"]), Some("count:3/8"));
        assert_eq!(
            nextest.get(1).map(|s| s.args.join(" ")).as_deref(),
            Some("nextest run --workspace --partition count:3/8")
        );
        assert_eq!(
            nextest.get(3).map(|s| s.args.join(" ")).as_deref(),
            Some("xtask check")
        );
        assert!(plan(root, base, &[], Some("count:1/8")).is_empty());
    }

    #[test]
    fn vnest_summary_line_counts_zero() {
        assert_eq!(summary(&[]), "nested workspaces=0");
        assert_eq!(
            summary(&strings(&["scribe2"])),
            "nested workspaces=1 dirs=scribe2"
        );
        assert_eq!(
            summary(&strings(&["a", "b"])),
            "nested workspaces=2 dirs=a,b"
        );
    }

    #[test]
    fn vnest_target_dir_under_root_target() {
        let root = Path::new("/r");
        assert_eq!(TARGET_SUB, "nested");
        assert_eq!(target_base(root, None), PathBuf::from("/r/target/nested"));
        assert_eq!(
            target_base(root, Some(OsString::new())),
            PathBuf::from("/r/target/nested")
        );
        assert_eq!(
            target_base(root, Some(OsString::from("/abs/t"))),
            PathBuf::from("/abs/t/nested")
        );
        assert_eq!(
            target_base(root, Some(OsString::from("rel/t"))),
            PathBuf::from("/r/rel/t/nested")
        );
    }

    #[test]
    fn vnest_run_zero_and_missing_cargo() {
        let root = fresh("run");
        put(&root, "c/Cargo.toml", "[package]\nname = \"c\"\n");
        let cargo = root.join("no-such-cargo");
        let cargo = cargo.to_string_lossy();
        assert_eq!(run(&root, &cargo, None), 0, "入れ子が無ければ撃たずに 0");
        put(&root, "s/Cargo.toml", "[workspace]\n");
        assert_eq!(run(&root, &cargo, None), 1, "cargo を起動できなければ 1");
        assert_eq!(
            run(&root.join("no-such-dir"), &cargo, None),
            1,
            "読めない根は 1"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn vnest_check_runs_last() {
        const MAIN: &str = include_str!("main.rs");
        assert!(MAIN.lines().any(|l| l == "mod nested;"), "mod nested;");
        let (_, check) = MAIN.split_once("fn check(").expect("fn check");
        let (check, _) = check.split_once("\n}\n").expect("fn check の終わり");
        let steps = check.find("for step in CHECK_STEPS").expect("cargo の段");
        let surface = check.find("surface_build(root)").expect("面の組み立て");
        let nested = check
            .find("nested::run(root, &cargo, part)")
            .expect("入れ子の段");
        assert!(
            steps < surface && surface < nested,
            "cargo の段 → 面 → 入れ子の順"
        );
        assert!(
            check[surface..nested].contains("return rc;"),
            "面の組み立てが落ちたら入れ子を撃たない"
        );
        assert!(
            check
                .trim_end()
                .ends_with("nested::run(root, &cargo, part)"),
            "入れ子の段の rc が check の rc"
        );
    }

    #[test]
    fn vnest_own_names_clean() {
        let lines: Vec<&str> = include_str!("nested.rs").lines().collect();
        let names: Vec<&str> = lines
            .windows(2)
            .filter(|w| w[0].trim() == "#[test]")
            .map(|w| {
                let rest = w[1]
                    .trim()
                    .strip_prefix("fn ")
                    .expect("test の属性の次は fn");
                rest.split('(').next().unwrap_or(rest)
            })
            .collect();
        assert_eq!(names.len(), 7, "{names:?}");
        let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
        let unique: BTreeSet<&str> = words.iter().copied().collect();
        assert_eq!(words.len(), 409, "filter の語の数");
        assert_eq!(unique.len(), 409, "filter の語が重なる");
        for name in &names {
            let rest = name
                .strip_prefix("vnest_")
                .unwrap_or_else(|| panic!("{name} が vnest_ で始まらない"));
            for word in &words {
                assert!(!rest.contains(word), "{name} が {word} を含む");
            }
        }
    }
}
