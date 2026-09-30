//! 行 g-unref-word の歯: 語の鍵 l_unref の「?」の注釈を、2 つの board のどちらでも正しい字にする。
//! 1 行目は何の数か・項は未反映の 3 種（要件 FR13）・詳しくは測れていないの印の意味。
//! 字数は要件 FR14 の上限に収め、規則の行 R-19 に拠る決め（「・」の連ね・括弧の入れ子）も見る。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_surface::vocab::vocab;
use tsuzuri_surface::widgets::help::{Inline, Line, Note, note};

/// 要件 FR14 の数: 1 行目の字数の上限。
const HEAD_MAX: usize = 40;
/// 要件 FR14 の数: 箇条書きの項の数の上限。
const ITEMS_MAX: usize = 4;
/// 要件 FR14 の数: 1 項の字数の上限。
const ITEM_MAX: usize = 20;

const KEY: &str = "l_unref";

const LINES: [&str; 6] = [
    "design-intent にまだ写っていないものの数",
    "・ memo = 昇格先の無い memo",
    "・ 決定 = 処分の宣言が無い決定",
    "・ 要望 = 反映されていない要望",
    "▸",
    "{st:unknown} 付きは まだ数えていない種類がある",
];

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn unref_note() -> Note {
    note(KEY).expect("語彙に l_unref が在る")
}

fn text(s: &str) -> Inline {
    Inline::Text(s.to_string())
}

/// 片の字数（Sym の片は 0 字・Text と Code は Unicode の scalar の数）。
fn chars(parts: &[Inline]) -> usize {
    parts
        .iter()
        .map(|p| match p {
            Inline::Text(s) | Inline::Code(s) => s.chars().count(),
            Inline::Sym(_) => 0,
        })
        .sum()
}

/// (1) 面の語の辞書の l_unref の label・note・internal。
#[test]
fn uword_note_text() {
    let term = vocab().term(KEY).expect("語彙に l_unref が在る");
    assert_eq!(term.label, "未反映");
    assert_eq!(term.note, LINES.join("\n"));
    assert_eq!(
        term.internal,
        "unreflected(G).length\n他の project は open memo ∧ ¬promoted_to"
    );
    assert!(!term.note.contains("tsuzuri 以外"), "{}", term.note);
}

/// (2) help の note の分け方: head・3 つの項・詳しくの 1 行。
#[test]
fn uword_note_lines() {
    let n = unref_note();
    assert_eq!(n.label, "未反映");
    assert_eq!(n.head, vec![text("design-intent にまだ写っていないものの数")]);
    let item = |body: &str| Line {
        sym: vec![text("・")],
        text: vec![text(body)],
    };
    assert_eq!(
        n.items,
        vec![
            item("memo = 昇格先の無い memo"),
            item("決定 = 処分の宣言が無い決定"),
            item("要望 = 反映されていない要望"),
        ]
    );
    assert_eq!(
        n.more,
        vec![Line {
            sym: vec![Inline::Sym("st:unknown".to_string())],
            text: vec![text("付きは まだ数えていない種類がある")],
        }]
    );
}

/// (3) 要件 FR14 の字数の上限と、規則の行 R-19 に拠る「・」の連ねと括弧の入れ子。
#[test]
fn uword_budget_r19() {
    let n = unref_note();
    assert_eq!(chars(&n.head), 27);
    assert!(chars(&n.head) <= HEAD_MAX, "1 行目 {}", chars(&n.head));
    assert_eq!(n.items.len(), 3);
    assert!(n.items.len() <= ITEMS_MAX, "項 {}", n.items.len());
    let lens: Vec<usize> = n.items.iter().map(|l| chars(&l.text)).collect();
    assert_eq!(lens, vec![18, 15, 15]);
    for len in lens {
        assert!(len <= ITEM_MAX, "項の字数 {len}");
    }
    let term = vocab().term(KEY).expect("語彙に l_unref が在る");
    for line in term.note.split('\n') {
        assert!(
            line.matches('・').count() < 2,
            "「・」を 2 つ以上持つ行 {line}"
        );
        let mut depth = 0i32;
        for c in line.chars() {
            match c {
                '(' | '（' => {
                    depth += 1;
                    assert!(depth < 2, "括弧が入れ子の行 {line}");
                }
                ')' | '）' => depth -= 1,
                _ => {}
            }
        }
    }
}

/// (4) 注釈の Sym の片は st:unknown の 1 つだけ。
#[test]
fn uword_symbols_known() {
    let n = unref_note();
    let mut parts: Vec<&Inline> = n.head.iter().collect();
    for l in n.items.iter().chain(&n.more) {
        parts.extend(l.sym.iter().chain(&l.text));
    }
    parts.extend(n.internal.iter().flatten());
    let syms: Vec<&str> = parts
        .into_iter()
        .filter_map(|p| match p {
            Inline::Sym(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(syms, vec!["st:unknown"]);
    for s in syms {
        for head in ["gi:", "nxm:", "thr:", "tk:", "band:"] {
            assert!(!s.starts_with(head), "{s} が {head} で始まる");
        }
    }
}

/// 着地済みの行と第 3 波から第 8 波のほかの行の verify の filter の語。
const FILTERS: &[&str] = &[
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
    "qgate_",
    "nsum_",
    "ntime_",
    "ptitle_",
    "ncard_",
    "hsym_",
    "smore_",
    "lcard_",
    "aaround_",
    "hruling_",
    "sxaxis_",
    "plimit_",
    "bport_",
    "fmark_",
    "fstop_",
    "fserve_",
    "nsumw_",
    "cadopt_",
    "tipx_",
    "hcsess_",
    "hcproj_",
    "sesplit_",
    "mstore_",
    "athr_",
    "qblock_",
    "wsteady_",
    "gsum_",
    "nstall_",
    "pfold_",
    "cround_",
    "cgdom_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
];

/// (6) この file の歯の名はどれも uword_ で始まり、残りの字は filter の語を含まない。
#[test]
fn uword_names_stay_apart() {
    assert_eq!(FILTERS.len(), 106);
    let text = read("tests/uword.rs");
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
    assert!(names.len() >= 5, "歯の数 {}", names.len());
    for name in names {
        let rest = name
            .strip_prefix("uword_")
            .unwrap_or_else(|| panic!("{name} が uword_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
