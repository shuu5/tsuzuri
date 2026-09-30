//! 画面を開く env の欄 display-env の読みの歯（行 i-1b・接頭辞 denv_）。
//! 直した本文は、fixture の字にちょうど 1 つ在る字（term-b の display-env の行か、term-a の display の行）を
//! 字の置き換えで直して組む。
#![cfg(test)]

use std::fs;
use std::path::Path;

use tsuzuri_boundary::stage::terminal::{HEADER, KEYS, Os, Terminal, lookup};

const FIXTURE: &str = "stage/terminals.toml";

/// fixture の term-b の display-env の行。
const DENV_LINE: &str =
    "display-env = [\"WAYLAND_DISPLAY=wayland-1\", \"XDG_RUNTIME_DIR=/fake/run-b\"]";

/// fixture の term-a の display の行。
const DISPLAY_LINE: &str = "display = \":0\"";

/// 行 i-1b の外で決めた filter の語（空白で区切る・この行の接頭辞 denv_ は並べない）。
const FILTER_WORDS: &str = concat!(
    "aaround_ accept_ account_ acctcore_ acctdoc_ accthb_ accthome_ acctled_ acctlook_ acctpcore_ ",
    "acctproj_ acctsess_ acctwin_ acctwire_ afocus_ aord_ apop_ askcard_ athr_ batchpanel_ ",
    "board_min_ bport_ brand_ btuck_ cadopt_ cgdom_ contract_form_ cround_ csled_ flight_ ",
    "fmark_ frame_ fserve_ fstop_ gapspage_ ghb_ gnav_ graph_ gsum_ gtuck_ ",
    "gview_ hbconf_ hbpost_ hbproc_ hbroute_ hcard_ hcproj_ hcsess_ hfig_ hook_ ",
    "hruling_ hsblock_ hsderive_ hspage_ hsym_ ilink_ kcli_ klink_ lcard_ ledgerblock_ ",
    "lhome_ lspark_ mapview_ mkeys_ mlink_ mstore_ mtree_ nact_ nbatch_ ncard_ ",
    "nextstep_ nodepage_ nstall_ nsum_ nsumw_ ntime_ nxact_ parts_ pclosed_ pfold_ ",
    "pipe_ plimit_ pmore_ project_ ptitle_ pwhole_ qblock_ qgate_ qkey_ question_ ",
    "rhold_ runsdoc_ saxis_ seatblock_ seatcard_ server_ sesplit_ shb_ skeleton_ smore_ ",
    "stage_ stats_ steady_ sxaxis_ ticker_ tipx_ topbar_ tz_ urpanel_ uword_ ",
    "wstrip_",
);

/// 節の表（done (5) の 10 通り・term-b の display-env の行を丸ごと替える字）。
const PAIR_FORMS: [&str; 10] = [
    "display-env = \"WAYLAND_DISPLAY=wayland-1\"",
    "display-env = [WAYLAND_DISPLAY=wayland-1]",
    "display-env = [\"WAYLAND_DISPLAY\"]",
    "display-env = [\"wayland_display=wayland-1\"]",
    "display-env = [\"9W=wayland-1\"]",
    "display-env = [\"WAYLAND_DISPLAY=\"]",
    "display-env = [\"=wayland-1\"]",
    "display-env = [\"WAYLAND_DISPLAY=wayland-1\", \"WAYLAND_DISPLAY=wayland-2\"]",
    "display-env = [\"WAYLAND_DISPLAY=wayland-1\"",
    concat!(
        "display-env = [\"WAYLAND_DISPLAY=wayland-1\", \"XDG_RUNTIME_DIR=/fake/run-b\"]\n",
        "display-env = [\"WAYLAND_DISPLAY=wayland-1\", \"XDG_RUNTIME_DIR=/fake/run-b\"]",
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

/// fixture の字にちょうど 1 つ在る `from` を `to` に替えた本文。
fn edited(from: &str, to: &str) -> String {
    let text = fixture();
    assert_eq!(text.matches(from).count(), 1, "{from:?} は fixture にちょうど 1 つ");
    text.replacen(from, to, 1)
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn term_a() -> Terminal {
    Terminal {
        name: "term-a".to_string(),
        ssh: "me@term-a".to_string(),
        chrome: "/fake/bin/chrome".to_string(),
        os: Os::Linux,
        display: Some(":0".to_string()),
        ime_env: pairs(&[("GTK_IM_MODULE", "fcitx"), ("XMODIFIERS", "@im=fcitx")]),
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
        display_env: pairs(&[
            ("WAYLAND_DISPLAY", "wayland-1"),
            ("XDG_RUNTIME_DIR", "/fake/run-b"),
        ]),
    }
}

/// Err が字 `name` と角括弧で囲んだ `key` を含み、KEYS のほかの key を角括弧で囲んだ字を含まないこと。
fn names_only(err: &str, name: &str, key: &str, case: &str) {
    assert!(err.contains(name), "{case}: {name} が無い: {err}");
    assert!(err.contains(&format!("[{key}]")), "{case}: [{key}] が無い: {err}");
    for other in KEYS.iter().filter(|k| **k != key) {
        assert!(
            !err.contains(&format!("[{other}]")),
            "{case}: [{other}] を名指す: {err}"
        );
    }
}

#[test]
fn denv_key_and_pairs_read() {
    assert_eq!(
        KEYS,
        ["name", "ssh", "chrome", "os", "display", "ime-env", "profile-dir", "display-env"]
    );
    let text = fixture();
    let lines: Vec<&str> = text.lines().collect();
    let denv: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with("display-env"))
        .map(|(i, _)| i)
        .collect();
    let last = lines
        .iter()
        .rposition(|l| *l == HEADER)
        .expect("見出しの行");
    assert_eq!(denv.len(), 1, "display-env で始まる行の数");
    assert!(denv[0] > last, "display-env の行は最後の見出しより後");
    assert_eq!(lines[denv[0]], DENV_LINE);
    assert_eq!(lines[denv[0] - 1], "os = \"linux\"");
    assert_eq!(lines[denv[0] + 1], "profile-dir = \"/fake/profile-b\"");
    assert_eq!(lookup(&text, "term-b"), Ok(term_b()));
    assert_eq!(lookup(&text, "term-a"), Ok(term_a()));
}

#[test]
fn denv_x_screen_as_env() {
    let cases: [(&str, &[(&str, &str)]); 2] = [
        ("display-env = [\"DISPLAY=:0\"]", &[("DISPLAY", ":0")]),
        (
            "display-env = [\"XDG_RUNTIME_DIR=/fake/run-b\", \"WAYLAND_DISPLAY=wayland-1\"]",
            &[
                ("XDG_RUNTIME_DIR", "/fake/run-b"),
                ("WAYLAND_DISPLAY", "wayland-1"),
            ],
        ),
    ];
    for (to, want) in cases {
        let text = edited(DENV_LINE, to);
        let want = Terminal {
            display_env: pairs(want),
            ..term_b()
        };
        assert_eq!(lookup(&text, "term-b"), Ok(want), "{to}");
    }
}

#[test]
fn denv_both_refused() {
    for value in ["[\"A=1\"]", "[\"DISPLAY=:0\"]", "[]"] {
        let to = format!("{DISPLAY_LINE}\ndisplay-env = {value}");
        let text = edited(DISPLAY_LINE, &to);
        let err = lookup(&text, "term-a").expect_err("両方の行");
        names_only(&err, "term-a", "display-env", &to);
        assert_eq!(lookup(&text, "term-b"), Ok(term_b()), "{to}");
    }
    let to = "display = \"wayland-1\"\ndisplay-env = [\"A=1\"]";
    let text = edited(DISPLAY_LINE, to);
    let err = lookup(&text, "term-a").expect_err("X の形でない display と両方");
    names_only(&err, "term-a", "display-env", to);
    let to = "display = \"wayland-1\"";
    let text = edited(DISPLAY_LINE, to);
    let err = lookup(&text, "term-a").expect_err("X の形でない display");
    names_only(&err, "term-a", "display", to);
    assert!(err.contains("display-env"), "{err}");
}

#[test]
fn denv_pair_forms_refused() {
    for to in PAIR_FORMS {
        let text = edited(DENV_LINE, to);
        let err = lookup(&text, "term-b").expect_err("形の外");
        names_only(&err, "term-b", "display-env", to);
        assert_eq!(lookup(&text, "term-a"), Ok(term_a()), "{to}");
    }
}

#[test]
fn denv_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 111, "filter の語の数");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/stdenv.rs");
    let src = fs::read_to_string(&path).expect("tests/stdenv.rs を読む");
    let lines: Vec<&str> = src.lines().collect();
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
            .strip_prefix("denv_")
            .unwrap_or_else(|| panic!("{name} は denv_ で始まらない"));
        for word in &words {
            assert!(!rest.contains(word), "{name} が {word} を含む");
        }
    }
}
