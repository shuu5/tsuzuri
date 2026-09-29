//! POST /api/stage/targets — account board の表示先の書きの受付（一括と project ごと・行 e-stage-target）。
//! 群の宣言は account board の読みの state dir から引き、無ければ --repo の project だけを受ける。

use tsuzuri_contract::stage::ALL_PATH;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, guarded_strict};
use crate::stagecall;

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(ALL_PATH),
    },
    handle: post_stage_targets,
};

/// 守りは `guarded_strict`（頭 Origin の無い要求も断る）・読むだけの server も受ける。
/// 本文の読みと名の形と project の引きと撃ちは `stagecall::accept_all`。200 の本文は tz の標準出力の字。
fn post_stage_targets(req: &Request, shared: &Shared) -> Response {
    let body = match guarded_strict(req, |t| Some(t.to_string())) {
        Ok(body) => body,
        Err(response) => return response,
    };
    let dir = shared.acct.as_ref().map(|acct| acct.host_state_dir());
    let (status, text) = stagecall::accept_all(&shared.stage, dir, &body);
    Response::text(status, &text)
}
