//! 端末の一覧の読みの歯（行 i-1・接頭辞 stage_term_）。
//! fixture は偽の端末の 2 行（term-a と term-b）を持つ host の面の字で、直した本文は fixture の字の中の
//! 最初の一致（term-a の行）を字の置き換えで直して組む。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::stage::terminal::{
    self, FACE, HEADER, KEYS, Os, Terminal, lookup, names, read_face,
};

const FIXTURE: &str = "stage/terminals.toml";

/// 行 i-1 の外で決めた filter の語（この行の接頭辞 stage_term_ は並べない）。
const FILTER_WORDS: [&str; 111] = [
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
    "bport_",
    "fmark_",
    "fstop_",
    "fserve_",
    "nsumw_",
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
    "cgraph_",
    "csled_",
    "lhome_",
    "shb_",
    "ghb_",
    "nact_",
    "aord_",
    "mtree_",
    "cgdom_",
    "rhold_",
    "qkey_",
    "stage_cdp_",
];

/// 節の表（done (8) の 14 通り・直す前の字・直した字・Err が角括弧で囲んで名指す字）。
const FORMS: [(&str, &str, &str); 14] = [
    ("display = \":0\"", "display_env = [\"A=1\"]", "display_env"),
    ("os = \"linux\"", "os = \"linux\"\nos = \"linux\"", "os"),
    ("os = \"linux\"\n", "", "os"),
    ("os = \"linux\"", "os = \"Linux\"", "os"),
    ("ssh = \"me@term-a\"\n", "", "ssh"),
    ("ssh = \"me@term-a\"", "ssh = \"me term-a\"", "ssh"),
    ("ssh = \"me@term-a\"", "ssh = \"-oX\"", "ssh"),
    ("chrome = \"/fake/bin/chrome\"", "chrome = \"\"", "chrome"),
    ("display = \":0\"", "display = :0", "display"),
    ("\"XMODIFIERS=@im=fcitx\"", "\"XMODIFIERS\"", "ime-env"),
    ("\"XMODIFIERS=@im=fcitx\"", "\"xmods=1\"", "ime-env"),
    ("\"XMODIFIERS=@im=fcitx\"", "\"9K=1\"", "ime-env"),
    ("\"XMODIFIERS=@im=fcitx\"", "\"K=\"", "ime-env"),
    (
        "profile-dir = \"/fake/profile-a\"",
        "oops\nprofile-dir = \"/fake/profile-a\"",
        "oops",
    ),
];

fn fixture() -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(FIXTURE),
    )
    .unwrap_or_else(|e| panic!("{FIXTURE} を読む: {e}"))
}

/// fixture の字の中の最初の `from`（term-a の行の中）を `to` に替えた本文。
fn edited(from: &str, to: &str) -> String {
    let text = fixture();
    let at = text.find(from).unwrap_or_else(|| panic!("fixture に {from:?} が無い"));
    let second = text.rfind(HEADER).expect("2 つ目の見出し");
    assert!(at < second, "{from:?} は term-a の行の中");
    text.replacen(from, to, 1)
}

fn term_a() -> Terminal {
    Terminal {
        name: "term-a".to_string(),
        ssh: "me@term-a".to_string(),
        chrome: "/fake/bin/chrome".to_string(),
        os: Os::Linux,
        display: Some(":0".to_string()),
        ime_env: vec![
            ("GTK_IM_MODULE".to_string(), "fcitx".to_string()),
            ("XMODIFIERS".to_string(), "@im=fcitx".to_string()),
        ],
        profile_dir: "/fake/profile-a".to_string(),
        display_env: Vec::new(),
    }
}

fn term_b() -> Terminal {
    Terminal {
        name: "term-b".to_string(),
        ssh: "me@term-b".to_string(),
        chrome: "/fake/bin/chrome".to_string(),
        os: Os::Linux,
        display: None,
        ime_env: Vec::new(),
        profile_dir: "/fake/profile-b".to_string(),
        display_env: vec![
            ("WAYLAND_DISPLAY".to_string(), "wayland-1".to_string()),
            ("XDG_RUNTIME_DIR".to_string(), "/fake/run-b".to_string()),
        ],
    }
}

/// 型を書いた fn の値に束ねた lookup。
const LOOKUP: fn(&str, &str) -> Result<Terminal, String> = lookup;

fn refused(text: &str, name: &str) -> String {
    LOOKUP(text, name).expect_err("Err のはず")
}

/// 歯ごとの作業場（CARGO_TARGET_TMPDIR の下の stterm と歯の名の dir・前の回の残りは消す）。
fn workdir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("stterm")
        .join(test);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("作業場");
    dir
}

#[test]
fn stage_term_shape_and_words() {
    assert_eq!(HEADER, "[[device]]");
    assert_eq!(FACE, "host.toml");
    assert_eq!(
        KEYS,
        ["name", "ssh", "chrome", "os", "display", "ime-env", "profile-dir", "display-env"]
    );
    assert_eq!(Os::ALL, [Os::Linux, Os::Macos, Os::Windows]);
    assert_eq!(
        Os::ALL.map(Os::word),
        ["linux", "macos", "windows"]
    );
    let _: fn(&str, &str) -> Result<Terminal, String> = terminal::lookup;
    let names_fn: fn(&str) -> Vec<String> = terminal::names;
    let read_fn: fn(&Path) -> Result<String, String> = terminal::read_face;
    let _ = (names_fn, read_fn);
    for (word, os) in [("macos", Os::Macos), ("windows", Os::Windows)] {
        let text = edited("os = \"linux\"", &format!("os = \"{word}\""));
        assert_eq!(LOOKUP(&text, "term-a").expect("引ける").os, os, "{word}");
    }
}

#[test]
fn stage_term_reads_fixture_rows() {
    let text = fixture();
    assert_eq!(
        text.lines().filter(|l| *l == HEADER).count(),
        2,
        "見出しの行の数"
    );
    assert_eq!(LOOKUP(&text, "term-a"), Ok(term_a()));
    assert_eq!(LOOKUP(&text, "term-b"), Ok(term_b()));
}

#[test]
fn stage_term_names_in_order() {
    let text = fixture();
    assert_eq!(names(&text), vec!["term-a".to_string(), "term-b".to_string()]);
    let head = &text[..text.find(HEADER).expect("見出し")];
    assert!(head.contains("name = \"grp-a\""), "前の字は表 account-group を持つ");
    assert!(names(head).is_empty(), "見出しより前の字");
    assert!(names("").is_empty(), "空の字");
}

#[test]
fn stage_term_reads_face_file() {
    let dir = workdir("stage_term_reads_face_file");
    let face = dir.join("face");
    let empty = dir.join("empty");
    fs::create_dir_all(&face).expect("face の dir");
    fs::create_dir_all(&empty).expect("空の dir");
    fs::write(face.join("host.toml"), fixture()).expect("host.toml を書く");
    assert_eq!(read_face(&face), Ok(fixture()));
    let err = read_face(&empty).expect_err("host.toml の無い dir");
    let path = empty.join("host.toml");
    assert!(err.contains(&path.display().to_string()), "{err}");
}

#[test]
fn stage_term_unknown_or_twice_refused() {
    let text = fixture();
    let err = refused(&text, "term-c");
    for word in ["term-c", "term-a", "term-b"] {
        assert!(err.contains(word), "{word} が無い: {err}");
    }
    refused(&text, "grp-a");
    refused("", "term-a");
    let twice = format!(
        "{text}\n{HEADER}\nname = \"term-a\"\nssh = \"me@term-a\"\nchrome = \"/fake/bin/chrome\"\nos = \"linux\"\nprofile-dir = \"/fake/profile-a\"\n"
    );
    let err = refused(&twice, "term-a");
    assert!(err.contains("term-a"), "{err}");
}

#[test]
fn stage_term_profile_dir_required() {
    let text = fixture();
    let from = "profile-dir = \"/fake/profile-b\"";
    assert!(text.contains(from));
    let text = text.replacen(from, "", 1);
    let err = refused(&text, "term-b");
    assert!(err.contains("term-b") && err.contains("profile-dir"), "{err}");
    assert_eq!(LOOKUP(&text, "term-a"), Ok(term_a()));
}

#[test]
fn stage_term_display_x_only() {
    for ok in [":1.0", ":12"] {
        let text = edited("display = \":0\"", &format!("display = \"{ok}\""));
        let want = Terminal {
            display: Some(ok.to_string()),
            ..term_a()
        };
        assert_eq!(LOOKUP(&text, "term-a"), Ok(want), "{ok}");
        assert_eq!(LOOKUP(&text, "term-b"), Ok(term_b()), "{ok}");
    }
    for bad in ["wayland-1", ":", ":a", ":0.", "0", "host:0"] {
        let text = edited("display = \":0\"", &format!("display = \"{bad}\""));
        let err = refused(&text, "term-a");
        assert!(
            err.contains("term-a") && err.contains("display-env"),
            "{bad}: {err}"
        );
        assert_eq!(LOOKUP(&text, "term-b"), Ok(term_b()), "{bad}");
    }
}

#[test]
fn stage_term_value_forms_refused() {
    for (from, to, key) in FORMS {
        let text = edited(from, to);
        let err = refused(&text, "term-a");
        assert!(err.contains("term-a"), "{to:?}: {err}");
        assert!(err.contains(&format!("[{key}]")), "{to:?}: [{key}] が無い: {err}");
        for other in KEYS.iter().filter(|k| **k != key) {
            assert!(
                !err.contains(&format!("[{other}]")),
                "{to:?}: [{other}] を名指す: {err}"
            );
        }
    }
}

#[test]
fn stage_term_pure_reader() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/stage/terminal.rs");
    let src = fs::read_to_string(&path).expect("src/stage/terminal.rs を読む");
    let print = ["print", "!"].concat();
    let println = ["println", "!"].concat();
    for word in ["unsafe", "std::env", "std::process", "std::net", &print, &println] {
        assert!(!src.contains(word), "{word} を含む");
    }
}

#[test]
fn stage_term_own_names_clean() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/stterm.rs");
    let src = fs::read_to_string(&path).expect("tests/stterm.rs を読む");
    let lines: Vec<&str> = src.lines().collect();
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
            .strip_prefix("stage_term_")
            .unwrap_or_else(|| panic!("{name} は stage_term_ で始まらない"));
        for word in FILTER_WORDS {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
