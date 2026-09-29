//! POST /api/stage/target — project board の表示先の書きの受付（行 e-stage-target・裁定 t3-hub.52.29）。
//! 本文は端末の名か null（上書きを外す）だけで、書く project は server の --repo から引く。

use tsuzuri_contract::stage::PATH;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, guarded_strict};
use crate::stagecall;

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(PATH),
    },
    handle: post_stage_target,
};

/// 守りは `guarded_strict`（頭 Origin の無い要求も断る）・読むだけの server も受ける。
/// 本文の読みと名の形と撃ちは `stagecall::accept_own`。200 の本文は tz の標準出力の字。
fn post_stage_target(req: &Request, shared: &Shared) -> Response {
    let body = match guarded_strict(req, |t| Some(t.to_string())) {
        Ok(body) => body,
        Err(response) => return response,
    };
    let (status, text) = stagecall::accept_own(&shared.stage, &body);
    Response::text(status, &text)
}
