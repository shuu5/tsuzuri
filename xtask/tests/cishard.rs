//! CI の歯の分け方（行 t-ci-shard）: ci.yml の job check の matrix が 1 から N の全部を持ち、env の N が列の数と同じで、
//! 変数の名は ci.yml と xtask の const にだけ在る（器の common-verify は変数を持たず歯の全部を撃つ）。
//! 外の依存を使わず、ci.yml の行を字で読む。

use std::path::PathBuf;

const VAR: &str = "TSUZURI_CHECK_PARTITION";
const RUN_LINE: &str = "cargo run -q -p xtask -- check";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask は workspace の root の直下に在る")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

/// job check の行（見出しの次から、4 つの空白で始まる行か注の行が続く間）。
fn job_check(ci: &str) -> Vec<&str> {
    ci.lines()
        .skip_while(|l| *l != "  check:")
        .skip(1)
        .take_while(|l| l.starts_with("    ") || l.trim_start().starts_with('#'))
        .collect()
}

/// `key: value` の行の value（行の頭の空白と注の行は除く）。
fn values<'a>(lines: &[&'a str], key: &str) -> Vec<&'a str> {
    lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.strip_prefix(key)?.strip_prefix(':'))
        .map(str::trim)
        .collect()
}

#[test]
fn cishard_matrix_covers_every_partition() {
    let ci = read(".github/workflows/ci.yml");
    let job = job_check(&ci);
    assert!(!job.is_empty(), "ci.yml に job check が在る");

    let shards = values(&job, "shard");
    assert_eq!(shards.len(), 1, "matrix の shard の列は 1 つ: {shards:?}");
    let list = shards[0]
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or_else(|| panic!("shard の列は [ … ] の形: {}", shards[0]));
    let listed: Vec<u32> = list
        .split(',')
        .map(|s| {
            s.trim()
                .parse()
                .unwrap_or_else(|_| panic!("shard の字: {s:?}"))
        })
        .collect();
    let n = u32::try_from(listed.len()).expect("shard の数");
    assert!(n >= 2, "shard は 2 つ以上: {listed:?}");
    assert_eq!(
        listed,
        (1..=n).collect::<Vec<_>>(),
        "shard は 1 から N まで"
    );

    let envs = values(&job, VAR);
    assert_eq!(envs.len(), 1, "env の行は 1 つ: {envs:?}");
    assert_eq!(envs[0], format!("count:${{{{ matrix.shard }}}}/{n}"));

    let runs: Vec<&str> = job
        .iter()
        .map(|l| l.trim())
        .filter(|l| l.starts_with("- run:"))
        .collect();
    assert_eq!(runs, [format!("- run: {RUN_LINE}")], "run の行は 1 つ");

    assert_eq!(
        ci.matches(VAR).count(),
        1,
        "ci.yml の中で {VAR} は 1 度だけ"
    );

    let main = read("xtask/src/main.rs");
    assert!(
        main.contains(&format!("const PARTITION_ENV: &str = \"{VAR}\";")),
        "xtask の main.rs の const PARTITION_ENV"
    );
    assert!(
        !read(".vessel.toml").contains(VAR),
        ".vessel.toml は {VAR} を持たない"
    );
}
