//! 器の口 pipe preflight の done の項の印の照らしの歯（接頭辞 `vpflit_`・tsuzuri のノート surface-v4a の行 v-pf-literal・親
//! `tests/e2e/pipe/intake.rs` の helper を `use super::*` で使う）。
//!
//! toy の導出の行 `a` の done を 1 句ずつ替えて preflight を撃ち、done の項の二重鉤括弧の印の字が節 1 の本文に在る周は通り、
//! 無い字を囲む項を足した周は preflight だけの断りの 1 行になることを、rc と `refuse=` の行と末尾の行から測る。

use super::*;

/// done の欄 `done`（TOML の字）の導出の行 `a` を toy に置き、preflight を 1 回撃った出力（repo と置き場は畳む）。verify は filter 語 1 つの
/// 行にして verify-common に当たらない形にする。
fn preflight_with(done: &str) -> Output {
    let verify = "[\"cargo nextest run -p toy --no-tests=fail derive_\"]";
    let row = derive_row("a", &[("done", done), ("verify", verify)]);
    let (repo, state) = derive_repo(&table_doc(&table_region(&[row])));
    let out = preflight_raw(&repo, &state, "docs/design/toy.md#a", "s2-a", true);
    assert_eq!(run_dirs(&state), Vec::<String>::new(), "run dir を作らない");
    clean(&[&repo, &state]);
    out
}

/// 印の字が節 1 の本文（`本文。`）に在る周は rc 0 で末尾 ok、材料に無い字を囲む項を足した周は rc 1 で preflight だけの断りの 1 行
/// （行の pointer と項の番号と字を名指す）と末尾の件数 1。
#[test]
fn vpflit_binary_refuses_a_literal_missing_from_the_section() {
    let ok = "\"(1) 字 『本文。』 を出す\"";
    let out = preflight_with(ok);
    let text = stdout_of(&out);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{text} {}", stderr_of(&out));
    assert!(fact_lines(&out, "refuse=").is_empty(), "{text}");
    assert_eq!(tail_line(&out), "preflight: ok", "{text}");

    let out = preflight_with("\"(1) 字 『本文。』 を出す (2) 字 『無い字』 を出す\"");
    let text = stdout_of(&out);
    assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{text}");
    let want = "refuse=section-literal:行 docs/design/toy.md#a の done の項 2 の字 『無い字』 が審査の材料の節に無い — 節の本文に同じ字を書くか印を外す";
    assert_eq!(fact_lines(&out, "refuse="), [want], "{text}");
    assert_eq!(tail_line(&out), "preflight: refused n=1", "{text}");
}
