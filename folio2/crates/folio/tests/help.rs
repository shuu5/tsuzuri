//! 便 192（docs/design/delivery-192.md §1 (c)・台帳 f2-648.185・天井の 31 周目 実態 F-4・判断の記録 ADR-12 の帰結）:
//! 命令の説明の字（`folio <命令> --help` の 1 行目）が今の実装と揃っていることの歯。期待の字は歯の側の手書き（P-10.1）。
//! 1. folio face は 5 面（見本 3 面は 2026-09-18 に退役・生成器は 5 面とも在る）。
//! 2. folio check は正本 7 file の形だけでなく、判断の記録・設計ノート・凍結 anchor・索引の欄の決まり・参照 id・索引と面が組めるか（面は便 187）を数える。
//! 3. folio serve は同じ端末の中（loopback）か tailnet の中（条 N-6.1 の第 1.2 版の字）。
//! 4. folio derive の --from-root の説明が在る。

use std::process::Command;

/// `folio <args> --help` の標準出力。
fn help(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_folio"))
        .args(args)
        .arg("--help")
        .output()
        .expect("folio を起動できない");
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

/// 説明の 1 行目。
fn about(cmd: &str) -> String {
    help(&[cmd]).lines().next().unwrap_or_default().to_string()
}

#[test]
fn f192_face_help_names_the_five_faces() {
    assert_eq!(
        about("face"),
        "正本から 5 面（入口・憲法・要件書・判断の記録・設計ノート）の 1 面を導出して書く（--write）・検査する（--check）"
    );
    // --face が受ける名の並びも 5 つ（説明と同じ数）
    assert!(
        help(&["face"]).contains("面の名（index・constitution・srs・adr・note）"),
        "{}",
        help(&["face"])
    );
}

#[test]
fn f192_check_help_names_what_the_floor_counts() {
    assert_eq!(
        about("check"),
        "設計文書の置き場の床（正本 7 file〔憲法・規則の表・語彙・要件書・入口・相談窓口・天井〕の形と、判断の記録・設計ノート・凍結 anchor・索引の欄の決まりと、参照 id と、索引と面が組めるか）を検査し、合格 0 / 不合格 1 / まだ分からない 2 で終わる"
    );
}

#[test]
fn f192_serve_help_names_loopback_and_the_tailnet() {
    assert_eq!(
        about("serve"),
        "配信先を同じ端末の中（loopback）か tailnet の中だけで見せる（bind 先がそのどちらでもなければ起動を拒む）"
    );
}

#[test]
fn f192_derive_help_names_from_root() {
    let h = help(&["derive"]);
    assert!(
        h.contains("--from-root")
            && h.contains("--out の相対を、置き場を含む版管理の根（無ければ置き場の親 dir・器の導出 file を探す根と同じ）からの相対に解く"),
        "{h}"
    );
}

#[test]
fn f192_the_command_list_carries_the_same_texts() {
    // 命令の一覧（folio --help）の行も同じ字（古い字が残らない）
    let top = help(&[]);
    for old in ["見本 3 面", "本便で生成器を持つのは", "design-intent の正本 7 file の形を検査し", "tailnet の内側だけで見せる"] {
        assert!(!top.contains(old), "古い字「{old}」が残る: {top}");
    }
    for cmd in ["face", "check", "serve"] {
        assert!(top.contains(&about(cmd)), "{cmd}: {top}");
    }
}
