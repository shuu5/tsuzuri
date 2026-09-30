//! tz の口の folio 由来の 11 subcommand（行 k-tz-entry・要件 FR17・判断の記録 ADR-8 の決定 (1)〜(6)）。
//! 口の名は folio の同じ名の口と同じで、引数と終了 code（合格 0・不合格 1・まだ分からない 2）も folio の口のまま、
//! folio の lib の入口 `folio::entry::run` を命令の名 tz で撃つだけ。inject と serve は持たない。
//! 索引の旗（`INDEX_FLAGS`）を 1 つでも持つ tz graph は folio の graph の口へ渡し（吸収・決定 (3)）、持たない tz graph は
//! 今の口（`cli::graph`）のまま。2 つを混ぜた引数は使い方の誤り 1。

use std::io::Write;

use super::graph;

/// 口の名（要件 FR17 の字の順）。
pub const NAMES: [&str; 11] = [
    "check", "schema", "derive", "ceiling", "init", "parts", "face", "figure", "build", "intake",
    "hello",
];

/// folio の graph の口へ渡す索引の旗。
pub const INDEX_FLAGS: [&str; 3] = ["--print", "--digest", "--summary"];

/// 索引の旗と混ぜない今の tz graph の旗。
pub const GRAPH_FLAGS: [&str; 7] = [
    "--check",
    "--design",
    "--repo",
    "--bd",
    "--folio",
    "--state-dir",
    "--project",
];

/// 索引の形の tz graph の使い方の行。
const INDEX_USAGE: &str =
    "usage: tz graph (--print | --digest) [--summary] [--dir <dir>]（--summary は --print と使う）";

/// 口 `name` を folio の入口で撃つ（命令の名は tz・書き先は標準出力と標準エラー）。
pub fn run(name: &str, rest: &[&str]) -> u8 {
    let args = ["tz", name].into_iter().chain(rest.iter().copied());
    let (mut out, mut err) = (std::io::stdout(), std::io::stderr());
    let rc = ::folio::entry::run(args, &mut out, &mut err);
    let _ = out.flush();
    rc
}

/// 索引の旗を 1 つでも持つか（旗は値を取らないので字の一致で見る）。
pub fn is_index(rest: &[&str]) -> bool {
    rest.iter().any(|a| INDEX_FLAGS.contains(a))
}

/// 索引の形の tz graph。今の tz graph の旗（`--名` と `--名=値` の名）が在れば混ぜた使い方として 1、無ければ口 graph を撃つ。
pub fn graph(rest: &[&str]) -> u8 {
    let mixed = rest
        .iter()
        .map(|a| a.split_once('=').map_or(*a, |(name, _)| name))
        .find(|name| GRAPH_FLAGS.contains(name));
    match mixed {
        Some(flag) => {
            eprintln!(
                "tz graph: 索引の旗（{}）と {flag} は混ぜない\n{}\n{INDEX_USAGE}",
                INDEX_FLAGS.join("・"),
                graph::USAGE
            );
            1
        }
        None => run("graph", rest),
    }
}
