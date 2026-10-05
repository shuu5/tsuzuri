//! GET /api/around?id=&k=&fold= — 節点の近傍（AroundDoc）。

use tsuzuri_contract::graph::Fold;
use tsuzuri_contract::wire;
use tsuzuri_core::graph::around::{AROUND_STEPS, AROUND_STEPS_RANGE};

use crate::server::board::{self, Sources};
use crate::server::http::{Request, Response};
use crate::server::{aged, json};
use crate::server::route::{Entry, Key, Match};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/around"),
    },
    handle: |req, shared| around(req, &shared.sources),
};

/// 節点の近傍（query は id・k・fold の順に読み、最初に当たった断りを返す）。
/// id が無いか空は 400 no-id・k が 1 から 3 の整数でなければ 400 steps・fold が 4 つの字のどれでもなければ 400 fold・
/// 節点が無ければ 404 no-node。字を集めるのは query が読めた後だけ。
fn around(req: &Request, sources: &Sources) -> Response {
    let Some(id) = req.query("id").filter(|id| !id.is_empty()) else {
        return Response::text(400, "no-id");
    };
    let steps = match req.query("k") {
        None => AROUND_STEPS,
        Some(k) => match k.parse::<u8>() {
            Ok(k) if AROUND_STEPS_RANGE.contains(&k) => k,
            _ => return Response::text(400, "steps"),
        },
    };
    let fold = match req.query("fold").as_deref() {
        None | Some("none") => Fold::None,
        Some("up") => Fold::Up,
        Some("down") => Fold::Down,
        Some("both") => Fold::Both,
        Some(_) => return Response::text(400, "fold"),
    };
    let (texts, stale) = sources.gather_held(true, true);
    let response = match board::around(&texts, &id, steps, fold) {
        Some(doc) => json(200, wire::encode(&doc)),
        None => Response::text(404, "no-node"),
    };
    aged(response, stale)
}
