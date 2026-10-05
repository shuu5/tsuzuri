//! POST /api/policy — 方針の受付（PolicyRequest）。

use tsuzuri_contract::surface::PolicyRequest;
use tsuzuri_contract::wire;

use crate::out::emit_err;
use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::ruling::refusal_line;
use crate::server::{Shared, events, guarded, json, policy, read_only, refusal};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(policy::PATH),
    },
    handle: post_policy,
};

/// 方針の受付（守りは `guarded`・読むだけの server は受付の前に `read_only` で断る）。断りと 4xx と 503 は何も書いていない。
/// 502 と 500 の本文は作った問いの id か方針の id を名指す（ledger-create は作れたかが分からない）。
/// 200 でなければ問いの id を持たない `refusal_line` の 1 行を標準エラーに書いてから返す（行 c-ruling-reread）。
fn post_policy(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<PolicyRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    if shared.read_only {
        return read_only(policy::PATH, &[]);
    }
    let outcome = policy::accept(&body, &shared.sources.ledger, &shared.writer, events::now());
    let response = match outcome {
        policy::Outcome::Recorded(response) => json(200, wire::encode(&response)),
        policy::Outcome::Refused(reason) => refusal(reason),
        policy::Outcome::LedgerUnknown => Response::text(503, "ledger-unknown"),
        policy::Outcome::BadScope => Response::text(400, "scope"),
        policy::Outcome::NoRoot => Response::text(503, "no-root"),
        policy::Outcome::CreateFailed => Response::text(502, "ledger-create"),
        policy::Outcome::IdShape(q) => Response::text(500, &format!("policy-id-shape {q}")),
        policy::Outcome::AppendFailed(q) => Response::text(502, &format!("ledger-append {q}")),
        policy::Outcome::CloseFailed(id) => Response::text(502, &format!("ledger-close {id}")),
    };
    if response.status != 200 {
        emit_err(&refusal_line(
            policy::PATH,
            response.status,
            &response.body,
            &[],
        ));
    }
    response
}
