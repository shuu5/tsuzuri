//! host の口と host の種類の見張りの歯（接頭辞 hapi_・持ち主の裁定 t3-hub.77.28）。
//! kernel の file の写しの木を一時の dir に置き、根を替えた読み（`Host::with_root`）で字の集めと速さの控えと見張りを測る。
//! 撃った後に歯ごとの木が一時の dir に残らないことも測る。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::Route;
use tsuzuri_boundary::server::events::Hub;
use tsuzuri_boundary::server::host::{HOST_POLL, Host, RATE_GAP};
use tsuzuri_boundary::server::route::{Key, Match};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::host::{Gauge, PATH};
use tsuzuri_contract::surface::ChangeKind;

const LOADAVG: &str = "40.00 1.00 1.00 1/2 3\n";

/// 木の置き場（一時の dir の下・名に pid と字 name）。
fn place(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("hapi-{}-{name}", std::process::id()))
}

/// 木を作って `body` を撃ち、断言の前に木を消す（歯ごとの dir を一時の dir に残さない・memo t3-hub.74.49.10）。
fn shot<T>(name: &str, body: impl FnOnce(&Path) -> T) -> T {
    let dir = place(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("一時の dir");
    let got = body(&dir);
    let _ = fs::remove_dir_all(&dir);
    got
}

fn put(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("親")).expect("dir");
    fs::write(path, text).expect("書く");
}

/// kernel の file と、cgroup の親 a.slice の下の scope 3 つ（上限なし 1 つ）を置いた根。
fn kernel(root: &Path) {
    put(root, "proc/loadavg", LOADAVG);
    put(root, "sys/devices/system/cpu/online", "0-31\n");
    put(
        root,
        "proc/pressure/cpu",
        "some avg10=1.00 avg60=2.00 avg300=3.00 total=1\n",
    );
    put(
        root,
        "proc/pressure/memory",
        "some avg10=0.00 avg60=0.00 avg300=0.00 total=1\nfull avg10=0.00 avg60=0.50 avg300=0.00 total=1\n",
    );
    put(
        root,
        "proc/pressure/io",
        "some avg10=0.00 avg60=0.00 avg300=0.00 total=1\nfull avg10=9.00 avg60=6.00 avg300=0.00 total=1\n",
    );
    put(
        root,
        "proc/meminfo",
        "MemTotal: 4 kB\nMemAvailable: 2 kB\nSwapTotal: 1 kB\nSwapFree: 1 kB\n",
    );
    put(root, "proc/self/cgroup", "0::/a.slice/own.scope\n");
    for (scope, current, max) in [
        ("own.scope", "10", "100"),
        ("seat.scope", "26", "32"),
        ("free.scope", "5", "max"),
    ] {
        put(
            root,
            &format!("sys/fs/cgroup/a.slice/{scope}/memory.current"),
            current,
        );
        put(
            root,
            &format!("sys/fs/cgroup/a.slice/{scope}/memory.max"),
            max,
        );
    }
}

fn stat(sectors: u64) -> String {
    format!("1 2 3 4 5 6 {sectors} 8 9 10 11\n")
}

/// 書きの測りの表 2 行（摩耗の記録の path を持つ行と持たない行）を host の面に置いた state dir。
fn face(root: &Path) -> PathBuf {
    let state = root.join("state");
    let (dev, wear) = (root.join("dev-stat"), root.join("wear.json"));
    put(root, "dev-stat", &stat(1_000));
    put(
        root,
        "wear.json",
        r#"{"local_time":{"time_t":9},"nvme_smart_health_information_log":{"percentage_used":6,"data_units_written":1,"available_spare":100,"media_errors":0}}"#,
    );
    let text = format!(
        "[[device]]\nname = \"pc\"\n\n[[write-budget]]\nname = \"sys\"\nstat = \"{}\"\nwear = \"{}\"\n\n[[write-budget]]\nname = \"bak\"\nstat = \"{}\"\n",
        dev.display(),
        wear.display(),
        root.join("no-stat").display()
    );
    put(&state, "host.toml", &text);
    state
}

#[test]
fn hapi_route_and_kind() {
    assert_eq!(PATH, "/api/host");
    let key = Key {
        method: "GET",
        path: Match::Exact(PATH),
    };
    assert_eq!(Route::ALL.iter().filter(|r| r.key() == key).count(), 1);
    assert_eq!(ChangeKind::ALL.len(), 8);
    assert_eq!(ChangeKind::ALL[6..], [ChangeKind::Host, ChangeKind::Notice]);
    assert_eq!(
        (HOST_POLL, RATE_GAP),
        (Duration::from_secs(10), Duration::from_secs(5))
    );
}

/// 根の下の kernel の file と cgroup の親の下の上限を持つ scope と host の面の表の装置を読む。
#[test]
fn hapi_reads_the_kernel_tree() {
    let doc = shot("read", |root| {
        kernel(root);
        let state = face(root);
        Host::with_root(root, Some(&state)).doc()
    });
    assert!(matches!(doc.load, Reading::Known(l) if l.one == 4_000 && l.cores == 32));
    assert!(matches!(doc.memory, Reading::Known(m) if m.total == 4_096));
    assert_eq!(doc.over, [Gauge::Load, Gauge::Io]);
    let Reading::Known(scopes) = &doc.scopes else {
        panic!("scope が読めない: {:?}", doc.scopes);
    };
    let names: Vec<&str> = scopes.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["seat.scope", "own.scope"]);
    let devices: Vec<(&str, bool)> = doc
        .devices
        .iter()
        .map(|d| (d.name.as_str(), d.wear.is_some()))
        .collect();
    assert_eq!(devices, [("sys", true), ("bak", false)]);
    assert!(matches!(doc.devices[0].wear, Some(Reading::Known(w)) if w.used_pct == 6));
    assert_eq!(doc.devices[1].rate, Reading::Unknown);
}

/// 根の下に file が無ければ、測りは全部まだ分からないで、装置は空・注意は無い。
#[test]
fn hapi_missing_files_are_unknown() {
    let bare = shot("bare", |root| Host::with_root(root, None).doc());
    assert_eq!(
        (bare.load, bare.scopes),
        (Reading::Unknown, Reading::Unknown)
    );
    assert!(bare.devices.is_empty() && bare.over.is_empty());
}

/// 速さは前の読みとの差で、間が足りない読みは前の速さを返し、間が足りれば測り直す。
#[test]
fn hapi_rate_holds_inside_the_gap() {
    let rate = |host: &Host| host.doc().devices[0].rate.clone();
    let [first, inside, fresh, moved] = shot("rate", |root| {
        kernel(root);
        let state = face(root);
        let held = Host::with_root(root, Some(&state));
        let first = rate(&held);
        std::thread::sleep(Duration::from_millis(20));
        put(root, "dev-stat", &stat(9_000));
        let inside = rate(&held);
        let quick = Host::with_root(root, Some(&state)).with_gap(Duration::ZERO);
        let fresh = rate(&quick);
        std::thread::sleep(Duration::from_millis(20));
        put(root, "dev-stat", &stat(19_000));
        [first, inside, fresh, rate(&quick)]
    });
    assert_eq!(first, Reading::Unknown);
    assert_eq!(inside, Reading::Unknown);
    assert_eq!(fresh, Reading::Unknown);
    assert!(matches!(moved, Reading::Known(r) if r > 0));
}

/// 受け手を待つ。見張りの frame を `window` の間だけ集める。
fn frames(sub: &std::sync::mpsc::Receiver<String>, window: Duration) -> Vec<String> {
    let end = Instant::now() + window;
    let mut out = Vec::new();
    while let Some(left) = end.checked_duration_since(Instant::now()) {
        if let Ok(frame) = sub.recv_timeout(left) {
            out.push(frame);
        }
    }
    out
}

/// 受け手が居る周だけ読み、中身が動いた時だけ host の種類の board-changed を 1 件送る。
#[test]
fn hapi_watch_sends_host_on_change() {
    let (first, moved) = shot("watch", |root| {
        kernel(root);
        let hub = Arc::new(Hub::default());
        Arc::new(Host::with_root(root, None)).watch(&hub, Duration::from_millis(20));
        let sub = hub.subscribe();
        let first = frames(&sub, Duration::from_millis(400));
        put(root, "proc/loadavg", "1.00 1.00 1.00 1/2 3\n");
        (first, frames(&sub, Duration::from_millis(400)))
    });
    assert_eq!(first.len(), 1, "{first:?}");
    assert!(
        first[0].contains("event: board-changed") && first[0].contains("\"kinds\":[\"host\"]"),
        "{first:?}"
    );
    assert_eq!(moved.len(), 1, "{moved:?}");
}

/// 撃つ間は木が在り（kernel の file の負荷が読める）、撃った後は木の dir が無い。
#[test]
fn hapi_tree_is_gone_after_the_shot() {
    let load = shot("gone", |root| {
        kernel(root);
        Host::with_root(root, None).doc().load
    });
    assert!(
        matches!(load, Reading::Known(l) if l.one == 4_000),
        "木が在る間に読んだ: {load:?}"
    );
    let root = place("gone");
    assert!(!root.exists(), "撃った後は無い: {}", root.display());
}
