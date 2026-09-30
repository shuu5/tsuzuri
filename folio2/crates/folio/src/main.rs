//! folio の binary。命令の読みと印字は lib の入口 `folio::entry::run` が持ち、ここは標準出力と標準エラーを渡して終わるだけ。

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let (mut out, mut err) = (std::io::stdout(), std::io::stderr());
    let code = folio::entry::run(std::env::args_os(), &mut out, &mut err);
    let _ = out.flush();
    ExitCode::from(code)
}
