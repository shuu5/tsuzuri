//! 追認の札の族の歯（接頭辞 `vadopt_`・判断の記録 ADR-45 の門 H6・親 `tests/e2e/fleet.rs` の helper を `use super::*` で使う）。
//!
//! 発端の trailer を持つ後の commit の本文の行 `<Name>-Adopts: <sha> <bead id>…` が、切り替えの線より後の trailer の無い commit に
//! 発端を結び、全部の書き直しの出力がその commit を発端の判じ（台帳に在れば commit-landed・無ければ source-unresolved）に掛ける
//! ことを binary で測る（key は core の導出と独立に組む）。

use super::*;

/// trailer の key（NAME の先頭を大文字にして `-<stem>: ` を足す・core の導出と独立に組む）。
fn key(stem: &str) -> String {
    let mut chars = vessel::name::NAME.chars();
    let head: String = chars.next().map(|first| first.to_uppercase().to_string()).unwrap_or_default();
    format!("{head}{}-{stem}: ", chars.as_str())
}

/// toy repo に本文 `message` の空の commit を 1 つ積み、`refs/remotes/origin/main` をそこへ進めて sha を返す。
fn land(life: &Life, message: &str) -> String {
    crate::pipe::git(&life.repo, &["commit", "-q", "--allow-empty", "-m", message]);
    let sha = crate::pipe::git(&life.repo, &["rev-parse", "HEAD"]);
    crate::pipe::git(&life.repo, &["update-ref", "refs/remotes/origin/main", &sha]);
    sha
}

/// `fleet lifecycle show` の commit の部品の行から、sha の (局面, 理由) を引く（無ければ `None`）。
fn commit_phase(life: &Life, sha: &str) -> Option<(String, String)> {
    let shown = life.shown();
    let line = shown.lines().find(|line| line.starts_with(&format!("part=commit id={sha} ")))?;
    let field = |key: &str| line.split_whitespace().find_map(|word| word.strip_prefix(key)).map(str::to_owned);
    Some((field("phase=")?, field("reason=")?))
}

fn phase(word: &str, reason: &str) -> Option<(String, String)> {
    Some((word.to_owned(), reason.to_owned()))
}

/// 札の前は、切り替えの線より後の trailer の無い commit が 3 本とも commit-no-trailer の misfit。発端の trailer（台帳に在る `toy-c1`）を
/// 持つ commit の札が名指した 2 本は、台帳に在る id なら commit-landed・無い id なら source-unresolved に移る。発端の trailer を持たない
/// commit の札は何も結ばず、名指された 3 本目も札だけの commit 自身も commit-no-trailer のまま・札を持つ commit は自分の発端で判じる。
#[test]
fn vadopt_adopts_line_binds_the_source_and_leaves_other_commits() {
    let life = Life::new();
    assert_eq!(life.write(&[]).status.code(), Some(i32::from(RC_OK)), "切り替えの線を引く 1 周目");
    let old = land(&life, "trailer の無い commit");
    let unknown = land(&life, "台帳に無い id で結ぶ commit");
    let bare = land(&life, "札だけの commit が名指す commit");
    assert_eq!(life.write(&[]).status.code(), Some(i32::from(RC_OK)));
    for sha in [&old, &unknown, &bare] {
        assert_eq!(commit_phase(&life, sha), phase("misfit", "commit-no-trailer"), "札の前は misfit");
    }
    let (source, adopts) = (key("Source"), key("Adopts"));
    let adopter = land(&life, &format!("札\n\n{source}toy-c1\n{adopts}{old} toy-c1\n{adopts}{unknown} toy-zz"));
    let unsourced = land(&life, &format!("発端の trailer の無い札\n\n{adopts}{bare} toy-c1"));
    assert_eq!(life.write(&[]).status.code(), Some(i32::from(RC_OK)));
    assert_eq!(commit_phase(&life, &old), phase("commit-landed", "-"), "札が台帳に在る id を結ぶ");
    assert_eq!(commit_phase(&life, &unknown), phase("misfit", "source-unresolved"), "台帳に無い id は source-unresolved");
    assert_eq!(commit_phase(&life, &bare), phase("misfit", "commit-no-trailer"), "発端の trailer の無い commit の札は結ばない");
    assert_eq!(commit_phase(&life, &adopter), phase("commit-landed", "-"), "札を持つ commit は自分の発端で判じる");
    assert_eq!(commit_phase(&life, &unsourced), phase("misfit", "commit-no-trailer"), "札だけの commit は自分の外れを解かない");
    crate::pipe::clean(&[&life.repo, &life.state]);
}
