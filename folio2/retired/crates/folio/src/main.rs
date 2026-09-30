//! folio の binary。命令の読みと印字は lib の入口 `folio::entry::run` が持ち、ここは標準出力と標準エラーを渡して終わるだけ。
#![forbid(unsafe_code)]
#![deny(
    unused_must_use,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::exit,
    clippy::indexing_slicing,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason
)]

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let (mut out, mut err) = (std::io::stdout(), std::io::stderr());
    let code = folio::entry::run(std::env::args_os(), &mut out, &mut err);
    let _ = out.flush();
    ExitCode::from(code)
}
