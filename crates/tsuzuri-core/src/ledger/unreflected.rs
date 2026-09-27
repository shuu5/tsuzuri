//! 未反映の一覧（memo・裁定・要望の 3 種・種類ごとに「読めた一覧」か「まだ分からない」）。
//! memo は open の memo の全部（昇格先を指す辺がまだ無いので、open なら未反映）で、年齢の古い順。
//! 裁定（処分の宣言の無い裁定）と要望（反映されていない要望）は材料（処分の宣言・要望の印）が
//! 入力にまだ無いので「まだ分からない」を返す（0 件と区別する）。

use serde::Serialize;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::stats::UnreflectedKind;

use super::{Bead, read};

/// 未反映の 1 件（id・題・年齢の秒・作った時刻が読めなければ年齢は None）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnreflectedItem {
    pub id: String,
    pub title: String,
    pub age_s: Option<u64>,
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

/// 台帳の一覧の字と今の時刻から未反映の一覧を組む（台帳が読めなければ memo も「まだ分からない」）。
pub fn unreflected(ledger: &str, now: EpochSecs) -> Unreflected {
    of_beads(read(ledger).as_deref(), now)
}

/// 読めた bead から組む（None は台帳が読めない）。
pub(crate) fn of_beads(beads: Option<&[Bead]>, now: EpochSecs) -> Unreflected {
    let memos = beads.map(|beads| {
        let mut open: Vec<&Bead> = beads
            .iter()
            .filter(|b| b.kind == NodeKind::Memo && b.is_open(now))
            .collect();
        // 年齢の古い順（作った時刻の早い順・時刻の読めない memo は後ろ・同じなら台帳の順）。
        open.sort_by_key(|b| (b.created.is_none(), b.created));
        open.into_iter()
            .map(|b| UnreflectedItem {
                id: b.id.clone(),
                title: b.title.clone(),
                age_s: b.created.map(|c| now.saturating_sub(c)),
            })
            .collect()
    });
    Unreflected {
        memos: memos.map_or(Reading::Unknown, Reading::Known),
        rulings: Reading::Unknown,
        requests: Reading::Unknown,
    }
}
