//! 着地の commit の無い着地した契約の行の歯（境界・接頭辞 cmtcnt_・設計ノート surface-wave29a 行 k-commit-count・
//! 判断の記録 ADR-45 の門 H4・要件 FR3）。偽の bd（台帳の字の file を返す script）と偽の設計の道具（要約には要約の字・
//! ほかの引数には索引の字を返す script）を歯ごとの作業場に置き、event log の字を作業場の state dir に置いて、
//! tz graph --check を撃つ。否定の見本は正しい見本から 1 句だけ替える。
#![cfg(test)]
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsuzuri_boundary::cli::graph::{UNLANDED_HEAD, UNLANDED_UNKNOWN, unlanded_line};
use tsuzuri_boundary::server::board::{Texts, built, built_floors};
use tsuzuri_contract::board::Reading;
use tsuzuri_core::graph::check::unlanded_contracts;

/// 設計の索引（節点 1・辺 0）。
const INDEX: &str = "FR1\t要件\tsrs.yaml\t00000000\t面は 2 つ\n";

/// 節の要約の字（FR1 の 1 行）。
const SUM: &str = "{\"id\":\"FR1\",\"kind\":\"要件\",\"file\":\"srs.yaml\",\"plain\":\"見本\"}\n";

/// 着地の commit の名（40 字の小文字の 16 進）。
const SHA_1: &str = "1111111111111111111111111111111111111111";
const SHA_2: &str = "2222222222222222222222222222222222222222";

/// 根の epic x と契約 3 本（x.1 と x.2 は landed で閉じ、x.3 は取り下げで閉じる）。
const LEDGER: &str = r#"[
{"id": "x", "title": "根", "status": "open", "issue_type": "epic"},
{"id": "x.1", "title": "契約", "status": "closed", "issue_type": "task", "close_reason": "landed 1111111 ci=success", "acceptance_criteria": "design = contracts/n.toml#r1", "dependencies": [{"issue_id": "x.1", "depends_on_id": "x", "type": "parent-child"}]},
{"id": "x.2", "title": "契約", "status": "closed", "issue_type": "task", "close_reason": "landed 2222222 ci=success", "acceptance_criteria": "design = contracts/n.toml#r2", "dependencies": [{"issue_id": "x.2", "depends_on_id": "x", "type": "parent-child"}]},
{"id": "x.3", "title": "契約", "status": "closed", "issue_type": "task", "close_reason": "取り下げ 中身は x.1 の便に載った", "acceptance_criteria": "design = contracts/n.toml#r3", "dependencies": [{"issue_id": "x.3", "depends_on_id": "x", "type": "parent-child"}]}
]
"#;

const RUN_1: &str = "x.1-20260927T000000Z";
const RUN_2: &str = "x.2-20260927T010000Z";

/// event log の 1 行。
fn line(run: &str, kind: &str, detail: &str) -> String {
    format!(
        "{{\"schema\":1,\"ts\":\"2026-09-27T00:00:00Z\",\"kind\":\"{kind}\",\"run\":\"{run}\",\"detail\":\"{detail}\"}}\n"
    )
}

/// x.1 だけが着地の commit を持つ event log（x.2 の RunDone は札 sha: を持たない）。
fn events_one() -> String {
    [
        line(RUN_1, "RunCreated", ""),
        line(RUN_2, "RunCreated", ""),
        line(RUN_1, "RunDone", &format!("sha:{SHA_1} main:{SHA_1}")),
        line(RUN_2, "RunDone", "terminal:close:ok"),
    ]
    .concat()
}

/// x.1 と x.2 の両方が着地の commit を持つ event log（events_one から x.2 の RunDone の 1 句だけ替える）。
fn events_both() -> String {
    [
        line(RUN_1, "RunCreated", ""),
        line(RUN_2, "RunCreated", ""),
        line(RUN_1, "RunDone", &format!("sha:{SHA_1} main:{SHA_1}")),
        line(RUN_2, "RunDone", &format!("sha:{SHA_2} main:{SHA_2}")),
    ]
    .concat()
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("script");
    let mut perm = fs::metadata(path).expect("script の属性").permissions();
    perm.set_mode(0o755);
    fs::set_permissions(path, perm).expect("script の権限");
}

/// 歯ごとの作業場（偽の bd と偽の設計の道具・repo と state dir の置き場）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    state: PathBuf,
}

impl Place {
    fn new(name: &str, events: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cmtcnt")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let state = root.join("state");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::write(repo.join(".beads/issues.jsonl"), "{}\n").expect("印の file");
        fs::create_dir_all(state.join("fleet")).expect("state dir");
        fs::write(state.join("fleet/events.jsonl"), events).expect("event log");
        fs::write(root.join("ledger.json"), LEDGER).expect("台帳の字");
        fs::write(root.join("index.tsv"), INDEX).expect("索引の字");
        fs::write(root.join("summary.jsonl"), SUM).expect("要約の字");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("ledger.json").display()),
        );
        script(
            &root.join("folio"),
            &format!(
                "[ \"$3\" = --summary ] && exec cat '{}'\nexec cat '{}'",
                root.join("summary.jsonl").display(),
                root.join("index.tsv").display()
            ),
        );
        Place { root, repo, state }
    }

    /// tz graph --check を撃つ（`bd` が偽なら bd は無い path・`state` が偽なら state dir は無い path）。
    fn check(&self, bd: bool, state: bool) -> Output {
        let bd = if bd {
            self.root.join("bd")
        } else {
            self.root.join("no-such-bd")
        };
        let state = if state {
            self.state.clone()
        } else {
            self.root.join("no-such-state")
        };
        let args: Vec<OsString> = vec![
            "graph".into(),
            "--check".into(),
            "--repo".into(),
            self.repo.clone().into(),
            "--bd".into(),
            bd.into(),
            "--folio".into(),
            self.root.join("folio").into(),
            "--state-dir".into(),
            state.into(),
        ];
        Command::new(env!("CARGO_BIN_EXE_tz"))
            .args(args)
            .output()
            .expect("tz を撃つ")
    }
}

fn texts(events: &str) -> Texts {
    Texts {
        design: INDEX.to_string(),
        ledger: LEDGER.to_string(),
        events: events.to_string(),
        summary: SUM.to_string(),
        rulings: String::new(),
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("標準出力の字")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("標準エラーの字")
}

/// 標準出力の末の 3 行（着地した設計ノートの行・この行・要約の行）。
fn tail3(out: &Output) -> Vec<String> {
    let text = stdout(out);
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    lines[lines.len().saturating_sub(3)..].to_vec()
}

#[test]
fn cmtcnt_line_and_built_floors() {
    let head: &str = UNLANDED_HEAD;
    let unknown: &str = UNLANDED_UNKNOWN;
    assert_eq!(head, "着地の commit の無い着地した契約");
    assert_eq!(
        unknown,
        "着地の commit の無い着地した契約（台帳か走行の記録が読めない）"
    );
    let line: fn(&[String]) -> String = unlanded_line;
    assert_eq!(line(&[]), "着地の commit の無い着地した契約 0");
    assert_eq!(
        line(&["x.2".to_string(), "x.10".to_string()]),
        "着地の commit の無い着地した契約 2（x.2・x.10）"
    );

    let read = texts(&events_one());
    let (g, floors) = built_floors(&read);
    assert_eq!(g, built(&read));
    assert_eq!(floors.unlanded, unlanded_contracts(&g, LEDGER));
    assert_eq!(floors.unlanded, Reading::Known(vec!["x.2".to_string()]));

    let (_, floors) = built_floors(&texts(&events_both()));
    assert_eq!(floors.unlanded, Reading::Known(Vec::new()));

    let (_, floors) = built_floors(&texts(""));
    assert_eq!(floors.unlanded, Reading::Unknown);
}

#[test]
fn cmtcnt_check_names_after_landed_notes() {
    let place = Place::new("names", &events_one());
    let out = place.check(true, true);
    let tail = tail3(&out);
    assert_eq!(tail[0], "全部の行が着地した設計ノート 0", "{out:?}");
    assert_eq!(
        tail[1], "着地の commit の無い着地した契約 1（x.2）",
        "{out:?}"
    );
    assert!(tail[2].starts_with("tz graph --check: "), "{out:?}");
    let heads = stdout(&out)
        .lines()
        .filter(|l| l.starts_with(UNLANDED_HEAD))
        .count();
    assert_eq!(heads, 1);
    assert!(!stderr(&out).contains(UNLANDED_HEAD), "{out:?}");

    let place = Place::new("zero", &events_both());
    let out = place.check(true, true);
    assert_eq!(
        tail3(&out)[1],
        "着地の commit の無い着地した契約 0",
        "{out:?}"
    );
}

#[test]
fn cmtcnt_unread_goes_to_stderr() {
    let want = format!("# まだ分からない: {UNLANDED_UNKNOWN}");
    for (name, bd, state) in [("no-state", true, false), ("no-bd", false, true)] {
        let place = Place::new(name, &events_one());
        let out = place.check(bd, state);
        assert_eq!(out.status.code(), Some(2), "{name}: {out:?}");
        assert!(
            !stdout(&out).contains(UNLANDED_HEAD),
            "{name}: {}",
            stdout(&out)
        );
        let err = stderr(&out);
        assert_eq!(
            err.lines().filter(|l| *l == want).count(),
            1,
            "{name}: {err}"
        );
    }
}
