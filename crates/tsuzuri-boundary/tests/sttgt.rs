//! 表示先の設定の歯（行 i-7・接頭辞 sttgt_）。
//! 設定の fixture は tests/fixtures/stage/stage-target.toml（名は偽物）で、host の面は tests/fixtures/stage/terminals.toml に
//! 群の宣言を足した字。偽の器・git・tailnet の道具は sh の script で、ssh と席の目の Chrome は無い path を渡す
//! （本物の ssh と Chrome と tailnet の道具と器を撃たず網に出ない）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command as Process, Stdio};

use tsuzuri_boundary::stage::cli::{
    Aim, Setting, TargetCall, aim, config_path, parse_target, project, show,
};
use tsuzuri_boundary::stage::target::{
    DEFAULT, DIR, FILE, HEAD, Origin, PROJECT, SHOWN, Targets, known, load, mark, path, read,
    render, save, shaped,
};
use tsuzuri_boundary::stage::terminal::FACE;

/// contracts の verify の filter の語のうち、ほかの語を部分の字として含まない語と、計画の後の行 g-gz の接頭辞 pgz_ と、
/// 同じノートのもう 1 つの行の接頭辞 stbp_（この行の接頭辞 sttgt_ は並べない）。
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
    "wstrip_", "pgz_", "stbp_",
];

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

/// 節の host の面に足す群の宣言。
const GROUP: &str = "\n[[account-group]]\nname = \"grp-b\"\nanchors = [\"/fake/anchors/proj-a\", \"/fake/anchors/proj-c\"]\n";

fn fixture_text(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/stage")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// fixture の設定の値（既定 term-a・上書き proj-b は term-b・印 term-a は 1790000000）。
fn fixture_value() -> Targets {
    Targets {
        default: Some("term-a".to_string()),
        projects: BTreeMap::from([("proj-b".to_string(), "term-b".to_string())]),
        shown: BTreeMap::from([("term-a".to_string(), 1_790_000_000)]),
    }
}

/// 既定と上書きだけの設定。
fn value(default: Option<&str>, projects: &[(&str, &str)]) -> Targets {
    Targets {
        default: default.map(str::to_string),
        projects: projects
            .iter()
            .map(|(p, n)| (p.to_string(), n.to_string()))
            .collect(),
        shown: BTreeMap::new(),
    }
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
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
        .join("sttgt")
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
fn err<T: std::fmt::Debug>(got: Result<T, String>, what: &str) -> String {
    match got {
        Ok(v) => panic!("{what}: Ok {v:?}"),
        Err(e) => e,
    }
}

/// 偽の program（sh の script・権限 0755）。
fn fake(root: &Path, name: &str, body: &str) -> PathBuf {
    let path = root.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}")).expect("偽の program");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
    path
}

/// 偽の場（dir の名が proj-a の repo・群の宣言を足した host の面・偽の器と git と tailnet の道具・印の file・設定の path）。
struct Field {
    repo: PathBuf,
    scribe2: PathBuf,
    git: PathBuf,
    tailnet: PathBuf,
    absent: PathBuf,
    tmp: PathBuf,
    broken: PathBuf,
    config: PathBuf,
}

impl Field {
    fn new(name: &str) -> Field {
        let root = scratch(name);
        let repo = root.join("proj-a");
        let state = root.join("state");
        for dir in [&repo, &state] {
            fs::create_dir_all(dir).expect("場の dir");
        }
        fs::write(
            state.join(FACE),
            format!("{}{GROUP}", fixture_text("terminals.toml")),
        )
        .expect("host の面");
        let status = root.join("status");
        fs::write(&status, STATUS).expect("場の status");
        let broken = root.join("broken");
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
        let tailnet = fake(&root, "tailnet", &format!("exec cat '{}'\n", status.display()));
        let tmp = env::temp_dir().join(format!("sttgt-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("場の一時の dir");
        Field {
            repo,
            scribe2,
            git,
            tailnet,
            absent: root.join("absent"),
            tmp,
            broken,
            config: root.join("cfg").join(DIR).join(FILE),
        }
    }

    /// tz stage を撃つ（target の口なら ssh・tailnet・Chrome の旗を渡さない）。rc と標準出力の行を返す。
    fn tz(&self, args: &[&str]) -> (i32, Vec<String>) {
        let mut shot = Process::new(env!("CARGO_BIN_EXE_tz"));
        shot.arg("stage")
            .args(args)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--scribe2")
            .arg(&self.scribe2)
            .arg("--git")
            .arg(&self.git)
            .arg("--config")
            .arg(&self.config);
        if args.first() != Some(&"target") {
            shot.arg("--ssh")
                .arg(&self.absent)
                .arg("--tailnet")
                .arg(&self.tailnet)
                .arg("--chrome")
                .arg(&self.absent);
        }
        let out = shot
            .env_remove("CLAUDECODE")
            .env("TMPDIR", &self.tmp)
            .stdin(Stdio::null())
            .output()
            .expect("tz を撃つ");
        let text = String::from_utf8(out.stdout).expect("tz の標準出力の字");
        (
            out.status.code().unwrap_or(-1),
            text.lines().map(str::to_string).collect(),
        )
    }

    fn written(&self) -> Option<String> {
        fs::read_to_string(&self.config).ok()
    }
}

impl Drop for Field {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.tmp);
    }
}

#[test]
fn sttgt_path_rules() {
    assert_eq!(
        (DIR, FILE, DEFAULT, PROJECT, SHOWN),
        ("tsuzuri", "stage.toml", "default", "[project]", "[shown]")
    );
    assert_eq!(
        HEAD,
        "# tsuzuri の表示先の設定（tz stage target が書く・追跡されない file）"
    );
    let path_fn: fn(Option<&OsStr>, Option<&OsStr>) -> Option<PathBuf> = path;
    let _ = path_fn;
    let os = |v: Option<&'static str>| v.map(OsStr::new);
    let xdg = Some(PathBuf::from("/x/cfg/tsuzuri/stage.toml"));
    let home = Some(PathBuf::from("/h/.config/tsuzuri/stage.toml"));
    let table: [(Option<&str>, Option<&str>, Option<PathBuf>); 9] = [
        (Some("/x/cfg"), Some("/h"), xdg.clone()),
        (None, Some("/h"), home.clone()),
        (Some("rel/cfg"), Some("/h"), home.clone()),
        (Some(""), Some("/h"), home),
        (Some("/x/cfg"), None, xdg),
        (None, None, None),
        (None, Some(""), None),
        (None, Some("rel"), None),
        (Some("rel/cfg"), None, None),
    ];
    for (x, h, want) in table {
        assert_eq!(path(os(x), os(h)), want, "{x:?} {h:?}");
    }
}

#[test]
fn sttgt_read_render() {
    let text = fixture_text("stage-target.toml");
    let read_fn: fn(&str) -> Result<Targets, String> = read;
    let render_fn: fn(&Targets) -> String = render;
    let _ = (read_fn, render_fn);
    assert_eq!(read(&text), Ok(fixture_value()));
    assert_eq!(render(&fixture_value()), text);
    let empty = render(&Targets::default());
    assert_eq!(empty, format!("{HEAD}\n"));
    assert_eq!(read(&empty), Ok(Targets::default()));

    let table: [(&str, &str); 10] = [
        ("[other]\n", "1 行目: 知らない表 [other]"),
        ("color = \"x\"\n", "1 行目: 知らない鍵 color"),
        ("default = term-a\n", "1 行目: 値 term-a は引用符で囲んだ名でない"),
        (
            "default = \"term-a\"\ndefault = \"term-b\"\n",
            "2 行目: 鍵 default が 2 度ある",
        ),
        ("[project]\n[shown]\n[project]\n", "3 行目: 表 [project] が 2 度ある"),
        (
            "[project]\nproj-b = \"term-b\"\n",
            "2 行目: 鍵 proj-b は引用符で囲んだ名でない",
        ),
        (
            "[project]\n\"p\" = \"term-a\"\n\"p\" = \"term-b\"\n",
            "3 行目: 鍵 p が 2 度ある",
        ),
        ("[shown]\n\"term-a\" = -1\n", "2 行目: 値 -1 は epoch 秒の数でない"),
        (
            "[shown]\n\"term-a\" = \"1\"\n",
            "2 行目: 値 \"1\" は epoch 秒の数でない",
        ),
        ("default\n", "1 行目: 鍵 = 値 の形でない"),
    ];
    for (text, want) in table {
        assert_eq!(read(text), Err(want.to_string()), "{text:?}");
    }
    err(read("default = \"a\\\"b\"\n"), "逆斜線と引用符");
}

#[test]
fn sttgt_targets_ops() {
    let fixture = fixture_value();
    assert_eq!(fixture.effective("proj-b"), Some(("term-b", Origin::Project)));
    assert_eq!(fixture.effective("proj-x"), Some(("term-a", Origin::Default)));
    assert_eq!(Targets::default().effective("proj-b"), None);
    assert_eq!((Origin::Default.word(), Origin::Project.word()), ("既定", "上書き"));

    let mut t = fixture.clone();
    t.set_project("proj-x", "term-b");
    assert_eq!(t.effective("proj-x"), Some(("term-b", Origin::Project)));
    assert!(t.clear_project("proj-x"));
    assert!(!t.clear_project("proj-x"));
    assert_eq!(t, fixture);

    let mut t = fixture.clone();
    assert_eq!(t.set_all("term-b"), 1);
    assert_eq!(t.default.as_deref(), Some("term-b"));
    assert!(t.projects.is_empty());
    assert_eq!(t.shown, fixture.shown);
    layer_names();
}

/// 層 A の名の照らしと project の名の形。
fn layer_names() {
    let layer = strings(&["term-a", "term-b"]);
    assert_eq!(known("term-b", &layer), Ok(()));
    assert_eq!(
        known("term-x", &layer),
        Err("端末の名 term-x は層 A（host の面の [[device]]）に無い（在る名は term-a term-b）".to_string())
    );
    for bad in ["", "a\"b", "a\\b", "a\nb", "a=b"] {
        assert!(!shaped(bad), "{bad:?}");
    }
    assert!(shaped("proj.a-b_c"));
}

#[test]
fn sttgt_save_and_mark() {
    let root = scratch("save");
    let file = root.join("cfg").join(DIR).join(FILE);
    assert_eq!(load(&file), Ok(Targets::default()));
    save(&file, &fixture_value()).expect("save");
    assert_eq!(
        fs::read_to_string(&file).expect("書いた file"),
        fixture_text("stage-target.toml")
    );
    assert_eq!(mode(&file), 0o600);
    let dir = file.parent().expect("設定の dir");
    let left: Vec<String> = fs::read_dir(dir)
        .expect("設定の dir を読む")
        .map(|e| e.expect("dir の要素").file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, [FILE]);

    assert_eq!(mark(&file, "term-a", 1_790_000_100), Ok(false));
    assert_eq!(load(&file), Ok(fixture_value()));
    assert_eq!(mark(&file, "term-b", 1_790_000_100), Ok(true));
    let mut want = fixture_value();
    want.shown.insert("term-b".to_string(), 1_790_000_100);
    assert_eq!(load(&file), Ok(want));
    assert_eq!(mode(&file), 0o600);

    fs::write(&file, format!("{HEAD}\n[screen]\n")).expect("形の外れた file");
    let e = err(load(&file), "形の外れた file");
    let shown = file.display().to_string();
    for word in ["表示先の設定", shown.as_str(), "2 行目", "知らない表 [screen]"] {
        assert!(e.contains(word), "{word}: {e}");
    }
}

#[test]
fn sttgt_aim_table() {
    let fixture = fixture_value();
    let layer = strings(&["term-a", "term-b"]);
    let table: [(Option<&str>, &str, &Targets, Aim); 4] = [
        (Some("term-b"), "proj-x", &fixture, Aim::Named("term-b".to_string())),
        (
            None,
            "proj-b",
            &fixture,
            Aim::Chosen {
                name: "term-b".to_string(),
                first: true,
            },
        ),
        (
            None,
            "proj-x",
            &fixture,
            Aim::Chosen {
                name: "term-a".to_string(),
                first: false,
            },
        ),
        (None, "proj-b", &Targets::default(), Aim::Unset),
    ];
    for (to, project, targets, want) in table {
        assert_eq!(aim(to, project, targets, &layer), Ok(want), "{to:?} {project}");
    }
    assert_eq!(
        aim(None, "proj-b", &fixture, &strings(&["term-a"])),
        Err("表示先の設定の project proj-b に効く上書きの端末 term-b は層 A（host の面の [[device]]）に無い（既定へ落とさない・在る名は term-a）".to_string())
    );

    let root = scratch("aim");
    let repo = root.join("proj-a");
    fs::create_dir_all(&repo).expect("repo");
    assert_eq!(project(&repo), Ok("proj-a".to_string()));
    assert_eq!(project(&repo.join(".")), Ok("proj-a".to_string()));
    err(project(&root.join("none")), "無い dir");
    assert_eq!(
        config_path(Some(Path::new("/c/s.toml"))),
        Ok(PathBuf::from("/c/s.toml"))
    );
}

#[test]
fn sttgt_parse_target_table() {
    let call = |setting: Setting| TargetCall {
        setting,
        repo: PathBuf::from("."),
        scribe2: "scribe2".into(),
        git: "git".into(),
        config: None,
    };
    let table: [(&[&str], TargetCall); 5] = [
        (&["show"], call(Setting::Show)),
        (
            &["set", "--project", "proj-a", "term-b"],
            call(Setting::Project("proj-a".to_string(), "term-b".to_string())),
        ),
        (&["set", "term-a", "--all"], call(Setting::All("term-a".to_string()))),
        (
            &["clear", "--project=proj-a"],
            call(Setting::Clear("proj-a".to_string())),
        ),
        (
            &[
                "show", "--repo", "/r", "--scribe2", "/f/s2", "--git", "/f/git", "--config",
                "/c/s.toml",
            ],
            TargetCall {
                setting: Setting::Show,
                repo: PathBuf::from("/r"),
                scribe2: "/f/s2".into(),
                git: "/f/git".into(),
                config: Some(PathBuf::from("/c/s.toml")),
            },
        ),
    ];
    for (args, want) in table {
        assert_eq!(parse_target(args), Ok(want), "{args:?}");
    }
    let refusals: [(&[&str], &str); 8] = [
        (&[], "target の命令が無い（show・set・clear）"),
        (&["drop"], "知らない target の命令 drop"),
        (&["set", "term-a"], "target set の形でない"),
        (&["set", "--all", "--project", "p", "term-a"], "target set の形でない"),
        (&["clear", "--all"], "target clear の形でない"),
        (&["show", "--to", "term-a"], "target は旗 --to を受けない"),
        (&["set", "--all", "--all", "term-a"], "--all が 2 度ある"),
        (&["clear", "--project"], "--project の値が無い"),
    ];
    for (args, head) in refusals {
        let e = err(parse_target(args), &format!("{args:?}"));
        assert!(e.starts_with(head), "{args:?}: {e}");
    }
    show_lines();
}

/// 表示先の設定の show の行（上書きと既定と層 A の名）。
fn show_lines() {
    let fixture = fixture_value();
    assert_eq!(
        show(
            &fixture,
            &strings(&["proj-a", "proj-b"]),
            &strings(&["term-a"])
        ),
        [
            "既定 term-a",
            "上書き proj-b term-b",
            "project proj-a term-a（既定）",
            "project proj-b term-b（上書き・層 A に無い）",
            "層 A の名 term-a",
        ]
    );
    assert_eq!(
        show(
            &Targets::default(),
            &strings(&["proj-a"]),
            &strings(&["term-a", "term-b"])
        ),
        [
            "既定 無し",
            "project proj-a 無し（席の目と URL に落ちる）",
            "層 A の名 term-a term-b",
        ]
    );
}

#[test]
fn sttgt_cli_settings() {
    let field = Field::new("settings");
    let (rc, out) = field.tz(&["target", "show"]);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(
        out,
        [
            "既定 無し",
            "project proj-a 無し（席の目と URL に落ちる）",
            "project proj-c 無し（席の目と URL に落ちる）",
            "層 A の名 term-a term-b",
        ]
    );
    assert_eq!(field.written(), None);

    let (rc, out) = field.tz(&["target", "set", "--all", "term-a"]);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, ["全体の既定を term-a にし、project ごとの上書きを 0 個外した"]);
    let (rc, out) = field.tz(&["target", "set", "--project", "proj-c", "term-b"]);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, ["project proj-c の表示先を term-b にした（上書き）"]);
    let both = render(&value(Some("term-a"), &[("proj-c", "term-b")]));
    assert_eq!(field.written().as_deref(), Some(both.as_str()));
    assert_eq!(mode(&field.config), 0o600);

    let (rc, out) = field.tz(&["target", "show"]);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(
        out,
        [
            "既定 term-a",
            "上書き proj-c term-b",
            "project proj-a term-a（既定）",
            "project proj-c term-b（上書き）",
            "層 A の名 term-a term-b",
        ]
    );

    let (rc, out) = field.tz(&["target", "set", "--project", "proj-a", "term-x"]);
    assert_eq!(rc, 1, "{out:?}");
    let refused = err(known("term-x", &strings(&["term-a", "term-b"])), "term-x");
    assert_eq!(out, [refused]);
    assert_eq!(field.written().as_deref(), Some(both.as_str()));
    clear_and_broken(field);
}

/// 上書きを外す・全体の既定で上書きを外す・検証の落ちで書かない。
fn clear_and_broken(field: Field) {
    let (rc, out) = field.tz(&["target", "clear", "--project", "proj-c"]);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, ["project proj-c の上書きを外した（効く値は既定 term-a）"]);
    let (rc, out) = field.tz(&["target", "clear", "--project", "proj-c"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out, ["project proj-c に上書きは無い"]);

    let (rc, out) = field.tz(&["target", "set", "--project", "proj-a", "term-b"]);
    assert_eq!(rc, 0, "{out:?}");
    let (rc, out) = field.tz(&["target", "set", "--all", "term-b"]);
    assert_eq!(rc, 0, "{out:?}");
    assert_eq!(out, ["全体の既定を term-b にし、project ごとの上書きを 1 個外した"]);
    let last = render(&value(Some("term-b"), &[]));
    assert_eq!(field.written().as_deref(), Some(last.as_str()));

    fs::write(&field.broken, "").expect("印の file");
    let (rc, out) = field.tz(&["target", "set", "--all", "term-a"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 1, "{out:?}");
    for word in ["validate --state-dir", "rc 0"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    assert_eq!(field.written().as_deref(), Some(last.as_str()));
}

#[test]
fn sttgt_cli_refusals() {
    let field = Field::new("refusals");
    let (rc, out) = field.tz(&["open"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    assert!(
        out[0].starts_with(
            "tz stage open に --to が無く、表示先の設定に project proj-a の値も既定も無い"
        ),
        "{}",
        out[0]
    );
    assert_eq!(out[1], LINE);
    assert_eq!(field.written(), None);

    save(&field.config, &value(Some("term-z"), &[])).expect("既定 term-z の設定");
    let (rc, out) = field.tz(&["reload"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(
        out,
        [
            "表示先の設定の project proj-a に効く既定の端末 term-z は層 A（host の面の [[device]]）に無い（既定へ落とさない・在る名は term-a term-b）",
            LINE,
        ]
    );

    fs::write(&field.config, format!("{HEAD}\n[screen]\n")).expect("形の外れた設定");
    let (rc, out) = field.tz(&["reload"]);
    assert_eq!(rc, 1, "{out:?}");
    assert_eq!(out.len(), 2, "{out:?}");
    let shown = field.config.display().to_string();
    for word in ["表示先の設定", shown.as_str(), "の 2 行目", "知らない表 [screen]"] {
        assert!(out[0].contains(word), "{word}: {}", out[0]);
    }
    assert_eq!(out[1], LINE);
}

#[test]
fn sttgt_source_guards() {
    let text = src("target.rs");
    for word in [
        "bringToFront",
        "activateTarget",
        "createTarget",
        "setWindowBounds",
        "json/new",
        "json/activate",
        "Runtime.evaluate",
        "CLAUDECODE",
        "unsafe",
        "Command::new",
    ] {
        assert!(!text.contains(word), "target.rs が {word} を含む");
    }
    assert_eq!(src("mod.rs").matches("pub mod target;").count(), 1);
    let cli_text = src("cli.rs");
    for line in [
        "tunnel.window(board, hint.as_deref(), first)",
        "target::mark(path, &name, now())",
        "(true, Some(_)) => Some(lock(&base, &name, LOCK_WAIT)?)",
        "!target::load(path)?.shown.contains_key(&name)",
    ] {
        assert_eq!(cli_text.matches(line).count(), 1, "{line}");
    }
}

#[test]
fn sttgt_names_apart() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sttgt.rs");
    let text = fs::read_to_string(&path).expect("tests/sttgt.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 10, "歯の数");
    for name in names {
        let rest = name
            .strip_prefix("sttgt_")
            .unwrap_or_else(|| panic!("{name} は sttgt_ で始まらない"));
        for word in FILTERS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
