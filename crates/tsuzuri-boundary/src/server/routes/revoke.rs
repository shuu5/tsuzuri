//! POST /api/revoke — 答えた決定の取り消しの受付（RevokeRequest・行 e-revoke）。

use tsuzuri_contract::surface::{REVOKE_PATH, RevokeRequest};
use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::ruling::{self, Revoked};
use crate::server::{Shared, events, guarded, json, refusal};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(REVOKE_PATH),
    },
    handle: post_revoke,
};

/// 取り消しの受付（守りは `guarded`）。断りと 4xx と 503 は何も書いていない。
/// 200 でなければ `refusal_line` の 1 行を標準エラーに書いてから返す。
fn post_revoke(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<RevokeRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    let outcome = ruling::revoke(&body, &shared.sources.ledger, &shared.writer, events::now());
    let response = match outcome {
        Revoked::Recorded(response) => json(200, wire::encode(&response)),
        Revoked::Refused(reason) => refusal(reason),
        Revoked::LedgerUnknown => Response::text(503, "ledger-unknown"),
        Revoked::IdShape => Response::text(500, "ruling-id-shape"),
        Revoked::AppendFailed => Response::text(502, "ledger-append"),
        Revoked::ReopenFailed(id) => Response::text(502, &format!("ledger-reopen {id}")),
    };
    if response.status != 200 {
        eprintln!(
            "{}",
            ruling::refusal_line(REVOKE_PATH, response.status, &response.body, &[&body.question])
        );
    }
    response
}
