//! POST /api/seat/heartbeat — project board の停止の切り替えの受付。
//! 本文は向きだけで、撃つ席は server の --repo の anchor から引く。

use tsuzuri_contract::seathb::PATH;

use crate::accthb;
use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, guarded};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(PATH),
    },
    handle: post_seat_heartbeat,
};

/// 停止の切り替えの受付（守りは `guarded`・本文の読みと anchor の引きは `accthb::accept_own`）。
/// state dir が無ければ器を撃たず 404 no-project。200 の本文は JSON、ほかは字。
fn post_seat_heartbeat(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| Some(t.to_string())) {
        Ok(body) => body,
        Err(response) => return response,
    };
    let Some(acct) = &shared.acct else {
        return Response::text(404, accthb::NO_PROJECT);
    };
    match accthb::accept_own(acct, &shared.sources.ledger.repo, &body) {
        (200, text) => Response::json(200, text),
        (status, text) => Response::text(status, &text),
    }
}
