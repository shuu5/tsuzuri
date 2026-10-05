//! 歯の一時 dir の助け（`pipe::fixture` の `scratch` と `held`）の歯（memo t3-hub.74.49.10）。
//!
//! 助けが作った dir は歯の thread の終わりに消える（panic で終わった thread も）。
//! 歯は自分の作った dir を全部 `held` に預けるので、/tmp に dir を残さない（預けるのは在る無しを測った後）。

use super::fixture::{held, made_in_thread, scratch};
use std::fs;
use std::sync::mpsc;
use std::thread;

/// 作った dir は thread の中では在り、中に file を置いた周も join の後は無い。
#[test]
fn vschd_scratch_dir_is_gone_after_the_thread_ends() {
    let (inside, gone, dir) = made_in_thread(|| scratch("vschd-gone"));
    assert!(inside, "thread の中では在り file を置ける: {}", dir.display());
    assert!(gone, "join の後は無い: {}", dir.display());
}

/// panic で終わった thread の dir も join の後は無い。
#[test]
fn vschd_scratch_dir_is_gone_after_a_panicking_thread() {
    let (sent, got) = mpsc::channel();
    let joined = thread::spawn(move || {
        let dir = scratch("vschd-panic");
        let _ = fs::write(dir.join("f"), "1");
        let _ = sent.send(dir);
        panic!("歯の中の panic");
    })
    .join();
    let dir = got.recv().expect("thread が path を渡す");
    let gone = !dir.exists();
    let dir = held(dir);
    assert!(joined.is_err(), "thread は panic で終わった");
    assert!(gone, "panic の後も無い: {}", dir.display());
}

/// 同じ名の 2 回目は空の dir を返し、thread の後は無く、隣の同じ頭の語の別の名の dir は残る。
#[test]
fn vschd_scratch_removes_only_its_own_dir() {
    let sibling = held(std::env::temp_dir().join(format!("pipe-mutant-vschd-own-sibling-{}", std::process::id())));
    fs::create_dir_all(&sibling).expect("隣の dir");
    fs::write(sibling.join("keep"), "1").expect("隣の file");
    let (empty, dir) = thread::spawn(|| {
        let first = scratch("vschd-own");
        let _ = fs::write(first.join("f"), "1");
        let second = scratch("vschd-own");
        (fs::read_dir(&second).map(|mut entries| entries.next().is_none()).unwrap_or(false), second)
    })
    .join()
    .expect("thread が終わる");
    let gone = !dir.exists();
    let dir = held(dir);
    assert!(empty, "2 回目は空の dir: {}", dir.display());
    assert!(gone, "thread の後は無い: {}", dir.display());
    assert!(sibling.join("keep").is_file(), "隣の dir は残る: {}", sibling.display());
}
