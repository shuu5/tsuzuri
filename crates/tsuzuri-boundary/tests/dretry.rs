//! 配達の撃ち直しの歯（接頭辞 dretry_・設計ノート surface-wave23a 行 e-deliver-retry・要件 FR9）。
//! 偽の bd は作業場の out.json を標準出力へ出す script、偽の bdw は撃たれた回ごとの argv を記録する script、
//! 偽の器は回ごとの argv を記録し、作業場の scribe2.fails の数の回までは作業場の on-fail を sh で撃ってから
//! 標準エラーに 3 行（前の行・断りの 1 行・空の行）を書いて rc 1 で終わる script。
//! 裁定の受付の `deliver` と `redeliver` を直に呼ぶ。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tsuzuri_boundary::server::ledger::Source;
use tsuzuri_boundary::server::ruling::{self, Delivery, Pace, Parcel, Round, Writer};
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::surface::RulingId;
use tsuzuri_core::delivery::{Pending, Route, mark_line};

/// 器の配達の口の target。
const TARGET: &str = "tsuzuri:0.1";

/// 裁定（fixture の fx-s.2 の印の無い裁定）。
const QUESTION: &str = "fx-s.2";
const RULING: &str = "fx-s.2:20260928T0101Z-1";

/// 偽の器の断りの 1 行。
const REFUSED: &str = "seat deliver: refused reason=input-busy target=tsuzuri:0.1";

/// 歯の撃ち直しの組（`dretry_gives_up_at_span` の他）。
const FAST: Pace = Pace {
    step: Duration::from_millis(50),
    span: Duration::from_millis(2000),
};

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(rel: &str) -> String {
    fs::read_to_string(manifest().join("../../tests/fixtures").join(rel)).expect("fixture")
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

/// 場ごとの作業場（repo・state dir・記録の置き場・out.json・偽の program）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(tooth: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("dretry")
            .join(tooth);
        let _ = fs::remove_dir_all(&root);
        let (repo, state, log) = (root.join("repo"), root.join("state"), root.join("log"));
        for dir in [&repo, &state, &log] {
            fs::create_dir_all(dir).expect("作業場の dir");
        }
        fs::write(root.join("out.json"), fixture("stop/ledger.json")).expect("偽の bd の出力");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("out.json").display()),
        );
        script(
            &root.join("bdw"),
            &format!("{}\nexit 0", record(&log, "bdw")),
        );
        let r = root.display();
        script(
            &root.join("scribe2"),
            &format!(
                "{}\n\
                 if [ \"$n\" -le \"$(cat '{r}/scribe2.fails' 2>/dev/null || echo 0)\" ]; then\n\
                 [ -f '{r}/on-fail' ] && sh '{r}/on-fail'\n\
                 printf '%s\\n%s\\n\\n' 'seat deliver: 前の行' '{REFUSED}' >&2\n\
                 exit 1\n\
                 fi\n\
                 exit 0",
                record(&log, "scribe2")
            ),
        );
        Place {
            root,
            repo,
            state,
            log,
        }
    }

    /// 偽の器を最初の `n` 回だけ断らせる。
    fn fails(&self, n: u32) {
        fs::write(self.root.join("scribe2.fails"), n.to_string()).expect("断る回");
    }

    /// 偽の器が断る周に撃つ sh の字。
    fn on_fail(&self, body: &str) {
        fs::write(self.root.join("on-fail"), body).expect("on-fail");
    }

    /// 偽の program が撃たれた回の数。
    fn count(&self, name: &str) -> u32 {
        fs::read_to_string(self.log.join(format!("{name}.count")))
            .map_or(0, |c| c.trim().parse().expect("回の数"))
    }

    fn delivery(&self) -> Delivery {
        Delivery {
            program: self.root.join("scribe2").into(),
            state_dir: self.state.clone(),
            target: TARGET.to_string(),
        }
    }

    fn writer(&self) -> Writer {
        Writer {
            repo: self.repo.clone(),
            bdw: self.root.join("bdw").into(),
            delivery: Some(self.delivery()),
        }
    }

    fn source(&self) -> Source {
        Source::new(self.repo.clone(), self.root.join("bd"))
    }

    fn deliver(&self) -> Round {
        ruling::deliver(
            &self.delivery(),
            &self.writer(),
            &self.source(),
            &ruling_id(),
            &pending(),
        )
    }

    fn redeliver(&self, pace: Pace) {
        let parcel = Parcel {
            id: ruling_id(),
            pending: pending(),
        };
        ruling::redeliver(
            &self.delivery(),
            &self.writer(),
            &self.source(),
            &parcel,
            pace,
        );
    }
}

fn ruling_id() -> RulingId {
    RulingId::new(RULING).expect("裁定の id")
}

fn pending() -> Vec<Pending> {
    vec![Pending {
        question: BeadId::new(QUESTION).expect("bead id"),
        ruling: ruling_id(),
    }]
}

#[test]
fn dretry_pace_and_words() {
    assert_eq!(ruling::DELIVER_STEP, Duration::from_secs(15));
    assert_eq!(ruling::DELIVER_SPAN, Duration::from_secs(1800));
    assert_eq!(
        ruling::PACE,
        Pace {
            step: ruling::DELIVER_STEP,
            span: ruling::DELIVER_SPAN,
        }
    );
    assert_eq!(
        ruling::NOT_TAKEN,
        "配達の口が今は受けない（印を置かず、席の停止の hook が拾う）"
    );
    assert_eq!(
        ruling::GAVE_UP,
        "配達の撃ち直しを上限で止めた（印を置かず、席の停止の hook が拾う）"
    );
}

#[test]
fn dretry_round_keeps_vessel_line() {
    let place = Place::new("round_keeps_vessel_line-refused");
    place.fails(1);
    assert_eq!(
        place.deliver(),
        Round::NotTaken(format!("rc が 0 でない: {REFUSED}"))
    );
    assert_eq!(place.count("scribe2"), 1);
    assert_eq!(place.count("bdw"), 0, "受けないのに印を置く");

    let place = Place::new("round_keeps_vessel_line-taken");
    assert_eq!(place.deliver(), Round::Taken);
    assert_eq!(place.count("scribe2"), 1);
    assert_eq!(place.count("bdw"), 1);
}

#[test]
fn dretry_retries_until_taken() {
    let place = Place::new("retries_until_taken");
    place.fails(2);
    place.redeliver(FAST);
    assert_eq!(place.count("scribe2"), 3);
    assert_eq!(place.count("bdw"), 1);
}

#[test]
fn dretry_stops_when_marked() {
    let place = Place::new("stops_when_marked");
    let stop = mark_line(&ruling_id(), Route::Stop, "20260928T0110Z");
    let marked = fixture("stop/ledger.json").replacen(
        "逐語 = よい\"",
        &format!("逐語 = よい\\n{stop}\""),
        1,
    );
    fs::write(place.root.join("marked.json"), marked).expect("印の在る台帳");
    place.on_fail(&format!(
        "cp '{}' '{}'\n",
        place.root.join("marked.json").display(),
        place.root.join("out.json").display()
    ));
    place.fails(1000);
    place.redeliver(FAST);
    assert_eq!(place.count("scribe2"), 1);
    assert_eq!(place.count("bdw"), 0, "印の在る裁定に印を置く");
}

#[test]
fn dretry_gives_up_at_span() {
    let place = Place::new("gives_up_at_span");
    place.fails(1000);
    let from = Instant::now();
    place.redeliver(Pace {
        step: Duration::from_millis(50),
        span: Duration::from_millis(400),
    });
    assert!(from.elapsed() < Duration::from_secs(10), "{:?}", from.elapsed());
    let n = place.count("scribe2");
    assert!((2..=9).contains(&n), "偽の器を撃った回: {n}");
    assert_eq!(place.count("bdw"), 0, "受けないのに印を置く");
}

#[test]
fn dretry_spawns_use_pace() {
    let read = |rel: &str| fs::read_to_string(manifest().join(rel)).expect("src を読む");
    for (rel, want) in [("src/server/ruling.rs", 2), ("src/server/batch.rs", 1)] {
        let text = read(rel);
        assert_eq!(
            text.matches("std::thread::spawn(move || redeliver(").count(),
            want,
            "{rel}"
        );
        assert_eq!(text.matches("PACE));").count(), want, "{rel}");
        assert!(!text.contains("thread::spawn(move || deliver("), "{rel}");
    }
}
