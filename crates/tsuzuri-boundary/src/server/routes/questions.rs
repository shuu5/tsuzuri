//! GET /api/questions — 問いの一覧（QuestionList）。読むだけの server は answerable を偽にする（行 e-ask-own-only）。

use tsuzuri_contract::question::QuestionList;
use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/questions"),
    },
    handle: questions,
};

fn questions(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let got = sources.ledger.got();
    let text = got.text.unwrap_or_default();
    let list = QuestionList {
        answerable: !shared.read_only,
        ..tsuzuri_core::question::list(&text)
    };
    aged(json(200, wire::encode(&list)), got.stale)
}
