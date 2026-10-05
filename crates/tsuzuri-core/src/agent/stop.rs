//! 係の終える前の門の判じ（行 ag-stop・判断の記録 ADR-59 決定 (4)・要件 FR21）。
//! 係の終わりの門の入力（SubagentStop の入力）の欄を読み、係の記録と出力の dir と最後の文から欠けを数える。
//! 欠けは 3 つで、この順に並べる: 係の記録に席（team-lead）への SendMessage の呼びが無い・札の出す物の file が出力の dir に無い・
//! 最後の文に出力の dir の path が無い。止めるのは 1 度目の終わり（stop_hook_active が真でない終わり）だけで、2 度目は欠けを記帳して通す。

use serde_json::Value;

/// 係の出力の dir の名（係の dir の下）。
pub const OUT: &str = "w";

/// 席の名（名で呼び合う形の SendMessage の宛先）。
pub const LEAD: &str = "team-lead";

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

/// 係の記録の 1 行が、席への SendMessage の呼び（message の content の項で、名が SendMessage・input の to が席の名）を持つか。
fn tells(line: &[u8]) -> bool {
    let Ok(row) = serde_json::from_slice::<Value>(line) else {
        return false;
    };
    row.pointer("/message/content")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items.iter().any(|x| {
                x.get("name").and_then(Value::as_str) == Some("SendMessage")
                    && x.pointer("/input/to").and_then(Value::as_str) == Some(LEAD)
            })
        })
}

/// 欠けの字の列（席への知らせ・出す物の file ごと・最後の文の path の順）。`have` は出力の dir `out` の下に出す物の file が在るか。
pub fn lacks(
    record: &[u8],
    outputs: &[String],
    have: impl Fn(&str) -> bool,
    last: &str,
    out: &str,
) -> Vec<String> {
    let mut lacks = Vec::new();
    if !record.split(|b| *b == b'\n').any(tells) {
        lacks.push(format!(
            "係の記録に席（{LEAD}）への SendMessage の知らせが無い"
        ));
    }
    lacks.extend(
        outputs
            .iter()
            .filter(|o| !have(o))
            .map(|o| format!("出す物 {o} が {out}/ に無い")),
    );
    if !last.contains(out) {
        lacks.push(format!("最後の文に出力の dir の path {out} が無い"));
    }
    lacks
}

/// 1 度目の終わりを止める理由の字（欠けと次の一手）。
pub fn hold(lacks: &[String], out: &str) -> String {
    format!(
        "係の終える前の門は止める（{}） 次の一手 = 欠けを埋め、最後の文に {out} を書いて終える（止めるのは 1 度目の終わりだけ）",
        lacks.join("・")
    )
}
