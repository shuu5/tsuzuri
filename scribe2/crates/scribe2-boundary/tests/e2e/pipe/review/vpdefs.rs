//! `pipe index show --row` の欄 `patch` の項目（設計 docs/design/reverse-index.md §6 の項目 (iii)）。
//!
//! 親（`tests/e2e/pipe/review.rs`）の逆引きの toy（`gx_place`）に、欄 `patch` を持つ行と差の file と、field と注を持つ file kit.rs
//! （SCIP の document を toy の index の bytes の後ろに足す・repeated の document は bytes の連結で足せる）を足して撃つ。

use super::*;

/// toy の差（shape.rs の 2・9 行の置き換え・5 行の後の足し・1 と 12 行の注と属性・test の file・索引の外の toml・新しい file）。
const VPDEFS_PATCH: &str = "diff --git a/crates/toy/src/shape.rs b/crates/toy/src/shape.rs
--- a/crates/toy/src/shape.rs
+++ b/crates/toy/src/shape.rs
@@ -1,13 +1,14 @@
-/// 見る: [`Gadget`] の話
+/// 見る: [`Gadget`] の話（改）
-pub struct Gadget { pub x: u8 }
+pub struct Gadget { pub x: u16 }
 impl Gadget {
     pub fn new() -> Self {
         Self { x: 0 }
+        // 足す
     }
 }
 pub fn make() -> Gadget {
-    Gadget { x: 1 }
+    Gadget { x: 9 }
 }
 fn describe() -> String { format!(\"{Gadget}\") }
-#[cfg(test)]
+#[cfg(all(test))]
 mod tests;
diff --git a/crates/toy/src/kit.rs b/crates/toy/src/kit.rs
--- a/crates/toy/src/kit.rs
+++ b/crates/toy/src/kit.rs
@@ -1,9 +1,9 @@
 pub struct Kit {
-    pub size: u8,
+    pub size: u16,
     pub name: u8,
 }
 pub fn pack(k: Kit) -> u8 {
-    // 注
+    // 注を直す
     let a = k.size;
     a
 }
diff --git a/crates/toy/src/shape/tests.rs b/crates/toy/src/shape/tests.rs
--- a/crates/toy/src/shape/tests.rs
+++ b/crates/toy/src/shape/tests.rs
@@ -1 +1 @@
-fn builds() { let _ = crate::shape::make(); let _ = crate::shape::Gadget { x: 2 }; }
+fn builds() { let _ = crate::shape::make(); let _ = crate::shape::Gadget { x: 3 }; }
diff --git a/config/gadget.toml b/config/gadget.toml
--- a/config/gadget.toml
+++ b/config/gadget.toml
@@ -1,2 +1,2 @@
 [shape]
-kind = \"Gadget\"
+kind = \"Gizmo\"
diff --git a/crates/toy/src/fresh.rs b/crates/toy/src/fresh.rs
new file mode 100644
--- /dev/null
+++ b/crates/toy/src/fresh.rs
@@ -0,0 +1 @@
+pub fn fresh() {}
";

/// kit.rs の path。
const VPDEFS_KIT_RS: &str = "crates/toy/src/kit.rs";

/// kit.rs の本文（struct の field は行ごと・関数の本文の注は occurrence の間）。
const VPDEFS_KIT: &str = "pub struct Kit {\n    pub size: u8,\n    pub name: u8,\n}\npub fn pack(k: Kit) -> u8 {\n    // 注\n    let a = k.size;\n    a\n}\n";

/// kit.rs の SCIP の document（struct・field 2 つ・関数・関数の中の型と field の参照）。
fn vpdefs_kit_document() -> Vec<u8> {
    let kit = "rust-analyzer cargo toy 0.1.0 kit/Kit#";
    let occs = vec![
        gx_occ(VPDEFS_KIT, ("struct Kit", "Kit"), kit, Some("pub struct Kit {\n    pub size: u8,\n    pub name: u8,\n}")),
        gx_occ(VPDEFS_KIT, ("pub size", "size"), "rust-analyzer cargo toy 0.1.0 kit/Kit#size.", Some("pub size: u8")),
        gx_occ(VPDEFS_KIT, ("pub name", "name"), "rust-analyzer cargo toy 0.1.0 kit/Kit#name.", Some("pub name: u8")),
        gx_occ(VPDEFS_KIT, ("fn pack", "pack"), "rust-analyzer cargo toy 0.1.0 kit/pack().", Some("pub fn pack(k: Kit) -> u8 {\n    // 注\n    let a = k.size;\n    a\n}")),
        gx_occ(VPDEFS_KIT, ("k: Kit", "Kit"), kit, None),
        gx_occ(VPDEFS_KIT, ("k.size", "size"), "rust-analyzer cargo toy 0.1.0 kit/Kit#size.", None),
    ];
    scip_document(VPDEFS_KIT_RS, 1, &occs, &[])
}

/// toy に欄 `patch` を持つ行 t2（write-set は shape.rs・差は在る）と t3（差の file が無い）を足して commit する。
fn vpdefs_place() -> IdxPlace {
    let place = gx_place(true);
    let rows = [
        row_fields("t2", &["write-set"], &[r#"write-set = ["crates/toy/src/shape.rs"]"#, r#"patch = "docs/design/patch/p.patch""#]),
        row_fields("t3", &["write-set"], &[r#"write-set = ["crates/toy/src/shape.rs"]"#, r#"patch = "docs/design/patch/none.patch""#]),
    ];
    let toml = place.repo.join("contracts/t.toml");
    let mut body = fs::read_to_string(&toml).unwrap_or_default();
    for row in rows {
        body.push_str(&format!("\n[[contract]]\n{}\n", row.join("\n")));
    }
    assert!(fs::write(&toml, body).is_ok(), "t.toml を書ける");
    assert!(fs::create_dir_all(place.repo.join("docs/design/patch")).is_ok(), "差の dir を作れる");
    assert!(fs::write(place.repo.join("docs/design/patch/p.patch"), VPDEFS_PATCH).is_ok(), "差の file を書ける");
    assert!(fs::write(place.repo.join(VPDEFS_KIT_RS), VPDEFS_KIT).is_ok(), "kit.rs を書ける");
    let scip = [gx_scip_bytes(), scip_index(&[vpdefs_kit_document()], &[])].concat();
    assert!(fs::write(place.state.join("idx.scip"), scip).is_ok(), "kit.rs を足した SCIP を書ける");
    git(&place.repo, &["add", "-A"]);
    git(&place.repo, &["commit", "-q", "-m", "patch rows"]);
    place
}

/// 差の行は、項目の前に 1 行（定義の数・test の中の定義の数・引けない `-` 行の数・新しい file の数・索引の外の file）を置き、差が替える定義
/// を差の file の順と行の順に項目にする: `-` 行の定義（struct・field）・`-` 行の occurrence を囲む定義（make・field を持つ struct）・
/// occurrence の無い `-` 行を含む定義の広がり（注の行の pack）・`+` 行の塊の前の行を含む定義の広がり（new）。test の中の定義（builds）は
/// 項目にせず数え、どの定義にも引けない注と属性の行は引けない行に数え、toml は索引の外・fresh.rs は新しい file に数える。
#[test]
fn vpdefs_row_items_are_the_definitions_the_patch_changes() {
    let place = vpdefs_place();
    let shown = gx_shown(&place, &["--row", "contracts/t.toml#t2"]);
    assert_eq!(
        shown.lines().next(),
        Some("patch=docs/design/patch/p.patch defs=6 tests=1 unmapped=2 fresh=1 outside-index=1 config/gadget.toml"),
        "項目の前の 1 行: {shown}"
    );
    let items = ["shape::Gadget", "shape::Gadget::new", "shape::make", "kit::Kit::size", "kit::Kit", "kit::pack"];
    assert_eq!(gx_items(&shown), items, "差が替える定義を差の file の順と行の順に: {shown}");
    let make = gx_block(&shown, "shape::make");
    assert!(make.iter().any(|line| line == &format!("  symbol: {GX_MAKE}")), "名でなく symbol で解く: {make:?}");
    assert_eq!(gx_field(&make, "callers"), "2(外2)", "行の write-set の外の呼び手に印: {make:?}");
    assert_eq!(gx_sites(&make, "callers"), [format!("{GX_TESTS_RS}:1 test builds 外"), format!("{GX_USER_RS}:3 assemble 外")], "{make:?}");
    let gadget = gx_block(&shown, "shape::Gadget");
    assert_eq!(gx_field(&gadget, "literals"), "4(外2)", "struct の literal の site と外の印: {gadget:?}");
    clean(&[&place.repo, &place.state]);
}

/// 差の file を ref の木から読めない行は `patch=<path> unreadable` の 1 行だけで項目を足さず、欄 `patch` を持たない行は 1 行を持たない。
#[test]
fn vpdefs_unreadable_patch_and_rows_without_the_field() {
    let place = vpdefs_place();
    let missing = gx_shown(&place, &["--row", "contracts/t.toml#t3"]);
    assert_eq!(missing.trim_end(), "patch=docs/design/patch/none.patch unreadable", "読めない差の 1 行だけ: {missing}");
    let plain = gx_shown(&place, &["--row", "contracts/t.toml#t1"]);
    assert!(!plain.lines().any(|line| line.starts_with("patch=")), "欄の無い行は差の 1 行を持たない: {plain}");
    assert_eq!(gx_items(&plain), ["crate::shape::Gadget"], "欄の無い行の項目は touches のまま: {plain}");
    clean(&[&place.repo, &place.state]);
}
