//! 判定（3 値と終了コード）。実行できなかった検査は合格にしない（FR5）。

use std::fmt;

/// 検査 1 回の結果。終了コードは day-1 の床（scripts/check_draft.py）と同じ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// 合格（0）
    Pass,
    /// 不合格（1）
    Fail,
    /// まだ分からない（2）= 読めない・測れない
    Unknown,
}

impl Verdict {
    pub fn exit_code(self) -> i32 {
        match self {
            Verdict::Pass => 0,
            Verdict::Fail => 1,
            Verdict::Unknown => 2,
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Verdict::Pass => "合格",
            Verdict::Fail => "不合格",
            Verdict::Unknown => "まだ分からない",
        })
    }
}

/// 検査の所見を集める。違反と「読めない」（unknowns）と「測れない」（pendings・便 7）を分けて持つ。
#[derive(Debug, Default)]
pub struct Report {
    pub violations: Vec<(String, String)>,
    pub unknowns: Vec<String>,
    pub pendings: Vec<String>,
}

impl Report {
    pub fn violation(&mut self, kind: &str, msg: impl Into<String>) {
        self.violations.push((kind.to_string(), msg.into()));
    }

    pub fn unknown(&mut self, msg: impl Into<String>) {
        self.unknowns.push(msg.into());
    }

    /// 測れない（読めたうえで比較元が立たない）。違反が在れば違反が先に立つ。
    pub fn pending(&mut self, msg: impl Into<String>) {
        self.pendings.push(msg.into());
    }

    /// 床と同じ並び: 読めない → まだ分からない（読めない file の中身は数えていない＝不合格とも言えない）／
    /// 違反 → 不合格／測れない → まだ分からない／どれも無い → 合格（床の「1 と 2 が同時に立つときは 1」）。
    pub fn verdict(&self) -> Verdict {
        if !self.unknowns.is_empty() {
            Verdict::Unknown
        } else if !self.violations.is_empty() {
            Verdict::Fail
        } else if !self.pendings.is_empty() {
            Verdict::Unknown
        } else {
            Verdict::Pass
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_is_never_pass() {
        let mut r = Report::default();
        r.unknown("読めない");
        assert_eq!(r.verdict(), Verdict::Unknown);
        r.violation("x", "違反");
        assert_eq!(r.verdict().exit_code(), 2);
    }

    #[test]
    fn pending_yields_to_violations() {
        let mut r = Report::default();
        r.pending("測れない");
        assert_eq!(r.verdict(), Verdict::Unknown);
        r.violation("x", "違反");
        assert_eq!(r.verdict(), Verdict::Fail);
    }
}
