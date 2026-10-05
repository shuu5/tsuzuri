//! host の面の書き込みの測りの表 `[[write-budget]]` の行の組み立てと欄の検査と名の重複の検査（設計 write-budget.md §3・ADR-0112・FR113）。
//!
//! 表は**host の面にだけ**置ける（装置の path は host 固有で PUBLIC repo に載せない・CON2）。tracked の面に置いた表は
//! [`super::manifest`] の読み手が 1 表 1 件で断る。読み手はここを呼ぶだけで、見出しと受ける key の列もここの定数を引く。
//! 器は stat file の path を読むだけで、「`/sys/block/` + 名」を焼かない（差し替えの口を env に作れない・C2.2・host ごとの値は面だけ・N3）。

use super::manifest::{check_keys, text_field, RawRow, RawValue};
use super::RuleError;

/// 表の見出しの字面。
pub(super) const HEADER: &str = "[[write-budget]]";

/// 1 行が持てる key の全体（宣言順・閉じた 3 つ・wear だけ任意）。
pub(super) const KEYS: &[&str] = &["name", "stat", "wear"];

/// 1 行に必ず要る key（name と stat）。
pub(super) const REQUIRED: &[&str] = &["name", "stat"];

/// `[[write-budget]]` 1 行が名乗る測りの装置（host 固有の値・host の面にだけ書く）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteBudget {
    name: String,
    stat: String,
    wear: Option<String>,
    line: u64,
}

impl WriteBudget {
    /// 記録の dir の名（英数字と `-` と `_` の 1 字以上・host の面で一意）。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 装置の stat file の絶対 path（空白を含まない）。
    pub fn stat(&self) -> &str {
        &self.stat
    }

    /// 装置の摩耗の記録の file の絶対 path（空白を含まない・書かない行は None）。器は path の形だけを判じ、file を読まない。
    pub fn wear(&self) -> Option<&str> {
        self.wear.as_deref()
    }

    /// manifest の中でこの行が始まる物理行番号。
    pub fn line(&self) -> u64 {
        self.line
    }
}

/// `[[write-budget]]` 1 行を組む。欠けと未知 key と型違いは `check_keys` と `text_field` が 1 件ずつ積み、欄の値の形に外れるものは
/// その key の行番号で 1 件ずつ断る。
pub(super) fn build(raw: &RawRow, errors: &mut Vec<RuleError>) -> Option<WriteBudget> {
    let before = errors.len();
    check_keys(raw, errors);
    if raw.fields.iter().any(|(_, value, _)| matches!(value, RawValue::Broken)) {
        return None;
    }
    let name = text_field(raw, "name", errors);
    let stat = text_field(raw, "stat", errors);
    let wear = text_field(raw, "wear", errors);
    let (Some(name), Some(stat)) = (name, stat) else {
        return None;
    };
    if errors.len() > before {
        return None;
    }
    if name.is_empty() || !name.chars().all(|found| found.is_ascii_alphanumeric() || found == '-' || found == '_') {
        errors.push(RuleError::new(line_of(raw, "name"), format!("name {name:?} は英数字と - と _ の 1 字以上でない")));
    }
    check_path(raw, "stat", &stat, errors);
    if let Some(wear) = &wear {
        check_path(raw, "wear", wear, errors);
    }
    (errors.len() == before).then_some(WriteBudget { name, stat, wear, line: raw.line })
}

/// path の key の値が空白を含まない絶対 path でなければ、その key の行番号で 1 件断る（空の字も絶対 path でない）。
fn check_path(raw: &RawRow, key: &str, value: &str, errors: &mut Vec<RuleError>) {
    if !value.starts_with('/') {
        errors.push(RuleError::new(line_of(raw, key), format!("{key} {value:?} が絶対 path でない")));
    } else if value.chars().any(char::is_whitespace) {
        errors.push(RuleError::new(line_of(raw, key), format!("{key} が空白を含む: {value:?}")));
    }
}

/// key が書かれていた物理行番号（無ければ見出しの行）。
fn line_of(raw: &RawRow, key: &str) -> u64 {
    raw.fields.iter().find(|(found, _, _)| found == key).map_or(raw.line, |(_, _, line)| *line)
}

/// 測りの名の重複を、2 度目の行の見出しの行番号で 1 件ずつ断る（同じ名の記録の dir が 2 つ在ると行ごとに独立に進められない）。
pub(super) fn check_names(rows: &[WriteBudget], errors: &mut Vec<RuleError>) {
    for (index, row) in rows.iter().enumerate() {
        if rows.iter().take(index).any(|found| found.name == row.name) {
            errors.push(RuleError::new(row.line, format!("書き込みの測りの名 {} が重複する", row.name)));
        }
    }
}
