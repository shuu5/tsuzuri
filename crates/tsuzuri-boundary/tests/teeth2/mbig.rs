//! account board の読みが大きな字を要求ごとに写さない歯（接頭辞 mbig_・設計ノート surface-wave24a 行 m-big-copies の完了の条件）。
//! 偽の git は作業場の git-out の字を、偽の器と偽の bd は rc 1 を返す script（bd は作業場に置いた字の file を出し、
//! down の file が在れば rc 1）。event log の字は同じ長さの字に替えて更新時刻を戻すことで、印が同じまま字だけが替わる書きを作る。
#![cfg(test)]

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::common::script;
use tsuzuri_boundary::acct::Acct;
use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_contract::account::RunCounts;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;
use tsuzuri_core::account::host::HostTexts;
use tsuzuri_core::account::project::ProjectTexts;

/// 読みの今の時刻（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

const CREATED: &str = "{\"schema\":1,\"ts\":\"2026-09-27T11:00:00Z\",\"kind\":\"RunCreated\",\"run\":\"r.1-20260927T110000Z\",\"bead\":\"r.1\",\"host\":\"host-1\",\"actor\":\"machine\",\"stage\":\"Intake\",\"detail\":\"classes:\"}\n";
const SPAWNED: &str = "{\"schema\":1,\"ts\":\"2026-09-27T11:20:00Z\",\"kind\":\"RunStage\",\"run\":\"r.1-20260927T110000Z\",\"bead\":\"r.1\",\"host\":\"host-1\",\"actor\":\"machine\",\"stage\":\"Spawned\"}\n";

const LEDGER: &str = "[{\"id\":\"r.1\",\"title\":\"t\",\"status\":\"open\",\"issue_type\":\"task\",\"updated_at\":\"2026-09-27T07:39:00Z\"}]\n";

/// 歯ごとの作業場（state-h は引数の state dir・state-a と state-b は git が返す state dir）。
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(name: &str) -> Work {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("mbig")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        for dir in ["bin", "repo", "work/proj-a", "states/state-h"] {
            fs::create_dir_all(root.join(dir)).expect("作業場の dir");
        }
        for state in ["state-a", "state-b"] {
            fs::create_dir_all(root.join("states").join(state).join("fleet")).expect("state dir");
        }
        let work = Work { root };
        script(&work.root.join("bin/scribe2"), "exit 1");
        script(&work.root.join("bin/bd"), "exit 1");
        let git = format!("exec cat '{}/git-out'", work.root.display());
        script(&work.root.join("bin/git"), &git);
        let anchor = work.root.join("work/proj-a");
        fs::write(
            work.state("state-h").join("host.toml"),
            format!(
                "[[account-group]]\nname = \"g-a\"\nanchors = [\"{}\"]\naccounts = [\"acct-1\"]\n",
                anchor.display()
            ),
        )
        .expect("host.toml");
        work.git_says("state-a");
        work.write_log("state-a", &format!("{CREATED}{SPAWNED}"));
        work.write_log("state-b", &format!("{CREATED}{}", stopped(SPAWNED)));
        work
    }

    fn state(&self, name: &str) -> PathBuf {
        self.root.join("states").join(name)
    }

    fn log(&self, state: &str) -> PathBuf {
        self.state(state).join("fleet/events.jsonl")
    }

    /// 偽の git が返す state dir を替える。
    fn git_says(&self, state: &str) {
        fs::write(
            self.root.join("git-out"),
            format!("{}\n", self.state(state).display()),
        )
        .expect("git の出力");
    }

    fn write_log(&self, state: &str, text: &str) {
        fs::write(self.log(state), text).expect("event log");
    }

    /// event log の字を同じ長さの別の字に替え、更新時刻を替える前の値に戻す。
    fn swap_same_stamp(&self, state: &str, text: &str) {
        let path = self.log(state);
        let before = fs::metadata(&path).expect("印");
        assert_eq!(before.len() as usize, text.len(), "同じ長さの字");
        fs::write(&path, text).expect("字を替える");
        File::options()
            .write(true)
            .open(&path)
            .expect("開く")
            .set_modified(before.modified().expect("更新時刻"))
            .expect("更新時刻を戻す");
    }

    fn acct(&self) -> Acct {
        Acct::new(
            self.root.join("bin/scribe2"),
            self.root.join("bin/git"),
            self.root.join("bin/bd"),
            self.state("state-h"),
            self.root.join("repo"),
        )
        .with_git_hold(Duration::ZERO)
    }
}

/// 段 Spawned の行を、同じ長さの段 Stopped の行に替えた字。
fn stopped(line: &str) -> String {
    line.replace("Spawned", "Stopped")
}

/// project の行の run の 4 列。
fn runs(acct: &Acct) -> Reading<RunCounts> {
    let doc = acct.doc(NOW);
    assert_eq!(doc.projects.len(), 1, "宣言の project は 1 つ");
    doc.projects[0].runs.clone()
}

fn counts(run: u32, stop: u32) -> Reading<RunCounts> {
    Reading::Known(RunCounts {
        wait: 0,
        run,
        stop,
        land: 0,
    })
}

#[test]
fn mbig_log_kept() {
    let work = Work::new("kept");
    let acct = work.acct();
    assert_eq!(runs(&acct), counts(1, 0));
    work.swap_same_stamp("state-a", &format!("{CREATED}{}", stopped(SPAWNED)));
    assert_eq!(runs(&acct), counts(1, 0), "印が同じなら読み直さない");
    assert_eq!(runs(&acct), counts(1, 0), "3 回目も同じ");
    assert_eq!(runs(&work.acct()), counts(0, 1), "新しい Acct は読む");
}

#[test]
fn mbig_log_moved() {
    let work = Work::new("moved");
    let acct = work.acct();
    assert_eq!(runs(&acct), counts(1, 0));
    let stopped_line = stopped(SPAWNED);
    let grown = format!("{CREATED}{SPAWNED}{stopped_line}");
    work.write_log("state-a", &grown);
    assert_eq!(runs(&acct), counts(0, 1), "書き足せば読み直す");
    fs::remove_file(work.log("state-a")).expect("file を消す");
    assert_eq!(runs(&acct), Reading::Unknown, "file が無ければ Unknown");
    work.write_log("state-a", &format!("{CREATED}{SPAWNED}"));
    assert_eq!(runs(&acct), counts(1, 0), "置き直せば読む");
}

#[test]
fn mbig_log_released() {
    let work = Work::new("released");
    let acct = work.acct();
    assert_eq!(runs(&acct), counts(1, 0));
    work.git_says("state-b");
    assert_eq!(runs(&acct), counts(0, 1));
    work.swap_same_stamp("state-a", &format!("{CREATED}{}", stopped(SPAWNED)));
    work.git_says("state-a");
    assert_eq!(
        runs(&acct),
        counts(0, 1),
        "読まなかった state dir の字は放してあり、戻せば読み直す"
    );
}

/// 偽の bd（字の file を出し、down の file が在れば rc 1）を撃つ Source。
fn fake_source(name: &str) -> (Source, PathBuf) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("mbig")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("repo")).expect("repo");
    fs::write(root.join("bd-out"), LEDGER).expect("台帳の字");
    let bd = root.join("bd");
    script(
        &bd,
        &format!(
            "[ -e '{0}/down' ] && exit 1\nexec cat '{0}/bd-out'",
            root.display()
        ),
    );
    (Source::new(root.join("repo"), bd), root)
}

fn shared(a: &Option<Arc<str>>, b: &Option<Arc<str>>) -> bool {
    matches!((a, b), (Some(a), Some(b)) if Arc::ptr_eq(a, b))
}

#[test]
fn mbig_got_shared() {
    let (source, _) = fake_source("got-watched");
    let watched = source.watched();
    assert!(matches!(watched.read(), Reading::Known(_)));
    let (first, second, cloned) = (watched.got(), watched.got(), watched.clone().got());
    assert_eq!(first.text.as_deref(), Some(LEDGER));
    assert!(shared(&first.text, &second.text), "got は同じ確保");
    assert!(shared(&first.text, &cloned.text), "clone の got も同じ確保");

    let (alone, root) = fake_source("got-alone");
    let good = alone.got();
    assert_eq!(good.text.as_deref(), Some(LEDGER));
    assert_eq!(good.stale, None);
    fs::write(root.join("down"), "").expect("落とす印");
    let held = alone.got();
    assert!(held.stale.is_some(), "落ちた読みは stale が在る");
    assert!(shared(&good.text, &held.text), "落ちた後も同じ確保");
}

#[test]
fn mbig_texts_read_json() {
    let project = |text: &str| wire::decode::<ProjectTexts>(text).expect("ProjectTexts");
    assert_eq!(
        project("{\"state_dir_known\":true,\"events\":\"e-1\"}"),
        ProjectTexts {
            state_dir_known: true,
            events: Some("e-1".into()),
            ledger: None,
            ..ProjectTexts::default()
        }
    );
    assert_eq!(
        project("{\"events\":null,\"ledger\":\"[]\"}"),
        ProjectTexts {
            events: None,
            ledger: Some("[]".into()),
            ..ProjectTexts::default()
        }
    );
    assert_eq!(project("{}"), ProjectTexts::default());
    let host = |text: &str| wire::decode::<HostTexts>(text).expect("HostTexts");
    assert_eq!(
        host("{\"events\":\"h-1\"}"),
        HostTexts {
            events: Some("h-1".into()),
            ..HostTexts::default()
        }
    );
    assert_eq!(host("{}").events, None);
}
