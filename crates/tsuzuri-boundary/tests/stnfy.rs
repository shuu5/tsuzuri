//! 知らせの歯（行 i-10・接頭辞 stnfy_）。
//! host の面は tests/fixtures/stage/terminals.toml に macos の term-m と windows の term-w の行を足した字（名も宛先も path も偽物）。
//! 偽の器・git・tailnet の道具・ssh は sh の script で、偽の ssh は argv を記して最後の引数を環境を空にした sh で撃ち、
//! その PATH は場の far の dir だけ（歯が置く偽の notify-send と env の symlink）。本物の ssh と notify-send と器を撃たず網に出ない。
#![cfg(test)]

use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command as Process, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tsuzuri_boundary::stage::cli::USAGE;
use tsuzuri_boundary::stage::launch;
use tsuzuri_boundary::stage::notify::{
    self, ABSENT, DIR, NotifyCall, Outcome, PROGRAM, Record, TITLE_MAX, UNREACHED,
};
use tsuzuri_boundary::stage::target::{self, Targets};
use tsuzuri_boundary::stage::terminal::{self, FACE, Terminal};

/// main 8f8eeb8 の contracts の verify の filter の語を、ほかの語を部分の字として含まない語に畳んだ語
/// （この行の接頭辞 stnfy_ は並べない）。
const FILTERS: [&str; 202] = [
    "aaround_", "accept_", "acchold_", "account_", "acctcore_", "acctdoc_", "accthb_",
    "accthome_", "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_", "acctwin_",
    "acctwire_", "aface_", "afocus_", "alean_", "aord_", "aown_", "apop_", "askcard_", "athr_",
    "batchpanel_", "bhalf_", "board_min_", "bport_", "brand_", "btuck_", "cadl_", "cadopt_",
    "cadq_", "cdorm_", "cfsplit_", "cg9_", "cgdom_", "cmark_", "cnote_", "contract_form_",
    "cround_", "csled_", "cspk_", "ctick_", "cupd_", "cupdlist_", "denv_", "dnedge_", "dngrp_",
    "dnrow_", "dnskip_", "ecache_", "epolq_", "eretry_", "esig_", "evkind_", "flight_",
    "fmark_", "fprem_", "frame_", "fserve_", "fstop_", "fxpre_", "g3g7_", "gapspage_", "gatt_",
    "gbnote_", "gcoach_", "gfix_", "gfresh_", "ghb_", "gins_", "gjst_", "glabel_", "gnav_",
    "gpface_", "gpill_", "gpulse_", "graph_", "gsum_", "gtuck_", "gview_", "gwv_", "hacols_",
    "harest_", "hasplit_", "hbconf_", "hbmark_", "hbpost_", "hbproc_", "hbroute_", "hcard_",
    "hcled_", "hcnx_", "hcproj_", "hcsess_", "hdchip_", "hfig_", "hnunk_", "hook_", "hruling_",
    "hsblock_", "hsderive_", "hshist_", "hspage_", "hspk_", "hsym_", "hthr_", "http_",
    "hwstore_", "iclose_", "ilink_", "jcount_", "jrun_", "kcli_", "kg9_", "kindlab_", "klink_",
    "ksum_", "launch_", "lcard_", "ledgerblock_", "lgrp_", "lhome_", "lidle_", "lkind_",
    "lresume_", "lsnap_", "lspark_", "lstg_", "lstore_", "mapview_", "mkeys_", "mlink_",
    "mstore_", "mtips_", "mtree_", "nact_", "nbatch_", "ncard_", "nextstep_", "nodepage_",
    "nstall_", "nsum_", "nsumw_", "ntime_", "nxact_", "nxorg_", "parts_", "pci_", "pclosed_",
    "pfold_", "pgsw_", "pgz_", "pipe_", "plimit_", "pmisfit_", "pmore_", "pquest_", "pqueue_",
    "project_", "ptitle_", "pubscan_", "punmap_", "pwhole_", "qblock_", "qgate_", "qkey_",
    "question_", "rbusy_", "relay_", "rhold_", "runsdoc_", "rvk_", "saxis_", "sclosed_",
    "seatblock_", "seatcard_", "server::events::tests::", "server_", "sesplit_", "shb_",
    "skeleton_", "smore_", "stage_", "stats_", "stbp_", "stcli_", "steady_", "stskill_",
    "sttgt_", "sxaxis_", "ticker_", "tipx_", "tkad_", "tlic_", "topbar_", "topfit_", "tz_", "unow_",
    "urpanel_", "uword_", "wstrip_",
];

/// 節の題（空白と一重と二重の引用符を含む）。
const TITLE: &str = "問い 2 つ it's \"見て\"";

/// 節の board の URL（名は偽物・予約の頂の invalid）。
const URL: &str = "http://srv-a.tailnet.invalid:4801/";

/// 節の URL の行。
const LINE: &str = "board の URL http://srv-a.tailnet.invalid:4801/";

/// 節の偽の tailnet の道具の出力（住所は文書の例の住所）。
const STATUS: &str = r#"{
  "BackendState": "Running",
  "Self": {
    "HostName": "srv-a",
    "DNSName": "SRV-A.tailnet.invalid.",
    "TailscaleIPs": ["192.0.2.7"]
  },
  "Peer": {}
}
"#;

/// fixture に足す macos と windows の端末の行。
const EXTRA: &str = r#"
[[device]]
name = "term-m"
ssh = "me@term-m"
chrome = "/fake/bin/chrome"
os = "macos"
profile-dir = "/fake/profile-m"

[[device]]
name = "term-w"
ssh = "me@term-w"
chrome = "/fake/bin/chrome"
os = "windows"
profile-dir = "/fake/profile-w"
"#;

/// 節の term-a の 9 語目（遠くの shell の 1 語）。
const FAR_A: &str = r#"command -v notify-send >/dev/null 2>&1 || exit 127; exec 'env' 'DISPLAY=:0' 'notify-send' '問い 2 つ it'\''s "見て"' 'http://srv-a.tailnet.invalid:4801/' </dev/null >/dev/null 2>&1"#;

/// 節の term-b と題 見て の 9 語目。
const FAR_B: &str = "command -v notify-send >/dev/null 2>&1 || exit 127; exec 'env' 'WAYLAND_DISPLAY=wayland-1' 'XDG_RUNTIME_DIR=/fake/run-b' 'notify-send' '見て' 'http://srv-a.tailnet.invalid:4801/' </dev/null >/dev/null 2>&1";

/// 偽の ssh の記録の 1 度の撃ちの区切りの行。
const CUT: &str = "----";

fn fixture_text(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/stage")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 節の host の面（fixture に term-m と term-w を足した字）。
fn face() -> String {
    format!("{}{EXTRA}", fixture_text("terminals.toml"))
}

fn term(name: &str) -> Terminal {
    terminal::lookup(&face(), name).unwrap_or_else(|e| panic!("{name}: {e}"))
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
        .join("stnfy")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("作業場");
    root
}

/// 権限の下 9 bit。
fn mode(path: &Path) -> u32 {
    fs::metadata(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .permissions()
        .mode()
        & 0o777
}

/// Err の字（Ok なら落ちる）。
fn err<T: Debug>(got: Result<T, String>, what: &str) -> String {
    match got {
        Ok(v) => panic!("{what}: Ok {v:?}"),
        Err(e) => e,
    }
}

/// 偽の program（sh の script・権限 0755）。
fn fake(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}")).expect("偽の program");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
    path
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("epoch 秒")
        .as_secs()
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

fn debug<T: Debug + Clone + PartialEq + Eq>() {}

/// 節の記録。
fn record(title: &str) -> Record {
    Record {
        at: 1_790_000_000,
        project: "proj-a".to_string(),
        title: title.to_string(),
        url: URL.to_string(),
    }
}

/// 偽の場（dir の名が proj-a の repo・節の host の面・偽の器と git と tailnet の道具と ssh・far の dir・state の dir）。
struct Field {
    root: PathBuf,
    repo: PathBuf,
    scribe2: PathBuf,
    git: PathBuf,
    tailnet: PathBuf,
    ssh: PathBuf,
    far: PathBuf,
    config: PathBuf,
    broken: PathBuf,
    down: PathBuf,
    unreach: PathBuf,
    log: PathBuf,
    seen: PathBuf,
}

impl Field {
    fn new(name: &str) -> Field {
        let root = scratch(name);
        let repo = root.join("proj-a");
        let state = root.join("state");
        let far = root.join("far");
        for dir in [&repo, &state, &far] {
            fs::create_dir_all(dir).expect("場の dir");
        }
        fs::write(state.join(FACE), face()).expect("host の面");
        symlink("/usr/bin/env", far.join("env")).expect("far の env");
        let status = root.join("status");
        fs::write(&status, STATUS).expect("場の status");
        let broken = root.join("broken");
        let down = root.join("down");
        let unreach = root.join("unreach");
        let log = root.join("ssh.log");
        let scribe2 = fake(
            &root,
            "scribe2",
            &format!("if [ -e '{}' ]; then exit 1; fi\nexit 0\n", broken.display()),
        );
        let git = fake(
            &root,
            "git",
            &format!(
                "case \"$5\" in\n  scribe2.statedir) printf '%s\\n' '{}' ;;\n  tsuzuri.boardport) printf '4801\\n' ;;\n  *) exit 1 ;;\nesac\n",
                state.display()
            ),
        );
        let tailnet = fake(
            &root,
            "tailnet",
            &format!(
                "if [ -e '{}' ]; then exit 1; fi\nexec cat '{}'\n",
                down.display(),
                status.display()
            ),
        );
        let ssh = fake(
            &root,
            "ssh",
            &format!(
                "{{ printf '%s\\n' \"$@\"; printf '%s\\n' '{CUT}'; }} >> '{}'\nif [ -e '{}' ]; then exit 255; fi\nfor last; do :; done\nexec /usr/bin/env -i PATH='{}' /bin/sh -c \"$last\"\n",
                log.display(),
                unreach.display(),
                far.display()
            ),
        );
        Field {
            seen: root.join("seen"),
            config: root.join("cfg").join(target::DIR).join(target::FILE),
            root,
            repo,
            scribe2,
            git,
            tailnet,
            ssh,
            far,
            broken,
            down,
            unreach,
            log,
        }
    }

    /// far の dir に rc で終わる偽の notify-send を置く（引数と画面の env を場の file に記す）。
    fn notify_send(&self, rc: i32) {
        fake(
            &self.far,
            PROGRAM,
            &format!(
                "{{ printf '%s\\n' \"$@\"; printf 'DISPLAY=%s\\n' \"$DISPLAY\"; printf 'WAYLAND_DISPLAY=%s\\n' \"$WAYLAND_DISPLAY\"; printf 'XDG_RUNTIME_DIR=%s\\n' \"$XDG_RUNTIME_DIR\"; }} > '{}'\nexit {rc}\n",
                self.seen.display()
            ),
        );
    }

    /// 場の XDG_STATE_HOME と HOME で tz stage notify を撃つ。
    fn tz(&self, args: &[&str]) -> (i32, Vec<String>, String) {
        let xstate = self.root.join("xstate");
        let home = self.root.join("home");
        self.tz_env(args, xstate.as_os_str(), home.as_os_str())
    }

    /// tz stage notify を撃つ（席の中の印を立てる）。rc と標準出力の行と標準エラーの字を返す。
    fn tz_env(&self, args: &[&str], xstate: &OsStr, home: &OsStr) -> (i32, Vec<String>, String) {
        let out = Process::new(env!("CARGO_BIN_EXE_tz"))
            .arg("stage")
            .arg("notify")
            .args(args)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--ssh")
            .arg(&self.ssh)
            .arg("--scribe2")
            .arg(&self.scribe2)
            .arg("--git")
            .arg(&self.git)
            .arg("--tailnet")
            .arg(&self.tailnet)
            .arg("--config")
            .arg(&self.config)
            .env("CLAUDECODE", "1")
            .env("XDG_STATE_HOME", xstate)
            .env("HOME", home)
            .stdin(Stdio::null())
            .output()
            .expect("tz を撃つ");
        let text = String::from_utf8(out.stdout).expect("tz の標準出力の字");
        (
            out.status.code().unwrap_or(-1),
            text.lines().map(str::to_string).collect(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    /// 記録の file の path。
    fn record_path(&self) -> PathBuf {
        self.root.join("xstate").join(DIR).join("proj-a.json")
    }

    /// 記録（無いか読めなければ None）。
    fn record(&self) -> Option<Record> {
        fs::read_to_string(self.record_path())
            .ok()
            .and_then(|t| notify::read(&t))
    }

    /// 偽の ssh の撃ちごとの argv。
    fn shots(&self) -> Vec<Vec<String>> {
        let text = fs::read_to_string(&self.log).unwrap_or_default();
        let mut out = Vec::new();
        let mut shot = Vec::new();
        for line in text.lines() {
            if line == CUT {
                out.push(std::mem::take(&mut shot));
            } else {
                shot.push(line.to_string());
            }
        }
        out
    }

    /// 偽の notify-send が記した行（引数と画面の env）。
    fn seen(&self) -> Vec<String> {
        fs::read_to_string(&self.seen)
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }
}

#[test]
fn stnfy_shape_and_consts() {
    let program: &str = PROGRAM;
    let absent: i32 = ABSENT;
    let unreached: i32 = UNREACHED;
    let title_max: usize = TITLE_MAX;
    let dir: &str = DIR;
    assert_eq!(
        (program, absent, unreached, title_max, dir),
        ("notify-send", 127, 255, 200, "tsuzuri/notify")
    );
    debug::<NotifyCall>();
    debug::<Outcome>();
    debug::<Record>();
    let title_shaped: fn(&str) -> bool = notify::title_shaped;
    let parse: fn(&[&str]) -> Result<NotifyCall, String> = notify::parse;
    let notify_env: fn(&Terminal) -> Vec<(String, String)> = notify::notify_env;
    let argv: fn(&Terminal, &str, &str) -> Result<Vec<String>, String> = notify::argv;
    let outcome: fn(i32) -> Outcome = notify::outcome;
    type SendFn = fn(&OsStr, &Terminal, &str, &str, Duration) -> Result<Outcome, String>;
    let send: SendFn = notify::send;
    let line: fn(&str, &Outcome) -> String = notify::line;
    let path: fn(Option<&OsStr>, Option<&OsStr>, &str) -> Option<PathBuf> = notify::path;
    let render: fn(&Record) -> String = notify::render;
    let read: fn(&str) -> Option<Record> = notify::read;
    let save: fn(&Path, &Record) -> Result<(), String> = notify::save;
    let _ = (
        title_shaped,
        parse,
        notify_env,
        argv,
        send,
        line,
        path,
        render,
        read,
        save,
    );

    let table: [(i32, Outcome); 5] = [
        (0, Outcome::Sent),
        (127, Outcome::Absent),
        (255, Outcome::Unreached("ssh の rc 255".to_string())),
        (1, Outcome::Failed(1)),
        (2, Outcome::Failed(2)),
    ];
    for (rc, want) in table {
        assert_eq!(outcome(rc), want, "{rc}");
    }
    let lines: [(Outcome, &str); 4] = [
        (
            Outcome::Sent,
            "端末 term-a に知らせを出した（notify-send・窓は起こさず前に出さない）",
        ),
        (
            Outcome::Absent,
            "端末 term-a に notify-send が無い（知らせを出せない・持ち主へは board の URL を渡す）",
        ),
        (
            Outcome::Failed(1),
            "端末 term-a の notify-send が rc 1 で終わった（知らせを出せない・画面の env と session の bus を確かめる・持ち主へは board の URL を渡す）",
        ),
        (
            Outcome::Unreached("ssh の rc 255".to_string()),
            "端末 term-a に届かない（ssh の rc 255・持ち主へは board の URL を渡す）",
        ),
    ];
    for (outcome, want) in lines {
        assert_eq!(line("term-a", &outcome), want, "{outcome:?}");
    }
    assert!(
        USAGE.contains("tz stage notify [--to <端末の名>] <題>"),
        "{USAGE}"
    );
}

#[test]
fn stnfy_parse_table() {
    let full = [
        "--to", "term-b", "見て", "--repo=/r", "--ssh", "/s", "--scribe2", "/v", "--git", "/g",
        "--tailnet", "/t", "--config", "/c.toml",
    ];
    assert_eq!(
        notify::parse(&full),
        Ok(NotifyCall {
            title: "見て".to_string(),
            to: Some("term-b".to_string()),
            repo: PathBuf::from("/r"),
            ssh: OsString::from("/s"),
            scribe2: OsString::from("/v"),
            git: OsString::from("/g"),
            tailnet: OsString::from("/t"),
            config: Some(PathBuf::from("/c.toml")),
        })
    );
    let plain = |title: &str| NotifyCall {
        title: title.to_string(),
        to: None,
        repo: PathBuf::from("."),
        ssh: OsString::from("ssh"),
        scribe2: OsString::from("scribe2"),
        git: OsString::from("git"),
        tailnet: OsString::from("tailscale"),
        config: None,
    };
    assert_eq!(notify::parse(&[TITLE]), Ok(plain(TITLE)));
    let longest = "あ".repeat(TITLE_MAX);
    assert_eq!(notify::parse(&[&longest]), Ok(plain(&longest)));

    let over = "a".repeat(TITLE_MAX + 1);
    let over_head = format!("{over:?} は知らせに使えない");
    let refusals: [(&[&str], &str); 11] = [
        (&[], "notify の題は 1 つ（旗でない字が 0 個）"),
        (&["--repo", "/r"], "notify の題は 1 つ（旗でない字が 0 個）"),
        (&["a", "b"], "notify の題は 1 つ（旗でない字が 2 個）"),
        (&["-x"], "\"-x\" は知らせに使えない"),
        (&["a\nb"], "\"a\\nb\" は知らせに使えない"),
        (&[&over], &over_head),
        (&["a", "--chrome", "/c"], "notify は旗 --chrome を受けない"),
        (&["a", "--url", "http://x/"], "notify は旗 --url を受けない"),
        (&["a", "--to", "x", "--to", "y"], "--to が 2 度ある"),
        (&["a", "--to"], "--to の値が無い"),
        (&["a", "--to="], "--to の値が空"),
    ];
    for (args, head) in refusals {
        let e = err(notify::parse(args), &format!("{args:?}"));
        assert!(e.starts_with(head), "{args:?}: {e}");
    }

    for good in ["見て", "a-b"] {
        assert!(notify::title_shaped(good), "{good:?}");
    }
    for bad in ["", "-", "a\tb"] {
        assert!(!notify::title_shaped(bad), "{bad:?}");
    }
}

#[test]
fn stnfy_argv_table() {
    let a = term("term-a");
    let argv = notify::argv(&a, TITLE, URL).expect("term-a の argv");
    assert_eq!(
        argv,
        strings(&[
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "-o",
            "ControlPath=none",
            "--",
            "me@term-a",
            FAR_A,
        ])
    );
    let tunnel = launch::tunnel_argv(&a, Path::new("/x/cdp.sock"));
    assert_eq!(argv[..6], tunnel[3..9]);

    let b = notify::argv(&term("term-b"), "見て", URL).expect("term-b の argv");
    assert_eq!(b.len(), 9);
    assert_eq!(b[7], "me@term-b");
    assert_eq!(b[8], FAR_B);

    assert_eq!(
        notify::notify_env(&a),
        [("DISPLAY".to_string(), ":0".to_string())]
    );
    for (name, os) in [("term-m", "macos"), ("term-w", "windows")] {
        assert_eq!(
            notify::argv(&term(name), TITLE, URL),
            Err(format!(
                "端末 {name} の [os] {os} の端末に知らせを出す形をまだ持たない（知らせは linux の端末だけ・持ち主へは board の URL を渡す）"
            ))
        );
    }
}

#[test]
fn stnfy_record_rules() {
    let os = |v: Option<&'static str>| v.map(OsStr::new);
    let xdg = Some(PathBuf::from("/x/state/tsuzuri/notify/proj-a.json"));
    let home = Some(PathBuf::from("/h/.local/state/tsuzuri/notify/proj-a.json"));
    let table: [(Option<&str>, Option<&str>, Option<PathBuf>); 6] = [
        (Some("/x/state"), Some("/h"), xdg),
        (Some("rel"), Some("/h"), home.clone()),
        (Some(""), Some("/h"), home.clone()),
        (None, Some("/h"), home),
        (None, Some("h"), None),
        (None, None, None),
    ];
    for (x, h, want) in table {
        assert_eq!(notify::path(os(x), os(h), "proj-a"), want, "{x:?} {h:?}");
    }

    assert_eq!(
        notify::render(&record(TITLE)),
        "{\"at\":1790000000,\"project\":\"proj-a\",\"title\":\"問い 2 つ it's \\\"見て\\\"\",\"url\":\"http://srv-a.tailnet.invalid:4801/\"}\n"
    );
    for title in [TITLE, "a\\b", "{\"at\":1}", "a\nb\tc", "🙂"] {
        let want = record(title);
        assert_eq!(notify::read(&notify::render(&want)), Some(want), "{title:?}");
    }
    for text in [
        "",
        "{}",
        r#"{"at":"1","project":"p","title":"t","url":"u"}"#,
        r#"{"at":-1,"project":"p","title":"t","url":"u"}"#,
        r#"{"at":1,"project":"p","title":"t"}"#,
        r#"{"at":1,"project":2,"title":"t","url":"u"}"#,
    ] {
        assert_eq!(notify::read(text), None, "{text:?}");
    }

    let root = scratch("record");
    let file = root.join("xstate").join(DIR).join("proj-a.json");
    notify::save(&file, &record(TITLE)).expect("save");
    assert_eq!(
        fs::read_to_string(&file).expect("書いた file"),
        notify::render(&record(TITLE))
    );
    assert_eq!(mode(&file), 0o600);
    let later = Record {
        at: 1_790_000_100,
        ..record("見て")
    };
    notify::save(&file, &later).expect("2 度目の save");
    let dir = file.parent().expect("記録の dir");
    let left: Vec<String> = fs::read_dir(dir)
        .expect("記録の dir を読む")
        .map(|e| e.expect("dir の要素").file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, ["proj-a.json"]);
    assert_eq!(
        notify::read(&fs::read_to_string(&file).expect("書いた file")),
        Some(later)
    );
    assert_eq!(mode(&file), 0o600);
}

#[test]
fn stnfy_cli_sent() {
    let field = Field::new("sent");
    field.notify_send(0);
    let before = now();
    let (rc, out, _) = field.tz(&["--to", "term-a", TITLE]);
    let after = now();
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, [notify::line("term-a", &Outcome::Sent), LINE.to_string()]);
    assert_eq!(
        field.shots(),
        [notify::argv(&term("term-a"), TITLE, URL).expect("argv")]
    );
    assert_eq!(
        field.seen(),
        strings(&[TITLE, URL, "DISPLAY=:0", "WAYLAND_DISPLAY=", "XDG_RUNTIME_DIR="])
    );
    let got = field.record().expect("記録");
    assert!((before..=after).contains(&got.at), "{got:?}");
    assert_eq!(
        got,
        Record {
            at: got.at,
            ..record(TITLE)
        }
    );
    assert_eq!(mode(&field.record_path()), 0o600);
    assert!(!field.config.exists());

    target::save(
        &field.config,
        &Targets {
            default: Some("term-b".to_string()),
            ..Targets::default()
        },
    )
    .expect("既定 term-b の設定");
    let config = fs::read_to_string(&field.config).expect("設定の字");
    let (rc, out, _) = field.tz(&["見て"]);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, [notify::line("term-b", &Outcome::Sent), LINE.to_string()]);
    assert_eq!(
        field.seen(),
        strings(&[
            "見て",
            URL,
            "DISPLAY=",
            "WAYLAND_DISPLAY=wayland-1",
            "XDG_RUNTIME_DIR=/fake/run-b",
        ])
    );
    assert_eq!(field.record().map(|r| r.title).as_deref(), Some("見て"));
    assert_eq!(
        fs::read_to_string(&field.config).expect("設定の字"),
        config
    );
}

#[test]
fn stnfy_cli_not_sent() {
    let field = Field::new("not-sent");
    let (rc, out, _) = field.tz(&["--to", "term-a", "t1"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out, [notify::line("term-a", &Outcome::Absent), LINE.to_string()]);
    assert_eq!(field.record().map(|r| r.title).as_deref(), Some("t1"));
    assert!(field.seen().is_empty());

    field.notify_send(1);
    let (rc, out, _) = field.tz(&["--to", "term-a", "t2"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(
        out,
        [notify::line("term-a", &Outcome::Failed(1)), LINE.to_string()]
    );
    assert_eq!(field.record().map(|r| r.title).as_deref(), Some("t2"));
    assert_eq!(field.seen()[..2], strings(&["t2", URL]));

    fs::write(&field.unreach, "").expect("届かない印");
    let (rc, out, _) = field.tz(&["--to", "term-a", "t3"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(
        out,
        [
            notify::line("term-a", &Outcome::Unreached("ssh の rc 255".to_string())),
            LINE.to_string(),
        ]
    );
    assert_eq!(field.record().map(|r| r.title).as_deref(), Some("t3"));
    assert_eq!(field.shots().len(), 3);
}

#[test]
fn stnfy_cli_refusals() {
    let field = Field::new("refusals");
    field.notify_send(0);
    let title = |f: &Field| f.record().map(|r| r.title);

    let (rc, out, _) = field.tz(&[TITLE]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(
        out,
        [
            "表示先の設定に project proj-a の値も既定も無いので端末に知らせを出さない（持ち主へは board の URL を渡し、表示先は tz stage target set で決める）",
            LINE,
        ]
    );
    assert_eq!(title(&field).as_deref(), Some(TITLE));

    target::save(
        &field.config,
        &Targets {
            default: Some("term-z".to_string()),
            ..Targets::default()
        },
    )
    .expect("既定 term-z の設定");
    let (rc, out, _) = field.tz(&["z"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(
        out,
        [
            "表示先の設定の project proj-a に効く既定の端末 term-z は層 A（host の面の [[device]]）に無い（既定へ落とさない・在る名は term-a term-b term-m term-w）",
            LINE,
        ]
    );
    assert_eq!(title(&field).as_deref(), Some("z"));

    let (rc, out, _) = field.tz(&["--to", "term-m", "m"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(
        out,
        [
            err(notify::argv(&term("term-m"), "m", URL), "term-m"),
            LINE.to_string(),
        ]
    );

    fs::write(&field.broken, "").expect("器の検めの落ちる印");
    let (rc, out, _) = field.tz(&["--to", "term-a", "v"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    assert!(out[0].contains("validate --state-dir"), "{}", out[0]);
    assert_eq!(out[1], LINE);
    assert_eq!(title(&field).as_deref(), Some("v"));
    fs::remove_file(&field.broken).expect("印を消す");

    let (rc, out, _) = field.tz_env(&["--to", "term-a", "r"], OsStr::new("rel"), OsStr::new("h"));
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(
        out,
        [
            "知らせの記録の path を決められない（XDG_STATE_HOME も HOME も絶対の path でない）".to_string(),
            notify::line("term-a", &Outcome::Sent),
            LINE.to_string(),
        ]
    );
    assert_eq!(title(&field).as_deref(), Some("v"));

    fs::write(&field.down, "").expect("tailnet の道具の落ちる印");
    let (rc, out, _) = field.tz(&["--to", "term-a", "d"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 1, "{out:?}");
    assert!(out[0].contains("status --json"), "{}", out[0]);
    assert_eq!(title(&field).as_deref(), Some("v"));

    let (rc, out, stderr) = field.tz(&["--to", "term-a"]);
    assert_eq!(rc, 1, "{out:?}");
    assert!(out.is_empty(), "{out:?}");
    assert!(stderr.contains("notify の題は 1 つ"), "{stderr}");
    assert!(stderr.contains(USAGE), "{stderr}");

    assert_eq!(field.shots().len(), 1);
}

#[test]
fn stnfy_source_guards() {
    let text = src("notify.rs");
    for word in [
        "bringToFront",
        "activateTarget",
        "createTarget",
        "setWindowBounds",
        "json/new",
        "json/activate",
        "Runtime.evaluate",
        "remote-debugging-port",
        "launch_argv",
        "--app=",
        "CLAUDECODE",
        "unsafe",
        "println",
    ] {
        assert!(!text.contains(word), "notify.rs が {word} を含む");
    }
    assert_eq!(src("mod.rs").matches("pub mod notify;").count(), 1);
    let cli_text = src("cli.rs");
    let start = cli_text.find("fn tell(").expect("fn tell");
    let end = cli_text
        .find("/// 自分の anchor の state dir を引き")
        .expect("fn face_text の doc");
    assert!(start < end, "fn tell は fn face_text の前");
    let region = &cli_text[start..end];
    for word in [
        "window",
        "tunnel",
        "Tunnel",
        "Eyes",
        "relay",
        "lock(",
        "target::mark",
        "open(",
    ] {
        assert!(!region.contains(word), "fn tell から fn face_text までが {word} を含む");
    }
    for word in ["notify::send(", "notify::save("] {
        assert_eq!(cli_text.matches(word).count(), 1, "{word}");
    }
}

#[test]
fn stnfy_names_apart() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/stnfy.rs");
    let text = fs::read_to_string(&path).expect("tests/stnfy.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 9, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("stnfy_")
            .unwrap_or_else(|| panic!("{name} は stnfy_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
