//! 行 g-seatpill の歯（接頭辞 gpill_）: header の席の pill の部品（frame の SEAT と seat_shown と snapshot の行）・
//! pill の状態と応答なしの経過・席の card の字・board と pill の DOM の字の並び・この file の歯の名。
//! fixture は tests/fixtures/surface/seat-card.json（読むだけ）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::{self, HEADER, HeaderPart, PageId, SEAT};
use tsuzuri_surface::project::seat::{self, LIMIT_LINE, MOVE_WAIT, NEXT_TARGET, REASON, hmd};
use tsuzuri_surface::project::{NO_CONTENT, NOT_READ};
use tsuzuri_surface::seatpill::{self, ETA_STYLE, Pill, RESUME};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::{label, vocab};
use tsuzuri_surface::widgets::hover::{Card, ROW_CHARS};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 関数の本文（`fn name(` から次の行頭の `fn `・`pub fn `・`async fn `・`pub async fn ` まで）。
fn function<'a>(text: &'a str, name: &str) -> &'a str {
    let start = text
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("fn {name} が在る"));
    let rest = &text[start..];
    let end = ["\nfn ", "\npub fn ", "\nasync fn ", "\npub async fn "]
        .iter()
        .filter_map(|m| rest[1..].find(m).map(|i| i + 1))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

/// fixture の 5 組（組の名 → 電文の型）。
fn fixture() -> BTreeMap<String, SeatCard> {
    wire::decode(&read("../../tests/fixtures/surface/seat-card.json"))
        .expect("fixture の組が電文として読める")
}

fn card_of(name: &str) -> SeatCard {
    fixture()
        .remove(name)
        .unwrap_or_else(|| panic!("fixture に組 {name} が在る"))
}

fn body(card: &SeatCard) -> Fetched {
    Fetched::Body(wire::encode(card).expect("電文"))
}

/// 型が Clone と PartialEq と Eq と Debug を持つ（組めることで見る）。
fn traits<T: Clone + PartialEq + Eq + std::fmt::Debug>(_: &T) {}

/// 3 つの読めない本文。
fn unread() -> [(Fetched, &'static str); 3] {
    [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, REASON),
        (Fetched::Body("x".to_string()), NO_CONTENT),
    ]
}

/// (1) frame の SEAT と seat_shown・語の辞書の鍵 seat・stylesheet の規則・pill と card の型。
#[test]
fn gpill_part_and_pages() {
    assert_eq!(
        SEAT,
        HeaderPart {
            part: "seat",
            key: "seat",
            class: "seatpill",
            items: &[],
        }
    );
    assert_eq!(label(SEAT.key), "orchestrator");
    assert_eq!(vocab().term("seat").expect("鍵 seat").label, "orchestrator");
    let css = read("style.css");
    for w in [".seatpill {", ".seatpill .who", ".seatpill .eta", "--st-silent"] {
        assert!(css.contains(w), "stylesheet に {w} が無い");
    }
    assert_eq!(ETA_STYLE, "color:var(--st-silent)");

    let pill_fn: fn(&Fetched, EpochSecs) -> Pill = seatpill::pill;
    let card_fn: fn(&Fetched, EpochSecs) -> Card = seatpill::card;
    let p = pill_fn(&Fetched::NotRead, 0);
    let made = Pill {
        state: p.state,
        eta: p.eta.clone(),
    };
    traits(&made);
    assert_eq!(made.clone(), p);
    let _ = card_fn(&Fetched::NotRead, 0);

    assert!(!frame::seat_shown(PageId::Home));
    let mut shown: Vec<&str> = PageId::ALL
        .into_iter()
        .filter(|p| frame::seat_shown(*p))
        .map(|p| p.id())
        .collect();
    shown.sort_unstable();
    assert_eq!(shown, ["ask", "gaps", "map", "node"]);

    let parts: Vec<&str> = HEADER.iter().map(|h| h.part).collect();
    assert_eq!(parts, ["brand", "nav", "updated", "mode"]);
}

/// (2) header_snapshot は snapshot の file と同じ 10 行で、seat の行は updated と mode の間に 1 つ。
#[test]
fn gpill_snapshot_line() {
    let got = frame::header_snapshot();
    assert!(
        got == read("tests/snapshots/header.json"),
        "header の値が snapshot と違う。今の値:\n{got}"
    );
    let lines: Vec<&str> = got.lines().collect();
    assert_eq!(lines.len(), 10, "{got}");
    assert!(lines[5].contains("\"part\": \"updated\""), "{}", lines[5]);
    assert_eq!(
        lines[6],
        "    {\"part\": \"seat\", \"key\": \"seat\", \"class\": \"seatpill\", \"items\": []},"
    );
    assert!(lines[7].contains("\"part\": \"mode\""), "{}", lines[7]);
    let seats = lines
        .iter()
        .filter(|l| l.contains("\"part\": \"seat\""))
        .count();
    assert_eq!(seats, 1);
}

/// (3) pill の状態は電文の state のまま、経過は応答なしで since が在るときだけ（今と at の大きい方から）。
#[test]
fn gpill_state_and_eta() {
    for (name, card) in fixture() {
        let p = seatpill::pill(&body(&card), 1_790_511_000);
        assert_eq!(p.state, name, "組 {name}");
        assert_eq!(p.eta, None, "組 {name}");
    }
    let mut silent = card_of("silent");
    silent.since = Some(1_790_509_800);
    let f = body(&silent);
    for (now, want) in [
        (1_790_510_400, "10m"),
        (1_790_510_300, "10m"),
        (1_790_513_400, "1h"),
        (1_790_514_300, "1h15"),
        (1_790_681_400, "47h40"),
        (1_790_682_600, "2d"),
    ] {
        let p = seatpill::pill(&f, now);
        assert_eq!(p.state, "silent");
        assert_eq!(p.eta.as_deref(), Some(want), "時刻 {now}");
    }
    for (f, _) in unread() {
        assert_eq!(
            seatpill::pill(&f, 1_790_511_000),
            Pill {
                state: "unknown",
                eta: None,
            }
        );
    }
}

/// (4) 席の card の字（題・種類・値・出所・詳しく）と、読めないときの理由の card。
#[test]
fn gpill_card_rows() {
    let at = 1_790_510_400;
    let kind = |key: &str, since: EpochSecs| format!("{} · ◷ {} から", label(key), hmd(since, at));
    let model = |m: &str| format!("model {m}");
    let title = "orchestrator · tsuzuri-orch";
    let src = "seat/tsuzuri-orch/state.jsonl ほか";
    let want = [
        (
            "run",
            kind("st_run", 1_790_502_600),
            "口座 acct-4 · tick healthy · hb on",
            vec![model("opus")],
        ),
        (
            "limit",
            kind("st_limit", 1_790_503_200),
            "口座 acct-4 · tick healthy · hb on",
            vec![
                model("opus"),
                LIMIT_LINE.to_string(),
                // fixture の電文は鍵 reopens を持たず、測れていない。
                format!("{RESUME} {}", label("st_unknown")),
                format!("{NEXT_TARGET} acct-6"),
            ],
        ),
        (
            "wait",
            kind("st_wait", 1_790_508_900),
            "口座 acct-4 · tick stale · hb off",
            vec![model("opus"), format!("{MOVE_WAIT} acct-4 → acct-5")],
        ),
        (
            "silent",
            label("st_silent"),
            "口座 ? · tick ? · hb ?",
            vec![model("?")],
        ),
        (
            "unknown",
            kind("st_unknown", 1_790_510_340),
            "口座 acct-4 · tick ? · hb ?",
            vec![model("?")],
        ),
    ];
    card_matches(at, title, src, want);
}

/// 5 組の席の card の字と行の字数、読めないときの理由の card と口の path を見る。
fn card_matches(
    at: EpochSecs,
    title: &str,
    src: &str,
    want: [(&str, String, &str, Vec<String>); 5],
) {
    for (name, kind, value, more) in want {
        let c = seatpill::card(&body(&card_of(name)), at);
        assert_eq!(
            c,
            Card {
                title: title.to_string(),
                kind,
                value: value.to_string(),
                src: src.to_string(),
                more,
            },
            "組 {name}"
        );
        for (_, row) in c.rows() {
            assert!(row.chars().count() <= ROW_CHARS, "組 {name}: {row}");
        }
        for text in [&c.title, &c.kind, &c.value, &c.src] {
            assert!(text.chars().count() <= ROW_CHARS, "組 {name}: {text}");
        }
    }
    assert_eq!(
        seatpill::card(&body(&card_of("run")), at).kind,
        format!("{} · ◷ {} から", label("st_run"), hmd(1_790_502_600, at))
    );
    for (f, reason) in unread() {
        assert_eq!(
            seatpill::card(&f, at),
            Card {
                title: "orchestrator".to_string(),
                kind: label("st_unknown"),
                value: reason.to_string(),
                src: "/api/seat".to_string(),
                more: Vec::new(),
            }
        );
    }
    assert_eq!(seat::PATH, "/api/seat");
}

/// (5) board の top は home のほかの頁で pill を読み込み不良の印の後に置き、pill の DOM は seatpill の dom が描く。
#[test]
fn gpill_board_text() {
    let board = read("src/board.rs");
    let top = function(&board, "top");
    for w in ["frame::seat_shown(", "seatpill::view"] {
        assert_eq!(top.matches(w).count(), 1, "board の top の {w}: {top}");
    }
    let mark = top.find("{fresh::mark()}").expect("top に {fresh::mark()} が在る");
    assert!(top[mark..].contains("{pill}"), "{top}");

    let text = read("src/seatpill.rs");
    let dom = &text[text.find("mod dom {").expect("seatpill に mod dom が在る")..];
    for w in [
        "pub fn view() -> AnyView",
        "crate::net::read(seat::PATH)",
        "crate::net::ticker()",
        "delegate()",
        ".show(",
        ".leave(",
        "class=SEAT.class",
        "label(SEAT.key)",
        "style=ETA_STYLE",
        "state_icon(",
        "tabindex=\"0\"",
        "data-seat=\"1\"",
        "class=\"who\"",
        "class=\"eta num\"",
    ] {
        assert!(dom.contains(w), "seatpill の dom に {w} が無い");
    }
    assert!(!text.contains("\"/api/"), "seatpill に口の字が在る");
    assert!(!text.contains("decode::<"), "seatpill が電文を decode する");

    let lib = read("src/lib.rs");
    let lines: Vec<&str> = lib.lines().map(str::trim).collect();
    let at = lines
        .iter()
        .position(|l| *l == "pub mod seatpill;")
        .expect("lib に pub mod seatpill; が在る");
    assert!(at > 0, "pub mod seatpill; の前の行が無い");
    assert!(
        !lines[at - 1].contains("target_arch"),
        "seatpill が wasm の target だけで組まれる"
    );
}

/// (6) この file の歯の名はちょうど 6 で、どれも gpill_ で始まり、残りの字は filter の語を含まない。
#[test]
fn gpill_own_names_clean() {
    let text = read("tests/gpill.rs");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "#[test]")
        .map(|(i, _)| {
            lines[i + 1..]
                .iter()
                .find_map(|l| l.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next())
                .expect("test の属性の後の fn")
        })
        .collect();
    assert_eq!(names.len(), 6, "{names:?}");
    let words = [words_front(), words_middle(), words_back()].concat();
    names_avoid(names, &words);
}

/// filter の語の表の前の部分（44 語）。
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

/// filter の語の表の中の部分（44 語）。
fn words_middle() -> &'static [&'static str] {
    &[
        "gpulse_",
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

/// filter の語の表の後の部分（44 語）。
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

/// 歯の名の残りの字が filter の語を含まないことを見る。
fn names_avoid(names: Vec<&str>, words: &[&str]) {
    for name in names {
        let rest = name
            .strip_prefix("gpill_")
            .unwrap_or_else(|| panic!("{name} が gpill_ で始まらない"));
        for w in words {
            assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
        }
    }
}
