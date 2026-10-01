//! GET /api/pipeline — pipeline の板（PipelineBoard）。
//! 最初の要求が器の doctor の台帳の形の行の撃ちを許し（`Form::arm`・知らせの接続も許す）、口は持った組を読むだけで器を撃たない。
//! 札は集めた今の台帳の字から組み、形の崩れの一覧は見張りの読みの周の台帳の字と撃ちを終えた台帳の形の行の組から写す
//! （行 c-pipe-misfit・行 c-misfit-pair）。
//! 札の段は、state dir の器の局面の出力を要求のたびに読んで中核の `board_with_cases` が決める（`cases_of`・行 c-case-columns）。

use tsuzuri_contract::wire;
use tsuzuri_core::case::cases_of;

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
    let kept = sources.ledger.form().and_then(|form| {
        form.arm();
        form.kept()
    });
    let (out, marks) = shared.cases.texts();
    let cases = cases_of(&out, marks.as_deref());
    aged(
        json(
            200,
            wire::encode(&board::pipeline(
                &texts,
                kept.as_ref(),
                &cases,
                events::now(),
            )),
        ),
        stale,
    )
}
