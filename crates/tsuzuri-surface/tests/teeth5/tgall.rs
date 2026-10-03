//! 行 i-stage-all の歯（接頭辞 tgall_）: account board の block「表示先」（HOME の最後の段）の
//! 表の行・要求の本文・置き場・読みと送りが知らせの鎖に乗らないこと。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::stage::{self as contract, Origin, StageTargets, Targets};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::{self, Tab, stage};

fn read(rel: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(dir.join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 表の電文（二重引用符つきの 1 行）。
const READ: &str = r#"{"project":"proj-a","default":"term-a","overrides":[{"project":"proj-b","name":"term-b"}],"projects":[{"project":"proj-a","effective":{"name":"term-a","origin":"default","listed":true}},{"project":"proj-b","effective":{"name":"term-b","origin":"override","listed":false}},{"project":"proj-c","effective":null}],"names":["term-a"]}"#;

fn targets() -> StageTargets {
    wire::decode(READ).expect("節の電文は StageTargets に読める")
}

/// (1) 表の行は電文の projects の順・電文の project の窓の要求は None・ほかは名。
#[test]
fn tgall_lines_follow_projects() {
    let lines = stage::lines(&targets());
    let want = vec![
        stage::Line {
            project: "proj-a".to_string(),
            name: Some("term-a".to_string()),
            origin: Some(Origin::Default),
            listed: true,
            open: None,
        },
        stage::Line {
            project: "proj-b".to_string(),
            name: Some("term-b".to_string()),
            origin: Some(Origin::Override),
            listed: false,
            open: Some("proj-b".to_string()),
        },
        stage::Line {
            project: "proj-c".to_string(),
            name: None,
            origin: None,
            listed: false,
            open: Some("proj-c".to_string()),
        },
    ];
    assert_eq!(lines, want);
    let overridden: Vec<bool> = lines.iter().map(stage::Line::overridden).collect();
    assert_eq!(overridden, [false, true, false]);
    let chips: Vec<Option<String>> = lines.iter().map(stage::Line::chip).collect();
    assert_eq!(
        chips,
        [
            Some("既定".to_string()),
            Some("上書き · 層 A に無い".to_string()),
            None
        ]
    );
    assert_eq!(lines[0].shown(), "term-a");
    assert_eq!(lines[2].shown(), "表示先なし（席の目と URL に落ちる）");
    let names = targets().names;
    assert_eq!(stage::picked(&lines[0], &names).as_deref(), Some("term-a"));
    assert_eq!(stage::picked(&lines[1], &names).as_deref(), Some("term-a"));
    assert_eq!(stage::picked(&lines[2], &[]), None);
    assert_eq!(stage::default_text(Some("term-a")), "term-a");
    assert_eq!(
        stage::default_text(None),
        "表示先なし（席の目と URL に落ちる）"
    );
}

/// (2) 要求の本文は節の字で、契約の型に読める。
#[test]
fn tgall_bodies_match_contract() {
    assert_eq!(stage::all_body("term-a"), r#"{"all":{"to":"term-a"}}"#);
    assert_eq!(
        stage::project_body("proj-b", Some("term-b")),
        r#"{"project":{"project":"proj-b","to":"term-b"}}"#
    );
    assert_eq!(
        stage::project_body("proj-b", None),
        r#"{"project":{"project":"proj-b","to":null}}"#
    );
    let all: Targets = wire::decode(&stage::all_body("term-a")).expect("契約の Targets");
    assert_eq!(
        all,
        Targets::All {
            to: "term-a".to_string()
        }
    );
    let clear: Targets =
        wire::decode(&stage::project_body("proj-b", None)).expect("契約の Targets");
    assert_eq!(
        clear,
        Targets::Project {
            project: "proj-b".to_string(),
            to: None
        }
    );
    let put: Targets =
        wire::decode(&stage::project_body("proj-b", Some("term-b"))).expect("契約の Targets");
    assert_eq!(
        put,
        Targets::Project {
            project: "proj-b".to_string(),
            to: Some("term-b".to_string())
        }
    );
}

/// (3) tab は 3 つのまま・HOME の最後の段の最後に stage・session と各 project の tab には無い。
#[test]
fn tgall_block_last_on_home() {
    assert_eq!(Tab::ALL.len(), 3);
    let home = account::page(Tab::Home);
    let last = home.rows.last().expect("HOME の段");
    assert_eq!(last.blocks.last(), Some(&stage::BLOCK));
    let ids = home.block_ids();
    assert_eq!(ids.iter().filter(|id| **id == "stage").count(), 1);
    for tab in [Tab::Session, Tab::Projects] {
        assert!(!account::page(tab).block_ids().contains(&"stage"));
    }
    assert_eq!(stage::BLOCK.id, "stage");
    assert_eq!(stage::BLOCK.heading, "stage_target");
    assert_eq!(stage::BLOCK.class, "panel fold mvp");
    assert_eq!(stage::ALL_PATH, contract::ALL_PATH);
    assert_eq!(stage::ALL_PATH, "/api/stage/targets");
}

/// (4) block は読みを開いた時の 1 回だけ撃つ字で組み・知らせの鎖に乗らない。
#[test]
fn tgall_reads_off_the_chain() {
    let text = read("src/account/stage.rs");
    let once = |pat: &str, n: usize| {
        assert_eq!(text.matches(pat).count(), n, "stage.rs の {pat} の数");
    };
    once("load(f);", 1);
    let toggle = "            if event_target::<web_sys::Element>(&ev).has_attribute(\"open\") {\n                load(f);\n            }";
    once(toggle, 1);
    once("send(f, ALL_PATH, all_body(&to), true)", 1);
    once(
        "send(f, ALL_PATH, project_body(&project, Some(&to)), true)",
        1,
    );
    once("send(f, ALL_PATH, project_body(&project, None), true)", 1);
    once("send(f, OPEN_PATH, open_body(open.as_deref()), false)", 1);
    once("net::", 0);
    once("reload_all", 0);
    let board = read("src/account/board.rs");
    let arm = "id if id == stage::BLOCK.id => stage::view(),";
    assert_eq!(board.matches(arm).count(), 1, "board.rs の腕");
}
