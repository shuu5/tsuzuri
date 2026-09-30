//! 問いの起票の門の口の歯（行 f-gate・tz hook question-gate・接頭辞 qgate_）。
//! 偽の bd は受けた引数を記録の置き場に 1 行足してから作業場の ledger.json（fixture の写し）を出す script、
//! 偽の設計の道具は同じく引数を記録してから作業場の index.tsv を出す script（写しを消せば rc 1 で落ちる）。
//! 境界の歯は JSON を読まないので、tz の標準出力の字を、同じ字から中核の関数で組んだ値の字と比べる。

use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_boundary::hook::question_gate::{self, Args, USAGE};
use tsuzuri_boundary::server::design::{DESIGN_DIR, FOLIO_ARGS};
use tsuzuri_boundary::server::ledger::{BD, BD_ARGS};
use tsuzuri_core::gate::{self, Gate, Why};
use tsuzuri_core::graph::{Graph, Inputs, Source, build};

const INDEX: &str = "guard/question-4/index.tsv";
const LEDGER: &str = "guard/question-4/ledger.json";

/// 組 one-undisposed と all-disposed の not-relevant（cases.json と同じ字）。
const NR_THREE: &str = r#""P-14":"問いは判断の代行に触れない","R-1":"規則の行の値は変えない","FR3":"数え方は変えない""#;
const NR_ADR2: &str = r#""ADR-2":"面の技術の決めは先送りのまま""#;

/// 行 f-gate の外で決めた filter の語（この行の接頭辞 qgate_ は並べない）。
const FILTER_WORDS: [&str; 113] = [
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
    "wstrip_",
    "flight_",
    "qkey_",
    "stage_term_",
    "stage_cdp_",
    "pwhole_",
];

type Parse = fn(&[&str]) -> Result<Args, String>;
type GraphOf = fn(&Args) -> Graph;
type Answer = fn(&Args, &str) -> Option<String>;
type Run = fn(&[&str]) -> u8;

const PARSE: Parse = question_gate::parse;
const GRAPH: GraphOf = question_gate::graph;
const ANSWER: Answer = question_gate::answer;
const RUN: Run = question_gate::run;

fn read_fixture(rel: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn graph_of(index: &str, ledger: &str) -> Graph {
    build(&Inputs {
        design_index: index,
        ledger,
        events: "",
    })
}

fn fixture_graph() -> Graph {
    graph_of(&read_fixture(INDEX), &read_fixture(LEDGER))
}

fn fr4() -> Vec<String> {
    vec!["FR4".to_string()]
}

/// JSON の字の中身（二重引用符と逆斜線に逆斜線を付ける）。
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Bash の PreToolUse の hook の入力の字。
fn payload(command: &str) -> String {
    format!(
        r#"{{"session_id":"s-1","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":"{}"}}}}"#,
        escape(command)
    )
}

/// 問いの起票の command（not-relevant の中身と digest）。
fn question_command(not_relevant: &str, digest: &str) -> String {
    let meta = format!(
        r#"{{"touches":["FR4"],"not-relevant":{{{not_relevant}}},"digest":"{digest}"}}"#
    );
    format!("bdw create --parent=fx-q --labels=intake:question --metadata='{meta}' 門の歯の問い")
}

/// 問いの起票の payload（not-relevant の中身と digest）。
fn question(not_relevant: &str, digest: &str) -> String {
    payload(&question_command(not_relevant, digest))
}

/// 組 one-undisposed の command（digest は今の束の要約値）。
fn one_undisposed_command() -> String {
    question_command(NR_THREE, &gate::bundle_digest(&fixture_graph(), &fr4()))
}

/// 組 one-undisposed の payload。
fn one_undisposed() -> String {
    payload(&one_undisposed_command())
}

/// 組 all-disposed の payload（digest は今の束の要約値）。
fn all_disposed() -> String {
    question(
        &format!("{NR_THREE},{NR_ADR2}"),
        &gate::bundle_digest(&fixture_graph(), &fr4()),
    )
}

/// payload を `graph` で判じた答えの字と改行 1 つ（通すなら空の字）。
fn expected(payload: &str, graph: &Graph) -> String {
    gate::output(&gate::judge(&gate::drafts(payload), graph))
        .map(|t| format!("{t}\n"))
        .unwrap_or_default()
}

/// 問いの起票でない 6 つの payload。
fn others() -> Vec<String> {
    let command = escape(&one_undisposed_command());
    vec![
        format!(r#"{{"tool_name":"Edit","tool_input":{{"command":"{command}"}}}}"#),
        "not json".to_string(),
        payload("ls -la"),
        payload("bdw create --parent=fx-q --labels=intake:memo --metadata='{}' 門の歯の問い"),
        payload("bdw update fx-q.1 --add-label=intake:question"),
        payload("echo bdw create --labels=intake:question"),
    ]
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// drop で path を消す守り（dir なら中身ごと・file なら file を・誤りは捨てる）。
struct Tidy(PathBuf);

impl Drop for Tidy {
    fn drop(&mut self) {
        let _ = match fs::symlink_metadata(&self.0) {
            Ok(meta) if meta.is_dir() => fs::remove_dir_all(&self.0),
            _ => fs::remove_file(&self.0),
        };
    }
}

/// 歯ごとの作業場（.git が file の repo・記録の置き場・偽の bd と偽の設計の道具と、それらが出す字の写し）。
/// 作業場が drop されると、守りが repo の .git を消す（target/tmp に .git を残さない）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    log: PathBuf,
    _git: Tidy,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("qgate")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, log) = (root.join("repo"), root.join("log"));
        fs::create_dir_all(&repo).expect("repo の置き場");
        fs::create_dir_all(&log).expect("記録の置き場");
        let git = repo.join(".git");
        let tidy = Tidy(git.clone());
        fs::write(&git, "gitdir: /nonexistent/qgate\n").expect(".git の file");
        fs::write(root.join("ledger.json"), read_fixture(LEDGER)).expect("台帳の写し");
        fs::write(root.join("index.tsv"), read_fixture(INDEX)).expect("索引の写し");
        for (program, out) in [("bd", "ledger.json"), ("folio", "index.tsv")] {
            script(
                &root.join(program),
                &format!(
                    "printf '%s\\n' \"$*\" >> '{}'\nexec cat '{}'",
                    log.join(format!("{program}.log")).display(),
                    root.join(out).display()
                ),
            );
        }
        Place { root, repo, log, _git: tidy }
    }

    /// 偽の program を落とす（出す file が無いので rc 1）。
    fn fails(&self, out: &str) {
        fs::remove_file(self.root.join(out)).expect("出す file を消す");
    }

    /// 偽の program が撃たれた回ごとの引数の行。
    fn calls(&self, program: &str) -> Vec<String> {
        fs::read_to_string(self.log.join(format!("{program}.log")))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn program(&self, name: &str) -> OsString {
        self.root.join(name).into()
    }

    /// hook question-gate に --repo と偽の --bd と --folio を渡す引数。
    fn hook_args(&self) -> Vec<OsString> {
        vec![
            "hook".into(),
            "question-gate".into(),
            "--repo".into(),
            self.repo.clone().into(),
            "--bd".into(),
            self.program("bd"),
            "--folio".into(),
            self.program("folio"),
        ]
    }

    /// tz を撃ち、標準入力に payload を書いて閉じ、終わりまで待つ。
    fn tz(&self, args: &[OsString], payload: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("tz を撃つ");
        let mut stdin = child.stdin.take().expect("標準入力");
        let _ = stdin.write_all(payload.as_bytes());
        drop(stdin);
        child.wait_with_output().expect("tz の終わり")
    }
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

#[test]
fn qgate_parse_args() {
    assert!(USAGE.contains("tz hook question-gate"), "{USAGE}");
    assert_eq!(
        PARSE(&["--repo", "/r"]),
        Ok(Args {
            repo: PathBuf::from("/r"),
            bd: OsString::from(BD),
            folio: tsuzuri_boundary::cli::folio::program(None),
        })
    );
    let want = Ok(Args {
        repo: PathBuf::from("/r"),
        bd: OsString::from("b"),
        folio: OsString::from("f"),
    });
    assert_eq!(PARSE(&["--repo", "/r", "--bd", "b", "--folio", "f"]), want);
    assert_eq!(PARSE(&["--repo=/r", "--bd=b", "--folio=f"]), want);
    for bad in [
        &[][..],
        &["--bd", "b"],
        &["--repo", "/r", "--nope", "x"],
        &["--repo", "/r", "--repo", "/s"],
        &["--repo", "/r", "--folio", "f", "--folio=g"],
        &["--repo"],
        &["--repo="],
        &["--repo", "/r", "--bd="],
        &["--repo", "/r", "--folio", ""],
    ] {
        assert!(PARSE(bad).is_err(), "{bad:?}");
    }
    // 撃てない program では両方の出所が読めず、下書きの無い入力は何も撃たずに None。
    let none = Args {
        repo: std::env::temp_dir(),
        bd: "/nonexistent/tz-no-such-bd".into(),
        folio: "/nonexistent/tz-no-such-folio".into(),
    };
    assert_eq!(GRAPH(&none).unread, Source::ALL);
    assert_eq!(ANSWER(&none, "not json"), None);
    // 使い方の誤りで下書きの無い入力（歯の標準入力は空）は 1。
    assert_eq!(RUN(&[]), 1);
}

#[test]
fn qgate_bin_names_undisposed() {
    let place = Place::new("undisposed");
    let before = tree(&place.repo);
    let p = one_undisposed();
    let out = place.tz(&place.hook_args(), &p);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let want = expected(&p, &fixture_graph());
    assert!(want.contains("undisposed"), "{want}");
    assert_eq!(text(&out.stdout), want);
    assert_eq!(place.calls("bd"), [BD_ARGS.join(" ")]);
    assert_eq!(
        place.calls("folio"),
        [format!(
            "{} {}",
            FOLIO_ARGS.join(" "),
            place.repo.join(DESIGN_DIR).display()
        )]
    );
    assert!(before == tree(&place.repo), "repo の byte が変わる");
    // 作業場の drop で守りが .git を消す。
    let git = place.repo.join(".git");
    drop(place);
    assert!(fs::symlink_metadata(&git).is_err(), "drop の後に .git が残る");
    // 作業場を持ったままの panic でも、巻き戻しで .git が消える。
    let unwound = std::panic::catch_unwind(|| {
        let place = Place::new("undisposed");
        assert!(place.repo.join(".git").is_file(), ".git の file");
        panic!("作業場を持ったまま巻き戻す");
    });
    assert!(unwound.is_err());
    assert!(fs::symlink_metadata(&git).is_err(), "巻き戻しの後に .git が残る");
    // 守りは .git を書く前に束ねる（組みの途中の panic でも消す）。
    let own = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/qgate.rs"))
        .expect("歯の file");
    let body = own.split("impl Place {").nth(1).expect("Place の impl");
    let (bind, write) = (body.find("Tidy(git.clone())"), body.find("fs::write(&git"));
    assert!(bind.is_some() && write.is_some() && bind < write, "Tidy を .git の書きより前に束ねる");
}

#[test]
fn qgate_bin_silent_when_disposed() {
    let place = Place::new("disposed");
    let p = all_disposed();
    assert_eq!(expected(&p, &fixture_graph()), "");
    let out = place.tz(&place.hook_args(), &p);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
}

#[test]
fn qgate_bin_other_calls_spawn_nothing() {
    let place = Place::new("others");
    for p in others() {
        assert!(gate::drafts(&p).is_empty(), "{p}");
        let out = place.tz(&place.hook_args(), &p);
        assert_eq!(out.status.code(), Some(0), "{p}: {out:?}");
        assert!(out.stdout.is_empty(), "{p}: {out:?}");
        assert!(place.calls("bd").is_empty(), "{p}: 偽の bd を撃つ");
        assert!(place.calls("folio").is_empty(), "{p}: 偽の設計の道具を撃つ");
    }
}

#[test]
fn qgate_bin_unread_is_unknown() {
    let p = one_undisposed();
    let index = read_fixture(INDEX);
    let ledger = read_fixture(LEDGER);
    for (name, out_file, graph, id) in [
        ("bd-fails", "ledger.json", graph_of(&index, ""), "ledger"),
        ("folio-fails", "index.tsv", graph_of("", &ledger), "design"),
    ] {
        let place = Place::new(name);
        place.fails(out_file);
        assert_eq!(
            gate::judge(&gate::drafts(&p), &graph),
            Gate::Unknown {
                why: Why::Unread,
                ids: vec![id.to_string()],
                digest: None,
            }
        );
        let out = place.tz(&place.hook_args(), &p);
        assert_eq!(out.status.code(), Some(0), "{name}: {out:?}");
        assert_eq!(text(&out.stdout), expected(&p, &graph), "{name}");
    }
}

#[test]
fn qgate_bad_args_close_drafts_only() {
    let place = Place::new("bad-args");
    let file = place.root.join("file.txt");
    fs::write(&file, "x").expect("dir でない file");
    let head = || -> Vec<OsString> { vec!["hook".into(), "question-gate".into()] };
    let with = |rest: Vec<OsString>| -> Vec<OsString> {
        let mut args = head();
        args.extend(rest);
        args
    };
    let repo: OsString = place.repo.clone().into();
    let (bd, folio) = (place.program("bd"), place.program("folio"));
    let cases: Vec<Vec<OsString>> = vec![
        with(vec!["--bd".into(), bd.clone(), "--folio".into(), folio.clone()]),
        with(vec![
            "--repo".into(),
            repo.clone(),
            "--bd=".into(),
            "--folio".into(),
            folio.clone(),
        ]),
        with(vec![
            "--repo".into(),
            repo.clone(),
            "--bd".into(),
            bd.clone(),
            "--folio".into(),
            folio.clone(),
            "--nope".into(),
            "x".into(),
        ]),
        with(vec![
            "--repo".into(),
            repo.clone(),
            "--repo".into(),
            repo,
            "--bd".into(),
            bd.clone(),
            "--folio".into(),
            folio.clone(),
        ]),
        with(vec![
            "--repo".into(),
            file.into(),
            "--bd".into(),
            bd,
            "--folio".into(),
            folio,
        ]),
    ];
    let deny = gate::output(&Gate::Deny {
        why: Why::Args,
        ids: Vec::new(),
        digest: None,
    })
    .expect("args の答え");
    let q = one_undisposed();
    let ls = payload("ls -la");
    for args in &cases {
        let out = place.tz(args, &q);
        assert_eq!(out.status.code(), Some(0), "{args:?}: {out:?}");
        assert_eq!(text(&out.stdout), format!("{deny}\n"), "{args:?}");
        assert!(text(&out.stderr).contains("tz hook question-gate"), "{args:?}: {out:?}");

        let out = place.tz(args, &ls);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{args:?}: {out:?}");
        assert!(text(&out.stderr).contains("tz hook question-gate"), "{args:?}: {out:?}");

        assert!(place.calls("bd").is_empty(), "{args:?}: 偽の bd を撃つ");
        assert!(place.calls("folio").is_empty(), "{args:?}: 偽の設計の道具を撃つ");
    }
}

#[test]
fn qgate_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/qgate.rs");
    let src = fs::read_to_string(&path).expect("tests/qgate.rs を読む");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 7, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("qgate_")
            .unwrap_or_else(|| panic!("{name} は qgate_ で始まらない"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
