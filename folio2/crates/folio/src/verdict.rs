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

/// 検査の所見を集める。違反と「測れなかった」を分けて持つ。
#[derive(Debug, Default)]
pub struct Report {
    pub violations: Vec<(String, String)>,
    pub unknowns: Vec<String>,
}

impl Report {
    pub fn violation(&mut self, kind: &str, msg: impl Into<String>) {
        self.violations.push((kind.to_string(), msg.into()));
    }

    pub fn unknown(&mut self, msg: impl Into<String>) {
        self.unknowns.push(msg.into());
    }

    /// 読めない・測れないが 1 つでもあれば「まだ分からない」。床と同じく、読めた範囲の違反より先に立てる
    /// （読めない file の中身は数えていない＝不合格とも言えない）。
    pub fn verdict(&self) -> Verdict {
        if !self.unknowns.is_empty() {
            Verdict::Unknown
        } else if !self.violations.is_empty() {
            Verdict::Fail
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
}
