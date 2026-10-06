//! 席の名のずれの歯（接頭辞 `sdrift_`・設計 docs/design/seat-roles.md §5）。
//! tmux の session 名が登録とずれた席の SessionStart の 1 行と、権能の門の断りの末の字を、偽の tmux を置いた外形から測る。

use super::*;

/// ずれの見本の席の名（偽の tmux が今答える target と登録 row の target）。
const NOW: &str = "dnow:dwin";
const OLD: &str = "dold:dwin";

/// SessionStart の『ずれの 1 行』（[`NOW`] と [`OLD`] の見本の字）。
const DRIFT_LINE: &str = "[scribe2/SessionStart] seat-drift target=dnow:dwin registered=dold:dwin next=tmux rename-session -t dnow dold\
（tmux の session 名が席の登録とずれている・指示文を出さず書きは権能の門が断る）";

/// 権能の門の『断りの末の字』（[`NOW`] と [`OLD`] の見本の字）。
const DRIFT_TAIL: &str = "registered=dold:dwin next=tmux rename-session -t dnow dold";

/// どの引数にも `target` を答える偽の tmux を `name` の子 dir に置き、その dir を先頭に足した PATH の値を返す
/// （[`stub_tmux_path`] と同じ組み方・席の名の 2 つを分けて答える）。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn named_tmux(place: &RolePlace, name: &str, target: &str) -> String {
    use std::os::unix::fs::PermissionsExt;
    let bin_dir = place.sock_dir.join(format!("tmux-{name}"));
    fs::create_dir_all(&bin_dir).expect("偽 tmux の dir を作れる");
    let stub = bin_dir.join("tmux");
    fs::write(&stub, format!("#!/bin/sh\necho '{target}'\n")).expect("偽 tmux を書ける");
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).expect("偽 tmux に実行権を付ける");
    format!("{}:{}", bin_dir.display(), std::env::var("PATH").unwrap_or_default())
}

/// `target` を答える偽の tmux で session-start を 1 度撃って打刻を置き、`seat register` で `target` と `anchor` の登録 row を積む。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn registered(place: &RolePlace, target: &str, anchor: &Path) {
    let path = named_tmux(place, "registered", target);
    let out = run_stub_hook(&path, &["session-start", "--pane", STUB_PANE], &stamp_payload(&place.repo, "sid-drift"));
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "打刻の session-start は rc 0: {}", stderr_text(&out));
    let out = Command::new(bin())
        .args(["seat", "register", "--state-dir", &place.state.display().to_string(), "--target", target])
        .args(["--role", "orchestrator", "--account", "a1", "--launch", &place.launch, "--anchor", &anchor.display().to_string()])
        .output()
        .expect("binary を起動できる");
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "seat register は rc 0: {}", stderr_text(&out));
}

/// 今は [`NOW`] と答える偽の tmux で session-start を撃ち、名乗りの後ろの行と、増えた記録の行を返す。
fn start_now(place: &RolePlace) -> (Vec<String>, Vec<String>) {
    let path = named_tmux(place, "now", NOW);
    let before = inject_lines(&place.state).len();
    let args = ["session-start", "--pane", STUB_PANE, "--rules", &place.rules, "--bd", &place.bd];
    let out = run_stub_hook(&path, &args, &stamp_payload(&place.repo, "sid-drift"));
    let added = inject_lines(&place.state).split_off(before);
    (after_header(&out), added)
}

/// 席の名がずれた席の SessionStart は、名乗りの後ろにずれの 1 行だけを出し、記録を 1 行足す（指示文は出さない）。
#[test]
fn sdrift_session_start_names_the_registered_target() {
    let place = role_place();
    registered(&place, OLD, &place.repo);
    let (lines, added) = start_now(&place);
    assert_eq!(lines, vec![DRIFT_LINE.to_owned()], "名乗りの後ろはずれの 1 行だけ");
    assert_eq!(added.len(), 2, "記録は名乗りとずれの 2 行: {added:?}");
    let line = added.last().cloned().unwrap_or_default();
    assert_eq!(what_of(&line), "session-start-drift", "記録の what: {line}");
    assert!(line.contains("dnow_dwin"), "記録は今の席を名乗る: {line}");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// anchor の違う登録 row と窓の名の違う登録 row は、ずれと読まない（名乗りだけ・記録は名乗りの 1 行だけ）。
#[test]
fn sdrift_session_start_stays_silent_without_a_drift() {
    let other = git_repo();
    let place = role_place();
    registered(&place, OLD, &other);
    let (lines, added) = start_now(&place);
    assert_eq!(lines, Vec::<String>::new(), "anchor が違う登録 row はずれと読まない");
    assert_eq!(added.len(), 1, "記録は名乗りの 1 行だけ: {added:?}");
    assert_eq!(what_of(&added.last().cloned().unwrap_or_default()), "session-start-header");

    let place_window = role_place();
    registered(&place_window, "dold:dother", &place_window.repo);
    let (lines, added) = start_now(&place_window);
    assert_eq!(lines, Vec::<String>::new(), "窓の名が違う登録 row はずれと読まない");
    assert_eq!(added.len(), 1, "記録は名乗りの 1 行だけ: {added:?}");
    assert_eq!(what_of(&added.last().cloned().unwrap_or_default()), "session-start-header");
    clean(&[&other, &place.repo, &place.state, &place.sock_dir, &place_window.repo, &place_window.state, &place_window.sock_dir]);
}

/// 席の名がずれた席の権能の門の断りは、理由と route の後ろに登録の target と直し方を足す。ずれていない登録の無い席は足さない。
#[test]
fn sdrift_guard_names_the_registered_target() {
    let place = role_place();
    registered(&place, OLD, &place.repo);
    let path = named_tmux(&place, "now", NOW);
    let args = ["pre-tool-use", "--pane", STUB_PANE, "--rules", &place.rules];
    let payload = bash_payload(&place.repo, &answer_line());
    let text = assert_role_deny(&run_stub_hook(&path, &args, &payload), "ずれた席");
    assert!(text.contains("reason=unregistered（"), "理由の字面は不変: {text}");
    assert_eq!(text.matches("route=").count(), 1, "route は 1 句: {text}");
    assert!(text.trim_end().ends_with(DRIFT_TAIL), "断りの末は登録の target と直し方: {text}");

    let place_window = role_place();
    registered(&place_window, "dold:dother", &place_window.repo);
    let path = named_tmux(&place_window, "now", NOW);
    let payload = bash_payload(&place_window.repo, &answer_line());
    let text = assert_role_deny(&run_stub_hook(&path, &args, &payload), "窓の名が違う席");
    assert!(text.contains("reason=unregistered（"), "{text}");
    assert!(!text.contains("registered="), "ずれでない断りは登録の target を足さない: {text}");
    clean(&[&place.repo, &place.state, &place.sock_dir, &place_window.repo, &place_window.state, &place_window.sock_dir]);
}
