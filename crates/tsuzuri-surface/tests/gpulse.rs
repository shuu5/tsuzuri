//! 行 g-pulse の歯: 読みの脈（fresh の Pulse）の決め方と、net が読みの途中の口を数える字と 2 つの board が脈を置く字の並び。
//! 1 つでも口が読みの途中なら上端の帯の最終の記録の横に短い脈を出し、読み終えてから PULSE_MS まで残す（要件 NFR2）。
#![cfg(test)]

use std::fmt::Debug;
use std::path::PathBuf;

use tsuzuri_surface::fresh::{PULSE_MS, Pulse, pulse_class};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 関数の本文（`fn name(` か `fn name<` から次の行頭の `fn `・`pub fn `・`async fn `・`pub async fn ` まで）。
fn function<'a>(text: &'a str, name: &str) -> &'a str {
    let start = [format!("fn {name}("), format!("fn {name}<")]
        .iter()
        .filter_map(|m| text.find(m.as_str()))
        .min()
        .unwrap_or_else(|| panic!("fn {name} が在る"));
    let rest = &text[start..];
    let end = ["\nfn ", "\npub fn ", "\nasync fn ", "\npub async fn "]
        .iter()
        .filter_map(|m| rest[1..].find(m).map(|i| i + 1))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

/// `text` で最初に出る `word` の位置。
fn at(text: &str, word: &str) -> usize {
    text.find(word)
        .unwrap_or_else(|| panic!("{word} が無い: {text}"))
}

/// 型の持つ trait（Pulse の Default の値を渡して組めることで見る）。
fn traits<T: Copy + Default + PartialEq + Eq + Debug>(_: T) {}

/// stylesheet の規則 `selector` の行。
fn rule<'a>(css: &'a str, selector: &str) -> &'a str {
    css.lines()
        .find(|l| l.trim_start().starts_with(&format!("{selector} {{")))
        .unwrap_or_else(|| panic!("stylesheet に規則 {selector} が無い"))
}

/// (1) 脈の class と残す間と、stylesheet の輪の規則。
#[test]
fn gpulse_class_and_length() {
    assert_eq!(PULSE_MS, 800u64);
    let class: fn(bool) -> &'static str = pulse_class;
    assert_eq!(class(true), "st st-run beat");
    assert_eq!(class(false), "st");
    let _set: fn(&mut Pulse, usize, u64) = Pulse::set;
    let _on: fn(&Pulse, u64) -> bool = Pulse::on;
    traits(Pulse::default());

    let css = read("style.css");
    assert!(
        rule(&css, ".st.beat::after").contains("animation-duration: .8s"),
        "{}",
        rule(&css, ".st.beat::after")
    );
    assert!(
        rule(&css, ".st.st-run::after").contains("animation: tz-pulse 1.6s ease-out infinite"),
        "{}",
        rule(&css, ".st.st-run::after")
    );
}

/// (2) 脈は読みの途中の間と、読みの途中の口が 0 になってから PULSE_MS より前まで出す。
#[test]
fn gpulse_on_while_busy_and_after() {
    let mut p = Pulse::default();
    assert!(!p.on(0));
    p.set(0, 50);
    assert!(!p.on(50), "0 のままの知らせは脈を出さない");

    p.set(1, 1000);
    assert!(p.on(1000));
    assert!(p.on(99999), "読みの途中は出し続ける");
    p.set(2, 1100);
    assert!(p.on(1100));
    p.set(0, 1200);
    assert!(p.on(1200));
    assert!(p.on(1999));
    assert!(!p.on(2000), "PULSE_MS ちょうどは出さない");

    p.set(0, 2500);
    assert!(!p.on(2500), "0 のままの知らせは終わりの時刻を動かさない");

    p.set(1, 3000);
    p.set(0, 3100);
    assert!(p.on(3899));
    assert!(!p.on(3900));
}

/// (3) net は読みの印を変えるたびに読みの途中の口を数え直し、数を signal で読ませる（字の並び）。
#[test]
fn gpulse_net_text() {
    let net = read("src/net.rs");
    assert!(
        net.contains("pub fn busy() -> ReadSignal<usize>"),
        "net に pub fn busy が無い"
    );

    let count = function(&net, "count_busy");
    for w in ["READS.with_borrow(", "WATCHES.with_borrow(", "signal.set(n)"] {
        assert!(count.contains(w), "count_busy に {w} が無い: {count}");
    }
    assert_eq!(count.matches(".busy()").count(), 2, "{count}");

    for name in [
        "read_flight",
        "watch_flight",
        "read",
        "read_path",
        "reload_if",
    ] {
        let body = function(&net, name);
        assert!(body.contains("count_busy()"), "{name} に count_busy() が無い: {body}");
    }

    let read_fn = function(&net, "read");
    let c = at(read_fn, "count_busy()");
    assert!(at(read_fn, "READS.with_borrow_mut(") < c, "{read_fn}");
    assert!(c < at(read_fn, "load_at(slot, 1)"), "{read_fn}");

    let path = function(&net, "read_path");
    let c = at(path, "count_busy()");
    assert!(at(path, "flight.renew()") < c, "{path}");
    assert!(c < at(path, "load_watch(slot, round, 1)"), "{path}");

    let reload = function(&net, "reload_if");
    let last = reload.rfind("count_busy()").expect("reload_if に count_busy()");
    assert!(at(reload, "WATCHES.with_borrow_mut(") < last, "{reload}");
}

/// (4) 脈は fresh の dom が描き、2 つの board は最終の記録の後で読み込み不良の印の前に 1 つ置く（字の並び）。
#[test]
fn gpulse_boards_text() {
    let fresh = read("src/fresh.rs");
    let dom = &fresh[at(&fresh, "mod dom {")..];
    let pulse = &dom[at(dom, "fn pulse(")..];
    for w in [
        "fn pulse() -> AnyView",
        "crate::net::busy()",
        ".set(n, now)",
        "now + PULSE_MS",
        "set_timeout(",
        "PULSE_MS",
        "pulse_class(",
        ".on(",
        "aria-hidden=\"true\"",
    ] {
        assert!(pulse.contains(w), "fresh の pulse に {w} が無い: {pulse}");
    }

    let board = read("src/board.rs");
    let top = function(&board, "top");
    assert_eq!(top.matches("fresh::pulse()").count(), 1, "{top}");
    let p = at(top, "fresh::pulse()");
    assert!(at(top, "title=title>{at}</span>") < p, "{top}");
    assert!(p < at(top, "fresh::mark()"), "{top}");

    let account = read("src/account/board.rs");
    let top = function(&account, "top");
    assert_eq!(top.matches("fresh::pulse()").count(), 1, "{top}");
    let p = at(top, "fresh::pulse()");
    assert!(at(top, "updated_chip()") < p, "{top}");
    assert!(p < at(top, "fresh::mark()"), "{top}");
}

/// (5) この file の歯の名はちょうど 5 で、どれも gpulse_ で始まり、残りの字は契約表の verify の語を含まない。
#[test]
fn gpulse_own_names_clean() {
    let text = read("tests/gpulse.rs");
    let mut names = Vec::new();
    let mut rest = text.as_str();
    // 属性の行と次の行の `fn `（この字の literal は逆斜線の escape で改行を持たず当たらない）。
    let mark = "\n#[test]\nfn ";
    while let Some(i) = rest.find(mark) {
        let after = &rest[i + mark.len()..];
        let end = after.find('(').expect("fn の名の終わり");
        names.push(after[..end].trim().to_string());
        rest = &after[end..];
    }
    assert_eq!(names.len(), 5, "{names:?}");
    let words = [words_front(), words_middle(), words_back()].concat();
    names_avoid(names, &words);
}

/// 契約表の verify の語の表の前の部分（44 語）。
fn words_front() -> &'static [&'static str] {
    &[
        "aaround_",
        "accept_",
        "account_",
        "acctcore_",
        "acctdoc_",
        "accthb_",
        "accthome_",
        "acctled_",
        "acctlook_",
        "acctpcore_",
        "acctproj_",
        "acctsess_",
        "acctwin_",
        "acctwire_",
        "afocus_",
        "aord_",
        "apop_",
        "askcard_",
        "athr_",
        "batchpanel_",
        "bhalf_",
        "board_min_",
        "bport_",
        "brand_",
        "btuck_",
        "cadopt_",
        "cgdom_",
        "cmark_",
        "contract_form_",
        "cround_",
        "csled_",
        "denv_",
        "ecache_",
        "flight_",
        "fmark_",
        "frame_",
        "fserve_",
        "fstop_",
        "gapspage_",
        "gbnote_",
        "gfresh_",
        "ghb_",
        "glabel_",
        "gnav_",
    ]
}

/// 契約表の verify の語の表の中の部分（44 語）。
fn words_middle() -> &'static [&'static str] {
    &[
        "gpill_",
        "graph_",
        "gsum_",
        "gtuck_",
        "gview_",
        "hacols_",
        "hbconf_",
        "hbpost_",
        "hbproc_",
        "hbroute_",
        "hcard_",
        "hcled_",
        "hcnx_",
        "hcproj_",
        "hcsess_",
        "hfig_",
        "hnunk_",
        "hook_",
        "hruling_",
        "hsblock_",
        "hsderive_",
        "hspage_",
        "hsym_",
        "iclose_",
        "ilink_",
        "kcli_",
        "klink_",
        "launch_",
        "lcard_",
        "ledgerblock_",
        "lhome_",
        "lspark_",
        "mapview_",
        "mkeys_",
        "mlink_",
        "mstore_",
        "mtree_",
        "nact_",
        "nbatch_",
        "ncard_",
        "nextstep_",
        "nodepage_",
        "nstall_",
        "nsum_",
    ]
}

/// 契約表の verify の語の表の後の部分（44 語）。
fn words_back() -> &'static [&'static str] {
    &[
        "nsumw_",
        "ntime_",
        "nxact_",
        "parts_",
        "pclosed_",
        "pfold_",
        "pgz_",
        "pipe_",
        "plimit_",
        "pmore_",
        "pquest_",
        "project_",
        "ptitle_",
        "punmap_",
        "pwhole_",
        "qblock_",
        "qgate_",
        "qkey_",
        "question_",
        "relay_",
        "rhold_",
        "runsdoc_",
        "rvk_",
        "saxis_",
        "sclosed_",
        "seatblock_",
        "seatcard_",
        "server_",
        "sesplit_",
        "shb_",
        "skeleton_",
        "smore_",
        "stage_",
        "stats_",
        "stcli_",
        "steady_",
        "sxaxis_",
        "ticker_",
        "tipx_",
        "topbar_",
        "tz_",
        "urpanel_",
        "uword_",
        "wstrip_",
    ]
}

/// 歯の名の残りの字が契約表の verify の語を含まないことを見る。
fn names_avoid(names: Vec<String>, words: &[&str]) {
    for name in &names {
        let rest = name
            .strip_prefix("gpulse_")
            .unwrap_or_else(|| panic!("{name} が gpulse_ で始まらない"));
        for w in words {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
