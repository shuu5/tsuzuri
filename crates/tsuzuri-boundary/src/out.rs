//! 境界の crate の出力の手: 標準出力と標準エラーへ書くのはこの 2 つの関数だけ（器の境界の main の emit と同じ形）。
//! どちらも字の後に改行を 1 つ足して書く。lint の例外は各関数の理由を持つ expect の属性だけ（条 P-24.3）。

/// 標準出力へ 1 行を書く（字の後に改行）。
#[expect(
    clippy::print_stdout,
    reason = "境界の crate の標準出力をこの 1 関数に閉じるための例外"
)]
pub fn emit(line: &str) {
    println!("{line}");
}

/// 標準エラーへ 1 行を書く（字の後に改行）。
#[expect(
    clippy::print_stderr,
    reason = "境界の crate の標準エラーをこの 1 関数に閉じるための例外"
)]
pub fn emit_err(line: &str) {
    eprintln!("{line}");
}
