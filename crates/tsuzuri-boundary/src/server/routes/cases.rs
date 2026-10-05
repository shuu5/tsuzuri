//! GET /api/cases — 器の局面の出力（CaseDoc）。
//! 字は state dir の fleet/lifecycle.json と lifecycle.stale だけから読む（台帳の bd も器の CLI も撃たない）。

use tsuzuri_contract::case::PATH;
use tsuzuri_contract::wire;
use tsuzuri_core::case::cases_of;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(PATH),
    },
    handle: cases,
};

/// 出力か古さの印が無いか読めなければ parts は Unknown（中核の `cases_of`）。
fn cases(_req: &Request, shared: &Shared) -> Response {
    let (out, stale) = shared.cases.texts();
    json(200, wire::encode(&cases_of(&out, stale.as_deref())))
}
