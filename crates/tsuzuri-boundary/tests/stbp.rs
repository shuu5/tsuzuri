//! ほかの project の board の守りの歯（行 i-board-ports・接頭辞 stbp_）。
//! 群の宣言は test の中の字で、偽の git は sh の script（argv を file に足す・本物の git と tailnet の道具を撃たない）。
#![cfg(test)]

use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tsuzuri_boundary::stage::cdp::Command;
use tsuzuri_boundary::stage::cli::{on_board, on_boards};
use tsuzuri_boundary::stage::url::{Board, Ports, board_url, ports, self_hosts};

/// contracts の verify の filter の語のうち、ほかの語を部分の字として含まない語と、計画の後の行 g-gz の接頭辞 pgz_ と、
/// 同じノートのもう 1 つの行の接頭辞 sttgt_（この行の接頭辞 stbp_ は並べない）。
const FILTERS: [&str; 171] = [
    "aaround_", "accept_", "acchold_", "account_", "acctcore_", "acctdoc_", "accthb_",
    "accthome_", "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_",
    "acctwire_", "afocus_", "aord_", "apop_", "askcard_", "athr_", "batchpanel_", "bhalf_",
    "board_min_", "bport_", "brand_", "btuck_", "cadl_", "cadopt_", "cadq_", "cdorm_",
    "cfsplit_", "cgdom_", "cmark_", "contract_form_", "cround_", "csled_", "cspk_", "ctick_",
    "denv_", "dnedge_", "dngrp_", "dnrow_", "dnskip_", "ecache_", "epolq_", "flight_",
    "fmark_", "fprem_", "frame_", "fserve_", "fstop_", "fxpre_", "g3g7_", "gapspage_",
    "gbnote_", "gcoach_", "gfix_", "gfresh_", "ghb_", "gjst_", "glabel_", "gnav_", "gpface_",
    "gpill_", "gpulse_", "graph_", "gsum_", "gtuck_", "gview_", "hacols_", "harest_",
    "hasplit_", "hbconf_", "hbmark_", "hbpost_", "hbproc_", "hbroute_", "hcard_", "hcled_",
    "hcnx_", "hcproj_", "hcsess_", "hdchip_", "hfig_", "hnunk_", "hook_", "hruling_",
    "hsblock_", "hsderive_", "hshist_", "hspage_", "hspk_", "hsym_", "hwstore_", "iclose_",
    "ilink_", "kcli_", "kindlab_", "klink_", "launch_", "lcard_", "ledgerblock_", "lhome_",
    "lidle_", "lresume_", "lsnap_", "lspark_", "lstg_", "lstore_", "mapview_", "mkeys_",
    "mlink_", "mstore_", "mtips_", "mtree_", "nact_", "nbatch_", "ncard_", "nextstep_",
    "nodepage_", "nstall_", "nsum_", "nsumw_", "ntime_", "nxact_", "nxorg_", "parts_", "pci_",
    "pclosed_", "pfold_", "pipe_", "plimit_", "pmisfit_", "pmore_", "pquest_", "pqueue_",
    "project_", "ptitle_", "punmap_", "pwhole_", "qblock_", "qgate_", "qkey_", "question_",
    "relay_", "rhold_", "runsdoc_", "rvk_", "saxis_", "sclosed_", "seatblock_", "seatcard_",
    "server_", "sesplit_", "shb_", "skeleton_", "smore_", "stage_", "stats_", "stcli_",
    "steady_", "sxaxis_", "ticker_", "tipx_", "topbar_", "tz_", "urpanel_", "uword_",
    "wstrip_", "pgz_", "sttgt_",
];

/// 節の偽の tailnet の道具の出力（住所は文書の例の住所）。
const STATUS: &str = r#"{
  "BackendState": "Running",
  "Self": {
    "HostName": "srv-a",
    "DNSName": "SRV-A.tailnet.invalid.",
    "TailscaleIPs": [
      "192.0.2.7",
      "2001:DB8::7"
    ]
  },
  "Peer": {}
}
"#;

/// 節の群の宣言（grp-a は 1 行の配列・grp-b は複数の行の配列・grp-c は anchors 無し）。
const FACE: &str = r#"schema = 1

[[account-group]]
name = "grp-a"
anchors = ["/fake/anchors/proj-a", "/fake/anchors/proj-b"]

[[account-group]]
name = "grp-b"
anchors = [
  "/fake/anchors/proj-a",
  "/fake/anchors/proj-c",
  "/fake/anchors/proj-d",
]

[[account-group]]
name = "grp-c"
"#;

/// 節の Board（port 4801・host の列は節の status の self_hosts）。
fn own() -> Board {
    let hosts = self_hosts(STATUS).expect("節の status の host の列");
    Board {
        url: board_url(&hosts[0], 4801),
        hosts,
        port: 4801,
    }
}

fn src(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/stage")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 歯ごとの作業場（前の撃ちの残りを消す）。
fn scratch(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("stbp")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("作業場");
    root
}

#[test]
fn stbp_holds_any() {
    let holds_any: fn(&Board, &str, &[u16]) -> bool = Board::holds_any;
    let _ = holds_any;
    let own = own();
    let others: &[u16] = &[4802, 4803];
    let table: [(&str, bool, bool); 9] = [
        ("http://srv-a.tailnet.invalid:4801/x", true, true),
        ("http://srv-a:4802/", false, true),
        ("https://192.0.2.7:4803/p?q=1", false, true),
        ("http://[2001:db8::7]:4802/", false, true),
        ("http://localhost:4803/", false, true),
        ("http://127.0.0.1:4804/", false, false),
        ("http://other.tailnet.invalid:4802/", false, false),
        ("http://srv-a:80/", false, false),
        ("file:///fake/x.html", false, false),
    ];
    for (page, alone, with) in table {
        assert_eq!(own.holds(page), alone, "holds {page}");
        assert_eq!(own.holds_any(page, &[]), alone, "holds_any 空 {page}");
        assert_eq!(own.holds_any(page, others), with, "holds_any 在り {page}");
    }
}

#[test]
fn stbp_ports_read() {
    let ports_fn: fn(&str, &OsStr, &Path, Duration) -> Ports = ports;
    let _ = ports_fn;
    let root = scratch("read");
    let log = root.join("argv");
    let git = root.join("git");
    fs::write(
        &git,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\ncase \"$2\" in\n  */proj-a) printf '4802\\n' ;;\n  */proj-b) printf '4803\\n' ;;\n  */proj-c) printf 'x\\n' ;;\n  *) exit 1 ;;\nesac\n",
            log.display()
        ),
    )
    .expect("偽の git");
    fs::set_permissions(&git, fs::Permissions::from_mode(0o755)).expect("偽の git の権限");
    let timeout = Duration::from_secs(10);

    let read = ports(FACE, git.as_os_str(), &root, timeout);
    assert_eq!(
        read,
        Ports {
            ports: vec![4802, 4803],
            unread: vec!["proj-c".to_string(), "proj-d".to_string()],
        }
    );
    let argv = fs::read_to_string(&log).expect("偽の git の argv");
    let want: Vec<String> = ["proj-a", "proj-b", "proj-c", "proj-d"]
        .iter()
        .map(|p| format!("-C /fake/anchors/{p} config --get tsuzuri.boardport"))
        .collect();
    assert_eq!(argv.lines().collect::<Vec<_>>(), want);

    let absent = root.join("absent");
    assert_eq!(
        ports("schema = 1\n", absent.as_os_str(), &root, timeout),
        Ports::default()
    );
}

#[test]
fn stbp_on_boards() {
    let on_boards_fn: fn(&Command, &str, &Board, &[u16]) -> bool = on_boards;
    let _ = on_boards_fn;
    let own = own();
    let page = "http://srv-a:4802/";
    let others: &[u16] = &[4802];
    let guarded = [
        Command::Click { x: 1, y: 2 },
        Command::Type {
            text: "a".to_string(),
        },
        Command::Key {
            key: "Enter".to_string(),
        },
    ];
    for command in &guarded {
        assert!(on_boards(command, page, &own, others), "{command:?}");
        assert!(!on_board(command, page, &own), "on_board {command:?}");
        assert!(!on_boards(command, page, &own, &[]), "空 {command:?}");
        assert!(
            !on_boards(command, "http://srv-a:4809/", &own, others),
            "4809 {command:?}"
        );
    }
    let free = [
        Command::Navigate {
            url: "http://srv-a:4802/next".to_string(),
        },
        Command::Reload,
    ];
    for command in &free {
        assert!(!on_boards(command, page, &own, others), "{command:?}");
    }
}

#[test]
fn stbp_wiring() {
    let cli_text = src("cli.rs");
    for line in [
        "let read = url::ports(&text, &call.git, &call.repo, TIMEOUT);",
        "if !script.is_empty() {",
        "emit(&format!(\"project {name} の board の port が読めない（その board の頁の上の断りは広げない）\"));",
        "if on_boards(command, &page, board, ports) {",
        "shoot(&mut session, command, out.as_deref(), board, ports)",
    ] {
        assert_eq!(cli_text.matches(line).count(), 1, "{line}");
    }
    assert_eq!(
        cli_text.matches("drive(session, script, board, &ports)").count(),
        3
    );
    assert!(!cli_text.contains("board.holds(page)"));
    let url_text = src("url.rs");
    assert_eq!(url_text.matches("pub fn ports(").count(), 1);
    assert_eq!(url_text.matches("acct::BOARD_ARGS").count(), 2);
}

#[test]
fn stbp_names_apart() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/stbp.rs");
    let text = fs::read_to_string(&path).expect("tests/stbp.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 5, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("stbp_")
            .unwrap_or_else(|| panic!("{name} は stbp_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
