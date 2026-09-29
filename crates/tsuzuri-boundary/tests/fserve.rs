//! server の配達と印の歯（接頭辞 fserve_・設計ノート surface-wave6 行 f-mark-serve・要件 FR9）。
//! 偽の bd は作業場の out.json を標準出力へ出す script（無ければ rc 1）、
//! 偽の bdw と偽の器は撃たれた回ごとの argv と cwd を記録の置き場（repo と state dir の外）に書く script。
//! 裁定の受付の `deliver` を直に呼ぶ歯と、tz を撃って束の口へ送る歯を持つ。

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{ChildStderr, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_boundary::server::ruling::{self, Delivery, Writer};
use tsuzuri_boundary::server::{batch, events};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerWrite};
use tsuzuri_contract::question::QuestionList;
use tsuzuri_contract::surface::{BatchItem, BatchRequest, BatchResponse, RulingId};
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{Pending, RULING_PREFIX, Route, mark_line};

/// 器の配達の口の target。
const TARGET: &str = "tsuzuri:0.1";

/// 歯の中の束の id（台帳の字には無い）。
const BATCH: &str = "batch:20260928T0110Z-1";

/// 裁定の受付の `deliver` の型（歯が型を書いて束ねる）。
type DeliverFn = fn(&Delivery, &Writer, &Source, &RulingId, &[Pending]) -> ruling::Round;

const DELIVER: DeliverFn = ruling::deliver;

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(rel: &str) -> String {
    fs::read_to_string(manifest().join("../../tests/fixtures").join(rel)).expect("fixture")
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 撃たれた回ごとに argv と cwd を `<log>/<name>.<回>.args|cwd` に書き、`<log>/<name>.fail` の回なら rc 1 で終わる script。
fn recorder(path: &Path, log: &Path, name: &str) {
    let log = log.display();
    script(
        path,
        &format!(
            "n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
             echo \"$n\" > '{log}/{name}.count'\n\
             for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'\n\
             pwd -P > '{log}/{name}.'\"$n\"'.cwd'\n\
             if [ \"$n\" = \"$(cat '{log}/{name}.fail' 2>/dev/null)\" ]; then echo refused busy >&2; exit 1; fi\n\
             exit 0"
        ),
    );
}

/// 場ごとの作業場（repo・state dir・面の file・記録の置き場・out.json・偽の program）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    state: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(tooth: &str, scene: &str, ledger: Option<&str>) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("fserve")
            .join(tooth)
            .join(scene);
        let _ = fs::remove_dir_all(&root);
        let (repo, files, state, log) = (
            root.join("repo"),
            root.join("files"),
            root.join("state"),
            root.join("log"),
        );
        for dir in [&repo, &files, &state, &log] {
            fs::create_dir_all(dir).expect("作業場の dir");
        }
        if let Some(text) = ledger {
            fs::write(root.join("out.json"), text).expect("偽の bd の出力");
        }
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("out.json").display()),
        );
        recorder(&root.join("bdw"), &log, "bdw");
        recorder(&root.join("scribe2"), &log, "scribe2");
        Place {
            root,
            repo,
            files,
            state,
            log,
        }
    }

    /// 偽の program（bdw か scribe2）を `n` 回目で落とす。
    fn fail_at(&self, name: &str, n: u32) {
        fs::write(self.log.join(format!("{name}.fail")), n.to_string()).expect("落とす回");
    }

    /// 偽の program が撃たれた回ごとの（argv・cwd）。
    fn calls(&self, name: &str) -> Vec<(Vec<String>, String)> {
        let count: u32 = fs::read_to_string(self.log.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"));
        (1..=count)
            .map(|n| {
                let read = |ext: &str| {
                    fs::read_to_string(self.log.join(format!("{name}.{n}.{ext}"))).expect("記録")
                };
                (
                    read("args").lines().map(str::to_string).collect(),
                    read("cwd"),
                )
            })
            .collect()
    }

    fn argvs(&self, name: &str) -> Vec<Vec<String>> {
        self.calls(name).into_iter().map(|(argv, _)| argv).collect()
    }

    fn delivery(&self) -> Delivery {
        Delivery {
            program: self.root.join("scribe2").into(),
            state_dir: self.state.clone(),
            target: TARGET.to_string(),
        }
    }

    fn writer(&self) -> Writer {
        Writer {
            repo: self.repo.clone(),
            bdw: self.root.join("bdw").into(),
            delivery: Some(self.delivery()),
        }
    }

    fn source(&self) -> Source {
        Source::new(self.repo.clone(), self.root.join("bd"))
    }

    /// `deliver` を直に呼び、呼ぶ前と後の分を返す。
    fn call(&self, id: &str, pending: &[Pending]) -> [String; 2] {
        let from = ruling::minute(events::now());
        DELIVER(
            &self.delivery(),
            &self.writer(),
            &self.source(),
            &ruling_id(id),
            pending,
        );
        [from, ruling::minute(events::now())]
    }

    fn repo_real(&self) -> String {
        format!(
            "{}\n",
            self.repo.canonicalize().expect("repo の実体").display()
        )
    }

    /// 偽の器と偽の bdw の cwd はどれも repo の置き場。
    fn cwd_is_repo(&self) {
        for name in ["scribe2", "bdw"] {
            for (_, cwd) in self.calls(name) {
                assert_eq!(cwd, self.repo_real(), "{name} の cwd は repo の置き場");
            }
        }
    }
}

fn ruling_id(id: &str) -> RulingId {
    RulingId::new(id).expect("裁定の id")
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead id")
}

fn pending(question: &str, ruling: &str) -> Pending {
    Pending {
        question: bead(question),
        ruling: ruling_id(ruling),
    }
}

/// 器の配達の口の argv（偽の器が受ける字）。
fn deliver_argv(place: &Place, id: &str) -> Vec<String> {
    ruling::deliver_argv(&place.delivery(), &ruling_id(id))
        .into_iter()
        .map(|a| a.into_string().expect("argv の字"))
        .collect()
}

/// 印の書きの argv が呼ぶ前か後の分の形のどちらかと同じ。
fn is_mark(argv: &[String], question: &str, ruling: &str, minutes: &[String; 2]) -> bool {
    minutes.iter().any(|m| {
        argv == LedgerWrite::AppendNotes {
            id: bead(question),
            line: mark_line(&ruling_id(ruling), Route::Deliver, m),
        }
        .argv()
    })
}

/// 字 `pub fn accept(` から次の行頭の閉じ括弧 1 字だけの行までの字。
fn accept_body(text: &str) -> &str {
    let at = text.find("pub fn accept(").expect("accept の定義");
    let rest = &text[at..];
    let end = rest.find("\n}\n").expect("accept の本文の終わり");
    &rest[..end]
}

#[test]
fn fserve_words_and_deliver() {
    let read = |rel: &str| fs::read_to_string(manifest().join(rel)).expect("src を読む");
    let ruling_rs = read("src/server/ruling.rs");
    let batch_rs = read("src/server/batch.rs");
    assert_eq!(
        ruling::NOT_TAKEN,
        "配達の口が今は受けない（印を置かず、席の停止の hook が拾う）"
    );
    assert_eq!(RULING_PREFIX, ruling::LINE_PREFIX);
    for (name, text) in [("ruling.rs", &ruling_rs), ("batch.rs", &batch_rs)] {
        assert!(!text.contains("配達が落ちた"), "{name} に「配達が落ちた」");
        assert!(
            accept_body(text).contains("deliver("),
            "{name} の accept が deliver を通らない"
        );
    }
    assert_eq!(
        ruling_rs.matches("deliver_argv(").count(),
        2,
        "器の CLI を撃つ所は ruling の 1 か所"
    );
    for word in ["deliver_argv", "DELIVER_TIMEOUT"] {
        assert!(!batch_rs.contains(word), "batch.rs に {word}");
    }
}

#[test]
fn fserve_deliver_skips_marked() {
    let ledger = fixture("stop/ledger.json");
    let tooth = "deliver_skips_marked";

    // 印の在る裁定は撃たない。
    let place = Place::new(tooth, "marked", Some(&ledger));
    let id = "fx-s.1:20260928T0100Z-1";
    place.call(id, &[pending("fx-s.1", id)]);
    assert!(place.calls("scribe2").is_empty(), "印の在る裁定で偽の器を撃つ");
    assert!(place.calls("bdw").is_empty(), "印の在る裁定で偽の bdw を撃つ");

    // 印の無い裁定は撃って印を置く（別の bead の印は数えない）。
    let place = Place::new(tooth, "unmarked", Some(&ledger));
    let id = "fx-s.2:20260928T0101Z-1";
    let minutes = place.call(id, &[pending("fx-s.2", id)]);
    assert_eq!(place.argvs("scribe2"), [deliver_argv(&place, id)]);
    let bdw = place.argvs("bdw");
    assert_eq!(bdw.len(), 1, "{bdw:?}");
    assert!(is_mark(&bdw[0], "fx-s.2", id, &minutes), "{bdw:?}");
    place.cwd_is_repo();

    // 器が受けなければ印を置かない。
    let place = Place::new(tooth, "refused", Some(&ledger));
    place.fail_at("scribe2", 1);
    let id = "fx-s.3:20260928T0102Z-1";
    place.call(id, &[pending("fx-s.3", id)]);
    assert_eq!(place.argvs("scribe2"), [deliver_argv(&place, id)]);
    assert!(place.calls("bdw").is_empty(), "受けないのに印を置く");
    place.cwd_is_repo();

    // 台帳が読めなければ撃つ側に倒す。
    let place = Place::new(tooth, "bd-fails", None);
    let id = "fx-s.1:20260928T0100Z-1";
    let minutes = place.call(id, &[pending("fx-s.1", id)]);
    assert_eq!(place.argvs("scribe2"), [deliver_argv(&place, id)]);
    let bdw = place.argvs("bdw");
    assert_eq!(bdw.len(), 1, "{bdw:?}");
    assert!(is_mark(&bdw[0], "fx-s.1", id, &minutes), "{bdw:?}");
    place.cwd_is_repo();
}

#[test]
fn fserve_batch_delivers_once_and_marks_all() {
    let ledger = fixture("stop/ledger.json");
    let tooth = "batch_delivers_once_and_marks_all";
    let (r1, r2, r3, r4) = (
        "fx-s.1:20260928T0100Z-1",
        "fx-s.2:20260928T0101Z-1",
        "fx-s.3:20260928T0102Z-1",
        "fx-s.4:20260928T0103Z-1",
    );

    // 印の無い 2 つは束の id で 1 度撃ち、行の順に印を置く。
    let place = Place::new(tooth, "unmarked", Some(&ledger));
    let minutes = place.call(BATCH, &[pending("fx-s.2", r2), pending("fx-s.3", r3)]);
    assert_eq!(place.argvs("scribe2"), [deliver_argv(&place, BATCH)]);
    let bdw = place.argvs("bdw");
    assert_eq!(bdw.len(), 2, "{bdw:?}");
    assert!(is_mark(&bdw[0], "fx-s.2", r2, &minutes), "{bdw:?}");
    assert!(is_mark(&bdw[1], "fx-s.3", r3, &minutes), "{bdw:?}");
    place.cwd_is_repo();

    // どちらも印が在れば撃たない。
    let place = Place::new(tooth, "marked", Some(&ledger));
    place.call(BATCH, &[pending("fx-s.1", r1), pending("fx-s.4", r4)]);
    assert!(place.calls("scribe2").is_empty(), "印の在る束で偽の器を撃つ");
    assert!(place.calls("bdw").is_empty(), "印の在る束で偽の bdw を撃つ");

    // 1 つでも印が無ければ撃ち、全部に印を置く。
    let place = Place::new(tooth, "mixed", Some(&ledger));
    let minutes = place.call(BATCH, &[pending("fx-s.1", r1), pending("fx-s.2", r2)]);
    assert_eq!(place.argvs("scribe2"), [deliver_argv(&place, BATCH)]);
    let bdw = place.argvs("bdw");
    assert_eq!(bdw.len(), 2, "{bdw:?}");
    assert!(is_mark(&bdw[0], "fx-s.1", r1, &minutes), "{bdw:?}");
    assert!(is_mark(&bdw[1], "fx-s.2", r2, &minutes), "{bdw:?}");
    place.cwd_is_repo();

    // 器が受けなければ印を置かない。
    let place = Place::new(tooth, "refused", Some(&ledger));
    place.fail_at("scribe2", 1);
    place.call(BATCH, &[pending("fx-s.2", r2), pending("fx-s.3", r3)]);
    assert_eq!(place.argvs("scribe2"), [deliver_argv(&place, BATCH)]);
    assert!(place.calls("bdw").is_empty(), "受けないのに印を置く");
}

/// 要求を 1 つ撃ち、接続が閉じるまで応答を読む（状態の code と本文）。
fn send(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .as_bytes(),
    )
    .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse().ok())
        .expect("状態の code");
    (status, body.to_string())
}

/// 口 GET /api/questions の応答の card の digest。
fn digest(addr: SocketAddr, id: &str) -> String {
    let (status, body) = send(addr, "GET", "/api/questions", "");
    assert_eq!(status, 200, "{body}");
    let list: QuestionList = wire::decode(&body).expect("問いの一覧の形");
    let Reading::Known(cards) = list.cards else {
        panic!("読める台帳が Unknown");
    };
    cards
        .into_iter()
        .find(|c| c.id.as_str() == id)
        .unwrap_or_else(|| panic!("{id} の card"))
        .digest
}

/// 標準 error を行ごとに送る thread を立てる（tz が止まれば送り終える）。
fn stderr_lines(mut err: BufReader<ChildStderr>) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut line = String::new();
        while err.read_line(&mut line).is_ok_and(|n| n > 0) {
            if tx.send(std::mem::take(&mut line)).is_err() {
                break;
            }
        }
    });
    rx
}

/// `word` を含む行が来るまで 20 秒まで受け、受けた行を `out` に足す。
fn wait_line(rx: &mpsc::Receiver<String>, word: &str, out: &mut String) {
    let until = Instant::now() + Duration::from_secs(20);
    while let Some(left) = until.checked_duration_since(Instant::now()) {
        let Ok(line) = rx.recv_timeout(left) else {
            return;
        };
        out.push_str(&line);
        if line.contains(word) {
            return;
        }
    }
}

#[test]
fn fserve_batch_refusal_logs_once() {
    let place = Place::new(
        "batch_refusal_logs_once",
        "refused",
        Some(&fixture("surface/question-batch.json")),
    );
    place.fail_at("scribe2", 1);
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["surface", "serve", "--bind", "127.0.0.1:0", "--repo"])
        .arg(&place.repo)
        .arg("--files")
        .arg(&place.files)
        .arg(format!("--bd={}", place.root.join("bd").display()))
        .arg(format!("--bdw={}", place.root.join("bdw").display()))
        .args(["--seat", TARGET])
        .arg(format!("--state-dir={}", place.state.display()))
        .arg(format!("--scribe2={}", place.root.join("scribe2").display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tz を撃つ");
    let mut err = BufReader::new(child.stderr.take().expect("標準 error"));
    let mut line = String::new();
    err.read_line(&mut line).expect("口の住所の行");
    let addr: SocketAddr = line
        .trim()
        .strip_prefix("tz surface serve: http://")
        .and_then(|a| a.strip_suffix('/'))
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| panic!("口の住所の行でない: {line}"));
    let rx = stderr_lines(err);
    let reply = std::panic::catch_unwind(|| {
        let items = ["fx-b.2", "fx-b.3"]
            .map(|q| BatchItem {
                question: bead(q),
                seen_digest: digest(addr, q),
                verbatim: None,
            })
            .to_vec();
        let body = wire::encode(&BatchRequest {
            items,
            verbatim: "はい".to_string(),
        })
        .expect("要求の電文");
        send(addr, "POST", batch::PATH, &body)
    });
    let mut rest = String::new();
    // 配達は応答の後の thread なので、受けない行が出るまで読んでから止める。
    if reply.is_ok() {
        wait_line(&rx, ruling::NOT_TAKEN, &mut rest);
    }
    let _ = child.kill();
    let _ = child.wait();
    rest.extend(rx.iter());
    let (status, body) = reply.expect("応答を読めない");
    assert_eq!(status, 200, "{body}");
    let got: BatchResponse = wire::decode(&body).expect("束の応答の形");

    assert_eq!(
        place.argvs("scribe2"),
        [deliver_argv(&place, got.batch.as_str())],
        "束の id で 1 度だけ"
    );
    let bdw = place.argvs("bdw");
    assert_eq!(bdw.len(), 4, "行ごとの追記と閉じるだけ: {bdw:?}");
    let lines: Vec<&str> = rest
        .lines()
        .filter(|l| l.contains(ruling::NOT_TAKEN))
        .collect();
    assert_eq!(lines.len(), 1, "受けない行が 1 行でない: {rest}");
    assert!(lines[0].contains(got.batch.as_str()), "{rest}");
    assert!(!rest.contains("配達が落ちた"), "{rest}");
}

/// filter の語（着地済みの行と並ぶ行の接頭辞）。
const FILTER: [&str; 92] = [
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
    "fstop_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
];

#[test]
fn fserve_own_names_clean() {
    let text = fs::read_to_string(manifest().join("tests/fserve.rs")).expect("歯の file");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            w[1].trim()
                .strip_prefix("fn ")
                .and_then(|r| r.split('(').next())
                .unwrap_or_else(|| panic!("test の属性の後の行が fn でない: {}", w[1]))
        })
        .collect();
    assert_eq!(names.len(), 5, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("fserve_")
            .unwrap_or_else(|| panic!("fserve_ で始まらない: {name}"));
        for word in FILTER {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
