// 再 build の引き金にする path の列挙（設計 consumer-sync.md §18・契約表の行 i・`s2-07l.317`・tsuzuri の行 v-buildrs）。
//
// `crates/<NAME>/build.rs` と e2e の歯（`crates/<NAME>-boundary/tests/e2e/seat.rs` の `build_rerun_*`）が
// 同じこの file を `include!` で読む＝列挙の実装は 1 か所である。`include!` は item を読み手の module へ
// そのまま貼るので、inner doc（`//!`）と `use` を持たず、型と関数は full path で書く（読み手の `use` と
// 衝突させない）。依存は std だけ（build script は crate の依存を持たない）。

/// 器の入力の pathspec（器の根＝`<dir>/../..` から撃つ）。器の根の下の全部と、repo の根の組みの設定（manifest・錠・
/// toolchain・cargo の設定）。器の根が repo の根の木では repo の追跡の全 file と同じ列になる（tsuzuri の判断の記録
/// ADR-36 の決定 (8)・行 v-buildrs）。
const RERUN_PATHSPEC: &[&str] =
    &[".", ":(top)Cargo.toml", ":(top)Cargo.lock", ":(top)rust-toolchain.toml", ":(top)rust-toolchain", ":(top).cargo"];

/// `git -C <at> <args>` を撃ち、rc 0 なら stdout を返す。git が無い・rc ≠ 0 は `None`。
fn rerun_git(at: &std::path::Path, args: &[&str]) -> Option<Vec<u8>> {
    let out = std::process::Command::new("git").arg("-C").arg(at).args(args).output().ok()?;
    out.status.success().then_some(out.stdout)
}

/// 再 build の引き金にする path の列（在る file だけ・sort 済みで重複なし＝決定的な並び）。
///
/// 母集団は **器の入力の追跡 file** だけ: 器の根（`<dir>/../..`）で [`RERUN_PATHSPEC`] を `git ls-files -z --full-name`
/// に渡し、repo の根（`rev-parse --show-toplevel`）に繋いだ絶対 path。unstaged の変更も file の時刻が動くので
/// build script が再走し、`+dirty` が付く（§18 の出所）。untracked は列挙しない（`+dirty` は `--untracked-files=no`
/// で測る＝母集団を揃える）。git の meta（HEAD・index）は出さない: 出すと器の外の commit や build script の
/// `git status` が index を書き直すたびに crate を組み直す（tsuzuri の行 v-buildrs の測りで、器の全部の target が
/// 1 回 約 5 秒・書き 約 1.1 GB）。そのため器の外の commit の後と、器の直しを commit した後に器の入力を直さずに組む時は
/// 再走せず、build の値（build.rs の `build_commit`）は最後に再走した時の値のまま残る。
/// - 消えた tracked は出さない。存在しない path を `rerun-if-changed` に出すと cargo は落ちないが「file が無い＝
///   常に stale」と読んで **毎回** build script と crate を作り直す。
///
/// git が無い・repo でない・rc ≠ 0 の周は 0 本を返す＝build は落とさない（panic しない）。
fn rerun_paths(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let line = |bytes: Vec<u8>| String::from_utf8(bytes).ok().map(|text| std::path::PathBuf::from(text.trim()));
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    if let Some(top) = rerun_git(dir, &["rev-parse", "--show-toplevel"]).and_then(line) {
        let vessel = dir.join("..").join("..");
        let listed = rerun_git(&vessel, &[&["ls-files", "-z", "--full-name", "--"][..], RERUN_PATHSPEC].concat())
            .unwrap_or_default();
        for name in listed.split(|byte| *byte == 0).filter(|name| !name.is_empty()) {
            let name: &std::ffi::OsStr = std::os::unix::ffi::OsStrExt::from_bytes(name);
            paths.push(top.join(name));
        }
    }
    paths.retain(|path| path.is_file());
    paths.sort();
    paths.dedup();
    paths
}
