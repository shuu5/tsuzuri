//! 答えの口の書きの前の台帳の読みの上限と落ちた訳の歯（接頭辞 areread_・設計ノート surface-wave23a 行 e-answer-reread）。
//! server を立てず、`Source` の読みと `ruling::reread` を直に撃つ。
//! 偽の bd は撃たれた回を数え、場ごとに script に書いた秒だけ寝てから out.json（fixture の写し）を出す script で、
//! 落ちる場の偽の bd は標準エラーに 1 行 `database is locked` を書いて rc 1 で終わる。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::common::fixture;
use tsuzuri_boundary::server::ledger::{BD_TIMEOUT, Source};
use tsuzuri_boundary::server::ruling::{
    READ_TIMEOUT, READ_TRIES, RETRY_STEP, UNREAD, reread, unread_line,
};

/// 偽の bd の出力の見本。
const LEDGER: &str = "stop/ledger.json";

/// 落ちる偽の bd が標準エラーに書く 1 行。
const LOCKED: &str = "database is locked";

/// 偽の bd の振る舞い。
enum Fake {
    /// 秒だけ寝てから out.json を出す。
    Sleep(u32),
    /// 標準エラーに `LOCKED` の 1 行を書いて rc 1 で終わる。
    Locked,
}

/// 歯ごとの作業場（repo と偽の bd）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
}

impl Place {
    fn new(name: &str, fake: Fake) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("areread")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::write(root.join("out.json"), fixture(LEDGER)).expect("偽の bd の出力");
        let r = root.display();
        let tail = match fake {
            Fake::Sleep(secs) => format!("sleep {secs}\nexec cat '{r}/out.json'"),
            Fake::Locked => format!("echo '{LOCKED}' >&2\nexit 1"),
        };
        let bd = root.join("bd");
        fs::write(
            &bd,
            format!(
                "#!/bin/sh\n\
                 n=$(( $(cat '{r}/bd.count' 2>/dev/null || echo 0) + 1 ))\n\
                 echo \"$n\" > '{r}/bd.count'\n\
                 {tail}\n"
            ),
        )
        .expect("偽の bd");
        fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
        Place { root, repo }
    }

    fn source(&self) -> Source {
        Source::new(self.repo.clone(), self.root.join("bd"))
    }

    /// 偽の bd が撃たれた回の数。
    fn bd_count(&self) -> u32 {
        fs::read_to_string(self.root.join("bd.count"))
            .map_or(0, |c| c.trim().parse().expect("回の数"))
    }
}

/// (1) 上限と回と間の値、UNREAD の字と unread_line の 1 行。
#[test]
fn areread_limits_and_words() {
    assert_eq!(READ_TIMEOUT, Duration::from_secs(20));
    assert_eq!(BD_TIMEOUT, Duration::from_secs(5));
    assert_eq!(READ_TRIES, 3);
    assert_eq!(RETRY_STEP, Duration::from_secs(1));
    assert_eq!(UNREAD, "書きの前の台帳の読みが落ちた");
    assert_eq!(
        unread_line(&[
            "時間切れ".to_string(),
            format!("rc が 0 でない: {LOCKED}"),
        ]),
        "tz surface serve: 書きの前の台帳の読みが落ちた: 時間切れ・rc が 0 でない: database is locked"
    );
}

/// (2) 6 秒かかる読みは表示の読み（text_alone）では落ち、書きの前の読み（reread）では読める。
#[test]
fn areread_slow_read_is_taken() {
    let place = Place::new("slow", Fake::Sleep(6));
    let source = place.source();
    assert_eq!(source.text_alone(), None, "5 秒の上限で落ちない");
    assert_eq!(reread(&source), Some(fixture(LEDGER)));
    assert_eq!(place.bd_count(), 2, "偽の bd は合わせて 2 回");
}

/// (3) 落ちた訳の字（rc と時間切れ）と、どれも落ちたときの reread の回。
#[test]
fn areread_down_words() {
    let place = Place::new("locked", Fake::Locked);
    assert_eq!(
        place.source().text_within(BD_TIMEOUT),
        Err(format!("rc が 0 でない: {LOCKED}"))
    );
    let place = Place::new("locked-reread", Fake::Locked);
    assert_eq!(reread(&place.source()), None);
    assert_eq!(place.bd_count(), READ_TRIES, "偽の bd は 3 回");

    let place = Place::new("slow-limit", Fake::Sleep(3));
    assert_eq!(
        place.source().text_within(Duration::from_millis(500)),
        Err("時間切れ".to_string())
    );
}
