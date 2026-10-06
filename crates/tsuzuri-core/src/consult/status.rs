//! 窓の状態の 1 行の字（判断の記録 ADR-57 決定 (1)(コ)・(4)）。
//! 起動の口の設定の statusLine の命令の字と、Claude Code が命令の標準入力に渡す session の JSON からの文脈の割合
//! （鍵 `context_window.used_percentage`・無いか数でないか範囲の外なら出さない・ほかの鍵は読み飛ばす）と、窓の控え・所見の数・
//! 束の要約値から組む 1 行の字を持つ。材料は窓が書ける作業場の字なので、どの欄も制御の字を落として長さを切る。
//! 窓を起こした口座の設定の statusLine の命令の字と再描画の間を設定の字から読み、命令の出力を行ごとに色の制御の並びだけを通して
//! ほかの制御の並びと制御の字を落として切り、その行の後に窓の 1 行を置く（出せなかった周は窓の 1 行の末に訳の語を添える・
//! 判断の記録 ADR-67）。

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
    text.chars().filter(|c| shown(*c)).take(max).collect()
}

/// 端末に出してよい字か（制御の字と向きの上書きの字でない）。
fn shown(c: char) -> bool {
    !c.is_control() && !matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
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

/// 口座の命令の出力から出す行の数の上限。
pub const ACCOUNT_LINES: usize = 8;

/// 口座の命令の出力の 1 行の字の数の上限。
pub const ACCOUNT_LINE_MAX: usize = 400;

/// 口座の命令の行を出せなかった訳（窓の 1 行の末の欄 `口座 <語>` の語）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Miss {
    /// 口座の置き場が無いか、設定 file が読めないか JSON でない。
    Unread,
    /// 設定に statusLine の命令の字が無い。
    NoKey,
    /// 命令を起こせないか、rc が 0 でない。
    Failed,
    /// 命令が上限の内に終わらない。
    TimedOut,
    /// 口座の命令の中から撃たれた（入れ子では撃たない）。
    Nested,
}

impl Miss {
    /// 訳の語。
    pub fn word(self) -> &'static str {
        match self {
            Miss::Unread => "読めない",
            Miss::NoKey => "鍵なし",
            Miss::Failed => "落ちた",
            Miss::TimedOut => "時間切れ",
            Miss::Nested => "入れ子",
        }
    }
}

/// 口座の設定 file の字から statusLine の命令の字（鍵 `statusLine.command` の空でない字・型は見ない）。
/// JSON でなければ `Miss::Unread`、鍵が無いか字でないか空なら `Miss::NoKey`。
pub fn account_command(settings: &str) -> Result<String, Miss> {
    let value: Value = serde_json::from_str(settings).map_err(|_| Miss::Unread)?;
    value
        .pointer("/statusLine/command")
        .and_then(Value::as_str)
        .filter(|c| !c.trim().is_empty())
        .map(String::from)
        .ok_or(Miss::NoKey)
}

/// 口座の設定 file の字から statusLine の再描画の間（鍵 `statusLine.refreshInterval` の 0 以上の整数・無い・読めない・整数でなければ None）。
pub fn account_refresh(settings: &str) -> Option<u64> {
    let value: Value = serde_json::from_str(settings).ok()?;
    value.pointer("/statusLine/refreshInterval")?.as_u64()
}

/// 色の戻しの並び（色を通した口座の行の末に足す）。
pub const SGR_RESET: &str = "\u{1b}[0m";

/// 口座の命令の出力の 1 行から、色の制御の並び（SGR・ESC [ の後が数と ; だけで m で終わる）だけを通し、ほかの ESC の並び
/// （m 以外で終わるか数と ; でない字を持つ ESC [ の並びは終わりの字 @〜~ まで・ESC ] と ESC P の並びは BEL か ESC \ まで・
/// ほかの ESC は次の 1 字まで）を丸ごと落とし、制御の字と向きの上書きの字（`clean` と同じ）を落として、見える字を頭の `max` 字に
/// 切る。色を通した行は末に `SGR_RESET` を足す（次の行に色を持ち越さない）。
pub fn keep_colors(line: &str, max: usize) -> String {
    let mut out = String::new();
    let (mut seen, mut colored) = (0, false);
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            if shown(c) && seen < max {
                out.push(c);
                seen += 1;
            }
            continue;
        }
        match chars.next() {
            Some('[') => {
                let mut params = String::new();
                let end = chars.by_ref().find(|c| {
                    let done = ('@'..='~').contains(c);
                    if !done {
                        params.push(*c);
                    }
                    done
                });
                let sgr = params.chars().all(|p| p.is_ascii_digit() || p == ';');
                if end == Some('m') && sgr && seen < max {
                    out.push_str(&format!("\u{1b}[{params}m"));
                    colored = true;
                }
            }
            Some(']' | 'P') => {
                let mut esc = false;
                for c in chars.by_ref() {
                    if c == '\u{7}' || (esc && c == '\\') {
                        break;
                    }
                    esc = c == '\u{1b}';
                }
            }
            _ => {}
        }
    }
    if colored {
        out.push_str(SGR_RESET);
    }
    out
}

/// 口座の命令の出力を行に割り、頭の `ACCOUNT_LINES` 行の各々を `keep_colors` で `ACCOUNT_LINE_MAX` 字に切る（末の空の行は除く）。
pub fn account_lines(out: &str) -> Vec<String> {
    let mut lines: Vec<String> = out
        .lines()
        .take(ACCOUNT_LINES)
        .map(|l| keep_colors(l, ACCOUNT_LINE_MAX))
        .collect();
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

/// 状態の行の全体（口座の命令の行の後に窓の 1 行・出せなかった周は窓の 1 行の末に欄 `口座 <訳の語>` を足した 1 行だけ）。
pub fn stanza(account: Result<Vec<String>, Miss>, window: &str) -> String {
    match account {
        Ok(mut lines) => {
            lines.push(window.to_string());
            lines.join("\n")
        }
        Err(miss) => format!("{window}{SEP}口座 {}", miss.word()),
    }
}
