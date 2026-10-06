//! 帯の相談の口と相談の窓の歯（行 cs-bar・接頭辞 cwbar_・判断の記録 ADR-29 決定 (4)(9)）。
//! 純な関数（頼みの電文・一覧の段・応答の 1 行・窓の名と幅と語）を host で撃ち、窓の DOM（wasm の枝）は配線の字を読む。
#![cfg(test)]

use crate::common::read;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{
    FindingId, FindingRow, Form, RequestId, RequestRow, WindowId, WindowRow, WindowState,
};
use tsuzuri_surface::consultwin::{
    CONSULT_KEY, DELIVER_SPAN, LATE, MODELS, NONE_LINES, Row, UNREAD, board, findings, request,
    requests, sent_line, windows,
};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::topbar::{TODOS, Win, win_href, win_of_href};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::wins::frame_of;

/// file の wasm の枝（`mod dom {` から後）。
fn dom(rel: &str) -> String {
    let src = read(rel);
    let at = src
        .find("mod dom {")
        .unwrap_or_else(|| panic!("{rel} に wasm の枝が無い"));
    src[at..].to_string()
}

fn row(id: &str, topic: &str, note: &str, late: bool) -> Row {
    Row {
        id: id.to_string(),
        topic: topic.to_string(),
        note: note.to_string(),
        late,
    }
}

fn rid(s: &str) -> RequestId {
    RequestId::parse(s).expect("頼みの id")
}

/// 窓の名は 8 つ目に consult を足し（9 つ目は host の窓）、URL の query に読み戻り、幅は 720 で、題の語は語彙の「相談」と空でない注釈。
#[test]
fn cwbar_window_named_consult() {
    assert_eq!(Win::ALL.len(), 9);
    assert_eq!(Win::ALL.get(7), Some(&Win::Consult));
    assert_eq!(Win::Consult.key(), "consult");
    for mode in Mode::ALL {
        assert_eq!(
            win_of_href(&win_href(Win::Consult, mode)),
            Some((Win::Consult, None))
        );
    }
    assert_eq!(CONSULT_KEY, "consult");
    assert_eq!(frame_of(Win::Consult), (720, CONSULT_KEY));
    let term = vocab().term(CONSULT_KEY).expect("語彙の consult");
    assert_eq!(term.label, "相談");
    assert!(!term.note.is_empty(), "注釈が空");
}

/// 帯の相談の口は色の付く数を持たない（やる事の 3 つに入らない）・帯の wasm の枝は consult の語の button で窓を開く。
#[test]
fn cwbar_band_button_without_count() {
    assert_eq!(TODOS.len(), 3);
    assert!(
        TODOS.iter().all(|(_, _, w)| *w != Win::Consult),
        "やる事の数に相談が在る"
    );
    let bar = dom("src/topbar.rs");
    let button = "<button type=\"button\" class=\"cslt\" aria-label=label(CONSULT_KEY) on:click=move |_| wins.open(Win::Consult, false)>{label(CONSULT_KEY)}</button>";
    assert_eq!(bar.matches(button).count(), 1, "帯の相談の口");
    assert_eq!(
        bar.matches("Win::Consult").count(),
        1,
        "帯の wasm の枝の相談の窓の口"
    );
}

/// 頼みの電文は題の前後の空白を除き（空なら null）・形の語・model を持ち、model が選べる 3 つでなければ None。
#[test]
fn cwbar_request_body() {
    assert_eq!(MODELS, ["fable", "opus", "sonnet"]);
    assert_eq!(
        request("  t3-hub.78 ", Form::Ask, "opus").as_deref(),
        Some(r#"{"topic":"t3-hub.78","form":"ask","model":"opus"}"#)
    );
    assert_eq!(
        request(" \t", Form::Talk, "fable").as_deref(),
        Some(r#"{"topic":null,"form":"talk","model":"fable"}"#)
    );
    assert_eq!(request("t3-hub.78", Form::Ask, "haiku"), None);
    assert_eq!(request("t3-hub.78", Form::Ask, ""), None);
}

/// 受けの無い頼みは、頼みの時刻から 1800 秒を越えた時だけ「席に届いていない」で、ちょうど 1800 秒は待ち。
#[test]
fn cwbar_requests_late_after_span() {
    assert_eq!(DELIVER_SPAN, 1800);
    assert_eq!(LATE, "席に届いていない");
    let at = 1_000_000;
    let list = Reading::Known(vec![
        RequestRow {
            id: rid("rq-20261003T1412Z-1"),
            topic: Some("t3-hub.78".to_string()),
            at,
        },
        RequestRow {
            id: rid("rq-20261003T1442Z-1"),
            topic: None,
            at: at + 1,
        },
    ]);
    assert_eq!(
        requests(&list, at + DELIVER_SPAN + 1),
        Body::Filled(vec![
            row("rq-20261003T1412Z-1", "t3-hub.78", LATE, true),
            row("rq-20261003T1442Z-1", "題なし", "席の受けを待つ", false),
        ])
    );
    let Body::Filled(rows) = requests(&list, at + DELIVER_SPAN) else {
        panic!("2 行");
    };
    assert!(
        rows.iter().all(|r| !r.late),
        "ちょうど 1800 秒で届いていない"
    );
    assert_eq!(
        requests(&Reading::Known(Vec::new()), at),
        Body::Empty(NONE_LINES[2])
    );
    assert_eq!(requests(&Reading::Unknown, at), Body::Unmeasured(UNREAD));
}

fn window(n: u32, state: WindowState) -> WindowRow {
    WindowRow {
        id: WindowId::parse(&format!("cw{n}")).expect("窓の id"),
        form: Form::Ask,
        topic: None,
        state,
        opened: Some(1),
        findings: 2,
        undisposed: 1,
        account: None,
    }
}

/// 窓の段の行は口座を並べる（最後の印の口座の置き場の末の名・無ければ分からない・判断の記録 ADR-55 決定 (4)）。
#[test]
fn cwbar_window_note_holds_the_account() {
    let named = WindowRow {
        account: Some("acct-x".into()),
        ..window(1, WindowState::Stalled)
    };
    assert_eq!(
        windows(&Reading::Known(vec![named, window(2, WindowState::Live)])),
        Body::Filled(vec![
            row("cw1", "題なし", "問う・止まった／所見 2（処分なし 1）／口座 acct-x", false),
            row("cw2", "題なし", "問う・生きている／所見 2（処分なし 1）／口座 分からない", false),
        ])
    );
    assert_eq!(
        tsuzuri_surface::vocab::label("consult_account_unknown"),
        "分からない"
    );
}

/// 窓の段は退いた窓を載せず形と状態と所見の数を並べ、所見の段は席の受けを書き、読めない口と段は測れていない。
#[test]
fn cwbar_lists_and_unread() {
    let ws = Reading::Known(vec![
        window(1, WindowState::Live),
        window(2, WindowState::Retired),
    ]);
    assert_eq!(
        windows(&ws),
        Body::Filled(vec![row(
            "cw1",
            "題なし",
            "問う・生きている／所見 2（処分なし 1）／口座 分からない",
            false
        )])
    );
    let gone = Reading::Known(vec![window(2, WindowState::Retired)]);
    assert_eq!(windows(&gone), Body::Empty(NONE_LINES[0]));
    let fs = Reading::Known(vec![
        FindingRow {
            id: FindingId::parse("cw1-1").expect("所見の id"),
            topic: Some("t3-hub.78".to_string()),
            arrived: Some(1),
            received: Some(2),
        },
        FindingRow {
            id: FindingId::parse("cw1-2").expect("所見の id"),
            topic: None,
            arrived: Some(3),
            received: None,
        },
    ]);
    assert_eq!(
        findings(&fs),
        Body::Filled(vec![
            row("cw1-1", "t3-hub.78", "席が受けた", false),
            row("cw1-2", "題なし", "席が受けていない", false),
        ])
    );
    assert_eq!(findings(&Reading::Unknown), Body::Unmeasured(UNREAD));
    assert_eq!(board(&Fetched::Failed), Err(UNREAD));
    assert_eq!(board(&Fetched::NotRead), Err(UNREAD));
    assert_eq!(board(&Fetched::Body("[]".to_string())), Err(UNREAD));
    let text =
        r#"{"windows":"unknown","findings":{"known":[]},"requests":"unknown","quota":"unknown"}"#;
    let got = board(&Fetched::Body(text.to_string())).expect("一覧");
    assert_eq!(got.findings, Reading::Known(Vec::new()));
    assert_eq!(windows(&got.windows), Body::Unmeasured(UNREAD));
}

/// 頼みの口の応答の 1 行（200 は頼みの id・403・400・ほかの状態と本文・届かない）。
#[test]
fn cwbar_sent_lines() {
    assert_eq!(
        sent_line(Some((200, r#""rq-20261003T1412Z-1""#))),
        "頼み rq-20261003T1412Z-1 を送った（席が受けて窓を開く）"
    );
    assert_eq!(sent_line(Some((200, "x"))), "頼みの応答が読めない");
    assert_eq!(
        sent_line(Some((403, ""))),
        "読むだけの board なので送れない"
    );
    assert_eq!(
        sent_line(Some((400, "bad-line"))),
        "頼みの字が書けない（題の字を見直す）"
    );
    assert_eq!(
        sent_line(Some((503, "no-root"))),
        "頼みを置けなかった（状態 503・no-root）"
    );
    assert_eq!(sent_line(None), "board に届かない");
}

/// 窓の DOM の配線の字: 窓の名の振り分け・一覧の口の読み・頼みの口の送り・題の初めの字は窓を開いた link の id。
#[test]
fn cwbar_dom_wiring_text() {
    assert!(dom("src/wins.rs").contains("Win::Consult => crate::consultwin::body(),"));
    let win = dom("src/consultwin.rs");
    for needle in [
        "crate::net::read(PATH)",
        "crate::net::post(REQUEST_PATH, body).await",
        "crate::net::reload_all();",
        "use_context::<AskFocus>().and_then(|f| f.0.get_untracked())",
        "requests(&b.requests, now.get())",
    ] {
        assert_eq!(
            win.matches(needle).count(),
            1,
            "wasm の枝に {needle} が 1 つでない"
        );
    }
}
