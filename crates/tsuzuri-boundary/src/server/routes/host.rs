//! GET /api/host — host の負荷と書き（HostDoc・kernel の file と host の面の書きの測りの表を要求のたびに読む）。

use tsuzuri_contract::{host, wire};

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact(host::PATH),
    },
    handle: host_doc,
};

fn host_doc(_: &Request, shared: &Shared) -> Response {
    json(200, wire::encode(&shared.host.doc()))
}
