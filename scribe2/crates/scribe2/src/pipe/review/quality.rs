//! 契約の審査が数える質の 5 観点（tsuzuri の判断の記録 ADR-63 の決定 (7)・乙'）。
//!
//! lens が最終行の JSON に書く `quality`（5 観点の件数）と `quality_at`（0 でない観点の場所の列）を読み、`review.json` に写す形に
//! する。**判定は動かさない**——読めない周は写さないだけで、質の数で FAIL にも INCONCLUSIVE にもしない。観点の字は雛形の部品
//! `headless/lens-quality.txt` に在り、gate の雛形は持たない（gate は質を数えない）。

use crate::pipe::gate::counts_of;

/// 質の 5 観点の名（雛形の部品 `headless/lens-quality.txt` の並び・`review.json` の `quality` の字の順）。
pub(crate) const NAMES: [&str; 5] = ["delete", "stdlib", "native", "yagni", "shrink"];

/// 読めた質の数と場所の列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Quality {
    /// [`NAMES`] の順の件数（0 も持つ）。
    counts: Vec<u64>,
    /// lens の `quality_at`（空白だけでない字の周だけ）。
    at: Option<String>,
}

impl Quality {
    /// lens の `quality` と `quality_at` の字から読む。`quality` が無い・5 観点を 1 度ずつ持たない・件数が数でない周は `None`。
    pub(super) fn read(quality: Option<&str>, at: Option<&str>) -> Option<Self> {
        let counts = counts_of(quality?, &NAMES).ok()?;
        Some(Self { counts, at: at.filter(|found| !found.trim().is_empty()).map(str::to_owned) })
    }

    /// `review.json` の `quality` の字（[`NAMES`] の順・0 件も 0 と書く）。
    pub(super) fn field(&self) -> String {
        NAMES.iter().zip(&self.counts).map(|(name, count)| format!("{name}:{count}")).collect::<Vec<_>>().join(",")
    }

    /// `review.json` の `quality_at`（lens が書いた周だけ）。
    pub(super) fn at(&self) -> Option<&str> {
        self.at.as_deref()
    }
}

/// `review.json` の `quality` の字から delete の件数を読む（memo の口の質の原本・読みは [`Quality::read`] の 1 本・読めない周は `None`）。
pub(crate) fn delete_count(quality: &str) -> Option<u64> {
    Quality::read(Some(quality), None).and_then(|found| found.counts.first().copied())
}

#[cfg(test)]
mod tests {
    use super::{delete_count, Quality};

    /// 5 観点を 1 度ずつ持つ `quality` は並びを問わず [`super::NAMES`] の順の字に読み直し、`quality_at` は空白だけでない字だけを写す。
    #[test]
    fn vrqual_quality_reads_the_five_counts_in_declared_order() {
        let read = Quality::read(Some("shrink:5,yagni:4,native:3,stdlib:2,delete:1"), Some("delete:src/a.rs:3"));
        let found = read.expect("5 観点が揃えば読める");
        assert_eq!(found.field(), "delete:1,stdlib:2,native:3,yagni:4,shrink:5", "宣言の順");
        assert_eq!(found.at(), Some("delete:src/a.rs:3"), "場所の列は字のまま");
        for at in [None, Some(""), Some("  ")] {
            let bare = Quality::read(Some("delete:0,stdlib:0,native:0,yagni:0,shrink:0"), at).expect("0 件も読める");
            assert_eq!((bare.field().as_str(), bare.at()), ("delete:0,stdlib:0,native:0,yagni:0,shrink:0", None), "{at:?}");
        }
    }

    /// 1 句だけ外した `quality`（無い・1 観点の欠け・重なり・表に無い名・数でない件数・gate の観点）は読めない。
    #[test]
    fn vrqual_quality_refuses_a_count_short_of_the_five() {
        let all = "delete:1,stdlib:0,native:0,yagni:0,shrink:0";
        let cases = [
            None,
            Some("delete:1,stdlib:0,native:0,yagni:0"),
            Some("delete:1,delete:1,stdlib:0,native:0,yagni:0,shrink:0"),
            Some("delete:1,stdlib:0,native:0,yagni:0,shrink:0,typo:0"),
            Some("delete:x,stdlib:0,native:0,yagni:0,shrink:0"),
            Some("delete:1,stdlib:0,native:0,yagni:0,shrink:0,contract-fit:0"),
        ];
        for quality in cases {
            assert_eq!(Quality::read(quality, Some("delete:src/a.rs")), None, "{quality:?}");
        }
        assert!(Quality::read(Some(all), None).is_some(), "対: 5 観点が揃えば読める");
        assert_eq!(delete_count(all), Some(1), "delete の件数");
        assert_eq!(delete_count(&all.replace("delete:1", "delete:0")), Some(0), "0 は 0");
        assert_eq!(delete_count(&all.replace("delete:1,", "")), None, "delete の欠けた字は読めない");
    }
}
