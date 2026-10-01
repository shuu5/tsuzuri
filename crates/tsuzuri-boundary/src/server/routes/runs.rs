//! GET /api/runs?bead= — bead の走行の時間軸（RunsDoc・行 e-runs）。
//! 字は器の event log と便の dir の判定の file だけから読む（台帳の bd は撃たない・行 c-run-verdict）。

use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::runs::PATH;
use tsuzuri_contract::wire;
use tsuzuri_core::pipeline::{runs_of, with_verdicts};

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::runs::{GATE_FILE, REVIEW_FILE};
use crate::server::{Shared, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(PATH),
    },
    handle: runs,
};

/// bead が無いか空は 400 no-bead・bead の id の形が悪ければ 400 id-shape。
/// event log が読めなければ空の字を渡す（runs は Unknown）。走行ごとの判定は便の dir の file の字（読めなければ Unknown）。
fn runs(req: &Request, shared: &Shared) -> Response {
    let Some(bead) = req.query("bead").filter(|b| !b.is_empty()) else {
        return Response::text(400, "no-bead");
    };
    let Ok(bead) = BeadId::new(bead) else {
        return Response::text(400, "id-shape");
    };
    let runs = &shared.sources.runs;
    let events = runs.text().unwrap_or_default();
    let doc = with_verdicts(
        runs_of(&events, &bead),
        |run| runs.run_file(run, GATE_FILE),
        |run| runs.run_file(run, REVIEW_FILE),
    );
    json(200, wire::encode(&doc))
}
