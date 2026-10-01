//! 行 g-park-view の歯: 区画（器の park）を面が群と分けて出す。project board の席の下段の見出しと点線の chip と短い card・
//! 詳しくの今の口座と印（区画は比べない）・account board の各 project の表の区画の行と群の順の見出しと card・
//! HOME の次の一手の行の小字・語の辞書の鍵 park・DOM の字の配線・この file の歯の名。
//! 区画かの判じは、project board は席の card の群の行の欄 park、account board は電文の欄 parks だけで、名 Tier9 を見ない。
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::{GroupRow, Reading};
use tsuzuri_contract::seat::SeatCard;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::cards::{gproj_card, grp_card};
use tsuzuri_surface::account::home::next_all;
use tsuzuri_surface::account::projects::{
    GroupHead, NONE_MARK, PSort, group_word, is_park, row, table,
};
use tsuzuri_surface::frame::Mode;
use tsuzuri_surface::project::seat::{
    NG, NO_CURRENT, PARK_KIND, PARK_SRC, PARK_VALUE, group_head, low, more, park_card, park_of,
};
use tsuzuri_surface::vocab::vocab;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 席の card の fixture の組 wait（群の行を区画の行の写しに替える前）。
fn wait_card() -> SeatCard {
    let mut all: BTreeMap<String, SeatCard> =
        wire::decode(&read("../../tests/fixtures/surface/seat-card.json"))
            .expect("fixture の組が電文として読める");
    all.remove("wait").expect("fixture の組 wait")
}

/// 群の行を区画の行の写し（group Tier9・account -・next_account -）にした card。park は引数で置く。
fn card_with(park: bool) -> SeatCard {
    let mut card = wait_card();
    card.group = Reading::Known(GroupRow {
        group: "Tier9".to_string(),
        account: "-".to_string(),
        candidates: Vec::new(),
        next_account: Some("-".to_string()),
        remaining: Vec::new(),
        park,
    });
    card
}

/// account board の fixture の電文の proj-b の群を Tier9 にし、parks を引数の列にした電文。
fn doc_with(parks: &[&str]) -> AccountDoc {
    let mut doc: AccountDoc =
        wire::decode(&read("../../tests/fixtures/account/acct-doc.json"))
            .expect("fixture が AccountDoc として読める");
    doc.parks = parks.iter().map(|p| p.to_string()).collect();
    let b = index_of(&doc, "proj-b");
    doc.projects[b].group = Some("Tier9".to_string());
    doc
}

fn index_of(doc: &AccountDoc, name: &str) -> usize {
    doc.projects
        .iter()
        .position(|p| p.name == name)
        .unwrap_or_else(|| panic!("fixture の project {name}"))
}

/// (1) project board の下段と詳しく: 区画の行の写しなら見出しは park・今の口座は ―・印は比べない。偽なら群のまま。
#[test]
fn pkview_seat_low_and_more() {
    let park = card_with(true);
    assert!(park_of(&park));
    let l = low(&park);
    assert!(l.park);
    assert_eq!(l.group, Reading::Known("Tier9".to_string()));
    assert_eq!(group_head(true), "park");
    let m = more(&park);
    assert_eq!(NO_CURRENT, "―");
    assert_eq!(m.current, Reading::Known(NO_CURRENT.to_string()));
    assert_eq!(m.same, None);

    let group = card_with(false);
    assert!(!park_of(&group));
    assert!(!low(&group).park);
    assert_eq!(group_head(false), "group");
    let m = more(&group);
    assert_eq!(m.current, Reading::Known("-".to_string()));
    assert_eq!(m.same, Some(Reading::Known(NG)));
}

/// (2) 区画の chip の card は電文に依らない短い説明。
#[test]
fn pkview_park_card_text() {
    let c = park_card("Tier9");
    assert_eq!(c.title, "区画 Tier9");
    assert_eq!(c.kind, "今の口座を持たない");
    assert_eq!(c.value, "口座は席ごと · 閾値に届いた席だけ器が移す");
    assert_eq!(c.src, "host.toml の区画の行・doctor");
    assert!(c.more.is_empty());
    assert_eq!(c.kind, PARK_KIND);
    assert_eq!(c.value, PARK_VALUE);
    assert_eq!(c.src, PARK_SRC);
}

/// (3) account board: 区画は電文の parks だけで判じ、見出し・card・行の小字・HOME の小字・口座の欄の card が区画の字になる。
#[test]
fn pkview_acct_rows() {
    let doc = doc_with(&["Tier9"]);
    assert!(is_park(&doc, "Tier9"));
    assert!(!is_park(&doc, "Tier1"));

    let t = table(&doc, PSort::Group, Mode::Beginner);
    let head_of = |name: &str| {
        t.groups
            .iter()
            .find(|g| g.head.as_ref().and_then(|h| h.group.as_deref()) == Some(name))
            .unwrap_or_else(|| panic!("群 {name} の見出し"))
    };
    let g9 = head_of("Tier9");
    assert_eq!(
        g9.head,
        Some(GroupHead {
            group: Some("Tier9".to_string()),
            current: Some(NONE_MARK.to_string()),
            park: true,
        })
    );
    assert_eq!(g9.card, Some(park_card("Tier9")));
    assert_eq!(
        head_of("Tier1").head,
        Some(GroupHead {
            group: Some("Tier1".to_string()),
            current: Some("acct-1".to_string()),
            park: false,
        })
    );
    assert_eq!(grp_card(&doc, "Tier9"), Some(park_card("Tier9")));

    let b = index_of(&doc, "proj-b");
    let a = index_of(&doc, "proj-a");
    let rb = row(&doc, b, Mode::Beginner);
    let ra = row(&doc, a, Mode::Beginner);
    assert!(rb.park);
    assert_eq!(group_word(rb.group.as_deref(), rb.park), "区画 Tier9");
    assert!(!ra.park);
    assert_eq!(group_word(ra.group.as_deref(), ra.park), "Tier1");
    assert_eq!(group_word(None, false), "");
    next_and_blank(doc, b);
}

/// HOME の次の一手の行の小字と口座の欄の card が区画の字になり、parks が空の電文は今の群の字のまま。
fn next_and_blank(doc: AccountDoc, b: usize) {
    let nx = next_all(&doc);
    let group_of = |name: &str| {
        nx.iter()
            .find(|r| r.project == name)
            .unwrap_or_else(|| panic!("次の一手の行 {name}"))
            .group
            .clone()
    };
    assert_eq!(group_of("proj-b"), "区画 Tier9");
    assert_eq!(group_of("proj-a"), "Tier1");

    let c = gproj_card(&doc, &doc.projects[b]);
    assert_eq!(c.kind, "区画 Tier9");
    assert_eq!(c.value, "登録 acct-1 · 今の口座を持たない");

    // parks が空の電文は今の群の字のまま（区画の行は出ない）。
    let doc = doc_with(&[]);
    assert!(!is_park(&doc, "Tier9"));
    let t = table(&doc, PSort::Group, Mode::Beginner);
    let g9 = t
        .groups
        .iter()
        .find(|g| g.head.as_ref().and_then(|h| h.group.as_deref()) == Some("Tier9"))
        .expect("群 Tier9 の見出し");
    assert_eq!(
        g9.head,
        Some(GroupHead {
            group: Some("Tier9".to_string()),
            current: None,
            park: false,
        })
    );
    assert_eq!(g9.card, None);
    assert_eq!(grp_card(&doc, "Tier9"), None);
    let b = index_of(&doc, "proj-b");
    assert!(!row(&doc, b, Mode::Beginner).park);
    let nx = next_all(&doc);
    let nb = nx.iter().find(|r| r.project == "proj-b").expect("行 proj-b");
    assert_eq!(nb.group, "Tier9");
    assert_ne!(gproj_card(&doc, &doc.projects[b]).kind, "区画 Tier9");
}

/// (4) 語の辞書の鍵 park（見出しの語 区画・原語 park）。
#[test]
fn pkview_vocab_word() {
    let v = vocab();
    let term = v.term("park").expect("鍵 park");
    assert_eq!(term.label, "区画");
    assert!(term.note.contains("今の口座を持たない"));
    assert!(term.internal.contains("kind=park"));
    let text = read("vocab.json");
    let rephrase = text.find("\"rephrase\": {").expect("欄 rephrase");
    let at = text.find("\"park\": {").expect("鍵 park の字");
    assert!(at > rephrase, "鍵 park は欄 rephrase の中");
}

/// (5) DOM の字は wasm の target でしか組めないので src の字を読む（どれもちょうど 1 度）。
#[test]
fn pkview_dom_wiring() {
    let once = |text: &str, part: &str, what: &str| {
        assert_eq!(text.matches(part).count(), 1, "{what} の字 {part}");
    };
    let seat = read("src/project_dom/seat.rs");
    for part in [
        "class:park=park",
        "{hs(group_head(low.park))}",
        "{group_chip(low.group, low.park, group_doc)}",
        "park_card(&key)",
        "{more.same.map(sign_view)}",
    ] {
        once(&seat, part, "src/project_dom/seat.rs");
    }
    let projects = read("src/account/projects.rs");
    let dom = projects.find("\nmod dom {").expect("mod dom");
    for part in [
        "class:park=head.park",
        "use:attach_some=card>{pk}<span data-t=",
        "{group_word(row.group.as_deref(), row.park)}",
    ] {
        once(&projects[dom..], part, "src/account/projects.rs");
    }
    let css = read("style.css");
    for part in [
        ".pchip.grp.park { border-style: dashed; border-color: var(--line-2); }",
        ".pchip.grp.park .pk {",
    ] {
        once(&css, part, "style.css");
    }
}

/// filter の語（起草の時の main 3d57d349 の契約表の verify の最後の字 306 語を、ほかの語を部分の字として含まない語に畳んだ語）。
const FILTERS: &[&str] = &[
    "aaround_",
    "abss_",
    "abst_",
    "accept_",
    "acchold_",
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
    "aface_",
    "afocus_",
    "alean_",
    "aord_",
    "aown_",
    "apark_",
    "apop_",
    "areread_",
    "askcard_",
    "athr_",
    "batchpanel_",
    "bhalf_",
    "board_min_",
    "bport_",
    "brand_",
    "btuck_",
    "cadl_",
    "cadopt_",
    "cadq_",
    "cdorm_",
    "cexcl_",
    "cfsplit_",
    "cg9_",
    "cgdom_",
    "cmark_",
    "cnote_",
    "cnret_",
    "contract_form_",
    "cround_",
    "csled_",
    "cspk_",
    "ctick_",
    "cupd_",
    "cupdlist_",
    "denv_",
    "dnedge_",
    "dngrp_",
    "dnrow_",
    "dnskip_",
    "dretry_",
    "dstg_",
    "ecache_",
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
    "fdlv_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fundl_",
    "fxpre_",
    "g3g7_",
    "gacct_",
    "gapspage_",
    "gatt_",
    "gbnote_",
    "gchip_",
    "gcoach_",
    "gext_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gins_",
    "gjst_",
    "glabel_",
    "gmret_",
    "gmretw_",
    "gnav_",
    "gpface_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "gwv_",
    "hacols_",
    "harest_",
    "hasplit_",
    "hbconf_",
    "hbmark_",
    "hbon_",
    "hbpost_",
    "hbproc_",
    "hbroute_",
    "hcard_",
    "hcled_",
    "hcnx_",
    "hcproj_",
    "hcsess_",
    "hdchip_",
    "hfig_",
    "hnunk_",
    "hook_",
    "hruling_",
    "hsblock_",
    "hsderive_",
    "hshist_",
    "hspage_",
    "hspk_",
    "hsym_",
    "hthr_",
    "http_",
    "hwstore_",
    "iclose_",
    "ilink_",
    "jcount_",
    "jrun_",
    "kcli_",
    "kg9_",
    "kindlab_",
    "klink_",
    "ksum_",
    "lateface_",
    "launch_",
    "lcard_",
    "ledgerblock_",
    "lgrp_",
    "lhome_",
    "lidle_",
    "lkind_",
    "lresume_",
    "lsnap_",
    "lspark_",
    "lstg_",
    "lstore_",
    "mapview_",
    "mkeys_",
    "mlink_",
    "mqask_",
    "mqface_",
    "mstore_",
    "mtips_",
    "mtree_",
    "nact_",
    "nbatch_",
    "ncard_",
    "nextstep_",
    "nodepage_",
    "nstall_",
    "nsum_",
    "nsumw_",
    "ntc_",
    "ntime_",
    "nxact_",
    "nxorg_",
    "parts_",
    "pci_",
    "pclosed_",
    "pfold_",
    "pgsw_",
    "pgz_",
    "pipe_",
    "pkac_",
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
    "pubfp_",
    "pubscan_",
    "punmap_",
    "pwhole_",
    "qblock_",
    "qgate_",
    "qkey_",
    "qsig_",
    "question_",
    "rbusy_",
    "relay_",
    "rhold_",
    "runsdoc_",
    "rvk_",
    "saxis_",
    "sclosed_",
    "seatblock_",
    "seatcard_",
    "server::events::",
    "server_",
    "sesplit_",
    "sgrace_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stbp_",
    "stcli_",
    "steady_",
    "sthr_",
    "stnfy_",
    "stskill_",
    "sttgt_",
    "sxaxis_",
    "ticker_",
    "tipx_",
    "tkad_",
    "tlic_",
    "topbar_",
    "topfit_",
    "tz_",
    "udacct_",
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "wstrip_",
];

/// (6) この file の歯の名はちょうど 6 で、どれも pkview_ で始まり、残りの字は filter の語を含まず、接頭辞は語と重ならない。
#[test]
fn pkview_own_names_clean() {
    assert_eq!(FILTERS.len(), 230);
    let prefix = "pkview_";
    for word in FILTERS {
        assert!(!prefix.contains(word), "接頭辞が filter の語 {word} を含む");
        assert!(!word.contains(prefix), "filter の語 {word} が接頭辞を含む");
    }
    let text = read("tests/pkview.rs");
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
    assert_eq!(names.len(), 6, "歯の数 {names:?}");
    for name in names {
        let rest = name
            .strip_prefix(prefix)
            .unwrap_or_else(|| panic!("{name} が {prefix} で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が filter の語 {word} を含む");
        }
    }
}
