//! GET /api/graph — 導出グラフ（GraphDoc）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, board, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/graph"),
    },
    handle: graph,
};

fn graph(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let texts = sources.gather(true, true);
    json(200, wire::encode(&board::graph(&texts)))
}
