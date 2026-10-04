//! 行 g-pipe-cards の歯: pipeline の細い札（判断の記録 ADR-27 決定 (6)(7)・見本 board-v2 の cardHTML と cardLine）の
//! Queued の注意の秒が規則の行 R-37 の値であることと、段ごとの要の 1 行と、短い題とその代わりと、要の 1 行の材料（待つ相手と起票）と、
//! 札が押しで吹き出しを開き hover の card を付けない DOM の字。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{Ci, PipelineCard, Reading, Stage};
use tsuzuri_contract::case::{CaseLinks, CasePart};
use tsuzuri_contract::ledger::{BeadFact, BeadId, LedgerRow};
use tsuzuri_surface::project::pipeline::{CLOSED_TAG, kcard, short_title};
use tsuzuri_surface::view::clock_short;
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::keyline::{
    KeyLine, LINE_CLASSES, LINE_KEYS, LineSrc, QUEUED_WARN_S, WARN_MARK, WHY_CHARS, key_line,
    line_src, queued_warn,
};
use tsuzuri_surface::widgets::pop::Src;

const NOW: EpochSecs = 1_790_000_000;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn id(s: &str) -> BeadId {
    BeadId::new(s).unwrap_or_else(|e| panic!("{s}: {e:?}"))
}

fn card(x: &str, stage: Stage, since: EpochSecs) -> PipelineCard {
    PipelineCard {
        contract: id(x),
        runs: 2,
        stage,
        reason: None,
        account: Some("acct-a".to_string()),
        since: Some(since),
        ci: None,
    }
}

fn line(wait: Option<&str>, created: Option<EpochSecs>) -> LineSrc {
    LineSrc {
        card: card("a", Stage::Running, NOW),
        wait: wait.map(str::to_string),
        created,
    }
}

/// 札の電文を替えた材料で要の 1 行を組む。
fn kline(card: &PipelineCard, line: &LineSrc, now: EpochSecs) -> KeyLine {
    let src = LineSrc {
        card: card.clone(),
        ..line.clone()
    };
    key_line(&src, now)
}

fn kl(text: String, class: &'static str) -> KeyLine {
    KeyLine { text, class }
}

/// Queued の注意の秒は 1800 で、規則の行 R-37 の value の字の分の数に 60 を掛けた値。越えた時だけ注意（ちょうどは付けない）・
/// 列に入った時刻が無い札と Queued でない札は付けない。
#[test]
fn bvcols_queued_warn_is_rule_r37() {
    assert_eq!(QUEUED_WARN_S, 1_800);
    let rules = read("../../design-intent/rules.yaml");
    let row = rules
        .lines()
        .find(|l| l.contains("id: R-37,"))
        .expect("rules の file に行 R-37 が在る");
    let mins: u64 = row
        .split_once(" 分 を越えたら")
        .and_then(|(head, _)| head.rsplit(' ').next())
        .and_then(|n| n.parse().ok())
        .expect("R-37 の分の数");
    assert_eq!(mins * 60, QUEUED_WARN_S, "{row}");
    assert!(!queued_warn(&card("a", Stage::Queued, NOW - 1_800), NOW));
    assert!(queued_warn(&card("a", Stage::Queued, NOW - 1_801), NOW));
    let mut none = card("a", Stage::Queued, NOW);
    none.since = None;
    assert!(!queued_warn(&none, NOW));
    assert!(!queued_warn(&card("a", Stage::Blocked, NOW - 9_999), NOW));
}

/// 待つ段の要の 1 行（見本の cardLine）と色の class: Blocked は待つ相手と待ちの長さ・Queued は列に在る長さと起票からの経過
/// （30 分越えは注意の印と色）。
#[test]
fn bvcols_key_line_per_stage() {
    let [wait, _, queue, created, ..] = LINE_KEYS;
    let [plain, warn, ..] = LINE_CLASSES;
    let t = NOW - 600;
    let one = line(Some("t-5 待ち"), None);
    assert_eq!(
        kline(&card("a", Stage::Blocked, t), &one, NOW),
        kl("t-5 待ち · 10m".to_string(), plain)
    );
    assert_eq!(
        kline(&card("a", Stage::Blocked, t), &line(None, None), NOW),
        kl(format!("{} · 10m", label(wait)), plain)
    );
    let made = line(None, Some(NOW - 7_200));
    assert_eq!(
        kline(&card("a", Stage::Queued, t), &made, NOW),
        kl(
            format!("{} 10m · {} 2h", label(queue), label(created)),
            plain
        )
    );
    assert_eq!(
        kline(
            &card("a", Stage::Queued, NOW - 1_860),
            &line(None, None),
            NOW
        ),
        kl(
            format!("{WARN_MARK} {} 31m · {} ―", label(queue), label(created)),
            warn
        )
    );
}

/// 歯 `bvcols_key_line_run_stop_land` の続き: 止まりの理由を切る字数 `WHY_CHARS` は札の値の行と同じ 20 で、20 字を越える
/// 理由は頭の 19 字と切りの印 … にし、ちょうど 20 字は切らない。Questioned は理由が在っても問いの語を出す。run の回は札の電文の
/// 欄 runs の値（2 でない回でも見る）。
fn stop_cut_and_runs(made: &LineSrc, t: EpochSecs) {
    let [_, _, _, _, ask, _] = LINE_KEYS;
    let [_, _, stop, asked] = LINE_CLASSES;
    assert_eq!(WHY_CHARS, 20);
    let mut long = card("a", Stage::Failed, t);
    long.runs = 3;
    long.reason = Some("止".repeat(25));
    assert_eq!(
        kline(&long, made, NOW),
        kl(format!("{}… · 10m · run 3", "止".repeat(19)), stop)
    );
    long.reason = Some("止".repeat(20));
    assert_eq!(
        kline(&long, made, NOW).text,
        format!("{} · 10m · run 3", "止".repeat(20))
    );
    let mut halt = card("a", Stage::Stopped, t);
    halt.reason = Some("止".repeat(21));
    assert_eq!(
        kline(&halt, made, NOW),
        kl(format!("{}… · 10m · run 2", "止".repeat(19)), stop)
    );
    let mut gate = card("a", Stage::Gated, t);
    gate.runs = 5;
    assert_eq!(kline(&gate, made, NOW).text, "gate · run 5 · 10m · acct-a");
    let mut running = card("a", Stage::Running, t);
    running.runs = 6;
    assert_eq!(kline(&running, made, NOW).text, "run 6 · 10m · acct-a");
    let mut asking = card("a", Stage::Questioned, t);
    asking.runs = 4;
    asking.reason = Some("止".repeat(25));
    assert_eq!(
        kline(&asking, made, NOW),
        kl(format!("{} · 10m · run 4", label(ask)), asked)
    );
}

/// 走る段と止まりと着地の要の 1 行（見本の cardLine）と色の class・閉じた（着地せず）の札は段の字と経過・止まりの理由の切りと
/// run の回は `stop_cut_and_runs` で見る。
#[test]
fn bvcols_key_line_run_stop_land() {
    let [_, _, _, _, ask, landed] = LINE_KEYS;
    let [plain, _, stop, asked] = LINE_CLASSES;
    let t = NOW - 600;
    let made = line(None, Some(NOW - 7_200));
    assert_eq!(
        kline(&card("a", Stage::Running, t), &made, NOW),
        kl("run 2 · 10m · acct-a".to_string(), plain)
    );
    assert_eq!(
        kline(&card("a", Stage::Gated, t), &made, NOW),
        kl("gate · run 2 · 10m · acct-a".to_string(), plain)
    );
    let mut failed = card("a", Stage::Failed, t);
    failed.reason = Some("verify が赤".to_string());
    assert_eq!(
        kline(&failed, &made, NOW),
        kl("verify が赤 · 10m · run 2".to_string(), stop)
    );
    assert_eq!(
        kline(&card("a", Stage::Stopped, t), &made, NOW),
        kl("Stopped · 10m · run 2".to_string(), stop)
    );
    assert_eq!(
        kline(&card("a", Stage::Questioned, t), &made, NOW),
        kl(format!("{} · 10m · run 2", label(ask)), asked)
    );
    stop_cut_and_runs(&made, t);
    let mut land = card("a", Stage::Landed, t);
    land.ci = Some(Ci::Success);
    assert_eq!(
        kline(&land, &made, NOW),
        kl(
            format!(
                "{} {} · CI {}",
                clock_short(t, NOW),
                label(landed),
                label("ci_success")
            ),
            plain
        )
    );
    land.reason = Some(format!("{CLOSED_TAG}x"));
    assert_eq!(
        kline(&land, &made, NOW).text,
        "閉じた（着地せず） · 10m".to_string()
    );
    for k in LINE_KEYS {
        assert!(vocab().term(k).is_some(), "{k} が語の辞書に無い");
    }
}

/// 短い題は bead の事実の short、事実に無ければ台帳の題を 36 字に切った字、題も無ければ id。
#[test]
fn bvcols_short_title_or_fallback() {
    let c = card("t-1", Stage::Running, NOW);
    let rows = [LedgerRow {
        id: id("t-1"),
        kind: "task".to_string(),
        title: "題".repeat(40),
        status: "open".to_string(),
        updated_at: NOW,
        parent: None,
        labels: Vec::new(),
    }];
    let k = kcard(&c, &rows, NOW);
    let fact = BeadFact {
        id: id("t-1"),
        created_at: None,
        short: "短い題".to_string(),
        short_set: true,
        blocks: Vec::new(),
    };
    let facts = [fact];
    assert_eq!(short_title(&Reading::Known(&facts), &k), "短い題");
    let title = short_title(&Reading::Unknown, &k);
    assert_eq!(title, k.title.clone().unwrap_or_default());
    assert_eq!(title, "題".repeat(36));
    assert_eq!(
        short_title(&Reading::Known(&[]), &kcard(&c, &[], NOW)),
        "t-1"
    );
}

fn part(phase: &str, reason: &str, on: &[&str]) -> CasePart {
    CasePart {
        part: "contract".to_string(),
        id: "t-1".to_string(),
        phase: phase.to_string(),
        turn: "vessel".to_string(),
        since: None,
        reason: Some(reason.to_string()),
        closed: false,
        links: CaseLinks {
            on: on.iter().map(|s| s.to_string()).collect(),
            runs: Vec::new(),
        },
    }
}

fn src_of<'a>(facts: &'a [BeadFact], cards: &'a [PipelineCard], parts: &'a [CasePart]) -> Src<'a> {
    Src {
        facts: Reading::Known(facts),
        rows: Reading::Unknown,
        cards,
        parts: Reading::Known(parts),
        graph: None,
    }
}

/// 要の 1 行の材料: Blocked の頭は吹き出しと同じ待つ相手（1 つなら id と待ち・2 つ以上なら数と本待ち・承認待ちは持ち主の承認・
/// 分からなければ待ち）で、ほかの段は無い。起票の時刻は bead の事実から。
#[test]
fn bvcols_lines_from_pop_wait() {
    let cards = [card("t-1", Stage::Blocked, NOW)];
    let facts = [BeadFact {
        id: id("t-1"),
        created_at: Some(NOW - 60),
        short: "s".to_string(),
        short_set: false,
        blocks: Vec::new(),
    }];
    let one = [part("contract-queued", "dependency", &["t-5"])];
    let two = [part("contract-queued", "overlap", &["t-5", "t-6"])];
    let run = [part("contract-running", "run-blocked", &[])];
    let [wait, waits, ..] = LINE_KEYS;
    assert_eq!(
        line_src(&cards[0], &src_of(&facts, &cards, &one)),
        LineSrc {
            card: cards[0].clone(),
            ..line(Some(&format!("t-5 {}", label(wait))), Some(NOW - 60))
        }
    );
    assert_eq!(
        line_src(&cards[0], &src_of(&facts, &cards, &two)).wait,
        Some(format!("2 {}", label(waits)))
    );
    assert_eq!(
        line_src(&cards[0], &src_of(&facts, &cards, &run)).wait,
        Some(label("pw_approval"))
    );
    assert_eq!(
        line_src(&cards[0], &src_of(&facts, &cards, &[])).wait,
        Some(label(wait))
    );
    let q = [card("t-1", Stage::Queued, NOW)];
    let s = Src {
        facts: Reading::Unknown,
        rows: Reading::Unknown,
        cards: &q,
        parts: Reading::Unknown,
        graph: None,
    };
    assert_eq!(
        line_src(&q[0], &s),
        LineSrc {
            card: q[0].clone(),
            wait: None,
            created: None
        }
    );
}

/// 札は button で吹き出しの口の印を持ち押すと吹き出しを開き、hover の card と節点の頁への href を付けない。
/// 板の層は bead の事実と局面の出力の口を読んで短い題と要の 1 行の材料を置き、要の 1 行は 1 秒の時計（net の ticker・
/// `TICK_MS` は 1000）で書き直す。
#[test]
fn bvcols_click_not_hover() {
    let src = read("src/project/pipeline.rs");
    let dom = &src[src.find("mod dom {").expect("mod dom の字")..];
    for s in [
        "<button type=\"button\" class=card.class data-pop-card=card.id.clone() on:click=press>",
        "p.press(&id, Via::Card);",
        "crate::net::read(BEADS_PATH)",
        "crate::net::read(CASES_PATH)",
        "with_lines(content(p, l, now), &src)",
        "let tick = crate::net::ticker();",
        "let clock = move || tick.get();",
        "src.with_value(|s| key_line(s, clock()))",
    ] {
        assert!(dom.contains(s), "{s}");
    }
    let net = read("src/net.rs");
    assert!(net.contains("pub const TICK_MS: u64 = 1000;"));
    assert!(net.contains("Duration::from_millis(TICK_MS)"));
    for s in ["use:attach", "card_href(card", "<a class=card.class"] {
        assert!(!dom.contains(s), "{s} が残る");
    }
    let css = read("style.css");
    for s in [
        "button.kcard {",
        ".kcard .kl {",
        ".kcard .kl.klwarn {",
        ".kcard .kl.klstop {",
        ".kcard .kl.klask {",
    ] {
        assert!(css.contains(s), "{s}");
    }
}
