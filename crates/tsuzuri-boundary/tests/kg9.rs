//! 本文だけで名指した id の対の数の歯（接頭辞 kg9_・設計ノート surface-wave20b 行 k-g9-count・要件 FR3）。
//! 偽の bd（台帳の字の file を返す script）と偽の設計の道具（3 つ目の引数が --summary なら要約の字の file、
//! ほかは索引の字の file を返す script）を歯ごとの作業場に置き、event log の字を作業場の state dir に置いて、
//! tz graph --check を撃つ。
#![cfg(test)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsuzuri_boundary::cli::graph::{BARE_UNKNOWN, UNFIELDED_HEAD, UNFIELDED_UNKNOWN, unfielded_line};
use tsuzuri_boundary::server::board::{Texts, built, built_floors};
use tsuzuri_contract::board::Reading;
use tsuzuri_core::graph::check::{unfielded_mentions, unsummarized};

/// 設計の索引（節点 5・辺 3）。
const INDEX: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）\n\
A-1\t条\tconstitution.yaml\t00000000\t見本の条\n\
A-1.1\t規範文\tconstitution.yaml\t00000001\t見本の規範文\n\
FR1\t要件\tsrs.yaml\t00000002\t見本の要件\n\
R-2\t規則行\trules.yaml\t00000003\t見本の規則行\n\
nt#r1\t設計ノートの行\tdesign-note/nt.yaml\t00000004\t見本の設計ノートの行\n\
# 辺（1 行 = 端 / 端 / 型・タブ区切り）\n\
A-1\tA-1.1\tin-article\n\
A-1.1\tA-1\tin-article\n\
FR1\tA-1\trefs\n";

/// 索引に、本文で名指した 3 つの対の辺を足した字。
const INDEX_LINKED: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）\n\
A-1\t条\tconstitution.yaml\t00000000\t見本の条\n\
A-1.1\t規範文\tconstitution.yaml\t00000001\t見本の規範文\n\
FR1\t要件\tsrs.yaml\t00000002\t見本の要件\n\
R-2\t規則行\trules.yaml\t00000003\t見本の規則行\n\
nt#r1\t設計ノートの行\tdesign-note/nt.yaml\t00000004\t見本の設計ノートの行\n\
# 辺（1 行 = 端 / 端 / 型・タブ区切り）\n\
A-1\tA-1.1\tin-article\n\
A-1.1\tA-1\tin-article\n\
FR1\tA-1\trefs\n\
A-1\tR-2\trules\n\
nt#r1\tFR1\treq\n\
nt#r1\tR-2\trules\n";

/// 台帳（bead 3 本・どれも description の定型行を持つ）。
const LEDGER: &str = r#"[
{"id": "k", "title": "根", "status": "open", "issue_type": "epic", "description": "概要 = 根の概要"},
{"id": "k.1", "title": "契約", "status": "open", "issue_type": "task", "acceptance_criteria": "design = contracts/x.toml#a",
 "description": "概要 = 契約の概要",
 "dependencies": [{"issue_id": "k.1", "depends_on_id": "k", "type": "parent-child"}]},
{"id": "k.2", "title": "問い", "status": "open", "issue_type": "task", "labels": ["intake:question"],
 "description": "技術 = 問いの技術",
 "metadata": {"touches": ["FR1"]},
 "dependencies": [{"issue_id": "k.2", "depends_on_id": "k", "type": "parent-child"}]}
]
"#;

/// 要約（A-1 と FR1 と nt#r1 の 3 行・A-1.1 と R-2 の行は無い）。
const SUMMARY: &str = "{\"id\":\"A-1\",\"file\":\"constitution.yaml\",\"line\":3,\"plain\":\"FR1 と R-2 を見る\"}\n\
{\"id\":\"FR1\",\"file\":\"srs.yaml\",\"line\":7,\"eng\":\"A-1.1 に従う\"}\n\
{\"id\":\"nt#r1\",\"file\":\"design-note/nt.yaml\",\"line\":9,\"eng\":\"FR1 を満たす・R-2 を写す\"}\n";

/// event log（走行 1 本）。
const EVENTS: &str = "{\"schema\":1,\"ts\":\"2026-09-27T00:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"k.1-20260927T000000Z\",\"bead\":\"k.1\",\"stage\":\"Intake\"}\n";

/// 計画の verify の filter の語と同じノートのもう 1 行の接頭辞（歯の名が含んではならない部分の字）。
const FILTERS: [&str; 191] = [
    "aaround_",
    "accept_",
    "acchold_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "aface_",
    "afocus_",
    "alean_",
    "aord_",
    "aown_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadl_",
    "cadopt_",
    "cadq_",
    "cdorm_",
    "cfsplit_",
    "cg9_",
    "cgdom_",
    "cmark_",
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
    "ecache_",
    "epolq_",
    "eretry_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "g3g7_",
    "gapspage_",
    "gatt_",
    "gbnote_",
    "gcoach_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "gwv_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
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
    "hsderive_",
    "hshist_",
    "hspage_",
    "hspk_",
    "hsym_",
    "hthr_",
    "http_",
    "hwstore_",
    "iclose_",
    "ilink_",
    "jcount_",
    "jrun_",
    "kcli_",
    "kindlab_",
    "klink_",
    "ksum_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstg_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mstore_",
    "mtips_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntime_",
    "nxact_",
    "nxorg_",
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
    "pgz_",
    "pipe_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "question_",
    "rbusy_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stbp_",
    "stcli_",
    "steady_",
    "sttgt_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "cnote_",
];

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場（repo の置き場・state dir・字の file・偽の bd と偽の設計の道具）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
}

impl Place {
    /// `summary` が None なら偽の設計の道具はどの引数にも索引の字を返す（要約が読めない）。
    fn new(name: &str, index: &str, summary: Option<&str>) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("kg9")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let state = root.join("state");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::write(repo.join(".beads/issues.jsonl"), "{}\n").expect("印の file");
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::write(state.join("fleet/events.jsonl"), EVENTS).expect("event log");
        fs::write(root.join("ledger.json"), LEDGER).expect("台帳の字");
        fs::write(root.join("index.tsv"), index).expect("索引の字");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("ledger.json").display()),
        );
        let index = format!("exec cat '{}'", root.join("index.tsv").display());
        let body = match summary {
            Some(text) => {
                fs::write(root.join("summary.jsonl"), text).expect("要約の字");
                format!(
                    "[ \"$3\" = --summary ] && exec cat '{}'\n{index}",
                    root.join("summary.jsonl").display()
                )
            }
            None => index,
        };
        script(&root.join("folio"), &body);
        Place { root, repo, state }
    }

    /// tz graph --check を撃つ（`bd` が偽なら bd の program は無い path）。
    fn check(&self, bd: bool) -> Output {
        let bd = if bd {
            self.root.join("bd")
        } else {
            self.root.join("no-such-bd")
        };
        let args: Vec<OsString> = vec![
            "graph".into(),
            "--check".into(),
            "--repo".into(),
            self.repo.clone().into(),
            "--bd".into(),
            bd.into(),
            "--folio".into(),
            self.root.join("folio").into(),
            "--state-dir".into(),
            self.state.clone().into(),
        ];
        Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .output()
            .expect("tz を撃つ")
    }
}

fn texts(summary: &str) -> Texts {
    Texts {
        design: INDEX.to_string(),
        ledger: LEDGER.to_string(),
        events: EVENTS.to_string(),
        summary: summary.to_string(),
        rulings: String::new(),
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("標準出力の字")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("標準エラーの字")
}

fn summary(unknowns: usize) -> String {
    format!("tz graph --check: まだ分からない（違反 0・まだ分からない {unknowns}）\n")
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

/// 要約の無い節点の行（A-1.1 は要約の行が無く、R-2 は本文が無い）。
const BARE: &str = "要約の無い節点 2（A-1.1・R-2）\n";

/// 節の索引で本文だけで名指した id の対の行。
const UNFIELDED: &str = "本文だけで名指した id の対 3（A-1→R-2・nt#r1→FR1・nt#r1→R-2）\n";

/// 全部の行が着地した設計ノートの行（nt#r1 を指す bead は無い・行 c-note-stale）。
const LANDED: &str = "全部の行が着地した設計ノート 0\n";

#[test]
fn kg9_line_and_built_floors() {
    let head: &str = UNFIELDED_HEAD;
    let unknown: &str = UNFIELDED_UNKNOWN;
    assert_eq!(head, "本文だけで名指した id の対");
    assert_eq!(unknown, "本文だけで名指した id の対（要約か設計の索引が読めない）");
    let line: fn(&[(String, String)]) -> String = unfielded_line;
    assert_eq!(line(&[]), "本文だけで名指した id の対 0");
    assert_eq!(
        line(&pairs(&[("nt#r1", "FR1"), ("A-1", "R-2")])),
        "本文だけで名指した id の対 2（nt#r1→FR1・A-1→R-2）"
    );

    let read = texts(SUMMARY);
    let (g, floors) = built_floors(&read);
    assert_eq!(g, built(&read));
    assert_eq!(floors.bare, unsummarized(&g, true));
    assert_eq!(floors.unfielded, unfielded_mentions(&g, true));
    assert_eq!(
        floors.unfielded,
        Reading::Known(pairs(&[
            ("A-1", "R-2"),
            ("nt#r1", "FR1"),
            ("nt#r1", "R-2")
        ]))
    );

    let unread = texts("");
    let (g, floors) = built_floors(&unread);
    assert_eq!(g, built(&unread));
    assert_eq!(floors.bare, Reading::Unknown);
    assert_eq!(floors.unfielded, Reading::Unknown);
}

#[test]
fn kg9_check_names_pairs() {
    let place = Place::new("names", INDEX, Some(SUMMARY));
    let out = place.check(true);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(
        stdout(&out),
        format!("{BARE}{UNFIELDED}{LANDED}{}", summary(3))
    );
    let err = stderr(&out);
    assert!(!err.contains(UNFIELDED_HEAD), "{err}");

    let place = Place::new("linked", INDEX_LINKED, Some(SUMMARY));
    let out = place.check(true);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(
        stdout(&out),
        format!("{BARE}本文だけで名指した id の対 0\n{LANDED}{}", summary(3))
    );
}

#[test]
fn kg9_unread_goes_to_stderr() {
    let unfielded = format!("# まだ分からない: {UNFIELDED_UNKNOWN}");
    let bare = format!("# まだ分からない: {BARE_UNKNOWN}");
    let place = Place::new("unread", INDEX, None);
    let out = place.check(true);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(stdout(&out), format!("{LANDED}{}", summary(3)));
    let err = stderr(&out);
    assert_eq!(err.lines().filter(|l| *l == unfielded).count(), 1, "{err}");

    let place = Place::new("no-bd", INDEX, Some(SUMMARY));
    let out = place.check(false);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(stdout(&out), format!("{UNFIELDED}{}", summary(11)));
    let err = stderr(&out);
    assert_eq!(err.lines().filter(|l| *l == unfielded).count(), 0, "{err}");
    assert_eq!(err.lines().filter(|l| *l == bare).count(), 1, "{err}");
}

#[test]
fn kg9_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/kg9.rs"))
        .expect("この file");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("fn の行");
            rest.split('(').next().expect("fn の名")
        })
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("kg9_")
            .unwrap_or_else(|| panic!("{name} は kg9_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
