//! GET /api/unreceived — 席に届いていない裁定の id の列（Reading<Vec<RulingId>>）。
//! 印の無い裁定の id を台帳の順に Known で、台帳が読めなければ Unknown で返す（読むだけの server も同じ）。

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::undelivered;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/unreceived"),
    },
    handle: unreceived,
};

fn unreceived(_: &Request, shared: &Shared) -> Response {
    let got = shared.sources.ledger.got();
    let reading = match got.text.as_deref().map(undelivered) {
        Some(Reading::Known(pending)) => {
            Reading::Known(pending.into_iter().map(|p| p.ruling).collect::<Vec<_>>())
        }
        _ => Reading::Unknown,
    };
    aged(json(200, wire::encode(&reading)), got.stale)
}
