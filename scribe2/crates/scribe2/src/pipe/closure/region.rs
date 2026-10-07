//! 歯の区間の述語（設計 docs/design/contract-source.md §28・§62）と crate の配置の型。
//!
//! 歯の区間は検証行の crate と [`Scope`] が決める。`--test <名>` は crate の manifest の `[[test]]` の `path`
//! （[`crate::pipe::cargo_toml::test_paths`]）が名の対を持てばその file（`main.rs` ならその dir の下の全部）で、持たなければ
//! `tests/<名>.rs` とその下の `tests/<名>/` である。区間を決める述語は [`in_region`] の 1 本で、[`super::derive`] の
//! 置き場の導出（[`super::teeth_places`]・[`super::tooth_sites`]）と preflight の在りかの照らし（[`in_line_region`]）が通る。

use super::derive::{in_crate, in_scope, nextest_read, Base, Scope};
use super::Source;
use crate::pipe::cargo_toml::test_paths;
use crate::pipe::declaration::fixed_roots;

/// `[[test]]` の `path` が指す file の名で、これなら dir の下の全部が区間に入る。
const TEST_MAIN: &str = "main.rs";

/// crate の配置: crate の根の列（固定の根 + 宣言した根・§62 の 1 関数 `crate_of` が path を割る）と、読み込んだ crate の
/// manifest（`<根><crate の名>/Cargo.toml`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateLayout {
    /// crate の根の列。
    pub roots: Vec<String>,
    /// 読み込んだ `Cargo.toml`（manifest を読まない周は空）。
    pub manifests: Vec<Source>,
}

impl CrateLayout {
    /// crate の manifest の path が根と crate の名の後ろに繋ぐ字（manifest を読む `.rs` と同じ読み手の拡張子の位置）。
    pub const MANIFEST_TAIL: &'static str = "/Cargo.toml";

    /// 根の列 `roots` だけの配置（manifest は空）。
    pub fn bare(roots: Vec<String>) -> Self {
        Self { roots, manifests: Vec::new() }
    }

    /// 固定の根だけの配置（manifest は空）。
    pub fn fixed() -> Self {
        Self::bare(fixed_roots())
    }
}

/// `path` が検証行の crate `krate` と scope の歯の区間の中か。`--test <名>` で manifest が `[[test]]` の対を持てば対の path の
/// file（`main.rs` ならその dir の下の全部）で、crate の根の外の別の crate に在っても入る。他は [`in_crate`] と [`in_scope`]。
pub(super) fn in_region(layout: &CrateLayout, path: &str, krate: &str, scope: Scope<'_>) -> bool {
    if let Scope::Test(name) = scope {
        let places = manifest_places(layout, krate, name);
        if !places.is_empty() {
            return places.iter().any(|place| match place.strip_suffix(TEST_MAIN) {
                Some(dir) if dir.is_empty() || dir.ends_with('/') => path.starts_with(dir),
                _ => path == place,
            });
        }
    }
    in_crate(&layout.roots, path, krate) && in_scope(&layout.roots, path, krate, scope)
}

/// 検証行 `line` の歯の区間に `path` が入るか（[`nextest_read`] が読んだ crate と scope を [`in_region`] へ渡す・nextest の形でない行は
/// 偽）。
pub(crate) fn in_line_region(line: &str, path: &str, base: &Base<'_>) -> bool {
    nextest_read(line, base.core_crate).is_some_and(|(krate, _, scope, _)| in_region(base.layout, path, krate, scope))
}

/// crate `krate` の manifest が `[[test]]` の名 `name` に持つ path を、repo の根からの path へ正規化した列（manifest が無い・
/// 本文が読めない・対を持たない・`..` が repo の根の外へ出る path は含めない）。
fn manifest_places(layout: &CrateLayout, krate: &str, name: &str) -> Vec<String> {
    let mut places = Vec::new();
    for root in &layout.roots {
        let dir = format!("{root}{krate}");
        let manifest = format!("{dir}{}", CrateLayout::MANIFEST_TAIL);
        let Some(Ok(text)) = layout.manifests.iter().find(|source| source.path == manifest).map(|source| source.body.as_ref()) else {
            continue;
        };
        let pairs = test_paths(text);
        places.extend(pairs.iter().filter(|(found, _)| found == name).filter_map(|(_, path)| joined(&dir, path)));
    }
    places
}

/// dir `dir` から見た相対 path `path` を repo の根からの path にする（`.` の段を落とし `..` の段で 1 つ上る・根の外へ出るなら `None`）。
fn joined(dir: &str, path: &str) -> Option<String> {
    let mut steps: Vec<&str> = Vec::new();
    for step in dir.split('/').chain(path.split('/')) {
        match step {
            "" | "." => {}
            ".." => {
                steps.pop()?;
            }
            other => steps.push(other),
        }
    }
    Some(steps.join("/"))
}

#[cfg(test)]
mod tests {
    use super::super::derive::{teeth_places, tooth_sites, Base, Fields};
    use super::super::{texts_of, ClosureError, Source};
    use super::CrateLayout;
    use std::collections::BTreeSet;

    /// 検証行（crate b の `--test tz4` の歯 `fsch_`）。
    const LINE: &str = "cargo nextest run -p b --test tz4 --no-tests=fail fsch_";

    /// 読めた `.rs` か manifest 1 本。
    fn source(path: &str, body: &str) -> Source {
        Source { path: path.to_owned(), body: Ok(body.to_owned()) }
    }

    /// 根 `crates/` と `folio2/crates/` の配置（`manifests` を持つか持たないか）。
    fn layout(manifests: Vec<Source>) -> CrateLayout {
        let roots = vec!["crates/".to_owned(), "folio2/crates/".to_owned()];
        CrateLayout { manifests, ..CrateLayout::bare(roots) }
    }

    /// crate b の manifest（`[[test]]` の name tz4 と `path`）。
    fn manifest(path: &str) -> Vec<Source> {
        let body = format!("[package]\nname = \"b\"\n\n[[test]]\nname = \"tz4\"\npath = \"{path}\"\n");
        vec![source(&format!("crates/b{}", CrateLayout::MANIFEST_TAIL), &body)]
    }

    /// `line` の歯の置き場（`sources` を base の `.rs` に・歯の名の検索語は行の filter 語）。
    fn places(layout: &CrateLayout, sources: &[Source], line: &str) -> Result<BTreeSet<String>, ClosureError> {
        let base = Base { sources, snapshots: &[], tracked: &[], core_crate: "b", layout };
        let verify = [line.to_owned()];
        let fields = Fields { touches: &[], surfaces: &[], verify: &verify, creates: &[], tests: &[], also: &[], files: &[] };
        let texts = texts_of(sources)?;
        teeth_places(&fields, &base, &texts)
    }

    /// 歯 `fsch_one` を持つ crate f の tz4 の群（`main.rs` は `mod resolve;` だけ）。
    fn group() -> Vec<Source> {
        vec![
            source("folio2/crates/f/tests/tz4/main.rs", "mod resolve;\n"),
            source("folio2/crates/f/tests/tz4/resolve.rs", "#[test]\nfn fsch_one() {}\n"),
        ]
    }

    /// 歯 vtpath_region_follows_the_manifest_test_path: manifest の `[[test]]` の path が別 crate の dir の `main.rs` を指すと、`--test tz4` の
    /// 区間はその dir の下で、`teeth_places` は `resolve.rs` の 1 file・`tooth_sites` の歯 `fsch_one` の在りかもその file。
    #[test]
    fn vtpath_region_follows_the_manifest_test_path() {
        let layout = layout(manifest("../../folio2/crates/f/tests/tz4/main.rs"));
        let sources = group();
        let want: BTreeSet<String> = ["folio2/crates/f/tests/tz4/resolve.rs".to_owned()].into();
        assert_eq!(places(&layout, &sources, LINE), Ok(want), "区間は manifest の path の dir の下");
        let base = Base { sources: &sources, snapshots: &[], tracked: &[], core_crate: "b", layout: &layout };
        let sites: Vec<String> = tooth_sites(LINE, "fsch_one", &base).into_iter().map(|(path, _)| path).collect();
        assert_eq!(sites, ["folio2/crates/f/tests/tz4/resolve.rs"], "歯の在りかもその file");
    }

    /// 歯 vtpath_region_without_a_manifest_keeps_the_crate_rule: manifest の空の配置では crate の規則のまま（crate b の `tests/tz4` の下に
    /// file が無いので置き場は解けない）。
    #[test]
    fn vtpath_region_without_a_manifest_keeps_the_crate_rule() {
        let layout = layout(Vec::new());
        let found = places(&layout, &group(), LINE);
        assert_eq!(found, Err(ClosureError::TeethPlaceUnresolved { filter: "fsch_".to_owned() }));
    }

    /// 歯 vtpath_region_of_a_non_main_path_is_the_file: path の file 名が `main.rs` でないと区間はその file だけ（同じ dir の別の file は入らない）。
    #[test]
    fn vtpath_region_of_a_non_main_path_is_the_file() {
        let layout = layout(manifest("../../folio2/crates/f/tests/one.rs"));
        let sources = vec![
            source("folio2/crates/f/tests/one.rs", "#[test]\nfn fsch_one() {}\n"),
            source("folio2/crates/f/tests/two.rs", "#[test]\nfn fsch_two() {}\n"),
        ];
        let want: BTreeSet<String> = ["folio2/crates/f/tests/one.rs".to_owned()].into();
        assert_eq!(places(&layout, &sources, LINE), Ok(want), "区間は path の file だけ");
    }
}
