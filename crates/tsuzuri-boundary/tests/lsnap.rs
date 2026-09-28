//! GET の口が見張りの読みの字を返す歯（接頭辞 lsnap_・設計ノート surface-wave12f 行 e-snap の完了の条件）。
//! 偽の bd は撃たれるたびに記録の file に 1 行を足し、作業場に slow の file が在れば 1 秒眠り、
//! down の file が在れば rc 1、無ければ置いた字の file を出す script。
//! 作業場は CARGO_TARGET_TMPDIR の下に歯ごとに作り、server は同じ process の thread で 127.0.0.1 の空き port に立てる。

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::coalesce::Coalesce;
use tsuzuri_boundary::server::ledger::{
    Got, JOURNAL, MANIFEST, METADATA, NOMS, STORE_DIR, Source,
};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::LedgerList;
use tsuzuri_contract::wire;

/// 着地済みの verify の filter の語（新しい歯の名から lsnap_ を除いた字は、どれも部分の字として含まない）。
const FILTERS: &[&str] = &[
    "aaround_",
    "accept_",
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
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gbnote_",
    "gfresh_",
    "ghb_",
    "glabel_",
    "gnav_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "hsym_",
    "iclose_",
    "ilink_",
    "kcli_",
    "klink_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lhome_",
    "lspark_",
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
    "pclosed_",
    "pfold_",
    "pipe_",
    "plimit_",
    "pmore_",
    "pquest_",
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
    "lstore_",
    "lidle_",
];

/// 台帳を読む 10 の GET の口。
const ROUTES: [&str; 10] = [
    "/api/ledger",
    "/api/ledger/fx-hub.3",
    "/api/metrics",
    "/api/questions",
    "/api/pipeline",
    "/api/next",
    "/api/graph",
    "/api/graph/view",
    "/api/around?id=fx-hub.3",
    "/api/unreflected",
];

/// bead 8 本の fixture（bd の読み取りの口の出力の形）。
fn fixture() -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ledger/bd-list-8.json"),
    )
    .expect("fixture")
}

/// fixture の 1 本目の bead の object だけを持つ配列の字。
fn one() -> String {
    let all = fixture();
    let first = &all[..all.find("\n  },\n").expect("1 本目の終わり") + "\n  }".len()];
    format!("{first}\n]\n")
}

/// 歯ごとの作業場（repo の置き場・面の file の置き場・偽の bd と、その記録と置いた字）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("lsnap")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, files) = (root.join("repo"), root.join("files"));
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(&files).expect("面の file の置き場");
        fs::write(files.join("index.html"), "tz").expect("index.html");
        let r = root.display();
        let bd = root.join("bd");
        fs::write(
            &bd,
            format!(
                "#!/bin/sh\necho x >> '{r}/calls'\n\
                 if [ -e '{r}/slow' ]; then sleep 1; fi\n\
                 if [ -e '{r}/down' ]; then exit 1; fi\n\
                 exec cat '{r}/out.json'\n"
            ),
        )
        .expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
        let place = Place { root, repo, files };
        place.bd_returns(&fixture());
        place
    }

    fn bd(&self) -> PathBuf {
        self.root.join("bd")
    }

    /// 偽の bd が出す字を置く（別の名で書いてから置き替える）。
    fn bd_returns(&self, text: &str) {
        let tmp = self.root.join("out.json.tmp");
        fs::write(&tmp, text).expect("偽の bd の出力");
        fs::rename(&tmp, self.root.join("out.json")).expect("偽の bd の出力を置く");
    }

    /// 作業場の印の file（slow か down）を置く（true）か消す（false）。
    fn flag(&self, name: &str, on: bool) {
        let path = self.root.join(name);
        if on {
            fs::write(path, "").expect("印を置く");
        } else {
            fs::remove_file(path).expect("印を消す");
        }
    }

    /// 偽の bd が撃たれた回（記録の file の行の総数）。
    fn calls(&self) -> usize {
        fs::read_to_string(self.root.join("calls"))
            .unwrap_or_default()
            .lines()
            .count()
    }

    /// repo の .beads に組み込みの store（db の名 t3・JOURNAL だけを名指す MANIFEST と JOURNAL）を置き、
    /// NOMS の path を返す。
    fn store(&self) -> PathBuf {
        let beads = self.repo.join(".beads");
        fs::write(
            beads.join(METADATA),
            r#"{"dolt_mode": "embedded", "dolt_database": "t3"}"#,
        )
        .expect("metadata.json");
        let noms = beads.join(STORE_DIR).join("t3").join(NOMS);
        fs::create_dir_all(&noms).expect("NOMS の dir");
        fs::write(
            noms.join(MANIFEST),
            format!("5:__DOLT__:aaaa:bbbb:0000:{JOURNAL}:10"),
        )
        .expect("manifest");
        fs::write(noms.join(JOURNAL), "0123456789").expect("journal");
        noms
    }

    /// 偽の bd と、無い path の設計の道具と器と、state dir なしで server を立てる。
    fn serve(&self) -> SocketAddr {
        let config = Config {
            bd: self.bd().into(),
            folio: self.root.join("no-such-folio").into(),
            scribe2: self.root.join("no-such-scribe2").into(),
            state_dir: None,
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        };
        let server = Server::bind(&config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }
}

/// GET を 1 つ撃ち、接続が閉じるまで応答の（状態の code・本文）を読む。
fn get(addr: SocketAddr, path: &str) -> (u16, String) {
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
    (status, body.to_string())
}

/// 台帳の一覧の口の行の数（読めなければ None）。
fn rows(reply: &(u16, String)) -> Option<usize> {
    assert_eq!(reply.0, 200, "{}", reply.1);
    let list: LedgerList = wire::decode(&reply.1).expect("契約の型の形");
    match list.rows {
        Reading::Known(rows) => Some(rows.len()),
        Reading::Unknown => None,
    }
}

#[test]
fn lsnap_join_starts_nothing() {
    let shared: Coalesce<String> = Coalesce::new();
    assert_eq!(shared.join(Duration::from_secs(5)), None, "読みが無いのに合流");
    let (lead, short, long, took) = thread::scope(|s| {
        let lead = s.spawn(|| {
            shared.share(Duration::from_secs(5), || {
                thread::sleep(Duration::from_millis(500));
                Some("読めた".to_string())
            })
        });
        thread::sleep(Duration::from_millis(100));
        let short = s.spawn(|| {
            let started = Instant::now();
            let got = shared.join(Duration::from_millis(100));
            (got, started.elapsed())
        });
        let long = s.spawn(|| shared.join(Duration::from_secs(5)));
        let (short, took) = short.join().expect("短い合流の thread");
        (
            lead.join().expect("読みの thread"),
            short,
            long.join().expect("長い合流の thread"),
            took,
        )
    });
    assert_eq!(short, Some(None), "上限を越えた合流");
    assert!(
        took >= Duration::from_millis(100) && took < Duration::from_millis(400),
        "{took:?}"
    );
    assert_eq!(long, Some(Some("読めた".to_string())));
    assert_eq!(lead.as_deref(), Some("読めた"));
    assert_eq!(shared.join(Duration::from_secs(5)), None, "終わった読みに合流");
}

#[test]
fn lsnap_watched_source_reads_once() {
    let place = Place::new("source");
    let plain = Source::new(place.repo.clone(), place.bd());
    assert!(!plain.is_watched());
    let source = plain.watched();
    assert!(source.is_watched());
    // 一度も読みを終えていなければ読む。
    assert_eq!(
        source.got(),
        Got {
            text: Some(fixture()),
            stale: None
        }
    );
    assert_eq!(place.calls(), 1);
    // 字を替えても、読まなければ最後に終えた読みの字（bd を撃たない）。
    place.bd_returns(&one());
    assert_eq!(source.got().text, Some(fixture()));
    assert_eq!(source.text(), Some(fixture()));
    assert_eq!(place.calls(), 1);
    // 見張りの読みの後は、その字。
    assert!(
        matches!(source.read(), Reading::Known(r) if r.len() == 1),
        "1 本の台帳"
    );
    assert_eq!(
        source.got(),
        Got {
            text: Some(one()),
            stale: None
        }
    );
    assert_eq!(place.calls(), 2);
    // 見張りの読みが落ちれば、最後に読めた字と、その時刻。
    place.flag("down", true);
    let before = Instant::now();
    assert_eq!(source.read(), Reading::Unknown);
    let held = source.got();
    assert_eq!(held.text, Some(one()));
    let at = held.stale.expect("落ちた読みの時刻");
    assert!(at <= before, "{at:?} は落とす前の時刻 {before:?} より後");
    assert_eq!(place.calls(), 3);
    // watched でない Source は撃つたびに読む。
    place.flag("down", false);
    let alone = Source::new(place.repo.clone(), place.bd());
    alone.got();
    alone.got();
    assert_eq!(place.calls(), 5);
}

#[test]
fn lsnap_routes_call_no_bd() {
    let place = Place::new("routes");
    place.store();
    let addr = place.serve();
    assert_eq!(place.calls(), 1, "起動の読み");
    let mut before = Vec::new();
    for route in ROUTES {
        let reply = get(addr, route);
        assert_eq!(reply.0, 200, "{route}: {}", reply.1);
        before.push(reply.1);
    }
    assert_eq!(place.calls(), 1, "順に撃った 10 の口");
    // 印を動かさずに字を替え、10 の口を同時に撃つ。
    place.bd_returns(&one());
    let replies: Vec<(u16, String)> = thread::scope(|s| {
        let handles: Vec<_> = ROUTES
            .iter()
            .map(|route| s.spawn(move || get(addr, route)))
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("要求の thread"))
            .collect()
    });
    for ((route, reply), was) in ROUTES.iter().zip(&replies).zip(&before) {
        assert_eq!(reply.0, 200, "{route}: {}", reply.1);
        if matches!(*route, "/api/ledger" | "/api/questions" | "/api/ledger/fx-hub.3") {
            assert_eq!(&reply.1, was, "{route}");
        }
    }
    assert_eq!(place.calls(), 1, "同時に撃った 10 の口");
    assert_eq!(rows(&get(addr, "/api/ledger")), Some(8));
}

#[test]
fn lsnap_routes_wait_for_read() {
    let place = Place::new("wait");
    let noms = place.store();
    let addr = place.serve();
    let base = place.calls();
    place.flag("slow", true);
    place.bd_returns(&one());
    OpenOptions::new()
        .append(true)
        .open(noms.join(JOURNAL))
        .expect("journal を開く")
        .write_all(b"x")
        .expect("journal に足す");
    // 見張りの読みが始まる（記録の行が増える）まで 5 ミリ秒ごとに見る。
    let until = Instant::now() + Duration::from_secs(3);
    while place.calls() == base {
        assert!(Instant::now() < until, "見張りの読みが始まらない");
        thread::sleep(Duration::from_millis(5));
    }
    let started = Instant::now();
    let reply = get(addr, "/api/ledger");
    let took = started.elapsed();
    assert_eq!(rows(&reply), Some(1), "{}", reply.1);
    assert!(
        took >= Duration::from_millis(500) && took < Duration::from_millis(2500),
        "{took:?}"
    );
    assert_eq!(place.calls(), base + 1, "起動の後の bd の回");
}

#[test]
fn lsnap_own_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lsnap.rs"))
        .expect("自分の file");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            w[1].trim()
                .strip_prefix("fn ")
                .and_then(|r| r.split('(').next())
                .unwrap_or_else(|| panic!("test の属性の次が fn でない: {}", w[1]))
        })
        .collect();
    assert!(names.len() >= 5, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("lsnap_")
            .unwrap_or_else(|| panic!("lsnap_ で始まらない: {name}"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
