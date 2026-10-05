//! POST /api/account/heartbeat — 停止の切り替えの受付。

use tsuzuri_contract::account::HEARTBEAT_PATH;

use crate::accthb;
use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, guarded};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(HEARTBEAT_PATH),
    },
    handle: post_heartbeat,
};

/// 停止の切り替えの受付（守りは `guarded`・本文の読みは `accthb::accept`）。
/// state dir が無ければ器を撃たず 404 no-project。200 の本文は JSON、ほかは字。
fn post_heartbeat(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| Some(t.to_string())) {
        Ok(body) => body,
        Err(response) => return response,
    };
    let Some(acct) = &shared.acct else {
        return Response::text(404, accthb::NO_PROJECT);
    };
    match accthb::accept(acct, &body) {
        (200, text) => Response::json(200, text),
        (status, text) => Response::text(status, &text),
    }
}
