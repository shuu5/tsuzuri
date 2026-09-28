//! GET /api/pipeline — pipeline の板（PipelineBoard）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, board, events, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/pipeline"),
    },
    handle: pipeline,
};

fn pipeline(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let (texts, stale) = sources.gather_held(false, true);
    aged(
        json(200, wire::encode(&board::pipeline(&texts, events::now()))),
        stale,
    )
}
