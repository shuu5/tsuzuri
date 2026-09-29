//! GET /api/account — account board の読み（AccountDoc・行 h-wire）。

use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};
use std::thread;

use tsuzuri_contract::account::PATH;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;

use crate::acct::Acct;
use crate::server::http::Response;
use crate::server::route::{Entry, Key, Match};
use crate::server::{ACCT_MARKS_EVERY, Shared, aged, events, json, lock};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(PATH),
    },
    handle: |_, shared| account(shared),
};

/// account board の読み。state dir が無ければ器も git も撃たず、口座と群と移動が Unknown で列が空の電文（at は 0）。
/// 最初の要求で acct の印の取り直しを始める（要求は取り直しを待たない）。
/// 台帳の読みが落ちた project が在れば、頭に最も古い最後に読めた時からの秒を付ける（行 e-hold）。
fn account(shared: &Shared) -> Response {
    let (doc, stale) = match &shared.acct {
        Some(acct) => {
            shared
                .acct_watch
                .call_once(|| watch_acct_marks(Arc::clone(acct), &shared.acct_marks));
            acct.doc_read(events::now())
        }
        None => (
            tsuzuri_core::account::project::assemble(
                Reading::Unknown,
                Reading::Unknown,
                Reading::Unknown,
                Vec::new(),
                Vec::new(),
            ),
            None,
        ),
    };
    aged(json(200, wire::encode(&doc)), stale)
}

/// acct の印の一覧を、始めてすぐと `ACCT_MARKS_EVERY` ごとに `Acct::marks` で置き換える別の thread
/// （一覧の持ち手が落ちれば止まる）。
fn watch_acct_marks(acct: Arc<Acct>, marks: &Arc<Mutex<Vec<PathBuf>>>) {
    let weak: Weak<Mutex<Vec<PathBuf>>> = Arc::downgrade(marks);
    thread::spawn(move || {
        loop {
            let current = acct.marks();
            let Some(marks) = weak.upgrade() else {
                return;
            };
            *lock(&marks) = current;
            drop(marks);
            thread::sleep(ACCT_MARKS_EVERY);
        }
    });
}
