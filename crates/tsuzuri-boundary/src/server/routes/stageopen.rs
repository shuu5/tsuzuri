//! POST /api/stage/open — 持ち主の button が表示先の端末の窓を開く頼みの受付（行 e-stage-target・裁定 t3-hub.59.7）。
//! tz stage open を 1 回撃つ（席の印を除いた環境で撃つので、席が撃つ口は頭 Origin の無い要求の断りで閉じる）。

use tsuzuri_contract::stage::OPEN_PATH;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, guarded_strict};
use crate::stagecall;

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(OPEN_PATH),
    },
    handle: post_stage_open,
};

/// 守りは `guarded_strict`（頭 Origin の無い要求も断る）・読むだけの server も受ける。
/// 本文の読みと名の形と project の引きと撃ちは `stagecall::accept_open`。200 の本文は tz の標準出力の字。
fn post_stage_open(req: &Request, shared: &Shared) -> Response {
    let body = match guarded_strict(req, |t| Some(t.to_string())) {
        Ok(body) => body,
        Err(response) => return response,
    };
    let dir = shared.acct.as_ref().map(|acct| acct.host_state_dir());
    let (status, text) = stagecall::accept_open(&shared.stage, dir, &body);
    Response::text(status, &text)
}
