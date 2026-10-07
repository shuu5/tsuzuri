//! 断りの証拠の在り処と確からしさ（親 [`super`] の `Refuse::evidence` が返し、予想の base の弁別が読む）。

use super::overlap::covered;

/// 断りの証拠の在り処（**閉じた 4 値**・設計 docs/design/dispatcher.md §27 形 3）: 予想の base（未着地の依存を重ねた木）で
/// 撃った断りが、依存の着地で消えうるかを [`discern`] が動く file と突き合わせて決める材料である。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Evidence {
    /// 行の字と規則だけで決まる（base の木に依らない）。
    Row,
    /// 名指した file の列（base の tracked に在るか・行数で決まる）。
    Files(Vec<String>),
    /// 本文の読み手（`.rs` / `.snap` の閉包・歯の置き場・名指し）が解く名。
    Name,
    /// 置き場と host（live な便・便の履歴・base の木の実走・読めない面）。
    Place,
}

impl Evidence {
    /// 在り処の名（`row` / `files` / `name` / `place`）。
    pub(crate) fn as_str(&self) -> &'static str {
        match *self {
            Self::Row => "row",
            Self::Files(_) => "files",
            Self::Name => "name",
            Self::Place => "place",
        }
    }

    /// 結果の file の finding の行に書く字面（file の列は `files:<a>,<b>`・他は名だけ）。
    pub(crate) fn render(&self) -> String {
        match *self {
            Self::Files(ref files) => format!("{}:{}", self.as_str(), files.join(",")),
            Self::Row | Self::Name | Self::Place => self.as_str().to_owned(),
        }
    }
}

/// 予想の base で撃った断りの確からしさ（**閉じた 3 値**・§27 形 3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Certainty {
    /// 依存が着地しても消えない（確定）。
    Firm,
    /// 依存の着地で消えうる（暫定）。
    Provisional,
    /// 予想の base では測れない（型を持たない断り・置き場と host に依る断り）。
    Unmeasured,
}

impl Certainty {
    /// 結果の file に書く語。
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Firm => "firm",
            Self::Provisional => "provisional",
            Self::Unmeasured => "unmeasured",
        }
    }
}

/// 本文の読み手が読む file の拡張子（受付の材料の `.rs` と `.snap`）。
const BODY_READS: [&str; 2] = [".rs", ".snap"];

/// 在り処と動く file（宣言で重ねた祖先の write-set の file・接頭辞を剥がし dir は展開済み）から確からしさを決める
/// （**弁別の 1 関数**・pure・§27 形 3）: 行 → 確定、file の列 → 動く file と交わらなければ確定・交われば暫定、名 → 動く file に
/// 本文の読み手が読む file が 1 本も無ければ確定・在れば暫定、置き場と host → 測れない。
pub(crate) fn discern(evidence: &Evidence, moving: &[String]) -> Certainty {
    let crossed = match *evidence {
        Evidence::Row => false,
        Evidence::Files(ref files) => files
            .iter()
            .any(|file| covered(moving, file) || moving.iter().any(|found| covered(std::slice::from_ref(file), found))),
        Evidence::Name => moving.iter().any(|found| BODY_READS.iter().any(|ext| found.ends_with(ext))),
        Evidence::Place => return Certainty::Unmeasured,
    };
    if crossed {
        Certainty::Provisional
    } else {
        Certainty::Firm
    }
}

#[cfg(test)]
mod tests {
    // flip-check: moved t3-hub.92.10.6
    use super::{discern, Certainty, Evidence};

    /// 弁別の 1 関数は 3 値を返す（§27 形 3）: 在り処 4 値 × 動く file の有無（無い・交わる `.rs`・交わらない `.md`）。
    #[test]
    fn pipe_refuse_evidence_discern_returns_three_values_over_four_places() {
        let (none, rust, prose) = (Vec::new(), vec!["src/a.rs".to_owned()], vec!["docs/x.md".to_owned()]);
        let files = Evidence::Files(vec!["src/a.rs".to_owned()]);
        let (firm, provisional, unmeasured) = (Certainty::Firm, Certainty::Provisional, Certainty::Unmeasured);
        for (evidence, want) in [
            (Evidence::Row, [firm, firm, firm]),
            (files, [firm, provisional, firm]),
            (Evidence::Name, [firm, provisional, firm]),
            (Evidence::Place, [unmeasured, unmeasured, unmeasured]),
        ] {
            let got = [&none, &rust, &prose].map(|moving| discern(&evidence, moving));
            assert_eq!(got, want, "{} × 動く file（無い / 交わる .rs / 交わらない .md）", evidence.as_str());
        }
        let dir = Evidence::Files(vec!["src/".to_owned()]);
        assert_eq!(discern(&dir, &rust), provisional, "dir の項目は配下の動く file と交わる");
        let snap = vec!["tests/snapshots/x.snap".to_owned()];
        assert_eq!(discern(&Evidence::Name, &snap), provisional, "本文の読み手は .snap も読む");
        let words: Vec<&str> = [firm, provisional, unmeasured].iter().map(|found| found.as_str()).collect();
        assert_eq!(words, ["firm", "provisional", "unmeasured"], "結果の file の語");
    }
}
