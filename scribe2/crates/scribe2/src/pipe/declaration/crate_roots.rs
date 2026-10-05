//! 入れ子の source の根（vessel 宣言の任意 key `crate-roots`・設計 docs/design/contract-source.md §62・SRS FR34 / FR48 /
//! NFR4）。
//!
//! 固定の根 `crates/` は常に在り、key はそれに**足す**根を並べる（置き換えない＝宣言で器自身の検査の範囲を狭められない）。
//! 各項目は repo 相対の dir で末尾が `/`。根の列と path から「根・crate の名・残り」を返す 1 関数（[`crate_of`]）が、
//! 検出線の面・受付の上限の余地の読み手の共通の入口である。読むのは HEAD の宣言だけ（[`RootsAtHead`]・作業ツリーは
//! 読まない）。
//!
//! 任意 key `scope-paths` は、追随の再 gate と着地の後の検出線と差の当たりの面（`land::scope_touched` の 1 本）に足す repo 相対の
//! path の接頭辞を並べる（末尾 `/` は dir・ほかは file・面の字は器の code に書かず宣言が持つ）。同じ HEAD の口で根と一緒に読む。

use super::{head_declaration, list_of, DeclError, Raw};
use crate::pipe::land::{detection_needed, in_face};
use std::path::Path;

/// 任意 key の名。
pub const KEY: &str = "crate-roots";

/// 面に足す path の任意 key の名。
pub const SCOPE_KEY: &str = "scope-paths";

/// 固定の根（宣言に依らず常に在る）。
pub const FIXED_ROOT: &str = "crates/";

/// 根の列（固定の根だけ）。
pub fn fixed_roots() -> Vec<String> {
    vec![FIXED_ROOT.to_owned()]
}

/// 固定の根に宣言した根を足した列（固定の根が先頭）。
pub fn with_fixed(declared: &[String]) -> Vec<String> {
    let mut roots = fixed_roots();
    roots.extend(declared.iter().cloned());
    roots
}

/// crate の中の path（根・crate の名・crate の中の残り）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrateFile<'a> {
    /// 根（末尾 `/` を含む）。
    pub root: &'a str,
    /// crate の名（根の直後の 1 段・空でなく `/` を持たない）。
    pub name: &'a str,
    /// crate の中の残り（空でない）。
    pub rest: &'a str,
}

/// repo 相対の path が根のどれかの crate の中なら、根・crate の名・残りを返す（どの根の crate にも入らなければ `None`）。
pub fn crate_of<'a>(roots: &[String], path: &'a str) -> Option<CrateFile<'a>> {
    let root = roots.iter().find(|root| path.starts_with(root.as_str()))?;
    let (root, below) = path.split_at_checked(root.len())?;
    let (name, rest) = below.split_once('/')?;
    (!name.is_empty() && !rest.is_empty()).then_some(CrateFile { root, name, rest })
}

/// 宣言が足す物（2 key の値・無い key は空）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Added {
    /// 足す根（`crate-roots`・固定の根は含まない）。
    pub roots: Vec<String>,
    /// 面に足す path（`scope-paths`・書いた順・固定の面の中の path は持たない）。
    pub paths: Vec<String>,
}

/// HEAD の宣言の根と面の足し（閉じた 3 値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootsAtHead {
    /// 2 key のどちらかが足した物。
    Declared(Added),
    /// 固定の根と固定の面だけ（宣言 file が HEAD に無い・git を撃てない・2 key が無い）。
    Fixed,
    /// 読めない（宣言が在って不備）。「固定の根だけ」に倒さない（C10）。
    Unreadable,
}

impl RootsAtHead {
    /// HEAD の宣言から根を読む（読み手は [`head_declaration`] の 1 本・作業ツリーは読まない）。
    pub fn read(repo: &Path) -> Self {
        match head_declaration(repo) {
            None => Self::Fixed,
            Some(Err(_)) => Self::Unreadable,
            Some(Ok(declared)) if declared.added == Added::default() => Self::Fixed,
            Some(Ok(declared)) => Self::Declared(declared.added),
        }
    }
}

/// 宣言の本文から 2 key を読む（無い key は空・項目の不備は key と行番号を名指して積む）。
pub(super) fn declared_of(found: &[(String, Raw, u64)], errors: &mut Vec<DeclError>) -> Added {
    Added { roots: roots_of(found, errors), paths: paths_of(found, errors) }
}

/// 根の key を読む（無ければ空）。
fn roots_of(found: &[(String, Raw, u64)], errors: &mut Vec<DeclError>) -> Vec<String> {
    let (items, line) = list_of(found, KEY, errors);
    for (at, item) in items.iter().enumerate() {
        if let Some(why) = item_defect(item) {
            errors.push(DeclError::new(line, format!("{KEY} の項目 {item:?} は{why}")));
        }
        for other in items.iter().skip(at.saturating_add(1)).filter(|other| overlaps(item, other)) {
            errors.push(DeclError::new(line, format!("{KEY} の項目 {item:?} と {other:?} は重なる（1 つの path が 2 つの根に入る）")));
        }
    }
    items
}

/// 項目 1 つの不備（無ければ `None`）。
fn item_defect(item: &str) -> Option<&'static str> {
    if item.trim().is_empty() {
        Some("空である")
    } else if !item.ends_with('/') {
        Some("末尾が / でない")
    } else if item.starts_with('/') {
        Some("絶対 path である")
    } else if item.contains('~') {
        Some("home の短縮記号を持つ")
    } else if item.split('/').any(|part| part == "..") {
        Some(".. の段を持つ")
    } else if overlaps(item, FIXED_ROOT) {
        Some("固定の根 crates/ と重なる")
    } else {
        None
    }
}

/// 同じ項目か一方が他方の接頭辞になるか（末尾 `/` の項目どうしなので path の段で見る）。
fn overlaps(left: &str, right: &str) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

/// 面の path の key を読む（無ければ空）。面に何も足さない項目（git の path に当たらない字・固定の面の中・ほかの項目と重なる）は
/// key と行番号を名指す不備。照らしの dir と file の分けは面の照らしと同じ `in_face`。
fn paths_of(found: &[(String, Raw, u64)], errors: &mut Vec<DeclError>) -> Vec<String> {
    let (items, line) = list_of(found, SCOPE_KEY, errors);
    for (at, item) in items.iter().enumerate() {
        if let Some(why) = path_defect(item) {
            errors.push(DeclError::new(line, format!("{SCOPE_KEY} の項目 {item:?} は{why}")));
        }
        for other in items.iter().skip(at.saturating_add(1)).filter(|other| in_face(item, other) || in_face(other, item)) {
            errors.push(DeclError::new(line, format!("{SCOPE_KEY} の項目 {item:?} と {other:?} は重なる（一方が他方の中で足す物が無い）")));
        }
    }
    items
}

/// 面の path の項目 1 つの不備（無ければ `None`）。
fn path_defect(item: &str) -> Option<&'static str> {
    let parts: Vec<&str> = item.strip_suffix('/').unwrap_or(item).split('/').collect();
    if item.trim().is_empty() {
        Some("空である")
    } else if item.trim() != item {
        Some("前後に空白を持つ")
    } else if item.starts_with('/') {
        Some("絶対 path である")
    } else if parts.contains(&"..") {
        Some(".. の段を持つ")
    } else if parts.iter().any(|part| part.is_empty() || *part == ".") {
        Some("空の段か . の段を持つ（git の path に当たらない）")
    } else if detection_needed([item]) {
        Some("固定の面の中である（足す物が無い）")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{crate_of, fixed_roots, with_fixed, Added, CrateFile, RootsAtHead};
    use crate::pipe::declaration::Declared;
    use crate::pipe::land::scope_touched;

    /// 必須 key だけの宣言（3 行）の後ろに `extra` を足す。
    fn with(extra: &str) -> String {
        format!("schema = 1\nallowed-commands = [\"git\"]\ncommon-verify = [\"git diff --quiet\"]\n{extra}")
    }

    /// 宣言だけの 1 commit を持つ使い捨ての git repo（`None` は宣言 file を持たない repo）。
    fn repo_declaring(name: &str, declaration: Option<&str>) -> std::path::PathBuf {
        let repo = crate::pipe::fixture::scratch(name);
        let file = repo.join(crate::pipe::declaration::DECL_FILE);
        let setup: [&[&str]; 4] =
            [&["init", "-q", "-b", "main"], &["config", "user.name", "t"], &["config", "user.email", "t@example.invalid"], &["config", "commit.gpgsign", "false"]];
        for args in setup {
            assert!(crate::pipe::git_ok(&repo, args), "{args:?}");
        }
        assert!(declaration.is_none_or(|text| std::fs::write(&file, text).is_ok()), "宣言を書けた");
        assert!(crate::pipe::git_ok(&repo, &["add", "-A"]), "add");
        assert!(crate::pipe::git_ok(&repo, &["commit", "-q", "--allow-empty", "-m", "seed"]), "commit");
        repo
    }

    /// (a) key が無い宣言は根が空（＝固定の根だけ）、2 項目は 2 つ・足すだけで固定の根は先頭に残る。不備 7 形は key と行番号（4 行目）を名指す。
    #[test]
    fn declaration_crate_roots_reads_the_key_and_names_seven_defects() {
        assert_eq!(Declared::parse(&with("")).map(|found| found.added.roots), Ok(Vec::new()));
        let two = Declared::parse(&with("crate-roots = [\"nest/crates/\", \"apps/\"]\n")).map(|found| found.added.roots);
        assert_eq!(two, Ok(vec!["nest/crates/".to_owned(), "apps/".to_owned()]));
        assert_eq!(with_fixed(&two.unwrap_or_default()), ["crates/", "nest/crates/", "apps/"], "足すだけ（固定の根が先頭）");
        let home = format!("[\"{}\"]", concat!("~", "/x/"));
        let bad = ["[\" \"]", "[\"nest\"]", "[\"/nest/\"]", home.as_str(), "[\"a/../b/\"]", "[\"crates/x/\"]", "[\"nest/\", \"nest/deep/\"]"];
        for value in bad {
            let errors = Declared::parse(&with(&format!("crate-roots = {value}\n"))).expect_err(value);
            assert!(errors.iter().any(|error| error.line == 4 && error.reason.contains("crate-roots")), "{value}: {errors:?}");
        }
        let errors = Declared::parse(&with("crate-roots = [\"nest/\", \"apps/\"]\n")).err();
        assert_eq!(errors, None, "重ならない 2 項目は不備でない");
    }

    /// (b) 1 関数の表: 宣言した根の下は根・crate・残り、宣言しない repo では無し、固定の根は宣言に依らず同じ、根の直下の file と crate の名の無い path は無し。
    #[test]
    fn declaration_crate_roots_function_table() {
        let nested = with_fixed(&["nest/crates/".to_owned()]);
        let file = |root, name, rest| Some(CrateFile { root, name, rest });
        let table = [
            (&nested, "nest/crates/toy/src/a.rs", file("nest/crates/", "toy", "src/a.rs")),
            (&fixed_roots(), "nest/crates/toy/src/a.rs", None),
            (&nested, "crates/toy/src/a.rs", file("crates/", "toy", "src/a.rs")),
            (&fixed_roots(), "crates/toy/src/a.rs", file("crates/", "toy", "src/a.rs")),
            (&nested, "nest/crates/README.md", None),
            (&nested, "nest/crates/toy/", None),
            (&nested, "nest/crates//src/a.rs", None),
            (&nested, "nest/cratesx/toy/src/a.rs", None),
        ];
        for (roots, path, want) in table {
            assert_eq!(crate_of(roots, path), want, "{roots:?} {path}");
        }
    }

    /// (c) 面: 宣言した根の下の path だけの列は真、固定の根だけは偽、既存の面の表は根に依らず同じ。空の列は偽。
    #[test]
    fn declaration_crate_roots_scope_adds_the_declared_roots_to_the_fixed_face() {
        let declared = RootsAtHead::Declared(Added { roots: vec!["nest/crates/".to_owned()], paths: Vec::new() });
        let (nested, notes, fixed) = (["nest/crates/toy/src/a.rs"], ["notes/a.md"], ["crates/toy/src/a.rs", "Cargo.lock", "rules/x.toml"]);
        assert!(scope_touched(&declared, &nested), "宣言した根の下は面");
        assert!(!scope_touched(&RootsAtHead::Fixed, &nested), "宣言しない列は面の外");
        assert!(!scope_touched(&declared, &notes), "面の外は外のまま");
        assert!(!scope_touched(&declared, &[]), "空の列は偽");
        for roots in [RootsAtHead::Fixed, declared] {
            assert!(fixed.iter().all(|path| scope_touched(&roots, &[path])), "{roots:?} 既存の面は同じ");
        }
    }

    /// (e) HEAD の口: 宣言 file の無い repo は固定の根だけ・key を持つ宣言は宣言した根・壊した宣言は読めない。
    /// 読めない周は面の外の path だけの列でも「触れる」を返す。
    #[test]
    fn declaration_crate_roots_head_reads_three_values_and_unreadable_touches() {
        let none = repo_declaring("crate-roots-none", None);
        assert_eq!(RootsAtHead::read(&none), RootsAtHead::Fixed);
        let plain = repo_declaring("crate-roots-plain", Some(&with("")));
        assert_eq!(RootsAtHead::read(&plain), RootsAtHead::Fixed);
        let declared = repo_declaring("crate-roots-declared", Some(&with("crate-roots = [\"nest/crates/\"]\n")));
        assert_eq!(RootsAtHead::read(&declared), RootsAtHead::Declared(Added { roots: vec!["nest/crates/".to_owned()], paths: Vec::new() }));
        let broken = repo_declaring("crate-roots-broken", Some(&with("crate-roots = [\"/abs/\"]\n")));
        let unreadable = RootsAtHead::read(&broken);
        assert_eq!(unreadable, RootsAtHead::Unreadable);
        assert!(scope_touched(&unreadable, &["notes/a.md"]), "読めない周は面の外の列でも撃つ側");
    }

    /// 面の path（`vessel/` は toy の器の dir）の 3 項目を書いた順に持ち、根の key と並べても両方を持ち、2 key の無い宣言は空。
    #[test]
    fn vscope_declaration_reads_the_paths_in_written_order() {
        let three = ["vessel/rules/", "vessel/Cargo.toml", "vessel/Cargo.lock"].map(str::to_owned).to_vec();
        let paths = Declared::parse(&with("scope-paths = [\"vessel/rules/\", \"vessel/Cargo.toml\", \"vessel/Cargo.lock\"]\n"));
        assert_eq!(paths.map(|found| found.added), Ok(Added { roots: Vec::new(), paths: three }));
        let both = Declared::parse(&with("crate-roots = [\"nest/crates/\"]\nscope-paths = [\"vessel/rules/\"]\n")).map(|found| found.added);
        assert_eq!(both, Ok(Added { roots: vec!["nest/crates/".to_owned()], paths: vec!["vessel/rules/".to_owned()] }));
        assert_eq!(Declared::parse(&with("")).map(|found| found.added), Ok(Added::default()));
    }

    /// 面に何も足さない 13 形は、どれも key の名と項目と不備の字を持つ理由で 4 行目を名指す不備。重ならない 2 項目は不備でない。
    #[test]
    fn vscope_declaration_names_the_defects() {
        let bad = [
            ("\"vessel/rules/\"", "scope-paths は配列"),
            ("[]", "scope-paths の 配列が空"),
            ("[\" \"]", "scope-paths の項目 \" \" は空である"),
            ("[\" vessel/rules/\"]", "scope-paths の項目 \" vessel/rules/\" は前後に空白を持つ"),
            ("[\"/vessel/rules/\"]", "scope-paths の項目 \"/vessel/rules/\" は絶対 path である"),
            ("[\"vessel/../rules/\"]", "scope-paths の項目 \"vessel/../rules/\" は.. の段を持つ"),
            ("[\"./\"]", "scope-paths の項目 \"./\" は空の段か . の段を持つ"),
            ("[\"vessel/./rules/\"]", "scope-paths の項目 \"vessel/./rules/\" は空の段か . の段を持つ"),
            ("[\"vessel//rules/\"]", "scope-paths の項目 \"vessel//rules/\" は空の段か . の段を持つ"),
            ("[\"rules/x.toml\"]", "scope-paths の項目 \"rules/x.toml\" は固定の面の中である"),
            ("[\"Cargo.lock\"]", "scope-paths の項目 \"Cargo.lock\" は固定の面の中である"),
            ("[\"vessel/rules/\", \"vessel/rules/a.toml\"]", "scope-paths の項目 \"vessel/rules/\" と \"vessel/rules/a.toml\" は重なる"),
            ("[\"vessel/Cargo.toml\", \"vessel/Cargo.toml\"]", "scope-paths の項目 \"vessel/Cargo.toml\" と \"vessel/Cargo.toml\" は重なる"),
        ];
        for (value, want) in bad {
            let errors = Declared::parse(&with(&format!("scope-paths = {value}\n"))).expect_err(value);
            assert!(errors.iter().any(|error| error.line == 4 && error.reason.contains(want)), "{value}: {errors:?}");
        }
        let errors = Declared::parse(&with("scope-paths = [\"vessel/rules/\", \"vessel/rulesx/\", \"vessel/Cargo.toml\"]\n")).err();
        assert_eq!(errors, None, "重ならない項目は不備でない");
    }

    /// 面の表: 宣言した dir の下と宣言した file そのものは面、字の続く隣・ほかの dir の下・宣言の外は面の外、固定の面は同じ。
    #[test]
    fn vscope_scope_adds_the_declared_paths_to_the_face() {
        let paths = vec!["vessel/rules/".to_owned(), "vessel/Cargo.toml".to_owned()];
        let declared = RootsAtHead::Declared(Added { roots: Vec::new(), paths: paths.clone() });
        let table = [
            ("vessel/rules/manifest.toml", true),
            ("vessel/rules/deep/a.toml", true),
            ("vessel/Cargo.toml", true),
            ("vessel/Cargo.tomlx", false),
            ("vessel/rulesx/a.toml", false),
            ("other/vessel/rules/a.toml", false),
            ("vessel/README.md", false),
        ];
        for (path, want) in table {
            assert_eq!(scope_touched(&declared, &[path]), want, "{path}");
            assert!(!scope_touched(&RootsAtHead::Fixed, &[path]), "宣言しない列は面の外: {path}");
        }
        assert!(!scope_touched(&declared, &[]), "空の列は偽");
        let both = RootsAtHead::Declared(Added { roots: vec!["nest/crates/".to_owned()], paths });
        assert!(scope_touched(&both, &["nest/crates/toy/src/a.rs"]) && scope_touched(&both, &["vessel/rules/a.toml"]), "根と面の path の和");
        assert!(["crates/toy/src/a.rs", "Cargo.lock", "rules/x.toml"].iter().all(|path| scope_touched(&declared, &[path])), "固定の面は同じ");
    }

    /// HEAD の口: 面の path だけの宣言は面の path を持つ足し、不備の面の path を持つ宣言は読めない。
    #[test]
    fn vscope_head_reads_the_paths_and_a_defect_is_unreadable() {
        let declared = repo_declaring("vscope-declared", Some(&with("scope-paths = [\"vessel/rules/\"]\n")));
        let want = Added { roots: Vec::new(), paths: vec!["vessel/rules/".to_owned()] };
        assert_eq!(RootsAtHead::read(&declared), RootsAtHead::Declared(want));
        let broken = repo_declaring("vscope-broken", Some(&with("scope-paths = [\"rules/\"]\n")));
        assert_eq!(RootsAtHead::read(&broken), RootsAtHead::Unreadable);
    }
}
