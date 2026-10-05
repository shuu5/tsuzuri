//! 便 h-proj-more の歯: 各 project の表の行の詳しくの段（台帳の項・sparkline・session・口座の履歴・狭い幅で隠れる 4 列の値）と
//! 行の開閉・2 段目を … で切る fit_cut と、DOM の部分の字（描きの fn の本体の在る無し）。
#![cfg(test)]

use crate::common::{fixture, read};
use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::Reading;
use tsuzuri_surface::account::projects::{
    LedMore, MORE_LED, More, NO_TOGGLE, PSort, ProjLine, fit_cut, more, table,
};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::ledger::{Net, SPARK_H, SPARK_W, net, spark, spark_svg};
use tsuzuri_surface::vocab::vocab;

/// projects.rs の字「mod dom {」より後（DOM の部分）。
fn dom_part() -> String {
    let text = read("src/account/projects.rs");
    let at = text.find("mod dom {").expect("projects.rs に mod dom が在る");
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

/// svg の字の中の polyline の points の字。
fn points_of(svg: &str, class: &str) -> String {
    let head = format!("class=\"{class}\" fill=\"none\" points=\"");
    let at = svg.find(&head).unwrap_or_else(|| panic!("{class} の polyline が無い")) + head.len();
    svg[at..].split('"').next().unwrap_or_default().to_string()
}

/// (1) 鍵の並びと selector と型と関数の形。
#[test]
fn pmore_consts_and_shape() {
    assert_eq!(
        MORE_LED,
        ["l_ready", "l_blocked", "l_memo", "l_lead", "l_stale", "l_net7"]
    );
    assert_eq!(NO_TOGGLE, "button, a, .q");
    let f: fn(&AccountDoc, usize) -> More = more;
    let m = f(&fixture(), 0);
    let l: Option<LedMore> = m.ledger.clone();
    let l = l.expect("proj-a の台帳は読める");
    let _: [(&'static str, String); 6] = l.items;
    let _: Net = l.net7;
    let _: String = l.spark;
    let _: Vec<(&'static str, String)> = m.sessions;
    let _: Option<usize> = m.hist;
}

/// (2) proj-a の詳しく（台帳の 6 項・純増の class・sparkline・session・履歴）。
#[test]
fn pmore_alpha_row_values() {
    let doc = fixture();
    let m = more(&doc, 0);
    let l = m.ledger.expect("proj-a の台帳は読める");
    let items: Vec<(&str, &str)> = l.items.iter().map(|(k, v)| (*k, v.as_str())).collect();
    assert_eq!(
        items,
        vec![
            ("l_ready", "5"),
            ("l_blocked", "3"),
            ("l_memo", "3"),
            ("l_lead", "4.0d"),
            ("l_stale", "1"),
            ("l_net7", "+4"),
        ]
    );
    // net_drop_7d 4 は 7 日で open が 4 つ増えた純増。
    assert_eq!(l.net7, net(4));
    assert_eq!(l.net7.class, "net net-up");
    assert_eq!(l.net7.arrow, "↑");
    assert_eq!(l.net7.word, "純増");

    let Reading::Known(stats) = &doc.projects[0].ledger else {
        panic!("proj-a の台帳は Known");
    };
    assert_eq!(l.spark, spark_svg(&spark(&stats.days, SPARK_W, SPARK_H)));
    let created = points_of(&l.spark, "ls-created");
    assert!(created.starts_with("1.0,12.0"), "{created}");
    assert!(created.ends_with("119.0,22.0"), "{created}");
    assert_eq!(
        created,
        "1.0,12.0 10.1,22.0 19.2,2.0 28.2,22.0 37.3,12.0 46.4,22.0 55.5,12.0 64.5,22.0 73.6,12.0 82.7,22.0 91.8,12.0 100.8,22.0 109.9,12.0 119.0,22.0"
    );
    assert_eq!(
        points_of(&l.spark, "ls-closed"),
        "1.0,22.0 10.1,12.0 19.2,12.0 28.2,22.0 37.3,2.0 46.4,22.0 55.5,22.0 64.5,12.0 73.6,12.0 82.7,12.0 91.8,2.0 100.8,22.0 109.9,12.0 119.0,12.0"
    );

    assert_eq!(
        m.sessions,
        vec![
            ("run", "proj-a-orch".to_string()),
            ("run", "proj-a.3-20260927T113000Z".to_string()),
        ]
    );
    assert_eq!(m.hist, Some(1));
}

/// (3) 台帳が Unknown の行は None・席が Unknown の行の履歴は None。
#[test]
fn pmore_unknown_rows() {
    let doc = fixture();
    let b = more(&doc, 1);
    assert_eq!(b.ledger, None);
    assert_eq!(b.sessions, vec![("wait", "proj-b-orch".to_string())]);
    assert_eq!(b.hist, Some(0));
    let c = more(&doc, 2);
    assert_eq!(c.ledger, None);
    assert_eq!(c.sessions, vec![("unknown", "proj-c-orch".to_string())]);
    assert_eq!(c.hist, None);
}

/// (3) 名が空の字の session の行は入らない。
#[test]
fn pmore_empty_name_skipped() {
    let mut doc = fixture();
    let mut blank = doc.sessions[0].clone();
    blank.name = String::new();
    doc.sessions.insert(1, blank);
    let a = more(&doc, 0);
    assert_eq!(
        a.sessions,
        vec![
            ("run", "proj-a-orch".to_string()),
            ("run", "proj-a.3-20260927T113000Z".to_string()),
        ]
    );
}

/// (4) 4 つの並べ方の全部の行で more は doc と行の index から組んだ値と同じ・行の class に open は無い。
#[test]
fn pmore_line_field_matches() {
    let doc = fixture();
    for sort in PSort::ALL {
        let t = table(&doc, sort, Mode::Beginner);
        let rows: Vec<&ProjLine> = t.groups.iter().flat_map(|g| g.rows.iter()).collect();
        assert_eq!(rows.len(), doc.projects.len(), "{sort:?}");
        for r in rows {
            assert_eq!(r.more, more(&doc, r.index), "{sort:?} の {}", r.name);
            assert!(
                !r.class.split_whitespace().any(|c| c == "open"),
                "{sort:?} の {} の class {}",
                r.name,
                r.class
            );
        }
    }
}

/// (5) fit_cut は字数に収まらない字を先頭から切り、末尾の空白・中黒・開き括弧を除いて … を足す。
#[test]
fn pmore_fit_cut_cases() {
    assert_eq!(fit_cut("abcdef", 10), "abcdef");
    assert_eq!(fit_cut("abcdef", 6), "abcdef");
    assert_eq!(fit_cut("abcdef", 3), "abc…");
    assert_eq!(fit_cut("質問に答える・次へ", 7), "質問に答える…");
    assert_eq!(fit_cut("ab (cd", 4), "ab…");
    assert_eq!(fit_cut("ab（cd", 3), "ab…");
    assert_eq!(fit_cut("abc", 0), "…");
}

/// (6) DOM の部分に行の開閉と 2 段目の収めの字が在る。
#[test]
fn pmore_dom_words() {
    let dom = dom_part();
    for word in [
        "class=\"c-more\"",
        "aria-expanded",
        "NO_TOGGLE",
        "closest(",
        "shows_internal(",
        "fit_cut(",
        "scroll_width()",
        "client_width()",
        "ev::resize",
        "set_attribute(\"aria-label\"",
    ] {
        assert!(dom.contains(word), "DOM の部分に字 {word} が無い");
    }
}

/// (7) 詳しくの段の class は stylesheet に・語の鍵は語の辞書に在る。
#[test]
fn pmore_classes_and_keys() {
    let css = read("style.css");
    for c in [
        "c-more", "mi", "lk", "sp", "slist", "muted", "m-run", "m-orch", "m-acc", "m-led", "open",
    ] {
        assert!(css.contains(&format!(".{c}")), "stylesheet に class {c} が無い");
    }
    let mut keys: Vec<&str> = MORE_LED.to_vec();
    keys.extend(["session", "acct_hist", "j_none"]);
    for key in keys {
        let term = vocab()
            .term(key)
            .unwrap_or_else(|| panic!("鍵 {key} が vocab に無い"));
        assert!(!term.label.is_empty(), "鍵 {key} の語が空");
    }
}

/// (9) 欄の class で包むのは row_view だけ（詳しくの中の 4 つの値は狭い幅でも隠れない）。
#[test]
fn pmore_view_bodies() {
    let dom = dom_part();
    let cols = ["C_RUN", "C_ORCH", "C_ACC", "C_LED"];
    let words = ["c-run", "c-orch", "c-acc", "c-led"];
    for name in ["more_view", "runs_view", "acc_view", "led_view", "orch_view"] {
        let body = fn_body(&dom, name);
        for w in cols.iter().chain(words.iter()) {
            assert!(!body.contains(w), "fn {name} の本体に字 {w} が在る");
        }
    }
    let row = fn_body(&dom, "row_view");
    for w in cols {
        assert!(row.contains(w), "fn row_view の本体に字 {w} が無い");
    }
    assert!(row.contains("more_view("), "fn row_view が more_view を呼ばない");
    assert!(!row.contains("c-more"), "fn row_view の本体に字 c-more が在る");
    let more_body = fn_body(&dom, "more_view");
    for w in [
        "c-more",
        "mi m-run",
        "mi m-orch",
        "mi m-acc",
        "mi m-led",
        "runs_view(",
        "orch_view(",
        "acc_view(",
        "led_view(",
    ] {
        assert!(more_body.contains(w), "fn more_view の本体に字 {w} が無い");
    }
}

/// (10) この file の歯の名は pmore_ で始まり、残りの字は filter の語を含まない。
#[test]
fn pmore_names_prefixed() {
    const WORDS: [&str; 65] = [
        "accept_", "account_", "acctcore_", "acctdoc_", "acctframe_", "accthb_", "accthome_",
        "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
        "askcard_", "batchpanel_", "board_min_", "contract_form_", "frame_", "gapspage_",
        "gquestion_", "graph_", "gview_", "hbconf_", "hbpost_", "hbproc_", "hbroute_", "hook_",
        "hsblock_", "hsderive_", "hspage_", "ledgerblock_", "mapgraph_", "mapview_", "nextstep_",
        "nodepage_", "parts_", "pipe_", "project_", "question_", "seatblock_", "seatcard_",
        "server_", "skeleton_", "stage_", "stats_", "steady_", "topbar_", "tz_", "mlink_",
        "gnav_", "mkeys_", "klink_", "ilink_", "afocus_", "nxact_", "urpanel_", "saxis_",
        "ticker_", "hfig_", "lspark_", "apop_", "brand_", "runsdoc_", "nbatch_",
    ];
    let text = read("tests/teeth4/projmore.rs");
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
    assert_eq!(names.len(), 10, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("pmore_")
            .unwrap_or_else(|| panic!("歯の名 {name} が pmore_ で始まらない"));
        for w in WORDS {
            assert!(!rest.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
