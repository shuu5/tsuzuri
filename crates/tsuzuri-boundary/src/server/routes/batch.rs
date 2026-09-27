//! POST /api/batch — 束の受付（BatchRequest・便 e-batch）。

use tsuzuri_contract::surface::BatchRequest;
use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, batch, events, guarded, json, refusal};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(batch::PATH),
    },
    handle: post_batch,
};

/// 束の受付（守りは `guarded`）。断りと 4xx と 5xx は何も書いていない。
/// 502 の本文は、2 回とも書き終えた行だけを書いたとして持つ BatchResponse。
fn post_batch(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<BatchRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    match batch::accept(&body, &shared.sources.ledger, &shared.writer, events::now()) {
        batch::Outcome::Recorded(response) => json(200, wire::encode(&response)),
        batch::Outcome::Refused(reason) => refusal(reason),
        batch::Outcome::Duplicate => Response::text(400, "duplicate"),
        batch::Outcome::LedgerUnknown => Response::text(503, "ledger-unknown"),
        batch::Outcome::IdShape => Response::text(500, "ruling-id-shape"),
        batch::Outcome::WriteFailed(response) => json(502, wire::encode(&response)),
    }
}
