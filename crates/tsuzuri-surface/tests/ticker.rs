//! 便 g-tick の歯: 1 秒の時計（net の ticker）と、session の表の経過の字と退避までの残り秒を今で組み直す関数。
//! net の ticker と DOM の書き直しは wasm の target のときだけなので、字の歯で見る（組めることは xtask の surface-build）。
//! fixture は着地済みの tests/fixtures/account/acct-doc.json（読むだけ）。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::session::{NONE_MARK, elapsed_at, grace_left, row};

const AT: u64 = 1_790_510_400;

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

/// session.rs の DOM の部分（字「mod dom {」より後）。
fn dom_text() -> String {
    let text = read("src/account/session.rs");
    let at = text
        .find("mod dom {")
        .expect("session.rs に mod dom { が在る");
    text[at..].to_string()
}

/// (1) 経過の字は今と電文の at の大きい方から since を引いた秒（負は 0）で、since が無ければ「―」。
#[test]
fn ticker_elapsed_uses_later_of_now_and_at() {
    let since = Some(1_790_505_060);
    assert_eq!(elapsed_at(since, AT, AT), "1h");
    assert_eq!(elapsed_at(since, AT, AT + 3600), "2h");
    assert_eq!(elapsed_at(since, AT, AT - 1000), "1h");
    assert_eq!(elapsed_at(Some(AT + 10), AT, AT), "0s");
    assert_eq!(elapsed_at(None, AT, AT), NONE_MARK);
    assert_eq!(elapsed_at(None, AT, AT + 3600), "―");
}

/// (2) 残り秒は終わる時刻から今と電文の at の大きい方を引き、0 で止める（今が at より前なら at から引く）。
#[test]
fn ticker_grace_counts_down_to_zero() {
    let until = AT + 1101;
    assert_eq!(grace_left(until, AT, AT), 1101);
    assert_eq!(grace_left(until, AT, AT + 60), 1041);
    assert_eq!(grace_left(until, AT, AT + 1101), 0);
    assert_eq!(grace_left(until, AT, AT + 5000), 0);
    assert_eq!(grace_left(until, AT, AT - 100), 1101);
}

/// (3) 表の行は電文の同じ行の since を持ち、経過の字は電文の at を今とした字と同じ。
#[test]
fn ticker_row_keeps_since_and_read_time_text() {
    let doc = fixture();
    assert_eq!(doc.at, AT);
    assert_eq!(doc.sessions.len(), 4);
    let sinces: Vec<Option<u64>> = doc.sessions.iter().map(|l| l.since).collect();
    assert_eq!(
        sinces,
        vec![
            Some(1_790_505_060),
            Some(1_790_508_600),
            Some(1_790_508_900),
            None
        ]
    );
    let mut texts = Vec::new();
    for (i, line) in doc.sessions.iter().enumerate() {
        let r = row(&doc, i);
        assert_eq!(r.since, line.since, "行 {}", i + 1);
        assert_eq!(
            r.elapsed,
            elapsed_at(line.since, doc.at, doc.at),
            "行 {}",
            i + 1
        );
        texts.push(r.elapsed);
    }
    assert_eq!(texts, vec!["1h", "30m", "25m", "―"]);
}

/// (4) net.rs は 1 秒の時計を 1 本持つ（TICK_MS は 1000・interval の呼びは 1 か所）。
#[test]
fn ticker_net_holds_one_interval() {
    let net = read("src/net.rs");
    assert!(net.contains("pub const TICK_MS: u64 = 1000;"));
    assert!(net.contains("pub fn ticker() -> ReadSignal<EpochSecs>"));
    assert!(net.contains("Duration::from_millis(TICK_MS)"));
    assert_eq!(net.matches("set_interval(").count(), 1);
}

/// (5) session の DOM は時計の signal を受け、経過の字と残り秒を今で組み直す。
#[test]
fn ticker_dom_reads_the_clock() {
    let dom = dom_text();
    for needle in ["crate::net::ticker()", "elapsed_at(", "grace_left("] {
        assert!(dom.contains(needle), "DOM の部分に {needle} が在る");
    }
}

/// (7) この file の歯の名は ticker_ で始まり、残りの字は着地済みの行の filter の語を含まない。
#[test]
fn ticker_names_avoid_other_filter_words() {
    const WORDS: [&str; 65] = [
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
        "hfig_",
        "pmore_",
        "lspark_",
        "apop_",
        "brand_",
        "runsdoc_",
        "nbatch_",
    ];
    let text = read("tests/ticker.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<String> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .map(|w| {
            let name = w[1].strip_prefix("fn ").expect("#[test] の次の行は fn");
            name[..name.find('(').expect("fn の名の後に ( が在る")].to_string()
        })
        .collect();
    assert_eq!(names.len(), 6);
    for name in &names {
        let tail = name
            .strip_prefix("ticker_")
            .unwrap_or_else(|| panic!("{name} は ticker_ で始まる"));
        for word in WORDS {
            assert!(!tail.contains(word), "{name} は {word} を含まない");
        }
    }
}
