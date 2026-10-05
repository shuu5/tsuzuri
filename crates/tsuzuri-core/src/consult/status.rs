//! 窓の状態の 1 行の字（判断の記録 ADR-57 決定 (1)(コ)・(4)）。
//! 起動の口の設定の statusLine の命令の字と、Claude Code が命令の標準入力に渡す session の JSON からの文脈の割合
//! （鍵 `context_window.used_percentage`・無いか数でないか範囲の外なら出さない・ほかの鍵は読み飛ばす）と、窓の控え・所見の数・
//! 束の要約値から組む 1 行の字を持つ。材料は窓が書ける作業場の字なので、どの欄も制御の字を落として長さを切る。

use serde_json::Value;
use tsuzuri_contract::consult::WindowFile;

/// 状態の 1 行の口の名（席の口でないので窓の守りは通す）。
pub const VERB: &str = "statusline";

/// 欄の区切り。
pub const SEP: &str = " ｜ ";

/// 束の要約値の頭の字の数。
pub const DIGEST_HEAD: usize = 8;

/// 題・model・念入りさの欄の字の数の上限。
pub const FIELD_MAX: usize = 40;

/// 状態の 1 行の命令の字（作業場の絶対 path を渡す・時間切れで終わる・起動の口の settings と audit が同じ字を持つ）。
pub fn command(tz: &str, workspace: &str) -> String {
    format!("timeout 2 {tz} consult {VERB} {workspace}")
}

/// 字から制御の字（`char::is_control` の C0・DEL・C1 と、向きの上書きの U+202A〜U+202E・U+2066〜U+2069）を落とし、
/// 頭の `max` 字に切る（端末を操る字を持ち主の画面に出さない）。
pub fn clean(text: &str, max: usize) -> String {
    text.chars()
        .filter(|c| {
            !c.is_control() && !matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .take(max)
        .collect()
}

/// Claude Code が渡す session の JSON から文脈の割合（0〜100 の整数・四捨五入）。鍵が無い・数でない・範囲の外は None。
pub fn context_percent(text: &str) -> Option<u8> {
    let value: Value = serde_json::from_str(text).ok()?;
    let percent = value.pointer("/context_window/used_percentage")?.as_f64()?;
    if !(0.0..=100.0).contains(&percent) {
        return None;
    }
    (0..=100u8).find(|n| f64::from(*n) >= percent.round())
}

/// 1 行の字（窓・題・model と念入りさ・所見の数・束の要約値の頭・文脈の割合。無い欄は出さない・字の欄は `clean` を通す）。
pub fn render(
    window: Option<&WindowFile>,
    findings: Option<usize>,
    digest: Option<&str>,
    percent: Option<u8>,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    match window {
        Some(w) => {
            let topic = w
                .topic
                .as_deref()
                .map_or("なし".to_string(), |t| clean(t, FIELD_MAX));
            parts.push(format!("窓 {}", w.id));
            parts.push(format!("題 {topic}"));
            parts.push(format!(
                "{}・{}",
                clean(&w.model, FIELD_MAX),
                clean(&w.effort, FIELD_MAX)
            ));
        }
        None => parts.push("窓 ?".to_string()),
    }
    if let Some(n) = findings {
        parts.push(format!("所見 {n}"));
    }
    let digest = digest.map(|d| clean(d.trim(), DIGEST_HEAD));
    if let Some(d) = digest.filter(|d| !d.is_empty()) {
        parts.push(format!("束 {d}"));
    }
    if let Some(p) = percent {
        parts.push(format!("文脈 {p}%"));
    }
    parts.join(SEP)
}
