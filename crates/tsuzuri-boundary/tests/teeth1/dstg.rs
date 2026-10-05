//! 表示先の設定と窓を開く頼みの受付の歯（行 e-stage-target・接頭辞 dstg_・要件 FR16・判断の記録 ADR-15 の決定 (2)(6)）。
//! 偽の tz は sh の script で、受けた argv を 1 行と、環境の CLAUDECODE の値（無ければ字 none）を seat= の 1 行で記録に足し、
//! cwd を別の記録に足し、印の file rc が在れば rc 1・無ければ置いた字を出す。server は tz の binary を環境 CLAUDECODE=1 と
//! 偽の bd と器と --tz で子 process として立てる（歯の process の環境は書き替えない）。
//! host の面は fixture tests/fixtures/stage/terminals.toml に群の宣言（anchor は場の proj-a と proj-b の dir）を足した字。
#![cfg(test)]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, Command as Process, Stdio};
use std::time::Duration;

use crate::common::script;
use tsuzuri_boundary::stage::cli::{self, Setting, parse_target};
use tsuzuri_boundary::stage::target::{self, Targets as Setup};
use tsuzuri_boundary::stagecall::{
    BAD_BODY, BAD_NAME, BAD_REPLY, NO_PROJECT, OPEN_ARGS, OPEN_TIMEOUT, TARGET_ARGS, TARGET_TIMEOUT,
    TZ, TZ_FAILED, named,
};
use tsuzuri_contract::stage::{
    ALL_PATH, Effective, OPEN_PATH, OpenRequest, Origin, OwnTarget, Override, PATH, ProjectTarget,
    StageTargets, Targets,
};
use tsuzuri_contract::wire;

/// main 11affd91 の契約表の verify の filter の語 303 語を、ほかの語を部分の字として含まない語に畳んだ語
/// （この行の接頭辞 dstg_ は並べない）。
const FILTERS: [&str; 227] = [
    "aaround_", "abss_", "abst_", "accept_", "acchold_", "account_", "acctcore_", "acctdoc_",
    "accthb_", "accthome_", "acctled_", "acctlook_", "acctpcore_", "acctproj_", "acctsess_",
    "acctwin_", "acctwire_", "aface_", "afocus_", "alean_", "aord_", "aown_", "apark_", "apop_",
    "areread_", "askcard_", "athr_", "batchpanel_", "bhalf_", "board_min_", "bport_", "brand_",
    "btuck_", "cadl_", "cadopt_", "cadq_", "cdorm_", "cexcl_", "cfsplit_", "cg9_", "cgdom_",
    "cmark_", "cnote_", "cnret_", "contract_form_", "cround_", "csled_", "cspk_", "ctick_",
    "cupd_", "cupdlist_", "denv_", "dnedge_", "dngrp_", "dnrow_", "dnskip_", "dretry_", "ecache_",
    "epolq_", "eretry_", "esig_", "evkind_", "flight_", "fmark_", "fprem_", "frame_", "fserve_",
    "fstop_", "fundl_", "fxpre_", "g3g7_", "gacct_", "gapspage_", "gatt_", "gbnote_", "gchip_",
    "gcoach_", "gfix_", "gfresh_", "ghb_", "gins_", "gjst_", "glabel_", "gmret_", "gmretw_",
    "gnav_", "gpface_", "gpill_", "gpulse_", "graph_", "gsum_", "gtuck_", "gview_", "gwv_",
    "hacols_", "harest_", "hasplit_", "hbconf_", "hbmark_", "hbon_", "hbpost_", "hbproc_",
    "hbroute_", "hcard_", "hcled_", "hcnx_", "hcproj_", "hcsess_", "hdchip_", "hfig_", "hnunk_",
    "hook_", "hruling_", "hsblock_", "hsderive_", "hshist_", "hspage_", "hspk_", "hsym_", "hthr_",
    "http_", "hwstore_", "iclose_", "ilink_", "jcount_", "jrun_", "kcli_", "kg9_", "kindlab_",
    "klink_", "ksum_", "lateface_", "launch_", "lcard_", "ledgerblock_", "lgrp_", "lhome_",
    "lidle_", "lkind_", "lresume_", "lsnap_", "lspark_", "lstg_", "lstore_", "mapview_", "mkeys_",
    "mlink_", "mqask_", "mqface_", "mstore_", "mtips_", "mtree_", "nact_", "nbatch_", "ncard_",
    "nextstep_", "nodepage_", "nstall_", "nsum_", "nsumw_", "ntc_", "ntime_", "nxact_", "nxorg_",
    "parts_", "pci_", "pclosed_", "pfold_", "pgsw_", "pgz_", "pipe_", "pkac_", "plimit_",
    "pmisfit_", "pmore_", "pquest_", "pqueue_", "project_", "ptitle_", "pubfp_", "pubscan_",
    "punmap_", "pwhole_", "qblock_", "qgate_", "qkey_", "qsig_", "question_", "rbusy_", "relay_",
    "rhold_", "runsdoc_", "rvk_", "saxis_", "sclosed_", "seatblock_", "seatcard_",
    "server::events::", "server_", "sesplit_", "sgrace_", "shb_", "skeleton_", "smore_", "stage_",
    "stats_", "stbp_", "stcli_", "steady_", "sthr_", "stnfy_", "stskill_", "sttgt_", "sxaxis_",
    "ticker_", "tipx_", "tkad_", "tlic_", "topbar_", "topfit_", "tz_", "udacct_", "udash_",
    "unow_", "urpanel_", "uword_", "wstrip_",
];

/// 節の StageTargets の字（project は proj-a・既定は term-a・上書きは proj-b の term-b だけ）。
const WIRE: &str = r#"{"project":"proj-a","default":"term-a","overrides":[{"project":"proj-b","name":"term-b"}],"projects":[{"project":"proj-b","effective":{"name":"term-b","origin":"override","listed":false}},{"project":"proj-c","effective":null}],"names":["term-a"]}"#;

/// 偽の tz の既定の出力（tz が電文として読める 1 行）。
const REPLY_LINE: &str = "project proj-a の表示先を term-b にした（上書き）\n";

fn src(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 歯ごとの作業場（repo は proj-a・other は proj-b・state は host の面と群の宣言）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    other: PathBuf,
    files: PathBuf,
    state: PathBuf,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("dstg")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let place = Place {
            repo: root.join("work/proj-a"),
            other: root.join("work/proj-b"),
            files: root.join("files"),
            state: root.join("state"),
            root,
        };
        for dir in [&place.repo, &place.other, &place.files, &place.state] {
            fs::create_dir_all(dir).expect("置き場");
        }
        for dir in ["bin", "log"] {
            fs::create_dir_all(place.root.join(dir)).expect("置き場");
        }
        fs::write(place.files.join("index.html"), "tz").expect("index.html");
        let (a, b) = (place.repo.display(), place.other.display());
        fs::write(
            place.state.join("host.toml"),
            format!(
                "[[account]]\nlabel = \"acct-1\"\n\n[[account-group]]\nname = \"g-a\"\nanchors = [\"{a}\", \"{b}/\"]\naccounts = [\"acct-1\"]\n"
            ),
        )
        .expect("群の宣言");
        let root = place.root.display().to_string();
        script(
            &place.program("tz"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/argv'\n\
              printf 'seat=%s\\n' \"${CLAUDECODE-none}\" >> 'ROOT/log/argv'\n\
              pwd -P >> 'ROOT/log/cwd'\n\
              if [ -e 'ROOT/rc' ]; then exit 1; fi\n\
              exec cat 'ROOT/reply'"
                .replace("ROOT", &root),
        );
        script(&place.program("bd"), "echo '[]'");
        place.reply(REPLY_LINE);
        place
    }

    fn program(&self, name: &str) -> PathBuf {
        self.root.join("bin").join(name)
    }

    /// 偽の tz が出す字。
    fn reply(&self, text: &str) {
        fs::write(self.root.join("reply"), text).expect("偽の tz の出力");
    }

    /// 偽の tz を rc 1 で返らせる。
    fn fail(&self) {
        fs::write(self.root.join("rc"), "").expect("印");
    }

    /// server を tz の binary の子 process で立てる（環境 CLAUDECODE=1 の席の印を子にだけ渡し、偽の tz と器と bd・
    /// `state_dir` が偽なら --state-dir の無い server）。返りの Served を落とすと止める。
    fn serve(&self, state_dir: bool) -> (Served, SocketAddr) {
        let mut cmd = Process::new(env!("CARGO_BIN_EXE_tz"));
        cmd.args(["surface", "serve", "--repo"])
            .arg(&self.repo)
            .args(["--bind", "127.0.0.1:0", "--files"])
            .arg(&self.files)
            .arg("--bd")
            .arg(self.program("bd"));
        if state_dir {
            cmd.arg("--state-dir").arg(&self.state);
        }
        let mut child = cmd
            .arg("--scribe2")
            .arg(self.program("scribe2"))
            .arg("--tz")
            .arg(self.program("tz"))
            .env("CLAUDECODE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("tz surface serve");
        let mut stderr = BufReader::new(child.stderr.take().expect("標準エラー"));
        let mut addr = None;
        for _ in 0..20 {
            let mut line = String::new();
            if stderr.read_line(&mut line).expect("標準エラー") == 0 {
                break;
            }
            if let Some(rest) = line.trim().strip_prefix("tz surface serve: http://") {
                addr = rest.trim_end_matches('/').parse::<SocketAddr>().ok();
                break;
            }
        }
        let served = Served {
            child,
            _stderr: stderr,
        };
        (served, addr.expect("口の住所の行"))
    }

    /// 偽の tz の記録の行（1 回に argv の行と seat= の行）。
    fn log(&self) -> Vec<String> {
        fs::read_to_string(self.root.join("log/argv"))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// 撃った argv の行（撃った順）。
    fn calls(&self) -> Vec<String> {
        self.log().into_iter().step_by(2).collect()
    }

    /// 撃った時の環境の seat= の行（撃った順）。
    fn seats(&self) -> Vec<String> {
        self.log().into_iter().skip(1).step_by(2).collect()
    }

    /// 撃った時の cwd（撃った順）。
    fn cwds(&self) -> Vec<String> {
        fs::read_to_string(self.root.join("log/cwd"))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn clear(&self) {
        for name in ["argv", "cwd"] {
            let _ = fs::remove_file(self.root.join("log").join(name));
        }
    }

    /// 撃つ argv の字（命令の後に --repo と --scribe2 が付く）。
    fn argv(&self, words: &str, repo: &Path) -> String {
        format!(
            "{words} --repo {} --scribe2 {}",
            repo.display(),
            self.program("scribe2").display()
        )
    }
}

/// 応答（状態の code・本文）。
struct Reply {
    status: u16,
    body: String,
}

fn send(addr: SocketAddr, method: &str, path: &str, heads: &str, body: &str) -> Reply {
    let mut s = TcpStream::connect(addr).expect("接続");
    s.set_read_timeout(Some(Duration::from_secs(90)))
        .expect("timeout");
    s.write_all(
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\n{heads}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .as_bytes(),
    )
    .expect("要求を書く");
    let mut out = Vec::new();
    s.read_to_end(&mut out).expect("応答を読む");
    let text = String::from_utf8(out).expect("応答の字");
    let (head, body) = text.split_once("\r\n\r\n").expect("頭と本文");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|c| c.parse().ok())
        .expect("状態の code");
    Reply {
        status,
        body: body.to_string(),
    }
}

/// 頭 Origin が Host と同じ POST。
fn post(addr: SocketAddr, path: &str, body: &str) -> Reply {
    send(addr, "POST", path, &format!("Origin: http://{addr}\r\n"), body)
}

fn code(reply: &Reply) -> (u16, &str) {
    (reply.status, reply.body.as_str())
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn dstg_wire_forms() {
    assert_eq!(PATH, "/api/stage/target");
    assert_eq!(ALL_PATH, "/api/stage/targets");
    assert_eq!(OPEN_PATH, "/api/stage/open");
    let value = StageTargets {
        project: "proj-a".to_string(),
        default: Some("term-a".to_string()),
        overrides: vec![Override {
            project: "proj-b".to_string(),
            name: "term-b".to_string(),
        }],
        projects: vec![
            ProjectTarget {
                project: "proj-b".to_string(),
                effective: Some(Effective {
                    name: "term-b".to_string(),
                    origin: Origin::Override,
                    listed: false,
                }),
            },
            ProjectTarget {
                project: "proj-c".to_string(),
                effective: None,
            },
        ],
        names: strings(&["term-a"]),
    };
    assert_eq!(wire::encode(&value).expect("字"), WIRE);
    assert_eq!(wire::decode::<StageTargets>(WIRE).expect("読み"), value);
    assert_eq!(
        wire::encode(&Origin::Default).expect("字"),
        "\"default\"",
        "出所の字は kebab-case"
    );

    let own: OwnTarget = wire::decode(r#"{"to":"term-a"}"#).expect("名");
    assert_eq!(own.to.as_deref(), Some("term-a"));
    let own: OwnTarget = wire::decode(r#"{"to":null}"#).expect("null");
    assert_eq!(own.to, None);
    let all = Targets::All {
        to: "term-a".to_string(),
    };
    assert_eq!(wire::encode(&all).expect("字"), r#"{"all":{"to":"term-a"}}"#);
    assert_eq!(
        wire::decode::<Targets>(r#"{"all":{"to":"term-a"}}"#).expect("読み"),
        all
    );
    project_forms_and_refusals();
}

/// Targets::Project と OpenRequest の字の往復と、読まない字と名でない字の断りを見る。
fn project_forms_and_refusals() {
    let project = Targets::Project {
        project: "proj-b".to_string(),
        to: None,
    };
    assert_eq!(
        wire::encode(&project).expect("字"),
        r#"{"project":{"project":"proj-b","to":null}}"#
    );
    assert_eq!(
        wire::decode::<Targets>(r#"{"project":{"project":"proj-b","to":null}}"#).expect("読み"),
        project
    );
    let open = OpenRequest { project: None };
    assert_eq!(wire::encode(&open).expect("字"), r#"{"project":null}"#);
    assert_eq!(
        wire::decode::<OpenRequest>(r#"{"project":null}"#).expect("読み"),
        open
    );

    for text in [r#"{"to":"a","project":"b"}"#, "{}", "[]"] {
        assert!(wire::decode::<OwnTarget>(text).is_err(), "OwnTarget が {text} を読む");
    }
    for text in [
        r#"{"all":{"to":"a","x":1}}"#,
        r#"{"every":{"to":"a"}}"#,
        r#"{"project":{"to":"a"}}"#,
        r#"{"project":{"project":"p"}}"#,
    ] {
        assert!(wire::decode::<Targets>(text).is_err(), "Targets が {text} を読む");
    }
    for text in [r#"{"project":null,"to":"a"}"#, "{}"] {
        assert!(wire::decode::<OpenRequest>(text).is_err(), "OpenRequest が {text} を読む");
    }

    for name in ["term-a", "term b", "a-b"] {
        assert!(named(name), "{name}");
    }
    for name in ["-x", "--all", "-", "", "a=b", "a\"b", "a\\b", "a\nb"] {
        assert!(!named(name), "{name:?}");
    }
}

/// 節の host の面（fixture の群に anchor の proj-a と proj-b の dir を足した字）。
fn face(a: &Path, b: &Path) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/stage/terminals.toml");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let head = "name = \"grp-a\"\n";
    assert!(text.contains(head), "fixture の群の行");
    text.replace(
        head,
        &format!("{head}anchors = [\"{}\", \"{}\"]\n", a.display(), b.display()),
    )
}

#[test]
fn dstg_show_json_reads_setting() {
    // 旗の読み。
    let call = parse_target(&["show", "--json"]).expect("show --json");
    assert_eq!(call.setting, Setting::Json);
    assert_eq!(parse_target(&["show"]).expect("show").setting, Setting::Show);
    let twice = parse_target(&["show", "--json", "--json"]).expect_err("2 度");
    assert!(twice.starts_with("--json が 2 度ある"), "{twice}");
    for args in [
        vec!["set", "--all", "term-a", "--json"],
        vec!["clear", "--project", "proj-a", "--json"],
        vec!["set", "--project", "proj-a", "term-a", "--json"],
    ] {
        let err = parse_target(&args).expect_err("show のほかの --json");
        assert!(err.starts_with("--json は target show だけが受ける"), "{err}");
    }

    // 電文の組み（既定 term-a・上書き proj-b は term-b・層 A の名は term-a だけ）。
    let mut setup = Setup {
        default: Some("term-a".to_string()),
        ..Setup::default()
    };
    setup.set_project("proj-b", "term-b");
    let doc = cli::doc(
        &setup,
        "proj-a",
        &strings(&["proj-a", "proj-b"]),
        &strings(&["term-a"]),
    );
    assert_eq!(doc.project, "proj-a");
    assert_eq!(doc.default.as_deref(), Some("term-a"));
    assert_eq!(
        doc.overrides,
        vec![Override {
            project: "proj-b".to_string(),
            name: "term-b".to_string()
        }]
    );
    let effective = |name: &str, origin, listed| {
        Some(Effective {
            name: name.to_string(),
            origin,
            listed,
        })
    };
    assert_eq!(
        doc.projects,
        vec![
            ProjectTarget {
                project: "proj-a".to_string(),
                effective: effective("term-a", Origin::Default, true),
            },
            ProjectTarget {
                project: "proj-b".to_string(),
                effective: effective("term-b", Origin::Override, false),
            },
        ]
    );
    assert_eq!(doc.names, strings(&["term-a"]));
    real_tz_show();
}

/// 偽の器と偽の git と設定の file を置き、本物の tz の show を撃つ支度をする。
fn real_tz_show() {
    // 本物の tz（偽の器と偽の git・既定 term-a・上書き proj-b と proj-z・群の anchor は proj-a と proj-b）。
    let place = Place::new("show-json");
    let state = place.root.join("state-a");
    fs::create_dir_all(&state).expect("state dir");
    fs::write(state.join("host.toml"), face(&place.repo, &place.other)).expect("host の面");
    let scribe2 = place.program("scribe2");
    script(&scribe2, "exit 0");
    let git = place.program("git-cfg");
    script(
        &git,
        &format!(
            "case \"$5\" in scribe2.statedir) printf '%s\\n' '{}' ;; *) exit 1 ;; esac",
            state.display()
        ),
    );
    let config = place.root.join("cfg").join(target::DIR).join(target::FILE);
    let mut real = Setup {
        default: Some("term-a".to_string()),
        ..Setup::default()
    };
    real.set_project("proj-b", "term-b");
    real.set_project("proj-z", "term-z");
    target::save(&config, &real).expect("設定を書く");
    let run = |flags: &[&str]| {
        let out = Process::new(env!("CARGO_BIN_EXE_tz"))
            .args(["stage", "target", "show"])
            .args(flags)
            .arg("--repo")
            .arg(&place.repo)
            .arg("--scribe2")
            .arg(&scribe2)
            .arg("--git")
            .arg(&git)
            .arg("--config")
            .arg(&config)
            .env_remove("CLAUDECODE")
            .output()
            .expect("tz を撃つ");
        (
            out.status.code(),
            String::from_utf8(out.stdout).expect("標準出力"),
        )
    };
    show_runs(run);
}

/// 本物の tz の show を --json の有り無しで撃ち、電文と今までの行を見る。
fn show_runs(run: impl Fn(&[&str]) -> (Option<i32>, String)) {
    let (rc, out) = run(&["--json"]);
    assert_eq!(rc, Some(0), "{out}");
    assert_eq!(out.lines().count(), 1, "1 行の標準出力: {out}");
    let got: StageTargets = wire::decode(out.trim_end()).expect("電文");
    let listed = |name: &str, origin| {
        Some(Effective {
            name: name.to_string(),
            origin,
            listed: true,
        })
    };
    assert_eq!(got.project, "proj-a");
    assert_eq!(
        got.overrides,
        vec![
            Override {
                project: "proj-b".to_string(),
                name: "term-b".to_string()
            },
            Override {
                project: "proj-z".to_string(),
                name: "term-z".to_string()
            },
        ]
    );
    assert_eq!(
        got.projects,
        vec![
            ProjectTarget {
                project: "proj-a".to_string(),
                effective: listed("term-a", Origin::Default),
            },
            ProjectTarget {
                project: "proj-b".to_string(),
                effective: listed("term-b", Origin::Override),
            },
        ]
    );
    assert_eq!(got.names, strings(&["term-a", "term-b"]));

    // 旗の無い show は今までの行を出す。
    let (rc, plain) = run(&[]);
    assert_eq!(rc, Some(0), "{plain}");
    assert!(plain.contains("既定 term-a\n"), "{plain}");
    assert!(plain.contains("project proj-b term-b（上書き）\n"), "{plain}");
    assert!(!plain.contains('{'), "{plain}");
}

#[test]
fn dstg_route_reads_via_tz() {
    let place = Place::new("route-reads");
    let (_served, addr) = place.serve(false);
    place.reply(&format!("{WIRE}\n"));
    let reply = send(addr, "GET", PATH, "", "");
    assert_eq!(code(&reply), (200, WIRE));
    assert_eq!(
        place.calls(),
        [place.argv("stage target show --json", &place.repo)]
    );
    assert_eq!(place.seats(), ["seat=none"], "子の環境に席の印が無い");
    let cwd = place.repo.canonicalize().expect("repo");
    assert_eq!(place.cwds(), [cwd.display().to_string()]);

    place.reply("これは電文でない\n");
    let reply = send(addr, "GET", PATH, "", "");
    assert_eq!(code(&reply), (502, BAD_REPLY));
    place.reply("{\"project\":\"proj-a\"}\n");
    let reply = send(addr, "GET", PATH, "", "");
    assert_eq!(code(&reply), (502, BAD_REPLY), "欄の足りない字は電文でない");

    place.reply(&format!("{WIRE}\n"));
    place.fail();
    let reply = send(addr, "GET", PATH, "", "");
    assert_eq!(code(&reply), (502, TZ_FAILED));
}

#[test]
fn dstg_own_post_order() {
    let place = Place::new("own-post");
    let (_served, addr) = place.serve(false);
    let port = addr.port();
    let set = r#"{"to":"term-b"}"#;

    let reply = send(
        addr,
        "POST",
        PATH,
        &format!("Origin: http://evil.example:{port}\r\n"),
        set,
    );
    assert_eq!(code(&reply), (403, "origin"));
    for text in [
        r#"{"to":"a","project":"b"}"#,
        "{}",
        "{",
        r#"{"project":"proj-b"}"#,
    ] {
        let reply = post(addr, PATH, text);
        assert_eq!(code(&reply), (400, BAD_BODY), "{text}");
    }
    for text in [
        r#"{"to":"--all"}"#,
        r#"{"to":"a=b"}"#,
        r#"{"to":"-"}"#,
        r#"{"to":""}"#,
    ] {
        let reply = post(addr, PATH, text);
        assert_eq!(code(&reply), (400, BAD_NAME), "{text}");
    }
    assert!(place.calls().is_empty(), "断りで tz を撃つ");

    let reply = post(addr, PATH, set);
    assert_eq!(code(&reply), (200, REPLY_LINE));
    assert_eq!(
        place.calls(),
        [place.argv("stage target set --project proj-a term-b", &place.repo)]
    );
    let reply = post(addr, PATH, r#"{"to":null}"#);
    assert_eq!(code(&reply), (200, REPLY_LINE));
    assert_eq!(
        place.calls().last().map(String::as_str),
        Some(place.argv("stage target clear --project proj-a", &place.repo).as_str())
    );
    assert_eq!(place.calls().len(), 2);
    assert_eq!(place.seats(), ["seat=none", "seat=none"]);

    place.fail();
    let reply = post(addr, PATH, set);
    assert_eq!(code(&reply), (502, TZ_FAILED));
}

/// 群の宣言の anchor は proj-a（--repo の dir）と proj-b（other の dir）。
#[test]
fn dstg_all_post_scope() {
    let place = Place::new("all-post");
    let (_served, addr) = place.serve(true);
    let port = addr.port();

    let reply = send(
        addr,
        "POST",
        ALL_PATH,
        &format!("Origin: http://evil.example:{port}\r\n"),
        r#"{"all":{"to":"term-a"}}"#,
    );
    assert_eq!(code(&reply), (403, "origin"));
    for text in [r#"{"all":{"to":"a","x":1}}"#, "{}", r#"{"to":"a"}"#, "["] {
        let reply = post(addr, ALL_PATH, text);
        assert_eq!(code(&reply), (400, BAD_BODY), "{text}");
    }
    for text in [
        r#"{"all":{"to":"-x"}}"#,
        r#"{"all":{"to":"a=b"}}"#,
        r#"{"project":{"project":"proj-b","to":"--all"}}"#,
        r#"{"project":{"project":"-p","to":null}}"#,
    ] {
        let reply = post(addr, ALL_PATH, text);
        assert_eq!(code(&reply), (400, BAD_NAME), "{text}");
    }
    let reply = post(addr, ALL_PATH, r#"{"project":{"project":"proj-z","to":"term-a"}}"#);
    assert_eq!(code(&reply), (404, NO_PROJECT));
    assert!(place.calls().is_empty(), "断りで tz を撃つ");

    let repo = &place.repo;
    let wanted = [
        (r#"{"all":{"to":"term-a"}}"#, "stage target set --all term-a"),
        (
            r#"{"project":{"project":"proj-b","to":"term-b"}}"#,
            "stage target set --project proj-b term-b",
        ),
        (
            r#"{"project":{"project":"proj-b","to":null}}"#,
            "stage target clear --project proj-b",
        ),
        (
            r#"{"project":{"project":"proj-a","to":"term-a"}}"#,
            "stage target set --project proj-a term-a",
        ),
    ];
    for (i, (text, words)) in wanted.iter().enumerate() {
        let reply = post(addr, ALL_PATH, text);
        assert_eq!(code(&reply), (200, REPLY_LINE), "{text}");
        assert_eq!(place.calls().len(), i + 1);
        assert_eq!(
            place.calls().last().map(String::as_str),
            Some(place.argv(words, repo).as_str()),
            "{text}"
        );
    }
    bare_scope(place);
}

/// state dir の無い server が --repo の project だけを受けることを見る。
fn bare_scope(place: Place) {
    // state dir の無い server は --repo の project だけを受ける。
    let (_bare_served, bare) = place.serve(false);
    place.clear();
    let reply = post(bare, ALL_PATH, r#"{"project":{"project":"proj-b","to":"term-b"}}"#);
    assert_eq!(code(&reply), (404, NO_PROJECT));
    assert!(place.calls().is_empty());
    let reply = post(bare, ALL_PATH, r#"{"project":{"project":"proj-a","to":"term-a"}}"#);
    assert_eq!(code(&reply), (200, REPLY_LINE));
    assert_eq!(place.calls().len(), 1);
}

/// 子 process の server（drop で止める）。
struct Served {
    child: Child,
    _stderr: BufReader<ChildStderr>,
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn dstg_open_drops_seat_mark() {
    let place = Place::new("open");
    let (served, addr) = place.serve(true);
    let port = addr.port();

    let reply = send(
        addr,
        "POST",
        OPEN_PATH,
        &format!("Origin: http://evil.example:{port}\r\n"),
        r#"{"project":null}"#,
    );
    assert_eq!(code(&reply), (403, "origin"));
    let reply = post(addr, OPEN_PATH, r#"{"project":"proj-z"}"#);
    assert_eq!(code(&reply), (404, NO_PROJECT));
    let reply = post(addr, OPEN_PATH, r#"{"project":"-x"}"#);
    assert_eq!(code(&reply), (400, BAD_NAME));
    let reply = post(addr, OPEN_PATH, r#"{"to":"term-a"}"#);
    assert_eq!(code(&reply), (400, BAD_BODY));
    assert!(place.calls().is_empty(), "断りで tz を撃つ");

    let reply = post(addr, OPEN_PATH, r#"{"project":null}"#);
    assert_eq!(code(&reply), (200, REPLY_LINE));
    let reply = post(addr, OPEN_PATH, r#"{"project":"proj-b"}"#);
    assert_eq!(code(&reply), (200, REPLY_LINE));
    // 群の宣言に末尾の斜線付きで書いた anchor は書かれた字のまま --repo に渡す。
    let written = PathBuf::from(format!("{}/", place.other.display()));
    assert_eq!(
        place.calls(),
        [
            place.argv("stage open", &place.repo),
            place.argv("stage open", &written),
        ]
    );
    assert_eq!(place.seats(), ["seat=none", "seat=none"], "子の環境に席の印が無い");
    let dirs: Vec<String> = [&place.repo, &place.other]
        .iter()
        .map(|d| d.canonicalize().expect("dir").display().to_string())
        .collect();
    assert_eq!(place.cwds(), dirs);

    place.fail();
    let reply = post(addr, OPEN_PATH, r#"{"project":null}"#);
    assert_eq!(code(&reply), (502, TZ_FAILED));
    drop(served);
}

#[test]
fn dstg_post_needs_origin() {
    let place = Place::new("needs-origin");
    let (_served, addr) = place.serve(true);
    place.reply(&format!("{WIRE}\n"));
    let reply = send(addr, "GET", PATH, "", "");
    assert_eq!(code(&reply), (200, WIRE), "頭 Origin の無い GET は通る");
    place.clear();
    place.reply(REPLY_LINE);

    let bodies = [
        (PATH, r#"{"to":"term-b"}"#),
        (ALL_PATH, r#"{"all":{"to":"term-a"}}"#),
        (OPEN_PATH, r#"{"project":null}"#),
    ];
    for (path, body) in bodies {
        let reply = send(addr, "POST", path, "", body);
        assert_eq!(code(&reply), (403, "origin"), "{path}");
    }
    assert!(place.calls().is_empty(), "頭 Origin の無い要求で tz を撃つ");
    for (i, (path, body)) in bodies.iter().enumerate() {
        let reply = post(addr, path, body);
        assert_eq!(code(&reply), (200, REPLY_LINE), "{path}");
        assert_eq!(place.calls().len(), i + 1, "{path}");
    }

    let module = src("server/mod.rs");
    assert_eq!(module.matches("fn guarded_strict<").count(), 1);
    assert_eq!(module.matches("if req.origin.is_none()").count(), 1);
}

#[test]
fn dstg_wiring_text() {
    let call = src("stagecall.rs");
    for word in [
        "fs::",
        "File::",
        "OpenOptions",
        "rename",
        "remove_",
        "target::save",
        "target::mark",
        "Command",
    ] {
        assert!(!call.contains(word), "stagecall.rs が {word} を含む");
    }
    for (word, count) in [
        ("capture_unset(", 1),
        ("&[cli::SEAT_ENV]", 1),
        ("accthb::anchor(", 2),
        ("cli::project(", 2),
    ] {
        assert_eq!(call.matches(word).count(), count, "stagecall.rs の {word}");
    }
    for (file, word, post) in [
        ("stagetarget.rs", "stagecall::read(", false),
        ("stageset.rs", "accept_own(", true),
        ("stageall.rs", "accept_all(", true),
        ("stageopen.rs", "accept_open(", true),
    ] {
        let text = src(&format!("server/routes/{file}"));
        assert_eq!(text.matches(word).count(), 1, "{file} の {word}");
        for banned in ["fs::", "Command", "stage::target", "window"] {
            assert!(!text.contains(banned), "{file} が {banned} を含む");
        }
        if post {
            assert_eq!(text.matches("guarded_strict(req, ").count(), 1, "{file}");
            assert!(!text.contains("guarded(req,"), "{file} が guarded を通る");
        }
    }
    entry_and_consts();
}

/// 入口と設定の配線の字と、撃つ program・引数・待ちの定数を見る。
fn entry_and_consts() {
    assert_eq!(src("main.rs").matches("std::env::current_exe()").count(), 1);
    assert_eq!(
        src("server/config.rs").matches("stagecall::TZ.into()").count(),
        1
    );
    let (tz, target, open): (&str, [&str; 2], [&str; 2]) = (TZ, TARGET_ARGS, OPEN_ARGS);
    assert_eq!((tz, target, open), ("tz", ["stage", "target"], ["stage", "open"]));
    assert_eq!(TARGET_TIMEOUT, Duration::from_secs(20));
    assert_eq!(OPEN_TIMEOUT, Duration::from_secs(60));
}

#[test]
fn dstg_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/teeth1/dstg.rs");
    let text = fs::read_to_string(&path).expect("tests/teeth1/dstg.rs を読む");
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
            .strip_prefix("dstg_")
            .unwrap_or_else(|| panic!("{name} は dstg_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
    for word in FILTERS {
        assert!(!word.contains("dstg_"), "{word} が接頭辞 dstg_ を含む");
        assert!(!"dstg_".contains(word), "接頭辞 dstg_ が {word} を含む");
    }
}
