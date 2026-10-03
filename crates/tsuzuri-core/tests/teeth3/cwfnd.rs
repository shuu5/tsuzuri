//! 所見の欄の検めの歯（接頭辞 cwfnd_・設計ノート surface-wave27a 行 cs-finding・受入 AC16 の断りの半分）。
//! fixture: tests/fixtures/consult/finding-2/ の complete.json（12 欄が揃う）と missing.json（bundle_digest を欠く）。
#![cfg(test)]

use std::path::Path;

use serde_json::Value;
use tsuzuri_contract::consult::{Confidence, DRAFT_FIELDS};
use tsuzuri_core::consult::finding::{Refusal, check, path_ok};

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} を読む: {e}", path.display()))
}

fn complete() -> Value {
    serde_json::from_str(&read("tests/fixtures/consult/finding-2/complete.json")).expect("JSON")
}

#[test]
fn cwfnd_fixture_complete_and_missing() {
    let draft = check(&read("tests/fixtures/consult/finding-2/complete.json")).expect("揃った所見");
    assert_eq!(draft.topic, "fx-hub.7");
    assert_eq!(draft.options.len(), 2);
    assert_eq!(draft.attachments.len(), 2);
    let got = check(&read("tests/fixtures/consult/finding-2/missing.json"));
    assert_eq!(got, Err(Refusal::Missing(vec!["bundle_digest"])));
    let why = got.err().map(|r| r.to_string()).unwrap_or_default();
    assert_eq!(why, "欠けた欄: bundle_digest");
}

#[test]
fn cwfnd_each_field_missing_named() {
    assert_eq!(DRAFT_FIELDS.len(), 12);
    for field in DRAFT_FIELDS {
        let mut gone = complete();
        gone.as_object_mut().expect("object").remove(field);
        assert_eq!(
            check(&gone.to_string()),
            Err(Refusal::Missing(vec![field])),
            "{field} を除いた"
        );
        let mut null = complete();
        null[field] = Value::Null;
        assert_eq!(
            check(&null.to_string()),
            Err(Refusal::Missing(vec![field])),
            "{field} を null に"
        );
        if complete()[field].is_string() {
            let mut space = complete();
            space[field] = Value::from("  ");
            assert_eq!(
                check(&space.to_string()),
                Err(Refusal::Missing(vec![field])),
                "{field} を空白に"
            );
        }
    }
    let mut two = complete();
    let object = two.as_object_mut().expect("object");
    object.remove("sandbox_denials");
    object.remove("topic");
    assert_eq!(
        check(&two.to_string()),
        Err(Refusal::Missing(vec!["topic", "sandbox_denials"]))
    );
}

#[test]
fn cwfnd_unknown_and_not_object() {
    let mut extra = complete();
    extra["id"] = Value::from("cw1-1");
    extra["score"] = Value::from(3);
    assert_eq!(
        check(&extra.to_string()),
        Err(Refusal::Unknown(vec!["id".into(), "score".into()]))
    );
    for bad in ["[]", "x", "", "null", "3"] {
        assert_eq!(check(bad), Err(Refusal::NotObject), "{bad:?}");
    }
    let mut shape = complete();
    shape["basis"] = Value::from("ADR-29");
    assert!(matches!(check(&shape.to_string()), Err(Refusal::Shape(_))));
}

#[test]
fn cwfnd_options_one_adopted() {
    let base = complete();
    let a = base["options"][0].clone();
    let b = base["options"][1].clone();
    let mut adopted_b = b.clone();
    adopted_b["verdict"] = Value::from("adopted");
    let mut rejected_a = a.clone();
    rejected_a["verdict"] = Value::from("rejected");
    let cases = [
        (vec![a.clone()], false),
        (vec![a.clone(), adopted_b.clone()], false),
        (vec![rejected_a.clone(), b.clone()], false),
        (vec![rejected_a, adopted_b, b.clone()], true),
        (vec![a, b], true),
    ];
    for (options, ok) in cases {
        let mut v = base.clone();
        v["options"] = Value::from(options);
        let got = check(&v.to_string());
        if ok {
            assert!(got.is_ok(), "{got:?}");
        } else {
            assert_eq!(got, Err(Refusal::Options));
        }
    }
}

#[test]
fn cwfnd_confidence_four_words() {
    let words = [
        ("verified", Confidence::Verified),
        ("deduced", Confidence::Deduced),
        ("inferred", Confidence::Inferred),
        ("uncertain", Confidence::Uncertain),
    ];
    for (word, want) in words {
        let mut v = complete();
        v["claims"][0]["confidence"] = Value::from(word);
        let draft = check(&v.to_string()).expect("閉じた語");
        assert_eq!(draft.claims[0].confidence, want);
    }
    for bad in ["probable", "Verified", ""] {
        let mut v = complete();
        v["claims"][0]["confidence"] = Value::from(bad);
        assert!(
            matches!(check(&v.to_string()), Err(Refusal::Shape(_))),
            "{bad}"
        );
    }
}

#[test]
fn cwfnd_attachment_paths() {
    for good in ["work/report.md", "a.py", "work/x/y.csv", "work/..x/a"] {
        assert!(path_ok(good), "{good}");
    }
    for bad in [
        "",
        "/etc/passwd",
        "../x",
        "work/../../x",
        "work/..",
        "~/x",
        "a\\b",
        "a・b",
        "a、b",
        "a\nb",
    ] {
        assert!(!path_ok(bad), "{bad:?}");
        let mut v = complete();
        v["attachments"][1]["path"] = Value::from(bad);
        let want = if bad.is_empty() {
            Refusal::Path(String::new())
        } else {
            Refusal::Path(bad.to_string())
        };
        assert_eq!(check(&v.to_string()), Err(want), "{bad:?}");
    }
}
