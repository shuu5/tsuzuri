//! GET /api/graph — 導出グラフ（GraphDoc）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, board, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/graph"),
    },
    handle: graph,
};

fn graph(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let (texts, stale) = sources.gather_held(true, true);
    let doc = board::graph_outside(&texts, shared.others.outside());
    aged(json(200, wire::encode(&doc)), stale)
}
