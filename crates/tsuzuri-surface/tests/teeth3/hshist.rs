//! 行 h-sort-hist の歯: account board の 3 つの表の並べ方の押しを履歴に積む（見本の pushState）。
//! session の幅の押しは置き換えのまま（見本の setSpan）で、戻ると進むは board の popstate が block を組み直す。
//! session の止まった run の段の doc は見本の runStopped の写しと言い、値は変えない。
//! DOM は wasm の target のときだけなので src の字を読む。
#![cfg(test)]

use std::path::PathBuf;

use tsuzuri_contract::board::Stage;
use tsuzuri_surface::account::session::STALLED_STAGES;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// 空白を除いた字。
fn squash(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// file の DOM の部分（字 mod dom と開き波括弧より後）。
fn dom_part(text: &str) -> String {
    let at = text.find("mod dom {").expect("mod dom が在る");
    text[at + "mod dom {".len()..].to_string()
}

/// DOM の部分の字 fn と名と開き括弧から、次の行頭 4 空白の fn か pub fn の宣言の前まで。
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

/// 3 つの表の file と、並べの押しで URL を作る関数の名。
const SORTED: [(&str, &str); 3] = [
    ("session", "with_sort"),
    ("ledger", "with_sort"),
    ("projects", "with_psort"),
];

/// (1) 3 つの file の sort_bar の押しは、今の並べなら返り、ほかは push に URL を渡してから signal を置く。
/// fn push は history の push_state_with_url を撃ち、置き換えを撃たない。
#[test]
fn hshist_sort_pushes() {
    for (module, with) in SORTED {
        let dom = dom_part(&read(&format!("src/account/{module}.rs")));
        let bar = fn_body(&dom, "sort_bar");
        let pick = format!(
            "letpick=move|_|{{ifsort.get_untracked()==s{{return;}}push(&{with}(&search(),s));sort.set(s);}};"
        );
        assert_eq!(
            squash(&bar).matches(&pick).count(),
            1,
            "{module}.rs の sort_bar の押し"
        );
        assert!(
            !bar.contains("replace("),
            "{module}.rs の sort_bar が replace( を持つ"
        );

        let push = fn_body(&dom, "push");
        assert!(
            squash(&push).starts_with("fnpush(url:&str){"),
            "{module}.rs の fn push の宣言"
        );
        assert_eq!(
            push.matches("history.push_state_with_url(").count(),
            1,
            "{module}.rs の fn push の push_state_with_url"
        );
        assert!(
            !push.contains("replace_state"),
            "{module}.rs の fn push が replace_state を持つ"
        );
    }
}

/// (2) ledger.rs と projects.rs は置き換えを持たず、session.rs の幅の押しは置き換えのまま。
/// board は popstate で query を置き、main の中の閉包が今の tab の block を組み直す。
#[test]
fn hshist_span_and_back() {
    for module in ["ledger", "projects"] {
        let text = read(&format!("src/account/{module}.rs"));
        for w in ["replace_state_with_url(", "fn replace("] {
            assert!(!text.contains(w), "{module}.rs が {w} を持つ");
        }
    }

    let session = read("src/account/session.rs");
    for w in ["replace_state_with_url(", "push_state_with_url("] {
        assert_eq!(session.matches(w).count(), 1, "session.rs の {w}");
    }
    let span = fn_body(&dom_part(&session), "span_bar");
    assert_eq!(
        squash(&span)
            .matches("replace(&with_span(&search(),s));")
            .count(),
        1,
        "session.rs の span_bar の押し"
    );
    assert!(!span.contains("push("), "session.rs の span_bar が push( を持つ");

    let board = squash(&read("src/account/board.rs"));
    for w in [
        "window_event_listener(ev::popstate,move|_|{letnow=search();",
        "query.set(now);",
        "{move||page_view(query.with(|q|Tab::from_query(q)))}",
    ] {
        assert!(board.contains(w), "board.rs が {w} を持たない");
    }
}

/// (3) 面の止まった run の段は 3 つのままで、doc は見本の runStopped の写しと言う。
/// 中核の next_step の段は 2 つ（行 c-next-stall）。
#[test]
fn hshist_stalled_doc() {
    assert_eq!(
        STALLED_STAGES,
        [Stage::Questioned, Stage::Failed, Stage::Stopped]
    );

    let session = read("src/account/session.rs");
    let lines: Vec<&str> = session.lines().collect();
    let decl = lines
        .iter()
        .position(|l| l.starts_with("pub const STALLED_STAGES"))
        .expect("session.rs に STALLED_STAGES の宣言が在る");
    let doc: Vec<&str> = lines[..decl]
        .iter()
        .rev()
        .take_while(|l| l.starts_with("///"))
        .copied()
        .collect();
    assert!(!doc.is_empty(), "STALLED_STAGES の宣言の直前に doc が無い");
    let doc = doc.join("\n");
    assert!(
        doc.contains("見本の acct.js の runStopped の段の写し"),
        "{doc}"
    );
    assert!(!doc.contains("STALLED_STAGES の写し"), "{doc}");

    let core = read("../tsuzuri-core/src/next_step.rs");
    assert_eq!(
        core.matches("pub const STALLED_STAGES: [Stage; 2] = [Stage::Failed, Stage::Stopped];")
            .count(),
        1
    );
}

/// filter の語（main の verify の filter の語を畳んだ語と、並行の起草の行と計画の後の行の接頭辞）。
const FILTERS: &[&str] = &[
    "aaround_",
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
    "cdorm_",
    "cgdom_",
    "cmark_",
    "contract_form_",
    "cround_",
    "csled_",
    "denv_",
    "dnrow_",
    "ecache_",
    "epolq_",
    "flight_",
    "fmark_",
    "frame_",
    "fserve_",
    "fstop_",
    "gapspage_",
    "gbnote_",
    "gfix_",
    "gfresh_",
    "ghb_",
    "gjst_",
    "glabel_",
    "gnav_",
    "gpill_",
    "gpulse_",
    "graph_",
    "gsum_",
    "gtuck_",
    "gview_",
    "hacols_",
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
    "ntime_",
    "nxact_",
    "parts_",
    "pclosed_",
    "pfold_",
    "pipe_",
    "plimit_",
    "pmisfit_",
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
    "fprem_",
    "gpface_",
    "dnkind_",
];

/// (4) この file の歯の名は 4 つで hshist_ で始まり、名の全体は filter の語を含まない。
#[test]
fn hshist_own_names_clean() {
    assert_eq!(FILTERS.len(), 149);
    let text = read("tests/teeth3/hshist.rs");
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
    assert_eq!(names.len(), 4, "{names:?}");
    for name in names {
        assert!(
            name.starts_with("hshist_"),
            "歯の名 {name} が hshist_ で始まらない"
        );
        for w in FILTERS {
            assert!(!name.contains(w), "歯の名 {name} が filter の語 {w} を含む");
        }
    }
}
