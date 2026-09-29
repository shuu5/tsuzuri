//! 行 i-stage-own の歯（接頭辞 tgown_）: home の右の列の block「表示先」（project board・自分の project だけ）の
//! 置き場と畳める段・読みの応答から中身・自分の行と字・要求の本文・送った後の 1 行・読みが知らせの鎖に乗らないこと。

use std::path::PathBuf;

use tsuzuri_boundary::stagecall;
use tsuzuri_contract::stage::{self as contract, OpenRequest, OwnTarget, StageTargets};
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::{self, PageId};
use tsuzuri_surface::project::{Body, Module, stage};
use tsuzuri_surface::vocab::vocab;

fn read(rel: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(dir.join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 読みの電文（二重引用符つきの 1 行）。
const READ: &str = r#"{"project":"proj-a","default":"term-a","overrides":[{"project":"proj-b","name":"term-b"}],"projects":[{"project":"proj-a","effective":{"name":"term-a","origin":"default","listed":true}},{"project":"proj-b","effective":{"name":"term-b","origin":"override","listed":false}}],"names":["term-a"]}"#;

fn targets() -> StageTargets {
    wire::decode(READ).expect("節の電文は StageTargets に読める")
}

/// (5) home の右の列は orch と stage・block の枠と鍵と語・口の path・畳める段の記録の呼びの 1 行。
#[test]
fn tgown_block_placed_and_folded() {
    let columns: Vec<Vec<&str>> = frame::page(PageId::Home)
        .columns
        .iter()
        .map(|c| c.blocks.iter().map(|b| b.id).collect())
        .collect();
    assert_eq!(columns[1], vec!["orch", "stage"]);
    assert_eq!(stage::BLOCK.id, "stage");
    assert_eq!(stage::BLOCK.heading, "stage_target");
    assert_eq!(stage::BLOCK.class, "panel fold mvp");
    assert_eq!(stage::FOLDS, ["stage:block"]);
    assert!(stage::PATHS.is_empty());
    assert_eq!(stage::PATH, contract::PATH);
    assert_eq!(stage::OPEN_PATH, contract::OPEN_PATH);
    assert_eq!(stage::PATH, "/api/stage/target");
    assert_eq!(stage::OPEN_PATH, "/api/stage/open");
    assert_eq!(vocab().label("stage_target"), "表示先");
    let module = Module::ALL
        .into_iter()
        .find(|m| m.name() == "stage")
        .expect("Module の ALL に stage が在る");
    assert_eq!(module.block(), stage::BLOCK);
    let text = read("src/project/stage.rs");
    let call = "let (open, record) = fold(\"stage:block\".to_string(), || false);";
    assert!(text.contains(call), "畳める段の記録の呼びの 1 行が無い");
}

/// (1) 読みの応答から中身: 200 と電文は Filled（前後の空白と改行も）・ほかは理由の 1 行。
#[test]
fn tgown_content_reads_only_wire() {
    assert_eq!(stage::content(Some((200, READ))), Body::Filled(targets()));
    let padded = format!("\n  {READ}  \n");
    assert_eq!(
        stage::content(Some((200, padded.as_str()))),
        Body::Filled(targets())
    );
    assert_eq!(stage::content(None), Body::Unmeasured(stage::UNREACHED));
    for (status, text, want) in [
        (200, "project proj-a term-a", stage::BAD_READ),
        (502, "tz-failed", stage::TZ_DOWN),
        (502, "bad-reply", stage::BAD_READ),
        (403, "origin", stage::REFUSED_READ),
    ] {
        assert_eq!(
            stage::content(Some((status, text))),
            Body::Unmeasured(want),
            "{status} {text}"
        );
    }
    for reason in [
        stage::UNREACHED,
        stage::BAD_READ,
        stage::TZ_DOWN,
        stage::REFUSED_READ,
    ] {
        assert!(!reason.is_empty() && !reason.contains('\n'), "{reason}");
    }
    // server の語は境界の同じ名の定数と同じ字。
    assert_eq!(stage::TZ_FAILED, stagecall::TZ_FAILED);
    assert_eq!(stage::BAD_REPLY, stagecall::BAD_REPLY);
    assert_eq!(stage::BAD_BODY, stagecall::BAD_BODY);
    assert_eq!(stage::BAD_NAME, stagecall::BAD_NAME);
    assert_eq!(stage::NO_PROJECT, stagecall::NO_PROJECT);
}

/// (2) 自分の行と字: 既定・上書き（層 A に無い）・行が無い。
#[test]
fn tgown_own_row_and_words() {
    let a = stage::own(&targets());
    assert_eq!(a.project, "proj-a");
    assert!(!a.overridden());
    assert_eq!(a.picked().as_deref(), Some("term-a"));
    assert_eq!(a.clear_label(), "既定に戻す（term-a）");
    assert_eq!(
        stage::effective_line(a.effective.as_ref()),
        "term-a · 既定"
    );

    let mut t = targets();
    t.project = "proj-b".to_string();
    let b = stage::own(&t);
    assert!(b.overridden());
    assert_eq!(b.picked().as_deref(), Some("term-a"));
    assert_eq!(
        stage::effective_line(b.effective.as_ref()),
        "term-b · 上書き · 層 A に無い"
    );

    t.project = "proj-z".to_string();
    t.default = None;
    t.names = Vec::new();
    let z = stage::own(&t);
    assert_eq!(z.effective, None);
    assert!(!z.overridden());
    assert_eq!(z.picked(), None);
    assert_eq!(z.clear_label(), "既定に戻す");
    assert_eq!(
        stage::effective_line(None),
        "表示先なし（席の目と URL に落ちる）"
    );

    let names = ["term-a".to_string(), "term-b".to_string()];
    assert_eq!(
        stage::names_line(&names),
        "PC の一覧（層 A）は term-a・term-b（器の設定・ここでは変えない）"
    );
    assert_eq!(stage::origin_word(contract::Origin::Default), "既定");
    assert_eq!(stage::origin_word(contract::Origin::Override), "上書き");
}

/// (3) 要求の本文は契約の型の字で、読み戻すと同じ値。
#[test]
fn tgown_bodies_match_contract() {
    assert_eq!(stage::own_body(Some("term-b")), r#"{"to":"term-b"}"#);
    assert_eq!(stage::own_body(None), r#"{"to":null}"#);
    assert_eq!(stage::open_body(None), r#"{"project":null}"#);
    assert_eq!(stage::open_body(Some("proj-b")), r#"{"project":"proj-b"}"#);
    let clear: OwnTarget = wire::decode(&stage::own_body(None)).expect("契約の OwnTarget");
    assert_eq!(clear.to, None);
    let put: OwnTarget = wire::decode(&stage::own_body(Some("term-b"))).expect("契約の OwnTarget");
    assert_eq!(put.to.as_deref(), Some("term-b"));
    let open: OpenRequest =
        wire::decode(&stage::open_body(Some("proj-b"))).expect("契約の OpenRequest");
    assert_eq!(open.project.as_deref(), Some("proj-b"));
}

/// (4) 送った後の 1 行: 届かない・200・断り。
#[test]
fn tgown_outcome_lines() {
    let refused = |o: &stage::Outcome| matches!(o, stage::Outcome::Refused(_));
    let none = stage::outcome(None);
    assert_eq!(none, stage::Outcome::Refused("口に届かない".to_string()));
    let said = "project proj-a の表示先を term-b にした（上書き）";
    let done = stage::outcome(Some((200, format!("{said}\n").as_str())));
    assert_eq!(done, stage::Outcome::Done(said.to_string()));
    assert_eq!(done.line(), said);
    let empty = stage::outcome(Some((200, " \n")));
    assert_eq!(empty, stage::Outcome::Done("済み".to_string()));
    assert_eq!(empty.line(), "済み");
    for (status, text, want) in [
        (403, "origin", "別の origin からの頼みを断った"),
        (400, "bad-body", "要求の本文が読めない"),
        (400, "bad-name", "端末の名の形が違う"),
        (404, "no-project", "project が群の宣言に無い"),
        (
            502,
            "tz-failed",
            "tz の口が失敗した（撃てないか rc が 0 でないか時間内に返らない）",
        ),
    ] {
        let o = stage::outcome(Some((status, text)));
        assert!(refused(&o), "{status} {text}");
        assert_eq!(o.line(), format!("口が断った・{want}（{status}）"));
    }
    let big = stage::outcome(Some((413, "too-large")));
    assert!(refused(&big));
    assert_eq!(big.line(), "口が断った・413 too-large");
}

/// 関数の本文（頭の行から、次の行の頭の閉じ波括弧まで）。
fn function<'a>(text: &'a str, head: &str) -> &'a str {
    let start = text
        .find(head)
        .unwrap_or_else(|| panic!("{head} が net.rs に在る"));
    let rest = &text[start..];
    let end = rest
        .find("\n}\n")
        .unwrap_or_else(|| panic!("{head} の終わり"));
    &rest[..end]
}

/// (6) block は読みを 1 回だけ撃つ字で組み・知らせの鎖（登録・古さ・接続）に乗らない。
#[test]
fn tgown_reads_off_the_chain() {
    let text = read("src/project/stage.rs");
    let once = |pat: &str, n: usize| {
        assert_eq!(text.matches(pat).count(), n, "stage.rs の {pat} の数");
    };
    once("crate::net::get(PATH)", 1);
    once("crate::net::post(path, body)", 1);
    once("load(f);", 2);
    once("            if now {\n                load(f);\n            }", 1);
    once("            if reread {\n                load(f);\n            }", 1);
    once("send(f, PATH, own_body(Some(&to)), true)", 1);
    once("send(f, PATH, own_body(None), true)", 1);
    once("send(f, OPEN_PATH, open_body(None), false)", 1);
    for word in ["net::read(", "read_path(", "reload_all"] {
        once(word, 0);
    }
    assert!(!read("src/view.rs").contains("stage"), "view.rs が stage を含む");
    let net = read("src/net.rs");
    let get = function(&net, "pub async fn get(");
    assert!(get.contains("window.fetch_with_str(path)"), "{get}");
    for word in ["note(", "READS", "WATCHES", "connect(", "change("] {
        assert!(!get.contains(word), "get が {word} を含む: {get}");
    }
}
