//! 口座の信頼の印の書き手の歯（接頭辞 cwtrw_・設計ノート surface-wave29b 行 cs-trust・判断の記録 ADR-55 決定 (3)）。
//! 歯ごとの置き場（CARGO_TARGET_TMPDIR の下）を口座の置き場に見立て、書き手 `place_trust` を直に撃つ（本物の口座の置き場は読まない）。
#![cfg(test)]

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use tsuzuri_boundary::consult::trust::place_trust;
use tsuzuri_core::consult::trust::trusted;

/// 歯ごとの口座の置き場（前の撃ちの残りを消して作る）。
fn place(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("cwtrw")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// 信頼の印の書き手は、file が無ければ印だけの file を作り、在れば中核の trusted の字に置き換え（mode を保ち・一時 file を残さない）、
/// 既に真なら 1 byte も書かず（更新時刻も替えない）、JSON でない file は替えずに rc 2 の断りを返す。
#[test]
fn cwtrw_place_trust_writes_once() {
    let acct = place("once");
    let w = "/W";
    let file = acct.join(".claude.json");
    assert_eq!(place_trust(&acct, w), Ok(()));
    assert_eq!(fs::read_to_string(&file).ok(), trusted(None, w).unwrap());
    let before = r#"{"projects":{"/W":{"hasTrustDialogAccepted":false}},"z":1}"#;
    fs::write(&file, before).unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(place_trust(&acct, w), Ok(()));
    assert_eq!(
        fs::read_to_string(&file).ok(),
        trusted(Some(before), w).unwrap()
    );
    assert_eq!(fs::metadata(&file).unwrap().mode() & 0o777, 0o600);
    assert_eq!(
        fs::read_dir(&acct).unwrap().count(),
        1,
        "一時 file を残さない"
    );
    let at = fs::metadata(&file).unwrap().modified().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    assert_eq!(place_trust(&acct, w), Ok(()));
    assert_eq!(fs::metadata(&file).unwrap().modified().unwrap(), at);
    fs::write(&file, "{\"projects\": ").unwrap();
    let (rc, why) = place_trust(&acct, w).unwrap_err();
    assert!(rc == 2 && why.contains("JSON の字でない"), "{why}");
    assert_eq!(fs::read_to_string(&file).unwrap(), "{\"projects\": ");
}
