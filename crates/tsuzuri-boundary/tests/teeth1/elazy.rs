//! 見張りの Source の口の読みが印の遅れを見て読む歯（接頭辞 elazy_・設計ノート surface-wave23b 行 e-ledger-lazy の完了の条件）。
//! 偽の bd は撃たれるたびに記録の file に 1 行を足して字 [] を出す script。作業場は CARGO_TARGET_TMPDIR の下に歯ごとに作る。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::server::ledger::Source;

/// 歯ごとの作業場（前の歯の残りを消して作り直す）。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("elazy")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("repo/.beads")).expect("repo の置き場");
    root
}

/// 偽の bd が撃たれた回（記録の file の行の数）。
fn calls(root: &Path) -> usize {
    fs::read_to_string(root.join("calls"))
        .unwrap_or_default()
        .lines()
        .count()
}

/// 印の file に 1 行足した字を隣の file に書いてから rename で置き替える（更新時刻と長さが 1 度に動く）。
fn append_line(path: &Path) {
    let mut text = fs::read_to_string(path).unwrap_or_default();
    text.push_str("{}\n");
    let tmp = path.with_file_name("issues.jsonl.tmp");
    fs::write(&tmp, text).expect("隣の file");
    fs::rename(&tmp, path).expect("置き替える");
}

#[test]
fn elazy_watched_reads_when_behind() {
    let root = place("behind");
    let repo = root.join("repo");
    let issues = repo.join(".beads/issues.jsonl");
    fs::write(&issues, "{}\n").expect("issues.jsonl");
    let bd = root.join("bd");
    fs::write(
        &bd,
        format!(
            "#!/bin/sh\necho x >> '{}/calls'\necho '[]'\n",
            root.display()
        ),
    )
    .expect("偽の bd");
    fs::set_permissions(&bd, fs::Permissions::from_mode(0o755)).expect("偽の bd の権限");
    let source = Source::new(&repo, &bd).watched();
    source.read();
    assert_eq!(calls(&root), 1, "見張りの読み");
    // 印が動かなければ撃たない。
    assert_eq!(source.got().text.as_deref(), Some("[]\n"));
    assert_eq!(calls(&root), 1, "read の後の got");
    // 印が動けば、見張りを待たず自分で 1 回読む。
    append_line(&issues);
    assert_eq!(source.got().text.as_deref(), Some("[]\n"));
    assert_eq!(calls(&root), 2, "印が動いた後の got");
    // 読んだ後は、印が同じなので撃たない。
    source.got();
    source.text();
    assert_eq!(calls(&root), 2, "その後の got");
    let _ = fs::remove_dir_all(&root);
}
