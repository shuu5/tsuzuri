//! 席の停止の hook の歯（行 f-stop・接頭辞 fstop_）。
//! 偽の bd は受けた引数を記録の置き場に 1 行ずつ足してから作業場の out.json（既定は fixture の ledger.json）を出す script、
//! 偽の bdw は撃たれた回ごとの argv と cwd を記録の置き場に書く script（server_ask.rs の recorder と同じ形）。
//! tz は作業場の根（偽の bd と偽の bdw の在る dir）を PATH の頭に足して撃つ。
//! 偽の git は落ちる（鍵 tsuzuri.draftsdir の無い repo・相談の拾い〔行 cs-hooks〕は何も足さない）。
#![cfg(test)]

use std::ffi::OsString;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use crate::common::{Run, Tidy, now, read_fixture, script, text, tree};
use tsuzuri_boundary::hook::stop::{self, Args, MARK_BUDGET, USAGE};
use tsuzuri_boundary::server::{ledger, ruling};
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::ledger::{BDW, BeadId, LedgerWrite};
use tsuzuri_contract::surface::RulingId;
use tsuzuri_core::delivery::{self, Pending, Route};

const LEDGER: &str = "stop/ledger.json";
const LEDGER_DONE: &str = "stop/ledger-done.json";

const P1: &str = r#"{"session_id":"s-1","transcript_path":"/tmp/t.jsonl","cwd":"/tmp","hook_event_name":"Stop","stop_hook_active":false}"#;
const P2: &str = "{\"session_id\":\"s-1\",\"transcript_path\":\"/tmp/t.jsonl\",\"cwd\":\"/tmp\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":true}\n";
const P5: &str = "";
const P6: &str = "not json";
const P7: &str = "[]";

/// 行 f-stop の外で決めた filter の語（この行の接頭辞 fstop_ は並べない）。
const FILTER_WORDS: [&str; 92] = [
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
    "hcard_",
    "qgate_",
    "nsum_",
    "nsumw_",
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
];

type Parse = fn(&[&str]) -> Result<Args, String>;
type Mark = fn(&Args, &[Pending], EpochSecs, Duration) -> usize;

const PARSE: Parse = stop::parse;
const MARK: Mark = stop::mark;
const RUN: Run = stop::run;

/// 撃たれた回ごとに argv と cwd を `<log>/<name>.<回>.args|cwd` に書き、`<log>/<name>.fail` の回なら rc 1 で終わる script
/// （`pause` は記録の前に撃つ字）。
fn recorder(path: &Path, log: &Path, name: &str, pause: &str) {
    let log = log.display();
    script(
        path,
        &format!(
            "{pause}\n\
             n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
             echo \"$n\" > '{log}/{name}.count'\n\
             for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'\n\
             pwd -P > '{log}/{name}.'\"$n\"'.cwd'\n\
             if [ \"$n\" = \"$(cat '{log}/{name}.fail' 2>/dev/null)\" ]; then echo 落ちた >&2; exit 1; fi\n\
             exit 0"
        ),
    );
}

/// 歯ごとの作業場（空の repo・記録の置き場・偽の bd と偽の bdw・out.json）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        Place::with_pause(name, ":")
    }

    fn with_pause(name: &str, pause: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("fstop")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, log) = (root.join("repo"), root.join("log"));
        fs::create_dir_all(&repo).expect("repo の置き場");
        fs::create_dir_all(&log).expect("記録の置き場");
        script(
            &root.join("bd"),
            &format!(
                "printf '%s\\n' \"$*\" >> '{}'\nexec cat '{}'",
                log.join("bd.log").display(),
                root.join("out.json").display()
            ),
        );
        recorder(&root.join("bdw"), &log, "bdw", pause);
        script(&root.join("git"), "exit 1");
        let place = Place { root, repo, log };
        place.bd_returns(&read_fixture(LEDGER));
        place
    }

    /// 偽の bd が返す字を置く。
    fn bd_returns(&self, text: &str) {
        fs::write(self.root.join("out.json"), text).expect("偽の bd の出力");
    }

    /// 偽の bd を落とす（出す file が無いので rc 1）。
    fn bd_fails(&self) {
        fs::remove_file(self.root.join("out.json")).expect("偽の bd の出力を消す");
    }

    /// 偽の bdw を `n` 回目で落とす。
    fn fail_at(&self, n: u32) {
        fs::write(self.log.join("bdw.fail"), n.to_string()).expect("落とす回");
    }

    /// 偽の bd が撃たれた回ごとの引数の行。
    fn bd_calls(&self) -> Vec<String> {
        fs::read_to_string(self.log.join("bd.log"))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// 偽の bdw が撃たれた回ごとの（argv・cwd）。
    fn bdw_calls(&self) -> Vec<(Vec<String>, String)> {
        let count: u32 = fs::read_to_string(self.log.join("bdw.count"))
            .map_or(0, |c| c.trim().parse().expect("回の数"));
        (1..=count)
            .map(|n| {
                let read = |ext: &str| {
                    fs::read_to_string(self.log.join(format!("bdw.{n}.{ext}"))).expect("記録")
                };
                (
                    read("args").lines().map(str::to_string).collect(),
                    read("cwd"),
                )
            })
            .collect()
    }

    fn bd(&self) -> PathBuf {
        self.root.join("bd")
    }

    fn bdw(&self) -> PathBuf {
        self.root.join("bdw")
    }

    fn args(&self) -> Args {
        Args {
            repo: self.repo.clone(),
            bd: self.bd().into(),
            bdw: self.bdw().into(),
        }
    }

    /// hook stop に --repo と偽の --bd と --bdw を渡す引数。
    fn hook_args(&self, repo: &Path) -> Vec<OsString> {
        vec![
            "hook".into(),
            "stop".into(),
            "--repo".into(),
            repo.into(),
            "--bd".into(),
            self.bd().into(),
            "--bdw".into(),
            self.bdw().into(),
        ]
    }

    /// 作業場の根を PATH の頭に足した tz の Command（標準入力・出力・エラーは pipe）。
    fn command(&self, args: &[OsString]) -> Command {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut dirs = vec![self.root.clone()];
        dirs.extend(std::env::split_paths(&path));
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tz"));
        cmd.args(args)
            .env("PATH", std::env::join_paths(dirs).expect("PATH"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd
    }

    /// tz を撃ち、標準入力に payload を書いて閉じ、終わりまで待つ（書きの誤りは捨てる）。
    fn tz(&self, args: &[OsString], payload: &str) -> Output {
        let mut child = self.command(args).spawn().expect("tz を撃つ");
        let mut stdin = child.stdin.take().expect("標準入力");
        let _ = stdin.write_all(payload.as_bytes());
        drop(stdin);
        child.wait_with_output().expect("tz の終わり")
    }

    fn repo_real(&self) -> String {
        format!(
            "{}\n",
            self.repo.canonicalize().expect("repo の実体").display()
        )
    }
}

fn pending(question: &str, id: &str) -> Pending {
    Pending {
        question: BeadId::new(question).expect("bead の id"),
        ruling: RulingId::new(id).expect("裁定の id"),
    }
}

/// 節の 3 つの Pending（台帳の配列の順）。
fn three() -> Vec<Pending> {
    vec![
        pending("fx-s.2", "fx-s.2:20260928T0101Z-1"),
        pending("fx-s.3", "fx-s.3:20260928T0102Z-1"),
        pending("fx-s.4", "fx-s.4:20260928T0105Z-1"),
    ]
}

/// 節の 3 つの Pending の答えと改行 1 つ。
fn answer() -> String {
    format!("{}\n", delivery::block(&three()).expect("答え"))
}

/// 印の書きの argv（その裁定・停止の経路・分）。
fn mark_argv(p: &Pending, minute: &str) -> Vec<String> {
    LedgerWrite::AppendNotes {
        id: p.question.clone(),
        line: delivery::mark_line(&p.ruling, Route::Stop, minute),
    }
    .argv()
}

#[test]
fn fstop_parse_args() {
    assert_eq!(
        USAGE,
        "usage: tz hook stop --repo <dir> [--bd <program>] [--bdw <program>]"
    );
    assert_eq!(MARK_BUDGET, Duration::from_secs(20));
    assert_eq!(
        PARSE(&["--repo", "/r"]),
        Ok(Args {
            repo: PathBuf::from("/r"),
            bd: OsString::from(ledger::BD),
            bdw: OsString::from(BDW),
        })
    );
    assert_eq!(
        PARSE(&["--repo=/r", "--bd", "b", "--bdw=w"]),
        Ok(Args {
            repo: PathBuf::from("/r"),
            bd: OsString::from("b"),
            bdw: OsString::from("w"),
        })
    );
    for bad in [
        &[][..],
        &["--bd", "b"],
        &["--repo", "/r", "--nope", "x"],
        &["--repo", "/r", "--repo", "/s"],
        &["--repo"],
        &["--repo="],
        &["--repo", "/r", "--bd="],
    ] {
        assert!(PARSE(bad).is_err(), "{bad:?}");
    }
    // 使い方の誤りは標準入力を読まずに 1。
    assert_eq!(RUN(&[]), 1);
}

#[test]
fn fstop_blocks_then_marks() {
    let place = Place::new("blocks");
    let before_tree = tree(&place.repo);
    let before = now();
    let out = place.tz(&place.hook_args(&place.repo), P1);
    let after = now();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(text(&out.stdout), answer());
    assert_eq!(place.bd_calls(), ["--readonly list --all --limit 0 --json"]);
    let calls = place.bdw_calls();
    assert_eq!(calls.len(), 3, "偽の bdw は 3 回: {calls:?}");
    let minutes = [ruling::minute(before), ruling::minute(after)];
    let minute = minutes
        .iter()
        .find(|m| mark_argv(&three()[0], m) == calls[0].0)
        .unwrap_or_else(|| panic!("1 回目の argv が印の書きでない: {:?}", calls[0].0));
    for (p, (argv, cwd)) in three().iter().zip(&calls) {
        assert_eq!(argv, &mark_argv(p, minute), "3 回の分は同じ");
        assert_eq!(cwd, &place.repo_real(), "cwd は repo の置き場");
    }
    assert!(before_tree == tree(&place.repo), "repo の byte が変わる");
}

#[test]
fn fstop_output_before_marks() {
    let place = Place::with_pause("before", "sleep 2");
    let mut child = place
        .command(&place.hook_args(&place.repo))
        .spawn()
        .expect("tz を撃つ");
    let mut stdin = child.stdin.take().expect("標準入力");
    let _ = stdin.write_all(P1.as_bytes());
    drop(stdin);
    let mut stdout = BufReader::new(child.stdout.take().expect("標準出力"));
    let mut first = String::new();
    stdout.read_line(&mut first).expect("最初の行");
    let seen = place.bdw_calls().len();
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).expect("残りの標準出力");
    let out = child.wait_with_output().expect("tz の終わり");
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(format!("{first}{rest}"), answer());
    assert_eq!(seen, 0, "答えを出す前に印を置く");
    assert_eq!(place.bdw_calls().len(), 3);
}

#[test]
fn fstop_quiet_when_continued() {
    let place = Place::new("continued");
    for payload in [P2, P5, P6, P7] {
        let out = place.tz(&place.hook_args(&place.repo), payload);
        assert_eq!(out.status.code(), Some(0), "{payload:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{payload:?}: {out:?}");
        assert!(place.bd_calls().is_empty(), "{payload:?}: 偽の bd を撃つ");
        assert!(place.bdw_calls().is_empty(), "{payload:?}: 偽の bdw を撃つ");
    }
}

#[test]
fn fstop_quiet_when_nothing_pending() {
    let place = Place::new("done");
    place.bd_returns(&read_fixture(LEDGER_DONE));
    let out = place.tz(&place.hook_args(&place.repo), P1);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert!(place.bdw_calls().is_empty(), "未配達が無いのに偽の bdw を撃つ");

    for (name, broken) in [("bd-fails", None), ("bd-not-json", Some("not json"))] {
        let place = Place::new(name);
        match broken {
            Some(t) => place.bd_returns(t),
            None => place.bd_fails(),
        }
        let out = place.tz(&place.hook_args(&place.repo), P1);
        assert_eq!(out.status.code(), Some(0), "{name}: {out:?}");
        assert!(out.stdout.is_empty(), "{name}: {out:?}");
        assert!(place.bdw_calls().is_empty(), "{name}: 偽の bdw を撃つ");
        assert!(
            text(&out.stderr).lines().any(|l| l.contains("tz hook stop")),
            "{name}: {out:?}"
        );
    }
}

#[test]
fn fstop_mark_failure_keeps_output() {
    let place = Place::new("mark-fail");
    place.fail_at(2);
    let out = place.tz(&place.hook_args(&place.repo), P1);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(text(&out.stdout), answer());
    assert_eq!(place.bdw_calls().len(), 3);
    let err = text(&out.stderr);
    let count = |p: &Pending| {
        err.lines()
            .filter(|l| l.contains(p.ruling.as_str()))
            .count()
    };
    let p = three();
    assert_eq!(count(&p[1]), 1, "{err}");
    assert_eq!(count(&p[0]), 0, "{err}");
    assert_eq!(count(&p[2]), 0, "{err}");
}

#[test]
fn fstop_budget_bounds_marks() {
    let place = Place::new("budget");
    let args = place.args();
    assert_eq!(MARK(&args, &three(), now(), Duration::ZERO), 0);
    assert!(place.bdw_calls().is_empty(), "上限 0 で偽の bdw を撃つ");
    assert_eq!(MARK(&args, &three(), now(), MARK_BUDGET), 3);
    assert_eq!(place.bdw_calls().len(), 3);
    assert!(MARK_BUDGET + ledger::BD_TIMEOUT < Duration::from_secs(30));
}

#[test]
fn fstop_usage_errors_are_one() {
    let place = Place::new("usage");
    let file = place.root.join("file.txt");
    fs::write(&file, "x").expect("dir でない file");
    let repo: OsString = place.repo.clone().into();
    let cases: Vec<Vec<OsString>> = vec![
        vec!["hook".into()],
        vec!["hook".into(), "nope".into()],
        vec!["hook".into(), "stop".into()],
        vec![
            "hook".into(),
            "stop".into(),
            "--repo".into(),
            repo.clone(),
            "--bd=".into(),
        ],
        vec![
            "hook".into(),
            "stop".into(),
            "--repo".into(),
            repo,
            "--nope".into(),
        ],
        vec![
            "hook".into(),
            "stop".into(),
            "--repo".into(),
            file.into(),
        ],
    ];
    for args in &cases {
        let out = place.tz(args, P1);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{args:?}: {out:?}");
        assert!(place.bd_calls().is_empty(), "{args:?}: 偽の bd を撃つ");
        assert!(place.bdw_calls().is_empty(), "{args:?}: 偽の bdw を撃つ");
        if args.len() >= 2 && args[1] == "stop" {
            assert!(
                text(&out.stderr).contains("tz hook stop"),
                "{args:?}: {out:?}"
            );
        }
    }
}

#[test]
fn fstop_worktree_stays_quiet() {
    let place = Place::new("worktree");
    let (wt, other) = (place.root.join("wt"), place.root.join("other"));
    fs::create_dir_all(&wt).expect("worktree の形の dir");
    fs::create_dir_all(&other).expect("別の dir");
    let git = wt.join(".git");
    let tidy = Tidy(git.clone());
    fs::write(&git,format!("gitdir: {}\n", other.display())).expect(".git の file");
    let out = place.tz(&place.hook_args(&wt), P1);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    assert!(place.bd_calls().is_empty(), "worktree で偽の bd を撃つ");
    assert!(place.bdw_calls().is_empty(), "worktree で偽の bdw を撃つ");

    fs::remove_file(&git).expect(".git の file を消す");
    fs::create_dir(&git).expect(".git の dir");
    let out = place.tz(&place.hook_args(&wt), P1);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(text(&out.stdout), answer());

    drop(tidy);
    assert!(fs::symlink_metadata(&git).is_err(), "drop の後に .git が残る");
    let caught = std::panic::catch_unwind(|| {
        let _tidy = Tidy(git.clone());
        fs::create_dir(&git).expect(".git の dir");
        panic!("落ちた歯を模す");
    });
    assert!(caught.is_err(), "閉包が panic しない");
    assert!(fs::symlink_metadata(&git).is_err(), "panic の後に .git が残る");

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth2/fstop.rs");
    let src = fs::read_to_string(&path).expect("tests/teeth2/fstop.rs を読む");
    let body = src
        .split("fn fstop_worktree_stays_quiet()")
        .nth(1)
        .expect("歯の fn の字");
    let guard = body.find("Tidy(git.clone())").expect("守りの字");
    let write = body.find("fs::write(&git").expect(".git を書く字");
    assert!(guard < write, "守りが .git を書いた後に束ねられる");
}

#[test]
fn fstop_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth2/fstop.rs");
    let src = fs::read_to_string(&path).expect("tests/teeth2/fstop.rs を読む");
    let lines: Vec<&str> = src.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 10, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("fstop_")
            .unwrap_or_else(|| panic!("{name} は fstop_ で始まらない"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
