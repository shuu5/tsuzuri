//! 未反映の一覧（memo・裁定・要望の 3 種・種類ごとに「読めた一覧」か「まだ分からない」）。
//! どれが席の手番かは器（scribe2）の局面の出力が決め、tsuzuri は判じない（要件 FR13）。
//! 局面の出力を読む行 c-unref-lc までは 3 種とも「まだ分からない」を返す（open の memo を全部
//! 未反映と数える代用は外した）。

use serde::Serialize;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::UnreflectedKind;

/// 未反映の 1 件（id・題・作った時刻・作った時刻が読めなければ None）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnreflectedItem {
    pub id: String,
    pub title: String,
    pub created: Option<EpochSecs>,
}

/// 未反映の一覧（種類ごと）。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Unreflected {
    pub memos: Reading<Vec<UnreflectedItem>>,
    pub rulings: Reading<Vec<UnreflectedItem>>,
    pub requests: Reading<Vec<UnreflectedItem>>,
}

impl Unreflected {
    /// 種類の一覧。
    pub fn get(&self, kind: UnreflectedKind) -> &Reading<Vec<UnreflectedItem>> {
        match kind {
            UnreflectedKind::Memo => &self.memos,
            UnreflectedKind::Ruling => &self.rulings,
            UnreflectedKind::Request => &self.requests,
        }
    }

    /// 「まだ分からない」種類（閉じた一覧の順）。
    pub fn unknown(&self) -> Vec<UnreflectedKind> {
        UnreflectedKind::ALL
            .into_iter()
            .filter(|&k| matches!(self.get(k), Reading::Unknown))
            .collect()
    }
}

/// 未反映の一覧（局面の出力がまだ無いので、台帳の字と時刻を読まず 3 種とも「まだ分からない」）。
pub fn unreflected(_ledger: &str, _now: EpochSecs) -> Unreflected {
    not_yet()
}

/// 3 種とも「まだ分からない」の一覧。
pub(crate) fn not_yet() -> Unreflected {
    Unreflected {
        memos: Reading::Unknown,
        rulings: Reading::Unknown,
        requests: Reading::Unknown,
    }
}
