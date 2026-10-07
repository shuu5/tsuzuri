//! 係の終える前の門の判じ（判断の記録 ADR-59 決定 (4)・要件 FR21）。
//! 係の終わりの門の入力（SubagentStop の入力）の欄を読み、係の記録と出力の dir と最後の文から欠けを数える。
//! 欠けは 5 つで、この順に並べる: 札の出す物の file が出力の dir に無い・最後の答えの 1 行目が状態の語でない・
//! 最後の答えの行の数が上限を越える・最後の答えの字の数が上限を越える・最後の答えの最後の行に出力の dir の path が無い。
//! 最後の答えは、係の記録に引き渡しの道具（SubagentHandback）の呼びが在ればその message の字、無ければ最後の文である（判断の記録 ADR-78 決定 (1)）。
//! 出す物の .md の頭の見出しの欠けは `gist_lacks` が持つ。
//! 止めるのは 1 度目の終わり（stop_hook_active が真でない終わり）だけで、2 度目は欠けを記帳して通す。
//! 群の係の出す物の主張の表の欠けは `claim_lacks` が持つ（判断の記録 ADR-61 決定 (4)(8)・要件 FR22）。

use serde_json::Value;

/// 最後の答えの 1 行目に置く状態の語。
pub const STATES: [&str; 4] = ["DONE", "DONE_WITH_CONCERNS", "BLOCKED", "NEEDS_CONTEXT"];

/// 最後の答えの行の数の上限（規則の行 R-51）。
pub const MAX_LINES: usize = 5;

/// 最後の答えの字の数の上限（規則の行 R-51）。
pub const MAX_CHARS: usize = 600;

/// 出す物の .md の頭に置く見出しの語。
pub const HEADING: &str = "要点";

/// 見出しを探す出す物の頭の行の数。
pub const HEAD_LINES: usize = 40;

/// 係の引き渡しの道具の名（係の記録の content の項の名）。
pub const HANDBACK: &str = "SubagentHandback";

/// 2 度目の終わりを通した時の欠けの記帳の file（出力の dir の下）。
pub const GATE: &str = "STOP-GATE.txt";

/// 終わりの門の入力の欄（係の記録の path・最後の文・止めた後の終わりか・無い欄は空の字と偽）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct End {
    pub record: String,
    pub last: String,
    pub again: bool,
}

/// 終わりの門の入力の欄を読む（JSON でなければ全部空）。
pub fn end(payload: &str) -> End {
    let input: Value = serde_json::from_str(payload).unwrap_or_default();
    let text = |k: &str| {
        input
            .get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    End {
        record: text("agent_transcript_path"),
        last: text("last_assistant_message"),
        again: input.get("stop_hook_active") == Some(&Value::Bool(true)),
    }
}

/// 係の記録の最後の引き渡しの呼び（message の content の項で、名が HANDBACK・input の message が字）の message の字（無ければ None）。
fn handback(record: &[u8]) -> Option<String> {
    record
        .split(|b| *b == b'\n')
        .filter_map(|line| serde_json::from_slice::<Value>(line).ok())
        .filter_map(|row| {
            row.pointer("/message/content")?
                .as_array()?
                .iter()
                .rev()
                .find(|x| x.get("name").and_then(Value::as_str) == Some(HANDBACK))?
                .pointer("/input/message")?
                .as_str()
                .map(str::to_string)
        })
        .next_back()
}

/// 欠けの字の列（出す物の file ごと・最後の答えの状態の語・行の数・字の数・最後の行の path の順）。`have` は出力の dir `out` の下に出す物の file が在るか。
/// 最後の答えは、係の記録 `record` に引き渡しの呼びが在ればその message の字、無ければ最後の文 `last` で、前後の空白を除いて数える。
pub fn lacks(
    record: &[u8],
    outputs: &[String],
    have: impl Fn(&str) -> bool,
    last: &str,
    out: &str,
) -> Vec<String> {
    let mut lacks: Vec<String> = outputs
        .iter()
        .filter(|o| !have(o))
        .map(|o| format!("出す物 {o} が {out}/ に無い"))
        .collect();
    let handed = handback(record);
    let answer = handed.as_deref().unwrap_or(last).trim();
    if !STATES.contains(&answer.lines().next().unwrap_or_default().trim()) {
        lacks.push(format!(
            "最後の答えの 1 行目が状態の語（{} のどれか）でない",
            STATES.join("・")
        ));
    }
    let lines = answer.lines().count();
    if lines > MAX_LINES {
        lacks.push(format!(
            "最後の答えが {lines} 行で上限 {MAX_LINES} 行を越える"
        ));
    }
    let chars = answer.chars().count();
    if chars > MAX_CHARS {
        lacks.push(format!(
            "最後の答えが {chars} 字で上限 {MAX_CHARS} 字を越える"
        ));
    }
    if !answer.lines().next_back().unwrap_or_default().contains(out) {
        lacks.push(format!(
            "最後の答えの最後の行に出力の dir の path {out} が無い"
        ));
    }
    lacks
}

/// 出す物 `outputs` のうち名の末が .md で `read` が字を返す file ごとの欠けの字の列（頭 HEAD_LINES 行に、前の空白を除いて字 # で始まり
/// HEADING を含む行が無い）。.md でない出す物と読めない出す物は数えない。
pub fn gist_lacks(outputs: &[String], read: impl Fn(&str) -> Option<String>) -> Vec<String> {
    outputs
        .iter()
        .filter(|o| o.ends_with(".md"))
        .filter_map(|o| read(o).map(|text| (o, text)))
        .filter(|(_, text)| {
            !text.lines().take(HEAD_LINES).any(|l| {
                let l = l.trim_start();
                l.starts_with('#') && l.contains(HEADING)
            })
        })
        .map(|(o, _)| format!("出す物 {o} の頭 {HEAD_LINES} 行に「{HEADING}」を含む見出しが無い"))
        .collect()
}

/// 群の係の主張の表の確かさの印（確かめた・記録から・見立て・分からない）。
pub const CERTAINTY: [&str; 4] = ["V", "D", "I", "U"];

/// 表の行の欄（`|` で始まる行の頭と末の `|` を除き、`\|` でない `|` で割って空白を除く・`|` で始まらない行は None）。
fn cells(line: &str) -> Option<Vec<String>> {
    let body = line.trim().strip_prefix('|')?;
    let body = body
        .strip_suffix('|')
        .unwrap_or(body)
        .replace("\\|", "\u{1}");
    Some(
        body.split('|')
            .map(|c| c.trim().replace('\u{1}', "|"))
            .collect(),
    )
}

/// 群の係の出す物の字 `texts` の主張の表の欠けの字の列（割りの主張 `claims` の順に、表の行が無い・行に確かさの印が無い・行に証拠が無い）。
/// 主張の表の行は頭の欄が割りの主張の id の行で、欄は id・主張・確かさの印・証拠の path か命令の順（5 つ目からの欄は見ない）。
pub fn claim_lacks(texts: &[String], claims: &[String]) -> Vec<String> {
    let rows: Vec<Vec<String>> = texts
        .iter()
        .flat_map(|t| t.lines())
        .filter_map(cells)
        .collect();
    let mut lacks = Vec::new();
    for id in claims {
        let mine: Vec<&Vec<String>> = rows.iter().filter(|r| r.first() == Some(id)).collect();
        if mine.is_empty() {
            lacks.push(format!("割りの主張 {id} の表の行が出す物に無い"));
        }
        for row in mine {
            if !row.get(2).is_some_and(|m| CERTAINTY.contains(&m.as_str())) {
                lacks.push(format!("主張 {id} の行に確かさの印（V・D・I・U）が無い"));
            }
            if row.get(3).is_none_or(String::is_empty) {
                lacks.push(format!("主張 {id} の行に証拠の path か命令が無い"));
            }
        }
    }
    lacks
}

/// 1 度目の終わりを止める理由の字（欠けと次の一手）。
pub fn hold(lacks: &[String], out: &str) -> String {
    format!(
        "係の終える前の門は止める（{}） 次の一手 = 欠けを埋め、最後の答えを 1 行目が状態の語・{MAX_LINES} 行以内・{MAX_CHARS} 字以内・最後の行が {out} の形にして終える（止めるのは 1 度目の終わりだけ）",
        lacks.join("・")
    )
}
