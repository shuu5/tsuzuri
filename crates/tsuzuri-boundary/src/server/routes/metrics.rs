//! GET /api/metrics — 台帳の指標（LedgerStats）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, board, events, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/metrics"),
    },
    handle: metrics,
};

fn metrics(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let texts = sources.gather(false, false);
    json(200, wire::encode(&board::metrics(&texts, events::now())))
}
