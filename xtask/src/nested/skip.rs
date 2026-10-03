//! 入れ子の段を省くかの判じ（行 v-skip・判断の記録 ADR-34）。入れ子の dir ごとに、HEAD と BASE_REF の merge-base（base）から
//! 作業木までの差（index と作業木の直し・改名は消すと足すに割る）と未追跡の file（.gitignore の外）が、入れ子の入力
//! （`<dir>/` と SHARED）に 1 つも触れなければ省く。根が git の木の根でない時・base を読めない時・base から HEAD までに
//! 自分の commit が無い時・環境変数 FORCE_ENV が在る時は全部を撃つ。器 scribe2 の crate を根の workspace に入れる行（v-join）までの手当て。

use std::path::Path;
use std::process::{Command, Output};

/// 入れ子の段に全部を撃たせる環境変数（在れば値を問わない・CI の job check が置く）。省かせる向きの口は置かない。
pub const FORCE_ENV: &str = "TSUZURI_CHECK_NESTED_ALL";

/// base を取る相手（送り先の main）。local の main は器の land の確かめの木で squash の後に進むので使わない。
pub const BASE_REF: &str = "origin/main";

/// 入れ子の dir の外で、入れ子の段の結果を替えうる根の path（toolchain の pin・cargo の設定・段そのもの）。
pub const SHARED: &[&str] = &[
    "rust-toolchain",
    "rust-toolchain.toml",
    ".cargo/",
    "xtask/src/",
    "xtask/Cargo.toml",
];

/// 入れ子の dir 1 つの判じ。
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    /// 省く（base の commit の sha）。
    Skip(String),
    /// 全部を撃つ（理由の字）。
    Full(String),
}

/// 入れ子の dir の入力の path（`<dir>/` と SHARED の順）。
pub fn inputs(dir: &str) -> Vec<String> {
    std::iter::once(format!("{dir}/"))
        .chain(SHARED.iter().map(|p| (*p).to_string()))
        .collect()
}

/// 根の直下の入れ子の dir 1 つを判じる（forced なら git を読まずに全部を撃つ）。
pub fn decide(root: &Path, dir: &str, forced: bool) -> Verdict {
    if forced {
        return Verdict::Full(format!("{FORCE_ENV} が在る"));
    }
    let base = match base_of(root) {
        Ok(base) => base,
        Err(why) => return Verdict::Full(why),
    };
    match touched(root, &base, &inputs(dir)) {
        Ok(false) => Verdict::Skip(base),
        Ok(true) => Verdict::Full(format!(
            "base {} からの差が入れ子の入力に触れる",
            short(&base)
        )),
        Err(why) => Verdict::Full(why),
    }
}

/// 段を撃つ dir と出す行: dir の順に judge を 1 度ずつ呼んで行を 1 つずつ作り、全部を撃つ dir だけを順に残す。
pub fn select(dirs: &[String], judge: impl Fn(&str) -> Verdict) -> (Vec<String>, Vec<String>) {
    let mut keep = Vec::new();
    let mut lines = Vec::new();
    for dir in dirs {
        let verdict = judge(dir);
        lines.push(line(dir, &verdict));
        if matches!(verdict, Verdict::Full(_)) {
            keep.push(dir.clone());
        }
    }
    (keep, lines)
}

/// 判じの 1 行（字 nested と dir の名に続けて、省いた時は base の頭 12 字・全部を撃つ時は理由）。
pub fn line(dir: &str, verdict: &Verdict) -> String {
    match verdict {
        Verdict::Skip(base) => format!(
            "nested {dir}: 省いた（base {} からの差が入れ子の入力に触れない）",
            short(base)
        ),
        Verdict::Full(why) => format!("nested {dir}: 全部を撃つ（{why}）"),
    }
}

/// sha の頭 12 字（短い字はそのまま）。
fn short(sha: &str) -> &str {
    sha.get(..12).unwrap_or(sha)
}

/// 根で git を撃つ（path の絞りは字のまま読ませる）。
fn git(root: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("--literal-pathspecs")
        .args(args)
        .output()
        .map_err(|e| format!("git を起動できない: {e}"))
}

/// git の標準出力の字（前後の空白を除く・rc が 0 でなければ Err）。
fn text(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = git(root, args)?;
    if !out.status.success() {
        return Err(format!(
            "git {} が rc {:?}",
            args.join(" "),
            out.status.code()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// base（HEAD と BASE_REF の merge-base）: 根が git の木の根で、base から HEAD までに自分の commit が 1 つ以上在る時だけ読める。
fn base_of(root: &Path) -> Result<String, String> {
    let top = text(root, &["rev-parse", "--show-toplevel"])
        .map_err(|_| "根が git の木でない".to_string())?;
    match (Path::new(&top).canonicalize(), root.canonicalize()) {
        (Ok(a), Ok(b)) if a == b => {}
        _ => return Err("根が git の木の根でない".to_string()),
    }
    let base = text(root, &["merge-base", "HEAD", BASE_REF])
        .map_err(|_| format!("base を読めない（merge-base HEAD {BASE_REF}）"))?;
    let own = text(root, &["rev-list", "--count", &format!("{base}..HEAD")])?;
    if own == "0" {
        return Err(format!(
            "base {} から HEAD までに自分の commit が無い",
            short(&base)
        ));
    }
    Ok(base)
}

/// base から作業木までの差が path に在るか（diff の rc 1）、path の下に未追跡の file（.gitignore の外）が在るか。
fn touched(root: &Path, base: &str, paths: &[String]) -> Result<bool, String> {
    let mut diff = vec!["diff", "--quiet", "--no-renames", base, "--"];
    diff.extend(paths.iter().map(String::as_str));
    match git(root, &diff)?.status.code() {
        Some(0) => {}
        Some(1) => return Ok(true),
        code => return Err(format!("git diff が rc {code:?}")),
    }
    let mut others = vec!["ls-files", "--others", "--exclude-standard", "-z", "--"];
    others.extend(paths.iter().map(String::as_str));
    Ok(!text(root, &others)?.is_empty())
}
