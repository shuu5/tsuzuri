//! `folio init --dir <置き場>`（便 125・docs/design/delivery-125.md §1・判断の記録 ADR-16 決定 (3)・(2)(オ)(カ)・FR22）。
//! 新しい置き場に最初の文書一式（骨格）11 本を新規作成でだけ書く。断りの名が 1 つでも在れば 1 byte も書かずに 1。
//! 生成区間は空の印で書いてから `schema::run`（Write）を同じ呼び出しの中で呼んで埋める（`folio schema --write` と同じ導出）。
//! 人が書く部分は雛形の定数と、組み立て時に焼いた folio2 の正本 5 本から読む字（憲法の schema の節と行 R-16 は型付きで
//! 読み書きし、入口・天井・相談窓口の節は行で写して最小の欄に絞り、folio2 固有の id を持つ全角の括弧を落とす）。
//! 日付の欄は撃った日の UTC。注入の対象にしない。置き場の外には何も書かない。書き始めた後に書けなくなったら止めて 2（消さない）。

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::schema;
use crate::verdict::Verdict;
use crate::yaml::{self, Value};

/// 書く file の閉じた一覧（置き場からの相対・この順に書く）。
pub const FILES: [&str; 11] = [
    "constitution.yaml",
    "rules.yaml",
    "vocabulary.yaml",
    "srs.yaml",
    "index.yaml",
    "intake.yaml",
    "ceiling.yaml",
    "graph.yaml",
    "adr/schema.yaml",
    "adr/ADR-1.yaml",
    "design-note/schema.yaml",
];

/// 断りの名の閉じた一覧（どれか 1 つでも在れば何も書かない・壊れた symlink も在ると数える）。
pub const REFUSE: [&str; 11] = [
    "constitution.yaml",
    "rules.yaml",
    "vocabulary.yaml",
    "srs.yaml",
    "index.yaml",
    "intake.yaml",
    "ceiling.yaml",
    "graph.yaml",
    "adr",
    "design-note",
    "anchors",
];

/// 骨格の中に作る dir（置き場の直下）。
const DIRS: [&str; 2] = ["adr", "design-note"];

// ── 組み立て時に焼く folio2 の正本（実行時に folio2 の repo を読まない）──
const SRC_CONSTITUTION: &str = include_str!("../../../design-intent/constitution.yaml");
const SRC_RULES: &str = include_str!("../../../design-intent/rules.yaml");
const SRC_INDEX: &str = include_str!("../../../design-intent/index.yaml");
const SRC_CEILING: &str = include_str!("../../../design-intent/ceiling.yaml");
const SRC_INTAKE: &str = include_str!("../../../design-intent/intake.yaml");

/// 骨格自身の番号（写した字の括弧を落とす式で、folio2 固有の id に数えない）。
const OWN_IDS: [&str; 5] = ["ADR-1", "P-1", "P-1.1", "R-8", "R-16"];

/// 日付の字の置き場（雛形の中の印・撃った日の UTC の年-月-日に替える）。
const DATE: &str = "{DATE}";

// ── 雛形（人が書く部分・利用者が本来の中身に書き換える）──

const CONSTITUTION_HEAD: &str = "# 憲法 — 床の受付の宣言（folio init の雛形・名と本文は利用者が決めて書き換える）";

const CONSTITUTION_BODY: &str = r#"
meta:
  id: floor-declaration
  version: v1.0
  status: draft
  binding: false
  baseline: 未記入
  generated: {DATE}
  owner: 持ち主
  author: 未記入
  counts: {always: 1, ask-first: 0, never: 0}
  approval: {who: 未記入, date: 未記入, ruling: 未記入, verbatim: 未記入, surface: 規則の表の行 R-8 の対話面}

north_star:
  statement: 未記入
  for_whom: 未記入
  judged_by: 未記入

precedence:
  text: 条どうしの順位は、利用者の本来の憲法の順位に従う。
  plain: 条どうしが食い違ったときは、あなたの本来の憲法が決めた順位で解きます。
  binds: both
  mechanism: {kind: none, live: now, note: 判断の規則であって機械では検査できない。}
  rationale:
    - {kind: scribe2-article, ref: 本来の憲法の順位の節（番号は利用者が書く）}

articles:
  - id: P-1
    title: 承認の受け方と散文の門の値
    tier: always
    binds: both
    statements:
      - {id: P-1.1, pattern: ubiquitous, strength: must, text: 本来の憲法の条（番号は利用者が書く）に従い、承認は規則の表の行 R-8 の対話面を通ったものだけを受け取り、散文の門は行 R-16 の値で数える。}
    plain: 承認は決まった対話面を通ったものだけを受け取り、設計ノートの散文の門は規則の表の値で数えます。どちらも、あなたの本来の憲法の条に従います。
    rationale:
      - {kind: scribe2-article, ref: 本来の憲法の条（番号は利用者が書く）}
    mechanism: {kind: build-check, live: now, stage: post, polarity: fail-closed, note: 承認欄と設計ノートの散文を、床が行 R-8 と行 R-16 の値で数える。}
    relations: {rules: [R-8, R-16]}

rules_pointer: 数値の決まりは規則の表にまとめ、条文は数値を書かずに行の番号で呼ぶ。
amendment: 未記入
glossary_pointer: この章は、この文書に出てくる言葉の意味をまとめた一覧です。語と定義の元は語彙の file 1 つに置きます。
sources: []
"#;

const RULES_HEAD: &str = "# 規則の表 — 数値と開発規律の正本（folio init の雛形・行は利用者が書き換える）";
const RULES_ROWS_HEAD: &str = "# 行は人が書く（閾値行と開発規律行）";

const VOCABULARY: &str = r#"# 語彙 — 正本（folio init の雛形・語は利用者が足す）
terms: []
field_terms: []
identifiers:
  - {group: 骨格の雛形の語, words: [folio, ai, intake, rules, file, id, schema, repo], why: 骨格の雛形に出る識別子（命令名・file 名・code の語）}
"#;

const SRS: &str = r#"# 要件書 — 正本（folio init の雛形・要件は利用者が書く）
meta:
  id: srs
  title: 要件書
  version: v0.1
  status: draft
  generated: {DATE}
  approval: []
"#;

const INDEX_HEAD: &str = r#"# 入口 — 正本（folio init の雛形・案内文は利用者が書き換える）
meta:
  id: index
  title: 設計文書の入口
  version: v0.1
  status: draft
  generated: {DATE}
  approval: []
"#;

const INTAKE_HEAD: &str = r#"# 相談窓口 — 正本（folio init の雛形・質問は利用者が書き換える）
meta:
  id: intake
  title: 相談窓口
  version: v0.1
  status: draft
  generated: {DATE}
  approval: []
"#;

const CEILING_HEAD: &str = r#"# 天井の正本 — 観点と読む文書（folio init の雛形・利用者が書き換える）
meta:
  id: ceiling
  title: 天井の正本
  version: v0.1
  status: draft
  generated: {DATE}
  approval: []
"#;

const GRAPH: &str = r#"# 索引の欄の決まり — 正本（folio init の雛形）
meta:
  id: graph
  title: 索引の欄の決まり
  version: v0.1
  status: draft
  generated: {DATE}
  approval: []
"#;

const ADR_SCHEMA: &str = r#"# 判断の記録 — 欄の決まり（folio init の雛形・生成区間は folio schema --write が書く）
meta: {id: adr-schema, version: 1, date: {DATE}, decided_by: [ADR-1], owner: 持ち主, author: 未記入}
"#;

const NOTE_SCHEMA: &str = r#"# 設計ノート — 欄の決まり（folio init の雛形・生成区間は folio schema --write が書く）
meta: {id: design-note-schema, version: 1, date: {DATE}, decided_by: [ADR-1], owner: 持ち主, author: 未記入}
"#;

const ADR_1: &str = r#"# 判断の記録 ADR-1 — 最初の判断の記録（folio init の雛形・提案中・発効は持ち主の承認）
id: ADR-1
title: 判断の記録と設計ノートの欄の決まりに folio の床の定数を採る
status: proposed
date: {DATE}
context: 判断の記録と設計ノートは、欄と値域を決めた型付きの file で書く。欄の決まりを人が別に書くと、床の定数と食い違ったときにどちらを正とするかが決まらない。
decision: 欄の決まりは folio の床の定数の写し（各置き場の欄の決まりの file の生成区間）を採る。生成区間は手で直さない。
options:
  - {id: a, name: 採る, text: 欄の決まりに folio の床の定数の写しを採る。, verdict: adopted, reason: 床が写しとの 1 字の違いを落とすため、欄の決まりが 1 か所に定まる。}
  - {id: b, name: 採らない, text: 欄の決まりを利用者が別に書く。, verdict: rejected, reason: 床が写しとの 1 字の違いを落とすため、別に書いた欄の決まりは床を通らない。}
basis: [P-1]
retreat: {kind: ruling, condition: 未記入}
plain: 判断の記録と設計ノートの書き方の決まりは、道具の中の決まりをそのまま写して使います。写しは道具が書き、手で直しません。この記録はまだ提案中で、あなたが承認したときに発効します。
"#;

/// 行 R-8 の what と value（雛形の定数）。
const R8_WHAT: &str = "承認の対話面（どの対話面を通った承認を受け取るか）";
const R8_VALUE: &str = "持ち主と作業する席の対話面（記帳先 = 台帳の notes + 文書の承認欄）";

/// 入口の lanes で判断の記録を指す行の置き換え（骨格自身の最初の判断の記録）。
const LANE_ADR: &str = "{doc: adr, at: ADR-1, label: なぜそう決めたか（最初の判断の記録）}";

// ── 結果 ──

/// 1 回の結果。標準出力は file ごとの 1 行と次の手の 1 行、標準エラーは理由（「folio init: 」は口が付ける）。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

impl Outcome {
    fn stop(verdict: Verdict, stderr: Vec<String>) -> Self {
        Outcome {
            verdict,
            stdout: Vec::new(),
            stderr,
        }
    }
}

// ── 日付 ──

/// 撃った時点の UTC の年-月-日（std の時計から数える・外部 crate を使わない）。
fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    civil(secs / 86_400)
}

/// 1970-01-01 からの日数を年-月-日へ（グレゴリオ暦）。
fn civil(days: u64) -> String {
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

// ── folio2 固有の id と括弧の落とし ──

/// 字の中の id の形（前の字が英字でないもの）を全部取る。形 = ADR- と数・便 と数（間の空白は任意）・
/// FR / NFR / AC / CON に続く数・P- / N- / A- に数（. と数が続いてもよい）・R- / D- に数。
pub fn ids_in(text: &str) -> Vec<String> {
    let c: Vec<char> = text.chars().collect();
    let digits = |from: usize| c[from..].iter().take_while(|x| x.is_ascii_digit()).count();
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if i > 0 && c[i - 1].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let rest: String = c[i..c.len().min(i + 4)].iter().collect();
        let mut found: Option<(usize, String)> = None;
        if rest.starts_with("ADR-") && digits(i + 4) > 0 {
            let n = digits(i + 4);
            found = Some((4 + n, c[i..i + 4 + n].iter().collect()));
        } else if c[i] == '便' {
            let sp = c[i + 1..].iter().take_while(|x| **x == ' ').count();
            let n = digits(i + 1 + sp);
            if n > 0 {
                let num: String = c[i + 1 + sp..i + 1 + sp + n].iter().collect();
                found = Some((1 + sp + n, format!("便{num}")));
            }
        } else if let Some(p) = ["NFR", "CON", "FR", "AC"]
            .iter()
            .find(|p| rest.starts_with(**p) && digits(i + p.chars().count()) > 0)
        {
            let len = p.chars().count();
            let n = digits(i + len);
            found = Some((len + n, c[i..i + len + n].iter().collect()));
        } else if matches!(c[i], 'P' | 'N' | 'A' | 'R' | 'D')
            && c.get(i + 1) == Some(&'-')
            && digits(i + 2) > 0
        {
            let mut len = 2 + digits(i + 2);
            if matches!(c[i], 'P' | 'N' | 'A') && c.get(i + len) == Some(&'.') && digits(i + len + 1) > 0
            {
                len += 1 + digits(i + len + 1);
            }
            found = Some((len, c[i..i + len].iter().collect()));
        }
        match found {
            Some((len, id)) => {
                out.push(id);
                i += len;
            }
            None => i += 1,
        }
    }
    out
}

/// 骨格自身の番号でない id を持つか。
fn has_foreign_id(text: &str) -> bool {
    ids_in(text).iter().any(|id| !OWN_IDS.contains(&id.as_str()))
}

/// 全角の括弧（入れ子は数える）のうち、中に folio2 固有の id を持つものを中身ごと落とす。閉じない括弧は残す。
fn drop_foreign_parens(line: &str) -> String {
    let c: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < c.len() {
        if c[i] == '（' {
            let mut depth = 0;
            let mut j = i;
            while j < c.len() {
                match c[j] {
                    '（' => depth += 1,
                    '）' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            if j < c.len() {
                let group: String = c[i..=j].iter().collect();
                if !has_foreign_id(&group) {
                    out.push_str(&group);
                }
                i = j + 1;
                continue;
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

// ── 行で写す節 ──

/// 最上位の節の始まりの行か（行頭が空白でも # でもない・空でない）。
fn is_head(line: &str) -> bool {
    line.chars().next().is_some_and(|ch| ch != ' ' && ch != '#')
}

/// 最上位の節 `key` の行（注釈の行と末尾の空行を落とす）。
fn section(src: &str, key: &str) -> Result<Vec<String>, String> {
    let mut lines: Option<Vec<String>> = None;
    for line in src.lines() {
        if is_head(line) {
            if lines.is_some() {
                break;
            }
            if line.split(':').next() == Some(key) {
                lines = Some(vec![line.to_string()]);
            }
            continue;
        }
        if let Some(lines) = lines.as_mut()
            && !line.trim_start().starts_with('#')
        {
            lines.push(line.to_string());
        }
    }
    let mut lines = lines.ok_or_else(|| format!("節 {key} が無い"))?;
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    Ok(lines)
}

/// 1 行の表の行（行頭の - の後が波括弧）を、字下げと欄の組に分ける。括弧の深さが 0 の読点で区切る。
fn flow_row(line: &str) -> Option<(String, Vec<(String, String)>)> {
    let lead = line.len() - line.trim_start().len();
    let body = line[lead..].strip_prefix("- {")?.strip_suffix('}')?;
    let mut fields = Vec::new();
    let (mut depth, mut quoted, mut start) = (0i32, false, 0);
    let push = |part: &str, fields: &mut Vec<(String, String)>| {
        let part = part.trim();
        let name = part.split(':').next().unwrap_or("").trim().to_string();
        fields.push((name, part.to_string()));
    };
    for (k, ch) in body.char_indices() {
        match ch {
            '"' => quoted = !quoted,
            '[' | '{' if !quoted => depth += 1,
            ']' | '}' if !quoted => depth -= 1,
            ',' if !quoted && depth == 0 => {
                push(&body[start..k], &mut fields);
                start = k + 1;
            }
            _ => {}
        }
    }
    push(&body[start..], &mut fields);
    Some((line[..lead].to_string(), fields))
}

/// 1 行の表の行を欄 `keep` だけに絞る。
fn keep_fields(line: &str, keep: &[&str]) -> Result<String, String> {
    let (lead, fields) = flow_row(line).ok_or_else(|| format!("1 行の表の行でない: {line}"))?;
    let kept: Vec<String> = fields
        .into_iter()
        .filter(|(name, _)| keep.contains(&name.as_str()))
        .map(|(_, part)| part)
        .collect();
    Ok(format!("{lead}- {{{}}}", kept.join(", ")))
}

/// 節の中の表の行（- で始まる行）だけを欄 `keep` に絞る。
fn narrow_rows(lines: Vec<String>, keep: &[&str]) -> Result<Vec<String>, String> {
    lines
        .into_iter()
        .map(|l| {
            if l.trim_start().starts_with("- {") {
                keep_fields(&l, keep)
            } else {
                Ok(l)
            }
        })
        .collect()
}

/// 相談窓口の sheet の節を欄 file・title・explain だけに絞る（sections の一覧は写さない）。
fn narrow_sheet(lines: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for (k, line) in lines.into_iter().enumerate() {
        let lead = line.len() - line.trim_start().len();
        if k == 0 {
            out.push(line);
            continue;
        }
        if lead == 2 {
            let name = line.trim_start().split(':').next().unwrap_or("");
            skip = !matches!(name, "file" | "title" | "explain");
        }
        if !skip {
            out.push(line);
        }
    }
    out
}

/// 入口の lanes の行き先: doc が adr の行は各 stops の最初の位置の 1 行（ADR-1）に畳み、at が図の id の行は落とす。
fn fix_lanes(lines: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut adr_done = false;
    for line in lines {
        if line.trim_start().starts_with("stops:") {
            adr_done = false;
            out.push(line);
            continue;
        }
        let Some((lead, fields)) = flow_row(&line) else {
            out.push(line);
            continue;
        };
        let value = |name: &str| {
            fields
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, part)| part.split_once(':').map_or("", |(_, v)| v.trim()).to_string())
        };
        if value("doc").as_deref() == Some("adr") {
            if !adr_done {
                out.push(format!("{lead}- {LANE_ADR}"));
                adr_done = true;
            }
            continue;
        }
        if value("at").is_some_and(|at| at.starts_with("fig-")) {
            continue;
        }
        out.push(line);
    }
    out
}

/// 焼いた正本から節を写す（最小の欄に絞り、括弧を落とす）。
fn copy_sections(
    src: &str,
    keys: &[&str],
    narrow: impl Fn(&str, Vec<String>) -> Result<Vec<String>, String>,
) -> Result<String, String> {
    let mut out = String::new();
    for key in keys {
        let lines = narrow(key, section(src, key)?)?;
        for line in lines {
            out.push_str(&drop_foreign_parens(&line));
            out.push('\n');
        }
    }
    Ok(out)
}

// ── 型付きで読む字 ──

fn typed(src: &str, file: &str) -> Result<Value, String> {
    yaml::parse_typed(src).map_err(|e| format!("焼いた {file} を型付きで読めない: {e}"))
}

fn s(x: &str) -> Value {
    Value::Str(x.to_string())
}

/// 憲法の schema の節（型付きで読み、同じ書き手で書く＝注釈は写らない）。
fn constitution_schema() -> Result<String, String> {
    let root = typed(SRC_CONSTITUTION, "constitution.yaml")?;
    let schema = root
        .get("schema")
        .ok_or("焼いた constitution.yaml に schema の節が無い")?
        .clone();
    yaml::write(&Value::Map(vec![(s("schema"), schema)]), CONSTITUTION_HEAD)
}

/// 規則の表の thresholds と discipline の 2 節（行 R-8 は雛形・行 R-16 は what と value を folio2 から読む）。
fn rules_rows() -> Result<String, String> {
    let root = typed(SRC_RULES, "rules.yaml")?;
    let rows: Vec<&Value> = root
        .get("thresholds")
        .and_then(Value::as_seq)
        .unwrap_or(&[])
        .iter()
        .filter(|r| r.get("id").and_then(Value::as_str) == Some("R-16"))
        .collect();
    let [r16] = rows.as_slice() else {
        return Err(format!(
            "焼いた rules.yaml の行 R-16 がちょうど 1 つでない（{} 行）",
            rows.len()
        ));
    };
    let field = |k: &str| {
        r16.get(k)
            .cloned()
            .ok_or_else(|| format!("焼いた rules.yaml の行 R-16 に {k} が無い"))
    };
    let row = |id: &str, what: Value, value: Value| {
        Value::Map(vec![
            (s("id"), s(id)),
            (s("article"), s("P-1")),
            (s("what"), what),
            (s("value"), value),
            (s("kind"), s("build-check")),
            (s("status"), s("仮")),
            (s("ruling"), s("未記入")),
            (s("ruled_at"), s("未記入")),
            (s("stage"), s("post")),
        ])
    };
    let table = Value::Map(vec![
        (
            s("thresholds"),
            Value::Seq(vec![
                row("R-8", s(R8_WHAT), s(R8_VALUE)),
                row("R-16", field("what")?, field("value")?),
            ]),
        ),
        (s("discipline"), Value::Seq(Vec::new())),
    ]);
    yaml::write_plain(&table, RULES_ROWS_HEAD)
}

// ── 組み立て ──

fn region() -> String {
    format!("{}\n{}\n", schema::BEGIN, schema::END)
}

/// 11 本の中身（FILES の順）。焼いた字が読めなければ Err（何も書かない）。
fn compose(date: &str) -> Result<Vec<String>, String> {
    let d = |t: &str| t.replace(DATE, date);
    let constitution = format!("{}{}", constitution_schema()?, d(CONSTITUTION_BODY));
    let rules = format!("{RULES_HEAD}\n{}{}", region(), rules_rows()?);
    let vocabulary = format!("{VOCABULARY}{}", region());
    let srs = format!("{}{}", d(SRS), region());
    let index = format!(
        "{}{}{}",
        d(INDEX_HEAD),
        copy_sections(SRC_INDEX, &["audience", "shelf", "lanes", "intake"], |key, lines| {
            Ok(if key == "lanes" { fix_lanes(lines) } else { lines })
        })?,
        region()
    );
    let intake = format!(
        "{}{}{}",
        d(INTAKE_HEAD),
        copy_sections(
            SRC_INTAKE,
            &["answers", "targets", "questions", "sheet"],
            |key, lines| match key {
                "targets" => narrow_rows(lines, &["id", "type", "with"]),
                "sheet" => Ok(narrow_sheet(lines)),
                _ => Ok(lines),
            }
        )?,
        region()
    );
    let ceiling = format!(
        "{}{}{}",
        d(CEILING_HEAD),
        copy_sections(
            SRC_CEILING,
            &["weights", "documents", "viewpoints"],
            |key, lines| match key {
                "documents" => narrow_rows(lines, &["id", "file"]),
                _ => Ok(lines),
            }
        )?,
        region()
    );
    let graph = format!("{}{}", d(GRAPH), region());
    let adr_schema = format!("{}{}", d(ADR_SCHEMA), region());
    let adr_1 = d(ADR_1);
    let note_schema = format!("{}{}", d(NOTE_SCHEMA), region());
    Ok(vec![
        constitution,
        rules,
        vocabulary,
        srs,
        index,
        intake,
        ceiling,
        graph,
        adr_schema,
        adr_1,
        note_schema,
    ])
}

// ── 書く口 ──

/// 在るか（通常の file・dir・symlink・壊れた symlink のどれでも在ると数える）。
fn present(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// 新規作成でだけ書く（既に在れば開かない）。
fn create_new(path: &Path, body: &str) -> std::io::Result<()> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
    f.write_all(body.as_bytes())
}

/// 書いた file と書かなかった file の名の 2 行。
fn tally(written: &[&str]) -> Vec<String> {
    let rest: Vec<&str> = FILES.iter().filter(|f| !written.contains(f)).copied().collect();
    vec![
        format!("書いた file: {}", or_none(written)),
        format!("書かなかった file: {}", or_none(&rest)),
    ]
}

fn or_none(names: &[&str]) -> String {
    if names.is_empty() {
        "なし".to_string()
    } else {
        names.join("・")
    }
}

pub fn run(dir: &Path) -> Outcome {
    // 置き場の形（symlink か dir でない file なら書けない）
    if dir.is_symlink() || (present(dir) && !dir.is_dir()) {
        return Outcome::stop(
            Verdict::Unknown,
            vec![format!("{}: 置き場が dir でない（symlink か file）", dir.display())],
        );
    }
    // 断り（在ったものの名を全部）
    let found: Vec<String> = REFUSE
        .iter()
        .filter(|name| present(&dir.join(name)))
        .map(|name| format!("在る: {}", dir.join(name).display()))
        .collect();
    if !found.is_empty() {
        let mut stderr = vec!["正本か置き場が既に在るので何も書かない".to_string()];
        stderr.extend(found);
        return Outcome::stop(Verdict::Fail, stderr);
    }
    // 置き場の親が無ければ作らない
    let parent: PathBuf = match dir.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    if !present(dir) && !parent.is_dir() {
        return Outcome::stop(
            Verdict::Unknown,
            vec![format!("{}: 置き場の親 dir が無い（作らない）", parent.display())],
        );
    }
    // 焼いた字を読んで 11 本を組む（書く前に止める）
    let bodies = match compose(&today_utc()) {
        Ok(b) => b,
        Err(e) => return Outcome::stop(Verdict::Unknown, vec![e, "1 byte も書いていない".to_string()]),
    };
    // (1) 新規作成でだけ書く（生成区間は空の印）
    let mut written: Vec<&str> = Vec::new();
    let dirs = std::iter::once(dir.to_path_buf()).chain(DIRS.iter().map(|sub| dir.join(sub)));
    for path in dirs {
        if present(&path) {
            continue;
        }
        if let Err(e) = fs::create_dir(&path) {
            let mut stderr = vec![format!("{}: dir を作れない: {e}", path.display())];
            stderr.extend(tally(&written));
            return Outcome::stop(Verdict::Unknown, stderr);
        }
    }
    for (file, body) in FILES.iter().zip(&bodies) {
        let path = dir.join(file);
        if let Err(e) = create_new(&path, body) {
            let mut stderr = vec![format!("{}: 書けない: {e}", path.display())];
            stderr.extend(tally(&written));
            return Outcome::stop(Verdict::Unknown, stderr);
        }
        written.push(file);
    }
    // (2) 生成区間を `folio schema --write` と同じ導出で埋める
    let filled = schema::run(dir, schema::Mode::Write);
    if filled.verdict != Verdict::Pass {
        let mut stderr = vec![format!(
            "生成区間を埋められない: {}",
            filled.stderr.unwrap_or_default()
        )];
        stderr.extend(tally(&written));
        return Outcome::stop(Verdict::Unknown, stderr);
    }
    let mut stdout: Vec<String> = FILES
        .iter()
        .map(|file| {
            let path = dir.join(file);
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            format!("folio init: 書いた {}（{size} byte）", path.display())
        })
        .collect();
    stdout.push(format!(
        "folio init: 次の手 = 版管理に add して commit してから folio check --dir {} を撃つ（commit の前に撃つと「まだ分からない」が 1 件増える）。凍結の基準は後で凍結する",
        dir.display()
    ));
    Outcome {
        verdict: Verdict::Pass,
        stdout,
        stderr: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates() {
        assert_eq!(civil(0), "1970-01-01");
        assert_eq!(civil(20_720), "2026-09-24");
        assert_eq!(civil(11_016), "2000-02-29");
    }

    #[test]
    fn ids_and_parens() {
        assert_eq!(
            ids_in("ADR-16・便 45・NFR3・FR2・P-6.3・R-7・SHA-256・ADR-1"),
            ["ADR-16", "便45", "NFR3", "FR2", "P-6.3", "R-7", "ADR-1"]
        );
        assert_eq!(drop_foreign_parens("a（b ADR-8 （c））d（R-8）"), "ad（R-8）");
    }
}
