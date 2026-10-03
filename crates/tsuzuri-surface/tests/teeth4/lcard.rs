//! 行 g-ledger-card の歯: burndown の図の hover の card（題・純減と closed/日・open の始めと終わり・出所と時点・
//! 1 日おきの詳しく）・DOM の字（一覧の見出しの図の card・行 g-ledger-trim で台帳の block の lmid から移した）・語の鍵・歯の名。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::stats::LedgerStats;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::ledger::{BURN_SRC, burn_card};
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::hover::{Card, ROW_CHARS};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// fixture の組（組の名 → 電文）。
fn set(name: &str) -> LedgerStats {
    let mut sets: BTreeMap<String, LedgerStats> =
        wire::decode(&read("../../tests/fixtures/surface/ledger-stats.json"))
            .expect("fixture の組が電文として読める");
    sets.remove(name)
        .unwrap_or_else(|| panic!("fixture の組 {name}"))
}

/// 実物の口の本文の写し（字そのままと、鍵 known の下の指標）。
fn real() -> (String, LedgerStats) {
    let text = read("../../tests/fixtures/surface/metrics-body.json");
    let mut raw: BTreeMap<String, LedgerStats> =
        wire::decode(&text).expect("写しは鍵 known の下に指標を持つ");
    let inner = raw.remove("known").expect("鍵 known");
    (text, inner)
}

fn card(title: &str, kind: &str, value: &str, src: &str, more: &[&str]) -> Card {
    Card {
        title: title.to_string(),
        kind: kind.to_string(),
        value: value.to_string(),
        src: src.to_string(),
        more: more.iter().map(|m| m.to_string()).collect(),
    }
}

const FILLED_MORE: [&str; 7] = [
    "09-29 open 11 · +1 / −0",
    "10-01 open 9 · +3 / −2",
    "10-03 open 7 · +0 / −2",
    "10-05 open 12 · +4 / −1",
    "10-07 open 8 · +1 / −2",
    "10-09 open 9 · +2 / −1",
    "10-11 open 5 · +0 / −2",
];

fn filled_card() -> Card {
    card(
        "burndown",
        "↓−2 24h · ↓−4 7d · closed/日 1.7",
        "open 10 → 5（14 日）",
        "bd list --all の task · 時点 09:00 JST",
        &FILLED_MORE,
    )
}

/// (2) fixture の filled と empty と実物の本文の写しの card は節の表の値で、4 行はどれも 36 字以下で切られない。
#[test]
fn lcard_burn_on_fixture() {
    let filled = burn_card(&set("filled"));
    assert_eq!(filled, filled_card());

    let empty_more: Vec<String> = FILLED_MORE
        .iter()
        .map(|m| format!("{} open 0 · +0 / −0", &m[..5]))
        .collect();
    let empty = burn_card(&set("empty"));
    assert_eq!(
        empty,
        Card {
            title: "burndown".to_string(),
            kind: "→0 24h · →0 7d · closed/日 0.0".to_string(),
            value: "open 0 → 0（14 日）".to_string(),
            src: "bd list --all の task · 時点 09:00 JST".to_string(),
            more: empty_more,
        }
    );

    let (_, inner) = real();
    let got = burn_card(&inner);
    assert_eq!(
        got,
        card(
            "burndown",
            "↑+2 24h · ↑+2 7d · closed/日 2.7",
            "open 0 → 2（14 日）",
            "bd list --all の task · 時点 20:37 JST",
            &[
                "09-15 open 0 · +0 / −0",
                "09-17 open 0 · +0 / −0",
                "09-19 open 0 · +0 / −0",
                "09-21 open 0 · +0 / −0",
                "09-23 open 0 · +0 / −0",
                "09-25 open 0 · +0 / −0",
                "09-27 open 2 · +21 / −19",
            ],
        )
    );

    for c in [&filled, &empty, &got] {
        let plain = [&c.title, &c.kind, &c.value, &c.src];
        for ((_, row), text) in c.rows().iter().zip(plain) {
            assert!(text.chars().count() <= ROW_CHARS, "{text}");
            assert_eq!(row, text, "Card の rows が切らない");
        }
    }
    assert_eq!(filled.src.chars().count(), 35);
    assert_eq!(filled.kind.chars().count(), 31);
}

/// (1)(3) 規則: 出所の定数・日の数による値と詳しく（最後の日を必ず含む 1 日おき）・純減の矢印。ほかの欄は変わらない。
#[test]
fn lcard_burn_rules() {
    assert_eq!(BURN_SRC, "bd list --all の task");
    let base = set("filled");
    let full = burn_card(&base);
    assert_eq!(full.title, label("l_burn"));
    assert!(full.src.starts_with(BURN_SRC));

    let mut four = base.clone();
    four.days.truncate(4);
    let c = burn_card(&four);
    assert_eq!(c.value, "open 10 → 9（4 日）");
    assert_eq!(c.more, vec!["09-29 open 11 · +1 / −0", "10-01 open 9 · +3 / −2"]);
    assert_eq!((c.title, c.kind, c.src), (full.title.clone(), full.kind.clone(), full.src.clone()));

    let mut three = base.clone();
    three.days.truncate(3);
    let c = burn_card(&three);
    assert_eq!(c.value, "open 10 → 8（3 日）");
    assert_eq!(c.more, vec!["09-28 open 10 · +2 / −1", "09-30 open 8 · +0 / −3"]);

    let mut none = base.clone();
    none.days.clear();
    let c = burn_card(&none);
    assert_eq!(c.value, "open ―");
    assert!(c.more.is_empty());
    assert_eq!((c.title, c.kind, c.src), (full.title.clone(), full.kind.clone(), full.src.clone()));

    net_drop_arrows(base, full);
}

/// 純減の矢印は種類の行だけを替え、ほかの欄は変わらない。
fn net_drop_arrows(base: LedgerStats, full: Card) {
    let mut flat = base.clone();
    flat.net_drop_24h = 0;
    flat.net_drop_7d = 0;
    let c = burn_card(&flat);
    assert_eq!(c.kind, "→0 24h · →0 7d · closed/日 1.7");
    assert_eq!(
        (c.title, c.value, c.src, c.more),
        (full.title.clone(), full.value.clone(), full.src.clone(), full.more.clone())
    );

    let mut up = base.clone();
    up.net_drop_24h = 3;
    let c = burn_card(&up);
    assert_eq!(c.kind, "↑+3 24h · ↓−4 7d · closed/日 1.7");
    assert_eq!(
        (c.title, c.value, c.src, c.more),
        (full.title, full.value, full.src, full.more)
    );
}

/// (5) DOM の字: 一覧の見出しの図の tag に card と tabindex・card は指標の電文の burn_card・board.rs の card の層。
#[test]
fn lcard_dom_wiring() {
    let src = read("src/ledgerlist.rs");
    let dom = &src[src.find("mod dom {").expect("mod dom")..];
    let at = dom.find("<span class=\"ll-spark\"").expect("図の tag");
    let tag = &dom[at..at + dom[at..].find('>').expect("tag の終わり")];
    assert!(tag.contains("use:attach_some=card"), "{tag}");
    assert!(tag.contains("tabindex=\"0\""), "{tag}");
    assert!(dom.contains("stats(m).ok().map(|s| burn_card(&s))"));
    assert!(dom.contains("hover::attach_some"));
    let ledger = read("src/project/ledger.rs");
    assert!(!ledger.contains("class=\"lmid\""), "台帳の block に lmid が残る");

    let board = read("src/board.rs");
    assert!(board.contains("provide_context(HoverCtx::default())"));
    assert!(board.contains("<CardLayer/>"));
}

/// (6) 語の鍵 l_burn と l_rate は辞書に在り、語は burndown と closed/日。
#[test]
fn lcard_vocab_keys() {
    for (key, word) in [("l_burn", "burndown"), ("l_rate", "closed/日")] {
        assert!(vocab().term(key).is_some(), "鍵 {key}");
        assert_eq!(label(key), word);
    }
}

/// 着地済みの行と並べて走る行の verify の filter の語。
const FILTERS: [&str; 78] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hbconf_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hook_",
    "hsblock_",
    "hsderive_",
    "hspage_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "mlink_",
    "gnav_",
    "mkeys_",
    "klink_",
    "ilink_",
    "afocus_",
    "nxact_",
    "urpanel_",
    "saxis_",
    "ticker_",
    "hfig_",
    "pmore_",
    "lspark_",
    "apop_",
    "brand_",
    "runsdoc_",
    "nbatch_",
    "kcli_",
    "hcard_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
];

/// (8) この file の test の fn の名はどれも lcard_ で始まり、残りの字は filter の語を含まない。
#[test]
fn lcard_own_names_clean() {
    let text = read("tests/teeth4/lcard.rs");
    let lines: Vec<&str> = text.lines().collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() != "#[test]" {
            continue;
        }
        let next = lines[i + 1].trim();
        let name = next
            .strip_prefix("fn ")
            .and_then(|r| r.split('(').next())
            .unwrap_or_else(|| panic!("test の属性の次が fn でない: {next}"));
        names.push(name.to_string());
    }
    assert_eq!(names.len(), 5, "{names:?}");
    for name in &names {
        let rest = name
            .strip_prefix("lcard_")
            .unwrap_or_else(|| panic!("{name} が lcard_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
