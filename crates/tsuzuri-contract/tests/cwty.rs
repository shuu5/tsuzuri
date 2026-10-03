//! 相談の窓の型の歯（接頭辞 cwty_・設計ノート surface-wave27a 行 cs-types・判断の記録 ADR-29・要件 FR19）。
//! id の形の読みと断り・閉じた語・電文の往復・未知の鍵の断り・草稿の欄の名・保存の写し先の相対の path を見る。
#![cfg(test)]

use std::collections::BTreeSet;

use serde_json::Value;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{
    Adoption, Attachment, AttachmentKind, Claim, Confidence, ConsultBoard, ConsultRequest,
    ConsultUnreceived, DRAFT_FIELDS, Finding, FindingDraft, FindingId, FindingOption, FindingRow,
    Form, RequestId, RequestRow, SeatQuota, Starter, Verdict, Via, WindowId, WindowRow,
    WindowState, Word, kept_path, minute_ok,
};
use tsuzuri_contract::wire;

fn draft() -> FindingDraft {
    FindingDraft {
        topic: "t3-hub.75".into(),
        model: "fable・xhigh".into(),
        bundle_digest: "0123456789abcdef".into(),
        conclusion: "読む根を絞る".into(),
        options: vec![
            FindingOption {
                id: "a".into(),
                name: "絞る".into(),
                text: "fleet と pipe だけ".into(),
                verdict: Adoption::Adopted,
                reason: "資格が見えない".into(),
            },
            FindingOption {
                id: "b".into(),
                name: "丸ごと".into(),
                text: "state dir の全部".into(),
                verdict: Adoption::Rejected,
                reason: "資格が見える".into(),
            },
        ],
        recommendation: "a".into(),
        basis: vec!["ADR-29".into()],
        claims: vec![Claim {
            text: "accounts が読める".into(),
            confidence: Confidence::Verified,
            how: "撃った".into(),
        }],
        ask_owner: vec![],
        touches: vec!["FR19".into()],
        attachments: vec![Attachment {
            path: "work/report.md".into(),
            kind: AttachmentKind::Report,
            note: "調べ".into(),
        }],
        sandbox_denials: vec![],
    }
}

fn finding() -> Finding {
    let id = FindingId::parse("cw3-1").expect("所見 id");
    Finding::new(id, "20261003T1530Z", draft())
}

#[test]
fn cwty_window_ids_read_and_refuse() {
    let w = WindowId::parse("cw12").expect("窓 id");
    assert_eq!((w.n(), w.to_string()), (12, "cw12".to_string()));
    assert_eq!(w.name(), "consult-cw12");
    assert_eq!(w.retired_name(), "retired-consult-cw12");
    assert_eq!(WindowId::new(0), None);
    assert_eq!(WindowId::new(3).map(|w| w.to_string()), Some("cw3".into()));
    for bad in ["cw0", "cw", "cw01", "w1", "cw1x", "cw-1", "CW1", "cw1 "] {
        assert!(WindowId::parse(bad).is_err(), "{bad} が窓 id に読める");
    }
    let f = FindingId::parse("cw3-1").expect("所見 id");
    assert_eq!(
        (f.window().n(), f.k(), f.to_string()),
        (3, 1, "cw3-1".into())
    );
    assert_eq!(FindingId::new(w, 0), None);
    for bad in ["cw3-0", "cw3", "cw3-01", "cw0-1", "cw3-1-2", "cw3-", "-1"] {
        assert!(FindingId::parse(bad).is_err(), "{bad} が所見 id に読める");
    }
}

#[test]
fn cwty_request_ids_read_and_refuse() {
    let r = RequestId::parse("rq-20261003T1412Z-1").expect("頼み id");
    assert_eq!((r.minute(), r.n()), ("20261003T1412Z", 1));
    assert_eq!(r.to_string(), "rq-20261003T1412Z-1");
    assert_eq!(RequestId::new("20261003T1412", 1), None);
    for bad in [
        "rq-20261003T1412-1",
        "rq-20261003T1412Z-0",
        "rq-2026100XT1412Z-1",
        "rq-20261003T1412Z",
        "rq20261003T1412Z-1",
        "rq-20261003T1412Z-1x",
    ] {
        assert!(RequestId::parse(bad).is_err(), "{bad} が頼み id に読める");
    }
    assert!(minute_ok("20261003T1412Z"));
    assert!(!minute_ok("20261003T1412z"));
}

#[test]
fn cwty_words_closed() {
    fn words<T: Word>() -> Vec<&'static str> {
        T::ALL.iter().map(|v| v.word()).collect()
    }
    fn back<T: Word + std::fmt::Debug>() {
        for v in T::ALL {
            assert_eq!(T::from_word(v.word()), Some(*v));
        }
        assert_eq!(T::from_word("なし"), None);
    }
    assert_eq!(words::<Form>(), ["話す", "問う"]);
    assert_eq!(
        words::<Starter>(),
        ["席", "持ち主のチャット", "持ち主の button"]
    );
    assert_eq!(words::<Via>(), ["見張り", "完了", "hook", "一覧"]);
    assert_eq!(words::<Verdict>(), ["採る", "一部採る", "採らない"]);
    assert_eq!(
        words::<WindowState>(),
        ["生きている", "止まった", "閉じた", "退いた"]
    );
    back::<Form>();
    back::<Starter>();
    back::<Via>();
    back::<Verdict>();
    back::<WindowState>();
    let wire_words = |v: &str| wire::encode(&v).expect("字");
    assert_eq!(
        wire::encode(&Verdict::Partial).expect("電文"),
        wire_words("partial")
    );
    assert_eq!(wire::encode(&Via::Done).expect("電文"), wire_words("done"));
    assert_eq!(
        wire::encode(&Starter::Button).expect("電文"),
        wire_words("button")
    );
    assert_eq!(wire::encode(&Form::Ask).expect("電文"), wire_words("ask"));
    assert!(wire::decode::<Confidence>(r#""probable""#).is_err());
    assert!(wire::decode::<AttachmentKind>(r#""image""#).is_err());
}

#[test]
fn cwty_wire_roundtrip() {
    let f = finding();
    let text = wire::encode(&f).expect("電文");
    let v: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(v["id"], "cw3-1");
    assert_eq!(v["window"], "cw3");
    assert_eq!(v["options"][0]["verdict"], "adopted");
    assert_eq!(v["claims"][0]["confidence"], "verified");
    assert_eq!(v["attachments"][0]["kind"], "report");
    assert_eq!(wire::decode::<Finding>(&text).expect("往復"), f);
    let d = draft();
    let dt = wire::encode(&d).expect("電文");
    assert_eq!(wire::decode::<FindingDraft>(&dt).expect("往復"), d);
    let req = ConsultRequest {
        topic: None,
        form: Form::Talk,
        model: "fable".into(),
    };
    let rt = wire::encode(&req).expect("電文");
    assert_eq!(wire::decode::<ConsultRequest>(&rt).expect("往復"), req);
    let un = ConsultUnreceived {
        findings: vec![f.id],
        requests: vec![RequestId::parse("rq-20261003T1412Z-2").expect("頼み id")],
    };
    let ut = wire::encode(&un).expect("電文");
    assert_eq!(
        ut,
        r#"{"findings":["cw3-1"],"requests":["rq-20261003T1412Z-2"]}"#
    );
    assert_eq!(wire::decode::<ConsultUnreceived>(&ut).expect("往復"), un);
}

#[test]
fn cwty_board_roundtrip() {
    let f = finding();
    let board = ConsultBoard {
        windows: Reading::Known(vec![WindowRow {
            id: f.window,
            form: Form::Ask,
            topic: Some("t3-hub.75".into()),
            state: WindowState::Stalled,
            opened: Some(1_790_000_000),
            findings: 1,
            undisposed: 1,
        }]),
        findings: Reading::Known(vec![FindingRow {
            id: f.id,
            topic: None,
            arrived: Some(1_790_000_100),
            received: None,
        }]),
        requests: Reading::Known(vec![RequestRow {
            id: RequestId::parse("rq-20261003T1412Z-1").expect("頼み id"),
            topic: None,
            at: 1_790_000_200,
        }]),
        quota: Reading::Known(SeatQuota { today: 2, live: 1 }),
    };
    let bt = wire::encode(&board).expect("電文");
    assert_eq!(wire::decode::<ConsultBoard>(&bt).expect("往復"), board);
    let unknown = ConsultBoard {
        windows: Reading::Unknown,
        findings: Reading::Unknown,
        requests: Reading::Unknown,
        quota: Reading::Unknown,
    };
    let kt = wire::encode(&unknown).expect("電文");
    assert_eq!(wire::decode::<ConsultBoard>(&kt).expect("往復"), unknown);
}

#[test]
fn cwty_unknown_keys_refused() {
    let mut v: Value = serde_json::to_value(finding()).expect("値");
    v["extra"] = Value::from("x");
    assert!(wire::decode::<Finding>(&v.to_string()).is_err());
    let mut d: Value = serde_json::to_value(draft()).expect("値");
    d["id"] = Value::from("cw3-1");
    assert!(wire::decode::<FindingDraft>(&d.to_string()).is_err());
    let mut o: Value = serde_json::to_value(draft()).expect("値");
    o["options"][0]["score"] = Value::from(1);
    assert!(wire::decode::<FindingDraft>(&o.to_string()).is_err());
    let mut a: Value = serde_json::to_value(draft()).expect("値");
    a["attachments"][0]["size"] = Value::from(1);
    assert!(wire::decode::<FindingDraft>(&a.to_string()).is_err());
    let mut c: Value = serde_json::to_value(draft()).expect("値");
    c["claims"][0]["source"] = Value::from("x");
    assert!(wire::decode::<FindingDraft>(&c.to_string()).is_err());
    let mut w: Value = serde_json::to_value(finding()).expect("値");
    w["window"] = Value::from("cw0");
    assert!(wire::decode::<Finding>(&w.to_string()).is_err());
}

#[test]
fn cwty_draft_fields_and_kept_path() {
    let v: Value = serde_json::to_value(draft()).expect("値");
    let keys: BTreeSet<&str> = v
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    let want: BTreeSet<&str> = DRAFT_FIELDS.into_iter().collect();
    assert_eq!(DRAFT_FIELDS.len(), 12);
    assert_eq!(keys, want);
    let fv: Value = serde_json::to_value(finding()).expect("値");
    let fkeys: BTreeSet<&str> = fv
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    let mut fwant = want.clone();
    fwant.extend(["id", "window", "date"]);
    assert_eq!(fkeys, fwant);
    let id = FindingId::parse("cw3-1").expect("所見 id");
    assert_eq!(
        kept_path(id, "work/report.md"),
        "docs/consult/kept/cw3-1/work/report.md"
    );
}
