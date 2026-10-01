//! 眺めの口の歯（接頭辞 server_view_・設計ノート surface-base 便 e-view の完了の条件）。
//! 偽の bd（bead 8 本の fixture を返す script か、落ちる script）と偽の設計の道具（実物の索引の fixture を
//! 返す script）を歯ごとの一時の dir に置き、event log の fixture を一時の state dir に写す。
//! server は同じ process の thread で 127.0.0.1 の空き port に立てる。
//! 値は口の本文の字と、中核の関数の値を電文にした字で比べる（読み直して比べない）。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::design::{DESIGN_DIR, FOLIO_TIMEOUT};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::{BoxFold, Fold, GraphSource, GraphView};
use tsuzuri_contract::stats::{UnreflectedList, UnreflectedRow};
use tsuzuri_contract::wire;
use tsuzuri_core::graph::{self, Graph, Inputs};
use tsuzuri_core::ledger::{Unreflected, UnreflectedItem};

/// 偽の bd の振る舞い。
#[derive(Clone, Copy)]
enum Bd {
    /// 台帳の fixture を返して rc 0。
    Ok,
    /// 何も返さず rc 1。
    Fails,
}

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

/// 近傍の歯の中心（実物の索引に在る規則の行）。
const CENTER: &str = "R-25";

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
    fn new(name: &str, bd: Bd) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_view")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let files = root.join("files");
        let state = root.join("state");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(repo.join(DESIGN_DIR).join("adr")).expect("設計文書の dir");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::create_dir_all(root.join("calls")).expect("記録の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        for mark in ["issues.jsonl", "interactions.jsonl"] {
            fs::write(repo.join(".beads").join(mark), "{}\n").expect("印の file");
        }
        fs::write(repo.join(DESIGN_DIR).join("rules.yaml"), "rules: []\n").expect("設計文書");
        fs::write(repo.join(DESIGN_DIR).join("adr/ADR-7.yaml"), "id: ADR-7\n").expect("設計文書");
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::write(state.join("fleet/events.jsonl"), read_fixture(EVENTS)).expect("event log");
        let bd_body = match bd {
            Bd::Ok => format!("exec cat '{}'", fixture(LEDGER).display()),
            Bd::Fails => "echo 'bd が落ちる' >&2\nexit 1".to_string(),
        };
        script(&root.join("bd"), &bd_body);
        let (calls, index) = (root.join("calls"), fixture(INDEX));
        script(
            &root.join("folio"),
            &format!(
                "touch '{}'/$$.call\nexec cat '{}'",
                calls.display(),
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

    fn config(&self) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            state_dir: Some(self.state.clone()),
            folio: self.root.join("folio").into(),
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

    /// 偽の設計の道具が撃たれた回数。
    fn folio_calls(&self) -> usize {
        fs::read_dir(self.root.join("calls"))
            .expect("記録の置き場")
            .count()
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

/// 要求を 1 つ撃ち、接続が閉じるまで応答の（状態の code・頭・本文）を読む。
fn request(addr: SocketAddr, raw: &str) -> (u16, String, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(FOLIO_TIMEOUT * 3))
        .expect("timeout");
    s.write_all(raw.as_bytes()).expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse().ok())
        .expect("状態の code");
    (status, head.to_string(), body.to_string())
}

fn get_raw(addr: SocketAddr, path: &str) -> (u16, String, String) {
    request(addr, &format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n"))
}

/// GET を撃ち、200 と JSON を確かめて本文を返す。前後の時刻（epoch 秒）も返す。
fn get(addr: SocketAddr, path: &str) -> (String, u64, u64) {
    let from = now();
    let (status, head, body) = get_raw(addr, path);
    let to = now();
    assert_eq!(status, 200, "{path}: {body}");
    assert!(
        head.contains("Content-Type: application/json"),
        "{path}: {head}"
    );
    (body, from, to)
}

/// 口の電文を契約の型で読む（型は受ける側が決める）。
macro_rules! decode {
    ($body:expr) => {
        wire::decode(&$body).unwrap_or_else(|e| panic!("契約の型の形でない {e}: {}", $body))
    };
}

/// 3 つの字から組んだグラフ。
fn built(ledger: &str) -> Graph {
    graph::build(&Inputs {
        design_index: &read_fixture(INDEX),
        ledger,
        events: &read_fixture(EVENTS),
    })
}

/// 値を電文の字にする（境界の crate は serde に直接依存しないので、型ごとに wire の encode を呼ぶ）。
macro_rules! encoded {
    ($value:expr) => {
        wire::encode(&$value).expect("電文")
    };
}

/// 中核の未反映の一覧を電文の型に写す（歯の側の写し）。
fn wire_list(u: &Unreflected) -> UnreflectedList {
    let rows = |r: &Reading<Vec<UnreflectedItem>>| match r {
        Reading::Known(items) => Reading::Known(
            items
                .iter()
                .map(|i| UnreflectedRow {
                    id: i.id.clone(),
                    title: i.title.clone(),
                    created: i.created,
                })
                .collect(),
        ),
        Reading::Unknown => Reading::Unknown,
    };
    UnreflectedList {
        memos: rows(&u.memos),
        rulings: rows(&u.rulings),
        utterances: rows(&u.utterances),
        stale: u.stale.clone(),
    }
}

#[test]
fn server_view_graph_view_matches_core() {
    let place = Place::new("view", Bd::Ok);
    let addr = place.serve();
    let (body, _, _) = get(addr, "/api/graph/view");
    let want = graph::view(&built(&read_fixture(LEDGER)));
    assert!(want.unread.is_empty(), "{:?}", want.unread);
    assert!(!want.nodes.is_empty());
    assert_eq!(body, encoded!(want), "眺めの電文の字");
    let view: GraphView = decode!(body);
    assert!(view.unread.is_empty(), "{:?}", view.unread);
    assert_eq!(
        place.folio_calls(),
        3,
        "眺めの口 1 回に設計の道具 3 回（索引と要約と裁定の書き出し）"
    );
}

#[test]
fn server_view_around_matches_core() {
    let place = Place::new("around", Bd::Ok);
    let addr = place.serve();
    let g = built(&read_fixture(LEDGER));
    let base = get(addr, "/api/around?id=R-25").0;
    let want = graph::around(&g, CENTER, 2, Fold::None).expect("中心の節点");
    assert!(want.rows.len() > 1, "{want:?}");
    assert_eq!(base, encoded!(want), "既定の段数と畳み");
    for (query, steps, fold) in [
        ("id=R-25&k=1&fold=up", 1, Fold::Up),
        ("id=R-25&k=2&fold=none", 2, Fold::None),
        ("id=R-25&k=3&fold=down", 3, Fold::Down),
        ("fold=both&id=R-25", 2, Fold::Both),
        ("k=3&id=R-25", 3, Fold::None),
    ] {
        let body = get(addr, &format!("/api/around?{query}")).0;
        let want = graph::around(&g, CENTER, steps, fold).expect("中心の節点");
        assert_eq!(body, encoded!(want), "{query}");
    }
    // query の値は %XX と + を戻してから読む。
    for query in ["id=R%2D25", "id=%52-25&k=%32", "%69d=R-25&fold=n%6Fne"] {
        assert_eq!(
            get(addr, &format!("/api/around?{query}")).0,
            base,
            "{query}"
        );
    }
}

#[test]
fn server_view_around_refusals() {
    let place = Place::new("refusals", Bd::Ok);
    let addr = place.serve();
    for (query, status, word) in [
        ("", 400, "no-id"),
        ("?k=2", 400, "no-id"),
        ("?id=", 400, "no-id"),
        ("?id", 400, "no-id"),
        ("?id=&k=0&fold=all", 400, "no-id"),
        ("?id=R-25&k=0", 400, "steps"),
        ("?id=R-25&k=4", 400, "steps"),
        ("?id=R-25&k=x", 400, "steps"),
        ("?id=R-25&k=", 400, "steps"),
        ("?id=R-25&k=-1", 400, "steps"),
        ("?id=R-25&k=1.0", 400, "steps"),
        ("?id=R-25&k=9&fold=all", 400, "steps"),
        ("?id=R-25&fold=all", 400, "fold"),
        ("?id=R-25&fold=", 400, "fold"),
        ("?id=R-25&fold=UP", 400, "fold"),
        ("?id=R-25&k=1&fold=x", 400, "fold"),
        ("?id=no-such-node", 404, "no-node"),
        ("?id=no-such-node&k=1&fold=up", 404, "no-node"),
        ("?id=R+25", 404, "no-node"),
    ] {
        let (got, head, body) = get_raw(addr, &format!("/api/around{query}"));
        assert_eq!((got, body.as_str()), (status, word), "{query}");
        assert!(head.contains("Content-Type: text/plain"), "{query}: {head}");
    }
}

#[test]
fn server_view_unreflected_matches_core() {
    let place = Place::new("unreflected", Bd::Ok);
    let addr = place.serve();
    let ledger = read_fixture(LEDGER);
    let (body, _, _) = get(addr, "/api/unreflected");
    let list: UnreflectedList = decode!(body);
    // state dir の局面の出力が無いので、台帳が読めても 3 種とも「まだ分からない」（行 c-unref-lc）。
    assert_eq!(
        (list.memos, list.rulings, list.utterances),
        (Reading::Unknown, Reading::Unknown, Reading::Unknown),
        "{body}"
    );
    assert_eq!(
        encoded!(wire_list(&tsuzuri_core::ledger::unreflected(
            &ledger, "", None
        ))),
        body,
        "未反映の電文の字が中核の値と違う"
    );
    assert_eq!(place.folio_calls(), 0, "未反映の口は設計の道具を撃たない");
}

#[test]
fn server_view_bd_fails_unknown() {
    let place = Place::new("bd-fails", Bd::Fails);
    let addr = place.serve();
    let (body, _, _) = get(addr, "/api/unreflected");
    let list: UnreflectedList = decode!(body);
    assert_eq!(
        (list.memos, list.rulings, list.utterances),
        (Reading::Unknown, Reading::Unknown, Reading::Unknown),
        "{body}"
    );
    let (body, _, _) = get(addr, "/api/graph/view");
    let view: GraphView = decode!(body);
    assert_eq!(view.unread, vec![GraphSource::Ledger], "{body}");
    assert_eq!(body, encoded!(graph::view(&built(""))), "眺めの電文の字");
    // 台帳が読めなくても設計の索引の節点の近傍は返す。
    let body = get(addr, "/api/around?id=R-25").0;
    let want = graph::around(&built(""), CENTER, 2, Fold::None).expect("中心の節点");
    assert_eq!(body, encoded!(want));
}

/// id の字 : と # と空白を %XX にする（query に置く字）。
fn pct(id: &str) -> String {
    id.replace(':', "%3A")
        .replace('#', "%23")
        .replace(' ', "%20")
}

/// 口の query の open は字 , で分けて眺めに渡す（行 c-graph-fold）。
#[test]
fn gtuck_route_open_query() {
    let place = Place::new("tuck", Bd::Ok);
    let addr = place.serve();
    let g = built(&read_fixture(LEDGER));
    let base = graph::view(&g);
    let id = base
        .nodes
        .iter()
        .find(|n| n.fold == BoxFold::Folded)
        .map(|n| n.node.id.clone())
        .expect("畳んだ箱");
    let body = get(addr, &format!("/api/graph/view?open={}", pct(&id))).0;
    let asks = vec![id.clone()];
    assert_eq!(body, encoded!(graph::view_open(&g, &asks)), "{id}");
    let view: GraphView = decode!(body);
    assert_eq!(view.open, asks);
    for path in ["/api/graph/view?open=no-such", "/api/graph/view"] {
        assert_eq!(get(addr, path).0, encoded!(base), "{path}");
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

const ROUTES: [&str; 3] = [
    "/api/graph/view",
    "/api/around?id=R-25&k=3&fold=none",
    "/api/unreflected",
];

#[test]
fn server_view_routes_write_nothing() {
    let place = Place::new("bytes", Bd::Ok);
    let addr = place.serve();
    let before = (tree(&place.repo), tree(&place.state));
    for path in ROUTES.into_iter().chain(["/api/around?id=no-such-node"]) {
        let (status, _, body) = get_raw(addr, path);
        assert!(status == 200 || status == 404, "{path}: {status} {body}");
    }
    assert!(
        before == (tree(&place.repo), tree(&place.state)),
        "口を読んだ後に repo か state dir の byte が変わる"
    );
}

#[test]
fn server_view_non_get_405_writes_nothing() {
    let place = Place::new("method", Bd::Ok);
    let addr = place.serve();
    let before = (tree(&place.repo), tree(&place.state));
    for path in ROUTES {
        for method in ["POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"] {
            let body = r#"{"memos":"unknown"}"#;
            let (status, head, _) = request(
                addr,
                &format!(
                    "{method} {path} HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                ),
            );
            assert_eq!(status, 405, "{method} {path}");
            assert!(head.contains("Allow: GET"), "{head}");
        }
    }
    assert!(
        before == (tree(&place.repo), tree(&place.state)),
        "405 の後に repo か state dir の byte が変わる"
    );
    assert_eq!(place.folio_calls(), 0, "405 で設計の道具を撃つ");
}
