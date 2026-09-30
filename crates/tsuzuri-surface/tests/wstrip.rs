//! 行 g-strip の歯: workspace の release の profile の strip（面の wasm の関数の名の表を抜く）と、
//! trunk が cargo の release の profile で面の wasm を組むこと（Trunk.toml と index.html）と、この file の歯の名。

use std::path::PathBuf;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// trim した行が角括弧で始まるなら、最初の角括弧の中の字（表の見出し）。
fn heading(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix('[')?;
    let end = rest.find(']').expect("見出しの閉じの角括弧");
    Some(&rest[..end])
}

/// 見出し `name` の後から次の見出しの前までの、井桁で始まる行と空の行を除いた行（trim）。
fn section<'a>(text: &'a str, name: &str) -> Vec<&'a str> {
    let mut lines = text.lines();
    lines
        .by_ref()
        .find(|l| heading(l) == Some(name))
        .unwrap_or_else(|| panic!("見出し {name} が無い"));
    lines
        .take_while(|l| heading(l).is_none())
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// 等号の左の字（trim）と右の字。
fn key_value(line: &str) -> (&str, &str) {
    let (k, v) = line.split_once('=').unwrap_or_else(|| panic!("等号の無い行: {line}"));
    (k.trim(), v)
}

#[test]
fn wstrip_release_profile_keys() {
    let text = read("../../Cargo.toml");
    let mut heads: Vec<&str> = text.lines().filter_map(heading).collect();
    heads.sort_unstable();
    let mut want = vec![
        "profile.dev.package.folio",
        "workspace",
        "workspace.package",
        "workspace.dependencies",
        "workspace.lints.clippy",
        "workspace.lints.rust",
        "profile.release",
    ];
    want.sort_unstable();
    assert_eq!(heads, want);

    let lines = section(&text, "profile.release");
    let keys: Vec<&str> = lines.iter().map(|l| key_value(l).0).collect();
    assert_eq!(keys, ["strip"]);
    let value = key_value(lines[0]).1;
    let quoted: Vec<&str> = value.splitn(3, '"').collect();
    assert_eq!(quoted.len(), 3, "strip の値に引用符が 2 つ無い: {value}");
    assert_eq!(quoted[1], "symbols");

    // 行 k-join-folio: folio の crate だけを最適化して組む段は opt-level = 2 の 1 行だけ。
    assert_eq!(section(&text, "profile.dev.package.folio"), ["opt-level = 2"]);
}

#[test]
fn wstrip_trunk_release_build() {
    let trunk = read("Trunk.toml");
    let release: Vec<&str> = section(&trunk, "build")
        .into_iter()
        .map(key_value)
        .filter(|(k, _)| *k == "release")
        .map(|(_, v)| v.split('#').next().unwrap_or("").trim())
        .collect();
    assert_eq!(release, ["true"]);
    for line in trunk.lines() {
        if let Some((k, _)) = line.split_once('=') {
            assert_ne!(k.trim(), "cargo_profile", "Trunk.toml の行: {line}");
        }
    }

    let html = read("index.html");
    assert!(html.contains("rel=\"rust\""), "index.html に rel=\"rust\" が無い");
    assert!(!html.contains("data-cargo-profile"), "index.html に data-cargo-profile が在る");
}

#[test]
fn wstrip_own_names_clean() {
    let text = read("tests/wstrip.rs");
    let mut names = Vec::new();
    let mut rest = text.as_str();
    // 属性の行と次の行の `fn `（この字の literal は逆斜線の escape で改行を持たず当たらない）。
    let mark = "\n#[test]\nfn ";
    while let Some(i) = rest.find(mark) {
        let after = &rest[i + mark.len()..];
        let end = after.find('(').expect("fn の名の終わり");
        names.push(after[..end].trim().to_string());
        rest = &after[end..];
    }
    assert_eq!(names.len(), 3, "{names:?}");
    let words = [
        "accept_",
        "account_",
        "acctcore_",
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
        "askcard_",
        "batchpanel_",
        "board_min_",
        "contract_form_",
        "frame_",
        "gapspage_",
        "gquestion_",
        "graph_",
        "gview_",
        "hbconf_",
        "hbpost_",
        "hbproc_",
        "hbroute_",
        "hook_",
        "hsblock_",
        "hsderive_",
        "hspage_",
        "ledgerblock_",
        "mapgraph_",
        "mapview_",
        "nextstep_",
        "nodepage_",
        "parts_",
        "pipe_",
        "project_",
        "question_",
        "seatblock_",
        "seatcard_",
        "server_",
        "skeleton_",
        "stage_",
        "stats_",
        "steady_",
        "topbar_",
        "tz_",
        "mlink_",
        "gnav_",
        "mkeys_",
        "klink_",
        "ilink_",
        "afocus_",
        "nxact_",
        "urpanel_",
        "saxis_",
        "ticker_",
        "hfig_",
        "pmore_",
        "lspark_",
        "apop_",
        "brand_",
        "runsdoc_",
        "nbatch_",
        "kcli_",
        "qgate_",
        "nsum_",
        "hcard_",
        "ntime_",
        "ptitle_",
        "ncard_",
        "hsym_",
        "smore_",
        "lcard_",
        "aaround_",
        "hruling_",
        "sxaxis_",
        "plimit_",
        "bport_",
        "fstop_",
        "nsumw_",
        "fmark_",
        "fserve_",
        "cadopt_",
        "tipx_",
        "hcsess_",
        "hcproj_",
        "sesplit_",
        "mstore_",
        "athr_",
        "qblock_",
        "wsteady_",
        "gsum_",
        "nstall_",
        "pfold_",
        "uword_",
        "cround_",
        "cgdom_",
        "csled_",
        "lhome_",
        "shb_",
        "ghb_",
        "nact_",
        "aord_",
        "mtree_",
        "rhold_",
        "qkey_",
        "flight_",
    ];
    assert_eq!(words.len(), 110);
    for name in &names {
        let rest = name
            .strip_prefix("wstrip_")
            .unwrap_or_else(|| panic!("{name} が wstrip_ で始まらない"));
        for w in words {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
