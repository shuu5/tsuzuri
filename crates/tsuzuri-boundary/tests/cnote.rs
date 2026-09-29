//! 全部の行が着地した設計ノートの歯（接頭辞 cnote_・設計ノート surface-wave20b 行 c-note-stale・要件 FR3）。
//! 偽の bd（台帳の字の file を返す script）と偽の設計の道具（どの引数にも索引の字の file を返す script・要約と
//! 裁定の書き出しは読めない）を歯ごとの作業場に置き、event log の字を作業場の state dir に置いて、tz graph --check を撃つ。

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsuzuri_boundary::cli::graph::{LANDED_HEAD, LANDED_UNKNOWN, landed_line};
use tsuzuri_boundary::server::board::{Texts, built, built_floors};
use tsuzuri_contract::board::Reading;
use tsuzuri_core::graph::check::{LANDED, landed_notes};

/// 設計の索引（節点 12・どれも設計ノートの行・nh-1 だけ井桁が無い・辺は無い）。
const INDEX: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）\n\
na#r1\t設計ノートの行\tdesign-note/na.yaml\t00000001\t見本の行\n\
na#r2\t設計ノートの行\tdesign-note/na.yaml\t00000002\t見本の行\n\
nb#r1\t設計ノートの行\tdesign-note/nb.yaml\t00000003\t見本の行\n\
nc#r1\t設計ノートの行\tdesign-note/nc.yaml\t00000004\t見本の行\n\
nd#r1\t設計ノートの行\tdesign-note/nd.yaml\t00000005\t見本の行\n\
ne#r1\t設計ノートの行\tdesign-note/ne.yaml\t00000006\t見本の行\n\
ne#r2\t設計ノートの行\tdesign-note/ne.yaml\t00000007\t見本の行\n\
nf#r1\t設計ノートの行\tdesign-note/nf.yaml\t00000008\t見本の行\n\
ng#r1\t設計ノートの行\tdesign-note/ng.yaml\t00000009\t見本の行\n\
nh-1\t設計ノートの行\tdesign-note/nh.yaml\t00000010\t見本の行\n\
ni#r1\t設計ノートの行\tdesign-note/ni.yaml\t00000011\t見本の行\n\
nj#r1\t設計ノートの行\tdesign-note/nj.yaml\t00000012\t見本の行\n\
# 辺（1 行 = 端 / 端 / 型・タブ区切り）\n";

/// 台帳の task の 1 本（id・状態・閉じた理由・pointer の行の指す先）。
#[derive(Clone, Copy)]
struct Bead {
    id: &'static str,
    status: &'static str,
    reason: Option<&'static str>,
    pointer: &'static str,
}

const fn bead(
    id: &'static str,
    status: &'static str,
    reason: Option<&'static str>,
    pointer: &'static str,
) -> Bead {
    Bead {
        id,
        status,
        reason,
        pointer,
    }
}

/// 節の台帳の x の子の 13 本。
const BEADS: [Bead; 13] = [
    bead("x.1", "closed", Some("landed 1111111 ci=success"), "na.toml#r1"),
    bead("x.2", "closed", Some("landed 2222222 ci=success"), "na.toml#r2"),
    bead("x.3", "closed", Some("吸収 nb は nz へ"), "nb.toml#r1"),
    bead("x.4", "closed", Some("  landed   4444444  ci=success"), "nc.toml#r1"),
    bead("x.5", "closed", Some("landed 5555555 ci=success"), "ne.toml#r1"),
    bead("x.6", "open", None, "ne.toml#r2"),
    bead("x.7", "closed", Some("landed 7777777 ci=success"), "nf.toml#r1"),
    bead("x.8", "open", None, "nf.toml#r1"),
    bead("x.9", "closed", Some("landedx 9999999 ci=success"), "ng.toml#r1"),
    bead("x.10", "closed", Some("landed 1010101 ci=success"), "nh.toml#1"),
    bead("x.11", "closed", Some("landed 1111011 ci=success"), "zz.toml#r1"),
    bead("x.12", "closed", Some("着地（便 ni の見本）"), "ni.toml#r1"),
    bead("x.13", "closed", Some("着地点を見直す"), "nj.toml#r1"),
];

/// 根の epic x と、x の子の task の列から台帳の字を組む。
fn ledger(beads: &[Bead]) -> String {
    let mut rows = vec![r#"{"id": "x", "title": "根", "status": "open", "issue_type": "epic"}"#.to_string()];
    for b in beads {
        let reason = b
            .reason
            .map(|r| format!(r#", "close_reason": "{r}""#))
            .unwrap_or_default();
        rows.push(format!(
            r#"{{"id": "{id}", "title": "契約", "status": "{status}", "issue_type": "task"{reason}, "acceptance_criteria": "design = contracts/{pointer}", "dependencies": [{{"issue_id": "{id}", "depends_on_id": "x", "type": "parent-child"}}]}}"#,
            id = b.id,
            status = b.status,
            pointer = b.pointer,
        ));
    }
    format!("[\n{}\n]\n", rows.join(",\n"))
}

/// 節の台帳の 1 本を替えた字。
fn with(id: &str, status: &'static str, reason: Option<&'static str>) -> String {
    let beads: Vec<Bead> = BEADS
        .iter()
        .map(|b| {
            if b.id == id {
                Bead {
                    status,
                    reason,
                    ..*b
                }
            } else {
                *b
            }
        })
        .collect();
    ledger(&beads)
}

/// event log（走行 1 本）。
const EVENTS: &str = "{\"schema\":1,\"ts\":\"2026-09-27T00:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"x.1-20260927T000000Z\",\"bead\":\"x.1\",\"stage\":\"Intake\"}\n";

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
    "kg9_",
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
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cnote")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let state = root.join("state");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::write(repo.join(".beads/issues.jsonl"), "{}\n").expect("印の file");
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::write(state.join("fleet/events.jsonl"), EVENTS).expect("event log");
        fs::write(root.join("ledger.json"), ledger(&BEADS)).expect("台帳の字");
        fs::write(root.join("index.tsv"), INDEX).expect("索引の字");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("ledger.json").display()),
        );
        script(
            &root.join("folio"),
            &format!("exec cat '{}'", root.join("index.tsv").display()),
        );
        Place { root, repo, state }
    }

    /// tz graph --check を撃つ（`bd`・`folio` が偽ならその program は無い path）。
    fn check(&self, bd: bool, folio: bool) -> Output {
        let program = |name: &str, present: bool| {
            if present {
                self.root.join(name)
            } else {
                self.root.join(format!("no-such-{name}"))
            }
        };
        let args: Vec<OsString> = vec![
            "graph".into(),
            "--check".into(),
            "--repo".into(),
            self.repo.clone().into(),
            "--bd".into(),
            program("bd", bd).into(),
            "--folio".into(),
            program("folio", folio).into(),
            "--state-dir".into(),
            self.state.clone().into(),
        ];
        Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .output()
            .expect("tz を撃つ")
    }
}

fn texts(design: &str, ledger: &str) -> Texts {
    Texts {
        design: design.to_string(),
        ledger: ledger.to_string(),
        events: EVENTS.to_string(),
        summary: String::new(),
        rulings: String::new(),
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("標準出力の字")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("標準エラーの字")
}

fn owned(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// 台帳の字を節の索引と組んだグラフに渡した名指し。
fn named(ledger: &str) -> Reading<Vec<String>> {
    let g = built(&texts(INDEX, ledger));
    landed_notes(&g, ledger)
}

/// 要約の行（違反 0・まだ分からない 3 は g-3・g-7・g-9）。
const SUMMARY: &str = "tz graph --check: まだ分からない（違反 0・まだ分からない 3）\n";

#[test]
fn cnote_line_and_built_floors() {
    let heads: [&str; 2] = LANDED;
    assert_eq!(heads, ["landed", "着地"]);
    let head: &str = LANDED_HEAD;
    let unknown: &str = LANDED_UNKNOWN;
    assert_eq!(head, "全部の行が着地した設計ノート");
    assert_eq!(
        unknown,
        "全部の行が着地した設計ノート（設計の索引か台帳が読めない）"
    );
    let line: fn(&[String]) -> String = landed_line;
    assert_eq!(line(&[]), "全部の行が着地した設計ノート 0");
    assert_eq!(
        line(&owned(&["nc", "na"])),
        "全部の行が着地した設計ノート 2（nc・na）"
    );

    let text = ledger(&BEADS);
    let read = texts(INDEX, &text);
    let (g, floors) = built_floors(&read);
    assert_eq!(g, built(&read));
    assert_eq!(floors.landed, landed_notes(&g, &text));
    assert_eq!(floors.landed, Reading::Known(owned(&["na", "nc", "ni"])));

    for unread in [texts(INDEX, ""), texts("", &text)] {
        let (g, floors) = built_floors(&unread);
        assert_eq!(floors.landed, Reading::Unknown);
        assert_eq!(landed_notes(&g, &unread.ledger), Reading::Unknown);
    }
}

#[test]
fn cnote_rows_need_landed_and_no_open() {
    let known = |ids: &[&str]| Reading::Known(owned(ids));
    assert_eq!(named(&ledger(&BEADS)), known(&["na", "nc", "ni"]));
    assert_eq!(
        named(&with("x.2", "closed", Some("Landed 2222222 ci=success"))),
        known(&["nc", "ni"])
    );
    assert_eq!(
        named(&with("x.13", "closed", Some("着地 1313131"))),
        known(&["na", "nc", "ni", "nj"])
    );
    assert_eq!(
        named(&with("x.6", "closed", Some("landed 6666666"))),
        known(&["na", "nc", "ne", "ni"])
    );
    assert_eq!(
        named(&with("x.8", "closed", Some("見送り"))),
        known(&["na", "nc", "nf", "ni"])
    );
    assert_eq!(
        named(&with("x.9", "closed", Some("landed 9999999 ci=success"))),
        known(&["na", "nc", "ng", "ni"])
    );
    for status in ["open", "tombstone"] {
        assert_eq!(
            named(&with("x.3", status, Some("landed 3333333"))),
            known(&["na", "nc", "ni"]),
            "{status}"
        );
    }
}

#[test]
fn cnote_check_names_notes() {
    let place = Place::new("names");
    let out = place.check(true, true);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(
        stdout(&out),
        format!("全部の行が着地した設計ノート 3（na・nc・ni）\n{SUMMARY}")
    );
    let err = stderr(&out);
    assert!(!err.contains(LANDED_HEAD), "{err}");
}

#[test]
fn cnote_unread_goes_to_stderr() {
    let line = format!("# まだ分からない: {LANDED_UNKNOWN}");
    for (name, bd, folio) in [("no-bd", false, true), ("no-folio", true, false)] {
        let place = Place::new(name);
        let out = place.check(bd, folio);
        assert_eq!(out.status.code(), Some(2), "{name}: {out:?}");
        let text = stdout(&out);
        assert_eq!(text.lines().count(), 1, "{name}: {text}");
        assert!(text.starts_with("tz graph --check"), "{name}: {text}");
        assert!(!text.contains(LANDED_HEAD), "{name}: {text}");
        let err = stderr(&out);
        assert_eq!(
            err.lines().filter(|l| *l == line).count(),
            1,
            "{name}: {err}"
        );
    }
}

#[test]
fn cnote_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cnote.rs"))
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
    assert_eq!(names.len(), 5, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("cnote_")
            .unwrap_or_else(|| panic!("{name} は cnote_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
