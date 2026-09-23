//! `folio intake`（便 19・docs/design/delivery-19.md §1 (a)(b)）。相談窓口の正本 `intake.yaml`（ADR-6 決定 (1)・
//! 便 18 で床に入る・ここでは読むだけ）から、答えの無い質問と推奨回答を対話面の散文として標準出力へ書き（`--print`・
//! rules 行 D-10・選択式の窓を使わない）、回答（旗 `--answers` の YAML の file）と既に在る支度表（FR8）から
//! 「何を持つか」を写像して支度表 1 枚を `<dir>/<sheet.file>` へ書く（`--write`）。
//! 答えの無い質問は推奨回答で進め、その全部を支度表の別枠（recommended）に出す（FR2）。
//! 採否の判断は道具の外（P-1.2）＝「承認」を求めず、承認欄は空のまま書く（承認の記帳は持ち主の対話面と台帳・P-12.2）。
//! 導出できない入力は 2「まだ分からない」に倒し、出力先に 1 byte も書かない（P-4.1）。正規表現は使わない。

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::cursor::{self, R, X};
use crate::verdict::Verdict;
use crate::yaml::{self, Value};

/// 相談窓口の正本。
const INTAKE: &str = "intake.yaml";

/// 支度表の 1 行目の注釈（固定・数と日付は書かない・P-6.3）。
const HEADER: &str = "# folio2 支度表（folio intake の生成物・手で直さない・承認は対話面と台帳で）";

/// 支度表の meta の id と status（承認の後に status を変えるのは人）。
const SHEET_ID: &str = "folio2-intake-sheet";
const DRAFT: &str = "draft";

/// 回答の値（intake.yaml の answers の values・yes / no の行き先の選び分け）。
const YES: &str = "はい";
const NO: &str = "いいえ";

/// 回答の出所。
const ANSWERED: &str = "answered";
const RECOMMENDED: &str = "recommended";

pub enum Mode {
    Print,
    Write,
}

/// 1 回の実行の結果。`stdout` は 1 行ずつ・`stderr` は 1 行。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Vec<String>,
    pub stderr: Option<String>,
}

// ── 命令の口 ──

pub fn run(dir: &Path, answers: Option<&Path>, mode: Mode) -> Outcome {
    match build(dir, answers, mode) {
        Ok(outcome) => outcome,
        Err(reason) => Outcome {
            verdict: Verdict::Unknown,
            stdout: Vec::new(),
            stderr: Some(format!("folio intake: まだ分からない: {reason}")),
        },
    }
}

fn build(dir: &Path, answers_path: Option<&Path>, mode: Mode) -> R<Outcome> {
    let intake = load_intake(dir)?;
    let given = match answers_path {
        Some(path) => read_answers(dir, path, &intake)?,
        None => Vec::new(),
    };
    let sheet_path = dir.join(&intake.sheet_file);
    let carried = if sheet_path.exists() {
        read_sheet(dir, &intake)?
    } else {
        Vec::new()
    };
    let answers = resolve(&intake.questions, &given, &carried);

    match mode {
        Mode::Print => {
            let pending: Vec<&Question> = intake
                .questions
                .iter()
                .zip(&answers)
                .filter(|(_, a)| a.source == Source::Recommended)
                .map(|(q, _)| q)
                .collect();
            let mut out = Vec::new();
            if pending.is_empty() {
                out.push(format!(
                    "folio intake: 問う質問は無い（支度表 {} が全部答え済み）",
                    sheet_path.display()
                ));
            } else {
                out.push(format!(
                    "folio intake: 質問 {} つ（答えは旗 answers の YAML の file で渡す・答えなくてもおすすめで進む）",
                    pending.len()
                ));
                for q in pending {
                    out.push(format!(
                        "{}. {}（おすすめ: {}）— {}",
                        q.id, q.ask, q.recommend, q.why
                    ));
                }
            }
            Ok(Outcome {
                verdict: Verdict::Pass,
                stdout: out,
                stderr: None,
            })
        }
        Mode::Write => {
            let documents = map_documents(&intake.questions, &answers, &intake.targets)?;
            let text = yaml::write(&sheet_value(&intake, &documents, &answers), HEADER)
                .map_err(|e| format!("{}: {e}", sheet_path.display()))?;
            if !sheet_path.parent().is_some_and(Path::is_dir) {
                return Err(format!("{}: 出力先の親 dir が無い", sheet_path.display()));
            }
            fs::write(&sheet_path, &text)
                .map_err(|e| format!("{}: 書けない: {e}", sheet_path.display()))?;
            let recommended = answers
                .iter()
                .filter(|a| a.source == Source::Recommended)
                .count();
            Ok(Outcome {
                verdict: Verdict::Pass,
                stdout: vec![format!(
                    "folio intake: 支度表を書いた（持つ文書 {}・推奨で進めた項目 {recommended}・{}）",
                    documents.len(),
                    sheet_path.display()
                )],
                stderr: None,
            })
        }
    }
}

// ── 正本の読み ──

/// 写像の行き先（intake.yaml の targets の 1 行）。
#[derive(Debug, Clone, PartialEq)]
struct Target {
    id: String,
    kind: String,
    with: Vec<String>,
}

/// 質問（intake.yaml の questions の 1 行）。
#[derive(Debug, Clone, PartialEq)]
struct Question {
    id: String,
    ask: String,
    recommend: String,
    why: String,
    yes: Vec<String>,
    no: Vec<String>,
}

struct Intake {
    version: String,
    values: Vec<String>,
    targets: Vec<Target>,
    questions: Vec<Question>,
    sheet_file: String,
}

/// 相談窓口の正本を型付きで読む。欄が無い・型が違う・質問の id の重複・recommend が values に無い・
/// 行き先が targets に無い は Err（まだ分からない）。
fn load_intake(dir: &Path) -> R<Intake> {
    let value = cursor::load(dir, INTAKE)?;
    let root = X::root(&value, INTAKE);

    let version = root.f("meta")?.f("version")?.text()?;
    let answers = root.f("answers")?;
    let values = texts(&answers.f("values")?)?;
    // default は値域の外の固定の値（recommend）も取り得る＝欄の非空だけ見る
    answers.f("default")?.text()?;

    let mut targets = Vec::new();
    for row in root.f("targets")?.seq()? {
        targets.push(Target {
            id: row.f("id")?.text()?,
            kind: row.f("type")?.text()?,
            with: texts(&row.f("with")?)?,
        });
    }

    let mut questions = Vec::new();
    for row in root.f("questions")?.seq()? {
        questions.push(Question {
            id: row.f("id")?.text()?,
            ask: row.f("ask")?.text()?,
            recommend: row.f("recommend")?.text()?,
            why: row.f("why")?.text()?,
            yes: texts(&branch(&row, true)?)?,
            no: texts(&branch(&row, false)?)?,
        });
    }

    let sheet_file = root.f("sheet")?.f("file")?.text()?;

    let mut seen: HashSet<&str> = HashSet::new();
    for q in &questions {
        if !seen.insert(&q.id) {
            return Err(format!("{INTAKE}: questions の id「{}」が 2 度", q.id));
        }
        if !values.contains(&q.recommend) {
            return Err(format!(
                "{INTAKE}: questions の行 {} の recommend「{}」が answers の values に無い",
                q.id, q.recommend
            ));
        }
        for id in q.yes.iter().chain(&q.no) {
            if !targets.iter().any(|t| t.id == *id) {
                return Err(format!(
                    "{INTAKE}: questions の行 {} の行き先「{id}」が targets に無い",
                    q.id
                ));
            }
        }
    }

    Ok(Intake {
        version,
        values,
        targets,
        questions,
        sheet_file,
    })
}

/// 一覧の各要素の字面。
fn texts(x: &X<'_>) -> R<Vec<String>> {
    x.seq()?.iter().map(|v| v.text()).collect()
}

/// 質問の行の行き先の欄（`yes` / `no`）。型付きの木では引用符の無い `yes` と `no` のキーは真偽に倒れる
/// （YAML 1.1 の解決）ので、真偽のキーと文字列のキーのどちらでも引く。
fn branch<'a>(row: &X<'a>, yes: bool) -> R<X<'a>> {
    let name = if yes { "yes" } else { "no" };
    let entries = row
        .v
        .as_map()
        .ok_or_else(|| format!("{}: 表でない", row.at))?;
    entries
        .iter()
        .find(|(k, _)| *k == Value::Bool(yes) || k.as_str() == Some(name))
        .map(|(_, v)| X {
            v,
            at: format!("{}.{name}", row.at),
        })
        .ok_or_else(|| format!("{}: 欄 {name} が無い", row.at))
}

/// 回答の file（最上位の欄 answers の表・質問の id → 値）。
fn read_answers(dir: &Path, path: &Path, intake: &Intake) -> R<Vec<(String, String)>> {
    let name = path.to_string_lossy().into_owned();
    let value = cursor::load(dir, &name)?;
    let root = X::root(&value, &name);
    let mut out = Vec::new();
    for (q, v) in root.f("answers")?.pairs()? {
        if !intake.questions.iter().any(|x| x.id == q) {
            return Err(format!("{name}: 質問の id「{q}」が questions に無い"));
        }
        let value = v.text()?;
        if !intake.values.contains(&value) {
            return Err(format!(
                "{name}: 質問 {q} の値「{value}」が answers の values に無い"
            ));
        }
        out.push((q.to_string(), value));
    }
    Ok(out)
}

/// 既に在る支度表の answers の行（引き継ぎの元・FR8）。
#[derive(Debug, Clone, PartialEq)]
struct SheetAnswer {
    q: String,
    value: String,
    source: String,
}

fn read_sheet(dir: &Path, intake: &Intake) -> R<Vec<SheetAnswer>> {
    let value = cursor::load(dir, &intake.sheet_file)?;
    let root = X::root(&value, &intake.sheet_file);
    let mut out = Vec::new();
    for row in root.f("answers")?.seq()? {
        let q = row.f("q")?.text()?;
        if !intake.questions.iter().any(|x| x.id == q) {
            return Err(format!(
                "{}: answers の行の q「{q}」が questions に無い",
                intake.sheet_file
            ));
        }
        let v = row.f("value")?.text()?;
        if !intake.values.contains(&v) {
            return Err(format!(
                "{}: answers の行 {q} の value「{v}」が answers の values に無い",
                intake.sheet_file
            ));
        }
        let source = match row.g("source")? {
            Some(s) => s.text()?,
            None => String::new(),
        };
        out.push(SheetAnswer {
            q,
            value: v,
            source,
        });
    }
    Ok(out)
}

// ── 回答の解き方と写像 ──

/// 回答の出所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    /// 回答の file か、既に在る支度表の answered の行から
    Answered,
    /// 答えが無く推奨回答で進めた
    Recommended,
}

impl Source {
    fn label(self) -> &'static str {
        match self {
            Source::Answered => ANSWERED,
            Source::Recommended => RECOMMENDED,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Answer {
    q: String,
    value: String,
    source: Source,
}

/// 質問ごとに 回答の file → 既に在る支度表の source が answered の行 → 推奨回答 の順で値を決める（questions の順）。
/// 既に在る支度表の source が recommended の行は値を持たないものとして扱う（＝また問う・FR8）。
fn resolve(
    questions: &[Question],
    given: &[(String, String)],
    sheet: &[SheetAnswer],
) -> Vec<Answer> {
    questions
        .iter()
        .map(|q| {
            let from_file = given
                .iter()
                .find(|(id, _)| *id == q.id)
                .map(|(_, v)| v.clone());
            let carried = || {
                sheet
                    .iter()
                    .find(|r| r.q == q.id && r.source == ANSWERED)
                    .map(|r| r.value.clone())
            };
            match from_file.or_else(carried) {
                Some(value) => Answer {
                    q: q.id.clone(),
                    value,
                    source: Source::Answered,
                },
                None => Answer {
                    q: q.id.clone(),
                    value: q.recommend.clone(),
                    source: Source::Recommended,
                },
            }
        })
        .collect()
}

/// 持つ文書（行き先の id・型・付録・どの質問から決まったか）。
#[derive(Debug, Clone, PartialEq)]
struct Doc {
    id: String,
    kind: String,
    with: Vec<String>,
    from: String,
}

/// はい なら yes の一覧・いいえ なら no の一覧 の行き先を questions の順・一覧の順に集める。
/// 同じ id は最初の 1 回だけ持つ（from = その質問の id）。
fn map_documents(questions: &[Question], answers: &[Answer], targets: &[Target]) -> R<Vec<Doc>> {
    let mut out: Vec<Doc> = Vec::new();
    for (q, a) in questions.iter().zip(answers) {
        let ids = match a.value.as_str() {
            YES => &q.yes,
            NO => &q.no,
            other => {
                return Err(format!(
                    "{INTAKE}: 質問 {} の回答「{other}」は {YES} でも {NO} でもない",
                    q.id
                ));
            }
        };
        for id in ids {
            if out.iter().any(|d| d.id == *id) {
                continue;
            }
            let t = targets
                .iter()
                .find(|t| t.id == *id)
                .ok_or_else(|| format!("{INTAKE}: 行き先「{id}」が targets に無い"))?;
            out.push(Doc {
                id: id.clone(),
                kind: t.kind.clone(),
                with: t.with.clone(),
                from: q.id.clone(),
            });
        }
    }
    Ok(out)
}

// ── 支度表の字面 ──

fn s(text: &str) -> Value {
    Value::Str(text.to_string())
}

fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(
        entries
            .into_iter()
            .map(|(k, v)| (Value::Str(k.to_string()), v))
            .collect(),
    )
}

fn seq(items: &[String]) -> Value {
    Value::Seq(items.iter().map(|i| s(i)).collect())
}

/// 支度表の値の木（intake.yaml の sheet の節が決める 5 つの欄・数と日付は書かない）。
fn sheet_value(intake: &Intake, documents: &[Doc], answers: &[Answer]) -> Value {
    let documents: Vec<Value> = documents
        .iter()
        .map(|d| {
            map(vec![
                ("id", s(&d.id)),
                ("type", s(&d.kind)),
                ("with", seq(&d.with)),
                ("from", s(&d.from)),
            ])
        })
        .collect();
    let recommended: Vec<Value> = intake
        .questions
        .iter()
        .zip(answers)
        .filter(|(_, a)| a.source == Source::Recommended)
        .map(|(q, _)| {
            map(vec![
                ("q", s(&q.id)),
                ("ask", s(&q.ask)),
                ("recommend", s(&q.recommend)),
            ])
        })
        .collect();
    let received: Vec<Value> = answers
        .iter()
        .map(|a| {
            map(vec![
                ("q", s(&a.q)),
                ("value", s(&a.value)),
                ("source", s(a.source.label())),
            ])
        })
        .collect();
    map(vec![
        (
            "meta",
            map(vec![
                ("id", s(SHEET_ID)),
                ("version", s(&intake.version)),
                ("status", s(DRAFT)),
            ]),
        ),
        ("documents", Value::Seq(documents)),
        ("recommended", Value::Seq(recommended)),
        ("answers", Value::Seq(received)),
        // 承認の記帳は持ち主の対話面と台帳の後（P-12.2）＝道具は空のまま書く
        ("approval", Value::Seq(Vec::new())),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(id: &str, yes: &[&str]) -> Question {
        Question {
            id: id.to_string(),
            ask: format!("{id} を持ちますか"),
            recommend: YES.to_string(),
            why: format!("{id} の理由"),
            yes: yes.iter().map(|y| y.to_string()).collect(),
            no: Vec::new(),
        }
    }

    fn row(q: &str, value: &str, source: &str) -> SheetAnswer {
        SheetAnswer {
            q: q.to_string(),
            value: value.to_string(),
            source: source.to_string(),
        }
    }

    /// 回答の file → 既に在る支度表の answered の行 → 推奨回答 の順。既に在る recommended の行は引き継がない。
    #[test]
    fn sheet_resolve_prefers_file_then_answered_then_recommend() {
        let questions = vec![q("q1", &["a"]), q("q2", &["b"]), q("q3", &["c"])];
        let given = vec![("q1".to_string(), NO.to_string())];
        let sheet = vec![
            row("q1", YES, ANSWERED),
            row("q2", NO, ANSWERED),
            // 推奨で進めた行は答え済みにしない（また問う・FR8）
            row("q3", YES, RECOMMENDED),
        ];
        let got = resolve(&questions, &given, &sheet);
        assert_eq!(
            got,
            vec![
                Answer {
                    q: "q1".to_string(),
                    value: NO.to_string(),
                    source: Source::Answered
                },
                Answer {
                    q: "q2".to_string(),
                    value: NO.to_string(),
                    source: Source::Answered
                },
                Answer {
                    q: "q3".to_string(),
                    value: YES.to_string(),
                    source: Source::Recommended
                },
            ]
        );
    }

    /// 同じ行き先は最初の 1 回だけ・from は最初に決めた質問の id。
    #[test]
    fn sheet_map_keeps_the_first_target_once() {
        let questions = vec![q("q1", &["srs", "adr"]), q("q2", &["adr"])];
        let answers = resolve(&questions, &[], &[]);
        let targets = vec![
            Target {
                id: "srs".to_string(),
                kind: "要件書".to_string(),
                with: Vec::new(),
            },
            Target {
                id: "adr".to_string(),
                kind: "判断の記録".to_string(),
                with: Vec::new(),
            },
        ];
        let docs = map_documents(&questions, &answers, &targets).unwrap();
        let ids: Vec<(&str, &str)> = docs
            .iter()
            .map(|d| (d.id.as_str(), d.from.as_str()))
            .collect();
        assert_eq!(ids, vec![("srs", "q1"), ("adr", "q1")]);
        assert_eq!(docs[1].kind, "判断の記録");
    }
}
