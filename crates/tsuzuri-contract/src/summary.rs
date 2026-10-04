//! bead の 2 つの概要と本文の頭の 1 行の読み（判断の記録 ADR-30 決定 (2)(7)・要件 FR15・行 c-sum-two）。
//! 導出グラフ・問いの card・吹き出しが同じ関数で読む（中核と面の両方が使う純な関数・file も時計も触らない）。
//! 定型行（「概要 = 」「技術 = 」で始まる行）が在ればその行の残り、無ければ見出しの行（「## 概要」「## 技術」）の下の
//! 次の見出しまでの字を、その bead の概要とする（2 つの概要は別々に読む）。

/// 非エンジニア向けの概要の定型行の頭。
pub const PLAIN_LINE: &str = "概要 = ";

/// エンジニア向けの概要の定型行の頭。
pub const ENG_LINE: &str = "技術 = ";

/// 非エンジニア向けの概要の見出しの行。
pub const PLAIN_HEAD: &str = "## 概要";

/// エンジニア向けの概要の見出しの行。
pub const ENG_HEAD: &str = "## 技術";

/// 本文の頭の 1 行を取る見出しの行（その下の最初の空でない行を取る）。
pub const OBSERVATION_HEAD: &str = "### 観測";

/// 本文の頭の 1 行から外す頭の字。
pub const EXCERPT_LABEL: &str = "概要 =";

/// bead の 2 つの概要（無い方は None）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Summaries {
    pub plain: Option<String>,
    pub eng: Option<String>,
}

/// 本文の 2 つの概要（それぞれ定型行か、無ければ見出しの下の字）。
pub fn summaries(description: &str) -> Summaries {
    let one = |line: &str, head: &str| {
        typed_line(description, line).or_else(|| section(description, head))
    };
    Summaries {
        plain: one(PLAIN_LINE, PLAIN_HEAD),
        eng: one(ENG_LINE, ENG_HEAD),
    }
}

/// 定型行の値（行頭が `prefix` の最初の行の残り・前後の空白を除く・空なら None）。
pub fn typed_line(description: &str, prefix: &str) -> Option<String> {
    let rest = description
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .find_map(|l| l.strip_prefix(prefix))?
        .trim();
    (!rest.is_empty()).then(|| rest.to_string())
}

/// 見出しの行（末の空白を除いて `head` と同じ最初の行）の下から、次の字 # で始まる行の前までの字。
/// 行は str::lines の行（改行の \r\n も 1 つの改行）で、頭と末の空の行を除き、中の改行は残す。字が無ければ None。
pub fn section(description: &str, head: &str) -> Option<String> {
    let mut lines = description.lines();
    lines.find(|l| l.trim_end() == head)?;
    let body: Vec<&str> = lines.take_while(|l| !l.starts_with('#')).collect();
    let first = body.iter().position(|l| !l.trim().is_empty())?;
    let last = body.iter().rposition(|l| !l.trim().is_empty())?;
    Some(body.get(first..=last)?.join("\n"))
}

/// 本文の頭の 1 行（切らない）。見出しの行 `OBSERVATION_HEAD` が在ればその下の最初の空でない行、無ければ空の行と
/// 字 # で始まる行を飛ばした最初の行（どれも前後の空白を除く）から、頭の `EXCERPT_LABEL` とその後の空白を外した字。
/// 字が無ければ None。
pub fn excerpt(description: &str) -> Option<String> {
    let lines: Vec<&str> = description.lines().map(str::trim).collect();
    let line = match lines.iter().position(|l| *l == OBSERVATION_HEAD) {
        Some(at) => lines
            .get(at + 1..)
            .unwrap_or_default()
            .iter()
            .find(|l| !l.is_empty()),
        None => lines.iter().find(|l| !l.is_empty() && !l.starts_with('#')),
    };
    let line = line.copied().unwrap_or_default();
    let text = line
        .strip_prefix(EXCERPT_LABEL)
        .map_or(line, str::trim_start);
    (!text.is_empty()).then(|| text.to_string())
}
