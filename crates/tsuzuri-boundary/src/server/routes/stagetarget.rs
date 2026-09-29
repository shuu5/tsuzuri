//! GET /api/stage/target — 表示先の設定の読み（行 e-stage-target）。
//! tz の口 stage target show --json を 1 回撃った 1 行を返す（守りの `guarded` は通らず、頭 Origin の無い GET も通す）。

use tsuzuri_contract::stage::PATH;

use crate::server::Shared;
use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::stagecall;

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(PATH),
    },
    handle: get_stage_target,
};

/// 200 は電文の JSON、tz が撃てないか電文でない字を出せば 502（字 tz-failed か bad-reply）。
fn get_stage_target(_req: &Request, shared: &Shared) -> Response {
    match stagecall::read(&shared.stage) {
        (200, text) => Response::json(200, text),
        (status, text) => Response::text(status, &text),
    }
}
