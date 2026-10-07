//! 理由の字を組む関数（親 [`super`] の `Refuse::reason` の腕が呼ぶ）。

use super::Difference;
use crate::pipe::review::ROW_SAME_KIND_STOP;

/// 索引の組み立て中の断りの 1 行（設計 reverse-index.md §7 (b)・名と状態の語 `absent` か `building` を名指す）。
pub(super) fn index_building_reason(name: &str, state: &str) -> String {
    format!("{name} code の索引が作り中（{state}）＝touches に型を持つ行は索引が ready になるまで受け付けない")
}

/// 同型の停止の断りの 1 行（設計 contract-source.md §23・型の語と証の数と行の値と証の列を名指す）。
pub(super) fn repeated_reason(kind: &str, runs: &[String], stop: u64) -> String {
    let (count, joined) = (runs.len(), runs.join(", "));
    format!(
        "同じ種類の落ち {kind} が {count} 件で {ROW_SAME_KIND_STOP} の {stop} に達した（{joined}）＝契約か節を替えても数えは戻らない・調べ係に型と再発を調べさせ、構造の直しの行を起こすか持ち主に問う"
    )
}

/// 欄 `code-facts` を持つ行が索引を測れない断りの 1 行（設計 reverse-index.md §7 (c)・要素と状態の語を名指す）。
pub(super) fn unmeasured_reason(name: &str, element: &str, state: &str) -> String {
    format!("{name} 名乗りの事実 {element} を測れない（{state}）＝欄 code-facts を持つ行は索引が ready になるまで受け付けない")
}

/// 欄 `code-facts` の名乗りと実測の違いの断りの 1 行（設計 reverse-index.md §7 (c)・要素・名乗り・実測・site の先頭 3 つと残りの件数・母集団を名指す）。
pub(super) fn code_facts_reason(name: &str, found: &Difference) -> String {
    let Difference { element, claimed, measured, sites, rest, text } = found;
    let places = if sites.is_empty() { "なし".to_owned() } else { sites.join(", ") };
    format!("{name} {element} 名乗り {claimed} 実測 {measured}（実測の site: {places}・残り {rest} 件・{text}）")
}

/// 裁定 id の引用の断りの 1 行（設計 dispatcher.md §37 約束 4）。測れた周は置き場（設計の節 → 契約表の行の順・空の置き場は書かない）
/// ごとに id を全て名指し、件数は置き場ごとの字面の和。測れない周（`unmeasured`）は語を名指す。
pub(super) fn ruling_reason(name: &str, places: [&Vec<String>; 2], unmeasured: Option<&str>) -> String {
    if let Some(word) = unmeasured {
        return format!("{name} 裁定 id の引用を測れない（{word}）");
    }
    let count = places.iter().fold(0_usize, |sum, ids| sum.saturating_add(ids.len()));
    let named: Vec<String> = ["設計の節", "契約表の行"]
        .iter()
        .zip(places)
        .filter(|(_, ids)| !ids.is_empty())
        .map(|(place, ids)| format!("{place}: {}", ids.join(",")))
        .collect();
    format!("{name} 解けない裁定 id の引用 {count} 件（{}）", named.join("／"))
}

#[cfg(test)]
mod tests {
    // flip-check: moved t3-hub.92.10.6
    use super::super::tests::samples;
    use super::super::{discern, Certainty, ClosureError, Difference, Evidence, Refuse, INDEX_BUILD_TRIGGERS, REFUSALS};
    use crate::cli_outcome::{RC_BROKEN, RC_REFUSED};
    use crate::pipe::table::TableError;

    /// 受付の 2 門（設計 contract-source.md §23・`s2-07l.396`）: 宣言順の 18〜19 番目（約束の行の 2 理由の手前）・rc 1 で、
    /// 同型の停止は型の語と証の数と行の値と証の列（新しい順）を、焼き直しは型と対応の無かった項目を名乗る。
    #[test]
    fn refuse_repeat_reasons_are_last_and_name_kind_runs_and_value() {
        let found = samples();
        let pair: Vec<&str> = found.iter().skip(17).take(2).map(Refuse::as_str).collect();
        assert_eq!(pair, ["same-kind-repeated", "finding-unaddressed"], "宣言順の 18〜19 番目");
        let repeated = found.get(17).map(Refuse::reason).unwrap_or_default();
        for want in ["review-literal-mismatch", " 2 件", "review.same_kind_stop の 2", "b-2, b-1", "契約か節を替えても数えは戻らない"] {
            assert!(repeated.contains(want), "{want}: {repeated}");
        }
        assert!(!repeated.contains("焼き直しは書き直す"), "材料の比べの字は消えた: {repeated}");
        let unaddressed = found.get(18).map(Refuse::reason).unwrap_or_default();
        assert!(unaddressed.contains("teeth-outside-write-set") && unaddressed.contains("src/a.rs"), "{unaddressed}");
        assert!(found.iter().skip(17).take(2).all(|refuse| refuse.rc() == RC_REFUSED && !refuse.reason().contains('\n')), "rc 1・1 行");
    }

    /// write-set の導出の 6 理由（契約 (h)・設計 contract-source.md §3「write-set の導出」・§18 の fn 形・§20 の Declared
    /// 行の門）: 宣言順の末尾に並び、rc 1 で、理由は不足と余分 / filter 語 / 項目 / module の段と fn の名 /
    /// write-set に無い歯の file の全部を名乗る（字面は導出の側と同じ 1 本）。
    #[test]
    fn refuse_derive_reasons_are_last_and_name_their_payload() {
        let found = samples();
        // 契約 (b) の `hand-written-contract` は導出の理由ではないので、末尾の 6 つはその手前に並ぶ。
        let tail: Vec<&str> = found.iter().skip(10).take(6).map(Refuse::as_str).collect();
        assert_eq!(
            tail,
            [
                "write-set-drift",
                "teeth-place-unresolved",
                "also-names-rust",
                "tests-not-a-teeth-file",
                "fn-undeclared",
                "teeth-outside-write-set"
            ],
            "宣言順の 11〜16 番目"
        );
        let reasons: Vec<String> = found.iter().skip(10).take(6).map(Refuse::reason).collect();
        assert!(reasons.first().is_some_and(|line| line.contains("missing: src/a.rs") && line.contains("extra: docs/x.md")), "{reasons:?}");
        assert!(reasons.get(1).is_some_and(|line| line.contains("fresh_") && line.contains("tests")), "{reasons:?}");
        assert!(reasons.get(2).is_some_and(|line| line.contains("also の src/a.rs")), "{reasons:?}");
        assert!(reasons.get(3).is_some_and(|line| line.contains("tests の src/a.rs")), "{reasons:?}");
        assert!(
            reasons.get(4).is_some_and(|line| line.contains("pipe::cli::missing を宣言する file が base に無い")),
            "{reasons:?}"
        );
        assert!(
            reasons.get(5).is_some_and(|line| line.contains("歯の file が write-set に無い")
                && line.contains("src/a.rs ← filter 語 alpha_, tests/b.rs ← filter 語 beta_")),
            "{reasons:?}"
        );
        assert!(found.iter().skip(10).all(|refuse| refuse.rc() == RC_REFUSED && !refuse.reason().contains('\n')), "rc 1・1 行");
    }

    /// §41: 受付の側の `teeth-outside-write-set` の理由の 1 行が file と filter 語の両方を名乗り、導出の側の 1 本と同じ字面・
    /// 語と rc は不変。
    #[test]
    fn contract_teeth_origin_refuse_reason_names_file_and_filter() {
        let files = vec![("tests/b.rs".to_owned(), "beta_".to_owned())];
        let refuse = Refuse::TeethOutsideWriteSet { files: files.clone() };
        let reason = refuse.reason();
        assert!(reason.contains("tests/b.rs") && reason.contains("beta_"), "{reason}");
        assert_eq!(reason, ClosureError::TeethOutsideWriteSet { files }.reason(), "導出の側の 1 本を写す");
        assert_eq!((refuse.as_str(), refuse.rc()), ("teeth-outside-write-set", RC_REFUSED), "語と rc は不変");
    }

    /// 契約表の 3 理由（`s2-07l.208`・設計 contract-source.md §2 / §3）: 見出しは契約表の欠陥だけ理由の名を足し、
    /// 閉包の不足は**足りない file を全部**名乗る。読めない表だけ rc 2（NFR4）。
    #[test]
    fn refuse_contract_table_reasons_name_every_missing_file_and_keep_the_rc_of_the_table_error() {
        let found = samples();
        let labels: Vec<String> = found.iter().skip(4).take(3).map(Refuse::label).collect();
        assert_eq!(labels, ["write-set-incomplete", "write-set-dir-without-slash", "contract-table:section-missing"]);
        let incomplete = found.get(4).map(Refuse::reason).unwrap_or_default();
        assert!(incomplete.contains("src/a.rs, src/b.rs"), "足りない file を全部名乗る: {incomplete}");
        let unreadable = Refuse::ContractTable(TableError::Unreadable { line: 0, reason: "x を読めない".to_owned() });
        assert_eq!(unreadable.rc(), RC_BROKEN, "読めない表は rc 2");
        assert_eq!(unreadable.reason(), "x を読めない", "理由は表の欠陥の字面のまま");
    }

    /// 閉包の拡張の 3 理由（契約 (g)・設計 contract-source.md §3）: 導出の 4 理由の前に並び、rc 1 で、理由は項目 / file
    /// と余地と size / 名指しと在り処を名乗る。
    #[test]
    fn refuse_closure_ext_reasons_are_last_and_name_their_payload() {
        let found = samples();
        let tail: Vec<&str> = found.iter().skip(7).take(3).map(Refuse::as_str).collect();
        assert_eq!(tail, ["write-set-item-unresolved", "cap-headroom", "name-unresolved"], "(g) の 3 つ");
        let reasons: Vec<String> = found.iter().skip(7).take(3).map(Refuse::reason).collect();
        assert!(reasons.first().is_some_and(|line| line.contains("src/none.rs")), "項目を名乗る: {reasons:?}");
        let headroom = reasons.get(1).cloned().unwrap_or_default();
        assert!(headroom.contains("src/big.rs") && headroom.contains(" 7 ") && headroom.contains("size M"), "{headroom}");
        assert!(headroom.contains("見込み 30 行"), "file の見込みの値も名乗る（§46）: {headroom}");
        assert!(reasons.get(2).is_some_and(|line| line.contains("Guard::Rules") && line.contains("done")), "{reasons:?}");
        assert!(found.iter().skip(7).take(3).all(|refuse| refuse.rc() == RC_REFUSED), "前提違反は rc 1");
    }

    /// 宣言の同時の数の上限の断り（tsuzuri の判断の記録 ADR-63 の決定 (13)）は名と run / cap の 2 値だけの 1 行・rc 1・在り処は置き場。
    #[test]
    fn vrcap_refusal_names_the_run_and_the_cap() {
        let found = Refuse::RunCap { run: "r-1".to_owned(), cap: 1 };
        assert_eq!((found.as_str(), found.reason(), found.rc()), ("run-cap", "run-cap run=r-1 cap=1".to_owned(), RC_REFUSED));
        assert_eq!(found.evidence(), Evidence::Place);
    }

    // flip-check: retroactive s2-07l.736.33.21.5
    /// 裁定 id の引用の断り（設計 dispatcher.md §37 約束 4）: 測れた周は rc 1 で本文が置き場ごとに id を全て名指し（置き場の順は
    /// 設計の節 → 契約表の行・片方が空の置き場は書かない）、測れない周は rc 2 で語を名指す。在り処はどちらも Place（台帳の状態に依る）。
    /// 見本は名で引く（末尾の位置を取らない・行 d が見本の末尾に索引の組み立て中を足した）。
    #[test]
    fn refuse_ruling_names_every_id_per_place_and_keeps_two_rcs() {
        let both = samples().into_iter().find(|found| found.as_str() == "ruling-unresolved");
        let both = both.as_ref();
        assert_eq!(
            both.map(Refuse::reason),
            Some("ruling-unresolved 解けない裁定 id の引用 3 件（設計の節: batch:b7,s2-07l.9:20260930T0000Z-1／契約表の行: policy:x）".to_owned()),
            "置き場ごとに id を全て名指す"
        );
        assert_eq!(both.map(Refuse::rc), Some(RC_REFUSED), "測れた周は rc 1");
        let row_only = Refuse::RulingUnresolved { section: Vec::new(), row: vec!["batch:x".to_owned()], unmeasured: None };
        assert_eq!(row_only.reason(), "ruling-unresolved 解けない裁定 id の引用 1 件（契約表の行: batch:x）", "空の置き場は書かない");
        let section_only = Refuse::RulingUnresolved { section: vec!["policy:y".to_owned()], row: Vec::new(), unmeasured: None };
        assert_eq!(section_only.reason(), "ruling-unresolved 解けない裁定 id の引用 1 件（設計の節: policy:y）", "空の置き場は書かない");
        let unmeasured = Refuse::RulingUnresolved { section: Vec::new(), row: Vec::new(), unmeasured: Some("ledger".to_owned()) };
        assert_eq!(unmeasured.reason(), "ruling-unresolved 裁定 id の引用を測れない（ledger）", "測れない周は語を名指す");
        assert_eq!((unmeasured.rc(), unmeasured.as_str()), (RC_BROKEN, "ruling-unresolved"), "測れない周は同じ値の rc 2");
        for found in [both, Some(&row_only), Some(&section_only), Some(&unmeasured)] {
            assert_eq!(found.map(|refuse| refuse.evidence().render()), Some("place".to_owned()), "在り処は Place");
            assert_eq!(found.map(|refuse| refuse.reason().contains('\n')), Some(false), "理由は 1 行");
        }
    }

    /// 索引の組み立て中の断り（設計 reverse-index.md §7 (b)・行 d）: 語は `index-building`・在り処は置き場（予想の base では測れない）・rc 1 で、
    /// 断りの 1 行は語と状態の語（absent と building の 2 つ）を名指す。見本は variant を名で組む（宣言順の位置を取らない）。
    #[test]
    fn refuse_index_building_names_the_state_word_and_stays_in_the_place() {
        for state in ["absent", "building"] {
            let found = Refuse::IndexBuilding { state: state.to_owned() };
            assert_eq!(found.as_str(), "index-building", "語");
            assert_eq!(found.evidence(), Evidence::Place, "在り処は置き場");
            assert_eq!(found.rc(), RC_REFUSED, "rc 1");
            let line = found.reason();
            assert!(line.starts_with("index-building ") && line.contains(state) && !line.contains('\n'), "語と状態の語を名指す 1 行: {line}");
            assert_eq!(discern(&found.evidence(), &["src/a.rs".to_owned()]), Certainty::Unmeasured, "予想の base では測れない");
        }
        let (absent, building) = (Refuse::IndexBuilding { state: "absent".to_owned() }, Refuse::IndexBuilding { state: "building".to_owned() });
        assert_ne!(absent.reason(), building.reason(), "状態の語で 1 行が変わる");
        assert!(REFUSALS.contains(&absent.as_str()), "語は REFUSALS に在る");
    }

    /// 欄 `code-facts` の 2 断り（設計 reverse-index.md §7 (c)・行 e）: 語は REFUSALS の run-cap の前が code-facts・code-facts-unmeasured の順で、rc 1・1 行。
    /// 違いの断りは要素・名乗り・実測・site の先頭 3 つと残りの件数・母集団を名指し、在り処は本文の読み手。測れない周の断りは要素と状態の語を名指し、
    /// 在り処は置き場（undeclared だけ vessel 宣言の file）で、確からしさは absent が unmeasured・undeclared が firm。
    #[test]
    fn refuse_code_facts_names_the_difference_and_the_unmeasured_state() {
        let tail: Vec<&str> = REFUSALS.iter().rev().skip(6).take(2).copied().collect();
        assert_eq!(tail, ["code-facts-unmeasured", "code-facts"], "run-cap の前の 2 語は code-facts・code-facts-unmeasured の順");
        let differ = Refuse::CodeFacts(Box::new(Difference {
            element: "literals:crate::x::Y=2".to_owned(),
            claimed: "2".to_owned(),
            measured: "4".to_owned(),
            sites: vec!["src/a.rs:1".to_owned(), "src/b.rs:2".to_owned(), "src/c.rs:3".to_owned()],
            rest: 1,
            text: "text=7".to_owned(),
        }));
        let line = differ.reason();
        for want in ["code-facts ", "literals:crate::x::Y=2", "名乗り 2", "実測 4", "src/a.rs:1, src/b.rs:2, src/c.rs:3", "残り 1 件", "text=7"] {
            assert!(line.contains(want), "{want}: {line}");
        }
        assert_eq!((differ.rc(), differ.evidence()), (RC_REFUSED, Evidence::Name), "rc 1・在り処は本文の読み手");
        let unmeasured = |state: &str| Refuse::CodeFactsUnmeasured { element: "refs:crate::x::Y=1".to_owned(), state: state.to_owned() };
        for state in ["absent", "building", "failed:rc", "none"] {
            let line = unmeasured(state).reason();
            assert!(line.starts_with("code-facts-unmeasured ") && line.contains("refs:crate::x::Y=1") && line.contains(state), "{line}");
            assert_eq!((unmeasured(state).rc(), unmeasured(state).evidence()), (RC_REFUSED, Evidence::Place), "{state}: 置き場");
        }
        let undeclared = unmeasured("undeclared");
        assert_eq!(undeclared.evidence(), Evidence::Files(vec![crate::pipe::declaration::DECL_FILE.to_owned()]), "宣言の file");
        assert_eq!(discern(&undeclared.evidence(), &["src/a.rs".to_owned()]), Certainty::Firm, "undeclared は確定");
        assert_eq!(discern(&unmeasured("absent").evidence(), &["src/a.rs".to_owned()]), Certainty::Unmeasured, "absent は測れない");
        assert!([differ, undeclared].iter().all(|found| !found.reason().contains('\n')), "理由は 1 行");
        assert_eq!(INDEX_BUILD_TRIGGERS.last().copied(), Some("code-facts-unmeasured"), "契機の語の末尾");
    }
}
