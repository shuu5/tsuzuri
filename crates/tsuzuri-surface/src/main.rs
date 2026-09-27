//! 面の入口（trunk が wasm の target で組み立て、body に project board を載せる）。

#[cfg(target_arch = "wasm32")]
fn main() {
    tsuzuri_surface::board::mount();
}

/// host では画面を持たない（host の build と clippy を workspace の全部で通すための空の入口）。
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("面は wasm の target で組み立てる: cargo run -q -p xtask -- surface-build");
}
