//! POST /api/consult/request — board の button の相談の頼み（ConsultRequest・行 cs-server）。
//! 守りは方針の口と同じ `guarded`・読むだけの server は受付の前に `read_only` で断る。
//! 置き場に相談の頼みの行を 1 行足すだけで、窓を開かず、裁定の配達を撃たない。応答は頼みの id の電文。

use tsuzuri_contract::consult::{ConsultRequest, REQUEST_PATH};
use tsuzuri_contract::wire;

use crate::out::emit_err;
use crate::server::consult::{Outcome, request};
use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::ruling::refusal_line;
use crate::server::{Shared, events, guarded, json, read_only};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(REQUEST_PATH),
    },
    handle: post_request,
};

/// 頼みの受付（200 でなければ `refusal_line` の 1 行を標準エラーに書いてから返す）。
fn post_request(req: &Request, shared: &Shared) -> Response {
    let body = match guarded(req, |t| wire::decode::<ConsultRequest>(t).ok()) {
        Ok(body) => body,
        Err(response) => return response,
    };
    if shared.read_only {
        return read_only(REQUEST_PATH, &[]);
    }
    let response = match request(&body, &shared.sources.ledger, &shared.writer, events::now()) {
        Outcome::Recorded(id, _) => json(200, wire::encode(&id)),
        Outcome::LedgerUnknown => Response::text(503, "ledger-unknown"),
        Outcome::BadLine => Response::text(400, "bad-line"),
        Outcome::NoRoot => Response::text(503, "no-root"),
        Outcome::AppendFailed(id) => Response::text(502, &format!("ledger-append {id}")),
    };
    if response.status != 200 {
        emit_err(&refusal_line(
            REQUEST_PATH,
            response.status,
            &response.body,
            &[],
        ));
    }
    response
}
