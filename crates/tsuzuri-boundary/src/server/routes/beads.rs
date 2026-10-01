//! GET /api/beads — bead の事実の一覧（BeadFacts・行 c-bead-route）。
//! 台帳の字は口 /api/ledger と同じ読み（`Source::got`）から中核の `ledger::facts` で組み、古さの印も同じに付ける。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/beads"),
    },
    handle: beads,
};

fn beads(_: &Request, shared: &Shared) -> Response {
    let got = shared.sources.ledger.got();
    let facts = tsuzuri_core::ledger::facts(got.text.as_deref().unwrap_or_default());
    aged(json(200, wire::encode(&facts)), got.stale)
}
