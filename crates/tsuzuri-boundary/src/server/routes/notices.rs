//! GET /api/notices — 席の「見て」の知らせの project ごとの最新の 1 つ（Notices・要件 FR16）。
//! 材料は tz stage notify が書く記録の file だけで、ここは読むだけ（file は書かず、子 process を撃たない）。
//! 記録の dir は `Config::notify`（無ければ 503 no-notify-dir）、dir が読めなければ 503 notify-unread。
//! この board の project の名は記録の書き手と同じ `cli::project` で --repo から引く（引けなければ空の字）。

use tsuzuri_contract::notice::{Notice, Notices, PATH};
use tsuzuri_contract::wire;

use crate::server::events::now;
use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, json};
use crate::stage::{cli, notify};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(PATH),
    },
    handle: notices,
};

/// 記録の dir が決まらない応答の本文。
pub const NO_DIR: &str = "no-notify-dir";

/// 記録の dir が読めない応答の本文。
pub const UNREAD: &str = "notify-unread";

fn notices(_: &Request, shared: &Shared) -> Response {
    let Some(dir) = shared.notify.as_deref() else {
        return Response::text(503, NO_DIR);
    };
    let Ok((records, unread)) = notify::gather(dir) else {
        return Response::text(503, UNREAD);
    };
    let latest = records
        .into_iter()
        .map(|r| Notice {
            at: r.at,
            project: r.project,
            title: r.title,
            url: r.url,
        })
        .collect();
    let project = cli::project(&shared.sources.ledger.repo).unwrap_or_default();
    json(
        200,
        wire::encode(&Notices {
            at: now(),
            project,
            latest,
            unread,
        }),
    )
}
