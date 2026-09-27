//! GET /api/ledger/<id> — 台帳の 1 本（LedgerItem）。
//! id の形が読めなければ 400 id-shape・無ければ 404 no-item・台帳が読めなければ 503 ledger-unknown。

use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, json, ledger};

/// 口の path の頭。
const PREFIX: &str = "/api/ledger/";

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Prefix(PREFIX),
    },
    handle: item,
};

fn item(req: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let id = req.path().strip_prefix(PREFIX).unwrap_or_default();
    let Ok(id) = BeadId::new(id) else {
        return Response::text(400, "id-shape");
    };
    match ledger::item(&sources.ledger, &id) {
        ledger::Lookup::Found(item) => json(200, wire::encode(&item)),
        ledger::Lookup::Missing => Response::text(404, "no-item"),
        ledger::Lookup::Unknown => Response::text(503, "ledger-unknown"),
    }
}
