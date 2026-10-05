//! 行 r-nextest-timeout の歯: nextest の設定 .config/nextest.toml の歯の時間切れ（判断の記録 ADR-31 の決定 (2)）。
//! 注と空の行を除いた行は、表 profile.default の見出しと slow-timeout の 1 行（60 秒ごとに SLOW・5 回目で止める）の 2 行だけ。
#![cfg(test)]

use crate::common::read_root;

/// 井桁で始まる行と空の行を除いた行（trim）。
fn body_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

#[test]
fn ntmo_slow_timeout_is_sixty_seconds_five_times() {
    let text = read_root(".config/nextest.toml");
    assert_eq!(
        body_lines(&text),
        [
            "[profile.default]",
            r#"slow-timeout = { period = "60s", terminate-after = 5 }"#,
        ]
    );
}
