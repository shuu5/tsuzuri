//! GET /api/consult — 相談の窓の一覧（ConsultBoard・行 cs-server）。
//! 作業場の file と台帳の見張りの最後の読みの相談の行から tz consult list と同じ組みの電文を返す
//! （置き場か台帳が読めなければ 4 つの段が Unknown・読むだけの server も同じ）。

use tsuzuri_contract::consult::PATH;
use tsuzuri_contract::wire;

use crate::consult::minute_now;
use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, consult, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(PATH),
    },
    handle: board,
};

fn board(_: &Request, shared: &Shared) -> Response {
    let got = shared.sources.ledger.got();
    let board = consult::board(
        shared.consult.ctx().as_ref(),
        got.text.as_deref(),
        &minute_now(),
    );
    aged(json(200, wire::encode(&board)), got.stale)
}
