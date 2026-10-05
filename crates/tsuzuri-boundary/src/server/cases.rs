//! 器の局面の出力の読み（file を 2 つ読むだけ・書かない）。
//! file は `<state dir>/fleet/lifecycle.json`（出力）と同じ dir の `lifecycle.stale`（古さの印・器の case-lifecycle §5.1）。
//! 器の読み手と同じく古さの印 → 出力の順に読む。state dir を省いたときは、出力は空の字で古さの印は無い。
//! 出力が無いか読めなければ空の字、古さの印は無ければ None・在って読めなければ空の字（中核の `cases_of` の入力の形）。
//! 2 つの file は見張りもする（`Cases::watch`）: 器は配車の周ごとに出力を書き直し、生成の時刻と
//! 入力の印はそのたびに動くので、更新時刻と長さが動いた周だけ字を読み、面が読む中身（`cases_of` の古さの印の種類と
//! 読めない版かどうかと部品）が前と違う時だけ局面の出力の種類（`ChangeKind::Cases`）の board-changed を送る。

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::case::CasePart;
use tsuzuri_contract::surface::ChangeKind;
use tsuzuri_core::case::cases_of;

use super::events::{Hub, now, stamp};

/// 局面の出力の file（state dir の下の path の区切りの列）。
pub const LIFECYCLE_JSON: [&str; 2] = ["fleet", "lifecycle.json"];

/// 古さの印の file（state dir の下の path の区切りの列）。
pub const LIFECYCLE_STALE: [&str; 2] = ["fleet", "lifecycle.stale"];

/// 面が読む局面の出力の中身（古さの印の種類と読めない版かと部品・生成の時刻は面が読まないので持たない）。
/// 読めない版か（`unreadable`）を持つので、出力が無い周と読めない版の周の行き来（どちらも部品は Unknown）でも動く。
pub type Sight = (Vec<String>, bool, Reading<Vec<CasePart>>);

/// 2 つの file の更新時刻と長さ（出力・古さの印の順・無ければ None）。
type Stamps = [Option<(SystemTime, u64)>; 2];

/// 局面の出力の読みの出所（state dir を省けばどちらも None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cases {
    pub json: Option<PathBuf>,
    pub stale: Option<PathBuf>,
}

impl Cases {
    pub fn new(state_dir: Option<&Path>) -> Cases {
        let under = |parts: [&str; 2]| {
            state_dir.map(|dir| parts.iter().fold(dir.to_path_buf(), |p, s| p.join(s)))
        };
        Cases {
            json: under(LIFECYCLE_JSON),
            stale: under(LIFECYCLE_STALE),
        }
    }

    /// 出力の字（無いか読めなければ空の字）と古さの印の字（無ければ None・在って読めなければ空の字）。
    pub fn texts(&self) -> (String, Option<String>) {
        let stale = self
            .stale
            .as_ref()
            .and_then(|path| match std::fs::read_to_string(path) {
                Ok(text) => Some(text),
                Err(e) if e.kind() == ErrorKind::NotFound => None,
                Err(_) => Some(String::new()),
            });
        let json = self
            .json
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok());
        (json.unwrap_or_default(), stale)
    }

    /// 面が読む中身（`texts` の字から中核の `cases_of` が組んだ古さの印の種類と読めない版かと部品）。
    pub fn sight(&self) -> Sight {
        let (json, stale) = self.texts();
        let doc = cases_of(&json, stale.as_deref());
        (doc.stale, doc.unreadable, doc.parts)
    }

    /// 2 つの file の更新時刻と長さ。
    fn stamps(&self) -> Stamps {
        [&self.json, &self.stale].map(|path| path.as_deref().and_then(stamp))
    }

    /// 局面の出力の見張りを始める（state dir を省いた読みは見張らない・Hub が落ちれば止まる）。
    /// `poll` ごとに 2 つの file の更新時刻と長さを見て、動いた周だけ `sight` を読み、前の周の中身と違えば
    /// 局面の出力の種類の board-changed を 1 件送る。中身の変わらない書き直し（生成の時刻と入力の印だけが動く）では
    /// 送らない。最初の印と中身は戻る前に取る（戻った後の変化は取りこぼさない）。
    pub fn watch(&self, hub: &Arc<Hub>, poll: Duration) {
        if self.json.is_none() {
            return;
        }
        let cases = self.clone();
        let weak = Arc::downgrade(hub);
        let mut seen = cases.stamps();
        let mut sight = cases.sight();
        thread::spawn(move || {
            loop {
                thread::sleep(poll);
                if weak.strong_count() == 0 {
                    return;
                }
                let current = cases.stamps();
                if current == seen {
                    continue;
                }
                seen = current;
                let read = cases.sight();
                if read == sight {
                    continue;
                }
                sight = read;
                let Some(hub) = weak.upgrade() else {
                    return;
                };
                hub.board_changed(&[ChangeKind::Cases], now());
            }
        });
    }
}
