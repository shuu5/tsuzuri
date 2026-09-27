//! 口の型と振り分け（判断の記録 ADR-13・案 M2・条 P-21）。
//! 口は src/server/routes の下に 1 口 1 file で置き、各 file は `ROUTE` と口の本文を持つ。
//! 口の列（列挙 `Route`）は組み立ての script（build.rs）が dir から生成する。

use super::Route;
use super::Shared;
use super::http::{Request, Response};

/// 口の path の当たり方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match {
    /// path がこの字に等しい。
    Exact(&'static str),
    /// path がこの字で始まる。
    Prefix(&'static str),
}

/// 口の鍵（method の字 GET か POST と path の当たり方）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub method: &'static str,
    pub path: Match,
}

impl Key {
    /// 要求の method と path がこの鍵に当たる。
    pub fn matches(&self, method: &str, path: &str) -> bool {
        self.method == method
            && match self.path {
                Match::Exact(p) => path == p,
                Match::Prefix(p) => path.starts_with(p),
            }
    }
}

/// 口の本体（鍵と本文の関数）。
#[derive(Clone, Copy)]
pub(in crate::server) struct Entry {
    pub(in crate::server) key: Key,
    pub(in crate::server) handle: fn(&Request, &Shared) -> Response,
}

/// `Route::ALL` を宣言の順に見て、鍵が要求に当たる最初の口の本文を呼ぶ（当たらなければ None）。
pub(in crate::server) fn dispatch(req: &Request, shared: &Shared) -> Option<Response> {
    let path = req.path();
    Route::ALL
        .iter()
        .map(|route| route.entry())
        .find(|entry| entry.key.matches(&req.method, path))
        .map(|entry| (entry.handle)(req, shared))
}
