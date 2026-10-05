//! 行 g-accept-fix の歯（接頭辞 bvafix_）: 受入の残りの違反 2 つを 0 にする。
//! hover の欠けの数えを節点の頁の近傍の図の委ねの口（祖先の mouseover と mouseout）にも合わせ、測りの式は節点の札ごとに
//! 段ごとの受け手の種類を返すだけで、card を出せるかの判じは audit の側が持つ（欠けの在る fixture で数えが落ちることを
//! browser 無しで撃てる・憲法 P-7）。account board の各 project の表は、広い段の列が収まらない中の幅で列を行の中へ移し、
//! HOME の口座の表の列の見出しは収まらない分を次の行へ送る。
#![cfg(test)]

use std::fs;

use crate::common::{read, root};
use tsuzuri_boundary::audit::{Facts, RULES, Reach, count, facts};

fn fixture(name: &str) -> String {
    read(&format!("tests/fixtures/surface/accept-12/{name}"))
}

/// clean.json の reach の欄を `list` の字に替えた事実の字。
fn with_reach(list: &str) -> String {
    let clean = fixture("clean.json");
    assert_eq!(
        clean.matches("\"reach\": []").count(),
        1,
        "clean.json の reach"
    );
    clean.replace("\"reach\": []", &format!("\"reach\": {list}"))
}

/// 段ごとの受け手の種類の字の列から組んだ節点の札 1 つの JSON の字。
fn entry(name: &str, up: &[&str]) -> String {
    let up: Vec<String> = up.iter().map(|k| format!("\"{k}\"")).collect();
    format!("{{\"name\": \"{name}\", \"up\": [{}]}}", up.join(", "))
}

/// hover の条だけが `n` でほかは 0 の数の列。
fn hover_only(n: usize) -> [usize; 12] {
    let at = RULES
        .iter()
        .position(|(k, _)| *k == "hover")
        .expect("条 hover");
    let mut want = [0; 12];
    want[at] = n;
    want
}

/// (2) 1 つの段が mouseover と mouseout を対で持つか、どれかの段が pointerenter を持つ札だけが card を持ち、
/// 片方だけの段・2 つの段に分かれた対・受け手の無い札・段の無い札は欠けに数える（欠けの在る fixture で落ちる）。
#[test]
fn bvafix_pair_on_one_level() {
    let vocab = fixture("vocab.json");
    let held = [
        entry("g.node", &["", "", "mouseover mouseout"]),
        entry("a.mono", &["pointerenter", ""]),
        entry("a.btn", &["", "", "", "pointerenter"]),
    ];
    let gaps = [
        entry("g.node", &["", "", "mouseover"]),
        entry("g.node", &["", "", "mouseout"]),
        entry("g.node", &["mouseover", "mouseout"]),
        entry("a.mono", &["", "", ""]),
        entry("a.mono", &[]),
    ];
    let clean = facts(&with_reach(&format!("[{}]", held.join(", ")))).expect("card の在る札");
    assert_eq!(clean.reach.len(), 3);
    assert!(clean.reach.iter().all(Reach::has_card), "{:?}", clean.reach);
    assert_eq!(count(&clean, &vocab), [0; 12]);
    for gap in &gaps {
        let got = facts(&with_reach(&format!("[{}, {gap}]", held.join(", ")))).expect(gap);
        assert_eq!(count(&got, &vocab), hover_only(1), "{gap}");
    }
    let all = facts(&with_reach(&format!("[{}]", gaps.join(", ")))).expect("欠けの札");
    assert!(all.reach.iter().all(|r| !r.has_card()), "{:?}", all.reach);
    assert_eq!(count(&all, &vocab), hover_only(gaps.len()));
    let mouth = facts(&fixture("04-hover.json")).expect("04-hover.json");
    let both = facts(&with_reach(&format!("[{}]", gaps[0]))).expect("口と札");
    let mixed = Facts {
        reach: both.reach,
        ..mouth
    };
    assert_eq!(
        count(&mixed, &vocab),
        hover_only(2),
        "吹き出しの口の欠けと札の欠けを足す"
    );
}

/// (3) reach の欄が無い字と、札の名か段の欄が無い字と、段が字でない字は事実として読まない（読めない受け手を
/// 「欠け無し」にしない）。受入の fixture の 13 の事実の file は reach の欄を空で持つ。
#[test]
fn bvafix_missing_key_refused() {
    let clean = fixture("clean.json");
    let gone = clean.replace(" \"reach\": [],\n", "");
    assert_ne!(gone, clean, "clean.json の reach の行");
    let err = facts(&gone).expect_err("reach の欠け");
    assert!(err.contains("reach"), "{err}");
    for bad in [
        "[{\"name\": \"g.node\", \"up\": [1]}]",
        "[{\"name\": \"g.node\"}]",
        "[{\"up\": [\"pointerenter\"]}]",
        "[\"g.node\"]",
    ] {
        let err = facts(&with_reach(bad)).expect_err(bad);
        assert!(err.contains("reach"), "{bad}: {err}");
    }
    let dir = root().join("tests/fixtures/surface/accept-12");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .expect("accept-12 の dir")
        .map(|e| {
            e.expect("dir の項")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n.ends_with(".json") && n != "vocab.json")
        .collect();
    names.sort();
    assert_eq!(names.len(), 13, "{names:?}");
    for name in &names {
        let got = facts(&fixture(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(got.reach.is_empty(), "{name}");
    }
}

/// 字 `head` で始まる所から次の閉じの字 `close` の手前までの本文（JS の束縛や Rust の関数を 1 つ切る・`head` は 1 つだけ）。
fn item<'a>(text: &'a str, head: &str, close: &str) -> &'a str {
    assert_eq!(text.matches(head).count(), 1, "{head}");
    let body = &text[pos(text, head)..];
    &body[..pos(body, close)]
}

/// 字 `want` の位置（無ければ落ちる）。
fn pos(text: &str, want: &str) -> usize {
    text.find(want).unwrap_or_else(|| panic!("{want} が無い"))
}

/// 空白の続きを 1 つの空白に縮めた字（改行と字下げの違いを問わずに式の続きを比べる）。
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 名に使える字（英数字と _）の続きが `word` ちょうどである所の数（束縛の使い道を数える）。
fn uses(text: &str, word: &str) -> usize {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| *w == word)
        .count()
}

/// 関数 kinds: 祖先へ上る for の中でその段の要素 n の受け手を読み（束縛 l は読みと段の字の 2 所）、段の列 up は for の前に
/// 作って for の中で積み、for の後に名と一緒に返す（up は 3 所）。
fn kinds_walk(probe: &str) {
    let kinds = item(probe, "  const kinds = (e) => {\n", "\n  };\n");
    let walk = "    for (let n = e; n && n !== document.body; n = n.parentElement) {\n";
    let step = item(kinds, walk, "\n    }\n");
    assert!(
        pos(kinds, "    const up = [];\n") < pos(kinds, walk),
        "段の列 up は for の前"
    );
    for want in [
        "const l = getEventListeners(n);",
        "up.push([\"pointerenter\", \"mouseover\", \"mouseout\"].filter((k) => (l[k] || []).length > 0).join(\" \"));",
    ] {
        assert_eq!(step.matches(want).count(), 1, "for の中の {want}");
    }
    assert!(
        pos(step, "const l =") < pos(step, "up.push("),
        "段の受け手を読んでから積む"
    );
    assert_eq!(
        kinds.matches("getEventListeners(").count(),
        1,
        "kinds の受け手の読みは for の中の 1 つ"
    );
    assert_eq!(uses(step, "l"), 2, "受け手の束縛 l は読みと段の字の 2 所");
    assert_eq!(uses(kinds, "up"), 3, "段の列 up は作る・積む・返すの 3 所");
    let after = &kinds[pos(kinds, walk) + step.len()..];
    assert_eq!(
        after.matches("return { name: name(e), up };").count(),
        1,
        "for の後に名と段の列を返す"
    );
}

/// 面の近傍の図: 図の置き場に mouseover と mouseout を対で付け、節点の箱を関数 node_key の g.node で引き、over は的の
/// 節点の card を出し、out は card を離れる。
fn around_hooks() {
    let around = read("crates/tsuzuri-surface/src/project/nodearound.rs");
    assert!(
        around.contains("<div class=\"nb-graph\" node_ref=gz inner_html=picture\n                on:mouseover=over on:mouseout=out"),
        "近傍の図の委ねの口"
    );
    let key = item(&around, "    fn node_key(", "\n    }\n");
    assert!(key.contains("el.closest(\"g.node\")"), "節点の箱");
    let over = item(
        &around,
        "        let over = move |ev: ev::MouseEvent| {\n",
        "\n        };\n",
    );
    assert!(
        over.contains("node_key(ev.target())") && over.contains("hc.show(&ev, card);"),
        "{over}"
    );
    let out = item(
        &around,
        "        let out = move |ev: ev::MouseEvent| {\n",
        "\n        };\n",
    );
    assert!(out.contains("hc.leave(&ev);"), "{out}");
}

/// (4) 測りの式は節点の札（口の外の節点の頁への link と近傍の図の節点）ごとに、自分から body の手前の祖先まで段ごとに
/// pointerenter と mouseover と mouseout の受け手の種類を返し、nocard は吹き出しの口の欠けだけを持つ。
/// 段ごとの読みは関数 kinds の祖先へ上る for の中でその段の要素 n の受け手を読み、事実の欄 reach は見える要素の列 seen から
/// 2 つの filter と kinds の map を続けた 1 つの鎖で作る（鎖の頭から尾までを続きの字で比べる）。
/// 面の近傍の図は、節点の箱（g.node）の祖先の図の置き場に mouseover と mouseout を対で付け、card の出し消しは節点の箱で引く。
#[test]
fn bvafix_probe_text() {
    let probe = read("crates/tsuzuri-boundary/src/stage/measure.js");
    kinds_walk(&probe);
    for (head, want) in [
        (
            "  const nopop = ",
            "const nopop = seen.filter((e) => e.matches(mouth)).filter((e) => (getEventListeners(e).click || []).length === 0)",
        ),
        ("  const nocard = ", "const nocard = nopop.map(name)"),
        (
            "  const reach = ",
            "const reach = seen .filter((e) => (e.matches(\"a[href]\") && e.getAttribute(\"href\").includes(\"page=node\")) || e.matches(\".node\")) .filter((e) => !e.closest(mouth)) .map(kinds)",
        ),
    ] {
        assert_eq!(flat(item(&probe, head, ";\n")), want, "{head} の鎖");
    }
    let back = item(&probe, "  return JSON.stringify({\n", "\n  });\n");
    assert_eq!(
        back.matches("    nocard,\n    reach,\n").count(),
        1,
        "返す欄 reach は nocard の次"
    );
    assert!(!probe.contains("entered"), "前の pointerenter だけの見方");
    around_hooks();
}

/// 字の中の `head` で始まる規則の塊（次の `\n}\n` まで）。
fn block<'a>(css: &'a str, head: &str) -> &'a str {
    let at = css.find(head).unwrap_or_else(|| panic!("{head} が無い"));
    let body = &css[at..];
    &body[..body
        .find("\n}\n")
        .unwrap_or_else(|| panic!("{head} の閉じ"))]
}

/// 幅 760 px 以下の塊の頭（761 px 以上では効かない）。
const NARROW: &str = "@media (max-width: 760px)";

/// 中の幅の塊の頭。
const MID: &str = "@media (min-width: 761px) and (max-width: 1239px)";

/// stylesheet の規則 1 つ: 囲む塊の頭（最上位は空の字）・選びの列（, で割った字）・宣言の列（名と値）・`{` の位置。
struct Rule {
    within: String,
    sel: Vec<String>,
    decls: Vec<(String, String)>,
    at: usize,
}

/// 宣言の字（名と値の列）を名と値の対に割る。
fn decls(body: &str) -> Vec<(String, String)> {
    body.split(';')
        .filter_map(|d| d.split_once(':'))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .collect()
}

/// 注を空白に替えた stylesheet を波括弧の深さで規則に割る（@ で始まる頭は塊・それ以外は中に波括弧を持たない規則）。
fn rules(css: &str) -> Vec<Rule> {
    let mut text = css.to_owned();
    while let Some(a) = text.find("/*") {
        let b = text[a..].find("*/").map_or(text.len(), |b| a + b + 2);
        text.replace_range(a..b, &" ".repeat(b - a));
    }
    let (mut out, mut heads, mut start) = (Vec::new(), Vec::<String>::new(), 0);
    while let Some(off) = text[start..].find(['{', '}', ';']) {
        let at = start + off;
        let head = text[start..at].trim().to_owned();
        start = at + 1;
        match &text[at..=at] {
            "{" if head.starts_with('@') => heads.push(head),
            "{" => {
                let end = at + pos(&text[at..], "}");
                out.push(Rule {
                    within: heads.last().cloned().unwrap_or_default(),
                    sel: head.split(',').map(|s| s.trim().to_owned()).collect(),
                    decls: decls(&text[at + 1..end]),
                    at,
                });
                start = end + 1;
            }
            "}" => {
                heads.pop();
            }
            _ => {}
        }
    }
    out
}

/// 最上位の規則 `sel { body }` が 1 つだけ在り、その後に在って幅 761 px 以上で効く規則（最上位と、幅 760 px 以下の塊の外の塊）の
/// うち選びの列に同じ `sel` を持つ規則は、その宣言の名をどれも宣言しない（同じ詳しさの後の規則に上書きされない）。
fn wins(all: &[Rule], sel: &str, body: &str) {
    let want = decls(body);
    let mine: Vec<&Rule> = all
        .iter()
        .filter(|r| r.sel == [sel] && r.decls == want)
        .collect();
    assert_eq!(mine.len(), 1, "{sel} {{ {body} }}");
    assert_eq!(mine[0].within, "", "{sel} は media の塊の外（最上位）");
    let later = all
        .iter()
        .filter(|r| r.at > mine[0].at && r.within != NARROW && r.sel.iter().any(|s| s == sel));
    for r in later {
        assert!(
            !r.decls
                .iter()
                .any(|(k, _)| want.iter().any(|(w, _)| w == k)),
            "{sel} の宣言を後の規則（塊 {} の中・{:?}）が上書き",
            r.within,
            r.decls
        );
    }
}

/// 塊 `head` の中の規則は、選びの字を 1 つでも同じくして同じ名の宣言を持つ最上位の規則のどれよりも後に在る（cascade で勝つ）。
fn after_top(all: &[Rule], head: &str) {
    let inner: Vec<&Rule> = all.iter().filter(|r| r.within == head).collect();
    assert!(!inner.is_empty(), "{head} の規則");
    for r in &inner {
        let same = all
            .iter()
            .filter(|t| t.within.is_empty() && t.sel.iter().any(|s| r.sel.contains(s)))
            .filter(|t| {
                t.decls
                    .iter()
                    .any(|(k, _)| r.decls.iter().any(|(w, _)| w == k))
            });
        for top in same {
            assert!(
                top.at < r.at,
                "{head} の {:?} より後に最上位の {:?}",
                r.sel,
                top.sel
            );
        }
    }
}

/// grid-template-columns の字の列ごとの最小の px（minmax の最小か固い px）の和。
fn least(columns: &str) -> u32 {
    columns
        .split(") ")
        .flat_map(|part| part.split(' '))
        .filter_map(|w| {
            w.trim_start_matches("minmax(")
                .strip_suffix("px,")
                .or(w.strip_suffix("px"))
        })
        .map(|n| n.parse::<u32>().unwrap_or_else(|e| panic!("{n}: {e}")))
        .sum()
}

/// 台帳の列の 1 行目（語の中で折らない）・行の 1 行目の要素の送り・run の数・開くの button の規則は最上位に在って後の規則や
/// 中の幅の塊に上書きされず、幅 761 px 以上で効く規則に台帳の列の 1 行目と run の数を 1 行に留める規則は無い。
fn held(all: &[Rule]) {
    for (sel, body) in [
        (".prow .c-led .l1", "white-space: nowrap;"),
        (
            ".prow .l1",
            "display: flex; align-items: center; gap: 6px; flex-wrap: wrap; min-width: 0;",
        ),
        (
            ".prow .c-run .rc4",
            "display: inline-flex; flex-wrap: wrap; gap: 4px 8px; font-size: 14px;",
        ),
        (
            ".prow .c-open .btn",
            "font-size: 14px; padding: 0 8px; white-space: normal;",
        ),
    ] {
        wins(all, sel, body);
    }
    let nowrap = ("flex-wrap".to_owned(), "nowrap".to_owned());
    for r in all.iter().filter(|r| r.within != NARROW) {
        let keeps = r
            .sel
            .iter()
            .any(|s| s.starts_with(".prow") && (s.ends_with(".rc4") || s.ends_with(".c-led .l1")));
        assert!(
            !(keeps && r.decls.contains(&nowrap)),
            "1 行に留める規則 {:?}",
            r.sel
        );
    }
}

/// (5) account board の各 project の表: 広い段の列の最小の和は 1140 px で、761〜1239 px の中の幅は台帳・run の数・
/// orchestrator・口座の列を開いた行の中へ移した 5 列（最小の和に頁の左右の余白 82 px を足して 761 px に収まる）、
/// 台帳の列と run の数の列と開くの button（横の余白 8 px）は収まらない分を次の行へ送る。
/// 中の幅の塊の規則は、同じ選びで同じ性質を宣言する最上位の規則（広い段の .prow ほか）より後に在り、台帳の列の 1 行目・
/// run の数・開くの button の規則は最上位に在って後の規則や中の幅の塊に上書きされない。
#[test]
fn bvafix_ptab_mid_range() {
    let css = read("crates/tsuzuri-surface/style.css");
    let all = rules(&css);
    let wide = ".prow { display: grid; grid-template-columns: ";
    let at = css.find(wide).expect("広い段の .prow") + wide.len();
    let columns = css[at..].split(';').next().unwrap_or_default();
    assert_eq!(least(columns), 1140, "{columns}");
    let first = all
        .iter()
        .find(|r| r.sel == [".prow"])
        .expect("広い段の .prow");
    assert_eq!(
        first.within, "",
        "広い段の .prow は media の塊の外（最上位）"
    );
    let mid = block(&css, "@media (min-width: 761px) and (max-width: 1239px) {");
    let row = "  .prow { grid-template-columns: ";
    let at = mid.find(row).expect("中の幅の .prow") + row.len();
    let columns = mid[at..].split(';').next().unwrap_or_default();
    assert_eq!(least(columns), 638, "{columns}");
    assert!(least(columns) + 82 <= 761, "{columns}");
    for want in [
        "grid-template-areas: \"pn need wait un open\" \"more more more more more\";",
        "  .prow.hrow { grid-template-areas: \"pn need wait un open\"; }",
        "  .prow .c-led, .prow .c-run, .prow .c-orch, .prow .c-acc, .prow .h-ledger_block, .prow .h-runs4, .prow .h-seat, .prow .h-accounts { display: none; }",
        "  .prow .c-more .m-run, .prow .c-more .m-orch, .prow .c-more .m-acc, .prow .c-more .m-led { display: inline-flex; }",
    ] {
        assert_eq!(mid.matches(want).count(), 1, "中の幅の {want}");
    }
    after_top(&all, MID);
    for want in [
        ".prow .c-led .l1 { white-space: nowrap; } .prow .c-orch { flex-wrap: nowrap; white-space: nowrap; gap: 4px; }",
        ".prow .l1 { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; min-width: 0; }",
        ".prow .c-run .rc4 { display: inline-flex; flex-wrap: wrap; gap: 4px 8px; font-size: 14px; }",
        ".prow .c-open .btn { font-size: 14px; padding: 0 8px; white-space: normal; }",
    ] {
        assert_eq!(css.matches(want).count(), 1, "style.css の {want}");
    }
    held(&all);
    for gone in [
        ".prow .c-led .l1, .prow .c-orch",
        ".prow .c-run .rc4 { flex-wrap: nowrap; }",
    ] {
        assert!(!css.contains(gone), "1 行に留める前の規則 {gone}");
    }
}

/// (6) account board の HOME の口座の表（class acct-grid）の見出しの行は、見出しの箱（class hd）が折り返して縮み、
/// 見出しの字（class hd-t）は語の中でも折れる（初心者の「?」の分で 960 の列から溢れた）。
/// 見出しの 2 つの規則と表の列と見出しの箱の既定は最上位に在り、後の規則や幅 761 px 以上で効く塊に上書きされない。
#[test]
fn bvafix_grid_head_wraps() {
    let css = read("crates/tsuzuri-surface/style.css");
    let all = rules(&css);
    let want = ".acct-grid > .hrow .hd { flex-wrap: wrap; min-width: 0; } .acct-grid > .hrow .hd-t { overflow-wrap: anywhere; }";
    assert_eq!(css.matches(want).count(), 1, "style.css の {want}");
    let grid = ".acct-grid { display: grid; grid-template-columns: 80px 156px repeat(3, minmax(0, 1fr)) 150px 76px; gap: 0; }";
    assert_eq!(css.matches(grid).count(), 1, "口座の表の列");
    assert!(
        css.contains(".hd { display: inline-flex; align-items: center; gap: var(--s2); }"),
        "見出しの箱の既定"
    );
    for (sel, body) in [
        (".acct-grid > .hrow .hd", "flex-wrap: wrap; min-width: 0;"),
        (".acct-grid > .hrow .hd-t", "overflow-wrap: anywhere;"),
        (
            ".acct-grid",
            "display: grid; grid-template-columns: 80px 156px repeat(3, minmax(0, 1fr)) 150px 76px; gap: 0;",
        ),
        (
            ".hd",
            "display: inline-flex; align-items: center; gap: var(--s2);",
        ),
    ] {
        wins(&all, sel, body);
    }
}
