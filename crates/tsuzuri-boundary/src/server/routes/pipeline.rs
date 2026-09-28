//! GET /api/pipeline — pipeline の板（PipelineBoard）。
//! 最初の要求が器の doctor の台帳の形の行の撃ちを許し（`Form::arm`）、口は持った字を読むだけで器を撃たない。
//! 形の崩れの一覧は、見張りの読みの後の撃ちを終えた字から写す（行 c-pipe-misfit）。

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, aged, board, events, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/pipeline"),
    },
    handle: pipeline,
};

fn pipeline(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    let (texts, stale) = sources.gather_held(false, true);
    let doctor = sources.ledger.form().and_then(|form| {
        form.arm();
        form.text()
    });
    aged(
        json(
            200,
            wire::encode(&board::pipeline(&texts, doctor.as_deref(), events::now())),
        ),
        stale,
    )
}
