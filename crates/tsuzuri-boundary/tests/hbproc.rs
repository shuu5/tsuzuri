//! 子 process を撃つ部品と時刻の読みを台帳の読みから割る歯（接頭辞 hbproc_・設計ノート surface-hub 行 hb-proc の完了の条件）。
//! 歯は `proc` と `clock` を直に呼び、`ledger` の再公開の名でも呼んで同じ値かを比べる。
//! 定義の置き場は src の字を読んで見る。

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::server::{clock, ledger, proc};

/// capture の歯の上限。
const TIMEOUT: Duration = Duration::from_secs(5);

/// 歯ごとの一意の名の dir。
fn place(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("hbproc")
        .join(format!("{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("作業場");
    dir
}

fn src(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/server")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn hbproc_definitions_move_out_of_ledger() {
    let (ledger, proc, clock) = (src("ledger.rs"), src("proc.rs"), src("clock.rs"));
    let moved_proc = [
        "fn capture<",
        "fn stop(",
        "fn stop_with(",
        "const KILL:",
        "const WAIT_STEP:",
    ];
    let moved_clock = [
        "fn epoch_secs(",
        "fn is_leap(",
        "fn days_in_month(",
        "fn days_from_civil(",
    ];
    for def in moved_proc.iter().chain(&moved_clock) {
        assert!(!ledger.contains(def), "ledger.rs に {def} が残る");
    }
    for def in moved_proc {
        assert!(proc.contains(def), "proc.rs に {def} が無い");
    }
    for def in moved_clock {
        assert!(clock.contains(def), "clock.rs に {def} が無い");
    }
    for def in [
        "pub fn capture<I, S>(program: &OsStr, args: I, cwd: &Path, timeout: Duration) -> Option<Vec<u8>>",
        "pub fn stop_with(mut child: Child, kill: &OsStr) -> bool",
        "pub const KILL: &str = \"kill\";",
    ] {
        assert!(proc.contains(def), "proc.rs に {def} が無い");
    }
    assert!(clock.contains("pub fn epoch_secs(s: &str) -> Option<EpochSecs>"));
}

#[test]
fn hbproc_epoch_secs_reads_rfc3339_on_both_paths() {
    let read = |s: &str| {
        let direct = clock::epoch_secs(s);
        assert_eq!(ledger::epoch_secs(s), direct, "{s}");
        direct
    };
    assert_eq!(read("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(read("2026-09-27T07:39:00Z"), Some(1_790_494_740));
    assert_eq!(
        read("2026-09-27T16:39:00.123456789+09:00"),
        Some(1_790_494_740)
    );
    for bad in [
        "",
        "2026-09-27T07:39:00",
        "2026-02-29T00:00:00Z",
        "2026-13-01T00:00:00Z",
        "2026-09-27T24:00:00Z",
        "2026-09-27T07:39:00.Z",
        "2026-09-27T07:39:00+0900",
        "1969-12-31T23:59:59Z",
        "２０２６-09-27T07:39:00Z",
    ] {
        assert_eq!(read(bad), None, "{bad}");
    }
}

#[test]
fn hbproc_capture_returns_stdout_of_rc0_on_both_paths() {
    let dir = place("capture");
    let sh = OsStr::new("sh");
    let ok = ["-c", "printf hbproc-out"];
    let direct = proc::capture(sh, ok, &dir, TIMEOUT);
    assert_eq!(direct.as_deref(), Some(&b"hbproc-out"[..]));
    assert_eq!(ledger::capture(sh, ok, &dir, TIMEOUT), direct);
    let fail = ["-c", "printf hbproc-out; exit 1"];
    assert_eq!(proc::capture(sh, fail, &dir, TIMEOUT), None);
    assert_eq!(ledger::capture(sh, fail, &dir, TIMEOUT), None);
}

#[test]
fn hbproc_kill_and_stop_with_are_the_same_on_both_paths() {
    assert_eq!(proc::KILL, "kill");
    assert_eq!(ledger::KILL, proc::KILL);
    let spawn = || {
        Command::new("sleep")
            .arg("30")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("sleep")
    };
    // 道具が撃てなければ、どちらの path も子だけを止めて false を返す。
    let missing = OsStr::new("/nonexistent/tz-no-such-kill");
    assert!(!proc::stop_with(spawn(), missing));
    assert!(!ledger::stop_with(spawn(), missing));
}
