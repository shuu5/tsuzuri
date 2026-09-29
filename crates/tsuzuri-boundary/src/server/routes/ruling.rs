//! POST /api/ruling — 裁定の受付（RulingRequest・便 e-ask）。

use tsuzuri_contract::surface::RulingRequest;
use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::ruling::{self, Outcome};
use crate::server::{Shared, events, guarded, json, refusal};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(ruling::PATH),
    },
    handle: post_ruling,
};

/// 裁定の受付（守りは `guarded`）。断りと 4xx と 5xx は、notes への追記の前なら何も書いていない。
/// 200 でなければ `refusal_line` の 1 行を標準エラーに書いてから返す。
fn post_ruling(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<RulingRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    let outcome = ruling::accept(&body, &shared.sources.ledger, &shared.writer, events::now());
    let response = match outcome {
        Outcome::Recorded(response) => json(200, wire::encode(&response)),
        Outcome::Refused(reason) => refusal(reason),
        Outcome::LedgerUnknown => Response::text(503, "ledger-unknown"),
        Outcome::IdShape => Response::text(500, "ruling-id-shape"),
        Outcome::AppendFailed => Response::text(502, "ledger-append"),
        Outcome::CloseFailed(id) => Response::text(502, &format!("ledger-close {id}")),
    };
    if response.status != 200 {
        eprintln!(
            "{}",
            ruling::refusal_line(ruling::PATH, response.status, &response.body, &[&body.question])
        );
    }
    response
}
