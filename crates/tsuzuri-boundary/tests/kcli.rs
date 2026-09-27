//! tz graph の歯（接頭辞 kcli_・設計ノート surface-wave4 行 k-graph の完了の条件）。
//! 偽の bd（台帳の字の file を返す script）と偽の設計の道具（索引の字の file を返す script）を歯ごとの
//! 作業場に置き、event log の字を作業場の state dir に置いて、tz の binary を撃つ。

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsuzuri_boundary::cli::graph;
use tsuzuri_boundary::server::board::{self, Texts};
use tsuzuri_contract::graph::{EdgeType, GraphDoc, GraphSource, InvariantCheck, Verdict};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::build::DESIGN_EDGE_TYPES;
use tsuzuri_core::graph::check::{INVARIANTS, UNMEASURED};

/// 設計の索引（節点 3・辺 3）。
const INDEX: &str = "# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）\n\
A-1\t条\tconstitution.yaml\t00000000\t見本の条\n\
A-1.1\t規範文\tconstitution.yaml\t00000001\t見本の規範文\n\
FR1\t要件\tsrs.yaml\t00000002\t見本の要件\n\
# 辺（1 行 = 端 / 端 / 型・タブ区切り）\n\
A-1\tA-1.1\tin-article\n\
A-1.1\tA-1\tin-article\n\
FR1\tA-1\trefs\n";

/// 台帳（bead 3 本・notes は無い）。
const LEDGER: &str = r#"[
{"id": "k", "title": "根", "status": "open", "issue_type": "epic"},
{"id": "k.1", "title": "契約", "status": "open", "issue_type": "task", "acceptance_criteria": "design = contracts/x.toml#a",
 "dependencies": [{"issue_id": "k.1", "depends_on_id": "k", "type": "parent-child"}]},
{"id": "k.2", "title": "問い", "status": "open", "issue_type": "task", "labels": ["intake:question"],
 "metadata": {"touches": ["FR1"]},
 "dependencies": [{"issue_id": "k.2", "depends_on_id": "k", "type": "parent-child"}]}
]
"#;

/// 台帳から k.1 の欄 acceptance_criteria だけを除いた字（k.1 の pointer の行が 0）。
const LEDGER_CUT: &str = r#"[
{"id": "k", "title": "根", "status": "open", "issue_type": "epic"},
{"id": "k.1", "title": "契約", "status": "open", "issue_type": "task",
 "dependencies": [{"issue_id": "k.1", "depends_on_id": "k", "type": "parent-child"}]},
{"id": "k.2", "title": "問い", "status": "open", "issue_type": "task", "labels": ["intake:question"],
 "metadata": {"touches": ["FR1"]},
 "dependencies": [{"issue_id": "k.2", "depends_on_id": "k", "type": "parent-child"}]}
]
"#;

/// event log（走行 1 本）。
const EVENTS: &str = "{\"schema\":1,\"ts\":\"2026-09-27T00:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"k.1-20260927T000000Z\",\"bead\":\"k.1\",\"stage\":\"Intake\"}\n";

/// 着地済みの行と第 3 波と第 4 波の行の verify の語（歯の名が含んではならない部分の字）。
const WORDS: [&str; 69] = [
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
    "qgate_",
    "nsum_",
    "hcard_",
];

/// 歯ごとの作業場（repo の置き場・state dir・字の file・偽の bd と偽の設計の道具）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 撃つときの出所の置き方。
#[derive(Clone, Copy)]
struct Opts {
    /// bd の program が在る（false なら無い path）。
    bd: bool,
    /// 設計の道具の program が在る（false なら無い path）。
    folio: bool,
    /// --state-dir を渡す。
    state: bool,
}

const ALL: Opts = Opts {
    bd: true,
    folio: true,
    state: true,
};

impl Place {
    fn new(name: &str, ledger: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("kcli")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let state = root.join("state");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::write(repo.join(".beads/issues.jsonl"), "{}\n").expect("印の file");
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::write(state.join("fleet/events.jsonl"), EVENTS).expect("event log");
        fs::write(root.join("ledger.json"), ledger).expect("台帳の字");
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

    /// tz graph を撃つ（`head` は --check・--design などの前の引数）。
    fn tz(&self, head: &[&str], opts: Opts) -> Output {
        let mut args: Vec<OsString> = vec!["graph".into()];
        args.extend(head.iter().map(OsString::from));
        args.push("--repo".into());
        args.push(self.repo.clone().into());
        args.push("--bd".into());
        args.push(self.program("bd", opts.bd).into());
        args.push(format!("--folio={}", self.program("folio", opts.folio).display()).into());
        if opts.state {
            args.push("--state-dir".into());
            args.push(self.state.clone().into());
        }
        Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .output()
            .expect("tz を撃つ")
    }

    fn program(&self, name: &str, present: bool) -> PathBuf {
        if present {
            self.root.join(name)
        } else {
            self.root.join(format!("no-such-{name}"))
        }
    }
}

fn texts(ledger: &str) -> Texts {
    Texts {
        design: INDEX.to_string(),
        ledger: ledger.to_string(),
        events: EVENTS.to_string(),
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("標準出力の字")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("標準エラーの字")
}

fn decode(out: &Output) -> GraphDoc {
    let text = stdout(out);
    wire::decode(&text).unwrap_or_else(|e| panic!("GraphDoc の形でない {e}: {text}"))
}

/// dir の中の file の path と byte の一覧（書かれていないことを比べる）。
fn tree(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).expect("dir を読む") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                out.push((path.clone(), Vec::new()));
                stack.push(path);
            } else {
                out.push((path.clone(), fs::read(&path).expect("file を読む")));
            }
        }
    }
    out.sort();
    out
}

fn check(id: &str, verdict: Verdict) -> InvariantCheck {
    let ids: Vec<String> = match verdict {
        Verdict::Violation => vec!["k.1".to_string()],
        _ => Vec::new(),
    };
    InvariantCheck {
        id: id.to_string(),
        verdict,
        violations: ids.len() as u32,
        ids,
    }
}

/// 12 本の列（`set` に挙げた id だけ verdict を替え、ほかは合格）。
fn checks(set: &[(&str, Verdict)]) -> Vec<InvariantCheck> {
    INVARIANTS
        .iter()
        .map(|id| {
            let verdict = set
                .iter()
                .find(|(s, _)| s == id)
                .map_or(Verdict::Pass, |(_, v)| *v);
            check(id, verdict)
        })
        .collect()
}

#[test]
fn kcli_next_follows_invariants() {
    let next: [(&str, &str); 12] = graph::NEXT;
    let ids: Vec<&str> = next.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, INVARIANTS.to_vec());
    for (id, text) in next {
        assert!(!text.is_empty(), "{id}");
        assert!(!text.contains('\n'), "{id}");
    }
    let run: fn(&[&str]) -> u8 = graph::run;
    let view: fn(&GraphDoc) -> GraphDoc = graph::design_view;
    let _ = (run, view);
}

#[test]
fn kcli_overall_and_codes() {
    let overall: fn(&[InvariantCheck]) -> Verdict = graph::overall;
    let exit_code: fn(Verdict) -> u8 = graph::exit_code;
    assert_eq!(overall(&checks(&[])), Verdict::Pass);
    assert_eq!(
        overall(&checks(&[
            ("g-2", Verdict::Violation),
            ("g-3", Verdict::Unknown)
        ])),
        Verdict::Violation
    );
    assert_eq!(
        overall(&checks(&[("g-3", Verdict::Unknown)])),
        Verdict::Unknown
    );
    assert_eq!(overall(&[]), Verdict::Unknown);
    assert_eq!(
        Verdict::ALL.map(exit_code),
        [0, 1, 2],
        "合格・不合格・まだ分からない"
    );
}

#[test]
fn kcli_prints_same_doc_as_board() {
    let place = Place::new("same", LEDGER);
    let before = (tree(&place.repo), tree(&place.state));
    let out = place.tz(&[], ALL);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let doc = decode(&out);
    assert_eq!(doc, board::graph(&texts(LEDGER)));
    assert!(doc.unread.is_empty(), "{:?}", doc.unread);
    assert_eq!((doc.nodes.len(), doc.edges.len()), (7, 7));

    let out = place.tz(
        &[],
        Opts {
            state: false,
            ..ALL
        },
    );
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(decode(&out).unread, vec![GraphSource::Runs]);
    assert!(
        before == (tree(&place.repo), tree(&place.state)),
        "tz graph の後に repo か state dir の byte が変わる"
    );
}

#[test]
fn kcli_check_all_read_is_unknown() {
    let place = Place::new("check-all", LEDGER);
    let out = place.tz(&["--check"], ALL);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(
        stdout(&out),
        "tz graph --check: まだ分からない（違反 0・まだ分からない 3）\n"
    );
    let err = stderr(&out);
    for id in UNMEASURED {
        let line = format!("# まだ分からない: [{id}] 測る機構がまだ無い");
        assert_eq!(err.lines().filter(|l| *l == line).count(), 1, "{err}");
    }
    assert!(!err.contains("# 読めない出所"), "{err}");
}

#[test]
fn kcli_check_cut_line_names_k1() {
    let place = Place::new("check-cut", LEDGER_CUT);
    let out = place.tz(&["--check"], ALL);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let next = graph::NEXT
        .iter()
        .find(|(id, _)| *id == "g-2")
        .map(|(_, n)| *n)
        .expect("g-2 の次の 1 手");
    assert_eq!(
        stdout(&out),
        format!(
            "[g-2] 違反 1（k.1） next={next}\ntz graph --check: 不合格（違反 1・まだ分からない 3）\n"
        )
    );
}

#[test]
fn kcli_check_no_ledger_is_two() {
    let place = Place::new("check-no-bd", LEDGER);
    let out = place.tz(&["--check"], Opts { bd: false, ..ALL });
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(
        stdout(&out),
        "tz graph --check: まだ分からない（違反 0・まだ分からない 11）\n"
    );
    let err = stderr(&out);
    assert!(err.lines().any(|l| l == "# 読めない出所: 台帳"), "{err}");
    assert!(
        err.lines()
            .any(|l| l == "# まだ分からない: [g-2] 読めない出所が在る"),
        "{err}"
    );
}

#[test]
fn kcli_check_no_state_is_two() {
    let place = Place::new("check-no-state", LEDGER);
    let out = place.tz(
        &["--check"],
        Opts {
            state: false,
            ..ALL
        },
    );
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(
        stdout(&out),
        "tz graph --check: まだ分からない（違反 0・まだ分からない 6）\n"
    );
    let err = stderr(&out);
    let unread: Vec<&str> = err
        .lines()
        .filter(|l| l.starts_with("# 読めない出所"))
        .collect();
    assert_eq!(unread, vec!["# 読めない出所: 走行の記録"], "{err}");
}

#[test]
fn kcli_design_keeps_index_only() {
    let place = Place::new("design", LEDGER);
    let out = place.tz(
        &["--design"],
        Opts {
            bd: false,
            state: false,
            ..ALL
        },
    );
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let doc = decode(&out);
    let ids: Vec<&str> = doc.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, vec!["A-1", "A-1.1", "FR1"]);
    let kinds = tsuzuri_core::graph::Source::Design.kinds();
    assert!(doc.nodes.iter().all(|n| kinds.contains(&n.kind)));
    assert_eq!(doc.edges.len(), 3);
    let types = &EdgeType::ALL[..DESIGN_EDGE_TYPES];
    assert!(doc.edges.iter().all(|e| types.contains(&e.edge_type)));
    assert!(doc.beads.is_empty() && doc.runs.is_empty());
    assert!(doc.invariants.is_empty() && doc.unread.is_empty());
    assert_eq!(doc, graph::design_view(&board::graph(&texts(LEDGER))));
    let full = place.tz(&["--design"], ALL);
    assert_eq!(full.status.code(), Some(0), "{full:?}");
    assert_eq!(out.stdout, full.stdout);
}

#[test]
fn kcli_design_without_folio_is_two() {
    let place = Place::new("design-no-folio", LEDGER);
    let out = place.tz(
        &["--design"],
        Opts {
            folio: false,
            ..ALL
        },
    );
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let doc = decode(&out);
    assert_eq!(doc.unread, vec![GraphSource::Design]);
    assert!(doc.nodes.is_empty(), "{:?}", doc.nodes);
}

#[test]
fn kcli_usage_errors_are_one() {
    let place = Place::new("usage", LEDGER);
    let file = place.root.join("index.tsv");
    let cases: [Vec<OsString>; 4] = [
        vec!["--check".into(), "--design".into()],
        vec!["--nope".into()],
        vec!["--bd=".into()],
        vec!["--repo".into(), file.into()],
    ];
    for extra in cases {
        let out = Command::new(env!("CARGO_BIN_EXE_tz"))
            .arg("graph")
            .args(&extra)
            .output()
            .expect("tz を撃つ");
        assert_eq!(out.status.code(), Some(1), "{extra:?}: {out:?}");
        assert!(stderr(&out).contains("tz graph"), "{extra:?}: {out:?}");
    }
}

#[test]
fn kcli_own_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/kcli.rs"))
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
    assert!(names.len() >= 11, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("kcli_")
            .unwrap_or_else(|| panic!("{name} は kcli_ で始まらない"));
        for word in WORDS {
            assert!(!rest.contains(word), "{name} は {word} を含む");
        }
    }
}
