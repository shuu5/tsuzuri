//! GET /api/runs?bead= — bead の走行の時間軸（RunsDoc・行 e-runs）。
//! 字は器の event log だけから読む（台帳の bd は撃たない）。

use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::runs::PATH;
use tsuzuri_contract::wire;
use tsuzuri_core::pipeline::runs_of;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(PATH),
    },
    handle: runs,
};

/// bead が無いか空は 400 no-bead・bead の id の形が悪ければ 400 id-shape。
/// event log が読めなければ空の字を渡す（runs は Unknown）。
fn runs(req: &Request, shared: &Shared) -> Response {
    let Some(bead) = req.query("bead").filter(|b| !b.is_empty()) else {
        return Response::text(400, "no-bead");
    };
    let Ok(bead) = BeadId::new(bead) else {
        return Response::text(400, "id-shape");
    };
    let events = shared.sources.runs.text().unwrap_or_default();
    json(200, wire::encode(&runs_of(&events, &bead)))
}
