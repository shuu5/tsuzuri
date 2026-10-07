//! 大きさの数え（行 k-size-base・規則の行 R-4・判断の記録 ADR-25）。
//! 根の Cargo.toml の members のうち、除外の表 carry-exclusions.toml の R-4 の行が名指さない member の src の下の .rs を数える
//! （src の外の tests/ と build.rs は数えない）。行の重みは字数を幅で割って切り上げた数（1 以上）。
//! file の本体は行頭が `#[cfg(test)]` の最初の行より前、検査の区間はその行から末尾で、名が tests.rs か _tests.rs で終わる file は丸ごと検査の区間。
//! 1 module は検査の区間も含めた file 全体、歯:source は検査の区間の和と本体の和の 1 つの比を上限と比べる。
//! 中核の crate の本体の和は上限と比べず数だけ出し、検査の印の後の最初の行が mod でない file は名指して落とす。

use std::path::Path;

/// 1 module（file 全体）の行数の上限（規則の行 R-4 の値の 1 module 1,500 行）。
pub const MODULE_MAX: usize = 1_500;

/// 歯:source の上限の百分率（規則の行 R-4 の値の 1.0）。
pub const RATIO_PCT: usize = 100;

/// 行の幅（字数）。これを超える行は字数を幅で割って切り上げた行数に数える。
pub const WIDTH: usize = 120;

/// 上限の 3 つ。
pub struct Limits {
    pub module: usize,
    pub ratio_pct: usize,
    pub width: usize,
}

/// 規則の行 R-4 の上限（歯 ksize_limits_match_rule_r4 が行の value と照らす）。
pub const LIMITS: Limits = Limits {
    module: MODULE_MAX,
    ratio_pct: RATIO_PCT,
    width: WIDTH,
};

/// 中核の crate の dir（workspace の root から）。
const CORE_DIR: &str = "crates/tsuzuri-core";

/// 除外の表の path（workspace の root から）。
const EXCLUSIONS: &str = "carry-exclusions.toml";

/// 検査の区間の始まりの印（行頭に在るときだけ読む）。
const TEST_MARK: &str = "#[cfg(test)]";

/// 検査の印の後の最初の行が持つべき mod の頭 3 つ（後ろに空白が続く形で読む）。
const MOD_HEADS: [&str; 3] = ["mod", "pub mod", "pub(crate) mod"];

/// 1 行の重み（字数を幅で割って切り上げた数・空の行も 1）。
fn weight(line: &str, width: usize) -> usize {
    line.chars().count().div_ceil(width.max(1)).max(1)
}

/// file の (検査の区間の行数, 本体の行数)。名が tests.rs か _tests.rs で終わる file は丸ごと検査の区間。
fn split(name: &str, text: &str, width: usize) -> (usize, usize) {
    let file = name.rsplit('/').next().unwrap_or(name);
    let mut in_tests = file == "tests.rs" || file.ends_with("_tests.rs");
    let (mut tests, mut body) = (0, 0);
    for line in text.lines() {
        in_tests = in_tests || line.starts_with(TEST_MARK);
        if in_tests {
            tests += weight(line, width);
        } else {
            body += weight(line, width);
        }
    }
    (tests, body)
}

/// file の最初の行頭の印の後で、空でなく属性（#）でも doc の注（/// と //!）でもない最初の字が mod の頭でなければ印の行の番号（1 から）を返す。
/// 印の無い file と、名が tests.rs か _tests.rs で終わる file は None。印の行の同じ行の残りも候補にする。
fn mark_hole(name: &str, text: &str) -> Option<usize> {
    let file = name.rsplit('/').next().unwrap_or(name);
    if file == "tests.rs" || file.ends_with("_tests.rs") {
        return None;
    }
    let mut lines = text.lines().enumerate();
    let (at, mark) = lines.find(|(_, l)| l.starts_with(TEST_MARK))?;
    let rest = mark.strip_prefix(TEST_MARK).unwrap_or("");
    let first = std::iter::once(rest)
        .chain(lines.map(|(_, l)| l))
        .map(str::trim_start)
        .find(|l| !l.is_empty() && !["#", "///", "//!"].iter().any(|p| l.starts_with(p)));
    match first {
        Some(l) if MOD_HEADS.iter().any(|h| l.starts_with(&format!("{h} "))) => None,
        _ => Some(at + 1),
    }
}

/// files（path と字）を上限と比べ、事実の字（鍵 file-lines・core-lines・test-src-ratio の順）と違反の一覧を返す。
/// core_dir の src の下の file が中核で、その本体の和は上限と比べず数だけ出す。歯:source は歯 × 100 が本体 × 比の百分率 以下かで比べる。
/// 検査の印の後の最初の行が mod でない file は違反 test-mark で名指す。
fn judge(files: &[(String, String)], core_dir: &str, limits: &Limits) -> (String, Vec<String>) {
    let core_src = format!("{core_dir}/src/");
    let mut bad = Vec::new();
    let (mut widest, mut core, mut tests, mut body) = (0, 0, 0, 0);
    for (path, text) in files {
        let (t, b) = split(path, text, limits.width);
        widest = widest.max(t + b);
        if t + b > limits.module {
            bad.push(format!(
                "file-lines: {path} は {} 行で 1 module の上限 {} 行を越える",
                t + b,
                limits.module
            ));
        }
        if let Some(line) = mark_hole(path, text) {
            bad.push(format!(
                "test-mark: {path} の {line} 行目の検査の印の後の最初の行が mod でない"
            ));
        }
        if path.starts_with(&core_src) {
            core += b;
        }
        tests += t;
        body += b;
    }
    if tests * 100 > body * limits.ratio_pct {
        bad.push(format!(
            "test-src-ratio: 歯 {tests} 行が本体 {body} 行の {}% を越える",
            limits.ratio_pct
        ));
    }
    let facts = format!(
        "file-lines={widest}/{} core-lines={core} test-src-ratio={tests}/{body}",
        limits.module
    );
    (facts, bad)
}

/// 根の Cargo.toml の members の dir（`members = [` から `]` までの行・順のまま）。
fn members(manifest: &str) -> Vec<String> {
    manifest
        .lines()
        .map(str::trim)
        .skip_while(|l| *l != "members = [")
        .skip(1)
        .take_while(|l| *l != "]")
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.trim_end_matches(',').trim_matches('"').to_string())
        .collect()
}

/// 除外の表の規則 R-4 の行の path（末尾の字 / を除く）。
fn r4_paths(table: &str) -> Vec<String> {
    let mut rows: Vec<(String, String)> = Vec::new();
    for line in table.lines().map(str::trim) {
        if line == "[[exclusion]]" {
            rows.push((String::new(), String::new()));
        } else if let Some((k, v)) = line.split_once('=')
            && let Some(row) = rows.last_mut()
        {
            let v = v.trim().trim_matches('"').to_string();
            match k.trim() {
                "rule" => row.0 = v,
                "path" => row.1 = v,
                _ => {}
            }
        }
    }
    rows.into_iter()
        .filter(|(rule, _)| rule == "R-4")
        .map(|(_, path)| path.trim_end_matches('/').to_string())
        .collect()
}

/// 数える member（members のうち、除外の path そのものか、その下に在るものを除く・順のまま）。
fn counted(members: &[String], excluded: &[String]) -> Vec<String> {
    members
        .iter()
        .filter(|m| {
            !excluded.iter().any(|p| {
                *m == p
                    || m.strip_prefix(p.as_str())
                        .is_some_and(|r| r.starts_with('/'))
            })
        })
        .cloned()
        .collect()
}

/// dir（root からの相対）の下の .rs を名の順に辿り、(root からの相対の path, 字) を足す。dir が無ければ何も足さない。
fn collect(root: &Path, dir: &str, out: &mut Vec<(String, String)>) -> Result<(), String> {
    let entries = match std::fs::read_dir(root.join(dir)) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("{dir} を読めない: {e}")),
    };
    let mut names: Vec<(String, bool)> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("{dir} を読めない: {e}"))?;
        let is_dir = entry
            .file_type()
            .map_err(|e| format!("{dir} を読めない: {e}"))?
            .is_dir();
        names.push((entry.file_name().to_string_lossy().into_owned(), is_dir));
    }
    names.sort();
    for (name, is_dir) in names {
        let path = format!("{dir}/{name}");
        if is_dir {
            collect(root, &path, out)?;
        } else if name.ends_with(".rs") {
            let text = std::fs::read_to_string(root.join(&path))
                .map_err(|e| format!("{path} を読めない: {e}"))?;
            out.push((path, text));
        }
    }
    Ok(())
}

/// root の木を規則の行 R-4 の上限と比べる。収まれば事実の 1 行、越えれば違反の全部（と事実）を Err で返す。
pub fn measure(root: &Path) -> Result<String, String> {
    let read = |rel: &str| {
        std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel} を読めない: {e}"))
    };
    let all = members(&read("Cargo.toml")?);
    if all.is_empty() {
        return Err("Cargo.toml の members を読めない".to_string());
    }
    let mut files = Vec::new();
    for member in counted(&all, &r4_paths(&read(EXCLUSIONS)?)) {
        collect(root, &format!("{member}/src"), &mut files)?;
    }
    let (facts, bad) = judge(&files, CORE_DIR, &LIMITS);
    if bad.is_empty() {
        Ok(facts)
    } else {
        Err(format!("{}\n{facts}", bad.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// 起草の時の main 44f7f5e0 の契約表の verify の最後の字（339 語）。歯の名から接頭辞を除いた字がどの語も含まないことを見る。
    const FILTER_WORDS: &str = "
aaround_ aaround_src_ask_fold_embeds abss_ abst_ accept_ acchold_ account_ acctcore_
acctcore_pure_and_no_new_dependencies acctdoc_ acctframe_ accthb_ accthome_ acctled_ acctlook_
acctpcore_ acctproj_ acctsess_ acctwin_ acctwire_ acctwire_back_part_before_header
acctwire_back_steps aface_ afocus_ afocus_src_links_and_scroll alean_ aord_ aown_ apark_ apop_
areread_ askcard_ askcard_page_frame_and_nav athr_ batchpanel_ batchpanel_page_frame_two_columns
bhalf_ board_min_ bport_ brand_ brand_path_written_once_in_contract btuck_ cadl_ cadopt_ cadq_
cdorm_ cexcl_ cfsplit_ cg9_ cgdom_ check_canonical_design_intent_passes cishard_ cmark_ cnote_
cnret_ contract_form_ cround_ csled_ cspk_ ctick_ cupd_ cupdlist_ denv_ dnedge_ dngrp_ dnrow_
dnskip_ dretry_ dstg_ ecache_ ecache_handle_wiring_text eheld_acct_ eheld_design_ eheld_held_
eheld_marks_ eheld_vessel_ elazy_ epolq_ eretry_ esig_ evkind_ f123_ f152_ f159_ f192_ f212_
f2ret_ f89_ fdlt_ fdlv_ fdrop_ flight_ fmark_ fprem_ frame_ frame_home_blocks_in_order_and_map_page
frame_one_module_per_block fserve_ fstop_ fstop_usage_errors_are_one fundl_ fxpre_ g3g7_ gacct_
gapspage_ gapspage_page_frame_and_nav gatt_ gbnote_ gchip_ gcoach_ gext_ gfix_ gfresh_
gfresh_boards_text ghb_ gins_ gjst_ glabel_ gmret_ gmretw_ gnav_ gpface_ gpill_ gpulse_
gpulse_boards_text gquestion_ graph_ gsum_ gtuck_ gview_ gwv_ hacols_ harest_ hasplit_ hbconf_
hbconf_config_moves_to_config_rs hbconf_main_parse_fills_the_rest_with_new hbmark_ hbon_ hbpost_
hbpost_mod_rs_drops_post_fns hbproc_ hbroute_ hbroute_mod_rs_keeps_only_events hcard_ hcled_
hcnx_ hcproj_ hcsess_ hdchip_ hfig_ hnunk_ hook_ hruling_ hsblock_ hsblock_dispatch_by_list
hsblock_ids_match_consts hsblock_kit_holds_shared hsderive_ hsderive_consts_match_note hshist_
hspage_ hspage_snaps_match_files hspk_ hsym_ hthr_ http_ hwstore_ iclose_ iclose_handle_wiring_text
ilink_ inject_ jcount_ jrun_ kcli_ kdeny_ kg9_ kindlab_ klink_ klint_ klintf_ ksum_ lateface_
launch_ lcard_ ledgerblock_ lgrp_ lhome_ lidle_ lkind_ lresume_ lsnap_ lspark_ lstg_ lstore_
mapgraph_ mapview_ mapview_classes_and_keys_exist mbig_ mkeys_ mlink_ mqask_ mqface_ mstore_
mstore_module_text mtips_ mtree_ nact_ nbatch_ ncard_ nextstep_ nodepage_ nodepage_frame_and_nav
nstall_ nsum_ nsum_core_reads_no_files nsumw_ ntc_ ntime_ nxact_ nxorg_ p106_
p1_commands_closed_list parts_ parts_paths_distinct_in_block_modules pci_ pclosed_ pfold_ pgsw_
pgz_ pipe_ pkac_ pkview_ plimit_ pmisfit_ pmore_ popapp_ pquest_ pqueue_ project_ ptitle_ pubfp_
pubscan_ punmap_ pwhole_ qblock_ qgate_ qkey_ qsig_ question_ rbusy_ relay_ rhold_
rhold_wiring_text rsid_ runsdoc_ rvk_ saxis_ saxis_src_draws_legend_axis sclosed_ seatblock_
seatcard_ server::design::tests:: server::events:: server::events::tests:: server_ server_acct_
server_ask_ server_ask_http_ server_batch_ server_batch_policy_scope_and_refusals server_coalesce_
server_hb_ server_min_ server_min_bind_judge server_min_epoch_secs_reads_rfc3339 server_min_http_
server_min_hub_sends_to_each_subscriber server_min_parse_refuses_broken_lines
server_min_percent_decode server_read_ server_read_board_changed_within_5s
server_read_doc_copies_verdicts server_read_empty_texts_are_unread
server_read_watch_board_sends_on_marks server_reap_ server_seat_ server_src_
server_src_marks_are_two_files server_src_unstartable_bd_is_unknown server_src_watch_
server_src_watch_marks_trigger_reread server_view_ server_view_http_ sesplit_ sgrace_ shb_
skeleton_ smore_ smore_dom_text stage_ stage_cdp_ stage_term_ stats_
stats_detail_account_takes_the_last stats_epoch_secs_reads_rfc
stats_read_drops_tombstones_and_reads_parent_and_blocks stats_stage_of_closed_table stbp_ stcli_
steady_ steady_details_use_fold_record steady_net_uses_settle_but_lose_all_does_not sthr_ stnfy_
stskill_ sttgt_ sxaxis_ tgall_ tgown_ ticker_ ticker_net_holds_one_interval tipx_ tipx_dom_text
tkad_ tlic_ topbar_ topfit_ tz_ tzent_ tzpar_ tzself_ udacct_ udash_ unow_ urpanel_ uword_
wsteady_ wstrip_
";

    fn strings(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    /// 幅 10 で 5 行（字 fn a() {}・英小字 25 字・印・mod t {}・英小字 25 字）の file。
    fn five(mark: &str) -> String {
        let z = "z".repeat(25);
        ["fn a() {}", &z, mark, "mod t {}", &z].join("\n")
    }

    #[test]
    fn ksize_weight_and_split() {
        let z = "z".repeat(25);
        assert_eq!(split("a.rs", &five("#[cfg(test)]"), 10), (6, 4));
        assert_eq!(split("a.rs", &z, 10), (0, 3));
        assert_eq!(split("x_tests.rs", &five("#[cfg(test)]"), 10), (10, 0));
        assert_eq!(split("tests.rs", &z, 10), (3, 0));
        assert_eq!(split("a.rs", &five("    #[cfg(test)]"), 10), (0, 10));
        let wide = format!("{}\n{}\n\n", "a".repeat(120), "あ".repeat(121));
        assert_eq!(split("a.rs", &wide, 120), (0, 4));
    }

    #[test]
    fn ksize_judge_limits() {
        let files = vec![
            ("c/src/heavy.rs".to_string(), five("#[cfg(test)]")),
            ("c/src/bare.rs".to_string(), "z".repeat(25)),
            ("o/src/other.rs".to_string(), five("#[cfg(test)]")),
        ];
        let limits = |module, ratio_pct| Limits {
            module,
            ratio_pct,
            width: 10,
        };
        let (facts, bad) = judge(&files, "c", &limits(10, 100));
        assert_eq!(facts, "file-lines=10/10 core-lines=7 test-src-ratio=12/11");
        assert_eq!(bad.len(), 1, "{bad:?}");
        assert!(bad[0].starts_with("test-src-ratio:"), "{bad:?}");
        let (_, bad) = judge(&files, "c", &limits(9, 110));
        let heads: Vec<&str> = bad
            .iter()
            .map(|b| b.split(':').next().unwrap_or(""))
            .collect();
        assert_eq!(heads, ["file-lines", "file-lines"]);
        let (_, bad) = judge(&files, "c", &limits(10, 110));
        assert!(bad.is_empty(), "{bad:?}");
    }

    #[test]
    fn ksize_mark_hole() {
        let none: [&[&str]; 8] = [
            &["fn a() {}", "#[cfg(test)]", "mod t {}"],
            &[
                "fn a() {}",
                "#[cfg(test)]",
                "",
                "#[allow(dead_code)]",
                "/// 歯",
                "//! 歯",
                "mod t {}",
            ],
            &["#[cfg(test)]", "pub mod t {}"],
            &["#[cfg(test)]", "pub(crate) mod t {}"],
            &["fn a() {}", "#[cfg(test)] mod t {}"],
            &["fn a() {}", "    #[cfg(test)]", "    use x::Y;"],
            &[
                "fn a() {}",
                "#[cfg(test)]",
                "mod t {}",
                "#[cfg(test)]",
                "use x::Y;",
            ],
            &["fn a() {}", "fn module() {}"],
        ];
        for lines in none {
            let got = mark_hole("c/src/a.rs", &lines.join("\n"));
            assert_eq!(got, None, "{lines:?}");
        }
        let some: [(&[&str], usize); 5] = [
            (&["fn a() {}", "#[cfg(test)]", "use x::Y;", "mod t {}"], 2),
            (&["fn a() {}", "#[cfg(test)]", "// 注", "mod t {}"], 2),
            (&["fn a() {}", "#[cfg(test)]"], 2),
            (&["fn a() {}", "#[cfg(test)] use x::Y;", "mod t {}"], 2),
            (&["#[cfg(test)]", "module::f();"], 1),
        ];
        for (lines, line) in some {
            let got = mark_hole("c/src/a.rs", &lines.join("\n"));
            assert_eq!(got, Some(line), "{lines:?}");
        }
        let e = some[0].0.join("\n");
        assert_eq!(mark_hole("c/src/x_tests.rs", &e), None);
        assert_eq!(mark_hole("c/src/tests.rs", &e), None);
        let files = vec![
            ("c/src/h.rs".to_string(), e),
            (
                "c/src/ok.rs".to_string(),
                ["fn a() {}", "#[cfg(test)]", "mod t {}"].join("\n"),
            ),
        ];
        let limits = Limits {
            module: 100,
            ratio_pct: 1000,
            width: 10,
        };
        let (_, bad) = judge(&files, "c", &limits);
        assert_eq!(
            bad,
            ["test-mark: c/src/h.rs の 2 行目の検査の印の後の最初の行が mod でない"]
        );
    }

    fn rules_row() -> String {
        let text =
            std::fs::read_to_string(crate::workspace_root().join("design-intent/rules.yaml"))
                .expect("design-intent/rules.yaml");
        let mut rows = text.lines().filter(|l| l.contains("{id: R-4,"));
        let row = rows.next().expect("行 R-4 が在る");
        assert!(rows.next().is_none(), "行 R-4 が 2 つ在る");
        row.to_string()
    }

    fn field<'a>(row: &'a str, key: &str) -> &'a str {
        let mark = format!(" {key}: \"");
        let rest = row
            .split_once(&mark)
            .unwrap_or_else(|| panic!("{key} が無い"))
            .1;
        rest.split_once('"').expect("閉じの引用符").0
    }

    fn commas(n: usize) -> String {
        let s = n.to_string();
        let mut out = String::new();
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i).is_multiple_of(3) {
                out.push(',');
            }
            out.push(c);
        }
        out
    }

    #[test]
    fn ksize_limits_match_rule_r4() {
        assert_eq!((MODULE_MAX, RATIO_PCT, WIDTH), (1_500, 100, 120));
        assert_eq!(
            (LIMITS.module, LIMITS.ratio_pct, LIMITS.width),
            (MODULE_MAX, RATIO_PCT, WIDTH)
        );
        let row = rules_row();
        let value = field(&row, "value");
        for want in [
            format!("1 module {} 行 以下", commas(MODULE_MAX)),
            format!(
                "歯:source {}.{} 以下",
                RATIO_PCT / 100,
                RATIO_PCT % 100 / 10
            ),
        ] {
            assert!(value.contains(&want), "value に字 {want} が無い");
        }
        let population = field(&row, "population");
        for want in [
            format!("幅 {WIDTH} 字"),
            "workspace の部品の本体の置き場 src の下の .rs だけを数え、src の外の tests/ と build.rs は数えない".to_string(),
            "名が tests.rs か _tests.rs で終わる file は丸ごと検査の区間".to_string(),
            "1 module の行数は検査の区間も含めた file の全体で数え".to_string(),
            "数える部品の全部を合わせた 1 つの比で比べる".to_string(),
            "除外の表 carry-exclusions.toml が名指す持ち込みの code は、その表の行が消えるまで数えない".to_string(),
        ] {
            assert!(population.contains(&want), "population に字 {want} が無い");
        }
    }

    #[test]
    fn ksize_population_follows_carry() {
        let root = crate::workspace_root();
        let read = |rel: &str| std::fs::read_to_string(root.join(rel)).expect(rel);
        let all = members(&read("Cargo.toml"));
        assert_eq!(all.len(), 6, "{all:?}");
        assert_eq!(
            counted(&all, &r4_paths(&read(EXCLUSIONS))),
            [
                "crates/tsuzuri-contract",
                "crates/tsuzuri-core",
                "crates/tsuzuri-boundary",
                "crates/tsuzuri-surface",
                "folio2/crates/folio",
                "xtask"
            ]
        );
        let table = "[[exclusion]]\nrule = \"R-10\"\npath = \"a/\"\n\n[[exclusion]]\nrule = \"R-4\"\npath = \"b/\"\n";
        assert_eq!(r4_paths(table), ["b"]);
        assert!(r4_paths("# 行の無い表\n").is_empty());
        let three = strings(&["a", "b", "b2", "c"]);
        assert_eq!(counted(&three, &strings(&["b"])), ["a", "b2", "c"]);
        assert_eq!(counted(&three, &strings(&["a", "c"])), ["b", "b2"]);
        assert_eq!(counted(&three, &[]), three);
    }

    #[test]
    fn ksize_tree_within_limits() {
        let facts = measure(&crate::workspace_root()).expect("木は R-4 の上限に収まる");
        let keys: Vec<&str> = facts
            .split_whitespace()
            .map(|t| t.split('=').next().unwrap_or(""))
            .collect();
        assert_eq!(
            keys,
            ["file-lines", "core-lines", "test-src-ratio"],
            "{facts}"
        );
        let core = facts
            .split_whitespace()
            .find_map(|t| t.strip_prefix("core-lines="))
            .expect("core-lines");
        assert!(
            !core.is_empty() && core.chars().all(|c| c.is_ascii_digit()),
            "{facts}"
        );
    }

    #[test]
    fn ksize_check_wiring() {
        const MAIN: &str = include_str!("main.rs");
        assert!(MAIN.lines().any(|l| l == "mod size;"), "mod size;");
        let (_, check) = MAIN.split_once("fn check(").expect("fn check");
        let scan = check.find("pubscan::run(root)").expect("走査");
        let size = check.find("size::measure(root)").expect("大きさの数え");
        let steps = check.find("for step in CHECK_STEPS").expect("cargo の段");
        assert!(scan < size && size < steps, "走査 → 数え → cargo の段の順");
        assert!(
            check[size..steps].contains("return 1;"),
            "落ちたら後の段を撃たない"
        );
    }

    #[test]
    fn ksize_own_names_clean() {
        let lines: Vec<&str> = include_str!("size.rs").lines().collect();
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
        assert_eq!(names.len(), 8, "{names:?}");
        let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
        let unique: BTreeSet<&str> = words.iter().copied().collect();
        assert_eq!(words.len(), 339, "filter の語の数");
        assert_eq!(unique.len(), 339, "filter の語が重なる");
        for name in &names {
            let rest = name
                .strip_prefix("ksize_")
                .unwrap_or_else(|| panic!("{name} が ksize_ で始まらない"));
            for word in &words {
                assert!(!rest.contains(word), "{name} が {word} を含む");
            }
        }
    }
}
