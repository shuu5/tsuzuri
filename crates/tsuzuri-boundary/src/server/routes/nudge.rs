//! POST /api/surface/questions — 問いの合図の受付（QuestionNudge・行 e-signal）。
//! 席が問いを bdw で置いた後に送り、台帳の見張りの周期の待ちを終わらせる（台帳にも器にも書かない）。

use tsuzuri_contract::surface::QuestionNudge;
use tsuzuri_contract::wire;

use crate::server::events::{NUDGE_PATH, NUDGED};
use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, guarded};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "POST",
        path: Match::Exact(NUDGE_PATH),
    },
    handle: post_nudge,
};

/// 問いの合図の受付（守りは `guarded`・読むだけの server も受ける）。読めれば見張りを起こして 202 と `NUDGED`。
/// 台帳に在る id かは見ない（見るには見張りの読みの終わりを待つ・在らない id は印の動かない周なら何もしない）。
fn post_nudge(req: &Request, shared: &Shared) -> Response {
    if let Err(response) = guarded(req, |t| wire::decode::<QuestionNudge>(t).ok()) {
        return response;
    }
    shared.hub.nudge();
    Response::text(202, NUDGED)
}
