//! 索引の表の path を木からの相対に揃える歯（設計 docs/design/reverse-index.md §4 形 2・形 4・接頭辞 `pipe_index_paths_`）。
//!
//! 撃つ口は実 binary の `pipe index build` で、toy repo と偽の宣言の command は親の索引の組み立ての歯の置き場（`idxb_place`）を
//! 使う。期待の表は a1 の読み手と結び（`read_scip`・`read_roles`・`join`・`render`）で fixture から組み、器が置いた表の字と比べる。
//! 撃ち中の木（置き場の下の `<commit の sha>.tree`）の path は commit の後に決まるので、一致の絶対 path と SCIP の project_root
//! はその字から作る。同じ鍵の表は cached で返るので、見本ごとに表と記録を外してから撃ち直す。

use super::*;
use vessel::pipe::index::{flat, join, read_roles, scip};

/// 撃ち中の木の path の字（置き場の下の `<HEAD の sha>.tree`）。
fn ixp_tree(place: &IdxPlace) -> String {
    place.dir().join(format!("{}.tree", place.sha())).display().to_string()
}

/// fixture の SCIP の metadata の project_root を `root` に替えた bytes（fixture の頭 `file:///repo` を外して付け直す）。
fn ixp_scip_at(root: &str) -> Vec<u8> {
    let meta = |uri: &[u8]| pb_len(1, &[pb_int(1, 0), pb_len(3, uri)].concat());
    let bytes = idx_scip_bytes();
    let rest = bytes.strip_prefix(meta(b"file:///repo").as_slice()).unwrap_or_default();
    [meta(root.as_bytes()), rest.to_vec()].concat()
}

/// fixture の役の一致の file を頭 `lead` の下に置いた stream（頭の `./` は外してから付ける）。
fn ixp_roles_under(lead: &str) -> String {
    idx_roles_text().replace("\"file\":\"./", "\"file\":\"").replace("\"file\":\"", &format!("\"file\":\"{lead}/"))
}

/// fixture を頭 `heads` の下に 1 組ずつ置いた形で、a1 の読み手と結びで組んだ表の字（空の頭は repo の根・`roles` が偽なら一致なし）。
fn ixp_table(heads: &[&str], roles: bool) -> String {
    let (mut docs, mut matches, mut bodies) = (Vec::new(), Vec::new(), BTreeMap::new());
    for head in heads {
        let lead = |path: &str| if head.is_empty() { path.to_owned() } else { format!("{head}/{path}") };
        let mut read = scip::read_scip(&idx_scip_bytes()).unwrap_or_default();
        read.iter_mut().for_each(|doc| doc.path = lead(&doc.path));
        docs.extend(read);
        if roles {
            let mut found = read_roles(&idx_roles_text()).map(|read| read.matches).unwrap_or_default();
            found.iter_mut().for_each(|one| one.file = lead(&one.file));
            matches.extend(found);
        }
        bodies.extend(idx_bodies().into_iter().map(|(path, body)| (lead(&path), body)));
    }
    flat::render(&join(&docs, &matches, &bodies).unwrap_or_default())
}

/// 偽の SCIP の indexer が置き場の `ixp-<名>.scip`（名は撃つ行の 1 つ目の引数の末の dir の名）を `{out}` へ写し、偽の役の検出が
/// `roles` を stdout へ出す置き場で `pipe index build` を撃ち、置いた表の字を返す（built でなければ結末の行・撃った後は表と記録を外す）。
fn ixp_built(place: &IdxPlace, scips: &[(&str, &[u8])], roles: &str) -> String {
    for (name, bytes) in scips {
        assert!(fs::write(place.state.join(format!("ixp-{name}.scip")), bytes).is_ok(), "SCIP の見本を書ける");
    }
    assert!(fs::write(place.state.join("ixp.roles"), roles).is_ok(), "一致の見本を書ける");
    idxb_script(place, (IDXB_SCIP, "scip"), &format!("cp '{}/ixp-'\"$(basename \"$1\")\".scip \"$2\"\n", place.state.display()));
    idxb_script(place, (IDXB_ROLES, "roles"), &format!("cat '{}'\n", place.state.join("ixp.roles").display()));
    let line = idxb_line(&place.build(None, &[]));
    let key = idxb_field(&line, "key");
    let table = if idxb_word(&line) == "built" { fs::read_to_string(place.dir().join(format!("{key}.tsv"))).unwrap_or_default() } else { line };
    for ext in ["tsv", "rec"] {
        let _ = fs::remove_file(place.dir().join(format!("{key}.{ext}")));
    }
    table
}

/// 木の下の絶対 path の役の一致は、相対の一致と同じに結ばれる（表の字が等しい）。否定の見本は木の外の絶対 path で、字の頭だけが
/// 木と同じ兄弟の dir と、別の dir の 2 形（どちらも結ばれず、一致の無い表と等しい）。
#[test]
fn pipe_index_paths_absolute_role_files_join_like_relative_ones() {
    let place = idxb_place(IDXB_DECL, "", "");
    let (tree, root) = (ixp_tree(&place), idx_scip_bytes());
    let name = Path::new(&tree).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let (want, bare) = (ixp_table(&[""], true), ixp_table(&[""], false));
    assert_ne!(want, bare, "fixture の役の一致は表を替える");
    assert_eq!(ixp_built(&place, &[(&name, &root)], &ixp_roles_under(&tree)), want, "木の下の絶対 path の一致は結ばれる");
    for lead in [format!("{tree}x"), "/elsewhere".to_owned()] {
        assert_eq!(ixp_built(&place, &[(&name, &root)], &ixp_roles_under(&lead)), bare, "{lead}: 木の外の一致は結ばない");
    }
    clean(&[&place.repo, &place.state]);
}

/// 捕えた名が改行かタブを含む借りの役の一致（doc の行を丸ごと捕えた doclink）は字だけの行にならず、置いた表は読める（表は
/// その一致の無い fixture の表と等しい）。否定の見本は同じ一致の名から改行とタブを外した字で、字だけの行が 1 行足される。
#[test]
fn pipe_index_paths_names_with_line_breaks_leave_no_text_row() {
    let place = idxb_place(IDXB_DECL, "", "");
    let tree = ixp_tree(&place);
    let name = Path::new(&tree).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let root = idx_scip_bytes();
    let want = ixp_table(&[""], true);
    let line = |text: &str| idx_role_line("doclink", "src/a.rs", span_of(IDX_A, "/// 見る: [`Row`] の話", 0), Some(text));
    for broken in ["見る\\n続き", "見る\\t続き", "見る\\r続き"] {
        let roles = format!("{}{}\n", idx_roles_text(), line(broken));
        assert_eq!(ixp_built(&place, &[(&name, &root)], &roles), want, "{broken}: 字だけの行にしない");
    }
    let whole = format!("{}{}\n", idx_roles_text(), line("見る続き"));
    let table = ixp_built(&place, &[(&name, &root)], &whole);
    assert_eq!(table.lines().count(), want.lines().count() + 1, "1 行の名は字だけの行になる");
    assert!(table.lines().any(|row| row.contains("\ttext:見る続き\t")), "{table}");
    clean(&[&place.repo, &place.state]);
}

/// 下の dir を project_root に持つ SCIP（2 本目の行）は、木からの相対の頭を document の path に付けて表に載り、根の SCIP（1 本目の
/// 行・project_root は木そのもの）は頭なしで載る（表は 2 組の fixture を頭 `sub` と根に置いて組んだ字と等しい）。
#[test]
fn pipe_index_paths_scip_of_a_subdir_lands_with_its_head() {
    let decl = "index-scip = [\"index-fake-scip {tree} {out}\", \"index-fake-scip {tree}/sub {out}\"]\nindex-roles = [\"index-fake-roles {tree}\"]\n";
    let place = idxb_place(decl, "", "");
    for (path, body) in idx_bodies() {
        let target = place.repo.join("sub").join(path);
        assert!(target.parent().is_some_and(|dir| fs::create_dir_all(dir).is_ok()) && fs::write(&target, body).is_ok(), "sub の fixture を書ける");
    }
    git(&place.repo, &["add", "-A"]);
    git(&place.repo, &["commit", "-q", "-m", "sub"]);
    let tree = ixp_tree(&place);
    let name = Path::new(&tree).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let (root, sub) = (ixp_scip_at(&format!("file://{tree}")), ixp_scip_at(&format!("file://{tree}/sub")));
    let roles = [idx_roles_text(), ixp_roles_under("sub")].concat();
    let want = ixp_table(&["", "sub"], true);
    assert_ne!(want, ixp_table(&[""], true), "sub の組は表に行を足す");
    assert_eq!(ixp_built(&place, &[(&name, &root), ("sub", &sub)], &roles), want, "sub の document は頭つきで載る");
    clean(&[&place.repo, &place.state]);
}

/// 木の下の dir でない project_root（木そのもの・字の頭だけが木と同じ兄弟の dir・別の dir・`file://` の無い字）の SCIP は、document
/// の path を今のまま（頭なし）で表に載せる（表は根の fixture の表と等しい）。
#[test]
fn pipe_index_paths_project_root_outside_the_tree_keeps_the_paths() {
    let place = idxb_place(IDXB_DECL, "", "");
    let tree = ixp_tree(&place);
    let name = Path::new(&tree).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let want = ixp_table(&[""], true);
    for root in [format!("file://{tree}"), format!("file://{tree}x/sub"), "file:///elsewhere/sub".to_owned(), format!("{tree}/sub")] {
        assert_eq!(ixp_built(&place, &[(&name, &ixp_scip_at(&root))], &idx_roles_text()), want, "{root}: 頭を付けない");
    }
    clean(&[&place.repo, &place.state]);
}

/// 索引の子の箱の memory の上限（偽 systemd-run の argv の `MemoryMax`）は、子ごとに rules 行 `gate.job_memory_mb` の 2 倍。
#[test]
fn pipe_index_paths_children_box_holds_two_jobs_of_memory() {
    let place = idxb_with_fixtures();
    let line = idxb_line(&place.build(None, &[]));
    assert_eq!(idxb_word(&line), "built", "{line}");
    let job = Manifest::embedded().ok().and_then(|manifest| manifest.get("gate.job_memory_mb").map(|row| row.value.clone()));
    let job = match job {
        Some(RuleValue::Int(found)) => found,
        _ => 0,
    };
    assert!(job > 0, "rules 行 gate.job_memory_mb を読める");
    let want = format!("MemoryMax={}M", job * 2);
    for unit in ["-index-0-", "-index-1-"] {
        let record = crate::toolbox_record(&place.state, unit);
        assert!(record.lines().any(|row| row == want), "{unit}: {want}: {record}");
    }
    clean(&[&place.repo, &place.state]);
}

/// 索引の子（SCIP の行と役の行）の env の `CARGO_TARGET_DIR` は、親の env に在っても無くても木の下の `target` に固定される
/// （記録は子ごとに 1 行・親の値は子に渡らない）。
#[test]
fn pipe_index_paths_children_build_in_the_tree_target() {
    let place = idxb_place(IDXB_DECL, "", "");
    let tree = ixp_tree(&place);
    let seen = place.state.join("ixp-target");
    let record = format!("printf '%s\\n' \"${{CARGO_TARGET_DIR-unset}}\" >> '{}'\n", seen.display());
    idxb_script(&place, (IDXB_SCIP, "scip"), &format!("{record}cp '{}' \"$2\"\n", place.state.join("idx.scip").display()));
    idxb_script(&place, (IDXB_ROLES, "roles"), &format!("{record}cat '{}'\n", place.state.join("idx.roles").display()));
    let (state, repo) = (place.state.display().to_string(), place.repo.display().to_string());
    let outside = place.state.join("outside-target").display().to_string();
    for parent in [Some(outside.as_str()), None] {
        let mut cmd = bin_cmd();
        cmd.args(["pipe", "index", "build", "--state-dir", state.as_str(), "--repo", repo.as_str()]).env("PATH", place.path());
        match parent {
            Some(value) => cmd.env("CARGO_TARGET_DIR", value),
            None => cmd.env_remove("CARGO_TARGET_DIR"),
        };
        let out = cmd.output();
        let line = out.as_ref().map(idxb_line).unwrap_or_default();
        assert_eq!(idxb_word(&line), "built", "{parent:?}: {line}");
        let key = idxb_field(&line, "key");
        let lines = fs::read_to_string(&seen).unwrap_or_default();
        assert_eq!(lines.lines().collect::<Vec<_>>(), [format!("{tree}/target"), format!("{tree}/target")], "{parent:?}: 子ごとに木の下の target");
        for path in [seen.clone(), place.dir().join(format!("{key}.tsv")), place.dir().join(format!("{key}.rec"))] {
            assert!(fs::remove_file(path).is_ok(), "{parent:?}: 記録と表を外す");
        }
    }
    clean(&[&place.repo, &place.state]);
}
