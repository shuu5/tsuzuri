//! 便 h-cards-nx の歯: account board の HOME の次の一手の行と群の枠の project の chip の hover の card
//! （見本の `__tz_card` の nx と gproj の枝）と、DOM の部分の字。

use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::home::{self, GroupView, Home, NxRow, UNKNOWN_SIGN};
use tsuzuri_surface::account::cards::{GPROJ_SRC, gproj_card, nx_card, row_cards};
use tsuzuri_surface::project::Body;
use tsuzuri_surface::project::seat::{NG, OK};
use tsuzuri_surface::widgets::hover::Card;

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

fn next_rows(h: &Home) -> &[NxRow] {
    match &h.next {
        Body::Filled(rows) => rows,
        other => panic!("次の一手が中身ありでない: {other:?}"),
    }
}

fn groups(h: &Home) -> &[GroupView] {
    match &h.groups {
        Body::Filled(g) => g,
        other => panic!("群の枠が中身ありでない: {other:?}"),
    }
}

fn group<'a>(h: &'a Home, name: &str) -> &'a GroupView {
    groups(h)
        .iter()
        .find(|g| g.name == name)
        .unwrap_or_else(|| panic!("群 {name} が無い"))
}

fn member_card(g: &GroupView, project: &str) -> Option<Card> {
    g.members
        .iter()
        .find(|m| m.project == project)
        .unwrap_or_else(|| panic!("群 {} に {project} が無い", g.name))
        .card
        .clone()
}

/// (1) 次の一手の行は nx_card の card を持つ。
#[test]
fn hcnx_rows_carry_cards() {
    let doc = fixture();
    let h = home::home(&doc);
    let rows = next_rows(&h);
    let names: Vec<&str> = rows.iter().map(|r| r.project.as_str()).collect();
    assert_eq!(names, ["proj-a", "proj-b", "proj-c"]);
    for r in rows {
        let p = doc
            .projects
            .iter()
            .find(|p| p.name == r.project)
            .expect("同じ名の行が在る");
        assert_eq!(r.card, nx_card(p, doc.at), "{} の card", r.project);
        assert_eq!(r.card, row_cards(&doc, p).need, "{} の card と表の要対応", r.project);
    }

    let a = &rows[0].card;
    assert_eq!(a.title, "proj-a · (e) 質問");
    assert_eq!(a.kind, "各 project の次の一手 · e");
    assert_eq!(a.value, "proj-a.7 · 1 件");
    let b = &rows[1].card;
    assert_eq!(b.title, "proj-b · 測れていない");
    assert_eq!(b.kind, "各 project の次の一手 · ―");
    assert_eq!(b.value, "―");

    // ほかの欄は card の無い組み方と同じ。
    for (r, p) in rows.iter().zip(["proj-a", "proj-b", "proj-c"]) {
        let row = doc.projects.iter().find(|x| x.name == p).expect("行が在る");
        let again = home::nx_row(row, doc.at);
        assert_eq!(&again, r);
        assert_eq!(r.marks, home::marks(&row.next));
    }
}

/// (2) 群の枠の project の chip は gproj_card の card を持つ（電文に同じ名の行が無ければ None）。
#[test]
fn hcnx_member_cards() {
    let doc = fixture();
    let h = home::home(&doc);
    let proj = |name: &str| {
        doc.projects
            .iter()
            .find(|p| p.name == name)
            .expect("同じ名の行が在る")
    };

    let t1 = group(&h, "Tier1");
    let t2 = group(&h, "Tier2");
    let a = member_card(t1, "proj-a").expect("proj-a の card");
    assert_eq!(a.title, "proj-a-orch");
    assert_eq!(a.kind, "✓ 今の口座で動いている");
    assert_eq!(a.value, "登録 acct-1 = 群 acct-1");
    assert_eq!(a.src, GPROJ_SRC);
    assert_eq!(a.more, ["動いている"]);
    assert_eq!(a, gproj_card(&doc, proj("proj-a")));

    let c = member_card(t1, "proj-c").expect("proj-c の card");
    assert_eq!(c.title, "proj-c");
    assert_eq!(c.kind, "session なし");
    assert_eq!(c.value, "Tier1 の今の口座 acct-1");
    assert!(c.more.is_empty());
    assert_eq!(c, gproj_card(&doc, proj("proj-c")));

    let b = member_card(t2, "proj-b").expect("proj-b の card");
    assert_eq!(b.title, "proj-b-orch");
    assert_eq!(b.kind, "! 移動待ち");
    assert_eq!(b.value, "登録 acct-1 ≠ 群 acct-2");
    assert_eq!(b.more, ["待っている"]);
    assert_eq!(b, gproj_card(&doc, proj("proj-b")));

    // 群の project の名と印は電文の members の順と matches のまま。
    let Reading::Known(cards) = &doc.groups else {
        panic!("fixture の群は Known");
    };
    for (g, card) in groups(&h).iter().zip(cards) {
        let want: Vec<&str> = card.members.iter().map(|m| m.project.as_str()).collect();
        let got: Vec<&str> = g.members.iter().map(|m| m.project.as_str()).collect();
        assert_eq!(got, want, "{} の members", g.name);
        for (m, w) in g.members.iter().zip(&card.members) {
            let sign = match w.matches {
                Reading::Known(true) => OK,
                Reading::Known(false) => NG,
                Reading::Unknown => UNKNOWN_SIGN,
            };
            assert_eq!(m.sign, sign, "{} の印", m.project);
        }
    }

    // 電文に proj-c の行が無ければ chip は card を持たない。ほかは変わらない。
    let mut loose = doc.clone();
    loose.projects.retain(|p| p.name != "proj-c");
    let h2 = home::home(&loose);
    let t1b = group(&h2, "Tier1");
    assert_eq!(member_card(t1b, "proj-c"), None);
    assert_eq!(member_card(t1b, "proj-a"), Some(a));
    let strip = |g: &GroupView| {
        let mut g = g.clone();
        for m in &mut g.members {
            m.card = None;
        }
        g
    };
    assert_eq!(strip(t1b), strip(t1));
}

/// home.rs の字「mod dom {」より後（DOM の部分）。
fn dom_part() -> String {
    let text = read("src/account/home.rs");
    let at = text.find("mod dom {").expect("home.rs に mod dom が在る");
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

/// 本体の中の字 head の所から最初の > までの tag。
fn tag(body: &str, head: &str) -> String {
    let at = body
        .find(head)
        .unwrap_or_else(|| panic!("本体に字 {head} が無い"));
    let rest = &body[at..];
    rest[..rest.find('>').unwrap_or(rest.len())].to_string()
}

fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// (3) DOM の部分が次の一手の行と群の project の chip に card を付ける。
#[test]
fn hcnx_dom_wiring() {
    let dom = dom_part();
    let flat = squeeze(&dom);
    assert!(
        flat.contains("usecrate::widgets::hover::{Card,attach};")
            || flat.contains("usecrate::widgets::hover::{attach,Card};"),
        "mod dom の use に crate::widgets::hover の attach と Card が無い"
    );
    assert!(
        flat.contains("fnattach_some(el:web_sys::Element,card:Option<Card>){"),
        "fn attach_some の宣言の形が違う"
    );
    assert!(
        squeeze(&fn_body(&dom, "attach_some")).contains("attach(el,card)"),
        "fn attach_some が attach に el と card を渡さない"
    );

    let tab = "tabindex=\"0\"";
    let row = tag(&fn_body(&dom, "nx_row_view"), "<div class=r.class");
    assert!(row.contains(tab), "nx_row_view の行の tag {row} に {tab} が無い");
    assert!(row.contains("use:attach="), "nx_row_view の行の tag {row} に use:attach= が無い");

    let card = fn_body(&dom, "group_card");
    let chip = "<span class=\"pchip\"";
    let chips: Vec<String> = card
        .match_indices(chip)
        .map(|(i, _)| tag(&card[i..], chip))
        .collect();
    assert!(!chips.is_empty(), "group_card に project の chip の tag が無い");
    for t in &chips {
        assert!(t.contains(tab), "group_card の chip の tag {t} に {tab} が無い");
        assert!(t.contains("use:attach_some="), "group_card の chip の tag {t} に use:attach_some= が無い");
    }
}

/// (6) この file の歯の名は hcnx_ で始まり、残りの字は filter の語を含まない。
#[test]
fn hcnx_names_clean() {
    const WORDS: [&str; 104] = [
        "aaround_", "accept_", "account_", "acctcore_", "acctdoc_", "accthb_", "accthome_",
        "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_", "acctwire_",
        "afocus_", "aord_", "apop_", "askcard_", "athr_", "batchpanel_", "board_min_", "bport_",
        "brand_", "btuck_", "cadopt_", "cgdom_", "contract_form_", "cround_", "csled_", "denv_",
        "flight_", "fmark_", "frame_", "fserve_", "fstop_", "gapspage_", "ghb_", "glabel_",
        "gnav_", "graph_", "gsum_", "gtuck_", "gview_", "hbconf_", "hbpost_", "hbproc_",
        "hbroute_", "hcard_", "hcled_", "hcproj_", "hcsess_", "hfig_", "hook_", "hruling_",
        "hsblock_", "hsderive_", "hspage_", "hsym_", "iclose_", "ilink_", "kcli_", "klink_",
        "lcard_", "ledgerblock_", "lhome_", "lspark_", "mapview_", "mkeys_", "mlink_", "mstore_",
        "mtree_", "nact_", "nbatch_", "ncard_", "nextstep_", "nodepage_", "nstall_", "nsum_",
        "nsumw_", "ntime_", "nxact_", "parts_", "pclosed_", "pfold_", "pipe_", "plimit_",
        "pmore_", "project_", "ptitle_", "pwhole_", "qblock_", "qgate_", "qkey_", "question_",
        "rhold_", "runsdoc_", "saxis_", "seatblock_", "seatcard_", "server_", "sesplit_", "shb_",
        "skeleton_", "smore_",
    ];
    const MORE: [&str; 11] = [
        "stage_", "stats_", "steady_", "sxaxis_", "ticker_", "tipx_", "topbar_", "tz_",
        "urpanel_", "uword_", "wstrip_",
    ];
    let text = read("tests/hcnx.rs");
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
    assert!(names.len() >= 4, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("hcnx_")
            .unwrap_or_else(|| panic!("歯の名 {name} が hcnx_ で始まらない"));
        for w in WORDS.iter().chain(MORE.iter()) {
            assert!(!rest.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
