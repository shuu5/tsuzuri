//! GET /api/metrics — 台帳の指標（LedgerStats・未反映の 3 欄は state dir の局面の出力から読む・行 c-unref-lc）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, board, events, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/metrics"),
    },
    handle: metrics,
};

fn metrics(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let (texts, stale) = sources.gather_held(false, false);
    let (out, marks) = shared.cases.texts();
    aged(
        json(
            200,
            wire::encode(&board::metrics(
                &texts,
                &out,
                marks.as_deref(),
                events::now(),
            )),
        ),
        stale,
    )
}
