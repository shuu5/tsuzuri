//! 2 つの概要と本文の頭の 1 行の読みの歯（接頭辞 sumtw_・設計ノート surface-wave27b 行 c-sum-two・判断の記録 ADR-30 決定 (7)）。
//! 本文の字は歯の中に置く（2 つの概要の字は P と E で始まる違う字）。
#![cfg(test)]

use tsuzuri_contract::summary::{Summaries, excerpt, summaries};

fn two(plain: Option<&str>, eng: Option<&str>) -> Summaries {
    Summaries {
        plain: plain.map(str::to_string),
        eng: eng.map(str::to_string),
    }
}

#[test]
fn sumtw_typed_line_wins() {
    let body = "概要 = P-定型\n技術 = E-定型\n\n## 概要\nP-見出し\n\n## 技術\nE-見出し\n";
    assert_eq!(summaries(body), two(Some("P-定型"), Some("E-定型")));
    // 片方の定型行だけが在る本文は、もう片方を見出しから読む。
    let body = "概要 = P-定型\n\n## 概要\nP-見出し\n\n## 技術\nE-見出し\n";
    assert_eq!(summaries(body), two(Some("P-定型"), Some("E-見出し")));
    let body = "技術 = E-定型\n\n## 概要\nP-見出し\n\n## 技術\nE-見出し\n";
    assert_eq!(summaries(body), two(Some("P-見出し"), Some("E-定型")));
    // 定型行の値が空なら見出しの下の字。
    let body = "概要 = \n## 概要\nP-見出し\n";
    assert_eq!(summaries(body), two(Some("P-見出し"), None));
}

#[test]
fn sumtw_head_section_until_next_head() {
    let body = "題の行\n## 概要\n\nP-1 行目\n\nP-2 行目\n\n## 技術\nE-1 行目\n### 次の段\nX-段の字\n## 契約\nY\n";
    assert_eq!(
        summaries(body),
        two(Some("P-1 行目\n\nP-2 行目"), Some("E-1 行目"))
    );
    // 本文の末まで見出しが無ければ末まで。
    assert_eq!(
        summaries("## 技術\nE-1\nE-2\n").eng.as_deref(),
        Some("E-1\nE-2")
    );
    // 行の末の \r を除く。
    let crlf = "## 概要\r\n\r\nP-1\r\nP-2\r\n\r\n## 技術\r\nE-1\r\n";
    assert_eq!(summaries(crlf), two(Some("P-1\nP-2"), Some("E-1")));
    // 見出しの行の末の空白は見出しのまま。
    assert_eq!(summaries("## 概要  \nP-1\n").plain.as_deref(), Some("P-1"));
}

#[test]
fn sumtw_empty_is_none() {
    assert_eq!(summaries(""), Summaries::default());
    assert_eq!(summaries("本文だけ\n- 箇条\n"), Summaries::default());
    // 見出しの下が空の行だけで次の見出し。
    assert_eq!(
        summaries("## 概要\n\n  \n## 技術\nE-1\n"),
        two(None, Some("E-1"))
    );
    // 見出しの行の字が違う（段が違う・字が続く・行頭に空白）なら見出しでない。
    assert_eq!(summaries("### 概要\nP-1\n").plain, None);
    assert_eq!(summaries("## 概要の補足\nP-1\n").plain, None);
    assert_eq!(summaries(" ## 概要\nP-1\n").plain, None);
    assert_eq!(summaries("## 概要\nP-1\n").plain.as_deref(), Some("P-1"));
}

#[test]
fn sumtw_excerpt_keeps_whole_line() {
    let long = "長".repeat(150);
    let memo = format!("# 題\n### 観測\n\n概要 = {long}\n次の行\n");
    assert_eq!(excerpt(&memo), Some(long.clone()), "観測の下の行を切らない");
    let memo = format!("先の行\n### 観測\n{long}\n");
    assert_eq!(excerpt(&memo), Some(long.clone()), "観測の見出しが先");
    let typed = format!("\n# 題\n概要 =   {long}\n技術 = E\n");
    assert_eq!(excerpt(&typed), Some(long), "頭の字と空白を外す");
    let heads = "## 概要\nP-見出し\n## 技術\nE-見出し\n";
    assert_eq!(
        excerpt(heads).as_deref(),
        Some("P-見出し"),
        "見出しの行を飛ばす"
    );
    assert_eq!(excerpt("# 題\n\n## 概要\n"), None);
    assert_eq!(excerpt(""), None);
}
