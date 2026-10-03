//! 便 h-hb の歯: session の表の orchestrator の行の停止の切り替え（button の字・確かめの段の字・要求の本文・
//! 応答の出し方・送る間の状態の移り方）。fixture は便 b-acct の tests/fixtures/account/acct-doc.json（読むだけ）。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::account::{
    self as contract_account, AccountDoc, Heartbeat, HeartbeatRequest,
};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::heartbeat::{
    BAD_BODY, BUTTON, CANCEL, CMD, Event, FIRE, HEARTBEAT_PATH, LEAD_KEY, NO_PROJECT, NO_SEAT,
    NOT_REACHED, Outcome, REFUSED, RESUMED, RowState, SCOPE_KEY, STOPPED, SendState, Toggle,
    VESSEL_FAILED, button_key, command, heartbeat_of, opposite, outcome, panel, request_body,
    starts, step, toggle,
};
use tsuzuri_surface::account::session::{self, Sort, table};
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture() -> AccountDoc {
    wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
        .expect("fixture が AccountDoc として読める")
}

/// fixture の行の切り替え（行は sessions の順: proj-a の orchestrator・proj-a の pipeline・proj-b・proj-c）。
fn toggles(doc: &AccountDoc) -> Vec<Option<Toggle>> {
    doc.sessions.iter().map(|l| toggle(doc, l)).collect()
}

/// (1) button の字は on なら hb_to_off・off なら hb_to_on で、読めない行と pipeline の行は button を持たない。
#[test]
fn accthb_button_on_fixture() {
    let doc = fixture();
    let names: Vec<(&str, &str)> = doc
        .sessions
        .iter()
        .map(|l| (l.project.as_str(), l.name.as_str()))
        .collect();
    assert_eq!(
        names,
        vec![
            ("proj-a", "proj-a-orch"),
            ("proj-a", "proj-a.3-20260927T113000Z"),
            ("proj-b", "proj-b-orch"),
            ("proj-c", "proj-c-orch")
        ]
    );
    assert_eq!(heartbeat_of(&doc, "proj-a"), Reading::Known(Heartbeat::On));
    assert_eq!(heartbeat_of(&doc, "proj-b"), Reading::Known(Heartbeat::Off));
    assert_eq!(heartbeat_of(&doc, "proj-c"), Reading::Unknown);
    assert_eq!(heartbeat_of(&doc, "no-such"), Reading::Unknown);

    let got = toggles(&doc);
    assert_eq!(
        got,
        vec![
            Some(Toggle {
                project: "proj-a".to_string(),
                target: "proj-a-orch".to_string(),
                now: Heartbeat::On,
                to: Heartbeat::Off,
            }),
            None,
            Some(Toggle {
                project: "proj-b".to_string(),
                target: "proj-b-orch".to_string(),
                now: Heartbeat::Off,
                to: Heartbeat::On,
            }),
            None,
        ]
    );
    let a = got[0].as_ref().expect("proj-a");
    let b = got[2].as_ref().expect("proj-b");
    assert_eq!(a.button_key(), "hb_to_off");
    assert_eq!(b.button_key(), "hb_to_on");
    assert_eq!(button_key(Heartbeat::On), "hb_to_off");
    assert_eq!(button_key(Heartbeat::Off), "hb_to_on");
    for h in Heartbeat::ALL {
        assert_ne!(opposite(h), h);
        assert_eq!(opposite(opposite(h)), h);
    }
    assert_eq!(BUTTON, "btn hbbtn");
    rows_share_toggles(doc, got);
}

/// 表の行と session の行は fixture の切り替えをそのまま持つ。
fn rows_share_toggles(doc: AccountDoc, got: Vec<Option<Toggle>>) {
    // 表の行も同じ切り替えを持つ（並べ方に依らない）。
    for sort in [Sort::Project, Sort::Account, Sort::Stage, Sort::Elapsed] {
        let t = table(&doc, sort);
        for r in t.groups.iter().flat_map(|g| g.rows.iter()) {
            assert_eq!(r.toggle, got[r.index], "{sort:?} の {} 行目", r.index);
        }
    }
    assert_eq!(session::row(&doc, 0).toggle, got[0]);
}

/// (1) heartbeat が読めない・席の card が読めない・project の行が無い・席の名が空の行は button を持たない。
#[test]
fn accthb_no_button_when_unknown() {
    let base = fixture();

    let mut hb_unknown = base.clone();
    if let Reading::Known(card) = &mut hb_unknown.projects[0].seat {
        card.heartbeat = Reading::Unknown;
    } else {
        panic!("proj-a の席の card が読める");
    }
    assert_eq!(toggle(&hb_unknown, &hb_unknown.sessions[0]), None);
    assert_eq!(session::row(&hb_unknown, 0).toggle, None);

    let mut seat_unknown = base.clone();
    seat_unknown.projects[1].seat = Reading::Unknown;
    assert_eq!(toggle(&seat_unknown, &seat_unknown.sessions[2]), None);

    let mut no_project = base.clone();
    no_project.projects.retain(|p| p.name != "proj-a");
    assert_eq!(toggle(&no_project, &no_project.sessions[0]), None);

    let mut no_name = base.clone();
    no_name.sessions[0].name = String::new();
    assert_eq!(toggle(&no_name, &no_name.sessions[0]), None);

    // pipeline の行は project の heartbeat が読めても持たない。
    assert_eq!(heartbeat_of(&base, "proj-a"), Reading::Known(Heartbeat::On));
    assert_eq!(toggle(&base, &base.sessions[1]), None);
    assert_eq!(session::row(&base, 1).toggle, None);
}

/// (2) 確かめの段は語の鍵 hb_dlg_lead と hb_dlg_scope と、state dir を出さない撃つ字と、やめると撃つの 2 つの button。
#[test]
fn accthb_panel_texts() {
    let doc = fixture();
    let t = toggles(&doc);
    let a = t[0].as_ref().expect("proj-a");
    let b = t[2].as_ref().expect("proj-b");
    assert_eq!(
        a.command(),
        "scribe2 seat heartbeat off --target proj-a-orch"
    );
    assert_eq!(
        b.command(),
        "scribe2 seat heartbeat on --target proj-b-orch"
    );
    assert_eq!(
        command(Heartbeat::Off, "x-orch"),
        "scribe2 seat heartbeat off --target x-orch"
    );
    let p = panel(a);
    assert_eq!(p.lead_key, "hb_dlg_lead");
    assert_eq!(p.scope_key, "hb_dlg_scope");
    assert_eq!(p.command, a.command());
    assert_eq!((p.cancel, p.fire), ("やめる", "撃つ"));
    assert_eq!(
        (LEAD_KEY, SCOPE_KEY, CANCEL, FIRE),
        ("hb_dlg_lead", "hb_dlg_scope", "やめる", "撃つ")
    );
    assert!(!p.command.contains("--state-dir"));
    assert_eq!(CMD, "cmd");

    // 語の鍵は語の辞書に在る。
    for key in ["hb_to_off", "hb_to_on", LEAD_KEY, SCOPE_KEY] {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }
    // class は stylesheet に在る。
    let css = read("style.css");
    for class in BUTTON
        .split_whitespace()
        .chain([CMD, "tkhb", "primary", "small", "muted", "row", "stack"])
    {
        assert!(
            css.contains(&format!(".{class}")),
            "stylesheet に class {class} が無い"
        );
    }
}

/// (3) 要求の本文は HeartbeatRequest で project と to だけを持ち、口は契約の型の HEARTBEAT_PATH。
#[test]
fn accthb_request_body() {
    let doc = fixture();
    let t = toggles(&doc);
    let a = t[0].as_ref().expect("proj-a");
    let b = t[2].as_ref().expect("proj-b");
    let got: HeartbeatRequest = wire::decode(&a.request_body()).expect("本文が要求として読める");
    assert_eq!(
        got,
        HeartbeatRequest {
            project: "proj-a".to_string(),
            to: Heartbeat::Off,
        }
    );
    let got: HeartbeatRequest = wire::decode(&b.request_body()).expect("本文が要求として読める");
    assert_eq!(
        got,
        HeartbeatRequest {
            project: "proj-b".to_string(),
            to: Heartbeat::On,
        }
    );
    // 欄は project と to の 2 つだけ（席の名も state dir も送らない）。
    assert_eq!(a.request_body(), r#"{"project":"proj-a","to":"off"}"#);
    assert_eq!(b.request_body(), r#"{"project":"proj-b","to":"on"}"#);
    assert!(!a.request_body().contains("proj-a-orch"));
    assert_eq!(request_body("proj-a", Heartbeat::Off), a.request_body());

    assert_eq!(HEARTBEAT_PATH, contract_account::HEARTBEAT_PATH);
    let src = read("src/account/heartbeat.rs");
    assert!(src.contains("crate::net::post(HEARTBEAT_PATH, body)"));
}

/// 撃った向き・応答（届かなければ None）・出す字・口を全部読み直すか。
type Case = (Heartbeat, Option<(u16, &'static str)>, &'static str, bool);

/// (4) 応答の出し方: 200 は止めたか戻したで口を全部読み直す・決まった断りは節の字・ほかは口が断ったと状態の数・届かなければ口に届かない。
#[test]
fn accthb_reply_lines() {
    let ok = r#"{"target":"proj-a-orch","to":"off"}"#;
    let cases: Vec<Case> = vec![
        (Heartbeat::Off, Some((200, ok)), "止めた", true),
        (
            Heartbeat::On,
            Some((200, r#"{"target":"proj-b-orch","to":"on"}"#)),
            "戻した",
            true,
        ),
        (
            Heartbeat::Off,
            Some((404, "no-project")),
            "project が群の宣言に無い",
            false,
        ),
        (
            Heartbeat::On,
            Some((404, "no-seat")),
            "orchestrator の席が見つからない",
            false,
        ),
        (
            Heartbeat::Off,
            Some((400, "bad-body")),
            "要求の本文が読めない",
            false,
        ),
        (
            Heartbeat::Off,
            Some((502, "vessel-failed")),
            "器の CLI が失敗した（5 秒で返らないか rc が 0 でない）",
            false,
        ),
        (
            Heartbeat::Off,
            Some((403, "origin")),
            "口が断った（403）",
            false,
        ),
        (
            Heartbeat::Off,
            Some((413, "too-large")),
            "口が断った（413）",
            false,
        ),
        (
            Heartbeat::Off,
            Some((404, "not-found")),
            "口が断った（404）",
            false,
        ),
    ];
    reply_rest(ok, cases);
}

/// 残りの断りの組を足して出す字と読み直しを照らし、決まった応答の型と字の定数を見る。
fn reply_rest(ok: &str, mut cases: Vec<Case>) {
    cases.extend([
        (
            Heartbeat::Off,
            Some((400, "no-seat")),
            "口が断った（400）",
            false,
        ),
        (
            Heartbeat::Off,
            Some((502, "bad-body")),
            "口が断った（502）",
            false,
        ),
        (Heartbeat::On, Some((500, "")), "口が断った（500）", false),
        (Heartbeat::Off, None, "口に届かない", false),
    ]);
    for (to, reply, line, reloads) in cases {
        let o = outcome(to, reply);
        assert_eq!(o.line(), line, "{to:?} {reply:?}");
        assert_eq!(o.reloads(), reloads, "{to:?} {reply:?}");
    }
    assert_eq!(
        outcome(Heartbeat::Off, Some((200, ok))),
        Outcome::Done(Heartbeat::Off)
    );
    assert_eq!(
        outcome(Heartbeat::Off, Some((404, "no-seat\n"))),
        Outcome::Known(NO_SEAT)
    );
    assert_eq!(outcome(Heartbeat::Off, None), Outcome::NotReached);
    assert_eq!(
        outcome(Heartbeat::Off, Some((418, "x"))),
        Outcome::Refused(418)
    );
    assert_eq!(
        (
            STOPPED,
            RESUMED,
            NO_PROJECT,
            BAD_BODY,
            VESSEL_FAILED,
            REFUSED,
            NOT_REACHED
        ),
        (
            "止めた",
            "戻した",
            "project が群の宣言に無い",
            "要求の本文が読めない",
            "器の CLI が失敗した（5 秒で返らないか rc が 0 でない）",
            "口が断った",
            "口に届かない"
        )
    );
    // 200 の後だけ口を全部読み直す（net の reload_all）。
    let src = read("src/account/heartbeat.rs");
    assert!(src.contains("crate::net::reload_all()"));
}

/// (5) 送る間の状態: 段を開いて撃つと送っている・送っている間は押せず他の出来事を受けない・応答で段を閉じて字を出す。
#[test]
fn accthb_send_states() {
    let idle = RowState::default();
    assert_eq!(
        idle,
        RowState {
            open: false,
            send: SendState::Idle
        }
    );
    assert!(idle.can_press());
    assert_eq!(idle.reply_line(), None);

    // 段が閉じていれば撃たない。
    assert_eq!(step(&idle, Event::Fire), idle);
    assert!(!starts(&idle, &step(&idle, Event::Fire)));
    // 送っていなければ応答を受けない。
    assert_eq!(step(&idle, Event::Reply(Outcome::NotReached)), idle);

    let open = step(&idle, Event::Open);
    assert_eq!(
        open,
        RowState {
            open: true,
            send: SendState::Idle
        }
    );
    assert_eq!(step(&open, Event::Cancel), idle);

    let sending = step(&open, Event::Fire);
    assert_eq!(
        sending,
        RowState {
            open: true,
            send: SendState::Sending
        }
    );
    assert!(starts(&open, &sending));
    assert!(!sending.can_press());
    for e in [Event::Open, Event::Cancel, Event::Fire] {
        assert_eq!(step(&sending, e.clone()), sending, "{e:?}");
    }
    assert!(!starts(&sending, &step(&sending, Event::Fire)));
    after_reply(sending);
}

/// 応答で段を閉じて字を出し、応答の後も開き直して撃てる。
fn after_reply(sending: RowState) {
    let done = step(&sending, Event::Reply(Outcome::Done(Heartbeat::Off)));
    assert_eq!(
        done,
        RowState {
            open: false,
            send: SendState::Replied(Outcome::Done(Heartbeat::Off))
        }
    );
    assert!(done.can_press());
    assert_eq!(done.reply_line(), Some("止めた".to_string()));

    // 応答の後も開き直して撃てる（応答の字は次の応答まで残る）。
    let again = step(&done, Event::Open);
    assert!(again.open);
    assert_eq!(again.reply_line(), Some("止めた".to_string()));
    let resend = step(&again, Event::Fire);
    assert!(starts(&again, &resend));
    assert_eq!(resend.reply_line(), None);
    let refused = step(&resend, Event::Reply(Outcome::Refused(403)));
    assert_eq!(refused.reply_line(), Some("口が断った（403）".to_string()));

    // 送っている間は button を押せない（view の disabled）。
    let src = read("src/account/heartbeat.rs");
    assert!(src.contains("disabled=disabled"));
    assert!(src.contains("can_press()"));
}

/// (2)(6) dialog の要素を使わず、account の下の file は保存の口と口の字を持たない。
#[test]
fn accthb_no_dialog_and_no_storage() {
    let src = read("src/account/heartbeat.rs");
    assert!(!src.contains("<dialog"));
    assert!(!src.contains("HtmlDialogElement"));
    for word in ["/api/", "localStorage", "sessionStorage", "cookie"] {
        assert!(!src.contains(word), "heartbeat.rs に {word} の字が在る");
    }
    let session_src = read("src/account/session.rs");
    assert!(session_src.contains("heartbeat::button("));
    assert!(session_src.contains("heartbeat::below("));
}
