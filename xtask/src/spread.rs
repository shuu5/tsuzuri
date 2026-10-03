//! 歯でない段の割りの振り分け（行 v-ci-split・判断の記録 ADR-34 の決定 (9)）。環境変数 TSUZURI_CHECK_PARTITION が count:k/N の時、
//! 段の鍵ごとの表 TABLE の番号の役だけがその段を撃ち（番号は 8 つに割る時の役・N が違えば N で回す）、番号 0 の段（歯と根の build）は
//! どの役でも撃つ。割りの無い撃ちは全部の段を撃つ。表に無い段の鍵は割りの在る無しに依らず断る（段を足す時は表にも足す）。

/// 段の鍵と撃つ役の番号（0 はどの役でも撃つ・1〜SLOTS は 8 つに割った時の役）。番号は CI の役ごとの歯の秒の軽い役へ重い段を振った。
pub const TABLE: &[(&str, u64)] = &[
    ("cargo build --workspace", 0),
    ("cargo nextest run --workspace", 0),
    ("cargo clippy --workspace --all-targets -- -D warnings", 2),
    (
        "cargo clippy -p tsuzuri-surface --target wasm32-unknown-unknown -- -D warnings",
        8,
    ),
    ("surface-build", 7),
    ("nested build --workspace", 1),
    ("nested nextest run --workspace", 0),
    ("nested clippy --workspace --all-targets -- -D warnings", 5),
    ("nested xtask check", 6),
];

/// 表の番号の数（CI の割りの数）。
pub const SLOTS: u64 = 8;

/// 面の組み立ての段の鍵。
pub const SURFACE: &str = "surface-build";

/// 段 1 つの振り分け。
#[derive(Debug, PartialEq, Eq)]
pub enum Turn {
    /// この撃ちで撃つ。
    Run,
    /// ほかの役が撃つ（撃つ役の番号と割りの数）。
    Elsewhere(u64, u64),
}

/// 段の鍵: 頭の字（cargo か nested）と引数を空白でつないだ字（--partition から後は除く）。
pub fn key(head: &str, args: &[String]) -> String {
    std::iter::once(head)
        .chain(
            args.iter()
                .map(String::as_str)
                .take_while(|a| *a != "--partition"),
        )
        .collect::<Vec<_>>()
        .join(" ")
}

/// 段を撃つか（鍵が表に無いか番号が SLOTS を越えるか割りの字を読めなければ Err）。番号 0 の段と割りの無い撃ちは撃つ。
pub fn turn(key: &str, part: Option<&str>) -> Result<Turn, String> {
    let at = TABLE
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, at)| *at)
        .ok_or_else(|| format!("段 {key} が振り分けの表に無い"))?;
    if at > SLOTS {
        return Err(format!("段 {key} の役の番号 {at} が {SLOTS} を越える"));
    }
    let Some(part) = part else {
        return Ok(Turn::Run);
    };
    let (k, n) = slot(part).ok_or_else(|| format!("割りの字を読めない: {part}"))?;
    if at == 0 {
        return Ok(Turn::Run);
    }
    let owner = (at - 1) % n + 1;
    Ok(if owner == k {
        Turn::Run
    } else {
        Turn::Elsewhere(owner, n)
    })
}

/// ほかの役が撃つ段の 1 行。
pub fn line(key: &str, owner: u64, n: u64) -> String {
    format!("段 {key} は割りの役 {owner}/{n} が撃つ")
}

/// 割りの字 count:k/N の k と N（1 ≤ k ≤ N でなければ None）。
fn slot(part: &str) -> Option<(u64, u64)> {
    let (k, n) = part.strip_prefix("count:")?.split_once('/')?;
    let (k, n): (u64, u64) = (k.parse().ok()?, n.parse().ok()?);
    (1..=n).contains(&k).then_some((k, n))
}
