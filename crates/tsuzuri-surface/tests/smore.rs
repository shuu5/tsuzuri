//! 行 g-seat-more の歯: project board の block「orchestrator と口座」の詳しくの段に置く
//! 出所の 1 行と doctor の席の行（経験者向けの 1 行の字数に畳む）と、詳しくの初めの開きを mode から取る字。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::seat::{SeatCard, SeatState};
use tsuzuri_contract::wire;
use tsuzuri_surface::account::home::{EXPERT_CHARS, wrap_words};
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::seat::{MORE_SRC, content, more, seat, seat_line};
use tsuzuri_surface::view::Fetched;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の 5 組（組の名 → 電文の型）。
fn fixture() -> BTreeMap<String, SeatCard> {
    wire::decode(&read("../../tests/fixtures/surface/seat-card.json"))
        .expect("fixture の組が電文として読める")
}

fn card(name: &str) -> SeatCard {
    fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"))
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

const L1: &str = "seat: target=tsuzuri-orch account=acct-4 model=opus";

/// 節の表の組ごとの doctor の行。
fn want(name: &str) -> Vec<String> {
    match name {
        "run" | "limit" => s(&[L1, "heartbeat=on tick=healthy"]),
        "wait" => s(&[L1, "heartbeat=off tick=stale"]),
        "silent" => s(&["seat: target=tsuzuri-orch account=? model=? heartbeat=?", "tick=?"]),
        "unknown" => s(&[
            "seat: target=tsuzuri-orch account=acct-4 model=? heartbeat=?",
            "tick=?",
        ]),
        other => panic!("知らない組 {other}"),
    }
}

fn chars(t: &str) -> usize {
    t.chars().count()
}

/// (1)(2) doctor の席の行の形と、電文の欄の読み方。
#[test]
fn smore_seat_line_rules() {
    assert_eq!(
        MORE_SRC,
        "席の dir の state.jsonl と tick-last・器の doctor と seat tick status と fleet usage --show の出力"
    );
    let run = card("run");
    let line = "seat: target=tsuzuri-orch account=acct-4 model=opus heartbeat=on tick=healthy";
    assert_eq!(seat_line(&run), line);

    let mut c = run.clone();
    c.state = SeatState::Limit;
    c.since = None;
    c.group = Reading::Unknown;
    c.usage = Reading::Unknown;
    c.spans = Reading::Unknown;
    c.moves = Reading::Unknown;
    assert_eq!(seat_line(&c), line);

    let mut c = run.clone();
    c.account = None;
    assert!(seat_line(&c).contains(" account=? "), "{}", seat_line(&c));
    let mut c = run.clone();
    c.model = Some("sonnet".into());
    assert!(seat_line(&c).contains(" model=sonnet "), "{}", seat_line(&c));
    let mut c = run.clone();
    c.heartbeat = Reading::Unknown;
    assert!(seat_line(&c).contains(" heartbeat=? "), "{}", seat_line(&c));
    let mut c = run.clone();
    c.heartbeat = Reading::Known(false);
    assert!(seat_line(&c).contains(" heartbeat=off "), "{}", seat_line(&c));
    let mut c = run.clone();
    c.tick_healthy = Reading::Known(false);
    assert!(seat_line(&c).ends_with(" tick=stale"), "{}", seat_line(&c));
    let mut c = run.clone();
    c.tick_healthy = Reading::Unknown;
    assert!(seat_line(&c).ends_with(" tick=?"), "{}", seat_line(&c));

    for (_, c) in fixture() {
        let l = seat_line(&c);
        assert!(l.starts_with("seat: target="), "{l}");
        for no in ["role=", "anchor=", "pane="] {
            assert!(!l.contains(no), "{l} に {no} が在る");
        }
        let keys: Vec<&str> = l
            .split(' ')
            .skip(1)
            .map(|w| w.split('=').next().unwrap_or(""))
            .collect();
        assert_eq!(keys, ["target", "account", "model", "heartbeat", "tick"]);
    }
}

/// (3) fixture の 5 組の doctor の行と、more・seat・content が同じ行を持つこと。
#[test]
fn smore_lines_on_fixture() {
    let fx = fixture();
    assert_eq!(fx.len(), 5);
    for (name, c) in &fx {
        let m = more(c);
        assert_eq!(m.doctor, want(name), "組 {name}");
        assert_eq!(m.doctor, wrap_words(&seat_line(c), EXPERT_CHARS));
        assert_eq!(m.target, c.target);
        assert_eq!(seat(c).more, m);
        let text = wire::encode(c).expect("電文");
        match content(&Fetched::Body(text), c.at) {
            Body::Filled(v) => assert_eq!(v.more, m),
            other => panic!("組 {name} が中身を出さない: {other:?}"),
        }
    }
    assert_eq!(chars(&want("unknown")[0]), 60);
    assert_eq!(chars(L1), 51);
    assert_eq!(chars(&want("silent")[0]), 55);
}

/// (4) どの行も経験者向けの字数以下で、長い語は字数ごとに切る。
#[test]
fn smore_lines_within_budget() {
    let mut cards: Vec<SeatCard> = fixture().into_values().collect();
    let x70 = "x".repeat(70);
    let mut long = card("run");
    long.target = x70.clone();
    let mut mid = card("run");
    mid.target = "proj-with-a-rather-long-name:0.1".into();
    cards.push(long.clone());
    cards.push(mid.clone());
    for c in &cards {
        for l in more(c).doctor {
            assert!(chars(&l) <= EXPERT_CHARS, "{l} が {EXPERT_CHARS} 字を超える");
        }
    }
    assert_eq!(
        more(&long).doctor,
        vec![
            "seat:".to_string(),
            format!("target={}", "x".repeat(53)),
            format!("{} account=acct-4 model=opus heartbeat=on", "x".repeat(17)),
            "tick=healthy".to_string(),
        ]
    );
    assert_eq!(chars(&more(&long).doctor[1]), 60);
    assert_eq!(chars(&more(&long).doctor[2]), 56);
    assert_eq!(
        more(&mid).doctor,
        s(&[
            "seat: target=proj-with-a-rather-long-name:0.1 account=acct-4",
            "model=opus heartbeat=on tick=healthy",
        ])
    );
    assert_eq!(chars(&more(&mid).doctor[0]), 60);
    for c in &cards {
        let line = seat_line(c);
        if line.split(' ').all(|w| chars(w) <= EXPERT_CHARS) {
            assert_eq!(more(c).doctor.join(" "), line);
        }
    }
}

/// (5) 経験者向けの 1 行の字数は規則の行 R-19 の数と同じ。
#[test]
fn smore_budget_matches_r19() {
    let rules = read("../../design-intent/rules.yaml");
    let line = rules
        .lines()
        .find(|l| l.contains("id: R-19,"))
        .expect("規則の行 R-19 が在る");
    let key = "経験者向けの 1 行は";
    let rest = &line[line.find(key).expect("経験者向けの 1 行の字") + key.len()..];
    let num: String = rest[..rest.find('字').expect("字数の単位")]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert_eq!(num.parse::<usize>().expect("数"), EXPERT_CHARS);
    assert_eq!(EXPERT_CHARS, 60);
}

/// (6) 詳しくの段の DOM（src/project_dom/seat.rs）の字と、頁が HelpCtx を context に置く前提。
#[test]
fn smore_dom_text() {
    let dom = read("src/project_dom/seat.rs");
    let start = dom.find("fn more_view(").expect("more_view が在る");
    let len = dom[start..].find("\n}").expect("more_view の閉じ");
    let body = &dom[start..start + len];
    for w in ["fold(\"seat:more\"", "use_context::<HelpCtx>()", "shows_internal("] {
        assert!(body.contains(w), "more_view に {w} が無い");
    }
    assert!(!body.contains("|| false"), "more_view の初めの値がいつも閉じ");
    let end = body.find("</details>").expect("more_view に </details> が在る");
    for w in ["class=\"gm1 src\"", "class=\"gm1 int xo\"", "MORE_SRC", "<code>", "doctor"] {
        let at = body.find(w).unwrap_or_else(|| panic!("more_view に {w} が無い"));
        assert!(at < end, "{w} が </details> の後に在る");
    }
    let board = read("src/board.rs");
    assert!(board.contains("let help = HelpCtx {"));
    assert!(board.contains("provide_context(help);"));
}

/// (7) 詳しくの段の class は stylesheet の規則の名で、xo は経験者の mode のときだけ見える。
#[test]
fn smore_css_rules() {
    let css = read("style.css");
    for c in ["gmore", "gm", "gm1", "src", "int", "xo"] {
        let dotted = format!(".{c}");
        let found = css.match_indices(&dotted).any(|(i, _)| {
            css[i + dotted.len()..]
                .chars()
                .next()
                .is_none_or(|n| !(n.is_ascii_alphanumeric() || n == '-' || n == '_'))
        });
        assert!(found, "style.css に .{c} が無い");
    }
    assert!(css.contains("body.mode-expert .gmore .xo"));
}

/// filter の語（着地済みの行の接頭辞と、並べて走る行の接頭辞）。
const FILTER: [&str; 78] = [
    "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
    "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
    "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
    "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
    "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
    "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_", "server_",
    "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_", "gnav_", "mkeys_",
    "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_", "ticker_", "hfig_", "pmore_",
    "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_", "kcli_", "hcard_", "ntime_", "ptitle_",
    "ncard_", "hsym_", "lcard_", "aaround_", "hruling_", "sxaxis_", "plimit_", "bport_",
];

/// (9) この file の歯の名は smore_ で始まり、残りの字は filter の語を含まない。
#[test]
fn smore_own_names_clean() {
    let me = read("tests/smore.rs");
    let lines: Vec<&str> = me.lines().collect();
    let mut n = 0;
    for (i, l) in lines.iter().enumerate() {
        if l.trim() != "#[test]" {
            continue;
        }
        let f = lines[i + 1].trim();
        let name = f
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("test の属性の次が fn でない: {f}"));
        let rest = name
            .strip_prefix("smore_")
            .unwrap_or_else(|| panic!("{name} が smore_ で始まらない"));
        for w in FILTER {
            assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
        }
        n += 1;
    }
    assert_eq!(n, 7);
}
