//! GET /api/unreflected — 未反映の一覧（UnreflectedList・便 e-view・state dir の局面の出力から読む・行 c-unref-lc）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, board, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/unreflected"),
    },
    handle: unreflected,
};

fn unreflected(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let (texts, stale) = sources.gather_held(false, false);
    let (out, marks) = shared.cases.texts();
    aged(
        json(
            200,
            wire::encode(&board::unreflected(&texts, &out, marks.as_deref())),
        ),
        stale,
    )
}
