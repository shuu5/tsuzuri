//! POST /api/ruling — 裁定の受付（RulingRequest）。

use tsuzuri_contract::surface::RulingRequest;
use tsuzuri_contract::wire;

use crate::out::emit_err;
use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::ruling::{self, Outcome};
use crate::server::{Shared, events, guarded, json, read_only, refusal};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(ruling::PATH),
    },
    handle: post_ruling,
};

/// 裁定の受付（守りは `guarded`・読むだけの server は受付の前に `read_only` で断る）。
/// 断りと 4xx と 5xx は、器の答えの口を撃つ前なら何も書いていない。
/// 200 でなければ `refusal_line` の 1 行を標準エラーに書いてから返す。
fn post_ruling(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<RulingRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    if shared.read_only {
        return read_only(ruling::PATH, &[&body.question]);
    }
    let outcome = ruling::accept(
        &body,
        &shared.sources.ledger,
        &shared.writer,
        (shared.scribe2.as_os_str(), shared.state_dir.as_deref()),
        events::now(),
    );
    let response = match outcome {
        Outcome::Recorded(response) => json(200, wire::encode(&response)),
        Outcome::Refused(reason) => refusal(reason),
        Outcome::LedgerUnknown => Response::text(503, "ledger-unknown"),
        Outcome::NoStateDir => Response::text(503, "ruling-state-dir"),
        Outcome::IdShape => Response::text(500, "ruling-id-shape"),
        Outcome::AnswerFailed => Response::text(502, "ruling-answer"),
    };
    if response.status != 200 {
        emit_err(&ruling::refusal_line(
            ruling::PATH,
            response.status,
            &response.body,
            &[&body.question],
        ));
    }
    response
}
