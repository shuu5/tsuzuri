//! 口座の信頼の印の字の歯（接頭辞 cwtru_・設計ノート surface-wave29b 行 cs-trust・判断の記録 ADR-55 決定 (3)）。
#![cfg(test)]

use tsuzuri_core::consult::trust::{TRUST_FILE, TRUST_KEY, trusted};

/// 信頼の印は作業場の行だけに置き、ほかの鍵と行と作業場の行のほかの欄は替えない。file が無い時は印だけの字を組み、既に真なら
/// 書かない（None）。末の改行は元の字に合わせる。JSON でない字・object でない根か projects か作業場の行・真偽でない印は断る。
#[test]
fn cwtru_trusted_sets_only_the_workspace() {
    assert_eq!(
        (TRUST_FILE, TRUST_KEY),
        (".claude.json", "hasTrustDialogAccepted")
    );
    let fresh = "{\n  \"projects\": {\n    \"/W\": {\n      \"hasTrustDialogAccepted\": true\n    }\n  }\n}\n";
    assert_eq!(trusted(None, "/W"), Ok(Some(fresh.to_string())));
    assert_eq!(
        trusted(Some("{}"), "/W"),
        Ok(Some(fresh.trim_end().to_string()))
    );
    let before = r#"{"projects":{"/D":{"hasTrustDialogAccepted":false},"/W":{"allowedTools":[],"hasTrustDialogAccepted":false}},"z":3}"#;
    let want = "{\n  \"projects\": {\n    \"/D\": {\n      \"hasTrustDialogAccepted\": false\n    },\n    \"/W\": {\n      \
                \"allowedTools\": [],\n      \"hasTrustDialogAccepted\": true\n    }\n  },\n  \"z\": 3\n}";
    assert_eq!(trusted(Some(before), "/W"), Ok(Some(want.to_string())));
    assert_eq!(
        trusted(Some(&format!("{before}\n")), "/W"),
        Ok(Some(format!("{want}\n")))
    );
    assert_eq!(trusted(Some(want), "/W"), Ok(None));
    let sub = trusted(Some(want), "/W/sub")
        .expect("置ける")
        .expect("書く");
    assert!(
        sub.contains("\"/W/sub\": {\n      \"hasTrustDialogAccepted\": true\n    }"),
        "{sub}"
    );
    let odd = [
        "{",
        "",
        "[]",
        r#"{"projects": "x"}"#,
        r#"{"projects": {"/W": 1}}"#,
        r#"{"projects": {"/W": {"hasTrustDialogAccepted": "yes"}}}"#,
    ];
    for text in odd {
        assert!(trusted(Some(text), "/W").is_err(), "{text}");
    }
}
