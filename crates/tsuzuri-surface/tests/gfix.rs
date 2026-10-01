//! 行 g-pipe-misfit の歯（接頭辞 gfix_）: pipeline の板の 4 列の下の要修正の行（見本の案 A）の札の値・
//! 出さない入力・語と崩れの字と札の先・「?」の注釈の字数・stylesheet の規則・DOM の字・この file の歯の名。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::{Misfit, MisfitBead, PipelineBoard, PipelineCard, Reading, Stage};
use tsuzuri_contract::graph::title36;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::wire;
use tsuzuri_surface::frame::{Mode, node_href};
use tsuzuri_surface::project::pipeline::{
    BOTH_TEXT, MISFIT_CLASS, MISFIT_KEY, MisfitCard, NEITHER_TEXT, misfit_cards, misfit_href,
    misfit_text,
};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::help::{Inline, note};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 長い題（字 題の頭 と空白 2 つと、0123456789 の 4 組を空白 1 つでつないだ 48 字）。
const LONG: &str = "題の頭  0123456789 0123456789 0123456789 0123456789";

fn bead(id: &str, title: &str, misfit: Misfit) -> MisfitBead {
    MisfitBead {
        bead: BeadId::new(id).expect("id"),
        title: title.to_string(),
        misfit,
    }
}

fn body(board: &PipelineBoard) -> Fetched {
    Fetched::Body(wire::encode(board).expect("電文"))
}

fn known_card() -> PipelineCard {
    PipelineCard {
        contract: BeadId::new("mf.2").expect("id"),
        runs: 0,
        stage: Stage::Queued,
        reason: None,
        account: None,
        since: None,
        ci: None,
    }
}

/// 字数（記号の片は 0 字）。
fn chars(parts: &[Inline]) -> usize {
    parts
        .iter()
        .map(|p| match p {
            Inline::Text(t) | Inline::Code(t) => t.chars().count(),
            Inline::Sym(_) => 0,
        })
        .sum()
}

/// (1) 欄 misfits が Known のとき、欄の順に id と 36 字に切った題と崩れの字の札（欄 cards が Known でも Unknown でも同じ）。
#[test]
fn gfix_cards_from_wire() {
    assert_eq!(LONG.chars().count(), 48);
    let cut = "題の頭 0123456789 0123456789 0123456789";
    assert_eq!(title36(LONG), cut);
    assert_eq!(cut.chars().count(), 36);
    let misfits = Reading::Known(vec![
        bead("mf.1", LONG, Misfit::Neither),
        bead("mf.4", "t mf.4", Misfit::Both),
    ]);
    let want = vec![
        MisfitCard {
            id: "mf.1".to_string(),
            title: cut.to_string(),
            why: NEITHER_TEXT,
        },
        MisfitCard {
            id: "mf.4".to_string(),
            title: "t mf.4".to_string(),
            why: BOTH_TEXT,
        },
    ];
    for cards in [Reading::Known(vec![known_card()]), Reading::Unknown] {
        let board = PipelineBoard {
            cards: cards.clone(),
            misfits: misfits.clone(),
        };
        assert_eq!(misfit_cards(&body(&board)), want, "{cards:?}");
    }
}

/// (2) 欄 misfits が Known の空か Unknown・口を読んでいない・読めない・本文が電文として読めないときは空。
#[test]
fn gfix_hidden_cases() {
    for misfits in [Reading::Known(vec![]), Reading::Unknown] {
        let board = PipelineBoard {
            cards: Reading::Known(vec![known_card()]),
            misfits: misfits.clone(),
        };
        assert_eq!(misfit_cards(&body(&board)), vec![], "{misfits:?}");
    }
    let only_cards = r#"{"cards":"unknown"}"#;
    for fetched in [
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("{}".to_string()),
        Fetched::Body(only_cards.to_string()),
    ] {
        assert_eq!(misfit_cards(&fetched), vec![], "{fetched:?}");
    }
}

/// (3) 見出しの語・崩れの字・札の先。
#[test]
fn gfix_words_and_href() {
    assert_eq!(MISFIT_KEY, "misfit");
    assert_eq!(MISFIT_CLASS, "fixrow");
    assert_eq!(NEITHER_TEXT, "印が無い — 設計の参照も控えの印も無い");
    assert_eq!(BOTH_TEXT, "印が両方 — 設計の参照と控えの印の両方が在る");
    let text_fn: fn(Misfit) -> &'static str = misfit_text;
    assert_eq!(text_fn(Misfit::Neither), NEITHER_TEXT);
    assert_eq!(text_fn(Misfit::Both), BOTH_TEXT);
    assert_eq!(Misfit::ALL.len(), 2);

    let term = read("vocab.json");
    let at = term.find("\"rephrase\"").expect("欄 rephrase");
    assert!(term[at..].contains("\"misfit\": {"), "rephrase に鍵 misfit が無い");
    assert_eq!(vocab().label(MISFIT_KEY), "要修正");
    for w in [NEITHER_TEXT, BOTH_TEXT] {
        let json = read("vocab.json");
        for line in json.lines().filter(|l| l.trim_start().starts_with("\"label\"")) {
            assert!(!line.contains(&format!("\"{w}\"")), "見出しの語 {w} が在る");
        }
    }
    card_href();
}

/// 要修正の札の先は節点の頁の link で、どの mode でも mode を残すことを見る。
fn card_href() {
    let card = MisfitCard {
        id: "mf.1".to_string(),
        title: "t".to_string(),
        why: NEITHER_TEXT,
    };
    assert_eq!(
        misfit_href(&card, Mode::Expert),
        "?page=node&id=mf.1&mode=expert"
    );
    for mode in Mode::ALL {
        assert_eq!(misfit_href(&card, mode), node_href("mf.1", mode), "{mode:?}");
    }
}

/// (4) 「?」の注釈は 1 行目が 40 字以下・項が 4 つで各 20 字以下で、plain と internal の行は
/// 「・」を 2 つ以上持たず、括弧の入れ子と記号の片を持たない。
#[test]
fn gfix_note_budget() {
    let n = note(MISFIT_KEY).expect("鍵 misfit の注釈");
    assert!(chars(&n.head) <= 40, "1 行目が 40 字を越える: {:?}", n.head);
    assert!(chars(&n.head) > 0, "1 行目が空");
    assert_eq!(n.items.len(), 4, "{:?}", n.items);
    for item in &n.items {
        let len = chars(&item.sym) + chars(&item.text);
        assert!(len <= 20, "項が 20 字を越える: {item:?}");
    }
    assert!(!n.more.is_empty(), "詳しくが無い");

    let term = vocab().term(MISFIT_KEY).expect("鍵 misfit").clone();
    for line in term.note.lines().chain(term.internal.lines()) {
        assert!(line.matches('・').count() < 2, "・ が 2 つ以上: {line}");
        assert!(!line.contains('{') && !line.contains('}'), "記号の片: {line}");
        let mut depth = 0i32;
        for c in line.chars() {
            match c {
                '(' | '（' => {
                    depth += 1;
                    assert!(depth < 2, "括弧の入れ子: {line}");
                }
                ')' | '）' => depth -= 1,
                _ => {}
            }
        }
    }
}

/// (5) stylesheet の 6 つの規則の値。
#[test]
fn gfix_css_rules() {
    let css = read("style.css");
    let rule = |sel: &str| -> String {
        let head = format!("{sel} {{");
        css.lines()
            .find(|l| l.starts_with(&head))
            .unwrap_or_else(|| panic!("style.css に {head} の行が無い"))
            .to_string()
    };
    for (sel, values) in [
        (
            ".fixrow",
            &[
                "display: flex;",
                "flex-wrap: wrap;",
                "border: 1px dashed var(--s-stop);",
                "background: color-mix(in srgb, var(--s-stop) 10%, var(--panel));",
            ][..],
        ),
        (".fixrow > header", &["flex: 0 0 150px;"][..]),
        (".fixcards", &["flex: 1 1 260px;", "flex-wrap: wrap;"][..]),
        (".fixcard", &["flex: 0 1 260px;"][..]),
        (".fixcard .tt", &["font-weight: 650;"][..]),
        (".fixcard .why", &["color: var(--s-stop);"][..]),
    ] {
        let line = rule(sel);
        for v in values {
            assert!(line.contains(v), "{sel} に {v} が無い: {line}");
        }
    }
}

/// (6) mod dom の view は板の後に要修正の行を置き、misfit_view は札が 0 枚なら None で、ほかは案 A の行を組む。
#[test]
fn gfix_dom_wiring() {
    let text = read("src/project/pipeline.rs");
    let dom = &text[text.find("mod dom {").expect("mod dom が在る")..];
    let range = |from: &str, to: &str| -> &str {
        let s = dom.find(from).unwrap_or_else(|| panic!("{from} が在る"));
        let e = dom[s..].find(to).unwrap_or_else(|| panic!("{to} が在る")) + s;
        &dom[s..e]
    };
    let view = range("pub fn view() -> AnyView {", "fn board_view(");
    let mut at = 0;
    for w in [
        "let board = match pipe.with(|p| rows.with(|l| graph.with(|g| with_nodes(content(p, l, now), g)))) {",
        "let fix = pipe.with(|p| misfit_view(misfit_cards(p), mode()));",
        "view! { {board}{fix} }.into_any()",
    ] {
        let i = view[at..]
            .find(w)
            .unwrap_or_else(|| panic!("view に {w} が順に無い: {view}"));
        at += i + w.len();
    }

    let fix = range("fn misfit_view(", "fn kcard_view(");
    for w in [
        "fn misfit_view(cards: Vec<MisfitCard>, mode: Mode) -> Option<AnyView>",
        "if cards.is_empty() {",
        "return None;",
        "<a class=\"fixcard\" href=misfit_href(&c, mode)>",
        "<span class=\"kid\">{c.id.clone()}</span>",
        "<span class=\"tt\">{c.title.clone()}</span>",
        "<span class=\"why\">{c.why}</span>",
        ".collect_view()",
        "<div class=MISFIT_CLASS>",
        "<header>{hs(MISFIT_KEY)}</header>",
        "<div class=\"fixcards\">{cards}</div>",
        "Some(",
    ] {
        assert!(fix.contains(w), "misfit_view に {w} が無い: {fix}");
    }
}

/// (8) この file の歯の名はどれも gfix_ で始まり、残りの字は filter の語を含まない。
#[test]
fn gfix_own_names_clean() {
    let text = read("tests/gfix.rs");
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
    assert!(names.len() >= 7, "{names:?}");
    let words = [words_front(), words_middle(), words_back()].concat();
    names_avoid(names, &words);
}

/// filter の語の表の前の部分（47 語）。
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
        "epolq_",
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
        "gpill_",
        "gpulse_",
    ]
}

/// filter の語の表の中の部分（47 語）。
fn words_middle() -> &'static [&'static str] {
    &[
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
        "lidle_",
        "lsnap_",
        "lspark_",
        "lstore_",
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
        "nsumw_",
    ]
}

/// filter の語の表の後の部分（45 語）。
fn words_back() -> &'static [&'static str] {
    &[
        "ntime_",
        "nxact_",
        "parts_",
        "pclosed_",
        "pfold_",
        "pipe_",
        "plimit_",
        "pmore_",
        "pquest_",
        "pqueue_",
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
        "pgz_",
        "pmisfit_",
    ]
}

/// 表の語の数と、歯の名の残りの字が filter の語を含まないことを見る。
fn names_avoid(names: Vec<&str>, words: &[&str]) {
    assert_eq!(words.len(), 139);
    for name in names {
        let rest = name
            .strip_prefix("gfix_")
            .unwrap_or_else(|| panic!("{name} が gfix_ で始まらない"));
        for w in words {
            assert!(!rest.contains(w), "{name} が filter の語 {w} を含む");
        }
    }
}
