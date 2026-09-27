//! GET /api/next — 次の一手（NextStep）。席の card が読めるときは席の card も受けて判じる。

use std::thread;

use tsuzuri_contract::wire;

use crate::server::http::{Request, Response};
use crate::server::route::{Entry, Key, Match};
use crate::server::{Shared, board, events, json};

pub(in crate::server) const ROUTE: Entry = Entry {
    key: Key {
        method: "GET",
        path: Match::Exact("/api/next"),
    },
    handle: next,
};

fn next(_: &Request, shared: &Shared) -> Response {
    let sources = &shared.sources;
    // 席の card と台帳の字は並べて集める（待ちは 1 本分の上限まで）。
    let now = events::now();
    let (texts, card) = thread::scope(|s| {
        let card = s.spawn(|| shared.seats.known(now));
        let texts = sources.gather(false, true);
        (texts, card.join().ok().flatten())
    });
    let step = match &card {
        Some(card) => board::next_seat(&texts, card, now),
        None => board::next(&texts, now),
    };
    json(200, wire::encode(&step))
}
