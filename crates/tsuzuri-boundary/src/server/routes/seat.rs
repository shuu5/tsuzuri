//! GET /api/seat — 席の card（SeatCard・便 e-seat）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, events, json, seat};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(seat::PATH),
    },
    handle: card,
};

fn card(_: &Request, shared: &Shared) -> Response {
    json(200, wire::encode(&shared.seats.card(events::now())))
}
