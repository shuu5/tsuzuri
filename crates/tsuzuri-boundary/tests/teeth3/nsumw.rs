//! 要約の読みの歯（接頭辞 nsumw_・設計ノート surface-wave6 行 c-summary-wire の完了の条件）。
//! 偽の bd（bead 8 本の fixture を返す script）と偽の設計の道具（引数に --summary が無ければ実物の索引の
//! fixture を返し、在れば作業場の振る舞いに従う script）を歯ごとの作業場に置き、event log の fixture を
//! 作業場の state dir に写す。server は同じ process の thread で 127.0.0.1 の空き port に立て、tz は binary を撃つ。
//! 値は口の本文の字と、中核の関数の値を電文にした字で比べる。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::design::{DESIGN_DIR, Design, FOLIO_ARGS, FOLIO_TIMEOUT, SUMMARY_ARGS};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::graph::{Fold, GraphDoc, GraphNode};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::{self, Graph, Inputs};

fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(rel)
}

fn read_fixture(rel: &str) -> String {
    fs::read_to_string(fixture(rel)).expect("fixture")
}

const LEDGER: &str = "ledger/bd-list-8.json";
const INDEX: &str = "graph/real/design-index.tsv";
const EVENTS: &str = "graph/real/events.jsonl";

/// folio の要約の字（3 行・A-1 は file が索引と違うので写らない）。
const SUMW: &str = concat!(
    r#"{"id":"R-25","kind":"規則行","file":"rules.yaml","line":94,"title":"直接依存の予算","plain":null,"eng":"直接依存の予算。"}"#,
    "\n",
    r#"{"id":"FR15","kind":"要件","file":"srs.yaml","line":242,"title":"概要は data で持つ","plain":"説明は文書に書きます。","eng":"概要は data で持つ。"}"#,
    "\n",
    r#"{"id":"A-1","kind":"条","file":"rules.yaml","line":5,"title":"確認","plain":"違う file です。","eng":null}"#,
    "\n",
);

/// 着地済みの行と第 3 波から第 7 波の行の verify の語（歯の名が含んではならない部分の字）。
const WORDS: [&str; 92] = [
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
    "nsum_",
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

/// 偽の設計の道具が --summary で撃たれたときの振る舞い。
#[derive(Clone, Copy)]
enum Summ {
    /// 作業場の SUMW の file を返して rc 0。
    Lines,
    /// 標準エラーに字を出して rc 2（--summary を知らない folio）。
    Refuse,
    /// 返さない（`FOLIO_TIMEOUT` より長く眠る）。
    Hang,
}

/// 歯ごとの作業場（repo の置き場・面の file の置き場・state dir・偽の bd と偽の設計の道具）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    state: PathBuf,
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

impl Place {
    fn new(name: &str, summ: Summ) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("nsumw")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let files = root.join("files");
        let state = root.join("state");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(repo.join(DESIGN_DIR)).expect("設計文書の dir");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        for mark in ["issues.jsonl", "interactions.jsonl"] {
            fs::write(repo.join(".beads").join(mark), "{}\n").expect("印の file");
        }
        fs::write(repo.join(DESIGN_DIR).join("rules.yaml"), "rules: []\n").expect("設計文書");
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::write(state.join("fleet/events.jsonl"), read_fixture(EVENTS)).expect("event log");
        fs::write(root.join("sumw.jsonl"), SUMW).expect("要約の字");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", fixture(LEDGER).display()),
        );
        let (r, index) = (root.display(), fixture(INDEX));
        let summary = match summ {
            Summ::Lines => format!("exec cat '{r}/sumw.jsonl'"),
            Summ::Refuse => "echo 'folio: unknown flag --summary' >&2\nexit 2".to_string(),
            Summ::Hang => "exec sleep 30".to_string(),
        };
        script(
            &root.join("folio"),
            &format!(
                "printf '%s\\n' \"$*\" >> '{r}/folio.calls'\n\
                 for a in \"$@\"; do\n\
                 if [ \"$a\" = --summary ]; then\n{summary}\nfi\n\
                 done\n\
                 exec cat '{}'",
                index.display()
            ),
        );
        Place {
            root,
            repo,
            files,
            state,
        }
    }

    fn folio(&self) -> PathBuf {
        self.root.join("folio")
    }

    fn config(&self) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            state_dir: Some(self.state.clone()),
            folio: self.folio().into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    fn serve(&self) -> SocketAddr {
        let server = Server::bind(&self.config()).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }

    /// tz graph を撃ち（`head` は --design などの前の引数）、rc と標準出力を返す。
    fn tz(&self, head: &[&str]) -> (Option<i32>, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_tz"))
            .arg("graph")
            .args(head)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--bd")
            .arg(self.root.join("bd"))
            .arg("--folio")
            .arg(self.folio())
            .arg("--state-dir")
            .arg(&self.state)
            .output()
            .expect("tz を撃つ");
        (
            out.status.code(),
            String::from_utf8(out.stdout).expect("標準出力の字"),
        )
    }
}

/// GET を撃ち、200 を確かめて本文を返す。
fn get(addr: SocketAddr, path: &str) -> String {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(FOLIO_TIMEOUT * 3))
        .expect("timeout");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    let status = head.split(' ').nth(1).expect("状態の code");
    assert_eq!(status, "200", "{path}: {body}");
    body.to_string()
}

/// 口の電文を契約の型で読む（型は受ける側が決める）。
macro_rules! decode {
    ($body:expr) => {
        wire::decode(&$body).unwrap_or_else(|e| panic!("契約の型の形でない {e}: {}", $body))
    };
}

/// 値を電文の字にする（境界の crate は serde に直接依存しないので、型ごとに wire の encode を呼ぶ）。
macro_rules! encoded {
    ($value:expr) => {
        wire::encode(&$value).expect("電文")
    };
}

/// 3 つの fixture の字を build しただけのグラフ。
fn built() -> Graph {
    graph::build(&Inputs {
        design_index: &read_fixture(INDEX),
        ledger: &read_fixture(LEDGER),
        events: &read_fixture(EVENTS),
    })
}

/// 3 つの fixture の字を build し、SUMW で add_summary を撃ったグラフ G。
fn with_summary() -> Graph {
    let mut g = built();
    assert!(graph::build::add_summary(&mut g, SUMW), "要約が読めない");
    g
}

fn node<'a>(nodes: &'a [GraphNode], id: &str) -> &'a GraphNode {
    nodes
        .iter()
        .find(|n| n.id == id)
        .unwrap_or_else(|| panic!("節点 {id} が無い"))
}

/// 節点の行と 2 つの概要。
fn fields(n: &GraphNode) -> (Option<u32>, Option<&str>, Option<&str>) {
    (n.line, n.plain.as_deref(), n.eng.as_deref())
}

const R25: (Option<u32>, Option<&str>, Option<&str>) = (Some(94), None, Some("直接依存の予算。"));

/// 全部の節点の 3 つの欄が無し。
fn all_null(doc: &GraphDoc) -> bool {
    doc.nodes.iter().all(|n| fields(n) == (None, None, None))
}

#[test]
fn nsumw_summary_argv() {
    let design = Design::new("/r", "folio");
    assert_eq!(SUMMARY_ARGS, ["graph", "--print", "--summary", "--dir"]);
    assert_eq!(
        design.summary_args(),
        ["graph", "--print", "--summary", "--dir", "/r/design-intent"].map(std::ffi::OsString::from)
    );
    assert_eq!(FOLIO_ARGS, ["graph", "--print", "--dir"]);
    assert_eq!(
        design.args(),
        ["graph", "--print", "--dir", "/r/design-intent"].map(std::ffi::OsString::from)
    );
}

#[test]
fn nsumw_summary_reads_output() {
    let index = read_fixture(INDEX);
    let lines = Place::new("reads-lines", Summ::Lines);
    let design = Design::new(&lines.repo, lines.folio());
    assert_eq!(design.summary().as_deref(), Some(SUMW));
    assert_eq!(design.text().as_deref(), Some(index.as_str()));
    let refuse = Place::new("reads-refuse", Summ::Refuse);
    let design = Design::new(&refuse.repo, refuse.folio());
    assert_eq!(design.summary(), None, "--summary を知らない folio");
    assert_eq!(design.text().as_deref(), Some(index.as_str()));
    let missing = Design::new(&refuse.repo, refuse.root.join("no-such-folio"));
    assert_eq!(missing.summary(), None, "無い program");
}

#[test]
fn nsumw_routes_carry_fields() {
    let place = Place::new("routes", Summ::Lines);
    let addr = place.serve();
    let g = with_summary();
    let doc: GraphDoc = decode!(get(addr, "/api/graph"));
    assert!(doc.unread.is_empty(), "{:?}", doc.unread);
    assert_eq!(doc.nodes, g.nodes, "節点");
    assert_eq!(fields(node(&doc.nodes, "R-25")), R25, "R-25");
    assert_eq!(
        fields(node(&doc.nodes, "FR15")),
        (
            Some(242),
            Some("説明は文書に書きます。"),
            Some("概要は data で持つ。")
        ),
        "FR15"
    );
    assert_eq!(
        fields(node(&doc.nodes, "A-1")),
        (None, None, None),
        "A-1 は file が違う"
    );

    let body = get(addr, "/api/around?id=R-25");
    let want = graph::around(&g, "R-25", 2, Fold::None).expect("中心の節点");
    assert_eq!(body, encoded!(want), "近傍の電文の字");
    let center = want
        .rows
        .iter()
        .find(|r| r.col == 0)
        .expect("列 0 の行");
    assert_eq!(center.node.id, "R-25");
    assert_eq!(fields(&center.node), R25, "近傍の中心");
}

#[test]
fn nsumw_refused_summary_stays_null() {
    let place = Place::new("refuse", Summ::Refuse);
    let addr = place.serve();
    let doc: GraphDoc = decode!(get(addr, "/api/graph"));
    assert!(doc.unread.is_empty(), "{:?}", doc.unread);
    let design = graph::Source::Design.kinds();
    assert!(
        doc.nodes.iter().any(|n| design.contains(&n.kind)),
        "設計の節点が無い"
    );
    assert!(all_null(&doc), "要約が読めないのに行か概要が在る");
    assert_eq!(doc.nodes, built().nodes, "節点");
}

#[test]
fn nsumw_hung_summary_keeps_index() {
    let place = Place::new("hang", Summ::Hang);
    let addr = place.serve();
    let started = Instant::now();
    let doc: GraphDoc = decode!(get(addr, "/api/graph"));
    let took = started.elapsed();
    assert!(took < FOLIO_TIMEOUT + Duration::from_secs(2), "{took:?}");
    assert!(doc.unread.is_empty(), "{:?}", doc.unread);
    assert!(all_null(&doc), "要約が読めないのに行か概要が在る");
}

#[test]
fn nsumw_cli_doc_carries_fields() {
    let place = Place::new("cli", Summ::Lines);
    let g = with_summary();
    let (rc, out) = place.tz(&[]);
    assert_eq!(rc, Some(0), "{out}");
    let doc: GraphDoc = decode!(out);
    assert_eq!(doc.nodes, g.nodes, "節点");
    let (rc, out) = place.tz(&["--design"]);
    assert_eq!(rc, Some(0), "{out}");
    let doc: GraphDoc = decode!(out);
    assert_eq!(fields(node(&doc.nodes, "R-25")), R25, "--design の R-25");
}

#[test]
fn nsumw_own_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth3/nsumw.rs"))
        .expect("歯の file");
    let mut names = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let name = lines
            .next()
            .and_then(|l| l.trim().strip_prefix("fn "))
            .and_then(|l| l.split('(').next())
            .unwrap_or_else(|| panic!("#[test] の次の行が fn でない"));
        names.push(name.to_string());
    }
    assert_eq!(names.len(), 7, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("nsumw_")
            .unwrap_or_else(|| panic!("{name} が nsumw_ で始まらない"));
        for word in WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
