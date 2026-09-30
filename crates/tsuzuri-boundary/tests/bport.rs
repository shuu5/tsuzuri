//! project board の port の読みの歯（接頭辞 bport_・設計ノート surface-wave5b 便 h-board-url の完了の条件）。
//! 偽の git は受けた argv を記録の置き場に 1 行ずつ足し、-C の次の path の最後の区切り（末尾の「/」を除く）と
//! 鍵（argv の 5 つ目）の組で作業場の git の下の file を選んで出す script（file が無ければ rc 1）。
//! 偽の器は argv を問わず rc 1 で終わり、bd の program は無い path。字の中の path は実行の時に組む（行 D-4）。
#![cfg(test)]

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::acct::{Acct, board_port};
use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::wire::encode;
use tsuzuri_core::account::host::HostTexts;
use tsuzuri_core::account::project::{self, ProjectTexts};

/// 読みの今の時刻（2026-09-27T12:00:00Z）。
const NOW: u64 = 1_790_510_400;

/// 無い program の path。
const NO_PROGRAM: &str = "/nonexistent/tz-no-such-program";

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

/// 歯ごとの作業場。
struct Place {
    root: PathBuf,
    work: PathBuf,
    states: PathBuf,
    host_toml: String,
}

impl Place {
    fn new(name: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("bport")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        let place = Place {
            work: root.join("work"),
            states: root.join("states"),
            host_toml: String::new(),
            root,
        };
        for dir in ["bin", "git", "log", "repo"] {
            fs::create_dir_all(place.root.join(dir)).expect("置き場");
        }
        for p in ["proj-a", "proj-b", "proj-c", "proj-d", "proj-e"] {
            fs::create_dir_all(place.work.join(p)).expect("anchor");
        }
        fs::create_dir_all(place.states.join("state-h")).expect("引数の state dir");
        fs::create_dir_all(place.states.join("state-a")).expect("state dir");
        let root = place.root.display().to_string();
        script(
            &place.program("git"),
            &"printf '%s\\n' \"$*\" >> 'ROOT/log/git'\n\
              f=\"${2%/}\"; f=\"${f##*/}\"\n\
              exec cat 'ROOT/git/'\"$f.$5\""
                .replace("ROOT", &root),
        );
        script(&place.program("scribe2"), "exit 1");
        place.lay()
    }

    fn program(&self, name: &str) -> PathBuf {
        self.root.join("bin").join(name)
    }

    fn anchor(&self, project: &str) -> String {
        self.work.join(project).display().to_string()
    }

    fn state(&self, name: &str) -> PathBuf {
        self.states.join(name)
    }

    fn git(&self, project: &str, key: &str, text: &str) {
        fs::write(self.root.join("git").join(format!("{project}.{key}")), text)
            .expect("偽の git の字");
    }

    /// 字を組んで置く（anchor の path は実行の時の作業場から）。
    fn lay(mut self) -> Place {
        let [a, b, c, d, e] =
            ["proj-a", "proj-b", "proj-c", "proj-d", "proj-e"].map(|p| self.anchor(p));
        self.host_toml = format!(
            "[[account-group]]\nname = \"g-a\"\nanchors = [\"{a}\", \"{b}\", \"{c}/\"]\n\n\
             [[account-group]]\nname = \"g-b\"\nanchors = [\"{d}\", \"{e}\", \"{a}\"]\n"
        );
        fs::write(self.state("state-h").join("host.toml"), &self.host_toml).expect("host.toml");
        let sa = self.state("state-a").display().to_string();
        self.git("proj-a", "scribe2.statedir", &format!("{sa}\n"));
        self.git("proj-a", "tsuzuri.boardport", "40001\n");
        self.git("proj-b", "tsuzuri.boardport", " 40002 \n");
        self.git("proj-c", "tsuzuri.boardport", "host-1:40003\n");
        self.git("proj-e", "tsuzuri.boardport", "  \n");
        self
    }

    fn acct_with(&self, git: impl Into<PathBuf>) -> Acct {
        Acct::new(
            self.program("scribe2"),
            git.into(),
            NO_PROGRAM,
            self.state("state-h"),
            self.root.join("repo"),
        )
    }

    fn acct(&self) -> Acct {
        self.acct_with(self.program("git"))
    }

    /// 作業場と同じ字を中核の組み立ての入口に直に渡した電文（board はどれも None）。
    fn core_doc(&self) -> AccountDoc {
        let host = HostTexts {
            host_toml: Some(self.host_toml.clone()),
            ..HostTexts::default()
        };
        let mut projects = BTreeMap::new();
        for anchor in [
            self.anchor("proj-a"),
            self.anchor("proj-b"),
            format!("{}/", self.anchor("proj-c")),
            self.anchor("proj-d"),
            self.anchor("proj-e"),
        ] {
            projects.insert(anchor, ProjectTexts::default());
        }
        projects.insert(
            self.anchor("proj-a"),
            ProjectTexts {
                state_dir_known: true,
                ..ProjectTexts::default()
            },
        );
        project::doc(&host, &projects, NOW)
    }
}

#[test]
fn bport_port_text_rules() {
    for (text, want) in [
        ("40001", Some(40001)),
        (" 40001\n", Some(40001)),
        ("1", Some(1)),
        ("65535", Some(65535)),
        ("08121", Some(8121)),
    ] {
        assert_eq!(board_port(text), want, "{text:?}");
    }
    for text in [
        "0",
        "65536",
        "99999999999",
        "",
        "  \n",
        "+40001",
        "-1",
        "40001/",
        ":40001",
        "host-1:40001",
        "40 001",
        "abc",
    ] {
        assert_eq!(board_port(text), None, "{text:?}");
    }
}

#[test]
fn bport_git_key_per_anchor() {
    let place = Place::new("key");
    let acct = place.acct();
    let board = |p: &str| acct.board(&place.work.join(p));
    assert_eq!(board("proj-a").as_deref(), Some("40001"));
    assert_eq!(
        board("proj-b").as_deref(),
        Some("40002"),
        "前後の空白を除く"
    );
    assert_eq!(
        board("proj-c").as_deref(),
        Some("host-1:40003"),
        "port かどうかは見ない"
    );
    assert_eq!(board("proj-d"), None, "git が落ちる");
    assert_eq!(board("proj-e"), None, "空白だけ");
    assert_eq!(
        place
            .acct_with(NO_PROGRAM)
            .board(&place.work.join("proj-a")),
        None,
        "git が撃てない"
    );
    assert_eq!(
        acct.state_dir(&place.work.join("proj-a")),
        Some(place.state("state-a"))
    );
    assert_eq!(acct.state_dir(&place.work.join("proj-b")), None);
}

#[test]
fn bport_doc_carries_port_only() {
    let place = Place::new("doc");
    let got = place.acct().doc(NOW);
    let names: Vec<&str> = got.projects.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["proj-a", "proj-b", "proj-c", "proj-d", "proj-e"]);
    let boards: Vec<Option<u16>> = got.projects.iter().map(|r| r.board).collect();
    assert_eq!(boards, [Some(40001), Some(40002), None, None, None]);
    assert!(got.projects[0].state_dir_known);
    assert!(!got.projects[1].state_dir_known);
    let mut bare = got.clone();
    for row in &mut bare.projects {
        row.board = None;
    }
    assert_eq!(bare, place.core_doc(), "board のほかは中核の電文のまま");
    let text = encode(&got).expect("電文の字");
    assert_eq!(text.matches("\"board\":40001").count(), 1, "{text}");
    assert_eq!(text.matches("\"board\":40002").count(), 1, "{text}");
    assert!(!text.contains("host-1"), "{text}");
}

/// 着地済みの歯と別の波の行の接頭辞（新しい歯の名はこれを部分の字として含まない）。
const FILTER_WORDS: [&str; 80] = [
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
    "hcard_",
    "qgate_",
    "nsum_",
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
];

#[test]
fn bport_names_filtered() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/bport.rs"))
        .expect("歯の file");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0] == "#[test]")
        .map(|w| {
            let rest = w[1].strip_prefix("fn ").expect("test の属性の次は fn");
            &rest[..rest.find('(').expect("fn の名")]
        })
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    for name in names {
        let tail = name
            .strip_prefix("bport_")
            .unwrap_or_else(|| panic!("接頭辞: {name}"));
        for word in FILTER_WORDS {
            assert!(!tail.contains(word), "{name} が {word} を含む");
        }
    }
}
