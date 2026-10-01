//! 行 c-abs-seat の歯（面・接頭辞 abss_・要件 NFR2）: 席の card と account board の電文の時点は材料の時刻で、
//! 面は描く今で at と最後の区間の終わりを伸ばして描く（今が at より前なら at のまま）。猶予の残り秒は終わる時刻から引く。
//! DOM は wasm の target のときだけなので、今の渡し方は src の file の字で見る。
//! fixture: tests/fixtures/surface/seat-card.json と tests/fixtures/account/acct-doc.json（読むだけ）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::account::{AccountDoc, DormantSeat};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::cards::orch_card;
use tsuzuri_surface::account::session::{Move, grace_left, row};
use tsuzuri_surface::account::{self, dormant_card};
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::seat::{MOVING, Span, WIDTH, band, content, drawn, seat, strip};
use tsuzuri_surface::seatpill;
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::label;

/// 電文の at（材料の時刻）と、描く今。
const AT: u64 = 1_790_510_400;
const LATER: u64 = 1_790_511_000;
const EARLIER: u64 = 1_790_510_300;

/// 翌日の同じ時刻（日本の日が違う）。
const NEXT_DAY: u64 = 1_790_596_800;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn run_card() -> SeatCard {
    let mut sets: BTreeMap<String, SeatCard> =
        wire::decode(&read("../../tests/fixtures/surface/seat-card.json"))
            .expect("fixture の組が電文として読める");
    sets.remove("run").expect("組 run")
}

fn acct_doc() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

fn body(card: &SeatCard) -> Fetched {
    Fetched::Body(wire::encode(card).expect("電文"))
}

/// (6) 面の project の seat: 描く card は at と最後の区間の終わりを今まで伸ばし、今が at より前なら変えない。
#[test]
fn abss_seat_drawn_at_now() {
    let card = run_card();
    assert_eq!(card.at, AT);
    let d = drawn(card.clone(), LATER);
    assert_eq!(d.at, LATER);
    let (Reading::Known(before), Reading::Known(after)) = (&card.spans, &d.spans) else {
        panic!("組 run の区間が Unknown");
    };
    assert_eq!(after.last().map(|s| s.to), Some(LATER));
    assert_eq!(&after[..after.len() - 1], &before[..before.len() - 1]);
    assert_eq!(drawn(card.clone(), EARLIER), card);

    // block の中身は描いた card の電文の本文から組む。
    let f = body(&card);
    assert_eq!(content(&f, LATER), Body::Filled(seat(&drawn(card.clone(), LATER))));
    // 3h の窓の最後の矩形は右端まで（幅の和は WIDTH）。
    let Reading::Known(rects) = strip(&d, Span::H3).rects else {
        panic!("矩形が Unknown");
    };
    let last = rects.last().expect("矩形");
    assert_eq!(last.x + last.width, WIDTH);

    // 猶予の内の残り秒は終わる時刻から描いた card の at を引く。
    let mut g = card.clone();
    g.move_to = Reading::Known(Some("acct-5".to_string()));
    g.grace_until = Reading::Known(Some(1_790_511_501));
    let b = band(&drawn(g.clone(), 1_790_510_460)).expect("猶予の内の帯");
    assert!(b.l1.contains(&format!("{MOVING} 1041 秒")), "{:?}", b.l1);

    // 席の pill の card の時刻の月日は描く今の日で決める。
    let day = |now: u64| seatpill::card(&body(&card), now).kind;
    assert!(!day(AT).contains("09-27"), "{}", day(AT));
    assert!(day(NEXT_DAY).contains("09-27"), "{}", day(NEXT_DAY));
}

/// (7) 面の account: 描く電文は at と席の card の at と最後の区間の終わりを今まで伸ばし、退避の残り秒は終わる時刻から引く。
#[test]
fn abss_acct_drawn_at_now() {
    let doc = acct_doc();
    assert_eq!(doc.at, AT);
    let d = account::drawn(doc.clone(), LATER);
    assert_eq!(d.at, LATER);
    let mut cards = 0;
    for p in &d.projects {
        if let Reading::Known(c) = &p.seat {
            cards += 1;
            assert_eq!(c.at, LATER, "{}", p.name);
            if let Reading::Known(spans) = &c.spans {
                assert_eq!(spans.last().map(|s| s.to), if spans.is_empty() { None } else { Some(LATER) });
            }
        }
    }
    assert!(cards > 0);
    let mut lines = 0;
    for s in &d.sessions {
        if let Reading::Known(spans) = &s.spans
            && let Some(last) = spans.last()
        {
            lines += 1;
            assert_eq!(last.to, LATER, "{}", s.name);
        }
    }
    assert!(lines > 0);
    assert_eq!(account::drawn(doc.clone(), EARLIER), doc);
    grace_and_dormant(d);
}

/// 退避の残り秒は終わる時刻から引き、休止中の card の月日は描く今の日で決める。
fn grace_and_dormant(d: AccountDoc) {
    let b = d.projects.iter().find(|p| p.name == "proj-b").expect("proj-b");
    assert_eq!(b.move_until, Some(1_790_511_501));
    let card = orch_card(&d, b).expect("proj-b は席を持つ");
    assert!(
        card.more.contains(&format!("{} 501 秒", label("move_grace"))),
        "{:?}",
        card.more
    );
    assert_eq!(row(&d, 2).moving, Some(Move::Grace(1_790_511_501)));
    assert_eq!(grace_left(1_790_511_501, d.at, 1_790_511_060), 441);

    // 休止中の card の時刻の月日は描く今の日で決める。
    let dormant = |now: u64| {
        let mut doc = acct_doc();
        doc.dormant = vec![DormantSeat {
            project: "proj-a".to_string(),
            target: "proj-a:0.1".to_string(),
            account: Some("acct-1".to_string()),
            last: 1_790_506_800,
            tick_healthy: Reading::Known(true),
            heartbeat: Reading::Known(true),
        }];
        dormant_card(&account::drawn(doc, now)).expect("休止中の card").more[0].clone()
    };
    assert!(!dormant(AT).contains("09-27"), "{}", dormant(AT));
    assert!(dormant(NEXT_DAY).contains("09-27"), "{}", dormant(NEXT_DAY));
}

/// src の file の字の数。
fn count(text: &str, needle: &str) -> usize {
    text.matches(needle).count()
}

/// (8) DOM は組む時に今を渡し、電文の秒の欄の字を持たない。
#[test]
fn abss_face_wiring() {
    let seat_dom = read("src/project_dom/seat.rs");
    let pill = read("src/seatpill.rs");
    let board = read("src/account/board.rs");
    let home = read("src/account/home.rs");
    let projects = read("src/account/projects.rs");
    let session = read("src/account/session.rs");
    for (text, needle) in [
        (&seat_dom, "fetched.with(|f| content(f, crate::net::now()))"),
        (&seat_dom, "let now = crate::net::now();"),
        (&pill, "card(f, crate::net::now())"),
        (&pill, "Ok(c) => seat::drawn(c, now),"),
        (&board, "fetched.with(doc).ok().map(|d| drawn(d, net::now()))"),
        (&home, "fetched.with(|f| content(f, crate::net::now()))"),
        (&projects, "content(f, sort.get(), m, crate::net::now())"),
        (&session, "content(f, sort.get(), crate::net::now())"),
        (&session, "grace_left(until, at, tick.get())"),
    ] {
        assert_eq!(count(text, needle), 1, "字 {needle} の数");
    }
    for file in [
        "src/project_dom/seat.rs",
        "src/seatpill.rs",
        "src/account/home.rs",
        "src/account/projects.rs",
        "src/account/session.rs",
        "src/account/cards.rs",
        "src/project/seat.rs",
    ] {
        let text = read(file);
        for gone in ["move_left_s", "grace_left:", ".grace_left"] {
            assert!(!text.contains(gone), "{file} に字 {gone} が残る");
        }
    }
}
