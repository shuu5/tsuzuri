//! GET /api/consult/unreceived — 席に受けられていない所見と頼み（Reading<ConsultUnreceived>・行 cs-server）。
//! 席の hook の安い判じが撃つ（置き場か台帳が読めなければ Unknown・読むだけの server も同じ）。

use tsuzuri_contract::consult::UNRECEIVED_PATH;
use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, consult, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(UNRECEIVED_PATH),
    },
    handle: unreceived,
};

fn unreceived(_: &Request, shared: &Shared) -> Response {
    let got = shared.sources.ledger.got();
    let reading = consult::waiting(shared.consult.ctx().as_ref(), got.text.as_deref());
    aged(json(200, wire::encode(&reading)), got.stale)
}
