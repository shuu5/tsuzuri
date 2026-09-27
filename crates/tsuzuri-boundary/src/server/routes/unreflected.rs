//! GET /api/unreflected — 未反映の一覧（UnreflectedList・便 e-view）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, board, events, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/unreflected"),
    },
    handle: unreflected,
};

fn unreflected(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let texts = sources.gather(false, false);
    json(
        200,
        wire::encode(&board::unreflected(&texts, events::now())),
    )
}
