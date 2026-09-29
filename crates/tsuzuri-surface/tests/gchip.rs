//! 行 c-seat-group の歯: project board の席の block の群の chip の card（見本の pa:group）は、
//! 口 /api/account の群の枠（GroupCard）を名で選んで写す（一致は電文の matches のまま・記録が在る時だけいつから）。
//! 電文が読めない・群の枠が無い時は測れていないの card。DOM は口 /api/account を 1 度読み、指を置いた時に card を組む。

use std::path::PathBuf;

use tsuzuri_contract::account::{AccountDoc, GroupCard};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::wire;
use tsuzuri_surface::account::{BAD_BODY, PATH, UNREAD};
use tsuzuri_surface::project::NOT_READ;
use tsuzuri_surface::project::seat::{
    GROUP_CARD, GROUP_MISSING, MATCHED, MEMBER_UNKNOWN, NG, OK, hmd,
};
use tsuzuri_surface::view::Fetched;
use tsuzuri_surface::vocab::label;
use tsuzuri_surface::widgets::hover::Card;

/// 描く今（電文の at と同じ・fixture の時点）。
const NOW: u64 = 1790510400;

/// 群の chip の card（描く今は電文の at）。
fn group_card(account: &Fetched, name: &str) -> Card {
    tsuzuri_surface::project::seat::group_card(account, name, NOW)
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_text() -> String {
    read("../../tests/fixtures/account/acct-doc.json")
}

fn fixture() -> AccountDoc {
    wire::decode(&fixture_text()).expect("fixture が AccountDoc として読める")
}

/// 電文を本文に持つ Fetched。
fn body(doc: &AccountDoc) -> Fetched {
    Fetched::Body(wire::encode(doc).expect("電文を字にできる"))
}

/// 名の群の枠を書き換える。
fn edit(doc: &mut AccountDoc, name: &str, f: impl FnOnce(&mut GroupCard)) {
    let Reading::Known(gs) = &mut doc.groups else {
        panic!("fixture の群の枠の列は読める");
    };
    f(gs.iter_mut()
        .find(|g| g.row.group == name)
        .expect("fixture に名の群の枠が在る"));
}

fn title(name: &str) -> String {
    format!("{} {name}", label("group"))
}

/// 測れていないの card（値だけが理由の字）。
fn unknown_card(name: &str, reason: &str) -> Card {
    Card {
        title: title(name),
        kind: label("st_unknown"),
        value: reason.to_string(),
        src: PATH.to_string(),
        more: Vec::new(),
    }
}

#[test]
fn gchip_card_copies_group() {
    assert_eq!(GROUP_CARD, "pa:group");
    assert_eq!(MATCHED, "席が一致");
    assert_eq!(MEMBER_UNKNOWN, "?");
    assert_eq!(OK.glyph, "✓");
    assert_eq!(NG.glyph, "!");
    assert_eq!(label("group"), "project の群");
    assert_eq!(label("current_account"), "今の口座");
    assert_eq!(label("seat_none"), "session なし");
    assert_eq!(label("seat_mismatch"), "移動待ち");

    let fetched = Fetched::Body(fixture_text());
    assert_eq!(
        group_card(&fetched, "Tier1"),
        Card {
            title: title("Tier1"),
            kind: format!(
                "{} acct-1 ◷ {}",
                label("current_account"),
                hmd(1790503200, 1790510400)
            ),
            value: format!("✓? 1 / 2 {MATCHED}"),
            src: "groups/Tier1.account ほか".to_string(),
            more: vec![
                "✓ proj-a acct-1".to_string(),
                format!("? proj-c {}", label("seat_none")),
            ],
        }
    );
    assert_eq!(
        group_card(&fetched, "Tier2"),
        Card {
            title: title("Tier2"),
            kind: format!("{} acct-2", label("current_account")),
            value: format!("! 0 / 1 {MATCHED}"),
            src: "groups/Tier2.account ほか".to_string(),
            more: vec![format!("! proj-b acct-1 {}", label("seat_mismatch"))],
        }
    );
}

#[test]
fn gchip_unknown_card() {
    assert_eq!(label("st_unknown"), "測れていない");
    assert_eq!(PATH, "/api/account");
    for (fetched, reason) in [
        (Fetched::NotRead, NOT_READ),
        (Fetched::Failed, UNREAD),
        (Fetched::Body("{".to_string()), BAD_BODY),
    ] {
        assert_eq!(
            group_card(&fetched, "Tier1"),
            unknown_card("Tier1", reason),
            "{fetched:?}"
        );
    }
    let fetched = Fetched::Body(fixture_text());
    for name in ["Tier9", "tier1", "Tier", ""] {
        assert_eq!(
            group_card(&fetched, name),
            unknown_card(name, GROUP_MISSING),
            "{name}"
        );
    }
    let mut doc = fixture();
    doc.groups = Reading::Unknown;
    assert_eq!(
        group_card(&body(&doc), "Tier1"),
        unknown_card("Tier1", GROUP_MISSING)
    );
}

#[test]
fn gchip_matches_copied() {
    let mut doc = fixture();
    edit(&mut doc, "Tier1", |g| {
        g.recorded = false;
        g.members[0].seat_account = Some("acct-9".to_string());
        g.members[1].seat_account = Some("acct-1".to_string());
    });
    edit(&mut doc, "Tier2", |g| {
        g.since = Some(1790503200);
        g.members[0].matches = Reading::Known(true);
    });
    let fetched = body(&doc);

    let t1 = group_card(&fetched, "Tier1");
    assert_eq!(t1.value, format!("✓? 1 / 2 {MATCHED}"));
    assert_eq!(t1.more, vec!["✓ proj-a acct-9", "? proj-c acct-1"]);
    assert_eq!(t1.kind, format!("{} acct-1", label("current_account")));

    let t2 = group_card(&fetched, "Tier2");
    assert_eq!(t2.value, format!("✓ 1 / 1 {MATCHED}"));
    assert_eq!(t2.more, vec!["✓ proj-b acct-1"]);
    assert_eq!(t2.kind, format!("{} acct-2", label("current_account")));
}

#[test]
fn gchip_dom_text() {
    let dom = read("src/project_dom/seat.rs");
    for text in [
        "crate::net::read(crate::account::PATH)",
        "account.with_untracked(|a| group_card(a, &key, now))",
        "data-card=GROUP_CARD",
        "on:mouseenter=over on:mouseleave=out",
        "{group_chip(low.group, group_doc)}",
        "crate::net::ticker()",
    ] {
        assert_eq!(dom.matches(text).count(), 1, "{text}");
    }
    assert!(!dom.contains("set_interval"));

    let src = read("src/project/seat.rs");
    let start = src.find("pub fn group_card(").expect("group_card が在る");
    let len = src[start..].find("\n}").expect("group_card の終わりが在る");
    let body = &src[start..start + len];
    assert_eq!(body.matches("crate::account::doc(account)").count(), 1);
    for text in ["decode::<", "row.account ==", "== g.row.account", "seat_account =="] {
        assert!(!body.contains(text), "{text}");
    }
}

/// 着地済みの contracts の verify の filter の語（この行の語を除く）。
const WORDS: &[&str] = &[
    "aaround_",
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
    "apop_",
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
    "cfsplit_",
    "cg9_",
    "cgdom_",
    "cmark_",
    "cnote_",
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
    "ecache_",
    "epolq_",
    "eretry_",
    "esig_",
    "evkind_",
    "flight_",
    "fmark_",
    "fprem_",
    "frame_",
    "fserve_",
    "fstop_",
    "fxpre_",
    "g3g7_",
    "gapspage_",
    "gatt_",
    "gbnote_",
    "gcoach_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gins_",
    "gjst_",
    "glabel_",
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
    "plimit_",
    "pmisfit_",
    "pmore_",
    "pquest_",
    "pqueue_",
    "project_",
    "ptitle_",
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
    "server_",
    "sesplit_",
    "shb_",
    "skeleton_",
    "smore_",
    "stage_",
    "stats_",
    "stbp_",
    "stcli_",
    "steady_",
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
    "udash_",
    "unow_",
    "urpanel_",
    "uword_",
    "wstrip_",
];

#[test]
fn gchip_own_names_clean() {
    assert_eq!(WORDS.len(), 208);
    let text = read("tests/gchip.rs");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の次の行は fn");
            &rest[..rest.find('(').expect("fn の名の後に (")]
        })
        .collect();
    assert_eq!(names.len(), 5, "{names:?}");
    for name in names {
        let rest = name
            .strip_prefix("gchip_")
            .unwrap_or_else(|| panic!("{name} は gchip_ で始まる"));
        for w in WORDS {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
