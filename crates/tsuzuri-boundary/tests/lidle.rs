//! 受け手が 0 人の間は印の動かない読み直しをしない歯（接頭辞 lidle_・設計ノート surface-wave12f 行 e-idle の完了の条件）。
//! 作業場は CARGO_TARGET_TMPDIR の下に歯ごとに作る。server の歯の偽の bd は撃たれるたびに記録の file に 1 行を足して
//! 字 [] を出す script で、server は同じ process の thread で 127.0.0.1 の空き port に立てる。
#![cfg(test)]

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::events::{Hub, Timing};
use tsuzuri_boundary::server::{Config, Server};

/// 着地済みの verify の filter の語（新しい歯の名から lidle_ を除いた字は、どれも部分の字として含まない）。
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
    "lsnap_",
];

/// 歯ごとの作業場（前の歯の残りを消して作り直す）。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("lidle")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("作業場");
    root
}

/// file に 1 行を足した字を隣の file に書いてから rename で置き替える
/// （見張りから書きかけは見えず、更新時刻と長さが 1 度に動く）。
fn append_line(path: &Path) {
    let mut text = fs::read_to_string(path).unwrap_or_default();
    text.push_str("{}\n");
    let name = path.file_name().expect("file の名").to_string_lossy();
    let tmp = path.with_file_name(format!("{name}.tmp"));
    fs::write(&tmp, text).expect("隣の file");
    fs::rename(&tmp, path).expect("置き替える");
}

/// `until` の前に `ok` が立つまで 5 ミリ秒ごとに見る（立てば true）。
fn wait(until: Instant, ok: impl Fn() -> bool) -> bool {
    loop {
        if ok() {
            return true;
        }
        if Instant::now() >= until {
            return false;
        }
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn lidle_listeners_count() {
    let hub = Hub::default();
    assert_eq!(hub.listeners(), 0);
    let a = hub.subscribe();
    let b = hub.subscribe();
    assert_eq!(hub.listeners(), 2);
    drop(b);
    assert_eq!(hub.listeners(), 1);
    hub.ledger_changed(7);
    assert_eq!(
        a.try_recv().expect("1 件"),
        "id: 1\nevent: ledger-changed\ndata: {\"at\":7}\n\n"
    );
    drop(a);
    assert_eq!(hub.listeners(), 0);
}

#[test]
fn lidle_watch_idle_then_attach() {
    let root = place("watch");
    let mark = root.join("mark");
    fs::write(&mark, "{}\n").expect("印の file");
    let reads = Arc::new(AtomicUsize::new(0));
    let r = Arc::clone(&reads);
    let timing = Timing {
        poll: Duration::from_millis(20),
        reread: Duration::from_millis(100),
        store_reread: Duration::from_secs(60),
    };
    let hub = Hub::watch(
        vec![mark.clone()],
        move || {
            r.fetch_add(1, Ordering::SeqCst);
            0u32
        },
        timing,
    );
    let n = || reads.load(Ordering::SeqCst);
    // 受け手 0 人の間は印が動かなければ読み直さない。
    thread::sleep(Duration::from_millis(500));
    assert_eq!(n(), 1, "受け手 0 人で印が動かないのに読む");
    // 印が動いても受け手 0 人の間は読まない。
    append_line(&mark);
    thread::sleep(Duration::from_millis(500));
    assert_eq!(n(), 1, "印が動いて受け手 0 人で読む");
    // 受け手が付いた周で 1 回読み、その後は読み直しの間隔ごとに読む。
    let rx = hub.subscribe();
    assert!(
        wait(Instant::now() + Duration::from_millis(300), || n() >= 2),
        "受け手が付いて 300 ミリ秒以内に読まない: {}",
        n()
    );
    assert!(
        wait(Instant::now() + Duration::from_millis(600), || n() >= 4),
        "受け手が居る間に読み直さない: {}",
        n()
    );
    // 受け手が落ちれば読み直しは止まる。
    drop(rx);
    thread::sleep(Duration::from_millis(200));
    let idle = n();
    thread::sleep(Duration::from_millis(500));
    assert_eq!(n(), idle, "受け手が落ちた後も読み直す");
    // また付けば 1 回読む。
    let _rx = hub.subscribe();
    assert!(
        wait(Instant::now() + Duration::from_millis(300), || n() > idle),
        "受け手がまた付いて 300 ミリ秒以内に読まない: {}",
        n()
    );
    let _ = fs::remove_dir_all(&root);
}

/// 偽の bd の記録の file の行の数。
fn calls(root: &Path) -> usize {
    fs::read_to_string(root.join("calls"))
        .unwrap_or_default()
        .lines()
        .count()
}

#[test]
fn lidle_live_reads_on_attach() {
    let root = place("live");
    let (repo, files) = (root.join("repo"), root.join("files"));
    fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
    fs::create_dir_all(&files).expect("面の file の置き場");
    fs::write(files.join("index.html"), "tz").expect("index.html");
    let issues = repo.join(".beads").join("issues.jsonl");
    fs::write(&issues, "{}\n").expect("issues.jsonl");
    let bd = root.join("bd");
    fs::write(
        &bd,
        format!(
            "#!/bin/sh\necho x >> '{}/calls'\necho '[]'\n",
            root.display()
        ),
    )
    .expect("偽の bd");
    fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
    let config = Config {
        bd: bd.into(),
        ..Config::new(
            repo.clone(),
            "127.0.0.1:0".parse().expect("bind 先"),
            files.clone(),
        )
    };
    run_and_count(root, issues, config);
}

/// 起こした server の偽の bd の読みの数を、受け手 0 人の間と口と知らせの口で見る。
fn run_and_count(root: PathBuf, issues: PathBuf, config: Config) {
    let server = Server::bind(&config).expect("起動");
    let addr = server.local_addr().expect("口の住所");
    thread::spawn(move || server.run());
    assert_eq!(calls(&root), 1, "起動の読み");
    // 受け手 0 人で store の無い印の間は、5 秒の読み直しをしない。
    thread::sleep(Duration::from_secs(6));
    assert_eq!(calls(&root), 1, "受け手 0 人で読み直す");
    // 印が動いても受け手 0 人の間は見張りは読まない。
    append_line(&issues);
    thread::sleep(Duration::from_millis(1500));
    assert_eq!(calls(&root), 1, "印が動いて受け手 0 人で読む");
    // 口 /api/ledger の要求が印の遅れを見て 1 回読む。
    let mut r = TcpStream::connect(addr).expect("接続");
    r.write_all(b"GET /api/ledger HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
        .expect("要求");
    let mut body = String::new();
    r.read_to_string(&mut body).expect("応答");
    assert!(body.starts_with("HTTP/1.1 200"), "{body}");
    assert_eq!(calls(&root), 2, "口の読み");
    // 知らせの口を開くと、受け手が付いた周で 1 回読む。
    let mut s = TcpStream::connect(addr).expect("接続");
    s.write_all(
        b"GET /api/surface/events HTTP/1.1\r\nHost: x\r\nAccept: text/event-stream\r\n\r\n",
    )
    .expect("要求");
    s.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    let mut head = [0u8; 256];
    let got = s.read(&mut head).expect("頭を読む");
    assert!(got > 0, "頭の字が無い");
    thread::sleep(Duration::from_millis(1500));
    assert_eq!(calls(&root), 3, "受け手が付いた周の読み");
    let mut again = TcpStream::connect(addr).expect("接続");
    again
        .write_all(b"GET /api/ledger HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
        .expect("要求");
    let mut rest = String::new();
    again.read_to_string(&mut rest).expect("応答");
    assert_eq!(calls(&root), 3, "印が同じなら口は撃たない");
    drop(s);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn lidle_own_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lidle.rs"))
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
    assert!(names.len() >= 4, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("lidle_")
            .unwrap_or_else(|| panic!("lidle_ で始まらない: {name}"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
