//! POST /api/batch — 束の受付（BatchRequest・便 e-batch）。

use tsuzuri_contract::surface::BatchRequest;
use tsuzuri_contract::wire;

use crate::out::emit_err;
use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::ruling::refusal_line;
use crate::server::{Shared, batch, events, guarded, json, read_only, refusal};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(batch::PATH),
    },
    handle: post_batch,
};

/// 束の受付（守りは `guarded`・読むだけの server は受付の前に `read_only` で断る）。断りと 4xx と 5xx は何も書いていない。
/// 502 の本文は、要求の全部の行の結果を要求の順に持つ BatchResponse
/// （書いた・閉じていない・書いていない）。
/// 200 でなければ要求の全部の行の問いの id を並べた `refusal_line` の 1 行を標準エラーに書いてから返す。
fn post_batch(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<BatchRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    if shared.read_only {
        let questions: Vec<_> = body.items.iter().map(|i| &i.question).collect();
        return read_only(batch::PATH, &questions);
    }
    let response = match batch::accept(&body, &shared.sources.ledger, &shared.writer, events::now())
    {
        batch::Outcome::Recorded(response) => json(200, wire::encode(&response)),
        batch::Outcome::Refused(reason) => refusal(reason),
        batch::Outcome::Duplicate => Response::text(400, "duplicate"),
        batch::Outcome::LedgerUnknown => Response::text(503, "ledger-unknown"),
        batch::Outcome::IdShape => Response::text(500, "ruling-id-shape"),
        batch::Outcome::WriteFailed(response) => json(502, wire::encode(&response)),
    };
    if response.status != 200 {
        let questions: Vec<_> = body.items.iter().map(|i| &i.question).collect();
        emit_err(&refusal_line(
            batch::PATH,
            response.status,
            &response.body,
            &questions,
        ));
    }
    response
}
