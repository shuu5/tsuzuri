//! 器の口 pipe preflight の verify の欄の照らしの歯（接頭辞 `vpfgap_`・tsuzuri のノート surface-wave29d の行 v-preflight-gaps・親
//! `tests/e2e/pipe/intake.rs` の helper を `use super::*` で使う）。
//!
//! toy の導出の行の verify を 1 句ずつ替えて preflight を撃ち、filter 語が 2 つの nextest の行と、宣言の common-verify（toy は
//! `git status`）と同じ行だけが preflight だけの断りの 1 行になり、ほかの行は通ることを、rc と `refuse=` の行と末尾の行から測る。

use super::*;

/// verify の欄 `verify`（TOML の配列の字）の導出の行 `a` を toy に置き、preflight を 1 回撃った出力（repo と置き場は畳む）。
fn preflight_with(verify: &str) -> Output {
    let row = derive_row("a", &[("verify", verify)]);
    let (repo, state) = derive_repo(&table_doc(&table_region(&[row])));
    let out = preflight_raw(&repo, &state, "docs/design/toy.md#a", "s2-a", true);
    assert_eq!(run_dirs(&state), Vec::<String>::new(), "run dir を作らない");
    clean(&[&repo, &state]);
    out
}

/// 通る見本（filter 語 1 つの行・--manifest-path の引数を語に数えない行）は rc 0 で末尾 ok、filter 語が 2 つの行と共通 verify と同じ行は
/// rc 1 で preflight だけの断りの 1 行（verify-filters・verify-common）と末尾の件数 1 で、断りは行の pointer と直し方を名指す。
#[test]
fn vpfgap_binary_refuses_two_filter_lines_and_the_common_line() {
    let ok = "cargo nextest run -p toy --no-tests=fail derive_";
    let cases = [
        (format!("[\"{ok}\"]"), None),
        ("[\"cargo nextest run --manifest-path crates/toy/Cargo.toml -p toy --no-tests=fail derive_\"]".to_owned(), None),
        ("[\"cargo nextest run -p toy --no-tests=fail derive_ other_\"]".to_owned(), Some(("verify-filters", "1 行に filter の語を 1 つずつ割る"))),
        (format!("[\"{ok}\", \"git status\"]"), Some(("verify-common", "行の verify から外す"))),
    ];
    for (verify, refused) in cases {
        let out = preflight_with(&verify);
        let text = stdout_of(&out);
        let lines = fact_lines(&out, "refuse=");
        match refused {
            None => {
                assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "{verify}: {text} {}", stderr_of(&out));
                assert!(lines.is_empty(), "{verify}: {text}");
                assert_eq!(tail_line(&out), "preflight: ok", "{verify}: {text}");
            }
            Some((name, fix)) => {
                assert_eq!(out.status.code(), Some(i32::from(RC_REFUSED)), "{verify}: {text}");
                let head = format!("refuse={name}:行 docs/design/toy.md#a の verify ");
                assert!(matches!(lines.as_slice(), [one] if one.starts_with(&head) && one.ends_with(fix)), "{verify}: {text}");
                assert_eq!(tail_line(&out), "preflight: refused n=1", "{verify}: {text}");
            }
        }
    }
}
