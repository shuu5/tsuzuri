//! tsuzuri の plugin の相談の窓の skill と tz の解き方の歯（接頭辞 cwplg_・設計ノート surface-wave27b 行 cs-plugin）。
//! workspace の根の plugin/ の file を読み、bin/tzw は歯ごとの置き場（CARGO_TARGET_TMPDIR の下）の plugin の形の dir に
//! plugin.json と一緒に写して撃つ（偽の tz は受けた引数と環境を記録する・PATH は /usr/bin と /bin と偽の dir だけ）。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tsuzuri_core::consult::launch::PLUGIN_VERSION;

/// skill の本文が字のまま持つ句。
const SKILL_WORDS: [&str; 9] = [
    "tzw consult open",
    "tzw consult launch",
    "tzw consult watch",
    "tzw consult show",
    "tzw consult dispose",
    "run_in_background",
    "R-38",
    "承認にならない",
    "! で打つ命令は囲いの外で走る",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("偽の program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("偽の program の権限");
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .expect("git");
    assert!(ok.success(), "git {args:?}");
}

/// 歯ごとの置き場（plugin の形の dir・同じ checkout の build の置き場・PATH の偽の dir・宣言の在る repo と無い repo）。
struct Fx {
    work: PathBuf,
    plugin: PathBuf,
}

impl Fx {
    fn new(name: &str) -> Fx {
        let work = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cwplg")
            .join(name);
        let _ = fs::remove_dir_all(&work);
        let plugin = work.join("plugin");
        for d in [
            "plugin/bin",
            "plugin/.claude-plugin",
            "target/debug",
            "path",
            "decl",
            "bare",
        ] {
            fs::create_dir_all(work.join(d)).expect("置き場");
        }
        for rel in ["bin/tzw", ".claude-plugin/plugin.json"] {
            fs::copy(root().join("plugin").join(rel), plugin.join(rel))
                .expect("plugin の file を写す");
        }
        git(&work.join("decl"), &["init", "-q"]);
        git(
            &work.join("decl"),
            &["config", "scribe2.statedir", "/nonexistent/state"],
        );
        git(&work.join("bare"), &["init", "-q"]);
        Fx { work, plugin }
    }

    /// 偽の tz を置く（`at` は置き場からの相対・`who` は記録の頭の字）。
    fn tz(&self, at: &str, who: &str) {
        let rec = self.work.join("rec");
        script(
            &self.work.join(at),
            &format!(
                "{{ echo '{who}'; for a in \"$@\"; do printf '%s\\n' \"$a\"; done; echo \"v=$TZ_PLUGIN_VERSION\"; echo \"w=$TZ_WRAPPER\"; }} > '{}'\necho out",
                rec.display()
            ),
        );
    }

    /// plugin の bin/tzw を撃つ（`project` は環境の CLAUDE_PROJECT_DIR の dir の名）。
    fn tzw(&self, project: &str, args: &[&str]) -> Output {
        let _ = fs::remove_file(self.work.join("rec"));
        Command::new(self.plugin.join("bin/tzw"))
            .args(args)
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.work.join("path").display()),
            )
            .env("CLAUDE_PROJECT_DIR", self.work.join(project))
            .env_remove("TZ_PLUGIN_VERSION")
            .env_remove("TZ_WRAPPER")
            .output()
            .expect("tzw を撃つ")
    }

    /// 偽の tz の記録の行（撃たれていなければ None）。
    fn rec(&self) -> Option<Vec<String>> {
        let text = fs::read_to_string(self.work.join("rec")).ok()?;
        Some(text.lines().map(String::from).collect())
    }

    fn want(&self, who: &str, args: &[&str]) -> Option<Vec<String>> {
        let mut out = vec![who.to_string()];
        out.extend(args.iter().map(|a| (*a).to_string()));
        out.push(format!("v={PLUGIN_VERSION}"));
        let here = self
            .plugin
            .join("bin")
            .canonicalize()
            .expect("bin の正しい path");
        out.push(format!("w={}", here.join("tzw").display()));
        Some(out)
    }
}

/// (1) plugin.json の version の字は中核の PLUGIN_VERSION と同じで、鍵は description・name・version の 3 つのまま。
#[test]
fn cwplg_version_is_plugin_version() {
    let plugin: Value = serde_json::from_str(&read("plugin/.claude-plugin/plugin.json"))
        .expect("plugin.json は JSON");
    assert_eq!(plugin["version"].as_str(), Some(PLUGIN_VERSION));
    let mut keys: Vec<&str> = plugin
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["description", "name", "version"]);
}

/// (2) tzw は plugin と同じ checkout の build（plugin の根の親の target/debug/tz）を PATH の tz より先に撃ち、
/// 無ければ PATH の tz を撃ち、どちらにも引数をそのまま渡し、環境に plugin.json の版の字と自分の絶対 path を置く。
#[test]
fn cwplg_tzw_resolves_checkout_then_path() {
    let fx = Fx::new("resolve");
    fx.tz("target/debug/tz", "checkout");
    fx.tz("path/tz", "path");
    let out = fx.tzw("decl", &["consult", "list", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "out\n");
    assert_eq!(
        fx.rec(),
        fx.want("checkout", &["consult", "list", "--json"])
    );
    fs::remove_file(fx.work.join("target/debug/tz")).expect("checkout の tz を外す");
    let out = fx.tzw("decl", &["consult", "list"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(fx.rec(), fx.want("path", &["consult", "list"]));
}

/// (3) tz が解けない時、hook の口は何も撃たずに rc 0、ほかの口は字 tzw と半角のコロンで始まる 1 行を標準エラーに出して rc 1。
#[test]
fn cwplg_tzw_without_tz() {
    let fx = Fx::new("none");
    let out = fx.tzw("decl", &["hook", "stop", "--repo", "x"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    let out = fx.tzw("decl", &["consult", "list"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.starts_with("tzw: ") && err.lines().count() == 1,
        "{err}"
    );
    assert_eq!(fx.rec(), None);
}

/// (4) hook の口は、CLAUDE_PROJECT_DIR の repo の git config に scribe2.statedir が無ければ tz を撃たずに rc 0 で、
/// 在れば撃つ。hook でない口は宣言を見ない（宣言の無い repo でも撃つ）。
#[test]
fn cwplg_hook_needs_declaration() {
    let fx = Fx::new("decl");
    fx.tz("target/debug/tz", "checkout");
    let args = ["hook", "stop", "--repo", "r"];
    let out = fx.tzw("bare", &args);
    assert_eq!((out.status.code(), fx.rec()), (Some(0), None), "{out:?}");
    let out = fx.tzw("decl", &args);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(fx.rec(), fx.want("checkout", &args));
    let out = fx.tzw("bare", &["consult", "list"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(fx.rec(), fx.want("checkout", &["consult", "list"]));
}

/// (5) skill は plugin/skills/consult/SKILL.md で、頭の --- の間に name の値 consult と空でない description を持ち、
/// disable-model-invocation を持たず（席も撃てる）、本文は SKILL_WORDS の 9 句を字のまま持ち、字 target/debug/tz を持たない。
#[test]
fn cwplg_skill_shape() {
    let text = read("plugin/skills/consult/SKILL.md");
    let mut parts = text.splitn(3, "---\n");
    assert_eq!(parts.next(), Some(""), "頭は ---");
    let head = parts.next().expect("頭の --- の間");
    let body = parts.next().expect("本文");
    let lines: Vec<&str> = head.lines().collect();
    assert!(lines.contains(&"name: consult"), "{head}");
    assert!(
        lines.iter().any(|l| l
            .strip_prefix("description: ")
            .is_some_and(|d| !d.trim().is_empty())),
        "{head}"
    );
    assert!(!head.contains("disable-model-invocation"), "{head}");
    for word in SKILL_WORDS {
        assert!(body.contains(word), "本文に {word} が無い");
    }
    assert!(!text.contains("target/debug/tz"), "tz を直に名指す");
}

/// (6) tzw は字 #!/bin/sh で始まり、持ち主と group とほかの者の全部が撃てる mode の file。
#[test]
fn cwplg_tzw_is_executable_sh() {
    let path = root().join("plugin/bin/tzw");
    let mode = fs::metadata(&path).expect("tzw").permissions().mode();
    assert_eq!(mode & 0o111, 0o111, "{mode:o}");
    assert!(read("plugin/bin/tzw").starts_with("#!/bin/sh\n"));
}
