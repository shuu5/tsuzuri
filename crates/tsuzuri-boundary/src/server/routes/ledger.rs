//! GET /api/ledger — 台帳の一覧（LedgerList）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, json, ledger};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/ledger"),
    },
    handle: list,
};

fn list(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let got = sources.ledger.got();
    aged(json(200, wire::encode(&ledger::list(&got))), got.stale)
}
