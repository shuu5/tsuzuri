//! 束の受付の口と方針の受付の口の歯（接頭辞 server_batch_・設計ノート surface-base 便 e-batch の完了の条件）。
//! 偽の bd は作業場の out.json（既定は fixture の question-batch.json）を標準出力へ出す script、
//! 偽の bdw と偽の器は受けた argv を記録の置き場（repo と state dir の外）に 1 行ずつ書く script。
//! 見た版の要約値は GET /api/questions の応答の card の digest から取る。
//! server は同じ process の thread で 127.0.0.1 の空き port に立てる。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::ledger::epoch_secs;
use tsuzuri_boundary::server::{Config, Server, batch, policy, ruling};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadId, LedgerWrite};
use tsuzuri_contract::question::QuestionList;
use tsuzuri_contract::surface::{
    BatchItem, BatchRequest, BatchResponse, ItemOutcome, PolicyRequest, PolicyResponse, Refusal,
    RefusalResponse, RulingId,
};
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{Route, mark_line};

const FIXTURE: &str = "surface/question-batch.json";

/// 定型行を持つ open の問い 2 本・A-1 の印を持つ open の問い・closed の問い・台帳に無い id。
const Q2: &str = "fx-b.2";
const Q3: &str = "fx-b.3";
const A1: &str = "fx-b.4";
const CLOSED: &str = "fx-b.5";
const MISSING: &str = "fx-b.9";

/// 偽の bdw が子を作る書きの回に出す、作った方針の問いの id。
const CREATED: &str = "fx-b.6";

fn read_fixture() -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(FIXTURE),
    )
    .expect("fixture")
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 撃たれた回ごとに argv と cwd を `<log>/<name>.<回>.args|cwd` に書き、`<log>/<name>.fail` の回なら rc 1 で終わる script。
/// 落とさない回で最初の引数が `create` なら作った子の id（`CREATED`）を標準出力へ出す。
fn recorder(path: &Path, log: &Path, name: &str) {
    let log = log.display();
    script(
        path,
        &format!(
            "n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
             echo \"$n\" > '{log}/{name}.count'\n\
             for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'\n\
             pwd -P > '{log}/{name}.'\"$n\"'.cwd'\n\
             if [ \"$n\" = \"$(cat '{log}/{name}.fail' 2>/dev/null)\" ]; then echo 落ちた >&2; exit 1; fi\n\
             if [ \"$1\" = create ]; then echo {CREATED}; fi\n\
             exit 0"
        ),
    );
}

/// 歯ごとの作業場（repo・面の file・state dir・偽の program・記録の置き場）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    state: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_batch")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, files, state, log) = (
            root.join("repo"),
            root.join("files"),
            root.join("state"),
            root.join("log"),
        );
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::create_dir_all(&log).expect("記録の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        for mark in ["issues.jsonl", "interactions.jsonl"] {
            fs::write(repo.join(".beads").join(mark), "{}\n").expect("印の file");
        }
        fs::write(state.join("fleet/events.jsonl"), "").expect("event log");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("out.json").display()),
        );
        recorder(&root.join("bdw"), &log, "bdw");
        recorder(&root.join("scribe2"), &log, "scribe2");
        let place = Place {
            root,
            repo,
            files,
            state,
            log,
        };
        place.bd_returns(&read_fixture());
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

    /// 偽の program の `n` 回目の cwd の記録が空でなくなるまで 10 秒まで待つ（配達は応答の後の thread）。
    fn wait(&self, name: &str, n: u32) {
        let path = self.log.join(format!("{name}.{n}.cwd"));
        let until = Instant::now() + Duration::from_secs(10);
        while !fs::read(&path).is_ok_and(|b| !b.is_empty()) && Instant::now() < until {
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn config(&self, seat: Option<&str>, state_dir: bool) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            state_dir: state_dir.then(|| self.state.clone()),
            bdw: self.root.join("bdw").into(),
            seat: seat.map(str::to_string),
            scribe2: self.root.join("scribe2").into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    fn serve_with(&self, config: &Config) -> SocketAddr {
        let server = Server::bind(config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }

    /// 席の target も state dir も持つ server。
    fn serve(&self) -> SocketAddr {
        self.serve_with(&self.config(Some("tsuzuri:0.1"), true))
    }

    fn repo_real(&self) -> String {
        format!(
            "{}\n",
            self.repo.canonicalize().expect("repo の実体").display()
        )
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

struct Reply {
    status: u16,
    head: String,
    body: String,
}

/// 本文と足す頭の行で要求を 1 つ撃ち、接続が閉じるまで応答を読む。
fn send(addr: SocketAddr, method: &str, path: &str, heads: &str, body: &str) -> Reply {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\n{heads}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
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
    Reply {
        status,
        head: head.to_string(),
        body: body.to_string(),
    }
}

/// 口 GET /api/questions の応答の card の digest（見た版の要約値）。
fn digest(addr: SocketAddr, id: &str) -> String {
    let reply = send(addr, "GET", "/api/questions", "", "");
    assert_eq!(reply.status, 200, "{}", reply.body);
    let list: QuestionList = wire::decode(&reply.body).expect("問いの一覧の形");
    let Reading::Known(cards) = list.cards else {
        panic!("読める台帳が Unknown");
    };
    cards
        .into_iter()
        .find(|c| c.id.as_str() == id)
        .unwrap_or_else(|| panic!("{id} の card"))
        .digest
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead id")
}

fn item(question: &str, digest: &str, verbatim: Option<&str>) -> BatchItem {
    BatchItem {
        question: bead(question),
        seen_digest: digest.to_string(),
        verbatim: verbatim.map(str::to_string),
    }
}

fn batch_body(items: Vec<BatchItem>, verbatim: &str) -> String {
    wire::encode(&BatchRequest {
        items,
        verbatim: verbatim.to_string(),
    })
    .expect("要求の電文")
}

fn post_batch(addr: SocketAddr, items: Vec<BatchItem>, verbatim: &str) -> Reply {
    send(addr, "POST", batch::PATH, "", &batch_body(items, verbatim))
}

fn policy_body(scope: &str, verbatim: &str) -> String {
    wire::encode(&PolicyRequest {
        scope: scope.to_string(),
        verbatim: verbatim.to_string(),
    })
    .expect("要求の電文")
}

fn post_policy(addr: SocketAddr, scope: &str, verbatim: &str) -> Reply {
    send(
        addr,
        "POST",
        policy::PATH,
        "",
        &policy_body(scope, verbatim),
    )
}

fn json_body(reply: &Reply) {
    assert!(
        reply.head.contains("Content-Type: application/json"),
        "{}",
        reply.head
    );
}

fn batched(reply: &Reply) -> BatchResponse {
    assert_eq!(reply.status, 200, "{}", reply.body);
    json_body(reply);
    wire::decode(&reply.body).unwrap_or_else(|e| panic!("束の応答の形でない {e}: {}", reply.body))
}

fn policied(reply: &Reply) -> PolicyResponse {
    assert_eq!(reply.status, 200, "{}", reply.body);
    json_body(reply);
    wire::decode(&reply.body).unwrap_or_else(|e| panic!("方針の応答の形でない {e}: {}", reply.body))
}

fn refused(reply: &Reply) -> Refusal {
    json_body(reply);
    wire::decode::<RefusalResponse>(&reply.body)
        .unwrap_or_else(|e| panic!("断りの形でない {e}: {}", reply.body))
        .reason
}

/// id の分（`20260927T1034Z` の形）を epoch 秒にする（歯の側で独立に読む）。
fn minute_secs(minute: &str) -> u64 {
    let b = minute.as_bytes();
    assert!(
        b.len() == 14 && b[8] == b'T' && b[13] == b'Z',
        "分の形でない: {minute}"
    );
    let rfc = format!(
        "{}-{}-{}T{}:{}:00Z",
        &minute[0..4],
        &minute[4..6],
        &minute[6..8],
        &minute[9..11],
        &minute[11..13]
    );
    epoch_secs(&rfc).unwrap_or_else(|| panic!("分の形でない: {minute}"))
}

/// id を（頭・分・数）に分ける（`<頭>:<分>-<数>`）。
fn split_id(id: &str) -> (&str, &str, u32) {
    let (head, rest) = id.split_once(':').expect("id の「:」");
    let (minute, n) = rest.rsplit_once('-').expect("id の「-」");
    (head, minute, n.parse().expect("数"))
}

/// 分が受付の前後の時刻の分に収まる。
fn minute_within(minute: &str, from: u64, to: u64) {
    let at = minute_secs(minute);
    assert!(
        from / 60 * 60 <= at && at <= to,
        "分は受付の時刻の UTC の分: {minute} not in {from}..={to}"
    );
}

/// 行が要求の順に書いた（Written）で、裁定の id を返す。
fn written(got: &BatchResponse, questions: &[&str]) -> Vec<RulingId> {
    let ids: Vec<&str> = got.items.iter().map(|r| r.question.as_str()).collect();
    assert_eq!(ids, questions, "行は要求の順");
    got.items
        .iter()
        .map(|r| match &r.outcome {
            ItemOutcome::Written { ruling } => ruling.clone(),
            other => panic!("{} が書いたでない: {other:?}", r.question),
        })
        .collect()
}

/// 束の 1 行の 2 回の書き（notes への追記と閉じる）の argv。
fn row_argvs(
    question: &str,
    ruling: &RulingId,
    batch: &RulingId,
    verbatim: &str,
) -> [Vec<String>; 2] {
    [
        LedgerWrite::AppendNotes {
            id: bead(question),
            line: format!("裁定 id = {ruling}・問い = {question}・束 = {batch}・逐語 = {verbatim}"),
        }
        .argv(),
        LedgerWrite::CloseItem {
            id: bead(question),
            reason: format!("裁定 {ruling} 束 {batch}"),
        }
        .argv(),
    ]
}

/// 問いの notes に配達の口の印を足す書きの argv。
fn mark_argv(question: &str, ruling: &RulingId, minute: &str) -> Vec<String> {
    LedgerWrite::AppendNotes {
        id: bead(question),
        line: mark_line(ruling, Route::Deliver, minute),
    }
    .argv()
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

#[test]
fn server_batch_writes_rows_in_order() {
    let place = Place::new("write");
    let addr = place.serve();
    let (d2, d3) = (digest(addr, Q2), digest(addr, Q3));
    let from = now();
    let got = batched(&post_batch(
        addr,
        vec![
            item(Q2, &d2, Some("減らしてよい\n2 行目 \\ 逆斜線")),
            item(Q3, &d3, None),
        ],
        "束の答え",
    ));
    let to = now();
    let (head, minute, n) = split_id(got.batch.as_str());
    assert_eq!((head, n), ("batch", 1), "{}", got.batch);
    minute_within(minute, from, to);
    let rulings = written(&got, &[Q2, Q3]);
    for (ruling, q) in rulings.iter().zip([Q2, Q3]) {
        assert_eq!(
            ruling.as_str(),
            format!("{q}:{minute}-1"),
            "裁定の id は束と同じ分"
        );
    }

    place.wait("bdw", 6);
    let calls = place.calls("bdw");
    let [a2, c2] = row_argvs(
        Q2,
        &rulings[0],
        &got.batch,
        "減らしてよい\\n2 行目 \\\\ 逆斜線",
    );
    let [a3, c3] = row_argvs(Q3, &rulings[1], &got.batch, "束の答え");
    let argvs: Vec<Vec<String>> = calls.iter().map(|(a, _)| a.clone()).collect();
    assert_eq!(argvs.len(), 6, "{argvs:?}");
    assert_eq!(argvs[..4], [a2, c2, a3, c3], "追記・閉じる・追記・閉じるの順");
    for (at, (q, ruling)) in [(4, (Q2, &rulings[0])), (5, (Q3, &rulings[1]))] {
        let marks: Vec<Vec<String>> = [ruling::minute(from), ruling::minute(to)]
            .iter()
            .map(|m| mark_argv(q, ruling, m))
            .collect();
        assert!(
            marks.contains(&argvs[at]),
            "{} 回目は {q} の配達の口の印: {:?}",
            at + 1,
            argvs[at]
        );
    }
    for (_, cwd) in &calls {
        assert_eq!(cwd, &place.repo_real(), "cwd は repo の置き場");
    }
}

#[test]
fn server_batch_own_verbatim_or_batch_verbatim() {
    let place = Place::new("verbatim");
    let addr = place.serve();
    let (d2, d3) = (digest(addr, Q2), digest(addr, Q3));
    // 空白だけの個別の逐語は束の逐語に代わる。
    let got = batched(&post_batch(
        addr,
        vec![
            item(Q2, &d2, Some(" \n\t　")),
            item(Q3, &d3, Some("個別の字")),
        ],
        "束の字",
    ));
    let rulings = written(&got, &[Q2, Q3]);
    let [a2, _] = row_argvs(Q2, &rulings[0], &got.batch, "束の字");
    let [a3, _] = row_argvs(Q3, &rulings[1], &got.batch, "個別の字");
    place.wait("bdw", 6);
    let argvs = place.argvs("bdw");
    assert_eq!(argvs.len(), 6);
    assert_eq!((&argvs[0], &argvs[2]), (&a2, &a3));
    // 個別の逐語が在れば束の逐語は空白だけでよい。
    let place = Place::new("verbatim-own");
    let addr = place.serve();
    let got = batched(&post_batch(
        addr,
        vec![item(Q2, &d2, Some("こちら")), item(Q3, &d3, Some("あちら"))],
        "  ",
    ));
    let rulings = written(&got, &[Q2, Q3]);
    place.wait("bdw", 6);
    let argvs = place.argvs("bdw");
    assert_eq!(
        argvs[0],
        row_argvs(Q2, &rulings[0], &got.batch, "こちら")[0]
    );
    assert_eq!(
        argvs[2],
        row_argvs(Q3, &rulings[1], &got.batch, "あちら")[0]
    );
}

#[test]
fn server_batch_refusals_write_nothing() {
    let place = Place::new("refuse");
    let addr = place.serve();
    let (d2, d3, d4) = (digest(addr, Q2), digest(addr, Q3), digest(addr, A1));
    let stale = "0000000000000000";
    let rows = vec![
        // A-1 の印を持つ問いを含む束。
        (
            vec![item(Q2, &d2, None), item(A1, &d4, Some("個別"))],
            "はい",
            Refusal::A1InBatch,
        ),
        // 見た版が 1 つでも古い束（別の問いの digest も古い扱い）。
        (
            vec![item(Q2, &d2, None), item(Q3, stale, None)],
            "はい",
            Refusal::StaleVersion,
        ),
        (vec![item(Q2, &d3, None)], "はい", Refusal::StaleVersion),
        // 台帳に無い id・closed の問い。
        (
            vec![item(Q2, &d2, None), item(MISSING, &d2, None)],
            "はい",
            Refusal::UnknownQuestion,
        ),
        (
            vec![item(CLOSED, &d2, None)],
            "はい",
            Refusal::UnknownQuestion,
        ),
        // 空の束。
        (vec![], "はい", Refusal::EmptyBatch),
        // 逐語が決まらない行。
        (
            vec![item(Q2, &d2, Some("個別")), item(Q3, &d3, None)],
            " \n",
            Refusal::EmptyVerbatim,
        ),
        (
            vec![item(Q2, &d2, Some("")), item(Q3, &d3, Some("個別"))],
            "",
            Refusal::EmptyVerbatim,
        ),
    ];
    refuse_rows(place, addr, rows, (d2, d3, d4), stale);
}

/// 断る行が 2 つ在る束を表に足して撃ち、表の外の断りと断りで書かないことを見る。
fn refuse_rows(
    place: Place,
    addr: SocketAddr,
    rows: Vec<(Vec<BatchItem>, &str, Refusal)>,
    (d2, d3, d4): (String, String, String),
    stale: &str,
) {
    for (items, verbatim, want) in rows.into_iter().chain([
        // 断る行が 2 つ在れば、要求の順で先の行の理由。
        (
            vec![item(MISSING, &d2, None), item(A1, &d4, None)],
            "はい",
            Refusal::UnknownQuestion,
        ),
        (
            vec![item(A1, &d4, None), item(MISSING, &d2, None)],
            "はい",
            Refusal::A1InBatch,
        ),
        (
            vec![item(Q2, stale, None), item(A1, &d4, None)],
            "はい",
            Refusal::StaleVersion,
        ),
        (
            vec![item(A1, &d4, None), item(Q2, stale, None)],
            "はい",
            Refusal::A1InBatch,
        ),
        (
            vec![item(Q3, stale, None), item(CLOSED, &d2, None)],
            "はい",
            Refusal::StaleVersion,
        ),
    ]) {
        let label = format!("{items:?} {verbatim:?}");
        let reply = post_batch(addr, items, verbatim);
        assert_eq!(reply.status, want.http_status(), "{label}: {}", reply.body);
        assert_eq!(refused(&reply), want, "{label}");
    }
    assert_eq!(Refusal::A1InBatch.http_status(), 400);
    assert_eq!(Refusal::StaleVersion.http_status(), 409);
    assert_eq!(Refusal::UnknownQuestion.http_status(), 404);
    assert_eq!(Refusal::EmptyBatch.http_status(), 400);
    assert_eq!(Refusal::EmptyVerbatim.http_status(), 400);
    // 同じ問いを 2 度含む束は 400 と duplicate。
    let reply = post_batch(
        addr,
        vec![
            item(Q2, &d2, None),
            item(Q3, &d3, None),
            item(Q2, &d2, Some("また")),
        ],
        "はい",
    );
    assert_eq!((reply.status, reply.body.as_str()), (400, "duplicate"));
    // 台帳が読めなければ 503。
    place.bd_fails();
    let reply = post_batch(addr, vec![item(Q2, &d2, None)], "はい");
    assert_eq!(reply.status, 503, "{}", reply.body);
    assert!(place.calls("bdw").is_empty(), "断りで偽の bdw を撃つ");
    assert!(place.calls("scribe2").is_empty(), "断りで偽の器を撃つ");
}

#[test]
fn server_batch_id_counts_up_in_same_minute() {
    for _ in 0..3 {
        let place = Place::new("count");
        let addr = place.serve();
        let d2 = digest(addr, Q2);
        let first = batched(&post_batch(addr, vec![item(Q2, &d2, None)], "はい"));
        place.wait("bdw", 3);
        // 1 つ目の束の追記を台帳に映す（偽の bd は書きを映さないので字を置き直す）。
        let appended = place.argvs("bdw")[0][2]
            .strip_prefix("--append-notes=")
            .expect("追記の旗")
            .to_string();
        let old = format!("\"notes\":\"{Q2} の notes の 1 行\"");
        let fixture = read_fixture();
        assert!(fixture.contains(&old), "{old}");
        place.bd_returns(&fixture.replace(
            &old,
            &format!("\"notes\":\"{Q2} の notes の 1 行\\n{appended}\""),
        ));
        let d3 = digest(addr, Q3);
        let second = batched(&post_batch(addr, vec![item(Q3, &d3, None)], "はい"));
        let (_, m1, n1) = split_id(first.batch.as_str());
        let (_, m2, n2) = split_id(second.batch.as_str());
        if m1 != m2 {
            continue; // 分を跨いだ（撃ち直す）。
        }
        assert_eq!((n1, n2), (1, 2), "{} {}", first.batch, second.batch);
        return;
    }
    panic!("3 回とも分を跨いだ");
}

#[test]
fn server_batch_write_failure_is_502_with_done_rows() {
    let place = Place::new("fail-3");
    place.fail_at("bdw", 3);
    let addr = place.serve();
    let (d2, d3) = (digest(addr, Q2), digest(addr, Q3));
    let reply = post_batch(addr, vec![item(Q2, &d2, None), item(Q3, &d3, None)], "はい");
    assert_eq!(reply.status, 502, "{}", reply.body);
    json_body(&reply);
    let got: BatchResponse = wire::decode(&reply.body).expect("束の応答の形");
    let rulings = written(
        &BatchResponse {
            batch: got.batch.clone(),
            items: got.items[..1].to_vec(),
        },
        &[Q2],
    );
    assert_eq!(got.items.len(), 2, "要求の全部の行: {:?}", got.items);
    assert_eq!(
        (got.items[1].question.as_str(), &got.items[1].outcome),
        (Q3, &ItemOutcome::Unwritten),
        "追記が落ちた行は書いていない"
    );
    let argvs = place.argvs("bdw");
    assert_eq!(
        argvs.len(),
        3,
        "2 つ目の問いの閉じる書きは撃たない: {argvs:?}"
    );
    assert_eq!(argvs[..2], row_argvs(Q2, &rulings[0], &got.batch, "はい"));
    assert_eq!(
        argvs[2][0..2],
        ["update", Q3],
        "3 回目は 2 つ目の問いの追記"
    );
    assert!(
        place.calls("scribe2").is_empty(),
        "落ちた書きの後に配達する"
    );
    // 1 回目が落ちれば 2 行とも書いていない。
    let place = Place::new("fail-1");
    place.fail_at("bdw", 1);
    let addr = place.serve();
    let reply = post_batch(addr, vec![item(Q2, &d2, None), item(Q3, &d3, None)], "はい");
    assert_eq!(reply.status, 502, "{}", reply.body);
    let got: BatchResponse = wire::decode(&reply.body).expect("束の応答の形");
    let rows: Vec<(&str, &ItemOutcome)> = got
        .items
        .iter()
        .map(|r| (r.question.as_str(), &r.outcome))
        .collect();
    assert_eq!(
        rows,
        [(Q2, &ItemOutcome::Unwritten), (Q3, &ItemOutcome::Unwritten)]
    );
    assert_eq!(place.calls("bdw").len(), 1);
}

#[test]
fn server_batch_delivers_once_per_batch() {
    let place = Place::new("deliver");
    let addr = place.serve();
    let (d2, d3) = (digest(addr, Q2), digest(addr, Q3));
    let got = batched(&post_batch(
        addr,
        vec![item(Q2, &d2, None), item(Q3, &d3, None)],
        "はい",
    ));
    written(&got, &[Q2, Q3]);
    place.wait("bdw", 6);
    let state = place.state.display().to_string();
    let want: Vec<Vec<String>> = vec![
        [
            "seat",
            "deliver",
            "--state-dir",
            state.as_str(),
            "--target",
            "tsuzuri:0.1",
            "--ruling",
            got.batch.as_str(),
        ]
        .map(str::to_string)
        .to_vec(),
    ];
    assert_eq!(place.argvs("scribe2"), want, "束の id で 1 度だけ");
    // 片方でも無ければ撃たない。
    for (name, seat, state_dir) in [
        ("no-seat", None, true),
        ("no-state", Some("tsuzuri:0.1"), false),
    ] {
        let place = Place::new(name);
        let addr = place.serve_with(&place.config(seat, state_dir));
        batched(&post_batch(
            addr,
            vec![item(Q2, &d2, None), item(Q3, &d3, None)],
            "はい",
        ));
        assert_eq!(place.calls("bdw").len(), 4, "{name}");
        assert!(place.calls("scribe2").is_empty(), "{name}: 偽の器を撃つ");
    }
    // 配達が落ちても 200。
    let place = Place::new("deliver-fail");
    place.fail_at("scribe2", 1);
    let addr = place.serve();
    batched(&post_batch(
        addr,
        vec![item(Q2, &d2, None), item(Q3, &d3, None)],
        "はい",
    ));
    place.wait("scribe2", 1);
    assert_eq!(place.calls("scribe2").len(), 1);
    assert_eq!(place.calls("bdw").len(), 4, "受けなければ印を置かない");
}

#[test]
fn server_batch_guards_write_nothing() {
    let place = Place::new("guard");
    let addr = place.serve();
    let d2 = digest(addr, Q2);
    let port = addr.port();
    let bodies = [
        (batch::PATH, batch_body(vec![item(Q2, &d2, None)], "はい")),
        (policy::PATH, policy_body("all", "はい")),
    ];
    for (path, body) in &bodies {
        // Origin の host と port が Host と違えば 403。
        for origin in [
            format!("http://evil.example:{port}"),
            format!("http://127.0.0.1:{}", port.wrapping_add(1)),
            "null".to_string(),
        ] {
            let reply = send(addr, "POST", path, &format!("Origin: {origin}\r\n"), body);
            assert_eq!(reply.status, 403, "{path} {origin}: {}", reply.body);
        }
        // 読めない本文は 400 と bad-body（別の口の要求の形も読めない）。
        let other = if *path == batch::PATH {
            policy_body("all", "はい")
        } else {
            batch_body(vec![item(Q2, &d2, None)], "はい")
        };
        for bad in ["{", "", "[]", other.as_str()] {
            let reply = send(addr, "POST", path, "", bad);
            assert_eq!(
                (reply.status, reply.body.as_str()),
                (400, "bad-body"),
                "{path} {bad:?}"
            );
        }
    }
    // 本文が 65536 byte を越えれば 413（逐語を伸ばした正しい形の要求）。
    let long = "あ".repeat(22_000);
    for (path, big) in [
        (batch::PATH, batch_body(vec![item(Q2, &d2, None)], &long)),
        (policy::PATH, policy_body("all", &long)),
    ] {
        assert!(big.len() > 65_536);
        assert_eq!(send(addr, "POST", path, "", &big).status, 413, "{path}");
    }
    assert!(place.calls("bdw").is_empty(), "守りで偽の bdw を撃つ");
    assert!(place.calls("scribe2").is_empty(), "守りで偽の器を撃つ");
    guards_pass_same_origin(place, addr, bodies);
}

/// 同じ Origin の要求は通り、GET でない要求を受けない口と method は 405 であることを見る。
fn guards_pass_same_origin(place: Place, addr: SocketAddr, bodies: [(&str, String); 2]) {
    // 同じ Origin は通る（束の配達の印を待ってから方針を送る）。
    for (path, body) in &bodies {
        let reply = send(
            addr,
            "POST",
            path,
            &format!("Origin: http://{addr}\r\n"),
            body,
        );
        assert_eq!(reply.status, 200, "{path}: {}", reply.body);
        if *path == batch::PATH {
            place.wait("bdw", 3);
        }
    }
    // 束は追記と閉じるの 2 回と印の 1 回・方針は作る・足す・閉じるの 3 回。
    assert_eq!(place.calls("bdw").len(), 6);
    // GET でない要求を受ける口は 3 つだけ（ほかの POST は 405）。
    for path in ["/api/batch/x", "/api/policy/x", "/api/batches"] {
        assert_eq!(send(addr, "POST", path, "", "{}").status, 405, "{path}");
    }
    for method in ["PUT", "DELETE"] {
        assert_eq!(send(addr, method, batch::PATH, "", "{}").status, 405);
        assert_eq!(send(addr, method, policy::PATH, "", "{}").status, 405);
    }
}

#[test]
fn server_batch_repo_and_state_bytes_unchanged() {
    let place = Place::new("bytes");
    let addr = place.serve();
    let before = (tree(&place.repo), tree(&place.state));
    let (d2, d3, d4) = (digest(addr, Q2), digest(addr, Q3), digest(addr, A1));
    assert_eq!(
        post_batch(addr, vec![item(Q2, &d2, None), item(A1, &d4, None)], "はい").status,
        400
    );
    assert_eq!(post_policy(addr, MISSING, "はい").status, 400);
    batched(&post_batch(
        addr,
        vec![item(Q2, &d2, None), item(Q3, &d3, None)],
        "はい",
    ));
    place.wait("bdw", 6);
    policied(&post_policy(addr, "all", "はい"));
    // 束 6 回（2 行の追記と閉じるの 4 回と印の 2 回）・方針 3 回（作る・足す・閉じる）。
    assert_eq!(place.calls("bdw").len(), 9);
    assert_eq!(place.calls("scribe2").len(), 1);
    assert!(
        before == (tree(&place.repo), tree(&place.state)),
        "受付の後に repo か state dir の byte が変わる"
    );
}

#[test]
fn server_batch_fixture_shape_and_no_new_dependencies() {
    let fixture = read_fixture();
    assert!(
        fixture.len() <= 5000,
        "fixture は 5000 byte 以下: {}",
        fixture.len()
    );
    // 外の依存は足さない（境界の crate の直接の依存は workspace の中の 2 つだけ）。
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("Cargo.toml");
    let mut names = Vec::new();
    let mut inside = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line.ends_with("dependencies]");
            continue;
        }
        if inside
            && !line.starts_with('#')
            && let Some((name, _)) = line.split_once('=')
        {
            names.push(name.trim().to_string());
        }
    }
    assert_eq!(names, ["folio", "tsuzuri-contract", "tsuzuri-core"]);
    // 束の定型行の形（裁定の定型行の頭のまま・束 = を挟む）。
    let id = RulingId::new("fx-b.2:20260927T1034Z-1").expect("id");
    let b = RulingId::new("batch:20260927T1034Z-1").expect("id");
    assert_eq!(
        batch::line(&id, &bead(Q2), &b, "a\nb"),
        "裁定 id = fx-b.2:20260927T1034Z-1・問い = fx-b.2・束 = batch:20260927T1034Z-1・逐語 = a\\nb"
    );
}
