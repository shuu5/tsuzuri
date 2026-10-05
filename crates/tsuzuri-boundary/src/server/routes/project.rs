//! GET /api/project — board の project の名（ProjectName）。
//! 名は --repo の path を canonicalize した path の最後の名（できなければ字のままの path の最後の名）。

use tsuzuri_contract::project::{PATH, ProjectName};
use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(PATH),
    },
    handle: project,
};

fn project(_: &Request, shared: &Shared) -> Response {
    let repo = &shared.sources.ledger.repo;
    let path = repo.canonicalize().unwrap_or_else(|_| repo.clone());
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => json(
            200,
            wire::encode(&ProjectName {
                name: name.to_string(),
            }),
        ),
        None => Response::text(503, "no-name"),
    }
}
