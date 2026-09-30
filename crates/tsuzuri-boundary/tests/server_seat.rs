//! 席の card の口の歯（接頭辞 server_seat_・設計ノート surface-base 便 e-seat の完了の条件）。
//! 偽の器は、受けた argv を記録の置き場（repo と state dir の外）に 1 行ずつ足し、argv の頭に応じて
//! 作業場の out-<出力> の字を標準出力へ出す script（file が無ければ rc 1・slow-<出力> が在れば 8 秒眠る）。
//! 字は fixture の seat-inputs.json の組から置く。server は同じ process の thread で 127.0.0.1 の空き port に立てる。
#![cfg(test)]

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::acct::{HEARTBEAT_OFF, HEARTBEAT_ON};
use tsuzuri_boundary::server::seat::{self, DOCTOR_HOLD, HOLD, SLOW_HOLD, Seat};
use tsuzuri_boundary::server::{Config, Server};
use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::seat::{SeatCard, SeatState};
use tsuzuri_contract::stats::NextStep;
use tsuzuri_contract::wire;
use tsuzuri_core::seat::SeatTexts;

const FIXTURE: &str = "seat/seat-inputs.json";

/// fixture の席の名（組の期待の card の target と同じ）。
const TARGET: &str = "proj-1:0.1";

/// 器の 3 つの出力の名（偽の器の out-<名> の file）。
const OUTPUTS: [&str; 3] = ["tick", "doctor", "usage"];

fn fixture(name: &str) -> SeatTexts {
    let text = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(FIXTURE),
    )
    .expect("fixture");
    let mut cases: BTreeMap<String, SeatTexts> = wire::decode(&text).expect("fixture の形");
    cases
        .remove(name)
        .unwrap_or_else(|| panic!("組 {name} が無い"))
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 群の名（server と同じく doctor の席の行の anchor と群の宣言から）。
fn group_of(texts: &SeatTexts) -> Option<String> {
    let anchor = tsuzuri_core::seat::anchor(texts.doctor.as_deref()?, TARGET)?;
    tsuzuri_core::seat::group_name(texts.host_toml.as_deref()?, &anchor)
}

/// 歯ごとの作業場（repo・面の file・state dir・群の記録の dir・記録の置き場）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    files: PathBuf,
    state: PathBuf,
    groups: PathBuf,
    log: PathBuf,
    texts: SeatTexts,
}

impl Place {
    /// 組 `case` の字を置いた作業場（出力の字が無い組は out の file を置かない）。
    fn new(name: &str, case: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("server_seat")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let place = Place {
            repo: root.join("repo"),
            files: root.join("files"),
            state: root.join("state"),
            groups: root.join("scribe2-host/groups"),
            log: root.join("log"),
            texts: fixture(case),
            root,
        };
        for dir in [
            place.repo.join(".beads"),
            place.files.clone(),
            place.state.join("fleet"),
            place.seat_dir(),
            place.groups.join("history"),
            place.log.clone(),
        ] {
            fs::create_dir_all(dir).expect("置き場");
        }
        fs::write(place.files.join("index.html"), "tz").expect("index.html");
        fs::write(place.state.join("fleet/events.jsonl"), "").expect("event log");
        script(&place.root.join("bd"), "echo '[]'");
        let (log, root) = (place.log.display(), place.root.display());
        script(
            &place.root.join("scribe2"),
            &format!(
                "printf '%s\\n' \"$*\" >> '{log}/argv'\n\
                 case \"$1\" in seat) out=tick ;; doctor) out=doctor ;; fleet) out=usage ;; *) exit 2 ;; esac\n\
                 if [ -e '{root}/slow-'\"$out\" ]; then exec sleep 8; fi\n\
                 exec cat '{root}/out-'\"$out\""
            ),
        );
        let t = &place.texts;
        let outputs = [&t.tick_status, &t.doctor, &t.usage];
        for (out, text) in OUTPUTS.iter().zip(outputs) {
            if let Some(text) = text {
                fs::write(place.root.join(format!("out-{out}")), text).expect("出力の字");
            }
        }
        let seat_dir = place.seat_dir();
        for (path, text) in [
            (seat_dir.join("state.jsonl"), &t.state_log),
            (seat_dir.join("tick-last"), &t.tick_last),
            (place.state.join("host.toml"), &t.host_toml),
        ] {
            if let Some(text) = text {
                fs::write(path, text).expect("state dir の file");
            }
        }
        if let Some(group) = group_of(t) {
            for (i, record) in t.records.iter().enumerate() {
                let path = if i == 0 {
                    place.groups.join(format!("{group}.account"))
                } else {
                    place
                        .groups
                        .join(format!("history/{group}.account.20260927T000000Z.{i}"))
                };
                fs::write(path, record).expect("群の記録");
            }
            // 別の群の記録は読まない。
            fs::write(
                place.groups.join("g-other.account"),
                "account=acct-9\nts=1\n",
            )
            .expect("別の群");
        }
        place
    }

    fn seat_dir(&self) -> PathBuf {
        self.state.join("seat").join(TARGET.replace(':', "_"))
    }

    /// 偽の器の出力 `out` を落とす（file が無いので rc 1）。
    fn fail(&self, out: &str) {
        fs::remove_file(self.root.join(format!("out-{out}"))).expect("出力の file を消す");
    }

    /// 偽の器の出力 `out` を 5 秒で返さなくする。
    fn slow(&self, out: &str) {
        fs::write(self.root.join(format!("slow-{out}")), "").expect("眠りの印");
    }

    /// 偽の器が受けた argv（1 回 1 行）。
    fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.log.join("argv"))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn config(&self, seat: bool, state_dir: bool) -> Config {
        Config {
            bd: self.root.join("bd").into(),
            state_dir: state_dir.then(|| self.state.clone()),
            bdw: self.root.join("bdw").into(),
            seat: seat.then(|| TARGET.to_string()),
            scribe2: self.root.join("scribe2").into(),
            ..Config::new(
                self.repo.clone(),
                "127.0.0.1:0".parse().expect("bind 先"),
                self.files.clone(),
            )
        }
    }

    fn serve_with(&self, config: &Config) -> SocketAddr {
        let server = Server::bind(config).expect("起動");
        let addr = server.local_addr().expect("口の住所");
        thread::spawn(move || server.run());
        addr
    }

    fn serve(&self) -> SocketAddr {
        self.serve_with(&self.config(true, true))
    }

    /// server が読んだはずの字（`drop` の出力を読めない扱いにする・群の記録は群の名が得られるときだけ）。
    fn expected_texts(&self, drop: &[&str]) -> SeatTexts {
        let mut t = self.texts.clone();
        for out in drop {
            match *out {
                "tick" => t.tick_status = None,
                "doctor" => t.doctor = None,
                "usage" => t.usage = None,
                other => panic!("知らない出力の名: {other}"),
            }
        }
        if group_of(&t).is_none() {
            t.records.clear();
        }
        t
    }
}

/// 中核の関数を同じ字で直に呼んだ card。
fn core_card(texts: &SeatTexts, now: u64) -> SeatCard {
    let anchor = texts
        .doctor
        .as_deref()
        .and_then(|d| tsuzuri_core::seat::anchor(d, TARGET));
    tsuzuri_core::seat::card(TARGET, anchor.as_deref(), texts, now)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("時計")
        .as_secs()
}

/// GET を 1 つ撃ち、（状態の code・本文）を返す。
fn get(addr: SocketAddr, path: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse().ok())
        .expect("状態の code");
    (status, body.to_string())
}

/// 口 /api/seat の card（200 で電文の形）。
fn seat_card(addr: SocketAddr) -> SeatCard {
    let (status, body) = get(addr, seat::PATH);
    assert_eq!(status, 200, "{body}");
    wire::decode(&body).unwrap_or_else(|e| panic!("席の card の形でない {e}: {body}"))
}

/// dir の中の file の path と byte の一覧。
fn tree(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).expect("dir を読む") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                out.push((path.clone(), Vec::new()));
                stack.push(path);
            } else {
                out.push((path.clone(), fs::read(&path).expect("file を読む")));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn server_seat_card_matches_core() {
    for case in ["run", "wait", "limit", "silent"] {
        let place = Place::new(&format!("match-{case}"), case);
        let addr = place.serve();
        let got = seat_card(addr);
        assert!(got.at <= now(), "時点は今より前の材料の時刻: {}", got.at);
        assert_eq!(
            got,
            core_card(&place.expected_texts(&[]), got.at),
            "組 {case}"
        );
        assert_eq!(got.target, TARGET);
    }
    // 読める組の中身（時刻に依らない欄）。
    let place = Place::new("match-fields", "run");
    let got = seat_card(place.serve());
    assert_eq!(got.state, SeatState::Run);
    assert_eq!(
        (got.account.as_deref(), got.model.as_deref()),
        (Some("acct-1"), Some("opus"))
    );
    assert_eq!(
        (&got.tick_healthy, &got.heartbeat),
        (&Reading::Known(true), &Reading::Known(true))
    );
    let Reading::Known(group) = &got.group else {
        panic!("群が Unknown");
    };
    assert_eq!(group.group, "g-a");
    let Reading::Known(moves) = &got.moves else {
        panic!("移動が Unknown");
    };
    assert_eq!(moves.len(), 2, "群の今の記録と過去の記録: {moves:?}");
}

#[test]
fn server_seat_argv_exact() {
    let place = Place::new("argv", "run");
    seat_card(place.serve());
    let mut calls = place.calls();
    calls.sort();
    let state = place.state.display().to_string();
    let mut want = vec![
        format!("seat tick status --state-dir {state}"),
        format!("doctor --state-dir {state}"),
        format!("fleet usage --show --state-dir {state}"),
    ];
    want.sort();
    assert_eq!(calls, want);
    assert!(calls.iter().any(|c| c.starts_with("fleet usage --show ")));
    assert_eq!(seat::USAGE_ARGS, ["fleet", "usage", "--show"]);
}

#[test]
fn server_seat_without_seat_or_state_dir() {
    for (name, seat, state_dir) in [
        ("no-seat", false, true),
        ("no-state", true, false),
        ("neither", false, false),
    ] {
        let place = Place::new(name, "run");
        let addr = place.serve_with(&place.config(seat, state_dir));
        let got = seat_card(addr);
        assert_eq!(got.state, SeatState::Unknown, "{name}");
        assert_eq!(got.since, None, "{name}");
        assert_eq!((got.account, got.model), (None, None), "{name}");
        assert_eq!(
            (got.tick_healthy, got.heartbeat),
            (Reading::Unknown, Reading::Unknown),
            "{name}"
        );
        assert_eq!(got.group, Reading::Unknown, "{name}");
        assert_eq!(got.usage, Reading::Unknown, "{name}");
        assert_eq!(got.spans, Reading::Unknown, "{name}");
        assert_eq!(got.moves, Reading::Unknown, "{name}");
        // 次の一手の口も器を撃たない。
        assert_eq!(get(addr, "/api/next").0, 200);
        assert!(
            place.calls().is_empty(),
            "{name}: 偽の器を撃つ {:?}",
            place.calls()
        );
    }
}

#[test]
fn server_seat_failed_output_only_its_fields() {
    for out in OUTPUTS {
        let place = Place::new(&format!("fail-{out}"), "run");
        place.fail(out);
        let got = seat_card(place.serve());
        assert_eq!(
            got,
            core_card(&place.expected_texts(&[out]), got.at),
            "{out} が落ちる"
        );
        assert_ne!(
            got,
            core_card(&place.expected_texts(&[]), got.at),
            "{out} が落ちても同じ card"
        );
        // 状態の記録から組む欄は残る。
        assert_eq!(got.state, SeatState::Run, "{out}");
        assert!(matches!(got.spans, Reading::Known(_)), "{out}");
    }
}

#[test]
fn server_seat_slow_output_is_unknown() {
    let place = Place::new("slow", "run");
    place.slow("usage");
    let addr = place.serve();
    let from = Instant::now();
    let got = seat_card(addr);
    let took = from.elapsed();
    assert!(took >= Duration::from_secs(4), "5 秒待たずに返る: {took:?}");
    assert!(
        took < Duration::from_secs(8),
        "眠りの終わりまで待つ: {took:?}"
    );
    assert_eq!(got.usage, Reading::Unknown);
    assert_eq!(got, core_card(&place.expected_texts(&["usage"]), got.at));
    assert!(matches!(got.group, Reading::Known(_)), "doctor の欄は残る");
}

#[test]
fn server_seat_next_uses_card() {
    let place = Place::new("next", "limit");
    let addr = place.serve();
    let (status, body) = get(addr, "/api/next");
    assert_eq!(status, 200, "{body}");
    let got: NextStep = wire::decode(&body).expect("次の一手の形");
    let card = core_card(&place.expected_texts(&[]), now());
    let want = tsuzuri_core::next_step::next_step_seat("[]\n", "", now(), Some(&card));
    assert_eq!(got, want);
    assert_eq!(got.lead, NextMove::LimitOrMove);
    assert_ne!(got, tsuzuri_core::next_step::next_step("[]\n", "", now()));
}

#[test]
fn server_seat_repo_and_state_bytes_unchanged() {
    let place = Place::new("bytes", "run");
    let addr = place.serve();
    let snapshot = || (tree(&place.repo), tree(&place.state), tree(&place.groups));
    let before = snapshot();
    seat_card(addr);
    assert_eq!(get(addr, "/api/next").0, 200);
    assert!(!place.calls().is_empty());
    assert!(
        before == snapshot(),
        "口の読みの後に repo か state dir か群の記録の byte が変わる"
    );
}

/// SSE の接続から `event` の frame を `limit` まで待つ。
fn wait_event(s: &mut TcpStream, seen: &mut String, event: &str, limit: Duration) -> bool {
    let deadline = Instant::now() + limit;
    let needle = format!("event: {event}\n");
    let mut buf = [0u8; 4096];
    while Instant::now() < deadline {
        if let Some(at) = seen.find(&needle) {
            seen.drain(..at + needle.len());
            return true;
        }
        let left = deadline.saturating_duration_since(Instant::now());
        s.set_read_timeout(Some(left.max(Duration::from_millis(10))))
            .expect("timeout");
        match s.read(&mut buf) {
            Ok(0) => return false,
            Ok(n) => seen.push_str(&String::from_utf8_lossy(&buf[..n])),
            Err(_) => {}
        }
    }
    seen.contains(&needle)
}

#[test]
fn server_seat_state_files_send_board_changed() {
    let place = Place::new("sse", "run");
    let addr = place.serve();
    let mut s = TcpStream::connect(addr).expect("接続");
    s.write_all(format!("GET /api/surface/events HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut seen = String::new();
    assert!(
        !wait_event(
            &mut s,
            &mut seen,
            "board-changed",
            Duration::from_millis(1500)
        ),
        "file が動かないのに知らせる"
    );
    let dir = place.seat_dir();
    let log = dir.join("state.jsonl");
    let mut text = fs::read_to_string(&log).expect("状態の記録");
    text.push_str(
        "{\"schema\":1,\"state\":\"idle\",\"event\":\"stop\",\"ts\":1790510399,\"sid\":\"s-1\"}\n",
    );
    fs::write(&log, text).expect("状態の記録を書き換える");
    assert!(
        wait_event(&mut s, &mut seen, "board-changed", Duration::from_secs(5)),
        "state.jsonl の変化が 5 秒以内に届かない"
    );
    fs::write(
        dir.join("tick-last"),
        "ts=1790510399 decision=nudge reason=state-stale\n",
    )
    .expect("合図の最後の判定を書き換える");
    assert!(
        wait_event(&mut s, &mut seen, "board-changed", Duration::from_secs(5)),
        "tick-last の変化が 5 秒以内に届かない"
    );
}

#[test]
fn server_seat_holds_outputs_five_seconds() {
    let place = Place::new("hold", "run");
    let addr = place.serve();
    let count = |calls: &[String], head: &str| calls.iter().filter(|c| c.starts_with(head)).count();
    let heads = ["seat tick status ", "doctor ", "fleet usage --show "];
    let first = seat_card(addr);
    let second = seat_card(addr);
    let calls = place.calls();
    assert_eq!(calls.len(), 3, "5 秒の中の 2 回の読み: {calls:?}");
    for head in heads {
        assert_eq!(count(&calls, head), 1, "{head}");
    }
    assert_eq!(first.state, second.state);
    thread::sleep(HOLD + Duration::from_millis(300));
    seat_card(addr);
    let calls = place.calls();
    assert_eq!(calls.len(), 4, "5 秒の後の読みは tick status だけ撃ち直す: {calls:?}");
    assert_eq!(count(&calls, heads[0]), 2, "{}", heads[0]);
    for head in &heads[1..] {
        assert_eq!(count(&calls, head), 1, "doctor は {DOCTOR_HOLD:?}・usage は {SLOW_HOLD:?} 持つ: {head}");
    }
}

#[test]
fn server_seat_no_new_dependencies() {
    let deps = |krate: &str| -> Vec<String> {
        let text = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join(krate)
                .join("Cargo.toml"),
        )
        .expect("Cargo.toml");
        let mut names = Vec::new();
        let mut inside = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                inside = line.ends_with("dependencies]");
                continue;
            }
            if inside
                && let Some((name, _)) = line.split_once('=')
                && !line.starts_with('#')
            {
                let name = name.trim();
                names.push(name.strip_suffix(".workspace").unwrap_or(name).to_string());
            }
        }
        names
    };
    assert_eq!(
        deps("tsuzuri-boundary"),
        ["folio", "tsuzuri-contract", "tsuzuri-core"]
    );
    assert_eq!(
        deps("tsuzuri-core"),
        ["serde", "serde_json", "tsuzuri-contract"]
    );
}

/// 停止の記録を置くか消し、偽の器の seat tick status の席の行の heartbeat の語を合わせる（器と同じ決まり）。
fn hbmark_switch(place: &Place, off: bool) {
    let path = place.seat_dir().join(HEARTBEAT_OFF);
    if off {
        fs::write(&path, "").expect("停止の記録を置く");
    } else {
        fs::remove_file(&path).expect("停止の記録を消す");
    }
    let out = place.root.join("out-tick");
    let (from, to) = if off {
        ("heartbeat=on", "heartbeat=off")
    } else {
        ("heartbeat=off", "heartbeat=on")
    };
    let row = format!("target={TARGET} ");
    let text: String = fs::read_to_string(&out)
        .expect("tick の出力の字")
        .lines()
        .map(|l| {
            let l = if l.contains(&row) {
                l.replace(from, to)
            } else {
                l.to_string()
            };
            format!("{l}\n")
        })
        .collect();
    fs::write(&out, text).expect("tick の出力の字を替える");
}

#[test]
fn hbmark_marks_name_three_files() {
    let one = Seat {
        program: "scribe2".into(),
        state_dir: "/s".into(),
        target: TARGET.to_string(),
        cwd: "/r".into(),
    };
    let dir = Path::new("/s/seat/proj-1_0.1");
    assert_eq!(
        one.marks()[..3],
        [
            dir.join("state.jsonl"),
            dir.join("tick-last"),
            dir.join("heartbeat-off")
        ]
    );
    assert_eq!(HEARTBEAT_OFF, "heartbeat-off");
    let odd = Seat {
        target: "../x".to_string(),
        ..one
    };
    assert!(odd.marks().is_empty(), "{:?}", odd.marks());
}

#[test]
fn hbmark_off_file_sends_board_changed() {
    let place = Place::new("hbmark-sse", "run");
    let addr = place.serve();
    let mut s = TcpStream::connect(addr).expect("接続");
    s.write_all(format!("GET /api/surface/events HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut seen = String::new();
    assert!(
        !wait_event(
            &mut s,
            &mut seen,
            "board-changed",
            Duration::from_millis(1500)
        ),
        "file が動かないのに知らせる"
    );
    let off = place.seat_dir().join(HEARTBEAT_OFF);
    fs::write(&off, "").expect("停止の記録を置く");
    assert!(
        wait_event(&mut s, &mut seen, "board-changed", Duration::from_secs(5)),
        "停止の記録を置いても 5 秒以内に届かない"
    );
    fs::remove_file(&off).expect("停止の記録を消す");
    assert!(
        wait_event(&mut s, &mut seen, "board-changed", Duration::from_secs(5)),
        "停止の記録を消しても 5 秒以内に届かない"
    );
}

#[test]
fn hbmark_card_follows_off_file() {
    let place = Place::new("hbmark-card", "run");
    let addr = place.serve();
    let count = |head: &str| place.calls().iter().filter(|c| c.starts_with(head)).count();
    assert_eq!(seat_card(addr).heartbeat, Reading::Known(true));
    assert_eq!(place.calls().len(), 3, "{:?}", place.calls());
    hbmark_switch(&place, true);
    assert_eq!(
        seat_card(addr).heartbeat,
        Reading::Known(false),
        "停止の記録の後の読み"
    );
    assert_eq!(place.calls().len(), 5, "tick status と doctor だけ: {:?}", place.calls());
    assert_eq!(seat_card(addr).heartbeat, Reading::Known(false));
    assert_eq!(place.calls().len(), 5, "印が動かなければ持ち回す");
    hbmark_switch(&place, false);
    assert_eq!(
        seat_card(addr).heartbeat,
        Reading::Known(true),
        "停止の記録を消した後の読み"
    );
    assert_eq!(place.calls().len(), 7, "{:?}", place.calls());
    for (head, n) in [("seat tick status ", 3), ("doctor ", 3), ("fleet usage --show ", 1)] {
        assert_eq!(count(head), n, "{head}");
    }
}

/// 偽の器の seat tick status の席の行の欄 heartbeat= を `words`（heartbeat= と heartbeat_by= の字）に替える。
fn hbon_tick_row(place: &Place, words: &str) {
    let out = place.root.join("out-tick");
    let row = format!("target={TARGET} ");
    let text: String = fs::read_to_string(&out)
        .expect("tick の出力の字")
        .lines()
        .map(|l| {
            let l = if l.contains(&row) {
                l.split(' ')
                    .filter(|t| !t.starts_with("heartbeat_by="))
                    .map(|t| {
                        if t.starts_with("heartbeat=") {
                            words.to_string()
                        } else {
                            t.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                l.to_string()
            };
            format!("{l}\n")
        })
        .collect();
    fs::write(&out, text).expect("tick の出力の字を替える");
}

#[test]
fn hbon_marks_name_on_file() {
    let one = Seat {
        program: "scribe2".into(),
        state_dir: "/s".into(),
        target: TARGET.to_string(),
        cwd: "/r".into(),
    };
    let dir = Path::new("/s/seat/proj-1_0.1");
    assert_eq!(
        one.marks(),
        [
            dir.join("state.jsonl"),
            dir.join("tick-last"),
            dir.join("heartbeat-off"),
            dir.join("heartbeat-on")
        ]
    );
    assert_eq!(HEARTBEAT_ON, "heartbeat-on");
}

#[test]
fn hbon_on_file_sends_board_changed() {
    let place = Place::new("hbon-sse", "run");
    let addr = place.serve();
    let mut s = TcpStream::connect(addr).expect("接続");
    s.write_all(format!("GET /api/surface/events HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .expect("要求を書く");
    let mut seen = String::new();
    assert!(
        !wait_event(
            &mut s,
            &mut seen,
            "board-changed",
            Duration::from_millis(1500)
        ),
        "file が動かないのに知らせる"
    );
    let on = place.seat_dir().join(HEARTBEAT_ON);
    fs::write(&on, "").expect("明示の on の記録を置く");
    assert!(
        wait_event(&mut s, &mut seen, "board-changed", Duration::from_secs(5)),
        "明示の on の記録を置いても 5 秒以内に届かない"
    );
    fs::remove_file(&on).expect("明示の on の記録を消す");
    assert!(
        wait_event(&mut s, &mut seen, "board-changed", Duration::from_secs(5)),
        "明示の on の記録を消しても 5 秒以内に届かない"
    );
}

#[test]
fn hbon_card_reads_tick_word() {
    let place = Place::new("hbon-card", "run");
    hbon_tick_row(&place, "heartbeat=off heartbeat_by=group");
    let addr = place.serve();
    let on = place.seat_dir().join(HEARTBEAT_ON);
    assert_eq!(seat_card(addr).heartbeat, Reading::Known(false));
    assert_eq!(place.calls().len(), 3, "{:?}", place.calls());
    fs::write(&on, "").expect("明示の on の記録を置く");
    hbon_tick_row(&place, "heartbeat=on heartbeat_by=explicit");
    assert_eq!(
        seat_card(addr).heartbeat,
        Reading::Known(true),
        "明示の on の記録の後の読み"
    );
    assert_eq!(place.calls().len(), 5, "tick status と doctor だけ: {:?}", place.calls());
    assert_eq!(seat_card(addr).heartbeat, Reading::Known(true));
    assert_eq!(place.calls().len(), 5, "印が動かなければ持ち回す");
    fs::remove_file(&on).expect("明示の on の記録を消す");
    assert_eq!(
        seat_card(addr).heartbeat,
        Reading::Known(true),
        "file の有無でなく行の字で読む"
    );
    assert_eq!(place.calls().len(), 7, "{:?}", place.calls());
    fs::write(&on, "").expect("明示の on の記録を置き直す");
    hbon_tick_row(&place, "heartbeat=unreadable heartbeat_by=unreadable");
    assert_eq!(seat_card(addr).heartbeat, Reading::Unknown, "読めない字");
    assert_eq!(place.calls().len(), 9, "{:?}", place.calls());
}
