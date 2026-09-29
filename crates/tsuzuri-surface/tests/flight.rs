//! 行 g-reads の歯: 口ごとの読みの印（flight の Flight）の決め方と、net がそれを呼ぶ字の並び。
//! 読みの途中の呼びは読みの後の 1 回にまとめ、見ている部品が無い間の呼びは部品が戻ったときの 1 回にまとめる。
//! 最初の読みは知らせの接続が開くまで待つ（上限 OPEN_WAIT_MS）。

use std::path::PathBuf;

use tsuzuri_surface::flight::{Flight, OPEN_WAIT_MS};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// net.rs の関数の本文（`fn name(` から次の行頭の `fn `・`pub fn `・`async fn `・`pub async fn ` まで）。
fn function<'a>(text: &'a str, name: &str) -> &'a str {
    let start = text
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("net に fn {name} が在る"));
    let rest = &text[start..];
    let end = ["\nfn ", "\npub fn ", "\nasync fn ", "\npub async fn "]
        .iter()
        .filter_map(|m| rest[1..].find(m).map(|i| i + 1))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

/// (1) 読みの途中の呼びは何度来ても読みの後の 1 回にまとまる。
#[test]
fn flight_one_more_after_busy() {
    let mut f = Flight::default();
    assert!(!f.attach());
    assert_eq!(f.users(), 1);
    assert!(!f.busy());
    assert!(f.start());
    assert!(f.busy());
    assert!(!f.start());
    assert!(!f.call());
    assert!(f.end(), "読みの途中の呼びの後にもう 1 回読む");
    assert!(f.busy());
    assert!(!f.end());
    assert!(!f.busy());
    assert!(f.call());
    assert!(!f.end());
}

/// (2) 見ている部品が無い間の呼びは読まず、部品が戻ったときの 1 回にまとまる。呼びが無ければ戻っても読まない。
#[test]
fn flight_unwatched_waits_for_user() {
    let mut f = Flight::default();
    f.attach();
    f.detach();
    assert_eq!(f.users(), 0);
    assert!(!f.call());
    assert!(!f.call());
    assert!(!f.busy());
    assert!(f.attach(), "見ていない間の呼びは戻ったときに読む");
    assert!(f.busy());
    assert!(!f.end());
    f.detach();
    assert!(!f.attach(), "呼びが無ければ戻っても読まない");
    f.detach();
    f.detach();
    assert_eq!(f.users(), 0);
}

/// (3) 読みの途中の呼びの後に部品が片付けば、読みの終わりで続けずに部品が戻ったときに読む。
/// 読みの途中に片付いて呼びが来た後に部品が戻れば、読みの終わりで続けて 1 回読む。
#[test]
fn flight_again_waits_when_unwatched() {
    let mut f = Flight::default();
    f.attach();
    assert!(f.start());
    assert!(!f.call());
    f.detach();
    assert!(!f.end());
    assert!(!f.busy());
    assert!(f.attach());

    let mut g = Flight::default();
    g.attach();
    assert!(g.start());
    g.detach();
    assert!(!g.call());
    assert!(!g.attach(), "読みの途中なので重ねない");
    assert!(g.end());
    assert!(!g.end());
}

/// (4) path が変わると読みの途中を捨てて round を進める（見ている部品の数は変えない）。
#[test]
fn flight_renew_drops_flight() {
    let mut f = Flight::default();
    f.attach();
    assert!(f.start());
    assert!(!f.call());
    assert_eq!(f.round(), 0);
    assert_eq!(f.renew(), 1);
    assert_eq!(f.round(), 1);
    assert!(!f.busy());
    assert_eq!(f.users(), 1);
    assert!(f.start());
    assert!(!f.end(), "前の path の呼びは続けない");
}

/// (5) net は最初の読みを接続の開きまで待ち、口ごとに読みの印を呼ぶ（字の並び）。
#[test]
fn flight_net_wiring_text() {
    assert_eq!(OPEN_WAIT_MS, 1000u64);
    let lib = read("src/lib.rs");
    let lines: Vec<&str> = lib.lines().map(str::trim).collect();
    let at = lines
        .iter()
        .position(|l| *l == "pub mod flight;")
        .expect("lib に pub mod flight; が在る");
    assert!(at > 0, "pub mod flight; の前の行が無い");
    assert!(
        !lines[at - 1].contains("#[cfg(target_arch = \"wasm32\")]"),
        "flight が wasm の target だけで組まれる"
    );

    let net = read("src/net.rs");
    let has = |name: &str, words: &[&str]| {
        let body = function(&net, name);
        for w in words {
            assert!(body.contains(w), "{name} に {w} が無い: {body}");
        }
    };
    has("read", &["LIVE.get()", ".attach()", "on_cleanup(", ".detach()"]);
    has(
        "read_path",
        &["LIVE.get()", ".attach()", ".renew()", "on_cleanup(", ".detach()"],
    );
    has("load_at", &[".end()"]);
    has("load_watch", &[".end()", ".round()"]);
    has("go_live", &["LIVE.set(true)", "reload_all()"]);
    has("connect", &["set_timeout(", "OPEN_WAIT_MS"]);

    let reload = function(&net, "reload_if");
    assert!(reload.matches(".call()").count() >= 2, "{reload}");
    let connect = function(&net, "connect");
    assert_eq!(connect.matches("new(go_live)").count(), 1, "{connect}");
    assert_eq!(connect.matches("new(reload_changed)").count(), 1, "{connect}");

    let watch = &net[net.find("fn load_watch(").expect("fn load_watch")..];
    let spawn = watch.find("spawn_local(").expect("load_watch の spawn_local");
    assert!(
        watch[..spawn].contains("if path.is_empty() {"),
        "load_watch が空の path を読む前に戻らない"
    );
}

/// (7) この file の歯の名はどれも flight_ で始まり、残りの字は着地済みの行の verify の語を含まない。
#[test]
fn flight_own_names_clean() {
    let text = read("tests/flight.rs");
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
    assert_eq!(names.len(), 6, "{names:?}");
    let words = [
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
        "qgate_",
        "nsum_",
        "hcard_",
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
        "fstop_",
        "nsumw_",
        "fmark_",
        "fserve_",
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
        "uword_",
        "cround_",
        "cgdom_",
        "csled_",
        "lhome_",
        "shb_",
        "ghb_",
        "nact_",
        "aord_",
        "mtree_",
        "rhold_",
        "qkey_",
        "wstrip_",
    ];
    assert_eq!(words.len(), 110);
    for name in &names {
        let rest = name
            .strip_prefix("flight_")
            .unwrap_or_else(|| panic!("{name} が flight_ で始まらない"));
        for w in words {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
