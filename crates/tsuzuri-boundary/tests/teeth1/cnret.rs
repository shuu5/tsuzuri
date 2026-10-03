//! 廃止した設計ノートを名指しから外す歯（接頭辞 cnret_・設計ノート surface-wave23b 行 c-note-retired・要件 FR3）。
//! 偽の bd（台帳の字の file を返す script）と偽の設計の道具（3 つ目の引数が --summary なら要約の字の file、
//! ほかは索引の字の file を返す script）を歯ごとの作業場に置き、event log の字を作業場の state dir に置いて、
//! tz graph --check を撃つ。
#![cfg(test)]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsuzuri_boundary::cli::graph::{LANDED_HEAD, LANDED_UNKNOWN};
use tsuzuri_boundary::server::board::{Texts, built, built_floors};
use tsuzuri_contract::board::Reading;
use tsuzuri_core::graph::check::{RETIRED, landed_notes, note_states};

/// 設計の索引（設計ノートの行 5・辺は無い）。
const INDEX: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）\n\
ra#r1\t設計ノートの行\tdesign-note/ra.yaml\t00000001\t見本の行\n\
ra#r2\t設計ノートの行\tdesign-note/ra.yaml\t00000002\t見本の行\n\
rb#r1\t設計ノートの行\tdesign-note/rb.yaml\t00000003\t見本の行\n\
rc#r1\t設計ノートの行\tdesign-note/rc.yaml\t00000004\t見本の行\n\
rd#r1\t設計ノートの行\tdesign-note/rd.yaml\t00000005\t見本の行\n\
# 辺（1 行 = 端 / 端 / 型・タブ区切り）\n";

/// 台帳の task の 1 本（id・状態・閉じた理由・pointer の行の指す先）。
type Bead = (&'static str, &'static str, Option<&'static str>, &'static str);

/// 節の台帳の y の子の 5 本（y.1 から y.4 は着地して closed・y.5 は open）。
const BEADS: [Bead; 5] = [
    ("y.1", "closed", Some("landed 1111111 ci=success"), "ra.toml#r1"),
    ("y.2", "closed", Some("landed 2222222 ci=success"), "ra.toml#r2"),
    ("y.3", "closed", Some("landed 3333333 ci=success"), "rb.toml#r1"),
    ("y.4", "closed", Some("landed 4444444 ci=success"), "rc.toml#r1"),
    ("y.5", "open", None, "rd.toml#r1"),
];

/// 根の epic y と、y の子の task の列から台帳の字を組む。
fn ledger(beads: &[Bead]) -> String {
    let mut rows = vec![r#"{"id": "y", "title": "根", "status": "open", "issue_type": "epic"}"#.to_string()];
    for (id, status, reason, pointer) in beads {
        let reason = reason
            .map(|r| format!(r#", "close_reason": "{r}""#))
            .unwrap_or_default();
        rows.push(format!(
            r#"{{"id": "{id}", "title": "契約", "status": "{status}", "issue_type": "task"{reason}, "acceptance_criteria": "design = contracts/{pointer}", "dependencies": [{{"issue_id": "{id}", "depends_on_id": "y", "type": "parent-child"}}]}}"#,
        ));
    }
    format!("[\n{}\n]\n", rows.join(",\n"))
}

/// 節の台帳の全部の bead を open にした字。
fn all_open() -> String {
    let beads: Vec<Bead> = BEADS
        .iter()
        .map(|(id, _, _, pointer)| (*id, "open", None, *pointer))
        .collect();
    ledger(&beads)
}

/// 要約の行 1 つ（種類・欄 status は字か null）。
fn row(id: &str, kind: &str, status: Option<&str>) -> String {
    let status = status.map_or("null".to_string(), |s| format!("\"{s}\""));
    format!("{{\"id\":\"{id}\",\"kind\":\"{kind}\",\"file\":\"design-note/x.yaml\",\"status\":{status}}}\n")
}

/// 要約の字（判断の記録の行 1 つと、設計ノートの行 5 つ・状態は行 id と字の対で渡す）。
fn summary(states: &[(&str, Option<&str>)]) -> String {
    let mut text = row("ADR-1", "判断の記録", Some("accepted"));
    for (id, status) in states {
        text.push_str(&row(id, "設計ノートの行", *status));
    }
    text
}

/// 節の要約の字（ra は retired・rb は effective・rc は draft・rd は retired）。
fn sum() -> String {
    summary(&[
        ("ra#r1", Some("retired")),
        ("ra#r2", Some("retired")),
        ("rb#r1", Some("effective")),
        ("rc#r1", Some("draft")),
        ("rd#r1", Some("retired")),
    ])
}

/// 節の要約の字の 1 つの行の状態だけを替えた字。
fn sum_with(id: &str, status: Option<&str>) -> String {
    let states: Vec<(&str, Option<&str>)> = [
        ("ra#r1", Some("retired")),
        ("ra#r2", Some("retired")),
        ("rb#r1", Some("effective")),
        ("rc#r1", Some("draft")),
        ("rd#r1", Some("retired")),
    ]
    .into_iter()
    .map(|(row, s)| if row == id { (row, status) } else { (row, s) })
    .collect();
    summary(&states)
}

/// 節の要約の字から 1 つの行を除いた字。
fn sum_without(id: &str) -> String {
    let states: Vec<(&str, Option<&str>)> = [
        ("ra#r1", Some("retired")),
        ("ra#r2", Some("retired")),
        ("rb#r1", Some("effective")),
        ("rc#r1", Some("draft")),
        ("rd#r1", Some("retired")),
    ]
    .into_iter()
    .filter(|(row, _)| *row != id)
    .collect();
    summary(&states)
}

/// 欄 status の無い要約（folio 0c910db より前の形）。
fn sum_bare() -> String {
    ["ra#r1", "ra#r2", "rb#r1", "rc#r1", "rd#r1"]
        .iter()
        .map(|id| {
            format!("{{\"id\":\"{id}\",\"kind\":\"設計ノートの行\",\"file\":\"design-note/x.yaml\"}}\n")
        })
        .collect()
}

/// event log（走行 1 本）。
const EVENTS: &str = "{\"schema\":1,\"ts\":\"2026-09-27T00:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"y.1-20260927T000000Z\",\"bead\":\"y.1\",\"stage\":\"Intake\"}\n";

/// 計画の verify の filter の語を、ほかの語を部分の字として含まない語に畳んだ字（歯の名が含んではならない部分の字）。
const FILTERS: [&str; 220] = [
    "aaround_",
    "abss_",
    "abst_",
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
    "apark_",
    "apop_",
    "areread_",
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
    "cexcl_",
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
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "g3g7_",
    "gacct_",
    "gapspage_",
    "gatt_",
    "gbnote_",
    "gchip_",
    "gcoach_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gins_",
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
    "hbon_",
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
    "mapview_",
    "mkeys_",
    "mlink_",
    "mqask_",
    "mqface_",
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
    "ntc_",
    "ntime_",
    "nxact_",
    "nxorg_",
    "parts_",
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
    "pubfp_",
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
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server::events::",
    "server_",
    "sesplit_",
    "sgrace_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stbp_",
    "stcli_",
    "steady_",
    "sthr_",
    "stnfy_",
    "stskill_",
    "sttgt_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "wstrip_",
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
    fn new(name: &str, summary: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cnret")
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
        fs::write(root.join("summary.jsonl"), summary).expect("要約の字");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("ledger.json").display()),
        );
        script(
            &root.join("folio"),
            &format!(
                "[ \"$3\" = --summary ] && exec cat '{}'\nexec cat '{}'",
                root.join("summary.jsonl").display(),
                root.join("index.tsv").display()
            ),
        );
        Place { root, repo, state }
    }

    /// tz graph --check を撃つ。
    fn check(&self) -> Output {
        let args: Vec<OsString> = vec![
            "graph".into(),
            "--check".into(),
            "--repo".into(),
            self.repo.clone().into(),
            "--bd".into(),
            self.root.join("bd").into(),
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

fn texts(ledger: &str, summary: &str) -> Texts {
    Texts {
        design: INDEX.to_string(),
        ledger: ledger.to_string(),
        events: EVENTS.to_string(),
        summary: summary.to_string(),
        rulings: String::new(),
    }
}

fn owned(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// 台帳の字と要約の字を渡した名指し。
fn named(ledger: &str, summary: &str) -> Reading<Vec<String>> {
    let g = built(&texts(ledger, summary));
    landed_notes(&g, ledger, summary)
}

fn table(rows: &[(&str, &str)]) -> BTreeMap<String, String> {
    rows.iter()
        .map(|(id, status)| (id.to_string(), status.to_string()))
        .collect()
}

#[test]
fn cnret_state_decides() {
    let word: &str = RETIRED;
    assert_eq!(word, "retired");
    let five = [
        ("ra#r1", "retired"),
        ("ra#r2", "retired"),
        ("rb#r1", "effective"),
        ("rc#r1", "draft"),
        ("rd#r1", "retired"),
    ];
    assert_eq!(note_states(&sum()), Some(table(&five)));
    assert_eq!(note_states(""), None);
    assert_eq!(note_states(&format!("{}これは JSON でない\n", sum())), None);
    assert_eq!(
        note_states(&sum_with("rb#r1", None)),
        Some(table(&[five[0], five[1], five[3], five[4]]))
    );

    let text = ledger(&BEADS);
    let read = texts(&text, &sum());
    let (g, floors) = built_floors(&read);
    assert_eq!(g, built(&read));
    assert_eq!(floors.landed, landed_notes(&g, &text, &sum()));
    assert_eq!(floors.landed, Reading::Known(owned(&["rb", "rc"])));
    assert_eq!(named(&text, &sum()), Reading::Known(owned(&["rb", "rc"])));
    assert_eq!(
        named(&text, &sum_with("rb#r1", Some("retired"))),
        Reading::Known(owned(&["rc"]))
    );
    let effective = summary(&[
        ("ra#r1", Some("effective")),
        ("ra#r2", Some("effective")),
        ("rb#r1", Some("effective")),
        ("rc#r1", Some("effective")),
        ("rd#r1", Some("effective")),
    ]);
    assert_eq!(
        named(&text, &effective),
        Reading::Known(owned(&["ra", "rb", "rc"]))
    );
}

#[test]
fn cnret_unread_state_is_unknown() {
    let text = ledger(&BEADS);
    for (name, unread) in [
        ("状態の欄の無い要約", sum_bare()),
        ("空の要約", String::new()),
        ("ra#r2 の行の無い要約", sum_without("ra#r2")),
        ("状態の揃わない要約", sum_with("ra#r2", Some("effective"))),
        ("rb#r1 の状態が null の要約", sum_with("rb#r1", None)),
    ] {
        assert_eq!(named(&text, &unread), Reading::Unknown, "{name}");
        let (_, floors) = built_floors(&texts(&text, &unread));
        assert_eq!(floors.landed, Reading::Unknown, "{name}");
    }
    assert_eq!(
        named(&text, &sum_without("rd#r1")),
        Reading::Known(owned(&["rb", "rc"]))
    );

    let open = all_open();
    for unread in [String::new(), sum_bare()] {
        assert_eq!(named(&open, &unread), Reading::Known(Vec::new()));
    }
}

#[test]
fn cnret_check_leaves_retired_out() {
    let place = Place::new("retired", &sum());
    let out = place.check();
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let text = String::from_utf8(out.stdout.clone()).expect("標準出力の字");
    let heads: Vec<&str> = text.lines().filter(|l| l.starts_with(LANDED_HEAD)).collect();
    assert_eq!(heads, ["全部の行が着地した設計ノート 2（rb・rc）"], "{text}");
    let err = String::from_utf8(out.stderr.clone()).expect("標準エラーの字");
    let unknown = format!("# まだ分からない: {LANDED_UNKNOWN}");
    assert_eq!(err.lines().filter(|l| *l == unknown).count(), 0, "{err}");

    let place = Place::new("bare", &sum_bare());
    let out = place.check();
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let text = String::from_utf8(out.stdout.clone()).expect("標準出力の字");
    assert_eq!(
        text.lines().filter(|l| l.starts_with(LANDED_HEAD)).count(),
        0,
        "{text}"
    );
    let err = String::from_utf8(out.stderr.clone()).expect("標準エラーの字");
    assert_eq!(err.lines().filter(|l| *l == unknown).count(), 1, "{err}");
}

#[test]
fn cnret_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/cnret.rs"))
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
            .strip_prefix("cnret_")
            .unwrap_or_else(|| panic!("{name} は cnret_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
    for (i, a) in FILTERS.iter().enumerate() {
        for b in &FILTERS[i + 1..] {
            assert_ne!(a, b, "filter の語が重なる");
        }
    }
}
