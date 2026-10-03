//! 起動の掃きの歯（接頭辞 bvdsweep_・設計ノート surface-wave26b 行 c-deliver-retry・要件 FR9）。
//! 偽の bd は作業場の out.json を標準出力へ出す script（out.json が無ければ標準エラーに 1 行を書いて rc 1）、
//! 偽の bdw と偽の器は撃たれた回ごとの argv を記録して rc 0 で終わる script。
//! 台帳（`Ids::ledger`）は問い 4 本で、今の分に記帳した fx-w.1 と fx-w.3 の裁定は印が無く、fx-w.2 の裁定は印が在り、
//! 2 時間前の分に記帳した fx-w.4 の裁定は印が無い。
//! `sweep` と `sweep_at_start` を直に呼び、起動の入口は tz の binary を撃つ。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_boundary::server::ruling::{
    Delivery, Pace, SWEEP_FOUND, SWEEP_UNREAD, Writer, id_minute, minute, sweep, sweep_at_start,
};
use tsuzuri_boundary::server::{Config, events};
use tsuzuri_contract::surface::RulingId;

/// 器の配達の口の target。
const TARGET: &str = "tsuzuri:0.1";

/// 古い裁定の分（今から 2 時間前）。
const OLD_SECS: u64 = 7200;

/// 歯の撃ち直しの組。
const FAST: Pace = Pace {
    step: Duration::from_millis(50),
    span: Duration::from_millis(2000),
};

/// 台帳の 4 つの裁定の id（今の分の印の無い 2 つ・今の分の印の在る 1 つ・2 時間前の分の印の無い 1 つ）。
struct Ids {
    first: String,
    second: String,
    marked: String,
    old: String,
}

impl Ids {
    fn now() -> Ids {
        let now = events::now();
        let (at, old) = (minute(now), minute(now - OLD_SECS));
        Ids {
            first: format!("fx-w.1:{at}-1"),
            second: format!("fx-w.3:{at}-1"),
            marked: format!("fx-w.2:{at}-1"),
            old: format!("fx-w.4:{old}-1"),
        }
    }

    /// 偽の bd が出す台帳（`marks` の裁定と fx-w.2 の裁定には経路が停止の印の行を足す）。
    fn ledger(&self, marks: &[&str]) -> String {
        let beads: Vec<String> = [
            ("fx-w.1", &self.first),
            ("fx-w.2", &self.marked),
            ("fx-w.3", &self.second),
            ("fx-w.4", &self.old),
        ]
        .iter()
        .map(|(q, r)| {
            let mut notes = format!("裁定 id = {r}・問い = {q}・逐語 = はい");
            if *r == &self.marked || marks.contains(&r.as_str()) {
                notes.push_str(&format!("\\n配達 = {r}・経路 = 停止・時刻 = 20261002T0101Z"));
            }
            format!(
                "{{\"id\":\"{q}\",\"status\":\"closed\",\"labels\":[\"intake:question\"],\"notes\":\"{notes}\"}}"
            )
        })
        .collect();
        format!("[{}]", beads.join(",\n"))
    }
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 撃たれた回ごとに argv を `<log>/<name>.<回>.args` に書き、回の数を `<log>/<name>.count` に書く字。
fn record(log: &Path, name: &str) -> String {
    let log = log.display();
    format!(
        "n=$(( $(cat '{log}/{name}.count' 2>/dev/null || echo 0) + 1 ))\n\
         echo \"$n\" > '{log}/{name}.count'\n\
         for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{log}/{name}.'\"$n\"'.args'"
    )
}

/// 歯ごとの作業場（repo・面の file の置き場・state dir・記録の置き場・偽の program）。
struct Place {
    root: PathBuf,
    log: PathBuf,
}

impl Place {
    /// `ledger` が Some なら偽の bd はその字を出し、None なら落ちる。
    fn new(tooth: &str, ledger: Option<&str>) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("bvdsweep")
            .join(tooth);
        let _ = fs::remove_dir_all(&root);
        let log = root.join("log");
        for dir in [
            root.join("repo/.beads"),
            root.join("files"),
            root.join("state"),
            log.clone(),
        ] {
            fs::create_dir_all(dir).expect("作業場の dir");
        }
        fs::write(root.join("files/index.html"), "tz").expect("index.html");
        if let Some(ledger) = ledger {
            fs::write(root.join("out.json"), ledger).expect("偽の bd の出力");
        }
        let out = root.join("out.json");
        script(
            &root.join("bd"),
            &format!(
                "[ -f '{o}' ] || {{ echo 'database is locked' >&2; exit 1; }}\nexec cat '{o}'",
                o = out.display()
            ),
        );
        script(
            &root.join("bdw"),
            &format!("{}\nexit 0", record(&log, "bdw")),
        );
        script(
            &root.join("scribe2"),
            &format!("{}\nexit 0", record(&log, "scribe2")),
        );
        Place { root, log }
    }

    fn delivery(&self) -> Delivery {
        Delivery {
            program: self.root.join("scribe2").into(),
            state_dir: self.root.join("state"),
            target: TARGET.to_string(),
        }
    }

    fn writer(&self) -> Writer {
        Writer {
            repo: self.root.join("repo"),
            bdw: self.root.join("bdw").into(),
            delivery: Some(self.delivery()),
        }
    }

    fn source(&self) -> Source {
        Source::new(self.root.join("repo"), self.root.join("bd"))
    }

    /// 起動の引数と同じ形の Config（`seat` と `state` で --seat と --state-dir を選ぶ）。
    fn config(&self, seat: bool, state: bool, read_only: bool) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            bdw: self.root.join("bdw").into(),
            scribe2: self.root.join("scribe2").into(),
            seat: seat.then(|| TARGET.to_string()),
            state_dir: state.then(|| self.root.join("state")),
            read_only,
            ..Config::new(
                self.root.join("repo"),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.root.join("files"),
            )
        }
    }

    /// 偽の program が撃たれた回の数。
    fn count(&self, name: &str) -> u32 {
        fs::read_to_string(self.log.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"))
    }

    /// 偽の program の `n` 回目の argv（1 つ 1 行）。
    fn args(&self, name: &str, n: u32) -> Vec<String> {
        fs::read_to_string(self.log.join(format!("{name}.{n}.args")))
            .expect("argv の記録")
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// 偽の program が撃たれた全部の回の argv。
    fn all_args(&self, name: &str) -> Vec<String> {
        (1..=self.count(name))
            .flat_map(|n| self.args(name, n))
            .collect()
    }

    /// 偽の器が `n` 回撃たれるまで 30 秒を上限に待つ（越えれば偽）。
    fn wait_scribe2(&self, n: u32) -> bool {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(30) {
            if self.count("scribe2") >= n {
                return true;
            }
            thread::sleep(Duration::from_millis(50));
        }
        false
    }

    /// 歯の組で `sweep` を撃つ。
    fn sweep(&self) -> usize {
        sweep(
            &self.delivery(),
            &self.writer(),
            &self.source(),
            FAST,
            events::now(),
        )
    }
}

/// 器の配達の口の argv（`--ruling` の値まで）。
fn deliver_args(place: &Place, ruling: &str) -> Vec<String> {
    [
        "seat",
        "deliver",
        "--state-dir",
        &place.root.join("state").display().to_string(),
        "--target",
        TARGET,
        "--ruling",
        ruling,
    ]
    .map(str::to_string)
    .to_vec()
}

/// (3) 起動の掃きの log の 2 つの字と、裁定の id の分の字の読み。
#[test]
fn bvdsweep_words() {
    assert_eq!(SWEEP_FOUND, "起動の周に印の無い裁定を撃ち直す");
    assert_eq!(
        SWEEP_UNREAD,
        "起動の周の台帳の読みが落ちた（撃ち直さず、印の無い裁定は席の停止の hook が拾う）"
    );
    let id = |s: &str| RulingId::new(s).expect("id");
    assert_eq!(
        id_minute(&id("fx-w.1:20261002T0100Z-1")),
        Some("20261002T0100Z")
    );
    assert_eq!(
        id_minute(&id("batch:20261002T0100Z-12")),
        Some("20261002T0100Z")
    );
    for bad in [
        "fx-w.1",
        "fx-w.1:20261002T0100Z",
        "fx-w.1:2026100T0100Z-1",
        "fx-w.1:20261002T0100Z-x",
    ] {
        assert_eq!(id_minute(&id(bad)), None, "{bad}");
    }
}

/// (4) 今の分の印の無い 2 つの裁定を台帳の順に 1 つずつ器の配達の口で撃ち、受けた周ごとに印を 1 つ書き、
/// 印の在る裁定と、上限より前の分の印の無い裁定は撃たない。
#[test]
fn bvdsweep_fires_unmarked_in_order() {
    let ids = Ids::now();
    let place = Place::new("unmarked", Some(&ids.ledger(&[])));
    assert_eq!(place.sweep(), 2, "撃った荷の数");
    assert_eq!(place.count("scribe2"), 2, "偽の器は 2 回");
    assert_eq!(place.args("scribe2", 1), deliver_args(&place, &ids.first));
    assert_eq!(place.args("scribe2", 2), deliver_args(&place, &ids.second));
    assert_eq!(place.count("bdw"), 2, "印の書きは 2 回");
    for (n, ruling) in [(1, &ids.first), (2, &ids.second)] {
        let mark = format!("配達 = {ruling}・経路 = 配達の口・時刻 = ");
        assert!(
            place.args("bdw", n).iter().any(|a| a.contains(&mark)),
            "{n} 回目の印: {:?}",
            place.args("bdw", n)
        );
    }
    let all = place.all_args("scribe2");
    for skipped in [&ids.marked, &ids.old] {
        assert!(!all.contains(skipped), "{skipped} を撃つ");
    }
}

/// (5) 印の無い裁定が上限より前の分のものだけの台帳と、読めない台帳では、器も bdw も撃たず 0 を返す。
#[test]
fn bvdsweep_quiet_when_nothing() {
    let ids = Ids::now();
    let place = Place::new("old-only", Some(&ids.ledger(&[&ids.first, &ids.second])));
    assert_eq!(place.sweep(), 0);
    assert_eq!((place.count("scribe2"), place.count("bdw")), (0, 0));

    let place = Place::new("down", None);
    assert_eq!(place.sweep(), 0);
    assert_eq!((place.count("scribe2"), place.count("bdw")), (0, 0));
}

/// (6) sweep_at_start は読むだけの server・--seat の無い server・--state-dir の無い server では始めず偽を返し、
/// 両方が在る読むだけでない server では真を返して別の thread で今の分の最初の印の無い裁定を撃つ。
#[test]
fn bvdsweep_start_gates() {
    let ids = Ids::now();
    let place = Place::new("gates", Some(&ids.ledger(&[])));
    assert!(!sweep_at_start(&place.config(true, true, true)), "読むだけ");
    assert!(
        !sweep_at_start(&place.config(false, true, false)),
        "--seat が無い"
    );
    assert!(
        !sweep_at_start(&place.config(true, false, false)),
        "--state-dir が無い"
    );
    thread::sleep(Duration::from_millis(500));
    assert_eq!(place.count("scribe2"), 0, "始めない server は撃たない");

    assert!(sweep_at_start(&place.config(true, true, false)));
    assert!(place.wait_scribe2(1), "始めた掃きが器を撃つ");
    assert_eq!(place.args("scribe2", 1), deliver_args(&place, &ids.first));
}

/// (7) tz surface serve は --seat と --state-dir が在れば、起動の後に今の分の最初の印の無い裁定を器の配達の口で撃つ。
#[test]
fn bvdsweep_serve_sweeps_at_start() {
    let ids = Ids::now();
    let place = Place::new("serve", Some(&ids.ledger(&[])));
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz"))
        .args(["surface", "serve", "--bind", "127.0.0.1:0", "--repo"])
        .arg(place.root.join("repo"))
        .arg("--files")
        .arg(place.root.join("files"))
        .arg(format!("--bd={}", place.root.join("bd").display()))
        .arg(format!("--bdw={}", place.root.join("bdw").display()))
        .args(["--seat", TARGET])
        .arg(format!(
            "--state-dir={}",
            place.root.join("state").display()
        ))
        .arg(format!(
            "--scribe2={}",
            place.root.join("scribe2").display()
        ))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("tz を撃つ");
    let fired = place.wait_scribe2(1);
    let _ = child.kill();
    let _ = child.wait();
    assert!(fired, "起動の後に器を撃たない");
    assert_eq!(place.args("scribe2", 1), deliver_args(&place, &ids.first));
}
