//! 未反映の一覧（memo・裁定・発話の 3 種・種類ごとに「読めた一覧」か「まだ分からない」・行 c-unref-lc）。
//! どれが席の手番かは器（scribe2）の局面の出力が決め、tsuzuri は判じない（要件 FR13・要望は種類にしない）。
//! 一覧は局面の出力の部品から読む（種類ごとの部品の種類と局面の語は `PHASES` の 1 か所の表）。出力が無いか
//! 読めないか、古さの印の file が在って読めないか、出力の unmeasured の欄が無いか形が違えば 3 種とも
//! 「まだ分からない」で古さの印は空にし、unmeasured が部品の種類を名指せばその種類だけを「まだ分からない」にし、
//! 古さの印が在れば印の種類を一覧に添える（中核の `case::cases_of` の読み）。
//! 題と作った時刻は台帳の字から引く（台帳に無い bead の題は空・発話は題を持たず、作った時刻は id の発話の ts）。

use serde::Serialize;
use serde_json::Value;
use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::case::CasePart;
use tsuzuri_contract::stats::UnreflectedKind;

use super::{epoch_secs, read};
use crate::case::cases_of;

/// 種類ごとの部品の種類と、未反映に数える局面の語（1 か所の表・`UnreflectedKind::ALL` の順）。
pub const PHASES: [(UnreflectedKind, &str, &[&str]); 3] = [
    (
        UnreflectedKind::Memo,
        "memo",
        &["memo-actionable", "misfit"],
    ),
    (UnreflectedKind::Ruling, "question", &["ruling-unreflected"]),
    (UnreflectedKind::Utterance, "utterance", &["utterance-open"]),
];

/// 未反映の 1 件（id・題・作った時刻・作った時刻が読めなければ None）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnreflectedItem {
    pub id: String,
    pub title: String,
    pub created: Option<EpochSecs>,
}

/// 未反映の一覧（種類ごと・古さの印の種類）。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Unreflected {
    pub memos: Reading<Vec<UnreflectedItem>>,
    pub rulings: Reading<Vec<UnreflectedItem>>,
    pub utterances: Reading<Vec<UnreflectedItem>>,
    /// 局面の出力の古さの印の種類（印が無ければ空・空でなければ一覧は古い）。
    pub stale: Vec<String>,
}

impl Unreflected {
    /// 種類の一覧。
    pub fn get(&self, kind: UnreflectedKind) -> &Reading<Vec<UnreflectedItem>> {
        match kind {
            UnreflectedKind::Memo => &self.memos,
            UnreflectedKind::Ruling => &self.rulings,
            UnreflectedKind::Utterance => &self.utterances,
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

/// 未反映の一覧。`json` は局面の出力の字（無いか読めなければ空の字）、`stale` は古さの印の字
/// （file が無ければ None・在って読めなければ空の字）、`ledger` は台帳の一覧の字（題と作った時刻にだけ使う）。
pub fn unreflected(ledger: &str, json: &str, stale: Option<&str>) -> Unreflected {
    let doc = cases_of(json, stale);
    let Reading::Known(parts) = doc.parts else {
        return not_yet();
    };
    let Some(unmeasured) = unmeasured_parts(json) else {
        return not_yet();
    };
    let beads = read(ledger).unwrap_or_default();
    let item = |p: &CasePart| {
        let bead = beads.iter().find(|b| b.id == p.id);
        UnreflectedItem {
            id: p.id.clone(),
            title: bead.map(|b| b.title.clone()).unwrap_or_default(),
            created: match p.part.as_str() {
                "utterance" => epoch_secs(&p.id),
                _ => bead.and_then(|b| b.created),
            },
        }
    };
    let [memos, rulings, utterances] = PHASES.map(|(_, part, phases)| {
        if unmeasured.iter().any(|n| n == part) {
            Reading::Unknown
        } else {
            Reading::Known(
                parts
                    .iter()
                    .filter(|p| p.part == part && phases.contains(&p.phase.as_str()))
                    .map(item)
                    .collect(),
            )
        }
    });
    Unreflected {
        memos,
        rulings,
        utterances,
        stale: doc.stale,
    }
}

/// 出力の unmeasured が名指す部品の種類（欄が無いか形が違えば None）。
fn unmeasured_parts(json: &str) -> Option<Vec<String>> {
    let value: Value = serde_json::from_str(json).ok()?;
    value
        .get("unmeasured")?
        .as_array()?
        .iter()
        .map(|u| u.get("part")?.as_str().map(str::to_string))
        .collect()
}

/// 3 種とも「まだ分からない」の一覧（古さの印は無し）。
pub(crate) fn not_yet() -> Unreflected {
    Unreflected {
        memos: Reading::Unknown,
        rulings: Reading::Unknown,
        utterances: Reading::Unknown,
        stale: Vec::new(),
    }
}
