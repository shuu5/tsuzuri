//! GET /api/questions — 問いの一覧（QuestionList）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/questions"),
    },
    handle: questions,
};

fn questions(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let text = sources.ledger.text().unwrap_or_default();
    json(200, wire::encode(&tsuzuri_core::question::list(&text)))
}
