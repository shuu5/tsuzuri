//! 行 c-bead-route の歯（境界・接頭辞 bvfroute_）: 口 GET /api/beads は bead の事実の一覧（BeadFacts）を返す。
//! 台帳の字は口 /api/ledger と同じ読みから組み、読めなければ行は Unknown、読みが落ちれば /api/ledger と同じ古さの印を付ける。
//! 偽の bd は歯ごとの作業場の sh の script で、落とす印の file（down）が在れば rc 1、無ければ置いた字の file を cat する。
//! 作業場は CARGO_TARGET_TMPDIR の下に歯ごとに作り、server は同じ process の thread で 127.0.0.1 の空き port に立てる。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BeadFact, BeadFacts, BeadId, LedgerList, READ_AGE_HEADER};
use tsuzuri_contract::wire;

/// fixture の fx-hub.7 の created_at（2026-09-22T01:00:00Z）の epoch 秒。
const CREATED_7: u64 = 1_790_038_800;

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// 読める台帳の字（bead 8 本・fx-hub.7 は fx-hub.6 を blocks に持つ）。
fn fixture() -> String {
    fs::read_to_string(manifest("../../tests/fixtures/ledger/bd-list-8.json")).expect("fixture")
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("見本の bead id")
}

/// 書いてから名を移す（撃たれている script や読まれている file を書きかけで見せない）。
fn put(path: &Path, text: &str, mode: u32) {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text).expect("file を書く");
    fs::set_permissions(&tmp, fs::Permissions::from_mode(mode)).expect("file の権限");
    fs::rename(&tmp, path).expect("file を移す");
}

/// 歯ごとの作業場（repo・面の file・偽の bd と、その落とす印と置いた字）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
}

impl Place {
    /// 偽の bd が fixture の字を出す作業場（`down` なら初めから落ちている）で server を立てる。
    fn serve(name: &str, down: bool) -> (Place, SocketAddr) {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("bvfroute")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, files) = (root.join("repo"), root.join("files"));
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        fs::write(repo.join(".beads/issues.jsonl"), "{}\n").expect("印の file");
        let place = Place { root, repo };
        put(&place.root.join("out.json"), &fixture(), 0o644);
        let at = place.root.display();
        put(
            &place.root.join("bd"),
            &format!(
                "#!/bin/sh\nif [ -e '{at}/down' ]; then exit 1; fi\nexec cat '{at}/out.json'\n"
            ),
            0o755,
        );
        if down {
            place.down();
        }
        let config = Config {
            bd: place.root.join("bd").into(),
            folio: place.root.join("no-such-folio").into(),
            scribe2: place.root.join("no-such-scribe2").into(),
            state_dir: None,
            ..Config::new(
                place.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                files,
            )
        };
        let server = Server::bind(&config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        (place, addr)
    }

    /// 偽の bd を落とす（落とす印の file を置く）。
    fn down(&self) {
        fs::write(self.root.join("down"), "").expect("落とす印");
    }

    /// 偽の bd を戻す（落とす印の file を消す）。
    fn up(&self) {
        fs::remove_file(self.root.join("down")).expect("落とす印を消す");
    }

    /// 印の file（issues.jsonl）に 1 行を足す（変化の見張りに読ませる）。
    fn touch(&self) {
        let path = self.repo.join(".beads/issues.jsonl");
        let was = fs::read_to_string(&path).expect("印の file");
        fs::write(&path, format!("{was}{{}}\n")).expect("印の file");
    }
}

struct Reply {
    status: u16,
    head: String,
    body: String,
}

impl Reply {
    /// 頭 X-Tz-Read-Age の値（名の大小は問わない・無ければ None）。
    fn age(&self) -> Option<String> {
        let name = format!("{}:", READ_AGE_HEADER.to_ascii_lowercase());
        self.head.lines().find_map(|l| {
            l.to_ascii_lowercase()
                .starts_with(&name)
                .then(|| l[name.len()..].trim().to_string())
        })
    }

    /// 本文を bead の事実の一覧として読む。
    fn facts(&self) -> BeadFacts {
        assert_eq!(self.status, 200, "{}", self.body);
        wire::decode(&self.body)
            .unwrap_or_else(|e| panic!("BeadFacts の形でない {e}: {}", self.body))
    }
}

fn get(addr: SocketAddr, path: &str) -> Reply {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
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

/// 頭の値を秒の整数として読む。
fn secs(path: &str, age: Option<String>) -> u64 {
    let age = age.unwrap_or_else(|| panic!("{path} に頭が無い"));
    age.parse()
        .unwrap_or_else(|_| panic!("{path} の頭の値が整数でない: {age}"))
}

#[test]
fn bvfroute_shape_on_fixture() {
    let (_place, addr) = Place::serve("shape", false);
    let Reading::Known(rows) = get(addr, "/api/beads").facts().rows else {
        panic!("読める台帳で rows が Unknown");
    };
    let list: LedgerList = wire::decode(&get(addr, "/api/ledger").body).expect("一覧の形");
    let Reading::Known(ledger) = list.rows else {
        panic!("読める台帳で一覧が Unknown");
    };
    let ids: Vec<&BeadId> = rows.iter().map(|r| &r.id).collect();
    let listed: Vec<&BeadId> = ledger.iter().map(|r| &r.id).collect();
    assert_eq!(ids, listed, "行の id と順が /api/ledger の一覧と同じ");
    assert_eq!(rows.len(), 8);
    let blocked = BeadFact {
        id: bead("fx-hub.7"),
        created_at: Some(CREATED_7),
        short: "3".into(),
        short_set: false,
        summary: "fixture の bead（本文は見本の字）".into(),
        blocks: vec![bead("fx-hub.6")],
    };
    assert_eq!(rows.first(), Some(&blocked));
    let short = |id: &str| {
        rows.iter()
            .find(|r| r.id.as_str() == id)
            .map(|r| r.short.as_str())
    };
    assert_eq!(short("fx-hub.4"), Some("問い"));
    assert_eq!(short("fx-hub"), Some("(EPIC) fixture の根"));
    assert!(
        rows.iter()
            .filter(|r| r.id != blocked.id)
            .all(|r| r.blocks.is_empty())
    );
}

#[test]
fn bvfroute_unknown_when_unread() {
    let (_place, addr) = Place::serve("unread", true);
    let reply = get(addr, "/api/beads");
    assert_eq!(reply.facts().rows, Reading::Unknown, "一度も読めない台帳");
    let list: LedgerList = wire::decode(&get(addr, "/api/ledger").body).expect("一覧の形");
    assert_eq!(list.rows, Reading::Unknown, "/api/ledger も Unknown");
}

#[test]
fn bvfroute_aged_like_ledger() {
    let (place, addr) = Place::serve("aged", false);
    let before = get(addr, "/api/beads");
    assert_eq!(before.age(), None, "{}", before.head);
    assert!(matches!(before.facts().rows, Reading::Known(_)));
    // 口は見張りの読みの字を返すので、印を動かして見張りの落ちた読みを待つ。
    place.down();
    place.touch();
    let until = Instant::now() + Duration::from_millis(2500);
    while get(addr, "/api/ledger").age().is_none() {
        assert!(
            Instant::now() < until,
            "落として 2.5 秒の後も /api/ledger に頭が無い"
        );
        thread::sleep(Duration::from_millis(50));
    }
    let held = get(addr, "/api/beads");
    assert!(secs("/api/beads", held.age()) <= 59, "{}", held.head);
    assert_eq!(held.body, before.body, "落ちた間は最後に読めた字から組む");
    // 戻して印を動かせば、/api/ledger と同じく頭が消える。
    place.up();
    place.touch();
    let until = Instant::now() + Duration::from_secs(5);
    while get(addr, "/api/ledger").age().is_some() {
        assert!(
            Instant::now() < until,
            "戻して 5 秒の後も /api/ledger に頭が在る"
        );
        thread::sleep(Duration::from_millis(100));
    }
    let back = get(addr, "/api/beads");
    assert_eq!(back.age(), None, "{}", back.head);
    assert_eq!(back.body, before.body);
}
