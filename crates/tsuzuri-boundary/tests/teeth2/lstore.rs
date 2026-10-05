//! 台帳の store の印の歯（接頭辞 lstore_・設計ノート surface-wave12f 行 e-marks の完了の条件）。
//! 印は metadata.json が名指す store の manifest の字と manifest が名指す file の長さで、
//! bd の読みで動く更新時刻と journal.idx は見ない（行 e-mark-meta）。
//! 作業場は CARGO_TARGET_TMPDIR の下に歯ごとに作る。
#![cfg(test)]

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use crate::common::fixture;
use tsuzuri_boundary::server::events::{Hub, POLL, REREAD, STORE_REREAD, TIMING, Timing, stamp};
use tsuzuri_boundary::server::ledger::{
    JOURNAL, MANIFEST, METADATA, Mark, NOMS, STORE_DIR, Source, TABLE_SUFFIX,
};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::Reading;

/// 着地済みの verify の filter の語（新しい歯の名から lstore_ を除いた字は、どれも部分の字として含まない）。
const FILTERS: [&str; 132] = [
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
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gfresh_",
    "ghb_",
    "glabel_",
    "gnav_",
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
    "lsnap_",
    "lidle_",
];

/// manifest の見本の字（table の file 2 つと journal の本体を名指す）。
const MANIFEST_TEXT: &str =
    "5:__DOLT__:aaaa:bbbb:0000:tbl1:3:tbl2:4:vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv:10";

/// 歯ごとの作業場（中の repo の置き場）。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("lstore")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("repo").join(".beads")).expect("repo の置き場");
    root
}

/// 組み込みの store の db を名指す metadata.json の字。
fn metadata(db: &str) -> String {
    format!(
        r#"{{"database": "dolt", "backend": "dolt", "dolt_mode": "embedded", "dolt_database": "{db}"}}"#
    )
}

/// repo の .beads に db を名指す METADATA を置き、db の dir の NOMS に manifest と 2 つの table の file と
/// journal と journal.idx と空の LOCK を置き、NOMS の path を返す。
fn store(repo: &Path, db: &str) -> PathBuf {
    let beads = repo.join(".beads");
    fs::write(beads.join(METADATA), metadata(db)).expect("metadata.json");
    let noms = beads.join(STORE_DIR).join(db).join(NOMS);
    fs::create_dir_all(&noms).expect("NOMS の dir");
    fs::write(noms.join(MANIFEST), MANIFEST_TEXT).expect("manifest");
    fs::write(noms.join(format!("tbl1{TABLE_SUFFIX}")), "12345").expect("table の file");
    fs::write(noms.join("tbl2"), "abc").expect("table の file");
    fs::write(noms.join(JOURNAL), "0123456789").expect("journal");
    fs::write(noms.join("journal.idx"), "idx").expect("journal.idx");
    fs::write(noms.join("LOCK"), "").expect("LOCK");
    noms
}

/// file に字を足す。
fn append(path: &Path, text: &str) {
    OpenOptions::new()
        .append(true)
        .open(path)
        .expect("足す file を開く")
        .write_all(text.as_bytes())
        .expect("足す");
}

/// bd の読みの見立て（更新時刻だけを動かし、manifest を同じ字で置き替え、journal.idx の長さを替える）。
fn read_like(noms: &Path) {
    let later = SystemTime::now() + Duration::from_secs(5);
    let table = format!("tbl1{TABLE_SUFFIX}");
    for name in [JOURNAL, "journal.idx", &table, "tbl2"] {
        OpenOptions::new()
            .append(true)
            .open(noms.join(name))
            .expect("追記の形で開く")
            .set_modified(later)
            .expect("更新時刻を動かす");
    }
    let tmp = noms.join("nbs_manifest_1");
    let same = fs::read(noms.join(MANIFEST)).expect("manifest の字");
    fs::write(&tmp, same).expect("置き替えの manifest");
    fs::rename(&tmp, noms.join(MANIFEST)).expect("manifest を置き替える");
    fs::write(noms.join("journal.idx"), "idx-longer").expect("journal.idx の長さを替える");
}

#[test]
fn lstore_values_and_dirs() {
    assert_eq!(STORE_REREAD, Duration::from_secs(600));
    assert_eq!(POLL, Duration::from_millis(500));
    assert_eq!(REREAD, Duration::from_secs(5));
    assert_eq!(
        TIMING,
        Timing {
            poll: POLL,
            reread: REREAD,
            store_reread: STORE_REREAD,
        }
    );
    assert_eq!(STORE_DIR, "embeddeddolt");
    assert_eq!(NOMS, ".dolt/noms");
    assert_eq!(MANIFEST, "manifest");
    assert_eq!(JOURNAL, "v".repeat(32));
    assert_eq!(METADATA, "metadata.json");
    assert_eq!(TABLE_SUFFIX, ".darc");
    dirs_and_metadata();
}

/// metadata.json の字と置き方から store の dir と印を読むかを見る。
fn dirs_and_metadata() {
    let root = place("dirs");
    let repo = root.join("repo");
    let meta = repo.join(".beads").join(METADATA);
    let source = Source::new(&repo, "bd");
    assert_eq!(source.store(), None, "metadata.json が無いのに在る");
    let noms = store(&repo, "t3");
    assert_eq!(source.store(), Some(noms.clone()));
    // 組み込みでない置き方と、名の無い・名でない・store の無い db は store の無い置き方。
    for text in [
        r#"{"database": "dolt", "backend": "dolt", "dolt_mode": "server", "dolt_database": "t3"}"#,
        r#"{"database": "dolt", "backend": "dolt", "dolt_mode": "embedded"}"#,
        r#"{"database": "dolt", "backend": "dolt", "dolt_mode": "embedded", "dolt_database": "../t3"}"#,
        r#"{"database": "dolt", "backend": "dolt", "dolt_mode": "embedded", "dolt_database": "x9"}"#,
    ] {
        fs::write(&meta, text).expect("metadata.json");
        assert_eq!(source.store(), None, "{text}");
    }
    // 欄の順と数の値の欄と値が dolt_mode の字の欄とコロンの前後の空白は読みを変えない。
    for text in [
        r#"{"dolt_database": "t3", "dolt_mode": "embedded", "backend": "dolt", "database": "dolt"}"#,
        r#"{"note": "dolt_mode", "schema": 2, "dolt_mode":"embedded", "dolt_database":"t3"}"#,
        r#"{
  "dolt_mode" :  "embedded",
  "version": 1,
  "dolt_database"   :"t3"
}
"#,
    ] {
        fs::write(&meta, text).expect("metadata.json");
        assert_eq!(source.store(), Some(noms.clone()), "{text}");
    }
    fs::remove_file(noms.join(MANIFEST)).expect("manifest を消す");
    assert_eq!(source.store(), None, "manifest が無いのに在る");
    assert!(
        matches!(source.mark(), Mark::Files(_)),
        "manifest が無いのに store の印"
    );
}

#[test]
fn lstore_mark_ignores_mtime() {
    let root = place("mtime");
    let repo = root.join("repo");
    let beads = repo.join(".beads");
    for mark in ["issues.jsonl", "interactions.jsonl"] {
        fs::write(beads.join(mark), "{}\n").expect("印の file");
    }
    let source = Source::new(&repo, "bd");
    let files: Vec<_> = source.marks().iter().map(|m| stamp(m)).collect();
    assert!(files.iter().all(Option::is_some));
    assert_eq!(source.mark(), Mark::Files(files));
    let noms = store(&repo, "t3");
    let first = source.mark();
    assert_eq!(
        first,
        Mark::Store {
            manifest: MANIFEST_TEXT.as_bytes().to_vec(),
            sizes: vec![
                ("tbl1".to_string(), Some(5)),
                ("tbl2".to_string(), Some(3)),
                (JOURNAL.to_string(), Some(10)),
            ],
        }
    );
    // 更新時刻と manifest の置き替えと journal.idx と jsonl の動きは印を動かさない。
    read_like(&noms);
    append(&beads.join("issues.jsonl"), "{}\n");
    assert_eq!(source.mark(), first, "読みの見立てで印が動く");
    // journal の長さが動けば印も動く。
    append(&noms.join(JOURNAL), "x");
    let grown = source.mark();
    assert_ne!(grown, first, "journal の長さで印が動かない");
    // manifest の根の hash の字が替われば、長さが同じでも印が動く。
    fs::write(noms.join(MANIFEST), MANIFEST_TEXT.replace("bbbb", "cccc")).expect("manifest");
    let swapped = source.mark();
    assert_ne!(swapped, grown, "manifest の字で印が動かない");
    // table の file の長さが動けば印も動く。
    append(&noms.join("tbl2"), "x");
    let folded = source.mark();
    assert_ne!(folded, swapped, "table の file の長さで印が動かない");
    fs::remove_file(noms.join(JOURNAL)).expect("journal を消す");
    let Mark::Store { sizes, .. } = source.mark() else {
        panic!("store の印でない");
    };
    assert_eq!(sizes.len(), 3);
    assert_eq!(
        sizes[2],
        (JOURNAL.to_string(), None),
        "journal が無いのに長さが在る"
    );
}

/// 同じ印の値を返し続ける見張りに、受け手を 1 人足してから 1000 ミリ秒の後の読みの回。
fn reads_in_a_second(mark: Mark, reading: Reading<u32>) -> usize {
    let reads = Arc::new(AtomicUsize::new(0));
    let r = Arc::clone(&reads);
    let timing = Timing {
        poll: Duration::from_millis(20),
        reread: Duration::from_millis(150),
        store_reread: Duration::from_secs(60),
    };
    let hub = Hub::watch_ledger(
        move || mark.clone(),
        move || {
            r.fetch_add(1, Ordering::SeqCst);
            reading.clone()
        },
        timing,
    );
    let rx = hub.subscribe();
    thread::sleep(Duration::from_millis(1000));
    let n = reads.load(Ordering::SeqCst);
    drop(rx);
    drop(hub);
    n
}

#[test]
fn lstore_watch_intervals() {
    let store = Mark::Store {
        manifest: b"m".to_vec(),
        sizes: Vec::new(),
    };
    let files = Mark::Files(vec![None, None]);
    let n = reads_in_a_second(store.clone(), Reading::Known(1));
    assert!(n <= 2, "store の印で読めた後も読み直す: {n}");
    let n = reads_in_a_second(store, Reading::Unknown);
    assert!(n >= 3, "store の印で落ちた後に読み直さない: {n}");
    let n = reads_in_a_second(files, Reading::Known(1));
    assert!(n >= 3, "jsonl の印で読み直さない: {n}");
}

/// 偽の bd の出力を置く（別の名で書いてから置き替える）。
fn bd_returns(root: &Path, text: &str) {
    let tmp = root.join("out.json.tmp");
    fs::write(&tmp, text).expect("偽の bd の出力");
    fs::rename(&tmp, root.join("out.json")).expect("偽の bd の出力を置く");
}

/// 偽の bd が撃たれた回（記録の file の行の数）。
fn bd_calls(root: &Path) -> usize {
    fs::read_to_string(root.join("calls"))
        .unwrap_or_default()
        .lines()
        .count()
}

/// SSE の口から `until` まで（か `stop` が立つまで）読む。
fn read_until(s: &mut TcpStream, buf: &mut String, until: Instant, stop: impl Fn(&str) -> bool) {
    let mut chunk = [0u8; 4096];
    while !stop(buf) {
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return;
        }
        s.set_read_timeout(Some(left)).expect("timeout");
        match s.read(&mut chunk) {
            Ok(0) => return,
            Ok(n) => buf.push_str(std::str::from_utf8(&chunk[..n]).expect("SSE の字")),
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => return,
            Err(e) => panic!("SSE を読む: {e}"),
        }
    }
}

fn events(b: &str) -> usize {
    b.matches("event: ledger-changed\n").count()
}

#[test]
fn lstore_live_journal_moves() {
    let root = place("live");
    let repo = root.join("repo");
    let files = root.join("files");
    fs::create_dir_all(&files).expect("面の file の置き場");
    fs::write(files.join("index.html"), "tz").expect("index.html");
    for mark in ["issues.jsonl", "interactions.jsonl"] {
        fs::write(repo.join(".beads").join(mark), "{}\n").expect("印の file");
    }
    let noms = store(&repo, "t3");
    let (calls, out) = (root.join("calls"), root.join("out.json"));
    let script = format!(
        "#!/bin/sh\necho x >> '{}'\nexec cat '{}'\n",
        calls.display(),
        out.display()
    );
    let bd = root.join("bd");
    fs::write(&bd, script).expect("偽の bd");
    fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
    let text = fixture();
    bd_returns(&root, &text);
    let config = Config {
        bd: bd.into(),
        ..Config::new(repo.clone(), "127.0.0.1:0".parse().expect("bind 先"), files)
    };
    serve_and_watch(root, repo, noms, text, config);
}

/// 起こした server の知らせの口で、読みの見立てと journal の動きへの bd の回と知らせの件を見る。
fn serve_and_watch(root: PathBuf, repo: PathBuf, noms: PathBuf, text: String, config: Config) {
    let server = Server::bind(&config).expect("起動");
    let addr: SocketAddr = server.local_addr().expect("口の住所");
    thread::spawn(move || server.run());
    assert_eq!(bd_calls(&root), 1, "起こした直後の bd の回");
    let mut s = TcpStream::connect(addr).expect("接続");
    s.write_all(
        b"GET /api/surface/events HTTP/1.1\r\nHost: x\r\nAccept: text/event-stream\r\n\r\n",
    )
    .expect("要求");
    let mut buf = String::new();
    read_until(
        &mut s,
        &mut buf,
        Instant::now() + Duration::from_secs(5),
        |b| b.contains("retry: 1000\n\n"),
    );
    assert!(buf.contains("retry: 1000\n\n"), "{buf}");
    thread::sleep(Duration::from_secs(2));
    let base = bd_calls(&root);
    assert!(base <= 2, "受け手が付いた後の bd の回: {base}");
    // 読みの見立てと jsonl の動きでは bd を撃たない。
    read_like(&noms);
    append(&repo.join(".beads").join("interactions.jsonl"), "{}\n");
    read_until(
        &mut s,
        &mut buf,
        Instant::now() + Duration::from_secs(6),
        |b| events(b) >= 1,
    );
    assert_eq!(bd_calls(&root), base, "印の動かない 6 秒の間に bd を撃つ");
    assert_eq!(events(&buf), 0, "{buf}");
    // 書きの見立て（journal が伸びる）で 1.5 秒以内に 1 件。
    bd_returns(
        &root,
        &text.replacen("\"status\": \"open\"", "\"status\": \"closed\"", 1),
    );
    let changed = Instant::now();
    append(&noms.join(JOURNAL), "x");
    read_until(
        &mut s,
        &mut buf,
        changed + Duration::from_millis(1500),
        |b| events(b) >= 1 && b.ends_with("\n\n"),
    );
    let took = changed.elapsed();
    assert_eq!(
        events(&buf),
        1,
        "1.5 秒以内に 1 件でない（{took:?}）: {buf}"
    );
    assert!(took < Duration::from_millis(1500), "{took:?}");
    assert_eq!(bd_calls(&root), base + 1, "journal の 1 回の動きの bd の回");
    drop(s);
}

#[test]
fn lstore_own_names_clean() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth2/lstore.rs"))
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
            .strip_prefix("lstore_")
            .unwrap_or_else(|| panic!("lstore_ で始まらない: {name}"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
