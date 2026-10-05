//! 会話の印の読みの歯（接頭辞 cwstm_・設計ノート surface-wave29b 行 cs-acct-mark・判断の記録 ADR-55 決定 (2)）。
//! hook の入力の JSON から事と会話の id と始まりの種類を読み、知らない事と uuid の形でない id を断り、印の file の字の読める行と最後の id を引くことを見る。
#![cfg(test)]

use tsuzuri_contract::consult::{Stamp, StampEvent};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::stamp::{is_uuid, last_sid, lines, read};

const SID: &str = "5e1d0c2a-3b4f-4a6e-9d8c-7b6a5f4e3d21";
const SID2: &str = "772C5B4C-0000-4000-8000-00000000ABCD";

fn payload(event: &str, sid: &str, source: Option<&str>) -> String {
    let mut v = serde_json::json!({"hook_event_name": event, "session_id": sid, "cwd": "/W"});
    if let Some(s) = source {
        v["source"] = s.into();
    }
    v.to_string()
}

#[test]
fn cwstm_uuid_shape() {
    for ok in [SID, SID2, "00000000-0000-0000-0000-000000000000"] {
        assert!(is_uuid(ok), "{ok}");
    }
    for bad in [
        "",
        "5e1d0c2a-3b4f-4a6e-9d8c-7b6a5f4e3d2",
        "5e1d0c2a-3b4f-4a6e-9d8c-7b6a5f4e3d218",
        "5e1d0c2a3b4f4a6e9d8c7b6a5f4e3d21",
        "5e1d0c2a-3b4f-4a6e-9d8c-7b6a5f4e3d2g",
        "5e1d0c2a-3b4f-4a6e-9d8c7-b6a5f4e3d21",
        "5e1d0c2a-3b4f-4a6e-9d8c-7b6a-5f4e3d21",
        " 5e1d0c2a-3b4f-4a6e-9d8c-7b6a5f4e3d21",
        "5e1d0c2a-3b4f-4a6e-9d8c-7b6a5f4e3d21-0000",
    ] {
        assert!(!is_uuid(bad), "{bad:?}");
    }
}

#[test]
fn cwstm_reads_the_three_events() {
    let start = read(&payload("SessionStart", SID, Some("clear")), 7);
    let want = Stamp {
        at: 7,
        event: StampEvent::Start,
        sid: SID.into(),
        source: Some("clear".into()),
    };
    assert_eq!(start, Some(want));
    for (name, event) in [
        ("UserPromptSubmit", StampEvent::Prompt),
        ("Stop", StampEvent::Stop),
    ] {
        let got = read(&payload(name, SID2, Some("resume")), 9);
        let want = Stamp {
            at: 9,
            event,
            sid: SID2.into(),
            source: None,
        };
        assert_eq!(got, Some(want), "{name} は始まりの種類を持たない");
    }
    let bare = read(&payload("SessionStart", SID, None), 1).expect("source なし");
    assert_eq!(bare.source, None);
}

#[test]
fn cwstm_refuses_other_payloads() {
    for p in [
        payload("PreToolUse", SID, None),
        payload("SessionEnd", SID, None),
        payload("Stop", "not-a-uuid", None),
        payload("Stop", "", None),
        r#"{"hook_event_name":"Stop"}"#.to_string(),
        r#"{"session_id":"5e1d0c2a-3b4f-4a6e-9d8c-7b6a5f4e3d21"}"#.to_string(),
        r#"{"hook_event_name":"Stop","session_id":7}"#.to_string(),
        "not json".to_string(),
        String::new(),
    ] {
        assert_eq!(read(&p, 1), None, "{p}");
    }
}

#[test]
fn cwstm_lines_and_last_sid() {
    let a = read(&payload("SessionStart", SID, Some("startup")), 1).unwrap();
    let b = read(&payload("Stop", SID2, None), 2).unwrap();
    let text = format!(
        "{}\nnot json\n{{\"at\":3,\"event\":\"stop\",\"sid\":\"x\",\"source\":null,\"extra\":1}}\n\n{}\n",
        wire::encode(&a).unwrap(),
        wire::encode(&b).unwrap()
    );
    let got = lines(&text);
    assert_eq!(got, [a.clone(), b]);
    assert_eq!(last_sid(&got), Some(SID2));
    assert_eq!(last_sid(&got[..1]), Some(SID));
    assert_eq!(last_sid(&[]), None);
    assert_eq!(lines(""), []);
}
