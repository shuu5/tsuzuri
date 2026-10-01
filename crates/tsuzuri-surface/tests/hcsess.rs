//! 便 h-cards-sess の歯: session の表の session の欄の hover の card（見本の seat・run・proj の枝と鍵の選び方）と、
//! DOM の部分の字。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::{AccountDoc, SessionLine};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::SeatState;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::cards::orch_card;
use tsuzuri_surface::account::session::{
    PROJ_SHOWN, RUN_SRC, SessRow, Sort, proj_card, row, run_card, sess_card, table,
};
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::hover::Card;

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

/// card の 4 行と詳しくを比べる。
fn check(card: &Card, (title, kind): (&str, &str), value: &str, src: &str, more: &[&str]) {
    assert_eq!(card.title, title);
    assert_eq!(card.kind, kind, "{title} の種類");
    assert_eq!(card.value, value, "{title} の値");
    assert_eq!(card.src, src, "{title} の出所");
    assert_eq!(card.more, more, "{title} の詳しく");
}

/// (1) run の行の card。
#[test]
fn hcsess_run_card_rules() {
    assert_eq!(RUN_SRC, "fleet/events.jsonl · RunStage");
    let f: fn(&SessionLine, EpochSecs) -> Card = run_card;
    let doc = fixture();
    assert_eq!(doc.at, 1790510400);
    let line = &doc.sessions[1];
    check(
        &f(line, doc.at),
        ("proj-a.3-20260927T113000Z", "pipeline · proj-a"),
        "Running · 動いている",
        "fleet/events.jsonl · RunStage",
        &["口座 acct-1", "◷ 20:30 JST から"],
    );

    let with = |g: &dyn Fn(&mut SessionLine)| {
        let mut l = line.clone();
        g(&mut l);
        run_card(&l, doc.at)
    };
    assert_eq!(
        with(&|l| l.account = None).more,
        ["口座 測れていない", "◷ 20:30 JST から"]
    );
    assert_eq!(with(&|l| l.since = None).more, ["口座 acct-1"]);
    assert_eq!(with(&|l| l.stage = None).value, "― · 動いている");
    assert_eq!(
        with(&|l| l.state = SeatState::Limit).value,
        "Running · 限度で止まっている"
    );
    assert_eq!(
        with(&|l| l.since = Some(1790427900)).more[1],
        "◷ 09-26 22:05 JST から"
    );
}

/// (2) 席の無い project の行の card。
#[test]
fn hcsess_proj_card_rules() {
    assert_eq!(PROJ_SHOWN, 5);
    let f: fn(&AccountDoc, &str) -> Card = proj_card;
    let doc = fixture();
    check(
        &f(&doc, "proj-c"),
        ("proj-c", "project · Tier1"),
        "session 1 · orchestrator 1",
        "proj-c · state dir",
        &["proj-c-orch ?", "まだ無い"],
    );
    check(
        &proj_card(&doc, "proj-a"),
        ("proj-a", "project · Tier1"),
        "session 2 · orchestrator 1",
        "proj-a · state dir",
        &[
            "proj-a-orch acct-1",
            "proj-a.3-20260927T113000Z acct-1",
            "board :40001",
        ],
    );
    check(
        &proj_card(&doc, "proj-z"),
        ("proj-z", "project · ―"),
        "session なし",
        "proj-z · state dir",
        &["まだ無い"],
    );

    // 詳しくは初めの PROJ_SHOWN 行だけ。
    let mut many = doc.clone();
    for i in 1..=6 {
        let mut l = doc.sessions[1].clone();
        l.name = format!("run-{i}");
        many.sessions.push(l);
    }
    let card = proj_card(&many, "proj-a");
    assert_eq!(card.value, "session 8 · orchestrator 1");
    assert_eq!(
        card.more,
        [
            "proj-a-orch acct-1",
            "proj-a.3-20260927T113000Z acct-1",
            "run-1 acct-1",
            "run-2 acct-1",
            "run-3 acct-1",
            "board :40001",
        ]
    );

    // 名の無い行は数えない・board が無ければまだ無い。
    let mut bare = doc.clone();
    bare.projects[0].board = None;
    bare.sessions[0].name = String::new();
    let card = proj_card(&bare, "proj-a");
    assert_eq!(card.value, "session 1 · orchestrator 0");
    assert_eq!(
        card.more,
        ["proj-a.3-20260927T113000Z acct-1", "まだ無い"]
    );
}

/// (3) 鍵の選び方と、表の行が card を持つ。
#[test]
fn hcsess_rows_carry_cards() {
    let f: fn(&AccountDoc, usize) -> Card = sess_card;
    let doc = fixture();
    assert_eq!(doc.sessions.len(), 4);
    let a = orch_card(&doc, &doc.projects[0]).expect("proj-a は席を持つ");
    let b = orch_card(&doc, &doc.projects[1]).expect("proj-b は席を持つ");
    assert_eq!(a.title, "proj-a-orch");
    assert_eq!(b.title, "proj-b-orch");
    assert_eq!(f(&doc, 0), a);
    assert_eq!(f(&doc, 1), run_card(&doc.sessions[1], doc.at));
    assert_eq!(f(&doc, 2), b);
    assert_eq!(f(&doc, 3), proj_card(&doc, "proj-c"));

    let mut blank = doc.clone();
    blank.sessions[0].name = String::new();
    assert_eq!(sess_card(&blank, 0), proj_card(&blank, "proj-a"));

    let mut away = doc.clone();
    away.sessions[2].project = "proj-z".to_string();
    assert_eq!(sess_card(&away, 2), proj_card(&away, "proj-z"));

    let mut unseated = doc.clone();
    unseated.projects[1].seat = Reading::Unknown;
    assert_eq!(sess_card(&unseated, 2), proj_card(&unseated, "proj-b"));

    rows_in_tables(doc);
}

/// 行ごとの card と、並べ方ごとの表の行の card。
fn rows_in_tables(doc: AccountDoc) {
    for i in 0..doc.sessions.len() {
        let r: SessRow = row(&doc, i);
        assert_eq!(r.card, sess_card(&doc, i), "位置 {i} の行の card");
        for line in [&r.card.title, &r.card.kind, &r.card.value, &r.card.src] {
            assert!(line.chars().count() <= 36, "位置 {i} の行 {line} が 36 字を超える");
        }
    }
    for sort in Sort::ALL {
        let t = table(&doc, sort);
        let rows: Vec<&SessRow> = t.groups.iter().flat_map(|g| g.rows.iter()).collect();
        assert_eq!(rows.len(), doc.sessions.len(), "{sort:?}");
        for r in rows {
            assert_eq!(r.card, sess_card(&doc, r.index), "{sort:?} の位置 {}", r.index);
        }
    }
}

/// session.rs の字「mod dom {」より後（DOM の部分）。
fn dom_part(text: &str) -> String {
    let at = text.find("mod dom {").expect("session.rs に mod dom が在る");
    text[at + "mod dom {".len()..].to_string()
}

/// DOM の部分の fn の本体（宣言の字から、次の行頭 4 空白の fn か pub fn の宣言の前まで・無ければ終わりまで）。
fn fn_body(dom: &str, name: &str) -> String {
    let start = dom
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("DOM の部分に fn {name} が無い"));
    let rest = &dom[start..];
    let first = rest.find('\n').map_or(rest.len(), |i| i + 1);
    let mut end = rest.len();
    let mut at = first;
    for line in rest[first..].split_inclusive('\n') {
        if line.starts_with("    fn ") || line.starts_with("    pub fn ") {
            end = at;
            break;
        }
        at += line.len();
    }
    rest[..end].to_string()
}

fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// (4) DOM の部分が session の欄に card を付ける。
#[test]
fn hcsess_dom_wiring() {
    let text = read("src/account/session.rs");
    let dom = dom_part(&text);
    assert!(dom.contains("widgets::hover"), "DOM の部分に字 widgets::hover が無い");
    assert!(!text.contains("fn attach("), "session.rs に自前の fn attach が在る");

    let body = squeeze(&fn_body(&dom, "row_view"));
    let want = "use:attach=row.card.clone()>{session}</div>";
    let hits: Vec<usize> = body.match_indices(want).map(|(i, _)| i).collect();
    assert_eq!(hits.len(), 1, "row_view に {want} がちょうど 1 つ無い");
    let open = body[..hits[0]].rfind('<').expect("tag の開きが在る");
    let tag = &body[open..hits[0]];
    assert!(tag.starts_with("<div"), "session の欄の tag {tag} が <div で始まらない");
    assert!(tag.contains("tabindex=\"0\""), "session の欄の tag {tag} に tabindex が無い");

    // 行の前提: account の board.rs の頁が card の層を持つ。
    let board = read("src/account/board.rs");
    assert!(board.contains("provide_context(HoverCtx::default())"));
    assert!(board.contains("<CardLayer/>"));
}

/// (5) card が引く語の鍵は語の辞書に在って語が空でない。
#[test]
fn hcsess_vocab_keys() {
    for key in [
        "role:pipeline",
        "st_unknown",
        "st_run",
        "st_wait",
        "st_limit",
        "st_silent",
        "seat_none",
        "not_yet",
    ] {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }
}

/// (7) この file の歯の名は hcsess_ で始まり、残りの字は filter の語を含まない。
#[test]
fn hcsess_names_clean() {
    const WORDS: [&str; 92] = [
        "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
        "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
        "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
        "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
        "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
        "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
        "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_",
        "gnav_", "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_",
        "ticker_", "hfig_", "pmore_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_",
        "kcli_", "qgate_", "nsum_", "hcard_", "ntime_", "ptitle_", "ncard_", "hsym_", "smore_",
        "lcard_", "aaround_", "hruling_", "sxaxis_", "plimit_", "bport_", "fmark_", "fstop_",
        "fserve_", "nsumw_", "cadopt_", "tipx_", "hcproj_", "sesplit_", "mstore_", "athr_",
        "qblock_",
    ];
    let text = read("tests/hcsess.rs");
    let lines: Vec<&str> = text.lines().collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() != "#[test]" {
            continue;
        }
        let decl = lines[i + 1..]
            .iter()
            .find(|l| l.trim_start().starts_with("fn "))
            .expect("test の属性の後に fn が在る");
        let name = decl.trim_start()["fn ".len()..]
            .split('(')
            .next()
            .unwrap_or_default()
            .to_string();
        names.push(name);
    }
    assert_eq!(names.len(), 6, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("hcsess_")
            .unwrap_or_else(|| panic!("歯の名 {name} が hcsess_ で始まらない"));
        for w in WORDS {
            assert!(!rest.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
