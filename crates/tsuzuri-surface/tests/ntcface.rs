//! 席からの知らせの block の歯（接頭辞 ntc_・行 i-11 の完了の条件 (5)〜(8)）。
//! project board の block notice は自分の project の 1 つを、account board の block notices は全部を同じ電文から読む。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::notice::{self as contract, Notice, Notices};
use tsuzuri_contract::surface::ChangeKind;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::{self, Tab, notices};
use tsuzuri_surface::frame::{self, PageId};
use tsuzuri_surface::project::notice::{self, Line};
use tsuzuri_surface::project::{Body, NOT_READ};
use tsuzuri_surface::view::{Fetched, path_kinds};
use tsuzuri_surface::vocab::vocab;

/// 電文を組んだ時刻（知らせの時刻との差が 20 時間の内）。
const AT: u64 = 1_790_000_460;

/// 2 日後の組んだ時刻（差が 20 時間を越える）。
const LATER: u64 = AT + 2 * 86_400;

const MAP_URL: &str = "http://srv-a.tailnet.invalid:8121/?page=map";
const ASK_URL: &str = "https://srv-a.tailnet.invalid:8120/?page=ask";

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn notice_of(at: u64, project: &str, title: &str, url: &str) -> Notice {
    Notice {
        at,
        project: project.to_string(),
        title: title.to_string(),
        url: url.to_string(),
    }
}

/// 節の 2 つの知らせ（proj-b が新しい）。
fn latest(kiri_url: &str) -> Vec<Notice> {
    vec![
        notice_of(1_790_000_400, "proj-b", "地図を見て", MAP_URL),
        notice_of(1_790_000_300, "proj-kiri", "問いを見て", kiri_url),
    ]
}

/// 電文の本文。
fn body(at: u64, latest: Vec<Notice>, unread: &[&str]) -> Fetched {
    let doc = Notices {
        at,
        project: "proj-kiri".to_string(),
        latest,
        unread: unread.iter().map(|s| s.to_string()).collect(),
    };
    Fetched::Body(wire::encode(&doc).expect("電文"))
}

fn line(project: &str, title: &str, href: Option<&str>, when: &str) -> Line {
    Line {
        project: project.to_string(),
        title: title.to_string(),
        href: href.map(str::to_string),
        when: when.to_string(),
    }
}

/// (5) project board の block notice の中身と link の先。
#[test]
fn ntc_own_block() {
    for (fetched, want) in [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, notice::REASON),
        (Fetched::Body("not json".to_string()), notice::BAD_BODY),
    ] {
        assert_eq!(notice::content(&fetched), Body::Unmeasured(want), "{fetched:?}");
    }
    let reasons = [NOT_READ, notice::REASON, notice::BAD_BODY, notice::OWN_UNREAD];
    for (i, a) in reasons.iter().enumerate() {
        assert!(!a.trim().is_empty() && !a.contains('\n'));
        for b in &reasons[i + 1..] {
            assert_ne!(a, b, "理由が重なる");
        }
    }

    let own = line("proj-kiri", "問いを見て", Some(ASK_URL), "23:18 JST");
    assert_eq!(
        notice::content(&body(AT, latest(ASK_URL), &[])),
        Body::Filled(own.clone())
    );
    assert_eq!(
        notice::content(&body(LATER, latest(ASK_URL), &[])),
        Body::Filled(line("proj-kiri", "問いを見て", Some(ASK_URL), "09-21 23:18 JST"))
    );
    // 自分の記録が無ければ空・自分の名が読めなければ測れていない・ほかの名だけが読めなければ自分の 1 行。
    let others = vec![latest(ASK_URL)[0].clone()];
    assert_eq!(
        notice::content(&body(AT, others, &[])),
        Body::Empty(notice::NONE_LINE)
    );
    assert_eq!(notice::content(&body(AT, vec![], &[])), Body::Empty(notice::NONE_LINE));
    assert_eq!(notice::NONE_LINE, "席からの知らせはまだ無い");
    assert_eq!(
        notice::content(&body(AT, vec![latest(ASK_URL)[0].clone()], &["proj-kiri"])),
        Body::Unmeasured(notice::OWN_UNREAD)
    );
    assert_eq!(
        notice::content(&body(AT, latest(ASK_URL), &["proj-y", "proj-z"])),
        Body::Filled(own)
    );

    let hrefs: Vec<Option<String>> = [
        MAP_URL,
        ASK_URL,
        "javascript:alert(1)",
        "file:///etc/passwd",
        "//srv-a.tailnet.invalid/",
        "",
    ]
    .iter()
    .map(|u| notice::href(u))
    .collect();
    assert_eq!(
        hrefs,
        [
            Some(MAP_URL.to_string()),
            Some(ASK_URL.to_string()),
            None,
            None,
            None,
            None
        ]
    );
}

/// (6) account board の block notices の中身と読めない記録の行。
#[test]
fn ntc_all_block() {
    for (fetched, want) in [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, notice::REASON),
        (Fetched::Body(String::new()), notice::BAD_BODY),
    ] {
        assert_eq!(notices::content(&fetched), Body::Unmeasured(want), "{fetched:?}");
    }
    assert_eq!(
        notices::content(&body(AT, vec![], &[])),
        Body::Empty(notices::NONE_LINE)
    );
    assert_eq!(notices::NONE_LINE, "どの project の席からも知らせはまだ無い");
    assert_eq!(
        notices::content(&body(AT, latest("ftp://x"), &["proj-z"])),
        Body::Filled(notices::Rows {
            lines: vec![
                line("proj-b", "地図を見て", Some(MAP_URL), "23:20 JST"),
                line("proj-kiri", "問いを見て", None, "23:18 JST"),
            ],
            unread: vec!["proj-z".to_string()],
        })
    );
    assert_eq!(
        notices::content(&body(AT, vec![], &["proj-z"])),
        Body::Filled(notices::Rows {
            lines: vec![],
            unread: vec!["proj-z".to_string()],
        })
    );
    assert_eq!(notices::unread_line(&[]), None);
    assert_eq!(
        notices::unread_line(&["proj-y".to_string(), "proj-z".to_string()]),
        Some("知らせの記録が読めない project: proj-y・proj-z".to_string())
    );
}

/// (7) 置き場と語と口の path と変化の種類。
#[test]
fn ntc_blocks_placed() {
    let home = frame::page(PageId::Home);
    let first = home.columns[0].blocks[0];
    assert_eq!((first.id, first.heading, first.class), ("notice", "notice", "panel"));
    assert_eq!(first, notice::BLOCK);
    let acct = account::page(Tab::Home);
    assert_eq!(acct.rows[0].blocks, [notices::BLOCK]);
    let b = notices::BLOCK;
    assert_eq!((b.id, b.heading, b.class), ("notices", "notices", "panel"));
    assert_eq!(vocab().label("notice"), "席からの知らせ");
    assert_eq!(vocab().label("notices"), "各 project の席からの知らせ");
    assert_eq!(notice::PATH, contract::PATH);
    assert_eq!(notices::PATH, contract::PATH);
    assert_eq!(notice::PATHS, [contract::PATH]);
    assert_eq!(path_kinds(contract::PATH), [ChangeKind::Notice]);
    assert_eq!(ChangeKind::ALL.last(), Some(&ChangeKind::Notice));
}

/// (8) 2 つの block は窓を前に出す・起こす・保存する・書く字を持たず、account の link は project の窓の名を使う
/// （link の窓を用意するのは board の open_named だけ）。
#[test]
fn ntc_blocks_raise_nothing() {
    let own = read("src/project/notice.rs");
    let all = read("src/account/notices.rs");
    for (name, text) in [("project/notice.rs", &own), ("account/notices.rs", &all)] {
        for word in [
            "focus(",
            "window.open",
            "open(",
            "bringToFront",
            "createTarget",
            "press(",
            "localStorage",
            "post(",
        ] {
            assert!(!text.contains(word), "{name} が {word} を含む");
        }
    }
    assert_eq!(all.matches("win_name(&l.project)").count(), 1);
}
