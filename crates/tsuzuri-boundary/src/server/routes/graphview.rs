//! GET /api/graph/view?open= — 地図のグラフの眺め（GraphView・便 e-view・開く列は行 c-graph-fold）。

use tsuzuri_contract::wire;
use tsuzuri_core::graph::fold::open_list;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, board, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/graph/view"),
    },
    handle: view,
};

/// 眺め（query の open を字 , で分けた列を開く・無ければ空の列）。
fn view(req: &Request, shared: &Shared) -> Response {
    let open = req.query("open").map(|s| open_list(&s)).unwrap_or_default();
    let sources = &shared.sources;
    let (texts, stale) = sources.gather_held(true, true);
    aged(
        json(200, wire::encode(&board::view_open(&texts, &open))),
        stale,
    )
}
