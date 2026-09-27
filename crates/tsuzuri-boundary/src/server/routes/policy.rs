//! POST /api/policy — 方針の受付（PolicyRequest・便 e-batch）。

use tsuzuri_contract::surface::PolicyRequest;
use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, events, guarded, json, policy, refusal};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(policy::PATH),
    },
    handle: post_policy,
};

/// 方針の受付（守りは `guarded`）。断りと 4xx と 5xx は何も書いていない。
fn post_policy(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<PolicyRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    match policy::accept(&body, &shared.sources.ledger, &shared.writer, events::now()) {
        policy::Outcome::Recorded(response) => json(200, wire::encode(&response)),
        policy::Outcome::Refused(reason) => refusal(reason),
        policy::Outcome::LedgerUnknown => Response::text(503, "ledger-unknown"),
        policy::Outcome::BadScope => Response::text(400, "scope"),
        policy::Outcome::NoMemo => Response::text(503, "no-policy-memo"),
        policy::Outcome::IdShape => Response::text(500, "policy-id-shape"),
        policy::Outcome::AppendFailed => Response::text(502, "ledger-append"),
    }
}
