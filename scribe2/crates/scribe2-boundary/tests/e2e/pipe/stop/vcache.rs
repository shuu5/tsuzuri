//! 起草の置き場の写しの下の組みの置き場の掃除の歯（接頭辞 `vcache_`・設計 dispatcher.md §33 形 8・
//! 親 `tests/e2e/pipe/stop.rs` の helper を `use super::*` で使う）。
//!
//! `.git` を持たない写しの下と起草の木の中で、直下に署名の合う印の file（CACHEDIR.TAG）を持つ dir だけが、書きの線より前なら
//! 終端の周に消え、消した dir の数と MiB が stderr の `sweep:` の行に載り、便の木と量の線の候補は替わらないことを外形から測る。

use super::*;

/// 印の file の頭に在るべき署名の字（cargo の target が直下に置く印と同じ）。
const SIGNATURE: &str = "Signature: 8a477f597d28d172789f06886806bc55";

/// 席 `seat` の起草の置き場に `.git` を持たない写し `name` を作る（中に file を 1 本置き、写しの path を返す）。
fn drafts_copy(state: &Path, seat: &str, name: &str) -> PathBuf {
    let copy = state.join("seat").join(seat).join("drafts").join(name);
    put_file(&copy.join("brief.md"), "x\n");
    copy
}

/// 印の dir を置く（直下に頭が `head` の印の file・下に `kib` KiB の file 1 本）。
fn tag_dir(dir: &Path, head: &str, kib: usize) {
    put_file(&dir.join("CACHEDIR.TAG"), &format!("{head}\n# a cache directory tag\n"));
    put_sized(&dir.join("debug").join("app.o"), kib);
}

/// 終端（live な便 `r-end` の `pipe stop`）を埋め込みの manifest で撃つ（rc は変わらない）。
fn end_stop(state: &Path) -> std::process::Output {
    let out = run_pipe(&["stop", "--run", "r-end", "--state-dir", &state.display().to_string()]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "終端の rc は変わらない: {}", stderr_of(&out));
    out
}

/// 写しの下の 7 時間前の印の dir は、写しの直下（`target/`）も深い所（`w/retired/t-old/`）も消え、写しと印の無い file は残り、
/// 行は消した dir の数 2 と、消す前に測った byte の和を MiB に切り上げた 2 を載せる。
#[test]
fn vcache_old_tagged_dirs_under_a_copy_without_git_are_removed() {
    let (repo, state) = repo_with_state();
    drafts_run(&repo, &state);
    let copy = drafts_copy(&state, "s1", "w1");
    let (target, retired) = (copy.join("target"), copy.join("w").join("retired").join("t-old"));
    for dir in [&target, &retired] {
        tag_dir(dir, SIGNATURE, 768);
        rewind(dir, 7);
    }
    put_file(&copy.join("w").join("notes.md"), "x\n");

    let out = end_stop(&state);
    assert!(!target.exists(), "写しの直下の古い印の dir は消える");
    assert!(!retired.exists(), "写しの深い所の古い印の dir も消える");
    for kept in [copy.join("brief.md"), copy.join("w").join("notes.md"), copy.join("w").join("retired")] {
        assert!(kept.exists(), "{} は残る", kept.display());
    }
    let want = "sweep: removed=2 runs=0 failed=0 drafts=0 nogit=1 cache=2 cache_mb=2";
    assert_eq!(sweep_line(&out), want, "stderr: {}", stderr_of(&out));
    clean(&[&repo, &state]);
}

/// 深い所に書いたばかりの file を持つ 7 時間前の印の dir と、書いたばかりの印の dir は残り、7 時間前の印の dir だけが消える。
#[test]
fn vcache_tagged_dirs_with_a_fresh_write_stay() {
    let (repo, state) = repo_with_state();
    drafts_run(&repo, &state);
    let copy = drafts_copy(&state, "s1", "w1");
    let (busy, fresh, old) = (copy.join("target"), copy.join("target-new"), copy.join("target-old"));
    for dir in [&busy, &old] {
        tag_dir(dir, SIGNATURE, 4);
        rewind(dir, 7);
    }
    tag_dir(&fresh, SIGNATURE, 4);
    let deps = busy.join("debug").join("deps");
    put_file(&deps.join("fresh.o"), "x\n");
    age_one(&deps, 7);
    age_one(&busy.join("debug"), 7);

    let out = end_stop(&state);
    assert!(deps.join("fresh.o").is_file(), "深い所に書いたばかりの file を持つ印の dir は残る");
    assert!(fresh.join("CACHEDIR.TAG").is_file(), "書いたばかりの印の dir は残る");
    assert!(!old.exists(), "全部古い印の dir だけが消える");
    assert_eq!(sweep_line(&out), "sweep: removed=1 runs=0 failed=0 drafts=0 nogit=1 cache=1 cache_mb=1", "stderr: {}", stderr_of(&out));
    clean(&[&repo, &state]);
}

/// 印の無い 7 時間前の dir・印の dir の親・印の dir を指す symlink・直下に印を持つ写しそのものは残り、印の dir だけが消える。
#[test]
fn vcache_dirs_without_the_tag_stay() {
    let (repo, state) = repo_with_state();
    drafts_run(&repo, &state);
    let copy = drafts_copy(&state, "s1", "w1");
    let (plain, outer, inner) = (copy.join("plain"), copy.join("outer"), copy.join("outer").join("inner"));
    put_sized(&plain.join("debug").join("app.o"), 4);
    put_file(&outer.join("keep.txt"), "x\n");
    tag_dir(&inner, SIGNATURE, 4);
    let outside = state.join("outside");
    tag_dir(&outside, SIGNATURE, 4);
    std::os::unix::fs::symlink(&outside, copy.join("link")).expect("symlink を置ける");
    let whole = drafts_copy(&state, "s1", "w2");
    tag_dir(&whole, SIGNATURE, 4);
    for dir in [&plain, &outer, &outside, &whole] {
        rewind(dir, 7);
    }
    rewind_to(&copy.join("link"), ago_h(7));

    let out = end_stop(&state);
    assert!(!inner.exists(), "印の dir は消える");
    for kept in [plain.join("debug").join("app.o"), outer.join("keep.txt"), outside.join("CACHEDIR.TAG"), whole.join("CACHEDIR.TAG")] {
        assert!(kept.is_file(), "{} は残る", kept.display());
    }
    assert!(fs::symlink_metadata(copy.join("link")).is_ok(), "印の dir を指す symlink は辿らず残る");
    assert_eq!(sweep_line(&out), "sweep: removed=1 runs=0 failed=0 drafts=0 nogit=2 cache=1 cache_mb=1", "stderr: {}", stderr_of(&out));
    clean(&[&repo, &state]);
}

/// 印の file の頭の字が署名と違う 7 時間前の dir と、署名より短い 7 時間前の dir は残り、署名の合う dir だけが消える。
#[test]
fn vcache_dirs_with_another_signature_stay() {
    let (repo, state) = repo_with_state();
    drafts_run(&repo, &state);
    let copy = drafts_copy(&state, "s1", "w1");
    let (wrong, short, right) = (copy.join("wrong"), copy.join("short"), copy.join("right"));
    tag_dir(&wrong, "Signature: 00000000000000000000000000000000", 4);
    tag_dir(&short, "Signature: 8a477f59", 4);
    tag_dir(&right, SIGNATURE, 4);
    for dir in [&wrong, &short, &right] {
        rewind(dir, 7);
    }

    let out = end_stop(&state);
    assert!(wrong.join("CACHEDIR.TAG").is_file(), "署名の違う印の dir は残る");
    assert!(short.join("CACHEDIR.TAG").is_file(), "署名より短い印の dir は残る");
    assert!(!right.exists(), "署名の合う印の dir だけが消える");
    assert_eq!(sweep_line(&out), "sweep: removed=1 runs=0 failed=0 drafts=0 nogit=1 cache=1 cache_mb=1", "stderr: {}", stderr_of(&out));
    clean(&[&repo, &state]);
}

/// 起草の木の追跡されている印の dir と、木の中の入れ子の clone と写しの中の入れ子の clone の印の dir は 7 時間前でも残り
/// （入れ子の clone の中は印を探さない）、木の中と写しの直下の未追跡の印の dir だけが消える。
#[test]
fn vcache_tracked_files_in_trees_stay() {
    let (repo, state) = repo_with_state();
    tag_dir(&repo.join("cache"), SIGNATURE, 4);
    git(&repo, &["add", "cache"]);
    git(&repo, &["commit", "-q", "-m", "tracked cache"]);
    drafts_run(&repo, &state);
    let tree = draft_tree(&repo, &state, "s1", "t1");
    tag_dir(&tree.join("tgt"), SIGNATURE, 4);
    let copy = drafts_copy(&state, "s1", "w1");
    let (inner, nested) = (tree.join("w").join("clone"), copy.join("clone"));
    for clone in [&inner, &nested] {
        git(&repo, &["clone", "-q", "--local", ".", &clone.display().to_string()]);
    }
    tag_dir(&copy.join("target"), SIGNATURE, 4);
    for dir in [tree.join("cache"), tree.join("tgt"), tree.join("w"), nested.join("cache"), copy.join("target")] {
        rewind(&dir, 7);
    }

    let out = end_stop(&state);
    for kept in [tree.join("cache"), inner.join("cache"), nested.join("cache")] {
        assert!(kept.join("debug").join("app.o").is_file(), "{} は残る", kept.display());
    }
    assert!(!tree.join("tgt").exists(), "木の中の未追跡の印の dir は消える");
    assert!(!copy.join("target").exists(), "写しの直下の印の dir は消える");
    assert_eq!(sweep_line(&out), "sweep: removed=2 runs=0 failed=0 drafts=1 nogit=1 cache=2 cache_mb=1", "stderr: {}", stderr_of(&out));
    clean(&[&repo, &state]);
}

/// 行 seat.drafts_stale_h を持たない tmp manifest の周は、写しだけの置き場でも 7 時間前の印の dir を残し、語 `no-rule` を残す。
#[test]
fn vcache_a_missing_rule_row_keeps_tagged_dirs_and_says_no_rule() {
    let (repo, state) = repo_with_state();
    drafts_run(&repo, &state);
    let copy = drafts_copy(&state, "s1", "w1");
    tag_dir(&copy.join("target"), SIGNATURE, 4);
    rewind(&copy.join("target"), 7);

    let rules = drafts_rules(&state, "");
    let out = run_pipe(&["stop", "--run", "r-end", "--state-dir", &state.display().to_string(), "--rules", &rules]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "終端の rc は変わらない: {}", stderr_of(&out));
    assert!(copy.join("target").join("CACHEDIR.TAG").is_file(), "行を読めない周は既定値へ倒さず消さない");
    assert_eq!(sweep_line(&out), "sweep: removed=0 runs=0 failed=0 drafts=no-rule nogit=1", "stderr: {}", stderr_of(&out));
    clean(&[&repo, &state]);
}

/// 起草の木の中で名が列に無い 7 時間前の印の dir は、木の直下（`target-p2/`）も深い所（`w/retired/t-old/`）も消え、木の追跡
/// されている file と印の無い dir は残り、行は木の数 1 と消した dir の数 2 と、消す前に測った byte の和を MiB に切り上げた 2 を載せる。
#[test]
fn vcache_old_tagged_dirs_in_a_tree_outside_the_names_are_removed() {
    let (repo, state) = repo_with_state();
    drafts_run(&repo, &state);
    let tree = draft_tree(&repo, &state, "s1", "t1");
    let (top, deep, plain) = (tree.join("target-p2"), tree.join("w").join("retired").join("t-old"), tree.join("w").join("plain"));
    for dir in [&top, &deep] {
        tag_dir(dir, SIGNATURE, 768);
    }
    put_sized(&plain.join("app.o"), 4);
    for dir in [&top, &tree.join("w")] {
        rewind(dir, 7);
    }

    let out = end_stop(&state);
    assert!(!top.exists(), "木の直下の名が列に無い古い印の dir は消える");
    assert!(!deep.exists(), "木の深い所の古い印の dir も消える");
    for kept in [tree.join("src").join("lib.rs"), plain.join("app.o")] {
        assert!(kept.is_file(), "{} は残る", kept.display());
    }
    assert_eq!(sweep_line(&out), "sweep: removed=2 runs=0 failed=0 drafts=1 nogit=0 cache=2 cache_mb=2", "stderr: {}", stderr_of(&out));
    clean(&[&repo, &state]);
}

/// 起草の木の中の、深い所に書いたばかりの file を持つ 7 時間前の印の dir（下の 7 時間前の印の dir ごと・印の dir の下は歩かない）と
/// 書いたばかりの印の dir は残り、7 時間前の印の dir だけが消え、残した印の dir は量の線の候補に数わらない（量の記録は used=0 busy=0）。
#[test]
fn vcache_tagged_dirs_in_a_tree_with_a_fresh_write_stay() {
    let (repo, state) = repo_with_state();
    drafts_run(&repo, &state);
    let tree = draft_tree(&repo, &state, "s1", "t1");
    let (busy, fresh, old) = (tree.join("target-steps"), tree.join("target-new"), tree.join("target-old"));
    tag_dir(&busy.join("nested"), SIGNATURE, 4);
    for dir in [&busy, &old] {
        tag_dir(dir, SIGNATURE, 4);
        rewind(dir, 7);
    }
    tag_dir(&fresh, SIGNATURE, 4);
    let deps = busy.join("debug").join("deps");
    put_file(&deps.join("fresh.o"), "x\n");
    age_one(&deps, 7);
    age_one(&busy.join("debug"), 7);

    let out = end_stop(&state);
    assert!(deps.join("fresh.o").is_file(), "深い所に書いたばかりの file を持つ印の dir は残る");
    assert!(busy.join("nested").join("CACHEDIR.TAG").is_file(), "残した印の dir の下の古い印の dir は残る");
    assert!(fresh.join("CACHEDIR.TAG").is_file(), "書いたばかりの印の dir は残る");
    assert!(!old.exists(), "全部古い印の dir だけが消える");
    assert_eq!(sweep_line(&out), "sweep: removed=1 runs=0 failed=0 drafts=1 nogit=0 cache=1 cache_mb=1", "stderr: {}", stderr_of(&out));
    assert_eq!(cap_line(&state), "drafts-cap=ok used=0 cap=102400 over=0 busy=0 unmeasured=0", "残した印の dir は量の線に数えない");
    clean(&[&repo, &state]);
}

/// 便の木（退役先）の名が列に無い 7 時間前の印の dir は残り、`target/` だけが今のまま消え、行に `cache=` は載らない。
#[test]
fn vcache_run_trees_keep_tagged_dirs() {
    let (repo, state) = repo_with_state();
    live_run(&state, "r-end");
    record_event(&state, &["--kind", "RunStage", "--run", "r-done", "--bead", "b", "--stage", "Landed", "--detail", "x"]);
    let retired = vessel::pipe::worktrees_dir(&repo).join("retired").join("r-done");
    let tree = sweep_tree(&repo, &state, "r-done", &retired);
    put_file(&tree.join("target").join("debug").join("app.o"), "x\n");
    tag_dir(&tree.join("target-p2"), SIGNATURE, 4);
    rewind(&tree.join("target-p2"), 7);

    let out = end_stop(&state);
    assert!(!tree.join("target").exists(), "便の木の target/ は今のまま消える");
    assert!(tree.join("target-p2").join("CACHEDIR.TAG").is_file(), "便の木の印の dir は残る");
    assert_eq!(sweep_line(&out), "sweep: removed=1 runs=1 failed=0", "stderr: {}", stderr_of(&out));
    clean(&[&repo, &state]);
}
