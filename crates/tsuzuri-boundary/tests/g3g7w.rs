//! 不変条件 g-3・g-7 の境界の歯（接頭辞 g3g7_・行 c-g3g7）: 設計の道具の裁定の書き出しの読みと、
//! 口の電文と tz graph --check。偽の bd（台帳の字の file を返す script）と偽の設計の道具（引数に --emit-rulings が
//! 在れば書き出しの字・--summary が在れば rc 1・ほかは索引の字を返す script）を歯ごとの作業場に置いて撃つ。

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsuzuri_boundary::cli::graph::NEXT;
use tsuzuri_boundary::server::board::{self, Texts};
use tsuzuri_boundary::server::design::{Design, RULINGS_ARGS};
use tsuzuri_contract::graph::{EdgeType, Verdict};
use tsuzuri_core::graph::{self, Inputs};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/graph/unruled")
        .join(name)
}

fn read(name: &str) -> String {
    fs::read_to_string(fixture(name)).unwrap_or_else(|e| panic!("{name} を読む: {e}"))
}

/// q7-fx.4 の notes の裁定の行。
const Q4_LINE: &str = "裁定 id = q7-fx.4:20260928T0100Z-1・ADR-7 を発効する";

/// 問いでない bead（memo q7-fx.1）の notes の裁定 id。
const MEMO_RULING: &str = "q7-fx.1:20260928T0700Z-1";

/// folio の裁定 id の文法の外の裁定の行。
const USER_LINE: &str = "裁定 id = user 2026-09-28T01:00Z・見本";

/// q7-fx.4 の裁定の行だけを別の字に替えた台帳。
fn cut() -> String {
    let text = read("ledger.json");
    assert!(text.contains(Q4_LINE), "台帳に q7-fx.4 の裁定の行");
    text.replace(Q4_LINE, "見本の字")
}

/// 台帳の q7-fx.1 の notes を替えた字。
fn with_memo(ledger: &str, notes: &str) -> String {
    let from = "\"notes\": \"見本の memo の notes\"";
    assert!(ledger.contains(from), "台帳に q7-fx.1 の notes");
    ledger.replace(from, &format!("\"notes\": \"{}\"", notes.replace('\n', "\\n")))
}

/// q7-fx.1 の notes に文法の外の id の裁定の行を足した字。
fn plus_user(ledger: &str) -> String {
    with_memo(ledger, &format!("見本の memo の notes\n{USER_LINE}"))
}

/// q7-fx.1 の notes を文法の外の id と文法の内の id の裁定の行 2 つにした字。
fn memo() -> String {
    with_memo(
        &read("ledger.json"),
        &format!("{USER_LINE}\n裁定 id = {MEMO_RULING}・見本"),
    )
}

/// 書き出しに node ADR-8・form bead の行を 1 つ足した字。
fn plus_adr8(rulings: &str, bead: &str) -> String {
    format!(
        "{rulings}{{\"ruling\":\"{bead}\",\"form\":\"bead\",\"bead\":\"{bead}\",\"node\":\"ADR-8\",\"file\":\"adr/ADR-8.yaml\",\"line\":9,\"field\":\"approval.ruling\"}}\n"
    )
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場（repo の置き場・state dir・台帳と書き出しの字の file・偽の bd と偽の設計の道具）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
}

impl Place {
    /// `rulings_rc` は書き出しを出した後の偽の設計の道具の終了 code。
    fn new(name: &str, ledger: &str, rulings: &str, rulings_rc: u8) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("g3g7w")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let state = root.join("state");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::write(repo.join(".beads/issues.jsonl"), "{}\n").expect("印の file");
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::write(state.join("fleet/events.jsonl"), read("events.jsonl")).expect("event log");
        fs::write(root.join("ledger.json"), ledger).expect("台帳の字");
        fs::write(root.join("rulings.jsonl"), rulings).expect("書き出しの字");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("ledger.json").display()),
        );
        script(
            &root.join("folio"),
            &format!(
                "for a in \"$@\"; do\n\
                 [ \"$a\" = --emit-rulings ] && {{ cat '{}'; exit {rulings_rc}; }}\n\
                 [ \"$a\" = --summary ] && exit 1\n\
                 done\n\
                 exec cat '{}'",
                root.join("rulings.jsonl").display(),
                fixture("index.tsv").display()
            ),
        );
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

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("標準出力の字")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("標準エラーの字")
}

fn next(id: &str) -> &'static str {
    NEXT.iter()
        .find(|(i, _)| *i == id)
        .map(|(_, n)| *n)
        .expect("次の 1 手")
}

fn summary(word: &str, violations: usize, unknowns: usize) -> String {
    format!("tz graph --check: {word}（違反 {violations}・まだ分からない {unknowns}）\n")
}

fn unknown(id: &str, why: &str) -> String {
    format!("# まだ分からない: [{id}] {why}")
}

const UNREAD: &str = "読めない出所が在る";
const NO_TOOL: &str = "測る機構がまだ無い";

fn lines_of(err: &str, line: &str) -> usize {
    err.lines().filter(|l| *l == line).count()
}

#[test]
fn g3g7_rulings_args_and_unread() {
    assert_eq!(RULINGS_ARGS, ["check", "--emit-rulings", "--dir"]);
    assert_eq!(
        Design::new("/r", "folio").rulings_args(),
        ["check", "--emit-rulings", "--dir", "/r/design-intent"].map(OsString::from)
    );
    let design = Design::new(std::env::temp_dir(), "/nonexistent/tz-no-such-folio");
    assert_eq!(design.rulings(), None);
}

#[test]
fn g3g7_check_names_the_record() {
    let place = Place::new("names", &read("ledger.json"), &read("rulings.jsonl"), 0);
    let out = place.check(true);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(stdout(&out), summary("まだ分からない", 0, 1));
    assert_eq!(stderr(&out), format!("{}\n", unknown("g-9", NO_TOOL)));

    let place = Place::new("names-cut", &cut(), &read("rulings.jsonl"), 0);
    let out = place.check(true);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(
        stdout(&out),
        format!(
            "[g-3] 違反 1（ADR-7） next={}\n{}",
            next("g-3"),
            summary("不合格", 1, 1)
        )
    );
}

#[test]
fn g3g7_check_without_ledger_is_two() {
    let place = Place::new("no-bd", &cut(), &read("rulings.jsonl"), 0);
    let out = place.check(false);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let err = stderr(&out);
    assert!(
        err.lines()
            .any(|l| l.starts_with("# 読めない出所") && l.contains("台帳")),
        "{err}"
    );
    for id in ["g-3", "g-7"] {
        assert_eq!(lines_of(&err, &unknown(id, UNREAD)), 1, "{id}: {err}");
    }
    assert!(!stdout(&out).contains("[g-3]"), "{out:?}");
}

#[test]
fn g3g7_failed_floor_is_unknown() {
    let place = Place::new("floor", &plus_user(&cut()), &read("rulings.jsonl"), 1);
    let out = place.check(true);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(stdout(&out), summary("まだ分からない", 0, 3));
    let err = stderr(&out);
    for id in ["g-3", "g-7"] {
        assert_eq!(lines_of(&err, &unknown(id, UNREAD)), 1, "{id}: {err}");
    }
    assert!(!err.contains("# 読めない出所"), "{err}");
}

#[test]
fn g3g7_doc_and_view_carry_edges() {
    let (index, ledger, events, rulings) = (
        read("index.tsv"),
        read("ledger.json"),
        read("events.jsonl"),
        read("rulings.jsonl"),
    );
    let texts = Texts {
        design: index.clone(),
        ledger: ledger.clone(),
        events: events.clone(),
        rulings: rulings.clone(),
        summary: String::new(),
    };
    let doc = board::graph(&texts);
    let mut g = graph::build(&Inputs {
        design_index: &index,
        ledger: &ledger,
        events: &events,
    });
    assert!(graph::build::add_rulings(&mut g, &rulings));
    assert_eq!(doc.edges, g.edges);
    let ruled = |edges: &[tsuzuri_contract::graph::GraphEdge]| {
        edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::RuledBy)
            .count()
    };
    assert_eq!(ruled(&doc.edges), 10);
    let verdict = |doc: &tsuzuri_contract::graph::GraphDoc, id: &str| {
        doc.invariants
            .iter()
            .find(|i| i.id == id)
            .map(|i| i.verdict)
            .expect("不変条件")
    };
    for id in ["g-3", "g-7"] {
        assert_eq!(verdict(&doc, id), Verdict::Pass, "{id}");
    }
    let view = board::view(&texts);
    assert!(view.edges.iter().any(|e| e.edge_type == EdgeType::RuledBy));

    let empty = Texts {
        rulings: String::new(),
        ..texts
    };
    let doc = board::graph(&empty);
    assert_eq!(ruled(&doc.edges), 0);
    assert!(
        board::view(&empty)
            .edges
            .iter()
            .all(|e| e.edge_type != EdgeType::RuledBy)
    );
    for id in ["g-3", "g-7"] {
        assert_eq!(verdict(&doc, id), Verdict::Unknown, "{id}");
    }
}

#[test]
fn g3g7_check_counts_outside_ids() {
    let outside = unknown("g-7", "folio の裁定 id の文法の外の id の裁定 1");
    let user = plus_user(&read("ledger.json"));
    let place = Place::new("outside", &user, &read("rulings.jsonl"), 0);
    let out = place.check(true);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(stdout(&out), summary("まだ分からない", 0, 2));
    assert_eq!(
        stderr(&out),
        format!("{outside}\n{}\n", unknown("g-9", NO_TOOL))
    );

    let far = plus_adr8(&read("rulings.jsonl"), "s9-far.2");
    let place = Place::new("outside-far", &user, &far, 0);
    let out = place.check(true);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(stdout(&out), summary("まだ分からない", 0, 3));
    assert_eq!(
        stderr(&out),
        format!(
            "{}\n{outside}\n{}\n",
            unknown("g-3", UNREAD),
            unknown("g-9", NO_TOOL)
        )
    );

    let place = Place::new("outside-memo", &memo(), &read("rulings.jsonl"), 0);
    let out = place.check(true);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(
        stdout(&out),
        format!(
            "[g-7] 違反 1（{MEMO_RULING}） next={}\n{}",
            next("g-7"),
            summary("不合格", 1, 1)
        )
    );
    assert_eq!(stderr(&out), format!("{}\n", unknown("g-9", NO_TOOL)));
}

/// 計画の verify の filter の語（歯の名が含んではならない部分の字）。
const WORDS: &[&str] = &[
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
    "afocus_",
    "aord_",
    "apop_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadopt_",
    "cdorm_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "denv_",
    "dnedge_",
    "dnrow_",
    "dnskip_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "gapspage_",
    "gbnote_",
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
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lidle_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mstore_",
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
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
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
    "stcli_",
    "steady_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "topbar_",
    "tz_",
    "urpanel_",
    "uword_",
    "wstrip_",
    "pgz_",
];

#[test]
fn g3g7_own_names_clean() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut names: Vec<String> = Vec::new();
    for file in [
        dir.join("tests/g3g7w.rs"),
        dir.join("../tsuzuri-core/tests/g3g7.rs"),
    ] {
        let text = fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("{} を読む: {e}", file.display()));
        let lines: Vec<&str> = text.lines().collect();
        names.extend(
            lines
                .windows(2)
                .filter(|w| w[0].trim() == "#[test]")
                .map(|w| {
                    let rest = w[1].trim().strip_prefix("fn ").expect("fn の行");
                    rest.split('(').next().expect("fn の名").to_string()
                }),
        );
    }
    assert!(names.len() >= 13, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("g3g7_")
            .unwrap_or_else(|| panic!("{name} は g3g7_ で始まらない"));
        for word in WORDS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
