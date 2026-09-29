//! 行 g-ruling-busy の歯: 問いの頁の送る button は送っている間に字を替え、server の台帳の断り
//! （503 ledger-unknown・502 ledger-append・502 ledger-close）は理由と次の手の字にし、断りの 1 行は目立つ class と role で出す。

use tsuzuri_contract::surface::RulingId;
use tsuzuri_surface::project::ask::{self, Outcome};

fn dom_text() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/project/ask.rs");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("ask.rs を読む: {e}"));
    let at = text.find("mod dom {").expect("ask.rs に mod dom が無い");
    text[at..].to_string()
}

/// 送っている間は「送っています…」・ほかは決定の語に空白と › を足した字。
#[test]
fn rbusy_send_text() {
    assert_eq!(ask::SENDING, "送っています…");
    assert_eq!(ask::send_text(true, "決定"), ask::SENDING);
    assert_eq!(ask::send_text(false, "決定"), "決定 ›");
    assert_eq!(ask::send_text(false, "ruling"), "ruling ›");
}

/// server の台帳の断りは理由と次の手の字・ほかの読めない本文は「状態 <数>」のまま。
#[test]
fn rbusy_reasons() {
    assert_eq!(
        ask::LEDGER_BUSY,
        "台帳が混んでいて読めなかった（503）・何も書いていない・少し待ってもう一度押す"
    );
    assert_eq!(
        ask::APPEND_FAILED,
        "台帳に書けなかった（502）・何も書いていない・もう一度押す"
    );
    assert_eq!(
        ask::CLOSE_FAILED,
        "記録したが問いを閉じられなかった（502）・もう一度押さず席に知らせる"
    );
    assert_eq!(
        ask::outcome(Some((503, "ledger-unknown"))),
        Outcome::Refused(ask::LEDGER_BUSY.to_string())
    );
    assert_eq!(
        ask::outcome(Some((502, "ledger-append"))),
        Outcome::Refused(ask::APPEND_FAILED.to_string())
    );
    let id = RulingId::new("qa.2:20260929T0231Z-1").expect("裁定の id");
    let body = format!("ledger-close {id}");
    assert_eq!(
        ask::outcome(Some((502, &body))),
        Outcome::Refused(format!("{}（{id}）", ask::CLOSE_FAILED))
    );
    // 前後の空白は除いて読む。
    assert_eq!(
        ask::server_reason(503, " ledger-unknown\n"),
        Some(ask::LEDGER_BUSY.to_string())
    );
    assert_eq!(ask::server_reason(503, "busy"), None);
    assert_eq!(ask::server_reason(502, "ledger-unknown"), None);
    assert_eq!(ask::server_reason(503, "ledger-append"), None);
    assert_eq!(ask::server_reason(502, "ledger-close"), None);
    assert_eq!(
        ask::outcome(Some((500, "ruling-id-shape"))),
        Outcome::Refused("状態 500".to_string())
    );
    assert_eq!(
        ask::outcome(Some((503, "busy"))),
        Outcome::Refused("状態 503".to_string())
    );
}

/// 断りの 1 行は warn と alert・ほかは small と status で、mod dom はその組で 1 行を組み、button の中身は send の閉包。
#[test]
fn rbusy_note_is_alert() {
    let id = RulingId::new("qa.2:20260929T0231Z-1").expect("裁定の id");
    assert_eq!(
        Outcome::Refused(ask::LEDGER_BUSY.to_string()).note(),
        ("warn", "alert")
    );
    assert_eq!(Outcome::Stale.note(), ("small", "status"));
    assert_eq!(
        Outcome::Recorded {
            ruling: id,
            at: 1_790_650_260
        }
        .note(),
        ("small", "status")
    );
    let dom = dom_text();
    for want in [
        "let send = move || send_text(busy.get(), &label(",
        "on:click=click>{send}</button>",
        "let (class, role) = o.note();",
        "<div class=class role=role>{o.line()}</div>",
    ] {
        assert!(dom.contains(want), "ask.rs の mod dom に {want} が無い");
    }
}
