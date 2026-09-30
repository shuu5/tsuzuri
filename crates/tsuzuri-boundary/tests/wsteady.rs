//! 板の印の見張りの揺れの歯（接頭辞 wsteady_・計画 surface-plan の行 e-watch-steady の完了の条件）。
//! Hub::watch_board に呼びの回を数える board の関数を渡し、周の進みを呼びの回で数える。
//! 印の file は作業場の外の隣の dir に書いてから移すので、見張りから書きかけは見えない。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::events::Hub;
use tsuzuri_contract::surface::ChangeKind::Design;

/// 着地済みの行と第 3 波から第 8 波の行の verify の filter の語（この行の接頭辞は並べない）。
const FILTERS: [&str; 106] = [
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
    "fmark_",
    "fstop_",
    "fserve_",
    "nsumw_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
    "gsum_",
    "nstall_",
    "pfold_",
    "uword_",
    "cround_",
    "cgraph_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
];

/// 印を書き換える歯の本文に無いはずの字（file の中身をその場で書く字）。
const WRITERS: [&str; 4] = ["fs::write(", "File::create(", "OpenOptions", "File::options("];

/// 本文を見る歯（file と fn の名）。
const BODIES: [(&str, &str); 4] = [
    ("src/server/events.rs", "server_src_watch_marks_trigger_reread"),
    ("src/server/events.rs", "server_read_watch_board_sends_on_marks"),
    ("tests/server_read.rs", "server_read_board_changed_within_5s"),
    ("tests/wsteady.rs", "wsteady_absent_listed_path_changes"),
];

/// 見張りの間隔。
const POLL: Duration = Duration::from_millis(20);

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 歯ごとの印の dir（名に process の id と時刻を入れる）。
fn place(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("wsteady")
        .join(format!("{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("印の dir");
    dir
}

/// 印の dir の外の隣の dir（印の dir の名に -put を足す）。
fn put_dir(mark: &Path) -> PathBuf {
    let dir = mark.parent().expect("印の dir");
    let name = dir.file_name().expect("印の dir の名").to_string_lossy();
    dir.with_file_name(format!("{name}-put"))
}

/// 印の file の中身を丸ごと置く（隣の dir に書いてから移すので、見張りから書きかけは見えない）。
fn put(mark: &Path, text: &str) {
    let dir = put_dir(mark);
    fs::create_dir_all(&dir).expect("移しの dir");
    let tmp = dir.join(mark.file_name().expect("印の名"));
    fs::write(&tmp, text).expect("移す前の印");
    fs::rename(&tmp, mark).expect("印を移す");
}

/// 印の dir と移しの dir を消す。
fn clean(dir: &Path) {
    let _ = fs::remove_dir_all(put_dir(&dir.join("a")));
    let _ = fs::remove_dir_all(dir);
}

/// 呼びの回が `target` に届くまで待つ（5 秒で届かなければ落ちる）。
fn wait_calls(calls: &AtomicUsize, target: usize) {
    let until = Instant::now() + Duration::from_secs(5);
    while calls.load(Ordering::SeqCst) < target {
        assert!(
            Instant::now() < until,
            "5 秒で呼びが {target} 回に届かない（{} 回）",
            calls.load(Ordering::SeqCst)
        );
        thread::sleep(Duration::from_millis(2));
    }
}

/// 今までに受けた board-changed の件数（ほかの知らせは落ちる）。
fn received(rx: &Receiver<String>) -> usize {
    let mut n = 0;
    while let Ok(frame) = rx.try_recv() {
        assert!(frame.contains("event: board-changed\n"), "{frame}");
        n += 1;
    }
    n
}

/// 字の中の fn の本文（`fn <名>(` の行から、次の test の属性の行の前か終わりまで）。
fn body<'a>(text: &'a str, name: &str) -> Vec<&'a str> {
    let head = format!("fn {name}(");
    let mut lines = text.lines();
    lines
        .by_ref()
        .find(|l| l.trim().starts_with(&head))
        .unwrap_or_else(|| panic!("{name} の本文が無い"));
    let mut out = vec![];
    for line in lines {
        if line.trim() == "#[test]" {
            break;
        }
        out.push(line);
    }
    out
}

#[test]
fn wsteady_vanished_path_one_event() {
    let dir = place("vanished");
    let (a, b) = (dir.join("a"), dir.join("b"));
    put(&a, "1");
    put(&b, "2");
    let hub = Arc::new(Hub::default());
    let rx = hub.subscribe();
    let calls = Arc::new(AtomicUsize::new(0));
    let c = Arc::clone(&calls);
    let (fa, fb) = (a.clone(), b.clone());
    // 6 回目の呼びの中で b を消してから a と b を返し、7 回目からは a だけを返す。
    let board = move || {
        let n = c.fetch_add(1, Ordering::SeqCst) + 1;
        if n == 6 {
            fs::remove_file(&fb).expect("b を消す");
        }
        if n <= 6 {
            vec![(Design, fa.clone()), (Design, fb.clone())]
        } else {
            vec![(Design, fa.clone())]
        }
    };
    Hub::watch_board(&hub, board, POLL);
    wait_calls(&calls, 14);
    assert_eq!(received(&rx), 1, "消し 1 回の知らせが 1 件でない");
    clean(&dir);
}

#[test]
fn wsteady_absent_listed_path_changes() {
    let dir = place("absent");
    let (a, p) = (dir.join("a"), dir.join("p"));
    put(&a, "1");
    let list = Arc::new(Mutex::new(vec![a.clone()]));
    let hub = Arc::new(Hub::default());
    let rx = hub.subscribe();
    let calls = Arc::new(AtomicUsize::new(0));
    let (c, l) = (Arc::clone(&calls), Arc::clone(&list));
    let board = move || {
        c.fetch_add(1, Ordering::SeqCst);
        let files = l.lock().expect("lock").clone();
        files.into_iter().map(|f| (Design, f)).collect::<Vec<_>>()
    };
    Hub::watch_board(&hub, board, POLL);
    let from = calls.load(Ordering::SeqCst);
    wait_calls(&calls, from + 13);
    assert_eq!(received(&rx), 0, "印が動かないのに知らせる");
    // 無い path が列に増える・無い file ができる・在る file が消える・無い path が列から減る、のどれも 1 件。
    for (what, change) in [
        (
            "無い path を列に足す",
            Box::new(|| list.lock().expect("lock").push(p.clone())) as Box<dyn Fn()>,
        ),
        ("p を移しで作る", Box::new(|| put(&p, "x"))),
        (
            "p を消す",
            Box::new(|| fs::remove_file(&p).expect("p を消す")),
        ),
        (
            "p を列から除く",
            Box::new(|| list.lock().expect("lock").retain(|f| f != &p)),
        ),
    ] {
        change();
        let from = calls.load(Ordering::SeqCst);
        wait_calls(&calls, from + 13);
        assert_eq!(received(&rx), 1, "{what}: 知らせが 1 件でない");
    }
    clean(&dir);
}

#[test]
fn wsteady_marks_change_whole() {
    for (rel, name) in BODIES {
        let text = read(&manifest(rel));
        let lines = body(&text, name);
        assert!(!lines.is_empty(), "{rel} の {name} の本文が空");
        for line in lines {
            for w in WRITERS {
                assert!(!line.contains(w), "{rel} の {name} が {w} を含む: {line}");
            }
        }
    }
    for rel in ["src/server/events.rs", "tests/server_read.rs"] {
        assert!(
            read(&manifest(rel)).contains("fs::rename("),
            "{rel} に移しの fn が無い"
        );
    }
}

#[test]
fn wsteady_own_names_clean() {
    let text = read(&manifest("tests/wsteady.rs"));
    let mut names = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines.next().expect("test の属性の次の行");
        let name = decl
            .trim()
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("fn の宣言でない: {decl}"));
        names.push(name.to_string());
    }
    assert!(names.len() >= 4, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("wsteady_")
            .unwrap_or_else(|| panic!("{name} が wsteady_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
