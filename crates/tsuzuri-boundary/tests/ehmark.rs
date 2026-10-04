//! 台帳の印が最後に読めた時と同じ間の持ち回しの歯（接頭辞 ehmark_・設計ノート surface-wave27b 行 e-hold-mark・判断の記録 ADR-30 決定 (10)）。
//! 偽の bd は撃たれるたびに記録の file に 1 行を足し、作業場に down の file が在れば rc 1、once の file が在ればそれを消して rc 1、
//! slow の file が在れば 6 秒眠ってから、置いた字の file（out.json）を出す script。印は repo の .beads/issues.jsonl（store は置かない）。
//! 作業場は CARGO_TARGET_TMPDIR の下に歯ごとに作る。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::ledger::{BD_TIMEOUT, Got, Source, WATCH_TIMEOUT};
use tsuzuri_contract::board::Reading;

/// 偽の bd が出す 2 つの字（どちらも読める台帳の字）。
const A: &str = "[]\n";
const B: &str = "[ ]\n";

/// 持ち回しの上限を短くした歯の上限と、それを越える眠り。
const HOLD: Duration = Duration::from_millis(300);
const PAST: Duration = Duration::from_millis(600);

struct Place {
    root: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("ehmark")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("repo/.beads")).expect("repo の置き場");
        fs::write(root.join("repo/.beads/issues.jsonl"), "{}\n").expect("印の file");
        let r = root.display();
        let script = format!(
            "#!/bin/sh\necho x >> '{r}/calls'\nif [ -e '{r}/down' ]; then exit 1; fi\n\
             if [ -e '{r}/once' ]; then rm '{r}/once'; exit 1; fi\n\
             if [ -e '{r}/slow' ]; then sleep 6; fi\nexec cat '{r}/out.json'\n"
        );
        fs::write(root.join("bd"), script).expect("偽の bd");
        fs::set_permissions(root.join("bd"), fs::Permissions::from_mode(0o755)).expect("権限");
        let place = Place { root };
        place.returns(A);
        place
    }

    /// 見張りの付いた Source（面の server の口が読む形）。
    fn source(&self) -> Source {
        Source::new(self.root.join("repo"), self.root.join("bd")).watched()
    }

    /// 偽の bd が撃たれた回。
    fn calls(&self) -> usize {
        fs::read_to_string(self.root.join("calls"))
            .unwrap_or_default()
            .lines()
            .count()
    }

    fn flag(&self, name: &str, on: bool) {
        let path = self.root.join(name);
        if on {
            fs::write(path, "").expect("印の file");
        } else {
            fs::remove_file(path).expect("印の file を消す");
        }
    }

    fn returns(&self, text: &str) {
        fs::write(self.root.join("out.json"), text).expect("置いた字");
    }

    /// 台帳の印を動かす（issues.jsonl に 1 行足した字を隣に書いて rename で置き替える）。
    fn touch(&self) {
        let path = self.root.join("repo/.beads/issues.jsonl");
        let text = fs::read_to_string(&path).unwrap_or_default() + "{}\n";
        let tmp = path.with_file_name("issues.jsonl.tmp");
        fs::write(&tmp, text).expect("隣の file");
        fs::rename(&tmp, &path).expect("置き替える");
    }
}

fn fresh(text: &str) -> Got {
    Got {
        text: Some(text.into()),
        stale: None,
    }
}

#[test]
fn ehmark_same_mark_no_read() {
    let place = Place::new("same");
    let source = place.source();
    assert_eq!(source.got(), fresh(A));
    assert_eq!(place.calls(), 1, "最初の読み");
    // 字を替えても印が同じなら bd を撃たず、最後に読めた字を新しい字として返す。
    place.returns(B);
    assert_eq!(source.got(), fresh(A));
    assert_eq!(source.text().as_deref(), Some(A));
    assert_eq!(place.calls(), 1, "印が同じ間の got と text");
    // 印が動けば読む。
    place.touch();
    assert_eq!(source.got(), fresh(B));
    assert_eq!(place.calls(), 2, "印が動いた後の got");
}

#[test]
fn ehmark_fail_keeps_text() {
    let place = Place::new("keeps");
    let source = place.source().with_hold(HOLD);
    assert_eq!(source.got(), fresh(A));
    // 見張りの読みが落ちても、印が同じなら上限を越えても最後に読めた字を新しい字として返す。
    place.flag("down", true);
    assert_eq!(source.read(), Reading::Unknown);
    let calls = place.calls();
    thread::sleep(PAST);
    assert_eq!(source.got(), fresh(A));
    assert_eq!(source.text().as_deref(), Some(A));
    assert_eq!(place.calls(), calls, "落ちた後も印が同じ間は撃たない");
}

#[test]
fn ehmark_moved_mark_holds() {
    let place = Place::new("moved");
    let source = place.source().with_hold(HOLD);
    let before = Instant::now();
    assert_eq!(source.got(), fresh(A));
    let after = Instant::now();
    // 印が動いて読みが落ちれば、上限の内は最後に読めた字と読めた時刻。
    place.flag("down", true);
    place.touch();
    let held = source.got();
    assert_eq!(held.text.as_deref(), Some(A));
    let at = held.stale.expect("落ちた読みの時刻");
    assert!(before <= at && at <= after, "{at:?} は読めた時刻でない");
    // 上限を越えれば字は無い（時刻は同じ）。
    thread::sleep(PAST);
    assert_eq!(
        source.got(),
        Got {
            text: None,
            stale: Some(at)
        }
    );
}

#[test]
fn ehmark_retry_after_fail() {
    let place = Place::new("retry");
    let source = place.source();
    assert_eq!(source.got(), fresh(A));
    // 印が動き、次の 1 回だけ落ちる。落ちた結果を持たず、次の got が読み直す。
    place.returns(B);
    place.flag("once", true);
    place.touch();
    let held = source.got();
    assert_eq!(held.text.as_deref(), Some(A));
    assert!(held.stale.is_some(), "落ちた読み");
    assert_eq!(place.calls(), 2);
    assert_eq!(source.got(), fresh(B));
    assert_eq!(place.calls(), 3, "落ちた後の got は読み直す");
    assert_eq!(source.got(), fresh(B));
    assert_eq!(place.calls(), 3, "読めた後は印が同じなので撃たない");
}

#[test]
fn ehmark_watch_timeout() {
    assert_eq!(WATCH_TIMEOUT, Duration::from_secs(30));
    assert_eq!(BD_TIMEOUT, Duration::from_secs(5));
    let place = Place::new("watch");
    let source = place.source();
    // 一度も読めていない Source の read（server の起動の読み）は 5 秒で落ちる。
    place.flag("slow", true);
    assert_eq!(source.read(), Reading::Unknown, "起動の読み");
    place.flag("slow", false);
    assert_eq!(source.got(), fresh(A));
    // 6 秒かかる bd: 口の要求の読み（5 秒）は落ち、見張りの読み（30 秒）は読める。
    place.returns(B);
    place.flag("slow", true);
    place.touch();
    let held = source.got();
    assert_eq!(held.text.as_deref(), Some(A));
    assert!(held.stale.is_some(), "5 秒を越えた要求の読み");
    assert!(
        matches!(source.read(), Reading::Known(_)),
        "6 秒の見張りの読み"
    );
    let calls = place.calls();
    assert_eq!(source.got(), fresh(B));
    assert_eq!(place.calls(), calls, "見張りの読みの後は撃たない");
}
