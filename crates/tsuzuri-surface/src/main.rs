//! 面の入口（trunk が wasm の target で組み立て、body に board を載せる）。
//! URL の query の board が account のときだけ account board を、ほかは今のまま project board を載せる。
//! 載せる前に stylesheet の部品を head の末尾へ入れる。

#[cfg(target_arch = "wasm32")]
fn main() {
    tsuzuri_surface::style::inject();
    let search = leptos::prelude::window()
        .location()
        .search()
        .unwrap_or_default();
    if tsuzuri_surface::account::selects(&search) {
        tsuzuri_surface::account::board::mount();
    } else {
        tsuzuri_surface::board::mount();
    }
}

/// host の入口の出力の手（行 k-lint-print）: 標準エラーへ 1 行を書く（字の後に改行）。面の crate が書くのはこの関数だけ。
#[cfg(not(target_arch = "wasm32"))]
#[expect(
    clippy::print_stderr,
    reason = "面の crate の host の入口の標準エラーをこの 1 関数に閉じるための例外"
)]
fn emit_err(line: &str) {
    eprintln!("{line}");
}

/// host では画面を持たない（host の build と clippy を workspace の全部で通すための空の入口）。
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    emit_err("面は wasm の target で組み立てる: cargo run -q -p xtask -- surface-build");
}
