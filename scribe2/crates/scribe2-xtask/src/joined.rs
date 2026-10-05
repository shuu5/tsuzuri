//! 根に入った形の木の読み（消費側の repo の根の workspace の member になった器・自分の `Cargo.toml` を持たない root）。
//!
//! 器の root が自分の workspace の manifest を持つ木（入れ子と器だけの repo）は今の読みのまま変えない。持たない木では、
//! root の祖先のうち行 `[workspace]` を持つ `Cargo.toml` の在る最も近い dir を workspace の根と読み、その members のうち
//! root の下の項目を root からの相対で member にする。workspace の根の `Cargo.toml` の members は複数行の配列も読む。

use crate::check::{read_text, Layout};
use std::path::{Path, PathBuf};

/// workspace の manifest の名。
const MANIFEST: &str = "Cargo.toml";

impl Layout {
    /// root が自分の `Cargo.toml` を持たず、祖先の workspace の根の members が root の下の項目を持つ木（根の workspace の
    /// member になった形）か。manifest を持たない一時の木（measure の歯の fixture）は、祖先の members に root の下が無いので偽。
    pub(crate) fn joined(&self) -> bool {
        !self.root.join(MANIFEST).is_file() && members(&self.root).is_ok()
    }

    /// workspace の根の dir（根に入った形の木〔[`Layout::joined`]〕は [`joined_workspace`] の dir・ほかは root・読めなければ root）。
    pub(crate) fn workspace_dir(&self) -> PathBuf {
        if !self.joined() {
            return self.root.clone();
        }
        joined_workspace(&self.root).map_or_else(|_| self.root.clone(), |(dir, _)| dir)
    }
}

/// root の祖先のうち、行 `[workspace]` を持つ `Cargo.toml` の在る最も近い dir（正規化した path）と、その manifest の本文。
pub(crate) fn joined_workspace(root: &Path) -> Result<(PathBuf, String), String> {
    let canonical = root
        .canonicalize()
        .map_err(|err| format!("{} を正規化できない: {err}", root.display()))?;
    for dir in canonical.ancestors().skip(1) {
        let path = dir.join(MANIFEST);
        if !path.is_file() {
            continue;
        }
        let text = read_text(&path)?;
        if text.lines().any(|line| line.trim() == "[workspace]") {
            return Ok((dir.to_path_buf(), text));
        }
    }
    Err(format!("{} の祖先に行 [workspace] の {MANIFEST} が無い", root.display()))
}

/// 自分の `Cargo.toml` を持たない root の member（root からの相対・workspace の根の members の順）。root の下の項目が
/// 1 つも無い木は `Err`（0 件の member に化けさせない）。
pub(crate) fn members(root: &Path) -> Result<Vec<String>, String> {
    let (dir, manifest) = joined_workspace(root)?;
    let canonical = root
        .canonicalize()
        .map_err(|err| format!("{} を正規化できない: {err}", root.display()))?;
    let rel = canonical
        .strip_prefix(&dir)
        .map_err(|err| format!("{} が {} の下に無い: {err}", canonical.display(), dir.display()))?;
    let head = format!("{}/", rel.to_string_lossy());
    let found: Vec<String> = block_members(&manifest)
        .iter()
        .filter_map(|member| member.strip_prefix(&head))
        .map(str::to_owned)
        .collect();
    if found.is_empty() {
        return Err(format!("{} の workspace の members に {head} の下の項目が無い", dir.display()));
    }
    Ok(found)
}

/// 節 `[workspace]` の key `members` の配列の項目（1 行の配列も複数行の配列も読む・項目は二重引用符の字・順のまま）。
pub(crate) fn block_members(manifest: &str) -> Vec<String> {
    let mut in_workspace = false;
    let mut collected = String::new();
    let mut open = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if open {
            collected.push_str(trimmed);
            open = !trimmed.contains(']');
            continue;
        }
        if trimmed.starts_with('[') && !trimmed.starts_with("[\"") {
            in_workspace = trimmed == "[workspace]";
            continue;
        }
        let value = trimmed
            .strip_prefix("members")
            .map(str::trim_start)
            .and_then(|rest| rest.strip_prefix('='));
        if let (true, Some(value)) = (in_workspace, value) {
            collected.push_str(value.trim());
            open = !value.contains(']');
        }
    }
    collected.split('"').skip(1).step_by(2).map(str::to_owned).collect()
}

#[cfg(test)]
mod tests {
    use super::{block_members, members};
    use crate::check::Layout;
    use std::fs;
    use std::path::PathBuf;

    /// 一時の dir に (root 相対 path, 本文) の列を書く。
    fn tree(tag: &str, files: &[(&str, &str)]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("xtask-joined-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for (rel, text) in files {
            let path = root.join(rel);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|err| panic!("{}: {err}", parent.display()));
            }
            fs::write(&path, text).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        }
        root
    }

    /// 根の workspace の member になった器の木（器の dir は nest/・根の members は複数行・器の外の member を 1 つ持つ）。
    const JOINED: &[(&str, &str)] = &[
        ("Cargo.toml", "[workspace]\nresolver = \"3\"\nmembers = [\n    \"crates/app\",\n    \"nest/crates/demo\",\n    \"nest/crates/demo-boundary\",\n    \"nest/crates/demo-xtask\",\n]\n\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n"),
        ("crates/app/Cargo.toml", "[package]\nname = \"app\"\n"),
        ("nest/crates/demo/Cargo.toml", "[package]\nname = \"demo\"\n"),
        ("nest/crates/demo/src/name.rs", "pub const NAME: &str = \"demo\";\n"),
        ("nest/crates/demo-boundary/Cargo.toml", "[package]\nname = \"demo-boundary\"\n"),
        ("nest/crates/demo-xtask/Cargo.toml", "[package]\nname = \"scribe2-xtask\"\n"),
    ];

    #[test]
    fn vjxt_joined_root_reads_the_members_under_it_from_the_workspace_root() {
        let root = tree("members", JOINED);
        let nest = root.join("nest");
        assert_eq!(
            members(&nest),
            Ok(vec![
                "crates/demo".to_owned(),
                "crates/demo-boundary".to_owned(),
                "crates/demo-xtask".to_owned()
            ])
        );
        let layout = Layout::discover(&nest).unwrap_or_else(|reason| panic!("根に入った形の Layout: {reason}"));
        assert!(layout.joined(), "自分の Cargo.toml の無い root は根に入った形");
        assert_eq!(
            layout.workspace_dir(),
            root.canonicalize().unwrap_or_default(),
            "workspace の根は祖先の dir"
        );
        assert_eq!(layout.member_dirs.len(), 3, "member は器の dir の下の 3 つ");
        assert_eq!(layout.core_dir, nest.join("crates/demo"), "core は name.rs を持つ member");
        assert_eq!(layout.name, "demo");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn vjxt_own_manifest_keeps_the_nested_reading() {
        let mut files = JOINED.to_vec();
        files.push((
            "nest/Cargo.toml",
            "[workspace]\nmembers = [\"crates/demo\", \"crates/demo-boundary\"]\n",
        ));
        let root = tree("own", &files);
        let nest = root.join("nest");
        let layout = Layout::discover(&nest).unwrap_or_else(|reason| panic!("入れ子の Layout: {reason}"));
        assert!(!layout.joined(), "自分の Cargo.toml を持つ root は入れ子の形");
        assert_eq!(layout.workspace_dir(), nest, "workspace の根は root");
        assert_eq!(
            layout.member_dirs,
            vec![nest.join("crates/demo"), nest.join("crates/demo-boundary")],
            "自分の members だけを読む"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn vjxt_joined_root_refuses_a_tree_without_its_members() {
        // 否定の見本: 根の members から器の dir の下の 3 項目だけを外した木は、0 件の member に化けずに断る。
        let mut files = JOINED.to_vec();
        files[0] = ("Cargo.toml", "[workspace]\nmembers = [\n    \"crates/app\",\n]\n");
        let root = tree("none", &files);
        let refused = members(&root.join("nest")).expect_err("器の下の項目が無い木");
        assert!(refused.contains("nest/"), "{refused}");
        let _ = fs::remove_dir_all(&root);
    }

    /// 器の lint の表（REQUIRED_LINTS）の clippy の lint を deny（panic だけは引数の level）で持ち、表の外の lint を 1 つ足した
    /// 根の manifest の lint の節（skip の lint は書かない）。
    fn floor_table(skip: Option<&str>, panic_level: &str) -> String {
        let mut table = String::from(
            "[workspace.lints.rust]\nunsafe_code = \"forbid\"\nunused_must_use = \"deny\"\n\n[workspace.lints.clippy]\nallow_attributes_without_reason = \"deny\"\n",
        );
        for (_, lint, _) in crate::limits::REQUIRED_LINTS.iter().filter(|(section, _, _)| *section == "clippy") {
            if Some(*lint) == skip {
                continue;
            }
            let level = if *lint == "panic" { panic_level } else { "deny" };
            table.push_str(&format!("{lint} = \"{level}\"\n"));
        }
        table
    }

    /// 根に入った形の木の lints-set: 器の表の各 lint が根の表に deny か forbid で在れば違反 0（表の外の lint を許す）。
    /// 1 つを warn に下げた表と 1 つを欠いた表は、その lint を名指す 1 件ずつの違反。
    #[test]
    fn vjxt_joined_lints_floor_accepts_deny_and_names_a_weaker_or_missing_lint() {
        for (tag, table, want) in [
            ("floor-ok", floor_table(None, "deny"), Vec::<&str>::new()),
            ("floor-warn", floor_table(None, "warn"), vec!["clippy.panic"]),
            ("floor-miss", floor_table(Some("exit"), "deny"), vec!["clippy.exit"]),
        ] {
            let mut files = JOINED.to_vec();
            let manifest = format!("[workspace]\nmembers = [\n    \"nest/crates/demo\",\n    \"nest/crates/demo-boundary\",\n]\n\n{table}");
            files[0] = ("Cargo.toml", &manifest);
            let root = tree(tag, &files);
            let layout = Layout::discover(&root.join("nest")).unwrap_or_else(|reason| panic!("{tag}: {reason}"));
            let set = crate::check_facts::measure_lints(&layout)
                .into_iter()
                .next()
                .unwrap_or_else(|| panic!("{tag}: lints-set"));
            let named: Vec<&str> = want
                .iter()
                .copied()
                .filter(|lint| set.violations.iter().any(|line| line.contains(lint)))
                .collect();
            assert_eq!(set.violations.len(), want.len(), "{tag}: {:?}", set.violations);
            assert_eq!(named, want, "{tag}: {:?}", set.violations);
            assert!(set.fact.starts_with("lints-set="), "{tag}: {}", set.fact);
            let _ = fs::remove_dir_all(&root);
        }
    }

    /// 根に入った形の木の deps-empty は器の member の manifest だけを数え、根の manifest の [workspace.dependencies] を数えない。
    /// 同じ節を自分の manifest に持つ入れ子の木は、その節を違反に名指す（自分の manifest は母集団のまま）。
    #[test]
    fn vjxt_joined_deps_count_only_the_member_manifests() {
        let mut files = JOINED.to_vec();
        let manifest = "[workspace]\nmembers = [\n    \"nest/crates/demo\",\n    \"nest/crates/demo-boundary\",\n]\n\n[workspace.dependencies]\nserde = \"1\"\n";
        files[0] = ("Cargo.toml", manifest);
        let root = tree("deps", &files);
        let layout = Layout::discover(&root.join("nest")).unwrap_or_else(|reason| panic!("{reason}"));
        let joined = crate::check_facts::measure_deps_empty(&layout);
        assert_eq!(joined.violations, Vec::<String>::new());
        assert_eq!(joined.fact, "deps-empty=2");
        files.push((
            "nest/Cargo.toml",
            "[workspace]\nmembers = [\"crates/demo\", \"crates/demo-boundary\"]\n\n[workspace.dependencies]\nserde = \"1\"\n",
        ));
        let own = tree("deps-own", &files);
        let layout = Layout::discover(&own.join("nest")).unwrap_or_else(|reason| panic!("{reason}"));
        let nested = crate::check_facts::measure_deps_empty(&layout);
        assert_eq!(nested.fact, "deps-empty=3");
        assert_eq!(nested.violations.len(), 1, "{:?}", nested.violations);
        assert!(
            nested.violations.iter().all(|line| line.contains("serde")),
            "{:?}",
            nested.violations
        );
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&own);
    }

    /// 根に入った形の木の nextest-tmux-group は workspace の根の .config/nextest.toml を読み、filter は境界 crate の package を
    /// 名指す形 package(<NAME>-boundary) & test(/^(…)$/) を固定形とする。package の句の無い filter は固定形でないと名指す。
    #[test]
    fn vjxt_joined_tmux_group_reads_the_root_config_with_the_package_head() {
        let rules = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules/manifest.toml");
        let text = fs::read_to_string(&rules).unwrap_or_else(|err| panic!("{}: {err}", rules.display()));
        let limits = crate::limits::Limits::read(&text).unwrap_or_else(|reason| panic!("{reason}"));
        let threads = limits.tmux_test_threads;
        let seat = "#[test]\nfn seat_x() {\n    let _guard = start_seat(\"x\");\n}\n";
        for (tag, filter, ok) in [
            ("tmux-pkg", "package(demo-boundary) & test(/^(seat::seat_x)$/)", true),
            ("tmux-bare", "test(/^(seat::seat_x)$/)", false),
        ] {
            let config = format!("[profile.default]\nslow-timeout = {{ period = \"60s\", terminate-after = 5 }}\n\n[test-groups.tmux]\nmax-threads = {threads}\n\n[[profile.default.overrides]]\nfilter = '{filter}'\ntest-group = 'tmux'\n");
            let mut files = JOINED.to_vec();
            files.push((".config/nextest.toml", &config));
            files.push(("nest/crates/demo-boundary/tests/e2e/seat.rs", seat));
            let root = tree(tag, &files);
            let layout = Layout::discover(&root.join("nest")).unwrap_or_else(|reason| panic!("{tag}: {reason}"));
            let got = crate::check_facts::measure_nextest_tmux_group(&layout, &limits);
            if ok {
                assert_eq!(got.violations, Vec::<String>::new(), "{tag}");
                assert_eq!(got.fact, "nextest-tmux-group=ok tests=1 files=1", "{tag}");
            } else {
                assert_eq!(got.violations.len(), 1, "{tag}: {:?}", got.violations);
                assert!(
                    got.violations.iter().all(|line| line.contains("固定形")),
                    "{tag}: {:?}",
                    got.violations
                );
            }
            let _ = fs::remove_dir_all(&root);
        }
    }

    #[test]
    fn vjxt_block_members_reads_one_line_and_many_lines_inside_workspace_only() {
        assert_eq!(block_members("[workspace]\nmembers = [\"a\", \"b\"]\n"), ["a", "b"]);
        assert_eq!(
            block_members("[workspace]\nmembers = [\n  \"a\",\n  \"b\",\n]\nresolver = \"3\"\n"),
            ["a", "b"]
        );
        // 否定の見本: 別の節の members は読まない。
        assert!(block_members("[workspace]\nresolver = \"3\"\n[package]\nmembers = [\"x\"]\n").is_empty());
        // 否定の見本: 配列の後の key の字は項目に混ぜない。
        assert_eq!(block_members("[workspace]\nmembers = [\n  \"a\",\n]\nexclude = [\"z\"]\n"), ["a"]);
    }
}
